//! Capacity: will this server cope? Answered only from what FinStats already holds: the busiest moments on record (how
//! many streams at once, how many of them transcoded, with hardware or without), how fast each disk fills by the files
//! Jellyfin added, and what that says about the server's own settings. FinStats cannot know what the machine *could*
//! do, so nothing here ever says "your server can handle N", the same honesty as never saying "dubbed". Every rule is a
//! pure function over plain rows; nothing reads the database, so each can be tried on invented data.

use std::collections::{BTreeMap, HashMap};

use anyhow::Result;
use serde_json::{Value, json};

use crate::auth::ServerViewer;
use crate::db::rusqlite::{Connection, params};
use crate::state::{ApiResult, App};
use axum::Json;
use axum::extract::State;

/// How a play reached the screen, as far as capacity cares: as the file was, or transcoded, and then whether Jellyfin
/// said it used hardware, said it used none, or did not say (an older server, an imported play).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Way {
    Direct,
    Hardware,
    Software,
    Unsaid,
}

impl Way {
    /// From a play's method and the `hw_accel` and `is_video_direct` its transcode kept (`media::compact_transcode`). A
    /// transcode that kept the picture (only the sound or the container converted) makes no video, which is what costs,
    /// so it counts as direct here, as the Playback page's hardware chart leaves it out.
    pub fn of(play_method: Option<&str>, hw_accel: Option<&str>, video_direct: Option<bool>) -> Way {
        match (play_method, hw_accel, video_direct) {
            (Some("Transcode"), _, Some(true)) => Way::Direct,
            (Some("Transcode"), Some("none"), _) => Way::Software,
            (Some("Transcode"), Some(_), _) => Way::Hardware,
            (Some("Transcode"), None, _) => Way::Unsaid,
            _ => Way::Direct,
        }
    }
}

/// One play, as much of it as capacity reads.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Play {
    pub start: i64,
    pub end: i64,
    pub way: Way,
}

/// What was playing at one moment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Moment {
    pub at: i64,
    pub streams: i64,
    pub transcodes: i64,
    pub hardware: i64,
    pub software: i64,
    pub unsaid: i64,
}

/// How many of the busiest moments are listed.
pub const BUSIEST_LISTED: usize = 5;
/// "Lately": the busiest moment of the last 90 days is shown beside the busiest ever, since a household changes.
pub const RECENT_DAYS: i64 = 90;

/// The busiest moment of each spell of playing (from the first stream starting to the last one ending), busiest spells
/// first: most streams, then most transcodes, then the earlier. One evening of four streams is one moment, not the
/// dozen instants it touched four.
pub fn busiest(plays: &[Play], n: usize) -> Vec<Moment> {
    let mut points: Vec<u64> = plays.iter().filter(|p| p.end > p.start).flat_map(|p| points_of(p.start, p.end, p.way)).collect();
    busiest_of(&mut points, n)
}

/// A play's start and end as two points of eight bytes each: the instant, then end before start at the same instant (so
/// back-to-back plays are not two at once), then how it played. Sorting the numbers sorts the points. A long history is
/// read straight into these, never into a list of plays first: ten million plays were the better part of a gigabyte.
pub fn points_of(start: i64, end: i64, way: Way) -> [u64; 2] {
    let way = match way {
        Way::Direct => 0,
        Way::Hardware => 1,
        Way::Software => 2,
        Way::Unsaid => 3,
    };
    [((start.max(0) as u64) << 3) | 0b100 | way, ((end.max(0) as u64) << 3) | way]
}

/// [`busiest`] over points from [`points_of`], in any order (they are sorted here).
pub fn busiest_of(points: &mut [u64], n: usize) -> Vec<Moment> {
    points.sort_unstable();
    let mut now = Moment::default();
    let mut spells: Vec<Moment> = vec![];
    let mut best: Option<Moment> = None;
    let rank = |a: &Moment, b: &Moment| (b.streams, b.transcodes).cmp(&(a.streams, a.transcodes)).then(a.at.cmp(&b.at));
    for (i, &point) in points.iter().enumerate() {
        let (at, delta) = ((point >> 3) as i64, if point & 0b100 != 0 { 1 } else { -1 });
        now.streams += delta;
        match point & 0b11 {
            1 => now.hardware += delta,
            2 => now.software += delta,
            3 => now.unsaid += delta,
            _ => {}
        }
        now.transcodes = now.hardware + now.software + now.unsaid;
        now.at = at;
        // A spell only ends when an instant leaves nothing playing: an evening of episodes back to back is one spell.
        let instant_done = points.get(i + 1).is_none_or(|next| (next >> 3) as i64 != at);
        if !instant_done {
            continue;
        }
        if best.is_none_or(|b| (now.streams, now.transcodes) > (b.streams, b.transcodes)) && now.streams > 0 {
            best = Some(now);
        }
        if now.streams == 0 {
            spells.extend(best.take());
            // Only the best few are kept: a long history has a spell for nearly every evening.
            if spells.len() >= 4 * n.max(1) {
                spells.sort_by(rank);
                spells.truncate(n);
            }
        }
    }
    spells.sort_by(rank);
    spells.truncate(n);
    spells
}

/// One library folder as Jellyfin's storage read reports it (10.11 and newer).
#[derive(Debug, Clone, PartialEq)]
pub struct Folder {
    pub library: String,
    pub path: String,
    pub free: i64,
    pub used: i64,
    pub device: Option<String>,
}

/// One file FinStats knows of: where it is, how big, and when Jellyfin added it.
#[derive(Debug, Clone, PartialEq)]
pub struct File {
    pub library: Option<String>,
    pub path: String,
    pub bytes: i64,
    pub added: i64,
}

/// How far back growth is measured, and the month it is told in.
pub const GROWTH_MONTHS: i64 = 6;
const MONTH_S: i64 = 2_629_746; // a Gregorian year over twelve

/// A disk: the library folders that share it, what is free on it, and how fast what is added under them fills it.
#[derive(Debug, Clone, PartialEq)]
pub struct Disk {
    pub folders: Vec<String>,
    pub libraries: Vec<String>,
    pub free: i64,
    pub used: i64,
    /// Bytes added under these folders per month, over [`Disk::window_months`].
    pub per_month: i64,
    /// Months until full at that pace; `None` when nothing was added.
    pub months_left: Option<f64>,
    /// How long the pace was measured over: [`GROWTH_MONTHS`], or less on a disk that was filled more recently.
    pub window_months: f64,
    /// Filled too recently to have a pace yet (see [`SETTLE_S`]).
    pub young: bool,
}

/// A disk's first month is its filling, not its growth: Jellyfin dates every file of a library it was just given within
/// days, and read as growth that is a disk "full within a month". The pace is measured from this long after the disk's
/// first file...
pub const SETTLE_S: i64 = 30 * 86_400;
/// ...and told only once it covers this long.
pub const MIN_WINDOW_S: i64 = 30 * 86_400;

/// The disks behind the library folders. Folders on the device Jellyfin names (the mount, on Linux; the drive, on Windows)
/// are one disk, whatever free space was read for each: Jellyfin reads folder by folder, and a disk being written to
/// reports a little less for the next. Without a device, the same free space is the same disk, since two disks with the
/// very same number of free bytes do not happen. A file counts towards the disk whose folder its path is under, with
/// either separator a Jellyfin may use.
pub fn disks(folders: &[Folder], files: &[File], now: i64) -> Vec<Disk> {
    let mut by_disk: BTreeMap<(Option<String>, i64), Vec<&Folder>> = BTreeMap::new();
    for f in folders {
        let key = match &f.device {
            Some(device) => (Some(device.clone()), 0),
            None => (None, f.free),
        };
        by_disk.entry(key).or_default().push(f);
    }
    let since = now - GROWTH_MONTHS * MONTH_S;
    let sep = |c: char| c == '/' || c == '\\';
    let under = |path: &str, folder: &str| path.strip_prefix(folder.trim_end_matches(sep)).is_some_and(|rest| rest.starts_with(sep));
    by_disk
        .into_values()
        .map(|on| {
            let mut paths: Vec<String> = on.iter().map(|f| f.path.clone()).collect();
            paths.sort();
            paths.dedup();
            let mut libraries: Vec<String> = on.iter().map(|f| f.library.clone()).collect();
            libraries.sort();
            libraries.dedup();
            let here: Vec<&File> = files.iter().filter(|f| paths.iter().any(|p| under(&f.path, p))).collect();
            let start = here.iter().map(|f| f.added).min().map_or(since, |first| since.max(first + SETTLE_S));
            let window = now - start;
            let young = window < MIN_WINDOW_S;
            let added: i64 = here.iter().filter(|f| f.added >= start).map(|f| f.bytes.max(0)).sum();
            let per_month = if young { 0 } else { (added as i128 * MONTH_S as i128 / window as i128) as i64 };
            let (free, used) = (on.iter().map(|f| f.free).min().unwrap_or(0), on.iter().map(|f| f.used).max().unwrap_or(0));
            let window_months = if young { 0.0 } else { (window as f64 / MONTH_S as f64 * 10.0).round() / 10.0 };
            Disk { folders: paths, libraries, free, used, per_month, months_left: (per_month > 0).then(|| free as f64 / per_month as f64), window_months, young }
        })
        .collect()
}

/// Growth per library over the same window, for when no free space was reported (Jellyfin before 10.11) and beside it.
pub fn growth_by_library(files: &[File], now: i64) -> BTreeMap<String, i64> {
    let since = now - GROWTH_MONTHS * MONTH_S;
    let mut out: BTreeMap<String, i64> = BTreeMap::new();
    for f in files.iter().filter(|f| f.added >= since) {
        if let Some(lib) = &f.library {
            *out.entry(lib.clone()).or_default() += f.bytes.max(0);
        }
    }
    out
}

/// Something worth knowing, with the numbers behind it.
#[derive(Debug, Clone, PartialEq)]
pub struct Finding {
    /// Kind and subject: the same at every recompute, so a dismissal finds it again.
    pub key: String,
    pub kind: &'static str,
    pub title: String,
    pub evidence: Value,
    /// What the finding rests on; a dismissal holds while this does.
    pub fingerprint: String,
}

/// Software transcodes are looked at over this window...
pub const SOFTWARE_WINDOW_DAYS: i64 = 90;
// The software finding is worked out from the plays of the last [`RECENT_DAYS`], so its window must lie within them.
const _: () = assert!(SOFTWARE_WINDOW_DAYS <= RECENT_DAYS);
/// ...and said about only with this many transcodes Jellyfin said either way about: a handful proves nothing.
pub const SOFTWARE_MIN_SAID: i64 = 20;
/// ...and only where transcoding is common: at least this share of plays. A server that rarely transcodes has no need of
/// hardware for it.
pub const SOFTWARE_SHARE_PCT: i64 = 25;

/// Every transcode Jellyfin said anything about in the window ran in software, and transcoding is common.
pub fn software_transcodes(plays: &[Play], now: i64) -> Option<Finding> {
    let since = now - SOFTWARE_WINDOW_DAYS * 86_400;
    let recent: Vec<Play> = plays.iter().filter(|p| p.start >= since).copied().collect();
    let count = |way: Way| recent.iter().filter(|p| p.way == way).count() as i64;
    let (hardware, software, unsaid) = (count(Way::Hardware), count(Way::Software), count(Way::Unsaid));
    let transcodes = hardware + software + unsaid;
    let total = recent.len() as i64;
    if hardware > 0 || software < SOFTWARE_MIN_SAID || transcodes * 100 < total * SOFTWARE_SHARE_PCT {
        return None;
    }
    let at_once = busiest(&recent.iter().filter(|p| p.way == Way::Software).copied().collect::<Vec<_>>(), 1).first().map_or(0, |m| m.streams);
    Some(Finding {
        key: "capacity:software_transcodes".into(),
        kind: "software_transcodes",
        title: format!("Every transcode in the last {SOFTWARE_WINDOW_DAYS} days that Jellyfin reported on ran without hardware acceleration"),
        evidence: json!({ "days": SOFTWARE_WINDOW_DAYS, "plays": total, "transcodes": transcodes, "software": software, "hardware": hardware, "unsaid": unsaid, "most_at_once": at_once }),
        // The situation, not its numbers: they move every day, and a dismissal is of the situation.
        fingerprint: "no hardware transcodes".into(),
    })
}

/// A library scan that runs this share of the time or more, by its own last run against its own interval, is worth a look:
/// a scan reads every folder, and a server scanning a quarter of the time is busy with it a quarter of the time.
pub const SCAN_DUTY_PCT: i64 = 25;

/// Jellyfin's library scan as its scheduled task reports it: how often it runs, and how long it last took.
pub fn scan_busy(interval_s: Option<i64>, last_duration_s: Option<i64>, items: i64) -> Option<Finding> {
    let (interval, took) = (interval_s.filter(|i| *i > 0)?, last_duration_s?);
    let duty = took * 100 / interval;
    (duty >= SCAN_DUTY_PCT).then(|| Finding {
        key: "capacity:scan_busy".into(),
        kind: "scan_busy",
        title: "Jellyfin's library scan runs for a large share of the time".into(),
        evidence: json!({ "interval_s": interval, "last_duration_s": took, "duty_pct": duty, "items": items }),
        // The owner's setting: changing the interval is a new situation, a slower run of the same one is not.
        fingerprint: format!("every {interval} s"),
    })
}

/// A disk with less than this many months of space at the pace it has been filling.
pub const RUNWAY_SHORT_MONTHS: f64 = 6.0;
/// Where a finding about space is told again, dismissed or not: a dismissal at six months does not silence one month.
const RUNWAY_MARKS: [f64; 3] = [1.0, 3.0, 6.0];

pub fn runway(disk: &Disk) -> Option<Finding> {
    let months = disk.months_left.filter(|m| *m < RUNWAY_SHORT_MONTHS)?;
    let mark = RUNWAY_MARKS.iter().find(|m| months < **m).copied().unwrap_or(RUNWAY_SHORT_MONTHS);
    let place = disk.folders.join(", ");
    Some(Finding {
        key: format!("capacity:runway:{place}"),
        kind: "runway",
        title: format!("{place}: about {} of space left at the pace of the last {} months", months_said(months), disk.window_months),
        evidence: json!({ "folders": disk.folders, "libraries": disk.libraries, "free_bytes": disk.free, "added_per_month": disk.per_month,
                          "months_left": (months * 10.0).round() / 10.0, "window_months": disk.window_months }),
        fingerprint: format!("under {mark} months"),
    })
}

/// "about four months", "under a month".
fn months_said(m: f64) -> String {
    match m {
        m if m < 1.0 => "under a month".into(),
        m if m < 1.5 => "a month".into(),
        m => format!("{} months", m.round() as i64),
    }
}

// ---------------------------------------------------------------- worked out after a read, kept, read back

fn moment_json(m: &Moment) -> Value {
    json!({ "at": m.at, "streams": m.streams, "transcodes": m.transcodes, "hardware": m.hardware, "software": m.software, "unsaid": m.unsaid })
}

/// Everything again from what is stored: the plays, the files and the server's last details. Run after a library read
/// and after a read of the server's details, never on a request. Answers how many findings there are.
pub fn recompute(conn: &mut Connection, now: i64) -> Result<usize> {
    // How a play went is read from its transcode only where there is one, so the history is not parsed play by play.
    const PLAY: &str = "SELECT started_at, ended_at, play_method,
                               CASE WHEN play_method = 'Transcode' THEN json_extract(transcode, '$.hw_accel') END,
                               CASE WHEN play_method = 'Transcode' THEN json_extract(transcode, '$.is_video_direct') END
                        FROM visible_playbacks WHERE ended_at > started_at";
    let way = |r: &crate::db::rusqlite::Row| -> crate::db::rusqlite::Result<Way> {
        Ok(Way::of(r.get::<_, Option<String>>(2)?.as_deref(), r.get::<_, Option<String>>(3)?.as_deref(), r.get::<_, Option<bool>>(4)?))
    };
    let mut points: Vec<u64> = vec![];
    {
        let mut stmt = conn.prepare(PLAY)?;
        let mut rows = stmt.query([])?;
        while let Some(r) = rows.next()? {
            points.extend(points_of(r.get(0)?, r.get(1)?, way(r)?));
        }
    }
    let busiest_ever = busiest_of(&mut points, BUSIEST_LISTED);
    drop(points);
    // The last 90 days, through the index on when plays ended: a play that began in them ended in them.
    let recent_since = now - RECENT_DAYS * 86_400;
    let recent: Vec<Play> = conn
        .prepare(&format!("{PLAY} AND ended_at >= ?1 AND started_at >= ?1"))?
        .query_map([recent_since], |r| Ok(Play { start: r.get(0)?, end: r.get(1)?, way: way(r)? }))?
        .collect::<Result<_, _>>()?;
    let files: Vec<File> = conn
        .prepare("SELECT l.name, i.path, i.size_bytes, i.date_created FROM items i LEFT JOIN libraries l ON l.id = i.library_id
                  WHERE i.removed = 0 AND i.path IS NOT NULL AND i.size_bytes > 0 AND i.date_created IS NOT NULL")?
        .query_map([], |r| Ok(File { library: r.get(0)?, path: r.get(1)?, bytes: r.get(2)?, added: r.get(3)? }))?
        .collect::<Result<_, _>>()?;
    let info: Value = crate::db::get_setting(conn, "server_info")?.and_then(|raw| serde_json::from_str(&raw).ok()).unwrap_or(Value::Null);
    let folders: Vec<Folder> = info["storage"].as_array().map(Vec::as_slice).unwrap_or_default().iter()
        .filter(|f| f["kind"] == "library")
        .filter_map(|f| Some(Folder { library: f["label"].as_str()?.to_string(), path: f["path"].as_str()?.to_string(), free: f["free_bytes"].as_i64()?,
                                      used: f["used_bytes"].as_i64().unwrap_or(0), device: f["device"].as_str().map(str::to_string) }))
        .collect();
    let scan = info["scheduled_tasks"].as_array().map(Vec::as_slice).unwrap_or_default().iter().find(|t| t["key"] == "RefreshLibrary").cloned().unwrap_or(Value::Null);

    let disks = disks(&folders, &files, now);
    let mut found: Vec<Finding> = vec![];
    found.extend(disks.iter().filter_map(runway));
    found.extend(scan_busy(scan["interval_s"].as_i64(), scan["last_duration_s"].as_i64(), files.len() as i64));
    found.extend(software_transcodes(&recent, now));
    found.sort_by(|a, b| a.kind.cmp(b.kind).then(a.key.cmp(&b.key)));

    let facts = json!({
        "computed_at": now,
        "busiest": busiest_ever.iter().map(moment_json).collect::<Vec<_>>(),
        "recent": busiest(&recent, 1).first().map(moment_json),
        "recent_days": RECENT_DAYS,
        "free_space_reported": !folders.is_empty(),
        "disks": disks.iter().map(|d| json!({ "folders": d.folders, "libraries": d.libraries, "free_bytes": d.free, "used_bytes": d.used,
                                                "added_per_month": d.per_month, "months_left": d.months_left.map(|m| (m * 10.0).round() / 10.0),
                                                "window_months": d.window_months, "young": d.young })).collect::<Vec<_>>(),
        "growth": growth_by_library(&files, now).into_iter().map(|(library, added)| json!({ "library": library, "added_bytes": added })).collect::<Vec<_>>(),
        "window_months": GROWTH_MONTHS,
    });

    let tx = conn.transaction()?;
    let first: HashMap<String, i64> = tx.prepare("SELECT key, found_at FROM capacity_findings")?.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?.collect::<Result<_, _>>()?;
    tx.execute("DELETE FROM capacity_findings", [])?;
    {
        let mut row = tx.prepare("INSERT INTO capacity_findings(key, kind, title, evidence, fingerprint, found_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)")?;
        for f in &found {
            row.execute(params![f.key, f.kind, f.title, f.evidence.to_string(), f.fingerprint, first.get(&f.key).copied().unwrap_or(now)])?;
        }
    }
    crate::db::set_setting(&tx, "capacity", &facts.to_string())?;
    tx.commit()?;
    Ok(found.len())
}

/// What `GET /api/server/capacity` answers: the stored facts and findings, each with its dismissal. `null` before the
/// first recompute.
pub fn answer(conn: &Connection) -> Result<Value> {
    let Some(mut facts) = crate::db::get_setting(conn, "capacity")?.and_then(|raw| serde_json::from_str::<Value>(&raw).ok()) else { return Ok(Value::Null) };
    let findings: Vec<Value> = conn
        .prepare("SELECT f.key, f.kind, f.title, f.evidence, f.found_at, d.key IS NOT NULL, d.note, d.at, d.by
                  FROM capacity_findings f LEFT JOIN health_dismissed d ON d.key = f.key AND d.fingerprint = f.fingerprint ORDER BY f.kind, f.key")?
        .query_map([], |r| {
            Ok(json!({ "key": r.get::<_, String>(0)?, "kind": r.get::<_, String>(1)?, "title": r.get::<_, String>(2)?,
                       "evidence": serde_json::from_str::<Value>(&r.get::<_, String>(3)?).unwrap_or(Value::Null), "found_at": r.get::<_, i64>(4)?,
                       "dismissed": r.get::<_, bool>(5)?, "note": r.get::<_, Option<String>>(6)?, "dismissed_at": r.get::<_, Option<i64>>(7)?,
                       "dismissed_by": r.get::<_, Option<String>>(8)? }))
        })?
        .collect::<Result<_, _>>()?;
    facts["findings"] = json!(findings);
    Ok(facts)
}

/// `GET /api/server/capacity`: what the last recompute found. Read only; worked out after the server's reads.
pub async fn get_capacity(State(app): State<App>, ServerViewer(_): ServerViewer) -> ApiResult {
    Ok(Json(app.db.call(|c| answer(c)).await?))
}

#[cfg(test)]
mod tests {
    use super::*;

    const DAY: i64 = 86_400;
    const NOW: i64 = 1_800_000_000;

    fn play(start: i64, end: i64, way: Way) -> Play {
        Play { start, end, way }
    }

    #[test]
    fn a_play_is_direct_or_transcoded_with_hardware_without_or_unsaid() {
        assert_eq!(Way::of(Some("DirectPlay"), None, None), Way::Direct);
        assert_eq!(Way::of(Some("DirectStream"), Some("none"), None), Way::Direct);
        assert_eq!(Way::of(Some("Transcode"), Some("nvenc"), Some(false)), Way::Hardware);
        assert_eq!(Way::of(Some("Transcode"), Some("none"), Some(false)), Way::Software);
        assert_eq!(Way::of(Some("Transcode"), None, None), Way::Unsaid);
        assert_eq!(Way::of(None, None, None), Way::Direct);
    }

    /// Converting only the sound (or only the container) makes no picture, which is what a server has to cope with: the
    /// Playback page's hardware chart leaves such plays out, and so does Capacity, whatever Jellyfin says of hardware.
    #[test]
    fn a_transcode_that_kept_the_picture_is_not_a_video_transcode() {
        assert_eq!(Way::of(Some("Transcode"), Some("nvenc"), Some(true)), Way::Direct);
        assert_eq!(Way::of(Some("Transcode"), Some("none"), Some(true)), Way::Direct);
        assert_eq!(Way::of(Some("Transcode"), None, Some(true)), Way::Direct);
        assert_eq!(Way::of(Some("Transcode"), Some("none"), None), Way::Software, "a transcode that did not say keeps what it said of hardware");
    }

    #[test]
    fn the_busiest_moments_are_one_per_spell_of_playing_busiest_first() {
        let plays = [
            // An evening: three at once at 1,500, two of them transcoded (one in hardware, one not said).
            play(1_000, 4_000, Way::Direct), play(1_200, 3_000, Way::Hardware), play(1_500, 2_000, Way::Unsaid),
            // Back to back is not two at once.
            play(4_000, 5_000, Way::Direct),
            // Another day: two at once, both in software.
            play(DAY, DAY + 900, Way::Software), play(DAY + 100, DAY + 800, Way::Software),
            // A third: two at once, direct.
            play(2 * DAY, 2 * DAY + 600, Way::Direct), play(2 * DAY + 60, 2 * DAY + 700, Way::Direct),
        ];
        let b = busiest(&plays, 5);
        assert_eq!(b.len(), 3, "one moment per spell: {b:?}");
        assert_eq!(b[0], Moment { at: 1_500, streams: 3, transcodes: 2, hardware: 1, software: 0, unsaid: 1 });
        assert_eq!((b[1].at, b[1].streams, b[1].transcodes, b[1].software), (DAY + 100, 2, 2, 2), "two with transcodes before two without");
        assert_eq!((b[2].at, b[2].streams, b[2].transcodes), (2 * DAY + 60, 2, 0));
        assert_eq!(busiest(&plays, 1).len(), 1);
        assert!(busiest(&[], 5).is_empty());
    }

    #[test]
    fn of_many_spells_the_busiest_are_kept_wherever_they_fall() {
        // Twenty evenings, one stream each, but for three: four streams on the first, three on the tenth, four on the last.
        let mut plays = vec![];
        for day in 0..20i64 {
            let at_once = match day { 0 | 19 => 4, 9 => 3, _ => 1 };
            plays.extend((0..at_once).map(|k| play(day * DAY + k, day * DAY + 3_600, Way::Direct)));
        }
        let b = busiest(&plays, 2);
        assert_eq!(b.iter().map(|m| (m.at / DAY, m.streams)).collect::<Vec<_>>(), [(0, 4), (19, 4)], "{b:?}");
        assert_eq!(busiest(&plays, 3)[2].streams, 3);
        assert_eq!(busiest(&plays, 50).len(), 20);
    }

    #[test]
    fn within_a_spell_the_moment_with_most_transcodes_at_the_most_streams_is_kept() {
        // Three at once twice in one spell: first all direct, later two of them transcoding.
        let plays = [play(0, 1_000, Way::Direct), play(100, 300, Way::Direct), play(150, 250, Way::Direct), play(500, 900, Way::Hardware), play(600, 800, Way::Hardware)];
        assert_eq!(busiest(&plays, 1)[0], Moment { at: 600, streams: 3, transcodes: 2, hardware: 2, software: 0, unsaid: 0 });
    }

    fn folder(library: &str, path: &str, free: i64, device: Option<&str>) -> Folder {
        Folder { library: library.into(), path: path.into(), free, used: 1_000, device: device.map(str::to_string) }
    }
    fn file(library: &str, path: &str, gb: i64, days_ago: i64) -> File {
        File { library: Some(library.into()), path: path.into(), bytes: gb * 1_000_000_000, added: NOW - days_ago * DAY }
    }

    #[test]
    fn folders_reporting_one_free_space_are_one_disk_and_fill_by_what_is_added_under_them() {
        let folders = [
            folder("Films", "/media/films", 600_000_000_000, Some("sda1")),
            folder("Shows", "/media/shows", 600_000_000_000, Some("sda1")),
            folder("Shows", "/mnt/big/shows", 9_000_000_000_000, Some("sdb1")),
            folder("Music", "/media/music", 7, None),
        ];
        let files = [
            file("Films", "/media/films/Big Buck Bunny (2008)/bbb.mkv", 60, 30),
            file("Shows", "/media/shows/Low Orbit/S01E01.mkv", 30, 100),
            file("Shows", "/mnt/big/shows/Harbour Street/S01E01.mkv", 90, 10),
            file("Shows", "/mnt/big/shows/Harbour Street/S00E01.mkv", 1, 300), // long before: the disk is not new
            file("Films", "/media/films/Sintel (2010)/sintel.mkv", 500, 400), // before the window: not growth
            file("Films", "/media/filmsx/elsewhere.mkv", 5, 1),               // a folder whose name only starts the same
        ];
        let d = disks(&folders, &files, NOW);
        assert_eq!(d.len(), 3, "{d:?}");
        let small = d.iter().find(|x| x.free == 600_000_000_000).unwrap();
        assert_eq!(small.folders, ["/media/films", "/media/shows"]);
        assert_eq!(small.libraries, ["Films", "Shows"]);
        assert_eq!(small.per_month, 90_000_000_000 / GROWTH_MONTHS, "60 and 30 GB in six months");
        let months = small.months_left.unwrap();
        assert!((months - 40.0).abs() < 0.1, "600 GB at 15 GB a month: {months}");
        let big = d.iter().find(|x| x.free == 9_000_000_000_000).unwrap();
        assert_eq!((big.libraries.clone(), big.per_month), (vec!["Shows".to_string()], 15_000_000_000));
        let quiet = d.iter().find(|x| x.free == 7).unwrap();
        assert_eq!((quiet.per_month, quiet.months_left), (0, None), "nothing added: no pace to tell");
    }

    /// A library's first weeks are its filling, not its growth: a Jellyfin set up three months ago dates every file in the
    /// last three months, and read as growth that is "a month of space left" for a disk that is barely filling. The pace
    /// is measured from a month after the disk's first file, and not at all until a month after that.
    #[test]
    fn a_young_library_is_measured_from_after_its_filling() {
        let folders = [folder("Films", "/media/films", 2_000_000_000_000, Some("/media"))];
        // Ten TB arrived 120 days ago, the day Jellyfin was set up; since then 100 GB 60 days ago and 100 GB 10 days ago.
        let files = [file("Films", "/media/films/all.mkv", 10_000, 120), file("Films", "/media/films/a.mkv", 100, 60), file("Films", "/media/films/b.mkv", 100, 10)];
        let d = &disks(&folders, &files, NOW)[0];
        let window = 90.0 * DAY as f64 / MONTH_S as f64; // from 30 days after the first file
        assert!((d.per_month as f64 - 200_000_000_000.0 / window).abs() < 1_000.0, "{d:?}");
        assert!((d.window_months - 3.0).abs() < 0.05, "{d:?}");
        assert!(!d.young);
        // Set up 50 days ago: not a month of growth since its filling yet, so no pace and no runway.
        let files = [file("Films", "/media/films/all.mkv", 10_000, 50), file("Films", "/media/films/b.mkv", 100, 10)];
        let d = &disks(&folders, &files, NOW)[0];
        assert_eq!((d.per_month, d.months_left, d.young), (0, None, true), "{d:?}");
        assert!(runway(d).is_none());
    }

    /// Jellyfin reads each folder's free space on its own, so a disk being written to reports a little less for the next
    /// folder: where it names the device (the mount, on Linux), that alone says it is one disk, at the freest reading.
    #[test]
    fn folders_on_one_named_device_are_one_disk_whatever_the_free_space_read_for_each() {
        let d = disks(&[folder("Films", "/mnt/media/films", 500_000, Some("/mnt/media")), folder("Shows", "/mnt/media/shows", 499_000, Some("/mnt/media"))], &[], NOW);
        assert_eq!(d.len(), 1, "{d:?}");
        assert_eq!((d[0].folders.len(), d[0].free), (2, 499_000), "the lower reading: the disk is filling");
    }

    /// A Jellyfin on Windows names folders and files with backslashes.
    #[test]
    fn a_windows_folder_fills_by_what_is_added_under_it() {
        let d = disks(&[folder("Films", r"D:\Movies", 600_000_000_000, Some(r"D:\"))],
                      &[file("Films", r"D:\Movies\Big Buck Bunny (2008)\bbb.mkv", 60, 30), file("Films", r"D:\Moviesx\elsewhere.mkv", 5, 1),
                        file("Films", r"D:\Movies\Sintel (2010)\sintel.mkv", 1, 400)], NOW);
        assert_eq!(d[0].per_month, 60_000_000_000 / GROWTH_MONTHS, "{d:?}");
    }

    #[test]
    fn a_device_name_keeps_two_disks_apart_even_at_the_same_free_space() {
        let d = disks(&[folder("Films", "/a", 500, Some("sda1")), folder("Shows", "/b", 500, Some("sdb1"))], &[], NOW);
        assert_eq!(d.len(), 2);
    }

    #[test]
    fn growth_by_library_counts_the_window_only() {
        let g = growth_by_library(&[file("Films", "/f/a", 60, 30), file("Films", "/f/b", 500, 400), file("Shows", "/s/a", 30, 179)], NOW);
        assert_eq!(g.get("Films"), Some(&60_000_000_000));
        assert_eq!(g.get("Shows"), Some(&30_000_000_000));
    }

    fn many(n: i64, way: Way, days_ago: i64) -> Vec<Play> {
        (0..n).map(|i| play(NOW - days_ago * DAY + i * 10_000, NOW - days_ago * DAY + i * 10_000 + 3_600, way)).collect()
    }

    #[test]
    fn every_transcode_in_software_is_said_once_transcoding_is_common_and_enough_said() {
        let mut plays = many(25, Way::Software, 10);
        plays.extend(many(50, Way::Direct, 10));
        let f = software_transcodes(&plays, NOW).expect("25 of 75 transcoded, all in software");
        assert_eq!(f.key, "capacity:software_transcodes");
        assert_eq!(f.kind, "software_transcodes");
        assert_eq!((f.evidence["plays"].as_i64(), f.evidence["transcodes"].as_i64(), f.evidence["software"].as_i64(), f.evidence["hardware"].as_i64()), (Some(75), Some(25), Some(25), Some(0)));
        assert_eq!(f.evidence["days"], SOFTWARE_WINDOW_DAYS);

        let mut with_hw = plays.clone();
        with_hw.extend(many(1, Way::Hardware, 5));
        assert!(software_transcodes(&with_hw, NOW).is_none(), "one hardware transcode: hardware works");
        let mut rare = many(25, Way::Software, 10);
        rare.extend(many(100, Way::Direct, 10));
        assert!(software_transcodes(&rare, NOW).is_none(), "a fifth of plays is not common");
        let mut few = many(19, Way::Software, 10);
        few.extend(many(5, Way::Unsaid, 10));
        assert!(software_transcodes(&few, NOW).is_none(), "nineteen said is too few, however many were not");
        assert!(software_transcodes(&many(40, Way::Software, SOFTWARE_WINDOW_DAYS + 5), NOW).is_none(), "outside the window");
        assert!(software_transcodes(&many(40, Way::Unsaid, 10), NOW).is_none(), "a server that never says is not said to use software");
        // The same situation a month later is the same finding: a dismissal holds while it does.
        let later: Vec<Play> = plays.iter().map(|p| play(p.start + 30 * DAY, p.end + 30 * DAY, p.way)).collect();
        assert_eq!(software_transcodes(&later, NOW + 30 * DAY).unwrap().fingerprint, f.fingerprint);
    }

    #[test]
    fn a_scan_running_a_quarter_of_the_time_is_worth_a_look() {
        let f = scan_busy(Some(3_600), Some(900), 12_000).expect("15 minutes of every hour");
        assert_eq!((f.key.as_str(), f.evidence["duty_pct"].as_i64(), f.evidence["items"].as_i64()), ("capacity:scan_busy", Some(25), Some(12_000)));
        assert!(scan_busy(Some(3_600), Some(899), 12_000).is_none(), "just under a quarter");
        assert!(scan_busy(Some(43_200), Some(900), 12_000).is_none(), "twice a day");
        assert!(scan_busy(None, Some(900), 12_000).is_none(), "no interval: a time of day or a start-up");
        assert!(scan_busy(Some(3_600), None, 12_000).is_none(), "never run");
        assert_ne!(scan_busy(Some(1_800), Some(900), 1).unwrap().fingerprint, f.fingerprint, "a new interval is a new setting to look at");
        assert_eq!(scan_busy(Some(3_600), Some(1_200), 1).unwrap().fingerprint, f.fingerprint, "a slower run of the same setting is not");
    }

    fn disk(free_gb: i64, per_month_gb: i64) -> Disk {
        let (free, per_month) = (free_gb * 1_000_000_000, per_month_gb * 1_000_000_000);
        Disk { folders: vec!["/media/films".into()], libraries: vec!["Films".into()], free, used: 1, per_month, months_left: (per_month > 0).then(|| free as f64 / per_month as f64), window_months: 6.0, young: false }
    }

    #[test]
    fn a_disk_with_under_six_months_left_is_said_and_said_again_as_it_nears_full() {
        assert!(runway(&disk(700, 100)).is_none(), "seven months");
        assert!(runway(&disk(700, 0)).is_none(), "not filling");
        let five = runway(&disk(500, 100)).unwrap();
        assert_eq!(five.kind, "runway");
        assert_eq!(five.evidence["months_left"].as_f64(), Some(5.0));
        assert_eq!(runway(&disk(400, 100)).unwrap().fingerprint, five.fingerprint, "four months: the same mark");
        assert_ne!(runway(&disk(250, 100)).unwrap().fingerprint, five.fingerprint, "under three: said again");
        assert_ne!(runway(&disk(50, 100)).unwrap().fingerprint, runway(&disk(250, 100)).unwrap().fingerprint, "under one: again");
        assert_eq!(runway(&disk(250, 100)).unwrap().key, five.key, "the same disk, the same finding");
    }

    // ------------------------------------------------------------ stored, and read back

    fn db() -> crate::db::rusqlite::Connection {
        let c = crate::db::rusqlite::Connection::open_in_memory().unwrap();
        for m in crate::db::MIGRATIONS {
            c.execute_batch(m).unwrap();
        }
        c.execute_batch("INSERT INTO libraries(id, name, updated_at) VALUES ('lf', 'Films', 0);").unwrap();
        for (n, gb, days) in [(1, 60, 20), (2, 60, 50), (3, 1, 400)] {
            c.execute(
                "INSERT INTO items(id, type, name, library_id, path, size_bytes, date_created, removed, updated_at) VALUES (?1, 'Movie', ?2, 'lf', ?3, ?4, ?5, 0, 0)",
                crate::db::rusqlite::params![format!("f{n}"), format!("Big Buck Bunny {n}"), format!("/media/films/bbb{n}.mkv"), gb * 1_000_000_000i64, NOW - days * DAY],
            ).unwrap();
        }
        let info = json!({ "storage": [{ "label": "Films", "path": "/media/films", "free_bytes": 100_000_000_000i64, "used_bytes": 900_000_000_000i64, "kind": "library", "device": "sda1" },
                                       { "label": "Cache", "path": "/cache", "free_bytes": 5, "used_bytes": 5, "kind": "system" }],
                           "scheduled_tasks": [{ "key": "RefreshLibrary", "name": "Scan Media Library", "interval_s": 3600, "last_duration_s": 1200 }] });
        crate::db::set_setting(&c, "server_info", &info.to_string()).unwrap();
        let mut stmt = c.prepare("INSERT INTO playbacks(source, user_id, user_name, item_id, item_name, item_type, started_at, ended_at, duration_s, play_method, transcode)
                                  VALUES ('live', 'u1', 'alice', 'f1', 'Big Buck Bunny', 'Movie', ?1, ?2, 600, ?3, ?4)").unwrap();
        for i in 0..75i64 {
            let at = NOW - 10 * DAY + i * 10_000;
            let (method, tc) = if i < 25 { ("Transcode", Some(r#"{"hw_accel":"none"}"#)) } else { ("DirectPlay", None) };
            stmt.execute(crate::db::rusqlite::params![at, at + 600, method, tc]).unwrap();
        }
        drop(stmt);
        c
    }

    /// How long a recompute takes and what it holds on a long history: `cargo test --release capacity_at_scale -- --ignored
    /// --nocapture`, under `/usr/bin/time -v` for the memory. Not part of the suite.
    #[test]
    #[ignore]
    fn capacity_at_scale() {
        let mut c = db();
        let n: i64 = std::env::var("PLAYS").ok().and_then(|v| v.parse().ok()).unwrap_or(2_000_000);
        c.execute_batch(&format!(
            "WITH RECURSIVE k(i) AS (SELECT 1 UNION ALL SELECT i + 1 FROM k WHERE i < {n})
             INSERT INTO playbacks(source, user_id, user_name, item_id, item_name, item_type, started_at, ended_at, duration_s, play_method, transcode)
             SELECT 'live', 'u' || (i % 40), 'alice', 'f1', 'Big Buck Bunny', 'Movie', {NOW} - i * 300, {NOW} - i * 300 + 2400, 2400,
                    CASE WHEN i % 4 = 0 THEN 'Transcode' ELSE 'DirectPlay' END,
                    CASE WHEN i % 4 = 0 THEN '{{\"hw_accel\":\"nvenc\",\"is_video_direct\":false}}' END FROM k"
        )).unwrap();
        let kb = |field: &str| -> i64 {
            std::fs::read_to_string("/proc/self/status").unwrap().lines().find(|l| l.starts_with(field)).unwrap().split_whitespace().nth(1).unwrap().parse().unwrap()
        };
        std::fs::write("/proc/self/clear_refs", "5").unwrap(); // the peak starts again from here
        let (rss, t) = (kb("VmRSS:"), std::time::Instant::now());
        recompute(&mut c, NOW).unwrap();
        println!("{n} plays: recompute took {:?} and {} MB more at its peak", t.elapsed(), (kb("VmHWM:") - rss) / 1024);
    }

    #[test]
    fn a_recompute_reads_transcodes_that_kept_the_picture_as_direct() {
        let mut c = db();
        c.execute(r#"UPDATE playbacks SET transcode = '{"hw_accel":"none","is_video_direct":true}' WHERE play_method = 'Transcode'"#, []).unwrap();
        assert_eq!(recompute(&mut c, NOW).unwrap(), 2, "a busy scan and a disk; the sound converted is no software transcode");
        assert_eq!(answer(&c).unwrap()["busiest"][0]["transcodes"], 0);
    }

    #[test]
    fn a_recompute_stores_what_it_found_and_a_page_reads_it_back() {
        let mut c = db();
        assert_eq!(recompute(&mut c, NOW).unwrap(), 3, "software, a busy scan, and a disk with five months");
        let a = answer(&c).unwrap();
        assert_eq!(a["computed_at"], NOW);
        assert_eq!(a["free_space_reported"], true);
        assert_eq!(a["busiest"][0]["streams"], 1);
        let disk = &a["disks"][0];
        assert_eq!((disk["folders"].clone(), disk["libraries"].clone()), (json!(["/media/films"]), json!(["Films"])), "the cache is not a library disk");
        assert_eq!(disk["added_per_month"], 20_000_000_000i64, "120 GB in six months");
        assert_eq!(a["growth"], json!([{ "library": "Films", "added_bytes": 120_000_000_000i64 }]));
        let kinds: Vec<&str> = a["findings"].as_array().unwrap().iter().map(|f| f["kind"].as_str().unwrap()).collect();
        assert_eq!(kinds, ["runway", "scan_busy", "software_transcodes"]);
        assert!(a["findings"].as_array().unwrap().iter().all(|f| f["dismissed"] == false && f["found_at"] == NOW));

        // Found again later: still found when it first was.
        recompute(&mut c, NOW + DAY).unwrap();
        assert_eq!(answer(&c).unwrap()["findings"][0]["found_at"], NOW);
        // A finding that stops being true goes.
        c.execute("UPDATE playbacks SET transcode = '{\"hw_accel\":\"vaapi\"}' WHERE id = (SELECT MIN(id) FROM playbacks)", []).unwrap();
        assert_eq!(recompute(&mut c, NOW + DAY).unwrap(), 2);
    }

    #[test]
    fn a_capacity_finding_is_dismissed_with_library_healths_dismissal_and_comes_back_when_it_changes() {
        let mut c = db();
        recompute(&mut c, NOW).unwrap();
        assert!(crate::health::dismiss(&c, "capacity:scan_busy", Some("on purpose"), "alice", NOW).unwrap());
        let scan = |c: &crate::db::rusqlite::Connection| answer(c).unwrap()["findings"].as_array().unwrap().iter().find(|f| f["kind"] == "scan_busy").cloned().unwrap();
        assert_eq!((scan(&c)["dismissed"].clone(), scan(&c)["note"].clone()), (json!(true), json!("on purpose")));
        // The owner sets the scan to every half hour: a new setting, looked at again.
        let info = crate::db::get_setting(&c, "server_info").unwrap().unwrap().replace("\"interval_s\":3600", "\"interval_s\":1800");
        crate::db::set_setting(&c, "server_info", &info).unwrap();
        recompute(&mut c, NOW).unwrap();
        assert_eq!(scan(&c)["dismissed"], false);
        assert!(crate::health::undismiss(&c, "capacity:scan_busy").unwrap());
    }

    #[test]
    fn without_free_space_from_jellyfin_growth_is_still_told() {
        let mut c = db();
        let info = json!({ "storage": [], "scheduled_tasks": [] });
        crate::db::set_setting(&c, "server_info", &info.to_string()).unwrap();
        recompute(&mut c, NOW).unwrap();
        let a = answer(&c).unwrap();
        assert_eq!((a["free_space_reported"].clone(), a["disks"].clone()), (json!(false), json!([])));
        assert_eq!(a["growth"][0]["added_bytes"], 120_000_000_000i64);
        assert!(a["findings"].as_array().unwrap().iter().all(|f| f["kind"] != "runway"));
    }

    #[test]
    fn before_the_first_recompute_there_is_nothing_to_show() {
        let c = db();
        assert_eq!(answer(&c).unwrap(), Value::Null);
    }
}
