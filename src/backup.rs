//! finstats' own backups: everything that cannot be had again from Jellyfin, in one file.
//!
//! The format is gzip-compressed JSON Lines (`finstats-backup-YYYYMMDD-HHMMSS.jsonl.gz`). The first
//! line describes the backup; every other line is one row, `{"t": "<table>", "r": {column: value}}`.
//! A row is written and read back *by column name*, so a backup from an older finstats restores into
//! a newer one (missing columns take their defaults) and the other way round (unknown columns are
//! dropped). It streams both ways: size is bounded by disk, not memory.
//!
//! What is in it: plays and their timelines, manual seen-marks, permissions, home addresses, the
//! server log, known devices, finstats' own audit log and the settings. What is not, on purpose: the Jellyfin address and API
//! key, sign-in sessions, API keys, and the library itself (items, people, played flags), which the next sync
//! reads from Jellyfin again. A backup is still a complete viewing history with IP addresses in it:
//! treat the file accordingly.
//!
//! Restoring *merges*. A play that is already there (same import id, or same user, item and start)
//! is skipped, so restoring twice, or into an instance that has been running for a while, is safe.

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use serde::Serialize;
use serde_json::{Map, Value, json};

use crate::db::rusqlite::types::ValueRef;
use crate::db::rusqlite::{Connection, params_from_iter};
use crate::db::{self, Db, SqlValue};
use crate::state::{Settings, Tasks};

pub const FORMAT: i64 = 1;
const PREFIX: &str = "finstats-backup-";
const SUFFIX: &str = ".jsonl.gz";

/// In the order they are written, which is also the order they must be read in: a timeline row
/// needs its play to exist.
const TABLES: [&str; 11] = ["playbacks", "playback_events", "manual_seen", "user_permissions", "home_addresses", "server_events", "devices", "security_alerts", "audit", "watchlist", "located"];

pub fn dir(data_dir: &Path) -> PathBuf {
    data_dir.join("backups")
}

pub fn new_name() -> String {
    format!("{PREFIX}{}{SUFFIX}", chrono::Local::now().format("%Y%m%d-%H%M%S"))
}

/// Only names finstats itself would give a backup. This is what stands between a download or
/// delete request and the rest of the disk.
pub fn valid_name(name: &str) -> bool {
    name.strip_prefix(PREFIX)
        .and_then(|rest| rest.strip_suffix(SUFFIX))
        .is_some_and(|stamp| stamp.len() == 15 && stamp.bytes().enumerate().all(|(i, b)| if i == 8 { b == b'-' } else { b.is_ascii_digit() }))
}

#[derive(Serialize, Default, Debug)]
pub struct ExportResult {
    pub name: String,
    pub size_bytes: u64,
    pub plays: i64,
    pub rows: i64,
}

#[derive(Serialize, Default, Debug)]
pub struct RestoreResult {
    pub plays_imported: i64,
    pub plays_skipped: i64,
    pub events: i64,
    pub other_rows: i64,
    pub settings_restored: bool,
    pub from_version: Option<String>,
}

fn raw_row(row: &crate::db::rusqlite::Row) -> Map<String, Value> {
    let stmt = row.as_ref();
    let mut m = Map::new();
    for i in 0..stmt.column_count() {
        let v = match row.get_ref(i) {
            Ok(ValueRef::Integer(n)) => json!(n),
            Ok(ValueRef::Real(f)) => json!(f),
            Ok(ValueRef::Text(t)) => json!(String::from_utf8_lossy(t)),
            _ => Value::Null,
        };
        m.insert(stmt.column_name(i).unwrap_or_default().to_string(), v);
    }
    m
}

/// Write a backup into `dir`. The file only gets its real name once it is complete.
pub fn export(db: &Db, dir: &Path, tasks: Option<(&Tasks, &'static str)>) -> Result<ExportResult> {
    std::fs::create_dir_all(dir).context("creating the backups folder")?;
    // Only one backup is written at a time, so a partial file already here was a backup that was killed. Only files
    // with a backup's own name are touched.
    if let Ok(entries) = std::fs::read_dir(dir) {
        for e in entries.flatten() {
            let file = e.file_name();
            if file.to_str().and_then(|n| n.strip_suffix(".part")).is_some_and(valid_name) {
                let _ = std::fs::remove_file(e.path());
            }
        }
    }
    let name = new_name();
    let (part, path) = (dir.join(format!("{name}.part")), dir.join(&name));
    let report = |msg: String, p: f64| {
        if let Some((t, id)) = tasks {
            t.update(id, msg, Some(p));
        }
    };

    let mut conn = db.conn()?;
    // One read transaction: the file is a snapshot of a single moment even while plays keep arriving. Deferred by name,
    // because a pooled connection's transactions take the write lock by default and this one must never hold it.
    let tx = conn.transaction_with_behavior(crate::db::rusqlite::TransactionBehavior::Deferred)?;
    let count = |table: &str| -> Result<i64> { Ok(tx.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))?) };
    let mut counts = Map::new();
    for t in TABLES {
        counts.insert(t.to_string(), json!(count(t)?));
    }
    let total: i64 = counts.values().filter_map(Value::as_i64).sum::<i64>().max(1);

    let result = (|| -> Result<ExportResult> {
        let mut out = GzEncoder::new(BufWriter::with_capacity(1 << 20, File::create(&part)?), Compression::new(6));
        let header = json!({
            "finstats_backup": FORMAT, "app_version": env!("CARGO_PKG_VERSION"), "created_at": db::now(),
            "server_name": db::get_setting(&tx, "server_name")?, "counts": counts.clone(),
        });
        writeln!(out, "{header}")?;
        if let Some(raw) = db::get_setting(&tx, "settings")? {
            writeln!(out, "{}", json!({ "t": "settings", "r": { "key": "settings", "value": raw } }))?;
        }
        let mut written = 0i64;
        for table in TABLES {
            // Most of these tables are WITHOUT ROWID and come out in primary-key order by themselves.
            let order = if matches!(table, "playbacks" | "playback_events" | "server_events" | "audit") { " ORDER BY id" } else { "" };
            let mut stmt = tx.prepare(&format!("SELECT * FROM {table}{order}"))?;
            let mut rows = stmt.query([])?;
            while let Some(row) = rows.next()? {
                serde_json::to_writer(&mut out, &json!({ "t": table, "r": raw_row(row) }))?;
                out.write_all(b"\n")?;
                written += 1;
                if written % 5000 == 0 {
                    report(format!("Writing {table}: {written} of {total} rows"), written as f64 / total as f64);
                }
            }
        }
        out.finish()?.into_inner().map_err(|e| anyhow::anyhow!("{e}"))?.sync_all()?;
        std::fs::rename(&part, &path)?;
        Ok(ExportResult { name: name.clone(), size_bytes: std::fs::metadata(&path)?.len(), plays: counts["playbacks"].as_i64().unwrap_or(0), rows: written })
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&part);
    }
    result
}

/// The backups on disk, newest first.
pub fn list(dir: &Path) -> Vec<Value> {
    let mut out: Vec<(i64, Value)> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            if !valid_name(&name) {
                return None;
            }
            let meta = e.metadata().ok()?;
            let at = meta.modified().ok()?.duration_since(std::time::UNIX_EPOCH).ok()?.as_secs() as i64;
            Some((at, json!({ "name": name, "size_bytes": meta.len(), "created_at": at })))
        })
        .collect();
    out.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| b.1["name"].as_str().cmp(&a.1["name"].as_str())));
    out.into_iter().map(|(_, v)| v).collect()
}

pub fn newest_at(dir: &Path) -> Option<i64> {
    list(dir).first().and_then(|b| b["created_at"].as_i64())
}

/// Keep the newest `keep` backups. Returns how many were removed.
pub fn prune(dir: &Path, keep: usize) -> usize {
    let mut removed = 0;
    for b in list(dir).into_iter().skip(keep.max(1)) {
        if let Some(name) = b["name"].as_str() {
            if std::fs::remove_file(dir.join(name)).is_ok() {
                removed += 1;
            }
        }
    }
    removed
}

fn table_columns(conn: &Connection, table: &str) -> Result<HashSet<String>> {
    let mut stmt = conn.prepare(&format!("SELECT name FROM pragma_table_info('{table}')"))?;
    let cols = stmt.query_map([], |r| r.get::<_, String>(0))?.collect::<Result<HashSet<_>, _>>()?;
    Ok(cols)
}

fn sql_value(v: &Value) -> SqlValue {
    match v {
        Value::Null => SqlValue::Null,
        Value::Bool(b) => (*b as i64).into(),
        Value::Number(n) => n.as_i64().map(SqlValue::from).or_else(|| n.as_f64().map(SqlValue::from)).unwrap_or(SqlValue::Null),
        Value::String(s) => s.clone().into(),
        other => other.to_string().into(),
    }
}

/// Insert one row by column name. Columns this database does not have are dropped; returns whether a row went in.
fn insert_row(conn: &Connection, table: &str, known: &HashSet<String>, row: &Map<String, Value>, skip: &[&str], verb: &str) -> Result<bool> {
    let cols: Vec<&String> = row.keys().filter(|k| known.contains(*k) && !skip.contains(&k.as_str())).collect();
    if cols.is_empty() {
        return Ok(false);
    }
    let sql = format!(
        "{verb} INTO {table}({}) VALUES ({})",
        cols.iter().map(|c| format!("\"{c}\"")).collect::<Vec<_>>().join(", "),
        vec!["?"; cols.len()].join(", ")
    );
    let args: Vec<SqlValue> = cols.iter().map(|c| sql_value(&row[*c])).collect();
    Ok(conn.prepare_cached(&sql)?.execute(params_from_iter(args.iter()))? > 0)
}

/// Merge a backup into this database. `with_settings` also brings back the settings and permissions.
pub fn restore(db: &Db, path: &Path, with_settings: bool, tasks: Option<(&Tasks, &'static str)>) -> Result<RestoreResult> {
    let mut file = File::open(path).context("opening the backup")?;
    let total_bytes = file.metadata()?.len().max(1);
    let mut magic = [0u8; 2];
    let gz = file.read(&mut magic)? == 2 && magic == [0x1f, 0x8b];
    file.seek(SeekFrom::Start(0))?;
    // Progress follows the compressed bytes consumed, which is the only total known up front.
    let counter = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
    let counted = Counted { inner: file, read: counter.clone() };
    let reader: Box<dyn BufRead> =
        if gz { Box::new(BufReader::with_capacity(1 << 20, GzDecoder::new(counted))) } else { Box::new(BufReader::with_capacity(1 << 20, counted)) };

    let mut conn = db.conn()?;
    // This install's own idea of how far apart two sightings can be and still be one viewing.
    let merge_window_s = Settings::load(&conn)?.merge_window_s;
    let tx = conn.transaction()?;
    let mut res = RestoreResult::default();
    let mut known: HashMap<&'static str, HashSet<String>> = HashMap::new();
    for t in TABLES {
        known.insert(t, table_columns(&tx, t)?);
    }
    // Old play id → the id it got here. Plays that were already present are absent: their timeline is too.
    let mut play_ids: HashMap<i64, i64> = HashMap::new();
    let mut settings_raw: Option<String> = None;
    let mut saw_header = false;

    for (n, line) in reader.lines().enumerate() {
        let line = line.context("The file is damaged or not a finstats backup (could not be read)")?;
        if line.trim().is_empty() {
            continue;
        }
        let v: Value = serde_json::from_str(&line).with_context(|| format!("The file is not a finstats backup (line {} is not JSON)", n + 1))?;
        if !saw_header {
            let Some(format) = v["finstats_backup"].as_i64() else {
                bail!("This is not a finstats backup. A Jellystat backup goes under “Import from Jellystat”.");
            };
            if format > FORMAT {
                bail!("This backup was made by a newer finstats (format {format}). Update finstats, then restore it.");
            }
            res.from_version = v["app_version"].as_str().map(str::to_string);
            saw_header = true;
            continue;
        }
        let (Some(table), Some(row)) = (v["t"].as_str(), v["r"].as_object()) else { continue };
        match table {
            "settings" => settings_raw = row.get("value").and_then(Value::as_str).map(str::to_string),
            "playbacks" => {
                let exists = crate::playback::already_recorded(
                    &tx,
                    crate::playback::Play {
                        source: row.get("source").and_then(Value::as_str).unwrap_or("live"),
                        source_id: row.get("source_id").and_then(Value::as_str),
                        user_id: row.get("user_id").and_then(Value::as_str).unwrap_or_default(),
                        item_id: row.get("item_id").and_then(Value::as_str).unwrap_or_default(),
                        started_at: row.get("started_at").and_then(Value::as_i64).unwrap_or_default(),
                        ended_at: row.get("ended_at").and_then(Value::as_i64).unwrap_or_default(),
                    },
                    merge_window_s,
                )?;
                if exists {
                    res.plays_skipped += 1;
                    continue;
                }
                // Nothing restored is still playing, and who watched together is worked out again afterwards.
                let mut row = row.clone();
                row.insert("active".into(), json!(0));
                if insert_row(&tx, "playbacks", &known["playbacks"], &row, &["id", "group_id"], "INSERT")? {
                    if let Some(old) = row.get("id").and_then(Value::as_i64) {
                        play_ids.insert(old, tx.last_insert_rowid());
                    }
                    res.plays_imported += 1;
                }
            }
            "playback_events" => {
                let Some(new_id) = row.get("playback_id").and_then(Value::as_i64).and_then(|old| play_ids.get(&old)) else { continue };
                let mut row = row.clone();
                row.insert("playback_id".into(), json!(new_id));
                if insert_row(&tx, "playback_events", &known["playback_events"], &row, &["id"], "INSERT")? {
                    res.events += 1;
                }
            }
            "user_permissions" => {
                if with_settings && insert_row(&tx, table, &known["user_permissions"], row, &[], "INSERT OR REPLACE")? {
                    res.other_rows += 1;
                }
            }
            // `located`: where the owner said a title is. A choice this database already has for the same id stands.
            "manual_seen" | "home_addresses" | "server_events" | "devices" | "located" => {
                let t = TABLES.iter().find(|t| **t == table).copied().unwrap_or("devices");
                if insert_row(&tx, t, &known[t], row, &[], "INSERT OR IGNORE")? {
                    res.other_rows += 1;
                }
            }
            // Somebody's list: one entry per title and person, the older date kept.
            "watchlist" if crate::watchlist::restore_row(&tx, row)? => res.other_rows += 1,
            "watchlist" => {}
            // Its own ids mean nothing here; `dedupe` keeps an alert this database already has from doubling.
            "security_alerts" if insert_row(&tx, table, &known["security_alerts"], row, &["id"], "INSERT OR IGNORE")? => res.other_rows += 1,
            "security_alerts" => {}
            // Its ids mean nothing here either, and it has no natural key: a row is appended unless the
            // same thing, by the same person, at the same moment, is already written down.
            "audit" => {
                let get = |k: &str| row.get(k).cloned().unwrap_or(Value::Null);
                let dup: i64 = tx.query_row(
                    "SELECT COUNT(*) FROM audit WHERE at = ?1 AND kind = ?2 AND COALESCE(user_id, '') = COALESCE(?3, '') AND COALESCE(target, '') = COALESCE(?4, '') AND detail = ?5",
                    params_from_iter([sql_value(&get("at")), sql_value(&get("kind")), sql_value(&get("user_id")), sql_value(&get("target")), sql_value(&get("detail"))].iter()),
                    |r| r.get(0),
                )?;
                if dup == 0 && insert_row(&tx, "audit", &known["audit"], row, &["id"], "INSERT")? {
                    res.other_rows += 1;
                }
            }
            _ => {} // a table from a newer finstats
        }
        if n % 5000 == 0 {
            if let Some((t, id)) = tasks {
                let p = counter.load(std::sync::atomic::Ordering::Relaxed) as f64 / total_bytes as f64;
                t.update(id, format!("Restoring: {} plays so far", res.plays_imported), Some(p.min(0.97)));
            }
        }
    }
    if !saw_header {
        bail!("The file is empty");
    }

    if with_settings {
        if let Some(raw) = settings_raw {
            // Through the type, so a backup cannot plant values this version would refuse. What cannot be
            // read at all is left out, never read as the defaults: that would reset every setting there is.
            match serde_json::from_str::<Settings>(&raw) {
                Ok(parsed) if parsed.validate().is_ok() => {
                    db::set_setting(&tx, "settings", &serde_json::to_string(&parsed)?)?;
                    res.settings_restored = true;
                }
                Ok(_) => tracing::warn!("the backup's settings were not restored: this version refuses them"),
                Err(e) => tracing::warn!("the backup's settings were not restored: {e}"),
            }
        }
    }
    if let Some((t, id)) = tasks {
        t.update(id, "Linking plays to the library", Some(0.98));
    }
    let settings = Settings::load(&tx)?;
    crate::network::set_manual(&tx, &settings.home_addresses)?;
    crate::network::reclassify(&tx)?;
    crate::sync::backfill_playbacks(&tx)?;
    crate::playback::backfill_seek_origins(&tx)?;
    tx.commit()?;
    // Who watched together is derived data with a transaction of its own; the restore itself is already safe.
    crate::groups::detect(&mut conn, settings.group_window_s, None)?;
    Ok(res)
}

struct Counted {
    inner: File,
    read: std::sync::Arc<std::sync::atomic::AtomicU64>,
}

impl Read for Counted {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = self.inner.read(buf)?;
        self.read.fetch_add(n as u64, std::sync::atomic::Ordering::Relaxed);
        Ok(n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_finstats_own_backup_names_are_accepted() {
        assert!(valid_name("finstats-backup-20260920-031500.jsonl.gz"));
        assert!(valid_name(&new_name()));
        for bad in ["../finstats.db", "finstats-backup-20260920-031500.jsonl.gz/../../x", "finstats-backup-2026092-0315000.jsonl.gz", "finstats-backup-20260920-031500.jsonl", "finstats-backup-abcdefgh-ijklmn.jsonl.gz", ""] {
            assert!(!valid_name(bad), "{bad}");
        }
    }

    #[test]
    fn a_backup_killed_half_way_leaves_nothing_behind_once_the_next_one_is_written() {
        // Only one backup is ever written at a time, so a `.part` beside a new one is a backup that was killed:
        // on a big history that is gigabytes nobody will ever see, kept for good.
        let tmp = std::env::temp_dir().join(format!("finstats-backup-part-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(tmp.join("backups")).unwrap();
        let stale = tmp.join("backups").join("finstats-backup-20200101-000000.jsonl.gz.part");
        std::fs::write(&stale, b"half a backup").unwrap();
        let unrelated = tmp.join("backups").join("notes.part");
        std::fs::write(&unrelated, b"somebody else's").unwrap();
        let db = Db::open(&tmp.join("finstats.db")).unwrap();
        export(&db, &tmp.join("backups"), None).unwrap();
        assert!(!stale.exists(), "the killed backup's partial file is still there");
        assert!(unrelated.exists(), "a file finstats did not name was removed");
        drop(db);
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn a_backup_round_trips_and_restoring_twice_adds_nothing() {
        let tmp = std::env::temp_dir().join(format!("finstats-backup-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let source = Db::open(&tmp.join("source.db")).unwrap();
        {
            let c = source.conn().unwrap();
            c.execute_batch(
                "INSERT INTO playbacks(id, source, user_id, user_name, item_id, item_name, item_type, started_at, ended_at, duration_s, remote_ip, active, group_id)
                   VALUES (7, 'live', 'u1', 'alice', 'i1', 'Big Buck Bunny', 'Movie', 1000, 1600, 600, '192.168.1.10', 1, 7),
                          (9, 'jellystat', 'u2', 'bob', 'i1', 'Big Buck Bunny', 'Movie', 1005, 1600, 590, '203.0.113.7', 0, 7);
                 UPDATE playbacks SET source_id = 'jellystat:abc' WHERE id = 9;
                 INSERT INTO playback_events(playback_id, at, kind, position_s) VALUES (7, 1000, 'start', 0), (7, 1300, 'pause', 300);
                 INSERT INTO manual_seen(user_id, item_id, created_at) VALUES ('u1', 'e1', 5);
                 INSERT INTO user_permissions(user_id, permissions, updated_at) VALUES ('u2', '[\"see_everyone\"]', 5);
                 INSERT INTO settings(key, value) VALUES ('settings', '{\"min_play_s\": 42, \"home_addresses\": [\"203.0.113.7\"]}'), ('jellyfin_api_key', 'secret-key'), ('jellyfin_url', 'http://jellyfin.internal:8096'), ('device_id', 'secret-device');
                 INSERT INTO sessions(token_hash, user_id, user_name, is_admin, created_at, expires_at) VALUES ('secret-session-hash', 'u1', 'alice', 1, 1, 9999999999);
                 INSERT INTO services(kind, name, url, username, secret, created_at) VALUES ('sonarr', 'Sonarr', 'http://sonarr.internal:8989', 'secret-service-user', 'secret-service-key', 1);
                 INSERT INTO api_keys(token_hash, user_id, name, scope, created_at) VALUES ('secret-key-hash', 'u1', 'laptop', 'full', 1);
                 INSERT INTO audit(at, kind, user_id, user_name, target, detail) VALUES (5, 'setting_changed', 'u1', 'alice', NULL, '{\"changed\":[{\"key\":\"min_play_s\"}]}');",
            )
            .unwrap();
        }
        let made = export(&source, &dir(&tmp), None).unwrap();
        assert_eq!((made.plays, made.rows), (2, 7), "the audit row travels; the key does not");
        let file = dir(&tmp).join(&made.name);
        let mut text = String::new();
        GzDecoder::new(File::open(&file).unwrap()).read_to_string(&mut text).unwrap();
        for secret in ["secret-key", "jellyfin.internal", "secret-device", "secret-session-hash", "secret-service-key", "secret-service-user", "sonarr.internal", "secret-key-hash"] {
            assert!(!text.contains(secret), "{secret} must never be in a backup");
        }

        let target = Db::open(&tmp.join("target.db")).unwrap();
        let first = restore(&target, &file, true, None).unwrap();
        assert_eq!((first.plays_imported, first.plays_skipped, first.events, first.settings_restored), (2, 0, 2, true));
        let again = restore(&target, &file, true, None).unwrap();
        assert_eq!((again.plays_imported, again.plays_skipped, again.events), (0, 2, 0));

        let c = target.conn().unwrap();
        let n = |sql: &str| -> i64 { c.query_row(sql, [], |r| r.get(0)).unwrap() };
        assert_eq!(n("SELECT COUNT(*) FROM playbacks WHERE active = 1"), 0);
        // The timeline followed its play to the new id, and the home address made bob's play local.
        assert_eq!(n("SELECT COUNT(*) FROM playback_events e JOIN playbacks p ON p.id = e.playback_id WHERE p.user_id = 'u1'"), 2);
        assert_eq!(n("SELECT is_local FROM playbacks WHERE user_id = 'u2'"), 1);
        assert_eq!(n("SELECT COUNT(DISTINCT group_id) FROM playbacks WHERE group_id IS NOT NULL"), 1);
        assert_eq!(n("SELECT COUNT(*) FROM audit WHERE kind = 'setting_changed'"), 1, "restoring twice adds no audit rows");
        assert_eq!(n("SELECT COUNT(*) FROM api_keys"), 0, "a key never arrives with a backup");
        assert_eq!(n("SELECT COUNT(*) FROM user_permissions"), 1);
        assert_eq!(Settings::load(&c).unwrap().min_play_s, 42);

        // Without settings: history only.
        let plain = Db::open(&tmp.join("plain.db")).unwrap();
        let r = restore(&plain, &file, false, None).unwrap();
        assert!(!r.settings_restored);
        let c2 = plain.conn().unwrap();
        assert_eq!(c2.query_row("SELECT COUNT(*) FROM user_permissions", [], |r| r.get::<_, i64>(0)).unwrap(), 0);

        std::fs::write(tmp.join("not.jsonl"), "{\"hello\": 1}\n").unwrap();
        assert!(restore(&plain, &tmp.join("not.jsonl"), false, None).unwrap_err().to_string().contains("not a finstats backup"));
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn a_watchlist_travels_and_is_merged_keeping_the_older_date() {
        let tmp = std::env::temp_dir().join(format!("finstats-watchlist-backup-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let source = Db::open(&tmp.join("source.db")).unwrap();
        source.conn().unwrap().execute_batch(
            "INSERT INTO watchlist(id, user_id, kind, item_id, tmdb_id, title, year, added_at) VALUES (1, 'u1', 'Movie', 'i1', '10378', 'Big Buck Bunny', 2008, 50);
             INSERT INTO watchlist(id, user_id, kind, tvdb_id, title, added_at) VALUES (2, 'u1', 'Series', '380002', 'Nightfall Bay', 60);
             INSERT INTO watchlist(id, user_id, kind, imdb_id, title, added_at) VALUES (3, 'u2', 'Movie', 'tt1727587', 'Sintel', 70);",
        ).unwrap();
        let made = export(&source, &dir(&tmp), None).unwrap();
        let file = dir(&tmp).join(&made.name);

        let target = Db::open(&tmp.join("target.db")).unwrap();
        // Here already: the same film, put on the list later, by its id alone.
        target.conn().unwrap().execute_batch("INSERT INTO watchlist(id, user_id, kind, tmdb_id, title, added_at) VALUES (1, 'u1', 'Movie', '10378', 'Big Buck Bunny', 80);").unwrap();
        restore(&target, &file, false, None).unwrap();
        restore(&target, &file, false, None).unwrap();
        let c = target.conn().unwrap();
        let rows: Vec<(String, String, i64)> = c.prepare("SELECT user_id, title, added_at FROM watchlist ORDER BY user_id, added_at").unwrap()
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))).unwrap().map(Result::unwrap).collect();
        assert_eq!(rows, [("u1".into(), "Big Buck Bunny".into(), 50), ("u1".into(), "Nightfall Bay".into(), 60), ("u2".into(), "Sintel".into(), 70)],
            "one entry per title and person, with the older date; twice is once");
        let _ = std::fs::remove_dir_all(&tmp);
    }

    /// Where the owner located a title is theirs and not Jellyfin's to give back: it travels with a backup, and the plays
    /// it is about are attached by it as they are restored.
    #[test]
    fn where_a_title_was_located_travels_and_attaches_what_is_restored() {
        let tmp = std::env::temp_dir().join(format!("finstats-located-backup-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let source = Db::open(&tmp.join("source.db")).unwrap();
        source.conn().unwrap().execute_batch(
            "INSERT INTO located(from_id, to_id, at, by) VALUES ('plex:5', 'empire', 100, 'u1');
             INSERT INTO playbacks(source, source_id, user_id, user_name, item_id, item_name, item_type, started_at, ended_at, duration_s)
               VALUES ('tautulli', 'tautulli:1', 'u1', 'alice', 'plex:5', 'Star Wars: Episode V - The Empire Strikes Back (1980)', 'Movie', 1000, 2000, 1000);",
        ).unwrap();
        let made = export(&source, &dir(&tmp), None).unwrap();
        let target = Db::open(&tmp.join("target.db")).unwrap();
        target.conn().unwrap().execute_batch("INSERT INTO items(id, type, name, production_year, updated_at) VALUES ('empire', 'Movie', 'The Empire Strikes Back', 1980, 1)").unwrap();
        restore(&target, &dir(&tmp).join(&made.name), false, None).unwrap();
        let c = target.conn().unwrap();
        let to: String = c.query_row("SELECT to_id FROM located WHERE from_id = 'plex:5'", [], |r| r.get(0)).unwrap();
        let at: String = c.query_row("SELECT item_id FROM playbacks WHERE source_id = 'tautulli:1'", [], |r| r.get(0)).unwrap();
        assert_eq!((to.as_str(), at.as_str()), ("empire", "empire"));
        let _ = std::fs::remove_dir_all(&tmp);
    }

    /// Settings this version cannot read are not restored: reading them as "all defaults" would quietly
    /// reset every setting the install has.
    #[test]
    fn settings_that_cannot_be_read_leave_the_install_s_own_alone() {
        let tmp = std::env::temp_dir().join(format!("finstats-bad-settings-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let target = Db::open(&tmp.join("target.db")).unwrap();
        db::set_setting(&target.conn().unwrap(), "settings", &serde_json::to_string(&Settings { min_play_s: 42, ..Settings::default() }).unwrap()).unwrap();
        let file = tmp.join("backup.jsonl");
        let lines = [
            json!({ "finstats_backup": FORMAT, "app_version": "2.0.1" }),
            json!({ "t": "settings", "r": { "key": "settings", "value": r#"{"min_play_s":"soon"}"# } }),
        ];
        std::fs::write(&file, lines.iter().map(|l| l.to_string() + "\n").collect::<String>()).unwrap();
        let r = restore(&target, &file, true, None).unwrap();
        assert!(!r.settings_restored);
        assert_eq!(Settings::load(&target.conn().unwrap()).unwrap().min_play_s, 42);
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn a_backup_from_before_seek_origins_were_kept_gets_them_back_on_restore() {
        // A file written by a version before migration 21 carries a seek's origin only in its label.
        // Restoring is the one other way such a row arrives, so the restore does what the migration did.
        let tmp = std::env::temp_dir().join(format!("finstats-seek-origin-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let source = Db::open(&tmp.join("source.db")).unwrap();
        source
            .conn()
            .unwrap()
            .execute_batch(
                "INSERT INTO playbacks(id, source, user_id, user_name, item_id, item_name, item_type, started_at, ended_at, duration_s)
                   VALUES (7, 'live', 'u1', 'alice', 'i1', 'Big Buck Bunny', 'Movie', 1000, 1600, 600);
                 INSERT INTO playback_events(playback_id, at, kind, position_s, from_s, detail) VALUES
                   (7, 1000, 'start', 0, NULL, NULL), (7, 1100, 'seek', 900, NULL, '1:45 → 15:00'), (7, 1200, 'seek', 30, NULL, '16:40 → 0:30');",
            )
            .unwrap();
        let made = export(&source, &dir(&tmp), None).unwrap();
        let target = Db::open(&tmp.join("target.db")).unwrap();
        restore(&target, &dir(&tmp).join(&made.name), false, None).unwrap();
        let c = target.conn().unwrap();
        let from: Vec<Option<i64>> = c
            .prepare("SELECT from_s FROM playback_events ORDER BY at").unwrap()
            .query_map([], |r| r.get(0)).unwrap().collect::<Result<_, _>>().unwrap();
        assert_eq!(from, [None, Some(105), Some(1000)]);
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn a_play_another_tracker_already_holds_is_not_restored_again_but_a_restart_still_is() {
        let tmp = std::env::temp_dir().join(format!("finstats-restore-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let source = Db::open(&tmp.join("source.db")).unwrap();
        source
            .conn()
            .unwrap()
            .execute_batch(
                "INSERT INTO playbacks(source, user_id, user_name, item_id, item_name, item_type, started_at, ended_at, duration_s, device_id)
                   VALUES ('live', 'u1', 'alice', 'i1', 'Big Buck Bunny', 'Movie', 100000, 100600, 600, 'tv'),
                          ('live', 'u1', 'alice', 'i1', 'Big Buck Bunny', 'Movie', 100180, 100900, 600, 'phone');",
            )
            .unwrap();
        let made = export(&source, &dir(&tmp), None).unwrap();
        let file = dir(&tmp).join(&made.name);

        // Both are real: alice started it on the television and again on her phone three minutes
        // later. A restore must keep both.
        let empty = Db::open(&tmp.join("empty.db")).unwrap();
        assert_eq!(restore(&empty, &file, false, None).unwrap().plays_imported, 2);

        // But an install that imported the same evening from Streamystats already has it.
        let imported = Db::open(&tmp.join("imported.db")).unwrap();
        imported
            .conn()
            .unwrap()
            .execute_batch(
                "INSERT INTO playbacks(source, source_id, user_id, user_name, item_id, item_name, item_type, started_at, ended_at, duration_s)
                   VALUES ('streamystats', 'streamystats:1', 'u1', 'alice', 'i1', 'Big Buck Bunny', 'Movie', 100045, 100650, 600);",
            )
            .unwrap();
        let r = restore(&imported, &file, false, None).unwrap();
        assert_eq!((r.plays_imported, r.plays_skipped), (0, 2));
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
