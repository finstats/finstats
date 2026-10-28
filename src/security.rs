//! Where people watch from, and what looks wrong about it.
//!
//! A *sighting* is a moment FinStats knows where an account was: a play (`playbacks.remote_ip`, for as long
//! as it ran) or a sign-in in Jellyfin's activity log (`server_events.remote_ip`). `geo.rs` turns the address
//! into a place; plays from the home network count as being at home, which is wherever this network's own
//! public address is. Two things are reported, both derived from the whole history of one person by the pure
//! `detect()`, so a rescan always finds the same alerts and `dedupe` keeps them single:
//!
//! - `impossible_travel`: two sightings far apart (`travel_min_km`) that no flight connects
//!   (`travel_speed_kmh`), or that overlap. One alert per pair of places per day; a pair the owner has
//!   muted (a VPN, a mobile carrier that is "in" the capital) never reports again.
//! - `new_country`: the first sighting in a country, once there is a history to compare with.
//!
//! What is older than `HISTORIC_S` when it is found (an import, the first database) is filed as resolved.
//! A place is a city centre at best. Alerts are a reason to look, never proof.

use std::collections::{BTreeMap, HashMap, HashSet};

use anyhow::Result;
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::auth::{AuthUser, Manager};
use crate::db::rusqlite::{Connection, OptionalExtension, params, params_from_iter, types::Value as SqlValue};
use crate::db::{self, norm_id};
use crate::geo::{self, Database, Place, distance_km};
use crate::notify::{self, Fanout, Kind};
use crate::state::{ApiError, ApiResult, App, Settings};
use crate::stats::{FilterQuery, Scope};

/// Found this long after it happened, an alert is history, not news.
const HISTORIC_S: i64 = 30 * 86_400;
/// The same two places report once a day at most.
const PAIR_QUIET_S: i64 = 86_400;
/// Activity-log entries that say where an account was.
const SIGN_IN_TYPES: &str = "'AuthenticationSucceeded','SessionStarted'";

#[derive(Clone, Copy, Debug)]
pub struct Rules {
    pub speed_kmh: f64,
    pub min_km: f64,
}

impl Rules {
    pub fn of(s: &Settings) -> Self {
        Rules { speed_kmh: s.travel_speed_kmh as f64, min_km: s.travel_min_km as f64 }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Spot {
    /// Two sightings are in the same place when this is equal: `home`, or the coordinate to a tenth of a degree.
    pub key: String,
    pub label: String,
    pub country_code: Option<String>,
    pub country: Option<String>,
    pub lat: f64,
    pub lon: f64,
}

#[derive(Clone, Debug)]
pub struct Sighting {
    pub at: i64,
    pub until: i64,
    pub spot: Spot,
    /// "Big Buck Bunny on Living room TV", "signed in"
    pub what: What,
    pub ip: String,
    /// Stable across rescans: `play:12`, `event:340`.
    pub id: String,
}

/// What a sighting was: a title played on a device, or the line Jellyfin's log wrote. Kept in parts, so an alert says
/// it in the reader's language and not in the words it was filed in.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct What {
    /// "Big Buck Bunny", "Rookery · Episode 3", or the log's own line.
    pub title: String,
    /// The device a title was played on.
    pub device: Option<String>,
}

impl From<&str> for What {
    fn from(title: &str) -> Self {
        What { title: title.into(), device: None }
    }
}

impl What {
    /// In `lang`'s words: the title, on its device.
    fn said(&self, lang: &str) -> String {
        match &self.device {
            Some(device) => crate::i18n::say(lang, "server.security.played_on", &[("title", (&self.title).into()), ("device", device.into())]),
            None => self.title.clone(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Finding {
    pub kind: &'static str,
    pub severity: &'static str,
    pub at: i64,
    pub dedupe: String,
    pub details: Value,
}

/// "London, United Kingdom", in the language of the request. Every caller in FinStats runs on a thread of the
/// database's and says which language it means (`place_label_in`).
#[cfg(test)]
fn place_label(p: &Place) -> String {
    place_label_in(p, crate::i18n::current())
}

/// `place_label` in `lang`'s words, for work on a thread where no request's language is.
fn place_label_in(p: &Place, lang: &str) -> String {
    match (&p.city, &p.country) {
        (Some(city), Some(country)) => format!("{city}, {country}"),
        (None, Some(country)) => country.clone(),
        (Some(city), None) => city.clone(),
        (None, None) => p.country_code.clone().unwrap_or_else(|| crate::i18n::say(lang, "server.security.unknown_place", &[])),
    }
}

/// The language an alert is filed in. It is said again in the reader's when it is read (`say_alert`).
const FILED_IN: &str = crate::i18n::LOCALES[0];

fn spot_of(p: &Place, home: bool) -> Option<Spot> {
    let (lat, lon) = (p.latitude?, p.longitude?);
    Some(Spot {
        key: if home { "home".into() } else { format!("{lat:.1},{lon:.1}") },
        label: if home { crate::i18n::say(FILED_IN, "server.security.home_place", &[("place", place_label_in(p, FILED_IN).into())]) } else { place_label_in(p, FILED_IN) },
        country_code: p.country_code.clone(),
        country: p.country.clone(),
        lat,
        lon,
    })
}

fn pair_key(a: &Spot, b: &Spot) -> String {
    if a.key <= b.key { format!("{}|{}", a.key, b.key) } else { format!("{}|{}", b.key, a.key) }
}

fn side(s: &Sighting) -> Value {
    json!({ "place": s.spot.label, "country_code": s.spot.country_code, "latitude": s.spot.lat, "longitude": s.spot.lon,
            "at": s.at, "until": s.until, "what": s.what.said(FILED_IN), "title": s.what.title, "device": s.what.device,
            "ip": s.ip, "home": s.spot.key == "home" })
}

/// One side of a stored alert in the reader's words: its place and what was seen there. Alerts filed before the parts
/// were kept say what they were filed with.
fn say_side(side: &mut Value, lang: &str) {
    let unknown = crate::i18n::say(FILED_IN, "server.security.unknown_place", &[]);
    let place = |p: &str| if p == unknown { crate::i18n::say(lang, "server.security.unknown_place", &[]) } else { p.to_string() };
    if let Some(filed) = side["place"].as_str() {
        // "Home (Oslo, Norway)": the words around the place are the filed language's, the place is the database's.
        let home = crate::i18n::say(FILED_IN, "server.security.home_place", &[("place", "\u{0}".into())]);
        let (before, after) = home.split_once('\u{0}').unwrap_or((home.as_str(), ""));
        let said = match filed.strip_prefix(before).and_then(|p| p.strip_suffix(after)).filter(|_| side["home"] == true) {
            Some(inner) => crate::i18n::say(lang, "server.security.home_place", &[("place", place(inner).into())]),
            None => place(filed),
        };
        side["place"] = json!(said);
    }
    if let (Some(title), Some(device)) = (side["title"].as_str(), side["device"].as_str()) {
        let what = What { title: title.into(), device: Some(device.into()) }.said(lang);
        side["what"] = json!(what);
    }
}

/// A stored alert in the reader's words: both sides, and a note FinStats wrote itself.
fn say_alert(alert: &mut Value, lang: &str) {
    for side in ["from", "to"] {
        if alert["details"][side].is_object() {
            say_side(&mut alert["details"][side], lang);
        }
    }
    let own = [("server.security.found_in_older_history", alert["resolved_by"] == "finstats"), ("server.security.same_two_places", true)];
    for (key, by_finstats) in own {
        if by_finstats && alert["note"].as_str() == Some(crate::i18n::say(FILED_IN, key, &[]).as_str()) {
            alert["note"] = json!(crate::i18n::say(lang, key, &[]));
        }
    }
}

/// Everything wrong with one person's sightings, which must be in the order they began.
/// `muted` holds the place pairs (`pair_key`) the owner has declared fine for this person.
pub fn detect(user_id: &str, sightings: &[Sighting], rules: &Rules, muted: &HashSet<String>) -> Vec<Finding> {
    let mut out = vec![];
    let mut countries: HashSet<&str> = HashSet::new();
    let mut last_report: HashMap<String, i64> = HashMap::new();
    // The sighting before this one, and the one that lasted longest: a film still running at home
    // says more about a sign-in abroad than a short play that ended hours ago.
    let mut prev: Option<&Sighting> = None;
    let mut longest: Option<&Sighting> = None;

    for s in sightings {
        let mut worst: Option<(&Sighting, f64, i64, Option<f64>)> = None;
        for earlier in [prev, longest].into_iter().flatten() {
            if earlier.spot.key == s.spot.key {
                continue;
            }
            let km = distance_km(earlier.spot.lat, earlier.spot.lon, s.spot.lat, s.spot.lon);
            if km < rules.min_km {
                continue;
            }
            let gap_s = s.at - earlier.until;
            let speed = (gap_s > 0).then(|| km / (gap_s as f64 / 3600.0));
            if speed.is_some_and(|v| v <= rules.speed_kmh) {
                continue;
            }
            // Overlapping beats fast, faster beats fast.
            let rank = |v: Option<f64>| v.unwrap_or(f64::INFINITY);
            if worst.as_ref().is_none_or(|w| rank(speed) > rank(w.3)) {
                worst = Some((earlier, km, gap_s, speed));
            }
        }
        if let Some((earlier, km, gap_s, speed)) = worst {
            let pair = pair_key(&earlier.spot, &s.spot);
            let quiet = last_report.get(&pair).is_some_and(|t| s.at - t < PAIR_QUIET_S);
            if !muted.contains(&pair) && !quiet {
                last_report.insert(pair.clone(), s.at);
                out.push(Finding {
                    kind: "impossible_travel",
                    severity: "high",
                    at: s.at,
                    dedupe: format!("travel:{user_id}:{}", s.id),
                    details: json!({
                        "from": side(earlier), "to": side(s), "pair": pair,
                        "distance_km": km.round(), "gap_s": gap_s.max(0), "overlap": gap_s <= 0,
                        "speed_kmh": speed.map(f64::round),
                    }),
                });
            }
        }

        if let Some(cc) = s.spot.country_code.as_deref()
            && countries.insert(cc)
            && countries.len() > 1
        {
            out.push(Finding {
                kind: "new_country",
                severity: "medium",
                at: s.at,
                dedupe: format!("country:{user_id}:{cc}"),
                details: json!({ "to": side(s), "country": s.spot.country, "country_code": cc, "known": countries.len() - 1 }),
            });
        }

        if longest.is_none_or(|l| s.until >= l.until) {
            longest = Some(s);
        }
        prev = Some(s);
    }
    out
}

// ---------------------------------------------------------------- an alert as news

/// "4 hours", "35 minutes": how far apart two sightings were, in the words a message uses.
fn human_gap(seconds: i64) -> String {
    let s = seconds.max(0);
    match s {
        0 => "at the same time".into(),
        1..=90 => format!("{} apart", count(s, "second")),
        91..=5_400 => format!("{} apart", count((s as f64 / 60.0).round() as i64, "minute")),
        5_401..=172_800 => format!("{} apart", count((s as f64 / 3600.0).round() as i64, "hour")),
        _ => format!("{} apart", count((s as f64 / 86_400.0).round() as i64, "day")),
    }
}

/// "1 minute", "2 minutes".
fn count(n: i64, one: &str) -> String {
    format!("{n} {one}{}", if n == 1 { "" } else { "s" })
}

/// How long a run of things took: "under a minute", "12 minutes".
fn human_span(seconds: i64) -> String {
    match seconds.max(0) {
        0..=60 => "under a minute".into(),
        s @ 61..=5_400 => count((s as f64 / 60.0).round() as i64, "minute"),
        s => count((s as f64 / 3600.0).round() as i64, "hour"),
    }
}

fn place_of(side: &Value) -> String {
    side["place"].as_str().unwrap_or("an unknown place").to_string()
}

/// One alert, as the thing a destination is told. Places are in the message; the addresses and the
/// coordinates behind them are `private_field`s, so they only ever reach a destination that asked.
fn as_event(f: &Finding, user_id: &str, user_name: &str) -> notify::Event {
    let d = &f.details;
    let (from, to) = (&d["from"], &d["to"]);
    let event = match f.kind {
        "new_country" => {
            let country = d["country"].as_str().unwrap_or("a new country").to_string();
            notify::Event::new(
                Kind::NewCountry,
                format!("notify:{}", f.dedupe),
                format!("{user_name} appeared in {country}"),
                format!("First time this account has been seen there. {}, {}.", place_of(to), to["what"].as_str().unwrap_or("seen")),
            )
            .field("Person", user_name)
            .field("Place", place_of(to))
        }
        _ => {
            let overlap = d["overlap"].as_bool().unwrap_or(false);
            let km = d["distance_km"].as_f64().unwrap_or(0.0);
            let when = if overlap { "at the same time".to_string() } else { human_gap(d["gap_s"].as_i64().unwrap_or(0)) };
            notify::Event::new(
                Kind::Travel,
                format!("notify:{}", f.dedupe),
                format!("Impossible travel: {user_name}"),
                format!("{} and {}, {km:.0} km apart, {when}.", place_of(from), place_of(to)),
            )
            .field("Person", user_name)
            .field("From", format!("{}, {}", place_of(from), from["what"].as_str().unwrap_or("seen")))
            .field("To", format!("{}, {}", place_of(to), to["what"].as_str().unwrap_or("seen")))
            .field("Apart", format!("{km:.0} km, {when}"))
        }
    };
    let private = |e: notify::Event, label: &str, side: &Value| match side["ip"].as_str() {
        Some(ip) if !ip.is_empty() => e.private_field(label, ip),
        _ => e,
    };
    let event = private(event, "From address", from);
    let event = private(event, "To address", to);
    event.at(f.at).about(user_id, user_name).link("/security")
}

// ---------------------------------------------------------------- failed sign-ins

/// Several failed sign-ins in a row are the one thing on this page that needs no geolocation database:
/// it is counting. `within_s` is the window, `min` how many it takes to be worth saying.
const BURST_WINDOW_S: i64 = 900;
const BURST_MIN: usize = 5;
/// How far back a scan looks. Anything older has been reported already, or never will be.
const BURST_LOOKBACK_S: i64 = 7 * 86_400;

#[derive(Debug, PartialEq)]
pub struct Burst {
    /// The account name Jellyfin refused, or the address it came from when there is no account.
    pub who: String,
    pub user_id: Option<String>,
    pub ip: String,
    pub first_at: i64,
    pub at: i64,
    /// Which window of `within_s` it fell in: what makes the same burst the same burst on a rescan.
    pub window: i64,
    pub count: usize,
}

/// Every account's failures in one window of `within_s`, once there are at least `min` of them, all of them counted.
/// Pure and re-derivable: a rescan finds exactly the same bursts, so `dedupe` keeps them single. Counted in runs of
/// `min`, ten thousand tries were two thousand notices and none of them could ever be an alert.
pub fn bursts(rows: &[(i64, i64, String, Option<String>, String)], within_s: i64, min: usize) -> Vec<Burst> {
    let mut found: BTreeMap<(String, i64), Burst> = BTreeMap::new();
    for (_, at, who, user_id, ip) in rows {
        let window = at.div_euclid(within_s);
        let b = found.entry((who.clone(), window)).or_insert_with(|| Burst { who: who.clone(), user_id: None, ip: String::new(), first_at: *at, at: *at, window, count: 0 });
        b.count += 1;
        b.first_at = b.first_at.min(*at);
        if *at >= b.at || b.count == 1 {
            (b.at, b.user_id, b.ip) = (*at, user_id.clone(), ip.clone());
        }
    }
    found.into_values().filter(|b| b.count >= min).collect()
}

/// How loud a burst of `count` failures is.
fn sign_in_severity(count: usize) -> &'static str {
    if count >= BURST_MIN * 4 { notify::ALERT } else { notify::WARN }
}

/// A burst's notice: one while it is a warning, and one more should it grow into an alert, so a window says at most two
/// things about one account, or about the names that are none (`who` None), however long it goes on.
fn burst_dedupe(who: Option<&str>, window: i64, severity: &str) -> String {
    let who = who.map_or_else(|| "nobody".to_string(), |w| format!("user:{w}"));
    match severity {
        notify::ALERT => format!("notify:signins:{who}:{window}:alert"),
        _ => format!("notify:signins:{who}:{window}"),
    }
}

/// Jellyfin's own words before the name on a failed sign-in, in English: "Failed login try from alice" in older
/// versions, "Failed login attempt from alice" in newer ones.
const SIGN_IN_TEMPLATES: [&str; 2] = ["Failed login try from ", "Failed login attempt from "];

/// The server's own words before the name, in a language FinStats does not know them in: what every failed sign-in's
/// line begins with, up to a space. Only from two different lines or more: one alone does not say where its name starts.
fn template_prefix<'a>(lines: impl Iterator<Item = &'a str>) -> Option<String> {
    let distinct: std::collections::BTreeSet<&str> = lines.filter(|l| !SIGN_IN_TEMPLATES.iter().any(|t| l.starts_with(t))).collect();
    if distinct.len() < 2 {
        return None;
    }
    let first = *distinct.first()?;
    let mut shared = first.len();
    for line in &distinct {
        shared = first.char_indices().zip(line.chars()).take_while(|((_, a), b)| a == b).last().map_or(0, |((i, a), _)| i + a.len_utf8()).min(shared);
    }
    let prefix = &first[..shared];
    Some(prefix[..prefix.rfind(' ')? + 1].to_string()).filter(|p| !p.trim().is_empty())
}

/// The name as it was typed on a failed sign-in: the whole of what follows the server's own words, never just its end.
fn typed_name<'a>(line: &'a str, learned: Option<&str>) -> Option<&'a str> {
    SIGN_IN_TEMPLATES.iter().copied().chain(learned).find_map(|p| line.strip_prefix(p)).map(str::trim).filter(|n| !n.is_empty())
}

/// Read the failures out of the activity log and file what is new. Needs no geolocation database, so it
/// runs on every install; `check` does the rest.
/// A failed sign-in: id, when, the account's name if it is one, user id, address.
type Failure = (i64, i64, Option<String>, Option<String>, String);
/// A failed sign-in as the activity log has it: id, when, its line, the name of the user it names by id, user id, address.
type LogLine = (i64, i64, String, Option<String>, Option<String>, String);

pub fn scan_sign_ins(conn: &Connection, bus: &Fanout) -> Result<usize> {
    let since = db::now() - BURST_LOOKBACK_S;
    // Jellyfin writes no user id on a failed sign-in, only a line ending in the name typed, in the server's language: an
    // account is a user whose name is the whole of what was typed.
    let read: Vec<LogLine> = conn
        .prepare_cached(
            "SELECT e.id, e.date, COALESCE(e.name, ''), u.name, e.user_id, COALESCE(e.remote_ip, '')
             FROM server_events e LEFT JOIN users u ON u.id = e.user_id
             WHERE e.type = 'AuthenticationFailed' AND e.date >= ?1 ORDER BY e.date, e.id",
        )?
        .query_map([since], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?)))?
        .collect::<Result<_, _>>()?;
    let accounts_named: HashMap<String, String> = conn
        .prepare_cached("SELECT name FROM users ORDER BY length(name)")?
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(|n| (n.to_lowercase(), n))
        .collect();
    let learned = template_prefix(read.iter().map(|r| r.2.as_str()));
    let rows: Vec<Failure> = read
        .into_iter()
        .map(|(id, at, line, by_id, user_id, ip)| {
            let account = by_id.or_else(|| typed_name(&line, learned.as_deref()).and_then(|n| accounts_named.get(&n.to_lowercase()).cloned()));
            (id, at, account, user_id, ip)
        })
        .collect();
    let mut added = 0;
    // An account's failures are its own bursts. A name that is no account is whatever the one knocking typed, as many
    // as they like: keyed by it, a thousand invented names were a thousand notifications carrying their text, and a
    // real alert waited behind them. Those are one notice a window, for the whole server, and name nobody.
    let (accounts, nobody): (Vec<_>, Vec<_>) = rows.into_iter().partition(|r| r.2.is_some());
    let accounts: Vec<(i64, i64, String, Option<String>, String)> = accounts.into_iter().map(|(id, at, name, user, ip)| (id, at, name.unwrap_or_default(), user, ip)).collect();
    for b in bursts(&accounts, BURST_WINDOW_S, BURST_MIN) {
        let severity = sign_in_severity(b.count);
        let mut event = notify::Event::new(
            Kind::FailedSignIns,
            burst_dedupe(Some(&b.who), b.window, severity),
            format!("{} failed sign-ins for {}", b.count, b.who),
            format!("{} attempts in {}.", b.count, human_span(b.at - b.first_at)),
        )
        .at(b.at)
        .severity(severity)
        .field("Account", b.who.clone())
        .field("Attempts", b.count.to_string())
        .link("/server");
        if let Some(user_id) = &b.user_id {
            event = event.about(user_id.clone(), b.who.clone());
        }
        if !b.ip.is_empty() {
            event = event.private_field("From address", b.ip.clone());
        }
        added += usize::from(notify::raise_in(conn, bus, &event)?);
    }
    let mut windows: BTreeMap<i64, Vec<&Failure>> = BTreeMap::new();
    for r in &nobody {
        windows.entry(r.1.div_euclid(BURST_WINDOW_S)).or_default().push(r);
    }
    for (window, tries) in windows.into_iter().filter(|(_, t)| t.len() >= BURST_MIN) {
        let (first, last) = (tries[0].1, tries[tries.len() - 1].1);
        let addresses = tries.iter().map(|r| r.4.as_str()).collect::<std::collections::BTreeSet<_>>().len();
        let event = notify::Event::new(
            Kind::FailedSignIns,
            burst_dedupe(None, window, sign_in_severity(tries.len())),
            format!("{} failed sign-ins for names that are not accounts", tries.len()),
            format!("{} attempts in {}, from {addresses} address{}.", tries.len(), human_span(last - first), if addresses == 1 { "" } else { "es" }),
        )
        .at(last)
        .severity(sign_in_severity(tries.len()))
        .field("Attempts", tries.len().to_string())
        .link("/server");
        added += usize::from(notify::raise_in(conn, bus, &event)?);
    }
    Ok(added)
}

/// After the activity log has been read: was anybody knocking?
pub async fn check_sign_ins(app: &App) {
    let bus_app = app.clone();
    let done = app.db.call(move |c| {
        let bus = Fanout::of(c, &bus_app)?;
        scan_sign_ins(c, &bus)
    }).await;
    match done {
        Ok(n) if n > 0 => {
            tracing::warn!("{n} burst{} of failed sign-ins", if n == 1 { "" } else { "s" });
            app.notify_wake.notify_one();
        }
        Ok(_) => {}
        Err(e) => tracing::warn!("could not look at failed sign-ins: {e:#}"),
    }
}

// ---------------------------------------------------------------- addresses → places

/// The address in an activity-log line ("IP address: 203.0.113.9"). The wording follows the server's
/// language, so this takes the first word that is an address rather than trusting the label.
pub fn event_ip(short_overview: &str) -> Option<String> {
    short_overview.split_whitespace().find_map(|w| {
        let w = w.trim_matches(|c: char| !c.is_ascii_hexdigit() && c != ':' && c != '.');
        crate::network::canonical(w).or_else(|| crate::network::canonical(w.trim_end_matches(['.', ':'])))
    })
}

/// Sign-ins restored from an older backup, or synced before this existed, have no address column yet.
fn fill_event_ips(conn: &Connection) -> Result<usize> {
    let rows: Vec<(i64, Option<String>)> = conn
        .prepare("SELECT id, short_overview FROM server_events WHERE remote_ip IS NULL")?
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<Result<_, _>>()?;
    let mut stmt = conn.prepare_cached("UPDATE server_events SET remote_ip = ?1 WHERE id = ?2")?;
    for (id, text) in &rows {
        stmt.execute(params![text.as_deref().and_then(event_ip).unwrap_or_default(), id])?;
    }
    Ok(rows.len())
}

/// Give every address that has none yet a row in `ip_locations` (an empty one when it has no place).
fn locate_new(conn: &Connection, geo: &Database) -> Result<usize> {
    let ips: Vec<String> = conn
        .prepare(
            "SELECT ip FROM (SELECT remote_ip AS ip FROM playbacks WHERE remote_ip IS NOT NULL
                             UNION SELECT remote_ip FROM server_events WHERE remote_ip <> ''
                             UNION SELECT ip FROM home_addresses)
             WHERE ip NOT IN (SELECT ip FROM ip_locations)",
        )?
        .query_map([], |r| r.get(0))?
        .collect::<Result<_, _>>()?;
    let now = db::now();
    let mut stmt = conn.prepare_cached(
        "INSERT OR REPLACE INTO ip_locations(ip, country_code, country, region, city, latitude, longitude, timezone, looked_up_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
    )?;
    for ip in &ips {
        let p = geo.lookup(ip).unwrap_or_default();
        stmt.execute(params![ip, p.country_code, p.country, p.region, p.city, p.latitude, p.longitude, p.timezone, now])?;
    }
    Ok(ips.len())
}

fn stored_place(conn: &Connection, ip: &str) -> Result<Option<Place>> {
    Ok(conn
        .prepare_cached("SELECT country_code, country, region, city, latitude, longitude, timezone FROM ip_locations WHERE ip = ?1")?
        .query_row([ip], |r| {
            Ok(Place { country_code: r.get(0)?, country: r.get(1)?, region: r.get(2)?, city: r.get(3)?, latitude: r.get(4)?, longitude: r.get(5)?, timezone: r.get(6)? })
        })
        .optional()?)
}

/// Where "at home" is: the place of this network's public address, the most recently seen one that has a place.
pub fn home_place(conn: &Connection) -> Result<Option<Place>> {
    Ok(conn
        .prepare_cached(
            "SELECT l.country_code, l.country, l.region, l.city, l.latitude, l.longitude, l.timezone
             FROM home_addresses h JOIN ip_locations l ON l.ip = h.ip
             WHERE l.latitude IS NOT NULL ORDER BY (h.source = 'lookup') DESC, h.last_seen DESC LIMIT 1",
        )?
        .query_row([], |r| {
            Ok(Place { country_code: r.get(0)?, country: r.get(1)?, region: r.get(2)?, city: r.get(3)?, latitude: r.get(4)?, longitude: r.get(5)?, timezone: r.get(6)? })
        })
        .optional()?)
}

// ---------------------------------------------------------------- scanning

fn sightings_of(conn: &Connection, user_id: &str, home: Option<&Spot>) -> Result<Vec<Sighting>> {
    let mut stmt = conn.prepare_cached(&format!(
        "SELECT 'play:' || p.id, p.started_at, p.ended_at, p.remote_ip, p.is_local,
                COALESCE(p.series_name || ' · ', '') || p.item_name, p.device_name
         FROM visible_playbacks p WHERE p.user_id = ?1 AND p.remote_ip IS NOT NULL
         UNION ALL
         SELECT 'event:' || e.id, e.date, e.date, e.remote_ip, NULL, e.name, NULL
         FROM server_events e WHERE e.user_id = ?1 AND e.remote_ip <> '' AND e.type IN ({SIGN_IN_TYPES})
         ORDER BY 2, 1"
    ))?;
    let rows: Vec<(String, i64, i64, String, Option<bool>, What)> = stmt
        .query_map([user_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, What { title: r.get(5)?, device: r.get(6)? })))?
        .collect::<Result<_, _>>()?;
    let mut spots: HashMap<String, Option<Spot>> = HashMap::new();
    let mut out = Vec::with_capacity(rows.len());
    for (id, at, until, ip, is_local, what) in rows {
        if !spots.contains_key(&ip) {
            let local = match is_local {
                Some(l) => l,
                None => crate::network::classify(conn, &ip)?.unwrap_or(false),
            };
            let spot = if local { home.cloned() } else { stored_place(conn, &ip)?.and_then(|p| spot_of(&p, false)) };
            spots.insert(ip.clone(), spot);
        }
        if let Some(spot) = spots[&ip].clone() {
            out.push(Sighting { at, until: until.max(at), spot, what, ip, id });
        }
    }
    Ok(out)
}

/// Look at one person's history, or everyone's, and file what is new. Returns how many alerts were added.
pub fn scan(conn: &Connection, geo: &Database, rules: &Rules, only_user: Option<&str>, bus: Option<&Fanout>) -> Result<usize> {
    // Only sign-ins restored or read before addresses were kept lack one: a whole scan fills them, not each play's.
    if only_user.is_none() {
        fill_event_ips(conn)?;
    }
    locate_new(conn, geo)?;
    file_alerts(conn, rules, only_user, bus)
}

/// The part of a scan that needs no database file: every address already has its place.
fn file_alerts(conn: &Connection, rules: &Rules, only_user: Option<&str>, bus: Option<&Fanout>) -> Result<usize> {
    let home = home_place(conn)?.and_then(|p| spot_of(&p, true));
    let users: Vec<(String, String)> = conn
        .prepare(
            "SELECT p.user_id, COALESCE(u.name, MAX(p.user_name)) FROM visible_playbacks p LEFT JOIN users u ON u.id = p.user_id
             WHERE p.remote_ip IS NOT NULL AND (?1 IS NULL OR p.user_id = ?1) GROUP BY p.user_id
             UNION
             SELECT e.user_id, u.name FROM server_events e JOIN users u ON u.id = e.user_id
             WHERE e.remote_ip <> '' AND (?1 IS NULL OR e.user_id = ?1) GROUP BY e.user_id",
        )?
        .query_map([only_user], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<Result<_, _>>()?;
    let now = db::now();
    let mut added = 0;
    let mut seen_users = HashSet::new();
    for (user_id, user_name) in users {
        if !seen_users.insert(user_id.clone()) {
            continue;
        }
        let muted: HashSet<String> = conn
            .prepare_cached("SELECT details FROM security_alerts WHERE user_id = ?1 AND muted = 1")?
            .query_map([&user_id], |r| r.get::<_, String>(0))?
            .filter_map(|d| serde_json::from_str::<Value>(&d.ok()?).ok()?["pair"].as_str().map(str::to_string))
            .collect();
        let sightings = sightings_of(conn, &user_id, home.as_ref())?;
        for f in detect(&user_id, &sightings, rules, &muted) {
            let historic = now - f.at > HISTORIC_S;
            let filed = conn.prepare_cached(
                "INSERT OR IGNORE INTO security_alerts(kind, severity, user_id, user_name, at, dedupe, details, created_at, resolved_at, resolved_by, note)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            )?
            .execute(params![f.kind, f.severity, user_id, user_name, f.at, f.dedupe, f.details.to_string(), now,
                historic.then_some(now), historic.then_some("finstats"), historic.then(|| crate::i18n::say(FILED_IN, "server.security.found_in_older_history", &[]))])?;
            added += filed;
            // An alert nobody is looking at is the whole reason notifications exist.
            if filed > 0 && let Some(bus) = bus {
                notify::raise_in(conn, bus, &as_event(&f, &user_id, &user_name))?;
            }
        }
    }
    Ok(added)
}

/// After a play begins or the log was read: is anything new wrong? Does nothing without a database.
/// Scans of one person's history, one at a time: `begin` says whether to start one, and one asked for while another runs
/// is noted, so that `finish` says to run once more.
#[derive(Default)]
pub struct Coalesce(std::sync::Mutex<(HashSet<String>, HashSet<String>)>);

impl Coalesce {
    pub fn begin(&self, who: &str) -> bool {
        let mut s = self.0.lock().unwrap();
        if s.0.contains(who) {
            s.1.insert(who.to_string());
            return false;
        }
        s.0.insert(who.to_string())
    }

    pub fn finish(&self, who: &str) -> bool {
        let mut s = self.0.lock().unwrap();
        if s.1.remove(who) {
            return true;
        }
        s.0.remove(who);
        false
    }
}

/// A play began: scan that person's history, unless a scan of it is running, in which case once more after it. Every
/// play that begins asked for one, and a client starting plays from new device ids ran several at once.
pub async fn check_person(app: &App, who: String) {
    static RUNNING: std::sync::OnceLock<Coalesce> = std::sync::OnceLock::new();
    let running = RUNNING.get_or_init(Coalesce::default);
    if !running.begin(&who) {
        return;
    }
    loop {
        check(app, Some(who.clone())).await;
        if !running.finish(&who) {
            break;
        }
    }
}

pub async fn check(app: &App, user_id: Option<String>) {
    let Some(geo) = app.geo.get() else { return };
    let rules = Rules::of(&app.settings());
    let bus_app = app.clone();
    match app.db.call(move |c| {
        let bus = notify::Fanout::of(c, &bus_app)?;
        scan(c, &geo, &rules, user_id.as_deref(), Some(&bus))
    }).await {
        Ok(n) if n > 0 => tracing::warn!("{n} new security alert{}", if n == 1 { "" } else { "s" }),
        Ok(_) => {}
        Err(e) => tracing::warn!("security scan failed: {e:#}"),
    }
}

/// The database changed: every address gets its place again, then everyone is looked at.
pub async fn refresh_all(app: &App) {
    let geo = app.geo.get();
    let rules = Rules::of(&app.settings());
    let bus_app = app.clone();
    let result = app
        .db
        .call(move |c| {
            let tx = c.transaction()?;
            tx.execute("DELETE FROM ip_locations", [])?;
            let added = match &geo {
                Some(geo) => {
                    let bus = notify::Fanout::of(&tx, &bus_app)?;
                    scan(&tx, geo, &rules, None, Some(&bus))?
                }
                None => 0,
            };
            tx.commit()?;
            Ok(added)
        })
        .await;
    if let Err(e) = result {
        tracing::warn!("re-reading places failed: {e:#}");
    }
}

// ---------------------------------------------------------------- API

/// Places are network details about everybody: both permissions, or nothing.
fn gate(user: &AuthUser) -> Result<(), ApiError> {
    if user.perms.see_network && user.perms.see_everyone { Ok(()) } else { Err(ApiError::not_permitted("see where people watch from")) }
}

fn database_json(app: &App) -> Value {
    match app.geo.get() {
        Some(db) => json!({ "kind": db.kind, "built_at": db.built_at, "dbip": db.is_dbip(), "file": db.path.file_name().map(|n| n.to_string_lossy().into_owned()) }),
        None => Value::Null,
    }
}

pub fn status_json(app: &App) -> Value {
    json!({ "database": database_json(app), "folder": geo::dir(&app.data_dir).display().to_string(),
            "from_env": std::env::var("FINSTATS_GEOIP_DB").is_ok_and(|p| !p.trim().is_empty()), "source": geo::download_url("YYYY-MM") })
}

#[derive(Default)]
struct PlaceAgg {
    place: Place,
    home: bool,
    plays: i64,
    watch_s: i64,
    sign_ins: i64,
    ips: i64,
    last_seen: i64,
    users: BTreeMap<String, (String, i64, i64)>,
}

fn place_at(r: &crate::db::rusqlite::Row, first: usize) -> crate::db::rusqlite::Result<Place> {
    Ok(Place { latitude: r.get(first)?, longitude: r.get(first + 1)?, city: r.get(first + 2)?, region: r.get(first + 3)?, country: r.get(first + 4)?, country_code: r.get(first + 5)?, timezone: None })
}

const PLACE_COLS: &str = "l.latitude, l.longitude, l.city, l.region, l.country, l.country_code";

pub async fn overview(State(app): State<App>, user: AuthUser, Query(q): Query<FilterQuery>) -> ApiResult {
    gate(&user)?;
    let scope = Scope::new(&app, &user, &q);
    let mut live = app.live.read().unwrap().clone();
    let geo = app.geo.get();
    let with_failed = user.perms.see_server;
    let lang = crate::i18n::current();
    let mut body = app
        .db
        .call(move |c| {
            let scope = scope.resolve(c)?;
            let since = scope.since.unwrap_or(0);
            // The map can be about several people at once now, so "whose" is a clause rather than
            // one bound value. Empty = everybody, which is what `1` says.
            let who = scope.user_ids.clone();
            let whose = |col: &str| match who.len() {
                0 => "1".to_string(),
                n => format!("{col} IN ({})", std::iter::repeat_n("?", n).collect::<Vec<_>>().join(", ")),
            };
            let with_who = |first: SqlValue| -> Vec<SqlValue> {
                let mut a = vec![first];
                a.extend(who.iter().map(|u| SqlValue::from(u.clone())));
                a
            };
            let home = home_place(c)?;
            let mut places: BTreeMap<String, PlaceAgg> = BTreeMap::new();
            let key = |p: &Place| format!("{:.4},{:.4}", p.latitude.unwrap_or(0.0), p.longitude.unwrap_or(0.0));

            // Plays away from home, by place and person.
            let mut stmt = c.prepare(&format!(
                "SELECT {PLACE_COLS}, p.user_id, COALESCE(u.name, p.user_name), COUNT(*), COALESCE(SUM(p.duration_s), 0), MAX(p.started_at), COUNT(DISTINCT p.remote_ip)
                 FROM visible_playbacks p JOIN ip_locations l ON l.ip = p.remote_ip LEFT JOIN users u ON u.id = p.user_id
                 WHERE COALESCE(p.is_local, 0) = 0 AND l.latitude IS NOT NULL AND p.started_at >= ?1 AND {}
                 GROUP BY l.latitude, l.longitude, p.user_id", whose("p.user_id")
            ))?;
            let rows = stmt.query_map(params_from_iter(with_who(since.into()).iter()), |r| Ok((place_at(r, 0)?, r.get::<_, String>(6)?, r.get::<_, String>(7)?, r.get::<_, i64>(8)?, r.get::<_, i64>(9)?, r.get::<_, i64>(10)?, r.get::<_, i64>(11)?)))?;
            for row in rows {
                let (place, uid, name, plays, watch_s, last, ips) = row?;
                let a = places.entry(key(&place)).or_insert_with(|| PlaceAgg { place: place.clone(), ..Default::default() });
                a.plays += plays;
                a.watch_s += watch_s;
                a.ips += ips;
                a.last_seen = a.last_seen.max(last);
                let u = a.users.entry(uid).or_insert((name, 0, 0));
                u.1 += plays;
            }

            // Plays at home: one dot where this network is.
            if let Some(h) = &home {
                let mut stmt = c.prepare(&format!(
                    "SELECT p.user_id, COALESCE(u.name, p.user_name), COUNT(*), COALESCE(SUM(p.duration_s), 0), MAX(p.started_at)
                     FROM visible_playbacks p LEFT JOIN users u ON u.id = p.user_id
                     WHERE p.is_local = 1 AND p.started_at >= ?1 AND {} GROUP BY p.user_id", whose("p.user_id")
                ))?;
                let rows = stmt.query_map(params_from_iter(with_who(since.into()).iter()), |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, i64>(2)?, r.get::<_, i64>(3)?, r.get::<_, i64>(4)?)))?;
                for row in rows {
                    let (uid, name, plays, watch_s, last) = row?;
                    let a = places.entry("home".into()).or_insert_with(|| PlaceAgg { place: h.clone(), home: true, ..Default::default() });
                    a.plays += plays;
                    a.watch_s += watch_s;
                    a.last_seen = a.last_seen.max(last);
                    a.users.entry(uid).or_insert((name, 0, 0)).1 += plays;
                }
            }

            // Sign-ins away from home.
            let mut stmt = c.prepare(&format!(
                "SELECT {PLACE_COLS}, e.user_id, u.name, COUNT(*), MAX(e.date)
                 FROM server_events e JOIN ip_locations l ON l.ip = e.remote_ip JOIN users u ON u.id = e.user_id
                 WHERE e.type = 'AuthenticationSucceeded' AND e.remote_ip <> '' AND l.latitude IS NOT NULL
                   AND e.remote_ip NOT IN (SELECT ip FROM home_addresses) AND e.date >= ?1 AND {}
                 GROUP BY l.latitude, l.longitude, e.user_id", whose("e.user_id")
            ))?;
            let rows = stmt.query_map(params_from_iter(with_who(since.into()).iter()), |r| Ok((place_at(r, 0)?, r.get::<_, String>(6)?, r.get::<_, String>(7)?, r.get::<_, i64>(8)?, r.get::<_, i64>(9)?)))?;
            for row in rows {
                let (place, uid, name, n, last) = row?;
                let a = places.entry(key(&place)).or_insert_with(|| PlaceAgg { place: place.clone(), ..Default::default() });
                a.sign_ins += n;
                a.last_seen = a.last_seen.max(last);
                a.users.entry(uid).or_insert((name, 0, 0)).2 += n;
            }

            // Failed sign-ins have no account, only a name somebody typed. Part of the server log.
            let mut failed: BTreeMap<String, (Place, i64, i64, Vec<String>)> = BTreeMap::new();
            if with_failed && who.is_empty() {
                let mut stmt = c.prepare(&format!(
                    "SELECT {PLACE_COLS}, e.name, COUNT(*), MAX(e.date)
                     FROM server_events e JOIN ip_locations l ON l.ip = e.remote_ip
                     WHERE e.type = 'AuthenticationFailed' AND e.remote_ip <> '' AND l.latitude IS NOT NULL
                       AND e.remote_ip NOT IN (SELECT ip FROM home_addresses) AND e.date >= ?1
                     GROUP BY l.latitude, l.longitude, e.name ORDER BY MAX(e.date) DESC"
                ))?;
                let rows = stmt.query_map(params![since], |r| Ok((place_at(r, 0)?, r.get::<_, String>(6)?, r.get::<_, i64>(7)?, r.get::<_, i64>(8)?)))?;
                for row in rows {
                    let (place, name, n, last) = row?;
                    let f = failed.entry(key(&place)).or_insert_with(|| (place.clone(), 0, 0, vec![]));
                    f.1 += n;
                    f.2 = f.2.max(last);
                    if f.3.len() < 5 {
                        f.3.push(name);
                    }
                }
            }

            // Streams running right now.
            if !who.is_empty() {
                live.retain(|s| s["user_id"].as_str().is_some_and(|u| who.iter().any(|w| w == u)));
            }
            let mut now_playing = vec![];
            for s in &live {
                let Some(ip) = s["remote_ip"].as_str() else { continue };
                let local = crate::network::classify(c, ip)?.unwrap_or(false);
                let place = if local { home.clone() } else { geo.as_ref().and_then(|g| g.lookup(ip)) };
                let Some(p) = place.filter(|p| p.latitude.is_some()) else { continue };
                now_playing.push(json!({
                    "user_id": s["user_id"], "user_name": s["user_name"], "item_name": s["item_name"], "series_name": s["series_name"],
                    "device_name": s["device_name"], "home": local, "place": place_label_in(&p, lang), "latitude": p.latitude, "longitude": p.longitude,
                }));
            }

            let mut countries: BTreeMap<String, (String, i64, HashSet<String>)> = BTreeMap::new();
            for a in places.values() {
                if let Some(cc) = &a.place.country_code {
                    let e = countries.entry(cc.clone()).or_insert_with(|| (a.place.country.clone().unwrap_or_else(|| cc.clone()), 0, HashSet::new()));
                    e.1 += a.plays;
                    e.2.extend(a.users.keys().cloned());
                }
            }
            // A place the database knows only by its country cannot be drawn on the map, but it is a country all the same.
            let mut stmt = c.prepare(&format!(
                "SELECT l.country_code, COALESCE(l.country, l.country_code), p.user_id, COUNT(*)
                 FROM visible_playbacks p JOIN ip_locations l ON l.ip = p.remote_ip
                 WHERE COALESCE(p.is_local, 0) = 0 AND l.latitude IS NULL AND l.country_code IS NOT NULL AND p.started_at >= ?1 AND {}
                 GROUP BY l.country_code, p.user_id", whose("p.user_id")
            ))?;
            let rows = stmt.query_map(params_from_iter(with_who(since.into()).iter()), |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?, r.get::<_, i64>(3)?)))?;
            for row in rows {
                let (cc, name, uid, plays) = row?;
                let e = countries.entry(cc).or_insert_with(|| (name, 0, HashSet::new()));
                e.1 += plays;
                e.2.insert(uid);
            }
            let mut countries: Vec<Value> = countries.into_iter().map(|(cc, (name, plays, users))| json!({ "code": cc, "name": name, "plays": plays, "users": users.len() })).collect();
            countries.sort_by_key(|v| std::cmp::Reverse(v["plays"].as_i64().unwrap_or(0)));

            let mut list: Vec<Value> = places
                .into_values()
                .map(|a| {
                    let mut users: Vec<Value> = a.users.into_iter().map(|(id, (name, plays, sign_ins))| json!({ "id": id, "name": name, "plays": plays, "sign_ins": sign_ins })).collect();
                    users.sort_by_key(|u| std::cmp::Reverse(u["plays"].as_i64().unwrap_or(0) * 1000 + u["sign_ins"].as_i64().unwrap_or(0)));
                    json!({
                        "label": place_label_in(&a.place, lang), "home": a.home, "city": a.place.city, "region": a.place.region, "country": a.place.country,
                        "country_code": a.place.country_code, "latitude": a.place.latitude, "longitude": a.place.longitude,
                        "plays": a.plays, "watch_s": a.watch_s, "sign_ins": a.sign_ins, "addresses": a.ips, "last_seen": a.last_seen, "users": users,
                    })
                })
                .collect();
            list.sort_by_key(|v| std::cmp::Reverse(v["plays"].as_i64().unwrap_or(0) * 1000 + v["sign_ins"].as_i64().unwrap_or(0)));
            let failed: Vec<Value> = failed
                .into_values()
                .map(|(p, n, last, names)| json!({ "label": place_label_in(&p, lang), "country_code": p.country_code, "latitude": p.latitude, "longitude": p.longitude, "attempts": n, "last_at": last, "events": names }))
                .collect();

            let open_alerts: i64 = c.query_row(
                &format!("SELECT COUNT(*) FROM security_alerts WHERE resolved_at IS NULL AND {}", whose("user_id")),
                params_from_iter(who.iter()),
                |r| r.get(0),
            )?;
            let unplaced: i64 = c.query_row(
                &format!("SELECT COUNT(DISTINCT p.remote_ip) FROM visible_playbacks p JOIN ip_locations l ON l.ip = p.remote_ip
                          WHERE COALESCE(p.is_local, 0) = 0 AND l.latitude IS NULL AND l.country_code IS NULL AND p.started_at >= ?1 AND {}", whose("p.user_id")),
                params_from_iter(with_who(since.into()).iter()),
                |r| r.get(0),
            )?;
            Ok(json!({ "places": list, "countries": countries, "failed": failed, "now_playing": now_playing, "open_alerts": open_alerts,
                       "home_known": home.is_some(), "addresses_without_place": unplaced, "days": scope.days }))
        })
        .await?;
    body["database"] = database_json(&app);
    body["can_manage"] = json!(user.perms.manage);
    Ok(Json(body))
}

#[derive(Deserialize)]
pub struct AlertsQuery {
    /// open (default) | resolved | all
    status: Option<String>,
    user_id: Option<String>,
    page: Option<i64>,
    per_page: Option<i64>,
}

fn alert_json(r: &crate::db::rusqlite::Row) -> crate::db::rusqlite::Result<Value> {
    let details: String = r.get(6)?;
    Ok(json!({
        "id": r.get::<_, i64>(0)?, "kind": r.get::<_, String>(1)?, "severity": r.get::<_, String>(2)?,
        "user_id": r.get::<_, String>(3)?, "user_name": r.get::<_, String>(4)?, "at": r.get::<_, i64>(5)?,
        "details": serde_json::from_str::<Value>(&details).unwrap_or(Value::Null),
        "resolved_at": r.get::<_, Option<i64>>(7)?, "resolved_by": r.get::<_, Option<String>>(8)?, "note": r.get::<_, Option<String>>(9)?,
        "muted": r.get::<_, bool>(10)?, "has_image": r.get::<_, Option<bool>>(11)?.unwrap_or(false),
    }))
}

pub async fn alerts(State(app): State<App>, user: AuthUser, Query(q): Query<AlertsQuery>) -> ApiResult {
    gate(&user)?;
    let per_page = q.per_page.unwrap_or(25).clamp(1, 100);
    let page = crate::stats::page_number(q.page);
    // The page's person picker sends several (`a,b`), as every filter does; read as one id it matched nobody.
    let who: Vec<String> = q.user_id.as_deref().unwrap_or_default().split(',').map(norm_id).filter(|s| !s.is_empty()).collect();
    let status = q.status.unwrap_or_else(|| "open".into());
    let mut body = app
        .db
        .call(move |c| {
            let mut wh = vec!["1 = 1".to_string()];
            let mut args: Vec<SqlValue> = vec![];
            match status.as_str() {
                "resolved" => wh.push("a.resolved_at IS NOT NULL".into()),
                "all" => {}
                _ => wh.push("a.resolved_at IS NULL".into()),
            }
            if !who.is_empty() {
                wh.push(format!("a.user_id IN ({})", vec!["?"; who.len()].join(", ")));
                args.extend(who.into_iter().map(SqlValue::from));
            }
            let wh = wh.join(" AND ");
            let total: i64 = c.query_row(&format!("SELECT COUNT(*) FROM security_alerts a WHERE {wh}"), params_from_iter(args.iter()), |r| r.get(0))?;
            let open: i64 = c.query_row("SELECT COUNT(*) FROM security_alerts WHERE resolved_at IS NULL", [], |r| r.get(0))?;
            let sql = format!(
                "SELECT a.id, a.kind, a.severity, a.user_id, COALESCE(u.name, a.user_name), a.at, a.details, a.resolved_at, a.resolved_by, a.note, a.muted, (u.image_tag IS NOT NULL)
                 FROM security_alerts a LEFT JOIN users u ON u.id = a.user_id WHERE {wh}
                 ORDER BY a.at DESC, a.id DESC LIMIT {per_page} OFFSET {}",
                (page - 1) * per_page
            );
            let rows: Vec<Value> = c.prepare(&sql)?.query_map(params_from_iter(args.iter()), alert_json)?.collect::<Result<_, _>>()?;
            Ok(json!({ "rows": rows, "total": total, "open": open, "page": page, "per_page": per_page }))
        })
        .await?;
    let lang = crate::i18n::current();
    for alert in body["rows"].as_array_mut().into_iter().flatten() {
        say_alert(alert, lang);
    }
    Ok(Json(body))
}

#[derive(Deserialize, Default)]
pub struct ResolveBody {
    #[serde(default)]
    note: Option<String>,
    /// For impossible travel: never report these two places for this person again.
    #[serde(default)]
    mute: bool,
}

pub async fn resolve(State(app): State<App>, Manager(user): Manager, Path(id): Path<i64>, Json(body): Json<ResolveBody>) -> ApiResult {
    gate(&user)?;
    let note = body.note.map(|n| n.trim().chars().take(500).collect::<String>()).filter(|n| !n.is_empty());
    let mute = body.mute;
    let actor = crate::audit::Actor::from(&user);
    let changed = app
        .db
        .call(move |c| {
            let tx = c.transaction()?;
            let found: Option<(String, String, String)> = tx.query_row("SELECT kind, user_id, details FROM security_alerts WHERE id = ?1", [id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))).optional()?;
            let Some((kind, user_id, details)) = found else { return Ok(None) };
            let now = db::now();
            tx.execute(
                // A mute is undone by reopening, never by resolving again without the box ticked.
                "UPDATE security_alerts SET resolved_at = COALESCE(resolved_at, ?1), resolved_by = ?2, note = ?3, muted = MAX(muted, ?4) WHERE id = ?5",
                params![now, user.name, note, mute && kind == "impossible_travel", id],
            )?;
            let mut also = 0;
            if mute && kind == "impossible_travel" {
                // The same two places, still open: they are answered by this too.
                let pair = serde_json::from_str::<Value>(&details).ok().and_then(|d| d["pair"].as_str().map(str::to_string));
                if let Some(pair) = pair {
                    also = tx.execute(
                        "UPDATE security_alerts SET resolved_at = ?1, resolved_by = ?2, note = ?5 WHERE resolved_at IS NULL AND user_id = ?3 AND kind = 'impossible_travel' AND json_extract(details, '$.pair') = ?4",
                        params![now, user.name, user_id, pair, crate::i18n::say(FILED_IN, "server.security.same_two_places", &[])],
                    )?;
                }
            }
            tx.commit()?;
            Ok(Some(also))
        })
        .await?;
    match changed {
        Some(also) => {
            crate::audit::record(&app, crate::audit::Entry::new("alert_resolved", actor).target(id.to_string()).detail(json!({ "muted": mute, "also_resolved": also })));
            Ok(Json(json!({ "ok": true, "also_resolved": also })))
        }
        None => Err(ApiError::not_found("Alert")),
    }
}

pub async fn reopen(State(app): State<App>, Manager(user): Manager, Path(id): Path<i64>) -> ApiResult {
    gate(&user)?;
    let n = app.db.call(move |c| Ok(c.execute("UPDATE security_alerts SET resolved_at = NULL, resolved_by = NULL, note = NULL, muted = 0 WHERE id = ?1", [id])?)).await?;
    if n == 0 {
        return Err(ApiError::not_found("Alert"));
    }
    crate::audit::record(&app, crate::audit::Entry::new("alert_reopened", crate::audit::Actor::from(&user)).target(id.to_string()));
    Ok(Json(json!({ "ok": true })))
}

pub async fn resolve_all(State(app): State<App>, Manager(user): Manager) -> ApiResult {
    gate(&user)?;
    let actor = crate::audit::Actor::from(&user);
    let n = app
        .db
        .call(move |c| Ok(c.execute("UPDATE security_alerts SET resolved_at = ?1, resolved_by = ?2 WHERE resolved_at IS NULL", params![db::now(), user.name])?))
        .await?;
    crate::audit::record(&app, crate::audit::Entry::new("alert_resolved", actor).target("all").detail(json!({ "resolved": n })));
    Ok(Json(json!({ "ok": true, "resolved": n })))
}

/// Fetch the database now: the `geoip` job by hand, for a manager who may read this page, waiting the job's minute.
pub async fn download_database(State(app): State<App>, Manager(user): Manager) -> ApiResult<Response> {
    gate(&user)?;
    if std::env::var("FINSTATS_GEOIP_DB").is_ok_and(|p| !p.trim().is_empty()) {
        return Err(ApiError::bad_request(crate::i18n::t("server.security.database_from_env", &[])));
    }
    crate::api::by_hand(&user, "geoip")?;
    if !geo::spawn_download(&app) {
        return Err(ApiError::new(StatusCode::CONFLICT, crate::i18n::t("server.security.already_downloading", &[])));
    }
    crate::audit::record(&app, crate::audit::Entry::new("task_run", crate::audit::Actor::from(&user)).target("geoip"));
    Ok((StatusCode::ACCEPTED, Json(json!({ "ok": true }))).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The button that fetches the database took `manage` alone, skipped the minute every job by hand waits, and worked
    /// every place out again over the whole history at each press, for somebody not allowed to read the Security page.
    #[tokio::test]
    async fn fetching_the_database_needs_the_security_page_and_waits_the_jobs_minute() {
        let app = crate::state::test_app();
        let manager = AuthUser {
            id: "m-geo".into(), name: "mallory".into(), is_admin: false, credential: crate::auth::Credential::Session, ip: None,
            perms: crate::auth::Perms { manage: true, ..Default::default() },
        };
        let refused = |r: ApiResult<Response>| r.err().map(|e| e.0);
        assert_eq!(refused(download_database(State(app.clone()), Manager(manager.clone())).await), Some(StatusCode::FORBIDDEN));
        let reader = AuthUser { perms: crate::auth::Perms { see_network: true, see_everyone: true, ..manager.perms }, ..manager };
        crate::api::by_hand(&reader, "geoip").unwrap();
        assert_eq!(refused(download_database(State(app.clone()), Manager(reader)).await), Some(StatusCode::TOO_MANY_REQUESTS), "the job was run a moment ago");
    }

    #[test]
    fn an_alert_is_filed_in_the_source_and_read_in_the_readers_language() {
        let spot = |home: bool| spot_of(&Place { city: Some("Oslo".into()), country: Some("Norway".into()), latitude: Some(59.9), longitude: Some(10.7), ..Default::default() }, home).unwrap();
        let played = Sighting { at: 0, until: 60, spot: spot(true), what: What { title: "Big Buck Bunny".into(), device: Some("Living room TV".into()) }, ip: "192.168.1.10".into(), id: "play:1".into() };
        let signed_in = Sighting { what: "alice signed in".into(), spot: spot(false), ..played.clone() };
        let (from, to) = (side(&played), side(&signed_in));
        assert_eq!((from["place"].as_str(), from["what"].as_str()), (Some("Home (Oslo, Norway)"), Some("Big Buck Bunny on Living room TV")), "filed in the source's words");
        let mut alert = json!({ "details": { "from": from, "to": to }, "note": "Found in older history", "resolved_by": "finstats" });
        say_alert(&mut alert, "nb-NO");
        assert_eq!(alert["details"]["from"]["place"], "Hjemme (Oslo, Norway)");
        assert_eq!(alert["details"]["from"]["what"], "Big Buck Bunny på Living room TV");
        assert_eq!((alert["details"]["to"]["place"].as_str(), alert["details"]["to"]["what"].as_str()), (Some("Oslo, Norway"), Some("alice signed in")), "the log's own line is Jellyfin's words");
        assert_eq!(alert["note"], "Funnet i eldre historikk");
        // Filed before the parts were kept: the words it was filed with.
        let mut old = json!({ "details": { "from": { "place": "Somewhere", "what": "Big Buck Bunny on TV" } }, "note": "On holiday", "resolved_by": "alice" });
        say_alert(&mut old, "nn-NO");
        assert_eq!((old["details"]["from"]["what"].as_str(), old["note"].as_str()), (Some("Big Buck Bunny on TV"), Some("On holiday")));
    }

    #[test]
    fn how_far_apart_and_how_long_are_said_in_words_with_the_right_number() {
        for (s, said) in [(0, "at the same time"), (-5, "at the same time"), (1, "1 second apart"), (45, "45 seconds apart"), (90, "90 seconds apart"),
            (91, "2 minutes apart"), (5_400, "90 minutes apart"), (5_401, "2 hours apart"), (172_800, "48 hours apart"), (259_201, "3 days apart")] {
            assert_eq!(human_gap(s), said, "{s} s apart");
        }
        for (s, said) in [(0, "under a minute"), (60, "under a minute"), (61, "1 minute"), (89, "1 minute"), (90, "2 minutes"), (5_400, "90 minutes"),
            (5_401, "2 hours"), (7_200, "2 hours")] {
            assert_eq!(human_span(s), said, "{s} s long");
        }
    }

    #[test]
    fn a_place_is_named_by_as_much_of_it_as_is_known() {
        let p = |city: Option<&str>, country: Option<&str>, code: Option<&str>| Place { city: city.map(Into::into), country: country.map(Into::into), country_code: code.map(Into::into), ..Place::default() };
        assert_eq!(place_label(&p(Some("London"), Some("United Kingdom"), Some("GB"))), "London, United Kingdom");
        assert_eq!(place_label(&p(None, Some("Norway"), Some("NO"))), "Norway");
        assert_eq!(place_label(&p(Some("Reykjavík"), None, None)), "Reykjavík");
        assert_eq!(place_label(&p(None, None, Some("IS"))), "IS");
        assert_eq!(place_label(&p(None, None, None)), "Unknown place");
    }

    /// Names that are no account are typed by whoever is knocking, as many as they like: keyed by the name, a thousand
    /// invented ones were a thousand notifications, a real alert waited behind them, and each carried the typed text.
    /// They are one notice a window, server-wide, and say no name.
    #[test]
    fn a_thousand_invented_names_are_one_notice_a_window_and_name_none_of_them() {
        use crate::notify::tests::{bus, target};
        let c = conn();
        c.execute("INSERT INTO users(id, name, updated_at) VALUES ('ua', 'alice', 1)", []).unwrap();
        let f = bus(&c, vec![target(1, None, &[Kind::FailedSignIns])]);
        let now = (db::now() - 3_600).div_euclid(BURST_WINDOW_S) * BURST_WINDOW_S; // the start of a window, so none of it falls in the next
        let mut id = 0;
        for name in 0..200 {
            for n in 0..5 {
                id += 1;
                c.execute(
                    "INSERT INTO server_events(id, date, name, type, severity, user_id, remote_ip) VALUES (?1, ?2, ?3, 'AuthenticationFailed', 'Error', NULL, '203.0.113.9')",
                    params![id, now + n, format!("Failed login try from <!channel> name {name}")],
                )
                .unwrap();
            }
        }
        for n in 0..5 {
            id += 1;
            // As Jellyfin writes it: no user id, the name at the end of the line.
            c.execute("INSERT INTO server_events(id, date, name, type, severity, user_id, remote_ip) VALUES (?1, ?2, 'Failed login try from Alice', 'AuthenticationFailed', 'Error', NULL, '203.0.113.9')", params![id, now + 100 + n]).unwrap();
        }
        assert_eq!(scan_sign_ins(&c, &f).unwrap(), 2, "one notice for the invented names, one for alice");
        let titles: Vec<String> = c.prepare("SELECT data FROM notify_events").unwrap().query_map([], |r| r.get(0)).unwrap().map(Result::unwrap).collect();
        assert!(titles.iter().all(|t| !t.contains("<!channel>")), "a typed name reached a notification: {titles:?}");
        assert_eq!(scan_sign_ins(&c, &f).unwrap(), 0, "a rescan found them again");
    }

    /// Every play that begins scans its person's history, and nothing coalesced them: a client starting plays from new
    /// device ids ran several whole-history scans a second on the pool the collector shares. One runs at a time for a
    /// person, and any number asked for meanwhile are one more after it.
    #[test]
    fn scans_of_one_person_asked_for_while_one_runs_are_one_more() {
        let s = Coalesce::default();
        assert!(s.begin("alice"), "nothing running: scan");
        assert!(!s.begin("alice") && !s.begin("alice"), "one is running: noted, not started");
        assert!(s.begin("bob"), "somebody else's runs beside it");
        assert!(s.finish("alice"), "asked for meanwhile: once more");
        assert!(!s.finish("alice"), "and then done");
        assert!(s.begin("alice"));
    }

    #[test]
    fn several_failed_sign_ins_in_a_window_are_one_burst_however_many_there_are() {
        let row = |id: i64, at: i64, who: &str| (id, at, who.to_string(), Some(format!("u-{who}")), "203.0.113.9".to_string());
        // Four for alice is nothing; the fifth completes a burst. Bob's two in between are his own count.
        let mut rows: Vec<_> = (0..4).map(|n| row(n, 1_000 + n * 60, "alice")).collect();
        rows.push(row(10, 1_100, "bob"));
        rows.push(row(11, 1_150, "bob"));
        rows.push(row(12, 1_300, "alice"));
        let found = bursts(&rows, 900, 5);
        assert_eq!(found.len(), 1, "one burst, for the account that reached five");
        assert_eq!((found[0].who.as_str(), found[0].window, found[0].count), ("alice", 1, 5));

        // Spread beyond the window, five failures are somebody who keeps mistyping, not a burst.
        let slow: Vec<_> = (0..5).map(|n| row(n, 1_000 + n * 1_000, "alice")).collect();
        assert!(bursts(&slow, 900, 5).is_empty());

        // Ten thousand in a row are one burst a window, counting all of them: each run of five was a notice of its own,
        // and none could ever reach an alert.
        let many: Vec<_> = (0..10_000).map(|n| row(n, 1_000 + n / 20, "alice")).collect();
        let found = bursts(&many, 900, 5);
        assert_eq!(found.iter().map(|b| (b.window, b.count)).collect::<Vec<_>>(), vec![(1, 10_000)]);
        assert_eq!(sign_in_severity(found[0].count), notify::ALERT);
        assert_eq!(sign_in_severity(5), notify::WARN);
    }

    /// A burst on an account is said once as a warning and, should it grow to an alert, once more as one: never once for
    /// every five tries, and an alert is reachable however the reads of the activity log fall.
    #[test]
    fn a_long_run_of_failed_sign_ins_is_a_warning_and_then_an_alert_and_nothing_more() {
        use crate::notify::tests::{bus, target};
        let c = conn();
        c.execute("INSERT INTO users(id, name, updated_at) VALUES ('ua', 'alice', 1)", []).unwrap();
        let f = bus(&c, vec![target(1, None, &[Kind::FailedSignIns])]);
        let start = (db::now() - 3_600).div_euclid(BURST_WINDOW_S) * BURST_WINDOW_S;
        let fail = |from: i64, to: i64| {
            for id in from..to {
                c.execute("INSERT INTO server_events(id, date, name, type, severity, user_id, remote_ip) VALUES (?1, ?2, 'Failed login try from alice', 'AuthenticationFailed', 'Error', NULL, '203.0.113.9')", params![id, start + id]).unwrap();
            }
        };
        fail(0, 5);
        assert_eq!(scan_sign_ins(&c, &f).unwrap(), 1, "five: a warning");
        fail(5, 400);
        assert_eq!(scan_sign_ins(&c, &f).unwrap(), 1, "four hundred: an alert");
        assert_eq!(scan_sign_ins(&c, &f).unwrap(), 0, "a rescan found them again");
        let severities: Vec<String> = c.prepare("SELECT severity FROM notify_events ORDER BY id").unwrap().query_map([], |r| r.get(0)).unwrap().map(Result::unwrap).collect();
        assert_eq!(severities, vec![notify::WARN, notify::ALERT]);
    }

    /// Jellyfin writes the name as it was typed after its own words, so a name is an account's only when the whole of it
    /// is: `x alice` is somebody typing a name that is no account, not twenty tries on alice's.
    #[test]
    fn a_failed_sign_in_is_an_account_s_only_when_the_whole_name_typed_is_its_name() {
        assert_eq!(typed_name("Failed login try from alice", None), Some("alice"));
        assert_eq!(typed_name("Failed login attempt from x alice", None), Some("x alice"));
        let lines = ["Échec de connexion de alice", "Échec de connexion de bob", "Échec de connexion de x alice"];
        let learned = template_prefix(lines.iter().copied());
        assert_eq!(learned.as_deref(), Some("Échec de connexion de "), "the server's own words, from what its lines share");
        assert_eq!(typed_name(lines[2], learned.as_deref()), Some("x alice"));
        assert_eq!(template_prefix(["Échec de connexion de x alice"].into_iter()), None, "one line alone says nothing about where the name starts");

        use crate::notify::tests::{bus, target};
        let c = conn();
        c.execute("INSERT INTO users(id, name, updated_at) VALUES ('ua', 'alice', 1)", []).unwrap();
        let f = bus(&c, vec![target(1, None, &[Kind::FailedSignIns])]);
        let start = (db::now() - 3_600).div_euclid(BURST_WINDOW_S) * BURST_WINDOW_S;
        for id in 0..20 {
            c.execute("INSERT INTO server_events(id, date, name, type, severity, user_id, remote_ip) VALUES (?1, ?2, 'Failed login try from x alice', 'AuthenticationFailed', 'Error', NULL, '203.0.113.9')", params![id, start + id]).unwrap();
        }
        assert_eq!(scan_sign_ins(&c, &f).unwrap(), 1);
        let said: String = c.query_row("SELECT title FROM notify_events", [], |r| r.get(0)).unwrap();
        assert!(!said.contains("alice"), "a name typed after another was filed as alice's: {said}");
    }


    fn spot(key: &str, cc: &str, lat: f64, lon: f64) -> Spot {
        Spot { key: key.into(), label: key.into(), country_code: Some(cc.into()), country: Some(cc.into()), lat, lon }
    }
    fn london() -> Spot { spot("home", "GB", 51.5, -0.1) }
    fn paris() -> Spot { spot("48.9,2.4", "FR", 48.86, 2.35) }
    fn new_york() -> Spot { spot("40.7,-74.0", "US", 40.71, -74.0) }
    fn leeds() -> Spot { spot("53.8,-1.5", "GB", 53.8, -1.55) }

    fn seen(id: i64, spot: Spot, at: i64, minutes: i64) -> Sighting {
        Sighting { at, until: at + minutes * 60, spot, what: "Big Buck Bunny".into(), ip: "203.0.113.9".into(), id: format!("play:{id}") }
    }
    const RULES: Rules = Rules { speed_kmh: 900.0, min_km: 300.0 };
    const H: i64 = 3600;

    fn kinds(f: &[Finding]) -> Vec<&str> {
        f.iter().map(|f| f.kind).collect()
    }

    #[test]
    fn a_flight_is_fine_and_a_teleport_is_not() {
        // London, then New York eight hours after the film ended: about 700 km/h.
        let ok = [seen(1, london(), 0, 90), seen(2, new_york(), 90 * 60 + 8 * H, 30)];
        assert_eq!(kinds(&detect("alice", &ok, &RULES, &HashSet::new())), ["new_country"]);

        // The same trip in one hour.
        let bad = [seen(1, london(), 0, 90), seen(2, new_york(), 90 * 60 + H, 30)];
        let found = detect("alice", &bad, &RULES, &HashSet::new());
        assert_eq!(kinds(&found), ["impossible_travel", "new_country"]);
        assert_eq!(found[0].dedupe, "travel:alice:play:2");
        assert_eq!(found[0].details["overlap"], false);
        assert!(found[0].details["speed_kmh"].as_f64().unwrap() > 5000.0);
        assert_eq!(found[0].details["from"]["home"], true);
    }

    #[test]
    fn two_places_at_once_is_reported_without_a_speed() {
        let both = [seen(1, london(), 0, 120), seen(2, new_york(), 30 * 60, 20)];
        let found = detect("alice", &both, &RULES, &HashSet::new());
        assert_eq!(found[0].kind, "impossible_travel");
        assert_eq!(found[0].details["overlap"], true);
        assert!(found[0].details["speed_kmh"].is_null());
    }

    #[test]
    fn a_long_play_still_counts_after_a_short_one() {
        // A film at home until 03:00, a glance at the phone at home, then New York at 03:20.
        // Against the glance it is a slow trip; against the film it is not.
        let s = [seen(1, london(), 0, 180), seen(2, london(), 600, 0), seen(3, new_york(), 3 * H + 20 * 60, 10)];
        assert_eq!(kinds(&detect("alice", &s, &RULES, &HashSet::new())), ["impossible_travel", "new_country"]);
    }

    #[test]
    fn nearby_places_never_count_however_fast() {
        // Home Wi-Fi, then a phone whose carrier is "in" Leeds a minute later: 270 km, under the minimum.
        let s = [seen(1, london(), 0, 10), seen(2, leeds(), 11 * 60, 10)];
        assert!(detect("alice", &s, &RULES, &HashSet::new()).is_empty());
    }

    #[test]
    fn a_pair_reports_once_a_day_and_never_when_muted() {
        // Flapping between home and a VPN exit all evening, and again two days later.
        let mut s = vec![];
        for i in 0..6 {
            s.push(seen(i, if i % 2 == 0 { london() } else { new_york() }, i * 20 * 60, 5));
        }
        s.push(seen(10, london(), 48 * H, 5));
        s.push(seen(11, new_york(), 48 * H + 600, 5));
        let found = detect("alice", &s, &RULES, &HashSet::new());
        assert_eq!(found.iter().filter(|f| f.kind == "impossible_travel").count(), 2);

        let muted: HashSet<String> = [pair_key(&london(), &new_york())].into();
        assert_eq!(kinds(&detect("alice", &s, &RULES, &muted)), ["new_country"]);
    }

    #[test]
    fn the_first_country_is_home_and_each_new_one_reports_once() {
        let s = [seen(1, london(), 0, 10), seen(2, leeds(), 24 * H, 10), seen(3, paris(), 48 * H, 10), seen(4, paris(), 72 * H, 10), seen(5, london(), 96 * H, 10)];
        let found = detect("alice", &s, &RULES, &HashSet::new());
        assert_eq!(kinds(&found), ["new_country"]);
        assert_eq!(found[0].dedupe, "country:alice:FR");
        assert_eq!(found[0].details["known"], 1);
    }

    #[test]
    fn the_address_is_found_whatever_the_servers_language() {
        assert_eq!(event_ip("IP address: 203.0.113.9").as_deref(), Some("203.0.113.9"));
        assert_eq!(event_ip("IP-adresse: ::ffff:203.0.113.9").as_deref(), Some("203.0.113.9"));
        assert_eq!(event_ip("Adresse IP : 2001:db8::1.").as_deref(), Some("2001:db8::1"));
        assert_eq!(event_ip("App: Jellyfin Web"), None);
        assert_eq!(event_ip(""), None);
    }

    fn conn() -> Connection {
        let c = Connection::open_in_memory().unwrap();
        for m in crate::db::MIGRATIONS {
            c.execute_batch(m).unwrap();
        }
        c
    }

    #[test]
    fn old_findings_are_filed_as_history_and_a_rescan_adds_nothing() {
        let c = conn();
        let now = db::now();
        c.execute_batch(&format!(
            "INSERT INTO ip_locations(ip, country_code, country, city, latitude, longitude, looked_up_at) VALUES
                ('198.51.100.1', 'GB', 'United Kingdom', 'London', 51.5, -0.1, 0), ('198.51.100.2', 'US', 'United States', 'New York', 40.7, -74.0, 0);
             INSERT INTO playbacks(source, user_id, user_name, item_id, item_name, item_type, started_at, ended_at, duration_s, remote_ip, is_local) VALUES
                ('live', 'u1', 'alice', 'i1', 'Big Buck Bunny', 'Movie', {old}, {old} + 600, 600, '198.51.100.1', 0),
                ('live', 'u1', 'alice', 'i1', 'Big Buck Bunny', 'Movie', {old} + 1200, {old} + 1800, 600, '198.51.100.2', 0),
                ('live', 'u1', 'alice', 'i1', 'Big Buck Bunny', 'Movie', {recent}, {recent} + 600, 600, '198.51.100.1', 0),
                ('live', 'u1', 'alice', 'i1', 'Big Buck Bunny', 'Movie', {recent} + 1200, {recent} + 1800, 600, '198.51.100.2', 0);",
            old = now - 90 * 86_400,
            recent = now - 86_400
        ))
        .unwrap();
        assert_eq!(file_alerts(&c, &RULES, None, None).unwrap(), 3);
        let filed: Vec<(String, bool, Option<String>)> = c
            .prepare("SELECT kind, resolved_at IS NOT NULL, resolved_by FROM security_alerts ORDER BY at")
            .unwrap()
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .unwrap()
            .map(Result::unwrap)
            .collect();
        // Three months old: history. Yesterday: open.
        assert_eq!(filed[0], ("impossible_travel".into(), true, Some("finstats".into())));
        assert_eq!(filed[1], ("new_country".into(), true, Some("finstats".into())));
        assert_eq!(filed[2], ("impossible_travel".into(), false, None));
        assert_eq!(file_alerts(&c, &RULES, None, None).unwrap(), 0);
        assert_eq!(file_alerts(&c, &RULES, Some("u1"), None).unwrap(), 0);
    }
}
