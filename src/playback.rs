//! The playback row, as written by both the live collector and the importer.

use anyhow::Result;
use serde_json::Value;

use crate::db::rusqlite::{Connection, named_params, params};
use crate::media::Streams;

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
    /// already: either this very row has been imported before, or another tracker — or the
    /// collector — got there first. See [`already_recorded`].
    pub fn insert_imported(&self, conn: &Connection, merge_window_s: i64) -> Result<Option<i64>> {
        if already_recorded(conn, self.source, self.source_id.as_deref(), &self.user_id, &self.item_id, self.started_at, merge_window_s)? {
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

/// Is this play already in the history? The one rule a restore and both importers share, so that
/// the same evening cannot be counted twice however it arrives.
///
/// Its own id settles it when there is one. Otherwise it is the same person, the same item and a
/// start close enough to be the same viewing — where "close enough" depends on who recorded the
/// other row. **Between** sources it is `window` seconds, because the trackers disagree about what
/// they record: Jellystat stores the *end* of a play and finstats derives the start from it, while
/// Streamystats and the collector store the real start, so one evening seen by two of them lands a
/// little apart. **Within** one source it is the very same second and nothing wider, because a
/// tracker never exports the same play twice: a second row of the same item minutes later is a
/// restart the viewer really made, and a window there would silently drop it.
pub fn already_recorded(conn: &Connection, source: &str, source_id: Option<&str>, user_id: &str, item_id: &str, started_at: i64, window: i64) -> Result<bool> {
    if let Some(sid) = source_id
        && conn.prepare_cached("SELECT 1 FROM playbacks WHERE source_id = ?1")?.exists([sid])?
    {
        return Ok(true);
    }
    Ok(conn
        .prepare_cached(
            "SELECT 1 FROM playbacks WHERE user_id = ?1 AND item_id = ?2
               AND (started_at = ?3 OR (source <> ?4 AND ABS(started_at - ?3) <= ?5))",
        )?
        .exists(params![user_id, item_id, started_at, source, window.max(0)])?)
}

/// One thing that happened during a play (pause, skip, track switch…).
#[derive(Debug, Clone)]
pub struct PlayEvent {
    pub at: i64,
    pub kind: &'static str,
    pub position_s: Option<i64>,
    pub detail: Option<String>,
}

pub fn insert_events(conn: &Connection, playback_id: i64, events: &[PlayEvent]) -> Result<()> {
    let mut stmt = conn.prepare_cached("INSERT INTO playback_events(playback_id, at, kind, position_s, detail) VALUES (?1, ?2, ?3, ?4, ?5)")?;
    for e in events {
        stmt.execute(crate::db::rusqlite::params![playback_id, e.at, e.kind, e.position_s, e.detail])?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::rusqlite::Connection;

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
    fn a_play_already_here_is_recognised_by_its_own_id_whatever_its_start_says() {
        let c = conn();
        assert!(already_recorded(&c, "jellystat", Some("jellystat:abc"), "u2", "i1", 123, 600).unwrap());
        assert!(!already_recorded(&c, "streamystats", Some("streamystats:abc"), "u2", "i1", 123, 600).unwrap());
    }

    #[test]
    fn the_same_evening_from_another_tracker_is_recognised_by_person_item_and_start() {
        let c = conn();
        // Jellystat records the end and finstats derives the start; Streamystats records the true
        // start. The same play therefore arrives a little off, and must still be the same play.
        for start in [10000, 10045, 9955] {
            assert!(already_recorded(&c, "streamystats", Some("streamystats:x"), "u1", "i1", start, 600).unwrap(), "{start}");
        }
        // Beyond the window it is a second viewing, not the same one.
        assert!(!already_recorded(&c, "streamystats", Some("streamystats:x"), "u1", "i1", 10601, 600).unwrap());
        // Another person, or another item, is never the same play.
        assert!(!already_recorded(&c, "streamystats", Some("streamystats:x"), "u3", "i1", 10000, 600).unwrap());
        assert!(!already_recorded(&c, "streamystats", Some("streamystats:x"), "u1", "i2", 10000, 600).unwrap());
    }

    #[test]
    fn within_one_source_only_the_very_same_second_counts_as_the_same_play() {
        let c = conn();
        // One tracker never exports the same play twice, so a second row of the same item minutes
        // later is a restart the viewer really made. Widening the window inside a source would
        // silently drop it — on an import and, worse, on a restore of finstats' own backup.
        assert!(!already_recorded(&c, "live", None, "u1", "i1", 10180, 600).unwrap());
        assert!(already_recorded(&c, "live", None, "u1", "i1", 10000, 600).unwrap());
    }

    #[test]
    fn a_window_of_zero_asks_for_the_very_same_second_from_anyone() {
        let c = conn();
        assert!(already_recorded(&c, "streamystats", None, "u1", "i1", 10000, 0).unwrap());
        assert!(!already_recorded(&c, "streamystats", None, "u1", "i1", 10001, 0).unwrap());
    }
}
