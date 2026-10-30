//! Import of Jellystat backups (`.jsonl`, plus the older single-document `.json`).
//!
//! What the backup means, learned from real exports:
//! - `ActivityDateInserted` is when the play *ended*; start = end − `PlaybackDuration`.
//! - For episodes `NowPlayingItemId` is the *series*; the episode itself is `EpisodeId`.
//! - `PlayState.PositionTicks` is usually the position when Jellystat first noticed the
//!   session, not where playback stopped, so it is not imported as a position.
//! - bigint columns (`PlaybackDuration`, `RunTimeTicks`, `Size`…) arrive as strings.

use std::fs::File;
use std::io::{BufRead, BufReader, Read};
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

/// What an import is doing while it reads `table`, in the language of whoever started it. The words of every import
/// are here (`server.import`), the other trackers' readers use them too.
fn phase_label(table: &str) -> String {
    let key = match table {
        "jf_libraries" => "reading_libraries",
        "jf_library_items" => "reading_items",
        "jf_library_seasons" => "reading_seasons",
        "jf_library_episodes" => "reading_episodes",
        "jf_users" => "reading_users",
        "jf_playback_activity" => "importing_history",
        "jf_item_info" => "reading_file_details",
        _ => "reading_backup",
    };
    crate::i18n::t(&format!("server.import.{key}"), &[])
}

/// The most one row of a backup may be, as a line or inside one document. A row of the largest export seen is a few
/// hundred kilobytes. Read as JSON, a row takes many times its size (`[0,0,…]` is 32 bytes for every two), so a row
/// the upload limit allowed ran the process out of memory inside the import's transaction.
pub(crate) const ROW_MAX: u64 = 8 << 20;

/// The longest line a backup's reader takes: one row. A file that is one endless line (or a little gzip that inflates
/// into one) was otherwise kept whole in memory.
pub(crate) const LINE_MAX: u64 = ROW_MAX;

thread_local! {
    /// What the row being read may still take, while one is (`None` between rows: stepping over a section nobody reads
    /// keeps nothing).
    static ROW_LEFT: std::cell::Cell<Option<u64>> = const { std::cell::Cell::new(None) };
}

/// Read one row of a document through `read` with at most `ROW_MAX` of the file under it. The reader must be a
/// [`RowBudget`].
pub(crate) fn one_row<T>(read: impl FnOnce() -> T) -> T {
    ROW_LEFT.set(Some(ROW_MAX));
    let row = read();
    ROW_LEFT.set(None);
    row
}

/// A reader that refuses to go past `ROW_MAX` inside a row being read through [`one_row`].
pub(crate) struct RowBudget<R>(pub R);

impl<R: Read> Read for RowBudget<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let Some(left) = ROW_LEFT.get() else { return self.0.read(buf) };
        if left == 0 {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, crate::i18n::t("server.import.line_too_long", &[])));
        }
        let most = buf.len().min(left as usize);
        let n = self.0.read(&mut buf[..most])?;
        ROW_LEFT.set(Some(left - n as u64));
        Ok(n)
    }
}

/// `read_line`, refusing a line longer than `max`.
pub(crate) fn read_line_at_most(reader: &mut impl BufRead, line: &mut String, max: u64) -> std::io::Result<usize> {
    let n = reader.by_ref().take(max + 1).read_line(line)?;
    if n as u64 > max && !line.ends_with('\n') {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, crate::i18n::t("server.import.line_too_long", &[])));
    }
    Ok(n)
}

/// A reader that counts what has been read through it, for how far along an import is.
struct Counted<'a, R> {
    inner: R,
    read: &'a std::cell::Cell<u64>,
}

impl<R: Read> Read for Counted<'_, R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = self.inner.read(buf)?;
        self.read.set(self.read.get() + n as u64);
        Ok(n)
    }
}

/// The older Jellystat format, `[{"<table>": [rows…]}, …]`, read as it arrives: each row is handed to `each` with its
/// table as soon as it has been read. Read whole into memory first, it took several times the file's size.
fn legacy_rows<R: Read>(reader: R, mut each: impl FnMut(&str, Value) -> Result<()>) -> Result<()> {
    let mut de = serde_json::Deserializer::from_reader(RowBudget(reader));
    serde::de::DeserializeSeed::deserialize(legacy::Sections(&mut each), &mut de)?;
    de.end()?;
    Ok(())
}

mod legacy {
    use anyhow::Result;
    use serde::de::{DeserializeSeed, Error, IgnoredAny, MapAccess, SeqAccess, Visitor};
    use serde_json::Value;
    use std::fmt::Formatter;

    /// Anything where a section or its rows should be is stepped over, as the whole-document reader did.
    macro_rules! step_over {
        () => {
            fn visit_bool<E: Error>(self, _: bool) -> Result<(), E> { Ok(()) }
            fn visit_i64<E: Error>(self, _: i64) -> Result<(), E> { Ok(()) }
            fn visit_u64<E: Error>(self, _: u64) -> Result<(), E> { Ok(()) }
            fn visit_f64<E: Error>(self, _: f64) -> Result<(), E> { Ok(()) }
            fn visit_str<E: Error>(self, _: &str) -> Result<(), E> { Ok(()) }
            fn visit_unit<E: Error>(self) -> Result<(), E> { Ok(()) }
        };
    }

    pub struct Sections<'a, F>(pub &'a mut F);

    impl<'de, F: FnMut(&str, Value) -> Result<()>> DeserializeSeed<'de> for Sections<'_, F> {
        type Value = ();
        fn deserialize<D: serde::Deserializer<'de>>(self, de: D) -> Result<(), D::Error> {
            de.deserialize_seq(self)
        }
    }

    impl<'de, F: FnMut(&str, Value) -> Result<()>> Visitor<'de> for Sections<'_, F> {
        type Value = ();
        fn expecting(&self, f: &mut Formatter) -> std::fmt::Result {
            f.write_str("a list of tables")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<(), A::Error> {
            while seq.next_element_seed(Section(&mut *self.0))?.is_some() {}
            Ok(())
        }
    }

    struct Section<'a, F>(&'a mut F);

    impl<'de, F: FnMut(&str, Value) -> Result<()>> DeserializeSeed<'de> for Section<'_, F> {
        type Value = ();
        fn deserialize<D: serde::Deserializer<'de>>(self, de: D) -> Result<(), D::Error> {
            de.deserialize_any(self)
        }
    }

    impl<'de, F: FnMut(&str, Value) -> Result<()>> Visitor<'de> for Section<'_, F> {
        type Value = ();
        fn expecting(&self, f: &mut Formatter) -> std::fmt::Result {
            f.write_str("a table and its rows")
        }
        fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<(), A::Error> {
            while let Some(table) = map.next_key::<String>()? {
                map.next_value_seed(Rows { table: &table, each: &mut *self.0 })?;
            }
            Ok(())
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<(), A::Error> {
            while seq.next_element::<IgnoredAny>()?.is_some() {}
            Ok(())
        }
        step_over!();
    }

    struct Rows<'a, F> {
        table: &'a str,
        each: &'a mut F,
    }

    impl<'de, F: FnMut(&str, Value) -> Result<()>> DeserializeSeed<'de> for Rows<'_, F> {
        type Value = ();
        fn deserialize<D: serde::Deserializer<'de>>(self, de: D) -> Result<(), D::Error> {
            de.deserialize_any(self)
        }
    }

    impl<'de, F: FnMut(&str, Value) -> Result<()>> Visitor<'de> for Rows<'_, F> {
        type Value = ();
        fn expecting(&self, f: &mut Formatter) -> std::fmt::Result {
            f.write_str("a list of rows")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<(), A::Error> {
            while let Some(row) = super::one_row(|| seq.next_element::<Value>())? {
                (self.each)(self.table, row).map_err(|e| A::Error::custom(format!("{e:#}")))?;
            }
            Ok(())
        }
        fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<(), A::Error> {
            while map.next_entry::<IgnoredAny, IgnoredAny>()?.is_some() {}
            Ok(())
        }
        step_over!();
    }
}

/// Blocking. Reads the backup at `path` through once with no lock held, keeping aside the rows worth writing
/// ([`Spool`]), then writes those in a single transaction, so a failed import leaves the database exactly as it was.
/// Read inside the transaction, a file of a few real rows and gigabytes of anything else kept every other write
/// waiting until it had been read to the end.
pub fn run(db: &Db, path: &Path, tasks: Option<&Tasks>) -> Result<ImportResult> {
    let file = File::open(path).context("opening the uploaded backup")?;
    let total_bytes = file.metadata()?.len().max(1);
    let mut reader = BufReader::with_capacity(1 << 20, file);

    // The older Jellystat format is one JSON array; the current one is a record per line.
    let legacy = loop {
        let buf = reader.fill_buf()?;
        let Some(&first) = buf.first() else { bail!(crate::i18n::t("server.import.empty_file", &[])) };
        if first.is_ascii_whitespace() {
            reader.consume(1);
            continue;
        }
        break first == b'[';
    };

    let mut res = ImportResult::default();
    let report = |msg: &str, p: f64| {
        if let Some(t) = tasks {
            t.update(TASK, msg, Some(p));
        }
    };
    let mut spool = Spool::beside(path)?;

    if legacy {
        report(&crate::i18n::t("server.import.reading_legacy", &[]), 0.05);
        let read = std::cell::Cell::new(0u64);
        let mut current = String::new();
        let mut failed = None;
        let parsed = legacy_rows(Counted { inner: reader, read: &read }, |table, row| {
            if table != current {
                current = table.to_string();
                report(&phase_label(table), 0.05 + 0.45 * read.get() as f64 / total_bytes as f64);
            }
            spool.keep(table, row, &mut res).inspect_err(|e| failed = Some(anyhow::anyhow!("{e:#}")))
        });
        if let Some(e) = failed {
            return Err(e);
        }
        parsed.context(crate::i18n::t("server.import.not_json", &[]))?;
    } else {
        let mut line = String::new();
        let mut read_bytes = 0u64;
        let mut line_no = 0u64;
        let mut recognised = 0u64;
        let mut current_table = String::new();
        loop {
            line.clear();
            let n = read_line_at_most(&mut reader, &mut line, LINE_MAX)?;
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
                    bail!(crate::i18n::t("server.import.not_jellystat_at", &[("line", line_no.to_string().into()), ("error", e.to_string().into())]))
                }
                Err(e) => bail!(crate::i18n::t("server.import.damaged_at", &[("line", line_no.to_string().into()), ("error", e.to_string().into())])),
            };
            match v["type"].as_str() {
                Some("table") => {
                    recognised += 1;
                    current_table = v["table"].as_str().unwrap_or_default().to_string();
                }
                Some("row") => {
                    recognised += 1;
                    let table = v["table"].as_str().map(str::to_string).unwrap_or_else(|| current_table.clone());
                    spool.keep(&table, v["data"].take(), &mut res)?;
                }
                _ => res.unknown_rows += 1,
            }
            if line_no.is_multiple_of(2000) {
                report(&phase_label(&current_table), 0.5 * read_bytes as f64 / total_bytes as f64);
            }
            if line_no == 50 && recognised == 0 {
                bail!(crate::i18n::t("server.import.no_records_at_first", &[]));
            }
        }
        if recognised == 0 {
            bail!(crate::i18n::t("server.import.no_records", &[]));
        }
    }

    let mut conn = db.conn()?;
    let settings = crate::state::Settings::load(&conn)?;
    // Immediate: the import will write, so it takes the write lock before its first read rather than
    // failing to upgrade later when another connection (the collector, an audit row) commits meanwhile.
    let tx = conn.transaction_with_behavior(crate::db::rusqlite::TransactionBehavior::Immediate)?;
    let kept = spool.kept();
    let mut current = String::new();
    for (n, row) in spool.rows()?.enumerate() {
        let (table, data) = row?;
        if table != current || n % 2000 == 0 {
            report(&phase_label(&table), 0.5 + 0.45 * n as f64 / kept.max(1) as f64);
            current = table.clone();
        }
        import_row(&tx, &table, &data, &mut res, settings.merge_window_s)?;
    }

    report(&crate::i18n::t("server.import.linking", &[]), 0.96);
    finalize(&tx)?;
    report(&crate::i18n::t("server.import.saving", &[]), 0.99);
    tx.commit()?;
    crate::groups::detect(&mut conn, settings.group_window_s, None)?;
    conn.execute_batch("PRAGMA optimize;")?;
    Ok(res)
}

/// The tables `import_row` writes anything from; a row of any other is counted, and never kept aside.
const WRITTEN: [&str; 7] = ["jf_playback_activity", "jf_users", "jf_libraries", "jf_library_items", "jf_library_seasons", "jf_library_episodes", "jf_item_info"];

/// The rows of an upload worth writing, kept on disk beside it while the upload is read, so the read holds no lock and
/// the write that follows reads only them. Removed when it is dropped.
pub(crate) struct Spool {
    path: std::path::PathBuf,
    out: Option<std::io::BufWriter<File>>,
    kept: u64,
}

impl Spool {
    /// A spool next to the upload at `upload`, in the same folder (the data folder's own, for an upload); for a file the
    /// command line was given in a folder FinStats may not write to, in the temporary folder. Readable by FinStats alone.
    pub(crate) fn beside(upload: &Path) -> Result<Spool> {
        let mut name = upload.file_name().map(|n| n.to_os_string()).unwrap_or_default();
        name.push(".rows");
        let open = |path: std::path::PathBuf| -> std::io::Result<Spool> {
            let _ = std::fs::remove_file(&path);
            let file = crate::db::private_new_file().open(&path)?;
            Ok(Spool { path, out: Some(std::io::BufWriter::new(file)), kept: 0 })
        };
        open(upload.with_file_name(&name))
            .or_else(|_| open(std::env::temp_dir().join(format!("finstats-{}-{}", std::process::id(), name.to_string_lossy()))))
            .context("keeping the backup's rows aside")
    }

    /// One row of `table`: kept when it is one that is written, else counted as unknown.
    fn keep(&mut self, table: &str, row: Value, res: &mut ImportResult) -> Result<()> {
        match table {
            "jf_playback_reporting_plugin_data" => Ok(()),
            t if WRITTEN.contains(&t) => self.push(table, row),
            _ => {
                res.unknown_rows += 1;
                Ok(())
            }
        }
    }

    /// Keep one row under `table` (any word the reader of [`Spool::rows`] understands).
    pub(crate) fn push(&mut self, table: &str, row: Value) -> Result<()> {
        use std::io::Write;
        let out = self.out.as_mut().expect("a spool is written before it is read");
        serde_json::to_writer(&mut *out, &(table, row))?;
        out.write_all(b"\n")?;
        self.kept += 1;
        Ok(())
    }

    pub(crate) fn kept(&self) -> u64 {
        self.kept
    }

    /// What was kept, in the order it came.
    pub(crate) fn rows(&mut self) -> Result<impl Iterator<Item = Result<(String, Value)>> + '_> {
        use std::io::Write;
        if let Some(mut out) = self.out.take() {
            out.flush()?;
        }
        let lines = BufReader::new(File::open(&self.path)?).lines();
        Ok(lines.map(|l| Ok(serde_json::from_str::<(String, Value)>(&l?)?)))
    }
}

impl Drop for Spool {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
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
        // A title in a library a read has filled, and that the read did not see, is a title that is gone: it names the
        // plays it had, never one more live title.
        "jf_library_items" => {
            let Some(id) = d["Id"].as_str() else { return Ok(()) };
            let genres = d["Genres"].as_array().filter(|g| !g.is_empty()).map(|g| Value::Array(g.clone()).to_string());
            res.items += conn
                .prepare_cached(
                    "INSERT OR IGNORE INTO items(id, library_id, type, name, runtime_s, production_year, premiere_date,
                        date_created, community_rating, genres, image_tag, backdrop_tag, removed, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13 OR EXISTS (SELECT 1 FROM items WHERE library_id = ?2 AND updated_at > 0), 0)",
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
                    crate::playback::index_number(&d["IndexNumber"]),
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
                        community_rating, official_rating, removed, index_number_end, updated_at)
                     VALUES (?1, 'Episode', ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, 0)",
                )?
                .execute(params![
                    norm_id(id),
                    d["Name"].as_str().unwrap_or("Episode"),
                    d["SeriesId"].as_str().map(norm_id),
                    d["SeasonId"].as_str().map(norm_id),
                    opt_str(&d["SeriesName"]),
                    crate::playback::index_number(&d["IndexNumber"]),
                    crate::playback::index_number(&d["ParentIndexNumber"]),
                    ticks_to_s(&d["RunTimeTicks"]),
                    d["ProductionYear"].as_i64(),
                    opt_str(&d["PremiereDate"]),
                    d["DateCreated"].as_str().and_then(parse_ts),
                    d["CommunityRating"].as_f64(),
                    opt_str(&d["OfficialRating"]),
                    d["archived"].as_bool().unwrap_or(false),
                    crate::playback::index_number(&d["IndexNumberEnd"]),
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
        started_at: ended_at.saturating_sub(duration_s),
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

    /// A film, a show with one episode, alice, and one play of each, in the line-per-record format.
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

    /// Reading the file takes no lock: the write lock is taken only once what is worth writing has been read. Held from the
    /// first line, a file of a valid head and gigabytes of lines of no known kind kept every other write waiting, and
    /// failing, until it had been read to the end.
    #[test]
    fn nothing_waits_on_an_import_while_it_reads_the_file() {
        use std::io::Write;
        let (db, _, dir) = backup("reading", "");
        let fifo = dir.join("upload.jsonl");
        assert!(std::process::Command::new("mkfifo").arg(&fifo).status().unwrap().success());
        let (reader_db, path) = (db.clone(), fifo.clone());
        let import = std::thread::spawn(move || run(&reader_db, &path, None));
        let mut upload = std::fs::OpenOptions::new().write(true).open(&fifo).unwrap();
        writeln!(upload, r#"{{"type":"table","table":"jf_users"}}"#).unwrap();
        writeln!(upload, r#"{{"type":"row","data":{{"Id":"AAAA-1111","Name":"alice"}}}}"#).unwrap();
        writeln!(upload, r#"{{"type":"x","n":[0,0,0]}}"#).unwrap();
        upload.flush().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(300));
        let other = db.conn().unwrap();
        other.busy_timeout(std::time::Duration::ZERO).unwrap();
        let wrote = other.execute("INSERT INTO welcomed(user_id, at) VALUES ('bob', 1)", []);
        writeln!(upload, r#"{{"type":"x","n":[0,0,0]}}"#).unwrap();
        drop(upload);
        let res = import.join().unwrap().unwrap();
        assert!(wrote.is_ok(), "a write waited on an import still reading its file: {wrote:?}");
        assert_eq!((res.users, res.unknown_rows), (1, 2));
        let _ = std::fs::remove_dir_all(dir);
    }

    /// Titles a backup brings into a library a read has already filled are not the read's to lose: the guard against a
    /// broken read counts only what a read stored, and such titles come in as gone. A file of 1,200 invented titles in a
    /// 50-title library made every library read refuse and halt FinStats, at every start, blaming Jellyfin.
    #[test]
    fn titles_a_backup_plants_in_a_library_already_read_never_halt_the_next_read() {
        let mut text = String::from("{\"type\":\"table\",\"table\":\"jf_library_items\"}\n");
        for n in 0..1200 {
            text.push_str(&format!("{{\"type\":\"row\",\"data\":{{\"Id\":\"PLANTED-{n}\",\"ParentId\":\"LIB-1\",\"Type\":\"Movie\",\"Name\":\"x\"}}}}\n"));
        }
        let (db, file, dir) = backup("planted", &text);
        {
            let c = db.conn().unwrap();
            for n in 0..50 {
                c.execute("INSERT INTO items(id, library_id, type, name, removed, updated_at) VALUES (?1, 'lib1', 'Movie', 'Big Buck Bunny', 0, 1000)", [format!("real{n}")]).unwrap();
            }
        }
        run(&db, &file, None).unwrap();
        let c = db.conn().unwrap();
        let current = crate::sync::held_by_reads(&c, "lib1").unwrap();
        assert_eq!(current, 50, "titles only a backup brought were counted as the read's");
        assert!(crate::sync::trustworthy_removal(50, current));
        let live: i64 = c.query_row("SELECT COUNT(*) FROM items WHERE library_id = 'lib1' AND removed = 0", [], |r| r.get(0)).unwrap();
        assert_eq!(live, 50, "a backup planted live titles in a library a read had filled");
        let _ = std::fs::remove_dir_all(dir);
    }

    /// Jellystat's seasons table: each season becomes one, under its show by Jellyfin's id, and one Jellystat archived
    /// (gone from the server) is kept as removed rather than as a season to show.
    #[test]
    fn seasons_are_read_under_their_show_and_an_archived_one_is_removed() {
        let db = Db::open_in_memory().unwrap();
        let c = db.conn().unwrap();
        let mut res = ImportResult::default();
        for row in [
            json!({ "Id": "SEASON-1", "Name": "Season 1", "SeriesId": "SHOW-1", "SeriesName": "Low Orbit", "IndexNumber": 1 }),
            json!({ "Id": "SEASON-0", "Name": "Specials", "SeriesId": "SHOW-1", "SeriesName": "Low Orbit", "IndexNumber": 0, "archived": true }),
            json!({ "Name": "A season with no id" }),
        ] {
            import_row(&c, "jf_library_seasons", &row, &mut res, 300).unwrap();
        }
        assert_eq!(res.seasons, 2, "a row without an id is not a season");
        let seasons: Vec<(String, String, String, Option<String>, Option<i64>, bool)> = c
            .prepare("SELECT id, type, name, series_id, index_number, removed FROM items WHERE type = 'Season' ORDER BY index_number")
            .unwrap()
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?)))
            .unwrap()
            .map(Result::unwrap)
            .collect();
        assert_eq!(seasons, vec![
            ("season0".into(), "Season".into(), "Specials".into(), Some("show1".into()), Some(0), true),
            ("season1".into(), "Season".into(), "Season 1".into(), Some("show1".into()), Some(1), false),
        ]);
        // Twice is once.
        import_row(&c, "jf_library_seasons", &json!({ "Id": "SEASON-1", "Name": "Season 1", "SeriesId": "SHOW-1", "IndexNumber": 1 }), &mut res, 300).unwrap();
        assert_eq!(res.seasons, 2);
    }

    /// An export that kept where a two-episode file ends brings it, so the file reads as no gap before the library is read.
    #[test]
    fn an_imported_episode_keeps_where_its_file_ends() {
        let db = Db::open_in_memory().unwrap();
        let c = db.conn().unwrap();
        let mut res = ImportResult::default();
        let row = json!({ "EpisodeId": "EP-1", "Name": "Pilot", "SeriesId": "SHOW-1", "IndexNumber": 1, "IndexNumberEnd": 2, "ParentIndexNumber": 1 });
        import_row(&c, "jf_library_episodes", &row, &mut res, 300).unwrap();
        let end: Option<i64> = c.query_row("SELECT index_number_end FROM items WHERE id = 'ep1'", [], |r| r.get(0)).unwrap();
        assert_eq!(end, Some(2));
    }

    /// Jellyfin keeps a season or episode number as an `int32` and never below zero: a number past that in a file is no
    /// number at all, or Library health counted every number between it and the next episode.
    #[test]
    fn an_episode_number_no_jellyfin_could_have_is_no_number() {
        let db = Db::open_in_memory().unwrap();
        let c = db.conn().unwrap();
        let mut res = ImportResult::default();
        let row = json!({ "EpisodeId": "EP-1", "Name": "Pilot", "SeriesId": "SHOW-1", "IndexNumber": i64::MIN, "IndexNumberEnd": i64::MAX, "ParentIndexNumber": -1 });
        import_row(&c, "jf_library_episodes", &row, &mut res, 300).unwrap();
        import_row(&c, "jf_library_seasons", &json!({ "Id": "SEASON-1", "Name": "Season 1", "SeriesId": "SHOW-1", "IndexNumber": 1i64 << 40 }), &mut res, 300).unwrap();
        let numbers: (Option<i64>, Option<i64>, Option<i64>) =
            c.query_row("SELECT index_number, parent_index_number, index_number_end FROM items WHERE id = 'ep1'", [], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))).unwrap();
        assert_eq!(numbers, (None, None, None));
        let season: Option<i64> = c.query_row("SELECT index_number FROM items WHERE id = 'season1'", [], |r| r.get(0)).unwrap();
        assert_eq!(season, None);
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

    /// A line is read no further than the longest one a backup could hold: a file that is one endless line (or a little gzip
    /// that inflates into one) was otherwise kept whole in memory.
    #[test]
    fn a_line_longer_than_any_backup_holds_is_refused_not_kept() {
        let mut r = std::io::BufReader::new(&b"short\nmuch too long a line\nlast"[..]);
        let mut line = String::new();
        assert_eq!(read_line_at_most(&mut r, &mut line, 10).unwrap(), 6);
        line.clear();
        assert!(read_line_at_most(&mut r, &mut line, 10).is_err());
        let mut r = std::io::BufReader::new(&b"last"[..]);
        assert_eq!(read_line_at_most(&mut r, &mut line, 10).unwrap(), 4, "a last line with no end");
    }

    /// The older format is one document, read as it arrives: a row is handed over as soon as it has been read, never after
    /// the whole file is in memory (several times its size, for a file up to the largest an upload may be).
    #[test]
    fn the_older_format_is_read_a_row_at_a_time() {
        let cut = r#"[{"jf_users":[{"Id":"A"},{"Id":"B"}]},{"jf_playback_activity":[{"Id":"p1"},{"Id":"p2"#;
        let mut seen = vec![];
        let r = legacy_rows(cut.as_bytes(), |table, row| {
            seen.push(format!("{table}/{}", row["Id"].as_str().unwrap_or_default()));
            Ok(())
        });
        assert!(r.is_err(), "a document that stops half way");
        assert_eq!(seen, ["jf_users/A", "jf_users/B", "jf_playback_activity/p1"]);
    }

    /// One row of the older format had no limit: `[[0,0,0,…]]` as a row is 32 bytes of memory for every two of file, so a
    /// file the upload limit allows ran the process out of memory inside the import's transaction. A row stops at
    /// `ROW_MAX`, and what is stepped over (a section nobody reads) is not counted against it.
    #[test]
    fn one_row_of_the_older_format_is_read_no_further_than_any_row_could_be() {
        let row = format!("[{{\"jf_users\":[[{}0]]}}]", "0,".repeat(ROW_MAX as usize));
        let began = std::time::Instant::now();
        assert!(legacy_rows(row.as_bytes(), |_, _| Ok(())).is_err(), "a row past the most a row may be");
        assert!(began.elapsed() < std::time::Duration::from_secs(5));
        let skipped = format!("[[{}0],{{\"jf_users\":[{{\"Id\":\"A\"}}]}}]", "0,".repeat(ROW_MAX as usize));
        let mut seen = 0;
        legacy_rows(skipped.as_bytes(), |_, _| { seen += 1; Ok(()) }).unwrap();
        assert_eq!(seen, 1);
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
        assert_eq!(kept, 999, "what FinStats read from Jellyfin itself is fresher than an old backup");
        let _ = std::fs::remove_dir_all(dir);
    }
}
