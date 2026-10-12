//! Jellyfin's own background jobs: what they are, in plain words, and how far along they are.
//!
//! Jellyfin lists its scheduled tasks under names like "Detect and Analyze Media Segments", which says
//! what the code is called rather than what it is doing to your server. `explain` answers the question
//! the name does not: what it goes through, why you would want it, and whether it is one of the heavy
//! ones. Unknown tasks (a plugin's own) fall back to Jellyfin's description, and say where they came from.
//!
//! **A run is timed by watching it, because Jellyfin does not say when it started.** A task carries a
//! percentage and nothing else (no start time for the run in progress), so `Watch` remembers the
//! percentages this FinStats has seen and `eta_s` works the rest out from the rate they moved at. When
//! nothing has been seen to move there is no number, and the page says so rather than guessing.
//!
//! **The watching does not wait for somebody to look.** `observe` feeds the same `Watch` from task lists
//! FinStats has already read for other reasons (the scan check every 5 minutes, the server details every
//! 15), so a run that has been going for an hour is an estimate the moment the page opens, instead of an
//! ellipsis until it has been watched. Those reads happen either way; this only stops throwing the
//! percentages in them away.
//!
//! **Nothing is asked for unless somebody is looking.** The list is read from Jellyfin *for this endpoint*
//! at most every `MIN_GAP_S`, only when it is called, and FinStats never starts, stops or changes a task
//! there: it is the same read-only relationship as everywhere else.

use std::collections::HashMap;

use serde_json::{Value, json};

use crate::auth::ServerViewer;
use crate::db;
use crate::state::{ApiResult, App};

/// Jellyfin is asked for its task list at most this often, however many people are watching.
const MIN_GAP_S: i64 = 3;
/// While a job runs, the backend reads the list this often by itself, so a rate is measured whether or not
/// anybody has the page open.
const BUSY_EVERY_S: u64 = 10;
/// .NET ticks: 10 million to the second.
const TICKS_PER_S: i64 = 10_000_000;

// ---------------------------------------------------------------- what a job actually does

/// The plain sentence for a task, in the reader's language (`server.jobs.explain.<which>`). `None` when
/// FinStats has never heard of it.
pub fn explain(key: &str, name: &str) -> Option<String> {
    explain_in(crate::i18n::current(), key, name)
}

fn explain_in(lang: &str, key: &str, name: &str) -> Option<String> {
    which(key, name).map(|w| crate::i18n::say(lang, &format!("server.jobs.explain.{w}"), &[]))
}

/// Which job this is, by Jellyfin's key first (which does not change with the server's language) and by
/// its name second.
fn which(key: &str, name: &str) -> Option<&'static str> {
    let k = key.to_ascii_lowercase();
    let n = name.to_ascii_lowercase();
    let is = |needles: &[&str]| needles.iter().any(|needle| k == *needle || n.contains(needle));

    if is(&["refreshlibrary", "scan media library"]) {
        return Some("scan_library");
    }
    if is(&["mediasegment", "media segments", "intro skip", "introskipper"]) {
        return Some("media_segments");
    }
    if is(&["refreshtrickplay", "trickplay"]) {
        return Some("trickplay");
    }
    if is(&["refreshchapterimages", "chapter image"]) {
        return Some("chapter_images");
    }
    if is(&["keyframeextraction", "keyframe"]) {
        return Some("keyframes");
    }
    if is(&["audionormalization", "audio normalization", "audio normalisation"]) {
        return Some("audio_normalization");
    }
    if is(&["refreshpeople", "refresh people"]) {
        return Some("refresh_people");
    }
    if is(&["refreshguide", "refresh guide"]) {
        return Some("refresh_guide");
    }
    if is(&["optimizedatabase", "optimize database", "optimise database"]) {
        return Some("optimize_database");
    }
    if is(&["cleanactivitylog", "activity log"]) {
        return Some("clean_activity_log");
    }
    if is(&["cleanlogs", "log file", "log files"]) {
        return Some("clean_logs");
    }
    if is(&["cleancache", "cache director", "cache file"]) {
        return Some("clean_cache");
    }
    if is(&["cleantranscode", "deletetranscodefiles", "transcode director", "transcoding temp"]) {
        return Some("clean_transcode");
    }
    if is(&["pluginupdate", "plugin update", "check for plugin"]) {
        return Some("plugin_updates");
    }
    if is(&["subtitle"]) {
        return Some("subtitles");
    }
    if is(&["thumbnail", "image extraction"]) {
        return Some("thumbnails");
    }
    if is(&["systemupdate", "check for updates", "update task"]) {
        return Some("system_update");
    }
    if is(&["userdata", "people images", "extract"]) {
        return None; // too vague to guess at: Jellyfin's own description is better than a wrong sentence
    }
    None
}

/// What FinStats says about a task it does not know: Jellyfin's own description, when there is one.
fn fallback(description: Option<&str>) -> String {
    match description.map(str::trim).filter(|d| !d.is_empty()) {
        Some(d) => crate::i18n::t("server.jobs.from_plugin", &[("description", d.into())]),
        None => crate::i18n::t("server.jobs.unknown", &[]),
    }
}

// ---------------------------------------------------------------- how long is left

/// What is left of a run, in seconds, **only when it has been measured**. The rate is taken against the
/// most recent reading far enough back to say anything (at least `MIN_GAIN` over `MIN_WINDOW_S`, within
/// `WINDOW_S`), so a job that speeds up or slows down is described by the pace it has now rather than the
/// one it averaged, and a job creeping a percent every few minutes is still measurable.
///
/// `None` means FinStats has not seen the percentage move and has nothing to work from. There is a
/// tempting number to put there (how long the last run took, applied to the fraction that is left), and
/// it is a guess: it stands still while the job does, it knows nothing about how much of this run has
/// already happened, and on the screen it is indistinguishable from an estimate that was earned. So it is
/// not offered. The page says it does not know, and `last_duration_s` is shown beside it for anyone who
/// wants to judge for themselves.
pub fn eta_s(progress: f64, gained: f64, over_s: i64) -> Option<i64> {
    const MAX_ETA_S: i64 = 30 * 86_400;
    let left = (100.0 - progress.clamp(0.0, 100.0)).max(0.0);
    if left < 0.05 {
        return Some(0);
    }
    if gained >= MIN_GAIN && over_s >= MIN_WINDOW_S {
        let per_s = gained / over_s as f64;
        return Some(((left / per_s).round() as i64).clamp(0, MAX_ETA_S));
    }
    None
}

// ---------------------------------------------------------------- when it runs

fn hhmm(ticks: i64) -> String {
    let total = (ticks / TICKS_PER_S).rem_euclid(86_400);
    format!("{:02}:{:02}", total / 3600, (total % 3600) / 60)
}

fn every(lang: &str, seconds: i64) -> String {
    let (one, many, count) = match seconds {
        s if s % 86_400 == 0 && s >= 86_400 => ("day", "days", s / 86_400),
        s if s % 3_600 == 0 && s >= 3_600 => ("hour", "hours", s / 3600),
        s if s >= 60 => ("minute", "minutes", (s as f64 / 60.0).round() as i64),
        s => ("second", "seconds", s),
    };
    // "every day", not "every 1 day": one of a thing is said by its own words.
    match count {
        1 => crate::i18n::say(lang, &format!("server.jobs.every.{one}"), &[]),
        n => crate::i18n::say(lang, &format!("server.jobs.every.{many}"), &[("count", n.into())]),
    }
}

/// Jellyfin's names for the days (it writes the name; an older server wrote its number), Sunday first, and
/// each one's key in the catalogue.
const DAYS: [(&str, &str); 7] = [("Sunday", "sun"), ("Monday", "mon"), ("Tuesday", "tue"), ("Wednesday", "wed"), ("Thursday", "thu"), ("Friday", "fri"), ("Saturday", "sat")];

/// When Jellyfin will run this, in the words its own settings page uses. A time of day is the *server's*
/// local time, which is why it is printed as a time rather than turned into a countdown FinStats cannot
/// promise; an interval is a real countdown, because it is measured from the last run.
pub fn schedule(triggers: &[Value], last_run_at: Option<i64>) -> (Vec<String>, Option<i64>) {
    schedule_in(crate::i18n::current(), triggers, last_run_at)
}

fn schedule_in(lang: &str, triggers: &[Value], last_run_at: Option<i64>) -> (Vec<String>, Option<i64>) {
    let mut words = vec![];
    let mut next_at: Option<i64> = None;
    for t in triggers {
        let kind = t["Type"].as_str().unwrap_or("");
        let ticks = |key: &str| t[key].as_i64();
        match kind {
            "DailyTrigger" => words.push(crate::i18n::say(lang, "server.jobs.daily", &[("time", hhmm(ticks("TimeOfDayTicks").unwrap_or(0)).into())])),
            "WeeklyTrigger" => {
                let time = hhmm(ticks("TimeOfDayTicks").unwrap_or(0));
                let day = match &t["DayOfWeek"] {
                    Value::String(s) => DAYS.iter().find(|(name, _)| name.eq_ignore_ascii_case(s)).map(|(_, key)| *key),
                    v => v.as_i64().and_then(|d| DAYS.get(d as usize)).map(|(_, key)| *key),
                };
                words.push(match day {
                    Some(key) => crate::i18n::say(lang, &format!("server.jobs.weekly.{key}"), &[("time", time.into())]),
                    None => crate::i18n::say(lang, "server.jobs.weekly.any", &[("time", time.into())]),
                });
            }
            "IntervalTrigger" => {
                let seconds = ticks("IntervalTicks").unwrap_or(0) / TICKS_PER_S;
                if seconds > 0 {
                    words.push(every(lang, seconds));
                    if let Some(last) = last_run_at {
                        let due = last + seconds;
                        next_at = Some(next_at.map_or(due, |n: i64| n.min(due)));
                    }
                }
            }
            "StartupTrigger" => words.push(crate::i18n::say(lang, "server.jobs.at_startup", &[])),
            "" => {}
            other => words.push(other.trim_end_matches("Trigger").to_ascii_lowercase()),
        }
    }
    (words, next_at)
}

// ---------------------------------------------------------------- watching a run

/// What FinStats has seen of one run. Jellyfin reports a percentage and never says when the run began,
/// so this is the only clock there is: the recent percentages, and when the last one actually changed.
pub struct Run {
    /// When FinStats first saw this run, which may be long after Jellyfin started it.
    pub first_at: i64,
    /// When the percentage last moved. Equal to `first_at` until it does.
    pub changed_at: i64,
    /// (when, percentage), oldest first, kept for at most `WINDOW_S`.
    samples: Vec<(i64, f64)>,
}

/// How far back a rate is measured. Long enough for a job that moves a percent every few minutes to be
/// measurable at all, short enough that an hour-old rate is not used to describe what is happening now.
const WINDOW_S: i64 = 900;
/// A percentage that moves by less than this has not moved: Jellyfin's numbers jitter in the last digits.
const MOVED: f64 = 0.01;
/// Movement worth measuring a rate from, and the shortest stretch it may be measured over.
const MIN_GAIN: f64 = 0.5;
const MIN_WINDOW_S: i64 = 5;

impl Run {
    fn new(at: i64, progress: f64) -> Run {
        Run { first_at: at, changed_at: at, samples: vec![(at, progress)] }
    }

    fn note(&mut self, at: i64, progress: f64) {
        if self.samples.last().is_none_or(|(_, p)| (progress - p).abs() > MOVED) {
            self.changed_at = at;
        }
        self.samples.push((at, progress));
        // Only the recent past decides the rate, but never fewer than two points to measure between.
        let cutoff = at - WINDOW_S;
        let keep = self.samples.iter().position(|(t, _)| *t >= cutoff).unwrap_or(0);
        if keep > 0 && self.samples.len() - keep >= 2 {
            self.samples.drain(..keep);
        }
    }

    /// (how much the percentage gained, over how many seconds), measured against the *most recent* point
    /// far enough back to say anything: a job moving quickly is described by the last few seconds, a slow
    /// one by as much of the window as it takes to see it move at all. Averaging over the whole run
    /// instead would describe a job that has changed pace by the pace it no longer has.
    fn measured(&self) -> (f64, i64) {
        let Some(&(t1, p1)) = self.samples.last() else { return (0.0, 0) };
        let anchor = self.samples.iter().rev().find(|(t, p)| p1 - p >= MIN_GAIN && t1 - t >= MIN_WINDOW_S);
        match anchor.or_else(|| self.samples.first()) {
            Some(&(t0, p0)) => (p1 - p0, t1 - t0),
            None => (0.0, 0),
        }
    }

    /// A run whose percentage went backwards is the next run, not this one running in reverse.
    fn restarted(&self, progress: f64) -> bool {
        self.samples.last().is_some_and(|(_, p)| progress + MOVED < *p)
    }
}

/// What FinStats remembers between two reads: the last answer, and how each running job has moved.
#[derive(Default)]
pub struct Watch {
    pub fetched_at: i64,
    pub answer: Option<Value>,
    runs: HashMap<String, Run>,
}

impl Watch {
    /// Note where a running job is now, and hand back what has been seen of this run so far.
    fn note(&mut self, id: &str, progress: f64, now: i64) -> &Run {
        let run = self.runs.entry(id.to_string()).or_insert_with(|| Run::new(now, progress));
        if run.restarted(progress) {
            *run = Run::new(now, progress);
        } else {
            run.note(now, progress);
        }
        run
    }

    /// How soon the backend should read Jellyfin's jobs again by itself: while it knows of a run in progress, often
    /// enough to time it; otherwise not at all, since the scheduler's five-minute read notices the next run.
    pub fn next_look(&self) -> Option<std::time::Duration> {
        (!self.runs.is_empty()).then_some(std::time::Duration::from_secs(BUSY_EVERY_S))
    }

    fn forget(&mut self, running: &[String]) {
        self.runs.retain(|id, _| running.iter().any(|r| r == id));
    }

    /// Note where every running job in a task list has got to. Unlike the endpoint's own path this
    /// touches nothing else: not `answer`, not `fetched_at`, so what a page is served and how often
    /// Jellyfin is read for it are exactly as they were.
    ///
    /// A job the list says is idle has finished, and its run is dropped. A job the list does not mention
    /// at all is left alone, because the lists FinStats reads elsewhere leave out the hidden tasks and a
    /// hidden job the page is watching must not be forgotten by a read that could not see it.
    pub fn observe(&mut self, tasks: &[Value], now: i64) {
        for t in tasks {
            let Some(id) = t["Id"].as_str().or_else(|| t["Key"].as_str()).filter(|s| !s.is_empty()) else { continue };
            if t["State"].as_str().unwrap_or("Idle") == "Idle" {
                self.runs.remove(id);
            } else {
                self.note(id, t["CurrentProgressPercentage"].as_f64().unwrap_or(0.0).clamp(0.0, 100.0), now);
            }
        }
    }
}

/// Time the runs in a task list FinStats read for another reason. Costs no request: the caller already
/// has the list in hand.
pub fn observe(app: &App, tasks: &[Value]) {
    if tasks.is_empty() {
        return;
    }
    app.jf_jobs.lock().unwrap().observe(tasks, db::now());
}

// ---------------------------------------------------------------- the answer

fn job_json(t: &Value, watch: &mut Watch, now: i64) -> Value {
    let key = t["Key"].as_str().unwrap_or("");
    let name = t["Name"].as_str().unwrap_or("");
    let id = t["Id"].as_str().unwrap_or(key);
    let state = t["State"].as_str().unwrap_or("Idle");
    let running = state != "Idle";
    let last = &t["LastExecutionResult"];
    let (started, ended) = (last["StartTimeUtc"].as_str().and_then(db::parse_ts), last["EndTimeUtc"].as_str().and_then(db::parse_ts));
    let last_duration_s = started.zip(ended).map(|(s, e)| (e - s).max(0));
    let (_, next_at) = schedule(t["Triggers"].as_array().map(Vec::as_slice).unwrap_or_default(), ended.or(started));
    let progress = running.then(|| t["CurrentProgressPercentage"].as_f64().unwrap_or(0.0).clamp(0.0, 100.0));

    let (eta, watched_since, unchanged_for) = match progress {
        Some(p) => {
            let run = watch.note(id, p, now);
            let (gained, over_s) = run.measured();
            let (first_at, stalled) = (run.first_at, now - run.changed_at);
            (eta_s(p, gained, over_s), Some(first_at), Some(stalled.max(0)))
        }
        None => (None, None, None),
    };

    json!({
        "id": id, "key": key, "name": name, "category": t["Category"],
        "state": state, "running": running, "hidden": t["IsHidden"].as_bool().unwrap_or(false),
        "progress": progress,
        "eta_s": eta,
        // When FinStats first saw this run. It may have been going for hours before anybody opened the page.
        "watching_since": watched_since,
        // How long the percentage has been standing still. A slow job is not a broken page.
        "unchanged_for_s": unchanged_for,
        "description": t["Description"],
        // Said in the reader's words as the answer is served (`worded`), which takes these out again.
        "triggers": t["Triggers"],
        "next_at": next_at,
        "last_result": last["Status"],
        "last_error": last["ErrorMessage"],
        "last_run_at": ended.or(started),
        "last_duration_s": last_duration_s,
    })
}

/// `GET /api/jellyfin/jobs`: what Jellyfin is doing right now, and what it will do later. Read-only, and
/// asked of Jellyfin at most every few seconds however many people are watching the page.
pub async fn jobs(axum::extract::State(app): axum::extract::State<App>, _viewer: ServerViewer) -> ApiResult {
    let now = db::now();
    {
        let watch = app.jf_jobs.lock().unwrap();
        if let Some(answer) = &watch.answer
            && now - watch.fetched_at < MIN_GAP_S
        {
            return Ok(axum::Json(worded(answer.clone())));
        }
    }
    Ok(axum::Json(worded(refresh(&app, now).await)))
}

/// The answer in the words of whoever asked. What is kept between reads is shared by every reader (and the
/// backend's own reads have none), so it holds what Jellyfin said, and the sentences are said here, as it is served.
fn worded(mut answer: Value) -> Value {
    for j in answer["jobs"].as_array_mut().into_iter().flatten() {
        let (key, name) = (j["key"].as_str().unwrap_or("").to_string(), j["name"].as_str().unwrap_or("").to_string());
        let triggers = j.as_object_mut().and_then(|o| o.remove("triggers")).unwrap_or_default();
        let what = explain(&key, &name);
        j["known"] = json!(what.is_some());
        j["what"] = json!(what.unwrap_or_else(|| fallback(j["description"].as_str())));
        j["schedule"] = json!(schedule(triggers.as_array().map(Vec::as_slice).unwrap_or_default(), None).0);
        if name.is_empty() {
            j["name"] = json!(crate::i18n::t("server.jobs.unnamed", &[]));
        }
    }
    answer
}

/// One read of Jellyfin's jobs, timed and kept as the answer the page is served. The endpoint and the backend's
/// own watch (`run`) both go through here, so what a page is told was measured by the same clock either way.
async fn refresh(app: &App, now: i64) -> Value {
    let Some(jf) = app.jellyfin() else {
        return json!({ "jobs": [], "running": 0, "fetched_at": now, "error": crate::i18n::t("server.jobs.not_connected", &[]) });
    };
    let tasks = match jf.scheduled_tasks_all().await {
        Ok(tasks) => tasks,
        Err(e) => {
            tracing::debug!("could not read Jellyfin's scheduled tasks: {e:#}");
            let stale = app.jf_jobs.lock().unwrap().answer.clone();
            let mut answer = stale.unwrap_or_else(|| json!({ "jobs": [], "running": 0 }));
            answer["error"] = json!(crate::i18n::t("server.jobs.no_answer", &[]));
            answer["fetched_at"] = json!(now);
            return answer;
        }
    };
    let mut watch = app.jf_jobs.lock().unwrap();
    let mut jobs: Vec<Value> = tasks.iter().map(|t| job_json(t, &mut watch, now)).collect();
    watch.forget(&jobs.iter().filter(|j| j["running"] == true).filter_map(|j| j["id"].as_str().map(str::to_string)).collect::<Vec<_>>());
    // Running first, then whatever ran most recently: the page is read from the top.
    jobs.sort_by(|a, b| {
        let key = |j: &Value| (!j["running"].as_bool().unwrap_or(false), -j["last_run_at"].as_i64().unwrap_or(0));
        key(a).cmp(&key(b))
    });
    let running = jobs.iter().filter(|j| j["running"] == true).count();
    let answer = json!({ "jobs": jobs, "running": running, "fetched_at": now });
    watch.fetched_at = now;
    watch.answer = Some(answer.clone());
    answer
}

/// The backend's own watch over Jellyfin's jobs: while one runs, read the list every `BUSY_EVERY_S` so its rate is
/// measured with nobody looking, and the page is told an estimate the moment it opens. Idle, it makes no request:
/// the scheduler's five-minute read (`observe`) is what notices the next run, and this looks at what that read left
/// in memory every few seconds, which costs nothing.
pub async fn run(app: App) {
    loop {
        let next = app.jf_jobs.lock().unwrap().next_look();
        match next {
            Some(every) => {
                let now = db::now();
                let fresh = app.jf_jobs.lock().unwrap().fetched_at;
                // A page asking every three seconds has just read it; there is no need to ask again.
                if now - fresh >= every.as_secs() as i64 {
                    refresh(&app, now).await;
                }
                tokio::time::sleep(every).await;
            }
            None => tokio::time::sleep(std::time::Duration::from_secs(5)).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_schedule_is_said_the_way_jellyfin_s_own_page_says_it() {
        let ticks = |s: i64| s * TICKS_PER_S;
        let t = |v: Value| schedule(&[v], None).0;
        assert_eq!(t(json!({"Type": "DailyTrigger", "TimeOfDayTicks": ticks(3 * 3600 + 30 * 60)})), ["every day at 03:30"]);
        // Jellyfin writes the day as a name; an older server wrote its number, Sunday first.
        assert_eq!(t(json!({"Type": "WeeklyTrigger", "DayOfWeek": "Tuesday", "TimeOfDayTicks": ticks(22 * 3600)})), ["every Tuesday at 22:00"]);
        assert_eq!(t(json!({"Type": "WeeklyTrigger", "DayOfWeek": 0, "TimeOfDayTicks": 0})), ["every Sunday at 00:00"]);
        assert_eq!(t(json!({"Type": "WeeklyTrigger", "DayOfWeek": 9})), ["every week at 00:00"]);
        assert_eq!(t(json!({"Type": "StartupTrigger"})), ["when Jellyfin starts"]);
        assert_eq!(t(json!({"Type": "SomethingNewTrigger"})), ["somethingnew"]);
        assert!(t(json!({"Type": ""})).is_empty() && t(json!({"Type": "IntervalTrigger", "IntervalTicks": 0})).is_empty());
        for (s, said) in [(86_400, "every day"), (3 * 86_400, "every 3 days"), (3_600, "every hour"), (6 * 3_600, "every 6 hours"), (900, "every 15 minutes"),
            (60, "every minute"), (90, "every 2 minutes"), (30, "every 30 seconds"), (1, "every second")] {
            assert_eq!(t(json!({"Type": "IntervalTrigger", "IntervalTicks": ticks(s)})), [said], "{s} s");
        }
        // An interval is a real countdown, from the last run; with two, the sooner one.
        let (_, next) = schedule(&[json!({"Type": "IntervalTrigger", "IntervalTicks": ticks(3_600)}), json!({"Type": "IntervalTrigger", "IntervalTicks": ticks(900)})], Some(1_000));
        assert_eq!(next, Some(1_900));
        assert_eq!(schedule(&[json!({"Type": "IntervalTrigger", "IntervalTicks": ticks(900)})], None).1, None, "a countdown without a last run");
    }

    #[test]
    fn the_jobs_people_ask_about_are_explained_in_words_that_are_not_their_name() {
        // The one that started this: a name that says what the code is called.
        let segments = explain("MediaSegmentDetect", "Detect and Analyze Media Segments").expect("media segments");
        assert!(segments.contains("skip") && segments.contains("heaviest"), "{segments}");
        assert!(explain("RefreshLibrary", "Scan Media Library").is_some_and(|s| s.contains("FinStats reads your library")));
        assert!(explain("", "Generate Trickplay Images").is_some(), "matched by name when the key is unknown");
        assert!(explain("OptimizeDatabase", "").is_some(), "matched by key when the name is in another language");
        assert!(explain("SomePluginTask", "Do the thing").is_none());
        assert!(fallback(Some("What the plugin says")).starts_with("What the plugin says"));
        assert!(fallback(None).contains("plugin"));
        for (key, name) in [("RefreshLibrary", "Scan Media Library"), ("CleanTranscode", "Clean Transcode Directory"), ("PluginUpdates", "Check for plugin updates")] {
            let text = explain(key, name).unwrap();
            assert!(text.len() > 80 && text.ends_with('.'), "{key}: {text}");
        }
    }

    #[test]
    fn what_is_left_comes_from_the_rate_it_has_actually_been_moving_at() {
        // Ten percent in twenty seconds, nine tenths to go: three minutes.
        assert_eq!(eta_s(10.0, 10.0, 20), Some(180));
        assert_eq!(eta_s(100.0, 0.0, 0), Some(0));
    }

    #[test]
    fn a_number_is_only_ever_shown_when_it_was_measured() {
        // The fault this was written for: a percentage that does not move left "about 2 minutes left" on
        // the screen for a quarter of an hour. There was a number to show (the last run's duration for
        // the fraction that is left), and it was a guess that stood still while the job did. Nothing that
        // was not measured is offered now, however tempting it looks.
        assert_eq!(eta_s(90.0, 0.0, 600), None, "ten minutes of watching a percentage that did not move");
        assert_eq!(eta_s(25.0, 0.0, 2), None, "nothing seen yet is nothing to say");
        assert_eq!(eta_s(50.0, 0.4, 3), None, "a window too short to believe is not believed");
        assert_eq!(eta_s(50.0, 0.4, 900), None, "nor is a stretch too small to measure over any length of time");
        // …and the moment there is something to measure, there is a number again.
        assert_eq!(eta_s(50.0, 0.5, 5), Some(500));
    }

    #[test]
    fn a_schedule_reads_the_way_jellyfins_own_settings_page_reads_it() {
        let daily = json!([{ "Type": "DailyTrigger", "TimeOfDayTicks": 3i64 * 3600 * TICKS_PER_S }]);
        let (words, next) = schedule(daily.as_array().unwrap(), Some(1_000));
        assert_eq!(words, vec!["every day at 03:00"]);
        assert_eq!(next, None, "a time of day is the server's own, so FinStats does not turn it into a countdown");

        let interval = json!([{ "Type": "IntervalTrigger", "IntervalTicks": 86_400i64 * TICKS_PER_S }, { "Type": "StartupTrigger" }]);
        let (words, next) = schedule(interval.as_array().unwrap(), Some(1_000));
        assert_eq!(words, vec!["every day", "when Jellyfin starts"]);
        assert_eq!(next, Some(1_000 + 86_400), "an interval is measured from the last run, so it is a real countdown");
    }

    #[test]
    fn a_run_is_timed_by_watching_it_and_a_restart_starts_the_watch_again() {
        let mut w = Watch::default();
        assert_eq!(w.note("a", 10.0, 100).first_at, 100);
        let run = w.note("a", 30.0, 140);
        let (gained, over_s) = run.measured();
        assert_eq!((gained, over_s, run.first_at, run.changed_at), (20.0, 40, 100, 140));
        assert_eq!(eta_s(30.0, gained, over_s), Some(140));

        // Standing still: the window still spans the same seconds, and `changed_at` stops moving.
        let run = w.note("a", 30.0, 200);
        assert_eq!((run.changed_at, run.measured()), (140, (20.0, 100)));

        // The percentage went back to the beginning: that is the next run, not the same one going backwards.
        let run = w.note("a", 2.0, 260);
        assert_eq!((run.first_at, run.changed_at, run.measured()), (260, 260, (0.0, 0)));

        w.forget(&["b".to_string()]);
        assert_eq!(w.note("a", 5.0, 300).first_at, 300, "a job that stopped is not remembered");
    }

    /// The whole point of `observe`: the reads FinStats already makes time the run, so the first thing a
    /// page is ever told about a slow job is a number rather than an ellipsis.
    #[test]
    fn a_job_is_already_timed_by_the_time_anybody_opens_the_page() {
        let task = |state: &str, pct: f64| json!({ "Id": "trickplay", "Key": "RefreshTrickplay", "State": state, "CurrentProgressPercentage": pct });
        let mut w = Watch::default();

        // Two of the five-minute scan checks, with nobody looking at anything.
        w.observe(&[task("Running", 4.0)], 0);
        w.observe(&[task("Running", 6.0)], 300);

        // Somebody opens the Server page. This is its first reading, and it already has an estimate:
        // 2% per five minutes, 94% to go.
        let run = w.note("trickplay", 6.0, 303);
        let (gained, over_s) = run.measured();
        assert_eq!(eta_s(6.0, gained, over_s), Some(14_241), "about four hours, said straight away");
        assert_eq!(run.first_at, 0, "watched since the first background reading, not since the page opened");

        // Without it, the same page would have nothing to say until the percentage moved under it.
        let mut cold = Watch::default();
        let run = cold.note("trickplay", 6.0, 303);
        let (gained, over_s) = run.measured();
        assert_eq!(eta_s(6.0, gained, over_s), None, "one reading is nothing to measure between");
    }

    /// The owner's request: the estimate lives in the backend, so the page is told one the moment it opens. A run is
    /// timed by FinStats itself while it goes on, not only while somebody has the page open.
    #[test]
    fn a_running_job_keeps_being_timed_with_nobody_looking() {
        let mut w = Watch::default();
        assert_eq!(w.next_look(), None, "nothing running: nothing to time, and no request made for it");
        w.observe(&[json!({ "Id": "scan", "State": "Running", "CurrentProgressPercentage": 3.0 })], 0);
        assert_eq!(w.next_look(), Some(std::time::Duration::from_secs(BUSY_EVERY_S)));
        w.observe(&[json!({ "Id": "scan", "State": "Idle" })], 60);
        assert_eq!(w.next_look(), None, "the run ended");
    }

    #[test]
    fn observing_forgets_what_has_stopped_but_never_what_it_could_not_see() {
        let mut w = Watch::default();
        w.note("hidden-one", 10.0, 0);
        w.note("scan", 10.0, 0);

        // The lists FinStats reads elsewhere leave the hidden tasks out. A job missing from one of them
        // has not stopped; it was never in the list to begin with.
        w.observe(&[json!({ "Id": "scan", "State": "Idle", "CurrentProgressPercentage": 0.0 })], 60);
        assert_eq!(w.note("hidden-one", 12.0, 60).first_at, 0, "a hidden run the read could not see is still being watched");
        assert_eq!(w.note("scan", 10.0, 60).first_at, 60, "a run the read says is over is dropped");
    }

    #[test]
    fn the_rate_is_measured_over_the_recent_past_not_the_whole_run() {
        let mut w = Watch::default();
        // An hour of one percent a minute, then three minutes of ten times that.
        for m in 0..60 {
            w.note("a", m as f64, m * 60);
        }
        for m in 1..=3 {
            w.note("a", 60.0 + (m as f64) * 10.0, 3_600 + m * 60);
        }
        let (gained, over_s) = w.runs["a"].measured();
        let per_min = gained / (over_s as f64 / 60.0);
        assert!(per_min > 8.0, "the hour that came before is still being averaged in: {per_min:.2}%/min");
        assert!(over_s <= WINDOW_S, "the window is not allowed to grow for ever: {over_s}s");

        // …and a job creeping a percent every five minutes is still measurable, which is why the window
        // reaches back as far as it does.
        let mut slow = Watch::default();
        for m in 0..10 {
            slow.note("b", m as f64, m * 300);
        }
        let (gained, over_s) = slow.runs["b"].measured();
        assert!(gained >= MIN_GAIN && over_s >= MIN_WINDOW_S, "a slow job cannot be measured at all: {gained}% over {over_s}s");
        assert_eq!(eta_s(9.0, gained, over_s), Some(27_300), "about seven and a half hours to go, at a percent every five minutes");
    }

    /// Every job FinStats knows is explained in every language it speaks, never by its key.
    #[test]
    fn every_known_job_is_explained_in_every_language() {
        let jobs = [
            "RefreshLibrary", "MediaSegmentDetect", "RefreshTrickplay", "RefreshChapterImages", "KeyframeExtraction", "AudioNormalization",
            "RefreshPeople", "RefreshGuide", "OptimizeDatabase", "CleanActivityLog", "CleanLogs", "CleanCache", "CleanTranscode",
            "PluginUpdates", "DownloadSubtitles", "Create thumbnail", "SystemUpdate",
        ];
        for lang in crate::i18n::LOCALES {
            let mut said = std::collections::BTreeSet::new();
            for key in jobs {
                let text = explain_in(lang, key, key).unwrap_or_else(|| panic!("{key} is not known"));
                assert!(!text.starts_with("server.") && text.ends_with(['.', '。', '।']), "{lang}: {key}: {text}"); // each language's own full stop
                said.insert(text);
            }
            assert_eq!(said.len(), jobs.len(), "{lang}: two jobs share a sentence");
        }
        assert!(explain_in("nb-NO", "MediaSegmentDetect", "Detect and Analyze Media Segments").is_some_and(|s| s.contains("Jellyfin")));
    }

    #[test]
    fn a_schedule_is_said_in_the_reader_s_language() {
        let ticks = |s: i64| s * TICKS_PER_S;
        let t = |lang: &str, v: Value| schedule_in(lang, &[v], None).0;
        assert_eq!(t("nb-NO", json!({"Type": "WeeklyTrigger", "DayOfWeek": "Tuesday", "TimeOfDayTicks": ticks(22 * 3600)})), ["hver tirsdag kl. 22:00"]);
        assert_eq!(t("nn-NO", json!({"Type": "WeeklyTrigger", "DayOfWeek": 0, "TimeOfDayTicks": 0})), ["kvar sundag kl. 00:00"]);
        assert_eq!(t("nb-NO", json!({"Type": "DailyTrigger", "TimeOfDayTicks": ticks(3 * 3600)})), ["hver dag kl. 03:00"]);
        assert_eq!(t("nb-NO", json!({"Type": "IntervalTrigger", "IntervalTicks": ticks(3 * 86_400)})), ["hver 3. dag"]);
        assert_eq!(t("nn-NO", json!({"Type": "IntervalTrigger", "IntervalTicks": ticks(900)})), ["kvart 15. minutt"]);
        assert_eq!(t("en-US", json!({"Type": "StartupTrigger"})), ["when Jellyfin starts"]);
    }

    /// What is kept between reads is shared by every reader, so it holds no words: they are said as it is served.
    #[test]
    fn the_kept_answer_holds_no_words_and_is_worded_when_served() {
        let mut w = Watch::default();
        let raw = job_json(&json!({ "Key": "RefreshLibrary", "Name": "Scan Media Library", "State": "Idle", "Triggers": [{ "Type": "StartupTrigger" }] }), &mut w, 0);
        assert!(raw.get("what").is_none() && raw.get("schedule").is_none(), "{raw}");
        let served = worded(json!({ "jobs": [raw] }));
        let job = &served["jobs"][0];
        assert_eq!(job["schedule"], json!(["when Jellyfin starts"]));
        assert_eq!(job["known"], true);
        assert!(job["what"].as_str().is_some_and(|s| s.contains("FinStats reads your library")), "{job}");
        assert!(job.get("triggers").is_none(), "Jellyfin's triggers are not part of the answer");
    }
}
