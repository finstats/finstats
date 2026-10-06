//! When finstats' own jobs run. Every job that can run by itself carries a list of triggers, the way Jellyfin's
//! scheduled tasks do: daily at a time, weekly on a day at a time, on an interval, at start-up — and one Jellyfin has
//! no need for, after Jellyfin's own library scan, because a library read in the middle of one is half old and half
//! new. A trigger may carry a time limit; a run that outlasts it is stopped and reported as failed.
//!
//! The owner's triggers live in `Settings::schedules`, one list per job. A job with no list there runs on its
//! defaults, which are what the settings before 2.0.4 said (`defaults`): following Jellyfin's scan, a backup every
//! so many days, the monthly GeoIP download. Those settings are no longer shown; they only decide the defaults of
//! a job nobody has scheduled yet.
//!
//! Times of day are in finstats' own time zone (`TZ`), the one every chart is drawn in.

use chrono::{Datelike, NaiveDate, NaiveTime, TimeZone};
use serde::{Deserialize, Serialize};

use anyhow::Result;

use crate::db::rusqlite::{Connection, params};
use crate::state::Settings;

/// What sets a job off.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum When {
    /// Every day at this minute of the day.
    Daily { at_min: u32 },
    /// Every week on this day (0 = Sunday, as Jellyfin counts) at this minute of the day.
    Weekly { day: u32, at_min: u32 },
    /// This long after the job last finished.
    Interval { every_s: i64 },
    /// Once, when finstats starts.
    Startup,
    /// When Jellyfin's "Scan Media Library" has finished since the job last started.
    AfterScan,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Trigger {
    #[serde(flatten)]
    pub when: When,
    /// Stop a run this trigger started once it has taken this long.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit_s: Option<i64>,
}

impl Trigger {
    pub fn new(when: When) -> Self {
        Trigger { when, limit_s: None }
    }
}

/// The jobs that can be scheduled. Imports and restores are not among them: each needs a file somebody uploads.
pub const SCHEDULED: [&str; 11] = [
    "sync_users", "sync_libraries", "sync_changes", "sync_events", "sync_server", "sync_userdata",
    "sync_upcoming", "sync_requests", "sync_grabs", "backup", "geoip",
];

/// The jobs "after Jellyfin's library scan" means something to: the ones that read the library.
pub const AFTER_SCAN: [&str; 3] = ["sync_libraries", "sync_userdata", "sync_changes"];

/// Jobs that run on the blocking pool and cannot be stopped half way, so a time limit would be a promise not kept.
const NO_LIMIT: [&str; 2] = ["backup", "geoip"];

/// Whether a run of this job can be stopped at a time limit.
pub fn takes_limit(task: &str) -> bool {
    SCHEDULED.contains(&task) && !NO_LIMIT.contains(&task)
}

pub const MAX_TRIGGERS: usize = 16;
pub const MIN_INTERVAL_S: i64 = 300;
pub const MAX_INTERVAL_S: i64 = 30 * 86_400;
pub const MAX_LIMIT_S: i64 = 7 * 86_400;

/// A job's triggers when nobody has set any: what the settings before 2.0.4 said.
pub fn defaults(task: &str, s: &Settings) -> Vec<Trigger> {
    let every = |every_s: i64| vec![Trigger::new(When::Interval { every_s })];
    match task {
        "sync_libraries" | "sync_userdata" if s.follow_jellyfin_scan => vec![Trigger::new(When::AfterScan), Trigger::new(When::Interval { every_s: 7 * 86_400 })],
        "sync_libraries" | "sync_userdata" => every(s.sync_interval_h.clamp(1, 168) * 3600),
        "sync_users" | "sync_events" | "sync_server" | "sync_upcoming" | "sync_grabs" => every(900),
        "sync_requests" => every(300),
        "sync_changes" => every(3600),
        "backup" if s.backup_every_d > 0 => every(s.backup_every_d.min(30) * 86_400),
        "geoip" if s.geoip_download => every(86_400),
        _ => vec![],
    }
}

/// The triggers a job runs on: the owner's, else its defaults.
pub fn effective(task: &str, s: &Settings) -> Vec<Trigger> {
    s.schedules.get(task).cloned().unwrap_or_else(|| defaults(task, s))
}

/// Whether this list may be stored for this job.
pub fn validate(task: &str, triggers: &[Trigger]) -> Result<(), String> {
    if !SCHEDULED.contains(&task) {
        return Err(format!("“{task}” cannot be scheduled"));
    }
    if triggers.len() > MAX_TRIGGERS {
        return Err(format!("At most {MAX_TRIGGERS} triggers per task"));
    }
    for (i, t) in triggers.iter().enumerate() {
        if triggers[..i].contains(t) {
            return Err("The same trigger is there twice".into());
        }
        match t.when {
            When::Daily { at_min } | When::Weekly { at_min, .. } if at_min >= 1440 => return Err("A time of day is between 00:00 and 23:59".into()),
            When::Weekly { day, .. } if day > 6 => return Err("A day of the week is 0 (Sunday) to 6 (Saturday)".into()),
            When::Interval { every_s } if !(MIN_INTERVAL_S..=MAX_INTERVAL_S).contains(&every_s) => return Err("An interval is between 5 minutes and 30 days".into()),
            When::AfterScan if !AFTER_SCAN.contains(&task) => return Err("Only the jobs that read the library can follow Jellyfin’s scan".into()),
            _ => {}
        }
        match t.limit_s {
            Some(_) if NO_LIMIT.contains(&task) => return Err("This job cannot be stopped half way, so it takes no time limit".into()),
            Some(limit) if !(60..=MAX_LIMIT_S).contains(&limit) => return Err("A time limit is between 1 minute and 168 hours".into()),
            _ => {}
        }
    }
    Ok(())
}

/// What the scheduler knows at one look: the stretch of time since its last look, whether this is the first look
/// after start-up, when the job last started and finished, and when Jellyfin last finished a library scan (only
/// while no scan is running).
#[derive(Clone, Copy, Debug, Default)]
pub struct Moment {
    pub from: i64,
    pub to: i64,
    pub startup: bool,
    pub last_start: Option<i64>,
    pub last_end: Option<i64>,
    pub scan_done: Option<i64>,
}

/// Whether a trigger sets its job off at this look.
pub fn fires<Tz: TimeZone>(t: &Trigger, m: &Moment, tz: &Tz) -> bool {
    match t.when {
        When::Interval { every_s } => m.last_end.is_none_or(|end| m.to - end >= every_s),
        When::Startup => m.startup,
        When::AfterScan => m.scan_done.is_some_and(|done| m.last_start.is_none_or(|start| done > start)),
        When::Daily { .. } | When::Weekly { .. } => occurrences(&t.when, m.from, m.to, tz).next().is_some(),
    }
}

/// The moments a daily or weekly trigger names in (from, to], a look being a minute or so long.
fn occurrences<'a, Tz: TimeZone>(when: &'a When, from: i64, to: i64, tz: &'a Tz) -> impl Iterator<Item = i64> + 'a {
    let first = tz.timestamp_opt(from, 0).single().map(|d| d.date_naive()).unwrap_or_default();
    let days = (to - from).clamp(0, 400 * 86_400) / 86_400 + 2;
    (0..days).filter_map(move |n| {
        let date = first.checked_add_days(chrono::Days::new(n as u64))?;
        let at_min = match *when {
            When::Daily { at_min } => at_min,
            When::Weekly { day, at_min } if date.weekday().num_days_from_sunday() == day => at_min,
            _ => return None,
        };
        local_moment(date, at_min, tz)
    }).filter(move |&at| at > from && at <= to)
}

/// When the next of these triggers will set the job off, where that can be known in advance.
pub fn next_at<Tz: TimeZone>(triggers: &[Trigger], now: i64, last_end: Option<i64>, tz: &Tz) -> Option<i64> {
    triggers
        .iter()
        .filter_map(|t| match t.when {
            When::Interval { every_s } => Some(last_end.map_or(now, |end| (end + every_s).max(now))),
            When::Daily { .. } | When::Weekly { .. } => occurrences(&t.when, now, now + 8 * 86_400, tz).next(),
            When::Startup | When::AfterScan => None,
        })
        .min()
}

/// A minute of a local day as a moment. A time the clocks skip in spring is taken an hour later, and one they
/// pass twice in autumn the first time.
fn local_moment<Tz: TimeZone>(date: NaiveDate, at_min: u32, tz: &Tz) -> Option<i64> {
    let time = NaiveTime::from_num_seconds_from_midnight_opt(at_min * 60, 0)?;
    let local = date.and_time(time);
    tz.from_local_datetime(&local)
        .earliest()
        .or_else(|| tz.from_local_datetime(&(local + chrono::Duration::hours(1))).earliest())
        .map(|d| d.timestamp())
}

/// One look of the scheduler: the stretch since the last one, whether it is the first after start-up, and when
/// Jellyfin last finished a library scan (`None` while one is running, or when that is not known).
#[derive(Clone, Copy, Debug, Default)]
pub struct Look {
    pub from: i64,
    pub to: i64,
    pub startup: bool,
    pub scan_done: Option<i64>,
}

/// The jobs one look sets off, each with the time limit of the trigger that did. A job already running is not due.
pub fn due<Tz: TimeZone>(s: &Settings, snapshot: &[crate::state::TaskState], look: &Look, tz: &Tz) -> Vec<(&'static str, Option<i64>)> {
    SCHEDULED
        .into_iter()
        .filter_map(|task| {
            let run = snapshot.iter().find(|t| t.id == task);
            if run.is_some_and(|t| t.state == "running") {
                return None;
            }
            let m = Moment { from: look.from, to: look.to, startup: look.startup, last_start: run.and_then(|t| t.started_at), last_end: run.and_then(|t| t.finished_at), scan_done: look.scan_done };
            effective(task, s).into_iter().find(|t| fires(t, &m, tz)).map(|t| (task, t.limit_s))
        })
        .collect()
}

/// What was known of the library read and the backups before their runs were kept: the last library read
/// (`library_synced_at`) and the newest backup file. Without them, the first start of 2.0.4 would read the whole
/// library and write a backup at once.
pub fn seed(mut runs: Vec<LastRun>, library_read: Option<i64>, newest_backup: Option<i64>) -> Vec<LastRun> {
    let known = [("sync_libraries", library_read), ("sync_userdata", library_read), ("backup", newest_backup)];
    for (task, at) in known {
        if let Some(at) = at.filter(|_| !runs.iter().any(|r| r.task == task)) {
            runs.push(LastRun { task: task.into(), ok: true, started_at: Some(at), finished_at: Some(at), message: None, error: None });
        }
    }
    runs
}

/// A run under the time limit of the trigger that started it: past the limit it is dropped where it stands and
/// reported as failed. Every job that takes a limit writes nothing destructive until it has read everything, so a
/// run stopped half way leaves what was there.
pub async fn within<T>(limit: Option<std::time::Duration>, run: impl std::future::Future<Output = Result<T>>) -> Result<T> {
    let Some(limit) = limit else { return run.await };
    match tokio::time::timeout(limit, run).await {
        Ok(outcome) => outcome,
        Err(_) => Err(anyhow::anyhow!("Stopped: it ran past its time limit of {}", human(limit.as_secs() as i64))),
    }
}

/// "2 hours", "90 minutes": a limit as the owner set it.
fn human(s: i64) -> String {
    match s {
        s if s % 3600 == 0 => format!("{} hour{}", s / 3600, if s == 3600 { "" } else { "s" }),
        s => format!("{} minute{}", s / 60, if s / 60 == 1 { "" } else { "s" }),
    }
}

/// A job's last finished run, as `task_runs` keeps it.
#[derive(Clone, Debug, PartialEq)]
pub struct LastRun {
    pub task: String,
    pub ok: bool,
    pub started_at: Option<i64>,
    pub finished_at: Option<i64>,
    pub message: Option<String>,
    pub error: Option<String>,
}

/// Keeps the finished runs in `snapshot` that are newer than what is kept; a running job keeps its last finished run.
pub fn save_runs(c: &Connection, snapshot: &[crate::state::TaskState]) -> Result<usize> {
    let mut stmt = c.prepare_cached(
        "INSERT INTO task_runs(task, state, started_at, finished_at, message, error) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(task) DO UPDATE SET state = excluded.state, started_at = excluded.started_at, finished_at = excluded.finished_at,
            message = excluded.message, error = excluded.error
         WHERE task_runs.started_at IS NOT excluded.started_at OR task_runs.finished_at IS NOT excluded.finished_at",
    )?;
    let mut written = 0;
    for t in snapshot.iter().filter(|t| matches!(t.state, "ok" | "error") && t.finished_at.is_some()) {
        written += stmt.execute(params![t.id, t.state, t.started_at, t.finished_at, t.message, t.error])?;
    }
    Ok(written)
}

pub fn load_runs(c: &Connection) -> Result<Vec<LastRun>> {
    let mut stmt = c.prepare("SELECT task, state, started_at, finished_at, message, error FROM task_runs")?;
    let rows = stmt.query_map([], |r| {
        Ok(LastRun { task: r.get(0)?, ok: r.get::<_, String>(1)? == "ok", started_at: r.get(2)?, finished_at: r.get(3)?, message: r.get(4)?, error: r.get(5)? })
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_time_limit_is_said_as_the_owner_set_it() {
        for (s, said) in [(60, "1 minute"), (90 * 60, "90 minutes"), (3600, "1 hour"), (7200, "2 hours"), (5400, "90 minutes")] {
            assert_eq!(human(s), said, "{s} s");
        }
    }
    use chrono::FixedOffset;

    fn snapshot_with(done: &[(&'static str, i64, i64)], running: &[&'static str]) -> Vec<crate::state::TaskState> {
        let tasks = crate::state::Tasks::new();
        tasks.remember(done.iter().map(|&(task, started, finished)| LastRun { task: task.into(), ok: true, started_at: Some(started), finished_at: Some(finished), message: None, error: None }).collect());
        for id in running {
            tasks.try_start(id, "Starting…");
        }
        tasks.snapshot()
    }

    #[test]
    fn the_first_look_runs_what_never_ran_and_nothing_a_setting_left_off() {
        let now = at(29, 12, 0);
        let look = Look { from: now, to: now, startup: true, scan_done: None };
        let due: Vec<_> = due(&Settings::default(), &snapshot_with(&[], &[]), &look, &oslo()).into_iter().map(|(t, _)| t).collect();
        for t in ["sync_users", "sync_events", "sync_server", "sync_changes", "sync_libraries", "sync_userdata", "backup", "sync_upcoming", "sync_requests", "sync_grabs"] {
            assert!(due.contains(&t), "{t} never ran and is not due: {due:?}");
        }
        assert!(!due.contains(&"geoip"), "the download was never switched on");
        assert!(!due.contains(&"import"));
    }

    #[test]
    fn a_job_is_due_by_its_own_triggers_and_never_while_it_runs() {
        let now = at(29, 12, 0);
        let look = Look { from: now - 60, to: now, startup: false, scan_done: Some(now - 30) };
        let recent = [("sync_users", now - 120, now - 100), ("sync_libraries", now - 3600, now - 3000), ("sync_userdata", now - 3600, now - 3000)];
        let due = due(&Settings::default(), &snapshot_with(&recent, &["sync_events"]), &look, &oslo());
        let ids: Vec<_> = due.iter().map(|(t, _)| *t).collect();
        assert!(!ids.contains(&"sync_users"), "ran two minutes ago, every 15 minutes");
        assert!(!ids.contains(&"sync_events"), "already running");
        assert!(ids.contains(&"sync_libraries") && ids.contains(&"sync_userdata"), "Jellyfin finished a scan after the last read: {ids:?}");
        let mut s = Settings::default();
        s.schedules.insert("sync_users".into(), vec![Trigger { when: When::Daily { at_min: 12 * 60 }, limit_s: Some(600) }]);
        let limited = super::due(&s, &snapshot_with(&recent, &[]), &look, &oslo());
        assert!(limited.contains(&("sync_users", Some(600))), "the trigger that fired brings its limit: {limited:?}");
    }

    #[test]
    fn the_library_read_and_the_backups_start_from_what_was_known_before() {
        let seeded = seed(vec![], Some(1_000), Some(2_000));
        let get = |t: &str| seeded.iter().find(|r| r.task == t).cloned();
        assert_eq!(get("sync_libraries").map(|r| (r.started_at, r.finished_at)), Some((Some(1_000), Some(1_000))));
        assert_eq!(get("sync_userdata").map(|r| r.finished_at), Some(Some(1_000)), "it was read with the library");
        assert_eq!(get("backup").map(|r| r.finished_at), Some(Some(2_000)));
        // A run that was kept wins over what was known before.
        let kept = LastRun { task: "backup".into(), ok: true, started_at: Some(5_000), finished_at: Some(5_001), message: None, error: None };
        assert_eq!(seed(vec![kept.clone()], Some(1_000), Some(2_000)).into_iter().filter(|r| r.task == "backup").collect::<Vec<_>>(), vec![kept]);
        assert!(seed(vec![], None, None).is_empty());
    }

    /// 2.2.0 stores where a multi-episode file ends, which only a library read brings: the update forgets the last read,
    /// so the next look reads the library once, whatever the read's own interval says.
    #[test]
    fn the_update_that_stores_where_an_episode_ends_reads_the_library_once() {
        let c = Connection::open_in_memory().unwrap();
        let before = crate::db::MIGRATIONS.iter().position(|m| m.contains("index_number_end")).expect("the migration that adds it");
        for m in &crate::db::MIGRATIONS[..before] {
            c.execute_batch(m).unwrap();
        }
        let now = at(29, 12, 0);
        c.execute("INSERT INTO task_runs(task, state, started_at, finished_at) VALUES ('sync_libraries', 'ok', ?1, ?1)", [now - 600]).unwrap();
        crate::db::set_setting(&c, "library_synced_at", &(now - 600).to_string()).unwrap();
        let due_now = |c: &Connection| {
            let library_read = crate::db::get_setting(c, "library_synced_at").unwrap().and_then(|v| v.parse().ok());
            let tasks = crate::state::Tasks::new();
            tasks.remember(seed(load_runs(c).unwrap(), library_read, None));
            let look = Look { from: now - 60, to: now, startup: true, scan_done: None };
            due(&Settings::default(), &tasks.snapshot(), &look, &oslo()).into_iter().any(|(t, _)| t == "sync_libraries")
        };
        assert!(!due_now(&c), "read ten minutes ago");
        c.execute_batch(crate::db::MIGRATIONS[before]).unwrap();
        assert!(due_now(&c), "the update reads the library again");
    }

    #[tokio::test]
    async fn a_run_past_its_time_limit_is_stopped_and_says_so() {
        let slow = async {
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            Ok("done")
        };
        let started = std::time::Instant::now();
        let err = within(Some(std::time::Duration::from_millis(50)), slow).await.unwrap_err();
        assert!(started.elapsed() < std::time::Duration::from_secs(5), "it was not stopped");
        assert!(format!("{err:#}").contains("time limit"), "{err:#}");
        assert_eq!(within(None, async { Ok("done") }).await.unwrap(), "done");
        assert_eq!(within(Some(std::time::Duration::from_secs(5)), async { Ok("done") }).await.unwrap(), "done");
    }

    #[test]
    fn a_finished_run_is_remembered_across_a_restart() {
        let c = Connection::open_in_memory().unwrap();
        for m in crate::db::MIGRATIONS {
            c.execute_batch(m).unwrap();
        }
        let tasks = crate::state::Tasks::new();
        tasks.try_start("sync_users", "Starting…");
        tasks.finish("sync_users", Ok(("12 users".into(), None)));
        tasks.try_start("sync_server", "Starting…");
        tasks.finish("sync_server", Err(anyhow::anyhow!("Jellyfin answered 500")));
        tasks.try_start("sync_events", "Starting…");
        assert_eq!(save_runs(&c, &tasks.snapshot()).unwrap(), 2, "a job still running has nothing to keep yet");
        assert_eq!(save_runs(&c, &tasks.snapshot()).unwrap(), 0, "nothing newer, nothing written");
        // A row a job that no longer exists left behind (`sync_artwork`, from while 2.0.4 was being made) is passed over.
        c.execute("INSERT INTO task_runs(task, state, started_at, finished_at) VALUES ('sync_artwork', 'ok', 1, 2)", []).unwrap();

        let after = crate::state::Tasks::new();
        after.remember(load_runs(&c).unwrap());
        let get = |id: &str| after.snapshot().into_iter().find(|t| t.id == id).unwrap();
        let users = get("sync_users");
        assert_eq!((users.state, users.message.as_deref()), ("ok", Some("12 users")));
        assert!(users.started_at.is_some() && users.finished_at.is_some());
        let server = get("sync_server");
        assert_eq!((server.state, server.error.as_deref()), ("error", Some("Jellyfin answered 500")));
        assert_eq!(get("sync_events").state, "idle");
    }

    fn t(when: When) -> Trigger {
        Trigger::new(when)
    }
    fn oslo() -> FixedOffset {
        FixedOffset::east_opt(2 * 3600).unwrap()
    }
    /// 2026-09-29 is a Tuesday. A moment on it, in the zone above.
    fn at(day: u32, h: u32, m: u32) -> i64 {
        oslo().with_ymd_and_hms(2026, 9, day, h, m, 0).unwrap().timestamp()
    }

    #[test]
    fn the_settings_before_scheduling_become_the_defaults() {
        let s = Settings::default();
        assert_eq!(defaults("sync_libraries", &s), vec![t(When::AfterScan), t(When::Interval { every_s: 7 * 86_400 })],
            "following Jellyfin's scan, with the weekly safety net");
        assert_eq!(defaults("sync_userdata", &s), defaults("sync_libraries", &s), "watched flags were read with the library");
        let timer = Settings { follow_jellyfin_scan: false, sync_interval_h: 6, ..Settings::default() };
        assert_eq!(defaults("sync_libraries", &timer), vec![t(When::Interval { every_s: 6 * 3600 })]);
        assert_eq!(defaults("backup", &s), vec![t(When::Interval { every_s: 7 * 86_400 })]);
        assert_eq!(defaults("backup", &Settings { backup_every_d: 0, ..Settings::default() }), vec![], "backups were off");
        assert_eq!(defaults("geoip", &s), vec![], "the download is off until somebody switches it on");
        assert_eq!(defaults("geoip", &Settings { geoip_download: true, ..Settings::default() }), vec![t(When::Interval { every_s: 86_400 })]);
        assert_eq!(defaults("sync_changes", &s), vec![t(When::Interval { every_s: 3600 })], "the owner's choice: every hour");
        for quarter in ["sync_users", "sync_events", "sync_server", "sync_upcoming", "sync_grabs"] {
            assert_eq!(defaults(quarter, &s), vec![t(When::Interval { every_s: 900 })], "{quarter}");
        }
        assert_eq!(defaults("sync_requests", &s), vec![t(When::Interval { every_s: 300 })]);
        assert_eq!(defaults("import", &s), vec![]);
    }

    #[test]
    fn the_owners_triggers_replace_the_defaults_even_when_there_are_none() {
        let mut s = Settings::default();
        assert_eq!(effective("backup", &s), defaults("backup", &s));
        s.schedules.insert("backup".into(), vec![t(When::Daily { at_min: 180 })]);
        assert_eq!(effective("backup", &s), vec![t(When::Daily { at_min: 180 })]);
        s.schedules.insert("backup".into(), vec![]);
        assert_eq!(effective("backup", &s), vec![], "every trigger removed means: only by hand");
    }

    #[test]
    fn only_what_a_job_can_do_is_accepted() {
        assert!(validate("backup", &[t(When::Daily { at_min: 180 }), t(When::Weekly { day: 0, at_min: 0 }), t(When::Startup)]).is_ok());
        assert!(validate("import", &[t(When::Startup)]).is_err(), "an import needs a file");
        assert!(validate("nonsense", &[]).is_err());
        assert!(validate("sync_libraries", &[t(When::AfterScan)]).is_ok());
        assert!(validate("backup", &[t(When::AfterScan)]).is_err(), "a backup has nothing to do with a scan");
        assert!(validate("backup", &[t(When::Daily { at_min: 1440 })]).is_err());
        assert!(validate("backup", &[t(When::Weekly { day: 7, at_min: 0 })]).is_err());
        assert!(validate("sync_users", &[t(When::Interval { every_s: 60 })]).is_err(), "not more often than every five minutes");
        assert!(validate("sync_users", &[t(When::Interval { every_s: 31 * 86_400 })]).is_err());
        let limited = Trigger { when: When::Interval { every_s: 3600 }, limit_s: Some(2 * 3600) };
        assert!(validate("sync_libraries", std::slice::from_ref(&limited)).is_ok());
        assert!(validate("backup", &[limited]).is_err(), "a backup cannot be stopped half way");
        assert!(validate("sync_libraries", &[Trigger { when: When::Startup, limit_s: Some(0) }]).is_err());
        assert!(validate("sync_users", &[t(When::Startup), t(When::Startup)]).is_err(), "the same trigger twice");
        assert!(validate("sync_users", &(0..17).map(|i| t(When::Daily { at_min: i * 15 })).collect::<Vec<_>>()).is_err(), "more than sixteen");
    }

    #[test]
    fn a_trigger_reads_and_writes_as_jellyfin_shaped_json() {
        let raw = r#"[{"type":"weekly","day":0,"at_min":180,"limit_s":7200},{"type":"after_scan"},{"type":"interval","every_s":900}]"#;
        let list: Vec<Trigger> = serde_json::from_str(raw).unwrap();
        assert_eq!(list[0], Trigger { when: When::Weekly { day: 0, at_min: 180 }, limit_s: Some(7200) });
        assert_eq!(serde_json::to_string(&list).unwrap(), raw);
    }

    #[test]
    fn an_interval_counts_from_the_end_of_the_last_run() {
        let every = t(When::Interval { every_s: 900 });
        let m = |last_end| Moment { from: 0, to: 10_000, last_end, ..Moment::default() };
        assert!(fires(&every, &m(None), &oslo()), "never run: run now");
        assert!(fires(&every, &m(Some(10_000 - 900)), &oslo()));
        assert!(!fires(&every, &m(Some(10_000 - 899)), &oslo()));
    }

    #[test]
    fn a_time_of_day_fires_once_when_it_passes_and_never_for_a_time_missed_while_stopped() {
        let three = t(When::Daily { at_min: 180 });
        let look = |from, to| Moment { from, to, ..Moment::default() };
        assert!(fires(&three, &look(at(29, 2, 59), at(29, 3, 0)), &oslo()));
        assert!(!fires(&three, &look(at(29, 3, 0), at(29, 3, 1)), &oslo()), "the same 03:00 twice");
        assert!(!fires(&three, &look(at(29, 3, 1), at(29, 4, 0)), &oslo()));
        assert!(fires(&three, &look(at(28, 23, 0), at(29, 3, 30)), &oslo()), "a look that spans midnight");
        // The trigger is in finstats' zone, not in UTC: 03:00 in Oslo is 01:00 UTC.
        assert!(!fires(&three, &look(at(29, 4, 59), at(29, 5, 1)), &oslo()));
        let tuesday = t(When::Weekly { day: 2, at_min: 180 });
        let monday = t(When::Weekly { day: 1, at_min: 180 });
        assert!(fires(&tuesday, &look(at(29, 2, 59), at(29, 3, 0)), &oslo()));
        assert!(!fires(&monday, &look(at(29, 2, 59), at(29, 3, 0)), &oslo()));
    }

    #[test]
    fn start_up_and_the_library_scan_fire_on_what_they_are_about() {
        let startup = t(When::Startup);
        assert!(fires(&startup, &Moment { startup: true, ..Moment::default() }, &oslo()));
        assert!(!fires(&startup, &Moment::default(), &oslo()));
        let scan = t(When::AfterScan);
        let m = |last_start, scan_done| Moment { last_start, scan_done, ..Moment::default() };
        assert!(fires(&scan, &m(Some(100), Some(200)), &oslo()), "a scan finished since the last read began");
        assert!(!fires(&scan, &m(Some(300), Some(200)), &oslo()));
        assert!(!fires(&scan, &m(Some(100), None), &oslo()), "no scan finished, or one is running");
        assert!(fires(&scan, &m(None, Some(200)), &oslo()), "never read");
    }

    #[test]
    fn the_next_run_is_the_soonest_that_can_be_known() {
        let now = at(29, 12, 0);
        assert_eq!(next_at(&[t(When::Daily { at_min: 180 })], now, None, &oslo()), Some(at(30, 3, 0)));
        assert_eq!(next_at(&[t(When::Daily { at_min: 13 * 60 })], now, None, &oslo()), Some(at(29, 13, 0)));
        assert_eq!(next_at(&[t(When::Weekly { day: 0, at_min: 0 })], now, None, &oslo()), Some(oslo().with_ymd_and_hms(2026, 10, 4, 0, 0, 0).unwrap().timestamp()));
        assert_eq!(next_at(&[t(When::Interval { every_s: 3600 })], now, Some(now - 600), &oslo()), Some(now + 3000));
        assert_eq!(next_at(&[t(When::Interval { every_s: 3600 }), t(When::Daily { at_min: 12 * 60 + 30 })], now, Some(now - 600), &oslo()), Some(at(29, 12, 30)));
        assert_eq!(next_at(&[t(When::AfterScan), t(When::Startup)], now, None, &oslo()), None, "nobody knows when Jellyfin will scan");
        assert_eq!(next_at(&[], now, None, &oslo()), None);
    }
}
