//! How finstats looks to one person (Settings → Appearance): a FinUI preset's code, theirs alone. `/assets/finui.css`
//! answers in the look of whoever asks for it; somebody signed out, and somebody who chose nothing, get FinUI as it
//! ships. A preset is made at FinUI create and applied as FinUI applies it (`finui::preset`).

use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use serde::Deserialize;
use serde_json::json;

use crate::audit::{self, Actor};
use crate::auth::AuthUser;
use crate::db::{self, rusqlite::{Connection, OptionalExtension, params}};
use crate::state::{ApiError, ApiResult, App};

/// The preset a person chose, or "" for none.
pub fn of(conn: &Connection, user_id: &str) -> anyhow::Result<String> {
    Ok(conn.query_row("SELECT finui_preset FROM appearance WHERE user_id = ?1", params![user_id], |r| r.get(0)).optional()?.unwrap_or_default())
}

/// Choose a preset for a person, or none with "". Err for a code that names no option, and nothing is stored.
pub fn set(conn: &Connection, user_id: &str, preset: &str, now: i64) -> anyhow::Result<Result<(), String>> {
    if crate::finui::preset::overlay(preset).is_none() {
        return Ok(Err(format!("`{preset}` names no FinUI preset: a code is one letter or digit per choice, such as 0101")));
    }
    if preset.is_empty() {
        conn.execute("DELETE FROM appearance WHERE user_id = ?1", params![user_id])?;
    } else {
        conn.execute(
            "INSERT INTO appearance(user_id, finui_preset, updated_at) VALUES (?1, ?2, ?3) ON CONFLICT(user_id) DO UPDATE SET finui_preset = excluded.finui_preset, updated_at = excluded.updated_at",
            params![user_id, preset, now],
        )?;
    }
    Ok(Ok(()))
}

/// The preset of whoever sent these headers: their session cookie's person, or "" for somebody signed out. Only the
/// cookie: a stylesheet is fetched by a <link>, which carries nothing else.
pub async fn of_request(app: &App, headers: &HeaderMap) -> String {
    let Some(token) = crate::auth::cookie_token(headers) else { return String::new() };
    let settings = app.settings();
    app.db
        .call(move |c| match crate::auth::resolve_session_in(c, &settings, &token, db::now())? {
            Some(user) => of(c, &user.id),
            None => Ok(String::new()),
        })
        .await
        .unwrap_or_default()
}

#[derive(Deserialize)]
pub struct Look {
    finui_preset: String,
}

/// `GET /api/me/appearance`: the caller's own look.
pub async fn get_mine(State(app): State<App>, user: AuthUser) -> ApiResult {
    let preset = app.db.call(move |c| of(c, &user.id)).await?;
    Ok(Json(json!({ "finui_preset": preset })))
}

/// `PUT /api/me/appearance` with `{"finui_preset"}`: the caller's own look, and nobody else's; "" for none.
pub async fn put_mine(State(app): State<App>, user: AuthUser, Json(look): Json<Look>) -> ApiResult {
    let actor = Actor::from(&user);
    let preset = look.finui_preset.trim().to_string();
    app.db
        .call(move |c| {
            if let Err(why) = set(c, &user.id, &preset, db::now())? {
                return Ok(Err(why));
            }
            audit::record_quietly(c, &audit::Entry::new("appearance_changed", actor).target(user.id.clone()).detail(json!({ "finui_preset": preset })));
            Ok(Ok(json!({ "finui_preset": preset })))
        })
        .await?
        .map(Json)
        .map_err(ApiError::bad_request)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Db;

    #[test]
    fn a_look_is_one_person_s_and_nobody_else_s() {
        let db = Db::open_in_memory().unwrap();
        let c = db.conn().unwrap();
        assert_eq!(of(&c, "alice").unwrap(), "", "nothing chosen: FinUI as it ships");
        set(&c, "alice", "0101", 1).unwrap().unwrap();
        assert_eq!(of(&c, "alice").unwrap(), "0101");
        assert_eq!(of(&c, "bob").unwrap(), "", "alice's choice is not bob's");
        set(&c, "bob", "02", 2).unwrap().unwrap();
        assert_eq!((of(&c, "alice").unwrap(), of(&c, "bob").unwrap()), ("0101".into(), "02".into()));
    }

    #[test]
    fn none_takes_the_choice_back() {
        let db = Db::open_in_memory().unwrap();
        let c = db.conn().unwrap();
        set(&c, "alice", "0101", 1).unwrap().unwrap();
        set(&c, "alice", "", 2).unwrap().unwrap();
        assert_eq!(of(&c, "alice").unwrap(), "");
        let rows: i64 = c.query_row("SELECT COUNT(*) FROM appearance", [], |r| r.get(0)).unwrap();
        assert_eq!(rows, 0, "none is no row, not an empty one");
    }

    #[test]
    fn a_code_that_names_no_preset_is_refused_and_changes_nothing() {
        let db = Db::open_in_memory().unwrap();
        let c = db.conn().unwrap();
        set(&c, "alice", "0101", 1).unwrap().unwrap();
        for bad in ["zz", "0Z", "curl", "01 "] {
            let err = set(&c, "alice", bad, 2).unwrap().expect_err(bad);
            assert!(err.contains("preset"), "{bad}: {err}");
        }
        assert_eq!(of(&c, "alice").unwrap(), "0101");
    }
}
