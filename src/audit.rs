//! finstats' own audit log: who signed in, who changed a setting, who granted a permission, who made
//! a key and what it did. Jellyfin's activity is mirrored meticulously in `server_events`; until now
//! finstats' own actions left nothing but a line in the process log.
//!
//! An entry is written where the thing happens, by the handler that did it, and **never fails the
//! caller**: a setting that was changed is changed whether or not the note about it could be
//! written. Nothing secret is ever in `detail` — a service is named by kind and name, never its
//! address or key; the settings entry is the diff of the settings blob, which holds no secret.
//! Reading it is a Jellyfin administrator's business. A year is kept.

use std::net::IpAddr;

use anyhow::Result;
use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::auth::{AuthUser, JellyfinAdmin};
use crate::db::{self, SqlValue};
use crate::db::rusqlite::{Connection, params, params_from_iter};
use crate::state::{ApiResult, App};
use crate::stats::{like_words, order_by, rows_json};

/// How long an entry is kept.
pub const KEEP_S: i64 = 365 * 86_400;

/// Every kind an entry can be. A kind the list is asked for that is not one of these is an empty
/// list, not an error.
pub const KINDS: [&str; 30] = [
    "sign_in", "sign_in_failed", "sign_in_refused", "sign_out", "setup_completed",
    "key_created", "key_revoked", "key_used",
    "setting_changed", "permissions_changed",
    "service_added", "service_changed", "service_removed",
    "target_added", "target_changed", "target_removed",
    "backup_made", "backup_restored", "backup_deleted", "backup_downloaded",
    "task_run", "import_started", "import_finished", "play_deleted", "alert_resolved", "alert_reopened",
    "profile_published", "profile_changed", "profile_unpublished", "profile_link_reset",
];

/// Who did it, and through what.
#[derive(Clone, Debug, Default)]
pub struct Actor {
    pub user_id: Option<String>,
    pub user_name: Option<String>,
    pub ip: Option<IpAddr>,
    pub key_id: Option<i64>,
}

impl From<&AuthUser> for Actor {
    fn from(u: &AuthUser) -> Self {
        Actor { user_id: Some(u.id.clone()), user_name: Some(u.name.clone()), ip: u.ip, key_id: u.key_id() }
    }
}

/// One thing that happened.
#[derive(Clone, Debug)]
pub struct Entry {
    pub kind: &'static str,
    pub at: i64,
    pub actor: Actor,
    pub target: Option<String>,
    pub detail: Value,
    pub outcome: &'static str,
}

impl Entry {
    pub fn new(kind: &'static str, actor: Actor) -> Self {
        Entry { kind, at: db::now(), actor, target: None, detail: json!({}), outcome: "ok" }
    }
    pub fn at(mut self, at: i64) -> Self {
        self.at = at;
        self
    }
    pub fn target(mut self, t: impl Into<String>) -> Self {
        self.target = Some(t.into());
        self
    }
    pub fn detail(mut self, d: Value) -> Self {
        self.detail = d;
        self
    }
    pub fn outcome(mut self, o: &'static str) -> Self {
        self.outcome = o;
        self
    }
}

/// Write one entry down. For use inside a `db.call` that is doing the thing itself.
pub fn record_in(conn: &Connection, e: &Entry) -> Result<()> {
    conn.execute(
        "INSERT INTO audit(at, kind, user_id, user_name, ip, key_id, target, detail, outcome) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![e.at, e.kind, e.actor.user_id, e.actor.user_name, e.actor.ip.map(|ip| ip.to_string()), e.actor.key_id, e.target, e.detail.to_string(), e.outcome],
    )?;
    Ok(())
}

/// The same, and a failure is a line in the log rather than the caller's problem.
pub fn record_quietly(conn: &Connection, e: &Entry) {
    if let Err(err) = record_in(conn, e) {
        tracing::debug!("audit entry {} not written: {err:#}", e.kind);
    }
}

/// Write one entry down from a handler: fire and forget, never the caller's error.
pub fn record(app: &App, e: Entry) {
    let app = app.clone();
    tokio::spawn(async move {
        let _ = app.db.call(move |c| { record_quietly(c, &e); Ok(()) }).await;
    });
}

/// The same, awaited: for a row that must be on record before what it announces begins — an import
/// takes one long transaction, and a write landing in the middle of its first reads would make its
/// own upgrade to writing fail (`SQLITE_BUSY_SNAPSHOT`). Still never the caller's error.
pub async fn record_now(app: &App, e: Entry) {
    let _ = app.db.call(move |c| { record_quietly(c, &e); Ok(()) }).await;
}

/// The settings that differ between two blobs: `[{key, from, to}]`, by key.
pub fn changed_keys(before: &Value, after: &Value) -> Vec<Value> {
    let (Some(b), Some(a)) = (before.as_object(), after.as_object()) else { return vec![] };
    let mut keys: Vec<&String> = b.keys().chain(a.keys()).collect();
    keys.sort();
    keys.dedup();
    keys.into_iter()
        .filter(|k| b.get(*k) != a.get(*k))
        .map(|k| json!({ "key": k, "from": b.get(k).cloned().unwrap_or(Value::Null), "to": a.get(k).cloned().unwrap_or(Value::Null) }))
        .collect()
}

/// Drop what is older than a year.
pub fn thin(conn: &Connection, now: i64) -> Result<usize> {
    Ok(conn.execute("DELETE FROM audit WHERE at < ?1", [now - KEEP_S])?)
}

// ---------------------------------------------------------------- reading it

#[derive(Deserialize, Default)]
pub struct AuditQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub q: Option<String>,
    pub kind: Option<String>,
    pub user_id: Option<String>,
    pub sort: Option<String>,
    pub dir: Option<String>,
}

const SORTS: [(&str, &str); 4] = [("when", "a.at"), ("kind", "a.kind"), ("user", "a.user_name COLLATE NOCASE"), ("outcome", "a.outcome")];

/// One page of the log, newest first unless asked otherwise, with the kinds it holds for the filter.
pub fn list(conn: &Connection, q: &AuditQuery) -> Result<Value> {
    let page = crate::stats::page_number(q.page);
    let per_page = q.per_page.unwrap_or(50).clamp(1, 200);
    let mut clauses: Vec<String> = vec![];
    let mut args: Vec<SqlValue> = vec![];
    if let Some(k) = q.kind.as_deref().filter(|k| !k.is_empty()) {
        // Not one of ours: nothing matches, and nothing is read.
        clauses.push("a.kind = ?".into());
        args.push(if KINDS.contains(&k) { k.to_string() } else { String::new() }.into());
    }
    if let Some(u) = q.user_id.as_deref().filter(|u| !u.is_empty()) {
        clauses.push("a.user_id = ?".into());
        args.push(db::norm_id(u).into());
    }
    for like in like_words(q.q.as_deref()) {
        clauses.push("(a.user_name LIKE ? ESCAPE '\\' OR a.target LIKE ? ESCAPE '\\' OR a.detail LIKE ? ESCAPE '\\' OR a.kind LIKE ? ESCAPE '\\')".into());
        args.extend([like.clone().into(), like.clone().into(), like.clone().into(), like.into()]);
    }
    let wh = if clauses.is_empty() { String::new() } else { format!("WHERE {}", clauses.join(" AND ")) };
    let total: i64 = conn.query_row(&format!("SELECT COUNT(*) FROM audit a {wh}"), params_from_iter(args.iter()), |r| r.get(0))?;
    let mut rows = rows_json(
        conn,
        &format!(
            "SELECT a.id, a.at, a.kind, a.user_id, a.user_name, (u.image_tag IS NOT NULL) AS has_image, a.ip, a.key_id, k.name AS key_name,
                    a.target, a.detail, a.outcome
             FROM audit a LEFT JOIN users u ON u.id = a.user_id LEFT JOIN api_keys k ON k.id = a.key_id {wh}
             ORDER BY {} LIMIT {per_page} OFFSET {}",
            order_by(&SORTS, q.sort.as_deref(), q.dir.as_deref(), "a.at DESC, a.id DESC"),
            (page - 1) * per_page
        ),
        &args,
    )?;
    for r in &mut rows {
        let detail = r.get("detail").and_then(Value::as_str).and_then(|d| serde_json::from_str::<Value>(d).ok()).unwrap_or(json!({}));
        r.insert("detail".into(), detail);
    }
    let mut stmt = conn.prepare("SELECT DISTINCT kind FROM audit ORDER BY kind")?;
    let kinds: Vec<String> = stmt.query_map([], |r| r.get(0))?.collect::<Result<_, _>>()?;
    Ok(json!({ "total": total, "page": page, "per_page": per_page, "rows": rows, "kinds": kinds }))
}

/// `GET /api/audit` — Jellyfin administrators only: it names who changed what.
pub async fn audit(State(app): State<App>, JellyfinAdmin(_): JellyfinAdmin, Query(q): Query<AuditQuery>) -> ApiResult {
    let out = app.db.call(move |c| list(c, &q)).await?;
    Ok(Json(out))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::rusqlite::Connection;
    use serde_json::json;

    fn conn() -> Connection {
        let c = Connection::open_in_memory().unwrap();
        for m in crate::db::MIGRATIONS {
            c.execute_batch(m).unwrap();
        }
        c
    }

    fn by(name: &str) -> Actor {
        Actor { user_id: Some(format!("u-{name}")), user_name: Some(name.to_string()), ip: Some("192.168.1.10".parse().unwrap()), key_id: None }
    }

    /// `rows_json` already reads `has_image` as a bool; read again as a number it was false for everybody,
    /// and the log showed initials where people have pictures.
    #[test]
    fn the_log_says_who_has_a_picture_and_any_page_number_is_answered() {
        let c = conn();
        c.execute("INSERT INTO users(id, name, is_admin, image_tag, updated_at) VALUES ('u-alice', 'alice', 1, 'tag1', 1)", []).unwrap();
        record_in(&c, &Entry::new("sign_in", by("alice"))).unwrap();
        let page = list(&c, &AuditQuery::default()).unwrap();
        assert_eq!(page["rows"][0]["has_image"], true, "{page}");
        let far = list(&c, &AuditQuery { page: Some(i64::MAX), ..AuditQuery::default() }).unwrap();
        assert_eq!(far["rows"].as_array().map(Vec::len), Some(0), "far past the end is an empty page, not the first one");
    }

    #[test]
    fn recording_never_fails_the_caller() {
        // A setting that was changed is changed whether or not the note about it could be written.
        let c = conn();
        c.execute_batch("DROP TABLE audit").unwrap();
        let e = Entry::new("setting_changed", by("alice")).detail(json!({ "changed": [] }));
        assert!(record_in(&c, &e).is_err(), "the write itself does fail");
        // The wrapper the handlers call swallows it; nothing to assert but that it returns.
        record_quietly(&c, &e);
    }

    #[test]
    fn the_settings_entry_lists_only_what_changed_and_never_a_secret() {
        let before = json!({ "min_play_s": 0, "public_url": "https://finstats.example", "home_addresses": ["203.0.113.7"] });
        let after = json!({ "min_play_s": 30, "public_url": "https://finstats.example", "home_addresses": ["203.0.113.7", "198.51.100.4"] });
        let changed = changed_keys(&before, &after);
        assert_eq!(changed.len(), 2);
        assert_eq!(changed[0], json!({ "key": "home_addresses", "from": ["203.0.113.7"], "to": ["203.0.113.7", "198.51.100.4"] }));
        assert_eq!(changed[1], json!({ "key": "min_play_s", "from": 0, "to": 30 }));
        assert!(changed_keys(&before, &before).is_empty());
        // The Jellyfin key lives in its own settings row, never in the Settings blob: nothing to redact,
        // and the entry is the diff of the blob and nothing else.
        let c = conn();
        let e = Entry::new("setting_changed", by("alice")).detail(json!({ "changed": changed }));
        record_in(&c, &e).unwrap();
        let stored: String = c.query_row("SELECT detail FROM audit", [], |r| r.get(0)).unwrap();
        assert!(stored.contains("min_play_s") && !stored.contains("api_key"));
    }

    #[test]
    fn the_list_filters_by_kind_and_sorts_only_on_known_columns() {
        let c = conn();
        record_in(&c, &Entry::new("sign_in", by("alice")).at(100)).unwrap();
        record_in(&c, &Entry::new("setting_changed", by("bob")).at(200).detail(json!({ "changed": [{ "key": "min_play_s" }] }))).unwrap();
        record_in(&c, &Entry::new("sign_in_failed", Actor { user_id: None, user_name: Some("mallory".into()), ip: Some("203.0.113.9".parse().unwrap()), key_id: None }).at(300).outcome("failed")).unwrap();
        let all = list(&c, &AuditQuery::default()).unwrap();
        assert_eq!(all["total"], 3);
        assert_eq!(all["rows"][0]["kind"], "sign_in_failed", "newest first by default");
        assert_eq!(all["rows"][0]["user_name"], "mallory");
        assert_eq!(all["rows"][0]["outcome"], "failed");
        let kinds: Vec<&str> = all["kinds"].as_array().unwrap().iter().map(|k| k.as_str().unwrap()).collect();
        assert_eq!(kinds, ["setting_changed", "sign_in", "sign_in_failed"]);
        let one = list(&c, &AuditQuery { kind: Some("setting_changed".into()), ..Default::default() }).unwrap();
        assert_eq!((one["total"].as_i64(), one["rows"][0]["detail"]["changed"][0]["key"].as_str()), (Some(1), Some("min_play_s")), "detail comes back as JSON");
        assert_eq!(list(&c, &AuditQuery { kind: Some("nonsense".into()), ..Default::default() }).unwrap()["total"], 0, "an unknown kind is an empty list, not an error");
        let found = list(&c, &AuditQuery { q: Some("min_play".into()), ..Default::default() }).unwrap();
        assert_eq!(found["total"], 1);
        let asc = list(&c, &AuditQuery { sort: Some("when".into()), dir: Some("asc".into()), ..Default::default() }).unwrap();
        assert_eq!(asc["rows"][0]["kind"], "sign_in");
        let bogus = list(&c, &AuditQuery { sort: Some("detail".into()), ..Default::default() }).unwrap();
        assert_eq!(bogus["rows"][0]["kind"], "sign_in_failed", "an unknown column falls back to the default order");
    }

    #[test]
    fn thinning_keeps_a_year() {
        let c = conn();
        let now = 2_000_000_000;
        record_in(&c, &Entry::new("sign_in", by("alice")).at(now - 366 * 86_400)).unwrap();
        record_in(&c, &Entry::new("sign_in", by("alice")).at(now - 364 * 86_400)).unwrap();
        assert_eq!(thin(&c, now).unwrap(), 1);
        let left: i64 = c.query_row("SELECT COUNT(*) FROM audit", [], |r| r.get(0)).unwrap();
        assert_eq!(left, 1);
    }
}
