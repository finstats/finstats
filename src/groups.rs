//! Group watching: different people playing the same title at the same time.
//!
//! Jellyfin does not expose SyncPlay groups through its sessions, so a group is inferred:
//! plays of one item by at least two different users that start within `group_window_s` of
//! each other and overlap long enough to have been watched together. Real history shows why
//! the window cannot be a few seconds: polling jitter and "you press play, I'm a moment late"
//! spread genuine group starts over up to a minute.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use anyhow::Result;
use axum::Json;
use axum::extract::{Query, State};
use serde_json::{Value, json};

use crate::auth::AuthUser;
use crate::db::SqlValue;
use crate::db::rusqlite::{Connection, params, params_from_iter};
use crate::state::{ApiResult, App};
use crate::stats::{FilterQuery, Scope};

/// Two plays were "together" only if they shared at least this much time.
const MIN_OVERLAP_S: i64 = 120;

struct Row {
    id: i64,
    user: String,
    start: i64,
    end: i64,
}

/// Given one item's plays in start order, the plays that belong to a group: (play id, group id).
fn cluster(rows: &[Row], window_s: i64) -> Vec<(i64, i64)> {
    let mut out = vec![];
    let mut i = 0;
    while i < rows.len() {
        // A chain of starts, each within the window of the one before it.
        let mut j = i + 1;
        while j < rows.len() && rows[j].start - rows[j - 1].start <= window_s {
            j += 1;
        }
        let chain = &rows[i..j];
        // Keep the plays that really ran alongside someone else's.
        let together: Vec<&Row> = chain
            .iter()
            .filter(|a| chain.iter().any(|b| b.user != a.user && a.end.min(b.end) - a.start.max(b.start) >= MIN_OVERLAP_S))
            .collect();
        if together.iter().map(|r| r.user.as_str()).collect::<BTreeSet<_>>().len() >= 2 {
            let group_id = together.iter().map(|r| r.id).min().expect("not empty");
            out.extend(together.iter().map(|r| (r.id, group_id)));
        }
        i = j;
    }
    out
}

/// The same idea for streams that are running right now: same title, different people, and either
/// started within the window or sitting at nearly the same position. While something is playing,
/// position is the stronger signal; start times are only as exact as the moment finstats first
/// noticed each stream. Adds `"group": {"size", "with": [{user_id, user_name}]}` to each member.
pub fn mark_live(sessions: &mut [Value], window_s: i64) {
    let near = window_s.max(30);
    let n = sessions.len();
    let mut with: Vec<Vec<usize>> = vec![vec![]; n];
    for a in 0..n {
        for b in 0..n {
            let (x, y) = (&sessions[a], &sessions[b]);
            if a == b || x["item_id"] != y["item_id"] || x["user_id"] == y["user_id"] {
                continue;
            }
            let close = |k: &str| matches!((x[k].as_i64(), y[k].as_i64()), (Some(p), Some(q)) if (p - q).abs() <= if k == "position_s" { near } else { window_s });
            if close("position_s") || close("started_at") {
                with[a].push(b);
            }
        }
    }
    for (i, others) in with.into_iter().enumerate() {
        if others.is_empty() {
            continue;
        }
        let mut people: Vec<Value> = vec![];
        for o in others {
            let who = json!({ "user_id": sessions[o]["user_id"], "user_name": sessions[o]["user_name"] });
            if !people.contains(&who) {
                people.push(who);
            }
        }
        sessions[i]["group"] = json!({ "size": people.len() + 1, "with": people });
    }
}

/// (Re)assign `group_id` for one item, or for everything. Returns the number of groups found.
pub fn detect(conn: &mut Connection, window_s: i64, only_item: Option<&str>) -> Result<usize> {
    let tx = conn.transaction()?;
    let mut groups = 0;
    {
        let filter = if only_item.is_some() { "AND item_id = ?1" } else { "" };
        let args: Vec<SqlValue> = only_item.map(|i| vec![i.to_string().into()]).unwrap_or_default();
        let mut stmt = tx.prepare(&format!("SELECT item_id, id, user_id, started_at, ended_at FROM playbacks WHERE 1 = 1 {filter} ORDER BY item_id, started_at, id"))?;
        let mut rows = stmt.query(params_from_iter(args.iter()))?;
        let mut assigned: Vec<(i64, i64)> = vec![];
        let mut current: Option<String> = None;
        let mut batch: Vec<Row> = vec![];
        let mut flush = |batch: &mut Vec<Row>| {
            if batch.len() >= 2 {
                assigned.extend(cluster(batch, window_s));
            }
            batch.clear();
        };
        while let Some(r) = rows.next()? {
            let item: String = r.get(0)?;
            if current.as_deref() != Some(item.as_str()) {
                flush(&mut batch);
                current = Some(item);
            }
            batch.push(Row { id: r.get(1)?, user: r.get(2)?, start: r.get(3)?, end: r.get(4)? });
        }
        flush(&mut batch);
        drop(rows);
        drop(stmt);

        tx.execute(&format!("UPDATE playbacks SET group_id = NULL WHERE group_id IS NOT NULL {filter}"), params_from_iter(args.iter()))?;
        let mut set = tx.prepare("UPDATE playbacks SET group_id = ?2 WHERE id = ?1")?;
        for (id, group_id) in &assigned {
            set.execute(params![id, group_id])?;
        }
        groups += assigned.iter().map(|(_, g)| *g).collect::<BTreeSet<_>>().len();
    }
    tx.commit()?;
    Ok(groups)
}

// ---------------------------------------------------------------- the fold (2.0)
//
// One grouped play as the SELECT hands it over; sessions are folded from these in Rust, and everything
// the page and the recap ask — pairs, time per day, each person's share — is read off the sessions
// by pure functions, so the recap can ask the same questions of one person and one year.

/// One row of the grouped-plays SELECT.
pub struct PlayRow {
    pub group_id: i64,
    pub user_id: String,
    pub user_name: String,
    pub has_image: bool,
    pub duration_s: i64,
    pub started_at: i64,
    pub item: Value,
    pub title_id: String,
    pub title_name: String,
}

/// One person's stay in a session.
#[derive(Clone, Debug)]
pub struct Member {
    pub name: String,
    pub has_image: bool,
    pub duration_s: i64,
}

/// One evening: a group, folded. `started_at` is when its first member pressed play.
pub struct Session {
    pub group_id: i64,
    pub started_at: i64,
    pub item: Value,
    pub title_id: String,
    pub title_name: String,
    pub per_user: BTreeMap<String, Member>,
}

impl Session {
    /// How long at least two people were watching: the second-longest stay.
    pub fn together_s(&self) -> i64 {
        let mut d: Vec<i64> = self.per_user.values().map(|m| m.duration_s).collect();
        d.sort_unstable_by(|a, b| b.cmp(a));
        d.get(1).copied().unwrap_or(0)
    }

    /// Everything its members watched, added up.
    pub fn person_s(&self) -> i64 {
        self.per_user.values().map(|m| m.duration_s).sum()
    }

    fn member_json(id: &str, m: &Member) -> Value {
        json!({ "user_id": id, "user_name": m.name, "has_image": m.has_image })
    }
}

/// A span of time, unix seconds: `[since, until)`. `until` is what a window of the past needs — the
/// window before this one, or the recap's one year.
pub struct Window {
    pub since: Option<i64>,
    pub until: Option<i64>,
}

/// Someone in a pair.
#[derive(Clone, Debug)]
pub struct Person {
    pub id: String,
    pub name: String,
    pub has_image: bool,
}

/// Two people and what they have watched together. Time together is the shorter of the two stays,
/// which is theirs alone: a session's own number describes any two of its members.
pub struct Pair {
    pub a: Person,
    pub b: Person,
    pub sessions: i64,
    pub together_s: i64,
    pub last_at: i64,
    pub top_title: Option<(String, String, i64)>,
}

/// The grouped plays in a window, folded into sessions. `user_ids` empty means everyone; otherwise
/// the whole of every group those people were in, which is what lets someone who may only see
/// themselves still see who they were watching with.
pub fn sessions_for(conn: &Connection, w: &Window, library_id: Option<&str>, user_ids: &[String]) -> Result<Vec<Session>> {
    let mut wh = vec!["p.group_id IS NOT NULL".to_string()];
    let mut args: Vec<SqlValue> = vec![];
    if let Some(s) = w.since {
        wh.push("p.started_at >= ?".into());
        args.push(s.into());
    }
    if let Some(u) = w.until {
        wh.push("p.started_at < ?".into());
        args.push(u.into());
    }
    if let Some(l) = library_id {
        wh.push("p.library_id = ?".into());
        args.push(l.to_string().into());
    }
    if !user_ids.is_empty() {
        let holes = std::iter::repeat_n("?", user_ids.len()).collect::<Vec<_>>().join(", ");
        wh.push(format!("p.group_id IN (SELECT group_id FROM playbacks WHERE user_id IN ({holes}) AND group_id IS NOT NULL)"));
        for u in user_ids {
            args.push(u.clone().into());
        }
    }
    let sql = format!(
        "SELECT p.group_id, p.user_id, COALESCE(u.name, p.user_name), (u.image_tag IS NOT NULL), p.duration_s, p.started_at,
                p.item_id, p.item_name, p.item_type, p.series_id, p.series_name, p.season_number, p.episode_number
         FROM playbacks p LEFT JOIN users u ON u.id = p.user_id WHERE {} ORDER BY p.group_id, p.started_at",
        wh.join(" AND ")
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params_from_iter(args.iter()), |r| {
        let (item_id, item_name, item_type): (String, String, String) = (r.get(6)?, r.get(7)?, r.get(8)?);
        let (series_id, series_name): (Option<String>, Option<String>) = (r.get(9)?, r.get(10)?);
        Ok(PlayRow {
            group_id: r.get(0)?,
            user_id: r.get(1)?,
            user_name: r.get(2)?,
            has_image: r.get(3)?,
            duration_s: r.get(4)?,
            started_at: r.get(5)?,
            title_id: series_id.clone().unwrap_or_else(|| item_id.clone()),
            title_name: series_name.clone().unwrap_or_else(|| item_name.clone()),
            item: json!({
                "item_id": item_id, "item_name": item_name, "item_type": item_type, "series_id": series_id, "series_name": series_name,
                "season_number": r.get::<_, Option<i64>>(11).unwrap_or(None), "episode_number": r.get::<_, Option<i64>>(12).unwrap_or(None),
                "image_item_id": series_id.clone().unwrap_or_else(|| item_id.clone()),
            }),
        })
    })?;
    Ok(fold_sessions(rows.collect::<Result<Vec<_>, _>>()?))
}

/// Rows into sessions, one per group, in group order.
pub fn fold_sessions(rows: impl IntoIterator<Item = PlayRow>) -> Vec<Session> {
    let mut sessions: BTreeMap<i64, Session> = BTreeMap::new();
    for r in rows {
        let s = sessions.entry(r.group_id).or_insert_with(|| Session {
            group_id: r.group_id, started_at: r.started_at, item: r.item.clone(), title_id: r.title_id.clone(), title_name: r.title_name.clone(), per_user: BTreeMap::new(),
        });
        s.started_at = s.started_at.min(r.started_at);
        let m = s.per_user.entry(r.user_id).or_insert(Member { name: r.user_name, has_image: r.has_image, duration_s: 0 });
        m.duration_s += r.duration_s;
    }
    sessions.into_values().collect()
}

/// Every pair of people who watched something together, most time together first. With
/// `must_include`, only pairs that include one of those people: a caller who may see only
/// themselves is answered their own pairs and nobody else's.
pub fn pairs(sessions: &[Session], must_include: &[String]) -> Vec<Pair> {
    // A pair so far, and how often it has watched each title: (name, sessions).
    type PerTitle = HashMap<String, (String, i64)>;
    let mut acc: BTreeMap<(String, String), (Pair, PerTitle)> = BTreeMap::new();
    for s in sessions {
        let members: Vec<(&String, &Member)> = s.per_user.iter().collect();
        for (i, (ia, ma)) in members.iter().enumerate() {
            for (ib, mb) in members.iter().skip(i + 1) {
                if !must_include.is_empty() && !must_include.contains(ia) && !must_include.contains(ib) {
                    continue;
                }
                let key = ((*ia).clone(), (*ib).clone());
                let (p, per_title) = acc.entry(key).or_insert_with(|| {
                    (Pair { a: Person { id: (*ia).clone(), name: ma.name.clone(), has_image: ma.has_image },
                            b: Person { id: (*ib).clone(), name: mb.name.clone(), has_image: mb.has_image },
                            sessions: 0, together_s: 0, last_at: 0, top_title: None }, HashMap::new())
                });
                p.sessions += 1;
                p.together_s += ma.duration_s.min(mb.duration_s);
                p.last_at = p.last_at.max(s.started_at);
                let t = per_title.entry(s.title_id.clone()).or_insert((s.title_name.clone(), 0));
                t.1 += 1;
            }
        }
    }
    let mut out: Vec<Pair> = acc
        .into_values()
        .map(|(mut p, per_title)| {
            p.top_title = per_title.into_iter().map(|(id, (name, n))| (id, name, n)).max_by_key(|(id, _, n)| (*n, std::cmp::Reverse(id.clone())));
            p
        })
        .collect();
    out.sort_by_key(|p| (std::cmp::Reverse(p.together_s), std::cmp::Reverse(p.sessions)));
    out
}

/// Per bucket of the caller's choosing (a day, a week): the time together of the sessions that
/// started in it, and everything the scoped people (everyone, when `scoped` is empty) watched in
/// them — what "alone" is taken from. A session belongs whole to the bucket it started in.
pub fn per_bucket(sessions: &[Session], scoped: &[String], mut bucket_of: impl FnMut(i64) -> Result<String>) -> Result<BTreeMap<String, (i64, i64)>> {
    let mut out: BTreeMap<String, (i64, i64)> = BTreeMap::new();
    for s in sessions {
        let grouped: i64 = s.per_user.iter().filter(|(id, _)| scoped.is_empty() || scoped.contains(id)).map(|(_, m)| m.duration_s).sum();
        let e = out.entry(bucket_of(s.started_at)?).or_insert((0, 0));
        e.0 += s.together_s();
        e.1 += grouped;
    }
    Ok(out)
}

/// Each person's time in company against their time alone. `totals` is everything each of them
/// watched (user id → name, has_image, seconds); someone in a group but not in the totals has no
/// time alone to speak of. Most together first.
pub fn people_shares(sessions: &[Session], totals: &BTreeMap<String, (String, bool, i64)>) -> Vec<Value> {
    let mut grouped: BTreeMap<&str, (String, bool, i64)> = BTreeMap::new();
    for s in sessions {
        for (id, m) in &s.per_user {
            let e = grouped.entry(id.as_str()).or_insert((m.name.clone(), m.has_image, 0));
            e.2 += m.duration_s;
        }
    }
    let mut out: Vec<Value> = totals
        .iter()
        .map(|(id, (name, img, total))| {
            let together = grouped.get(id.as_str()).map(|g| g.2).unwrap_or(0);
            let total = (*total).max(together);
            let share = if total > 0 { (together as f64 / total as f64 * 1000.0).round() / 1000.0 } else { 0.0 };
            json!({ "user_id": id, "user_name": name, "has_image": img, "together_s": together, "alone_s": total - together, "total_s": total, "share": share })
        })
        .chain(grouped.iter().filter(|(id, _)| !totals.contains_key(**id)).map(|(id, (name, img, t))| {
            json!({ "user_id": id, "user_name": name, "has_image": img, "together_s": t, "alone_s": 0, "total_s": t, "share": 1.0 })
        }))
        .collect();
    out.sort_by(|a, b| b["share"].as_f64().partial_cmp(&a["share"].as_f64()).unwrap_or(std::cmp::Ordering::Equal).then_with(|| b["together_s"].as_i64().cmp(&a["together_s"].as_i64())));
    out
}

/// The groups as sets of members — what the dashboard card has always shown — most time together first, at most eight.
pub fn companions(sessions: &[Session]) -> Vec<Value> {
    let mut acc: HashMap<Vec<String>, (Vec<Value>, i64, i64, i64)> = HashMap::new();
    for s in sessions {
        let key: Vec<String> = s.per_user.keys().cloned().collect();
        let c = acc.entry(key).or_insert_with(|| (s.per_user.iter().map(|(id, m)| Session::member_json(id, m)).collect(), 0, 0, 0));
        c.1 += 1;
        c.2 += s.together_s();
        c.3 = c.3.max(s.started_at);
    }
    let mut out: Vec<Value> = acc.into_values().map(|(members, n, t, last)| json!({ "members": members, "sessions": n, "together_s": t, "last_at": last })).collect();
    out.sort_by_key(|c| std::cmp::Reverse(c["together_s"].as_i64().unwrap_or(0)));
    out.truncate(8);
    out
}

/// The titles watched together most, at most eight.
pub fn titles(sessions: &[Session]) -> Vec<Value> {
    let mut acc: HashMap<String, (String, i64, i64)> = HashMap::new();
    for s in sessions {
        let t = acc.entry(s.title_id.clone()).or_insert((s.title_name.clone(), 0, 0));
        t.1 += 1;
        t.2 += s.together_s();
    }
    let mut out: Vec<Value> = acc.into_iter().map(|(id, (name, n, t))| json!({ "id": id, "name": name, "image_item_id": id, "sessions": n, "together_s": t })).collect();
    out.sort_by_key(|c| std::cmp::Reverse(c["together_s"].as_i64().unwrap_or(0)));
    out.truncate(8);
    out
}

/// The newest sessions, ten of them, each with its members and how long each stayed.
pub fn recent(sessions: &[Session]) -> Vec<Value> {
    let mut list: Vec<&Session> = sessions.iter().collect();
    list.sort_by_key(|s| std::cmp::Reverse(s.started_at));
    list.into_iter()
        .take(10)
        .map(|s| {
            let mut v = s.item.clone();
            v["group_id"] = json!(s.group_id);
            v["started_at"] = json!(s.started_at);
            v["together_s"] = json!(s.together_s());
            v["members"] = json!(s.per_user.iter().map(|(id, m)| json!({ "user_id": id, "user_name": m.name, "has_image": m.has_image, "duration_s": m.duration_s })).collect::<Vec<_>>());
            v
        })
        .collect()
}

/// Everything each scoped person watched in a window: user id → (name, has_image, seconds).
fn watch_totals(c: &Connection, cond: &crate::stats::Cond) -> Result<BTreeMap<String, (String, bool, i64)>> {
    let mut stmt = c.prepare(&format!(
        "SELECT p.user_id, COALESCE(u.name, p.user_name), (u.image_tag IS NOT NULL), COALESCE(SUM(p.duration_s), 0)
         FROM playbacks p LEFT JOIN users u ON u.id = p.user_id {} GROUP BY p.user_id",
        cond.sql()
    ))?;
    let rows = stmt.query_map(params_from_iter(cond.args.iter()), |r| Ok((r.get::<_, String>(0)?, (r.get(1)?, r.get(2)?, r.get(3)?))))?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// The scoped people's time inside group sessions, and their time overall: what the share is made of.
fn share_of(sessions: &[Session], scoped: &[String], totals: &BTreeMap<String, (String, bool, i64)>) -> (i64, i64, Value) {
    let grouped: i64 = sessions.iter().flat_map(|s| s.per_user.iter()).filter(|(id, _)| scoped.is_empty() || scoped.contains(id)).map(|(_, m)| m.duration_s).sum();
    let watch: i64 = totals.values().map(|t| t.2).sum::<i64>().max(grouped);
    let share = if watch > 0 { json!(((grouped as f64 / watch as f64) * 1000.0).round() / 1000.0) } else { Value::Null };
    (grouped, watch, share)
}

fn pair_json(p: &Pair) -> Value {
    let person = |x: &Person| json!({ "user_id": x.id, "user_name": x.name, "has_image": x.has_image });
    json!({
        "members": [person(&p.a), person(&p.b)], "sessions": p.sessions, "together_s": p.together_s, "last_at": p.last_at,
        "top_title": p.top_title.as_ref().map(|(id, name, n)| json!({ "id": id, "name": name, "image_item_id": id, "sessions": n })),
    })
}

/// What the Together page, the dashboard card and the profile card read, from one call: the lists the
/// cards always had, plus the pairs, a gap-free series of time together against time alone, each
/// person's share, and the same window right before for the deltas.
pub(crate) fn answer(c: &Connection, scope: &Scope) -> Result<Value> {
    let sessions = sessions_for(c, &Window { since: scope.since, until: None }, scope.library_id.as_deref(), &scope.user_ids)?;
    let people_seen: BTreeSet<&str> = sessions.iter().flat_map(|s| s.per_user.keys().map(String::as_str)).collect();
    let cond = scope.cond();
    let totals = watch_totals(c, &cond)?;
    let (_, watch_s, share) = share_of(&sessions, &scope.user_ids, &totals);

    // The series: `daily` gives the grid and everything watched per bucket; the sessions give the time together.
    let (template, bucket) = crate::stats::daily(c, scope, &cond)?;
    let mut date_stmt = c.prepare_cached(if bucket == "week" { "SELECT date(?1, 'unixepoch', 'localtime', 'weekday 0', '-6 days')" } else { "SELECT date(?1, 'unixepoch', 'localtime')" })?;
    let buckets = per_bucket(&sessions, &scope.user_ids, |t| Ok(date_stmt.query_row([t], |r| r.get::<_, String>(0))?))?;
    let series: Vec<Value> = template
        .iter()
        .map(|d| {
            let date = d["date"].as_str().unwrap_or_default();
            let (t, g) = buckets.get(date).copied().unwrap_or((0, 0));
            json!({ "date": date, "together_s": t, "alone_s": (d["watch_s"].as_i64().unwrap_or(0) - g).max(0) })
        })
        .collect();

    let pairs_json: Vec<Value> = pairs(&sessions, &scope.user_ids).iter().take(20).map(pair_json).collect();
    // Without "see everyone" the scope is the caller alone, and so is this list: companions' names are
    // theirs to see, companions' time alone is not.
    let people: Vec<Value> = people_shares(&sessions, &totals)
        .into_iter()
        .filter(|p| scope.user_ids.is_empty() || p["user_id"].as_str().is_some_and(|id| scope.user_ids.iter().any(|u| u == id)))
        .collect();

    let previous = match (scope.since, scope.previous_cond()) {
        (Some(since), Some(pcond)) => {
            let before = sessions_for(c, &Window { since: Some(since - scope.days * 86_400), until: Some(since) }, scope.library_id.as_deref(), &scope.user_ids)?;
            let ptotals = watch_totals(c, &pcond)?;
            let (_, _, pshare) = share_of(&before, &scope.user_ids, &ptotals);
            let ppeople: BTreeSet<&str> = before.iter().flat_map(|s| s.per_user.keys().map(String::as_str)).collect();
            json!({ "sessions": before.len(), "together_s": before.iter().map(Session::together_s).sum::<i64>(), "people": ppeople.len(), "share": pshare })
        }
        _ => Value::Null,
    };

    Ok(json!({
        "totals": {
            "sessions": sessions.len(),
            "together_s": sessions.iter().map(Session::together_s).sum::<i64>(),
            "person_s": sessions.iter().map(Session::person_s).sum::<i64>(),
            "people": people_seen.len(),
            "watch_s": watch_s,
            "share": share,
        },
        "previous": previous, "bucket": bucket, "series": series, "pairs": pairs_json, "people": people,
        "companions": companions(&sessions), "titles": titles(&sessions), "recent": recent(&sessions),
    }))
}

/// `GET /api/stats/groups` — who watches together, what, and for how long.
/// Without "see everyone" only groups the caller was part of are returned.
pub async fn groups(State(app): State<App>, user: AuthUser, Query(q): Query<FilterQuery>) -> ApiResult {
    let scope = Scope::new(&app, &user, &q);
    let out = app.db.call(move |c| answer(c, &scope.resolve(c)?)).await?;
    Ok(Json(out))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(id: i64, user: &str, start: i64, len: i64) -> Row {
        Row { id, user: user.into(), start, end: start + len }
    }

    // ---- the fold and what is read off it (2.0)

    fn play(gid: i64, user: &str, dur: i64, started: i64, title: &str) -> PlayRow {
        PlayRow { group_id: gid, user_id: user.into(), user_name: user.to_uppercase(), has_image: false, duration_s: dur, started_at: started,
                  item: json!({ "item_id": title, "item_name": title }), title_id: title.into(), title_name: title.into() }
    }

    fn conn() -> Connection {
        let c = Connection::open_in_memory().unwrap();
        for m in crate::db::MIGRATIONS {
            c.execute_batch(m).unwrap();
        }
        c
    }

    #[test]
    fn a_three_person_evening_counts_for_each_of_its_three_pairs() {
        // The card keyed companions by the exact set of members, so an evening of three counted for
        // none of its pairs. A pair's time together is the shorter of the two stays.
        let sessions = fold_sessions(vec![play(1, "a", 3000, 100, "Film"), play(1, "b", 2800, 105, "Film"), play(1, "c", 1200, 110, "Film")]);
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].together_s(), 2800, "the session's own time together is the second-longest stay");
        let ps = pairs(&sessions, &[]);
        let key = |p: &Pair| (p.a.id.clone(), p.b.id.clone(), p.together_s);
        assert_eq!(ps.iter().map(key).collect::<Vec<_>>(), [("a".into(), "b".into(), 2800), ("a".into(), "c".into(), 1200), ("b".into(), "c".into(), 1200)]);
        assert!(ps.iter().all(|p| p.sessions == 1));
    }

    #[test]
    fn a_sessions_time_together_lands_on_the_day_it_started() {
        // Started just before midnight and running past it: the whole of it belongs to the day it began.
        let sessions = fold_sessions(vec![play(1, "a", 3000, 86_399, "Film"), play(1, "b", 2500, 86_400 + 30, "Film")]);
        assert_eq!(sessions[0].started_at, 86_399, "a session starts when its first member did");
        let buckets = per_bucket(&sessions, &[], |t| Ok(format!("d{}", t / 86_400))).unwrap();
        assert_eq!(buckets.get("d0"), Some(&(2500, 5500)), "time together, and everything the people in it watched");
        assert!(!buckets.contains_key("d1"));
        // Scoped to one person, the grouped watch is that person's alone.
        let mine = per_bucket(&sessions, &["a".into()], |t| Ok(format!("d{}", t / 86_400))).unwrap();
        assert_eq!(mine.get("d0"), Some(&(2500, 3000)));
    }

    #[test]
    fn alone_is_what_was_watched_outside_a_group() {
        let sessions = fold_sessions(vec![play(1, "a", 3000, 100, "Film"), play(1, "b", 2500, 105, "Film")]);
        let mut totals = BTreeMap::new();
        totals.insert("a".to_string(), ("A".to_string(), false, 10_000));
        totals.insert("b".to_string(), ("B".to_string(), true, 2_500));
        totals.insert("c".to_string(), ("C".to_string(), false, 400));
        let people = people_shares(&sessions, &totals);
        let of = |id: &str| people.iter().find(|p| p["user_id"] == id).cloned().unwrap();
        assert_eq!((of("a")["together_s"].as_i64(), of("a")["alone_s"].as_i64(), of("a")["share"].as_f64()), (Some(3000), Some(7000), Some(0.3)));
        assert_eq!((of("b")["alone_s"].as_i64(), of("b")["share"].as_f64(), of("b")["has_image"].as_bool()), (Some(0), Some(1.0), Some(true)));
        assert_eq!((of("c")["together_s"].as_i64(), of("c")["share"].as_f64()), (Some(0), Some(0.0)), "never in a group");
        assert!(people[0]["share"].as_f64() >= people[1]["share"].as_f64(), "most together first");
    }

    #[test]
    fn without_see_everyone_only_pairs_with_the_caller_are_answered() {
        let sessions = fold_sessions(vec![play(1, "a", 3000, 100, "X"), play(1, "b", 2500, 105, "X"), play(2, "c", 900, 5000, "Y"), play(2, "d", 900, 5001, "Y")]);
        let mine = pairs(&sessions, &["a".into()]);
        assert_eq!(mine.len(), 1);
        assert_eq!((mine[0].a.id.as_str(), mine[0].b.id.as_str()), ("a", "b"));
        assert_eq!(pairs(&sessions, &[]).len(), 2, "unscoped, every pair");
        // And from the database: the whole of a's group comes back, and nothing of c and d's.
        let c = conn();
        c.execute_batch(
            "INSERT INTO playbacks(id, source, user_id, user_name, item_id, item_name, item_type, started_at, ended_at, duration_s, group_id) VALUES
               (1, 'live', 'a', 'A', 'x', 'X', 'Movie', 100, 3100, 3000, 1), (2, 'live', 'b', 'B', 'x', 'X', 'Movie', 105, 2605, 2500, 1),
               (3, 'live', 'c', 'C', 'y', 'Y', 'Movie', 5000, 5900, 900, 3), (4, 'live', 'd', 'D', 'y', 'Y', 'Movie', 5001, 5901, 900, 3),
               (5, 'live', 'a', 'A', 'z', 'Z', 'Movie', 9000, 9900, 900, NULL);",
        )
        .unwrap();
        let s = sessions_for(&c, &Window { since: None, until: None }, None, &["a".into()]).unwrap();
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].per_user.keys().cloned().collect::<Vec<_>>(), ["a", "b"], "b is in a's group, so b is answered");
        assert_eq!(sessions_for(&c, &Window { since: None, until: None }, None, &[]).unwrap().len(), 2);
    }

    #[test]
    fn the_previous_window_is_the_same_length_right_before() {
        // `until` is what makes a window of the past: the recap asks for one year, the page for the
        // window before this one. A session belongs to the window its start is in.
        let c = conn();
        c.execute_batch(
            "INSERT INTO playbacks(id, source, user_id, user_name, item_id, item_name, item_type, started_at, ended_at, duration_s, group_id) VALUES
               (1, 'live', 'a', 'A', 'x', 'X', 'Movie', 900, 3000, 2100, 1), (2, 'live', 'b', 'B', 'x', 'X', 'Movie', 905, 3000, 2095, 1),
               (3, 'live', 'a', 'A', 'y', 'Y', 'Movie', 1010, 3000, 1990, 3), (4, 'live', 'b', 'B', 'y', 'Y', 'Movie', 1012, 3000, 1988, 3),
               (5, 'live', 'a', 'A', 'z', 'Z', 'Movie', 1000 + 7 * 86400 + 5, 700000, 1000, 5), (6, 'live', 'b', 'B', 'z', 'Z', 'Movie', 1000 + 7 * 86400 + 9, 700000, 1000, 5);",
        )
        .unwrap();
        let w = Window { since: Some(1000), until: Some(1000 + 7 * 86400) };
        let s = sessions_for(&c, &w, None, &[]).unwrap();
        assert_eq!(s.iter().map(|x| x.title_id.as_str()).collect::<Vec<_>>(), ["y"]);
        let year = sessions_for(&c, &Window { since: Some(0), until: Some(31_536_000) }, None, &["a".into()]).unwrap();
        assert_eq!(year.len(), 3, "the recap: one person, one year");
    }

    #[test]
    fn a_pair_remembers_when_it_last_watched_and_what_it_watches_most() {
        let sessions = fold_sessions(vec![
            play(1, "a", 3000, 1000, "X"), play(1, "b", 3000, 1001, "X"),
            play(3, "a", 3000, 3000, "Y"), play(3, "b", 2000, 3001, "Y"),
            play(5, "a", 3000, 5000, "X"), play(5, "b", 1000, 5002, "X"),
        ]);
        let ps = pairs(&sessions, &[]);
        assert_eq!(ps.len(), 1);
        let p = &ps[0];
        assert_eq!((p.sessions, p.together_s, p.last_at), (3, 3000 + 2000 + 1000, 5000));
        assert_eq!(p.top_title.as_ref().map(|t| (t.0.as_str(), t.1.as_str(), t.2)), Some(("X", "X", 2)));
    }

    #[test]
    fn the_answer_keeps_the_keys_the_cards_already_read() {
        let sessions = fold_sessions(vec![play(1, "a", 3000, 1000, "X"), play(1, "b", 2500, 1001, "X")]);
        let comps = companions(&sessions);
        assert_eq!(comps[0]["members"].as_array().unwrap().len(), 2);
        assert_eq!((comps[0]["sessions"].as_i64(), comps[0]["together_s"].as_i64(), comps[0]["last_at"].as_i64()), (Some(1), Some(2500), Some(1000)));
        let ts = titles(&sessions);
        assert_eq!((ts[0]["id"].as_str(), ts[0]["sessions"].as_i64(), ts[0]["together_s"].as_i64()), (Some("X"), Some(1), Some(2500)));
        let rc = recent(&sessions);
        assert_eq!((rc[0]["started_at"].as_i64(), rc[0]["together_s"].as_i64(), rc[0]["members"].as_array().unwrap().len()), (Some(1000), Some(2500), 2));
        assert_eq!(rc[0]["members"][0]["duration_s"], 3000);
    }

    #[test]
    fn the_page_is_answered_pairs_a_series_people_and_the_window_before() {
        // Everything the Together page draws, from one call: the old keys, the pairs, a gap-free
        // series of time together against time alone, each person's share, and the window before.
        let c = conn();
        let since: i64 = c.query_row("SELECT CAST(strftime('%s', date('now', 'localtime', '-6 days'), 'utc') AS INTEGER)", [], |r| r.get(0)).unwrap();
        let (now_ish, before) = (since + 3600, since - 3600);
        c.execute_batch(&format!(
            "INSERT INTO playbacks(id, source, user_id, user_name, item_id, item_name, item_type, started_at, ended_at, duration_s, group_id) VALUES
               (1, 'live', 'a', 'A', 'x', 'X', 'Movie', {now_ish}, {}, 3000, 1), (2, 'live', 'b', 'B', 'x', 'X', 'Movie', {}, {}, 2500, 1),
               (3, 'live', 'a', 'A', 'z', 'Z', 'Movie', {}, {}, 1000, NULL),
               (4, 'live', 'a', 'A', 'y', 'Y', 'Movie', {before}, {}, 900, 4), (5, 'live', 'c', 'C', 'y', 'Y', 'Movie', {}, {}, 800, 4);",
            now_ish + 3000, now_ish + 5, now_ish + 2505, now_ish + 7200, now_ish + 8200, before + 900, before + 2, before + 802
        ))
        .unwrap();
        let scope = Scope { days: 7, since: Some(since), user_ids: vec![], library_id: None, min_play_s: 0, perms: crate::auth::Perms::ALL };
        let a = answer(&c, &scope).unwrap();
        assert_eq!((a["totals"]["sessions"].as_i64(), a["totals"]["together_s"].as_i64(), a["totals"]["watch_s"].as_i64()), (Some(1), Some(2500), Some(6500)));
        assert_eq!(a["totals"]["share"], ((5500.0 / 6500.0) * 1000.0f64).round() / 1000.0);
        assert_eq!(a["bucket"], "day");
        let series = a["series"].as_array().unwrap();
        assert_eq!(series.len(), 7, "one entry per day of the window, gaps included");
        assert_eq!((series[0]["together_s"].as_i64(), series[0]["alone_s"].as_i64()), (Some(2500), Some(1000)), "a's solo play is the only time alone that day");
        assert!(series[1..].iter().all(|d| d["together_s"] == 0 && d["alone_s"] == 0));
        assert_eq!(a["pairs"].as_array().unwrap().len(), 1);
        assert_eq!(a["pairs"][0]["members"].as_array().unwrap().len(), 2);
        assert_eq!(a["pairs"][0]["top_title"]["name"], "X");
        let people = a["people"].as_array().unwrap();
        assert_eq!(people.len(), 2);
        assert_eq!(a["previous"]["sessions"], 1, "the week before had one evening");
        assert_eq!(a["previous"]["together_s"], 800);
        assert!(a["companions"].is_array() && a["titles"].is_array() && a["recent"].is_array());
        // All time: no window before.
        let all = answer(&c, &Scope { days: 0, since: None, ..scope }).unwrap();
        assert!(all["previous"].is_null());
        assert_eq!(all["totals"]["sessions"], 2);
    }

    #[test]
    fn people_starting_together_form_a_group() {
        // a and b press play 4 s apart, c joins 40 s later; d watches the same thing an hour on.
        let rows = vec![r(1, "a", 0, 1500), r(2, "b", 4, 1500), r(3, "c", 44, 1400), r(4, "d", 3600, 1500)];
        assert_eq!(cluster(&rows, 60), vec![(1, 1), (2, 1), (3, 1)]);
        assert_eq!(cluster(&rows, 5), vec![(1, 1), (2, 1)], "a tight window loses the late joiner");
    }

    #[test]
    fn live_streams_at_the_same_spot_are_a_group() {
        let s = |user: &str, item: &str, pos: i64, started: i64| json!({ "user_id": user, "user_name": user, "item_id": item, "position_s": pos, "started_at": started });
        // a and b: same episode, same position, though finstats noticed them minutes apart (it restarted).
        // c: same episode, far behind, started long before. d: something else at the same position.
        let mut live = vec![s("a", "ep", 902, 5000), s("b", "ep", 905, 5400), s("c", "ep", 120, 1000), s("d", "other", 902, 5000), s("a", "ep", 903, 5001)];
        mark_live(&mut live, 60);
        assert_eq!(live[0]["group"]["size"], 2);
        assert_eq!(live[0]["group"]["with"][0]["user_name"], "b");
        assert_eq!(live[1]["group"]["with"].as_array().unwrap().len(), 1, "a on two devices is still one person");
        assert!(live[2]["group"].is_null() && live[3]["group"].is_null());
    }

    #[test]
    fn coincidences_and_solo_viewing_are_not_groups() {
        // Same person on two devices is not a group.
        assert!(cluster(&[r(1, "a", 0, 1500), r(2, "a", 3, 1500)], 60).is_empty());
        // Started together, but one of them left after a minute: never really together.
        assert!(cluster(&[r(1, "a", 0, 1500), r(2, "b", 2, 60)], 60).is_empty());
        // A restart by the same person stays in the group; the one who bailed at once does not.
        let rows = vec![r(1, "a", 0, 900), r(2, "b", 3, 1500), r(3, "c", 5, 30), r(4, "a", 50, 600)];
        assert_eq!(cluster(&rows, 60), vec![(1, 1), (2, 1), (4, 1)]);
    }
}
