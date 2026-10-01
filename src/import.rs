//! Import of Jellystat backups (`.jsonl`, plus the older single-document `.json`).
//!
//! What the backup means, learned from real exports:
//! - `ActivityDateInserted` is when the play *ended*; start = end − `PlaybackDuration`.
//! - For episodes `NowPlayingItemId` is the *series*; the episode itself is `EpisodeId`.
//! - `PlayState.PositionTicks` is usually the position when Jellystat first noticed the
//!   session, not where playback stopped, so it is not imported as a position.
//! - bigint columns (`PlaybackDuration`, `RunTimeTicks`, `Size`…) arrive as strings.

use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::Path;

use anyhow::{Context, Result, bail};
use serde::Serialize;
use serde_json::Value;

use crate::db::{Db, norm_id, parse_ts, rusqlite::Connection, rusqlite::OptionalExtension, rusqlite::params};
use crate::media::{self, Streams, int, ticks_to_s};
use crate::playback::PlayRecord;
use crate::state::Tasks;
use crate::sync::backfill_playbacks;

const TASK: &str = "import";

/// What a play from Jellystat says it came from.
pub const SOURCE: &str = "jellystat";

#[derive(Debug, Default, Serialize, Clone)]
pub struct ImportResult {
    pub plays_imported: u64,
    pub plays_skipped: u64,
    pub users: u64,
    pub libraries: u64,
    pub items: u64,
    pub seasons: u64,
    pub episodes: u64,
    pub item_info: u64,
    pub unknown_rows: u64,
}

fn opt_str(v: &Value) -> Option<String> {
    v.as_str().map(str::trim).filter(|s| !s.is_empty()).map(str::to_string)
}

fn phase_label(table: &str) -> &'static str {
    match table {
        "jf_libraries" => "Reading libraries",
        "jf_library_items" => "Reading movies, shows and music",
        "jf_library_seasons" => "Reading seasons",
        "jf_library_episodes" => "Reading episodes",
        "jf_users" => "Reading users",
        "jf_playback_activity" => "Importing playback history",
        "jf_item_info" => "Reading file details",
        _ => "Reading backup",
    }
}

/// Blocking. Parses the backup at `path` and writes it in a single transaction, so a
/// failed import leaves the database exactly as it was.
pub fn run(db: &Db, path: &Path, tasks: Option<&Tasks>) -> Result<ImportResult> {
    let mut file = File::open(path).context("opening the uploaded backup")?;
    let total_bytes = file.metadata()?.len().max(1);

    // The older Jellystat format is one JSON array; the current one is a record per line.
    let mut first = [0u8; 1];
    let legacy = loop {
        if file.read(&mut first)? == 0 {
            bail!("The file is empty");
        }
        if !first[0].is_ascii_whitespace() {
            break first[0] == b'[';
        }
    };
    file.seek(SeekFrom::Start(0))?;

    let mut conn = db.conn()?;
    let settings = crate::state::Settings::load(&conn)?;
    // Immediate: the import will write, so it takes the write lock before its first read rather than
    // failing to upgrade later when another connection (the collector, an audit row) commits meanwhile.
    let tx = conn.transaction_with_behavior(crate::db::rusqlite::TransactionBehavior::Immediate)?;
    let mut res = ImportResult::default();
    let report = |msg: &str, p: f64| {
        if let Some(t) = tasks {
            t.update(TASK, msg, Some(p));
        }
    };

    if legacy {
        report("Reading backup (older single-file format, this needs more memory)", 0.05);
        let doc: Value = serde_json::from_reader(BufReader::with_capacity(1 << 20, file))
            .context("The file is not a valid Jellystat backup (JSON could not be parsed)")?;
        let sections = doc.as_array().context("The file is not a Jellystat backup")?;
        let count = sections.len().max(1);
        for (i, section) in sections.iter().enumerate() {
            let Some(obj) = section.as_object() else { continue };
            for (table, rows) in obj {
                report(phase_label(table), 0.1 + 0.8 * i as f64 / count as f64);
                for row in rows.as_array().map(Vec::as_slice).unwrap_or_default() {
                    import_row(&tx, table, row, &mut res, settings.merge_window_s)?;
                }
            }
        }
    } else {
        let mut reader = BufReader::with_capacity(1 << 20, file);
        let mut line = String::new();
        let mut read_bytes = 0u64;
        let mut line_no = 0u64;
        let mut recognised = 0u64;
        let mut current_table = String::new();
        loop {
            line.clear();
            let n = reader.read_line(&mut line)?;
            if n == 0 {
                break;
            }
            read_bytes += n as u64;
            line_no += 1;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let mut v: Value = match serde_json::from_str(trimmed) {
                Ok(v) => v,
                Err(e) if line_no <= 3 && recognised == 0 => {
                    bail!("This does not look like a Jellystat backup (line {line_no}: {e})")
                }
                Err(e) => bail!("The backup is damaged at line {line_no}: {e}"),
            };
            match v["type"].as_str() {
                Some("table") => {
                    recognised += 1;
                    current_table = v["table"].as_str().unwrap_or_default().to_string();
                }
                Some("row") => {
                    recognised += 1;
                    let table = v["table"].as_str().map(str::to_string).unwrap_or_else(|| current_table.clone());
                    let data = v["data"].take();
                    import_row(&tx, &table, &data, &mut res, settings.merge_window_s)?;
                }
                _ => res.unknown_rows += 1,
            }
            if line_no % 2000 == 0 {
                report(phase_label(&current_table), 0.95 * read_bytes as f64 / total_bytes as f64);
            }
            if line_no == 50 && recognised == 0 {
                bail!("This does not look like a Jellystat backup: none of the first lines are backup records");
            }
        }
        if recognised == 0 {
            bail!("This does not look like a Jellystat backup: no backup records found");
        }
    }

    report("Linking plays to your library", 0.96);
    finalize(&tx)?;
    report("Saving", 0.99);
    tx.commit()?;
    crate::groups::detect(&mut conn, settings.group_window_s, None)?;
    conn.execute_batch("PRAGMA optimize;")?;
    Ok(res)
}

fn import_row(conn: &Connection, table: &str, d: &Value, res: &mut ImportResult, merge_window_s: i64) -> Result<()> {
    match table {
        "jf_playback_activity" => import_play(conn, d, res, merge_window_s),
        "jf_users" => {
            let Some(id) = d["Id"].as_str() else { return Ok(()) };
            res.users += conn
                .prepare_cached(
                    "INSERT OR IGNORE INTO users(id, name, is_admin, image_tag, last_login_at, last_activity_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0)",
                )?
                .execute(params![
                    norm_id(id),
                    d["Name"].as_str().unwrap_or("Unknown"),
                    d["IsAdministrator"].as_bool().unwrap_or(false),
                    opt_str(&d["PrimaryImageTag"]),
                    d["LastLoginDate"].as_str().and_then(parse_ts),
                    d["LastActivityDate"].as_str().and_then(parse_ts),
                ])? as u64;
            Ok(())
        }
        "jf_libraries" => {
            let Some(id) = d["Id"].as_str() else { return Ok(()) };
            res.libraries += conn
                .prepare_cached(
                    "INSERT OR IGNORE INTO libraries(id, name, collection_type, image_tag, removed, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, 0)",
                )?
                .execute(params![
                    norm_id(id),
                    d["Name"].as_str().unwrap_or("Library"),
                    opt_str(&d["CollectionType"]),
                    opt_str(&d["ImageTagsPrimary"]),
                    d["archived"].as_bool().unwrap_or(false),
                ])? as u64;
            Ok(())
        }
        "jf_library_items" => {
            let Some(id) = d["Id"].as_str() else { return Ok(()) };
            let genres = d["Genres"].as_array().filter(|g| !g.is_empty()).map(|g| Value::Array(g.clone()).to_string());
            res.items += conn
                .prepare_cached(
                    "INSERT OR IGNORE INTO items(id, library_id, type, name, runtime_s, production_year, premiere_date,
                        date_created, community_rating, genres, image_tag, backdrop_tag, removed, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, 0)",
                )?
                .execute(params![
                    norm_id(id),
                    d["ParentId"].as_str().map(norm_id),
                    d["Type"].as_str().unwrap_or("Unknown"),
                    d["Name"].as_str().unwrap_or("Unknown"),
                    ticks_to_s(&d["RunTimeTicks"]),
                    d["ProductionYear"].as_i64(),
                    opt_str(&d["PremiereDate"]),
                    d["DateCreated"].as_str().and_then(parse_ts),
                    d["CommunityRating"].as_f64(),
                    genres,
                    opt_str(&d["ImageTagsPrimary"]),
                    opt_str(&d["BackdropImageTags"]),
                    d["archived"].as_bool().unwrap_or(false),
                ])? as u64;
            Ok(())
        }
        "jf_library_seasons" => {
            let Some(id) = d["Id"].as_str() else { return Ok(()) };
            res.seasons += conn
                .prepare_cached(
                    "INSERT OR IGNORE INTO items(id, type, name, series_id, series_name, index_number, removed, updated_at)
                     VALUES (?1, 'Season', ?2, ?3, ?4, ?5, ?6, 0)",
                )?
                .execute(params![
                    norm_id(id),
                    d["Name"].as_str().unwrap_or("Season"),
                    d["SeriesId"].as_str().map(norm_id),
                    opt_str(&d["SeriesName"]),
                    d["IndexNumber"].as_i64(),
                    d["archived"].as_bool().unwrap_or(false),
                ])? as u64;
            Ok(())
        }
        "jf_library_episodes" => {
            // `Id` is Jellystat's own composite key; `EpisodeId` is the Jellyfin item.
            let Some(id) = d["EpisodeId"].as_str() else { return Ok(()) };
            res.episodes += conn
                .prepare_cached(
                    "INSERT OR IGNORE INTO items(id, type, name, series_id, season_id, series_name, index_number,
                        parent_index_number, runtime_s, production_year, premiere_date, date_created,
                        community_rating, official_rating, removed, updated_at)
                     VALUES (?1, 'Episode', ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, 0)",
                )?
                .execute(params![
                    norm_id(id),
                    d["Name"].as_str().unwrap_or("Episode"),
                    d["SeriesId"].as_str().map(norm_id),
                    d["SeasonId"].as_str().map(norm_id),
                    opt_str(&d["SeriesName"]),
                    d["IndexNumber"].as_i64(),
                    d["ParentIndexNumber"].as_i64(),
                    ticks_to_s(&d["RunTimeTicks"]),
                    d["ProductionYear"].as_i64(),
                    opt_str(&d["PremiereDate"]),
                    d["DateCreated"].as_str().and_then(parse_ts),
                    d["CommunityRating"].as_f64(),
                    opt_str(&d["OfficialRating"]),
                    d["archived"].as_bool().unwrap_or(false),
                ])? as u64;
            Ok(())
        }
        "jf_item_info" => {
            let Some(id) = d["Id"].as_str() else { return Ok(()) };
            let st = Streams::extract(&d["MediaStreams"], &Value::Null);
            let path = opt_str(&d["Path"]);
            let container = path.as_deref().and_then(|p| p.rsplit_once('.')).map(|(_, ext)| ext.to_lowercase()).filter(|e| e.len() <= 5);
            // Only fills gaps: a live sync knows better than an old backup.
            res.item_info += conn
                .prepare_cached(
                    "UPDATE items SET path = ?2, size_bytes = ?3, bitrate = ?4, container = COALESCE(container, ?5),
                        video_codec = ?6, width = ?7, height = ?8, video_range = ?9, audio_codec = ?10, audio_channels = ?11,
                        audio_languages = ?12, subtitle_languages = ?13
                     WHERE id = ?1 AND size_bytes IS NULL",
                )?
                .execute(params![
                    norm_id(id),
                    path,
                    int(&d["Size"]),
                    int(&d["Bitrate"]),
                    container,
                    st.video_codec,
                    st.width,
                    st.height,
                    st.video_range,
                    st.audio_codec,
                    st.audio_channels,
                    crate::media::track_languages(&d["MediaStreams"], "Audio"),
                    crate::media::track_languages(&d["MediaStreams"], "Subtitle"),
                ])? as u64;
            Ok(())
        }
        // Raw Playback Reporting plugin rows: Jellystat has already folded these into
        // jf_playback_activity (flagged `imported`), so taking them again would double count.
        "jf_playback_reporting_plugin_data" => Ok(()),
        _ => {
            res.unknown_rows += 1;
            Ok(())
        }
    }
}

fn import_play(conn: &Connection, d: &Value, res: &mut ImportResult, merge_window_s: i64) -> Result<()> {
    let imported = match record(conn, d)? {
        Some(rec) => rec.insert_imported(conn, merge_window_s)?.is_some(),
        None => false,
    };
    match imported {
        true => res.plays_imported += 1,
        false => res.plays_skipped += 1,
    }
    Ok(())
}

/// One row of `jf_playback_activity` as a play, by the rules at the top of this file; `None` for a row that cannot
/// be one (no id, person, item or date). Reads the library only for the type of an item that is not an episode.
fn record(conn: &Connection, d: &Value) -> Result<Option<PlayRecord>> {
    let (Some(source_id), Some(user_id), Some(np_id)) = (d["Id"].as_str(), d["UserId"].as_str(), d["NowPlayingItemId"].as_str()) else {
        return Ok(None);
    };
    let Some(ended_at) = d["ActivityDateInserted"].as_str().and_then(parse_ts) else {
        return Ok(None);
    };
    let duration_s = int(&d["PlaybackDuration"]).unwrap_or(0).max(0);
    let play_state = &d["PlayState"];
    let streams = Streams::extract(&d["MediaStreams"], play_state);
    let transcode = media::compact_transcode(&d["TranscodingInfo"]);
    let reported = d["PlayMethod"].as_str().or_else(|| play_state["PlayMethod"].as_str());
    let play_method = media::effective_play_method(reported, transcode.as_ref());

    let episode_id = d["EpisodeId"].as_str().filter(|s| !s.is_empty());
    let (item_id, series_id) = match episode_id {
        Some(ep) => (norm_id(ep), Some(norm_id(np_id))),
        None => (norm_id(np_id), None),
    };
    let item_type = if episode_id.is_some() {
        "Episode".to_string()
    } else {
        conn.prepare_cached("SELECT type FROM items WHERE id = ?1")?
            .query_row([&item_id], |r| r.get::<_, String>(0))
            .optional()?
            // Not in the library and Jellystat records no type, so go by what the stream looked like:
            // a picture without a file container is a Live TV channel, with one it was a film.
            .unwrap_or_else(|| {
                match (streams.has_video(), opt_str(&d["OriginalContainer"]).is_some()) {
                    (true, false) => "TvChannel",
                    (true, true) => "Movie",
                    (false, _) => "Audio",
                }
                .to_string()
            })
    };

    Ok(Some(PlayRecord {
        source: SOURCE,
        source_id: Some(format!("jellystat:{source_id}")),
        active: false,
        user_id: norm_id(user_id),
        user_name: d["UserName"].as_str().unwrap_or("Unknown").to_string(),
        item_id,
        item_name: d["NowPlayingItemName"].as_str().unwrap_or("Unknown").to_string(),
        item_type,
        series_id,
        series_name: opt_str(&d["SeriesName"]),
        season_id: d["SeasonId"].as_str().map(norm_id),
        season_number: None,
        episode_number: None,
        started_at: ended_at - duration_s,
        ended_at,
        duration_s,
        paused_s: 0,
        position_s: None,
        runtime_s: None,
        client: opt_str(&d["Client"]),
        device_name: opt_str(&d["DeviceName"]),
        device_id: opt_str(&d["DeviceId"]),
        app_version: opt_str(&d["ApplicationVersion"]),
        remote_ip: opt_str(&d["RemoteEndPoint"]),
        play_method,
        container: opt_str(&d["OriginalContainer"]).map(|c| c.split(',').next().unwrap_or_default().to_string()),
        streams,
        transcode,
        pause_count: 0,
        seek_count: 0,
        start_position_s: None,
    }))
}

/// Cross-table fix-ups that don't depend on the order tables appear in the backup. What the library
/// can say about a play lives in [`backfill_playbacks`], which runs after a library read too.
pub(crate) fn finalize(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "UPDATE items SET library_id = (SELECT s.library_id FROM items s WHERE s.id = items.series_id)
         WHERE library_id IS NULL AND series_id IS NOT NULL;",
    )?;
    backfill_playbacks(conn)?;
    crate::network::reclassify(conn)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::path::PathBuf;

    /// A backup in a folder of its own, and a fresh database beside it.
    fn backup(name: &str, text: &str) -> (Db, PathBuf, PathBuf) {
        let dir = std::env::temp_dir().join(format!("finstats-import-test-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("backup.jsonl");
        std::fs::write(&file, text).unwrap();
        (Db::open(&dir.join("finstats.db")).unwrap(), file, dir)
    }

    fn at(ts: &str) -> i64 {
        parse_ts(ts).unwrap()
    }

    /// A film, a show with one episode, alice, and one play of each — in the line-per-record format.
    const JSONL: &str = r#"{"type":"table","table":"jf_users"}
{"type":"row","table":"jf_users","data":{"Id":"AAAA-1111","Name":"alice","IsAdministrator":false}}
{"type":"table","table":"jf_libraries"}
{"type":"row","data":{"Id":"LIB-1","Name":"Films","CollectionType":"movies"}}
{"type":"table","table":"jf_library_items"}
{"type":"row","data":{"Id":"FILM-1","ParentId":"LIB-1","Type":"Movie","Name":"Big Buck Bunny","RunTimeTicks":"6000000000","ProductionYear":2008}}
{"type":"row","data":{"Id":"SHOW-1","ParentId":"LIB-1","Type":"Series","Name":"Low Orbit"}}
{"type":"table","table":"jf_library_episodes"}
{"type":"row","data":{"Id":"jellystat-own-key","EpisodeId":"EP-1","Name":"Pilot","SeriesId":"SHOW-1","SeasonId":"SEASON-1","SeriesName":"Low Orbit","IndexNumber":1,"ParentIndexNumber":1,"RunTimeTicks":"18000000000"}}
{"type":"table","table":"jf_playback_activity"}
{"type":"row","data":{"Id":"p1","UserId":"AAAA-1111","UserName":"alice","NowPlayingItemId":"FILM-1","NowPlayingItemName":"Big Buck Bunny","ActivityDateInserted":"2026-03-01T21:00:00.000Z","PlaybackDuration":"5400","Client":"Jellyfin Web","PlayMethod":"DirectPlay","PlayState":{"PositionTicks":12345678}}}
{"type":"row","data":{"Id":"p2","UserId":"AAAA-1111","UserName":"alice","NowPlayingItemId":"SHOW-1","EpisodeId":"EP-1","NowPlayingItemName":"Pilot","SeriesName":"Low Orbit","ActivityDateInserted":"2026-03-02T20:30:00.000Z","PlaybackDuration":1500}}
{"type":"table","table":"jf_playback_reporting_plugin_data"}
{"type":"row","data":{"rowid":1,"ItemId":"FILM-1"}}
{"type":"table","table":"jf_something_new"}
{"type":"row","data":{"anything":true}}
"#;

    #[test]
    fn a_backup_imports_its_people_titles_and_plays_and_twice_is_once() {
        let (db, file, dir) = backup("whole", JSONL);
        let res = run(&db, &file, None).unwrap();
        assert_eq!((res.users, res.libraries, res.items, res.episodes), (1, 1, 2, 1));
        assert_eq!((res.plays_imported, res.plays_skipped), (2, 0));
        assert_eq!(res.unknown_rows, 1, "a table it does not know is counted; the plugin's own rows are already in the activity");

        type Row = (String, String, Option<String>, String, i64, i64, Option<i64>, Option<i64>, Option<String>, Option<i64>, Option<i64>);
        let rows: Vec<Row> = db
            .conn()
            .unwrap()
            .prepare("SELECT user_id, item_id, series_id, item_type, started_at, ended_at, position_s, runtime_s, library_id, season_number, episode_number FROM playbacks ORDER BY started_at")
            .unwrap()
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?, r.get(7)?, r.get(8)?, r.get(9)?, r.get(10)?)))
            .unwrap()
            .map(Result::unwrap)
            .collect();
        let (film, episode) = (&rows[0], &rows[1]);
        assert_eq!((film.0.as_str(), film.1.as_str(), film.3.as_str()), ("aaaa1111", "film1", "Movie"), "ids as Jellyfin's, without dashes");
        assert_eq!((film.4, film.5), (at("2026-03-01T21:00:00Z") - 5400, at("2026-03-01T21:00:00Z")), "the date is the end; the start is worked back");
        assert_eq!(film.6, None, "Jellystat's position is where it first noticed the play, not where it stopped");
        assert_eq!((film.7, film.8.as_deref()), (Some(600), Some("lib1")), "the library says how long it runs and where it lives");
        assert_eq!((episode.1.as_str(), episode.2.as_deref(), episode.3.as_str()), ("ep1", Some("show1"), "Episode"), "for an episode the item is the series");
        assert_eq!((episode.4, episode.9, episode.10), (at("2026-03-02T20:30:00Z") - 1500, Some(1), Some(1)));
        assert_eq!(episode.8.as_deref(), Some("lib1"), "an episode lives where its show does");

        let again = run(&db, &file, None).unwrap();
        assert_eq!((again.plays_imported, again.plays_skipped), (0, 2), "the same backup twice is once");
        let _ = std::fs::remove_dir_all(dir);
    }

    /// A play row as Jellystat writes one, with `extra` laid over it.
    fn play(extra: Value) -> Value {
        let mut d = json!({ "Id": "p1", "UserId": "AAAA-1111", "UserName": "alice", "NowPlayingItemId": "ITEM-1",
            "NowPlayingItemName": "Big Buck Bunny", "ActivityDateInserted": "2026-03-01T21:00:00.000Z", "PlaybackDuration": "600" });
        for (k, v) in extra.as_object().unwrap() {
            d[k] = v.clone();
        }
        d
    }

    fn library() -> Db {
        let db = Db::open_in_memory().unwrap();
        db.conn().unwrap().execute_batch("INSERT INTO items(id, type, name, updated_at) VALUES ('item1', 'MusicVideo', 'Big Buck Bunny', 1)").unwrap();
        db
    }

    #[test]
    fn a_row_without_an_id_a_person_an_item_or_a_date_is_not_a_play() {
        let db = library();
        let c = db.conn().unwrap();
        assert!(record(&c, &play(json!({}))).unwrap().is_some());
        for (key, value) in [("Id", Value::Null), ("UserId", Value::Null), ("NowPlayingItemId", Value::Null), ("ActivityDateInserted", Value::Null), ("ActivityDateInserted", "yesterday".into())] {
            assert!(record(&c, &play(json!({ key: value }))).unwrap().is_none(), "{key} = {value}");
        }
    }

    /// Jellystat records no type. The library knows the ones it has; anything else is typed by what its stream was.
    #[test]
    fn a_title_the_library_does_not_know_is_typed_by_what_its_stream_looked_like() {
        let db = library();
        let c = db.conn().unwrap();
        let video = json!([{ "Type": "Video", "Codec": "h264" }, { "Type": "Audio", "Codec": "aac" }]);
        let audio = json!([{ "Type": "Audio", "Codec": "flac" }]);
        let typed = |extra: Value| record(&c, &play(extra)).unwrap().unwrap();
        assert_eq!(typed(json!({ "MediaStreams": video })).item_type, "MusicVideo", "the library's type, whatever the stream");
        let elsewhere = |mut extra: Value| {
            extra["NowPlayingItemId"] = json!("GONE-1");
            typed(extra)
        };
        assert_eq!(elsewhere(json!({ "MediaStreams": video })).item_type, "TvChannel", "a picture with no file behind it is a channel");
        let film = elsewhere(json!({ "MediaStreams": video, "OriginalContainer": "mkv,webm" }));
        assert_eq!((film.item_type.as_str(), film.container.as_deref()), ("Movie", Some("mkv")), "with a file it was a film, in the first container named");
        assert_eq!(elsewhere(json!({ "MediaStreams": audio })).item_type, "Audio");
        assert_eq!(elsewhere(json!({ "MediaStreams": video, "EpisodeId": "EP-1" })).item_type, "Episode", "an episode id says it all");
    }

    /// The same remux rule as a live play: a "transcode" that copied both picture and sound is a direct stream.
    #[test]
    fn a_transcode_that_copied_picture_and_sound_is_a_direct_stream() {
        let db = library();
        let c = db.conn().unwrap();
        let method = |extra: Value| record(&c, &play(extra)).unwrap().unwrap().play_method;
        let copied = json!({ "IsVideoDirect": true, "IsAudioDirect": true, "VideoCodec": "h264", "AudioCodec": "aac" });
        let converted = json!({ "IsVideoDirect": false, "IsAudioDirect": true, "VideoCodec": "h264", "AudioCodec": "aac" });
        assert_eq!(method(json!({ "PlayMethod": "Transcode", "TranscodingInfo": copied })), "DirectStream");
        assert_eq!(method(json!({ "PlayMethod": "Transcode", "TranscodingInfo": converted })), "Transcode");
        assert_eq!(method(json!({ "PlayState": { "PlayMethod": "DirectStream" } })), "DirectStream", "or as the play state says it");
        assert_eq!(method(json!({})), "DirectPlay", "said nowhere: played directly");
    }

    #[test]
    fn a_length_that_cannot_be_one_is_none_at_all() {
        let db = library();
        let c = db.conn().unwrap();
        for length in [json!("-30"), Value::Null, json!("soon")] {
            let rec = record(&c, &play(json!({ "PlaybackDuration": length }))).unwrap().unwrap();
            assert_eq!((rec.duration_s, rec.started_at), (0, rec.ended_at), "{length}");
        }
    }

    #[test]
    fn the_older_single_document_format_imports_the_same() {
        let legacy = r#"[{"jf_users":[{"Id":"AAAA-1111","Name":"alice"}]},
            {"jf_playback_activity":[{"Id":"p1","UserId":"AAAA-1111","UserName":"alice","NowPlayingItemId":"FILM-1","NowPlayingItemName":"Big Buck Bunny","ActivityDateInserted":"2026-03-01T21:00:00.000Z","PlaybackDuration":"5400"}]}]"#;
        let (db, file, dir) = backup("legacy", &format!("\n  {legacy}"));
        let res = run(&db, &file, None).unwrap();
        assert_eq!((res.users, res.plays_imported), (1, 1));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_file_that_is_not_a_backup_or_is_damaged_changes_nothing() {
        let (db, file, dir) = backup("empty", "  \n ");
        assert!(format!("{:#}", run(&db, &file, None).unwrap_err()).contains("empty"));
        let _ = std::fs::remove_dir_all(dir);

        let (db, file, dir) = backup("foreign", "name,watched\nalice,Big Buck Bunny\n");
        assert!(format!("{:#}", run(&db, &file, None).unwrap_err()).contains("does not look like a Jellystat backup"));
        let _ = std::fs::remove_dir_all(dir);

        // Good records first, then a broken line: one transaction, so not even the good ones stay.
        let damaged = format!("{}{{\"type\":\"row\",\"data\":", &JSONL[..JSONL.find("{\"type\":\"table\",\"table\":\"jf_playback_reporting").unwrap()]);
        let (db, file, dir) = backup("damaged", &damaged);
        let err = format!("{:#}", run(&db, &file, None).unwrap_err());
        assert!(err.contains("damaged at line 13"), "{err}");
        let c = db.conn().unwrap();
        let counts: (i64, i64, i64) = c.query_row("SELECT (SELECT COUNT(*) FROM playbacks), (SELECT COUNT(*) FROM users), (SELECT COUNT(*) FROM items)", [], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))).unwrap();
        assert_eq!(counts, (0, 0, 0), "nothing of a failed import is kept");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn what_a_backup_knows_about_a_file_only_fills_gaps() {
        let info = r#"{"type":"table","table":"jf_item_info"}
{"type":"row","data":{"Id":"FILM-1","Path":"/media/films/Big Buck Bunny (2008).MKV","Size":"1048576","MediaStreams":[{"Type":"Video","Codec":"h264","Width":1920,"Height":1080},{"Type":"Audio","Codec":"aac","Language":"eng"}]}}
{"type":"row","data":{"Id":"SHOW-1","Path":"/media/shows/Low Orbit","Size":"1"}}
"#;
        let (db, file, dir) = backup("info", &format!("{JSONL}{info}"));
        db.conn().unwrap().execute_batch("INSERT INTO items(id, type, name, size_bytes, updated_at) VALUES ('show1', 'Series', 'Low Orbit', 999, 1)").unwrap();
        let res = run(&db, &file, None).unwrap();
        assert_eq!(res.item_info, 1, "only the film had a gap to fill");
        let c = db.conn().unwrap();
        let film: (Option<String>, i64, Option<String>, Option<String>) =
            c.query_row("SELECT container, size_bytes, video_codec, audio_languages FROM items WHERE id = 'film1'", [], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))).unwrap();
        assert_eq!(film, (Some("mkv".into()), 1_048_576, Some("h264".into()), Some(r#"["eng"]"#.into())));
        let kept: i64 = c.query_row("SELECT size_bytes FROM items WHERE id = 'show1'", [], |r| r.get(0)).unwrap();
        assert_eq!(kept, 999, "what finstats read from Jellyfin itself is fresher than an old backup");
        let _ = std::fs::remove_dir_all(dir);
    }
}
