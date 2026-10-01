//! Import of Streamystats backups (its **Settings → Backup & Import → Download Backup**).
//!
//! What the export means, learned from a real file rather than from documentation:
//! - It is **sessions only**. There are no items, users or libraries in it, so an import brings no
//!   library data with it — only history, and the people it names.
//! - A row carries **one moment or two**, and which it is decides what the moment means. Two
//!   (`startTime` < `endTime`) is a play Streamystats watched itself: the first is the real start,
//!   and whatever the pair leaves over `playDuration` is time the play was not running. One
//!   (`startTime` == `endTime`) is a play it imported from Jellystat, and that moment is the
//!   **end** — Jellystat's `ActivityDateInserted`, copied into both fields. Reading it as a start
//!   would move every one of those evenings forward by the length of the film, which is the kind
//!   of mistake that is invisible until every chart is quietly wrong.
//! - `isInferred`, or an id beginning `inferred:`, is **not a play**: Jellyfin reported the item
//!   watched, so Streamystats wrote a row as long as the whole runtime for a viewing nobody saw.
//! - `itemId` is the item in both kinds — Streamystats re-links renamed items. `mediaSourceId` is
//!   *not* an item id.
//! - A play Streamystats watched itself says **nothing about the file**: `videoCodec`,
//!   `resolution*`, `audioCodec` and `videoRangeType` are empty in every row a real export holds.
//!   Only the transcode it was serving is there. A play from Jellystat is the other way round: it
//!   carries the whole session in `rawData`, and *there* the flat `transcoding*` columns are a copy
//!   of the source and `transcodeReasons` a placeholder, so only the raw `TranscodingInfo` is true.
//! - `isActive` is `true` on nearly every row of a backup and means nothing.
//! - `positionTicks` counts only where the row also keeps the `runtimeTicks` it is a position in:
//!   Jellystat's is the position it happened to catch, not where playback stopped.

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use anyhow::{Context, Result, bail};
use serde::Serialize;
use serde::de::{DeserializeSeed, IgnoredAny, MapAccess, SeqAccess, Visitor};
use serde_json::{Value, json};

use crate::db::{Db, norm_id, parse_ts, rusqlite::Connection, rusqlite::OptionalExtension, rusqlite::params};
use crate::media::{self, Streams, int, ticks_to_s};
use crate::playback::PlayRecord;
use crate::state::Tasks;

const TASK: &str = "import_streamystats";

/// What a play from Streamystats says it came from.
pub const SOURCE: &str = "streamystats";

#[derive(Debug, Default, Serialize, Clone)]
pub struct ImportResult {
    pub sessions_read: u64,
    pub plays_imported: u64,
    pub plays_skipped: u64,
    /// Rows that were never a play: Jellyfin said the item was watched and Streamystats wrote one.
    pub marked_watched: u64,
    pub users: u64,
    pub unreadable_rows: u64,
}

fn opt_str(v: &Value) -> Option<String> {
    v.as_str().map(str::trim).filter(|s| !s.is_empty()).map(str::to_string)
}

fn lower(v: &Value) -> Option<String> {
    opt_str(v).map(|s| s.to_lowercase())
}

/// What a row in the export is.
#[derive(Debug, PartialEq, Eq)]
enum Kind {
    /// A play — either one Streamystats watched or one it imported. Which of the two shows in its
    /// timestamps rather than in any flag; see [`timing`].
    Play,
    /// Not a play: a row as long as the whole runtime, for a viewing nobody saw.
    MarkedWatched,
}

fn kind(d: &Value) -> Kind {
    let inferred = d["isInferred"].as_bool().unwrap_or(false) || d["id"].as_str().is_some_and(|id| id.starts_with("inferred:"));
    if inferred { Kind::MarkedWatched } else { Kind::Play }
}

/// A row Streamystats imported from Jellystat rather than watched itself. Its `rawData` is the
/// session Jellystat kept, and its own flat `transcoding*` columns must not be read.
fn from_jellystat(d: &Value) -> bool {
    d["rawData"]["ActivityDateInserted"].is_string()
}

#[derive(Debug, PartialEq, Eq)]
struct Timing {
    started_at: i64,
    ended_at: i64,
    duration_s: i64,
}

/// When the play ran, and for how much of that it was actually running.
fn timing(d: &Value) -> Option<Timing> {
    let start = parse_ts(d["startTime"].as_str()?)?;
    let end = d["endTime"].as_str().and_then(parse_ts).unwrap_or(start);
    let played = int(&d["playDuration"]).unwrap_or(0).max(0);
    if end > start {
        // Two moments: the play began at the first, and nothing can have run for longer than the
        // time between them. What the pair leaves over is time it was paused or simply sitting
        // there — Streamystats does not say which, so finstats does not either.
        Some(Timing { started_at: start, ended_at: end, duration_s: played.min(end - start) })
    } else {
        // One moment, and it is the end.
        Some(Timing { started_at: end - played, ended_at: end, duration_s: played })
    }
}

/// The file, as far as the row says anything about it at all.
fn streams_of(d: &Value) -> Streams {
    let raw = &d["rawData"];
    if raw["MediaStreams"].as_array().is_some_and(|s| !s.is_empty()) {
        return Streams::extract(&raw["MediaStreams"], &raw["PlayState"]);
    }
    let bitrate = match (int(&d["videoBitRate"]), int(&d["audioBitRate"])) {
        (None, None) => None,
        (v, a) => Some(v.unwrap_or(0) + a.unwrap_or(0)),
    };
    Streams {
        bitrate,
        video_codec: lower(&d["videoCodec"]),
        width: int(&d["resolutionWidth"]),
        height: int(&d["resolutionHeight"]),
        video_range: opt_str(&d["videoRangeType"]),
        audio_codec: lower(&d["audioCodec"]),
        audio_channels: int(&d["audioChannels"]),
        // Streamystats keeps no bit depth and no track language of its own.
        ..Streams::default()
    }
}

/// What the server was transcoding to, or `None` when the row cannot say.
fn transcode_of(d: &Value) -> Option<Value> {
    let raw = &d["rawData"]["TranscodingInfo"];
    if raw.is_object() {
        return media::compact_transcode(raw);
    }
    // A row from Jellystat whose session kept no `TranscodingInfo` knows nothing about the
    // transcode: its flat columns are the source codec and container with a placeholder reason, and
    // stored as a transcode they read as a file transcoded into itself for reasons unknown.
    if from_jellystat(d) || d["transcodingVideoCodec"].is_null() {
        return None;
    }
    media::compact_transcode(&json!({
        "VideoCodec": d["transcodingVideoCodec"],
        "AudioCodec": d["transcodingAudioCodec"],
        "Container": d["transcodingContainer"],
        "Bitrate": d["transcodingBitrate"],
        "Width": d["transcodingWidth"],
        "Height": d["transcodingHeight"],
        "AudioChannels": d["transcodingAudioChannels"],
        "IsVideoDirect": d["transcodingIsVideoDirect"],
        "IsAudioDirect": d["transcodingIsAudioDirect"],
        "HardwareAccelerationType": d["transcodingHardwareAccelerationType"],
        "TranscodeReasons": d["transcodeReasons"],
    }))
}

/// One session as a play row, or `None` when it is not a play or names no person, item or time.
fn record(conn: &Connection, d: &Value) -> Result<Option<PlayRecord>> {
    if kind(d) != Kind::Play {
        return Ok(None);
    }
    let (Some(user_id), Some(item_id), Some(t)) = (opt_str(&d["userId"]), opt_str(&d["itemId"]), timing(d)) else {
        return Ok(None);
    };
    let item_id = norm_id(&item_id);
    let series_id = opt_str(&d["seriesId"]).map(|s| norm_id(&s));
    let series_name = opt_str(&d["seriesName"]);
    let known: Option<(String, String)> = conn
        .prepare_cached("SELECT type, name FROM items WHERE id = ?1")?
        .query_row([&item_id], |r| Ok((r.get(0)?, r.get(1)?)))
        .optional()?;
    let streams = streams_of(d);
    let container = opt_str(&d["rawData"]["OriginalContainer"]).map(|c| c.split(',').next().unwrap_or_default().to_string());

    let item_type = match (&known, &series_id) {
        (Some((t, _)), _) => t.clone(),
        (None, Some(_)) => "Episode".to_string(),
        // Not an episode and not in the library. Where the row came with a session it can be read
        // the same way a Jellystat import reads one: something with video and no file container was
        // a television channel. A play Streamystats watched keeps nothing about the file, and there
        // is no honest way to tell a deleted film from a channel, so it stays unknown.
        (None, None) => match (streams.has_video(), container.is_some(), streams.audio_codec.is_some()) {
            (true, false, _) => "TvChannel",
            (true, true, _) => "Movie",
            (false, _, true) => "Audio",
            (false, _, false) => "Unknown",
        }
        .to_string(),
    };

    // A row from Jellystat names the series where the episode should be, because that is all
    // Jellystat kept. The library knows what the episode is called.
    let row_name = opt_str(&d["itemName"]).unwrap_or_else(|| "Unknown".to_string());
    let item_name = match (&known, &series_name) {
        (Some((_, name)), Some(series)) if &row_name == series => name.clone(),
        _ => row_name,
    };

    let runtime_s = ticks_to_s(&d["runtimeTicks"]).filter(|r| *r > 0);
    let position_s = runtime_s.and(ticks_to_s(&d["positionTicks"])).filter(|p| *p >= 0);
    let reported = d["playMethod"].as_str().or_else(|| d["rawData"]["PlayState"]["PlayMethod"].as_str());
    let transcode = transcode_of(d);

    Ok(Some(PlayRecord {
        source: SOURCE,
        source_id: opt_str(&d["id"]).map(|id| format!("streamystats:{id}")),
        // Nothing in a backup is playing now, whatever `isActive` says.
        active: false,
        user_id: norm_id(&user_id),
        user_name: opt_str(&d["userName"]).unwrap_or_else(|| "Unknown".to_string()),
        item_id,
        item_name,
        item_type,
        series_id,
        series_name,
        season_id: opt_str(&d["seasonId"]).map(|s| norm_id(&s)),
        // Which season and episode is the library's to say; `import::finalize` fills them in.
        season_number: None,
        episode_number: None,
        started_at: t.started_at,
        ended_at: t.ended_at,
        duration_s: t.duration_s,
        paused_s: 0,
        position_s,
        runtime_s,
        client: opt_str(&d["clientName"]),
        device_name: opt_str(&d["deviceName"]),
        device_id: opt_str(&d["deviceId"]),
        app_version: opt_str(&d["applicationVersion"]),
        remote_ip: opt_str(&d["remoteEndPoint"]),
        play_method: media::effective_play_method(reported, transcode.as_ref()),
        container,
        streams,
        transcode,
        // Streamystats keeps one row per play, so there is no timeline behind it.
        pause_count: 0,
        seek_count: 0,
        start_position_s: None,
    }))
}

/// Blocking. Reads the export at `path` and writes it in a single transaction, so a failed import
/// leaves the database exactly as it was.
///
/// The file is walked rather than loaded: everything but the session list is stepped over and each
/// session is written as it is parsed, because years of history run to hundreds of megabytes and a
/// document that size held as JSON would be several times worse.
pub fn run(db: &Db, path: &Path, tasks: Option<&Tasks>) -> Result<ImportResult> {
    let file = File::open(path).context("opening the uploaded backup")?;
    if file.metadata()?.len() == 0 {
        bail!("The file is empty");
    }
    let mut conn = db.conn()?;
    let settings = crate::state::Settings::load(&conn)?;
    // Immediate: the import will write, so it takes the write lock before its first read rather than
    // failing to upgrade later when another connection (the collector, an audit row) commits meanwhile.
    let tx = conn.transaction_with_behavior(crate::db::rusqlite::TransactionBehavior::Immediate)?;
    let mut res = ImportResult::default();
    let report = |msg: &str, p: Option<f64>| {
        if let Some(t) = tasks {
            t.update(TASK, msg, p);
        }
    };

    report("Reading backup", Some(0.02));
    // How many sessions the file says it holds, when it says so before the list itself.
    let expected = std::cell::Cell::new(0u64);
    let mut each = |row: Value| -> Result<()> {
        res.sessions_read += 1;
        session(&tx, &row, &mut res, settings.merge_window_s)?;
        if res.sessions_read % 500 == 0 {
            let (done, total) = (res.sessions_read, expected.get());
            report("Importing playback history", (total > 0).then(|| 0.95 * done as f64 / total.max(done) as f64));
        }
        Ok(())
    };
    let mut de = serde_json::Deserializer::from_reader(BufReader::with_capacity(1 << 20, file));
    let found = Doc { each: &mut each, expected: &expected }.deserialize(&mut de).map_err(|e| unexpected(path, Some(e)))?;
    if !found {
        return Err(unexpected(path, None));
    }

    report("Linking plays to your library", Some(0.96));
    crate::import::finalize(&tx)?;
    report("Saving", Some(0.99));
    tx.commit()?;
    crate::groups::detect(&mut conn, settings.group_window_s, None)?;
    conn.execute_batch("PRAGMA optimize;")?;
    Ok(res)
}

/// Say what is wrong with the file in a way an operator can act on, naming the other tracker when
/// this is plainly *its* backup in the wrong place.
fn unexpected(path: &Path, e: Option<serde_json::Error>) -> anyhow::Error {
    if is_jellystat(path) {
        return anyhow::anyhow!("That is a Jellystat backup. It goes under “Import from Jellystat”.");
    }
    match e {
        Some(e) => anyhow::anyhow!("This does not look like a Streamystats backup ({e})."),
        None => anyhow::anyhow!("This does not look like a Streamystats backup: there is no list of sessions in it."),
    }
}

/// Jellystat writes a record per line, each tagged `table` or `row`. Only the first line is read:
/// the file itself may be hundreds of megabytes.
fn is_jellystat(path: &Path) -> bool {
    let Ok(file) = File::open(path) else { return false };
    let mut first = String::new();
    if std::io::BufRead::read_line(&mut BufReader::new(file).take(4096), &mut first).is_err() {
        return false;
    }
    let looks = |k: &str| first.contains(&format!("\"{k}\""));
    looks("type") && (looks("table") || looks("row"))
}

/// One session: written, counted as already known, or counted as never having been a play.
fn session(conn: &Connection, d: &Value, res: &mut ImportResult, merge_window_s: i64) -> Result<()> {
    let Some(rec) = record(conn, d)? else {
        match kind(d) {
            Kind::MarkedWatched => res.marked_watched += 1,
            Kind::Play => res.unreadable_rows += 1,
        }
        return Ok(());
    };
    // The export names no users of its own, so the people in the history are all it can give back.
    // Anything else about them — whether they administer the server, their picture — is Jellyfin's
    // to say, and the next sync says it.
    res.users += conn
        .prepare_cached("INSERT OR IGNORE INTO users(id, name, updated_at) VALUES (?1, ?2, 0)")?
        .execute(params![rec.user_id, rec.user_name])? as u64;
    match rec.insert_imported(conn, merge_window_s)? {
        Some(_) => res.plays_imported += 1,
        None => res.plays_skipped += 1,
    }
    Ok(())
}

/// Walks the export, handing every session to `each` and stepping over the rest. Answers whether a
/// session list was there at all. A bare list of sessions is accepted as well as a whole backup.
struct Doc<'a, F> {
    each: &'a mut F,
    expected: &'a std::cell::Cell<u64>,
}

impl<'de, F: FnMut(Value) -> Result<()>> DeserializeSeed<'de> for Doc<'_, F> {
    type Value = bool;

    fn deserialize<D: serde::Deserializer<'de>>(self, de: D) -> std::result::Result<bool, D::Error> {
        de.deserialize_any(self)
    }
}

impl<'de, F: FnMut(Value) -> Result<()>> Visitor<'de> for Doc<'_, F> {
    type Value = bool;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("a Streamystats backup, or a list of its sessions")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> std::result::Result<bool, A::Error> {
        let mut found = false;
        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "sessions" => {
                    map.next_value_seed(Each { each: self.each })?;
                    found = true;
                }
                // How many there are, when the file says so before the list itself: only used to
                // tell the operator how far along the import is.
                "counts" => {
                    let counts: Value = map.next_value()?;
                    self.expected.set(counts["sessions"].as_u64().unwrap_or(0));
                }
                _ => {
                    map.next_value::<IgnoredAny>()?;
                }
            }
        }
        Ok(found)
    }

    fn visit_seq<A: SeqAccess<'de>>(self, seq: A) -> std::result::Result<bool, A::Error> {
        Each { each: self.each }.read(seq)?;
        Ok(true)
    }
}

/// The session list, one row at a time.
struct Each<'a, F> {
    each: &'a mut F,
}

impl<F: FnMut(Value) -> Result<()>> Each<'_, F> {
    fn read<'de, A: SeqAccess<'de>>(self, mut seq: A) -> std::result::Result<(), A::Error> {
        while let Some(row) = seq.next_element::<Value>()? {
            (self.each)(row).map_err(|e| serde::de::Error::custom(format!("{e:#}")))?;
        }
        Ok(())
    }
}

impl<'de, F: FnMut(Value) -> Result<()>> DeserializeSeed<'de> for Each<'_, F> {
    type Value = ();

    fn deserialize<D: serde::Deserializer<'de>>(self, de: D) -> std::result::Result<(), D::Error> {
        de.deserialize_seq(self)
    }
}

impl<'de, F: FnMut(Value) -> Result<()>> Visitor<'de> for Each<'_, F> {
    type Value = ();

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("a list of sessions")
    }

    fn visit_seq<A: SeqAccess<'de>>(self, seq: A) -> std::result::Result<(), A::Error> {
        self.read(seq)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::rusqlite::Connection;
    use serde_json::json;

    /// 2026-01-01T00:00:00Z. The fixtures below are dated from here.
    const MIDNIGHT: i64 = 1_767_225_600;

    fn conn() -> Connection {
        let c = Connection::open_in_memory().unwrap();
        for m in crate::db::MIGRATIONS {
            c.execute_batch(m).unwrap();
        }
        c
    }

    /// `json!` with this many keys at once trips the macro's recursion limit, so the fixtures are
    /// built from two halves.
    fn merge(mut into: Value, more: Value) -> Value {
        let obj = into.as_object_mut().unwrap();
        for (k, v) in more.as_object().unwrap() {
            obj.insert(k.clone(), v.clone());
        }
        into
    }

    /// A session Streamystats watched itself: it knows when the play began, and nothing at all
    /// about the file — the source columns are empty in every row a real export contains.
    fn watched_live() -> Value {
        let session = json!({
            "id": "11111111-1111-1111-1111-111111111111",
            "userId": "u1", "userName": "alice",
            "itemId": "i1", "itemName": "Big Buck Bunny",
            "deviceId": "dev-1", "deviceName": "Living room TV", "clientName": "Jellyfin Web",
            "applicationVersion": "10.11.4", "remoteEndPoint": "192.168.1.10",
            "playDuration": 600,
            "startTime": "2026-01-01T00:00:00.000Z",
            "endTime": "2026-01-01T00:12:00.000Z",
            "lastActivityDate": "2026-01-01T00:12:10.000Z",
            "runtimeTicks": 6_000_000_000i64, "positionTicks": 5_400_000_000i64, "percentComplete": 90.0,
            "completed": true, "isPaused": false, "isActive": true, "isInferred": false,
            "playMethod": "Transcode", "mediaSourceId": "i1",
            "audioStreamIndex": 1, "subtitleStreamIndex": -1,
        });
        let media = json!({
            "videoCodec": null, "audioCodec": null, "resolutionWidth": null, "resolutionHeight": null,
            "videoBitRate": null, "audioBitRate": null, "audioChannels": null, "videoRangeType": null,
            "isTranscoded": true,
            "transcodingVideoCodec": "h264", "transcodingAudioCodec": "aac", "transcodingContainer": "mp4",
            "transcodingWidth": 1280, "transcodingHeight": 720, "transcodingBitrate": 3_872_000i64,
            "transcodingAudioChannels": 2, "transcodingIsVideoDirect": false, "transcodingIsAudioDirect": false,
            "transcodingHardwareAccelerationType": "nvenc",
            "transcodeReasons": ["ContainerBitrateExceedsLimit"],
            "rawData": { "sessionKey": "sid:1", "transcodeReasons": ["ContainerBitrateExceedsLimit"] },
        });
        merge(session, media)
    }

    fn came_from_jellystat() -> Value {
        let streams = json!([
            { "Type": "Video", "Codec": "hevc", "Index": 0, "Width": 1920, "Height": 1080, "BitRate": 2_108_590i64, "BitDepth": 10, "VideoRangeType": "SDR" },
            { "Type": "Audio", "Codec": "eac3", "Index": 1, "Channels": 2, "BitRate": 128_000i64, "Language": "eng", "IsDefault": true },
            { "Type": "Subtitle", "Codec": "ass", "Index": 3, "Language": "nor", "IsDefault": true },
        ]);
        let transcoding = json!({
            "VideoCodec": "h264", "AudioCodec": "aac", "Container": "mp4", "Bitrate": 7_284_633i64,
            "Width": 1920, "Height": 1080, "AudioChannels": 2, "IsVideoDirect": false, "IsAudioDirect": false,
            "HardwareAccelerationType": "none",
            "TranscodeReasons": ["ContainerNotSupported", "VideoCodecNotSupported"],
        });
        let raw = json!({
            "ActivityDateInserted": "2026-01-02T21:20:00.000Z",
            "NowPlayingItemId": "s1", "EpisodeId": "e1",
            "NowPlayingItemName": "Big Buck Bunny", "SeriesName": "Big Buck Bunny",
            "PlaybackDuration": "1200", "OriginalContainer": "mkv",
            "PlayState": { "AudioStreamIndex": 1, "SubtitleStreamIndex": 3, "PlayMethod": "Transcode" },
            "MediaStreams": streams, "TranscodingInfo": transcoding,
        });
        let session = json!({
            "id": "22222222-2222-2222-2222-222222222222",
            "userId": "u2", "userName": "bob",
            "itemId": "e1", "itemName": "Big Buck Bunny",
            "seriesId": "s1", "seriesName": "Big Buck Bunny", "seasonId": "sea1",
            "deviceId": "dev-2", "deviceName": "Phone", "clientName": "Jellyfin Android",
            "applicationVersion": "2.6.1", "remoteEndPoint": "203.0.113.7",
            "playDuration": 1200,
            "startTime": "2026-01-02T21:20:00.000Z",
            "endTime": "2026-01-02T21:20:00.000Z",
            "lastActivityDate": "2026-01-02T21:20:00.000Z",
        });
        let media = json!({
            "runtimeTicks": null, "positionTicks": 1_382_020i64, "percentComplete": null,
            "completed": false, "isPaused": false, "isActive": true, "isInferred": false,
            "playMethod": "Transcode", "mediaSourceId": "ms1",
            "audioStreamIndex": 1, "subtitleStreamIndex": 3,
            "videoCodec": "hevc", "audioCodec": "eac3", "resolutionWidth": 1920, "resolutionHeight": 1080,
            "videoBitRate": 2_108_590i64, "audioBitRate": 128_000i64, "audioChannels": 2, "videoRangeType": "SDR",
            "isTranscoded": true,
        });
        let mut row = merge(session, media);
        // Streamystats' own transcoding columns are a copy of the *source* in a row like this, and
        // its reasons are a placeholder. The raw session it kept is what really happened.
        row["transcodingVideoCodec"] = json!("hevc");
        row["transcodingAudioCodec"] = json!("eac3");
        row["transcodingContainer"] = json!("mkv");
        row["transcodeReasons"] = json!(["Unknown"]);
        row["rawData"] = raw;
        row
    }

    /// Not a play at all: Jellyfin said the item was watched, so Streamystats wrote a row for a
    /// viewing nobody saw, the length of the whole film.
    fn marked_watched() -> Value {
        json!({
            "id": "inferred:mark-watched:1:u1:i9:2026-01-03T00:00:00.000Z",
            "userId": "u1", "userName": "alice", "itemId": "i9", "itemName": "Big Buck Bunny",
            "playDuration": 9727,
            "startTime": "2026-01-03T00:00:00.000Z", "endTime": "2026-01-03T00:00:00.000Z",
            "lastActivityDate": null, "isInferred": true, "isActive": false,
            "rawData": { "source": "mark-watched", "inferredAt": "2026-01-03T00:00:00.000Z" },
        })
    }

    #[test]
    fn a_play_streamystats_watched_keeps_the_start_it_recorded() {
        let t = timing(&watched_live()).unwrap();
        assert_eq!((t.started_at, t.ended_at, t.duration_s), (MIDNIGHT, MIDNIGHT + 720, 600));
    }

    #[test]
    fn a_play_it_imported_from_jellystat_is_dated_backwards_from_its_end() {
        // One moment only, and that moment is the end: the play ran for the twenty minutes before
        // it. Reading it as a start would move this evening twenty minutes into the future.
        let t = timing(&came_from_jellystat()).unwrap();
        let end = 1_767_388_800;
        assert_eq!((t.started_at, t.ended_at, t.duration_s), (end - 1200, end, 1200));
    }

    #[test]
    fn nothing_may_be_watched_for_longer_than_it_lasted() {
        let mut row = watched_live();
        row["playDuration"] = json!(99_999);
        assert_eq!(timing(&row).unwrap().duration_s, 720);
    }

    #[test]
    fn a_row_marked_watched_is_not_a_play() {
        let c = conn();
        assert!(matches!(kind(&marked_watched()), Kind::MarkedWatched));
        assert!(record(&c, &marked_watched()).unwrap().is_none());
        // Its two neighbours are.
        assert!(record(&c, &watched_live()).unwrap().is_some());
        assert!(record(&c, &came_from_jellystat()).unwrap().is_some());
    }

    #[test]
    fn a_running_session_in_a_backup_is_history_not_a_live_play() {
        let c = conn();
        // Every row in a real export says `isActive: true`; none of them is playing now.
        let rec = record(&c, &watched_live()).unwrap().unwrap();
        assert!(!rec.active);
        assert_eq!(rec.source, "streamystats");
        assert_eq!(rec.source_id.as_deref(), Some("streamystats:11111111-1111-1111-1111-111111111111"));
    }

    #[test]
    fn the_source_file_of_a_play_streamystats_watched_is_not_invented() {
        let c = conn();
        let rec = record(&c, &watched_live()).unwrap().unwrap();
        // It recorded nothing about the file, so finstats claims nothing about it.
        assert_eq!(rec.streams.video_codec, None);
        assert_eq!(rec.streams.width, None);
        assert_eq!(rec.streams.audio_codec, None);
        assert_eq!(rec.streams.bitrate, None);
        assert_eq!(rec.container, None);
        // What it did record is the transcode it was serving, and where playback stopped.
        assert_eq!(rec.transcode.as_ref().unwrap()["video_codec"], json!("h264"));
        assert_eq!(rec.transcode.as_ref().unwrap()["container"], json!("mp4"));
        assert_eq!(rec.transcode.as_ref().unwrap()["hw_accel"], json!("nvenc"));
        assert_eq!(rec.transcode.as_ref().unwrap()["reasons"], json!(["ContainerBitrateExceedsLimit"]));
        assert_eq!((rec.position_s, rec.runtime_s), (Some(540), Some(600)));
    }

    #[test]
    fn a_play_it_imported_from_jellystat_reads_the_session_jellystat_kept() {
        let c = conn();
        let rec = record(&c, &came_from_jellystat()).unwrap().unwrap();
        assert_eq!(rec.streams.video_codec.as_deref(), Some("hevc"));
        assert_eq!((rec.streams.width, rec.streams.height), (Some(1920), Some(1080)));
        assert_eq!(rec.streams.bit_depth, Some(10));
        assert_eq!(rec.streams.audio_language.as_deref(), Some("eng"));
        assert_eq!(rec.streams.subtitle_language.as_deref(), Some("nor"));
        assert_eq!(rec.container.as_deref(), Some("mkv"));
        // The transcode is the one the session reported, not Streamystats' copy of the source.
        let t = rec.transcode.unwrap();
        assert_eq!(t["video_codec"], json!("h264"));
        assert_eq!(t["container"], json!("mp4"));
        assert_eq!(t["reasons"], json!(["ContainerNotSupported", "VideoCodecNotSupported"]));
        // Jellystat's position is the one it happened to catch, not where playback stopped, and it
        // kept no runtime for the position to be a position in.
        assert_eq!((rec.position_s, rec.runtime_s), (None, None));
    }

    #[test]
    fn a_transcode_streamystats_cannot_describe_is_not_described() {
        let c = conn();
        let mut row = came_from_jellystat();
        // A third of the rows from Jellystat are like this: the session it kept has no
        // `TranscodingInfo`, while Streamystats' own columns say the file was transcoded into
        // itself, for reasons unknown. That is not a transcode, it is a gap.
        row["rawData"]["TranscodingInfo"] = json!(null);
        let rec = record(&c, &row).unwrap().unwrap();
        assert!(rec.transcode.is_none());
        assert_eq!(rec.play_method, "Transcode");
    }

    #[test]
    fn an_episode_takes_its_series_from_the_row_and_its_own_name_from_the_library() {
        let c = conn();
        c.execute_batch(
            "INSERT INTO items(id, type, name, series_id, series_name, index_number, parent_index_number, updated_at)
               VALUES ('e1', 'Episode', 'The Big Meadow', 's1', 'Big Buck Bunny', 3, 1, 0);",
        )
        .unwrap();
        let rec = record(&c, &came_from_jellystat()).unwrap().unwrap();
        assert_eq!(rec.item_type, "Episode");
        assert_eq!(rec.item_id, "e1");
        assert_eq!(rec.series_id.as_deref(), Some("s1"));
        assert_eq!(rec.season_id.as_deref(), Some("sea1"));
        // Jellystat kept only the series name, so the row's own `itemName` is the series again.
        // The library knows what the episode is called.
        assert_eq!(rec.item_name, "The Big Meadow");
    }

    #[test]
    fn an_item_the_library_knows_is_typed_by_the_library() {
        let c = conn();
        c.execute_batch("INSERT INTO items(id, type, name, updated_at) VALUES ('i1', 'Audio', 'Big Buck Bunny', 0);").unwrap();
        assert_eq!(record(&c, &watched_live()).unwrap().unwrap().item_type, "Audio");
    }

    #[test]
    fn an_item_nobody_can_name_is_left_unknown_rather_than_guessed() {
        let c = conn();
        // Not an episode, not in the library, and Streamystats kept nothing about the file: there
        // is no honest way to tell a deleted film from a television channel.
        assert_eq!(record(&c, &watched_live()).unwrap().unwrap().item_type, "Unknown");
    }

    #[test]
    fn a_row_without_a_person_or_an_item_or_a_time_is_no_use() {
        let c = conn();
        for missing in ["userId", "itemId", "startTime"] {
            let mut row = watched_live();
            row[missing] = json!(null);
            assert!(record(&c, &row).unwrap().is_none(), "{missing}");
        }
    }

    fn write(name: &str, text: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("finstats-streamystats-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(name);
        std::fs::write(&path, text).unwrap();
        path
    }

    fn export_file() -> String {
        let doc = json!({
            "exportInfo": { "timestamp": "2026-01-04T00:00:00.000Z", "serverName": "Home", "version": "streamystats", "exportType": "backup" },
            "counts": { "sessions": 3 },
            "server": { "id": 1, "name": "Home", "url": "http://jellyfin.example:8096" },
            "sessions": [watched_live(), came_from_jellystat(), marked_watched()],
            "hiddenRecommendations": [],
        });
        doc.to_string()
    }

    /// Its progress belongs on its own card: the Jellystat card watches `import` and keeps its last result.
    #[test]
    fn progress_is_reported_on_the_streamystats_task_and_leaves_jellystats_alone() {
        let db = Db::open_in_memory().unwrap();
        let tasks = crate::state::Tasks::new();
        assert!(tasks.try_start("import_streamystats", "Receiving backup"));
        run(&db, &write("progress.json", &export_file()), Some(&tasks)).unwrap();
        let task = |id: &str| tasks.snapshot().into_iter().find(|t| t.id == id).unwrap();
        assert!(task("import").message.is_none(), "the Jellystat card was written to: {:?}", task("import").message);
        assert_ne!(task("import_streamystats").message.as_deref(), Some("Receiving backup"), "its own card never moved");
    }

    #[test]
    fn a_whole_export_becomes_history_and_the_people_it_names() {
        let db = Db::open_in_memory().unwrap();
        let path = write("backup.json", &export_file());
        let r = run(&db, &path, None).unwrap();
        assert_eq!((r.sessions_read, r.plays_imported, r.plays_skipped, r.marked_watched), (3, 2, 0, 1));
        // Two people appear in the history; nothing else in the file is library data.
        assert_eq!(r.users, 2);
        // One connection at a time: take and release it per question, so the second import below
        // can have it.
        let n = |sql: &str| -> i64 { db.conn().unwrap().query_row(sql, [], |r| r.get(0)).unwrap() };
        assert_eq!(n("SELECT COUNT(*) FROM playbacks WHERE source = 'streamystats'"), 2);
        assert_eq!(n("SELECT COUNT(*) FROM playbacks WHERE active = 1"), 0);
        assert_eq!(n("SELECT COUNT(*) FROM items"), 0);
        // alice's evening is where Streamystats saw it start, bob's twenty minutes before its end.
        assert_eq!(n("SELECT started_at FROM playbacks WHERE user_name = 'alice'"), MIDNIGHT);
        assert_eq!(n("SELECT started_at FROM playbacks WHERE user_name = 'bob'"), 1_767_388_800 - 1200);
        // alice watched from home.
        assert_eq!(n("SELECT is_local FROM playbacks WHERE user_name = 'alice'"), 1);

        // The same file again changes nothing.
        let again = run(&db, &path, None).unwrap();
        assert_eq!((again.plays_imported, again.plays_skipped), (0, 2));
        assert_eq!(n("SELECT COUNT(*) FROM playbacks"), 2);
    }

    #[test]
    fn an_evening_the_history_already_holds_is_not_imported_twice() {
        let db = Db::open_in_memory().unwrap();
        db.conn()
            .unwrap()
            .execute_batch(
                "INSERT INTO playbacks(source, user_id, user_name, item_id, item_name, item_type, started_at, ended_at, duration_s)
                   VALUES ('live', 'u1', 'alice', 'i1', 'Big Buck Bunny', 'Movie', 1767225645, 1767226320, 600);",
            )
            .unwrap();
        let r = run(&db, &write("overlap.json", &export_file()), None).unwrap();
        // alice's evening was already there, forty-five seconds off. Only bob's is new.
        assert_eq!((r.plays_imported, r.plays_skipped), (1, 1));
    }

    #[test]
    fn a_bare_list_of_sessions_is_a_backup_too() {
        let db = Db::open_in_memory().unwrap();
        let path = write("list.json", &json!([watched_live()]).to_string());
        assert_eq!(run(&db, &path, None).unwrap().plays_imported, 1);
    }

    #[test]
    fn a_file_from_somewhere_else_is_refused_by_name() {
        let db = Db::open_in_memory().unwrap();
        let jellystat = write("jellystat.jsonl", "{\"type\":\"table\",\"table\":\"jf_users\"}\n");
        let msg = run(&db, &jellystat, None).unwrap_err().to_string();
        assert!(msg.contains("Jellystat"), "{msg}");

        let empty = write("empty.json", "");
        assert!(run(&db, &empty, None).unwrap_err().to_string().contains("empty"));

        let other = write("other.json", "{\"hello\": 1}");
        let msg = run(&db, &other, None).unwrap_err().to_string();
        assert!(msg.contains("Streamystats"), "{msg}");
    }
}
