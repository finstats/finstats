//! Public profiles (2.0): the only thing a reader without an account can see.
//!
//! Every other read resolves through `AuthUser` and a `Scope` that *subtracts* what the caller may
//! not see. This one runs the other way: a `Published` names one person and the sections they chose,
//! and `answer` builds each of those sections and nothing else: no `Perms`, no `Scope`, no `AuthUser`.
//! The answer is typed, never `row_json`, so a column later added to a shared SELECT cannot reach a
//! stranger. Devices, clients, play methods, addresses, places, file paths, other people and the
//! Jellyfin login name are never produced, whatever the flags say.

use std::collections::HashSet;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use serde_json::Value;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use serde_json::json;

use crate::audit::{self, Actor};
use crate::auth::{AuthUser, Credential, JellyfinAdmin};
use crate::db::{self, rusqlite::{Connection, OptionalExtension, params, params_from_iter}};
use crate::state::{ApiError, ApiResult, App, Settings};
use crate::stats::{self, Cond};
use crate::story::StoryYear;

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
    pub recap: Option<StoryYear>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recent: Option<Vec<RecentPlay>>,
    /// The published year's cards, in order, by chapter key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub story: Option<Vec<&'static str>>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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

/// `[weekday Monday = 0][hour]`, over all time: the shape `charts.js`'s `heatmap` draws.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Heat {
    pub plays: Vec<Vec<i64>>,
    pub watch_s: Vec<Vec<i64>>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Habits {
    pub longest_streak_days: i64,
    pub active_days: i64,
    pub heatmap: Heat,
    pub genres: Vec<Genre>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Persona {
    pub title: String,
    pub line: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct RecentPlay {
    /// The local day it ended on; never the time of day.
    pub day: String,
    pub name: String,
    pub sub: Option<String>,
    pub image: Option<String>,
}

/// A token to the profile it opens, or `None` for every reason there is none: the server switch is
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

/// Everything the owner published, built from their id alone, and only from plays that ended a day
/// ago or more: a figure that moved the moment somebody pressed play would tell a stranger polling the
/// page that they are at home watching now.
pub fn answer(c: &Connection, p: &Published, min_play_s: i64, now: i64) -> Result<PublicProfile> {
    let until = now - DELAY_S;
    let mut cond = Cond::default();
    cond.add("p.user_id = ?", p.user_id.clone());
    if min_play_s > 0 {
        cond.add("p.duration_s >= ?", min_play_s);
    }
    cond.raw("p.active = 0");
    cond.add("p.ended_at <= ?", until);
    let s = p.sections;
    let recap = if s.recap { recap(c, p, min_play_s, until)? } else { None };
    Ok(PublicProfile {
        // Never the Jellyfin user name: that is half of a sign-in.
        name: p.display_name.trim().to_string(),
        avatar: p.show_avatar,
        totals: if s.totals { Some(totals(c, &cond)?) } else { None },
        habits: if s.habits { Some(habits(c, &cond)?) } else { None },
        story: recap.as_ref().map(|y| y.chapters().into_iter().map(|ch| ch.key()).collect()),
        recap,
        recent: if s.recent { Some(recent(c, &cond)?) } else { None },
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
             FROM visible_playbacks p {}",
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

fn habits(c: &Connection, cond: &Cond) -> Result<Habits> {
    // The longest run and the days active; never the current streak, which is about today.
    let mut stmt = c.prepare(&format!("SELECT DISTINCT date(p.started_at, 'unixepoch', 'localtime') FROM visible_playbacks p {}", cond.sql()))?;
    let days: std::collections::BTreeSet<chrono::NaiveDate> = stmt
        .query_map(params_from_iter(cond.args.iter()), |r| r.get::<_, String>(0))?
        .filter_map(|d| d.ok().and_then(|d| chrono::NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok()))
        .collect();
    let longest = crate::recap::longest_run(&days).map_or(0, |(len, _, _)| len);
    let heatmap = serde_json::from_value(stats::heatmap(c, cond)?).unwrap_or_default();
    let genres = stats::genre_buckets(c, cond)?
        .iter()
        .filter(|g| g["name"] != "Other")
        .take(8)
        .map(|g| Genre { name: text(g, "name").unwrap_or_default(), watch_s: int(g, "watch_s") })
        .collect();
    Ok(Habits { longest_streak_days: longest, active_days: days.len() as i64, heatmap, genres })
}

/// The ready year, as the whole story (2.0): the same `StoryYear` the app's cards are drawn from, so it
/// carries no companion, no rank among other people and no app, and it wears the name the owner chose.
fn recap(c: &Connection, p: &Published, min_play_s: i64, until: i64) -> Result<Option<StoryYear>> {
    let r = crate::recap::build(c, Some(p.user_id.clone()), min_play_s, "", "", Some(until))?;
    Ok(StoryYear::from_recap(&r, &p.display_name))
}

/// Before this long has passed since a play ended, a stranger is not told about it: "watching now" or
/// "an hour ago" tells them when somebody is at home.
const DELAY_S: i64 = 86_400;

fn recent(c: &Connection, cond: &Cond) -> Result<Vec<RecentPlay>> {
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
         FROM visible_playbacks p LEFT JOIN items i ON i.id = p.item_id LEFT JOIN items s ON s.id = p.series_id
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
        for ch in r.chapters() {
            out.extend(r.poster_ids(ch));
        }
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

/// What a save did, for the audit log: a link going live and a link going dark are the two that matter.
fn save_kind(was: Option<bool>, now: bool) -> &'static str {
    match (was.unwrap_or(false), now) {
        (false, true) => "profile_published",
        (true, false) => "profile_unpublished",
        _ => "profile_changed",
    }
}

pub const NAME_MAX: usize = 60;

/// A new token; every link already shared stops working.
pub fn reset(c: &Connection, user_id: &str, now: i64) -> Result<Option<String>> {
    let token = new_token();
    let n = c.execute("UPDATE public_profiles SET token = ?2, updated_at = ?3 WHERE user_id = ?1", params![user_id, token, now])?;
    Ok((n > 0).then_some(token))
}

// ---------------------------------------------------------------- the owner's side

/// Making something public is done with a session, never a key: a key left in a script should not be
/// able to publish its owner's viewing.
fn only_a_session(user: &AuthUser) -> Result<(), ApiError> {
    match user.credential {
        Credential::Session => Ok(()),
        Credential::Key { .. } => Err(ApiError::new(StatusCode::FORBIDDEN, "Sign in to publish a profile; a key cannot make anything public")),
    }
}

/// The address of a profile: absolute when FinStats knows where it answers from outside.
fn link(settings: &Settings, token: &str) -> String {
    format!("{}/u/{token}", settings.public_url.trim_end_matches('/'))
}

fn mine(c: &Connection, settings: &Settings, user_id: &str) -> Result<Value> {
    let row = c
        .query_row(
            "SELECT token, published, display_name, show_avatar, sections FROM public_profiles WHERE user_id = ?1",
            [user_id],
            |r| Ok((r.get::<_, String>(0)?, r.get::<_, bool>(1)?, r.get::<_, String>(2)?, r.get::<_, bool>(3)?, r.get::<_, String>(4)?)),
        )
        .optional()?;
    let (url, published, display_name, show_avatar, sections) = match row {
        Some((token, p, n, a, s)) => (Some(link(settings, &token)), p, n, a, serde_json::from_str(&s).unwrap_or_default()),
        None => (None, false, String::new(), false, Sections::default()),
    };
    Ok(json!({
        "server_enabled": settings.public_profiles, "published": published, "url": url,
        "display_name": display_name, "show_avatar": show_avatar, "sections": sections,
    }))
}

/// `GET /api/me/public-profile`
pub async fn get_mine(State(app): State<App>, user: AuthUser) -> ApiResult {
    let settings = app.settings();
    Ok(Json(app.db.call(move |c| mine(c, &settings, &user.id)).await?))
}

/// `PUT /api/me/public-profile`: what to publish, and whether to. The first save mints the link.
pub async fn put_mine(State(app): State<App>, user: AuthUser, Json(e): Json<Edit>) -> ApiResult {
    only_a_session(&user)?;
    let settings = app.settings();
    if !settings.public_profiles {
        return Err(ApiError::new(StatusCode::CONFLICT, "An administrator has not allowed public profiles on this server"));
    }
    let actor = Actor::from(&user);
    let out = app
        .db
        .call(move |c| {
            let was: Option<bool> = c.query_row("SELECT published FROM public_profiles WHERE user_id = ?1", [&user.id], |r| r.get(0)).optional()?;
            save(c, &user.id, &e, db::now())?;
            audit::record_quietly(c, &audit::Entry::new(save_kind(was, e.published), actor).target(user.id.clone()).detail(json!({ "sections": e.sections, "show_avatar": e.show_avatar })));
            mine(c, &settings, &user.id)
        })
        .await?;
    Ok(Json(out))
}

/// `POST /api/me/public-profile/reset`: a new link; the old one is gone for good.
pub async fn reset_mine(State(app): State<App>, user: AuthUser) -> ApiResult {
    only_a_session(&user)?;
    let settings = app.settings();
    let actor = Actor::from(&user);
    let out = app
        .db
        .call(move |c| {
            if reset(c, &user.id, db::now())?.is_none() {
                return Ok(None);
            }
            audit::record_quietly(c, &audit::Entry::new("profile_link_reset", actor).target(user.id.clone()));
            Ok(Some(mine(c, &settings, &user.id)?))
        })
        .await?
        .ok_or_else(|| ApiError::not_found("Profile"))?;
    Ok(Json(out))
}

/// `GET /api/public-profiles`: who publishes what, for the administrators who answer for the server.
pub async fn list(State(app): State<App>, JellyfinAdmin(_): JellyfinAdmin) -> ApiResult {
    let rows = app
        .db
        .call(|c| {
            let mut stmt = c.prepare(
                "SELECT p.user_id, COALESCE(u.name, ''), p.display_name, p.published, p.sections, p.created_at, p.updated_at
                 FROM public_profiles p LEFT JOIN users u ON u.id = p.user_id ORDER BY p.published DESC, p.updated_at DESC",
            )?;
            let rows = stmt.query_map([], |r| {
                Ok(json!({
                    "user_id": r.get::<_, String>(0)?, "user_name": r.get::<_, String>(1)?, "display_name": r.get::<_, String>(2)?,
                    "published": r.get::<_, bool>(3)?, "sections": serde_json::from_str::<Sections>(&r.get::<_, String>(4)?).unwrap_or_default(),
                    "created_at": r.get::<_, i64>(5)?, "updated_at": r.get::<_, i64>(6)?,
                }))
            })?;
            Ok(rows.collect::<Result<Vec<_>, _>>()?)
        })
        .await?;
    Ok(Json(json!({ "enabled": app.settings().public_profiles, "profiles": rows })))
}

/// `DELETE /api/public-profiles/{user_id}`: an administrator takes a profile down. The owner's choices
/// are kept; publishing again is theirs to do.
pub async fn take_down(State(app): State<App>, JellyfinAdmin(user): JellyfinAdmin, Path(id): Path<String>) -> ApiResult {
    let id = db::norm_id(&id);
    let actor = Actor::from(&user);
    let done = app
        .db
        .call(move |c| {
            let n = c.execute("UPDATE public_profiles SET published = 0, updated_at = ?2 WHERE user_id = ?1 AND published = 1", params![id, db::now()])?;
            if n > 0 {
                audit::record_quietly(c, &audit::Entry::new("profile_unpublished", actor).target(id));
            }
            Ok(n > 0)
        })
        .await?;
    if !done {
        return Err(ApiError::not_found("Profile"));
    }
    Ok(Json(json!({ "ok": true })))
}

// ---------------------------------------------------------------- the stranger's side

/// Every way of there being nothing to show answers exactly this, so a link cannot tell "switched off"
/// from "taken down" from "never existed".
pub fn not_found() -> ApiError {
    ApiError::not_found("Profile")
}

/// A token to its profile and the answer, on the database thread.
pub async fn resolve(app: &App, token: String) -> Result<(Published, PublicProfile), ApiError> {
    let settings = app.settings();
    let min_play_s = settings.min_play_s;
    app.db
        .call(move |c| {
            let Some(p) = lookup(c, &settings, &token)? else { return Ok(None) };
            let a = answer(c, &p, min_play_s, db::now())?;
            Ok(Some((p, a)))
        })
        .await?
        .ok_or_else(not_found)
}

/// `GET /api/public/{token}`, with no session and no key: the link is the only thing asked for.
pub async fn read(State(app): State<App>, Path(token): Path<String>) -> ApiResult<Response> {
    let (_, a) = resolve(&app, token).await?;
    Ok(noindex(Json(a).into_response()))
}

/// The page's preview: what a chat app shows under a pasted link. Every value is escaped for HTML.
pub fn fill_page(page: &str, a: &PublicProfile, base: &str, token: &str) -> String {
    let url = format!("{base}/u/{token}");
    let title = if a.name.is_empty() { "A FinStats profile".to_string() } else { format!("{} on FinStats", a.name) };
    let description = match &a.totals {
        Some(t) => format!("{} hours watched, {} plays.", t.watch_s / 3600, t.plays),
        None => "What they watch, published by them.".to_string(),
    };
    page.replace("{{title}}", &html_escape(&title))
        .replace("{{description}}", &html_escape(&description))
        .replace("{{image}}", &html_escape(&format!("{url}/card.png")))
        .replace("{{url}}", &html_escape(&url))
}

/// For text and attribute values alike.
pub fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            // Characters XML does not allow at all: one in a title would make a card's SVG unreadable.
            '\t' | '\n' | '\r' => out.push(ch),
            c if c < ' ' || c == '\u{FFFE}' || c == '\u{FFFF}' => {}
            c => out.push(c),
        }
    }
    out
}

/// Published pages are for the people a link is sent to, not for search engines.
pub fn noindex(mut r: Response) -> Response {
    r.headers_mut().insert("x-robots-tag", HeaderValue::from_static("noindex, nofollow"));
    r
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
    fn the_weekday_grid_has_the_shape_the_apps_chart_draws() {
        let c = conn();
        let p = published(&c, &Edit { sections: Sections { habits: true, ..Sections::default() }, ..all() });
        let v = serde_json::to_value(answer(&c, &p, 0, NOW).unwrap()).unwrap();
        let heat = &v["habits"]["heatmap"];
        for k in ["plays", "watch_s"] {
            let grid = heat[k].as_array().unwrap_or_else(|| panic!("{k}: {heat}"));
            assert_eq!(grid.len(), 7);
            assert!(grid.iter().all(|r| r.as_array().unwrap().len() == 24));
        }
        let sum = |k: &str| heat[k].as_array().unwrap().iter().flat_map(|r| r.as_array().unwrap().iter().map(|x| x.as_i64().unwrap())).sum::<i64>();
        assert_eq!((sum("plays"), sum("watch_s")), (2, 600 + 1200), "the plays of a day ago and before");
    }

    #[test]
    fn nothing_a_stranger_sees_moves_while_somebody_is_watching() {
        // Anything that counts a play the moment it happens tells a stranger polling the page that
        // somebody is at home watching right now: the totals, the grid, the streak and the year as
        // much as the recent list. Every section counts only what ended a day ago or more.
        let c = conn();
        let p = published(&c, &all());
        let before = serde_json::to_string(&answer(&c, &p, 0, NOW).unwrap()).unwrap();
        c.execute_batch(&format!(
            "INSERT INTO playbacks(source, user_id, user_name, item_id, item_name, item_type, started_at, ended_at, duration_s, active) VALUES
               ('live', 'u-alice-0001', 'alice.login', 'm1', 'Big Buck Bunny', 'Movie', {n} - 7200, {n} - 1800, 5400, 0),
               ('live', 'u-alice-0001', 'alice.login', 's1', 'Pilot', 'Episode', {n} - 600, {n}, 600, 1);",
            n = NOW
        ))
        .unwrap();
        assert_eq!(serde_json::to_string(&answer(&c, &p, 0, NOW).unwrap()).unwrap(), before);
        // A day later the evening is history and may be told.
        assert_ne!(serde_json::to_string(&answer(&c, &p, 0, NOW + DAY).unwrap()).unwrap(), before);
    }

    #[test]
    fn the_recap_section_has_no_rank_and_no_clients() {
        let c = conn();
        let p = published(&c, &Edit { sections: Sections { recap: true, ..Sections::default() }, ..all() });
        let v = serde_json::to_value(answer(&c, &p, 0, NOW).unwrap()).unwrap();
        let recap = v["recap"].as_object().expect("a recap");
        // The published year is the whole story (2.0), records included; never the rank among other
        // people, the apps, the cast or whose login it is.
        for k in ["rank", "clients", "people", "scope"] {
            assert!(!recap.contains_key(k), "{k} is not published");
        }
    }

    #[test]
    fn the_published_year_is_the_whole_story_and_names_nobody() {
        let c = conn();
        let p = published(&c, &Edit { sections: Sections { recap: true, ..Sections::default() }, ..all() });
        let a = answer(&c, &p, 0, NOW).unwrap();
        let story = a.recap.as_ref().expect("a published year");
        assert_eq!(story.whose, "Alice", "the name the owner chose");
        let chapters = story.chapters();
        assert!(chapters.contains(&crate::story::Chapter::Year) && chapters.contains(&crate::story::Chapter::Together), "{chapters:?}");
        let text = serde_json::to_string(&a).unwrap();
        assert!(!text.contains("bob") && !text.contains("u-bob-0002"), "the companion is not named: {text}");
        let v: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(v["story"][0], "year", "the page is told which cards there are, not left to work it out: {}", v["story"]);
        assert_eq!(v["story"].as_array().unwrap().len(), chapters.len());
        let listed = listed_images(&a);
        for ch in chapters {
            for id in story.poster_ids(ch) {
                assert!(listed.contains(&id), "{id}, drawn on the {ch:?} card, cannot be fetched");
            }
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
    fn a_save_is_on_record_as_what_it_did_to_the_link() {
        assert_eq!(save_kind(None, true), "profile_published");
        assert_eq!(save_kind(Some(false), true), "profile_published");
        assert_eq!(save_kind(Some(true), true), "profile_changed");
        assert_eq!(save_kind(Some(true), false), "profile_unpublished");
        assert_eq!(save_kind(None, false), "profile_changed");
        for k in ["profile_published", "profile_changed", "profile_unpublished", "profile_link_reset"] {
            assert!(crate::audit::KINDS.contains(&k), "{k} is not an audit kind");
        }
    }

    #[test]
    fn the_page_preview_is_the_chosen_name_escaped_and_a_card() {
        let c = conn();
        let p = published(&c, &Edit { display_name: "<b>Al & \"Ice\"</b>".into(), ..all() });
        let a = answer(&c, &p, 0, NOW).unwrap();
        let page = "<title>{{title}}</title><meta property=\"og:description\" content=\"{{description}}\"><meta property=\"og:image\" content=\"{{image}}\"><meta property=\"og:url\" content=\"{{url}}\">";
        let html = fill_page(page, &a, "https://stats.example.com", "Tok3n");
        assert!(html.contains("&lt;b&gt;Al &amp; &quot;Ice&quot;&lt;/b&gt;"), "{html}");
        assert!(!html.contains("<b>"), "{html}");
        assert!(html.contains("content=\"https://stats.example.com/u/Tok3n/card.png\""), "{html}");
        assert!(html.contains("content=\"https://stats.example.com/u/Tok3n\""), "{html}");
        assert!(!html.contains("{{"), "every hole filled: {html}");
        // Without a name and without an address: a plain title, and a card on this server's own path.
        let p = published(&c, &Edit { display_name: String::new(), ..all() });
        let html = fill_page(page, &answer(&c, &p, 0, NOW).unwrap(), "", "Tok3n");
        assert!(html.contains("<title>A FinStats profile</title>") && html.contains("content=\"/u/Tok3n/card.png\""), "{html}");
    }

    #[test]
    fn a_token_is_long_and_unguessable() {
        let c = conn();
        let t = save(&c, "u-alice-0001", &all(), NOW).unwrap();
        assert!(t.len() >= 22 && t.chars().all(|ch| ch.is_ascii_alphanumeric()), "{t}");
    }
}
