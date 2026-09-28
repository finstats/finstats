//! Read-only statistics endpoints. Everything here goes through [`Scope`], which is where
//! the time window, user/library filters and the "non-admins only see themselves" rule live.

use anyhow::Result;
use axum::Json;
use axum::extract::{Path, Query, State};
use chrono::{Datelike, NaiveDate};
use serde::Deserialize;
use serde_json::{Map, Value, json};

use crate::auth::{AuthUser, Manager, ServerViewer};
use crate::db::rusqlite::types::ValueRef;
use crate::db::rusqlite::{Connection, OptionalExtension, Row, params_from_iter};
use crate::db::{self, SqlValue};
use crate::media;
use crate::state::{ApiError, ApiResult, App};

const BOOL_COLS: [&str; 11] = ["active", "is_admin", "is_disabled", "removed", "has_image", "item_exists", "has_backdrop", "is_local", "is_favorite", "is_actor", "is_director"];
const JSON_COLS: [&str; 7] = ["genres", "transcode", "studios", "provider_ids", "audio_languages", "subtitle_languages", "clients"];

// ---------------------------------------------------------------- plumbing

pub fn row_json(row: &Row) -> Map<String, Value> {
    let stmt = row.as_ref();
    let mut out = Map::new();
    for i in 0..stmt.column_count() {
        let name = stmt.column_name(i).unwrap_or("?").to_string();
        let v = match row.get_ref(i).unwrap_or(ValueRef::Null) {
            ValueRef::Null => Value::Null,
            ValueRef::Integer(n) if BOOL_COLS.contains(&name.as_str()) => Value::Bool(n != 0),
            ValueRef::Integer(n) => json!(n),
            ValueRef::Real(f) => json!(f),
            ValueRef::Text(t) => {
                let s = String::from_utf8_lossy(t);
                if JSON_COLS.contains(&name.as_str()) { serde_json::from_str(&s).unwrap_or(Value::Null) } else { Value::String(s.into_owned()) }
            }
            ValueRef::Blob(_) => Value::Null,
        };
        out.insert(name, v);
    }
    out
}

/// A list's page number from the address: 1 when absent, and never so large that `(page - 1) * per_page`
/// overflows — past the end is an empty page, never the first one again.
pub(crate) fn page_number(page: Option<i64>) -> i64 {
    page.unwrap_or(1).clamp(1, 100_000)
}

pub(crate) fn rows_json(conn: &Connection, sql: &str, args: &[SqlValue]) -> Result<Vec<Map<String, Value>>> {
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map(params_from_iter(args.iter()), |r| Ok(row_json(r)))?;
    Ok(rows.collect::<Result<_, _>>()?)
}

pub(crate) fn one_json(conn: &Connection, sql: &str, args: &[SqlValue]) -> Result<Option<Map<String, Value>>> {
    Ok(conn.query_row(sql, params_from_iter(args.iter()), |r| Ok(row_json(r))).optional()?)
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct FilterQuery {
    pub days: Option<i64>,
    pub user_id: Option<String>,
    pub library_id: Option<String>,
}

/// A WHERE clause under construction over `playbacks p`.
#[derive(Clone, Default)]
pub struct Cond {
    clauses: Vec<String>,
    pub(crate) args: Vec<SqlValue>,
}

impl Cond {
    pub fn add(&mut self, clause: &str, v: impl Into<SqlValue>) -> &mut Self {
        self.clauses.push(clause.to_string());
        self.args.push(v.into());
        self
    }
    /// `col IN (?, ?, …)`, or nothing at all when no value was named — which is how "all of them"
    /// is spelt throughout: a filter naming nothing is not a filter.
    pub fn add_in(&mut self, col: &str, vals: &[String]) -> &mut Self {
        if vals.is_empty() {
            return self;
        }
        let holes = std::iter::repeat_n("?", vals.len()).collect::<Vec<_>>().join(", ");
        self.clauses.push(format!("{col} IN ({holes})"));
        for v in vals {
            self.args.push(v.clone().into());
        }
        self
    }
    pub fn raw(&mut self, clause: &str) -> &mut Self {
        self.clauses.push(clause.to_string());
        self
    }
    pub fn with(&self, clause: &str, v: impl Into<SqlValue>) -> Self {
        let mut c = self.clone();
        c.add(clause, v);
        c
    }
    pub fn with_raw(&self, clause: &str) -> Self {
        let mut c = self.clone();
        c.raw(clause);
        c
    }
    pub fn sql(&self) -> String {
        if self.clauses.is_empty() { String::new() } else { format!("WHERE {}", self.clauses.join(" AND ")) }
    }
}

/// The resolved filter for one request.
#[derive(Clone)]
pub struct Scope {
    pub days: i64,
    /// Start of the window (local midnight `days-1` days ago); `None` = all time.
    pub since: Option<i64>,
    /// Whose plays are counted. Empty = everybody's. Never widened past what the caller may see.
    pub user_ids: Vec<String>,
    pub library_id: Option<String>,
    pub min_play_s: i64,
    /// What the caller may see; decides which fields survive and whose plays are counted.
    pub perms: crate::auth::Perms,
}

/// A filter that may name several values at once: `type=Movie,Episode`. Naming nothing is not a
/// filter, which is how "all of them" is spelt. Capped, so that no caller can make the SQL
/// arbitrarily long, and de-duplicated, so that repeating a value cannot pad it either.
pub(crate) fn many(v: Option<&str>) -> Vec<String> {
    const MOST: usize = 50;
    let mut out: Vec<String> = vec![];
    for part in v.unwrap_or_default().split(',').map(str::trim).filter(|s| !s.is_empty()) {
        if !out.iter().any(|x| x == part) {
            out.push(part.to_string());
        }
        if out.len() == MOST {
            break;
        }
    }
    out
}

/// Whose rows a request may be about — the one place this rule is written down. Empty means
/// everybody. With "see everyone", the people the URL names; without it, always exactly the caller,
/// whoever the URL names and however many: the filter can narrow what somebody sees and never widen it.
pub fn pinned_users(user: &AuthUser, asked: &[String]) -> Vec<String> {
    if !user.perms.see_everyone {
        return vec![user.id.clone()];
    }
    let mut out: Vec<String> = vec![];
    for id in asked.iter().map(|s| db::norm_id(s)).filter(|s| !s.is_empty()) {
        if !out.contains(&id) {
            out.push(id);
        }
    }
    out
}

/// The same rule where exactly one person is the subject (their requests, their places). `None`
/// means everybody. Several named, and the first is taken — those endpoints are about one person.
pub fn pinned_user(user: &AuthUser, asked: Option<&str>) -> Option<String> {
    pinned_users(user, &many(asked)).into_iter().next()
}

impl Scope {
    pub fn new(app: &App, user: &AuthUser, q: &FilterQuery) -> Self {
        let clean = |s: &Option<String>| s.as_deref().map(db::norm_id).filter(|s| !s.is_empty());
        Scope {
            days: q.days.unwrap_or(0).clamp(0, 36_500),
            since: None,
            // Without "see everyone" a request is pinned to the caller, whatever the URL asks for.
            user_ids: pinned_users(user, &many(q.user_id.as_deref())),
            library_id: clean(&q.library_id),
            min_play_s: app.settings().min_play_s,
            perms: user.perms,
        }
    }

    /// "Last 7 days" means today plus the six full local days before it, so the chart's
    /// buckets and the totals always describe exactly the same plays.
    pub fn resolve(mut self, conn: &Connection) -> Result<Self> {
        if self.days > 0 {
            let since: i64 = conn.query_row(
                "SELECT CAST(strftime('%s', date('now', 'localtime', ?1), 'utc') AS INTEGER)",
                [format!("-{} days", self.days - 1)],
                |r| r.get(0),
            )?;
            self.since = Some(since);
        }
        Ok(self)
    }

    fn base(&self) -> Cond {
        let mut c = Cond::default();
        c.add_in("p.user_id", &self.user_ids);
        if let Some(l) = &self.library_id {
            c.add("p.library_id = ?", l.clone());
        }
        if self.min_play_s > 0 {
            c.add("p.duration_s >= ?", self.min_play_s);
        }
        c
    }

    /// Whose rows the caller may see at all: the permission pin on its own, with no window and no
    /// other filter. For questions about a history rather than about one view of it.
    pub fn whose(&self) -> Cond {
        let mut c = Cond::default();
        c.add_in("p.user_id", &self.user_ids);
        c
    }

    pub fn cond(&self) -> Cond {
        let mut c = self.base();
        if let Some(s) = self.since {
            c.add("p.started_at >= ?", s);
        }
        c
    }

    /// The same window and pin, but plays of any length: for the one question where the minimum play
    /// length would hide the answer — a file nobody ever gets thirty seconds into.
    pub fn cond_any_length(&self) -> Cond {
        let mut c = Cond::default();
        c.add_in("p.user_id", &self.user_ids);
        if let Some(l) = &self.library_id {
            c.add("p.library_id = ?", l.clone());
        }
        if let Some(s) = self.since {
            c.add("p.started_at >= ?", s);
        }
        c
    }

    /// The window of equal length right before the current one.
    pub(crate) fn previous_cond(&self) -> Option<Cond> {
        let since = self.since?;
        let mut c = self.base();
        c.add("p.started_at >= ?", since - self.days * 86_400);
        c.add("p.started_at < ?", since);
        Some(c)
    }
}

async fn scoped<T, F>(app: &App, user: &AuthUser, q: &FilterQuery, f: F) -> ApiResult<T>
where
    T: Send + 'static,
    F: FnOnce(&Connection, &Scope) -> Result<T> + Send + 'static,
{
    let scope = Scope::new(app, user, q);
    Ok(app
        .db
        .call(move |c| {
            let scope = scope.resolve(c)?;
            f(c, &scope)
        })
        .await?)
}

// ---------------------------------------------------------------- building blocks

pub(crate) fn totals(conn: &Connection, cond: &Cond) -> Result<Value> {
    let sql = format!(
        "SELECT COUNT(*) AS plays, COALESCE(SUM(p.duration_s), 0) AS watch_s,
                COUNT(DISTINCT p.user_id) AS active_users, COUNT(DISTINCT p.item_id) AS distinct_items
         FROM playbacks p {}",
        cond.sql()
    );
    Ok(Value::Object(one_json(conn, &sql, &cond.args)?.unwrap_or_default()))
}

const TYPE_GROUPS: [&str; 4] = ["Movie", "Episode", "Audio", "Other"];

/// Gap-free time series. Day buckets, or ISO weeks once the span passes 120 days.
pub(crate) fn daily(conn: &Connection, scope: &Scope, cond: &Cond) -> Result<(Vec<Value>, &'static str)> {
    let today: String = conn.query_row("SELECT date('now', 'localtime')", [], |r| r.get(0))?;
    let today = NaiveDate::parse_from_str(&today, "%Y-%m-%d")?;
    let first: Option<String> = match scope.since {
        Some(s) => conn.query_row("SELECT date(?1, 'unixepoch', 'localtime')", [s], |r| r.get(0))?,
        None => conn.query_row(
            &format!("SELECT date(MIN(p.started_at), 'unixepoch', 'localtime') FROM playbacks p {}", cond.sql()),
            params_from_iter(cond.args.iter()),
            |r| r.get(0),
        )?,
    };
    let Some(first) = first.and_then(|f| NaiveDate::parse_from_str(&f, "%Y-%m-%d").ok()) else {
        return Ok((vec![], "day"));
    };
    let weekly = (today - first).num_days() > 120;
    let (bucket_sql, step, name) = if weekly {
        ("date(p.started_at, 'unixepoch', 'localtime', 'weekday 0', '-6 days')", 7, "week")
    } else {
        ("date(p.started_at, 'unixepoch', 'localtime')", 1, "day")
    };
    let sql = format!(
        "SELECT {bucket_sql} AS d,
                CASE p.item_type WHEN 'Movie' THEN 'Movie' WHEN 'Episode' THEN 'Episode' WHEN 'Audio' THEN 'Audio' ELSE 'Other' END AS g,
                COUNT(*), COALESCE(SUM(p.duration_s), 0)
         FROM playbacks p {} GROUP BY d, g",
        cond.sql()
    );
    let mut found: std::collections::HashMap<(String, String), (i64, i64)> = Default::default();
    let mut stmt = conn.prepare(&sql)?;
    let mut rows = stmt.query(params_from_iter(cond.args.iter()))?;
    while let Some(r) = rows.next()? {
        found.insert((r.get(0)?, r.get(1)?), (r.get(2)?, r.get(3)?));
    }

    let mut cursor = if weekly { first - chrono::Duration::days(first.weekday().num_days_from_monday() as i64) } else { first };
    let mut out = vec![];
    while cursor <= today {
        let date = cursor.format("%Y-%m-%d").to_string();
        let mut by_type = Map::new();
        let (mut plays, mut watch) = (0, 0);
        for g in TYPE_GROUPS {
            let (p, w) = found.get(&(date.clone(), g.to_string())).copied().unwrap_or((0, 0));
            plays += p;
            watch += w;
            by_type.insert(g.to_string(), json!([p, w]));
        }
        out.push(json!({ "date": date, "plays": plays, "watch_s": watch, "by_type": by_type }));
        cursor += chrono::Duration::days(step);
    }
    Ok((out, name))
}

pub(crate) fn heatmap(conn: &Connection, cond: &Cond) -> Result<Value> {
    let sql = format!(
        "SELECT CAST(strftime('%w', p.started_at, 'unixepoch', 'localtime') AS INTEGER),
                CAST(strftime('%H', p.started_at, 'unixepoch', 'localtime') AS INTEGER),
                COUNT(*), COALESCE(SUM(p.duration_s), 0)
         FROM playbacks p {} GROUP BY 1, 2",
        cond.sql()
    );
    let mut plays = vec![vec![0i64; 24]; 7];
    let mut watch = vec![vec![0i64; 24]; 7];
    let mut stmt = conn.prepare(&sql)?;
    let mut rows = stmt.query(params_from_iter(cond.args.iter()))?;
    while let Some(r) = rows.next()? {
        let (dow, hour): (i64, i64) = (r.get(0)?, r.get(1)?);
        let day = ((dow + 6) % 7) as usize; // SQLite: 0 = Sunday. Ours: 0 = Monday.
        plays[day][hour.clamp(0, 23) as usize] = r.get(2)?;
        watch[day][hour.clamp(0, 23) as usize] = r.get(3)?;
    }
    Ok(json!({ "plays": plays, "watch_s": watch }))
}

/// `expr` is grouped on; rows beyond `max` are folded into "Other".
fn buckets(conn: &Connection, cond: &Cond, expr: &str, from_extra: &str, max: usize) -> Result<Vec<Value>> {
    let sql = format!(
        "SELECT {expr} AS name, COUNT(*) AS plays, COALESCE(SUM(p.duration_s), 0) AS watch_s
         FROM playbacks p {from_extra} {} GROUP BY 1 HAVING name IS NOT NULL ORDER BY plays DESC, watch_s DESC",
        cond.sql()
    );
    Ok(top_and_other(rows_json(conn, &sql, &cond.args)?, max))
}

/// The first `max` buckets as they are, and the rest summed into one called "Other".
fn top_and_other(rows: Vec<Map<String, Value>>, max: usize) -> Vec<Value> {
    let mut out: Vec<Value> = vec![];
    let (mut other_p, mut other_w) = (0i64, 0i64);
    for (i, r) in rows.into_iter().enumerate() {
        if i < max {
            out.push(Value::Object(r));
        } else {
            other_p += r["plays"].as_i64().unwrap_or(0);
            other_w += r["watch_s"].as_i64().unwrap_or(0);
        }
    }
    if other_p > 0 {
        out.push(json!({ "name": "Other", "plays": other_p, "watch_s": other_w }));
    }
    out
}

const RESOLUTION_SQL: &str = "CASE
    WHEN p.width IS NULL AND p.height IS NULL THEN NULL
    WHEN p.width >= 3800 OR p.height >= 2000 THEN '4K'
    WHEN p.width >= 2500 OR p.height >= 1400 THEN '1440p'
    WHEN p.width >= 1900 OR p.height >= 1000 THEN '1080p'
    WHEN p.width >= 1260 OR p.height >= 700 THEN '720p'
    WHEN p.width >= 1000 OR p.height >= 560 THEN '576p'
    WHEN p.width >= 700 OR p.height >= 400 THEN '480p'
    ELSE 'SD' END";

const CHANNELS_SQL: &str = "CASE p.audio_channels WHEN 0 THEN NULL WHEN 1 THEN 'Mono' WHEN 2 THEN 'Stereo'
    WHEN 6 THEN '5.1' WHEN 8 THEN '7.1' ELSE p.audio_channels || ' ch' END";

pub(crate) fn top(conn: &Connection, cond: &Cond, kind: &str, limit: i64, by_plays: bool) -> Result<Vec<Value>> {
    let order = if by_plays { "plays DESC, watch_s DESC" } else { "watch_s DESC, plays DESC" };
    let agg = "COUNT(*) AS plays, COALESCE(SUM(p.duration_s), 0) AS watch_s, COUNT(DISTINCT p.user_id) AS users, MAX(p.ended_at) AS last_played";
    let (sql, cond) = match kind {
        "movies" | "music" => {
            let ty = if kind == "movies" { "Movie" } else { "Audio" };
            let sub = if kind == "movies" { "CAST(i.production_year AS TEXT)" } else { "COALESCE(i.album_artist, i.album)" };
            let c = cond.with("p.item_type = ?", ty.to_string());
            (
                format!(
                    "SELECT p.item_id AS id, COALESCE(i.name, MAX(p.item_name)) AS name, {sub} AS sub, p.item_id AS image_item_id, {agg}
                     FROM playbacks p LEFT JOIN items i ON i.id = p.item_id {} GROUP BY p.item_id ORDER BY {order} LIMIT {limit}",
                    c.sql()
                ),
                c,
            )
        }
        "series" => {
            let c = cond.with_raw("p.item_type = 'Episode'");
            (
                format!(
                    "SELECT p.series_id AS id, COALESCE(i.name, MAX(p.series_name), 'Unknown series') AS name,
                            CAST(i.production_year AS TEXT) AS sub, p.series_id AS image_item_id, {agg}
                     FROM playbacks p LEFT JOIN items i ON i.id = p.series_id {}
                     GROUP BY COALESCE(p.series_id, p.series_name) ORDER BY {order} LIMIT {limit}",
                    c.sql()
                ),
                c,
            )
        }
        "users" => (
            format!(
                "SELECT p.user_id AS id, COALESCE(u.name, MAX(p.user_name)) AS name, NULL AS sub, NULL AS image_item_id,
                        COUNT(*) AS plays, COALESCE(SUM(p.duration_s), 0) AS watch_s, MAX(p.ended_at) AS last_played
                 FROM playbacks p LEFT JOIN users u ON u.id = p.user_id {} GROUP BY p.user_id ORDER BY {order} LIMIT {limit}",
                cond.sql()
            ),
            cond.clone(),
        ),
        "clients" | "devices" => {
            let col = if kind == "clients" { "p.client" } else { "p.device_name" };
            let c = cond.with_raw(&format!("{col} IS NOT NULL"));
            (
                format!(
                    "SELECT NULL AS id, {col} AS name, NULL AS sub, NULL AS image_item_id, {agg}
                     FROM playbacks p {} GROUP BY {col} ORDER BY {order} LIMIT {limit}",
                    c.sql()
                ),
                c,
            )
        }
        "libraries" => {
            let c = cond.with_raw("p.library_id IS NOT NULL");
            (
                format!(
                    "SELECT p.library_id AS id, COALESCE(l.name, 'Removed library') AS name, l.collection_type AS sub, p.library_id AS image_item_id, {agg}
                     FROM playbacks p LEFT JOIN libraries l ON l.id = p.library_id {} GROUP BY p.library_id ORDER BY {order} LIMIT {limit}",
                    c.sql()
                ),
                c,
            )
        }
        // Whatever the library holds: series for shows, the item itself for everything else.
        "library_items" => (
            format!(
                "SELECT COALESCE(p.series_id, p.item_id) AS id,
                        COALESCE(i.name, MAX(COALESCE(p.series_name, p.item_name))) AS name,
                        COALESCE(CAST(i.production_year AS TEXT), i.album_artist) AS sub,
                        COALESCE(p.series_id, p.item_id) AS image_item_id, {agg}
                 FROM playbacks p LEFT JOIN items i ON i.id = COALESCE(p.series_id, p.item_id) {}
                 GROUP BY COALESCE(p.series_id, p.item_id) ORDER BY {order} LIMIT {limit}",
                cond.sql()
            ),
            cond.clone(),
        ),
        _ => anyhow::bail!("unknown kind"),
    };
    Ok(rows_json(conn, &sql, &cond.args)?.into_iter().map(Value::Object).collect())
}

// ---------------------------------------------------------------- plays

const PLAY_SELECT: &str = "SELECT p.id, p.source, p.active, p.user_id, COALESCE(u.name, p.user_name) AS user_name,
    p.item_id, p.item_name, p.item_type, p.series_id, p.series_name, p.season_number, p.episode_number,
    CASE WHEN p.item_type = 'Episode' AND p.series_id IS NOT NULL THEN p.series_id ELSE p.item_id END AS image_item_id,
    (i.id IS NOT NULL AND i.removed = 0) AS item_exists,
    p.started_at, p.ended_at, p.duration_s, p.paused_s, p.position_s,
    COALESCE(p.runtime_s, i.runtime_s) AS runtime_s,
    p.client, p.device_name, p.device_id, p.app_version, p.remote_ip, p.play_method, p.container, p.bitrate,
    p.video_codec, p.width, p.height, p.video_range, p.bit_depth,
    p.audio_codec, p.audio_channels, p.audio_language, p.subtitle_codec, p.subtitle_language, p.transcode,
    p.pause_count, p.seek_count, p.start_position_s, p.is_local, p.group_id,
    CASE WHEN p.group_id IS NULL THEN NULL ELSE (SELECT COUNT(DISTINCT g.user_id) FROM playbacks g WHERE g.group_id = p.group_id) END AS group_size
  FROM playbacks p
  LEFT JOIN items i ON i.id = p.item_id
  LEFT JOIN users u ON u.id = p.user_id";

const DETAIL_ONLY: [&str; 12] = [
    "device_id", "bitrate", "video_codec", "width", "height", "video_range", "bit_depth", "audio_codec", "audio_channels",
    "audio_language", "subtitle_codec", "subtitle_language",
];

fn decorate_play(mut m: Map<String, Value>, see_network: bool, detail: bool) -> Value {
    let text = |m: &Map<String, Value>, k: &str| m.get(k).and_then(Value::as_str).map(str::to_string);
    let num = |m: &Map<String, Value>, k: &str| m.get(k).and_then(Value::as_i64);

    let video = media::video_label(text(&m, "video_codec").as_deref(), num(&m, "width"), num(&m, "height"), text(&m, "video_range").as_deref());
    let audio = media::audio_label(text(&m, "audio_codec").as_deref(), num(&m, "audio_channels"), text(&m, "audio_language").as_deref());
    let subtitle = media::subtitle_label(if detail { text(&m, "subtitle_codec") } else { None }.as_deref(), text(&m, "subtitle_language").as_deref());

    // Live rows know where playback stopped. Imported rows only know how long it ran.
    let completion = match (num(&m, "runtime_s").filter(|r| *r > 0), num(&m, "position_s"), num(&m, "duration_s")) {
        (Some(rt), Some(pos), _) => Some((pos as f64 / rt as f64).clamp(0.0, 1.0)),
        (Some(rt), None, Some(d)) => Some((d as f64 / rt as f64).clamp(0.0, 1.0)),
        _ => None,
    };
    m.insert("video".into(), json!(video));
    m.insert("audio".into(), json!(audio));
    m.insert("subtitle".into(), json!(subtitle));
    m.insert("completion".into(), json!(completion.map(|c| (c * 1000.0).round() / 1000.0)));

    if !see_network {
        m.insert("remote_ip".into(), Value::Null);
        m.insert("device_id".into(), Value::Null);
        m.insert("is_local".into(), Value::Null);
    }
    if !detail {
        m.remove("start_position_s");
        for k in DETAIL_ONLY {
            m.remove(k);
        }
        m.remove("transcode");
    }
    Value::Object(m)
}

#[derive(Deserialize, Default)]
pub struct ActivityQuery {
    // Not `#[serde(flatten)]`: flattening makes serde_urlencoded hand numbers over as strings.
    days: Option<i64>,
    user_id: Option<String>,
    library_id: Option<String>,
    page: Option<i64>,
    per_page: Option<i64>,
    q: Option<String>,
    method: Option<String>,
    #[serde(rename = "type")]
    item_type: Option<String>,
    item_id: Option<String>,
    series_id: Option<String>,
    source: Option<String>,
    sort: Option<String>,
    dir: Option<String>,
}

/// `ORDER BY` for a paginated list: a whitelisted column, empty values last whichever way it runs,
/// and a fixed tiebreaker so pages never shuffle. Unknown keys fall back to the default order.
/// The words of a `q` filter, each one ready for LIKE: cut to a length a word can plausibly be, then
/// escaped. The cut is part of building the pattern rather than a check somewhere above it, because
/// SQLite refuses a LIKE pattern longer than `SQLITE_MAX_LIKE_PATTERN_LENGTH` (50 000) with an error —
/// so `?q=<50 000 letters>` was a 500 and a line in the log, from any signed-in caller. Escaping comes
/// after the cut: the other way round, a trim could leave half of an escape pair behind.
pub(crate) fn like_words(q: Option<&str>) -> Vec<String> {
    const WORDS: usize = 8;
    const LONGEST: usize = 100;
    q.unwrap_or_default()
        .split_whitespace()
        .take(WORDS)
        .map(|w| {
            let w: String = w.chars().take(LONGEST).collect();
            format!("%{}%", w.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_"))
        })
        .collect()
}

pub(crate) fn order_by(columns: &[(&str, &str)], sort: Option<&str>, dir: Option<&str>, default: &str) -> String {
    let Some((_, expr)) = sort.and_then(|k| columns.iter().find(|(key, _)| *key == k)) else { return default.to_string() };
    let dir = if dir == Some("asc") { "ASC" } else { "DESC" };
    format!("({expr}) IS NULL, {expr} {dir}, {default}")
}

/// The trackers a play can have come from: the live collector, or an import from one of the two
/// other trackers. In the order the Activity filter offers them.
pub(crate) const SOURCES: [&str; 3] = ["live", "jellystat", "streamystats"];

/// The `source` filter, chosen from [`SOURCES`] rather than passed through — the value reaches SQL.
/// Anything else, an empty value included, means all of them, which is what the filter shows when
/// nobody has picked one.
pub(crate) fn source_filter(asked: Option<&str>) -> Option<&'static str> {
    let asked = asked?;
    SOURCES.into_iter().find(|s| *s == asked)
}

/// Which trackers this history came from, in [`SOURCES`] order so the filter never reshuffles.
///
/// `whose` is the caller's permission scope and nothing else — no window, no library, no search. It
/// answers what somebody's history is *made of* rather than what the view in front of them happens
/// to contain, so the filter does not appear and disappear as they change the days. The scope still
/// applies: a person who may only see their own plays must not learn that somebody else's history
/// was imported from somewhere.
pub(crate) fn sources_present(conn: &Connection, whose: &Cond) -> Result<Vec<&'static str>> {
    let mut out = vec![];
    for source in SOURCES {
        let cond = whose.with("p.source = ?", source.to_string());
        let sql = format!("SELECT EXISTS(SELECT 1 FROM playbacks p {})", cond.sql());
        if conn.query_row(&sql, params_from_iter(cond.args.iter()), |r| r.get::<_, bool>(0))? {
            out.push(source);
        }
    }
    Ok(out)
}

/// The three kinds finstats charts by name; anything else is "Other".
const CHARTED_TYPES: [&str; 3] = ["Movie", "Episode", "Audio"];

/// The media-type filter as a clause and the values to bind, or `None` when it asks for everything
/// — nothing named, or all four, both of which are "no filter".
///
/// "Other" is not a type but the absence of the three named ones, so it cannot join them in an `IN`
/// list: asking for films *and* other has to mean either.
fn type_clause(types: &[String]) -> Option<(String, Vec<String>)> {
    if types.is_empty() {
        return None;
    }
    let other = types.iter().any(|t| t == "Other");
    let named: Vec<String> = types.iter().filter(|t| CHARTED_TYPES.contains(&t.as_str())).cloned().collect();
    if other && named.len() == CHARTED_TYPES.len() {
        return None;
    }
    let mut ors: Vec<String> = vec![];
    if !named.is_empty() {
        let holes = std::iter::repeat_n("?", named.len()).collect::<Vec<_>>().join(", ");
        ors.push(format!("p.item_type IN ({holes})"));
    }
    if other {
        let names = CHARTED_TYPES.map(|t| format!("'{t}'")).join(", ");
        ors.push(format!("p.item_type NOT IN ({names})"));
    }
    // Only names finstats does not chart: an IN of exactly those, nothing clever.
    if ors.is_empty() {
        let holes = std::iter::repeat_n("?", types.len()).collect::<Vec<_>>().join(", ");
        return Some((format!("(p.item_type IN ({holes}))"), types.to_vec()));
    }
    Some((format!("({})", ors.join(" OR ")), named))
}

const ACTIVITY_SORTS: [(&str, &str); 8] = [
    ("when", "p.ended_at"),
    ("user", "COALESCE(u.name, p.user_name) COLLATE NOCASE"),
    ("title", "COALESCE(p.series_name, p.item_name) COLLATE NOCASE"),
    ("watched", "p.duration_s"),
    // The same rule as the `completion` field: where it stopped, or for imported plays how long it ran.
    ("progress", "MIN(1.0, COALESCE(p.position_s, p.duration_s) * 1.0 / NULLIF(COALESCE(p.runtime_s, i.runtime_s), 0))"),
    ("client", "p.client COLLATE NOCASE"),
    ("method", "p.play_method"),
    ("ip", "p.remote_ip"),
];

pub async fn activity(State(app): State<App>, user: AuthUser, Query(q): Query<ActivityQuery>) -> ApiResult {
    let page = page_number(q.page);
    let per_page = q.per_page.unwrap_or(50).clamp(1, 200);
    let filter = FilterQuery { days: q.days, user_id: q.user_id.clone(), library_id: q.library_id.clone() };
    let out = scoped(&app, &user, &filter, move |c, scope| {
        let mut cond = scope.cond();
        cond.add_in("p.play_method", &many(q.method.as_deref()));
        if let Some((sql, vals)) = type_clause(&many(q.item_type.as_deref())) {
            cond.clauses.push(sql);
            for v in vals {
                cond.args.push(v.into());
            }
        }
        if let Some(id) = q.item_id.filter(|s| !s.is_empty()) {
            cond.add("p.item_id = ?", db::norm_id(&id));
        }
        if let Some(id) = q.series_id.filter(|s| !s.is_empty()) {
            cond.add("p.series_id = ?", db::norm_id(&id));
        }
        let sources: Vec<String> = many(q.source.as_deref()).iter().filter_map(|s| source_filter(Some(s))).map(str::to_string).collect();
        cond.add_in("p.source", &sources);
        // Word by word: "alya opera" finds plays of Alya… on Opera. Every word must be somewhere in the row.
        for like in like_words(q.q.as_deref()) {
            let mut fields = vec!["p.item_name", "p.series_name", "p.user_name", "p.client", "p.device_name"];
            if scope.perms.see_network {
                fields.push("p.remote_ip");
            }
            let ors: Vec<String> = fields.iter().map(|f| format!("{f} LIKE ? ESCAPE '\\'")).collect();
            cond.clauses.push(format!("({})", ors.join(" OR ")));
            for _ in &fields {
                cond.args.push(like.clone().into());
            }
        }
        let total: i64 = c.query_row(&format!("SELECT COUNT(*) FROM playbacks p {}", cond.sql()), params_from_iter(cond.args.iter()), |r| r.get(0))?;
        // Sorting by address is only for those who are shown addresses.
        let sort = q.sort.as_deref().filter(|k| *k != "ip" || scope.perms.see_network);
        let order = order_by(&ACTIVITY_SORTS, sort, q.dir.as_deref(), "p.ended_at DESC, p.id DESC");
        let sql = format!("{PLAY_SELECT} {} ORDER BY {order} LIMIT {per_page} OFFSET {}", cond.sql(), (page - 1) * per_page);
        let rows: Vec<Value> = rows_json(c, &sql, &cond.args)?.into_iter().map(|m| decorate_play(m, scope.perms.see_network, false)).collect();
        // What this history is made of, so the page can offer a filter for it — and leave it out
        // when there is only one answer.
        let sources = sources_present(c, &scope.whose())?;
        Ok(json!({ "total": total, "page": page, "per_page": per_page, "rows": rows, "sources": sources }))
    })
    .await?;
    Ok(Json(out))
}

pub async fn activity_detail(State(app): State<App>, user: AuthUser, Path(id): Path<i64>) -> ApiResult {
    let found = scoped(&app, &user, &FilterQuery::default(), move |c, scope| {
        let mut cond = Cond::default();
        cond.add("p.id = ?", id);
        cond.add_in("p.user_id", &scope.user_ids);
        let Some(play) = one_json(c, &format!("{PLAY_SELECT} {}", cond.sql()), &cond.args)? else { return Ok(None) };
        let mut play = decorate_play(play, scope.perms.see_network, true);
        let events = rows_json(c, "SELECT at, kind, position_s, from_s, detail FROM playback_events WHERE playback_id = ?1 ORDER BY id", &[id.into()])?;
        play["events"] = json!(events);
        // The people it was watched with. Visible to anyone who can see this play: it was a shared evening.
        let with = match play["group_id"].as_i64() {
            Some(g) => rows_json(
                c,
                "SELECT DISTINCT p.user_id, COALESCE(u.name, p.user_name) AS user_name FROM playbacks p LEFT JOIN users u ON u.id = p.user_id
                 WHERE p.group_id = ?1 AND p.user_id <> ?2 ORDER BY 2",
                &[g.into(), play["user_id"].as_str().unwrap_or_default().to_string().into()],
            )?,
            None => vec![],
        };
        play["watched_with"] = json!(with);
        Ok(Some(play))
    })
    .await?;
    found.map(Json).ok_or_else(|| ApiError::not_found("Play"))
}

pub async fn activity_delete(State(app): State<App>, Manager(user): Manager, Path(id): Path<i64>) -> ApiResult {
    let actor = crate::audit::Actor::from(&user);
    let n = app
        .db
        .call(move |c| {
            let gone: Option<(String, String, i64)> = c
                .query_row("SELECT item_name, user_name, started_at FROM playbacks WHERE id = ?1 AND active = 0", [id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
                .optional()?;
            let n = c.execute("DELETE FROM playbacks WHERE id = ?1 AND active = 0", [id])?;
            if let Some((title, who, started_at)) = gone.filter(|_| n > 0) {
                crate::audit::record_quietly(c, &crate::audit::Entry::new("play_deleted", actor).target(id.to_string()).detail(json!({ "title": title, "user": who, "started_at": started_at })));
            }
            Ok(n)
        })
        .await?;
    if n == 0 {
        return Err(ApiError::not_found("Play"));
    }
    Ok(Json(json!({ "ok": true })))
}

// ---------------------------------------------------------------- stats endpoints

fn overview_library(c: &Connection, scope: &Scope) -> Result<Option<Map<String, Value>>> {
    let lib = match &scope.library_id {
        Some(l) => Cond::default().with("library_id = ?", l.clone()),
        None => Cond::default(),
    }
    .with_raw("removed = 0");
    one_json(
        c,
        &format!(
            "SELECT COALESCE(SUM(type = 'Movie'), 0) AS movies, COALESCE(SUM(type = 'Series'), 0) AS series,
                    COALESCE(SUM(type = 'Episode'), 0) AS episodes, COALESCE(SUM(type = 'Audio'), 0) AS tracks,
                    COALESCE(SUM(size_bytes), 0) AS size_bytes,
                    (SELECT COUNT(*) FROM users WHERE removed = 0) AS users
             FROM items {}",
            lib.sql()
        ),
        &lib.args,
    )
}

pub async fn overview(State(app): State<App>, user: AuthUser, Query(q): Query<FilterQuery>) -> ApiResult {
    // The totals and the day-by-day series each read every play in the window, and need nothing from each other.
    let scope = resolved(&app, &user, &q).await?;
    let (totals, (series, bucket), (previous, library)) = tokio::try_join!(
        apart(app.clone(), scope.clone(), |c, s| totals(c, &s.cond())),
        apart(app.clone(), scope.clone(), |c, s| daily(c, s, &s.cond())),
        apart(app.clone(), scope.clone(), |c, s| Ok((s.previous_cond().map(|p| totals(c, &p)).transpose()?, overview_library(c, s)?))),
    )?;
    Ok(Json(json!({ "totals": totals, "previous": previous, "daily": series, "bucket": bucket, "library": library })))
}

#[derive(Deserialize)]
pub struct TopQuery {
    // Not `#[serde(flatten)]`: flattening makes serde_urlencoded hand numbers over as strings.
    days: Option<i64>,
    user_id: Option<String>,
    library_id: Option<String>,
    kind: Option<String>,
    limit: Option<i64>,
    sort: Option<String>,
}

pub async fn top_handler(State(app): State<App>, user: AuthUser, Query(q): Query<TopQuery>) -> ApiResult {
    let kind = q.kind.clone().unwrap_or_else(|| "movies".into());
    if !["movies", "series", "music", "users", "clients", "devices", "libraries"].contains(&kind.as_str()) {
        return Err(ApiError::bad_request("Unknown kind"));
    }
    let limit = q.limit.unwrap_or(10).clamp(1, 100);
    let by_plays = q.sort.as_deref() == Some("plays");
    let filter = FilterQuery { days: q.days, user_id: q.user_id.clone(), library_id: q.library_id.clone() };
    let rows = scoped(&app, &user, &filter, move |c, scope| top(c, &scope.cond(), &kind, limit, by_plays)).await?;
    Ok(Json(json!({ "rows": rows })))
}

pub async fn heatmap_handler(State(app): State<App>, user: AuthUser, Query(q): Query<FilterQuery>) -> ApiResult {
    Ok(Json(scoped(&app, &user, &q, |c, scope| heatmap(c, &scope.cond())).await?))
}

pub async fn playback(State(app): State<App>, user: AuthUser, Query(q): Query<FilterQuery>) -> ApiResult {
    let out = scoped(&app, &user, &q, |c, scope| {
        let cond = scope.cond();
        let video = cond.with_raw("p.video_codec IS NOT NULL");
        let video_transcode = cond.with_raw("p.play_method = 'Transcode' AND json_extract(p.transcode, '$.is_video_direct') = 0");
        Ok(json!({
            "methods": buckets(c, &cond, "p.play_method", "", 12)?,
            "transcode_reasons": buckets(c, &cond.with_raw("p.transcode IS NOT NULL"), "j.value", ", json_each(p.transcode, '$.reasons') j", 12)?,
            "hw_accel": buckets(c, &video_transcode, "COALESCE(json_extract(p.transcode, '$.hw_accel'), 'Software')", "", 12)?,
            "video_codecs": buckets(c, &cond, "UPPER(p.video_codec)", "", 12)?,
            "audio_codecs": buckets(c, &cond, "UPPER(p.audio_codec)", "", 12)?,
            "resolutions": buckets(c, &cond, RESOLUTION_SQL, "", 12)?,
            "video_ranges": buckets(c, &video, "p.video_range", "", 12)?,
            "containers": buckets(c, &cond, "LOWER(p.container)", "", 12)?,
            "audio_channels": buckets(c, &cond, CHANNELS_SQL, "", 12)?,
            "clients": buckets(c, &cond, "p.client", "", 12)?,
            "subtitles": buckets(c, &video, "COALESCE(p.subtitle_language, 'None')", "", 12)?,
        }))
    })
    .await?;
    Ok(Json(out))
}

// ---------------------------------------------------------------- users

fn user_rows(conn: &Connection, scope: &Scope, only: Option<&str>) -> Result<Vec<Value>> {
    let cond = scope.cond();
    let mut args = cond.args.clone();
    let mut filter = String::new();
    if let Some(id) = only {
        filter = "WHERE u.id = ?".into();
        args.push(id.to_string().into());
    }
    let sql = format!(
        "SELECT u.id, u.name, u.is_admin, u.is_disabled, u.removed, (u.image_tag IS NOT NULL) AS has_image,
                u.last_login_at, u.last_activity_at,
                COALESCE(s.plays, 0) AS plays, COALESCE(s.watch_s, 0) AS watch_s, l.last_played_at,
                (SELECT CASE WHEN x.series_name IS NOT NULL AND x.item_type = 'Episode' THEN x.series_name || ' — ' || x.item_name ELSE x.item_name END
                   FROM playbacks x WHERE x.user_id = u.id ORDER BY x.ended_at DESC LIMIT 1) AS last_item_name,
                (SELECT x.client FROM playbacks x WHERE x.user_id = u.id ORDER BY x.ended_at DESC LIMIT 1) AS last_client
         FROM users u
         LEFT JOIN (SELECT p.user_id, COUNT(*) AS plays, SUM(p.duration_s) AS watch_s FROM playbacks p {} GROUP BY p.user_id) s ON s.user_id = u.id
         LEFT JOIN (SELECT user_id, MAX(ended_at) AS last_played_at FROM playbacks GROUP BY user_id) l ON l.user_id = u.id
         {filter}
         ORDER BY watch_s DESC, u.name COLLATE NOCASE",
        cond.sql()
    );
    Ok(rows_json(conn, &sql, &args)?.into_iter().map(Value::Object).collect())
}

pub async fn users(State(app): State<App>, user: AuthUser, Query(q): Query<FilterQuery>) -> ApiResult {
    // The list is about everyone: a user filter from the URL would only blank out the others.
    let q = FilterQuery { user_id: None, ..q };
    let me = (!user.perms.see_everyone).then(|| user.id.clone());
    let rows = scoped(&app, &user, &q, move |c, scope| {
        let everyone = Scope { user_ids: vec![], ..scope.clone() };
        user_rows(c, &everyone, me.as_deref())
    })
    .await?;
    Ok(Json(json!({ "users": rows })))
}

/// The devices somebody played on. The id is `see_network`'s, as in every play (`decorate_play`).
fn user_devices(c: &Connection, cond: &Cond, see_network: bool) -> Result<Vec<Map<String, Value>>> {
    let mut rows = rows_json(
        c,
        &format!(
            "SELECT p.device_id, MAX(p.device_name) AS device_name, MAX(p.client) AS client, MAX(p.app_version) AS app_version,
                    COUNT(*) AS plays, MAX(p.ended_at) AS last_seen
             FROM playbacks p {} GROUP BY COALESCE(p.device_id, p.device_name) ORDER BY last_seen DESC LIMIT 50",
            cond.sql()
        ),
        &cond.args,
    )?;
    if !see_network {
        for d in &mut rows {
            d.insert("device_id".into(), Value::Null);
        }
    }
    Ok(rows)
}

pub async fn user_detail(State(app): State<App>, user: AuthUser, Path(id): Path<String>, Query(q): Query<FilterQuery>) -> ApiResult {
    let id = db::norm_id(&id);
    if !user.perms.see_everyone && user.id != id {
        return Err(ApiError::not_permitted("see other people's statistics"));
    }
    let q = FilterQuery { user_id: Some(id.clone()), ..q };
    // Own page or permitted: let the scope follow the id in the URL. Everything else keeps the caller's permissions.
    let as_viewer = AuthUser { perms: crate::auth::Perms { see_everyone: true, ..user.perms }, ..user.clone() };
    let see_network = user.perms.see_network;
    let out = scoped(&app, &as_viewer, &q, move |c, scope| {
        let list_scope = Scope { user_ids: vec![], ..scope.clone() };
        let Some(u) = user_rows(c, &list_scope, Some(&id))?.into_iter().next() else { return Ok(None) };
        let cond = scope.cond();
        let mut t = totals(c, &cond)?;
        let kinds = one_json(
            c,
            &format!(
                "SELECT COALESCE(SUM(p.item_type = 'Movie'), 0) AS movies, COALESCE(SUM(p.item_type = 'Episode'), 0) AS episodes,
                        COALESCE(SUM(p.item_type = 'Audio'), 0) AS tracks FROM playbacks p {}",
                cond.sql()
            ),
            &cond.args,
        )?
        .unwrap_or_default();
        if let Some(obj) = t.as_object_mut() {
            obj.extend(kinds);
            obj.remove("active_users");
        }
        let (series, bucket) = daily(c, scope, &cond)?;
        let devices = user_devices(c, &cond, see_network)?;
        let ips: Vec<Value> = if see_network {
            let ipc = cond.with_raw("p.remote_ip IS NOT NULL");
            rows_json(
                c,
                &format!(
                    "SELECT p.remote_ip AS ip, COUNT(*) AS plays, MIN(p.started_at) AS first_seen, MAX(p.ended_at) AS last_seen
                     FROM playbacks p {} GROUP BY p.remote_ip ORDER BY last_seen DESC LIMIT 100",
                    ipc.sql()
                ),
                &ipc.args,
            )?
            .into_iter()
            .map(|mut m| {
                let local = m.get("ip").and_then(Value::as_str).and_then(|ip| crate::network::classify(c, ip).ok().flatten()).unwrap_or(false);
                m.insert("is_local".into(), json!(local));
                Value::Object(m)
            })
            .collect()
        } else {
            vec![]
        };
        Ok(Some(json!({
            "user": u, "totals": t, "daily": series, "bucket": bucket,
            "heatmap": heatmap(c, &cond)?,
            "top_series": top(c, &cond, "series", 8, false)?,
            "top_movies": top(c, &cond, "movies", 8, false)?,
            "clients": buckets(c, &cond, "p.client", "", 8)?,
            "methods": buckets(c, &cond, "p.play_method", "", 8)?,
            "genres": genre_buckets(c, &cond)?,
            "jellyfin": jellyfin_flags(c, &id)?,
            // Streaks are about the whole history, whatever time range the page is showing.
            "streaks": crate::profile::streaks(c, &id, scope.min_play_s)?,
            "devices": devices, "ips": ips,
        })))
    })
    .await?;
    out.map(Json).ok_or_else(|| ApiError::not_found("User"))
}

// ---------------------------------------------------------------- libraries & items

fn library_rows(conn: &Connection, scope: &Scope, only: Option<&str>) -> Result<Vec<Value>> {
    let cond = scope.cond();
    let mut args = cond.args.clone();
    // A library that was deleted in Jellyfin and never had a single play is just noise in the list.
    // (It stays reachable by id, and one with history stays listed, marked as removed.)
    let mut filter = "WHERE (l.removed = 0 OR EXISTS (SELECT 1 FROM playbacks x WHERE x.library_id = l.id))".to_string();
    if let Some(id) = only {
        filter = "WHERE l.id = ?".into();
        args.push(id.to_string().into());
    }
    let sql = format!(
        "SELECT l.id, l.name, l.collection_type, l.removed,
                COALESCE(c.item_count, 0) AS item_count, COALESCE(c.series_count, 0) AS series_count,
                COALESCE(c.episode_count, 0) AS episode_count, COALESCE(c.size_bytes, 0) AS size_bytes,
                COALESCE(s.plays, 0) AS plays, COALESCE(s.watch_s, 0) AS watch_s, s.last_played_at
         FROM libraries l
         LEFT JOIN (SELECT library_id,
                           SUM(type IN ('Movie', 'Series', 'Audio', 'MusicVideo', 'Video', 'Book', 'AudioBook')) AS item_count,
                           SUM(type = 'Series') AS series_count, SUM(type = 'Episode') AS episode_count, SUM(size_bytes) AS size_bytes
                    FROM items WHERE removed = 0 GROUP BY library_id) c ON c.library_id = l.id
         LEFT JOIN (SELECT p.library_id, COUNT(*) AS plays, SUM(p.duration_s) AS watch_s, MAX(p.ended_at) AS last_played_at
                    FROM playbacks p {} GROUP BY p.library_id) s ON s.library_id = l.id
         {filter}
         ORDER BY l.removed, watch_s DESC, l.name COLLATE NOCASE",
        cond.sql()
    );
    Ok(rows_json(conn, &sql, &args)?.into_iter().map(Value::Object).collect())
}

const ITEM_CARD: &str = "SELECT i.id, i.name, i.type, i.production_year AS year,
    COALESCE(i.album_artist, i.series_name, CAST(i.production_year AS TEXT)) AS sub, i.id AS image_item_id, i.date_created FROM items i";

pub async fn libraries(State(app): State<App>, user: AuthUser, Query(q): Query<FilterQuery>) -> ApiResult {
    let q = FilterQuery { library_id: None, ..q };
    let rows = scoped(&app, &user, &q, |c, scope| library_rows(c, scope, None)).await?;
    Ok(Json(json!({ "libraries": rows })))
}

pub async fn library_detail(State(app): State<App>, user: AuthUser, Path(id): Path<String>, Query(q): Query<FilterQuery>) -> ApiResult {
    let id = db::norm_id(&id);
    let q = FilterQuery { library_id: Some(id.clone()), ..q };
    let out = scoped(&app, &user, &q, move |c, scope| {
        let list_scope = Scope { library_id: None, ..scope.clone() };
        let Some(lib) = library_rows(c, &list_scope, Some(&id))?.into_iter().next() else { return Ok(None) };
        let cond = scope.cond();
        let (series, bucket) = daily(c, scope, &cond)?;
        let recent = rows_json(
            c,
            &format!("{ITEM_CARD} WHERE i.library_id = ?1 AND i.removed = 0 AND i.type IN ('Movie', 'Series', 'MusicAlbum', 'Video', 'MusicVideo', 'Book', 'AudioBook') ORDER BY i.date_created DESC LIMIT 18"),
            &[id.clone().into()],
        )?;
        Ok(Some(json!({
            "library": lib, "top": top(c, &cond, "library_items", 10, false)?,
            "recently_added": recent, "daily": series, "bucket": bucket,
        })))
    })
    .await?;
    out.map(Json).ok_or_else(|| ApiError::not_found("Library"))
}

pub async fn item_detail(State(app): State<App>, user: AuthUser, Path(id): Path<String>, Query(q): Query<FilterQuery>) -> ApiResult {
    let id = db::norm_id(&id);
    let (caller, min_play) = (user.id.clone(), app.settings().min_play_s.max(120));
    // Where people open Jellyfin: the address an administrator gave, else the one finstats connects to.
    let jellyfin_base = Some(app.settings().jellyfin_public_url.trim().to_string())
        .filter(|u| !u.is_empty())
        .or_else(|| app.config.read().unwrap().as_ref().map(|c| c.url.clone()));
    let out = scoped(&app, &user, &q, move |c, scope| {
        let mut requested: Option<Value> = None;
        let item = one_json(
            c,
            "SELECT i.id, i.name, i.type, i.production_year AS year, i.overview, i.genres, i.community_rating, i.official_rating,
                    i.runtime_s, i.premiere_date, i.date_created, i.library_id, l.name AS library_name, i.removed,
                    i.series_id, i.series_name, i.parent_index_number AS season_number, i.index_number AS episode_number,
                    i.album, i.album_artist, i.container, i.size_bytes, i.bitrate, i.path,
                    i.video_codec, i.width, i.height, i.video_range, i.audio_codec, i.audio_channels,
                    i.bit_depth, i.framerate, i.studios, i.provider_ids, i.audio_languages, i.subtitle_languages,
                    (i.image_tag IS NOT NULL) AS has_image, (i.backdrop_tag IS NOT NULL) AS has_backdrop
             FROM items i LEFT JOIN libraries l ON l.id = i.library_id WHERE i.id = ?1",
            &[id.clone().into()],
        )?;
        // Deleted before finstats ever saw it, but it still has history: describe it from its plays.
        let item = match item {
            Some(i) => Some(i),
            None => one_json(
                c,
                "SELECT ?1 AS id, name, type, 1 AS removed, 0 AS has_backdrop FROM (
                    SELECT item_name AS name, item_type AS type, ended_at FROM playbacks WHERE item_id = ?1
                    UNION ALL
                    SELECT series_name, 'Series', ended_at FROM playbacks WHERE series_id = ?1
                 ) ORDER BY ended_at DESC LIMIT 1",
                &[id.clone().into()],
            )?,
        };
        let Some(mut item) = item else { return Ok(None) };
        let text = |m: &Map<String, Value>, k: &str| m.get(k).and_then(Value::as_str).map(str::to_string);
        let num = |m: &Map<String, Value>, k: &str| m.get(k).and_then(Value::as_i64);
        let video = media::video_label(text(&item, "video_codec").as_deref(), num(&item, "width"), num(&item, "height"), text(&item, "video_range").as_deref());
        let audio = media::audio_label(text(&item, "audio_codec").as_deref(), num(&item, "audio_channels"), None);
        item.insert("video".into(), json!(video));
        item.insert("audio".into(), json!(audio));
        for k in ["video_codec", "width", "height", "video_range", "audio_codec", "audio_channels"] {
            item.remove(k);
        }
        let provider_ids = item.remove("provider_ids");
        let external = external_links(item.get("type").and_then(Value::as_str), provider_ids);
        item.insert("external".into(), json!(external));
        // A title Jellyfin no longer has has no page there to open.
        let open = item.get("removed").and_then(Value::as_bool) != Some(true);
        item.insert("jellyfin_link".into(), json!(jellyfin_base.as_deref().filter(|_| open).map(|b| crate::jellyfin::web_link(b, &id))));
        if item.get("studios").is_none_or(Value::is_null) {
            item.insert("studios".into(), json!([]));
        }
        if !scope.perms.see_server {
            item.insert("path".into(), Value::Null);
        }

        let is_series = item.get("type").and_then(Value::as_str) == Some("Series");
        let cond = scope.cond().with(if is_series { "p.series_id = ?" } else { "p.item_id = ?" }, id.clone());
        let totals = one_json(
            c,
            &format!(
                "SELECT COUNT(*) AS plays, COALESCE(SUM(p.duration_s), 0) AS watch_s, COUNT(DISTINCT p.user_id) AS users,
                        MAX(p.ended_at) AS last_played_at FROM playbacks p {}",
                cond.sql()
            ),
            &cond.args,
        )?;
        let watchers = rows_json(
            c,
            &format!(
                "SELECT p.user_id, COALESCE(u.name, MAX(p.user_name)) AS user_name, COUNT(*) AS plays,
                        COALESCE(SUM(p.duration_s), 0) AS watch_s, MAX(p.ended_at) AS last_played_at
                 FROM playbacks p LEFT JOIN users u ON u.id = p.user_id {} GROUP BY p.user_id ORDER BY watch_s DESC LIMIT 50",
                cond.sql()
            ),
            &cond.args,
        )?;

        // Where plays of a film or an episode stopped; a show's episodes each have their own page for it.
        let insights = if matches!(item.get("type").and_then(Value::as_str), Some("Movie") | Some("Episode")) {
            item_insights(c, &cond, item.get("runtime_s").and_then(Value::as_i64))?
        } else {
            Value::Null
        };

        let seasons = if is_series { series_seasons(c, &cond, &id, item.get("removed").and_then(Value::as_bool).unwrap_or(false))? } else { vec![] };
        // A show or a season has no tracks of its own: say how many of its episodes have each language,
        // which is what tells a complete dub from one that stops after season one.
        let item_type = item.get("type").and_then(Value::as_str).unwrap_or("").to_string();
        let item_type = item_type.as_str();
        if matches!(item_type, "Series" | "Season") {
            let parent = if item_type == "Series" { "e.series_id" } else { "e.season_id" };
            let files = format!("FROM items e WHERE {parent} = ?1 AND e.type = 'Episode' AND e.removed = 0 AND e.size_bytes IS NOT NULL");
            let total: i64 = c.query_row(&format!("SELECT COUNT(*) {files}"), [&id], |r| r.get(0))?;
            if total > 0 {
                let per = |col: &str| -> Result<Vec<Value>> {
                    let sql = format!("SELECT j.value AS code, COUNT(*) AS episodes FROM items e, json_each(e.{col}) j WHERE {parent} = ?1 AND e.type = 'Episode' AND e.removed = 0 AND e.size_bytes IS NOT NULL GROUP BY 1 ORDER BY 2 DESC, MIN(j.key), 1");
                    Ok(rows_json(c, &sql, &[id.clone().into()])?.into_iter().map(Value::Object).collect())
                };
                item.insert("language_coverage".into(), json!({ "episodes": total, "audio": per("audio_languages")?, "subtitles": per("subtitle_languages")? }));
            }
        }
        // Who asked for it, when it arrived: only for the caller's own request, or for someone who may see everyone.
        if matches!(item_type, "Series" | "Movie")
            && let Some(mut request) = crate::pipeline::request_for_item(c, &id, scope.perms.see_everyone, &caller, min_play)?
        {
            requested = Some(std::mem::take(&mut request));
        }
        // What Sonarr or Radarr expect next for this title. About the title, not about people.
        if matches!(item_type, "Series" | "Movie") {
            let next = crate::pipeline::upcoming_for_item(c, &id)?;
            if !next.is_empty() {
                item.insert("upcoming".into(), json!(next));
            }
        }
        let (series, bucket) = daily(c, scope, &cond)?;
        // Jellyfin's own flags. For a series: anyone who has finished at least one episode.
        let mut pb_args: Vec<SqlValue> = vec![id.clone().into()];
        // Without "see everyone" the scope is exactly the caller, so there is one id to pin to.
        let only_me = match scope.user_ids.first() {
            Some(u) if !scope.perms.see_everyone => {
                pb_args.push(u.clone().into());
                "AND ui.user_id = ?2"
            }
            _ => "",
        };
        let played_by = rows_json(
            c,
            &format!(
                "SELECT ui.user_id, COALESCE(u.name, 'Unknown') AS user_name, MAX(ui.last_played_at) AS last_played_at,
                        MAX(ui.is_favorite) AS is_favorite
                 FROM user_items ui LEFT JOIN users u ON u.id = ui.user_id
                 WHERE (ui.item_id = ?1 OR ui.item_id IN (SELECT id FROM items WHERE series_id = ?1)) {only_me}
                 GROUP BY ui.user_id ORDER BY last_played_at DESC"
            ),
            &pb_args,
        )?;
        // Cast and crew are kept per film and show; an episode or season answers with its show's.
        let credited = item.get("series_id").and_then(Value::as_str).filter(|_| !is_series).map(str::to_string).unwrap_or_else(|| id.clone());
        let people = rows_json(
            c,
            "SELECT person_id AS id, name, kind, role, has_image FROM item_people WHERE item_id = ?1 ORDER BY (kind = 'Actor'), sort",
            &[credited.into()],
        )?;
        Ok(Some(json!({ "item": item, "request": requested, "totals": totals, "watchers": watchers, "insights": insights, "seasons": seasons, "daily": series, "bucket": bucket, "played_by": played_by, "people": people })))
    })
    .await?;
    let mut out = out.ok_or_else(|| ApiError::not_found("Item"))?;
    // How far along it is lives in memory, not in the database.
    if !out["request"].is_null() {
        let mut request = out["request"].take();
        crate::downloads::attach_progress(&app, &mut request);
        out["item"]["request"] = request;
    }
    if let Some(o) = out.as_object_mut() {
        o.remove("request");
    }
    Ok(Json(out))
}

// ---------------------------------------------------------------- v1.8: playback insights
//
// Where a title loses its viewers, drawn from where each play stopped. A stop is `position_s` for a
// play finstats recorded itself and `duration_s` for an imported one (a tracker keeps how long it ran,
// not where it was; the play is taken to have started at 0:00), and the two are never mixed silently:
// every curve carries how many stops were measured and how many estimated.

/// A curve is only a shape once there is more than an anecdote to draw it from.
pub(crate) const MIN_CURVE_PLAYS: i64 = 3;
/// The widest a bucket gets, in seconds; a curve has at most sixty of them.
const BUCKET_LADDER: [i64; 6] = [30, 60, 120, 300, 600, 900];

/// The bucket, in seconds, that keeps a runtime to sixty points or fewer: half a minute for a short
/// episode, two minutes for a film, and whole minutes beyond the ladder for anything longer.
fn bucket_width(runtime_s: i64) -> i64 {
    BUCKET_LADDER.iter().copied().find(|w| runtime_s <= w * 60).unwrap_or_else(|| (runtime_s + 3599) / 3600 * 60)
}

/// How many buckets a runtime takes at this width.
fn bucket_count(runtime_s: i64, bucket_s: i64) -> usize {
    ((runtime_s.max(1) + bucket_s - 1) / bucket_s).max(1) as usize
}

/// The share of plays still going at each bucket edge, from the start (everyone) to the runtime. A
/// stop past the runtime counts as finished; one before the first edge is gone by it.
fn retention_curve(stops: &[i64], runtime_s: i64, bucket_s: i64) -> Vec<f64> {
    let n = bucket_count(runtime_s, bucket_s);
    (0..=n)
        .map(|k| {
            let edge = (k as i64 * bucket_s).min(runtime_s);
            if stops.is_empty() { 0.0 } else { ((stops.iter().filter(|&&s| s >= edge).count() as f64 / stops.len() as f64) * 1000.0).round() / 1000.0 }
        })
        .collect()
}

/// How many of the positions fall in each bucket; the end, and anything past it, belongs to the last.
fn histogram(positions: &[i64], runtime_s: i64, bucket_s: i64) -> Vec<i64> {
    let n = bucket_count(runtime_s, bucket_s);
    let mut out = vec![0; n];
    for &p in positions {
        let i = ((p.max(0) / bucket_s) as usize).min(n - 1);
        out[i] += 1;
    }
    out
}

/// The shape of where plays of one film or episode stopped, or `null` when there is no runtime to draw
/// it on or fewer than [`MIN_CURVE_PLAYS`] plays in scope. `cond` already names the item and the caller's scope.
fn item_insights(c: &Connection, cond: &Cond, runtime_s: Option<i64>) -> Result<Value> {
    let ended = cond.with_raw("p.active = 0");
    let runtime = match runtime_s.filter(|r| *r > 0) {
        Some(r) => r,
        None => c
            .query_row(&format!("SELECT MAX(p.runtime_s) FROM playbacks p {}", ended.sql()), params_from_iter(ended.args.iter()), |r| r.get::<_, Option<i64>>(0))?
            .unwrap_or(0),
    };
    if runtime <= 0 {
        return Ok(Value::Null);
    }
    let mut stmt = c.prepare(&format!(
        "SELECT MAX(0, MIN(?, CASE WHEN p.source = 'live' AND p.position_s IS NOT NULL THEN p.position_s ELSE COALESCE(p.duration_s, 0) END)) AS stop_s,
                (p.source = 'live' AND p.position_s IS NOT NULL) AS measured
         FROM playbacks p {}",
        ended.sql()
    ))?;
    let mut args: Vec<SqlValue> = vec![runtime.into()];
    args.extend(ended.args.iter().cloned());
    let rows: Vec<(i64, bool)> = stmt.query_map(params_from_iter(args.iter()), |r| Ok((r.get(0)?, r.get(1)?)))?.collect::<Result<_, _>>()?;
    if (rows.len() as i64) < MIN_CURVE_PLAYS {
        return Ok(Value::Null);
    }
    let stops: Vec<i64> = rows.iter().map(|r| r.0).collect();
    let measured = rows.iter().filter(|r| r.1).count();
    let bucket_s = bucket_width(runtime);
    // Events exist only for plays finstats recorded itself, so these two are counted over those. A
    // rewind is a seek that landed before where it left from, counted where it landed; a switch-on
    // is a play's first subtitle change when it is to a track rather than to "Off" — the state before
    // the first change is never an event, so a later language change is not a second switch-on.
    let positions = |sql: &str| -> Result<Vec<i64>> {
        let mut stmt = c.prepare(&format!("SELECT e.position_s FROM playbacks p JOIN playback_events e ON e.playback_id = p.id {} AND {sql}", ended.sql()))?;
        let out = stmt.query_map(params_from_iter(ended.args.iter()), |r| r.get::<_, Option<i64>>(0))?.filter_map(Result::transpose).collect::<Result<_, _>>()?;
        Ok(out)
    };
    let rewinds = positions("e.kind = 'seek' AND e.from_s IS NOT NULL AND e.position_s IS NOT NULL AND e.from_s > e.position_s")?;
    let subtitles = positions(
        "e.kind = 'subtitle' AND e.detail IS NOT NULL AND e.detail <> 'Off' AND e.position_s IS NOT NULL
         AND e.id = (SELECT MIN(o.id) FROM playback_events o WHERE o.playback_id = p.id AND o.kind = 'subtitle')",
    )?;
    Ok(json!({
        "runtime_s": runtime,
        "bucket_s": bucket_s,
        "plays": rows.len(),
        "measured": measured,
        "estimated": rows.len() - measured,
        "curve": retention_curve(&stops, runtime, bucket_s),
        "rewinds": histogram(&rewinds, runtime, bucket_s),
        "subtitles": histogram(&subtitles, runtime, bucket_s),
    }))
}

/// A show's episodes, season by season, each with its plays in scope, who started it and how many of
/// those plays finished (stopped at 90 % of the runtime or later). "Everyone quits episode three" is
/// people who never press play on episode four, which is what `users` in episode order shows.
fn series_seasons(c: &Connection, cond: &Cond, series_id: &str, series_removed: bool) -> Result<Vec<Value>> {
    let mut args = cond.args.clone();
    args.push(series_id.to_string().into());
    let eps = rows_json(
        c,
        &format!(
            "SELECT e.id, e.name, e.index_number AS episode_number, e.parent_index_number AS season_number, e.season_id,
                    COALESCE(sn.name, 'Season ' || COALESCE(e.parent_index_number, '?')) AS season_name,
                    e.runtime_s, e.audio_languages, COALESCE(s.plays, 0) AS plays, COALESCE(s.watch_s, 0) AS watch_s,
                    COALESCE(s.users, 0) AS users, COALESCE(s.finished, 0) AS finished
             FROM items e
             LEFT JOIN items sn ON sn.id = e.season_id
             LEFT JOIN (SELECT p.item_id, COUNT(*) AS plays, SUM(p.duration_s) AS watch_s, COUNT(DISTINCT p.user_id) AS users,
                               SUM(COALESCE(p.runtime_s, 0) > 0 AND CASE WHEN p.source = 'live' AND p.position_s IS NOT NULL THEN p.position_s ELSE p.duration_s END >= 0.9 * p.runtime_s) AS finished
                        FROM playbacks p {} GROUP BY p.item_id) s ON s.item_id = e.id
             WHERE e.series_id = ? AND e.type = 'Episode' AND (e.removed = 0 OR {series_removed})
             ORDER BY COALESCE(e.parent_index_number, 9999), COALESCE(e.index_number, 9999), e.name",
            cond.sql(),
            series_removed = if series_removed { 1 } else { 0 },
        ),
        &args,
    )?;
    let mut seasons: Vec<Value> = vec![];
    for mut e in eps {
        let key = e.get("season_id").cloned().unwrap_or(Value::Null);
        let season_number = e.get("season_number").cloned().unwrap_or(Value::Null);
        let season_name = e.remove("season_name").unwrap_or(Value::Null);
        e.remove("season_id");
        e.remove("season_number");
        let same = seasons.last().is_some_and(|s: &Value| s["id"] == key && s["season_number"] == season_number);
        if !same {
            seasons.push(json!({ "id": key, "name": season_name, "season_number": season_number, "episodes": [] }));
        }
        if let Some(list) = seasons.last_mut().and_then(|s| s["episodes"].as_array_mut()) {
            list.push(Value::Object(e));
        }
    }
    Ok(seasons)
}

/// Files worth a look, from everyone's plays. The three lists are only built for a caller who may
/// see everyone: from one person's own plays they would be noise, and a title in "files nobody gets
/// into" is a fact about other people's viewing.
pub async fn file_signals(State(app): State<App>, user: AuthUser, Query(q): Query<FilterQuery>) -> ApiResult {
    let out = scoped(&app, &user, &q, file_signals_for).await?;
    Ok(Json(out))
}

/// The most-rewound query, with the caller's window in `where_sql`; on its own so a test can hold
/// its plan to the index over event kinds. A rewind is a seek that landed before where it left from.
fn rewound_sql(where_sql: &str) -> String {
    format!(
        "WITH r AS (SELECT e.playback_id, e.position_s FROM playback_events e WHERE e.kind = 'seek' AND e.from_s > e.position_s)
         SELECT p.item_id AS id, COALESCE(i.name, MAX(p.item_name)) AS name, MAX(p.item_type) AS type, MAX(p.series_id) AS series_id, MAX(p.series_name) AS series_name,
                COUNT(DISTINCT p.id) AS plays, COUNT(r.playback_id) AS rewinds, ROUND(COUNT(r.playback_id) * 1.0 / COUNT(DISTINCT p.id), 2) AS per_play,
                (SELECT (r2.position_s / 60) * 60 FROM r r2 JOIN playbacks p2 ON p2.id = r2.playback_id WHERE p2.item_id = p.item_id
                 GROUP BY 1 ORDER BY COUNT(*) DESC, 1 LIMIT 1) AS hot_s
         FROM playbacks p LEFT JOIN r ON r.playback_id = p.id LEFT JOIN items i ON i.id = p.item_id {where_sql}
         GROUP BY p.item_id HAVING COUNT(DISTINCT p.id) >= 2 AND COUNT(r.playback_id) >= 3
         ORDER BY per_play DESC, rewinds DESC LIMIT 15"
    )
}

fn file_signals_for(c: &Connection, scope: &Scope) -> Result<Value> {
    if !scope.perms.see_everyone {
        return Ok(json!({ "broken": [], "rewound": [], "subtitled": [] }));
    }
    // A broken file is one nobody ever gets thirty seconds into, however many times it is tried: the
    // minimum play length is exactly what those plays never reach, so it does not apply here.
    let any = scope.cond_any_length().with_raw("p.item_type IN ('Movie', 'Episode') AND p.active = 0");
    let broken = rows_json(
        c,
        &format!(
            "SELECT p.item_id AS id, COALESCE(i.name, MAX(p.item_name)) AS name, MAX(p.item_type) AS type, MAX(p.series_id) AS series_id, MAX(p.series_name) AS series_name,
                    COUNT(*) AS plays, COUNT(DISTINCT p.user_id) AS users, MAX(p.duration_s) AS longest_s, MAX(p.started_at) AS last_tried_at,
                    json_group_array(DISTINCT p.client) FILTER (WHERE p.client IS NOT NULL) AS clients
             FROM playbacks p LEFT JOIN items i ON i.id = p.item_id {}
             GROUP BY p.item_id HAVING COUNT(*) >= 3 AND MAX(p.duration_s) < 30
             ORDER BY plays DESC, last_tried_at DESC LIMIT 25",
            any.sql()
        ),
        &any.args,
    )?;
    // Rewinds and subtitle switch-ons exist only for plays finstats recorded itself.
    let live = scope.cond().with_raw("p.source = 'live' AND p.item_type IN ('Movie', 'Episode') AND p.active = 0");
    let rewound = rows_json(c, &rewound_sql(&live.sql()), &live.args)?;
    // A play's first subtitle change, when it is to a track and within the first ten minutes: the
    // moment somebody found they could not follow the sound.
    let subtitled = rows_json(
        c,
        &format!(
            "WITH f AS (SELECT playback_id, MIN(id) AS first_id FROM playback_events WHERE kind = 'subtitle' GROUP BY playback_id),
                  o AS (SELECT f.playback_id, e.position_s FROM f JOIN playback_events e ON e.id = f.first_id
                        WHERE e.detail IS NOT NULL AND e.detail <> 'Off' AND e.position_s <= 600)
             SELECT p.item_id AS id, COALESCE(i.name, MAX(p.item_name)) AS name, MAX(p.item_type) AS type, MAX(p.series_id) AS series_id, MAX(p.series_name) AS series_name,
                    COUNT(DISTINCT p.id) AS plays, COUNT(o.playback_id) AS switched_on,
                    ROUND(COUNT(o.playback_id) * 1.0 / COUNT(DISTINCT p.id), 2) AS share, CAST(AVG(o.position_s) AS INTEGER) AS typical_s
             FROM playbacks p LEFT JOIN o ON o.playback_id = p.id LEFT JOIN items i ON i.id = p.item_id {}
             GROUP BY p.item_id HAVING COUNT(o.playback_id) >= 2 ORDER BY share DESC, switched_on DESC LIMIT 15",
            live.sql()
        ),
        &live.args,
    )?;
    Ok(json!({ "broken": broken, "rewound": rewound, "subtitled": subtitled }))
}

/// One actor or director: what they are in, and how much of it was watched (within the caller's scope).
pub async fn person_detail(State(app): State<App>, user: AuthUser, Path(id): Path<String>, Query(q): Query<FilterQuery>) -> ApiResult {
    let id = db::norm_id(&id);
    let out = scoped(&app, &user, &q, move |c, scope| {
        let person = one_json(
            c,
            "SELECT person_id AS id, MAX(name) AS name, MAX(has_image) AS has_image,
                    MAX(kind = 'Actor') AS is_actor, MAX(kind = 'Director') AS is_director, COUNT(DISTINCT item_id) AS titles
             FROM item_people WHERE person_id = ?1 GROUP BY person_id",
            &[id.clone().into()],
        )?;
        let Some(person) = person else { return Ok(None) };

        // A person can act in and direct the same title; the subquery keeps a play from counting twice.
        const TITLE: &str = "COALESCE(p.series_id, p.item_id)";
        let cond = scope.cond().with(&format!("{TITLE} IN (SELECT item_id FROM item_people WHERE person_id = ?)"), id.clone());
        let totals = one_json(
            c,
            &format!(
                "SELECT COUNT(*) AS plays, COALESCE(SUM(p.duration_s), 0) AS watch_s, COUNT(DISTINCT p.user_id) AS users,
                        COUNT(DISTINCT {TITLE}) AS titles_watched, MAX(p.ended_at) AS last_played_at FROM playbacks p {}",
                cond.sql()
            ),
            &cond.args,
        )?;
        let mut args: Vec<SqlValue> = vec![id.clone().into()];
        args.extend(cond.args.iter().cloned());
        let titles = rows_json(
            c,
            &format!(
                "SELECT i.id, COALESCE(i.name, 'Unknown title') AS name, i.type, i.production_year AS year, COALESCE(i.removed, 1) AS removed,
                        ip.kinds, ip.role, COALESCE(s.plays, 0) AS plays, COALESCE(s.watch_s, 0) AS watch_s, s.last_played_at
                 FROM (SELECT item_id, GROUP_CONCAT(kind, ',') AS kinds, MAX(role) AS role FROM item_people WHERE person_id = ? GROUP BY item_id) ip
                 LEFT JOIN items i ON i.id = ip.item_id
                 LEFT JOIN (SELECT {TITLE} AS title_id, COUNT(*) AS plays, SUM(p.duration_s) AS watch_s, MAX(p.ended_at) AS last_played_at
                            FROM playbacks p {} GROUP BY 1) s ON s.title_id = ip.item_id
                 ORDER BY watch_s DESC, i.production_year DESC, name",
                cond.sql(),
            ),
            &args,
        )?;
        let watchers = rows_json(
            c,
            &format!(
                "SELECT p.user_id, COALESCE(u.name, MAX(p.user_name)) AS user_name, COUNT(*) AS plays,
                        COALESCE(SUM(p.duration_s), 0) AS watch_s, MAX(p.ended_at) AS last_played_at
                 FROM playbacks p LEFT JOIN users u ON u.id = p.user_id {} GROUP BY p.user_id ORDER BY watch_s DESC LIMIT 50",
                cond.sql()
            ),
            &cond.args,
        )?;
        Ok(Some(json!({ "person": person, "totals": totals, "titles": titles, "watchers": watchers })))
    })
    .await?;
    out.map(Json).ok_or_else(|| ApiError::not_found("Person"))
}

#[derive(Deserialize)]
pub struct SearchQuery {
    q: Option<String>,
    limit: Option<i64>,
}

pub async fn search(State(app): State<App>, user: AuthUser, Query(q): Query<SearchQuery>) -> ApiResult {
    let Some(query) = crate::fuzzy::Query::new(q.q.as_deref().unwrap_or_default()) else {
        return Ok(Json(json!({ "items": [], "users": [], "people": [] })));
    };
    let limit = q.limit.unwrap_or(12).clamp(1, 50) as usize;
    let see_everyone = user.perms.see_everyone;
    let out = app
        .db
        .call(move |c| {
            // Matching is done here rather than with LIKE: word by word, any order, accents and typos
            // forgiven. A library's worth of titles is a few thousand short strings, which is nothing.
            let mut scored: Vec<(i32, i32, Map<String, Value>)> = rows_json(
                c,
                &format!("{ITEM_CARD} WHERE i.removed = 0 AND i.type IN ('Movie', 'Series', 'MusicAlbum', 'Audio')"),
                &[],
            )?
            .into_iter()
            .filter_map(|row| {
                let name = row.get("name").and_then(Value::as_str).unwrap_or_default();
                let kind = match row.get("type").and_then(Value::as_str) {
                    Some("Series") => 0,
                    Some("Movie") => 1,
                    Some("MusicAlbum") => 2,
                    _ => 3,
                };
                // Music is also found by its artist, films and shows by title only.
                let by_title = query.score(name);
                let by_artist = if kind >= 2 { row.get("sub").and_then(Value::as_str).and_then(|a| query.score(&format!("{a} {name}"))).map(|s| s - 150) } else { None };
                by_title.max(by_artist).map(|s| (s, kind, row))
            })
            .collect();
            scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)).then_with(|| a.2["name"].as_str().cmp(&b.2["name"].as_str())));
            let items: Vec<Value> = scored.into_iter().take(limit).map(|(_, _, row)| Value::Object(row)).collect();

            let users = if see_everyone {
                let mut found: Vec<(i32, Map<String, Value>)> = rows_json(c, "SELECT id, name FROM users ORDER BY removed, name COLLATE NOCASE", &[])?
                    .into_iter()
                    .filter_map(|u| query.score(u.get("name").and_then(Value::as_str).unwrap_or_default()).map(|s| (s, u)))
                    .collect();
                found.sort_by(|a, b| b.0.cmp(&a.0));
                found.into_iter().take(8).map(|(_, u)| Value::Object(u)).collect()
            } else {
                vec![]
            };
            // Cast and crew of what is in the library. Names are scored bare, and only the best few dozen are looked up
            // (counting every person's titles first took a third of a second on 10,000 people). More titles first among
            // equals: the lead of five shows before someone with the same name who appears once.
            let mut named: Vec<(i32, String)> = Vec::new();
            {
                let mut stmt = c.prepare("SELECT DISTINCT person_id, name FROM item_people")?;
                let mut rows = stmt.query([])?;
                while let Some(r) = rows.next()? {
                    if let Some(score) = query.score(r.get_ref(1)?.as_str().unwrap_or_default()) {
                        named.push((score, r.get(0)?));
                    }
                }
            }
            named.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
            let mut seen = std::collections::HashSet::new();
            named.retain(|(_, id)| seen.insert(id.clone())); // one person under two spellings: the better match stays
            named.truncate(60);
            let mut people: Vec<(i32, i64, Map<String, Value>)> = Vec::new();
            for (score, id) in named {
                let found = one_json(
                    c,
                    "SELECT ip.person_id AS id, MAX(ip.name) AS name, MAX(ip.has_image) AS has_image, MAX(ip.kind = 'Actor') AS is_actor,
                            MAX(ip.kind = 'Director') AS is_director, COUNT(DISTINCT ip.item_id) AS titles
                     FROM item_people ip JOIN items i ON i.id = ip.item_id AND i.removed = 0 WHERE ip.person_id = ?1 GROUP BY ip.person_id",
                    &[id.into()],
                )?;
                if let Some(p) = found {
                    people.push((score, p.get("titles").and_then(Value::as_i64).unwrap_or(0), p));
                }
            }
            people.sort_by(|a, b| b.0.cmp(&a.0).then(b.1.cmp(&a.1)).then_with(|| a.2["name"].as_str().cmp(&b.2["name"].as_str())));
            let people: Vec<Value> = people.into_iter().take(8).map(|(_, _, p)| Value::Object(p)).collect();
            Ok(json!({ "items": items, "users": users, "people": people }))
        })
        .await?;
    Ok(Json(out))
}

// ---------------------------------------------------------------- live & light status

pub async fn now_playing(State(app): State<App>, user: AuthUser) -> ApiResult {
    let mut sessions = app.live.read().unwrap().clone();
    // Before anything is filtered out: you may see who you are watching with, even if you can't see their streams.
    crate::groups::mark_live(&mut sessions, app.settings().group_window_s);
    if !user.perms.see_everyone {
        sessions.retain(|s| s["user_id"].as_str() == Some(user.id.as_str()));
    }
    if !user.perms.see_network {
        for s in &mut sessions {
            s["remote_ip"] = Value::Null;
        }
    }
    Ok(Json(json!({ "sessions": sessions })))
}

pub async fn summary(State(app): State<App>, user: AuthUser) -> ApiResult {
    let scope_user = (!user.perms.see_everyone).then(|| user.id.clone());
    let (plays, last_sync): (i64, Option<i64>) = app
        .db
        .call(move |c| {
            let plays = match &scope_user {
                Some(u) => c.query_row("SELECT COUNT(*) FROM playbacks WHERE user_id = ?1", [u], |r| r.get(0))?,
                None => c.query_row("SELECT COUNT(*) FROM playbacks", [], |r| r.get(0))?,
            };
            let last = c.query_row("SELECT MAX(updated_at) FROM items", [], |r| r.get::<_, Option<i64>>(0))?.filter(|t| *t > 0);
            Ok((plays, last))
        })
        .await?;
    let collector = app.collector.read().unwrap().clone();
    let active = if user.perms.see_everyone {
        collector.active_sessions
    } else {
        app.live.read().unwrap().iter().filter(|s| s["user_id"].as_str() == Some(user.id.as_str())).count()
    };
    Ok(Json(json!({
        "active_sessions": active, "plays_total": plays, "last_sync_at": last_sync,
        "collector_ok": collector.connected, "collector_live": collector.socket_live, "version": env!("CARGO_PKG_VERSION"),
    })))
}

// ---------------------------------------------------------------- server log

#[derive(Deserialize)]
pub struct EventsQuery {
    page: Option<i64>,
    per_page: Option<i64>,
    q: Option<String>,
    #[serde(rename = "type")]
    kind: Option<String>,
    sort: Option<String>,
    dir: Option<String>,
}

const EVENT_SORTS: [(&str, &str); 4] =
    [("when", "e.date"), ("event", "e.name COLLATE NOCASE"), ("type", "e.type COLLATE NOCASE"), ("user", "u.name COLLATE NOCASE")];

pub async fn events(State(app): State<App>, ServerViewer(_): ServerViewer, Query(q): Query<EventsQuery>) -> ApiResult {
    let page = page_number(q.page);
    let per_page = q.per_page.unwrap_or(50).clamp(1, 200);
    let out = app
        .db
        .call(move |c| {
            let mut clauses: Vec<String> = vec![];
            let mut args: Vec<SqlValue> = vec![];
            if let Some(t) = q.kind.filter(|t| !t.is_empty()) {
                clauses.push("e.type = ?".into());
                args.push(t.into());
            }
            for like in like_words(q.q.as_deref()) {
                clauses.push("(e.name LIKE ? ESCAPE '\\' OR e.short_overview LIKE ? ESCAPE '\\' OR e.overview LIKE ? ESCAPE '\\')".into());
                args.extend([like.clone().into(), like.clone().into(), like.into()]);
            }
            let wh = if clauses.is_empty() { String::new() } else { format!("WHERE {}", clauses.join(" AND ")) };
            let total: i64 = c.query_row(&format!("SELECT COUNT(*) FROM server_events e {wh}"), params_from_iter(args.iter()), |r| r.get(0))?;
            let rows = rows_json(
                c,
                &format!(
                    "SELECT e.id, e.date, e.name, COALESCE(e.overview, e.short_overview) AS overview, e.type, e.severity,
                            e.user_id, u.name AS user_name, e.item_id
                     FROM server_events e LEFT JOIN users u ON u.id = e.user_id {wh}
                     ORDER BY {} LIMIT {per_page} OFFSET {}",
                    order_by(&EVENT_SORTS, q.sort.as_deref(), q.dir.as_deref(), "e.date DESC, e.id DESC"),
                    (page - 1) * per_page
                ),
                &args,
            )?;
            Ok(json!({ "total": total, "page": page, "per_page": per_page, "rows": rows }))
        })
        .await?;
    Ok(Json(out))
}

// ---------------------------------------------------------------- v0.2: insights

fn external_links(item_type: Option<&str>, provider_ids: Option<Value>) -> Vec<Value> {
    let Some(Value::Object(ids)) = provider_ids else { return vec![] };
    let clean = |v: &Value| v.as_str().filter(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')).map(str::to_string);
    let tmdb_kind = if matches!(item_type, Some("Series" | "Season" | "Episode")) { "tv" } else { "movie" };
    let mut out = vec![];
    for (key, value) in &ids {
        let Some(id) = clean(value) else { continue };
        let link = match key.to_ascii_lowercase().as_str() {
            "imdb" => ("IMDb", format!("https://www.imdb.com/title/{id}/")),
            "tmdb" => ("TMDB", format!("https://www.themoviedb.org/{tmdb_kind}/{id}")),
            "tvdb" if tmdb_kind == "tv" => ("TVDB", format!("https://www.thetvdb.com/dereferrer/series/{id}")),
            "musicbrainzalbum" => ("MusicBrainz", format!("https://musicbrainz.org/release/{id}")),
            _ => continue,
        };
        out.push(json!({ "label": link.0, "url": link.1 }));
    }
    out
}

/// Watch time per genre. Episodes carry no genres of their own, so they count towards their series'.
pub(crate) fn genre_buckets(conn: &Connection, cond: &Cond) -> Result<Vec<Value>> {
    // Counted per title first, then spread over each title's genres: looking a title and its genres up once per play
    // instead took twice as long (1.2 s on a million plays), for the same numbers.
    let sql = format!(
        "SELECT g.value AS name, SUM(t.plays) AS plays, SUM(t.watch_s) AS watch_s
         FROM (SELECT COALESCE(p.series_id, p.item_id) AS title, COUNT(*) AS plays, COALESCE(SUM(p.duration_s), 0) AS watch_s
               FROM playbacks p {} GROUP BY 1) t
         JOIN items gi ON gi.id = t.title, json_each(gi.genres) g
         GROUP BY 1 HAVING name IS NOT NULL ORDER BY plays DESC, watch_s DESC",
        cond.sql()
    );
    let mut v = top_and_other(rows_json(conn, &sql, &cond.args)?, 12);
    v.sort_by_key(|b| std::cmp::Reverse(b["watch_s"].as_i64().unwrap_or(0)));
    Ok(v)
}

fn jellyfin_flags(conn: &Connection, user_id: &str) -> Result<Value> {
    let row = one_json(
        conn,
        "SELECT COALESCE(SUM(ui.played AND i.type = 'Movie'), 0) AS played_movies,
                COALESCE(SUM(ui.played AND i.type = 'Episode'), 0) AS played_episodes,
                COALESCE(SUM(ui.is_favorite), 0) AS favorites, COUNT(*) AS n
         FROM user_items ui LEFT JOIN items i ON i.id = ui.item_id WHERE ui.user_id = ?1",
        &[user_id.to_string().into()],
    )?;
    Ok(match row {
        Some(mut r) if r.get("n").and_then(Value::as_i64).unwrap_or(0) > 0 => {
            r.remove("n");
            Value::Object(r)
        }
        _ => Value::Null,
    })
}

/// The local day of a moment, looked up once per quarter hour of UTC rather than once per moment: every UTC offset and
/// every daylight-saving change in use falls on a whole quarter hour, so a local date cannot change inside one. Asked
/// in order, as [`concurrency`] asks, a history needs as many lookups as quarter hours it spans, not as plays it holds.
struct LocalDays<F> {
    lookup: F,
    last: Option<(i64, String)>,
}

impl<F: FnMut(i64) -> Result<String>> LocalDays<F> {
    fn new(lookup: F) -> Self {
        Self { lookup, last: None }
    }

    fn of(&mut self, at: i64) -> Result<&str> {
        let quarter = at.div_euclid(900);
        if self.last.as_ref().map(|(q, _)| *q) != Some(quarter) {
            self.last = Some((quarter, (self.lookup)(quarter * 900)?));
        }
        Ok(self.last.as_ref().map(|(_, day)| day.as_str()).unwrap_or_default())
    }
}

/// Sweep over play intervals: the most streams (and transcodes) that ever overlapped, and the peak per bucket.
fn concurrency(conn: &Connection, scope: &Scope, cond: &Cond) -> Result<Value> {
    // COALESCE, because `play_method` may be NULL — a row restored from a backup written before the
    // column existed has nothing to put there — and `NULL = 'Transcode'` is NULL, which is not a
    // boolean and made this whole page a 500 for everybody until that one row was found.
    let sql = format!("SELECT p.started_at, p.ended_at, COALESCE(p.play_method = 'Transcode', 0) FROM playbacks p {} AND p.ended_at > p.started_at", cond.with_raw("1 = 1").sql());
    let mut points: Vec<(i64, i32, i32)> = vec![];
    let mut stmt = conn.prepare(&sql)?;
    let mut rows = stmt.query(params_from_iter(cond.args.iter()))?;
    while let Some(r) = rows.next()? {
        let (start, end, transcode): (i64, i64, bool) = (r.get(0)?, r.get(1)?, r.get(2)?);
        points.push((start, 1, transcode as i32));
        points.push((end, -1, -(transcode as i32)));
    }
    // Ends sort before starts at the same instant, so back-to-back episodes are not "two streams".
    points.sort_by_key(|p| (p.0, p.1));

    let (series_template, bucket) = daily(conn, scope, cond)?;
    let weekly = bucket == "week";
    let mut per_bucket: std::collections::HashMap<String, i32> = Default::default();
    let (mut live, mut live_tc, mut peak, mut peak_at, mut peak_tc) = (0, 0, 0, None, 0);
    let mut date_stmt = conn.prepare_cached(if weekly {
        "SELECT date(?1, 'unixepoch', 'localtime', 'weekday 0', '-6 days')"
    } else {
        "SELECT date(?1, 'unixepoch', 'localtime')"
    })?;
    let mut days = LocalDays::new(|at| Ok(date_stmt.query_row([at], |r| r.get(0))?));
    for (at, delta, tc_delta) in points {
        live += delta;
        live_tc += tc_delta;
        if delta > 0 {
            if live > peak {
                peak = live;
                peak_at = Some(at);
            }
            peak_tc = peak_tc.max(live_tc);
            let date = days.of(at)?;
            match per_bucket.get_mut(date) {
                Some(e) => *e = (*e).max(live),
                None => {
                    per_bucket.insert(date.to_string(), live);
                }
            }
        }
    }
    let series: Vec<Value> = series_template
        .iter()
        .map(|d| {
            let date = d["date"].as_str().unwrap_or_default();
            json!({ "date": date, "peak": per_bucket.get(date).copied().unwrap_or(0) })
        })
        .collect();
    Ok(json!({ "peak": peak, "peak_at": peak_at, "peak_transcodes": peak_tc, "series": series, "bucket": bucket }))
}

/// A caller's scope, resolved once, for a handler that runs its queries side by side.
async fn resolved(app: &App, user: &AuthUser, q: &FilterQuery) -> Result<Scope> {
    let scope = Scope::new(app, user, q);
    app.db.call(move |c| scope.resolve(c)).await
}

/// One part of a page on a pooled connection of its own, so that parts which need nothing from each other run side by
/// side: each is one core's work in SQLite, and one after another a page takes the sum of them.
async fn apart<T, F>(app: App, scope: Scope, f: F) -> Result<T>
where
    T: Send + 'static,
    F: FnOnce(&Connection, &Scope) -> Result<T> + Send + 'static,
{
    app.db.call(move |c| f(c, &scope)).await
}

fn insight_network(c: &Connection, scope: &Scope) -> Result<Vec<Value>> {
    if !scope.perms.see_network {
        return Ok(vec![]);
    }
    buckets(c, &scope.cond(), "CASE p.is_local WHEN 1 THEN 'Local' WHEN 0 THEN 'Remote' ELSE 'Unknown' END", "", 3)
}

/// What actually went over the wire: the transcoded bitrate when transcoding, else the file's.
fn insight_data_bytes(c: &Connection, scope: &Scope) -> Result<i64> {
    let cond = scope.cond();
    Ok(c.query_row(
        &format!(
            "SELECT CAST(COALESCE(SUM(COALESCE(json_extract(p.transcode, '$.bitrate'), p.bitrate) / 8.0 * p.duration_s), 0) AS INTEGER)
             FROM playbacks p {}",
            cond.sql()
        ),
        params_from_iter(cond.args.iter()),
        |r| r.get(0),
    )?)
}

fn insight_client_methods(c: &Connection, scope: &Scope) -> Result<Vec<Map<String, Value>>> {
    let cm = scope.cond().with_raw("p.client IS NOT NULL");
    rows_json(
        c,
        &format!(
            "SELECT p.client, COALESCE(SUM(p.play_method = 'DirectPlay'), 0) AS direct_play, COALESCE(SUM(p.play_method = 'DirectStream'), 0) AS direct_stream,
                    COALESCE(SUM(p.play_method = 'Transcode'), 0) AS transcode, COALESCE(SUM(p.duration_s), 0) AS watch_s
             FROM playbacks p {} GROUP BY p.client ORDER BY COUNT(*) DESC LIMIT 12",
            cm.sql()
        ),
        &cm.args,
    )
}

fn insight_completion(c: &Connection, scope: &Scope) -> Result<Vec<Value>> {
    let video = scope.cond().with_raw("p.item_type IN ('Movie', 'Episode') AND COALESCE(p.runtime_s, 0) > 0");
    let comp = one_json(
        c,
        &format!(
            "WITH r AS (SELECT MIN(1.0, COALESCE(p.position_s, p.duration_s) * 1.0 / p.runtime_s) AS f FROM playbacks p {})
             SELECT COALESCE(SUM(f < 0.1), 0) AS a, COALESCE(SUM(f >= 0.1 AND f < 0.5), 0) AS b,
                    COALESCE(SUM(f >= 0.5 AND f < 0.9), 0) AS c, COALESCE(SUM(f >= 0.9), 0) AS d FROM r",
            video.sql()
        ),
        &video.args,
    )?
    .unwrap_or_default();
    Ok([("Under 10%", "a"), ("10–50%", "b"), ("50–90%", "c"), ("Finished (90%+)", "d")]
        .iter()
        .map(|(name, k)| json!({ "name": name, "plays": comp.get(*k).cloned().unwrap_or(json!(0)) }))
        .collect())
}

fn insight_behaviour(c: &Connection, scope: &Scope) -> Result<Option<Map<String, Value>>> {
    let live = scope.cond().with_raw("p.source = 'live' AND p.active = 0");
    one_json(
        c,
        &format!(
            "SELECT COUNT(*) AS plays_measured, ROUND(COALESCE(AVG(p.pause_count), 0), 2) AS avg_pauses,
                    ROUND(COALESCE(AVG(p.seek_count), 0), 2) AS avg_seeks,
                    ROUND(COALESCE(AVG(COALESCE(p.start_position_s, 0) > 30), 0), 3) AS resumed_share
             FROM playbacks p {}",
            live.sql()
        ),
        &live.args,
    )
}

fn insight_failed_logins(c: &Connection, scope: &Scope) -> Result<Vec<Map<String, Value>>> {
    if !scope.perms.see_server {
        return Ok(vec![]);
    }
    let mut args: Vec<SqlValue> = vec![];
    let mut wh = "WHERE (e.type LIKE '%AuthenticationFail%' OR e.name LIKE '%failed%login%' OR e.name LIKE '%Failed login%')".to_string();
    if let Some(s) = scope.since {
        wh.push_str(" AND e.date >= ?");
        args.push(s.into());
    }
    rows_json(
        c,
        &format!(
            "SELECT e.date, e.name || COALESCE(' · ' || COALESCE(e.short_overview, e.overview), '') AS overview, u.name AS user_name
             FROM server_events e LEFT JOIN users u ON u.id = e.user_id {wh} ORDER BY e.date DESC LIMIT 10"
        ),
        &args,
    )
}

pub async fn insights(State(app): State<App>, user: AuthUser, Query(q): Query<FilterQuery>) -> ApiResult {
    // Passes over the history that need nothing from each other, in five groups of about equal cost, each on a
    // connection of its own (one of the pool's six stays free for the collector). One after another they took the sum
    // of their times, 4.1 s on a million plays.
    let scope = resolved(&app, &user, &q).await?;
    let (concurrency, genres, client_methods, (network, completion, behaviour), (data_bytes, failed_logins)) = tokio::try_join!(
        apart(app.clone(), scope.clone(), |c, s| concurrency(c, s, &s.cond())),
        apart(app.clone(), scope.clone(), |c, s| genre_buckets(c, &s.cond())),
        apart(app.clone(), scope.clone(), insight_client_methods),
        apart(app.clone(), scope.clone(), |c, s| Ok((insight_network(c, s)?, insight_completion(c, s)?, insight_behaviour(c, s)?))),
        apart(app.clone(), scope.clone(), |c, s| Ok((insight_data_bytes(c, s)?, insight_failed_logins(c, s)?))),
    )?;
    Ok(Json(json!({
        "concurrency": concurrency,
        "network": network, "data_bytes": data_bytes,
        "genres": genres,
        "client_methods": client_methods, "completion": completion,
        "behaviour": behaviour, "failed_logins": failed_logins,
    })))
}

// ---------------------------------------------------------------- v0.2: what the library is made of

#[derive(Deserialize)]
pub struct LibraryInsightsQuery {
    library_id: Option<String>,
}

fn lib_buckets(conn: &Connection, expr: &str, from: &str, wh: &str, args: &[SqlValue], order: &str, max: usize) -> Result<Vec<Value>> {
    let sql = format!(
        "SELECT {expr} AS name, COUNT(*) AS count, COALESCE(SUM(i.size_bytes), 0) AS size_bytes
         FROM items i {from} {wh} GROUP BY 1 HAVING name IS NOT NULL ORDER BY {order}"
    );
    let rows = rows_json(conn, &sql, args)?;
    let mut out: Vec<Value> = vec![];
    let (mut n, mut sz) = (0i64, 0i64);
    for (i, r) in rows.into_iter().enumerate() {
        if i < max {
            out.push(Value::Object(r));
        } else {
            n += r["count"].as_i64().unwrap_or(0);
            sz += r["size_bytes"].as_i64().unwrap_or(0);
        }
    }
    if n > 0 {
        out.push(json!({ "name": "Other", "count": n, "size_bytes": sz }));
    }
    Ok(out)
}

pub async fn library_insights(State(app): State<App>, _user: AuthUser, Query(q): Query<LibraryInsightsQuery>) -> ApiResult {
    let library = q.library_id.as_deref().map(db::norm_id).filter(|s| !s.is_empty());
    let out = app
        .db
        .call(move |c| {
            let (lib_sql, args): (&str, Vec<SqlValue>) = match &library {
                Some(l) => ("AND i.library_id = ?1", vec![l.clone().into()]),
                None => ("", vec![]),
            };
            let files = format!("WHERE i.removed = 0 AND i.size_bytes IS NOT NULL {lib_sql}");
            let video = format!("{files} AND i.video_codec IS NOT NULL");
            let titles = format!("WHERE i.removed = 0 AND i.type IN ('Movie', 'Series') {lib_sql}");
            let by_count = "count DESC, size_bytes DESC";
            let res_sql = RESOLUTION_SQL.replace("p.", "i.");

            let totals = one_json(
                c,
                &format!(
                    "SELECT COALESCE(SUM(i.size_bytes IS NOT NULL), 0) AS files, COALESCE(SUM(i.size_bytes), 0) AS size_bytes,
                            COALESCE(SUM(CASE WHEN i.size_bytes IS NOT NULL THEN i.runtime_s END), 0) AS runtime_s,
                            COALESCE(SUM(i.type = 'Movie'), 0) AS movies, COALESCE(SUM(i.type = 'Series'), 0) AS series,
                            COALESCE(SUM(i.type = 'Episode'), 0) AS episodes, COALESCE(SUM(i.type = 'Audio'), 0) AS tracks
                     FROM items i WHERE i.removed = 0 {lib_sql}"
                ),
                &args,
            )?;

            // Month buckets for the last two years, gap-free.
            let added_rows = rows_json(
                c,
                &format!(
                    "SELECT strftime('%Y-%m', i.date_created, 'unixepoch', 'localtime') AS month, COUNT(*) AS count
                     FROM items i WHERE i.removed = 0 AND i.type IN ('Movie', 'Episode', 'Audio', 'Video', 'MusicVideo')
                       AND i.date_created >= CAST(strftime('%s', 'now', 'start of month', '-23 months') AS INTEGER) {lib_sql} GROUP BY 1"
                ),
                &args,
            )?;
            let found: std::collections::HashMap<String, i64> =
                added_rows.iter().filter_map(|r| Some((r["month"].as_str()?.to_string(), r["count"].as_i64()?))).collect();
            let today: String = c.query_row("SELECT date('now', 'localtime', 'start of month')", [], |r| r.get(0))?;
            let mut cursor = NaiveDate::parse_from_str(&today, "%Y-%m-%d")?;
            let mut added = vec![];
            for _ in 0..24 {
                let key = cursor.format("%Y-%m").to_string();
                added.push(json!({ "month": key, "count": found.get(&key).copied().unwrap_or(0) }));
                cursor = (cursor - chrono::Duration::days(1)).with_day(1).unwrap_or(cursor);
            }
            added.reverse();

            // A series weighs what its episodes weigh.
            let sized = format!(
                "SELECT t.id, t.name, t.type, t.production_year AS year, t.date_created, t.id AS image_item_id,
                        COALESCE(t.size_bytes, (SELECT SUM(e.size_bytes) FROM items e WHERE e.series_id = t.id AND e.removed = 0), 0) AS size_bytes
                 FROM items t WHERE t.removed = 0 AND t.type IN ('Movie', 'Series') {}",
                lib_sql.replace("i.", "t.")
            );
            let largest = rows_json(c, &format!("{sized} ORDER BY size_bytes DESC LIMIT 15"), &args)?;
            // Nobody has played it here, and nobody has it marked as played in Jellyfin either.
            let never = "NOT EXISTS (SELECT 1 FROM playbacks p WHERE p.item_id = t.id OR p.series_id = t.id)
                 AND NOT EXISTS (SELECT 1 FROM user_items ui WHERE ui.played = 1 AND (ui.item_id = t.id OR ui.item_id IN (SELECT id FROM items WHERE series_id = t.id)))";
            let unwatched_items = rows_json(c, &format!("{sized} AND {never} ORDER BY size_bytes DESC LIMIT 25"), &args)?;
            let unwatched_totals =
                one_json(c, &format!("SELECT COUNT(*) AS count, COALESCE(SUM(size_bytes), 0) AS size_bytes FROM ({sized} AND {never})"), &args)?.unwrap_or_default();

            Ok(json!({
                "totals": totals,
                "resolutions": lib_buckets(c, &res_sql, "", &video, &args, by_count, 12)?,
                "video_codecs": lib_buckets(c, "UPPER(i.video_codec)", "", &video, &args, by_count, 12)?,
                "video_ranges": lib_buckets(c, "i.video_range", "", &video, &args, by_count, 12)?,
                "containers": lib_buckets(c, "LOWER(i.container)", "", &files, &args, by_count, 12)?,
                "audio_codecs": lib_buckets(c, "UPPER(i.audio_codec)", "", &files, &args, by_count, 12)?,
                "audio_languages": lib_buckets(c, "al.value", ", json_each(i.audio_languages) al", &video, &args, by_count, 12)?,
                "subtitle_languages": lib_buckets(c, "sl.value", ", json_each(i.subtitle_languages) sl", &video, &args, by_count, 12)?,
                // A series row has no size of its own, so a per-genre size would only count the films.
                "genres": lib_buckets(c, "g.value", ", json_each(i.genres) g", &titles, &args, by_count, 12)?
                    .into_iter()
                    .map(|mut g| {
                        g["size_bytes"] = json!(0);
                        g
                    })
                    .collect::<Vec<_>>(),
                "decades": lib_buckets(c, "(i.production_year / 10 * 10) || 's'", "", &format!("{titles} AND i.production_year > 1800"), &args, "name", 30)?,
                "added": added, "largest": largest,
                "unwatched": { "count": unwatched_totals.get("count"), "size_bytes": unwatched_totals.get("size_bytes"), "items": unwatched_items },
            }))
        })
        .await?;
    Ok(Json(out))
}

// ---------------------------------------------------------------- v0.2: the Jellyfin server

pub async fn server(State(app): State<App>, ServerViewer(user): ServerViewer) -> ApiResult {
    let see_network = user.perms.see_network;
    let out = app
        .db
        .call(move |c| {
            let mut snapshot: Value = db::get_setting(c, "server_info")?.and_then(|raw| serde_json::from_str(&raw).ok()).unwrap_or_else(|| {
                json!({ "fetched_at": null, "info": null, "storage": [], "plugins": [], "scheduled_tasks": [] })
            });
            // A device id is see_network's, here as in every play.
            let devices = rows_json(
                c,
                "SELECT CASE WHEN ?1 THEN d.device_id END AS device_id, d.device_name AS name, d.client AS app, d.app_version, d.user_id AS last_user_id,
                        COALESCE(u.name, d.last_user_name) AS last_user_name, d.last_seen
                 FROM devices d LEFT JOIN users u ON u.id = d.user_id ORDER BY d.last_seen DESC LIMIT 500",
                &[see_network.into()],
            )?;
            snapshot["devices"] = json!(devices);
            Ok(snapshot)
        })
        .await?;
    Ok(Json(out))
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
            "INSERT INTO playbacks(source, user_id, user_name, item_id, item_name, item_type, started_at, ended_at, duration_s) VALUES
               ('live',         'u1', 'alice', 'i1', 'Big Buck Bunny', 'Movie', 100, 700, 600),
               ('jellystat',    'u1', 'alice', 'i1', 'Big Buck Bunny', 'Movie', 900, 1500, 600),
               ('streamystats', 'u2', 'bob',   'i1', 'Big Buck Bunny', 'Movie', 2000, 2600, 600);",
        )
        .unwrap();
        c
    }

    fn caller(id: &str, see_everyone: bool) -> AuthUser {
        let perms = crate::auth::Perms { see_everyone, ..Default::default() };
        AuthUser { id: id.to_string(), name: "x".into(), is_admin: false, perms, credential: Default::default(), ip: None }
    }

    #[test]
    fn a_filter_can_name_several_values_and_nothing_it_should_not() {
        assert_eq!(many(Some("Movie,Episode")), ["Movie", "Episode"]);
        // Spelt the way a URL really arrives: spaces around the commas, a trailing one, repeats.
        assert_eq!(many(Some(" Movie , Episode , ")), ["Movie", "Episode"]);
        assert_eq!(many(Some("Movie,Movie,Episode")), ["Movie", "Episode"]);
        // Nothing asked for is not a filter at all, which is how "All" is spelt.
        for none in [None, Some(""), Some(","), Some("   "), Some(",,,")] {
            assert!(many(none).is_empty(), "{none:?}");
        }
        // A caller cannot make the SQL arbitrarily long by naming thousands of values.
        assert_eq!(many(Some(&(0..500).map(|i| i.to_string()).collect::<Vec<_>>().join(","))).len(), 50);
    }

    #[test]
    fn whose_plays_a_request_may_be_about_is_decided_in_one_place() {
        // With "see everyone": the people asked for, or everybody when nobody is named.
        assert_eq!(pinned_users(&caller("me", true), &[]), Vec::<String>::new());
        assert_eq!(pinned_users(&caller("me", true), &["a".into(), "b".into()]), ["a", "b"]);
        // Ids are normalised and repeats collapse, so the SQL cannot be padded out.
        assert_eq!(pinned_users(&caller("me", true), &["A-B".into(), "ab".into(), "".into()]), ["ab"]);
        // Without it: always exactly themselves, whoever the URL names, however many.
        assert_eq!(pinned_users(&caller("me", false), &[]), ["me"]);
        assert_eq!(pinned_users(&caller("me", false), &["someone".into()]), ["me"]);
        assert_eq!(pinned_users(&caller("me", false), &["someone".into(), "me".into(), "other".into()]), ["me"]);
    }

    // ---- v1.8: playback insights

    fn everyone() -> Scope {
        Scope { days: 0, since: None, user_ids: vec![], library_id: None, min_play_s: 0, perms: crate::auth::Perms::ALL }
    }

    #[test]
    fn concurrency_is_the_most_streams_at_once_overall_and_per_day() {
        let c = Connection::open_in_memory().unwrap();
        for m in crate::db::MIGRATIONS {
            c.execute_batch(m).unwrap();
        }
        let at = |sql: &str| -> i64 { c.query_row(&format!("SELECT CAST(strftime('%s', {sql}, 'utc') AS INTEGER)"), [], |r| r.get(0)).unwrap() };
        let (y, t) = (at("date('now', 'localtime', '-1 day') || ' 10:00:00'"), at("date('now', 'localtime') || ' 00:00:30'"));
        c.execute(
            "INSERT INTO playbacks(source, user_id, user_name, item_id, item_name, item_type, play_method, started_at, ended_at, duration_s) VALUES
               ('live', 'u1', 'alice', 'i1', 'Big Buck Bunny', 'Movie', 'Transcode',  ?1,        ?1 + 3600, 3600),
               ('live', 'u2', 'bob',   'i2', 'Sintel',         'Movie', 'DirectPlay', ?1 + 1800, ?1 + 5400, 3600),
               -- starts the second the one before it ends: one stream after another, not two at once
               ('live', 'u2', 'bob',   'i3', 'Tears of Steel', 'Movie', 'DirectPlay', ?1 + 5400, ?1 + 7200, 1800),
               ('live', 'u1', 'alice', 'i2', 'Sintel',         'Movie', 'DirectPlay', ?2,        ?2 + 60,   60)",
            [y, t],
        )
        .unwrap();
        let scope = Scope { days: 2, ..everyone() }.resolve(&c).unwrap();
        let v = concurrency(&c, &scope, &scope.cond()).unwrap();
        assert_eq!((v["peak"].as_i64(), v["peak_at"].as_i64(), v["peak_transcodes"].as_i64()), (Some(2), Some(y + 1800), Some(1)));
        let day = |s: i64| -> String { c.query_row("SELECT date(?1, 'unixepoch', 'localtime')", [s], |r| r.get(0)).unwrap() };
        assert_eq!(v["series"], json!([{ "date": day(y), "peak": 2 }, { "date": day(t), "peak": 1 }]));
    }

    /// A device id is `see_network`'s, like an address: on the user page as everywhere else.
    #[test]
    fn a_user_page_names_devices_but_gives_their_ids_only_with_see_network() {
        let c = conn();
        c.execute("UPDATE playbacks SET device_id = 'dev-1', device_name = 'Living room TV'", []).unwrap();
        let scope = everyone().resolve(&c).unwrap();
        let without = user_devices(&c, &scope.cond(), false).unwrap();
        assert!(!without.is_empty() && without.iter().all(|d| d["device_id"].is_null() && d["device_name"] == "Living room TV"), "{without:?}");
        let with = user_devices(&c, &scope.cond(), true).unwrap();
        assert_eq!(with[0]["device_id"], "dev-1");
    }

    #[test]
    fn genres_count_each_play_under_every_genre_of_its_title() {
        let c = conn();
        c.execute_batch(
            "DELETE FROM playbacks;
             INSERT INTO items(id, type, name, genres, updated_at) VALUES
               ('show', 'Series', 'Test Show', '[\"Drama\",\"Crime\"]', 0), ('film', 'Movie', 'Sintel', '[\"Animation\"]', 0);
             INSERT INTO playbacks(source, user_id, user_name, item_id, item_name, item_type, series_id, started_at, ended_at, duration_s) VALUES
               -- an episode counts under its show's genres
               ('live', 'u1', 'alice', 'e1', 'Pilot', 'Episode', 'show', 1, 2, 100),
               ('live', 'u1', 'alice', 'e2', 'Two',   'Episode', 'show', 3, 4, 300),
               ('live', 'u2', 'bob',   'film', 'Sintel', 'Movie', NULL, 5, 6, 500),
               -- a title the library does not know has no genre to count under
               ('live', 'u2', 'bob',   'gone', 'Lost', 'Movie', NULL, 7, 8, 900);",
        )
        .unwrap();
        let scope = everyone();
        let got: Vec<(String, i64, i64)> = genre_buckets(&c, &scope.cond()).unwrap().iter()
            .map(|b| (b["name"].as_str().unwrap().to_string(), b["plays"].as_i64().unwrap(), b["watch_s"].as_i64().unwrap())).collect();
        assert_eq!(got, [("Animation".into(), 1, 500), ("Crime".into(), 2, 400), ("Drama".into(), 2, 400)]);
    }

    #[test]
    fn a_local_day_is_looked_up_once_a_quarter_hour_at_most() {
        // concurrency() asked SQLite for the local day of every play's start: a million lookups on a million plays.
        let asked = std::cell::Cell::new(0);
        let mut days = LocalDays::new(|at| {
            asked.set(asked.get() + 1);
            Ok(format!("day {}", at / 86_400))
        });
        let seen: Vec<String> = [0, 10, 899, 900, 901, 1799, 86_400, 86_401].into_iter().map(|at| days.of(at).unwrap().to_string()).collect();
        assert_eq!(seen, ["day 0", "day 0", "day 0", "day 0", "day 0", "day 0", "day 1", "day 1"]);
        assert_eq!(asked.get(), 3, "three quarter hours: 0–899, 900–1799, and the one starting at 86,400");
    }

    #[test]
    fn bucket_width_keeps_a_curve_to_sixty_points_or_fewer() {
        assert_eq!(bucket_width(1500), 30);
        assert_eq!(bucket_width(1800), 30);
        assert_eq!(bucket_width(2700), 60);
        assert_eq!(bucket_width(7200), 120);
        assert_eq!(bucket_width(10800), 300);
        assert_eq!(bucket_width(60), 30);
        assert_eq!(bucket_width(0), 30);
        for rt in [1, 1500, 7020, 7200, 20000, 100000] {
            assert!((rt as f64 / bucket_width(rt) as f64).ceil() as i64 <= 60, "{rt}");
        }
    }

    #[test]
    fn the_curve_starts_at_everyone_and_never_rises() {
        let curve = retention_curve(&[7200, 7200, 240, 3600], 7200, 120);
        assert_eq!(curve.len(), 61, "one point per bucket edge, both ends included");
        assert_eq!(curve[0], 1.0);
        assert_eq!(curve[2], 1.0, "at four minutes everyone is still there");
        assert_eq!(curve[3], 0.75, "the one that stopped at four minutes is gone by six");
        assert_eq!(curve[30], 0.75);
        assert_eq!(curve[31], 0.5);
        assert_eq!(curve[60], 0.5, "two of four reached the end");
        assert!(curve.windows(2).all(|w| w[1] <= w[0]));
    }

    #[test]
    fn a_stop_beyond_the_runtime_or_before_the_start_stays_on_the_axis() {
        // A file replaced by a shorter one leaves old stops past the end: they count as finished.
        let curve = retention_curve(&[9999, 10], 7200, 120);
        assert_eq!(curve[60], 0.5);
        // A play shorter than one bucket is there at the start and gone at the first edge.
        assert_eq!(curve[0], 1.0);
        assert_eq!(curve[1], 0.5);
        // A runtime that is not a whole number of buckets still ends at the runtime.
        assert_eq!(retention_curve(&[7020, 7000], 7020, 120).len(), 60, "59 buckets and both ends");
        assert_eq!(retention_curve(&[7020, 7000], 7020, 120)[59], 0.5);
        assert_eq!(retention_curve(&[], 7200, 120), vec![0.0; 61].iter().map(|_| 0.0).collect::<Vec<f64>>());
    }

    #[test]
    fn histogram_puts_a_position_in_its_bucket_and_the_end_in_the_last_one() {
        let h = histogram(&[0, 119, 120, 7200, 7300, -5], 7200, 120);
        assert_eq!(h.len(), 60);
        assert_eq!(h[0], 3, "0, 119 and a negative all belong to the first bucket");
        assert_eq!(h[1], 1);
        assert_eq!(h[59], 2, "the end and anything past it belong to the last bucket");
        assert_eq!(h.iter().sum::<i64>(), 6);
    }

    fn with_plays(c: &Connection) {
        c.execute_batch(
            "INSERT INTO playbacks(id, source, user_id, user_name, item_id, item_name, item_type, started_at, ended_at, duration_s, position_s, runtime_s) VALUES
               -- finstats saw this one: it stopped at ten minutes after fifty minutes of playing (a resume)
               (10, 'live',         'u1', 'alice', 'i9', 'Sintel', 'Movie', 1000, 4000, 3000, 600, 6000),
               -- imported: only how long it ran is known, so it is taken to have stopped at fifty minutes
               (11, 'jellystat',    'u1', 'alice', 'i9', 'Sintel', 'Movie', 5000, 8000, 3000, NULL, 6000),
               (12, 'streamystats', 'u1', 'alice', 'i9', 'Sintel', 'Movie', 9000, 15000, 6000, NULL, 6000),
               (13, 'jellystat',    'u2', 'bob',   'i9', 'Sintel', 'Movie', 9000, 15000, 6000, NULL, 6000);",
        )
        .unwrap();
    }

    #[test]
    fn an_imported_play_stops_where_its_time_watched_ends_and_a_live_one_where_it_stopped() {
        let c = conn();
        with_plays(&c);
        let cond = everyone().cond().with("p.item_id = ?", "i9".to_string());
        let ins = item_insights(&c, &cond, Some(6000)).unwrap();
        assert_eq!(ins["plays"], 4);
        assert_eq!((ins["measured"].as_i64(), ins["estimated"].as_i64()), (Some(1), Some(3)), "said apart, never mixed silently");
        assert_eq!((ins["bucket_s"].as_i64(), ins["runtime_s"].as_i64()), (Some(120), Some(6000)), "the axis the curve is drawn on travels with it");
        let curve = ins["curve"].as_array().unwrap();
        assert_eq!(curve.len(), 51);
        assert_eq!(curve[5], 1.0, "at ten minutes all four are still going");
        assert_eq!(curve[6], 0.75, "the live play stopped at ten minutes, whatever it had watched");
        assert_eq!(curve[25], 0.75, "at fifty minutes the imported one is still counted");
        assert_eq!(curve[26], 0.5);
        assert_eq!(curve[50], 0.5, "two reached the end");
    }

    #[test]
    fn only_a_seek_that_went_backwards_is_a_rewind() {
        let c = conn();
        with_plays(&c);
        c.execute_batch(
            "INSERT INTO playback_events(playback_id, at, kind, position_s, from_s, detail) VALUES
               (10, 1100, 'seek', 1265, 1400, '23:20 → 21:05'),   -- back: a rewind, landing at 21:05
               (10, 1200, 'seek', 900, 100, '1:40 → 15:00'),      -- forward: a skip, not a rewind
               (10, 1300, 'seek', 500, NULL, 'garbled'),          -- no origin known: says nothing
               (10, 1400, 'pause', 1300, NULL, NULL);",
        )
        .unwrap();
        let ins = item_insights(&c, &everyone().cond().with("p.item_id = ?", "i9".to_string()), Some(6000)).unwrap();
        let rewinds = ins["rewinds"].as_array().unwrap();
        assert_eq!(rewinds.len(), 50, "one count per bucket, on the curve's grid");
        assert_eq!(rewinds.iter().map(|v| v.as_i64().unwrap()).sum::<i64>(), 1);
        assert_eq!(rewinds[1265 / 120], 1, "counted where it landed");
    }

    #[test]
    fn a_subtitle_turned_off_is_not_turned_on_and_only_the_first_change_counts() {
        let c = conn();
        with_plays(&c);
        c.execute_batch(
            "INSERT INTO playback_events(playback_id, at, kind, position_s, from_s, detail) VALUES
               (10, 1100, 'subtitle', 150, NULL, 'eng (subrip)'),  -- on at 2:30, then off again: one switch-on
               (10, 1200, 'subtitle', 900, NULL, 'Off'),
               (11, 5100, 'subtitle', 200, NULL, 'Off'),           -- only ever turned off: none
               (12, 9100, 'subtitle', 100, NULL, 'nor (subrip)'),  -- a language change later is not a second switch-on
               (12, 9200, 'subtitle', 400, NULL, 'eng (subrip)');",
        )
        .unwrap();
        let ins = item_insights(&c, &everyone().cond().with("p.item_id = ?", "i9".to_string()), Some(6000)).unwrap();
        let subs = ins["subtitles"].as_array().unwrap();
        assert_eq!(subs.len(), 50);
        assert_eq!(subs.iter().map(|v| v.as_i64().unwrap()).sum::<i64>(), 2);
        assert_eq!((&subs[150 / 120], &subs[100 / 120]), (&json!(1), &json!(1)), "one in each of the first two buckets");
    }

    #[test]
    fn a_show_says_how_many_people_started_each_episode_and_how_many_finished() {
        // "Everyone quits episode three" is people who never press play on episode four: a fact
        // between episodes, so each episode counts who started it, and how many of its plays finished.
        let c = conn();
        c.execute_batch(
            "INSERT INTO items(id, type, name, series_id, parent_index_number, index_number, runtime_s, updated_at) VALUES
               ('s1', 'Series', 'Sintel: the show', NULL, NULL, NULL, NULL, 1),
               ('e1', 'Episode', 'One', 's1', 1, 1, 1200, 1), ('e2', 'Episode', 'Two', 's1', 1, 2, 1200, 1), ('e3', 'Episode', 'Three', 's1', 1, 3, 1200, 1);
             INSERT INTO playbacks(source, user_id, user_name, item_id, item_name, item_type, series_id, started_at, ended_at, duration_s, position_s, runtime_s) VALUES
               ('live',      'u1', 'alice', 'e1', 'One', 'Episode', 's1', 100, 1300, 1200, 1200, 1200),
               ('jellystat', 'u2', 'bob',   'e1', 'One', 'Episode', 's1', 100, 1250, 1150, NULL, 1200),
               ('live',      'u3', 'cat',   'e1', 'One', 'Episode', 's1', 100, 1300, 1200, 1100, 1200),
               ('live',      'u1', 'alice', 'e2', 'Two', 'Episode', 's1', 2000, 3200, 1200, 1200, 1200),
               ('live',      'u2', 'bob',   'e2', 'Two', 'Episode', 's1', 2000, 2600, 600, 600, 1200),
               ('live',      'u3', 'cat',   'e2', 'Two', 'Episode', 's1', 2000, 2100, 100, 100, 1200),
               ('live',      'u3', 'cat',   'e2', 'Two', 'Episode', 's1', 2200, 2300, 100, 100, 1200),
               ('live',      'u1', 'alice', 'e3', 'Three', 'Episode', 's1', 4000, 4100, 100, 100, 1200);",
        )
        .unwrap();
        let cond = everyone().cond().with("p.series_id = ?", "s1".to_string());
        let seasons = series_seasons(&c, &cond, "s1", false).unwrap();
        let eps = seasons[0]["episodes"].as_array().unwrap();
        let col = |k: &str| eps.iter().map(|e| e[k].as_i64().unwrap()).collect::<Vec<_>>();
        assert_eq!(col("plays"), [3, 4, 1]);
        assert_eq!(col("users"), [3, 3, 1], "cat's two tries at episode two are one viewer");
        assert_eq!(col("finished"), [3, 1, 0], "bob's imported play ran 1150 of 1200 s: finished");
    }

    fn with_files(c: &Connection) {
        // A broken file: three tries by two people on two apps, none past thirty seconds, and one try
        // still going. A control that somebody once got 45 s into. Both films.
        c.execute_batch(
            "INSERT INTO playbacks(id, source, user_id, user_name, item_id, item_name, item_type, started_at, ended_at, duration_s, position_s, runtime_s, client, active) VALUES
               (20, 'live', 'u1', 'alice', 'i7', 'Broken Reel', 'Movie', 1000, 1008, 8, 8, 5400, 'Jellyfin Web', 0),
               (21, 'live', 'u2', 'bob',   'i7', 'Broken Reel', 'Movie', 2000, 2012, 12, 12, 5400, 'Kodi', 0),
               (22, 'live', 'u2', 'bob',   'i7', 'Broken Reel', 'Movie', 3000, 3005, 5, 5, 5400, 'Kodi', 0),
               (23, 'live', 'u1', 'alice', 'i7', 'Broken Reel', 'Movie', 4000, 4003, 3, 3, 5400, 'Kodi', 1),
               (24, 'live', 'u1', 'alice', 'i8', 'Fine Reel', 'Movie', 1000, 1008, 8, 8, 5400, 'Kodi', 0),
               (25, 'live', 'u2', 'bob',   'i8', 'Fine Reel', 'Movie', 2000, 2045, 45, 45, 5400, 'Kodi', 0),
               (26, 'live', 'u2', 'bob',   'i8', 'Fine Reel', 'Movie', 3000, 3005, 5, 5, 5400, 'Kodi', 0);",
        )
        .unwrap();
    }

    #[test]
    fn the_file_lists_are_empty_for_somebody_who_may_only_see_themselves() {
        let c = conn();
        with_files(&c);
        let mine = Scope { user_ids: vec!["u1".into()], perms: crate::auth::Perms { see_everyone: false, ..Default::default() }, ..everyone() };
        let out = file_signals_for(&c, &mine).unwrap();
        for key in ["broken", "rewound", "subtitled"] {
            assert_eq!(out[key], json!([]), "{key}: a list of files built from one person's own plays is noise, and names other people's viewing");
        }
    }

    #[test]
    fn a_file_nobody_gets_thirty_seconds_into_is_listed_with_who_tried_it() {
        let c = conn();
        with_files(&c);
        let out = file_signals_for(&c, &everyone()).unwrap();
        let broken = out["broken"].as_array().unwrap();
        assert_eq!(broken.len(), 1, "the control was got 45 s into once: {broken:?}");
        let b = &broken[0];
        assert_eq!((b["id"].as_str(), b["name"].as_str(), b["type"].as_str()), (Some("i7"), Some("Broken Reel"), Some("Movie")));
        assert_eq!((b["plays"].as_i64(), b["users"].as_i64(), b["longest_s"].as_i64(), b["last_tried_at"].as_i64()), (Some(3), Some(2), Some(12), Some(3000)), "the try still going is not counted");
        let mut clients: Vec<&str> = b["clients"].as_array().unwrap().iter().map(|v| v.as_str().unwrap()).collect();
        clients.sort();
        assert_eq!(clients, ["Jellyfin Web", "Kodi"]);
    }

    #[test]
    fn the_broken_file_list_ignores_the_minimum_play_length() {
        // The minimum play length is exactly what a broken file never reaches.
        let c = conn();
        with_files(&c);
        let strict = Scope { min_play_s: 60, ..everyone() };
        assert_eq!(file_signals_for(&c, &strict).unwrap()["broken"].as_array().unwrap().len(), 1);
    }

    fn with_rewinds(c: &Connection) {
        c.execute_batch(
            "INSERT INTO playbacks(id, source, user_id, user_name, item_id, item_name, item_type, started_at, ended_at, duration_s, position_s, runtime_s) VALUES
               (30, 'live', 'u1', 'alice', 'x', 'Mumbled', 'Movie', 1000, 6000, 5000, 5000, 5400),
               (31, 'live', 'u2', 'bob',   'x', 'Mumbled', 'Movie', 1000, 6000, 5000, 5000, 5400),
               (32, 'live', 'u1', 'alice', 'y', 'Clear', 'Movie', 1000, 6000, 5000, 5000, 5400),
               (33, 'live', 'u2', 'bob',   'y', 'Clear', 'Movie', 1000, 6000, 5000, 5000, 5400),
               (34, 'live', 'u3', 'cat',   'y', 'Clear', 'Movie', 1000, 6000, 5000, 5000, 5400),
               (35, 'live', 'u1', 'alice', 'z', 'Quiet', 'Movie', 1000, 6000, 5000, 5000, 5400),
               (36, 'live', 'u2', 'bob',   'z', 'Quiet', 'Movie', 1000, 6000, 5000, 5000, 5400);
             INSERT INTO playback_events(playback_id, at, kind, position_s, from_s, detail) VALUES
               (30, 1100, 'seek', 1260, 1400, '23:20 → 21:00'), (30, 1200, 'seek', 1290, 1420, '23:40 → 21:30'),
               (31, 1100, 'seek', 1270, 1400, '23:20 → 21:10'), (31, 1200, 'seek', 1280, 1400, '23:20 → 21:20'),
               (31, 1300, 'seek', 4000, 1500, '25:00 → 66:40'),
               (32, 1100, 'seek', 100, 400, '6:40 → 1:40'), (33, 1100, 'seek', 100, 400, '6:40 → 1:40'), (34, 1100, 'seek', 3000, 3200, '53:20 → 50:00'),
               (35, 1100, 'seek', 100, 400, '6:40 → 1:40'), (36, 1100, 'seek', 100, 400, '6:40 → 1:40');",
        )
        .unwrap();
    }

    #[test]
    fn most_rewound_ranks_by_rewinds_per_play_and_names_the_hot_spot() {
        let c = conn();
        with_rewinds(&c);
        let out = file_signals_for(&c, &everyone()).unwrap();
        let rows = out["rewound"].as_array().unwrap();
        let names: Vec<&str> = rows.iter().map(|r| r["name"].as_str().unwrap()).collect();
        assert_eq!(names, ["Mumbled", "Clear"], "two rewinds are not a pattern; the skip ahead is not a rewind");
        assert_eq!((rows[0]["plays"].as_i64(), rows[0]["rewinds"].as_i64(), rows[0]["per_play"].as_f64(), rows[0]["hot_s"].as_i64()), (Some(2), Some(4), Some(2.0), Some(1260)));
        assert_eq!((rows[1]["rewinds"].as_i64(), rows[1]["per_play"].as_f64()), (Some(3), Some(1.0)));
    }

    #[test]
    fn subtitles_switched_on_counts_the_first_ten_minutes_only() {
        let c = conn();
        with_rewinds(&c);
        c.execute_batch(
            "INSERT INTO playback_events(playback_id, at, kind, position_s, from_s, detail) VALUES
               (32, 1150, 'subtitle', 200, NULL, 'eng (subrip)'), (33, 1150, 'subtitle', 500, NULL, 'eng (subrip)'), (34, 1150, 'subtitle', 900, NULL, 'eng (subrip)'),
               (30, 1150, 'subtitle', 100, NULL, 'Off');",
        )
        .unwrap();
        let out = file_signals_for(&c, &everyone()).unwrap();
        let rows = out["subtitled"].as_array().unwrap();
        assert_eq!(rows.len(), 1, "{rows:?}");
        assert_eq!((rows[0]["name"].as_str(), rows[0]["plays"].as_i64(), rows[0]["switched_on"].as_i64(), rows[0]["share"].as_f64(), rows[0]["typical_s"].as_i64()),
                   (Some("Clear"), Some(3), Some(2), Some(0.67), Some(350)));
    }

    #[test]
    fn the_rewind_list_reads_seeks_through_the_kind_index_rather_than_every_event() {
        // Events are the biggest table on a long history and seeks a small part of it.
        let c = conn();
        let plan: Vec<String> = c
            .prepare(&format!("EXPLAIN QUERY PLAN {}", rewound_sql("")))
            .unwrap()
            .query_map([], |r| r.get::<_, String>(3))
            .unwrap()
            .map(Result::unwrap)
            .collect();
        assert!(plan.iter().any(|p| p.contains("idx_pbe_kind")), "{plan:?}");
        assert!(!plan.iter().any(|p| p.contains("SCAN playback_events")), "{plan:?}");
    }

    #[test]
    fn fewer_than_three_plays_or_no_runtime_means_no_curve_at_all() {
        let c = conn();
        // Three plays of i1 from `conn()`, but nothing knows how long it is.
        let cond = everyone().cond().with("p.item_id = ?", "i1".to_string());
        assert!(item_insights(&c, &cond, None).unwrap().is_null(), "no runtime, no axis");
        assert!(item_insights(&c, &cond, Some(0)).unwrap().is_null());
        // The runtime the plays carry will do when the item has none.
        c.execute_batch("UPDATE playbacks SET runtime_s = 600 WHERE id = 1").unwrap();
        assert_eq!(item_insights(&c, &cond, None).unwrap()["plays"], 3);
        c.execute_batch("DELETE FROM playbacks WHERE id = 1").unwrap();
        assert!(item_insights(&c, &cond, Some(600)).unwrap().is_null(), "two plays are an anecdote");
    }

    #[test]
    fn a_viewer_without_see_everyone_gets_a_curve_of_their_own_plays_only() {
        let c = conn();
        with_plays(&c);
        let mine = Scope { user_ids: vec!["u1".into()], perms: crate::auth::Perms { see_everyone: false, ..Default::default() }, ..everyone() };
        let ins = item_insights(&c, &mine.cond().with("p.item_id = ?", "i9".to_string()), Some(6000)).unwrap();
        assert_eq!(ins["plays"], 3, "bob's play is not on alice's curve");
    }

    #[test]
    fn a_scope_of_several_people_holds_to_those_people() {
        let c = conn();
        let two = |ids: &[&str]| -> i64 {
            let scope = Scope { days: 0, since: None, user_ids: ids.iter().map(|s| s.to_string()).collect(), library_id: None, min_play_s: 0, perms: crate::auth::Perms::ALL };
            let cond = scope.cond();
            c.query_row(&format!("SELECT COUNT(*) FROM playbacks p {}", cond.sql()), params_from_iter(cond.args.iter()), |r| r.get(0)).unwrap()
        };
        assert_eq!(two(&[]), 3, "nobody named means everybody");
        assert_eq!(two(&["u1"]), 2);
        assert_eq!(two(&["u1", "u2"]), 3);
        assert_eq!(two(&["u2"]), 1);
        assert_eq!(two(&["nobody"]), 0);
    }

    #[test]
    fn the_media_type_filter_can_ask_for_several_kinds_and_for_the_rest() {
        // "Other" is not a type but the absence of the three finstats names by themselves, so it
        // cannot go in the same IN list — and asking for Movies *and* Other has to mean either.
        assert_eq!(type_clause(&[]), None);
        let (sql, vals) = type_clause(&["Movie".into(), "Episode".into()]).unwrap();
        assert_eq!(sql, "(p.item_type IN (?, ?))");
        assert_eq!(vals, ["Movie", "Episode"]);
        let (sql, vals) = type_clause(&["Other".into()]).unwrap();
        assert_eq!(sql, "(p.item_type NOT IN ('Movie', 'Episode', 'Audio'))");
        assert!(vals.is_empty());
        let (sql, vals) = type_clause(&["Movie".into(), "Other".into()]).unwrap();
        assert_eq!(sql, "(p.item_type IN (?) OR p.item_type NOT IN ('Movie', 'Episode', 'Audio'))");
        assert_eq!(vals, ["Movie"]);
        // Asking for all four is asking for everything, which is no filter at all.
        assert_eq!(type_clause(&["Movie".into(), "Episode".into(), "Audio".into(), "Other".into()]), None);
    }

    #[test]
    fn only_a_tracker_finstats_knows_can_be_filtered_on() {
        // The value reaches SQL, so it is chosen from a list rather than passed through. Anything
        // else means "all of them", which is what the filter shows when it is not set.
        assert_eq!(source_filter(Some("live")), Some("live"));
        assert_eq!(source_filter(Some("jellystat")), Some("jellystat"));
        assert_eq!(source_filter(Some("streamystats")), Some("streamystats"));
        for all in [None, Some(""), Some("all"), Some("LIVE"), Some("' OR 1=1 --"), Some("jellystat' --")] {
            assert_eq!(source_filter(all), None, "{all:?}");
        }
    }

    #[test]
    fn the_trackers_a_history_came_from_are_listed_in_one_order_and_only_where_they_are() {
        let c = conn();
        // Always the same order, whatever order the rows are in, so the filter never reshuffles.
        assert_eq!(sources_present(&c, &Cond::default()).unwrap(), ["live", "jellystat", "streamystats"]);
        // Somebody who may only see their own plays is told only about their own history: bob's
        // Streamystats rows are not alice's business, and a filter for them would say they exist.
        let mut alice = Cond::default();
        alice.add("p.user_id = ?", "u1".to_string());
        assert_eq!(sources_present(&c, &alice).unwrap(), ["live", "jellystat"]);
        let mut bob = Cond::default();
        bob.add("p.user_id = ?", "u2".to_string());
        assert_eq!(sources_present(&c, &bob).unwrap(), ["streamystats"]);
        // An install that has never imported anything has nothing to choose between.
        c.execute("DELETE FROM playbacks WHERE source <> 'live'", []).unwrap();
        assert_eq!(sources_present(&c, &Cond::default()).unwrap(), ["live"]);
    }

    #[test]
    fn asking_which_trackers_a_history_came_from_never_reads_the_whole_table() {
        // It runs on every page of Activity, so it must be a seek per tracker rather than a scan:
        // on a long history three scans would cost more than the page itself.
        let c = conn();
        let plan: Vec<String> = c
            .prepare("EXPLAIN QUERY PLAN SELECT EXISTS(SELECT 1 FROM playbacks p WHERE p.source = 'jellystat')")
            .unwrap()
            .query_map([], |r| r.get::<_, String>(3))
            .unwrap()
            .map(Result::unwrap)
            .collect();
        assert!(plan.iter().any(|p| p.contains("idx_pb_source")), "{plan:?}");
        assert!(!plan.iter().any(|p| p.contains("SCAN playbacks")), "{plan:?}");
    }

    #[test]
    fn a_search_word_can_never_grow_into_a_pattern_sqlite_refuses() {
        // SQLite's own limit is 50 000 characters; over it, LIKE is an error and the request a 500.
        let long = "x".repeat(200_000);
        let words = like_words(Some(&long));
        assert_eq!(words.len(), 1);
        assert_eq!(words[0].len(), 102, "%…% around 100 characters");
        // Only the first eight words are used, and each is escaped so that % and _ are not wildcards.
        let many = (0..20).map(|i| format!("w{i}")).collect::<Vec<_>>().join(" ");
        assert_eq!(like_words(Some(&many)).len(), 8);
        assert_eq!(like_words(Some("100%_a\\b")), vec![r"%100\%\_a\\b%"]);
        assert!(like_words(None).is_empty() && like_words(Some("   ")).is_empty());
        // Escaping happens after the cut, so a trim can never leave half of an escape pair behind.
        let backslashes = "\\".repeat(200);
        let one = &like_words(Some(&backslashes))[0];
        assert_eq!(one.matches("\\\\").count(), 100, "{one}");
    }

    #[test]
    fn order_by_only_accepts_known_columns() {
        let default = "p.ended_at DESC, p.id DESC";
        assert_eq!(order_by(&ACTIVITY_SORTS, Some("watched"), Some("asc"), default), "(p.duration_s) IS NULL, p.duration_s ASC, p.ended_at DESC, p.id DESC");
        assert!(order_by(&ACTIVITY_SORTS, Some("watched"), Some("sideways"), default).contains("p.duration_s DESC"));
        // Anything that is not on the list never reaches the SQL.
        assert_eq!(order_by(&ACTIVITY_SORTS, Some("p.id; DROP TABLE playbacks"), Some("asc"), default), default);
        assert_eq!(order_by(&ACTIVITY_SORTS, None, None, default), default);
    }
}
