//! Upgrade steps: what a version needs done once to a database an older FinStats wrote, beyond its schema. A migration
//! changes tables; a step is the rest (a job made due so something new is worked out at the first start, a value filled
//! in). Each belongs to the version that brought it and runs once, oldest version first, so a database coming from far
//! back takes 2.3's steps before 2.4's. Each is recorded when it has run, like a migration, rather than judged by the
//! version that last opened the database: a database already opened by a build of a version before that version's step
//! existed (a development build, or a step added in a patch) still gets it.

use anyhow::{Context, Result};

use crate::db::rusqlite::Connection;
use crate::db::{get_setting, set_setting};

/// One step: the version it belongs to, a name that is never reused, why it exists, and what it does.
pub struct Step {
    pub version: &'static str,
    pub name: &'static str,
    #[allow(dead_code)] // read by people, in the code and in the log
    pub why: &'static str,
    pub run: fn(&Connection) -> Result<()>,
}

/// Every step there is, in any order: they are run by version.
pub const STEPS: &[Step] = &[Step {
    version: "2.3.0",
    name: "capacity-first-look",
    why: "Capacity is worked out after a read of the server's details; forgetting when they were last read makes that read, \
          and Capacity, come at the first start instead of up to a quarter of an hour later.",
    run: |c| {
        c.execute("DELETE FROM task_runs WHERE task = 'sync_server'", [])?;
        Ok(())
    },
}, Step {
    version: "2.3.1",
    name: "library-access-first-read",
    why: "FinStats follows Jellyfin's library access, which the users read brings, and until it has run nobody but an \
          administrator sees a library; forgetting when the users were last read makes that read come at the first start.",
    run: |c| {
        c.execute("DELETE FROM task_runs WHERE task = 'sync_users'", [])?;
        Ok(())
    },
}];

/// Settings key: the names of the steps this database has had.
const DONE_KEY: &str = "upgrade_steps";

/// Run what this database has not had of `steps`, up to and including the version `running`, oldest version first, each
/// in a transaction of its own with its record. A brand-new database (`fresh`) is made as this version makes them, so it
/// is only recorded as having had them. Answers the names that ran.
pub fn run(conn: &mut Connection, steps: &[Step], fresh: bool, running: &str) -> Result<Vec<&'static str>> {
    let version = |v: &str| crate::db::parse_version(v).unwrap_or((u64::MAX, 0, 0));
    let mut due: Vec<&Step> = steps.iter().filter(|s| version(s.version) <= version(running)).collect();
    due.sort_by_key(|s| version(s.version));
    let mut done: Vec<String> = get_setting(conn, DONE_KEY)?.and_then(|raw| serde_json::from_str(&raw).ok()).unwrap_or_default();
    let mut ran = vec![];
    for step in due {
        if done.iter().any(|d| d == step.name) {
            continue;
        }
        let tx = conn.transaction()?;
        if !fresh {
            (step.run)(&tx).with_context(|| format!("the upgrade step \"{}\" of FinStats {} failed; nothing of it was kept", step.name, step.version))?;
        }
        done.push(step.name.to_string());
        set_setting(&tx, DONE_KEY, &serde_json::to_string(&done)?)?;
        tx.commit()?;
        if !fresh {
            tracing::info!("applied the upgrade step \"{}\" of FinStats {}", step.name, step.version);
            ran.push(step.name);
        }
    }
    Ok(ran)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // What ran, per test (tests run side by side, so each has its own list).
    static RAN: Mutex<Vec<&'static str>> = Mutex::new(vec![]);
    static RAN_FRESH: Mutex<Vec<&'static str>> = Mutex::new(vec![]);

    fn note(name: &'static str) {
        RAN.lock().unwrap().push(name);
    }

    fn conn() -> Connection {
        let c = Connection::open_in_memory().unwrap();
        for m in crate::db::MIGRATIONS {
            c.execute_batch(m).unwrap();
        }
        c
    }

    // Listed out of order on purpose: 2.4's is first in the list and must still run after 2.3's.
    const LISTED: &[Step] = &[
        Step { version: "2.4.0", name: "twenty-four", why: "test", run: |_| { note("twenty-four"); Ok(()) } },
        Step { version: "2.3.0", name: "twenty-three", why: "test", run: |c| { note("twenty-three"); c.execute("INSERT INTO settings(key, value) VALUES ('step-was-here', '1')", [])?; Ok(()) } },
        Step { version: "2.5.0", name: "twenty-five", why: "test", run: |_| { note("twenty-five"); Ok(()) } },
    ];

    #[test]
    fn steps_run_oldest_version_first_once_each_and_none_from_a_newer_version() {
        let mut c = conn();
        assert_eq!(run(&mut c, LISTED, false, "2.4.0").unwrap(), ["twenty-three", "twenty-four"]);
        assert_eq!(*RAN.lock().unwrap(), ["twenty-three", "twenty-four"], "2.3 before 2.4, and nothing of 2.5");
        assert_eq!(get_setting(&c, "step-was-here").unwrap().as_deref(), Some("1"));
        assert!(run(&mut c, LISTED, false, "2.4.0").unwrap().is_empty(), "a step runs once");
        assert_eq!(run(&mut c, LISTED, false, "2.5.0").unwrap(), ["twenty-five"], "and the next version's comes with it");
    }

    #[test]
    fn a_new_database_is_recorded_as_having_had_every_step_and_runs_none() {
        const STEPS: &[Step] = &[
            Step { version: "2.3.0", name: "a", why: "test", run: |_| { RAN_FRESH.lock().unwrap().push("a"); Ok(()) } },
            Step { version: "2.5.0", name: "b", why: "test", run: |_| { RAN_FRESH.lock().unwrap().push("b"); Ok(()) } },
        ];
        let mut c = conn();
        assert!(run(&mut c, STEPS, true, "2.4.0").unwrap().is_empty());
        assert!(RAN_FRESH.lock().unwrap().is_empty(), "nothing to upgrade on a database made just now");
        assert_eq!(run(&mut c, STEPS, false, "2.5.0").unwrap(), ["b"], "but a later version's step is still due");
        assert_eq!(*RAN_FRESH.lock().unwrap(), ["b"]);
    }

    static RAN_OPEN: Mutex<Vec<&'static str>> = Mutex::new(vec![]);

    /// Through `Db::open` on a real file: a new database has no use for a step, a database opened before a step existed
    /// gets it at its next start.
    #[test]
    fn opening_a_database_runs_the_steps_it_has_not_had() {
        const FIRST: &[Step] = &[Step { version: "0.0.1", name: "first", why: "test", run: |_| { RAN_OPEN.lock().unwrap().push("first"); Ok(()) } }];
        const LATER: &[Step] = &[
            Step { version: "0.0.1", name: "first", why: "test", run: |_| { RAN_OPEN.lock().unwrap().push("first"); Ok(()) } },
            Step { version: "0.0.2", name: "later", why: "test", run: |_| { RAN_OPEN.lock().unwrap().push("later"); Ok(()) } },
        ];
        let dir = std::env::temp_dir().join(format!("finstats-upgrade-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("finstats.db");
        drop(crate::db::Db::open_with(&path, FIRST).unwrap());
        assert!(RAN_OPEN.lock().unwrap().is_empty(), "a database made just now");
        drop(crate::db::Db::open_with(&path, LATER).unwrap());
        assert_eq!(*RAN_OPEN.lock().unwrap(), ["later"], "only the step it had not had");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_step_that_fails_is_not_recorded_and_runs_again_at_the_next_start() {
        const BREAKS: &[Step] = &[
            Step { version: "2.3.0", name: "fine", why: "test", run: |_| Ok(()) },
            Step { version: "2.4.0", name: "breaks", why: "test", run: |c| { c.execute("INSERT INTO settings(key, value) VALUES ('half-done', '1')", [])?; anyhow::bail!("no room") } },
        ];
        let mut c = conn();
        let err = run(&mut c, BREAKS, false, "2.4.0").unwrap_err().to_string();
        assert!(err.contains("breaks") && err.contains("2.4.0"), "{err}");
        assert_eq!(get_setting(&c, "half-done").unwrap(), None, "what a failed step began is undone with it");
        assert_eq!(run(&mut c, &BREAKS[..1], false, "2.4.0").unwrap(), Vec::<&str>::new(), "the one before it stays done");
        assert!(run(&mut c, BREAKS, false, "2.4.0").is_err(), "and the failed one is tried again");
    }

    /// 2.3.0 brings Capacity, worked out after a read of the server's details: the first start reads them at once rather
    /// than up to a quarter of an hour later, by forgetting when they were last read.
    #[test]
    fn the_first_start_of_2_3_0_reads_the_server_at_once() {
        let mut c = conn();
        c.execute_batch("INSERT INTO task_runs(task, state, started_at, finished_at) VALUES ('sync_server', 'ok', 1, 2), ('sync_users', 'ok', 1, 2);").unwrap();
        assert_eq!(run(&mut c, STEPS, false, "2.3.0").unwrap(), ["capacity-first-look"]);
        let left: Vec<String> = c.prepare("SELECT task FROM task_runs ORDER BY task").unwrap().query_map([], |r| r.get(0)).unwrap().map(Result::unwrap).collect();
        assert_eq!(left, ["sync_users"], "only the server read is due");
    }

    /// 2.3.1 follows Jellyfin's library access, which a users read brings: until one has run, nobody but an administrator
    /// sees any library, so the first start reads the users at once.
    #[test]
    fn the_first_start_of_2_3_1_reads_the_users_at_once() {
        let mut c = conn();
        run(&mut c, STEPS, false, "2.3.0").unwrap();
        c.execute_batch("INSERT INTO task_runs(task, state, started_at, finished_at) VALUES ('sync_server', 'ok', 1, 2), ('sync_users', 'ok', 1, 2);").unwrap();
        assert_eq!(run(&mut c, STEPS, false, "2.3.1").unwrap(), ["library-access-first-read"]);
        let left: Vec<String> = c.prepare("SELECT task FROM task_runs ORDER BY task").unwrap().query_map([], |r| r.get(0)).unwrap().map(Result::unwrap).collect();
        assert_eq!(left, ["sync_server"], "only the users read is due");
    }
}
