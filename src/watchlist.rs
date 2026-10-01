//! A person's list of films and shows they mean to watch. It belongs to its owner and nobody else: every
//! endpoint acts on the caller and takes no `user_id`, and no permission — `see_everyone`, a Jellyfin
//! administrator — opens somebody else's. Written to finstats' own database only; nothing here talks to
//! Jellyfin, Seerr, Sonarr or Radarr.
//!
//! **An entry names a title in one of two ways**: by its Jellyfin item when it is in the library, or by a
//! provider id (TMDB, TVDB, IMDb) when it is not yet. Either way it keeps a snapshot — kind, title, year and
//! the provider ids — so it still reads as something after its title leaves the library, or before it comes.

use anyhow::Result;

use std::collections::{BTreeMap, HashMap, HashSet};

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
    if let Some((id, _)) = already(conn, user_id, item_id.as_deref(), &t, None)? {
        return Ok(Ok((id, false)));
    }
    let held: i64 = conn.query_row("SELECT COUNT(*) FROM watchlist WHERE user_id = ?1", [user_id], |r| r.get(0))?;
    if held >= MAX_ENTRIES {
        return Ok(Err(Refused::Full));
    }
    conn.execute(
        "INSERT INTO watchlist(user_id, kind, item_id, tmdb_id, tvdb_id, imdb_id, title, year, added_at, missing) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![user_id, t.kind, item_id, t.tmdb_id, t.tvdb_id, t.imdb_id, t.title, t.year, now, item_id.is_none()],
    )?;
    Ok(Ok((conn.last_insert_rowid(), true)))
}

/// One provider id out of an item's `provider_ids` JSON, as text, `NULL` when it has none.
fn provider_id_sql(column: &str, key: &str) -> String {
    format!("CASE WHEN json_valid({column}) THEN NULLIF(CAST(json_extract({column}, '$.{key}') AS TEXT), '') END")
}

/// A film or show in the library as an entry would remember it.
fn snapshot(conn: &Connection, item_id: &str) -> Result<Option<Title>> {
    let id = |key: &str| provider_id_sql("provider_ids", key);
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
/// `except`: an entry not to count, the one being asked about. (id, added at), the oldest if there are several.
fn already(conn: &Connection, user_id: &str, item_id: Option<&str>, t: &Title, except: Option<i64>) -> Result<Option<(i64, i64)>> {
    Ok(standing_for(conn, user_id, item_id, t, except)?.into_iter().next())
}

/// Every entry of `user_id`'s that stands for this title, oldest first, as `already` counts them.
fn standing_for(conn: &Connection, user_id: &str, item_id: Option<&str>, t: &Title, except: Option<i64>) -> Result<Vec<(i64, i64)>> {
    let rows = conn
        .prepare_cached(
            "SELECT id, added_at FROM watchlist WHERE user_id = ?1 AND id IS NOT ?7
               AND (item_id = ?2 OR (kind = ?3 AND (tmdb_id = ?4 OR tvdb_id = ?5 OR imdb_id = ?6)))
             ORDER BY added_at, id",
        )?
        .query_map(params![user_id, item_id, t.kind, t.tmdb_id, t.tvdb_id, t.imdb_id, except], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<Result<_, _>>()?;
    Ok(rows)
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

const FIND_SQL: &str = "SELECT x.item_id, x.source, x.value FROM item_external x JOIN items i ON i.id = x.item_id AND i.type = ?1 AND i.removed = 0
     WHERE x.item_id IN (SELECT item_id FROM item_external WHERE source = 'Tmdb' AND value = ?2
                  UNION SELECT item_id FROM item_external WHERE source = 'Tvdb' AND value = ?3
                  UNION SELECT item_id FROM item_external WHERE source = 'Imdb' AND value = ?4)";

/// The one live title of `kind` these provider ids can mean, if there is one. Several items that share the ids
/// are copies of one title (a film in an HD and a 4K library) and the lowest id stands for them, as it does for
/// Pipeline; ids that lead to *different* titles — two TMDB ids among the candidates, say — mean nothing is
/// sure, and nothing is chosen. Nothing here rewrites history, which is why copies are not refused the way
/// `relink.rs` refuses them.
pub fn find_title(conn: &Connection, t: &Title) -> Result<Option<String>> {
    let rows: Vec<(String, String, String)> = conn
        .prepare_cached(FIND_SQL)?
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

/// A request in Seerr that has not arrived yet, as a watchlist may speak of it.
struct Asked {
    media_type: String,
    tmdb_id: Option<String>,
    tvdb_id: Option<String>,
    imdb_id: Option<String>,
    mine: bool,
    user_name: Option<String>,
    poster: Option<(i64, i64)>,
}

impl Asked {
    fn is_for(&self, k: &Key) -> bool {
        let same = |a: &Option<String>, b: &Option<String>| a.is_some() && a == b;
        match k.kind.as_str() {
            "Movie" => self.media_type == "movie" && (same(&self.tmdb_id, &k.tmdb_id) || same(&self.imdb_id, &k.imdb_id)),
            _ => self.media_type == "tv" && (same(&self.tvdb_id, &k.tvdb_id) || same(&self.tmdb_id, &k.tmdb_id)),
        }
    }
}

/// What is still open in Seerr, under Pipeline's rule: the caller's own requests, and everybody's only for somebody
/// who may see everyone's activity. The caller's own come first, so "you asked for it" wins over "bob did".
fn open_requests(conn: &Connection, user_id: &str, everyone: bool) -> Result<Vec<Asked>> {
    let sql = format!(
        "SELECT r.media_type, CAST(r.tmdb_id AS TEXT), CAST(r.tvdb_id AS TEXT), r.imdb_id, COALESCE(r.user_id = ?1, 0), COALESCE(u.name, r.seerr_user_name),
                r.arr_service_id, r.arr_media_id
         FROM (SELECT r.*, {} AS state FROM requests r) r LEFT JOIN users u ON u.id = r.user_id
         WHERE r.state IN ('pending', 'approved', 'processing', 'partial') AND (?2 OR r.user_id = ?1)
         ORDER BY COALESCE(r.user_id = ?1, 0) DESC, r.requested_at",
        crate::pipeline::STATE_SQL
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![user_id, everyone], |r| {
        Ok(Asked {
            media_type: r.get(0)?, tmdb_id: r.get(1)?, tvdb_id: r.get(2)?, imdb_id: r.get(3)?, mine: r.get(4)?, user_name: r.get(5)?,
            poster: match (r.get::<_, Option<i64>>(6)?, r.get::<_, Option<i64>>(7)?) {
                (Some(s), Some(m)) => Some((s, m)),
                _ => None,
            },
        })
    })?;
    let asked = rows.collect::<Result<_, _>>()?;
    Ok(asked)
}

/// Is this calendar entry the title an entry names? A film by its TMDB id, a show by its TVDB or TMDB id.
fn is_on_calendar(e: &crate::pipeline::Entry, k: &Key) -> bool {
    let same = |a: Option<i64>, b: &Option<String>| a.is_some_and(|a| b.as_deref() == Some(a.to_string().as_str()));
    match (k.kind.as_str(), e.kind.as_str()) {
        ("Movie", "movie") => same(e.tmdb_id, &k.tmdb_id),
        ("Series", "episode") => same(e.tvdb_id, &k.tvdb_id) || same(e.tmdb_id, &k.tmdb_id),
        _ => false,
    }
}

/// One entry out of a backup. Its id means nothing here: it is added when the person has nothing standing for its
/// title, and otherwise only gives the entry that is here its date, when it is the older. `true` when it changed
/// something. The per-person bound holds here too.
pub fn restore_row(conn: &Connection, row: &serde_json::Map<String, Value>) -> Result<bool> {
    let text = |k: &str| row.get(k).and_then(Value::as_str).map(str::trim).filter(|s| !s.is_empty()).map(str::to_string);
    let (Some(user_id), Some(kind), Some(title), Some(added_at)) = (text("user_id"), text("kind"), text("title"), row.get("added_at").and_then(Value::as_i64)) else {
        return Ok(false);
    };
    let t = Title { kind, tmdb_id: text("tmdb_id"), tvdb_id: text("tvdb_id"), imdb_id: text("imdb_id"), title, year: row.get("year").and_then(Value::as_i64) };
    let item_id = text("item_id").map(|i| db::norm_id(&i));
    if !KINDS.contains(&t.kind.as_str()) || (item_id.is_none() && t.tmdb_id.is_none() && t.tvdb_id.is_none() && t.imdb_id.is_none()) {
        return Ok(false);
    }
    if let Some((id, here)) = already(conn, &user_id, item_id.as_deref(), &t, None)? {
        return Ok(added_at < here && conn.execute("UPDATE watchlist SET added_at = ?2 WHERE id = ?1", params![id, added_at])? > 0);
    }
    let held: i64 = conn.query_row("SELECT COUNT(*) FROM watchlist WHERE user_id = ?1", [&user_id], |r| r.get(0))?;
    if held >= MAX_ENTRIES {
        return Ok(false);
    }
    conn.execute(
        "INSERT INTO watchlist(user_id, kind, item_id, tmdb_id, tvdb_id, imdb_id, title, year, added_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![user_id, t.kind, item_id, t.tmdb_id, t.tvdb_id, t.imdb_id, t.title, t.year, added_at],
    )?;
    // Whether it is here is this library's to say, not the backup's.
    conn.execute(&format!("UPDATE watchlist SET missing = {GONE} WHERE id = ?1"), [conn.last_insert_rowid()])?;
    Ok(true)
}

/// After a library read: tell each person whose watchlist was waiting for a title that it is here — a title that was
/// missing at a library read and came after they put it on the list, in the last few hours. A new item alone is not an
/// arrival: a replaced file is a new path and so a new item, of a title that never left. Re-derived from that window every time, so saying it twice is
/// impossible and a missed read loses nothing; anything older is history and says nothing. Only ever to the person's own
/// destinations (`notify::Need::Owner`).
pub fn announce(conn: &Connection, bus: &crate::notify::Fanout) -> Result<usize> {
    use crate::notify::{Event, Kind};
    if !bus.anyone_wants(Kind::WatchlistAvailable) {
        return Ok(0);
    }
    type Arrived = (String, String, String, String, String, i64);
    let rows: Vec<Arrived> = conn
        .prepare_cached(
            "SELECT w.user_id, COALESCE(u.name, w.user_id), w.kind, i.id, i.name, i.date_created
             FROM watchlist w JOIN items i ON i.id = w.item_id AND i.removed = 0 LEFT JOIN users u ON u.id = w.user_id
             WHERE w.arrived_at >= ?1 AND i.date_created >= ?1 AND i.date_created > w.added_at",
        )?
        .query_map([db::now() - crate::notify::HISTORIC_S], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?)))?
        .collect::<Result<_, _>>()?;
    let mut told = 0;
    for (user_id, user_name, kind, item_id, name, arrived) in rows {
        let event = Event::new(
            Kind::WatchlistAvailable,
            format!("notify:watchlist:{user_id}:{item_id}"),
            format!("Now on the server: {name}"),
            format!("The {} on your watchlist is in the library now.", if kind == "Series" { "show" } else { "film" }),
        )
        .at(arrived)
        .field("Title", name)
        .link(format!("/items/{item_id}"))
        .about(user_id, user_name);
        told += usize::from(crate::notify::raise_in(conn, bus, &event)?);
    }
    Ok(told)
}

/// After a library read or a look at what changed: has anything somebody was waiting for arrived?
pub async fn check_arrivals(app: &App) {
    let bus_app = app.clone();
    let done = app.db.call(move |c| {
        let bus = crate::notify::Fanout::of(c, &bus_app)?;
        announce(c, &bus)
    }).await;
    match done {
        Ok(n) if n > 0 => app.notify_wake.notify_one(),
        Ok(_) => {}
        Err(e) => tracing::warn!("could not announce what arrived for watchlists: {e:#}"),
    }
}

/// An entry's title is not in the library: never attached, or attached to an item that is gone.
const GONE: &str = "NOT EXISTS (SELECT 1 FROM items i WHERE i.id = watchlist.item_id AND i.removed = 0)";

/// Entries the library does not have as far as they know: never attached, or attached to an item that is gone.
/// Bounded by the per-person cap, and each one's item looked up by its key.
const WAITING_SQL: &str = "SELECT w.id, w.user_id, w.kind, w.item_id, w.tmdb_id, w.tvdb_id, w.imdb_id, w.added_at
     FROM watchlist w LEFT JOIN items i ON i.id = w.item_id
     WHERE w.item_id IS NULL OR i.id IS NULL OR i.removed <> 0";

/// After a library read: attach every entry whose title is in the library now — one that was waiting for it, or one
/// whose title was renamed into a new item — and merge two entries that turn out to be one title, keeping the older.
/// Answers how many entries were attached.
pub fn resolve(conn: &Connection) -> Result<usize> {
    // What the library calls an attached title, first: an id it has learnt may be what makes a waiting entry the same title.
    let pid = |key: &str| provider_id_sql("i.provider_ids", key);
    conn.execute(
        &format!(
            "UPDATE watchlist SET title = i.name, year = COALESCE(i.production_year, watchlist.year),
                    tmdb_id = COALESCE({t}, watchlist.tmdb_id), tvdb_id = COALESCE({v}, watchlist.tvdb_id), imdb_id = COALESCE({m}, watchlist.imdb_id)
             FROM items i WHERE i.id = watchlist.item_id AND i.removed = 0 AND i.type = watchlist.kind
               AND (watchlist.title, watchlist.year, watchlist.tmdb_id, watchlist.tvdb_id, watchlist.imdb_id)
                   IS NOT (i.name, COALESCE(i.production_year, watchlist.year), COALESCE({t}, watchlist.tmdb_id), COALESCE({v}, watchlist.tvdb_id), COALESCE({m}, watchlist.imdb_id))",
            t = pid("Tmdb"),
            v = pid("Tvdb"),
            m = pid("Imdb")
        ),
        [],
    )?;
    let waiting: Vec<(i64, String, Title, i64)> = conn
        .prepare(WAITING_SQL)?
        .query_map([], |r| {
            let t = Title { kind: r.get(2)?, tmdb_id: r.get(4)?, tvdb_id: r.get(5)?, imdb_id: r.get(6)?, ..Default::default() };
            Ok((r.get(0)?, r.get(1)?, t, r.get(7)?))
        })?
        .collect::<Result<_, _>>()?;
    let (mut attached, mut merged) = (0, HashSet::new());
    for (id, user_id, t, added_at) in waiting {
        if merged.contains(&id) {
            continue; // folded into an older entry of the same title already
        }
        let Some(found) = find_title(conn, &t)? else { continue };
        // Other entries of theirs may already stand for this title — on it, on a copy of it, or by one of its ids. All of
        // them are one entry now, the oldest, and the rest go before it moves: one of them may hold the item.
        let mut same = match snapshot(conn, &found)? {
            Some(title) => standing_for(conn, &user_id, Some(&found), &title, Some(id))?,
            None => vec![],
        };
        same.push((id, added_at));
        let (keep, _) = same.iter().copied().min_by_key(|&(e, at)| (at, e)).expect("holds this entry");
        // If any of them had the title here at the last look, the one kept had it too: merging is not an arrival.
        let ids = serde_json::to_string(&same.iter().map(|&(e, _)| e).collect::<Vec<_>>())?;
        let missing: bool = conn.query_row("SELECT MIN(missing) FROM watchlist WHERE id IN (SELECT value FROM json_each(?1))", [&ids], |r| r.get(0))?;
        for &(e, _) in same.iter().filter(|&&(e, _)| e != keep) {
            conn.execute("DELETE FROM watchlist WHERE id = ?1", [e])?;
            merged.insert(e);
        }
        conn.execute("UPDATE watchlist SET item_id = ?2, missing = ?3 WHERE id = ?1", params![keep, found, missing])?;
        attached += 1;
    }
    if attached > 0 {
        tracing::info!("attached {attached} watchlist entries to titles in the library");
    }
    // What was missing at the last look and is here now has arrived; then note what is missing now, for the next one.
    conn.execute(&format!("UPDATE watchlist SET arrived_at = ?1, missing = 0 WHERE missing = 1 AND NOT {GONE}"), [db::now()])?;
    conn.execute(&format!("UPDATE watchlist SET missing = 1 WHERE missing = 0 AND {GONE}"), [])?;
    Ok(attached)
}

/// Every entry on `user_id`'s list, newest first, each with what is true of its title now — never stored, so it
/// cannot go stale. `everyone`: the caller may see everyone's activity, and so who asked for a title in Seerr.
///
/// The state is the first of these that holds: `watched` (by the profile's reading of "seen"; a show when every
/// episode on disk is), `started` or `on_server` (in the library), `requested` (open in Seerr), `coming_up` (Sonarr or
/// Radarr has a date), `left_library` (it was here and is not any more), `not_on_server`.
pub fn listing(conn: &Connection, user_id: &str, everyone: bool) -> Result<Vec<Value>> {
    let keys = keys(conn, user_id)?;
    let snapshot: HashMap<i64, (Option<String>, String, Option<i64>, i64)> = conn
        .prepare_cached("SELECT id, item_id, title, year, added_at FROM watchlist WHERE user_id = ?1")?
        .query_map([user_id], |r| Ok((r.get(0)?, (r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))))?
        .collect::<Result<_, _>>()?;
    let every_item: Vec<String> = keys.iter().flat_map(|k| k.item_ids.iter().cloned()).collect();
    let live: HashSet<String> = conn
        .prepare_cached("SELECT id FROM items WHERE id IN (SELECT value FROM json_each(?1)) AND removed = 0")?
        .query_map([serde_json::to_string(&every_item)?], |r| r.get(0))?
        .collect::<Result<_, _>>()?;

    // How far the caller got: films by the profile's rule, shows as the profile counts them (files only, no specials).
    let film_items: Vec<String> = keys.iter().filter(|k| k.kind == "Movie").flat_map(|k| k.item_ids.iter().cloned()).collect();
    let films: HashMap<String, &str> = crate::profile::films(conn, user_id, &film_items)?.into_iter().map(|(id, state, _)| (id, state)).collect();
    let mut shows: HashMap<String, (i64, i64, i64)> = HashMap::new(); // series → (seen, started, on disk)
    for e in crate::profile::episodes(conn, user_id, None)? {
        let s = shows.entry(e.series_id).or_default();
        s.0 += i64::from(e.state == "seen");
        s.1 += i64::from(e.state == "started");
        s.2 += 1;
    }
    let requests = open_requests(conn, user_id, everyone)?;
    let calendar = crate::pipeline::entries_for(conn, 90, user_id, false)?;

    let mut out = vec![];
    for k in &keys {
        let Some((item_id, title, year, added_at)) = snapshot.get(&k.id) else { continue };
        let here = k.item_ids.iter().find(|i| live.contains(*i));
        // Of several copies, the one furthest along speaks for the title.
        let (seen, begun, progress) = if k.kind == "Movie" {
            let rank = |i: &String| match films.get(i).copied() { Some("seen") => 2, Some("started") => 1, _ => 0 };
            let best = k.item_ids.iter().map(rank).max().unwrap_or(0);
            (best == 2, best == 1, Value::Null)
        } else {
            match k.item_ids.iter().filter_map(|i| shows.get(i)).max_by_key(|s| (s.0, s.2)) {
                Some(&(seen, started, total)) => (total > 0 && seen == total, seen + started > 0, json!({ "seen": seen, "total": total })),
                None => (false, false, Value::Null),
            }
        };
        let asked = requests.iter().find(|r| r.is_for(k));
        let next = calendar.iter().map(|(e, _)| e).find(|e| !e.has_file && is_on_calendar(e, k));
        let state = if seen {
            "watched"
        } else if here.is_some() {
            if begun { "started" } else { "on_server" }
        } else if asked.is_some() {
            "requested"
        } else if next.is_some() {
            "coming_up"
        } else if item_id.is_some() {
            "left_library"
        } else {
            "not_on_server"
        };
        let poster = match (here.or(item_id.as_ref()), next, asked.and_then(|a| a.poster)) {
            (Some(id), _, _) => json!({ "item_id": id }),
            (None, Some(e), _) => json!({ "service_id": e.service_id, "media_id": e.arr_media_id }),
            (None, None, Some((s, m))) => json!({ "service_id": s, "media_id": m }),
            _ => Value::Null,
        };
        let request = asked.map(|a| if a.mine { json!({ "by_you": true }) } else { json!({ "by_you": false, "user_name": a.user_name }) });
        out.push(json!({
            "id": k.id, "kind": k.kind, "title": title, "year": year, "added_at": added_at, "item_id": here,
            "tmdb_id": k.tmdb_id, "tvdb_id": k.tvdb_id, "imdb_id": k.imdb_id,
            "state": state, "progress": progress, "request": request, "poster": poster,
            "next": next.map(|e| json!({ "day": e.day, "at": e.at, "release": e.release, "season": e.season, "episode": e.episode })),
        }));
    }
    Ok(out)
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

/// `GET /api/me/watchlist` — the caller's list, each entry with its state, and who the caller is for the page's header.
pub async fn list_mine(State(app): State<App>, user: AuthUser) -> ApiResult {
    let (uid, everyone) = (user.id.clone(), user.perms.see_everyone);
    let (who, entries) = app
        .db
        .call(move |c| {
            let who = c
                .query_row("SELECT name, image_tag IS NOT NULL FROM users WHERE id = ?1", [&uid], |r| Ok((r.get::<_, String>(0)?, r.get::<_, bool>(1)?)))
                .optional()?;
            Ok((who, listing(c, &uid, everyone)?))
        })
        .await?;
    let (name, has_image) = who.unwrap_or_else(|| (user.name.clone(), false));
    Ok(Json(json!({ "user": { "id": user.id, "name": name, "has_image": has_image }, "entries": entries })))
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
    use serde_json::json;

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
    /// A household's library, Seerr and calendar, and alice's list with one entry for every state there is.
    fn household() -> Connection {
        let c = conn();
        c.execute_batch(
            r#"INSERT INTO users(id, name, updated_at) VALUES ('alice', 'alice', 0), ('bob', 'bob', 0);
               INSERT INTO items(id, type, name, runtime_s, removed, provider_ids, updated_at) VALUES
                 ('gone', 'Movie', 'Old Cut', 6000, 1, '{"Tmdb":"555"}', 0);
               UPDATE items SET runtime_s = 6000 WHERE type = 'Movie';
               INSERT INTO items(id, type, name, series_id, parent_index_number, index_number, path, runtime_s, updated_at) VALUES
                 ('e1', 'Episode', 'One', 'show', 1, 1, '/e1', 1000, 0), ('e2', 'Episode', 'Two', 'show', 1, 2, '/e2', 1000, 0),
                 ('e3', 'Episode', 'Three', 'show', 1, 3, '/e3', 1000, 0), ('e4', 'Episode', 'Four', 'show', 1, 4, '/e4', 1000, 0);
               UPDATE items SET path = NULL WHERE id = 'ep1';
               INSERT INTO playbacks(source, user_id, user_name, item_id, item_name, item_type, series_id, started_at, ended_at, duration_s, position_s, runtime_s) VALUES
                 ('live', 'alice', 'alice', 'bbb4k', 'Big Buck Bunny', 'Movie', NULL, 0, 5500, 5500, 5500, 6000),
                 ('live', 'alice', 'alice', 'sintel', 'Sintel', 'Movie', NULL, 0, 600, 600, 600, 6000),
                 ('live', 'alice', 'alice', 'e1', 'One', 'Episode', 'show', 0, 1000, 1000, 1000, 1000),
                 ('live', 'alice', 'alice', 'e2', 'Two', 'Episode', 'show', 0, 1000, 1000, 1000, 1000);
               INSERT INTO services(id, kind, name, url, secret, created_at) VALUES (1, 'sonarr', 'Sonarr', 'http://nas:8989', 'k', 1),
                 (2, 'radarr', 'Radarr', 'http://nas:7878', 'k', 1), (3, 'seerr', 'Seerr', 'http://nas:5055', 'k', 1);
               INSERT INTO upcoming(service_id, kind, external_id, release, at, series_title, title, season, episode, year, tvdb_id, tmdb_id, arr_media_id)
                 VALUES (1, 'episode', 71, 'air', CAST(strftime('%s', 'now', '+3 days') AS INTEGER), 'Nightfall Bay', 'Arrival', 1, 1, 2026, 380002, NULL, 12);
               INSERT INTO requests(service_id, request_id, media_type, tmdb_id, title, status, media_status, requested_at, updated_at, user_id, arr_service_id, arr_media_id) VALUES
                 (3, 1, 'movie', 990001, 'Winterline', 2, 3, 10, 10, 'alice', 2, 31),
                 (3, 2, 'movie', 990002, 'Glass Harbour', 1, 2, 10, 10, 'bob', 2, 32);"#,
        )
        .unwrap();
        let t = |kind: &str, tmdb: Option<&str>, tvdb: Option<&str>, title: &str| {
            Wanted::Title(Title { kind: kind.into(), tmdb_id: tmdb.map(Into::into), tvdb_id: tvdb.map(Into::into), title: title.into(), ..Default::default() })
        };
        for (n, w) in [
            Wanted::Item("bbb".into()),                                   // seen, on the 4K copy
            Wanted::Item("sintel".into()),                                // a tenth of it
            Wanted::Item("plain".into()),                                 // in the library, not begun
            Wanted::Item("show".into()),                                  // two of four episodes
            Wanted::Item("gone".into()),                                  // left the library
            t("Movie", Some("990001"), None, "Winterline"),               // alice asked for it
            t("Movie", Some("990002"), None, "Glass Harbour"),            // bob asked for it
            t("Series", None, Some("380002"), "Nightfall Bay"),           // Sonarr has a date
            t("Movie", Some("990003"), None, "Nowhere Yet"),              // none of the above
        ]
        .into_iter()
        .enumerate()
        {
            add(&c, "alice", w, 100 + n as i64).unwrap().unwrap();
        }
        add(&c, "bob", Wanted::Item("sintel".into()), 1).unwrap().unwrap();
        c
    }

    fn states(c: &Connection, everyone: bool) -> Vec<(String, String)> {
        listing(c, "alice", everyone).unwrap().iter().map(|e| (e["title"].as_str().unwrap().to_string(), e["state"].as_str().unwrap().to_string())).collect()
    }

    #[test]
    fn every_entry_says_what_is_true_of_its_title_now() {
        let c = household();
        assert_eq!(states(&c, false), [
            ("Nowhere Yet", "not_on_server"), ("Nightfall Bay", "coming_up"), ("Glass Harbour", "not_on_server"), ("Winterline", "requested"),
            ("Old Cut", "left_library"), ("Low Orbit", "started"), ("Home Video", "on_server"), ("Sintel", "started"), ("Big Buck Bunny", "watched"),
        ].map(|(a, b)| (a.to_string(), b.to_string())), "newest first; somebody else's request is not even hinted at");
        let list = listing(&c, "alice", false).unwrap();
        let get = |title: &str| list.iter().find(|e| e["title"] == title).unwrap().clone();
        assert_eq!(get("Low Orbit")["progress"], json!({ "seen": 2, "total": 4 }), "files only: the virtual episode is not one to watch");
        assert_eq!(get("Winterline")["request"], json!({ "by_you": true }));
        assert!(get("Nightfall Bay")["next"]["day"].is_string() && get("Nightfall Bay")["next"]["season"] == 1);
        assert_eq!(get("Nightfall Bay")["poster"], json!({ "service_id": 1, "media_id": 12 }), "a title not in the library has Sonarr's poster");
        assert_eq!(get("Winterline")["poster"], json!({ "service_id": 2, "media_id": 31 }), "…or Radarr's, from the request");
        assert_eq!(get("Glass Harbour")["poster"], Value::Null, "not from somebody else's request either");
        assert_eq!((get("Sintel")["item_id"].as_str(), get("Old Cut")["item_id"].as_str()), (Some("sintel"), None), "a link only to what is there");
    }

    #[test]
    fn who_else_asked_is_said_only_to_whoever_may_see_everyone() {
        let c = household();
        let list = listing(&c, "alice", true).unwrap();
        let glass = list.iter().find(|e| e["title"] == "Glass Harbour").unwrap();
        assert_eq!((glass["state"].as_str(), &glass["request"]), (Some("requested"), &json!({ "by_you": false, "user_name": "bob" })));
    }

    #[test]
    fn a_show_is_watched_when_every_episode_on_disk_is() {
        let c = household();
        c.execute_batch(
            "INSERT INTO manual_seen VALUES ('alice', 'e3', 0);
             INSERT INTO user_items(user_id, item_id, played) VALUES ('alice', 'e4', 1);",
        )
        .unwrap();
        let list = listing(&c, "alice", false).unwrap();
        let show = list.iter().find(|e| e["title"] == "Low Orbit").unwrap();
        assert_eq!((show["state"].as_str(), &show["progress"]), (Some("watched"), &json!({ "seen": 4, "total": 4 })));
    }

    #[test]
    fn a_list_is_its_owner_s_alone() {
        let c = household();
        let bobs = listing(&c, "bob", true).unwrap();
        assert_eq!(bobs.len(), 1);
        assert_eq!((bobs[0]["title"].as_str(), bobs[0]["state"].as_str()), (Some("Sintel"), Some("on_server")), "alice's plays are not bob's");
    }
    fn item_of(c: &Connection, id: i64) -> Option<String> {
        c.query_row("SELECT item_id FROM watchlist WHERE id = ?1", [id], |r| r.get(0)).unwrap()
    }

    #[test]
    fn an_entry_finds_its_title_when_the_library_brings_it() {
        let c = conn();
        let (id, _) = add(&c, "alice", Wanted::Title(film(Some("990001"), None, "Winterline")), 1).unwrap().unwrap();
        assert_eq!(resolve(&c).unwrap(), 0, "nothing to find yet");
        c.execute_batch(r#"INSERT INTO items(id, type, name, production_year, provider_ids, updated_at) VALUES ('win', 'Movie', 'Winterline', 2026, '{"Tmdb":"990001"}', 0);"#).unwrap();
        crate::pipeline::rebuild_external(&c).unwrap();
        assert_eq!(resolve(&c).unwrap(), 1);
        assert_eq!(item_of(&c, id).as_deref(), Some("win"));
        assert_eq!(resolve(&c).unwrap(), 0, "and once attached, it stays put");
    }

    #[test]
    fn a_renamed_title_takes_its_entries_along_and_a_deleted_one_leaves_them_be() {
        let c = conn();
        let (id, _) = add(&c, "alice", Wanted::Item("sintel".into()), 1).unwrap().unwrap();
        // Renamed on disk: Jellyfin makes a new item, the old one is removed.
        c.execute_batch(r#"UPDATE items SET removed = 1 WHERE id = 'sintel';
                           INSERT INTO items(id, type, name, provider_ids, updated_at) VALUES ('sintel2', 'Movie', 'Sintel', '{"Tmdb":"45745"}', 0);"#).unwrap();
        crate::pipeline::rebuild_external(&c).unwrap();
        assert_eq!(resolve(&c).unwrap(), 1);
        assert_eq!(item_of(&c, id).as_deref(), Some("sintel2"));
        // Deleted for good: nothing else has its ids, so it stays where it was — and reads as having left.
        c.execute_batch("UPDATE items SET removed = 1 WHERE id = 'sintel2';").unwrap();
        crate::pipeline::rebuild_external(&c).unwrap();
        assert_eq!(resolve(&c).unwrap(), 0);
        assert_eq!(item_of(&c, id).as_deref(), Some("sintel2"));
    }

    #[test]
    fn ids_that_point_at_two_titles_attach_to_neither() {
        let c = conn();
        c.execute("INSERT INTO watchlist(user_id, kind, tmdb_id, imdb_id, title, added_at) VALUES ('alice', 'Movie', '10378', 'tt1727587', 'Muddled', 1)", []).unwrap();
        assert_eq!(resolve(&c).unwrap(), 0);
        assert_eq!(item_of(&c, c.last_insert_rowid()), None);
    }

    #[test]
    fn two_entries_that_become_one_title_are_merged_keeping_the_older() {
        let c = conn();
        // Added from the library while the film had no ids yet…
        let (newer, _) = add(&c, "alice", Wanted::Item("plain".into()), 50).unwrap().unwrap();
        // …and earlier, from the calendar, by an id the library did not know then.
        c.execute("INSERT INTO watchlist(user_id, kind, tmdb_id, title, added_at) VALUES ('alice', 'Movie', '777', 'Home Video', 10)", []).unwrap();
        let older = c.last_insert_rowid();
        c.execute_batch(r#"UPDATE items SET provider_ids = '{"Tmdb":"777"}' WHERE id = 'plain';"#).unwrap();
        crate::pipeline::rebuild_external(&c).unwrap();
        resolve(&c).unwrap();
        let left: Vec<(i64, Option<String>, i64)> = c.prepare("SELECT id, item_id, added_at FROM watchlist WHERE user_id = 'alice'").unwrap()
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))).unwrap().map(Result::unwrap).collect();
        assert_eq!(left, [(older, Some("plain".to_string()), 10)], "one entry, the older, now attached");
        assert!(!left.iter().any(|(id, _, _)| *id == newer));
    }

    /// Three entries of one title — one on its item, two waiting by ids the others did not share — are one entry after a
    /// library read. Merging only the oldest twin moved a waiting entry onto the item another entry still held, the
    /// unique index refused it, and with it every library read, import and restore that came after.
    #[test]
    fn three_entries_that_become_one_title_are_merged_into_the_oldest() {
        let c = conn();
        c.execute_batch(
            "INSERT INTO watchlist(user_id, kind, item_id, title, added_at) VALUES ('alice', 'Movie', 'sintel', 'Sintel', 30);
             INSERT INTO watchlist(user_id, kind, tmdb_id, title, added_at) VALUES ('alice', 'Movie', '45745', 'Sintel', 10);
             INSERT INTO watchlist(user_id, kind, imdb_id, title, added_at) VALUES ('alice', 'Movie', 'tt1727587', 'Sintel', 20);",
        )
        .unwrap();
        resolve(&c).unwrap();
        let left: Vec<(Option<String>, i64)> = c.prepare("SELECT item_id, added_at FROM watchlist WHERE user_id = 'alice'").unwrap()
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?))).unwrap().map(Result::unwrap).collect();
        assert_eq!(left, [(Some("sintel".to_string()), 10)], "one entry, the oldest, on the title");
    }

    #[test]
    fn an_attached_entry_keeps_up_with_what_the_library_calls_its_title() {
        let c = conn();
        let (id, _) = add(&c, "alice", Wanted::Item("plain".into()), 1).unwrap().unwrap();
        c.execute_batch(r#"UPDATE items SET name = 'Home Video (Restored)', production_year = 2020, provider_ids = '{"Imdb":"tt0000777"}' WHERE id = 'plain';"#).unwrap();
        resolve(&c).unwrap();
        let (_, _, t) = row(&c, id);
        assert_eq!((t.title.as_str(), t.year, t.imdb_id.as_deref()), ("Home Video (Restored)", Some(2020), Some("tt0000777")), "so it still reads right once the title has gone");
    }

    #[test]
    fn a_library_read_attaches_entries() {
        let c = conn();
        let (id, _) = add(&c, "alice", Wanted::Title(film(Some("990001"), None, "Winterline")), 1).unwrap().unwrap();
        c.execute_batch(r#"INSERT INTO items(id, type, name, provider_ids, updated_at) VALUES ('win', 'Movie', 'Winterline', '{"Tmdb":"990001"}', 0);"#).unwrap();
        crate::sync::backfill_playbacks(&c).unwrap();
        assert_eq!(item_of(&c, id).as_deref(), Some("win"));
    }

    #[test]
    fn looking_for_what_to_attach_reads_by_index() {
        let c = conn();
        let plan = |sql: &str, args: &[&dyn crate::db::rusqlite::ToSql]| -> String {
            c.prepare(&format!("EXPLAIN QUERY PLAN {sql}")).unwrap().query_map(args, |r| r.get::<_, String>(3)).unwrap().map(Result::unwrap).collect::<Vec<_>>().join(" | ")
        };
        let p = plan(WAITING_SQL, &[]);
        assert!(p.contains("SEARCH i USING PRIMARY KEY") && !p.contains("SCAN i"), "{p}");
        let p = plan(FIND_SQL, &[&"Movie", &"1", &"2", &"tt3"]);
        assert!(p.contains("idx_item_external") && !p.contains("SCAN x") && !p.contains("SCAN item_external"), "{p}");
    }
    /// alice's on-the-server film, its file upgraded. A new file is a new path, and so a new item with a fresh DateCreated,
    /// which once read as "Now on the server" for a title that never left — at every upgrade, watched or not.
    #[test]
    fn a_title_that_never_left_is_not_an_arrival_when_its_file_is_replaced() {
        use crate::notify::{Kind, tests::{bus, target}};
        let c = conn();
        let now = db::now();
        c.execute_batch(r#"INSERT INTO users(id, name, updated_at) VALUES ('alice', 'alice', 0);
               INSERT INTO user_permissions(user_id, permissions, updated_at) VALUES ('alice', '["sign_in"]', 0);"#).unwrap();
        c.execute("UPDATE items SET date_created = ?1 WHERE id = 'sintel'", [now - 365 * 86_400]).unwrap();
        add(&c, "alice", Wanted::Item("sintel".into()), now - 7 * 86_400).unwrap().unwrap();
        resolve(&c).unwrap();
        // One library read later: the old file is gone and the upgrade is there under a new id.
        c.execute("UPDATE items SET removed = 1 WHERE id = 'sintel'", []).unwrap();
        c.execute(r#"INSERT INTO items(id, type, name, provider_ids, date_created, updated_at) VALUES ('sintel2', 'Movie', 'Sintel', '{"Tmdb":"45745","Imdb":"tt1727587"}', ?1, 0)"#, [now - 600]).unwrap();
        crate::pipeline::rebuild_external(&c).unwrap();
        resolve(&c).unwrap();
        let f = bus(&c, vec![target(1, Some("alice"), &[Kind::WatchlistAvailable])]);
        assert_eq!(announce(&c, &f).unwrap(), 0, "it never left");
    }

    /// The other side of the rule: a title that really left — missing at a library read — and came back is an arrival.
    #[test]
    fn a_title_that_left_and_came_back_is_an_arrival() {
        use crate::notify::{Kind, tests::{bus, target}};
        let c = conn();
        let now = db::now();
        c.execute_batch(r#"INSERT INTO users(id, name, updated_at) VALUES ('alice', 'alice', 0);
               INSERT INTO user_permissions(user_id, permissions, updated_at) VALUES ('alice', '["sign_in"]', 0);"#).unwrap();
        c.execute("UPDATE items SET date_created = ?1 WHERE id = 'sintel'", [now - 365 * 86_400]).unwrap();
        add(&c, "alice", Wanted::Item("sintel".into()), now - 7 * 86_400).unwrap().unwrap();
        c.execute("UPDATE items SET removed = 1 WHERE id = 'sintel'", []).unwrap();
        resolve(&c).unwrap();
        c.execute(r#"INSERT INTO items(id, type, name, provider_ids, date_created, updated_at) VALUES ('sintel2', 'Movie', 'Sintel', '{"Tmdb":"45745"}', ?1, 0)"#, [now - 600]).unwrap();
        crate::pipeline::rebuild_external(&c).unwrap();
        resolve(&c).unwrap();
        let f = bus(&c, vec![target(1, Some("alice"), &[Kind::WatchlistAvailable])]);
        assert_eq!(announce(&c, &f).unwrap(), 1);
    }

    /// Two entries found to be one title, one of them on it all along: the title was here, so nothing arrived.
    #[test]
    fn merging_an_entry_into_one_that_had_its_title_is_not_an_arrival() {
        use crate::notify::{Kind, tests::{bus, target}};
        let c = conn();
        let now = db::now();
        c.execute_batch(r#"INSERT INTO users(id, name, updated_at) VALUES ('alice', 'alice', 0);
               INSERT INTO user_permissions(user_id, permissions, updated_at) VALUES ('alice', '["sign_in"]', 0);"#).unwrap();
        c.execute("UPDATE items SET date_created = ?1 WHERE id = 'plain'", [now - 600]).unwrap();
        add(&c, "alice", Wanted::Item("plain".into()), now - 300).unwrap().unwrap();
        add(&c, "alice", Wanted::Title(film(Some("777"), None, "Home Video")), now - 3_600).unwrap().unwrap();
        resolve(&c).unwrap();
        c.execute_batch(r#"UPDATE items SET provider_ids = '{"Tmdb":"777"}' WHERE id = 'plain';"#).unwrap();
        crate::pipeline::rebuild_external(&c).unwrap();
        resolve(&c).unwrap();
        let f = bus(&c, vec![target(1, Some("alice"), &[Kind::WatchlistAvailable])]);
        assert_eq!(announce(&c, &f).unwrap(), 0);
    }

    /// Entries from before the rule learn where they stand from the library as it is, so nothing already here is an arrival.
    #[test]
    fn the_migration_marks_what_is_missing_from_the_library_as_it_is() {
        let c = Connection::open_in_memory().unwrap();
        let at = crate::db::MIGRATIONS.iter().position(|m| m.contains("ADD COLUMN missing")).expect("the watchlist's missing column");
        for m in &crate::db::MIGRATIONS[..at] {
            c.execute_batch(m).unwrap();
        }
        c.execute_batch(
            "INSERT INTO items(id, type, name, removed, updated_at) VALUES ('here', 'Movie', 'Here', 0, 0), ('gone', 'Movie', 'Gone', 1, 0);
             INSERT INTO watchlist(user_id, kind, item_id, tmdb_id, title, added_at) VALUES
               ('alice', 'Movie', 'here', NULL, 'Here', 1), ('alice', 'Movie', 'gone', NULL, 'Gone', 1), ('alice', 'Movie', NULL, '1', 'Waiting', 1);",
        )
        .unwrap();
        c.execute_batch(crate::db::MIGRATIONS[at]).unwrap();
        let missing: Vec<(String, bool)> = c.prepare("SELECT title, missing FROM watchlist ORDER BY id").unwrap()
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?))).unwrap().map(Result::unwrap).collect();
        assert_eq!(missing, [("Here".into(), false), ("Gone".into(), true), ("Waiting".into(), true)]);
    }

    #[test]
    fn an_arrival_is_told_once_to_whoever_was_waiting_for_it() {
        use crate::notify::{Kind, tests::{bus, target}};
        let c = conn();
        let now = db::now();
        c.execute_batch(
            r#"INSERT INTO users(id, name, updated_at) VALUES ('alice', 'alice', 0), ('bob', 'bob', 0), ('carol', 'carol', 0);
               INSERT INTO user_permissions(user_id, permissions, updated_at) VALUES ('alice', '["sign_in"]', 0), ('bob', '["sign_in"]', 0), ('carol', '["sign_in"]', 0);"#,
        )
        .unwrap();
        // alice put it on her list a day before it came; bob added it from its page once it was here; carol waited for a
        // title that came two years ago, before this install had anything to tell.
        c.execute("INSERT INTO watchlist(user_id, kind, tmdb_id, title, added_at) VALUES ('alice', 'Movie', '990001', 'Winterline', ?1)", [now - 86_400]).unwrap();
        c.execute(r#"INSERT INTO items(id, type, name, provider_ids, date_created, updated_at) VALUES ('win', 'Movie', 'Winterline', '{"Tmdb":"990001"}', ?1, 0)"#, [now - 600]).unwrap();
        c.execute("UPDATE items SET date_created = ?1 WHERE id = 'sintel'", [now - 2 * 365 * 86_400]).unwrap();
        c.execute("INSERT INTO watchlist(user_id, kind, tmdb_id, title, added_at) VALUES ('carol', 'Movie', '45745', 'Sintel', ?1)", [now - 3 * 365 * 86_400]).unwrap();
        crate::pipeline::rebuild_external(&c).unwrap();
        resolve(&c).unwrap();
        add(&c, "bob", Wanted::Item("win".into()), now).unwrap().unwrap();
        let all = [Kind::WatchlistAvailable];
        let f = bus(&c, vec![target(1, Some("alice"), &all), target(2, Some("bob"), &all), target(3, Some("carol"), &all)]);
        assert_eq!(announce(&c, &f).unwrap(), 1);
        assert_eq!(announce(&c, &f).unwrap(), 0, "told once");
        let told: Vec<(String, String, Option<String>)> = c.prepare("SELECT user_id, title, link FROM notify_events").unwrap()
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))).unwrap().map(Result::unwrap).collect();
        assert_eq!(told, [("alice".to_string(), "Now on the server: Winterline".to_string(), Some("/items/win".to_string()))]);
        let queued: Vec<i64> = c.prepare("SELECT target_id FROM notify_deliveries").unwrap().query_map([], |r| r.get(0)).unwrap().map(Result::unwrap).collect();
        assert_eq!(queued, [1], "to alice's destination and nobody else's");
    }
}
