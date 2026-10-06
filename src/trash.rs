//! The trash: what a person deletes — a play, a backup file — is kept for [`KEEP_S`] so it can come back, and is then
//! removed for good by [`purge`], on the housekeeping beat. A play in the trash is out of every reading of history
//! (`visible_playbacks`) and still there for identity (the table), so an import or a restore does not bring it back
//! as a duplicate while it waits.

use anyhow::Result;

use crate::db::rusqlite::{Connection, OptionalExtension, params};

/// How long a deleted play or backup waits in the trash before it is removed for good: long enough to notice a
/// mistake after a holiday or a month's break from the server, short enough that "deleted" soon means it.
pub const KEEP_S: i64 = 30 * 86_400;

/// A play's place in the trash after it was asked to move, and what it was, for the answer and the audit log.
#[derive(Debug)]
pub struct PlayState {
    pub title: String,
    pub user: String,
    pub started_at: i64,
    /// `None`: in history.
    pub deleted_at: Option<i64>,
    /// Whether this call moved it: asking for the state it is already in changes nothing.
    pub changed: bool,
}

impl PlayState {
    /// When the trash lets go of it for good.
    pub fn purge_at(&self) -> Option<i64> {
        self.deleted_at.map(|at| at + KEEP_S)
    }
}

/// Move a finished play into the trash, or out of it. `None` when there is nothing to move: no such play, one still
/// running, or — to come back — one that is not in the trash (or has been purged). Its title is grouped again either
/// way: whoever it was watched with watched alone without it, and in company again with it.
pub fn set_play_deleted(c: &mut Connection, id: i64, deleted: bool, now: i64, group_window_s: i64) -> Result<Option<PlayState>> {
    let row: Option<(String, String, String, i64, Option<i64>)> = c
        .query_row("SELECT item_id, item_name, user_name, started_at, deleted_at FROM playbacks WHERE id = ?1 AND active = 0", [id], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
        })
        .optional()?;
    let Some((item, title, user, started_at, was)) = row else { return Ok(None) };
    if !deleted && was.is_none() {
        return Ok(None);
    }
    let state = |deleted_at, changed| PlayState { title: title.clone(), user: user.clone(), started_at, deleted_at, changed };
    if deleted == was.is_some() {
        return Ok(Some(state(was, false)));
    }
    let deleted_at = deleted.then_some(now);
    // In the trash a play is in no group; the title's groups are worked out again without it, or with it once more.
    c.execute("UPDATE playbacks SET deleted_at = ?2, group_id = NULL WHERE id = ?1", params![id, deleted_at])?;
    crate::groups::detect(c, group_window_s, Some(&item))?;
    Ok(Some(state(deleted_at, true)))
}

/// What the purge removes: plays that have waited their thirty days. Read through the trash's own index.
const PURGE_PLAYS_SQL: &str = "DELETE FROM playbacks WHERE deleted_at IS NOT NULL AND deleted_at <= ?1";

/// Remove for good the plays that have been in the trash for [`KEEP_S`]; their timelines go with them (`ON DELETE
/// CASCADE`). How many went.
pub fn purge_plays(c: &Connection, now: i64) -> Result<usize> {
    Ok(c.execute(PURGE_PLAYS_SQL, [now - KEEP_S])?)
}

/// Empty what has waited [`KEEP_S`]: plays and backup files. On the housekeeping beat, not a job: it has nothing to
/// schedule and nearly always nothing to do. Audited once, by counts, when anything went.
pub async fn purge(app: &crate::state::App) {
    let now = crate::db::now();
    let backups = crate::backup::dir(&app.data_dir);
    let files = tokio::task::spawn_blocking(move || crate::backup::purge_trashed(&backups, now)).await.unwrap_or(0);
    let done = app
        .db
        .call(move |c| {
            let plays = purge_plays(c, now)?;
            if plays + files > 0 {
                crate::audit::record_quietly(c, &crate::audit::Entry::new("trash_purged", crate::audit::Actor::default()).detail(serde_json::json!({ "plays": plays, "backups": files })));
            }
            Ok(plays)
        })
        .await;
    match done {
        Ok(plays) if plays + files > 0 => tracing::info!("emptied the trash: {plays} plays and {files} backups waited {} days", KEEP_S / 86_400),
        Ok(_) => {}
        Err(e) => tracing::warn!("could not empty the trash: {e:#}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Db;

    const DAY: i64 = 86_400;

    fn two_watching_together() -> Db {
        let db = Db::open_in_memory().unwrap();
        let mut c = db.conn().unwrap();
        c.execute_batch(
            "INSERT INTO playbacks(id, source, user_id, user_name, item_id, item_name, item_type, started_at, ended_at, duration_s) VALUES
               (5, 'live', 'u1', 'alice', 'i1', 'Big Buck Bunny', 'Movie', 1000, 4600, 3600),
               (7, 'live', 'u2', 'bob',   'i1', 'Big Buck Bunny', 'Movie', 1010, 4610, 3600);
             INSERT INTO playback_events(playback_id, at, kind, position_s) VALUES (5, 1000, 'start', 0), (5, 2000, 'pause', 1000);",
        )
        .unwrap();
        crate::groups::detect(&mut c, 60, None).unwrap();
        drop(c);
        db
    }
    fn group(c: &Connection, id: i64) -> Option<i64> {
        c.query_row("SELECT group_id FROM playbacks WHERE id = ?1", [id], |r| r.get(0)).unwrap()
    }
    fn visible(c: &Connection) -> i64 {
        c.query_row("SELECT COUNT(*) FROM visible_playbacks", [], |r| r.get(0)).unwrap()
    }

    /// A play deleted out of a group takes the group with it when the one left is alone, and an undo brings both back:
    /// the play into history and the evening back into company.
    #[test]
    fn a_play_moved_to_the_trash_leaves_its_group_and_comes_back_into_it() {
        let db = two_watching_together();
        let mut c = db.conn().unwrap();
        assert_eq!(group(&c, 7), Some(5));
        let gone = set_play_deleted(&mut c, 5, true, 50_000, 60).unwrap().expect("a finished play is deleted");
        assert_eq!((gone.title.as_str(), gone.user.as_str(), gone.started_at, gone.deleted_at, gone.changed), ("Big Buck Bunny", "alice", 1000, Some(50_000), true));
        assert_eq!(gone.purge_at(), Some(50_000 + KEEP_S));
        assert_eq!((visible(&c), group(&c, 7), group(&c, 5)), (1, None, None), "bob watched alone after all, and the play in the trash is in no group");
        // Asked again, nothing changes: the trash keeps the first moment, which is what the 30 days count from.
        let again = set_play_deleted(&mut c, 5, true, 60_000, 60).unwrap().unwrap();
        assert_eq!((again.deleted_at, again.changed), (Some(50_000), false));
        let back = set_play_deleted(&mut c, 5, false, 70_000, 60).unwrap().expect("a play in the trash comes back");
        assert_eq!((back.deleted_at, back.changed, back.purge_at()), (None, true, None));
        assert_eq!((visible(&c), group(&c, 7), group(&c, 5)), (2, Some(5), Some(5)), "the evening is together again");
        assert!(set_play_deleted(&mut c, 5, false, 80_000, 60).unwrap().is_none(), "a play that is not in the trash is not restored");
        assert!(set_play_deleted(&mut c, 99, true, 80_000, 60).unwrap().is_none(), "a play that never was");
    }

    #[test]
    fn a_play_still_running_cannot_be_deleted() {
        let db = two_watching_together();
        let mut c = db.conn().unwrap();
        c.execute("UPDATE playbacks SET active = 1 WHERE id = 7", []).unwrap();
        assert!(set_play_deleted(&mut c, 7, true, 50_000, 60).unwrap().is_none());
        assert_eq!(visible(&c), 2);
    }

    /// Thirty days to change one's mind, to the second: at 29 days nothing goes; at 30 the play goes, its timeline
    /// with it, and the purge says how many.
    #[test]
    fn the_trash_is_emptied_after_thirty_days_and_not_a_day_before() {
        let db = two_watching_together();
        let mut c = db.conn().unwrap();
        set_play_deleted(&mut c, 5, true, 100 * DAY, 60).unwrap();
        assert_eq!(purge_plays(&c, 129 * DAY).unwrap(), 0, "a day before the thirty were up");
        assert_eq!(c.query_row("SELECT COUNT(*) FROM playback_events WHERE playback_id = 5", [], |r| r.get::<_, i64>(0)).unwrap(), 2);
        assert_eq!(purge_plays(&c, 130 * DAY).unwrap(), 1);
        assert_eq!(c.query_row("SELECT COUNT(*) FROM playbacks WHERE id = 5", [], |r| r.get::<_, i64>(0)).unwrap(), 0, "the play is still there");
        assert_eq!(c.query_row("SELECT COUNT(*) FROM playback_events WHERE playback_id = 5", [], |r| r.get::<_, i64>(0)).unwrap(), 0, "its timeline was left behind");
        assert_eq!(purge_plays(&c, 200 * DAY).unwrap(), 0, "a visible play is never purged");
        assert_eq!(visible(&c), 1);
    }

    /// The purge runs every quarter of an hour on every install, nearly always over an empty trash: it is one look at
    /// the trash's own index, never a pass over the history.
    #[test]
    fn purging_an_empty_trash_reads_only_the_trash() {
        let db = two_watching_together();
        let c = db.conn().unwrap();
        let plan: Vec<String> = c
            .prepare(&format!("EXPLAIN QUERY PLAN {PURGE_PLAYS_SQL}"))
            .unwrap()
            .query_map([1_000_000_000i64], |r| r.get::<_, String>(3))
            .unwrap()
            .map(Result::unwrap)
            .collect();
        assert!(plan.iter().any(|p| p.contains("idx_playbacks_trash")) && !plan.iter().any(|p| p.starts_with("SCAN playbacks")), "{plan:?}");
    }
}
