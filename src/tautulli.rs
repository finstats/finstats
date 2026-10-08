//! Import of Tautulli history: what Plex played, brought over to Jellyfin.
//!
//! A Tautulli backup is its own SQLite database (often zipped by the download). What it means, learned from a real one:
//! - **People are Plex's, not Jellyfin's.** Names do not line up, so nothing is guessed: the owner wires each Plex user
//!   to a Jellyfin user by hand (Settings → Import), several Plex users may go into one Jellyfin user, and a Plex user
//!   without a wire is not imported at all.
//! - **Titles are Plex's too**: every id is `plex://…` or `local://…`, with no TMDB, TVDB or IMDb id beside it. A play
//!   is written under a stand-in id (`plex:<rating key>`) with the names Jellyfin would give it, and `relink` attaches
//!   it the way it attaches a renamed title: a film by its title and year (written as `Title (2008)` so the year
//!   travels), an episode by its show and its season and episode number. One that is not on the server stays a play
//!   of that name, and is attached whenever the title arrives.
//! - **One viewing can be several rows**: a resumed play continues the row it resumed, and every row of it carries
//!   the first one's id in `reference_id`. Each viewing is one play here: from its first start to its last stop, the
//!   time it ran without the time it was paused (`paused_counter`), stopped where its last row left off.
//! - `view_offset` is where playback was when the row ended (milliseconds) and counts only beside the runtime
//!   (`duration`) it is a position in. `paused_counter` is seconds.
//! - **Music is not imported** (the owner's decision): a track's name alone does not say which track it is.
//! - `users` holds every Plex user's access tokens and e-mail address. They are never read: the board is built from
//!   ids, names and play counts only, and the uploaded file is removed after the import, on cancel and at start-up.

use std::collections::{HashMap, HashSet};
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::db::rusqlite::{Connection, OpenFlags};
#[cfg(test)]
use crate::db::rusqlite::params;
use crate::db::{Db, rusqlite};
use crate::media::Streams;
use crate::playback::PlayRecord;
use crate::state::Tasks;

pub const TASK: &str = "import_tautulli";

/// What a play from Tautulli says it came from.
pub const SOURCE: &str = "tautulli";

/// Where an uploaded backup waits for its wires, in the data folder.
pub const UPLOAD: &str = "tautulli-upload.db";

/// A Plex user as the board shows them.
#[derive(Debug, Serialize, PartialEq)]
pub struct PlexUser {
    pub id: i64,
    pub name: String,
    /// Viewings that would be imported: films and episodes.
    pub plays: i64,
    pub first_at: Option<i64>,
    pub last_at: Option<i64>,
}

/// One wire: a Plex user into a Jellyfin user.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Wire {
    pub plex_user_id: i64,
    pub jellyfin_user_id: String,
}

#[derive(Debug, Default, Serialize, Clone, PartialEq)]
pub struct ImportResult {
    pub plays_imported: u64,
    pub plays_skipped: u64,
    /// Viewings of Plex users who got no wire.
    pub not_wired: u64,
    /// Viewings of anything but a film or an episode (music, clips, photos, Live TV), which are not imported.
    pub other_media: u64,
    pub users_wired: u64,
}

/// If the upload is a zip (Tautulli's download is), put the one database inside in its place. A file that is not a
/// zip is left as it is, for `open` to judge.
pub fn unpack(path: &Path) -> Result<()> {
    let mut file = std::fs::File::open(path).context("opening the uploaded backup")?;
    let mut magic = [0u8; 4];
    if file.read(&mut magic)? < 4 || magic != *b"PK\x03\x04" {
        return Ok(());
    }
    let entry = zip_entry(&mut file)?;
    file.seek(SeekFrom::Start(entry.local_at))?;
    let mut header = [0u8; 30];
    file.read_exact(&mut header)?;
    let skip = u16_at(&header, 26) as u64 + u16_at(&header, 28) as u64;
    file.seek(SeekFrom::Current(skip as i64))?;
    let packed = (&mut file).take(entry.packed);
    let unpacked = path.with_extension("unpacking");
    let written = {
        let mut out = std::fs::File::create(&unpacked).context("unpacking the backup")?;
        match entry.method {
            0 => std::io::copy(&mut { packed }, &mut out),
            8 => std::io::copy(&mut flate2::read::DeflateDecoder::new(packed), &mut out),
            m => {
                let _ = std::fs::remove_file(&unpacked);
                bail!("The zip is packed in a way FinStats cannot open (method {m}). Unzip it and upload the .db file inside.");
            }
        }
    };
    match written {
        Ok(n) if n == entry.size => Ok(std::fs::rename(&unpacked, path)?),
        Ok(_) | Err(_) => {
            let _ = std::fs::remove_file(&unpacked);
            bail!("The zip is damaged. Download the Tautulli backup again.")
        }
    }
}

/// The one database in a zip, from its central directory.
struct ZipEntry {
    local_at: u64,
    packed: u64,
    size: u64,
    method: u16,
}

fn u16_at(b: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([b[at], b[at + 1]])
}

fn u32_at(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]])
}

fn zip_entry(file: &mut std::fs::File) -> Result<ZipEntry> {
    let len = file.metadata()?.len();
    // The directory's end record is at most 64 KiB of comment from the end of the file.
    let tail_len = len.min(22 + 65_535);
    file.seek(SeekFrom::Start(len - tail_len))?;
    let mut tail = vec![0u8; tail_len as usize];
    file.read_exact(&mut tail)?;
    let Some(end) = (0..tail.len().saturating_sub(21)).rev().find(|&i| u32_at(&tail, i) == 0x0605_4b50) else {
        bail!("The zip is damaged. Download the Tautulli backup again.");
    };
    let (count, dir_len, dir_at) = (u16_at(&tail, end + 10), u32_at(&tail, end + 12), u32_at(&tail, end + 16));
    file.seek(SeekFrom::Start(dir_at as u64))?;
    let mut dir = vec![0u8; dir_len as usize];
    file.read_exact(&mut dir).context("reading the zip")?;
    let (mut at, mut found) = (0usize, vec![]);
    for _ in 0..count {
        if at + 46 > dir.len() || u32_at(&dir, at) != 0x0201_4b50 {
            bail!("The zip is damaged. Download the Tautulli backup again.");
        }
        let name_len = u16_at(&dir, at + 28) as usize;
        let name = String::from_utf8_lossy(&dir[at + 46..at + 46 + name_len]).to_string();
        let base = name.rsplit('/').next().unwrap_or_default();
        if name.to_lowercase().ends_with(".db") && !name.starts_with("__MACOSX") && !base.starts_with("._") {
            found.push(ZipEntry { method: u16_at(&dir, at + 10), packed: u32_at(&dir, at + 20) as u64, size: u32_at(&dir, at + 24) as u64, local_at: u32_at(&dir, at + 42) as u64 });
        }
        at += 46 + name_len + u16_at(&dir, at + 30) as usize + u16_at(&dir, at + 32) as usize;
    }
    match found.len() {
        1 => Ok(found.pop().expect("one")),
        0 => bail!("There is no Tautulli database in this zip. Upload the .db file, or the zip Tautulli's backup download gives."),
        _ => bail!("This zip holds more than one database. Unzip it and upload the Tautulli one."),
    }
}

/// Remove an uploaded backup (and what an interrupted upload or unpacking left) once it is older than `older_than`:
/// it holds every Plex user's access tokens, so it is kept only while it waits for its wires. `true` when anything went.
pub fn sweep(data_dir: &Path, older_than: std::time::Duration) -> bool {
    let stem = UPLOAD.trim_end_matches(".db");
    let mut swept = false;
    for name in [UPLOAD.to_string(), format!("{stem}.part"), format!("{stem}.unpacking")] {
        let path = data_dir.join(name);
        let old = std::fs::metadata(&path).and_then(|m| m.modified()).is_ok_and(|at| at.elapsed().unwrap_or_default() >= older_than);
        if old && std::fs::remove_file(&path).is_ok() {
            swept = true;
        }
    }
    swept
}

/// The backup, read-only, once it is plainly a Tautulli database.
pub fn open(path: &Path) -> Result<Connection> {
    let not_one = || anyhow::anyhow!("This does not look like a Tautulli backup. Upload the .db file (or its zip) from Tautulli's backups.");
    let t = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX).map_err(|_| not_one())?;
    let tables: HashSet<String> = t
        .prepare("SELECT name FROM sqlite_master WHERE type = 'table'")
        .and_then(|mut s| s.query_map([], |r| r.get(0))?.collect())
        .map_err(|_| not_one())?;
    if !["session_history", "session_history_metadata", "users"].iter().all(|n| tables.contains(*n)) {
        return Err(not_one());
    }
    Ok(t)
}

/// What the board needs, and nothing else from `users`: never a token, never an address.
const PLEX_USERS_SQL: &str = "WITH people AS (SELECT user_id FROM users UNION SELECT DISTINCT user_id FROM session_history WHERE user_id IS NOT NULL)
    SELECT CAST(p.user_id AS INTEGER),
           COALESCE(NULLIF(TRIM(u.friendly_name), ''), NULLIF(TRIM(u.username), ''), (SELECT MAX(h.user) FROM session_history h WHERE h.user_id = p.user_id), 'Plex user ' || p.user_id) AS name,
           (SELECT COUNT(DISTINCT h.reference_id) FROM session_history h WHERE h.user_id = p.user_id AND h.media_type IN ('movie', 'episode')),
           (SELECT MIN(h.started) FROM session_history h WHERE h.user_id = p.user_id),
           (SELECT MAX(h.stopped) FROM session_history h WHERE h.user_id = p.user_id)
    FROM people p LEFT JOIN users u ON u.user_id = p.user_id
    ORDER BY name COLLATE NOCASE, p.user_id";

/// Every Plex user the backup knows, with how many viewings of theirs would come in. Never a token or an address.
pub fn plex_users(t: &Connection) -> Result<Vec<PlexUser>> {
    let rows = t
        .prepare(PLEX_USERS_SQL)?
        .query_map([], |r| Ok(PlexUser { id: r.get(0)?, name: r.get(1)?, plays: r.get(2)?, first_at: r.get(3)?, last_at: r.get(4)? }))?
        .collect::<Result<_, _>>()?;
    Ok(rows)
}

/// The wires as a map from Plex user to Jellyfin user, or what is wrong with them.
pub fn check_wires(wires: &[Wire], plex: &HashSet<i64>, jellyfin: &HashSet<String>) -> Result<HashMap<i64, String>, String> {
    if wires.is_empty() {
        return Err("Connect at least one Plex user to a Jellyfin user".into());
    }
    let mut map = HashMap::new();
    for w in wires {
        if !plex.contains(&w.plex_user_id) {
            return Err(format!("The backup has no Plex user {}", w.plex_user_id));
        }
        if !jellyfin.contains(&w.jellyfin_user_id) {
            return Err("One of the wires ends at somebody this server does not have".into());
        }
        if map.insert(w.plex_user_id, w.jellyfin_user_id.clone()).is_some() {
            return Err("A Plex user can be connected to one Jellyfin user only".into());
        }
    }
    Ok(map)
}

/// The board: every Plex user in the backup, and every Jellyfin user they can be wired to.
pub fn board(db: &Db, path: &Path) -> Result<Value> {
    let plex = plex_users(&open(path)?)?;
    let jellyfin: Vec<Value> = db
        .conn()?
        .prepare("SELECT id, name, image_tag IS NOT NULL FROM users WHERE removed = 0 ORDER BY name COLLATE NOCASE, id")?
        .query_map([], |r| Ok(json!({ "id": r.get::<_, String>(0)?, "name": r.get::<_, String>(1)?, "has_image": r.get::<_, bool>(2)? })))?
        .collect::<Result<_, _>>()?;
    Ok(json!({ "plex_users": plex, "jellyfin_users": jellyfin }))
}

/// Plex user → Jellyfin user, and every Jellyfin user's name.
type Wired = (HashMap<i64, String>, HashMap<String, String>);

/// The wires checked against who is really there (the backup's Plex users and this server's Jellyfin users), with the
/// Jellyfin names the plays will carry; or, as the inner error, what is wrong with them.
fn wired(conn: &Connection, t: &Connection, wires: &[Wire]) -> Result<Result<Wired, String>> {
    let plex: HashSet<i64> = plex_users(t)?.into_iter().map(|u| u.id).collect();
    let names: HashMap<String, String> =
        conn.prepare("SELECT id, name FROM users WHERE removed = 0")?.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?.collect::<Result<_, _>>()?;
    Ok(check_wires(wires, &plex, &names.keys().cloned().collect()).map(|map| (map, names)))
}

/// Whether these wires can be imported from the backup at `path`, before anything starts: the inner error says why not.
pub fn check(db: &Db, path: &Path, wires: &[Wire]) -> Result<Result<(), String>> {
    Ok(wired(&*db.conn()?, &open(path)?, wires)?.map(|_| ()))
}

/// One row of `session_history`, with its metadata and its media info.
#[derive(Debug, Default)]
struct Session {
    id: i64,
    reference_id: i64,
    started: i64,
    stopped: i64,
    paused: i64,
    user_id: i64,
    media_type: String,
    rating_key: Option<i64>,
    show_key: Option<i64>,
    view_offset_ms: Option<i64>,
    ip: Option<String>,
    player: Option<String>,
    product: Option<String>,
    product_version: Option<String>,
    machine_id: Option<String>,
    title: Option<String>,
    show: Option<String>,
    season: Option<i64>,
    episode: Option<i64>,
    year: Option<i64>,
    duration_ms: Option<i64>,
    live: bool,
    decision: Option<String>,
    container: Option<String>,
    streams: Streams,
    transcode: Transcode,
}

#[derive(Debug, Default)]
struct Transcode {
    container: Option<String>,
    video_codec: Option<String>,
    audio_codec: Option<String>,
    audio_channels: Option<i64>,
    width: Option<i64>,
    height: Option<i64>,
    hw: Option<String>,
    video_decision: Option<String>,
    audio_decision: Option<String>,
    bitrate: Option<i64>,
}

/// The columns read, in order. Tautulli has added columns over the years, so anything but the few every version has is
/// read only where the backup has it, and is empty otherwise.
const COLUMNS: [(&str, &str); 42] = [
    ("h", "reference_id"), ("h", "started"), ("h", "stopped"), ("h", "paused_counter"), ("h", "user_id"), ("h", "media_type"),
    ("h", "rating_key"), ("h", "grandparent_rating_key"), ("h", "view_offset"), ("h", "ip_address"), ("h", "player"), ("h", "product"),
    ("h", "product_version"), ("h", "machine_id"),
    ("m", "title"), ("m", "grandparent_title"), ("m", "parent_media_index"), ("m", "media_index"), ("m", "year"), ("m", "duration"), ("m", "live"),
    ("i", "transcode_decision"), ("i", "container"), ("i", "bitrate"), ("i", "video_codec"), ("i", "width"), ("i", "height"),
    ("i", "video_dynamic_range"), ("i", "video_bit_depth"), ("i", "audio_codec"), ("i", "audio_channels"), ("i", "audio_language_code"),
    ("i", "subtitle_codec"), ("i", "subtitle_language"),
    ("i", "transcode_container"), ("i", "transcode_video_codec"), ("i", "transcode_audio_codec"), ("i", "transcode_audio_channels"),
    ("i", "transcode_width"), ("i", "transcode_height"), ("i", "transcode_hw_encode_title"), ("i", "stream_video_decision"),
];

fn sessions_sql(t: &Connection) -> Result<String> {
    let table = |alias: &str| match alias {
        "h" => "session_history",
        "m" => "session_history_metadata",
        _ => "session_history_media_info",
    };
    let mut have: HashSet<(String, String)> = HashSet::new();
    for alias in ["h", "m", "i"] {
        let cols: Vec<String> = t.prepare(&format!("SELECT name FROM pragma_table_info('{}')", table(alias)))?.query_map([], |r| r.get(0))?.collect::<Result<_, _>>()?;
        have.extend(cols.into_iter().map(|c| (alias.to_string(), c)));
    }
    let mut picked: Vec<String> = COLUMNS.iter().map(|(a, c)| if have.contains(&(a.to_string(), c.to_string())) { format!("{a}.{c}") } else { "NULL".into() }).collect();
    picked.push(if have.contains(&("i".into(), "stream_audio_decision".into())) { "i.stream_audio_decision".into() } else { "NULL".into() });
    picked.push(if have.contains(&("i".into(), "stream_bitrate".into())) { "i.stream_bitrate".into() } else { "NULL".into() });
    picked.push("h.id".into());
    let info = if have.iter().any(|(a, _)| a == "i") { "LEFT JOIN session_history_media_info i ON i.id = h.id" } else { "LEFT JOIN (SELECT NULL AS id) i ON 0" };
    Ok(format!(
        "SELECT {} FROM session_history h LEFT JOIN session_history_metadata m ON m.id = h.id {info}
         WHERE h.reference_id IS NOT NULL AND h.started IS NOT NULL AND h.stopped IS NOT NULL
         ORDER BY h.reference_id, h.media_type, h.rating_key, h.started, h.id",
        picked.join(", ")
    ))
}

/// A number as Tautulli keeps it: an integer, or text (an empty string where it has none, and some numbers written
/// out), and nothing else. A strict read failed the whole import on the first film without a show number.
fn num(r: &rusqlite::Row, i: usize) -> rusqlite::Result<Option<i64>> {
    use rusqlite::types::ValueRef;
    Ok(match r.get_ref(i)? {
        ValueRef::Integer(n) => Some(n),
        ValueRef::Real(f) => Some(f as i64),
        ValueRef::Text(t) => std::str::from_utf8(t).ok().map(str::trim).and_then(|t| t.parse::<i64>().ok().or_else(|| t.parse::<f64>().ok().map(|f| f as i64))),
        ValueRef::Null | ValueRef::Blob(_) => None,
    })
}

fn session_of(r: &rusqlite::Row) -> rusqlite::Result<Session> {
    use rusqlite::types::ValueRef;
    // Text as leniently as numbers: a version or a container written as a number is still what it says.
    let text = |i: usize| -> rusqlite::Result<Option<String>> {
        Ok(match r.get_ref(i)? {
            ValueRef::Text(t) => Some(String::from_utf8_lossy(t).trim().to_string()),
            ValueRef::Integer(n) => Some(n.to_string()),
            ValueRef::Real(f) => Some(f.to_string()),
            ValueRef::Null | ValueRef::Blob(_) => None,
        }
        .filter(|s| !s.is_empty()))
    };
    Ok(Session {
        id: num(r, 44)?.unwrap_or_default(),
        reference_id: num(r, 0)?.unwrap_or_default(),
        started: num(r, 1)?.unwrap_or_default(),
        stopped: num(r, 2)?.unwrap_or_default(),
        paused: num(r, 3)?.unwrap_or(0).max(0),
        user_id: num(r, 4)?.unwrap_or(-1),
        media_type: text(5)?.unwrap_or_default(),
        rating_key: num(r, 6)?,
        show_key: num(r, 7)?,
        view_offset_ms: num(r, 8)?,
        ip: text(9)?,
        player: text(10)?,
        product: text(11)?,
        product_version: text(12)?,
        machine_id: text(13)?,
        title: text(14)?,
        show: text(15)?,
        season: num(r, 16)?,
        episode: num(r, 17)?,
        year: num(r, 18)?,
        duration_ms: num(r, 19)?,
        live: num(r, 20)?.unwrap_or(0) != 0,
        decision: text(21)?,
        container: text(22)?,
        streams: Streams {
            bitrate: num(r, 23)?.map(|kbps| kbps * 1000),
            video_codec: text(24)?,
            width: num(r, 25)?,
            height: num(r, 26)?,
            video_range: text(27)?,
            bit_depth: num(r, 28)?,
            audio_codec: text(29)?,
            audio_channels: num(r, 30)?,
            audio_language: text(31)?,
            subtitle_codec: text(32)?,
            subtitle_language: text(33)?,
        },
        transcode: Transcode {
            container: text(34)?,
            video_codec: text(35)?,
            audio_codec: text(36)?,
            audio_channels: num(r, 37)?,
            width: num(r, 38)?,
            height: num(r, 39)?,
            hw: text(40)?,
            video_decision: text(41)?,
            audio_decision: text(42)?,
            bitrate: num(r, 43)?.map(|kbps| kbps * 1000),
        },
    })
}

/// How Plex served it, as Jellyfin would have said it, and through the same remux rule as a live play.
fn served(s: &Session) -> (String, Option<Value>) {
    let reported = match s.decision.as_deref() {
        Some("transcode") => "Transcode",
        Some("copy") => "DirectStream",
        _ => "DirectPlay",
    };
    if reported == "DirectPlay" {
        return (reported.into(), None);
    }
    let direct = |d: &Option<String>| matches!(d.as_deref(), Some("copy" | "direct play"));
    let t = &s.transcode;
    let transcode = json!({
        "video_codec": t.video_codec, "audio_codec": t.audio_codec, "container": t.container,
        "is_video_direct": direct(&t.video_decision), "is_audio_direct": direct(&t.audio_decision),
        "hw_accel": t.hw, "reasons": [], "bitrate": t.bitrate, "width": t.width, "height": t.height, "audio_channels": t.audio_channels,
    });
    (crate::media::effective_play_method(Some(reported), Some(&transcode)), Some(transcode))
}

/// One viewing (the rows of one `reference_id` and one title) as a play of `user` (Jellyfin id, Jellyfin name).
fn record(rows: &[Session], user: (&str, &str)) -> PlayRecord {
    let (head, last) = (&rows[0], rows.iter().max_by_key(|s| s.stopped).expect("a viewing has a row"));
    let runtime_s = head.duration_ms.filter(|d| *d > 0).map(|d| d / 1000);
    let position_s = runtime_s.and(last.view_offset_ms).filter(|o| *o > 0).map(|o| o / 1000);
    let title = head.title.clone().unwrap_or_else(|| "Unknown".into());
    let (item_type, item_name, series_id, series_name) = match head.media_type.as_str() {
        "episode" => ("Episode", title, head.show_key.map(|k| format!("plex:{k}")), head.show.clone()),
        // The year travels in the name, for `relink` to read: remakes share titles.
        _ => ("Movie", head.year.map_or(title.clone(), |y| format!("{title} ({y})")), None, None),
    };
    let (play_method, transcode) = served(head);
    PlayRecord {
        source: SOURCE,
        // The viewing's first row, which is its reference for every chain but one Plex began with a theme song.
        source_id: Some(format!("tautulli:{}", head.id)),
        active: false,
        user_id: user.0.to_string(),
        user_name: user.1.to_string(),
        item_id: format!("plex:{}", head.rating_key.unwrap_or(head.reference_id)),
        item_name,
        item_type: item_type.into(),
        series_id,
        series_name,
        season_id: None,
        season_number: head.season.filter(|_| item_type == "Episode"),
        episode_number: head.episode.filter(|_| item_type == "Episode"),
        started_at: head.started,
        ended_at: last.stopped,
        duration_s: rows.iter().map(|s| (s.stopped - s.started - s.paused).max(0)).sum(),
        paused_s: rows.iter().map(|s| s.paused.min((s.stopped - s.started).max(0))).sum(),
        position_s,
        runtime_s,
        client: head.product.clone(),
        device_name: head.player.clone(),
        device_id: head.machine_id.clone(),
        app_version: head.product_version.clone(),
        remote_ip: head.ip.clone(),
        play_method,
        container: head.container.clone(),
        streams: head.streams.clone(),
        transcode,
        pause_count: 0,
        seek_count: 0,
        start_position_s: None,
    }
}

/// Blocking. Imports the viewings of the wired Plex users in one transaction, so a failed import leaves the database
/// exactly as it was.
pub fn run(db: &Db, path: &Path, wires: &[Wire], tasks: Option<&Tasks>) -> Result<ImportResult> {
    let report = |msg: &str, p: Option<f64>| {
        if let Some(t) = tasks {
            t.update(TASK, msg, p);
        }
    };
    report("Reading the backup", Some(0.02));
    let t = open(path)?;
    let total: i64 = t.query_row("SELECT COUNT(DISTINCT reference_id) FROM session_history", [], |r| r.get(0))?;

    let mut conn = db.conn()?;
    let settings = crate::state::Settings::load(&conn)?;
    // Immediate: the import will write, so it takes the write lock before its first read rather than failing to
    // upgrade later when another connection (the collector, an audit row) commits meanwhile.
    let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    let (map, names) = wired(&tx, &t, wires)?.map_err(|e| anyhow::anyhow!(e))?;
    let mut res = ImportResult { users_wired: map.len() as u64, ..Default::default() };

    let mut stmt = t.prepare(&sessions_sql(&t)?)?;
    let mut rows = stmt.query([])?;
    let (mut viewing, mut seen): (Vec<Session>, u64) = (vec![], 0);
    let mut take = |viewing: &mut Vec<Session>, res: &mut ImportResult| -> Result<()> {
        let Some(head) = viewing.first() else { return Ok(()) };
        seen += 1;
        match map.get(&head.user_id) {
            None => res.not_wired += 1,
            Some(_) if !matches!(head.media_type.as_str(), "movie" | "episode") || head.live => res.other_media += 1,
            Some(id) => match record(viewing, (id, &names[id])).insert_imported(&tx, settings.merge_window_s)? {
                Some(_) => res.plays_imported += 1,
                None => res.plays_skipped += 1,
            },
        }
        viewing.clear();
        if seen % 200 == 0 {
            report("Importing playback history", Some(0.05 + 0.9 * seen as f64 / total.max(1) as f64));
        }
        Ok(())
    };
    while let Some(r) = rows.next()? {
        let s = session_of(r)?;
        // One viewing is one reference and one title: Plex plays a show's theme while the show is open, and Tautulli
        // chains the episode that follows onto it.
        if viewing.first().is_some_and(|v| (v.reference_id, &v.media_type, v.rating_key) != (s.reference_id, &s.media_type, s.rating_key)) {
            take(&mut viewing, &mut res)?;
        }
        viewing.push(s);
    }
    take(&mut viewing, &mut res)?;
    drop(rows);

    report("Linking plays to your library", Some(0.96));
    crate::import::finalize(&tx)?;
    report("Saving", Some(0.99));
    tx.commit()?;
    crate::groups::detect(&mut conn, settings.group_window_s, None)?;
    conn.execute_batch("PRAGMA optimize;")?;
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// A Tautulli database as a backup holds one, with the columns this reads and the secrets it must leave alone.
    fn backup(dir: &Path) -> PathBuf {
        let path = dir.join("tautulli.db");
        let c = Connection::open(&path).unwrap();
        c.execute_batch(
            "CREATE TABLE version_info (key TEXT UNIQUE, value TEXT);
             INSERT INTO version_info VALUES ('version', 'v2.18.2');
             CREATE TABLE users (id INTEGER PRIMARY KEY, user_id INTEGER UNIQUE, username TEXT, friendly_name TEXT, email TEXT,
                 user_token TEXT, server_token TEXT, is_active INTEGER DEFAULT 1, deleted_user INTEGER DEFAULT 0);
             INSERT INTO users(user_id, username, friendly_name, email, user_token, server_token) VALUES
                 (101, 'alice.plex', 'Alice', 'alice@example.com', 'SECRET-TOKEN-A', 'SERVER-TOKEN-A'),
                 (102, 'bob_plex', NULL, 'bob@example.com', 'SECRET-TOKEN-B', 'SERVER-TOKEN-B'),
                 (103, 'carol', 'Carol', 'carol@example.com', 'SECRET-TOKEN-C', 'SERVER-TOKEN-C');
             CREATE TABLE session_history (id INTEGER PRIMARY KEY AUTOINCREMENT, reference_id INTEGER, started INTEGER, stopped INTEGER,
                 rating_key INTEGER, user_id INTEGER, user TEXT, ip_address TEXT, paused_counter INTEGER DEFAULT 0, player TEXT, product TEXT,
                 product_version TEXT, platform TEXT, platform_version TEXT, machine_id TEXT, parent_rating_key INTEGER,
                 grandparent_rating_key INTEGER, media_type TEXT, section_id INTEGER, view_offset INTEGER DEFAULT 0);
             CREATE TABLE session_history_metadata (id INTEGER PRIMARY KEY, rating_key INTEGER, parent_rating_key INTEGER,
                 grandparent_rating_key INTEGER, title TEXT, parent_title TEXT, grandparent_title TEXT, full_title TEXT, media_index INTEGER,
                 parent_media_index INTEGER, media_type TEXT, year INTEGER, duration INTEGER DEFAULT 0, guid TEXT, live INTEGER DEFAULT 0);
             CREATE TABLE session_history_media_info (id INTEGER PRIMARY KEY, rating_key INTEGER, video_decision TEXT, audio_decision TEXT,
                 transcode_decision TEXT, duration INTEGER DEFAULT 0, container TEXT, bitrate INTEGER, width INTEGER, height INTEGER,
                 video_bit_depth INTEGER, video_codec TEXT, video_dynamic_range TEXT, audio_codec TEXT, audio_channels INTEGER,
                 audio_language_code TEXT, subtitle_codec TEXT, subtitle_language TEXT, transcode_container TEXT, transcode_video_codec TEXT,
                 transcode_audio_codec TEXT, transcode_audio_channels INTEGER, transcode_width INTEGER, transcode_height INTEGER,
                 transcode_hw_encode_title TEXT, stream_video_decision TEXT, stream_audio_decision TEXT, stream_bitrate INTEGER);",
        )
        .unwrap();
        path
    }

    /// One session row, its metadata and its media info. `reference` 0 starts a viewing of its own.
    #[allow(clippy::too_many_arguments)]
    fn session(c: &Connection, reference: i64, user: i64, kind: &str, rating_key: i64, title: &str, show: Option<(&str, i64, i64, i64)>, year: Option<i64>, started: i64, stopped: i64, paused: i64, offset_ms: i64, duration_ms: i64, decision: &str) -> i64 {
        let (show_title, show_key, season, episode) = match show {
            Some((t, k, s, e)) => (Some(t), Some(k), Some(s), Some(e)),
            None => (None, None, None, None),
        };
        c.execute(
            "INSERT INTO session_history(reference_id, started, stopped, rating_key, user_id, user, ip_address, paused_counter, player, product,
                 product_version, platform, machine_id, grandparent_rating_key, media_type, view_offset)
             VALUES (?1, ?2, ?3, ?4, ?5, 'plex-name', '192.168.1.10', ?6, 'Living Room TV', 'Plex for LG', '8.40.0', 'webOS', 'MACHINE-1', ?7, ?8, ?9)",
            params![reference, started, stopped, rating_key, user, paused, show_key, kind, offset_ms],
        )
        .unwrap();
        let id = c.last_insert_rowid();
        if reference == 0 {
            c.execute("UPDATE session_history SET reference_id = id WHERE id = ?1", [id]).unwrap();
        }
        c.execute(
            "INSERT INTO session_history_metadata(id, rating_key, grandparent_rating_key, title, grandparent_title, media_index, parent_media_index, media_type, year, duration, guid)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 'plex://thing/1')",
            params![id, rating_key, show_key, title, show_title, episode, season, kind, year, duration_ms],
        )
        .unwrap();
        let (stream_video, stream_audio) = match decision {
            "copy" => ("copy", "copy"),
            "transcode" => ("copy", "transcode"),
            _ => ("direct play", "direct play"),
        };
        c.execute(
            "INSERT INTO session_history_media_info(id, rating_key, transcode_decision, container, bitrate, width, height, video_codec, video_dynamic_range,
                 audio_codec, audio_channels, audio_language_code, transcode_video_codec, transcode_audio_codec, transcode_container, stream_video_decision, stream_audio_decision)
             VALUES (?1, ?2, ?3, 'mkv', 8000, 1920, 1080, 'h264', 'SDR', 'eac3', 6, 'eng', 'h264', 'aac', 'mpegts', ?4, ?5)",
            params![id, rating_key, decision, stream_video, stream_audio],
        )
        .unwrap();
        id
    }

    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("finstats-tautulli-test-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// FinStats with two Jellyfin users and a little library.
    fn finstats(dir: &Path) -> Db {
        let db = Db::open(&dir.join("finstats.db")).unwrap();
        db.conn()
            .unwrap()
            .execute_batch(
                "INSERT INTO users(id, name, updated_at) VALUES ('ja', 'alice', 1), ('jb', 'bobby', 1);
                 INSERT INTO items(id, type, name, production_year, runtime_s, updated_at) VALUES ('film1', 'Movie', 'Big Buck Bunny', 2008, 600, 1),
                    ('show1', 'Series', 'Low Orbit', 2021, NULL, 1);
                 INSERT INTO items(id, type, name, series_id, parent_index_number, index_number, runtime_s, updated_at) VALUES
                    ('ep11', 'Episode', 'Pilot', 'show1', 1, 1, 1800, 1);",
            )
            .unwrap();
        db
    }

    fn wire(plex: i64, jellyfin: &str) -> Wire {
        Wire { plex_user_id: plex, jellyfin_user_id: jellyfin.into() }
    }

    type Row = (String, String, String, String, Option<String>, Option<i64>, Option<i64>, i64, i64, i64, i64, Option<i64>, Option<i64>, String);
    fn plays(db: &Db) -> Vec<Row> {
        db.conn()
            .unwrap()
            .prepare(
                "SELECT user_id, user_name, item_id, item_name, series_id, season_number, episode_number, started_at, ended_at, duration_s, paused_s,
                        position_s, runtime_s, play_method FROM playbacks ORDER BY started_at",
            )
            .unwrap()
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?, r.get(7)?, r.get(8)?, r.get(9)?, r.get(10)?, r.get(11)?, r.get(12)?, r.get(13)?)))
            .unwrap()
            .map(Result::unwrap)
            .collect()
    }

    #[test]
    fn the_board_names_each_plex_user_and_counts_what_would_come_never_a_token_or_an_address() {
        let dir = temp("board");
        let path = backup(&dir);
        {
            let c = Connection::open(&path).unwrap();
            let first = session(&c, 0, 101, "movie", 1, "Big Buck Bunny", None, Some(2008), 1000, 1600, 0, 600_000, 600_000, "direct play");
            session(&c, first, 101, "movie", 1, "Big Buck Bunny", None, Some(2008), 2000, 2100, 0, 600_000, 600_000, "direct play");
            session(&c, 0, 101, "track", 9, "A Song", None, None, 3000, 3200, 0, 0, 200_000, "direct play");
            session(&c, 0, 102, "episode", 11, "Pilot", Some(("Low Orbit", 10, 1, 1)), Some(2021), 5000, 6800, 0, 1_800_000, 1_800_000, "transcode");
        }
        let users = plex_users(&open(&path).unwrap()).unwrap();
        assert_eq!(users, vec![
            PlexUser { id: 101, name: "Alice".into(), plays: 1, first_at: Some(1000), last_at: Some(3200) },
            PlexUser { id: 102, name: "bob_plex".into(), plays: 1, first_at: Some(5000), last_at: Some(6800) },
            PlexUser { id: 103, name: "Carol".into(), plays: 0, first_at: None, last_at: None },
        ], "a viewing of several rows is one, music is not counted, and somebody who never played is still on the board");
        let said = serde_json::to_string(&users).unwrap();
        for secret in ["SECRET", "SERVER-TOKEN", "@example.com"] {
            assert!(!said.contains(secret), "the board said {secret}");
        }
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_wire_must_name_somebody_on_each_side_and_a_plex_user_may_have_only_one() {
        let plex: HashSet<i64> = [101, 102, 103].into();
        let jellyfin: HashSet<String> = ["ja".to_string(), "jb".to_string()].into();
        let ok = check_wires(&[wire(101, "ja"), wire(102, "ja")], &plex, &jellyfin).unwrap();
        assert_eq!(ok, HashMap::from([(101, "ja".to_string()), (102, "ja".to_string())]), "two Plex users into one Jellyfin user");
        assert!(check_wires(&[], &plex, &jellyfin).is_err(), "nothing wired: nothing to import");
        assert!(check_wires(&[wire(101, "ja"), wire(101, "jb")], &plex, &jellyfin).is_err(), "one Plex user into two");
        assert!(check_wires(&[wire(999, "ja")], &plex, &jellyfin).is_err(), "a Plex user the backup does not have");
        assert!(check_wires(&[wire(101, "nobody")], &plex, &jellyfin).is_err(), "a Jellyfin user this server does not have");
    }

    #[test]
    fn a_resumed_viewing_is_one_play_and_only_wired_people_come_in() {
        let dir = temp("viewings");
        let path = backup(&dir);
        {
            let c = Connection::open(&path).unwrap();
            // alice watches Big Buck Bunny in two sittings: 400 s with 100 s paused, then 150 s more, stopping at 9:00.
            let first = session(&c, 0, 101, "movie", 1, "Big Buck Bunny", None, Some(2008), 1000, 1500, 100, 400_000, 600_000, "direct play");
            session(&c, first, 101, "movie", 1, "Big Buck Bunny", None, Some(2008), 2000, 2150, 0, 540_000, 600_000, "direct play");
            session(&c, 0, 101, "track", 9, "A Song", None, None, 3000, 3200, 0, 0, 200_000, "direct play");
            session(&c, 0, 103, "movie", 1, "Big Buck Bunny", None, Some(2008), 4000, 4600, 0, 600_000, 600_000, "direct play");
        }
        let db = finstats(&dir);
        let res = run(&db, &path, &[wire(101, "ja")], None).unwrap();
        assert_eq!(res, ImportResult { plays_imported: 1, plays_skipped: 0, not_wired: 1, other_media: 1, users_wired: 1 });
        let got = plays(&db);
        assert_eq!(got.len(), 1);
        let p = &got[0];
        assert_eq!((p.0.as_str(), p.1.as_str()), ("ja", "alice"), "the play is the Jellyfin user's, by their Jellyfin name");
        assert_eq!((p.7, p.8, p.9, p.10), (1000, 2150, 550, 100), "first start to last stop; the time it ran, without the pause");
        assert_eq!((p.11, p.12), (Some(540), Some(600)), "stopped where the last sitting left off, in a film of ten minutes");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_title_on_the_server_is_attached_and_one_that_is_not_keeps_its_name_until_it_comes() {
        let dir = temp("titles");
        let path = backup(&dir);
        {
            let c = Connection::open(&path).unwrap();
            session(&c, 0, 101, "movie", 1, "Big Buck Bunny", None, Some(2008), 1000, 1600, 0, 600_000, 600_000, "direct play");
            session(&c, 0, 101, "movie", 2, "Sintel", None, Some(2010), 2000, 2600, 0, 600_000, 888_000, "direct play");
            session(&c, 0, 101, "episode", 11, "Pilot", Some(("Low Orbit", 10, 1, 1)), Some(2021), 3000, 4800, 0, 1_800_000, 1_800_000, "direct play");
            session(&c, 0, 101, "episode", 12, "Liftoff", Some(("Low Orbit", 10, 1, 2)), Some(2021), 5000, 6800, 0, 1_800_000, 1_800_000, "direct play");
        }
        let db = finstats(&dir);
        run(&db, &path, &[wire(101, "ja")], None).unwrap();
        type Named = (String, String, Option<String>, Option<i64>, Option<i64>);
        let got: Vec<Named> = plays(&db).into_iter().map(|p| (p.2, p.3, p.4, p.5, p.6)).collect();
        assert_eq!(got, vec![
            ("film1".into(), "Big Buck Bunny".into(), None, None, None),
            ("plex:2".into(), "Sintel (2010)".into(), None, None, None),
            ("ep11".into(), "Pilot".into(), Some("show1".into()), Some(1), Some(1)),
            ("plex:12".into(), "Liftoff".into(), Some("plex:10".into()), Some(1), Some(2)),
        ], "on the server: attached; not yet: named so that a library read attaches it when it comes");

        // Sintel arrives, and the next library read finds it.
        db.conn().unwrap().execute_batch("INSERT INTO items(id, type, name, production_year, updated_at) VALUES ('sintel', 'Movie', 'Sintel', 2010, 2)").unwrap();
        crate::sync::backfill_playbacks(&db.conn().unwrap()).unwrap();
        assert_eq!(plays(&db)[1].2, "sintel");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn how_a_play_was_served_is_read_as_finstats_reads_its_own() {
        let dir = temp("served");
        let path = backup(&dir);
        {
            let c = Connection::open(&path).unwrap();
            session(&c, 0, 101, "movie", 1, "Big Buck Bunny", None, Some(2008), 1000, 1600, 0, 600_000, 600_000, "direct play");
            session(&c, 0, 101, "movie", 1, "Big Buck Bunny", None, Some(2008), 3000, 3600, 0, 600_000, 600_000, "copy");
            session(&c, 0, 101, "movie", 1, "Big Buck Bunny", None, Some(2008), 5000, 5600, 0, 600_000, 600_000, "transcode");
        }
        let db = finstats(&dir);
        run(&db, &path, &[wire(101, "ja")], None).unwrap();
        let methods: Vec<String> = plays(&db).into_iter().map(|p| p.13).collect();
        assert_eq!(methods, ["DirectPlay", "DirectStream", "Transcode"], "a copy of both is a direct stream; converting the sound is a transcode");
        let c = db.conn().unwrap();
        let (client, device, ip, codec, transcode): (String, String, String, String, Option<String>) = c
            .query_row("SELECT client, device_name, remote_ip, video_codec, transcode FROM playbacks WHERE play_method = 'Transcode'", [], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
            })
            .unwrap();
        assert_eq!((client.as_str(), device.as_str(), ip.as_str(), codec.as_str()), ("Plex for LG", "Living Room TV", "192.168.1.10", "h264"));
        let t: Value = serde_json::from_str(&transcode.unwrap()).unwrap();
        assert_eq!((t["is_video_direct"].as_bool(), t["is_audio_direct"].as_bool(), t["audio_codec"].as_str()), (Some(true), Some(false), Some("aac")));
        let _ = std::fs::remove_dir_all(dir);
    }

    /// Plex plays a show's theme while the show is open, and Tautulli chains the episode that follows onto it: one
    /// reference, a track and then an episode. The episode is a viewing of its own, not music.
    #[test]
    fn an_episode_chained_onto_its_theme_song_is_still_an_episode() {
        let dir = temp("theme");
        let path = backup(&dir);
        {
            let c = Connection::open(&path).unwrap();
            let theme = session(&c, 0, 101, "track", 9, "Low Orbit Theme", None, None, 1000, 1030, 0, 30_000, 30_000, "direct play");
            session(&c, theme, 101, "episode", 11, "Pilot", Some(("Low Orbit", 10, 1, 1)), Some(2021), 1030, 2830, 0, 1_800_000, 1_800_000, "direct play");
        }
        assert_eq!(plex_users(&open(&path).unwrap()).unwrap()[0].plays, 1, "the board counts it");
        let db = finstats(&dir);
        let res = run(&db, &path, &[wire(101, "ja")], None).unwrap();
        assert_eq!((res.plays_imported, res.other_media), (1, 1), "the episode comes in, the theme does not");
        assert_eq!(plays(&db)[0].2, "ep11");
        let _ = std::fs::remove_dir_all(dir);
    }

    /// Tautulli writes an empty string where it has no number, and some numbers as text: both are read for what they say.
    #[test]
    fn a_number_kept_as_text_is_a_number_and_an_empty_one_is_none() {
        let dir = temp("text");
        let path = backup(&dir);
        {
            let c = Connection::open(&path).unwrap();
            let id = session(&c, 0, 101, "movie", 1, "Big Buck Bunny", None, Some(2008), 1000, 1600, 0, 600_000, 600_000, "direct play");
            c.execute_batch(&format!(
                "UPDATE session_history SET grandparent_rating_key = '' WHERE id = {id};
                 UPDATE session_history_metadata SET media_index = '', parent_media_index = '', year = '2008' WHERE id = {id};
                 UPDATE session_history_media_info SET width = '1920', height = '', bitrate = ' 8000 ', transcode_width = '' WHERE id = {id};"
            ))
            .unwrap();
        }
        let db = finstats(&dir);
        assert_eq!(run(&db, &path, &[wire(101, "ja")], None).unwrap().plays_imported, 1);
        let (item, width, height, bitrate): (String, Option<i64>, Option<i64>, Option<i64>) =
            db.conn().unwrap().query_row("SELECT item_id, width, height, bitrate FROM playbacks", [], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))).unwrap();
        assert_eq!((item.as_str(), width, height, bitrate), ("film1", Some(1920), None, Some(8_000_000)), "the year in text still finds the film");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn the_same_backup_twice_is_once() {
        let dir = temp("twice");
        let path = backup(&dir);
        {
            let c = Connection::open(&path).unwrap();
            session(&c, 0, 101, "movie", 1, "Big Buck Bunny", None, Some(2008), 1000, 1600, 0, 600_000, 600_000, "direct play");
        }
        let db = finstats(&dir);
        assert_eq!(run(&db, &path, &[wire(101, "ja")], None).unwrap().plays_imported, 1);
        let again = run(&db, &path, &[wire(101, "ja")], None).unwrap();
        assert_eq!((again.plays_imported, again.plays_skipped), (0, 1));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_file_that_is_not_a_tautulli_backup_is_refused_and_changes_nothing() {
        let dir = temp("foreign");
        let other = dir.join("other.db");
        Connection::open(&other).unwrap().execute_batch("CREATE TABLE notes (id INTEGER, text TEXT)").unwrap();
        assert!(format!("{:#}", open(&other).unwrap_err()).contains("Tautulli"));
        let garbage = dir.join("garbage.db");
        std::fs::write(&garbage, b"name,watched\nalice,Big Buck Bunny\n").unwrap();
        assert!(format!("{:#}", open(&garbage).unwrap_err()).contains("Tautulli"));
        let db = finstats(&dir);
        assert!(run(&db, &garbage, &[wire(101, "ja")], None).is_err());
        assert!(plays(&db).is_empty());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_zipped_backup_is_unpacked_and_a_plain_one_is_left_as_it_is() {
        use std::io::Write;
        let dir = temp("zip");
        let plain = backup(&dir);
        let bytes = std::fs::read(&plain).unwrap();
        // Stored, as some tools write it, and deflated, as Tautulli's download does.
        let stored = dir.join("stored.db.zip");
        std::fs::write(&stored, crate::card::zip(&[("tautulli.db".into(), bytes.clone())])).unwrap();
        let deflated = dir.join("deflated.db.zip");
        let mut enc = flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::default());
        enc.write_all(&bytes).unwrap();
        std::fs::write(&deflated, zip_deflated("tautulli.db", &bytes, &enc.finish().unwrap())).unwrap();
        for path in [&stored, &deflated, &plain] {
            unpack(path).unwrap();
            assert_eq!(std::fs::read(path).unwrap(), bytes, "{}", path.display());
            assert!(open(path).is_ok());
        }
        let not_a_db = dir.join("readme.zip");
        std::fs::write(&not_a_db, crate::card::zip(&[("readme.txt".into(), b"hello".to_vec())])).unwrap();
        assert!(format!("{:#}", unpack(&not_a_db).unwrap_err()).contains("Tautulli"));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn an_upload_left_waiting_is_swept_away_and_nothing_else_is() {
        let dir = temp("sweep");
        for name in [UPLOAD, "tautulli-upload.part", "tautulli-upload.unpacking", "finstats.db", "jellystat-upload.tmp"] {
            std::fs::write(dir.join(name), b"x").unwrap();
        }
        assert!(!sweep(&dir, std::time::Duration::from_secs(3600)), "younger than the limit: kept");
        assert!(dir.join(UPLOAD).exists());
        assert!(sweep(&dir, std::time::Duration::ZERO), "at start-up, whatever its age");
        let mut left: Vec<String> = std::fs::read_dir(&dir).unwrap().map(|e| e.unwrap().file_name().into_string().unwrap()).collect();
        left.sort();
        assert_eq!(left, ["finstats.db", "jellystat-upload.tmp"], "only what a Tautulli upload leaves");
        assert!(!sweep(&dir, std::time::Duration::ZERO), "nothing left to sweep");
        let _ = std::fs::remove_dir_all(dir);
    }

    /// A one-entry zip whose entry is deflated (method 8), as a zip tool writes it.
    fn zip_deflated(name: &str, raw: &[u8], packed: &[u8]) -> Vec<u8> {
        let crc = {
            let mut h = flate2::Crc::new();
            h.update(raw);
            h.sum()
        };
        let mut out = vec![];
        let local = |out: &mut Vec<u8>| {
            out.extend_from_slice(&0x0403_4b50u32.to_le_bytes());
            out.extend_from_slice(&[20, 0, 0, 0, 8, 0, 0, 0, 0, 0]);
            out.extend_from_slice(&crc.to_le_bytes());
            out.extend_from_slice(&(packed.len() as u32).to_le_bytes());
            out.extend_from_slice(&(raw.len() as u32).to_le_bytes());
            out.extend_from_slice(&(name.len() as u16).to_le_bytes());
            out.extend_from_slice(&0u16.to_le_bytes());
            out.extend_from_slice(name.as_bytes());
        };
        local(&mut out);
        out.extend_from_slice(packed);
        let central_at = out.len() as u32;
        out.extend_from_slice(&0x0201_4b50u32.to_le_bytes());
        out.extend_from_slice(&[20, 0, 20, 0, 0, 0, 8, 0, 0, 0, 0, 0]);
        out.extend_from_slice(&crc.to_le_bytes());
        out.extend_from_slice(&(packed.len() as u32).to_le_bytes());
        out.extend_from_slice(&(raw.len() as u32).to_le_bytes());
        out.extend_from_slice(&(name.len() as u16).to_le_bytes());
        out.extend_from_slice(&[0; 12]);
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(name.as_bytes());
        let central_len = out.len() as u32 - central_at;
        out.extend_from_slice(&0x0605_4b50u32.to_le_bytes());
        out.extend_from_slice(&[0, 0, 0, 0, 1, 0, 1, 0]);
        out.extend_from_slice(&central_len.to_le_bytes());
        out.extend_from_slice(&central_at.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out
    }
}
