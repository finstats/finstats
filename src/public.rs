//! Public profiles (2.0): the only thing a reader without an account can see.
//!
//! Every other read resolves through `AuthUser` and a `Scope` that *subtracts* what the caller may
//! not see. This one runs the other way: a `Published` names one person and the sections they chose,
//! and `answer` builds each of those sections and nothing else — no `Perms`, no `Scope`, no `AuthUser`.
//! The answer is typed, never `row_json`, so a column later added to a shared SELECT cannot reach a
//! stranger. Devices, clients, play methods, addresses, places, file paths, other people and the
//! Jellyfin login name are never produced, whatever the flags say.

use std::collections::HashSet;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use serde_json::Value;

use crate::db::rusqlite::{Connection, OptionalExtension, params, params_from_iter};
use crate::state::Settings;
use crate::stats::{self, Cond};

/// What a person chose to show. All off until they turn one on.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Sections {
    /// Watch time, plays, and top shows, films and music.
    pub totals: bool,
    /// Streaks, the weekday × hour grid, genres.
    pub habits: bool,
    /// The headline of the ready year's recap.
    pub recap: bool,
    /// Finished plays, a day old at the least.
    pub recent: bool,
}

/// One published profile, as a signed-out reader reaches it.
#[derive(Clone, Debug)]
pub struct Published {
    pub user_id: String,
    pub display_name: String,
    pub show_avatar: bool,
    pub sections: Sections,
}

#[derive(Clone, Debug, Serialize)]
pub struct PublicProfile {
    pub name: String,
    pub avatar: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub totals: Option<Totals>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub habits: Option<Habits>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recap: Option<Recap>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recent: Option<Vec<RecentPlay>>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Title {
    pub name: String,
    pub sub: Option<String>,
    pub plays: i64,
    pub watch_s: i64,
    /// The item whose poster may be shown; `None` when the library no longer has it.
    pub image: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Totals {
    pub plays: i64,
    pub watch_s: i64,
    pub movies: i64,
    pub episodes: i64,
    pub tracks: i64,
    pub top_series: Vec<Title>,
    pub top_movies: Vec<Title>,
    pub top_tracks: Vec<Title>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Genre {
    pub name: String,
    pub watch_s: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct Habits {
    pub longest_streak_days: i64,
    pub active_days: i64,
    /// Watch seconds, `[weekday Monday = 0][hour]`, over all time.
    pub heatmap: Vec<Vec<i64>>,
    pub genres: Vec<Genre>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Persona {
    pub title: String,
    pub line: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct Recap {
    pub year: i64,
    pub plays: i64,
    pub watch_s: i64,
    pub active_days: i64,
    pub persona: Option<Persona>,
    pub top_series: Option<Title>,
    pub top_movie: Option<Title>,
    pub top_genre: Option<String>,
    pub longest_streak_days: Option<i64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct RecentPlay {
    /// The local day it ended on; never the time of day.
    pub day: String,
    pub name: String,
    pub sub: Option<String>,
    pub image: Option<String>,
}

/// A token to the profile it opens, or `None` for every reason there is none — the server switch is
/// off, the token is unknown, the profile is not published, its owner was removed or disabled or may no
/// longer sign in. The caller answers all of them with the same 404.
pub fn lookup(c: &Connection, settings: &Settings, token: &str) -> Result<Option<Published>> {
    if !settings.public_profiles || !looks_like_token(token) {
        return Ok(None);
    }
    let row = c
        .query_row(
            "SELECT p.user_id, p.display_name, p.show_avatar, p.sections, u.is_admin
             FROM public_profiles p JOIN users u ON u.id = p.user_id
             WHERE p.token = ?1 AND p.published = 1 AND u.removed = 0 AND u.is_disabled = 0",
            [token],
            |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, i64>(2)?, r.get::<_, String>(3)?, r.get::<_, i64>(4)?)),
        )
        .optional()?;
    let Some((user_id, display_name, show_avatar, sections, is_admin)) = row else { return Ok(None) };
    // The same live rule as a session or a key: somebody who may not sign in publishes nothing either.
    let is_admin = is_admin != 0;
    let grants = if is_admin { vec![] } else { crate::auth::stored_grants(c, &user_id)? };
    if crate::auth::effective(is_admin, &grants, settings).is_none() {
        return Ok(None);
    }
    Ok(Some(Published { user_id, display_name, show_avatar: show_avatar != 0, sections: serde_json::from_str(&sections).unwrap_or_default() }))
}

fn looks_like_token(t: &str) -> bool {
    t.len() == TOKEN_LEN && t.bytes().all(|b| b.is_ascii_alphanumeric())
}

const TOKEN_LEN: usize = 22; // base62: about 131 bits

fn new_token() -> String {
    use rand::Rng;
    rand::rng().sample_iter(rand::distr::Alphanumeric).take(TOKEN_LEN).map(char::from).collect()
}

/// Everything the owner published, built from their id alone.
pub fn answer(c: &Connection, p: &Published, min_play_s: i64, now: i64) -> Result<PublicProfile> {
    let mut cond = Cond::default();
    cond.add("p.user_id = ?", p.user_id.clone());
    if min_play_s > 0 {
        cond.add("p.duration_s >= ?", min_play_s);
    }
    let s = p.sections;
    Ok(PublicProfile {
        // Never the Jellyfin user name: that is half of a sign-in.
        name: p.display_name.trim().to_string(),
        avatar: p.show_avatar,
        totals: if s.totals { Some(totals(c, &cond)?) } else { None },
        habits: if s.habits { Some(habits(c, &p.user_id, &cond, min_play_s)?) } else { None },
        recap: if s.recap { recap(c, &p.user_id, min_play_s)? } else { None },
        recent: if s.recent { Some(recent(c, &cond, now)?) } else { None },
    })
}

fn int(v: &Value, k: &str) -> i64 {
    v.get(k).and_then(Value::as_i64).unwrap_or(0)
}

fn text(v: &Value, k: &str) -> Option<String> {
    v.get(k).and_then(Value::as_str).map(str::to_string)
}

fn title(v: &Value) -> Title {
    Title {
        name: text(v, "name").unwrap_or_default(),
        sub: text(v, "sub"),
        plays: int(v, "plays"),
        watch_s: int(v, "watch_s"),
        image: text(v, "image_item_id"),
    }
}

fn titles(c: &Connection, cond: &Cond, kind: &str) -> Result<Vec<Title>> {
    Ok(stats::top(c, cond, kind, 5, false)?.iter().map(title).collect())
}

fn totals(c: &Connection, cond: &Cond) -> Result<Totals> {
    let t = stats::totals(c, cond)?;
    let (movies, episodes, tracks) = c.query_row(
        &format!(
            "SELECT COALESCE(SUM(p.item_type = 'Movie'), 0), COALESCE(SUM(p.item_type = 'Episode'), 0), COALESCE(SUM(p.item_type = 'Audio'), 0)
             FROM playbacks p {}",
            cond.sql()
        ),
        params_from_iter(cond.args.iter()),
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )?;
    Ok(Totals {
        plays: int(&t, "plays"),
        watch_s: int(&t, "watch_s"),
        movies,
        episodes,
        tracks,
        top_series: titles(c, cond, "series")?,
        top_movies: titles(c, cond, "movies")?,
        top_tracks: titles(c, cond, "music")?,
    })
}

fn habits(c: &Connection, user_id: &str, cond: &Cond, min_play_s: i64) -> Result<Habits> {
    // Longest run and days active only: the current streak says whether somebody watched today.
    let streaks = crate::profile::streaks(c, user_id, min_play_s)?;
    let heat = stats::heatmap(c, cond)?;
    let heatmap = serde_json::from_value(heat["watch_s"].clone()).unwrap_or_default();
    let genres = stats::genre_buckets(c, cond)?
        .iter()
        .filter(|g| g["name"] != "Other")
        .take(8)
        .map(|g| Genre { name: text(g, "name").unwrap_or_default(), watch_s: int(g, "watch_s") })
        .collect();
    Ok(Habits { longest_streak_days: int(&streaks["longest"], "days"), active_days: int(&streaks, "active_days"), heatmap, genres })
}

/// The ready year's headline, read off the recap and nothing more: not its rank among the other people
/// on the server, not the apps, not the records with their times of day.
fn recap(c: &Connection, user_id: &str, min_play_s: i64) -> Result<Option<Recap>> {
    let r = crate::recap::build(c, Some(user_id.to_string()), min_play_s, "", "")?;
    if r["empty"] == true {
        return Ok(None);
    }
    let first = |k: &str| r[k].as_array().and_then(|a| a.first()).map(title);
    let persona = r["persona"].as_object().map(|p| Persona {
        title: p.get("title").and_then(Value::as_str).unwrap_or_default().to_string(),
        line: p.get("line").and_then(Value::as_str).unwrap_or_default().to_string(),
    });
    Ok(Some(Recap {
        year: r["year"].as_i64().unwrap_or_default(),
        plays: int(&r["totals"], "plays"),
        watch_s: int(&r["totals"], "watch_s"),
        active_days: int(&r["totals"], "active_days"),
        persona,
        top_series: first("top_series"),
        top_movie: first("top_movies"),
        top_genre: r["top_genres"].as_array().and_then(|a| a.first()).and_then(|g| text(g, "name")),
        longest_streak_days: r["records"]["longest_streak"]["days"].as_i64(),
    }))
}

/// Before this long has passed since a play ended, a stranger is not told about it: "watching now" or
/// "an hour ago" tells them when somebody is at home.
const RECENT_DELAY_S: i64 = 86_400;

fn recent(c: &Connection, cond: &Cond, now: i64) -> Result<Vec<RecentPlay>> {
    let cond = cond.with_raw("p.active = 0").with("p.ended_at <= ?", now - RECENT_DELAY_S);
    let sql = format!(
        "SELECT date(p.ended_at, 'unixepoch', 'localtime'),
                CASE WHEN p.item_type = 'Episode' THEN COALESCE(s.name, p.series_name, p.item_name) ELSE COALESCE(i.name, p.item_name) END,
                CASE WHEN p.item_type = 'Episode' THEN
                       CASE WHEN p.season_number IS NOT NULL AND p.episode_number IS NOT NULL
                            THEN 'S' || p.season_number || 'E' || p.episode_number || ' · ' || p.item_name ELSE p.item_name END
                     WHEN p.item_type = 'Audio' THEN i.album_artist
                     ELSE CAST(i.production_year AS TEXT) END,
                CASE WHEN p.item_type = 'Episode' THEN CASE WHEN s.removed = 0 THEN s.id END
                     ELSE CASE WHEN i.removed = 0 THEN i.id END END
         FROM playbacks p LEFT JOIN items i ON i.id = p.item_id LEFT JOIN items s ON s.id = p.series_id
         {} ORDER BY p.ended_at DESC LIMIT 10",
        cond.sql()
    );
    let mut stmt = c.prepare(&sql)?;
    let rows = stmt.query_map(params_from_iter(cond.args.iter()), |r| {
        Ok(RecentPlay { day: r.get(0)?, name: r.get(1)?, sub: r.get(2)?, image: r.get(3)? })
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// The posters the page shows: the only item ids the public image route may serve for this profile.
pub fn listed_images(a: &PublicProfile) -> HashSet<String> {
    let mut out = HashSet::new();
    let mut add = |t: &Title| out.extend(t.image.clone());
    if let Some(t) = &a.totals {
        t.top_series.iter().chain(&t.top_movies).chain(&t.top_tracks).for_each(&mut add);
    }
    if let Some(r) = &a.recap {
        r.top_series.iter().chain(&r.top_movie).for_each(&mut add);
    }
    if let Some(recent) = &a.recent {
        out.extend(recent.iter().filter_map(|r| r.image.clone()));
    }
    out
}

/// The owner's side: what they set, and the token their link carries.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct Edit {
    pub published: bool,
    pub display_name: String,
    pub show_avatar: bool,
    pub sections: Sections,
}

/// Store what the owner set; the first save mints the token. Answers the token.
pub fn save(c: &Connection, user_id: &str, e: &Edit, now: i64) -> Result<String> {
    let name: String = e.display_name.trim().chars().take(NAME_MAX).collect();
    c.execute(
        "INSERT INTO public_profiles(user_id, token, published, display_name, show_avatar, sections, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)
         ON CONFLICT(user_id) DO UPDATE SET published = ?3, display_name = ?4, show_avatar = ?5, sections = ?6, updated_at = ?7",
        params![user_id, new_token(), e.published, name, e.show_avatar, serde_json::to_string(&e.sections)?, now],
    )?;
    Ok(c.query_row("SELECT token FROM public_profiles WHERE user_id = ?1", [user_id], |r| r.get(0))?)
}

pub const NAME_MAX: usize = 60;

/// A new token; every link already shared stops working.
pub fn reset(c: &Connection, user_id: &str, now: i64) -> Result<Option<String>> {
    let token = new_token();
    let n = c.execute("UPDATE public_profiles SET token = ?2, updated_at = ?3 WHERE user_id = ?1", params![user_id, token, now])?;
    Ok((n > 0).then_some(token))
}

#[cfg(test)]
mod tests {
    use super::*;

    const DAY: i64 = 86_400;
    const NOW: i64 = 1_790_000_000;

    fn conn() -> Connection {
        let c = Connection::open_in_memory().unwrap();
        for m in crate::db::MIGRATIONS {
            c.execute_batch(m).unwrap();
        }
        // alice publishes; bob watches the same film with her and must never be named.
        c.execute_batch(&format!(
            "INSERT INTO users(id, name, is_admin, updated_at) VALUES ('u-alice-0001', 'alice.login', 0, 1), ('u-bob-0002', 'bob', 0, 1);
             INSERT INTO user_permissions(user_id, permissions, updated_at) VALUES ('u-alice-0001', '[\"sign_in\"]', 1);
             INSERT INTO items(id, type, name, production_year, path, genres, updated_at) VALUES
               ('m1', 'Movie', 'Big Buck Bunny', 2008, '/media/films/Big Buck Bunny (2008).mkv', '[\"Animation\"]', 1),
               ('s1', 'Series', 'Sintel Stories', 2010, '/media/tv/Sintel Stories', '[\"Drama\"]', 1),
               ('e1', 'Episode', 'Pilot', NULL, '/media/tv/Sintel Stories/S01E01.mkv', '[]', 1);
             INSERT INTO playbacks(source, user_id, user_name, item_id, item_name, item_type, series_id, series_name, season_number, episode_number,
                                   started_at, ended_at, duration_s, client, device_name, device_id, remote_ip, play_method, active, group_id) VALUES
               ('live', 'u-alice-0001', 'alice.login', 'm1', 'Big Buck Bunny', 'Movie', NULL, NULL, NULL, NULL, {a}, {a} + 600, 600, 'Jellyfin Web', 'Living Room TV', 'dev-secret-1', '192.168.1.10', 'DirectPlay', 0, 1),
               ('live', 'u-bob-0002', 'bob', 'm1', 'Big Buck Bunny', 'Movie', NULL, NULL, NULL, NULL, {a}, {a} + 600, 600, 'Infuse', 'Bob Phone', 'dev-bob', '203.0.113.9', 'Transcode', 0, 1),
               ('live', 'u-alice-0001', 'alice.login', 'e1', 'Pilot', 'Episode', 's1', 'Sintel Stories', 1, 1, {b}, {b} + 1200, 1200, 'Jellyfin Web', 'Living Room TV', 'dev-secret-1', '192.168.1.10', 'DirectPlay', 0, NULL),
               ('live', 'u-alice-0001', 'alice.login', 'm1', 'Big Buck Bunny', 'Movie', NULL, NULL, NULL, NULL, {c}, {c} + 300, 300, 'Jellyfin Web', 'Living Room TV', 'dev-secret-1', '192.168.1.10', 'DirectPlay', 0, NULL),
               ('live', 'u-alice-0001', 'alice.login', 'e1', 'Pilot', 'Episode', 's1', 'Sintel Stories', 1, 1, {d}, {d} + 60, 60, 'Jellyfin Web', 'Living Room TV', 'dev-secret-1', '192.168.1.10', 'DirectPlay', 1, NULL);",
            a = NOW - 10 * DAY,
            b = NOW - 3 * DAY,
            c = NOW - 3600 * 2, // ended an hour and a half ago: too recent for a stranger
            d = NOW - 60,       // playing now
        ))
        .unwrap();
        c
    }

    fn on() -> Settings {
        Settings { public_profiles: true, ..Settings::default() }
    }

    fn all() -> Edit {
        Edit { published: true, display_name: "Alice".into(), show_avatar: true, sections: Sections { totals: true, habits: true, recap: true, recent: true } }
    }

    fn published(c: &Connection, e: &Edit) -> Published {
        let token = save(c, "u-alice-0001", e, NOW).unwrap();
        lookup(c, &on(), &token).unwrap().expect("a published profile is found")
    }

    #[test]
    fn a_profile_nobody_published_is_not_found() {
        let c = conn();
        assert!(lookup(&c, &on(), "no-such-token-at-all00").unwrap().is_none(), "unknown token");

        let token = save(&c, "u-alice-0001", &Edit { published: false, ..all() }, NOW).unwrap();
        assert!(lookup(&c, &on(), &token).unwrap().is_none(), "saved but not published");

        save(&c, "u-alice-0001", &all(), NOW).unwrap();
        assert!(lookup(&c, &on(), &token).unwrap().is_some(), "the same token once published");
        assert!(lookup(&c, &Settings::default(), &token).unwrap().is_none(), "the server switch is off by default");

        c.execute("DELETE FROM user_permissions WHERE user_id = 'u-alice-0001'", []).unwrap();
        assert!(lookup(&c, &on(), &token).unwrap().is_none(), "may no longer sign in");
        assert!(lookup(&c, &Settings { allow_user_login: true, ..on() }, &token).unwrap().is_some(), "everyone may sign in");

        c.execute("UPDATE users SET is_disabled = 1 WHERE id = 'u-alice-0001'", []).unwrap();
        assert!(lookup(&c, &Settings { allow_user_login: true, ..on() }, &token).unwrap().is_none(), "disabled in Jellyfin");
        c.execute("UPDATE users SET is_disabled = 0, removed = 1 WHERE id = 'u-alice-0001'", []).unwrap();
        assert!(lookup(&c, &Settings { allow_user_login: true, ..on() }, &token).unwrap().is_none(), "removed from Jellyfin");
    }

    #[test]
    fn an_unpublished_section_is_absent_not_empty() {
        let c = conn();
        let p = published(&c, &Edit { sections: Sections { habits: true, ..Sections::default() }, ..all() });
        let v = serde_json::to_value(answer(&c, &p, 0, NOW).unwrap()).unwrap();
        let keys: Vec<&str> = v.as_object().unwrap().keys().map(String::as_str).collect();
        assert_eq!(keys, ["avatar", "habits", "name"]);
    }

    #[test]
    fn nothing_but_what_was_published_reaches_the_answer() {
        let c = conn();
        let p = published(&c, &all());
        let a = answer(&c, &p, 0, NOW).unwrap();
        let text = serde_json::to_string(&a).unwrap();
        assert!(text.contains("Big Buck Bunny") && text.contains("Sintel Stories"), "the titles are there: {text}");
        for needle in ["alice.login", "bob", "Bob", "Living Room TV", "dev-secret-1", "Jellyfin Web", "Infuse", "192.168.1.10", "203.0.113", "DirectPlay", "Transcode", "/media/", "u-alice-0001", "u-bob-0002"] {
            assert!(!text.contains(needle), "{needle:?} reached a stranger: {text}");
        }
        assert_eq!(a.name, "Alice", "the name shown is the one the owner chose");
    }

    #[test]
    fn the_name_is_never_the_login_name_even_when_left_empty() {
        let c = conn();
        let p = published(&c, &Edit { display_name: "  ".into(), ..all() });
        assert_eq!(answer(&c, &p, 0, NOW).unwrap().name, "");
    }

    #[test]
    fn recent_plays_are_a_day_old_and_finished() {
        let c = conn();
        let p = published(&c, &all());
        let recent = answer(&c, &p, 0, NOW).unwrap().recent.unwrap();
        assert_eq!(recent.len(), 2, "the running play and the one from this evening are left out: {recent:?}");
        assert_eq!(recent[0].name, "Sintel Stories");
        assert_eq!(recent[0].sub.as_deref(), Some("S1E1 · Pilot"));
        assert_eq!(recent[1].name, "Big Buck Bunny");
        assert!(recent.iter().all(|r| r.day.len() == 10), "a day, never a time: {recent:?}");
    }

    #[test]
    fn the_recap_section_has_no_rank_and_no_clients() {
        let c = conn();
        let p = published(&c, &Edit { sections: Sections { recap: true, ..Sections::default() }, ..all() });
        let v = serde_json::to_value(answer(&c, &p, 0, NOW).unwrap()).unwrap();
        let recap = v["recap"].as_object().expect("a recap");
        for k in ["rank", "clients", "people", "records", "scope"] {
            assert!(!recap.contains_key(k), "{k} is not published");
        }
    }

    #[test]
    fn the_image_route_serves_only_what_the_page_lists() {
        let c = conn();
        let only_habits = published(&c, &Edit { sections: Sections { habits: true, ..Sections::default() }, ..all() });
        assert!(listed_images(&answer(&c, &only_habits, 0, NOW).unwrap()).is_empty(), "no titles, no posters");

        let p = published(&c, &all());
        let listed = listed_images(&answer(&c, &p, 0, NOW).unwrap());
        assert!(listed.contains("m1") && listed.contains("s1"), "{listed:?}");
        assert!(!listed.contains("e1"), "an episode is shown by its series' poster");
    }

    #[test]
    fn a_reset_link_leaves_the_old_one_not_found() {
        let c = conn();
        let old = save(&c, "u-alice-0001", &all(), NOW).unwrap();
        assert_eq!(save(&c, "u-alice-0001", &all(), NOW + 5).unwrap(), old, "saving again keeps the link");
        let new = reset(&c, "u-alice-0001", NOW + 10).unwrap().expect("a profile to reset");
        assert_ne!(new, old);
        assert!(lookup(&c, &on(), &old).unwrap().is_none());
        assert!(lookup(&c, &on(), &new).unwrap().is_some());
        assert!(reset(&c, "u-bob-0002", NOW).unwrap().is_none(), "nothing to reset for somebody who never published");
    }

    #[test]
    fn a_token_is_long_and_unguessable() {
        let c = conn();
        let t = save(&c, "u-alice-0001", &all(), NOW).unwrap();
        assert!(t.len() >= 22 && t.chars().all(|ch| ch.is_ascii_alphanumeric()), "{t}");
    }
}
