//! Whether somebody has been through the welcome: once a person, whichever browser they first sign in on. What the welcome
//! asks (the language, the formats, the colours, the layout) stays each browser's own; only that it was asked is theirs.

use axum::Json;
use axum::extract::State;
use serde_json::json;

use crate::audit::{self, Actor};
use crate::auth::AuthUser;
use crate::db::{self, rusqlite::{Connection, OptionalExtension, params}};
use crate::state::{ApiResult, App};

/// Whether `user_id` has been through the welcome, on any browser.
pub fn welcomed(conn: &Connection, user_id: &str) -> anyhow::Result<bool> {
    Ok(conn.query_row("SELECT 1 FROM welcomed WHERE user_id = ?1", params![user_id], |_| Ok(())).optional()?.is_some())
}

/// Notes that `user_id` has been through it; answers whether that is news. The first time is kept.
pub fn mark(conn: &Connection, user_id: &str, now: i64) -> anyhow::Result<bool> {
    Ok(conn.execute("INSERT OR IGNORE INTO welcomed(user_id, at) VALUES (?1, ?2)", params![user_id, now])? > 0)
}

/// `PUT /api/me/welcomed`: the caller has been through the welcome, and nobody else.
pub async fn put_mine(State(app): State<App>, user: AuthUser) -> ApiResult {
    let actor = Actor::from(&user);
    app.db
        .call(move |c| {
            if mark(c, &user.id, db::now())? {
                audit::record_quietly(c, &audit::Entry::new("welcomed", actor).target(user.id.clone()));
            }
            Ok(())
        })
        .await?;
    Ok(Json(json!({ "welcomed": true })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Db;

    #[test]
    fn a_welcome_is_one_person_s_and_counts_once() {
        let db = Db::open_in_memory().unwrap();
        let c = db.conn().unwrap();
        assert!(!welcomed(&c, "alice").unwrap(), "nobody has been welcomed yet");
        assert!(mark(&c, "alice", 100).unwrap(), "the first time is news");
        assert!(welcomed(&c, "alice").unwrap());
        assert!(!welcomed(&c, "bob").unwrap(), "alice's welcome is not bob's");
        assert!(!mark(&c, "alice", 200).unwrap(), "a second browser marking it again changes nothing");
        let at: i64 = c.query_row("SELECT at FROM welcomed WHERE user_id = 'alice'", [], |r| r.get(0)).unwrap();
        assert_eq!(at, 100, "when it first happened is kept");
    }
}
