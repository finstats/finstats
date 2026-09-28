//! The year in review: one endpoint that tells one person the story of their year. Your own, or,
//! for a Jellyfin administrator, any one user's.

use std::collections::BTreeSet;

use anyhow::Result;
use axum::Json;
use axum::extract::{Query, State};
use chrono::{Datelike, NaiveDate};
use serde::Deserialize;
use serde_json::{Map, Value, json};

use crate::auth::AuthUser;
use crate::db::rusqlite::{Connection, params_from_iter};
use crate::db::{self, SqlValue};
use crate::state::{ApiError, ApiResult, App};
use crate::stats::{one_json, rows_json};

#[derive(Deserialize)]
pub struct RecapQuery {
    year: Option<String>,
    /// Honoured for Jellyfin administrators only.
    user_id: Option<String>,
    /// `server`: the whole server's year, for Jellyfin administrators only.
    scope: Option<String>,
}

/// `WHERE` over `playbacks p` for the period and scope, plus its arguments.
struct Window {
    wh: String,
    args: Vec<SqlValue>,
    /// Same scope without the period: "first ever" questions need the full history.
    scope_wh: String,
    scope_args: Vec<SqlValue>,
    from: i64,
    to: i64,
}

impl Window {
    fn with(&self, extra: &str) -> String {
        format!("{} AND {extra}", self.wh)
    }
}

const TITLE_ID: &str = "COALESCE(p.series_id, p.item_id)";
const NOT_LIVE_TV: &str = "p.item_type NOT IN ('TvChannel', 'LiveTvChannel', 'Program', 'LiveTvProgram')";

/// Whose year a request is about: `(scope_user, server)`. A recap is one person's year. Everyone gets
/// their own; only a Jellyfin administrator may open someone else's, or the whole server's (2.0), and
/// no permission widens that. An unknown scope is refused rather than guessed.
fn whose_year(user: &AuthUser, q: &RecapQuery) -> Result<(Option<String>, bool), ApiError> {
    let server = match q.scope.as_deref() {
        None | Some("") | Some("user") => false,
        Some("server") if user.is_admin => true,
        Some("server") => return Err(ApiError::forbidden()),
        Some(_) => return Err(ApiError::bad_request("The scope is `user` or `server`")),
    };
    let requested_user = q.user_id.as_deref().map(db::norm_id).filter(|s| !s.is_empty());
    let scope_user = (!server).then(|| match requested_user {
        Some(other) if user.is_admin => other,
        _ => user.id.clone(),
    });
    Ok((scope_user, server))
}

/// The year a request asks for, whole server's already stripped of anybody in it.
async fn year_for(app: &App, user: &AuthUser, q: &RecapQuery) -> Result<(Value, bool, String), ApiError> {
    let (scope_user, server) = whose_year(user, q)?;
    let min_play_s = app.settings().min_play_s;
    let server_name = app.config.read().unwrap().as_ref().map(|c| c.server_name.clone()).unwrap_or_else(|| "Jellyfin".into());
    let requested = q.year.clone().unwrap_or_default();
    let name = server_name.clone();
    let out = app.db.call(move |c| build(c, scope_user, min_play_s, &name, &requested, None)).await?;
    let mut out = if server { server_edition(out) } else { out };
    // Which cards the year has: said here, so the page never works it out again and drifts.
    out["story"] = match crate::story::StoryYear::from_recap(&out, "") {
        Some(y) => json!(y.chapters().into_iter().map(|ch| ch.key()).collect::<Vec<_>>()),
        None => Value::Null,
    };
    Ok((out, server, server_name))
}

pub async fn recap(State(app): State<App>, user: AuthUser, Query(q): Query<RecapQuery>) -> ApiResult {
    let (out, _, _) = year_for(&app, &user, &q).await?;
    Ok(Json(out))
}

/// The year as a story (2.0). In the app a card names nobody, not even its owner: a login name is half a
/// sign-in, and the name a person wants on a card is the one they chose for their public profile. The
/// server's year says the server's name.
pub(crate) async fn story_for(app: &App, user: &AuthUser, q: &RecapQuery) -> Result<crate::story::StoryYear, ApiError> {
    let (out, server, server_name) = year_for(app, user, q).await?;
    crate::story::StoryYear::from_recap(&out, if server { &server_name } else { "" }).ok_or_else(|| ApiError::not_found("Year"))
}

/// `until`: count only plays that had ended by then — for a published recap, which must not move while
/// somebody is watching (`public::answer`).
pub(crate) fn build(c: &Connection, scope_user: Option<String>, min_play_s: i64, server_name: &str, requested: &str, until: Option<i64>) -> Result<Value> {
    // Live TV is left out of the recap altogether: a channel left on all evening says nothing about taste.
    let mut scope_wh = format!("WHERE {NOT_LIVE_TV}");
    let mut scope_args: Vec<SqlValue> = vec![];
    if let Some(u) = &scope_user {
        scope_wh.push_str(" AND p.user_id = ?");
        scope_args.push(u.clone().into());
    }
    if min_play_s > 0 {
        scope_wh.push_str(" AND p.duration_s >= ?");
        scope_args.push(min_play_s.into());
    }
    if let Some(t) = until {
        scope_wh.push_str(" AND p.active = 0 AND p.ended_at <= ?");
        scope_args.push(t.into());
    }

    let years: Vec<i64> = rows_json(
        c,
        &format!("SELECT DISTINCT CAST(strftime('%Y', p.started_at, 'unixepoch', 'localtime') AS INTEGER) AS y FROM playbacks p {scope_wh} ORDER BY y DESC"),
        &scope_args,
    )?
    .iter()
    .filter_map(|r| r["y"].as_i64())
    .collect();

    // Period boundaries are local midnights, expressed in UTC seconds.
    let local_midnight = |expr: &str| -> Result<i64> { Ok(c.query_row(&format!("SELECT CAST(strftime('%s', {expr}, 'utc') AS INTEGER)"), [], |r| r.get(0))?) };
    let (year_json, from, to) = if requested == "last12" {
        (json!("last12"), local_midnight("date('now', 'localtime', 'start of month', '-12 months')")?, db::now() + 1)
    } else {
        let (this_year, month): (i64, i64) = c.query_row(
            "SELECT CAST(strftime('%Y', 'now', 'localtime') AS INTEGER), CAST(strftime('%m', 'now', 'localtime') AS INTEGER)",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        let y = requested.parse::<i64>().ok().filter(|y| (1990..=9999).contains(y)).unwrap_or_else(|| default_year(this_year, month, &years));
        (json!(y), local_midnight(&format!("'{y:04}-01-01'"))?, local_midnight(&format!("'{:04}-01-01'", y + 1))?)
    };

    let mut args = scope_args.clone();
    args.extend([from.into(), to.into()]);
    let w = Window { wh: format!("{scope_wh} AND p.started_at >= ? AND p.started_at < ?"), args, scope_wh, scope_args, from, to };

    let user_name: Option<String> = match &scope_user {
        Some(u) => c
            .query_row("SELECT COALESCE((SELECT name FROM users WHERE id = ?1), (SELECT user_name FROM playbacks WHERE user_id = ?1 ORDER BY ended_at DESC LIMIT 1))", [u], |r| r.get(0))
            .unwrap_or(None),
        None => None,
    };

    let totals = totals_in(c, &w.wh, &w.args)?;
    let empty = totals.get("plays").and_then(Value::as_i64).unwrap_or(0) == 0;

    let mut out = json!({
        "years": years, "year": year_json, "from": from, "to": to,
        "scope": { "user_id": scope_user, "user_name": user_name, "server_name": server_name },
        "empty": empty, "totals": totals, "rank": null,
        "top_series": [], "top_movies": [], "top_tracks": [], "top_genres": [], "genres": null,
        "people": { "actors": [], "directors": [] }, "rewatch": null, "days": [],
        "months": [], "hours": [], "weekdays": [], "persona": null, "records": {},
        "discovery": { "new_series": 0, "one_and_done": [], "finished_movies": 0, "finished_episodes": 0 },
        "clients": [],
    });
    if empty {
        return Ok(out);
    }

    out["rank"] = rank(c, &w, scope_user.as_deref(), min_play_s)?;
    out["top_series"] = json!(top_titles(c, &w, "Episode")?);
    out["top_movies"] = json!(top_titles(c, &w, "Movie")?);
    out["top_tracks"] = json!(top_titles(c, &w, "Audio")?);
    out["top_genres"] = json!(rows_json(
        c,
        &format!(
            "SELECT g.value AS name, COUNT(*) AS plays, COALESCE(SUM(p.duration_s), 0) AS watch_s
             FROM playbacks p JOIN items gi ON gi.id = {TITLE_ID}, json_each(gi.genres) g {} GROUP BY 1 ORDER BY watch_s DESC LIMIT 6",
            w.wh
        ),
        &w.args,
    )?);
    out["genres"] = json!(one_json(
        c,
        &format!(
            "SELECT (SELECT COUNT(DISTINCT g.value) FROM playbacks p JOIN items gi ON gi.id = {TITLE_ID}, json_each(gi.genres) g {wh}) AS count,
                    COUNT(*) AS plays, COALESCE(SUM(p.duration_s), 0) AS watch_s
             FROM playbacks p JOIN items gi ON gi.id = {TITLE_ID} {wh} AND gi.genres IS NOT NULL",
            wh = w.wh
        ),
        &[w.args.clone(), w.args.clone()].concat(),
    )?);
    out["people"] = json!({ "actors": people(c, &w, "Actor")?, "directors": people(c, &w, "Director")? });
    out["rewatch"] = rewatch(c, &w)?;
    out["days"] = json!(rows_json(
        c,
        &format!(
            "SELECT date(p.started_at, 'unixepoch', 'localtime') AS date, COUNT(*) AS plays, COALESCE(SUM(p.duration_s), 0) AS watch_s
             FROM playbacks p {} GROUP BY 1 ORDER BY 1",
            w.wh
        ),
        &w.args,
    )?);
    out["months"] = json!(months(c, &w)?);

    let (hours, weekdays) = rhythm(c, &w)?;
    out["persona"] = persona(&out["totals"], &hours, &weekdays);
    if let Some(t) = out["totals"].as_object_mut() {
        t.remove("user_days");
    }
    out["hours"] = json!(hours);
    out["weekdays"] = json!(weekdays);

    out["records"] = records(c, &w)?;
    out["discovery"] = discovery(c, &w)?;
    out["clients"] = json!(rows_json(
        c,
        &format!("SELECT p.client AS name, COUNT(*) AS plays, COALESCE(SUM(p.duration_s), 0) AS watch_s FROM playbacks p {} GROUP BY 1 ORDER BY watch_s DESC LIMIT 3", w.with("p.client IS NOT NULL")),
        &w.args,
    )?);
    let watch_s = out["totals"]["watch_s"].as_i64().unwrap_or(0);
    out["together"] = together(c, &w, scope_user.as_deref(), until, watch_s)?;
    out["requests"] = requests(c, &w, scope_user.as_deref(), until)?;
    out["finished"] = match &scope_user {
        Some(u) => finished(c, &w, u, until.unwrap_or_else(db::now))?,
        None => Value::Null,
    };
    out["versus"] = match year_json.as_i64() {
        Some(y) => versus(c, &w, y, &local_midnight(&format!("'{:04}-01-01'", y - 1))?)?,
        None => Value::Null,
    };
    Ok(out)
}

/// Who this person watched with, from the evenings the group fold found: how many, how long in
/// company (for each evening the shorter of their stay and the longest other one), the title that
/// brought people together most, and at most three companions — named here, in the app, and nowhere
/// that leaves it. The whole server's year counts evenings and people and names nobody.
fn together(c: &Connection, w: &Window, user: Option<&str>, until: Option<i64>, watch_s: i64) -> Result<Value> {
    let end = until.map_or(w.to, |u| u.min(w.to));
    let window = crate::groups::Window { since: Some(w.from), until: Some(end) };
    let people: Vec<String> = user.map(|u| vec![u.to_string()]).unwrap_or_default();
    let sessions = crate::groups::sessions_for(c, &window, None, &people)?;
    let mine: Vec<&crate::groups::Session> = sessions.iter().filter(|s| user.is_none_or(|u| s.per_user.contains_key(u))).collect();
    if mine.is_empty() {
        return Ok(Value::Null);
    }
    let mut together_s = 0;
    let mut titles: std::collections::HashMap<&str, (&str, i64, i64)> = Default::default();
    let mut companions: std::collections::BTreeMap<&str, (&crate::groups::Member, i64, i64)> = Default::default();
    for s in &mine {
        let t = match user {
            Some(u) => {
                let me = s.per_user[u].duration_s;
                let others = s.per_user.iter().filter(|(id, _)| id.as_str() != u);
                let longest_other = others.clone().map(|(_, m)| m.duration_s).max().unwrap_or(0);
                for (id, m) in others {
                    let e = companions.entry(id.as_str()).or_insert((m, 0, 0));
                    e.1 += 1;
                    e.2 += me.min(m.duration_s);
                }
                me.min(longest_other)
            }
            None => s.together_s(),
        };
        together_s += t;
        let e = titles.entry(s.title_id.as_str()).or_insert((s.title_name.as_str(), 0, 0));
        e.1 += 1;
        e.2 += t;
    }
    let top_title = titles
        .iter()
        .max_by_key(|(id, (_, n, secs))| (*n, *secs, std::cmp::Reverse(**id)))
        .map(|(id, (name, n, _))| json!({ "id": id, "name": name, "image_item_id": id, "evenings": n }));
    let share = if watch_s > 0 { together_s as f64 / watch_s as f64 } else { 0.0 };
    let mut out = json!({ "evenings": mine.len(), "together_s": together_s, "share": share, "top_title": top_title });
    match user {
        Some(_) => {
            let mut list: Vec<_> = companions.into_iter().collect();
            list.sort_by_key(|(id, (_, n, secs))| (std::cmp::Reverse(*n), std::cmp::Reverse(*secs), *id));
            out["companions"] = json!(list.into_iter().take(3).map(|(id, (m, n, secs))| json!({
                "user_id": id, "user_name": m.name, "has_image": m.has_image, "evenings": n, "together_s": secs,
            })).collect::<Vec<_>>());
        }
        None => {
            let people: std::collections::BTreeSet<&str> = mine.iter().flat_map(|s| s.per_user.keys().map(String::as_str)).collect();
            out["people_in_company"] = json!(people.len());
        }
    }
    Ok(out)
}

/// What this person asked for through Seerr in the window, how much of it arrived, and how much of what
/// arrived they then watched (a play by them after it became available — watching it before does not count
/// as the request's doing). Nothing at all when no request was ever recorded: Seerr is not connected, or
/// nobody uses it. The whole server's year counts everybody's and names nobody.
fn requests(c: &Connection, w: &Window, user: Option<&str>, until: Option<i64>) -> Result<Value> {
    let any: bool = c.query_row("SELECT EXISTS (SELECT 1 FROM requests)", [], |r| r.get(0))?;
    if !any {
        return Ok(Value::Null);
    }
    let end = until.map_or(w.to, |u| u.min(w.to));
    let mut wh = "r.removed_at IS NULL AND r.requested_at >= ?1 AND r.requested_at < ?2".to_string();
    let mut args: Vec<SqlValue> = vec![w.from.into(), end.into()];
    if let Some(u) = user {
        wh.push_str(" AND r.user_id = ?3");
        args.push(u.to_string().into());
    }
    let watched_sql = "r.available_at IS NOT NULL AND r.item_id IS NOT NULL AND EXISTS (
        SELECT 1 FROM playbacks p WHERE p.user_id = r.user_id AND (p.item_id = r.item_id OR p.series_id = r.item_id) AND p.started_at >= r.available_at AND p.ended_at <= ?2)";
    let (made, available, watched): (i64, i64, i64) = c.query_row(
        &format!("SELECT COUNT(*), COALESCE(SUM(r.available_at IS NOT NULL AND r.available_at < ?2), 0), COALESCE(SUM({watched_sql}), 0) FROM requests r WHERE {wh}"),
        params_from_iter(args.iter()),
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )?;
    if made == 0 {
        return Ok(Value::Null);
    }
    let top = rows_json(
        c,
        &format!(
            "SELECT r.title, r.year, r.item_id, r.item_id AS image_item_id FROM requests r
             WHERE {wh} AND {watched_sql} ORDER BY r.available_at DESC LIMIT 5"
        ),
        &args,
    )?;
    Ok(json!({ "made": made, "available": available, "watched": watched, "top": top }))
}

/// In December the year is ready (`default_year` flips then): tell each person who watched in it, once.
pub fn announce_ready(c: &Connection, bus: &crate::notify::Fanout, today: NaiveDate) -> Result<usize> {
    if today.month() != 12 {
        return Ok(0);
    }
    let year = today.year();
    let from: i64 = c.query_row("SELECT CAST(strftime('%s', ?1, 'utc') AS INTEGER)", [format!("{year:04}-01-01")], |r| r.get(0))?;
    let to: i64 = c.query_row("SELECT CAST(strftime('%s', ?1, 'utc') AS INTEGER)", [format!("{:04}-01-01", year + 1)], |r| r.get(0))?;
    let people: Vec<(String, String)> = c
        .prepare(&format!(
            "SELECT p.user_id, COALESCE(u.name, MAX(p.user_name)) FROM playbacks p LEFT JOIN users u ON u.id = p.user_id
             WHERE p.started_at >= ?1 AND p.started_at < ?2 AND {NOT_LIVE_TV} GROUP BY p.user_id ORDER BY p.user_id"
        ))?
        .query_map([from, to], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<Result<_, _>>()?;
    let mut told = 0;
    for (user_id, name) in people {
        let event = crate::notify::Event::new(
            crate::notify::Kind::RecapReady,
            format!("notify:recap:{year}:{user_id}"),
            format!("Your {year} in review is ready"),
            format!("Hours, top titles, the shows finished and the rest of {name}'s {year}, ready to look back on — and to share."),
        )
        .field("Year", year.to_string())
        .link(format!("/recap?year={year}"))
        .about(user_id, name);
        told += usize::from(crate::notify::raise_in(c, bus, &event)?);
    }
    Ok(told)
}

/// The whole server's year, for Jellyfin administrators: every title and total, and nobody in it.
pub(crate) fn server_edition(mut v: Value) -> Value {
    if let Some(o) = v.as_object_mut() {
        o.remove("rank");
        o.remove("clients");
        if let Some(t) = o.get_mut("together").and_then(Value::as_object_mut) {
            t.remove("companions");
        }
    }
    v["scope"]["kind"] = json!("server");
    v["scope"]["user_id"] = Value::Null;
    v["scope"]["user_name"] = Value::Null;
    v
}

/// A show counts as left behind when it was begun in the window, less than half of it is seen, and
/// nothing of it was played in this long before the window closed (or before now, in a year still
/// going): long enough that a show between seasons, or saved for the holidays, is not called dropped.
const DROPPED_QUIET_S: i64 = 60 * 86_400;

/// The shows this person finished in the window — every episode on the server seen, the last of them
/// inside it — and the ones they began and left, by the same reading of "seen" as the profile's
/// progress bars (`profile::episodes`).
fn finished(c: &Connection, w: &Window, user: &str, now: i64) -> Result<Value> {
    let end = w.to.min(now);
    let within = |t: i64| t >= w.from && t < end;
    let eps = crate::profile::episodes(c, user)?;
    let day = |t: i64| -> Result<String> { Ok(c.query_row("SELECT date(?1, 'unixepoch', 'localtime')", [t], |r| r.get(0))?) };
    let (mut done, mut dropped) = (vec![], vec![]);
    for show in eps.chunk_by(|a, b| a.series_id == b.series_id) {
        let first = &show[0];
        let total = show.len() as i64;
        let seen = show.iter().filter(|e| e.state == "seen").count() as i64;
        let seen_last = show.iter().map(|e| e.seen_at).collect::<Option<Vec<i64>>>().and_then(|v| v.into_iter().max());
        if seen == total {
            if let Some(t) = seen_last.filter(|t| within(*t)) {
                done.push((t, json!({ "id": first.series_id, "name": first.series_name, "image_item_id": first.series_id, "episodes": total, "finished_on": day(t)? })));
            }
            continue;
        }
        let began = show.iter().filter_map(|e| e.first_at).min();
        let last = show.iter().filter_map(|e| e.last_at.max(e.seen_at)).max();
        if began.is_some_and(within) && seen * 2 < total && last.is_some_and(|l| l < end - DROPPED_QUIET_S) {
            dropped.push((began.unwrap_or(0), json!({ "id": first.series_id, "name": first.series_name, "image_item_id": first.series_id, "seen": seen, "total": total })));
        }
    }
    if done.is_empty() && dropped.is_empty() {
        return Ok(Value::Null);
    }
    done.sort_by_key(|(t, _)| std::cmp::Reverse(*t));
    dropped.sort_by_key(|(t, _)| *t);
    let (count, dropped_count) = (done.len(), dropped.len());
    Ok(json!({
        "series": done.into_iter().map(|(_, v)| v).take(10).collect::<Vec<_>>(), "count": count,
        "dropped": dropped.into_iter().map(|(_, v)| v).take(10).collect::<Vec<_>>(), "dropped_count": dropped_count,
    }))
}

/// The same headline numbers for the calendar year before, when there was one.
fn versus(c: &Connection, w: &Window, year: i64, from: &i64) -> Result<Value> {
    let mut args = w.scope_args.clone();
    args.extend([(*from).into(), w.from.into()]);
    let t = totals_in(c, &format!("{} AND p.started_at >= ? AND p.started_at < ?", w.scope_wh), &args)?;
    if t.get("plays").and_then(Value::as_i64).unwrap_or(0) == 0 {
        return Ok(Value::Null);
    }
    Ok(json!({ "year": year - 1, "plays": t["plays"], "watch_s": t["watch_s"], "active_days": t["active_days"] }))
}

/// A year's recap is "ready" in its December and stays the default until the next December.
/// Falls back to the newest year with plays when the ready year has none (a new install).
fn default_year(this_year: i64, month: i64, years_with_plays: &[i64]) -> i64 {
    let ready = if month == 12 { this_year } else { this_year - 1 };
    if years_with_plays.contains(&ready) { ready } else { years_with_plays.first().copied().unwrap_or(ready) }
}

/// A period's headline numbers over any WHERE clause on `playbacks p`: the year asked for, or the one
/// before it for the comparison.
fn totals_in(c: &Connection, wh: &str, args: &[SqlValue]) -> Result<Map<String, Value>> {
    Ok(one_json(
        c,
        &format!(
            "SELECT COUNT(*) AS plays, COALESCE(SUM(p.duration_s), 0) AS watch_s, COUNT(DISTINCT p.item_id) AS distinct_items,
                    COALESCE(SUM(p.item_type = 'Movie'), 0) AS movies, COALESCE(SUM(p.item_type = 'Episode'), 0) AS episodes,
                    COALESCE(SUM(p.item_type = 'Audio'), 0) AS tracks,
                    COALESCE(SUM(CASE WHEN p.item_type = 'Movie' THEN p.duration_s END), 0) AS movie_watch_s,
                    COALESCE(SUM(CASE WHEN p.item_type = 'Episode' THEN p.duration_s END), 0) AS episode_watch_s,
                    COALESCE(SUM(CASE WHEN p.item_type = 'Audio' THEN p.duration_s END), 0) AS track_watch_s,
                    COUNT(DISTINCT CASE WHEN p.item_type = 'Episode' THEN COALESCE(p.series_id, p.series_name) END) AS series_count,
                    COUNT(DISTINCT date(p.started_at, 'unixepoch', 'localtime')) AS active_days,
                    COUNT(DISTINCT date(p.started_at, 'unixepoch', 'localtime') || p.user_id) AS user_days
             FROM playbacks p {}",
            wh
        ),
        args,
    )?
    .unwrap_or_default())
}

fn rank(c: &Connection, w: &Window, user: Option<&str>, min_play_s: i64) -> Result<Value> {
    let Some(user) = user else { return Ok(Value::Null) };
    let rows = rows_json(
        c,
        &format!(
            "SELECT p.user_id, SUM(p.duration_s) AS watch_s FROM playbacks p
         WHERE p.started_at >= ?1 AND p.started_at < ?2 AND p.duration_s >= ?3 AND {NOT_LIVE_TV} GROUP BY p.user_id ORDER BY watch_s DESC"
        ),
        &[w.from.into(), w.to.into(), min_play_s.into()],
    )?;
    let total: i64 = rows.iter().filter_map(|r| r["watch_s"].as_i64()).sum();
    let Some(pos) = rows.iter().position(|r| r["user_id"].as_str() == Some(user)) else { return Ok(Value::Null) };
    let mine = rows[pos]["watch_s"].as_i64().unwrap_or(0);
    Ok(json!({ "position": pos + 1, "of": rows.len(), "share": if total > 0 { (mine as f64 / total as f64 * 1000.0).round() / 1000.0 } else { 0.0 } }))
}

fn top_titles(c: &Connection, w: &Window, item_type: &str) -> Result<Vec<Map<String, Value>>> {
    let is_series = item_type == "Episode";
    let (id, name, sub, episodes) = if is_series {
        ("p.series_id", "COALESCE(i.name, MAX(p.series_name), 'Unknown series')", "CAST(i.production_year AS TEXT)", "COUNT(DISTINCT p.item_id)")
    } else if item_type == "Audio" {
        ("p.item_id", "COALESCE(i.name, MAX(p.item_name))", "COALESCE(i.album_artist, i.album, MAX(p.series_name))", "NULL")
    } else {
        ("p.item_id", "COALESCE(i.name, MAX(p.item_name))", "CAST(i.production_year AS TEXT)", "NULL")
    };
    let group = if is_series { "COALESCE(p.series_id, p.series_name)" } else { "p.item_id" };
    let mut args = w.args.clone();
    args.push(item_type.to_string().into());
    rows_json(
        c,
        &format!(
            "SELECT {id} AS id, {name} AS name, {sub} AS sub, {id} AS image_item_id, COUNT(*) AS plays,
                    COALESCE(SUM(p.duration_s), 0) AS watch_s, {episodes} AS episodes, (i.id IS NOT NULL AND i.removed = 0) AS item_exists
             FROM playbacks p LEFT JOIN items i ON i.id = {id} {} AND p.item_type = ? GROUP BY {group} ORDER BY watch_s DESC LIMIT 5",
            w.wh
        ),
        &args,
    )
}

/// The people seen (or, for directors, watched) the most: watch time across every title they are in.
fn people(c: &Connection, w: &Window, kind: &str) -> Result<Vec<Map<String, Value>>> {
    let mut args = w.args.clone();
    args.push(kind.to_string().into());
    rows_json(
        c,
        &format!(
            "SELECT ip.person_id AS id, MAX(ip.name) AS name, MAX(ip.has_image) AS has_image, COUNT(*) AS plays,
                    COALESCE(SUM(p.duration_s), 0) AS watch_s, COUNT(DISTINCT ip.item_id) AS titles,
                    (SELECT COALESCE(i.name, 'Unknown title') FROM items i WHERE i.id = (
                        SELECT ip2.item_id FROM playbacks p JOIN item_people ip2 ON ip2.item_id = {TITLE_ID} {wh}
                           AND ip2.person_id = ip.person_id AND ip2.kind = ip.kind GROUP BY ip2.item_id ORDER BY SUM(p.duration_s) DESC LIMIT 1)) AS top_title
             FROM playbacks p JOIN item_people ip ON ip.item_id = {TITLE_ID} {wh} AND ip.kind = ? GROUP BY ip.person_id ORDER BY watch_s DESC LIMIT 5",
            wh = w.wh
        ),
        &[w.args.clone(), args].concat(),
    )
}

/// A rewatch is a film or episode coming back on a later day. Picking a play up again the same day is not one.
fn rewatch(c: &Connection, w: &Window) -> Result<Value> {
    let (sittings, items): (i64, i64) = c.query_row(
        &format!(
            "SELECT COUNT(*), COUNT(DISTINCT item_id) FROM (
                SELECT p.item_id AS item_id FROM playbacks p {} GROUP BY p.item_id, date(p.started_at, 'unixepoch', 'localtime'))",
            w.with("p.item_type IN ('Movie', 'Episode') AND p.duration_s >= 300")
        ),
        params_from_iter(w.args.iter()),
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    if sittings == 0 {
        return Ok(Value::Null);
    }
    let again = sittings - items;
    Ok(json!({ "sittings": sittings, "rewatches": again, "share": (again as f64 / sittings as f64 * 1000.0).round() / 1000.0 }))
}

fn months(c: &Connection, w: &Window) -> Result<Vec<Value>> {
    let sums = rows_json(
        c,
        &format!(
            "SELECT strftime('%Y-%m', p.started_at, 'unixepoch', 'localtime') AS month, COUNT(*) AS plays, COALESCE(SUM(p.duration_s), 0) AS watch_s
             FROM playbacks p {} GROUP BY 1",
            w.wh
        ),
        &w.args,
    )?;
    // The title that owned each month: the most watched series or film.
    let tops = rows_json(
        c,
        &format!(
            "SELECT month, id, name, id AS image_item_id, watch_s FROM (
                SELECT strftime('%Y-%m', p.started_at, 'unixepoch', 'localtime') AS month, {TITLE_ID} AS id,
                       COALESCE(MAX(p.series_name), MAX(p.item_name)) AS name, SUM(p.duration_s) AS watch_s,
                       ROW_NUMBER() OVER (PARTITION BY strftime('%Y-%m', p.started_at, 'unixepoch', 'localtime') ORDER BY SUM(p.duration_s) DESC) AS rn
                FROM playbacks p {} GROUP BY 1, 2
             ) WHERE rn = 1",
            w.with("p.item_type IN ('Movie', 'Episode')")
        ),
        &w.args,
    )?;
    let first: String = c.query_row("SELECT date(?1, 'unixepoch', 'localtime', 'start of month')", [w.from], |r| r.get(0))?;
    let last: String = c.query_row("SELECT date(MIN(?1 - 1, CAST(strftime('%s', 'now') AS INTEGER)), 'unixepoch', 'localtime', 'start of month')", [w.to], |r| r.get(0))?;
    let (mut cursor, last) = (NaiveDate::parse_from_str(&first, "%Y-%m-%d")?, NaiveDate::parse_from_str(&last, "%Y-%m-%d")?);
    let mut out = vec![];
    while cursor <= last && out.len() < 36 {
        let key = cursor.format("%Y-%m").to_string();
        let sum = sums.iter().find(|r| r["month"].as_str() == Some(&key));
        let top = tops.iter().find(|r| r["month"].as_str() == Some(&key)).map(|t| json!({ "id": t["id"], "name": t["name"], "image_item_id": t["image_item_id"], "watch_s": t["watch_s"] }));
        out.push(json!({
            "month": key,
            "watch_s": sum.and_then(|s| s["watch_s"].as_i64()).unwrap_or(0),
            "plays": sum.and_then(|s| s["plays"].as_i64()).unwrap_or(0),
            "top": top,
        }));
        cursor = if cursor.month() == 12 { NaiveDate::from_ymd_opt(cursor.year() + 1, 1, 1) } else { NaiveDate::from_ymd_opt(cursor.year(), cursor.month() + 1, 1) }.unwrap_or(last + chrono::Duration::days(1));
    }
    Ok(out)
}

fn rhythm(c: &Connection, w: &Window) -> Result<(Vec<i64>, Vec<i64>)> {
    let (mut hours, mut weekdays) = (vec![0i64; 24], vec![0i64; 7]);
    let sql = format!(
        "SELECT CAST(strftime('%w', p.started_at, 'unixepoch', 'localtime') AS INTEGER), CAST(strftime('%H', p.started_at, 'unixepoch', 'localtime') AS INTEGER),
                COALESCE(SUM(p.duration_s), 0) FROM playbacks p {} GROUP BY 1, 2",
        w.wh
    );
    let mut stmt = c.prepare(&sql)?;
    let mut rows = stmt.query(params_from_iter(w.args.iter()))?;
    while let Some(r) = rows.next()? {
        let (dow, hour, secs): (i64, i64, i64) = (r.get(0)?, r.get(1)?, r.get(2)?);
        hours[hour.clamp(0, 23) as usize] += secs;
        weekdays[((dow + 6) % 7) as usize] += secs;
    }
    Ok((hours, weekdays))
}

/// A label for how this year was watched. First rule that clearly applies wins; the
/// order goes from the most distinctive habit to the most common one.
fn persona(totals: &Value, hours: &[i64], weekdays: &[i64]) -> Value {
    let total: i64 = hours.iter().sum();
    if total == 0 {
        return Value::Null;
    }
    let share = |secs: i64| secs as f64 / total as f64;
    let pct = |f: f64| (f * 100.0).round() as i64;
    let night = share(hours[22..].iter().sum::<i64>() + hours[..4].iter().sum::<i64>());
    let morning = share(hours[5..10].iter().sum());
    let weekend = share(weekdays[5] + weekdays[6]);
    let n = |k: &str| totals[k].as_i64().unwrap_or(0) as f64;
    let plays = n("plays").max(1.0);
    // Per person: on a shared server ten people watching one episode each is not a binge.
    let per_day = n("episodes") / n("user_days").max(n("active_days")).max(1.0);
    let make = |key: &str, title: &str, line: String| json!({ "key": key, "title": title, "line": line });

    if night >= 0.35 {
        make("night_owl", "Night owl", format!("{}% of the watching happened between 22:00 and 04:00", pct(night)))
    } else if morning >= 0.30 {
        make("early_bird", "Early bird", format!("{}% of the watching happened before 10 in the morning", pct(morning)))
    } else if n("tracks") / plays >= 0.5 {
        make("music_lover", "Music lover", format!("{}% of everything played was music", pct(n("tracks") / plays)))
    } else if weekend >= 0.45 {
        make("weekend_warrior", "Weekend warrior", format!("{}% of the watching happened on Saturdays and Sundays", pct(weekend)))
    } else if per_day >= 4.0 {
        make("binge_watcher", "Binge watcher", format!("{per_day:.1} episodes on an average day of watching"))
    } else if n("movies") / plays >= 0.4 {
        make("movie_buff", "Movie buff", format!("{}% of plays were films", pct(n("movies") / plays)))
    } else {
        let peak = hours.iter().enumerate().max_by_key(|(_, s)| **s).map(|(h, _)| h).unwrap_or(20);
        make("creature_of_habit", "Creature of habit", format!("Most of the watching started around {peak:02}:00, week in, week out"))
    }
}

fn records(c: &Connection, w: &Window) -> Result<Value> {
    let day = "date(p.started_at, 'unixepoch', 'localtime')";
    let biggest_day = one_json(
        c,
        &format!("SELECT {day} AS date, COALESCE(SUM(p.duration_s), 0) AS watch_s, COUNT(*) AS plays FROM playbacks p {} GROUP BY 1 ORDER BY watch_s DESC LIMIT 1", w.wh),
        &w.args,
    )?;
    let biggest_binge = one_json(
        c,
        &format!(
            "SELECT {day} AS date, p.series_id, MAX(p.series_name) AS series_name, p.series_id AS image_item_id,
                    COUNT(DISTINCT p.item_id) AS episodes, COALESCE(SUM(p.duration_s), 0) AS watch_s
             FROM playbacks p {} GROUP BY 1, COALESCE(p.series_id, p.series_name) HAVING episodes >= 3 ORDER BY episodes DESC, watch_s DESC LIMIT 1",
            w.with("p.item_type = 'Episode'")
        ),
        &w.args,
    )?;
    let longest_play = one_json(
        c,
        &format!(
            "SELECT p.item_id, CASE WHEN p.item_type = 'Episode' AND p.series_name IS NOT NULL THEN p.series_name || ' — ' || p.item_name ELSE p.item_name END AS name,
                    {TITLE_ID} AS image_item_id, p.duration_s, {day} AS date
             FROM playbacks p LEFT JOIN items i ON i.id = p.item_id {} ORDER BY p.duration_s DESC LIMIT 1",
            // A session left open overnight is not a long play: the time must fit the runtime.
            w.with("p.duration_s <= COALESCE(p.runtime_s, i.runtime_s, p.duration_s) * 1.25")
        ),
        &w.args,
    )?;
    let most_rewatched = one_json(
        c,
        &format!(
            "SELECT p.item_id AS id, CASE WHEN p.item_type = 'Episode' AND MAX(p.series_name) IS NOT NULL THEN MAX(p.series_name) || ' — ' || MAX(p.item_name) ELSE MAX(p.item_name) END AS name,
                    p.item_type AS type, {TITLE_ID} AS image_item_id, COUNT(DISTINCT {day}) AS plays
             FROM playbacks p {} GROUP BY p.item_id HAVING plays >= 2 ORDER BY plays DESC, SUM(p.duration_s) DESC LIMIT 1",
            w.with("p.item_type IN ('Movie', 'Episode') AND p.duration_s >= 300")
        ),
        &w.args,
    )?;
    let first_play = one_json(
        c,
        &format!(
            "SELECT p.item_id, CASE WHEN p.item_type = 'Episode' AND p.series_name IS NOT NULL THEN p.series_name || ' — ' || p.item_name ELSE p.item_name END AS name,
                    {TITLE_ID} AS image_item_id, p.started_at AS at FROM playbacks p {} ORDER BY p.started_at LIMIT 1",
            w.wh
        ),
        &w.args,
    )?;
    let oldest_title = one_json(
        c,
        &format!(
            "SELECT i.id, i.name, i.production_year AS year, i.id AS image_item_id
             FROM playbacks p JOIN items i ON i.id = {TITLE_ID} {} GROUP BY i.id ORDER BY i.production_year, SUM(p.duration_s) DESC LIMIT 1",
            w.with("i.production_year > 1800 AND p.duration_s >= 300")
        ),
        &w.args,
    )?;

    // Longest run of consecutive local days with at least one play.
    let days: BTreeSet<NaiveDate> = rows_json(c, &format!("SELECT DISTINCT {day} AS d FROM playbacks p {}", w.wh), &w.args)?
        .iter()
        .filter_map(|r| NaiveDate::parse_from_str(r["d"].as_str()?, "%Y-%m-%d").ok())
        .collect();
    let longest_streak = longest_run(&days).filter(|(len, _, _)| *len >= 2).map(|(len, from, to)| json!({ "days": len, "from": from.to_string(), "to": to.to_string() }));

    Ok(json!({
        "biggest_day": biggest_day, "biggest_binge": biggest_binge, "longest_streak": longest_streak,
        "longest_play": longest_play, "most_rewatched": most_rewatched, "first_play": first_play, "oldest_title": oldest_title,
    }))
}

/// The longest run of consecutive days in a set: (length, first day, last day).
pub fn longest_run(days: &BTreeSet<NaiveDate>) -> Option<(i64, NaiveDate, NaiveDate)> {
    let mut best: Option<(i64, NaiveDate, NaiveDate)> = None;
    let mut run: Option<(NaiveDate, NaiveDate)> = None;
    for d in days {
        run = match run {
            Some((start, prev)) if *d == prev + chrono::Duration::days(1) => Some((start, *d)),
            _ => Some((*d, *d)),
        };
        if let Some((start, end)) = run {
            let len = (end - start).num_days() + 1;
            if best.is_none_or(|(b, _, _)| len > b) {
                best = Some((len, start, end));
            }
        }
    }
    best
}

fn discovery(c: &Connection, w: &Window) -> Result<Value> {
    // "First ever" is judged against the whole history of this scope, not just the period.
    let mut args = w.scope_args.clone();
    args.extend([w.from.into(), w.to.into()]);
    let firsts = format!(
        "SELECT p.series_id AS id, MAX(p.series_name) AS name, MIN(p.started_at) AS first_at, COUNT(DISTINCT p.item_id) AS episodes
         FROM playbacks p {} AND p.item_type = 'Episode' AND p.series_id IS NOT NULL GROUP BY p.series_id",
        w.scope_wh
    );
    let new_series: i64 = c.query_row(&format!("SELECT COUNT(*) FROM ({firsts}) WHERE first_at >= ? AND first_at < ?"), params_from_iter(args.iter()), |r| r.get(0))?;
    let one_and_done = rows_json(
        c,
        &format!("SELECT id, name, id AS image_item_id FROM ({firsts}) WHERE first_at >= ? AND first_at < ? AND episodes = 1 ORDER BY first_at DESC LIMIT 5"),
        &args,
    )?;
    let finished = one_json(
        c,
        &format!(
            "SELECT COALESCE(SUM(f AND t = 'Movie'), 0) AS finished_movies, COALESCE(SUM(f AND t = 'Episode'), 0) AS finished_episodes FROM (
                SELECT p.item_type AS t, MAX(COALESCE(p.position_s, p.duration_s) * 1.0 / COALESCE(p.runtime_s, i.runtime_s)) >= 0.9 AS f
                FROM playbacks p LEFT JOIN items i ON i.id = p.item_id {} GROUP BY p.item_id, p.user_id)",
            w.with("p.item_type IN ('Movie', 'Episode') AND COALESCE(p.runtime_s, i.runtime_s, 0) > 0")
        ),
        &w.args,
    )?
    .unwrap_or_default();
    Ok(json!({
        "new_series": new_series, "one_and_done": one_and_done,
        "finished_movies": finished.get("finished_movies"), "finished_episodes": finished.get("finished_episodes"),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- the chapters 2.0 added: invented people and titles, on 2025 and 2024.

    /// Noon on a day, local time, as SQLite reads it: what the recap's local-midnight windows compare against.
    fn at(c: &Connection, day: &str) -> i64 {
        c.query_row("SELECT CAST(strftime('%s', ?1 || ' 12:00:00', 'utc') AS INTEGER)", [day], |r| r.get(0)).unwrap()
    }

    fn year_db() -> Connection {
        let c = Connection::open_in_memory().unwrap();
        for m in crate::db::MIGRATIONS {
            c.execute_batch(m).unwrap();
        }
        c.execute_batch(
            "INSERT INTO users(id, name, is_admin, updated_at) VALUES ('ua', 'alice', 0, 1), ('ub', 'bob', 0, 1), ('uc', 'carol', 0, 1);
             INSERT INTO items(id, type, name, production_year, runtime_s, updated_at) VALUES
               ('m1', 'Movie', 'Big Buck Bunny', 2008, 600, 1), ('m2', 'Movie', 'Sintel', 2010, 900, 1);",
        )
        .unwrap();
        c
    }

    fn play(c: &Connection, user: &str, item: &str, ty: &str, day: &str, secs: i64, group: Option<i64>) -> i64 {
        let start = at(c, day);
        c.execute(
            "INSERT INTO playbacks(source, user_id, user_name, item_id, item_name, item_type, started_at, ended_at, duration_s, position_s, group_id)
             VALUES ('live', ?1, ?1, ?2, COALESCE((SELECT name FROM items WHERE id = ?2), ?2), ?3, ?4, ?4 + ?5, ?5, ?5, ?6)",
            crate::db::rusqlite::params![user, item, ty, start, secs, group],
        )
        .unwrap();
        c.last_insert_rowid()
    }

    fn year_of(c: &Connection, user: Option<&str>, year: &str) -> Value {
        build(c, user.map(str::to_string), 0, "", year, None).unwrap()
    }

    #[test]
    fn together_counts_evenings_in_company_and_names_at_most_three() {
        let c = year_db();
        // Two evenings of Big Buck Bunny with bob, one of Sintel with carol, and one film alone.
        for (day, g) in [("2025-03-01", 1), ("2025-04-01", 3)] {
            let a = play(&c, "ua", "m1", "Movie", day, 600, Some(g));
            play(&c, "ub", "m1", "Movie", day, 500, Some(g));
            c.execute("UPDATE playbacks SET group_id = ?1 WHERE id = ?1", [a]).unwrap();
            c.execute("UPDATE playbacks SET group_id = ?1 WHERE group_id = ?2", [a, g]).unwrap();
        }
        let a = play(&c, "ua", "m2", "Movie", "2025-05-01", 900, None);
        play(&c, "uc", "m2", "Movie", "2025-05-01", 900, Some(a));
        c.execute("UPDATE playbacks SET group_id = ?1 WHERE id = ?1", [a]).unwrap();
        play(&c, "ua", "m2", "Movie", "2025-06-01", 900, None);

        let t = &year_of(&c, Some("ua"), "2025")["together"];
        assert_eq!(t["evenings"], 3, "{t}");
        assert_eq!(t["together_s"], 500 + 500 + 900, "each evening counts the time she was in company");
        let share = t["share"].as_f64().unwrap();
        assert!((share - 1900.0 / 3000.0).abs() < 1e-9, "{share}");
        assert_eq!(t["top_title"]["name"], "Big Buck Bunny");
        assert_eq!(t["top_title"]["evenings"], 2);
        let names: Vec<&str> = t["companions"].as_array().unwrap().iter().map(|p| p["user_name"].as_str().unwrap()).collect();
        assert_eq!(names, ["bob", "carol"], "most evenings first");
        // Nobody else in the year: no chapter.
        assert!(year_of(&c, Some("ua"), "2024")["together"].is_null());
    }

    fn show(c: &Connection, id: &str, name: &str, episodes: &[&str]) {
        c.execute("INSERT INTO items(id, type, name, updated_at) VALUES (?1, 'Series', ?2, 1)", [id, name]).unwrap();
        for (i, e) in episodes.iter().enumerate() {
            c.execute(
                "INSERT INTO items(id, type, name, series_id, parent_index_number, index_number, path, runtime_s, updated_at) VALUES (?1, 'Episode', ?1, ?2, 1, ?3, '/f/' || ?1, 1000, 1)",
                crate::db::rusqlite::params![e, id, i as i64 + 1],
            )
            .unwrap();
        }
    }

    fn episode(c: &Connection, user: &str, ep: &str, day: &str, secs: i64) {
        let series: String = c.query_row("SELECT series_id FROM items WHERE id = ?1", [ep], |r| r.get(0)).unwrap();
        let start = at(c, day);
        c.execute(
            "INSERT INTO playbacks(source, user_id, user_name, item_id, item_name, item_type, series_id, started_at, ended_at, duration_s, position_s, runtime_s)
             VALUES ('live', ?1, ?1, ?2, ?2, 'Episode', ?3, ?4, ?4 + ?5, ?5, ?5, 1000)",
            crate::db::rusqlite::params![user, ep, series, start, secs],
        )
        .unwrap();
    }

    #[test]
    fn a_show_is_finished_in_the_year_its_last_episode_was_seen_and_one_barely_started_is_dropped() {
        let c = year_db();
        show(&c, "s1", "Sintel Stories", &["e1", "e2"]);
        episode(&c, "ua", "e1", "2025-03-01", 950);
        episode(&c, "ua", "e2", "2025-11-02", 950);
        show(&c, "s2", "Old Show", &["e3", "e4"]);
        episode(&c, "ua", "e3", "2024-03-01", 950);
        episode(&c, "ua", "e4", "2024-04-01", 950);
        // Seen by a play and by Jellyfin's own flag: the flag's date is when it was seen.
        show(&c, "s5", "Flagged", &["g1", "g2"]);
        episode(&c, "ua", "g1", "2025-01-10", 950);
        c.execute("INSERT INTO user_items(user_id, item_id, played, last_played_at) VALUES ('ua', 'g2', 1, ?1)", [at(&c, "2025-06-10")]).unwrap();
        // One episode of four in February and nothing since: dropped.
        show(&c, "s3", "Dropped Tales", &["f1", "f2", "f3", "f4"]);
        episode(&c, "ua", "f1", "2025-02-01", 950);
        // Half of it, but watched in December: still going, neither finished nor dropped.
        show(&c, "s4", "Half Done", &["h1", "h2"]);
        episode(&c, "ua", "h1", "2025-12-15", 950);

        let f = &year_of(&c, Some("ua"), "2025")["finished"];
        let names = |k: &str| f[k].as_array().unwrap().iter().map(|s| s["name"].as_str().unwrap().to_string()).collect::<Vec<_>>();
        assert_eq!(names("series"), ["Sintel Stories", "Flagged"], "newest first: {f}");
        assert_eq!(f["series"][0]["finished_on"], "2025-11-02");
        assert_eq!(f["series"][1]["finished_on"], "2025-06-10");
        assert_eq!(f["series"][0]["episodes"], 2);
        assert_eq!(f["count"], 2);
        assert_eq!(names("dropped"), ["Dropped Tales"]);
        assert_eq!((f["dropped"][0]["seen"].as_i64(), f["dropped"][0]["total"].as_i64()), (Some(1), Some(4)));
        assert_eq!(f["dropped_count"], 1);
        assert_eq!(names_of(&year_of(&c, Some("ua"), "2024")["finished"]["series"]), ["Old Show"]);
        assert!(year_of(&c, Some("ub"), "2025")["finished"].is_null(), "bob touched no show");
    }

    fn names_of(v: &Value) -> Vec<String> {
        v.as_array().map(|a| a.iter().map(|s| s["name"].as_str().unwrap_or_default().to_string()).collect()).unwrap_or_default()
    }

    fn request(c: &Connection, id: i64, user: &str, title: &str, item: Option<&str>, asked: &str, arrived: Option<&str>) {
        let (asked, arrived) = (at(c, asked), arrived.map(|d| at(c, d)));
        c.execute(
            "INSERT INTO requests(service_id, request_id, media_type, title, status, media_status, requested_at, updated_at, available_at, user_id, item_id)
             VALUES (9, ?1, 'movie', ?2, 2, ?3, ?4, ?4, ?5, ?6, ?7)",
            crate::db::rusqlite::params![id, title, if arrived.is_some() { 5 } else { 3 }, asked, arrived, user, item],
        )
        .unwrap();
    }

    #[test]
    fn requests_count_what_became_watchable_and_what_was_watched() {
        let c = year_db();
        c.execute("INSERT INTO services(id, kind, name, url, secret, created_at) VALUES (9, 'seerr', 'Seerr', 'http://nas:5055', 'k', 1)", []).unwrap();
        assert!(year_of(&c, Some("ua"), "2025")["requests"].is_null(), "no request rows at all: no chapter");
        request(&c, 1, "ua", "Big Buck Bunny", Some("m1"), "2025-02-01", Some("2025-02-03"));
        play(&c, "ua", "m1", "Movie", "2025-02-04", 600, None);
        request(&c, 2, "ua", "Sintel", Some("m2"), "2025-03-01", Some("2025-03-05"));   // arrived, never watched
        request(&c, 3, "ua", "Tears of Steel", None, "2025-04-01", None);              // still waiting
        request(&c, 4, "ua", "Elephants Dream", None, "2024-04-01", None);             // another year
        request(&c, 5, "ub", "Cosmos Laundromat", None, "2025-04-01", None);           // somebody else's
        play(&c, "ua", "m2", "Movie", "2025-02-20", 900, None);                        // watched before it arrived: not because of it

        let r = &year_of(&c, Some("ua"), "2025")["requests"];
        assert_eq!((r["made"].as_i64(), r["available"].as_i64(), r["watched"].as_i64()), (Some(3), Some(2), Some(1)), "{r}");
        assert_eq!(names_of_titles(&r["top"]), ["Big Buck Bunny"]);
        let everyone = &year_of(&c, None, "2025")["requests"];
        assert_eq!(everyone["made"], 4, "the server's year counts everybody's: {everyone}");
        assert!(everyone.get("top").is_none_or(|t| t.as_array().is_some_and(|a| a.iter().all(|x| x.get("user_name").is_none()))));
    }

    /// A show's request points at the series; its plays point at episodes.
    #[test]
    fn a_requested_show_is_watched_when_one_of_its_episodes_is() {
        let c = year_db();
        c.execute("INSERT INTO services(id, kind, name, url, secret, created_at) VALUES (9, 'seerr', 'Seerr', 'http://nas:5055', 'k', 1)", []).unwrap();
        show(&c, "s1", "Sintel Stories", &["e1"]);
        request(&c, 1, "ua", "Sintel Stories", Some("s1"), "2025-02-01", Some("2025-02-03"));
        c.execute("UPDATE requests SET media_type = 'tv'", []).unwrap();
        episode(&c, "ua", "e1", "2025-02-04", 950);
        let r = &year_of(&c, Some("ua"), "2025")["requests"];
        assert_eq!(r["watched"], 1, "{r}");
        assert_eq!(names_of_titles(&r["top"]), ["Sintel Stories"]);
    }

    fn names_of_titles(v: &Value) -> Vec<String> {
        v.as_array().map(|a| a.iter().map(|s| s["title"].as_str().unwrap_or_default().to_string()).collect()).unwrap_or_default()
    }

    #[test]
    fn the_server_edition_names_nobody_and_ranks_nobody() {
        let c = year_db();
        let a = play(&c, "ua", "m1", "Movie", "2025-03-01", 600, None);
        play(&c, "ub", "m1", "Movie", "2025-03-01", 500, Some(a));
        c.execute("UPDATE playbacks SET group_id = ?1 WHERE id = ?1", [a]).unwrap();
        c.execute("UPDATE playbacks SET client = 'Jellyfin Web'", []).unwrap();
        play(&c, "uc", "m2", "Movie", "2025-05-01", 900, None);
        let v = server_edition(year_of(&c, None, "2025"));
        let text = v.to_string();
        for name in ["alice", "bob", "carol", "ua", "ub", "uc", "Jellyfin Web"] {
            assert!(!text.contains(&format!("\"{name}\"")), "{name} is in the server's year: {text}");
        }
        assert!(v.get("rank").is_none() && v.get("clients").is_none(), "no ranking, no apps");
        assert_eq!(v["scope"]["kind"], "server");
        assert_eq!(v["together"]["people_in_company"], 2);
        assert_eq!(v["totals"]["plays"], 3, "everybody's year");
        assert!(v["persona"].is_object(), "the house has a habit too");
    }

    fn told(c: &Connection) -> Vec<(String, String)> {
        let mut stmt = c.prepare("SELECT dedupe, COALESCE(user_name, '') FROM notify_events WHERE kind = 'recap_ready' ORDER BY dedupe").unwrap();
        stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?))).unwrap().collect::<Result<_, _>>().unwrap()
    }

    #[test]
    fn a_ready_year_is_announced_in_december_once_to_each_person_who_watched() {
        use crate::notify::{Kind, tests::{bus, target}};
        let c = year_db();
        play(&c, "ua", "m1", "Movie", "2025-03-01", 600, None);
        play(&c, "ub", "m2", "Movie", "2025-06-01", 900, None);
        play(&c, "uc", "m2", "Movie", "2024-06-01", 900, None);   // carol watched nothing in 2025
        let f = bus(&c, vec![target(1, None, &[Kind::RecapReady])]);
        let day = |d: &str| NaiveDate::parse_from_str(d, "%Y-%m-%d").unwrap();

        assert_eq!(announce_ready(&c, &f, day("2025-11-30")).unwrap(), 0, "not before the year is ready");
        assert!(told(&c).is_empty());
        assert_eq!(announce_ready(&c, &f, day("2025-12-01")).unwrap(), 2);
        assert_eq!(told(&c), [("notify:recap:2025:ua".into(), "alice".into()), ("notify:recap:2025:ub".into(), "bob".into())]);
        assert_eq!(announce_ready(&c, &f, day("2025-12-02")).unwrap(), 0, "once a year, however often it is asked");
        assert_eq!(announce_ready(&c, &f, day("2026-01-05")).unwrap(), 0, "January is not a second December");
        let (title, link): (String, String) = c.query_row("SELECT title, link FROM notify_events WHERE dedupe = 'notify:recap:2025:ua'", [], |r| Ok((r.get(0)?, r.get(1)?))).unwrap();
        assert_eq!((title.as_str(), link.as_str()), ("Your 2025 in review is ready", "/recap?year=2025"));
    }

    #[test]
    fn versus_is_the_year_before_or_nothing() {
        let c = year_db();
        play(&c, "ua", "m1", "Movie", "2024-06-01", 600, None);
        play(&c, "ua", "m2", "Movie", "2025-06-01", 900, None);
        play(&c, "ua", "m2", "Movie", "2025-06-02", 900, None);
        let v = &year_of(&c, Some("ua"), "2025")["versus"];
        assert_eq!((v["year"].as_i64(), v["plays"].as_i64(), v["watch_s"].as_i64(), v["active_days"].as_i64()), (Some(2024), Some(1), Some(600), Some(1)), "{v}");
        assert!(year_of(&c, Some("ua"), "2024")["versus"].is_null(), "nothing in 2023");
        assert!(year_of(&c, Some("ua"), "last12")["versus"].is_null(), "only a calendar year has a year before");
    }

    #[test]
    fn the_default_recap_flips_in_december() {
        assert_eq!(default_year(2026, 9, &[2026, 2025]), 2025);
        assert_eq!(default_year(2026, 12, &[2026, 2025]), 2026);
        assert_eq!(default_year(2027, 1, &[2027, 2026, 2025]), 2026);
        assert_eq!(default_year(2027, 11, &[2027, 2026, 2025]), 2026);
        assert_eq!(default_year(2027, 12, &[2027, 2026, 2025]), 2027);
        // Brand new install: last year has nothing, so show what there is.
        assert_eq!(default_year(2026, 9, &[2026]), 2026);
        assert_eq!(default_year(2026, 9, &[]), 2025);
    }

    #[test]
    fn persona_prefers_the_most_distinctive_habit() {
        let mut hours = vec![0i64; 24];
        hours[23] = 50;
        hours[20] = 50;
        let weekdays = vec![10, 10, 10, 10, 10, 25, 25];
        let totals = json!({ "plays": 100, "episodes": 90, "movies": 10, "tracks": 0, "active_days": 60, "user_days": 60 });
        assert_eq!(persona(&totals, &hours, &weekdays)["key"], "night_owl");

        hours[23] = 0;
        hours[20] = 100;
        assert_eq!(persona(&totals, &hours, &weekdays)["key"], "weekend_warrior");
        assert!(persona(&totals, &vec![0; 24], &weekdays).is_null());
    }
}
