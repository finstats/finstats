//! What is arriving right now, from Sonarr's and Radarr's queues.
//!
//! They already talk to the download client, whichever it is, and report the same things for a torrent as for
//! a usenet download: what it is, how far along, and what went wrong on import. That is what FinStats reads,
//! rather than each client's own API: three services to keep up with instead of six, and nothing to set up
//! twice. What is lost is what only a client knows (ratio, peers), and the speed, which is worked out here
//! instead: bytes that moved between two readings, divided by the time between them.
//!
//! Two things fall out of the queue's shape, and `fold` is where they live:
//! - A season pack is several records with one `downloadId`: one download, not five.
//! - A record without one (Sonarr is between clients, or the client forgot it) still describes itself.
//!
//! The snapshot is kept in memory and never in the database: it is worthless a minute later, and nothing here
//! touches the database per tick.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::auth::DownloadsViewer;
use crate::db::rusqlite::Connection;
use crate::services::{self, Kind};
use crate::state::{ApiResult, App};

/// While somebody is looking, while nobody is but something is in the queue, and while the queue is empty.
const WATCHED_EVERY_S: u64 = 5;
const IDLE_EVERY_S: u64 = 60;
const EMPTY_EVERY_S: u64 = 300;
/// A page still counts as being watched this long after it last asked.
const WATCHING_FOR_S: i64 = 20;
const PER_SERVICE_TIMEOUT: Duration = Duration::from_secs(4);
/// Never list more than this, whatever a client is doing.
const MAX_ROWS: usize = 500;

// ---------------------------------------------------------------- the two halves

/// One record of a Sonarr or Radarr queue.
#[derive(Debug, Clone, PartialEq)]
pub struct Queued {
    pub service_id: i64,
    pub service_name: String,
    pub arr_media_id: Option<i64>,
    pub download_id: Option<String>,
    /// What Sonarr or Radarr is waiting for: "Low Orbit · S03E09", "Northern Static".
    pub title: String,
    pub sub: Option<String>,
    pub media_type: &'static str,
    pub tmdb_id: Option<i64>,
    pub tvdb_id: Option<i64>,
    pub season: Option<i64>,
    pub size: i64,
    pub left: i64,
    pub eta_s: Option<i64>,
    pub state: &'static str,
    pub protocol: Option<String>,
    pub client: Option<String>,
    /// What the release is called: the queue's own title.
    pub release: Option<String>,
    pub error: Option<String>,
}

/// Parameter names whose value is somebody's credential, as indexers, trackers, Jackett and download clients spell them.
/// Newznab's `r` and `i` and IPTorrents' `tp` and `u` are a key and a user id. A value that looks like a key is blanked
/// whatever it is called (`looks_like_key`), so this list is the short ones a shape cannot tell.
const SECRET_PARAMS: [&str; 27] = [
    "apikey", "api_key", "api-key", "api.key", "apitoken", "key", "passkey", "authkey", "torrent_pass", "rsskey", "token", "access_token", "auth", "password", "pass", "pwd",
    "secret", "secret_key", "sig", "signature", "jackett_apikey", "r", "i", "uid", "pid", "tp", "u",
];

/// A part of an address's path this long and all letters and digits is a key, not a name: private trackers put the
/// passkey there (`/0123…cdef/announce`). Where it is a hash instead, more is blanked than needed, never less.
const PATH_KEY_LEN: usize = 20;
/// A value this long and all letters and digits is a key, whatever its name or wherever it stands.
const KEY_LEN: usize = 16;

/// Does this look like a key: long, letters and digits only, and either all hexadecimal or both letters and digits? A word
/// as long is all letters, and a release name has dots between its parts.
fn looks_like_key(s: &str) -> bool {
    s.len() >= KEY_LEN
        && s.bytes().all(|b| b.is_ascii_alphanumeric())
        && (s.bytes().all(|b| b.is_ascii_hexdigit()) || (s.bytes().any(|b| b.is_ascii_digit()) && s.bytes().any(|b| b.is_ascii_alphabetic())))
}

/// A part of a path that holds a key: one of its pieces between `.` and `-` looks like one (`Name.2026.<key>`), or with its
/// dashes taken out it is a long hexadecimal one (`0123-4567-…`), or it is long and nothing but letters and digits.
fn path_part_is_key(part: &str) -> bool {
    let undashed: String = part.chars().filter(|c| *c != '-').collect();
    (part.len() >= PATH_KEY_LEN && part.bytes().all(|b| b.is_ascii_alphanumeric()))
        || part.split(['.', '-']).any(looks_like_key)
        || (undashed.len() >= 32 && undashed.bytes().all(|b| b.is_ascii_hexdigit()))
}

/// The address after its login: the host as it is, and every part of its path that is a key blanked.
fn blank_path_keys(addr: &str) -> String {
    let (host, path) = addr.split_at(addr.find('/').unwrap_or(addr.len()));
    let path: Vec<&str> = path.split('/').map(|part| if path_part_is_key(part) { "***" } else { part }).collect();
    format!("{host}{}", path.join("/"))
}

/// A service's own words with the credentials in them blanked: the login in an address (`http://user:pass@host`), the
/// value of any query parameter named like a key or a password or looking like a key, and any part of a path that looks
/// like one. Sonarr and Radarr quote the download client and the indexer they could not reach, addresses and all, and
/// those words are shown to everybody who may see the queue. That is who may see what is downloading, not the keys it is
/// downloading with.
pub fn redact(s: &str) -> String {
    let stop = |c: char| c.is_whitespace() || matches!(c, '?' | '#' | ')' | '(' | '"' | '\'' | ',' | '<' | '>');
    // The login part of every address. It ends at the last '@' before the address's end, or before the next '/' when that
    // is further: a password printed as it was typed may hold a '/', a ',' or a space. Where a path or the words after an
    // address have an '@' of their own, more is blanked than needed, never less.
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(i) = rest.find("://") {
        let (head, tail) = rest.split_at(i + 3);
        out.push_str(head);
        let end = tail.find(stop).unwrap_or(tail.len());
        let slash = tail.find('/').unwrap_or(tail.len());
        match [tail[..end].rfind('@'), tail[..slash].rfind('@')].into_iter().flatten().max() {
            Some(at) => {
                out.push_str("***@");
                let after = &tail[at + 1..];
                let end = after.find(stop).unwrap_or(after.len());
                out.push_str(&blank_path_keys(&after[..end]));
                rest = &after[end..];
            }
            None => {
                out.push_str(&blank_path_keys(&tail[..end]));
                rest = &tail[end..];
            }
        }
    }
    out.push_str(rest);
    // Then every `?name=value`, `&name=value` and `;name=value` whose name is a credential's or whose value looks like one.
    let value_stop = |c: char| c.is_whitespace() || matches!(c, '&' | ';' | '#' | ')' | '"' | '\'' | ',' | '<' | '>');
    let mut done = String::with_capacity(out.len());
    let mut rest = out.as_str();
    while let Some(i) = rest.find(['?', '&', ';']) {
        let (head, tail) = rest.split_at(i + 1);
        done.push_str(head);
        let name_end = tail.find(|c: char| !(c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))).unwrap_or(tail.len());
        let (name, after) = tail.split_at(name_end);
        let value = after.strip_prefix('=').unwrap_or("");
        let end = value.find(value_stop).unwrap_or(value.len());
        if after.starts_with('=') && (SECRET_PARAMS.iter().any(|k| k.eq_ignore_ascii_case(name)) || looks_like_key(&value[..end])) {
            done.push_str(name);
            done.push_str("=***");
            rest = &value[end..];
        } else {
            rest = tail;
        }
    }
    done.push_str(rest);
    done
}

fn text(v: &Value) -> Option<String> {
    v.as_str().map(str::trim).filter(|s| !s.is_empty()).map(str::to_string)
}

/// `00:12:34` or `1.02:03:04` (a .NET timespan) in seconds. `None` for anything that is not one, a number too large
/// to be one included.
pub fn timespan(v: &Value) -> Option<i64> {
    let raw = v.as_str()?;
    let (days, rest): (i64, &str) = match raw.split_once('.') {
        Some((d, rest)) if d.parse::<i64>().is_ok() && rest.contains(':') => (d.parse().unwrap_or(0), rest),
        _ => (0, raw),
    };
    let parts: Vec<i64> = rest.split(':').map(|p| p.split('.').next().unwrap_or("0").parse().unwrap_or(0)).collect();
    let [h, m, s] = parts[..] else { return None };
    days.checked_mul(24)?.checked_add(h)?.checked_mul(60)?.checked_add(m)?.checked_mul(60)?.checked_add(s)
}

/// What a queue record is doing, in FinStats' words. Importing and failing matter: that is where things get stuck.
pub fn queue_state(status: &str, tracked_state: &str, tracked_status: &str) -> &'static str {
    match tracked_state {
        "importPending" | "importBlocked" | "importing" => return "importing",
        "failed" | "failedPending" => return "failed",
        _ => {}
    }
    match status {
        "downloading" => "downloading",
        "queued" | "delay" => "queued",
        "paused" => "paused",
        "completed" => "importing",
        "failed" => "failed",
        "warning" | "downloadClientUnavailable" => {
            if tracked_status == "error" {
                "failed"
            } else {
                "stalled"
            }
        }
        _ => "unknown",
    }
}

fn queue_row(svc_id: i64, svc_name: &str, kind: Kind, r: &Value) -> Option<Queued> {
    let size = r["size"].as_f64().unwrap_or(0.0).max(0.0) as i64;
    let left = r["sizeleft"].as_f64().unwrap_or(0.0).max(0.0) as i64;
    let error = text(&r["errorMessage"]).or_else(|| r["statusMessages"].as_array().and_then(|a| a.first().and_then(|m| m["title"].as_str().map(str::to_string)))).map(|e| redact(&e));
    let (series, movie) = (&r["series"], &r["movie"]);
    let (title, sub, media_type, tmdb, tvdb, arr_media_id) = if kind == Kind::Sonarr {
        let episode = &r["episode"];
        let code = match (r["seasonNumber"].as_i64().or_else(|| episode["seasonNumber"].as_i64()), episode["episodeNumber"].as_i64()) {
            (Some(s), Some(e)) => Some(format!("S{s:02}E{e:02}")),
            (Some(s), None) => Some(season_line(SOURCE, s)),
            _ => None,
        };
        let name = text(&series["title"]).or_else(|| text(&r["title"])).unwrap_or_else(|| "Unknown".into());
        let sub = [code, text(&episode["title"])].into_iter().flatten().collect::<Vec<_>>().join(" · ");
        (name, (!sub.is_empty()).then_some(sub), "tv", series["tmdbId"].as_i64(), series["tvdbId"].as_i64(), r["seriesId"].as_i64())
    } else {
        let name = text(&movie["title"]).or_else(|| text(&r["title"])).unwrap_or_else(|| "Unknown".into());
        (name, movie["year"].as_i64().map(|y| y.to_string()), "movie", movie["tmdbId"].as_i64(), None, r["movieId"].as_i64())
    };
    Some(Queued {
        service_id: svc_id,
        service_name: svc_name.to_string(),
        arr_media_id: arr_media_id.filter(|n| *n > 0),
        download_id: text(&r["downloadId"]).map(|d| d.to_ascii_lowercase()),
        title,
        sub,
        media_type,
        tmdb_id: tmdb.filter(|n| *n > 0),
        tvdb_id: tvdb.filter(|n| *n > 0),
        season: r["seasonNumber"].as_i64(),
        size,
        left,
        eta_s: timespan(&r["timeleft"]),
        state: queue_state(r["status"].as_str().unwrap_or(""), r["trackedDownloadState"].as_str().unwrap_or(""), r["trackedDownloadStatus"].as_str().unwrap_or("")),
        protocol: text(&r["protocol"]),
        client: text(&r["downloadClient"]),
        release: text(&r["title"]),
        error,
    })
}

async fn read_queue(app: &App, svc: &services::Service) -> Result<Vec<Queued>> {
    let mut query = vec![("pageSize", "200".to_string()), ("page", "1".to_string())];
    if svc.kind == Kind::Sonarr {
        query.push(("includeSeries", "true".into()));
        query.push(("includeEpisode", "true".into()));
        query.push(("includeUnknownSeriesItems", "true".into()));
    } else {
        query.push(("includeMovie", "true".into()));
        query.push(("includeUnknownMovieItems", "true".into()));
    }
    let body = services::get_json_within(app, svc, "/api/v3/queue", &query, QUEUE_LIMITS).await?;
    Ok(queue_records(svc.id, &svc.name, svc.kind, &body))
}

/// A queue page of 200 is well under a megabyte; nothing a queue says is needed past what is ever listed.
const QUEUE_LIMITS: services::Limits = services::Limits { bytes: 8 << 20, items: 1_000, nodes: 500_000 };

/// A queue page's records as rows, no more of them than are ever listed, read where they lie.
fn queue_records(svc_id: i64, svc_name: &str, kind: Kind, body: &Value) -> Vec<Queued> {
    body["records"].as_array().map(|a| a.iter().filter_map(|r| queue_row(svc_id, svc_name, kind, r)).take(MAX_ROWS).collect()).unwrap_or_default()
}
// ---------------------------------------------------------------- one download out of its records

#[derive(Debug, Clone, PartialEq)]
pub struct Download {
    pub key: String,
    pub title: String,
    pub sub: Option<String>,
    pub state: &'static str,
    pub progress: f64,
    pub size: i64,
    /// Bytes still to come: what the speed is worked out from.
    pub left: i64,
    pub eta_s: Option<i64>,
    /// Worked out from two readings, because a queue reports no speed.
    pub down_bps: i64,
    /// What the release is called. Only ever shown to somebody who may see downloads.
    pub release: Option<String>,
    /// The download client Sonarr or Radarr handed it to.
    pub client: Option<String>,
    pub protocol: Option<String>,
    pub service_name: Option<String>,
    pub arr: Option<(i64, i64)>,
    pub media_type: Option<&'static str>,
    pub tmdb_id: Option<i64>,
    pub tvdb_id: Option<i64>,
    pub seasons: Vec<i64>,
    /// How many queue records share this download: a season pack is one download and many episodes.
    pub parts: usize,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Totals {
    pub down_bps: i64,
    pub downloading: usize,
    pub queued: usize,
    pub importing: usize,
    pub failed: usize,
}

/// The words FinStats uses for what a download is doing, worst first: what needs a person's attention is on top.
pub const STATES: [&str; 7] = ["failed", "importing", "downloading", "stalled", "queued", "paused", "checking"];

fn rank(state: &str) -> usize {
    STATES.iter().position(|s| *s == state).unwrap_or(STATES.len())
}

/// The queue is read in the background, where nobody's language is known, so its lines are written in the source's and
/// said again in the reader's when they are answered (`download_json`).
const SOURCE: &str = crate::i18n::LOCALES[0];

/// "Season 3": a record for a whole season.
fn season_line(lang: &str, season: i64) -> String {
    crate::i18n::say(lang, "server.downloads.season", &[("season", season.to_string().into())])
}

/// "Season 3 · 10 episodes": what a pack of `parts` records holds. A season is said as written (2024 is a season too).
fn pack_line(lang: &str, parts: usize, seasons: &[i64]) -> String {
    match seasons {
        [s] => crate::i18n::say(lang, "server.downloads.season_episodes", &[("season", s.to_string().into()), ("count", parts.into())]),
        [] => crate::i18n::say(lang, "server.downloads.episodes", &[("count", parts.into())]),
        seasons => crate::i18n::say(lang, "server.downloads.seasons_episodes", &[("seasons", seasons.len().to_string().into()), ("count", parts.into())]),
    }
}

/// A download's second line in the reader's language: FinStats' own lines are said again, anything else (an episode's
/// code and title) is the queue's own.
fn sub_of(d: &Download) -> Option<String> {
    let lang = crate::i18n::current();
    match d.seasons.as_slice() {
        _ if d.parts > 1 => Some(pack_line(lang, d.parts, &d.seasons)),
        [s] if d.sub.as_deref() == Some(season_line(SOURCE, *s).as_str()) => Some(season_line(lang, *s)),
        _ => d.sub.clone(),
    }
}

/// The queue records of every connected Sonarr and Radarr as one list. Pure: the loop only feeds it.
pub fn fold(queue: Vec<Queued>) -> (Vec<Download>, Totals) {
    let mut out: Vec<Download> = vec![];
    let mut at: HashMap<String, usize> = HashMap::new();
    for q in queue {
        // Several records with one download id are one download: a season pack, and the episodes it holds.
        // Without an id (a client that forgot it) a record can only stand for itself.
        let key = match &q.download_id {
            Some(id) => format!("{id}:{}", q.service_id),
            None => format!("arr:{}:{}:{}", q.service_id, q.title, q.sub.as_deref().unwrap_or("")),
        };
        if let Some(&i) = at.get(&key) {
            let d: &mut Download = &mut out[i];
            d.parts += 1;
            if let Some(s) = q.season
                && !d.seasons.contains(&s)
            {
                d.seasons.push(s);
            }
            d.size = d.size.max(q.size);
            d.left = d.left.max(q.left);
            if rank(q.state) < rank(d.state) {
                d.state = q.state;
            }
            d.error = d.error.take().or(q.error);
            continue;
        }
        at.insert(key.clone(), out.len());
        out.push(Download {
            key,
            title: q.title,
            sub: q.sub,
            state: q.state,
            progress: if q.size > 0 { ((q.size - q.left) as f64 / q.size as f64).clamp(0.0, 1.0) } else { 0.0 },
            size: q.size,
            left: q.left,
            eta_s: q.eta_s,
            down_bps: 0,
            release: q.release,
            client: q.client,
            protocol: q.protocol,
            service_name: Some(q.service_name),
            arr: q.arr_media_id.map(|m| (q.service_id, m)),
            media_type: Some(q.media_type),
            tmdb_id: q.tmdb_id,
            tvdb_id: q.tvdb_id,
            seasons: q.season.into_iter().collect(),
            parts: 1,
            error: q.error,
        });
    }

    // A pack knows what it holds only once every one of its records has been seen.
    let mut totals = Totals::default();
    for d in &mut out {
        if d.parts > 1 {
            d.seasons.sort_unstable();
            d.sub = Some(pack_line(SOURCE, d.parts, &d.seasons));
        }
        if d.size > 0 {
            d.progress = ((d.size - d.left) as f64 / d.size as f64).clamp(0.0, 1.0);
        }
        match d.state {
            "downloading" | "stalled" => totals.downloading += 1,
            "queued" | "paused" | "checking" => totals.queued += 1,
            "importing" => totals.importing += 1,
            "failed" => totals.failed += 1,
            _ => {}
        }
    }
    debug_assert!(out.iter().all(|d| STATES.contains(&d.state) || d.state == "unknown"), "a state nobody has a word for");
    out.sort_by(|a, b| rank(a.state).cmp(&rank(b.state)).then(b.progress.partial_cmp(&a.progress).unwrap_or(std::cmp::Ordering::Equal)).then_with(|| a.title.cmp(&b.title)));
    out.truncate(MAX_ROWS);
    (out, totals)
}

/// How fast it is going: what a queue never says. Bytes that moved since the last reading, divided by the
/// time between them. Nothing is claimed from a reading that is too close, too far apart, or of another size
/// (an upgrade replacing a download keeps the id but starts again).
pub fn speeds(rows: &mut [Download], before: &HashMap<String, (i64, i64)>, now: i64) -> HashMap<String, (i64, i64)> {
    for d in rows.iter_mut() {
        if let Some((left, at)) = before.get(&d.key) {
            let (moved, seconds) = (left - d.left, now - at);
            if (2..=180).contains(&seconds) && moved > 0 && moved <= d.size.max(*left) {
                d.down_bps = moved / seconds;
            }
        }
    }
    rows.iter().map(|d| (d.key.clone(), (d.left, now))).collect()
}

/// Every download's speed together. The figures are worked out from a queue's numbers, which are somebody else's.
pub fn total_bps(rows: &[Download]) -> i64 {
    rows.iter().fold(0i64, |sum, d| sum.saturating_add(d.down_bps))
}

// ---------------------------------------------------------------- the snapshot

#[derive(Default)]
pub struct Snapshot {
    pub rows: Vec<Download>,
    pub totals: Totals,
    pub at: i64,
    /// A service that did not answer, and why. Shown as a note, not as an empty list.
    pub problems: Vec<(String, services::Said)>,
    pub sources: usize,
}

/// Who asked for what, so a download can say whose wish it is. Rebuilt with the slow tick.
#[derive(Default)]
pub struct Wishes {
    by_media: HashMap<(String, i64), (Option<String>, String)>,
    item_of: HashMap<(String, i64), String>,
}

impl Wishes {
    fn read(conn: &Connection) -> Result<Wishes> {
        let mut w = Wishes::default();
        let mut stmt = conn.prepare(
            "SELECT r.media_type, r.tmdb_id, r.tvdb_id, r.user_id, COALESCE(u.name, r.seerr_user_name), r.item_id
             FROM requests r LEFT JOIN users u ON u.id = r.user_id WHERE r.removed_at IS NULL
             -- Something still being waited for wins, but a title that has arrived was still somebody's wish:
             -- what is in the queue then is an upgrade of it, and saying who asked for it is still true.
             ORDER BY (r.media_status < 5) DESC, r.requested_at",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, Option<i64>>(1)?, r.get::<_, Option<i64>>(2)?, r.get::<_, Option<String>>(3)?, r.get::<_, Option<String>>(4)?, r.get::<_, Option<String>>(5)?))
        })?;
        for row in rows {
            let (media_type, tmdb, tvdb, user_id, name, item_id) = row?;
            for (source, id) in [("tmdb", tmdb), ("tvdb", tvdb)] {
                if let Some(id) = id {
                    let key = (format!("{media_type}:{source}"), id);
                    w.by_media.entry(key.clone()).or_insert((user_id.clone(), name.clone().unwrap_or_else(|| "Somebody".into())));
                    if let Some(item) = &item_id {
                        w.item_of.entry(key).or_insert(item.clone());
                    }
                }
            }
        }
        Ok(w)
    }

    fn of(&self, d: &Download) -> (Option<&(Option<String>, String)>, Option<&String>) {
        let media_type = d.media_type.unwrap_or("");
        for (source, id) in [("tmdb", d.tmdb_id), ("tvdb", d.tvdb_id)] {
            let Some(id) = id else { continue };
            let key = (format!("{media_type}:{source}"), id);
            if let Some(who) = self.by_media.get(&key) {
                return (Some(who), self.item_of.get(&key));
            }
        }
        (None, None)
    }
}

/// `viewer`: whose wish it is is said only of their own, or of anybody's with *see everyone* (`/api/requests`' rule).
fn download_json(d: &Download, wishes: &Wishes, viewer: &crate::auth::AuthUser) -> Value {
    let (who, item) = wishes.of(d);
    let who = who.filter(|(id, _)| viewer.perms.see_everyone || id.as_deref() == Some(viewer.id.as_str()));
    let poster = match (item, d.arr) {
        (Some(id), _) => json!({ "item_id": id }),
        (None, Some((service_id, media_id))) => json!({ "service_id": service_id, "media_id": media_id }),
        _ => Value::Null,
    };
    json!({
        "key": d.key, "title": d.title, "sub": sub_of(d), "state": d.state, "progress": d.progress, "size": d.size, "left": d.left, "eta_s": d.eta_s,
        "down_bps": d.down_bps, "release": d.release, "client": d.client, "protocol": d.protocol, "service_name": d.service_name, "error": d.error, "poster": poster,
        "requested_by": who.map(|(id, name)| json!({ "user_id": id, "user_name": name })),
        "item_id": item,
    })
}

/// Is this title in the queue right now? The queue is a listing FinStats itself makes, so its posters are
/// ones it may show to the people who are allowed to see the queue in the first place.
pub fn in_queue(app: &App, service_id: i64, media_id: i64) -> bool {
    app.downloads.read().unwrap().rows.iter().any(|d| d.arr == Some((service_id, media_id)))
}

/// Add "how far along is it" to a request row. This is all somebody without `see_downloads` ever learns: never
/// what the release is called, which client has it, how fast it is going or what else is in the queue.
pub fn attach_progress(app: &App, row: &mut Value) {
    let seasons: Vec<i64> = row["seasons"].as_array().map(|a| a.iter().filter_map(Value::as_i64).collect()).unwrap_or_default();
    if let Some(p) = own_progress(app, row["media_type"].as_str().unwrap_or(""), row["tmdb_id"].as_i64(), row["tvdb_id"].as_i64(), &seasons) {
        row["download"] = p;
    }
}

/// How far the thing somebody asked for has got, if anything in the queue is it.
pub fn own_progress(app: &App, media_type: &str, tmdb: Option<i64>, tvdb: Option<i64>, seasons: &[i64]) -> Option<Value> {
    let snap = app.downloads.read().unwrap().clone();
    let want = |d: &Download| {
        d.media_type == Some(if media_type == "movie" { "movie" } else { "tv" })
            && ((tmdb.is_some() && d.tmdb_id == tmdb) || (tvdb.is_some() && d.tvdb_id == tvdb))
            && (seasons.is_empty() || d.seasons.is_empty() || d.seasons.iter().any(|s| seasons.contains(s)))
    };
    let d = snap.rows.iter().filter(|d| want(d)).min_by_key(|d| rank(d.state))?;
    Some(json!({ "state": d.state, "progress": d.progress, "eta_s": d.eta_s }))
}

// ---------------------------------------------------------------- the loop

async fn tick(app: &App, before: &HashMap<String, (i64, i64)>) -> HashMap<String, (i64, i64)> {
    let arrs = services::enabled(app, Kind::is_arr);
    if arrs.is_empty() {
        *app.downloads.write().unwrap() = Arc::new(Snapshot { at: crate::db::now(), ..Default::default() });
        return HashMap::new();
    }
    let queues = futures_util::future::join_all(arrs.iter().map(|svc| async move {
        let out = tokio::time::timeout(PER_SERVICE_TIMEOUT, read_queue(app, svc)).await.unwrap_or_else(|_| Err(services::words("server.downloads.no_answer", vec![("name", svc.name.clone())])));
        (svc.clone(), out)
    }))
    .await;

    let (mut rows, mut problems, mut sources) = (vec![], vec![], 0);
    for (svc, out) in queues {
        match out {
            Ok(records) => {
                sources += 1;
                rows.extend(records);
            }
            Err(e) => {
                tracing::debug!("{}: {e}", svc.name);
                problems.push((svc.name.clone(), services::Said::of(&e)));
            }
        }
    }
    let now = crate::db::now();
    let (mut rows, mut totals) = fold(rows);
    let after = speeds(&mut rows, before, now);
    totals.down_bps = total_bps(&rows);
    *app.downloads.write().unwrap() = Arc::new(Snapshot { rows, totals, at: now, problems, sources });
    after
}

/// How long before the queues are read again. Fast while a page is showing them; otherwise as slow as what is in
/// them allows. Something in the queue is still worth a minute (a request page tells the person who asked how far
/// along it is), but an empty queue has nothing to go out of date: reading it again in five minutes is soon enough,
/// and opening the page, connecting a service or a read of Seerr all wake the loop anyway.
pub fn wait_s(watching: bool, connected: bool, queued: bool) -> u64 {
    match (connected, watching, queued) {
        (false, ..) => IDLE_EVERY_S, // nothing to read; this costs no request
        (_, true, _) => WATCHED_EVERY_S,
        (_, false, true) => IDLE_EVERY_S,
        (_, false, false) => EMPTY_EVERY_S,
    }
}

/// Its own loop, like the collector: a queue that moves every second is no business of the 60-second scheduler.
pub async fn run(app: App) {
    let mut wishes_at = 0i64;
    // What was left of each download when it was last read, so a speed can be worked out.
    let mut seen: HashMap<String, (i64, i64)> = HashMap::new();
    // Something changed (a connection, a read of Seerr): look again, and ask who wished for what.
    let mut woken = true;
    loop {
        let anything = !services::enabled(&app, Kind::is_arr).is_empty();
        let now = crate::db::now();
        if anything {
            // Who asked for what changes slowly, and reading it is the only database work here.
            if woken || now - wishes_at >= 60 {
                wishes_at = now;
                if let Ok(w) = app.db.call(|c| Wishes::read(c)).await {
                    *app.wishes.write().unwrap() = Arc::new(w);
                }
            }
            seen = tick(&app, &seen).await;
        }
        // Fast only while a page is actually showing this.
        let watching = crate::db::now() - *app.downloads_watched.lock().unwrap() <= WATCHING_FOR_S;
        let wait = wait_s(watching, anything, !app.downloads.read().unwrap().rows.is_empty());
        woken = tokio::select! {
            _ = app.downloads_wake.notified() => true,
            _ = tokio::time::sleep(Duration::from_secs(wait)) => false,
        };
    }
}

// ---------------------------------------------------------------- API

#[derive(Deserialize)]
pub struct LiveQuery {
    /// The page is open and showing this: keep the snapshot fresh for the next few seconds.
    live: Option<String>,
}

pub async fn downloads(State(app): State<App>, DownloadsViewer(user): DownloadsViewer, Query(q): Query<LiveQuery>) -> ApiResult {
    if q.live.as_deref() == Some("1") {
        watched(&app, crate::db::now());
    }
    Ok(Json(answer(&app, &user)))
}

/// Somebody is looking at the queue (a page's `?live=1` poll, or a visible tab on the live channel): keep the snapshot
/// fresh for the next [`WATCHING_FOR_S`]. The first look after a quiet spell also wakes the loop, so a page just opened
/// is refreshed now instead of at the end of a minute-long sleep. Answers whether it woke it.
pub fn watched(app: &App, now: i64) -> bool {
    let was = std::mem::replace(&mut *app.downloads_watched.lock().unwrap(), now);
    let wake = now - was > WATCHING_FOR_S;
    if wake {
        app.downloads_wake.notify_waiters();
    }
    wake
}

/// Why a service's queue could not be read, as `viewer` may be told it. The service's own words name its address, and a
/// redirect where it points: those are an administrator's, as the connection list is. Anybody else who may see the queue
/// is told what kind of failure it was.
fn problem_for(why: &services::Said, viewer: &crate::auth::AuthUser) -> String {
    if viewer.is_admin {
        return why.now();
    }
    let key = match why {
        services::Said::Words(w) if matches!(w.key(), "server.services.timeout" | "server.downloads.no_answer") => "server.downloads.problem.timeout",
        services::Said::Words(w) if w.key() == "server.services.key_refused" => "server.downloads.problem.refused",
        _ => "server.downloads.problem.failed",
    };
    crate::i18n::t(key, &[])
}

/// `/downloads` as it is answered, by HTTP or by the live channel. The same for everybody who may see it.
pub fn answer(app: &App, viewer: &crate::auth::AuthUser) -> Value {
    let snap = app.downloads.read().unwrap().clone();
    let wishes = app.wishes.read().unwrap().clone();
    let t = &snap.totals;
    json!({
        "rows": snap.rows.iter().map(|d| download_json(d, &wishes, viewer)).collect::<Vec<_>>(),
        "totals": { "down_bps": t.down_bps, "downloading": t.downloading, "queued": t.queued, "importing": t.importing, "failed": t.failed },
        "at": snap.at, "sources": snap.sources,
        "problems": snap.problems.iter().map(|(name, why)| json!({ "service": name, "error": problem_for(why, viewer) })).collect::<Vec<_>>(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Whose wish a download is: one's own always, anybody else's only with *see everyone*, as `/api/requests` has it.
    /// Seeing the queue was enough to read "maria asked for this" on every row.
    #[test]
    fn whose_wish_a_download_is_is_said_only_to_who_may_see_it() {
        let d = Download { key: "k".into(), title: "Sintel".into(), sub: None, state: "downloading", progress: 0.5, size: 10, left: 5, eta_s: None, down_bps: 0,
            release: None, client: None, protocol: None, service_name: None, arr: None, media_type: Some("movie"), tmdb_id: Some(45745), tvdb_id: None,
            seasons: vec![], parts: 1, error: None };
        let mut w = Wishes::default();
        w.by_media.insert(("movie:tmdb".into(), 45745), (Some("u1".into()), "maria".into()));
        let viewer = |id: &str, perms: crate::auth::Perms| crate::auth::AuthUser { id: id.into(), name: id.into(), is_admin: false, perms, credential: crate::auth::Credential::Session, ip: None };
        let queue_only = crate::auth::Perms { see_downloads: true, ..Default::default() };
        assert_eq!(download_json(&d, &w, &viewer("u2", queue_only))["requested_by"], Value::Null, "somebody else's wish, without see everyone");
        assert_eq!(download_json(&d, &w, &viewer("u1", queue_only))["requested_by"]["user_name"], "maria", "one's own");
        let everyone = crate::auth::Perms { see_downloads: true, see_everyone: true, ..Default::default() };
        assert_eq!(download_json(&d, &w, &viewer("u2", everyone))["requested_by"]["user_name"], "maria");
    }

    /// Somebody looking at the queue, by a page's poll or by the live channel: the stamp that keeps it on its 5-s
    /// cadence, and a wake only for the first look in a while, so the loop is not woken on every one.
    #[test]
    fn a_look_stamps_the_queue_as_watched_and_wakes_it_only_after_a_quiet_spell() {
        let app = crate::state::test_app();
        assert!(watched(&app, 1_000), "nobody was watching: refresh now rather than after a minute");
        assert_eq!(*app.downloads_watched.lock().unwrap(), 1_000);
        assert!(!watched(&app, 1_010), "still watched: the loop is already fast");
        assert_eq!(*app.downloads_watched.lock().unwrap(), 1_010, "but the stamp moves on");
        assert!(watched(&app, 1_011 + WATCHING_FOR_S), "a look after a quiet spell wakes it again");
    }

    #[test]
    fn the_answer_is_the_snapshot_as_the_page_reads_it() {
        let app = crate::state::test_app();
        *app.downloads.write().unwrap() = Arc::new(Snapshot { at: 42, sources: 2, problems: vec![("Radarr".into(), "timed out".into())], ..Default::default() });
        let viewer = crate::auth::AuthUser { id: "u1".into(), name: "alice".into(), is_admin: true, perms: crate::auth::Perms::ALL, credential: crate::auth::Credential::Session, ip: None };
        assert_eq!(
            answer(&app, &viewer),
            json!({ "rows": [], "totals": { "down_bps": 0, "downloading": 0, "queued": 0, "importing": 0, "failed": 0 },
                    "at": 42, "sources": 2, "problems": [{ "service": "Radarr", "error": "timed out" }] })
        );
    }

    /// A few kilobytes of gzip were a million queue records, each built twice before the list was cut to 500. A queue is
    /// read within its own limits and no more of it is made into rows than is ever listed.
    #[test]
    fn a_queue_of_a_million_records_is_read_as_no_more_than_is_listed() {
        let mut million = b"{\"records\":[".to_vec();
        million.extend(std::iter::repeat_n(&b"{},"[..], 1_000_000).flatten());
        million.extend_from_slice(b"{}]}");
        assert!(million.len() <= QUEUE_LIMITS.bytes, "the byte cap alone lets it through");
        let body = services::parse_within(&million, QUEUE_LIMITS).unwrap();
        assert!(body["records"].as_array().unwrap().len() <= QUEUE_LIMITS.items);
        assert_eq!(queue_records(1, "Sonarr", Kind::Sonarr, &body).len(), MAX_ROWS);
    }

    /// A connection's address and where it redirects are an administrator's to read: seeing the queue is seeing what is
    /// downloading. Anybody else is told which service and what kind of failure; the redirect is blanked for everybody.
    #[test]
    fn a_queue_problem_names_the_service_and_never_its_address_to_who_is_not_an_administrator() {
        let svc = crate::services::tests::service(Kind::Sonarr, "http://127.0.0.1:9332/sonarr");
        let mut headers = axum::http::HeaderMap::new();
        headers.insert(axum::http::header::LOCATION, "https://sso.lan.example/oauth/authorize?state=1&apikey=leaked".parse().unwrap());
        let redirect = services::Said::of(&services::refuse(axum::http::StatusCode::FOUND, &headers, &svc));
        let down = services::Said::of(&services::words("server.services.timeout", vec![("service", "Radarr".into()), ("url", "http://127.0.0.1:7878".into())]));
        let app = crate::state::test_app();
        *app.downloads.write().unwrap() = Arc::new(Snapshot { problems: vec![("Sonarr".into(), redirect), ("Radarr".into(), down)], ..Default::default() });
        let viewer = |is_admin: bool| crate::auth::AuthUser { id: "u1".into(), name: "alice".into(), is_admin, perms: crate::auth::Perms { see_downloads: true, ..Default::default() }, credential: crate::auth::Credential::Session, ip: None };
        let said = answer(&app, &viewer(false))["problems"].to_string();
        assert!(!said.contains("127.0.0.1") && !said.contains("sso.lan.example") && !said.contains("leaked"), "{said}");
        assert!(said.contains("Sonarr") && said.contains("Radarr"), "{said}");
        let admin = answer(&app, &viewer(true))["problems"].to_string();
        assert!(admin.contains("127.0.0.1"), "an administrator reads the address they set: {admin}");
        assert!(!admin.contains("leaked"), "the redirect's key, even to an administrator: {admin}");
    }

    #[test]
    fn credentials_in_a_services_own_words_are_blanked() {
        // Sonarr and Radarr quote the download client and the indexer they could not reach, addresses and all.
        assert_eq!(redact("stalled (qBittorrent at http://admin:hunter2@10.0.0.5:8080/api/v2?apikey=abc123&x=1)"), "stalled (qBittorrent at http://***@10.0.0.5:8080/api/v2?apikey=***&x=1)");
        assert_eq!(redact("GET https://indexer.example/api?t=get&id=7&ApiKey=Zz9&passkey=p4ss failed"), "GET https://indexer.example/api?t=get&id=7&ApiKey=***&passkey=*** failed");
        assert_eq!(redact("https://tracker.example/dl?token=abc;api_key=def&password=ghi&pwd=x&auth=y&secret=z"), "https://tracker.example/dl?token=***;api_key=***&password=***&pwd=***&auth=***&secret=***");
        assert_eq!(redact("rtorrent at https://user@host/RPC2"), "rtorrent at https://***@host/RPC2", "a user name alone is still somebody's login");
        // Plain words are left alone, a key that is only part of a longer name is not a key, and nothing is cut mid-character.
        for plain in ["Not an upgrade for existing file", "One file was not imported", "mail me at someone@example.invalid", "monkey=banana&keyword=x", ""] {
            assert_eq!(redact(plain), plain);
        }
        assert_eq!(redact("Téléchargé http://ü:ö@hôte/?apikey=é&ünd=1 😀"), "Téléchargé http://***@hôte/?apikey=***&ünd=1 😀");
        assert_eq!(redact("http://a:b@"), "http://***@");
        // A password the client printed as it was typed, slash and all: the login still ends at its last '@'.
        assert_eq!(redact("at http://admin:pa/ss@10.0.0.5:8080/api/v2?x=1 failed"), "at http://***@10.0.0.5:8080/api/v2?x=1 failed");
        assert_eq!(redact("at http://admin:p@ss@10.0.0.5/ failed"), "at http://***@10.0.0.5/ failed");
        assert_eq!(redact("?apikey="), "?apikey=***");
        // Private trackers: Gazelle's authkey and torrent_pass, an RSS key, and a passkey that is a part of the path.
        assert_eq!(
            redact("GET https://tracker.example/torrents.php?action=download&id=1&authkey=a1b2&torrent_pass=c3d4&rsskey=e5 failed"),
            "GET https://tracker.example/torrents.php?action=download&id=1&authkey=***&torrent_pass=***&rsskey=*** failed"
        );
        assert_eq!(
            redact("Announce to https://tracker.example/0123456789abcdef0123456789abcdef/announce failed"),
            "Announce to https://tracker.example/***/announce failed"
        );
        assert_eq!(redact("http://10.0.0.5:8080/api/v2/torrents/info"), "http://10.0.0.5:8080/api/v2/torrents/info", "an ordinary path is left as it is");
    }

    /// A key is blanked by what it looks like as well as by what it is called: indexers, trackers and Jackett each name
    /// theirs differently, and a list of names is always one short.
    #[test]
    fn a_key_is_blanked_by_its_shape_whatever_it_is_called() {
        let secrets = ["abcdefghijklmnop1234", "0123456789abcdef0123456789abcdef", "424242", "s3cr3t", "pa,ss word", "keyvalue99"];
        let leaks = |said: &str| secrets.iter().filter(|s| said.contains(*s)).map(|s| s.to_string()).collect::<Vec<_>>();
        for quoted in [
            "Jackett: http://jackett:9117/dl/indexer/?jackett_apikey=abcdefghijklmnop1234&path=Q2hlY2s&file=Sintel",
            "https://indexer.example/getnzb/a1b2.nzb&i=424242&r=0123456789abcdef0123456789abcdef failed",
            "https://iptorrents.example/download.php/1/Sintel.torrent?torrent_pass=s3cr3t;tp=0123456789abcdef0123456789abcdef;u=424242",
            "https://tracker.example/download/Sintel.2026.1080p.12345.0123456789abcdef0123456789abcdef failed",
            "https://tracker.example/announce/0123-4567-89ab-cdef-0123-4567-89ab-cdef failed",
            "qBittorrent at http://admin:pa,ss word@10.0.0.5:8080/api failed",
            "https://indexer.example/api?api.key=keyvalue99&uid=424242&pid=424242&secret_key=s3cr3t",
            "https://indexer.example/api?anything=abcdefghijklmnop1234",
        ] {
            let said = redact(quoted);
            assert!(leaks(&said).is_empty(), "{quoted} -> {said}");
        }
        assert_eq!(redact("https://indexer.example/api?t=search&cat=5040&q=Big.Buck.Bunny"), "https://indexer.example/api?t=search&cat=5040&q=Big.Buck.Bunny", "nothing that looks like a key");
        assert_eq!(redact("https://tracker.example/Sintel.2026.1080p.WEB-DL.mkv"), "https://tracker.example/Sintel.2026.1080p.WEB-DL.mkv", "a release name is a name");
    }

    /// A queue's numbers are somebody else's: a time left of a billion days and speeds that add past what an i64 holds
    /// are read as nothing and as the most there can be, never as a panic.
    #[test]
    fn numbers_from_a_queue_cannot_overflow() {
        assert_eq!(timespan(&json!("99999999999999999.00:00:00")), None);
        assert_eq!(timespan(&json!("00:9223372036854775807:00")), None);
        let row = |bps: i64| Download { key: "k".into(), title: "Sintel".into(), sub: None, state: "downloading", progress: 0.5, size: 10, left: 5, eta_s: None, down_bps: bps,
            release: None, client: None, protocol: None, service_name: None, arr: None, media_type: None, tmdb_id: None, tvdb_id: None, seasons: vec![], parts: 1, error: None };
        assert_eq!(total_bps(&[row(i64::MAX), row(i64::MAX)]), i64::MAX);
        assert_eq!(total_bps(&[row(3), row(4)]), 7);
    }

    #[test]
    fn a_queue_error_never_carries_a_credential() {
        let r = json!({ "title": "Some.Film.2026", "movie": { "title": "Some Film", "year": 2026 }, "size": 1, "sizeleft": 1, "status": "warning",
            "trackedDownloadState": "downloading", "trackedDownloadStatus": "warning", "downloadId": "x",
            "errorMessage": "stalled (qBittorrent at http://admin:hunter2@10.0.0.5:8080/?apikey=abc123)" });
        let q = queue_row(1, "Radarr", Kind::Radarr, &r).unwrap();
        let e = q.error.unwrap();
        assert!(!e.contains("hunter2") && !e.contains("abc123") && e.contains("stalled"), "{e}");
        let r = json!({ "title": "x", "movie": { "title": "x" }, "status": "warning", "trackedDownloadState": "downloading", "downloadId": "y",
            "statusMessages": [{ "title": "Could not reach https://u:p4ss@indexer.example/?passkey=zz" }] });
        let e = queue_row(1, "Radarr", Kind::Radarr, &r).unwrap().error.unwrap();
        assert!(!e.contains("p4ss") && !e.contains("zz"), "{e}");
    }
    use serde_json::json;

    fn queued(id: i64, title: &str, download_id: Option<&str>, season: Option<i64>, state: &'static str, size: i64, left: i64) -> Queued {
        Queued {
            service_id: 1, service_name: "Sonarr".into(), arr_media_id: Some(7), download_id: download_id.map(|h| h.to_ascii_lowercase()), title: title.into(),
            sub: season.map(|s| format!("S{s:02}E01")), media_type: "tv", tmdb_id: None, tvdb_id: Some(id), season, size, left, eta_s: Some(600),
            state, protocol: Some("torrent".into()), client: Some("qBittorrent".into()), release: Some(format!("{title}.1080p.WEB-DL").replace(' ', ".")), error: None,
        }
    }

    #[test]
    fn a_season_pack_is_one_download_however_many_episodes_it_holds() {
        let queue = vec![queued(1, "Low Orbit", Some("AABB"), Some(3), "downloading", 900, 300), queued(1, "Low Orbit", Some("aabb"), Some(3), "downloading", 900, 300), queued(1, "Low Orbit", Some("aabb"), Some(3), "importing", 900, 300)];
        let (rows, totals) = fold(queue);
        assert_eq!(rows.len(), 1, "one download id is one download");
        assert_eq!((rows[0].sub.as_deref(), rows[0].state), (Some("Season 3 · 3 episodes"), "importing"), "the most interesting state wins");
        assert_eq!((rows[0].progress, rows[0].parts, totals.importing), (0.6666666666666666, 3, 1));
        // The same id in two instances is two downloads: they are each waiting for their own copy.
        let (rows, _) = fold(vec![queued(1, "Low Orbit", Some("aabb"), Some(3), "downloading", 900, 300), Queued { service_id: 2, ..queued(1, "Low Orbit", Some("aabb"), Some(3), "downloading", 900, 300) }]);
        assert_eq!(rows.len(), 2);
    }

    /// A pack of more than one season says how many, and one whose records name no season says only its episodes.
    #[test]
    fn a_pack_across_seasons_says_how_many_seasons_and_one_without_any_only_its_episodes() {
        let (rows, _) = fold([1, 1, 2, 2].iter().map(|s| queued(1, "Low Orbit", Some("cc"), Some(*s), "downloading", 900, 300)).collect());
        assert_eq!(rows[0].sub.as_deref(), Some("2 seasons · 4 episodes"));
        let (rows, _) = fold((0..3).map(|_| queued(1, "Low Orbit", Some("dd"), None, "downloading", 900, 300)).collect());
        assert_eq!(rows[0].sub.as_deref(), Some("3 episodes"));
    }

    #[test]
    fn a_record_without_a_download_id_still_stands_for_itself() {
        let (rows, totals) = fold(vec![queued(2, "Northern Static", None, None, "downloading", 1000, 250), queued(3, "Tiny Giants", None, Some(1), "queued", 100, 100)]);
        assert_eq!(rows.len(), 2, "two nameless records are not one download");
        let film = rows.iter().find(|r| r.title == "Northern Static").unwrap();
        assert_eq!((film.progress, film.eta_s, film.client.as_deref()), (0.75, Some(600), Some("qBittorrent")), "progress and the client come from the queue");
        assert_eq!((totals.downloading, totals.queued), (1, 1));
    }

    #[test]
    fn what_needs_attention_is_on_top() {
        let rows = fold(vec![
            queued(1, "Queued thing", Some("a"), None, "queued", 10, 10),
            queued(2, "Failed thing", Some("b"), None, "failed", 10, 5),
            queued(3, "Downloading thing", Some("c"), None, "downloading", 10, 5),
            queued(4, "Importing thing", Some("d"), None, "importing", 10, 0),
        ])
        .0;
        assert_eq!(rows.iter().map(|r| r.state).collect::<Vec<_>>(), ["failed", "importing", "downloading", "queued"]);
    }

    #[test]
    fn a_speed_is_what_moved_between_two_readings_and_nothing_is_invented() {
        let mut rows = fold(vec![queued(1, "Low Orbit", Some("aa"), Some(3), "downloading", 1000, 400)]).0;
        let key = rows[0].key.clone();
        // Nothing to compare with yet.
        let after = speeds(&mut rows, &HashMap::new(), 1000);
        assert_eq!((rows[0].down_bps, after[&key]), (0, (400, 1000)));
        // 100 bytes moved in 10 seconds.
        let mut rows = fold(vec![queued(1, "Low Orbit", Some("aa"), Some(3), "downloading", 1000, 300)]).0;
        speeds(&mut rows, &after, 1010);
        assert_eq!(rows[0].down_bps, 10);
        // Too long ago, and going backwards (an upgrade that starts again), say nothing.
        let mut rows = fold(vec![queued(1, "Low Orbit", Some("aa"), Some(3), "downloading", 1000, 300)]).0;
        speeds(&mut rows, &after, 1000 + 4000);
        assert_eq!(rows[0].down_bps, 0);
        let mut rows = fold(vec![queued(1, "Low Orbit", Some("aa"), Some(3), "downloading", 1000, 900)]).0;
        speeds(&mut rows, &after, 1010);
        assert_eq!(rows[0].down_bps, 0);
    }

    #[test]
    fn a_queue_record_says_what_it_is_waiting_for() {
        let r = json!({ "seriesId": 7, "seasonNumber": 3, "episode": { "seasonNumber": 3, "episodeNumber": 9, "title": "The Long Way Down" },
            "series": { "title": "Low Orbit", "tvdbId": 371002 }, "size": 2000.0, "sizeleft": 500.0, "timeleft": "00:12:30", "downloadId": "ABC123",
            "title": "Low.Orbit.S03E09.1080p.WEB-DL", "status": "downloading", "trackedDownloadState": "downloading", "trackedDownloadStatus": "ok", "protocol": "torrent", "downloadClient": "qBittorrent" });
        let q = queue_row(1, "Sonarr", Kind::Sonarr, &r).unwrap();
        assert_eq!((q.title.as_str(), q.sub.as_deref(), q.season, q.tvdb_id), ("Low Orbit", Some("S03E09 · The Long Way Down"), Some(3), Some(371002)));
        assert_eq!((q.download_id.as_deref(), q.eta_s, q.state), (Some("abc123"), Some(750), "downloading"));
        assert_eq!((q.release.as_deref(), q.client.as_deref(), q.protocol.as_deref()), (Some("Low.Orbit.S03E09.1080p.WEB-DL"), Some("qBittorrent"), Some("torrent")));

        let film = json!({ "movieId": 21, "movie": { "title": "Northern Static", "year": 2026, "tmdbId": 981001 }, "size": 10.0, "sizeleft": 0.0,
            "status": "completed", "trackedDownloadState": "importPending", "trackedDownloadStatus": "warning", "statusMessages": [{ "title": "Not an upgrade for existing file" }], "protocol": "usenet", "downloadClient": "SABnzbd" });
        let q = queue_row(2, "Radarr", Kind::Radarr, &film).unwrap();
        assert_eq!((q.title.as_str(), q.sub.as_deref(), q.state, q.media_type), ("Northern Static", Some("2026"), "importing", "movie"));
        assert_eq!((q.error.as_deref(), q.protocol.as_deref()), (Some("Not an upgrade for existing file"), Some("usenet")));

        // What the queue is doing, in our words.
        assert_eq!(queue_state("downloading", "downloading", "ok"), "downloading");
        assert_eq!(queue_state("completed", "importPending", "ok"), "importing");
        assert_eq!(queue_state("warning", "downloading", "warning"), "stalled");
        assert_eq!(queue_state("warning", "downloading", "error"), "failed");
        assert_eq!(queue_state("downloading", "failedPending", "ok"), "failed");
        assert_eq!(queue_state("paused", "downloading", "ok"), "paused");
        assert_eq!(queue_state("nonsense", "", ""), "unknown");

        assert_eq!(timespan(&json!("01:02:03")), Some(3723));
        assert_eq!(timespan(&json!("1.00:00:30")), Some(86_430));
        assert_eq!(timespan(&json!("00:00:04.5000000")), Some(4));
        assert_eq!(timespan(&json!(null)), None);
        assert_eq!(timespan(&json!("soon")), None);
    }

    /// The queue is read where nobody's language is known; FinStats' own lines are said again in the reader's, and a
    /// season stays the number it is (2024 is a season, not "2,024").
    #[test]
    fn the_lines_finstats_writes_about_a_download_are_said_in_the_readers_language() {
        assert_eq!(pack_line(SOURCE, 10, &[3]), "Season 3 · 10 episodes");
        assert_eq!(pack_line("nn-NO", 10, &[3]), "Sesong 3 · 10 episodar");
        assert_eq!(pack_line("nb-NO", 20, &[1, 2]), "2 sesonger · 20 episoder");
        assert_eq!(season_line("nb-NO", 2024), "Sesong 2024");
        let record = json!({ "seasonNumber": 2024, "series": { "title": "Low Orbit" }, "size": 1.0, "sizeleft": 1.0, "status": "queued" });
        assert_eq!(queue_row(1, "Sonarr", Kind::Sonarr, &record).unwrap().sub.as_deref(), Some("Season 2024"), "kept in the source's words");
    }

    #[test]
    fn an_empty_queue_is_read_far_less_often_than_a_busy_one() {
        assert_eq!(wait_s(true, true, true), WATCHED_EVERY_S, "somebody is watching it move");
        assert_eq!(wait_s(true, true, false), WATCHED_EVERY_S, "watching an empty queue still means watching");
        assert_eq!(wait_s(false, true, true), IDLE_EVERY_S, "nobody looking, but a request page shows how far it has got");
        assert_eq!(wait_s(false, true, false), EMPTY_EVERY_S, "nothing in it and nobody looking: Sonarr and Radarr are left alone");
        assert_eq!(wait_s(true, false, false), IDLE_EVERY_S, "no service connected: no request either way");
    }
}
