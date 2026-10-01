//! What belongs on a person's profile beyond the numbers: how far they are through each show,
//! the episodes they marked as seen by hand, and their day streaks.

use std::collections::BTreeSet;

use anyhow::Result;
use axum::Json;
use axum::extract::{Path, State};
use chrono::NaiveDate;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::auth::AuthUser;
use crate::db::rusqlite::{Connection, params};
use crate::db::{self};
use crate::playback::PLAY_FRAC;
use crate::recap::longest_run;
use crate::state::{ApiError, ApiResult, App};

/// Watched this much of an episode and it counts as seen. Lenient on purpose: skipped credits
/// and imported plays (which only know time watched) should not leave holes in a season.
const SEEN_AT: f64 = 0.8;

/// The one reading of "seen", for an episode and a film alike: a play that went far enough, then Jellyfin's played
/// flag, then a mark by hand; otherwise begun if it was ever played. (state, which source said seen).
fn verdict(frac: Option<f64>, jellyfin: bool, manual: bool) -> (&'static str, Option<&'static str>) {
    match (frac.is_some_and(|f| f >= SEEN_AT), jellyfin, manual, frac) {
        (true, _, _, _) => ("seen", Some("played")),
        (_, true, _, _) => ("seen", Some("jellyfin")),
        (_, _, true, _) => ("seen", Some("manual")),
        (_, _, _, Some(_)) => ("started", None),
        _ => ("none", None),
    }
}

/// Longest and current run of consecutive local days with at least one play, over all time.
pub fn streaks(conn: &Connection, user_id: &str, min_play_s: i64) -> Result<Value> {
    let mut stmt = conn.prepare(
        "SELECT DISTINCT date(started_at, 'unixepoch', 'localtime') FROM playbacks WHERE user_id = ?1 AND duration_s >= ?2",
    )?;
    let days: BTreeSet<NaiveDate> = stmt
        .query_map(params![user_id, min_play_s], |r| r.get::<_, String>(0))?
        .filter_map(|d| d.ok().and_then(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok()))
        .collect();
    let today: String = conn.query_row("SELECT date('now', 'localtime')", [], |r| r.get(0))?;
    let today = NaiveDate::parse_from_str(&today, "%Y-%m-%d")?;

    let longest = longest_run(&days).map(|(len, from, to)| json!({ "days": len, "from": from.to_string(), "to": to.to_string() }));
    // A streak is still alive if it reached yesterday: today is not over yet.
    let mut end = if days.contains(&today) { Some(today) } else { Some(today - chrono::Duration::days(1)).filter(|d| days.contains(d)) };
    let mut current = 0i64;
    let mut since = None;
    while let Some(d) = end.filter(|d| days.contains(d)) {
        current += 1;
        since = Some(d);
        end = Some(d - chrono::Duration::days(1));
    }
    Ok(json!({
        "longest": longest,
        "current": { "days": current, "since": since.map(|d| d.to_string()), "includes_today": days.contains(&today) },
        "active_days": days.len(),
    }))
}

/// One episode of a show somebody has touched, as far as they have got with it: episodes that exist
/// as files (no missing or unaired ones, no specials), each with what three sources know about it.
/// Shared by the profile's show progress and the recap's finished shows, so the two agree on "seen".
pub(crate) struct Episode {
    pub series_id: String,
    pub series_name: String,
    pub series_year: Option<i64>,
    pub series_removed: bool,
    pub id: String,
    pub season: i64,
    pub number: Option<i64>,
    pub name: String,
    /// "seen", "started" or "none"
    pub state: &'static str,
    /// For a seen episode: "played", "jellyfin" or "manual", the first that applies.
    pub source: Option<&'static str>,
    pub last_at: Option<i64>,
    /// When it became seen, by whichever source says so: the first play that went far enough, the
    /// date Jellyfin keeps with its flag, or the manual mark. `None` when that source keeps no date.
    pub seen_at: Option<i64>,
    /// When it was first pressed play on, if ever.
    pub first_at: Option<i64>,
}

/// `until`: only what had happened by then — plays ended, flags and marks set — for a published recap,
/// which must not move while somebody is watching. `None` is everything, as the profile shows it.
pub(crate) fn episodes(conn: &Connection, user_id: &str, until: Option<i64>) -> Result<Vec<Episode>> {
    let mut stmt = conn.prepare(&format!(
        "WITH mine AS (
            SELECT p.item_id, MAX(p.ended_at) AS last_at, MIN(p.started_at) AS first_at,
                   MAX({PLAY_FRAC}) AS frac,
                   MIN(CASE WHEN {PLAY_FRAC} >= ?2 THEN p.ended_at END) AS seen_at
            FROM playbacks p LEFT JOIN items i ON i.id = p.item_id
            WHERE p.user_id = ?1 AND p.item_type = 'Episode' AND (?3 IS NULL OR (p.active = 0 AND p.ended_at <= ?3)) GROUP BY p.item_id),
         touched AS (
            SELECT series_id FROM playbacks p WHERE user_id = ?1 AND item_type = 'Episode' AND series_id IS NOT NULL AND (?3 IS NULL OR (p.active = 0 AND p.ended_at <= ?3))
            UNION SELECT e.series_id FROM user_items ui JOIN items e ON e.id = ui.item_id WHERE ui.user_id = ?1 AND ui.played = 1 AND e.type = 'Episode'
              AND (?3 IS NULL OR ui.last_played_at IS NULL OR ui.last_played_at <= ?3)
            UNION SELECT e.series_id FROM manual_seen ms JOIN items e ON e.id = ms.item_id WHERE ms.user_id = ?1 AND (?3 IS NULL OR ms.created_at <= ?3))
         SELECT s.id, s.name, s.production_year, s.removed,
                e.id, COALESCE(e.parent_index_number, 1), e.index_number, e.name,
                m.frac, m.last_at, COALESCE(ui.played AND (?3 IS NULL OR ui.last_played_at IS NULL OR ui.last_played_at <= ?3), 0),
                (ms.item_id IS NOT NULL AND (?3 IS NULL OR ms.created_at <= ?3)),
                m.seen_at, m.first_at, ui.last_played_at, ms.created_at
         FROM items e
         JOIN items s ON s.id = e.series_id AND s.type = 'Series'
         LEFT JOIN mine m ON m.item_id = e.id
         LEFT JOIN user_items ui ON ui.user_id = ?1 AND ui.item_id = e.id
         LEFT JOIN manual_seen ms ON ms.user_id = ?1 AND ms.item_id = e.id
         WHERE e.type = 'Episode' AND e.series_id IN (SELECT series_id FROM touched)
           AND COALESCE(e.parent_index_number, 1) > 0
           AND (e.path IS NOT NULL OR e.size_bytes IS NOT NULL)
           AND (e.removed = 0 OR s.removed = 1)
         ORDER BY s.id, COALESCE(e.parent_index_number, 1), COALESCE(e.index_number, 9999), e.name"
    ))?;
    let mut rows = stmt.query(params![user_id, SEEN_AT, until])?;
    let mut out = vec![];
    while let Some(r) = rows.next()? {
        let frac: Option<f64> = r.get(8)?;
        let (jellyfin, manual): (bool, bool) = (r.get::<_, i64>(10)? != 0, r.get(11)?);
        let (state, source) = verdict(frac, jellyfin, manual);
        out.push(Episode {
            series_id: r.get(0)?, series_name: r.get(1)?, series_year: r.get(2)?, series_removed: r.get::<_, i64>(3)? != 0,
            id: r.get(4)?, season: r.get(5)?, number: r.get(6)?, name: r.get(7)?, state, source, last_at: r.get(9)?,
            seen_at: match source {
                Some("played") => r.get(12)?,
                Some("jellyfin") => r.get(14)?,
                Some("manual") => r.get(15)?,
                _ => None,
            },
            first_at: r.get(13)?,
        });
    }
    Ok(out)
}

/// How far one person got with each of these films (or any title that stands alone), by the same three sources and
/// the same 80% as an episode: (item id, "seen" | "started" | "none", which source said seen).
pub(crate) fn films(conn: &Connection, user_id: &str, item_ids: &[String]) -> Result<Vec<(String, &'static str, Option<&'static str>)>> {
    let mut stmt = conn.prepare_cached(&format!(
        "WITH wanted(id) AS (SELECT value FROM json_each(?2)),
         mine AS (
            SELECT p.item_id, MAX({PLAY_FRAC}) AS frac
            FROM playbacks p LEFT JOIN items i ON i.id = p.item_id
            WHERE p.user_id = ?1 AND p.item_id IN (SELECT id FROM wanted) GROUP BY p.item_id)
         SELECT w.id, m.frac, COALESCE(ui.played, 0), ms.item_id IS NOT NULL
         FROM wanted w LEFT JOIN mine m ON m.item_id = w.id
         LEFT JOIN user_items ui ON ui.user_id = ?1 AND ui.item_id = w.id
         LEFT JOIN manual_seen ms ON ms.user_id = ?1 AND ms.item_id = w.id"
    ))?;
    let rows = stmt.query_map(params![user_id, serde_json::to_string(item_ids)?], |r| {
        let (state, source) = verdict(r.get(1)?, r.get::<_, i64>(2)? != 0, r.get(3)?);
        Ok((r.get::<_, String>(0)?, state, source))
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}

fn user_shows(conn: &Connection, user_id: &str) -> Result<Vec<Value>> {
    let mut shows: Vec<Value> = vec![];
    for e in episodes(conn, user_id, None)? {
        if shows.last().is_none_or(|s| s["id"] != e.series_id.as_str()) {
            shows.push(json!({
                "id": e.series_id, "name": e.series_name, "year": e.series_year, "removed": e.series_removed,
                "image_item_id": e.series_id, "total": 0, "seen": 0, "started": 0, "last_played_at": null, "seasons": [],
            }));
        }
        let show = shows.last_mut().expect("just pushed");
        if show["seasons"].as_array().and_then(|a| a.last()).is_none_or(|s| s["season_number"] != e.season) {
            show["seasons"].as_array_mut().unwrap().push(json!({ "season_number": e.season, "total": 0, "seen": 0, "episodes": [] }));
        }
        let bump = |v: &mut Value, key: &str| v[key] = json!(v[key].as_i64().unwrap_or(0) + 1);
        bump(show, "total");
        if e.state == "seen" { bump(show, "seen") } else if e.state == "started" { bump(show, "started") }
        if e.last_at > show["last_played_at"].as_i64() {
            show["last_played_at"] = json!(e.last_at);
        }
        let s = show["seasons"].as_array_mut().unwrap().last_mut().unwrap();
        bump(s, "total");
        if e.state == "seen" { bump(s, "seen") }
        s["episodes"].as_array_mut().unwrap().push(json!({
            "id": e.id, "episode_number": e.number, "name": e.name, "state": e.state, "source": e.source,
        }));
    }
    // Shows where nothing is seen or started would only be noise (a Jellyfin flag on a since-removed episode, say).
    shows.retain(|s| s["seen"].as_i64().unwrap_or(0) + s["started"].as_i64().unwrap_or(0) > 0);
    Ok(shows)
}

pub async fn shows(State(app): State<App>, user: AuthUser, Path(id): Path<String>) -> ApiResult {
    let id = db::norm_id(&id);
    if !user.perms.see_everyone && user.id != id {
        return Err(ApiError::not_permitted("see other people's statistics"));
    }
    let editable = user.id == id;
    let min_play_s = app.settings().min_play_s;
    let (shows, streaks) = app.db.call(move |c| Ok((user_shows(c, &id)?, streaks(c, &id, min_play_s)?))).await?;
    Ok(Json(json!({ "shows": shows, "streaks": streaks, "editable": editable })))
}

#[derive(Deserialize)]
pub struct SeenBody {
    #[serde(default)]
    item_ids: Vec<String>,
    seen: bool,
}

/// Mark episodes as seen (or take the mark back) for yourself. Stays inside finstats:
/// nothing is written to Jellyfin, and plays that were really recorded are never touched.
pub async fn mark_seen(State(app): State<App>, user: AuthUser, Json(body): Json<SeenBody>) -> ApiResult {
    if body.item_ids.is_empty() || body.item_ids.len() > 5000 {
        return Err(ApiError::bad_request("Send between 1 and 5000 item ids"));
    }
    let ids: Vec<String> = body.item_ids.iter().map(|i| db::norm_id(i)).filter(|i| i.len() == 32 && i.chars().all(|c| c.is_ascii_hexdigit())).collect();
    if ids.len() != body.item_ids.len() {
        return Err(ApiError::bad_request("That doesn't look like a list of item ids"));
    }
    let (uid, seen) = (user.id.clone(), body.seen);
    let changed = app
        .db
        .call(move |c| {
            let tx = c.transaction()?;
            let mut n = 0;
            {
                let mut add = tx.prepare(
                    "INSERT OR IGNORE INTO manual_seen(user_id, item_id, created_at)
                     SELECT ?1, id, ?3 FROM items WHERE id = ?2 AND type = 'Episode'",
                )?;
                let mut del = tx.prepare("DELETE FROM manual_seen WHERE user_id = ?1 AND item_id = ?2")?;
                for id in &ids {
                    n += if seen { add.execute(params![uid, id, db::now()])? } else { del.execute(params![uid, id])? };
                }
            }
            tx.commit()?;
            Ok(n)
        })
        .await?;
    Ok(Json(json!({ "changed": changed })))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn db() -> Connection {
        let c = Connection::open_in_memory().unwrap();
        c.execute_batch(
            "CREATE TABLE items(id TEXT PRIMARY KEY, type TEXT, name TEXT, series_id TEXT, parent_index_number INTEGER, index_number INTEGER,
                                production_year INTEGER, removed INTEGER DEFAULT 0, path TEXT, size_bytes INTEGER, runtime_s INTEGER);
             CREATE TABLE playbacks(id INTEGER PRIMARY KEY, user_id TEXT, item_id TEXT, item_type TEXT, series_id TEXT, started_at INTEGER, ended_at INTEGER,
                                    duration_s INTEGER, position_s INTEGER, runtime_s INTEGER, active INTEGER NOT NULL DEFAULT 0);
             CREATE TABLE user_items(user_id TEXT, item_id TEXT, played INTEGER, last_played_at INTEGER, PRIMARY KEY(user_id, item_id));
             CREATE TABLE manual_seen(user_id TEXT, item_id TEXT, created_at INTEGER, PRIMARY KEY(user_id, item_id));
             INSERT INTO items(id, type, name) VALUES ('s', 'Series', 'Test Show');
             INSERT INTO items(id, type, name, series_id, parent_index_number, index_number, path, runtime_s) VALUES
                ('e1', 'Episode', 'One', 's', 1, 1, '/f1', 1000), ('e2', 'Episode', 'Two', 's', 1, 2, '/f2', 1000),
                ('e3', 'Episode', 'Three', 's', 1, 3, '/f3', 1000), ('e4', 'Episode', 'Four', 's', 1, 4, '/f4', 1000),
                ('sp', 'Episode', 'Special', 's', 0, 1, '/sp', 1000);
             -- season 2 has been announced but is not on the server: a virtual item without a file
             INSERT INTO items(id, type, name, series_id, parent_index_number, index_number) VALUES ('e5', 'Episode', 'Unaired', 's', 2, 1);
             INSERT INTO playbacks(user_id, item_id, item_type, series_id, started_at, ended_at, duration_s, position_s, runtime_s) VALUES
                ('u', 'e1', 'Episode', 's', 0, 950, 950, 950, 1000),      -- watched
                ('u', 'e2', 'Episode', 's', 0, 100, 100, 100, 1000);      -- only started
             INSERT INTO user_items VALUES ('u', 'e3', 1, NULL);                 -- Jellyfin says played
             INSERT INTO manual_seen VALUES ('u', 'e4', 0);                -- marked by hand",
        )
        .unwrap();
        c
    }

    #[test]
    fn progress_counts_real_files_only_and_merges_all_three_sources() {
        let shows = user_shows(&db(), "u").unwrap();
        assert_eq!(shows.len(), 1);
        let s = &shows[0];
        assert_eq!((s["total"].as_i64(), s["seen"].as_i64(), s["started"].as_i64()), (Some(4), Some(3), Some(1)), "no special, no unaired season");
        assert_eq!(s["seasons"].as_array().unwrap().len(), 1);
        let eps: Vec<(String, Value)> = s["seasons"][0]["episodes"].as_array().unwrap().iter().map(|e| (e["state"].as_str().unwrap().into(), e["source"].clone())).collect();
        assert_eq!(eps, vec![("seen".into(), json!("played")), ("started".into(), Value::Null), ("seen".into(), json!("jellyfin")), ("seen".into(), json!("manual"))]);
        assert!(user_shows(&db(), "someone-else").unwrap().is_empty());
    }

    #[test]
    fn streaks_find_the_longest_run() {
        let c = db();
        c.execute_batch("DELETE FROM playbacks;").unwrap();
        for day in ["2026-01-01", "2026-01-02", "2026-01-03", "2026-01-10", "2026-01-11"] {
            c.execute("INSERT INTO playbacks(user_id, item_id, item_type, started_at, duration_s) VALUES ('u', 'e1', 'Episode', CAST(strftime('%s', ?1 || ' 12:00:00') AS INTEGER), 600)", [day]).unwrap();
        }
        let s = streaks(&c, "u", 0).unwrap();
        assert_eq!((s["longest"]["days"].as_i64(), s["longest"]["from"].as_str(), s["longest"]["to"].as_str()), (Some(3), Some("2026-01-01"), Some("2026-01-03")));
        assert_eq!((s["current"]["days"].as_i64(), s["active_days"].as_i64()), (Some(0), Some(5)));
    }
    #[test]
    fn a_film_is_seen_by_the_same_rule_as_an_episode() {
        let c = db();
        c.execute_batch(
            "INSERT INTO items(id, type, name, runtime_s) VALUES ('m1', 'Movie', 'Watched', 6000), ('m2', 'Movie', 'Begun', 6000),
                ('m3', 'Movie', 'Flagged', 6000), ('m4', 'Movie', 'Marked', 6000), ('m5', 'Movie', 'Untouched', 6000), ('m6', 'Movie', 'Imported', 6000);
             INSERT INTO playbacks(user_id, item_id, item_type, started_at, ended_at, duration_s, position_s, runtime_s) VALUES
                ('u', 'm1', 'Movie', 0, 5000, 5000, 5000, 6000),
                ('u', 'm2', 'Movie', 0, 900, 900, 900, 6000),
                ('someone-else', 'm5', 'Movie', 0, 6000, 6000, 6000, 6000);
             -- an imported play: no position, only how long it ran, and the runtime from the library
             INSERT INTO playbacks(user_id, item_id, item_type, started_at, ended_at, duration_s) VALUES ('u', 'm6', 'Movie', 0, 5400, 5400);
             INSERT INTO user_items VALUES ('u', 'm3', 1, NULL);
             INSERT INTO manual_seen VALUES ('u', 'm4', 0);",
        )
        .unwrap();
        let ids: Vec<String> = ["m1", "m2", "m3", "m4", "m5", "m6"].map(String::from).to_vec();
        let mut got = films(&c, "u", &ids).unwrap();
        got.sort();
        assert_eq!(got, vec![
            ("m1".into(), "seen", Some("played")), ("m2".into(), "started", None), ("m3".into(), "seen", Some("jellyfin")),
            ("m4".into(), "seen", Some("manual")), ("m5".into(), "none", None), ("m6".into(), "seen", Some("played")),
        ]);
    }
}
