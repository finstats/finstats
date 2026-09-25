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
use crate::db::rusqlite::{Connection, OptionalExtension, params};
use crate::db::{self};
use crate::state::{ApiError, ApiResult, App};
use crate::stats::rows_json;

/// Live keys one person may hold at once.
pub const MAX_KEYS_PER_USER: i64 = 20;

/// Keys are made and revoked with a session, never with a key.
pub fn only_a_session(user: &AuthUser) -> Result<(), ApiError> {
    match user.credential {
        Credential::Session => Ok(()),
        Credential::Key { .. } => Err(ApiError::new(StatusCode::FORBIDDEN, "Sign in to manage keys; a key cannot make or revoke keys")),
    }
}

pub fn live_keys(c: &Connection, user_id: &str) -> Result<i64> {
    Ok(c.query_row("SELECT COUNT(*) FROM api_keys WHERE user_id = ?1 AND revoked_at IS NULL", [user_id], |r| r.get(0))?)
}

pub fn may_mint(live: i64) -> bool {
    live < MAX_KEYS_PER_USER
}

/// The name trimmed and bounded, the expiry in days within reason.
pub fn check_request(name: &str, expires_in_d: Option<i64>) -> Result<(String, Option<i64>), ApiError> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 60 {
        return Err(ApiError::bad_request("Give the key a name of up to 60 characters"));
    }
    if expires_in_d.is_some_and(|d| !(1..=3650).contains(&d)) {
        return Err(ApiError::bad_request("An expiry is 1 to 3650 days"));
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
        r.insert("has_image".into(), json!(r.get("has_image").and_then(Value::as_i64).unwrap_or(0) != 0));
    }
    Ok(rows.into_iter().map(Value::Object).collect())
}

/// `GET /api/keys` — one's own keys; every key, with its owner, for a Jellyfin administrator. Never the token.
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

/// `POST /api/keys` — a new key for the caller, shown this once.
pub async fn create(State(app): State<App>, user: AuthUser, Json(body): Json<CreateBody>) -> ApiResult<axum::response::Response> {
    only_a_session(&user)?;
    let (name, expires_in_d) = check_request(&body.name, body.expires_in_d)?;
    let scope = KeyScope::parse(body.scope.as_deref().unwrap_or("full")).ok_or_else(|| ApiError::bad_request("The scope is `full` or `calendar`"))?;
    let actor = Actor::from(&user);
    let (uid, name_for_audit) = (user.id.clone(), name.clone());
    let made = app
        .db
        .call(move |c| {
            if !may_mint(live_keys(c, &uid)?) {
                return Ok(None);
            }
            let now = db::now();
            let expires_at = expires_in_d.map(|d| now + d * 86_400);
            let (id, token) = mint_key(c, &uid, &name, scope, expires_at, now)?;
            audit::record_quietly(c, &audit::Entry::new("key_created", actor).target(id.to_string()).detail(json!({ "name": name_for_audit, "scope": scope.as_str(), "expires_at": expires_at })));
            Ok(Some(json!({ "id": id, "key": token, "name": name, "scope": scope.as_str(), "created_at": now, "expires_at": expires_at })))
        })
        .await?
        .ok_or_else(|| ApiError::bad_request(format!("You already hold {MAX_KEYS_PER_USER} keys; revoke one first")))?;
    Ok((StatusCode::CREATED, Json(made)).into_response())
}

/// `DELETE /api/keys/{id}` — revoke one's own key, or anyone's as an administrator. A key that is not
/// the caller's to see is not there.
pub async fn revoke(State(app): State<App>, user: AuthUser, Path(id): Path<i64>) -> ApiResult {
    only_a_session(&user)?;
    let actor = Actor::from(&user);
    let (uid, admin) = (user.id.clone(), user.is_admin);
    let done = app
        .db
        .call(move |c| {
            let owned: Option<(String, String)> = c
                .query_row("SELECT user_id, name FROM api_keys WHERE id = ?1 AND revoked_at IS NULL", [id], |r| Ok((r.get(0)?, r.get(1)?)))
                .optional()?;
            let Some((owner, name)) = owned.filter(|(owner, _)| admin || *owner == uid) else { return Ok(false) };
            c.execute("UPDATE api_keys SET revoked_at = ?2 WHERE id = ?1", params![id, db::now()])?;
            audit::record_quietly(c, &audit::Entry::new("key_revoked", actor).target(id.to_string()).detail(json!({ "name": name, "owner_id": owner })));
            Ok(true)
        })
        .await?;
    if !done {
        return Err(ApiError::not_found("Key"));
    }
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
        assert_eq!(live_keys(&c, "u2").unwrap(), MAX_KEYS_PER_USER);
        assert!(!may_mint(live_keys(&c, "u2").unwrap()));
        // A revoked one no longer counts.
        c.execute_batch("UPDATE api_keys SET revoked_at = 1500 WHERE id = 1").unwrap();
        assert!(may_mint(live_keys(&c, "u2").unwrap()));
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
