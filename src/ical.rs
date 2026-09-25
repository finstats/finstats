//! A calendar of what is coming, for a phone to subscribe to: `GET /api/calendar.ics?key=…`, the
//! one place a key travels in an address, because a subscribed calendar cannot send a header. A
//! calendar-scoped key opens this and nothing else. The feed names nobody: it is read through
//! `pipeline::entries_for`, whose answer holds no names by construction.
//!
//! RFC 5545, by hand: text escaped, lines folded at 75 octets, CRLF throughout. An episode is a
//! timed event of a flat hour — its runtime is an average at best and an "airs" block is a reminder,
//! not a schedule; a film is an all-day event on its day.

use axum::body::Body;
use axum::extract::{Query, State};
use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE};
use axum::response::Response;
use chrono::{DateTime, NaiveDate};
use serde::Deserialize;

use crate::auth::CalendarKey;
use crate::db;
use crate::pipeline::Entry;
use crate::state::{ApiResult, App};

const RELEASE_LABEL: [(&str, &str); 4] = [("air", "Airs"), ("cinema", "In cinemas"), ("digital", "Digital release"), ("physical", "Disc release")];
const FINALE_LABEL: [(&str, &str); 3] = [("season", "Season finale"), ("series", "Series finale"), ("midseason", "Mid-season finale")];

/// Text as a property value: backslash, semicolon and comma escaped, a line break spelled `\n`.
pub fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            ';' => out.push_str("\\;"),
            ',' => out.push_str("\\,"),
            '\n' => out.push_str("\\n"),
            '\r' => {}
            c => out.push(c),
        }
    }
    out
}

/// One content line, folded at 75 octets on character boundaries, every physical line ending CRLF.
pub fn fold(line: &str) -> String {
    let mut out = String::new();
    let mut width = 0;
    let mut first = true;
    for ch in line.chars() {
        let w = ch.len_utf8();
        let limit = if first { 75 } else { 74 };
        if width + w > limit {
            out.push_str("\r\n ");
            width = 0;
            first = false;
        }
        out.push(ch);
        width += w;
    }
    out.push_str("\r\n");
    out
}

fn label(table: &'static [(&'static str, &'static str)], key: &str) -> Option<&'static str> {
    table.iter().find(|(k, _)| *k == key).map(|(_, v)| *v)
}

/// "Low Orbit S03E10 · Re-entry"; "Title (2026) · In cinemas".
pub fn summary(e: &Entry) -> String {
    if e.kind == "episode" {
        let code = match (e.season, e.episode) {
            (Some(s), Some(n)) => format!(" S{s:02}E{n:02}"),
            (None, Some(n)) => format!(" E{n:02}"),
            _ => String::new(),
        };
        let show = e.series_title.clone().unwrap_or_default();
        let title = e.title.trim();
        if title.is_empty() || title.eq_ignore_ascii_case("tba") { format!("{show}{code}") } else { format!("{show}{code} · {title}") }
    } else {
        let year = e.year.map(|y| format!(" ({y})")).unwrap_or_default();
        let what = label(&RELEASE_LABEL, &e.release).unwrap_or("Release");
        format!("{}{year} · {what}", e.title)
    }
}

fn stamp(ts: i64) -> String {
    DateTime::from_timestamp(ts, 0).map(|t| t.format("%Y%m%dT%H%M%SZ").to_string()).unwrap_or_default()
}

fn compact_day(day: &str) -> String {
    day.replace('-', "")
}

fn next_day(day: &str) -> String {
    NaiveDate::parse_from_str(day, "%Y-%m-%d").ok().and_then(|d| d.succ_opt()).map(|d| d.format("%Y%m%d").to_string()).unwrap_or_else(|| compact_day(day))
}

/// One event. `public_url` empty means no link.
pub fn vevent(e: &Entry, now: i64, public_url: &str, you_follow: bool) -> String {
    let mut lines: Vec<String> = vec!["BEGIN:VEVENT".into()];
    lines.push(format!("UID:{}-{}-{}-{}@finstats", e.service_id, e.kind, e.external_id, e.release));
    lines.push(format!("DTSTAMP:{}", stamp(now)));
    match e.at.filter(|_| e.kind == "episode") {
        Some(at) => {
            lines.push(format!("DTSTART:{}", stamp(at)));
            lines.push("DURATION:PT1H".into());
        }
        None => {
            lines.push(format!("DTSTART;VALUE=DATE:{}", compact_day(&e.day)));
            lines.push(format!("DTEND;VALUE=DATE:{}", next_day(&e.day)));
        }
    }
    lines.push(format!("SUMMARY:{}", escape(&summary(e))));
    lines.push(format!("CATEGORIES:{}", if e.kind == "episode" { "Episode" } else { "Film" }));
    let link = match (&e.item_id, public_url.trim_end_matches('/')) {
        (Some(id), url) if !url.is_empty() => Some(format!("{url}/items/{id}")),
        _ => None,
    };
    let mut notes: Vec<String> = vec![];
    if let Some(f) = e.finale.as_deref().and_then(|f| label(&FINALE_LABEL, f)) {
        notes.push(f.to_string());
    }
    if e.has_file {
        notes.push("Already here".into());
    }
    if you_follow {
        notes.push("You watch this".into());
    }
    if let Some(l) = &link {
        notes.push(l.clone());
    }
    if !notes.is_empty() {
        lines.push(format!("DESCRIPTION:{}", escape(&notes.join("\n"))));
    }
    if let Some(l) = link {
        lines.push(format!("URL:{l}"));
    }
    lines.push("END:VEVENT".into());
    lines.iter().map(|l| fold(l)).collect()
}

/// The whole file around its events.
pub fn calendar(events: &[String]) -> String {
    let mut out = String::new();
    for l in ["BEGIN:VCALENDAR", "VERSION:2.0", "PRODID:-//finstats//EN", "CALSCALE:GREGORIAN", "METHOD:PUBLISH", "X-WR-CALNAME:finstats: coming up"] {
        out.push_str(&fold(l));
    }
    for e in events {
        out.push_str(e);
    }
    out.push_str(&fold("END:VCALENDAR"));
    out
}

#[derive(Deserialize)]
pub struct FeedQuery {
    /// Read by the extractor; here so the query parses.
    #[allow(dead_code)]
    key: Option<String>,
    days: Option<i64>,
    /// `1`, `true`, `yes`, `on` — an address typed by hand, not a form finstats wrote.
    mine: Option<String>,
}

/// Whether a hand-typed query flag says yes.
fn truthy(v: Option<&str>) -> bool {
    matches!(v.map(|s| s.trim().to_ascii_lowercase()).as_deref(), Some("1" | "true" | "yes" | "on"))
}

/// `GET /api/calendar.ics?key=&days=&mine=` — the key's owner's agenda, as a calendar.
pub async fn feed(State(app): State<App>, CalendarKey(user): CalendarKey, Query(q): Query<FeedQuery>) -> ApiResult<Response> {
    let days = q.days.unwrap_or(90).clamp(1, 90);
    let mine = truthy(q.mine.as_deref());
    let subject = user.id.clone();
    let public_url = app.settings().public_url.clone();
    let rows = app.db.call(move |c| crate::pipeline::entries_for(c, days, &subject, mine)).await?;
    let now = db::now();
    let events: Vec<String> = rows.iter().map(|(e, you)| vevent(e, now, &public_url, *you)).collect();
    Ok(Response::builder()
        .header(CONTENT_TYPE, "text/calendar; charset=utf-8")
        .header(CACHE_CONTROL, "private, no-store")
        .header("content-disposition", "inline; filename=\"finstats.ics\"")
        .body(Body::from(calendar(&events)))
        .unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::Entry;

    fn episode() -> Entry {
        Entry { service_id: 1, kind: "episode".into(), release: "air".into(), day: "2026-10-02".into(), at: Some(1_790_000_000), series_title: Some("Low Orbit".into()),
                title: "Re-entry".into(), season: Some(3), episode: Some(10), finale: Some("season".into()), year: None, tvdb_id: Some(5), tmdb_id: None,
                arr_media_id: 15, has_file: false, item_id: Some("abc".into()), external_id: 4242 }
    }
    fn film() -> Entry {
        Entry { service_id: 2, kind: "movie".into(), release: "cinema".into(), day: "2026-10-03".into(), at: None, series_title: None,
                title: "Paper Moons, Part 2; the Return".into(), season: None, episode: None, finale: None, year: Some(2026), tvdb_id: None, tmdb_id: Some(9),
                arr_media_id: 7, has_file: false, item_id: None, external_id: 77 }
    }

    #[test]
    fn text_is_escaped_the_way_rfc_5545_says() {
        assert_eq!(escape("a, b; c\nd\\e\r\n"), "a\\, b\\; c\\nd\\\\e\\n");
    }

    #[test]
    fn a_long_summary_folds_on_octets_not_characters() {
        let long = format!("SUMMARY:{}", "ø".repeat(60));   // 8 + 120 octets
        let folded = fold(&long);
        for line in folded.split("\r\n").filter(|l| !l.is_empty()) {
            assert!(line.len() <= 75, "{} octets", line.len());
            assert!(std::str::from_utf8(line.as_bytes()).is_ok());
        }
        assert!(folded.ends_with("\r\n"));
        assert!(folded.contains("\r\n "), "a continuation line starts with a space");
        assert_eq!(folded.replace("\r\n ", "").trim_end(), long, "unfolding gives the text back");
        assert_eq!(fold("short"), "short\r\n");
    }

    #[test]
    fn an_episode_is_a_timed_event_of_one_hour() {
        let folded = vevent(&episode(), 1_789_999_000, "https://finstats.example", true);
        assert!(folded.starts_with("BEGIN:VEVENT\r\n") && folded.ends_with("END:VEVENT\r\n"));
        let v = folded.replace("\r\n ", "");   // unfolded, to read the long lines whole
        assert!(v.contains("UID:1-episode-4242-air@finstats\r\n"));
        assert!(v.contains("DTSTAMP:20260921T"));
        assert!(v.contains("DTSTART:20260921T141320Z\r\n"), "{v}");
        assert!(v.contains("DURATION:PT1H\r\n"));
        assert!(v.contains("SUMMARY:Low Orbit S03E10 · Re-entry\r\n"));
        assert!(v.contains("CATEGORIES:Episode\r\n"));
        assert!(v.contains("DESCRIPTION:Season finale\\nYou watch this\\nhttps://finstats.example/items/abc\r\n"), "{v}");
        assert!(v.contains("URL:https://finstats.example/items/abc\r\n"));
    }

    #[test]
    fn a_film_is_an_all_day_event_ending_the_next_day() {
        let v = vevent(&film(), 1_789_999_000, "", false);
        assert!(v.contains("DTSTART;VALUE=DATE:20261003\r\n"));
        assert!(v.contains("DTEND;VALUE=DATE:20261004\r\n"));
        assert!(!v.contains("DURATION"));
        assert!(v.contains("SUMMARY:Paper Moons\\, Part 2\\; the Return (2026) · In cinemas\r\n"), "{v}");
        assert!(v.contains("CATEGORIES:Film\r\n"));
        assert!(!v.contains("URL:"), "no address of finstats, no link");
        assert!(!v.contains("DESCRIPTION:"), "nothing to say");
    }

    #[test]
    fn the_uid_is_the_upcoming_rows_key() {
        let mut e = film();
        e.release = "digital".into();
        assert!(vevent(&e, 0, "", false).contains("UID:2-movie-77-digital@finstats\r\n"));
    }

    #[test]
    fn a_flag_in_a_hand_typed_address_is_read_generously() {
        // Somebody subscribing a phone types `mine=1` as readily as `mine=true`; a bool would 400 the first.
        for yes in ["1", "true", "TRUE", "yes", "on"] { assert!(truthy(Some(yes)), "{yes}"); }
        for no in ["0", "false", "no", "off", ""] { assert!(!truthy(Some(no)), "{no}"); }
        assert!(!truthy(None));
    }

    #[test]
    fn a_calendar_wraps_its_events_with_crlf() {
        let c = calendar(&[vevent(&film(), 0, "", false)]);
        assert!(c.starts_with("BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//finstats//EN\r\n"));
        assert!(c.contains("METHOD:PUBLISH\r\n") && c.contains("X-WR-CALNAME:finstats: coming up\r\n"));
        assert!(c.ends_with("END:VCALENDAR\r\n"));
        assert!(!c.replace("\r\n", "").contains('\n'), "no bare newline anywhere");
    }
}
