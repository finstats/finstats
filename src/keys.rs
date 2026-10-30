//! API keys: a second way to prove who a request is, for scripts and calendars. Anyone signed in
//! may make one for themselves; it carries exactly their permissions, read fresh on every request,
//! and dies with their access. Administrators see and may revoke everyone's. A key is shown once,
//! when it is made, and stored only as a hash; managing keys takes a signed-in browser, so a leaked
//! key cannot mint its own successors.

use anyhow::Result;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::audit::{self, Actor};
use crate::auth::{AuthUser, Credential, KeyScope, mint_key};
use crate::db::rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use crate::db::{self};
use crate::state::{ApiError, ApiResult, App};
use crate::stats::rows_json;

/// What outlives a key is done with a session, never with a key: a grant, a setting that opens FinStats, a restore (which
/// can bring grants back), a service's connection. A leaked key revoked must take everything it could do with it.
pub fn needs_a_session(user: &AuthUser) -> Result<(), ApiError> {
    match user.credential {
        Credential::Session => Ok(()),
        Credential::Key { .. } => Err(ApiError::new(StatusCode::FORBIDDEN, crate::i18n::t("server.keys.session_needed", &[]))),
    }
}

/// Live keys one person may hold at once.
pub const MAX_KEYS_PER_USER: i64 = 20;

/// Keys are made and revoked with a session, never with a key.
pub fn only_a_session(user: &AuthUser) -> Result<(), ApiError> {
    match user.credential {
        Credential::Session => Ok(()),
        Credential::Key { .. } => Err(ApiError::new(StatusCode::FORBIDDEN, crate::i18n::t("server.keys.session_only", &[]))),
    }
}

/// Keys that still open something: not revoked, not run out (`auth::resolve_key_in`'s rule).
pub fn live_keys(c: &Connection, user_id: &str, now: i64) -> Result<i64> {
    Ok(c.query_row(
        "SELECT COUNT(*) FROM api_keys WHERE user_id = ?1 AND revoked_at IS NULL AND (expires_at IS NULL OR expires_at > ?2)",
        params![user_id, now],
        |r| r.get(0),
    )?)
}

pub fn may_mint(live: i64) -> bool {
    live < MAX_KEYS_PER_USER
}

/// The name trimmed and bounded, the expiry in days within reason.
pub fn check_request(name: &str, expires_in_d: Option<i64>) -> Result<(String, Option<i64>), ApiError> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 60 {
        return Err(ApiError::bad_request(crate::i18n::t("server.keys.name", &[])));
    }
    if expires_in_d.is_some_and(|d| !(1..=3650).contains(&d)) {
        return Err(ApiError::bad_request(crate::i18n::t("server.keys.expiry", &[])));
    }
    Ok((name.to_string(), expires_in_d))
}

const LIST_SQL: &str = "SELECT k.id, k.name, k.scope, k.user_id, COALESCE(u.name, '?') AS user_name, (u.image_tag IS NOT NULL) AS has_image,
                               k.created_at, k.expires_at, k.last_used_at, k.last_used_ip
                        FROM api_keys k LEFT JOIN users u ON u.id = k.user_id WHERE k.revoked_at IS NULL";

fn list_in(c: &Connection, user: &AuthUser) -> Result<Vec<Value>> {
    let mut rows = if user.is_admin {
        rows_json(c, &format!("{LIST_SQL} ORDER BY k.created_at DESC"), &[])?
    } else {
        rows_json(c, &format!("{LIST_SQL} AND k.user_id = ?1 ORDER BY k.created_at DESC"), &[user.id.clone().into()])?
    };
    for r in &mut rows {
        let mine = r.get("user_id").and_then(Value::as_str) == Some(user.id.as_str());
        r.insert("mine".into(), json!(mine));
    }
    Ok(rows.into_iter().map(Value::Object).collect())
}

/// `GET /api/keys`: one's own keys; every key, with its owner, for a Jellyfin administrator. Never the token.
pub async fn list(State(app): State<App>, user: AuthUser) -> ApiResult {
    let keys = app.db.call(move |c| list_in(c, &user)).await?;
    Ok(Json(json!({ "keys": keys })))
}

#[derive(Deserialize)]
pub struct CreateBody {
    name: String,
    scope: Option<String>,
    expires_in_d: Option<i64>,
}

fn create_key_in(c: &mut Connection, uid: &str, name: &str, scope: KeyScope, expires_in_d: Option<i64>) -> Result<Option<Value>> {
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let now = db::now();
    if !may_mint(live_keys(&tx, uid, now)?) {
        return Ok(None);
    }
    let expires_at = expires_in_d.map(|d| now + d * 86_400);
    let (id, token) = mint_key(&tx, uid, name, scope, expires_at, now)?;
    tx.commit()?;
    Ok(Some(json!({ "id": id, "key": token, "name": name, "scope": scope.as_str(), "created_at": now, "expires_at": expires_at })))
}

/// `POST /api/keys`: a new key for the caller, shown this once.
pub async fn create(State(app): State<App>, user: AuthUser, Json(body): Json<CreateBody>) -> ApiResult<axum::response::Response> {
    only_a_session(&user)?;
    let (name, expires_in_d) = check_request(&body.name, body.expires_in_d)?;
    let scope = KeyScope::parse(body.scope.as_deref().unwrap_or("full")).ok_or_else(|| ApiError::bad_request(crate::i18n::t("server.keys.scope", &[])))?;
    let actor = Actor::from(&user);
    let (uid, name_for_audit) = (user.id.clone(), name.clone());
    let made = app
        .db
        .call(move |c| {
            let made = create_key_in(c, &uid, &name, scope, expires_in_d)?;
            if let Some(ref made) = made {
                audit::record_quietly(c, &audit::Entry::new("key_created", actor).target(made["id"].to_string()).detail(json!({ "name": name_for_audit, "scope": scope.as_str(), "expires_at": made["expires_at"] })));
            }
            Ok(made)
        })
        .await?
        .ok_or_else(|| ApiError::bad_request(crate::i18n::t("server.keys.too_many", &[("max", MAX_KEYS_PER_USER.into())])))?;
    Ok((StatusCode::CREATED, Json(made)).into_response())
}

/// Revoke `id` if it is `uid`'s, or anyone's for an administrator. The row goes: a revoked key opens nothing either way,
/// the audit log keeps the record, and kept rows piled up for good when keys were made and revoked in a loop.
fn revoke_in(c: &Connection, id: i64, uid: &str, admin: bool, actor: Actor) -> anyhow::Result<bool> {
    let owned: Option<(String, String)> = c
        .query_row("SELECT user_id, name FROM api_keys WHERE id = ?1 AND revoked_at IS NULL", [id], |r| Ok((r.get(0)?, r.get(1)?)))
        .optional()?;
    let Some((owner, name)) = owned.filter(|(owner, _)| admin || owner == uid) else { return Ok(false) };
    c.execute("DELETE FROM api_keys WHERE id = ?1", [id])?;
    audit::record_quietly(c, &audit::Entry::new("key_revoked", actor).target(id.to_string()).detail(json!({ "name": name, "owner_id": owner })));
    Ok(true)
}

/// `DELETE /api/keys/{id}`: revoke one's own key, or anyone's as an administrator. A key that is not
/// the caller's to see is not there.
pub async fn revoke(State(app): State<App>, user: AuthUser, Path(id): Path<i64>) -> ApiResult {
    only_a_session(&user)?;
    let actor = Actor::from(&user);
    let (uid, admin) = (user.id.clone(), user.is_admin);
    let done = app.db.call(move |c| revoke_in(c, id, &uid, admin, actor)).await?;
    if !done {
        return Err(ApiError::not_found("Key"));
    }
    // A live connection opened with this key closes now rather than at its next look; nobody else's is asked.
    app.live_feed.recheck_key(id);
    Ok(Json(json!({ "ok": true })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::{Credential, KeyScope, Perms, mint_key};
    use crate::db::rusqlite::Connection;

    fn conn() -> Connection {
        let c = Connection::open_in_memory().unwrap();
        for m in crate::db::MIGRATIONS {
            c.execute_batch(m).unwrap();
        }
        c
    }
    fn user(credential: Credential) -> AuthUser {
        AuthUser { id: "u2".into(), name: "bob".into(), is_admin: false, perms: Perms::default(), credential, ip: None }
    }

    #[test]
    fn concurrent_creates_obey_the_cap() {
        {
            let dir = std::env::temp_dir().join(format!("finstats-cap-keys-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
            std::fs::create_dir_all(&dir).unwrap();
            let db = db::Db::open(&dir.join("finstats.db")).unwrap();
            db.conn().unwrap().execute("INSERT INTO users(id, name, updated_at) VALUES ('ub', 'bob', 1)", []).unwrap();
            for _ in 0..MAX_KEYS_PER_USER - 1 { assert!(create_key_in(&mut db.conn().unwrap(), "ub", "seed", KeyScope::Full, None).unwrap().is_some()); }
            let connections: Vec<_> = (0..6).map(|_| db.conn().unwrap()).collect();
            drop(connections);
            let barrier = std::sync::Arc::new(std::sync::Barrier::new(64));
            let threads: Vec<_> = (0..64).map(|_| {
                let db = db.clone();
                let barrier = barrier.clone();
                std::thread::spawn(move || {
                    barrier.wait();
                    create_key_in(&mut db.conn().unwrap(), "ub", "test", KeyScope::Full, None).unwrap().is_some()
                })
            }).collect();
            let made = threads.into_iter().map(|t| usize::from(t.join().unwrap())).sum::<usize>();
            assert_eq!(made, 1);
            let count: i64 = db.conn().unwrap().query_row("SELECT COUNT(*) FROM api_keys", [], |r| r.get(0)).unwrap();
            assert_eq!(count, MAX_KEYS_PER_USER);
            drop(db);
            std::fs::remove_dir_all(dir).unwrap();
        }
    }

    #[test]
    fn capped_key_insert_refuses_at_the_limit() {
        let mut c = conn();
        for _ in 0..MAX_KEYS_PER_USER {
            assert!(create_key_in(&mut c, "u2", "test", KeyScope::Full, None).unwrap().is_some());
        }
        assert!(create_key_in(&mut c, "u2", "test", KeyScope::Full, None).unwrap().is_none());
        assert_eq!(live_keys(&c, "u2", db::now()).unwrap(), MAX_KEYS_PER_USER);
    }

    #[test]
    fn a_key_cannot_mint_or_revoke_keys() {
        // A leaked key must not be able to mint its own successors: managing keys takes a signed-in browser.
        assert!(only_a_session(&user(Credential::Session)).is_ok());
        assert_eq!(only_a_session(&user(Credential::Key { id: 1, scope: KeyScope::Full })).unwrap_err().0, StatusCode::FORBIDDEN);
    }

    #[test]
    fn at_most_twenty_live_keys_per_person() {
        let c = conn();
        for i in 0..MAX_KEYS_PER_USER {
            mint_key(&c, "u2", &format!("k{i}"), KeyScope::Full, None, 1000).unwrap();
        }
        assert_eq!(live_keys(&c, "u2", 2000).unwrap(), MAX_KEYS_PER_USER);
        assert!(!may_mint(live_keys(&c, "u2", 2000).unwrap()));
        // A revoked one no longer counts, and neither does one that has run out: it opens nothing.
        c.execute_batch("UPDATE api_keys SET revoked_at = 1500 WHERE id = 1").unwrap();
        assert!(may_mint(live_keys(&c, "u2", 2000).unwrap()));
        mint_key(&c, "u2", "k-again", KeyScope::Full, None, 1000).unwrap();
        assert!(!may_mint(live_keys(&c, "u2", 2000).unwrap()));
        c.execute_batch("UPDATE api_keys SET expires_at = 1800 WHERE id = 2").unwrap();
        assert!(may_mint(live_keys(&c, "u2", 2000).unwrap()), "an expired key still counted");
    }

    /// A revoked key was kept for good, and anybody signed in could make and revoke keys in a loop: a row each time, never
    /// taken away. A revoked key opens nothing whether its row is there or not, and the audit log keeps its record.
    #[test]
    fn a_revoked_key_is_gone() {
        let c = conn();
        let (id, _) = mint_key(&c, "u2", "laptop", KeyScope::Full, None, 1000).unwrap();
        assert!(!revoke_in(&c, id, "u3", false, Actor::default()).unwrap(), "somebody else's key");
        assert!(revoke_in(&c, id, "u2", false, Actor::default()).unwrap());
        assert_eq!(c.query_row("SELECT COUNT(*) FROM api_keys", [], |r| r.get::<_, i64>(0)).unwrap(), 0);
        assert_eq!(c.query_row("SELECT COUNT(*) FROM audit WHERE kind = 'key_revoked'", [], |r| r.get::<_, i64>(0)).unwrap(), 1);
        assert!(!revoke_in(&c, id, "u2", false, Actor::default()).unwrap(), "revoked already");
    }

    #[test]
    fn a_key_is_named_and_given_an_expiry_within_reason() {
        assert_eq!(check_request("  laptop ", Some(30)).unwrap(), ("laptop".to_string(), Some(30)));
        assert!(check_request("", None).is_err(), "a name is required");
        assert!(check_request(&"x".repeat(61), None).is_err(), "sixty characters is plenty");
        assert!(check_request("phone", Some(0)).is_err());
        assert!(check_request("phone", Some(3651)).is_err());
        assert!(check_request("phone", Some(3650)).is_ok());
    }
}
