//! The playback row, as written by both the live collector and the importer.

use anyhow::Result;
use serde_json::Value;

use crate::db::rusqlite::{Connection, named_params, params};
use crate::media::Streams;

/// Where play `p` stopped, in seconds, the one rule every page uses: where it was when it ended, for a play that
/// knows (FinStats' own, and a Streamystats play that kept its runtime), and otherwise how long it ran, the play
/// taken to have started at 0:00 (Jellystat keeps a length, not a place).
pub const STOP_S: &str = "COALESCE(p.position_s, p.duration_s)";

/// Whether that stop was measured rather than worked out from the length.
pub const STOP_MEASURED: &str = "(p.position_s IS NOT NULL)";

/// How far play `p` got through its title `i`, from 0 to 1, by `STOP_S` and the runtime the play or the title has.
pub const PLAY_FRAC: &str = "MIN(1.0, COALESCE(p.position_s, p.duration_s) * 1.0 / NULLIF(COALESCE(p.runtime_s, i.runtime_s), 0))";

#[derive(Debug, Clone, Default)]
pub struct PlayRecord {
    pub source: &'static str,
    pub source_id: Option<String>,
    pub active: bool,
    pub user_id: String,
    pub user_name: String,
    pub item_id: String,
    pub item_name: String,
    pub item_type: String,
    pub series_id: Option<String>,
    pub series_name: Option<String>,
    pub season_id: Option<String>,
    pub season_number: Option<i64>,
    pub episode_number: Option<i64>,
    pub started_at: i64,
    pub ended_at: i64,
    pub duration_s: i64,
    pub paused_s: i64,
    pub position_s: Option<i64>,
    pub runtime_s: Option<i64>,
    pub client: Option<String>,
    pub device_name: Option<String>,
    pub device_id: Option<String>,
    pub app_version: Option<String>,
    pub remote_ip: Option<String>,
    pub play_method: String,
    pub container: Option<String>,
    pub streams: Streams,
    pub transcode: Option<Value>,
    pub pause_count: i64,
    pub seek_count: i64,
    /// Where playback picked up; > 0 means the viewer resumed something.
    pub start_position_s: Option<i64>,
}

const COLUMNS: &str = "source, source_id, active, user_id, user_name, item_id, item_name, item_type,
    series_id, series_name, season_id, season_number, episode_number, library_id,
    started_at, ended_at, duration_s, paused_s, position_s, runtime_s,
    client, device_name, device_id, app_version, remote_ip, play_method, container, bitrate,
    video_codec, width, height, video_range, bit_depth,
    audio_codec, audio_channels, audio_language, subtitle_codec, subtitle_language, transcode,
    pause_count, seek_count, start_position_s, is_local";

const VALUES: &str = ":source, :source_id, :active, :user_id, :user_name, :item_id, :item_name, :item_type,
    :series_id, :series_name, :season_id, :season_number, :episode_number,
    COALESCE((SELECT library_id FROM items WHERE id = :item_id), (SELECT library_id FROM items WHERE id = :series_id)),
    :started_at, :ended_at, :duration_s, :paused_s, :position_s, :runtime_s,
    :client, :device_name, :device_id, :app_version, :remote_ip, :play_method, :container, :bitrate,
    :video_codec, :width, :height, :video_range, :bit_depth,
    :audio_codec, :audio_channels, :audio_language, :subtitle_codec, :subtitle_language, :transcode,
    :pause_count, :seek_count, :start_position_s, :is_local";

impl PlayRecord {
    /// Inserts the row. Returns `None` when `source_id` already exists (duplicate import).
    pub fn insert(&self, conn: &Connection) -> Result<Option<i64>> {
        let sql = format!("INSERT OR IGNORE INTO playbacks ({COLUMNS}) VALUES ({VALUES})");
        let mut stmt = conn.prepare_cached(&sql)?;
        let transcode = self.transcode.as_ref().map(|t| t.to_string());
        let n = stmt.execute(named_params! {
            ":source": self.source,
            ":source_id": self.source_id,
            ":active": self.active,
            ":user_id": self.user_id,
            ":user_name": self.user_name,
            ":item_id": self.item_id,
            ":item_name": self.item_name,
            ":item_type": self.item_type,
            ":series_id": self.series_id,
            ":series_name": self.series_name,
            ":season_id": self.season_id,
            ":season_number": self.season_number,
            ":episode_number": self.episode_number,
            ":started_at": self.started_at,
            ":ended_at": self.ended_at,
            ":duration_s": self.duration_s,
            ":paused_s": self.paused_s,
            ":position_s": self.position_s,
            ":runtime_s": self.runtime_s,
            ":client": self.client,
            ":device_name": self.device_name,
            ":device_id": self.device_id,
            ":app_version": self.app_version,
            ":remote_ip": self.remote_ip,
            ":play_method": self.play_method,
            ":container": self.container,
            ":bitrate": self.streams.bitrate,
            ":video_codec": self.streams.video_codec,
            ":width": self.streams.width,
            ":height": self.streams.height,
            ":video_range": self.streams.video_range,
            ":bit_depth": self.streams.bit_depth,
            ":audio_codec": self.streams.audio_codec,
            ":audio_channels": self.streams.audio_channels,
            ":audio_language": self.streams.audio_language,
            ":subtitle_codec": self.streams.subtitle_codec,
            ":subtitle_language": self.streams.subtitle_language,
            ":transcode": transcode,
            ":pause_count": self.pause_count,
            ":seek_count": self.seek_count,
            ":start_position_s": self.start_position_s,
            ":is_local": match self.remote_ip.as_deref() { Some(ip) => crate::network::classify(conn, ip)?, None => None },
        })?;
        Ok((n > 0).then(|| conn.last_insert_rowid()))
    }

    /// Insert an imported row, unless the history already holds this play. `None` means it did
    /// already: either this very row has been imported before, or another tracker (or the
    /// collector) got there first. See [`already_recorded`].
    pub fn insert_imported(&self, conn: &Connection, merge_window_s: i64) -> Result<Option<i64>> {
        let this = Play {
            source: self.source,
            source_id: self.source_id.as_deref(),
            user_id: &self.user_id,
            item_id: &self.item_id,
            started_at: self.started_at,
            ended_at: self.ended_at,
        };
        if already_recorded(conn, this, merge_window_s)? {
            return Ok(None);
        }
        self.insert(conn)
    }

    /// Refresh the parts of a live row that change while it plays.
    pub fn update_progress(&self, conn: &Connection, row_id: i64) -> Result<()> {
        let transcode = self.transcode.as_ref().map(|t| t.to_string());
        conn.prepare_cached(
            "UPDATE playbacks SET active = :active, ended_at = :ended_at, duration_s = :duration_s, paused_s = :paused_s,
                 position_s = :position_s, play_method = :play_method, remote_ip = :remote_ip, is_local = :is_local,
                 pause_count = :pause_count, seek_count = :seek_count,
                 audio_codec = :audio_codec, audio_channels = :audio_channels, audio_language = :audio_language,
                 subtitle_codec = :subtitle_codec, subtitle_language = :subtitle_language,
                 transcode = COALESCE(:transcode, transcode)
             WHERE id = :id",
        )?
        .execute(named_params! {
            ":id": row_id,
            ":active": self.active,
            ":ended_at": self.ended_at,
            ":duration_s": self.duration_s,
            ":paused_s": self.paused_s,
            ":position_s": self.position_s,
            ":play_method": self.play_method,
            ":remote_ip": self.remote_ip,
            ":is_local": match self.remote_ip.as_deref() { Some(ip) => crate::network::classify(conn, ip)?, None => None },
            ":pause_count": self.pause_count,
            ":seek_count": self.seek_count,
            ":audio_codec": self.streams.audio_codec,
            ":audio_channels": self.streams.audio_channels,
            ":audio_language": self.streams.audio_language,
            ":subtitle_codec": self.streams.subtitle_codec,
            ":subtitle_language": self.streams.subtitle_language,
            ":transcode": transcode,
        })?;
        Ok(())
    }
}

/// The bits of a play that say which play it is.
#[derive(Debug, Clone, Copy)]
pub struct Play<'a> {
    pub source: &'a str,
    pub source_id: Option<&'a str>,
    pub user_id: &'a str,
    pub item_id: &'a str,
    pub started_at: i64,
    pub ended_at: i64,
}

/// Is this play already in the history? The one rule a restore and both importers share, so that
/// the same evening cannot be counted twice however it arrives.
///
/// Its own id settles it when there is one. Otherwise it is the same person, the same item, and one
/// of the play's two ends close enough to be the same viewing, where "close enough" depends on who
/// recorded the other row.
///
/// **Between** sources it is `window` seconds at *either* end, because the trackers disagree about
/// what they record. Jellystat keeps only the moment a play *ended*, so FinStats works the start
/// back from the seconds played, and every minute the viewer spent paused moves that start later;
/// Streamystats and the collector keep the real start. The ends, on the other hand, are the same
/// moment for all three, give or take how quickly each noticed. Measured against real history
/// (3,162 plays a Streamystats export and a Jellystat import held in common), matching either end
/// recognised 2,750 of them, the start alone 2,527, and the offsets of the ones the start missed
/// ran in an unbroken smear out past ten minutes, while the ends fall off a cliff inside one.
///
/// **Within** one source it is the very same second and nothing wider, because a tracker never
/// exports the same play twice: a second row of the same item minutes later is a restart the viewer
/// really made, and a window there would silently drop it.
pub fn already_recorded(conn: &Connection, p: Play<'_>, window: i64) -> Result<bool> {
    if let Some(sid) = p.source_id
        && conn.prepare_cached("SELECT 1 FROM playbacks WHERE source_id = ?1")?.exists([sid])?
    {
        return Ok(true);
    }
    Ok(conn.prepare_cached(SAME_PLAY_SQL)?.exists(params![p.user_id, p.item_id, p.started_at, p.ended_at, p.source, window.max(0)])?)
}

/// The same person, the same item, and the same second, or, from another tracker, either end within the window.
/// Written as three index ranges rather than `ABS(started_at - ?3) <= ?6`, which the index cannot narrow: that form
/// read every play this person ever made of the title, for every row of an import.
const SAME_PLAY_SQL: &str = "
    SELECT 1 FROM playbacks WHERE user_id = ?1 AND item_id = ?2 AND started_at = ?3
    UNION ALL
    SELECT 1 FROM playbacks WHERE user_id = ?1 AND item_id = ?2 AND started_at BETWEEN ?3 - ?6 AND ?3 + ?6 AND source <> ?5
    UNION ALL
    SELECT 1 FROM playbacks WHERE user_id = ?1 AND item_id = ?2 AND ended_at BETWEEN ?4 - ?6 AND ?4 + ?6 AND source <> ?5";

/// Apply [`already_recorded`] again to one title's history that is already written, and take back out what has
/// become a duplicate since. Answers how many rows went.
///
/// Needed because [`crate::relink`] rewrites `item_id`: a row whose item had been renamed in
/// Jellyfin matches nothing when it is imported (no other row carries that old id) and is then
/// pointed at the item that is really there, which is the one an older row from another tracker
/// already points at. So the duplicate appears *after* the rule ran, and the rule has to run again
/// wherever ids are rewritten rather than only where rows are written: for the titles plays were
/// moved onto, the only place one can have appeared.
///
/// Only a row somebody imported is ever removed, and never one FinStats recorded itself: its own
/// row carries a timeline and the counts that go with it, which no import can have. Between two
/// imported rows the one that arrived first stays. Rows of one tracker are never compared with each
/// other: a second row of the same item is a restart the viewer really made.
pub fn drop_relinked_duplicates(conn: &Connection, window: i64, item_id: &str) -> Result<usize> {
    Ok(conn.execute(&format!("{RELINKED_DUPLICATES_SQL} AND item_id = ?2"), crate::db::rusqlite::params![window.max(0), item_id])?)
}

/// [`drop_relinked_duplicates`]' rule: the same as [`SAME_PLAY_SQL`]'s, as two index ranges (the same second is inside
/// either). It runs at every start and after every library read, as one write: in the `ABS(…) <= ?1` form it held the
/// write lock for 25 s on 150,000 plays, long enough for the collector's own writes to give up.
/// A row in the trash may be taken out as a duplicate of a visible one, but never stands for one: dropping a visible
/// row for it would take the evening out of history with nothing in its place.
const RELINKED_DUPLICATES_SQL: &str = crate::relinked_duplicates_sql!("?1", " AND (k.deleted_at IS NULL OR playbacks.deleted_at IS NOT NULL)");

/// The statement behind [`RELINKED_DUPLICATES_SQL`], for a window given as SQL: a parameter here, and in the migration
/// that swept every install once, the merge window read from the settings. One text, so the two cannot drift.
#[macro_export]
macro_rules! relinked_duplicates_sql {
    // Migration 26's form, which must expand exactly as it did when it was released.
    ($window:literal) => {
        $crate::relinked_duplicates_sql!($window, "")
    };
    // `$keeper` narrows which rows may stand for a duplicate, `k` being the one kept and `playbacks` the one dropped.
    ($window:literal, $keeper:literal) => {
        concat!(
            "DELETE FROM playbacks WHERE source <> 'live' AND (
    EXISTS (SELECT 1 FROM playbacks k
             WHERE k.user_id = playbacks.user_id AND k.item_id = playbacks.item_id
               AND k.started_at BETWEEN playbacks.started_at - ", $window, " AND playbacks.started_at + ", $window, "
               AND k.source <> playbacks.source AND (k.source = 'live' OR k.id < playbacks.id)", $keeper, ")
    OR EXISTS (SELECT 1 FROM playbacks k
             WHERE k.user_id = playbacks.user_id AND k.item_id = playbacks.item_id
               AND k.ended_at BETWEEN playbacks.ended_at - ", $window, " AND playbacks.ended_at + ", $window, "
               AND k.source <> playbacks.source AND (k.source = 'live' OR k.id < playbacks.id)", $keeper, "))"
        )
    };
}

/// One thing that happened during a play (pause, skip, track switch…).
#[derive(Debug, Clone)]
pub struct PlayEvent {
    pub at: i64,
    pub kind: &'static str,
    pub position_s: Option<i64>,
    /// A seek only: where playback was expected to be when it jumped. `position_s` is where it landed.
    pub from_s: Option<i64>,
    pub detail: Option<String>,
}

/// Give a seek kept without its origin one from its label, which is what migration 21 did once for every
/// seek already in the database, repeated here for rows that arrive later without the column: a
/// backup written by a version from before it. The two statements must say the same thing.
pub fn backfill_seek_origins(conn: &Connection) -> Result<usize> {
    Ok(conn.execute(
        "UPDATE playback_events SET from_s = (
           WITH c(t) AS (SELECT substr(detail, 1, instr(detail, ' → ') - 1))
           SELECT CASE WHEN length(t) - length(replace(t, ':', '')) = 2
                       THEN CAST(substr(t, 1, instr(t, ':') - 1) AS INTEGER) * 3600
                          + CAST(substr(t, instr(t, ':') + 1, 2) AS INTEGER) * 60 + CAST(substr(t, -2) AS INTEGER)
                       ELSE CAST(substr(t, 1, instr(t, ':') - 1) AS INTEGER) * 60 + CAST(substr(t, -2) AS INTEGER) END
           FROM c)
         WHERE kind = 'seek' AND from_s IS NULL AND detail LIKE '%:__ → %'",
        [],
    )?)
}

pub fn insert_events(conn: &Connection, playback_id: i64, events: &[PlayEvent]) -> Result<()> {
    let mut stmt = conn.prepare_cached("INSERT INTO playback_events(playback_id, at, kind, position_s, from_s, detail) VALUES (?1, ?2, ?3, ?4, ?5, ?6)")?;
    for e in events {
        stmt.execute(crate::db::rusqlite::params![playback_id, e.at, e.kind, e.position_s, e.from_s, e.detail])?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::rusqlite::Connection;

    #[test]
    fn the_same_play_is_looked_for_in_a_window_of_one_persons_plays_of_one_title() {
        // Searched by title, then by person and title with the time tested row by row, every play read every other
        // play of its title: 150,000 plays took five minutes to import, and the sweep after every library read held
        // the write lock for 25 s, past the collector's 15-second wait, so live plays went unrecorded meanwhile.
        let c = conn();
        for (what, sql, n) in [("an import", SAME_PLAY_SQL, 6), ("the sweep after re-linking", RELINKED_DUPLICATES_SQL, 1)] {
            let args: Vec<Box<dyn crate::db::rusqlite::ToSql>> = (0..n).map(|i| Box::new(i as i64) as Box<dyn crate::db::rusqlite::ToSql>).collect();
            let plan: Vec<String> = c.prepare(&format!("EXPLAIN QUERY PLAN {sql}")).unwrap()
                .query_map(crate::db::rusqlite::params_from_iter(args.iter()), |r| r.get::<_, String>(3)).unwrap().map(Result::unwrap).collect();
            // (The sweep's delete also reaches each play's events through their foreign key, which is not the rule.)
            let searches: Vec<&String> = plan.iter().filter(|p| p.contains("SEARCH playbacks") || p.contains("SEARCH k ")).collect();
            assert!(!searches.is_empty() && searches.iter().all(|p| p.contains("user_id=? AND item_id=?") && (p.contains("started_at") || p.contains("ended_at"))), "{what}: {plan:?}");
        }
    }

    fn conn() -> Connection {
        let c = Connection::open_in_memory().unwrap();
        for m in crate::db::MIGRATIONS {
            c.execute_batch(m).unwrap();
        }
        c.execute_batch(
            "INSERT INTO playbacks(source, source_id, user_id, user_name, item_id, item_name, item_type, started_at, ended_at, duration_s)
               VALUES ('live', NULL, 'u1', 'alice', 'i1', 'Big Buck Bunny', 'Movie', 10000, 15400, 5400),
                      ('jellystat', 'jellystat:abc', 'u2', 'bob', 'i1', 'Big Buck Bunny', 'Movie', 90000, 95400, 5400);",
        )
        .unwrap();
        c
    }

    #[test]
    fn a_seek_is_stored_with_its_origin() {
        let c = conn();
        insert_events(&c, 1, &[
            PlayEvent { at: 10001, kind: "seek", position_s: Some(900), from_s: Some(105), detail: Some("1:45 → 15:00".into()) },
            PlayEvent { at: 10002, kind: "pause", position_s: Some(900), from_s: None, detail: None },
        ]).unwrap();
        let rows: Vec<(String, Option<i64>)> = c
            .prepare("SELECT kind, from_s FROM playback_events WHERE playback_id = 1 ORDER BY id").unwrap()
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?))).unwrap().collect::<Result<_, _>>().unwrap();
        assert_eq!(rows, [("seek".to_string(), Some(105)), ("pause".to_string(), None)]);
    }

    #[test]
    fn a_duplicate_that_relinking_creates_afterwards_is_taken_back_out() {
        let c = conn();
        // The rule runs before a row is written, against the item id the tracker gave. When that
        // item has since been renamed in Jellyfin, nothing matches it, and then `relink_orphans`
        // points the row at the item that is really there, which is the one an older row from
        // another tracker already points at. That is the duplicate the rule exists to stop,
        // made after it ran, so the rule has to be applied again wherever item ids are rewritten.
        c.execute_batch(
            "DELETE FROM playbacks;
             INSERT INTO playbacks(id, source, source_id, user_id, user_name, item_id, item_name, item_type, started_at, ended_at, duration_s) VALUES
               -- the same second, two trackers: the imported one goes
               (1, 'jellystat',    'js:1', 'u1', 'alice', 'i1', 'A track', 'Audio', 100, 101, 1),
               (2, 'streamystats', 'ss:1', 'u1', 'alice', 'i1', 'A track', 'Audio', 100, 101, 1),
               -- FinStats' own recording is the better row and is never the one removed, whenever it arrived
               (3, 'streamystats', 'ss:2', 'u1', 'alice', 'i2', 'A film', 'Movie', 1000, 4600, 3600),
               (4, 'live',         NULL,   'u1', 'alice', 'i2', 'A film', 'Movie', 1000, 4600, 3600),
               -- a real second viewing, days later: not a duplicate of anything
               (5, 'streamystats', 'ss:3', 'u1', 'alice', 'i1', 'A track', 'Audio', 900000, 900001, 1),
               -- two rows of one tracker are its own business, never touched here
               (6, 'streamystats', 'ss:4', 'u2', 'bob', 'i1', 'A track', 'Audio', 100, 101, 1),
               (7, 'streamystats', 'ss:5', 'u2', 'bob', 'i1', 'A track', 'Audio', 100, 101, 1);",
        )
        .unwrap();
        assert_eq!(drop_relinked_duplicates(&c, 600, "i1").unwrap() + drop_relinked_duplicates(&c, 600, "i2").unwrap(), 2);
        let left: Vec<i64> = c
            .prepare("SELECT id FROM playbacks ORDER BY id")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .map(Result::unwrap)
            .collect();
        assert_eq!(left, [1, 4, 5, 6, 7]);
        // And again changes nothing: there is no duplicate left to find.
        assert_eq!(drop_relinked_duplicates(&c, 600, "i1").unwrap() + drop_relinked_duplicates(&c, 600, "i2").unwrap(), 0);
    }

    #[test]
    fn the_sweep_for_one_title_leaves_every_other_title_alone() {
        // Re-linking moves plays onto a handful of titles; only there can a duplicate have appeared. Swept over the
        // whole history instead, it read every imported play at every start and after every library read: 2 s on a
        // million plays, all of it under the write lock, to find nothing.
        let c = conn();
        c.execute_batch(
            "DELETE FROM playbacks;
             INSERT INTO playbacks(id, source, source_id, user_id, user_name, item_id, item_name, item_type, started_at, ended_at, duration_s) VALUES
               (1, 'jellystat',    'js:1', 'u1', 'alice', 'i1', 'A track', 'Audio', 100, 101, 1),
               (2, 'streamystats', 'ss:1', 'u1', 'alice', 'i1', 'A track', 'Audio', 100, 101, 1),
               (3, 'streamystats', 'ss:2', 'u1', 'alice', 'i2', 'A film', 'Movie', 1000, 4600, 3600),
               (4, 'live',         NULL,   'u1', 'alice', 'i2', 'A film', 'Movie', 1000, 4600, 3600);",
        )
        .unwrap();
        assert_eq!(drop_relinked_duplicates(&c, 600, "i2").unwrap(), 1);
        let left: Vec<i64> = c.prepare("SELECT id FROM playbacks ORDER BY id").unwrap().query_map([], |r| r.get(0)).unwrap().map(Result::unwrap).collect();
        assert_eq!(left, [1, 2, 4], "the pair on i1 was not asked about");
        let plan: Vec<String> = c.prepare(&format!("EXPLAIN QUERY PLAN {RELINKED_DUPLICATES_SQL} AND item_id = ?2")).unwrap()
            .query_map(crate::db::rusqlite::params![600, "i2"], |r| r.get::<_, String>(3)).unwrap().map(Result::unwrap).collect();
        assert!(!plan.iter().any(|p| p.starts_with("SCAN playbacks")), "one title's plays, not every play: {plan:?}");
    }

    /// A row in the trash never displaces one in history: the evening would vanish with nothing in its place. The other
    /// way round it may go, as any duplicate does, since history still has the evening.
    #[test]
    fn the_sweep_never_keeps_a_deleted_row_and_drops_a_live_one() {
        let c = conn();
        c.execute_batch(
            "DELETE FROM playbacks;
             INSERT INTO playbacks(id, source, source_id, user_id, user_name, item_id, item_name, item_type, started_at, ended_at, duration_s, deleted_at) VALUES
               (1, 'live',      NULL,   'u1', 'alice', 'i1', 'A film', 'Movie', 1000, 4600, 3600, 5000),
               (2, 'jellystat', 'js:2', 'u1', 'alice', 'i1', 'A film', 'Movie', 1010, 4600, 3590, NULL),
               (3, 'live',      NULL,   'u1', 'alice', 'i2', 'A show', 'Episode', 1000, 4600, 3600, NULL),
               (4, 'jellystat', 'js:4', 'u1', 'alice', 'i2', 'A show', 'Episode', 1010, 4600, 3590, 5000);",
        )
        .unwrap();
        assert_eq!(drop_relinked_duplicates(&c, 600, "i1").unwrap(), 0, "a visible import was dropped for a play in the trash");
        assert_eq!(drop_relinked_duplicates(&c, 600, "i2").unwrap(), 1, "a trashed duplicate of a visible play stays a duplicate");
        let left: Vec<i64> = c.prepare("SELECT id FROM playbacks ORDER BY id").unwrap().query_map([], |r| r.get(0)).unwrap().map(Result::unwrap).collect();
        assert_eq!(left, [1, 2, 3]);
    }

    /// Identity sees the trash: a play in it is still that play, so importing the same file again, or the same evening
    /// from another tracker, does not bring it back as a visible duplicate.
    #[test]
    fn a_play_in_the_trash_is_still_recognised_when_it_arrives_again() {
        let c = conn();
        c.execute_batch(
            "DELETE FROM playbacks;
             INSERT INTO playbacks(id, source, source_id, user_id, user_name, item_id, item_name, item_type, started_at, ended_at, duration_s, deleted_at) VALUES
               (1, 'jellystat', 'jellystat:abc', 'u1', 'alice', 'i1', 'A film', 'Movie', 1000, 4600, 3600, 5000);",
        )
        .unwrap();
        assert!(already_recorded(&c, Play { source: "jellystat", source_id: Some("jellystat:abc"), user_id: "u1", item_id: "i1", started_at: 1000, ended_at: 4600 }, 600).unwrap());
        assert!(already_recorded(&c, Play { source: "streamystats", source_id: Some("streamystats:x"), user_id: "u1", item_id: "i1", started_at: 1200, ended_at: 4610 }, 600).unwrap());
    }

    #[test]
    fn a_play_already_here_is_recognised_by_its_own_id_whatever_its_start_says() {
        let c = conn();
        assert!(already_recorded(&c, Play { source: "jellystat", source_id: Some("jellystat:abc"), user_id: "u2", item_id: "i1", started_at: 123, ended_at: 456 }, 600).unwrap());
        assert!(!already_recorded(&c, Play { source: "streamystats", source_id: Some("streamystats:abc"), user_id: "u2", item_id: "i1", started_at: 123, ended_at: 456 }, 600).unwrap());
    }

    #[test]
    fn the_same_evening_from_another_tracker_is_recognised_by_person_item_and_start() {
        let c = conn();
        // Jellystat records the end and FinStats derives the start; Streamystats records the true
        // start. The same play therefore arrives a little off, and must still be the same play.
        for start in [10000, 10045, 9955] {
            assert!(already_recorded(&c, Play { source: "streamystats", source_id: Some("streamystats:x"), user_id: "u1", item_id: "i1", started_at: start, ended_at: start + 5400 }, 600).unwrap(), "{start}");
        }
        // Beyond the window it is a second viewing, not the same one.
        assert!(!already_recorded(&c, Play { source: "streamystats", source_id: Some("streamystats:x"), user_id: "u1", item_id: "i1", started_at: 10601, ended_at: 99999 }, 600).unwrap());
        // Another person, or another item, is never the same play.
        assert!(!already_recorded(&c, Play { source: "streamystats", source_id: Some("streamystats:x"), user_id: "u3", item_id: "i1", started_at: 10000, ended_at: 15400 }, 600).unwrap());
        assert!(!already_recorded(&c, Play { source: "streamystats", source_id: Some("streamystats:x"), user_id: "u1", item_id: "i2", started_at: 10000, ended_at: 15400 }, 600).unwrap());
    }

    #[test]
    fn the_same_evening_is_recognised_by_its_end_when_a_pause_moved_its_start() {
        let c = conn();
        // The trackers agree about when a play ended far better than about when it began: Jellystat
        // keeps only the end, so FinStats works its start back from the seconds played, and every
        // minute the viewer spent paused moves that start later. Half an hour of pause puts it well
        // outside any sane window, while both still say the play ended at the same moment.
        assert!(already_recorded(&c, Play { source: "jellystat", source_id: Some("jellystat:x"), user_id: "u1", item_id: "i1", started_at: 11_800, ended_at: 15_400 }, 600).unwrap());
        // Measured on real history: matching either end catches 2,750 of 3,162 plays the two
        // trackers held in common, against 2,527 for the start alone.
        assert!(already_recorded(&c, Play { source: "jellystat", source_id: Some("jellystat:x"), user_id: "u1", item_id: "i1", started_at: 11_800, ended_at: 15_430 }, 600).unwrap());
        // A different viewing of the same thing agrees on neither end.
        assert!(!already_recorded(&c, Play { source: "jellystat", source_id: Some("jellystat:x"), user_id: "u1", item_id: "i1", started_at: 30_000, ended_at: 35_400 }, 600).unwrap());
    }

    #[test]
    fn within_one_source_only_the_very_same_second_counts_as_the_same_play() {
        let c = conn();
        // One tracker never exports the same play twice, so a second row of the same item minutes
        // later is a restart the viewer really made. Widening the window inside a source would
        // silently drop it, on an import and, worse, on a restore of FinStats' own backup.
        assert!(!already_recorded(&c, Play { source: "live", source_id: None, user_id: "u1", item_id: "i1", started_at: 10180, ended_at: 99999 }, 600).unwrap());
        assert!(already_recorded(&c, Play { source: "live", source_id: None, user_id: "u1", item_id: "i1", started_at: 10000, ended_at: 99999 }, 600).unwrap());
    }

    #[test]
    fn a_window_of_zero_asks_for_the_very_same_second_from_anyone() {
        let c = conn();
        assert!(already_recorded(&c, Play { source: "streamystats", source_id: None, user_id: "u1", item_id: "i1", started_at: 10000, ended_at: 99999 }, 0).unwrap());
        assert!(!already_recorded(&c, Play { source: "streamystats", source_id: None, user_id: "u1", item_id: "i1", started_at: 10001, ended_at: 99999 }, 0).unwrap());
    }
}
