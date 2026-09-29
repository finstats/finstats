//! A person's list of films and shows they mean to watch. It belongs to its owner and nobody else: every
//! endpoint acts on the caller and takes no `user_id`, and no permission — `see_everyone`, a Jellyfin
//! administrator — opens somebody else's. Written to finstats' own database only; nothing here talks to
//! Jellyfin, Seerr, Sonarr or Radarr.
//!
//! **An entry names a title in one of two ways**: by its Jellyfin item when it is in the library, or by a
//! provider id (TMDB, TVDB, IMDb) when it is not yet. Either way it keeps a snapshot — kind, title, year and
//! the provider ids — so it still reads as something after its title leaves the library, or before it comes.

use anyhow::Result;

use std::collections::BTreeMap;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::auth::AuthUser;
use crate::db::rusqlite::{Connection, OptionalExtension, params};
use crate::db;
use crate::state::{ApiError, ApiResult, App};

/// The most one person may keep. An abuse bound, like the limits on keys and sessions.
pub const MAX_ENTRIES: i64 = 1000;

/// What can be put on a watchlist, spelled as Jellyfin types the item.
pub const KINDS: [&str; 2] = ["Movie", "Series"];

/// A title as the entry remembers it.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Title {
    pub kind: String,
    pub tmdb_id: Option<String>,
    pub tvdb_id: Option<String>,
    pub imdb_id: Option<String>,
    pub title: String,
    pub year: Option<i64>,
}

/// What somebody asked to add.
pub enum Wanted {
    Item(String),
    Title(Title),
}

#[derive(Debug, PartialEq)]
pub enum Refused {
    /// No film or show by that id in the library.
    NotATitle,
    /// Already `MAX_ENTRIES`.
    Full,
    /// Something in the request that cannot be a title.
    Invalid(&'static str),
}

/// Put a title on `user_id`'s list, or find it there already: (entry id, whether it was added now). Run it in a
/// transaction: it reads before it writes.
pub fn add(conn: &Connection, user_id: &str, wanted: Wanted, now: i64) -> Result<Result<(i64, bool), Refused>> {
    let (item_id, t) = match wanted {
        Wanted::Item(id) => {
            let id = crate::db::norm_id(&id);
            match snapshot(conn, &id)? {
                Some(t) => (Some(id), t),
                None => return Ok(Err(Refused::NotATitle)),
            }
        }
        Wanted::Title(t) => match valid(t) {
            Ok(t) => (find_title(conn, &t)?, t),
            Err(why) => return Ok(Err(why)),
        },
    };
    if let Some(id) = already(conn, user_id, item_id.as_deref(), &t)? {
        return Ok(Ok((id, false)));
    }
    let held: i64 = conn.query_row("SELECT COUNT(*) FROM watchlist WHERE user_id = ?1", [user_id], |r| r.get(0))?;
    if held >= MAX_ENTRIES {
        return Ok(Err(Refused::Full));
    }
    conn.execute(
        "INSERT INTO watchlist(user_id, kind, item_id, tmdb_id, tvdb_id, imdb_id, title, year, added_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![user_id, t.kind, item_id, t.tmdb_id, t.tvdb_id, t.imdb_id, t.title, t.year, now],
    )?;
    Ok(Ok((conn.last_insert_rowid(), true)))
}

/// A film or show in the library as an entry would remember it.
fn snapshot(conn: &Connection, item_id: &str) -> Result<Option<Title>> {
    let id = |key: &str| format!("CASE WHEN json_valid(provider_ids) THEN NULLIF(CAST(json_extract(provider_ids, '$.{key}') AS TEXT), '') END");
    Ok(conn
        .query_row(
            &format!("SELECT type, name, production_year, {}, {}, {} FROM items WHERE id = ?1 AND type IN ('Movie', 'Series')", id("Tmdb"), id("Tvdb"), id("Imdb")),
            [item_id],
            |r| Ok(Title { kind: r.get(0)?, title: r.get(1)?, year: r.get(2)?, tmdb_id: r.get(3)?, tvdb_id: r.get(4)?, imdb_id: r.get(5)? }),
        )
        .optional()?)
}

/// A title somebody sent, as it may be kept: a kind that can be listed, at least one id in the shape its service
/// gives it, and a name.
fn valid(mut t: Title) -> Result<Title, Refused> {
    let digits = |v: &Option<String>| v.as_deref().is_none_or(|v| (1..=12).contains(&v.len()) && v.bytes().all(|b| b.is_ascii_digit()));
    if !KINDS.contains(&t.kind.as_str()) {
        return Err(Refused::Invalid("Only films and shows can go on a watchlist"));
    }
    if t.tmdb_id.is_none() && t.tvdb_id.is_none() && t.imdb_id.is_none() {
        return Err(Refused::Invalid("A title that is not in the library needs a TMDB, TVDB or IMDb id"));
    }
    let imdb_ok = t.imdb_id.as_deref().is_none_or(|v| v.strip_prefix("tt").is_some_and(|n| (5..=12).contains(&n.len()) && n.bytes().all(|b| b.is_ascii_digit())));
    if !digits(&t.tmdb_id) || !digits(&t.tvdb_id) || !imdb_ok {
        return Err(Refused::Invalid("That doesn't look like a TMDB, TVDB or IMDb id"));
    }
    t.title = t.title.trim().to_string();
    if t.title.is_empty() || t.title.chars().count() > 300 {
        return Err(Refused::Invalid("A title needs a name of at most 300 characters"));
    }
    if t.year.is_some_and(|y| !(1870..=2200).contains(&y)) {
        return Err(Refused::Invalid("That year is not one a film or show could have"));
    }
    Ok(t)
}

/// The entry of `user_id`'s that already stands for this title: the same item, or the same kind and any one id.
fn already(conn: &Connection, user_id: &str, item_id: Option<&str>, t: &Title) -> Result<Option<i64>> {
    Ok(conn
        .query_row(
            "SELECT id FROM watchlist WHERE user_id = ?1
               AND (item_id = ?2 OR (kind = ?3 AND (tmdb_id = ?4 OR tvdb_id = ?5 OR imdb_id = ?6)))
             ORDER BY added_at, id LIMIT 1",
            params![user_id, item_id, t.kind, t.tmdb_id, t.tvdb_id, t.imdb_id],
            |r| r.get(0),
        )
        .optional()?)
}

/// Take an entry off `user_id`'s list. `false` when there is no such entry of theirs.
pub fn remove(conn: &Connection, user_id: &str, id: i64) -> Result<bool> {
    Ok(conn.execute("DELETE FROM watchlist WHERE id = ?1 AND user_id = ?2", params![id, user_id])? > 0)
}

/// One entry as the pages that offer the toggle need it: every item id it stands for in the library (the HD and
/// the 4K copy of a film are one title) and its provider ids, so a title not in the library is known too.
#[derive(Debug, PartialEq)]
pub struct Key {
    pub id: i64,
    pub kind: String,
    pub item_ids: Vec<String>,
    pub tmdb_id: Option<String>,
    pub tvdb_id: Option<String>,
    pub imdb_id: Option<String>,
}

/// One person's entries, newest first, each with every item in the library that shares one of its ids.
const KEYS_SQL: &str = "WITH mine AS (SELECT id, kind, item_id, tmdb_id, tvdb_id, imdb_id, added_at FROM watchlist WHERE user_id = ?1),
     copies AS (
        SELECT m.id, m.item_id AS item FROM mine m WHERE m.item_id IS NOT NULL
        UNION SELECT m.id, x.item_id FROM mine m JOIN item_external x ON x.source = 'Tmdb' AND x.value = m.tmdb_id JOIN items i ON i.id = x.item_id AND i.type = m.kind
        UNION SELECT m.id, x.item_id FROM mine m JOIN item_external x ON x.source = 'Tvdb' AND x.value = m.tvdb_id JOIN items i ON i.id = x.item_id AND i.type = m.kind
        UNION SELECT m.id, x.item_id FROM mine m JOIN item_external x ON x.source = 'Imdb' AND x.value = m.imdb_id JOIN items i ON i.id = x.item_id AND i.type = m.kind)
     SELECT m.id, m.kind, m.tmdb_id, m.tvdb_id, m.imdb_id, c.item FROM mine m LEFT JOIN copies c ON c.id = m.id
     ORDER BY m.added_at DESC, m.id DESC, c.item";

pub fn keys(conn: &Connection, user_id: &str) -> Result<Vec<Key>> {
    let mut out: Vec<Key> = vec![];
    let mut stmt = conn.prepare_cached(KEYS_SQL)?;
    let mut rows = stmt.query([user_id])?;
    while let Some(r) = rows.next()? {
        let id: i64 = r.get(0)?;
        if out.last().is_none_or(|k| k.id != id) {
            out.push(Key { id, kind: r.get(1)?, item_ids: vec![], tmdb_id: r.get(2)?, tvdb_id: r.get(3)?, imdb_id: r.get(4)? });
        }
        if let Some(item) = r.get::<_, Option<String>>(5)? {
            out.last_mut().expect("just pushed").item_ids.push(item);
        }
    }
    Ok(out)
}

/// The one live title of `kind` these provider ids can mean, if there is one. Several items that share the ids
/// are copies of one title (a film in an HD and a 4K library) and the lowest id stands for them, as it does for
/// Pipeline; ids that lead to *different* titles — two TMDB ids among the candidates, say — mean nothing is
/// sure, and nothing is chosen. Nothing here rewrites history, which is why copies are not refused the way
/// `relink.rs` refuses them.
pub fn find_title(conn: &Connection, t: &Title) -> Result<Option<String>> {
    let rows: Vec<(String, String, String)> = conn
        .prepare_cached(
            "SELECT x.item_id, x.source, x.value FROM item_external x JOIN items i ON i.id = x.item_id AND i.type = ?1 AND i.removed = 0
             WHERE x.item_id IN (SELECT item_id FROM item_external WHERE source = 'Tmdb' AND value = ?2
                          UNION SELECT item_id FROM item_external WHERE source = 'Tvdb' AND value = ?3
                          UNION SELECT item_id FROM item_external WHERE source = 'Imdb' AND value = ?4)",
        )?
        .query_map(params![t.kind, t.tmdb_id, t.tvdb_id, t.imdb_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
        .collect::<Result<_, _>>()?;
    let mut said: BTreeMap<&str, &str> = BTreeMap::new();
    for (_, source, value) in &rows {
        if *said.entry(source).or_insert(value) != value.as_str() {
            return Ok(None);
        }
    }
    Ok(rows.into_iter().map(|(id, _, _)| id).min())
}

/// A provider id as a page sends it: text, or a number (Upcoming has them as numbers).
fn provider_id(v: Option<Value>) -> Result<Option<String>, Refused> {
    match v {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) => Ok(Some(s.trim().to_string()).filter(|s| !s.is_empty())),
        Some(Value::Number(n)) if n.is_u64() => Ok(Some(n.to_string())),
        Some(_) => Err(Refused::Invalid("That doesn't look like a TMDB, TVDB or IMDb id")),
    }
}

// ---------------------------------------------------------------- the endpoints: the caller's own list, and nobody else's

impl From<Refused> for ApiError {
    fn from(r: Refused) -> Self {
        match r {
            Refused::NotATitle => ApiError::not_found("Film or show"),
            Refused::Full => ApiError::bad_request("Your watchlist holds 1,000 titles, the most it can. Remove one first."),
            Refused::Invalid(why) => ApiError::bad_request(why),
        }
    }
}

#[derive(Deserialize)]
pub struct AddBody {
    item_id: Option<String>,
    kind: Option<String>,
    tmdb_id: Option<Value>,
    tvdb_id: Option<Value>,
    imdb_id: Option<Value>,
    title: Option<String>,
    year: Option<i64>,
}

impl AddBody {
    fn wanted(self) -> Result<Wanted, Refused> {
        if let Some(id) = self.item_id.filter(|i| !i.trim().is_empty()) {
            return Ok(Wanted::Item(id));
        }
        Ok(Wanted::Title(Title {
            kind: self.kind.unwrap_or_default(),
            tmdb_id: provider_id(self.tmdb_id)?,
            tvdb_id: provider_id(self.tvdb_id)?,
            imdb_id: provider_id(self.imdb_id)?,
            title: self.title.unwrap_or_default(),
            year: self.year,
        }))
    }
}

/// `POST /api/me/watchlist` — `201` when it was added, `200` when it was there already.
pub async fn add_mine(State(app): State<App>, user: AuthUser, Json(body): Json<AddBody>) -> ApiResult<Response> {
    let wanted = body.wanted()?;
    let uid = user.id.clone();
    let (id, created) = app
        .db
        .call(move |c| {
            let tx = c.transaction()?;
            let done = add(&tx, &uid, wanted, db::now())?;
            tx.commit()?;
            Ok(done)
        })
        .await??;
    Ok(((if created { StatusCode::CREATED } else { StatusCode::OK }), Json(json!({ "id": id, "created": created }))).into_response())
}

/// `DELETE /api/me/watchlist/{id}` — only ever the caller's own; anybody else's is not there.
pub async fn remove_mine(State(app): State<App>, user: AuthUser, Path(id): Path<i64>) -> ApiResult {
    let uid = user.id.clone();
    if !app.db.call(move |c| remove(c, &uid, id)).await? {
        return Err(ApiError::not_found("Watchlist entry"));
    }
    Ok(Json(json!({ "ok": true })))
}

/// `GET /api/me/watchlist/keys` — what is on the caller's list, for the pages that offer the toggle.
pub async fn keys_mine(State(app): State<App>, user: AuthUser) -> ApiResult {
    let uid = user.id.clone();
    let entries = app.db.call(move |c| keys(c, &uid)).await?;
    let entries: Vec<Value> = entries
        .into_iter()
        .map(|k| json!({ "id": k.id, "kind": k.kind, "item_ids": k.item_ids, "tmdb_id": k.tmdb_id, "tvdb_id": k.tvdb_id, "imdb_id": k.imdb_id }))
        .collect();
    Ok(Json(json!({ "entries": entries })))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn conn() -> Connection {
        let c = Connection::open_in_memory().unwrap();
        for m in crate::db::MIGRATIONS {
            c.execute_batch(m).unwrap();
        }
        c.execute_batch(
            r#"INSERT INTO items(id, type, name, production_year, provider_ids, removed, updated_at) VALUES
                ('bbb', 'Movie', 'Big Buck Bunny', 2008, '{"Tmdb":"10378","Imdb":"tt1254207"}', 0, 0),
                ('bbb4k', 'Movie', 'Big Buck Bunny', 2008, '{"Tmdb":"10378"}', 0, 0),
                ('sintel', 'Movie', 'Sintel', 2010, '{"Tmdb":"45745","Imdb":"tt1727587"}', 0, 0),
                ('show', 'Series', 'Low Orbit', 2021, '{"Tvdb":"370001","Tmdb":"990010"}', 0, 0),
                ('plain', 'Movie', 'Home Video', 2019, NULL, 0, 0);
               INSERT INTO items(id, type, name, series_id, updated_at) VALUES ('ep1', 'Episode', 'Pilot', 'show', 0);"#,
        )
        .unwrap();
        crate::pipeline::rebuild_external(&c).unwrap();
        c
    }

    fn film(tmdb: Option<&str>, imdb: Option<&str>, title: &str) -> Title {
        Title { kind: "Movie".into(), tmdb_id: tmdb.map(Into::into), imdb_id: imdb.map(Into::into), title: title.into(), year: None, ..Default::default() }
    }

    fn row(c: &Connection, id: i64) -> (String, Option<String>, Title) {
        c.query_row("SELECT user_id, item_id, kind, tmdb_id, tvdb_id, imdb_id, title, year FROM watchlist WHERE id = ?1", [id], |r| {
            Ok((r.get(0)?, r.get(1)?, Title { kind: r.get(2)?, tmdb_id: r.get(3)?, tvdb_id: r.get(4)?, imdb_id: r.get(5)?, title: r.get(6)?, year: r.get(7)? }))
        })
        .unwrap()
    }

    #[test]
    fn a_title_from_the_library_is_kept_with_a_snapshot_of_itself() {
        let c = conn();
        let (id, created) = add(&c, "alice", Wanted::Item("BBB".into()), 100).unwrap().unwrap();
        assert!(created);
        let (user, item, t) = row(&c, id);
        assert_eq!((user.as_str(), item.as_deref()), ("alice", Some("bbb")));
        assert_eq!(t, Title { kind: "Movie".into(), tmdb_id: Some("10378".into()), tvdb_id: None, imdb_id: Some("tt1254207".into()), title: "Big Buck Bunny".into(), year: Some(2008) });
    }

    #[test]
    fn only_films_and_shows_can_be_added() {
        let c = conn();
        assert_eq!(add(&c, "alice", Wanted::Item("ep1".into()), 100).unwrap(), Err(Refused::NotATitle), "an episode");
        assert_eq!(add(&c, "alice", Wanted::Item("nothing".into()), 100).unwrap(), Err(Refused::NotATitle), "an id the library does not have");
        let mut t = film(Some("1"), None, "Something");
        t.kind = "Episode".into();
        assert!(matches!(add(&c, "alice", Wanted::Title(t), 100).unwrap(), Err(Refused::Invalid(_))));
    }

    #[test]
    fn a_title_not_in_the_library_needs_an_id_and_a_name_that_look_right() {
        let c = conn();
        for bad in [
            film(None, None, "No ids at all"),
            film(Some("12ab"), None, "A TMDB id is a number"),
            film(None, Some("1254207"), "An IMDb id starts with tt"),
            film(Some("1"), None, "   "),
            film(Some("1"), None, &"x".repeat(301)),
        ] {
            assert!(matches!(add(&c, "alice", Wanted::Title(bad.clone()), 100).unwrap(), Err(Refused::Invalid(_))), "{bad:?}");
        }
        let mut t = film(Some("990001"), None, "Winterline");
        t.year = Some(99_999);
        assert!(matches!(add(&c, "alice", Wanted::Title(t.clone()), 100).unwrap(), Err(Refused::Invalid(_))), "a year no film has");
        t.year = Some(2026);
        let (id, _) = add(&c, "alice", Wanted::Title(t.clone()), 100).unwrap().unwrap();
        let (_, item, kept) = row(&c, id);
        assert_eq!((item, kept), (None, t), "not in the library: kept by its id, as given");
    }

    #[test]
    fn adding_what_is_already_there_finds_it() {
        let c = conn();
        let (id, _) = add(&c, "alice", Wanted::Item("bbb".into()), 100).unwrap().unwrap();
        assert_eq!(add(&c, "alice", Wanted::Item("bbb".into()), 200).unwrap(), Ok((id, false)));
        assert_eq!(add(&c, "alice", Wanted::Item("bbb4k".into()), 200).unwrap(), Ok((id, false)), "the 4K copy is the same film");
        assert_eq!(add(&c, "alice", Wanted::Title(film(None, Some("tt1254207"), "Big Buck Bunny")), 200).unwrap(), Ok((id, false)), "by any of its ids");
        let (other, created) = add(&c, "bob", Wanted::Item("bbb".into()), 200).unwrap().unwrap();
        assert!(created && other != id, "bob's list is his own");
        // The same number as a show's TMDB id is not the same title.
        let (show, created) = add(&c, "alice", Wanted::Title(Title { kind: "Series".into(), tmdb_id: Some("10378".into()), title: "Other".into(), ..Default::default() }), 200).unwrap().unwrap();
        assert!(created && show != id);
    }

    #[test]
    fn a_title_already_in_the_library_is_attached_at_once() {
        let c = conn();
        let (id, _) = add(&c, "alice", Wanted::Title(film(Some("45745"), None, "Sintel")), 100).unwrap().unwrap();
        assert_eq!(row(&c, id).1.as_deref(), Some("sintel"));
    }

    #[test]
    fn copies_of_one_title_are_one_title_and_different_titles_are_nobody_s_guess() {
        let c = conn();
        assert_eq!(find_title(&c, &film(Some("10378"), None, "")).unwrap().as_deref(), Some("bbb"), "HD and 4K: the lowest id stands for both");
        assert_eq!(find_title(&c, &film(Some("10378"), Some("tt1254207"), "")).unwrap().as_deref(), Some("bbb"));
        assert_eq!(find_title(&c, &film(Some("10378"), Some("tt1727587"), "")).unwrap(), None, "the TMDB id says one film, the IMDb id another");
        assert_eq!(find_title(&c, &film(Some("990010"), None, "")).unwrap(), None, "a show's TMDB id is not a film's");
        assert_eq!(find_title(&c, &Title { kind: "Series".into(), tvdb_id: Some("370001".into()), ..Default::default() }).unwrap().as_deref(), Some("show"));
        c.execute_batch("UPDATE items SET removed = 1 WHERE id = 'sintel'; DELETE FROM item_external WHERE item_id = 'sintel';").unwrap();
        assert_eq!(find_title(&c, &film(Some("45745"), None, "")).unwrap(), None, "a removed title is not in the library");
    }

    #[test]
    fn a_list_holds_at_most_a_thousand_titles() {
        let c = conn();
        let (kept, _) = add(&c, "alice", Wanted::Item("bbb".into()), 1).unwrap().unwrap();
        for n in 1..MAX_ENTRIES {
            c.execute("INSERT INTO watchlist(user_id, kind, tmdb_id, title, added_at) VALUES ('alice', 'Movie', ?1, 'Filler', 1)", [(1_000_000 + n).to_string()]).unwrap();
        }
        assert_eq!(add(&c, "alice", Wanted::Item("sintel".into()), 2).unwrap(), Err(Refused::Full));
        assert_eq!(add(&c, "alice", Wanted::Item("bbb".into()), 2).unwrap(), Ok((kept, false)), "asking for one already there is not adding");
        assert!(add(&c, "bob", Wanted::Item("sintel".into()), 2).unwrap().is_ok(), "the bound is per person");
    }

    #[test]
    fn an_entry_is_removed_only_by_its_owner() {
        let c = conn();
        let (id, _) = add(&c, "alice", Wanted::Item("bbb".into()), 1).unwrap().unwrap();
        assert!(!remove(&c, "bob", id).unwrap());
        assert!(remove(&c, "alice", id).unwrap());
        assert!(!remove(&c, "alice", id).unwrap());
    }

    #[test]
    fn keys_name_every_copy_of_a_title_and_the_ids_of_one_not_here_yet() {
        let c = conn();
        let (film_id, _) = add(&c, "alice", Wanted::Item("bbb4k".into()), 1).unwrap().unwrap();
        let (show_id, _) = add(&c, "alice", Wanted::Title(Title { kind: "Series".into(), tvdb_id: Some("380002".into()), title: "Nightfall Bay".into(), ..Default::default() }), 2).unwrap().unwrap();
        add(&c, "bob", Wanted::Item("sintel".into()), 3).unwrap().unwrap();
        let k = keys(&c, "alice").unwrap();
        assert_eq!(k, vec![
            Key { id: show_id, kind: "Series".into(), item_ids: vec![], tmdb_id: None, tvdb_id: Some("380002".into()), imdb_id: None },
            Key { id: film_id, kind: "Movie".into(), item_ids: vec!["bbb".into(), "bbb4k".into()], tmdb_id: Some("10378".into()), tvdb_id: None, imdb_id: None },
        ], "newest first, and nothing of bob's");
    }

    #[test]
    fn a_title_without_provider_ids_is_its_own_key() {
        let c = conn();
        let (id, _) = add(&c, "alice", Wanted::Item("plain".into()), 1).unwrap().unwrap();
        assert_eq!(keys(&c, "alice").unwrap()[0].item_ids, ["plain"]);
        assert_eq!(add(&c, "alice", Wanted::Item("plain".into()), 2).unwrap(), Ok((id, false)));
    }

    #[test]
    fn one_person_s_list_is_read_through_its_index() {
        let c = conn();
        let plan = |sql: &str| -> String {
            c.prepare(&format!("EXPLAIN QUERY PLAN {sql}")).unwrap().query_map(params!["alice"], |r| r.get::<_, String>(3)).unwrap().map(Result::unwrap).collect::<Vec<_>>().join(" | ")
        };
        let p = plan(KEYS_SQL);
        assert!(p.contains("idx_watchlist_user") && !p.contains("SCAN item_external") && !p.contains("SCAN items"), "{p}");
    }

    #[test]
    fn an_id_arrives_as_a_number_or_as_text() {
        assert_eq!(provider_id(Some(json!(380002))), Ok(Some("380002".to_string())), "Upcoming hands ids over as numbers");
        assert_eq!(provider_id(Some(json!(" tt1254207 "))), Ok(Some("tt1254207".to_string())));
        assert_eq!(provider_id(Some(json!(""))), Ok(None));
        assert_eq!(provider_id(None), Ok(None));
        assert_eq!(provider_id(Some(Value::Null)), Ok(None));
        for bad in [json!(-3), json!(1.5), json!([1]), json!({ "a": 1 }), json!(true)] {
            assert!(provider_id(Some(bad.clone())).is_err(), "{bad}");
        }
    }
}
