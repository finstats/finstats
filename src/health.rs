//! Library health: what is wrong with a file only shows beside its neighbours — a season in another resolution, a hole
//! in a season, the same film twice, a file far too thin for what it claims, a dub that stops, a title Jellyfin never
//! identified. Every rule here is a pure function over item rows; nothing reads the database, so each can be tried on
//! invented data. finstats only reports: nothing here changes anything in Jellyfin.

use std::collections::BTreeMap;

use anyhow::Result;
use serde_json::{Value, json};

use crate::db::rusqlite::{Connection, OptionalExtension, params};
use sha2::{Digest, Sha256};

/// One live item, as much of it as the rules read.
#[derive(Debug, Clone, Default)]
pub struct Item {
    pub id: String,
    pub kind: String,
    pub name: String,
    pub library_id: Option<String>,
    pub series_id: Option<String>,
    pub season: Option<i64>,
    pub number: Option<i64>,
    pub number_end: Option<i64>,
    pub year: Option<i64>,
    pub path: Option<String>,
    pub size_bytes: Option<i64>,
    pub runtime_s: Option<i64>,
    pub bitrate: Option<i64>,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub video_codec: Option<String>,
    pub video_range: Option<String>,
    pub audio_languages: Vec<String>,
    /// Every id Jellyfin matched the title with, by provider ("Tmdb" → "10378").
    pub provider_ids: BTreeMap<String, String>,
}

impl Item {
    /// The profile's rule: an episode is on the server when it has a file.
    fn is_file(&self) -> bool {
        self.path.is_some() || self.size_bytes.is_some()
    }

    /// The profile's rule too: an episode without a season number is in season one.
    fn season_number(&self) -> i64 {
        self.season.unwrap_or(1)
    }
}

/// Something to look at, with the numbers that say so.
#[derive(Debug, Clone, PartialEq)]
pub struct Finding {
    /// Kind and subject ids: a rescan finds the same finding under the same key.
    pub key: String,
    pub kind: &'static str,
    /// Where a click leads.
    pub item_id: String,
    pub title: String,
    pub libraries: Vec<String>,
    pub evidence: Value,
    /// The values the rule read, hashed: a dismissal holds while they do.
    pub fingerprint: String,
    /// Space a second copy of the same thing takes; only copies have any.
    pub wasted_bytes: Option<i64>,
}

/// Every finding in these items, in a stable order.
pub fn findings(items: &[Item]) -> Vec<Finding> {
    let lib = Library::new(items);
    let mut out = gaps(&lib);
    out.extend(drift(&lib));
    out.extend(thin(items, &lib));
    out.extend(dubs(&lib));
    out.extend(unidentified(items));
    out.extend(copies(items, &lib));
    out.sort_by(|a, b| a.key.cmp(&b.key));
    out
}

/// The items, looked up the ways the rules need.
struct Library<'a> {
    by_id: BTreeMap<&'a str, &'a Item>,
    /// Episodes with a file, by show and season (season 0, the specials, left out).
    seasons: BTreeMap<(&'a str, i64), Vec<&'a Item>>,
}

impl<'a> Library<'a> {
    fn new(items: &'a [Item]) -> Self {
        let mut seasons: BTreeMap<(&str, i64), Vec<&Item>> = BTreeMap::new();
        for it in items.iter().filter(|i| i.kind == "Episode" && i.is_file()) {
            if let Some(series) = it.series_id.as_deref()
                && it.season_number() > 0
            {
                seasons.entry((series, it.season_number())).or_default().push(it);
            }
        }
        Library { by_id: items.iter().map(|i| (i.id.as_str(), i)).collect(), seasons }
    }

    /// Each show's seasons, in order.
    fn shows(&self) -> BTreeMap<&'a str, Vec<(i64, &Vec<&'a Item>)>> {
        let mut shows: BTreeMap<&str, Vec<(i64, &Vec<&Item>)>> = BTreeMap::new();
        for (&(series, season), files) in &self.seasons {
            shows.entry(series).or_default().push((season, files));
        }
        shows
    }

    /// What a show is called: its own row, else what its episodes say.
    fn title(&self, id: &str) -> String {
        self.by_id.get(id).map(|i| i.name.clone()).unwrap_or_else(|| id.to_string())
    }
}

/// A finding about these items, keyed `kind:subject`.
fn finding(kind: &'static str, subject: &str, item_id: &str, title: String, subjects: &[&Item], evidence: Value) -> Finding {
    let mut libraries: Vec<String> = subjects.iter().filter_map(|i| i.library_id.clone()).collect();
    libraries.sort();
    libraries.dedup();
    Finding {
        key: format!("{kind}:{subject}"),
        kind,
        item_id: item_id.to_string(),
        title,
        libraries,
        fingerprint: fingerprint(kind, subjects),
        evidence,
        wasted_bytes: None,
    }
}

/// Everything a rule could have read of these items. A replaced file, a re-encoded season or a new episode changes it.
fn fingerprint(kind: &str, subjects: &[&Item]) -> String {
    let mut ids: Vec<&&Item> = subjects.iter().collect();
    ids.sort_by(|a, b| a.id.cmp(&b.id));
    let mut h = Sha256::new();
    h.update(kind.as_bytes());
    for i in ids {
        let line = json!([i.id, i.kind, i.size_bytes, i.runtime_s, i.bitrate, i.width, i.height, i.video_codec, i.video_range,
            i.season, i.number, i.number_end, i.audio_languages, i.provider_ids, i.path.is_some()]);
        h.update(line.to_string().as_bytes());
        h.update(b"\n");
    }
    hex::encode(h.finalize())
}

// ---------------------------------------------------------------- stored

/// Every live film, show and episode, as the rules read them.
pub fn load(conn: &Connection) -> Result<Vec<Item>> {
    let mut stmt = conn.prepare(
        "SELECT id, type, name, library_id, series_id, parent_index_number, index_number, index_number_end, production_year, path, size_bytes,
                runtime_s, bitrate, width, height, video_codec, video_range, audio_languages, provider_ids
         FROM items WHERE removed = 0 AND type IN ('Movie', 'Series', 'Episode')",
    )?;
    let rows = stmt.query_map([], |r| {
        let languages: Option<String> = r.get(17)?;
        let ids: Option<String> = r.get(18)?;
        Ok(Item {
            id: r.get(0)?, kind: r.get(1)?, name: r.get(2)?, library_id: r.get(3)?, series_id: r.get(4)?, season: r.get(5)?, number: r.get(6)?,
            number_end: r.get(7)?, year: r.get(8)?, path: r.get(9)?, size_bytes: r.get(10)?, runtime_s: r.get(11)?, bitrate: r.get(12)?,
            width: r.get(13)?, height: r.get(14)?, video_codec: r.get(15)?, video_range: r.get(16)?,
            audio_languages: languages.and_then(|l| serde_json::from_str(&l).ok()).unwrap_or_default(),
            provider_ids: provider_ids(ids.as_deref()),
        })
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// The ids a title was matched with: text and not empty, as `pipeline::rebuild_external` reads them.
fn provider_ids(raw: Option<&str>) -> BTreeMap<String, String> {
    let Some(Value::Object(map)) = raw.and_then(|r| serde_json::from_str(r).ok()) else { return BTreeMap::new() };
    map.into_iter().filter_map(|(k, v)| Some((k, v.as_str().filter(|s| !s.is_empty())?.to_string()))).collect()
}

/// Works the findings out again and replaces the stored ones, keeping when each was first found. Answers how many there
/// are. Read first, written in one transaction: a page never waits on the rules, and never sees half a list.
pub fn recompute(conn: &mut Connection, now: i64) -> Result<usize> {
    let found = findings(&load(conn)?);
    let tx = conn.transaction()?;
    let first: std::collections::HashMap<String, i64> =
        tx.prepare("SELECT key, found_at FROM health_findings")?.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?.collect::<Result<_, _>>()?;
    tx.execute_batch("DELETE FROM health_findings; DELETE FROM health_libraries;")?;
    {
        let mut row = tx.prepare("INSERT INTO health_findings(key, kind, item_id, title, evidence, fingerprint, wasted_bytes, found_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)")?;
        let mut lib = tx.prepare("INSERT OR IGNORE INTO health_libraries(library_id, key) VALUES (?1, ?2)")?;
        for f in &found {
            let at = first.get(&f.key).copied().unwrap_or(now);
            row.execute(params![f.key, f.kind, f.item_id, f.title, f.evidence.to_string(), f.fingerprint, f.wasted_bytes, at])?;
            for l in &f.libraries {
                lib.execute(params![l, f.key])?;
            }
        }
    }
    crate::db::set_setting(&tx, "health_computed_at", &now.to_string())?;
    tx.commit()?;
    Ok(found.len())
}

/// After a look for metadata changes: the same recompute, once a library read has made the first. Until then the items
/// lack where a multi-episode file ends, and every such file would read as a hole.
pub fn after_changes(conn: &mut Connection, now: i64) -> Result<Option<usize>> {
    if crate::db::get_setting(conn, "health_computed_at")?.is_none() {
        return Ok(None);
    }
    recompute(conn, now).map(Some)
}

/// A finding `f` that is dismissed: the owner set it aside as it is now.
pub const DISMISSED_SQL: &str = "EXISTS (SELECT 1 FROM health_dismissed d WHERE d.key = f.key AND d.fingerprint = f.fingerprint)";

/// Sets a finding aside as it stands. `false` when there is no such finding.
pub fn dismiss(conn: &Connection, key: &str, note: Option<&str>, by: &str, now: i64) -> Result<bool> {
    let Some(fingerprint) = conn.query_row("SELECT fingerprint FROM health_findings WHERE key = ?1", [key], |r| r.get::<_, String>(0)).optional()? else { return Ok(false) };
    let note = note.map(str::trim).filter(|n| !n.is_empty());
    conn.execute(
        "INSERT INTO health_dismissed(key, fingerprint, note, at, by) VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(key) DO UPDATE SET fingerprint = excluded.fingerprint, note = excluded.note, at = excluded.at, by = excluded.by",
        params![key, fingerprint, note, now, by],
    )?;
    Ok(true)
}

/// Brings a dismissed finding back. `false` when it was not dismissed.
pub fn undismiss(conn: &Connection, key: &str) -> Result<bool> {
    Ok(conn.execute("DELETE FROM health_dismissed WHERE key = ?1", [key])? > 0)
}

// ---------------------------------------------------------------- gaps

/// Episode numbers missing between a season's lowest and highest file. What is missing after the last file cannot be
/// known from the library, so nothing is said about it.
fn gaps(lib: &Library) -> Vec<Finding> {
    let mut out = vec![];
    for (&(series, season), files) in &lib.seasons {
        let mut covered = std::collections::BTreeSet::new();
        for e in files {
            if let Some(n) = e.number {
                covered.extend(n..=e.number_end.unwrap_or(n).max(n));
            }
        }
        let (Some(&from), Some(&to)) = (covered.first(), covered.last()) else { continue };
        if to > EPISODE_NUMBER_MAX || to - from > GAP_SPAN_MAX {
            continue;
        }
        let mut missing: Vec<[i64; 2]> = vec![];
        for n in from..=to {
            if covered.contains(&n) {
                continue;
            }
            match missing.last_mut() {
                Some(run) if run[1] == n - 1 => run[1] = n,
                _ => missing.push([n, n]),
            }
        }
        if missing.is_empty() {
            continue;
        }
        let evidence = json!({ "season": season, "from": from, "to": to, "missing": missing, "files": files.len() });
        out.push(finding("gap", &format!("{series}:{season}"), series, lib.title(series), files, evidence));
    }
    out
}

// ---------------------------------------------------------------- quality drift

/// What a file looks like, in the terms drift compares: resolution class, range family, codec family.
fn look(e: &Item) -> [Option<String>; 3] {
    [class(e.width, e.height).map(|c| c.label().to_string()), range(e.video_range.as_deref()).map(str::to_string), codec(e.video_codec.as_deref())]
}
const LOOKS: [&str; 3] = ["resolution", "range", "codec"];

/// The value most of these files have, with how many have it. A tie goes to the value that sorts first, so it
/// never flips between two reads.
fn dominant<'v>(values: impl Iterator<Item = &'v Option<String>>) -> Option<(String, usize)> {
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for v in values.flatten() {
        *counts.entry(v.as_str()).or_default() += 1;
    }
    counts.into_iter().fold(None, |best: Option<(&str, usize)>, (v, n)| if best.is_none_or(|(_, b)| n > b) { Some((v, n)) } else { best }).map(|(v, n)| (v.to_string(), n))
}

/// A season whose usual look differs from another's (one finding per show, every season listed), and a file that
/// differs from its own season in resolution or range. A codec only counts between seasons: one H.264 episode among
/// HEVC ones plays the same, and mixes like that are everywhere.
fn drift(lib: &Library) -> Vec<Finding> {
    let mut out = vec![];
    for (series, seasons) in lib.shows() {
        let mut rows = vec![];
        let mut usual: Vec<[Option<String>; 3]> = vec![];
        for (season, files) in &seasons {
            let looks: Vec<[Option<String>; 3]> = files.iter().map(|e| look(e)).collect();
            let main: [Option<(String, usize)>; 3] = std::array::from_fn(|k| dominant(looks.iter().map(|l| &l[k])));
            usual.push(std::array::from_fn(|k| main[k].as_ref().map(|m| m.0.clone())));
            let said = |k: usize| main[k].as_ref().map(|m| m.0.as_str());
            rows.push(json!({ "season": season, "episodes": files.len(), "resolution": said(0), "range": said(1), "codec": said(2) }));
            // Resolution and range only, and only where the season has a clear look of its own.
            for (e, l) in files.iter().zip(&looks) {
                let odd: Vec<usize> = (0..2)
                    .filter(|&k| match (&l[k], &main[k]) {
                        (Some(v), Some((m, n))) => v != m && *n >= EPISODE_DRIFT_MIN_FILES && n * 100 >= looks.len() * EPISODE_DRIFT_SHARE_PCT,
                        _ => false,
                    })
                    .collect();
                if odd.is_empty() {
                    continue;
                }
                let mut ev = json!({ "season": season, "episode": e.number, "differs": odd.iter().map(|&k| LOOKS[k]).collect::<Vec<_>>() });
                for &k in &odd {
                    ev[LOOKS[k]] = json!(l[k]);
                    ev[format!("season_{}", LOOKS[k])] = json!(said(k));
                }
                out.push(finding("episode_drift", &e.id, &e.id, lib.title(series), &[e], ev));
            }
        }
        let differs: Vec<&str> = (0..3)
            .filter(|&k| {
                let mut seen = usual.iter().filter_map(|u| u[k].as_deref()).collect::<Vec<_>>();
                seen.sort();
                seen.dedup();
                seen.len() > 1
            })
            .map(|k| LOOKS[k])
            .collect();
        if !differs.is_empty() {
            let files: Vec<&Item> = seasons.iter().flat_map(|(_, f)| f.iter().copied()).collect();
            out.push(finding("season_drift", series, series, lib.title(series), &files, json!({ "differs": differs, "seasons": rows })));
        }
    }
    out
}

// ---------------------------------------------------------------- thin files

/// The lowest bitrate (bits per second, picture and sound together, as Jellyfin gives it) that is plausible for a
/// resolution and codec family; under it the file claims more than it can hold. Deliberately low: efficient encodes of
/// animation sit well under a typical film's rate, and on a real library of 13,000 1080p HEVC files 236 ran under
/// 1 Mbps but only 19 under 0.8. SD and codecs without a row are never judged: their rates vary too much to say.
const THIN: [(Class, &str, i64); 9] = [
    (Class::Uhd, "HEVC", 4_000_000),
    (Class::Uhd, "AV1", 4_000_000),
    (Class::Uhd, "H.264", 8_000_000),
    (Class::P1080, "HEVC", 800_000),
    (Class::P1080, "AV1", 800_000),
    (Class::P1080, "H.264", 1_500_000),
    (Class::P720, "HEVC", 400_000),
    (Class::P720, "AV1", 400_000),
    (Class::P720, "H.264", 700_000),
];

/// What a thin file is judged on: its class and codec, its bitrate and the floor for the two.
struct Thin {
    class: Class,
    codec: String,
    bps: i64,
    floor: i64,
}

impl Thin {
    fn json(&self) -> Value {
        json!({ "resolution": self.class.label(), "codec": self.codec, "bitrate_bps": self.bps, "threshold_bps": self.floor })
    }
}

/// How a file falls short, when it is under the floor for what it claims.
fn too_thin(e: &Item) -> Option<Thin> {
    let (size, runtime) = (e.size_bytes.filter(|&s| s > 0)?, e.runtime_s.filter(|&r| r > 0)?);
    let (class, codec) = (class(e.width, e.height)?, codec(e.video_codec.as_deref())?);
    let floor = THIN.iter().find(|(c, k, _)| *c == class && *k == codec)?.2;
    let bps = e.bitrate.filter(|&b| b > 0).unwrap_or(size.saturating_mul(8) / runtime);
    (bps < floor).then_some(Thin { class, codec, bps, floor })
}

/// A film far too thin for what it claims, and the thin episodes of a season as one finding — a season encoded the
/// same way is one thing to look at, not a dozen.
fn thin(items: &[Item], lib: &Library) -> Vec<Finding> {
    let mut out = vec![];
    for f in items.iter().filter(|i| i.kind == "Movie") {
        if let Some(t) = too_thin(f) {
            out.push(finding("thin", &f.id, &f.id, f.name.clone(), &[f], t.json()));
        }
    }
    for (&(series, season), files) in &lib.seasons {
        let thin: Vec<(&Item, Thin)> = files.iter().filter_map(|e| Some((*e, too_thin(e)?))).collect();
        if thin.is_empty() {
            continue;
        }
        let bps = thin.iter().map(|(_, t)| t.bps);
        let (lowest, highest) = (bps.clone().min(), bps.max());
        let episodes: Vec<Value> = thin
            .iter()
            .map(|(e, t)| {
                let mut ev = t.json();
                ev["id"] = json!(e.id);
                ev["episode"] = json!(e.number);
                ev
            })
            .collect();
        let ev = json!({ "season": season, "files": thin.len(), "of": files.len(), "lowest_bps": lowest, "highest_bps": highest, "episodes": episodes });
        let subjects: Vec<&Item> = thin.iter().map(|(e, _)| *e).collect();
        out.push(finding("thin", &format!("{series}:{season}"), series, lib.title(series), &subjects, ev));
    }
    out
}

// ---------------------------------------------------------------- dubs

/// An audio language that covers some seasons of a show completely and others not at all. The rule for a season is the
/// Languages card's (`media::language_counts`); `und` is a track nobody tagged, not a language that stops. A dub that
/// thins out without ever stopping is left alone: a partial season says less than a missing one.
fn dubs(lib: &Library) -> Vec<Finding> {
    let mut out = vec![];
    for (series, seasons) in lib.shows() {
        // language → season → episodes with it
        let mut by_language: BTreeMap<String, BTreeMap<i64, i64>> = BTreeMap::new();
        for (season, files) in &seasons {
            for (code, n) in crate::media::language_counts(files.iter().map(|e| e.audio_languages.as_slice())) {
                by_language.entry(code).or_default().insert(*season, n);
            }
        }
        for (code, counts) in by_language.into_iter().filter(|(c, _)| c != "und") {
            let (mut full, mut none, mut partial) = (vec![], vec![], vec![]);
            for (season, files) in &seasons {
                match counts.get(season).copied().unwrap_or(0) {
                    0 => none.push(*season),
                    n if n as usize == files.len() => full.push(*season),
                    n => partial.push(json!({ "season": season, "episodes": n, "of": files.len() })),
                }
            }
            if full.is_empty() || none.is_empty() {
                continue;
            }
            let files: Vec<&Item> = seasons.iter().flat_map(|(_, f)| f.iter().copied()).collect();
            let ev = json!({ "language": code, "full": full, "none": none, "partial": partial });
            out.push(finding("dub", &format!("{series}:{code}"), series, lib.title(series), &files, ev));
        }
    }
    out
}

// ---------------------------------------------------------------- never identified

/// Films and shows Jellyfin matched with nothing: no provider id at all. Pipeline, the watchlist and re-linking's
/// first rule know titles by those ids, so none of them can recognise these either. Not Unlinked media, which is
/// plays that match no title; this is titles that match no catalogue.
fn unidentified(items: &[Item]) -> Vec<Finding> {
    items
        .iter()
        .filter(|i| matches!(i.kind.as_str(), "Movie" | "Series") && i.provider_ids.is_empty())
        .map(|i| finding("unidentified", &i.id, &i.id, i.name.clone(), &[i], json!({ "type": i.kind, "year": i.year, "path": i.path })))
        .collect()
}

// ---------------------------------------------------------------- copies

/// The ids "copies are one title" goes by, as `item_external` keeps them.
const COPY_SOURCES: [&str; 3] = ["Tmdb", "Tvdb", "Imdb"];

/// Films and shows that are copies of one title, by the rule the watchlist and Pipeline already use: items of one type
/// that share a provider id, unless the ids among them lead to different titles (`watchlist::find_title`). Each
/// group's members are in id order, so the first is the one that stands for them. A version Jellyfin merged into one
/// item is one item, and never a copy.
pub fn copy_groups(items: &[Item]) -> Vec<Vec<&Item>> {
    let titles: Vec<&Item> = items.iter().filter(|i| matches!(i.kind.as_str(), "Movie" | "Series")).collect();
    let id_of = |t: &'_ Item, s: &str| t.provider_ids.get(s).filter(|v| !v.is_empty()).cloned();
    let mut index: std::collections::HashMap<(String, &str, String), Vec<usize>> = std::collections::HashMap::new();
    for (n, t) in titles.iter().enumerate() {
        for s in COPY_SOURCES {
            if let Some(v) = id_of(t, s) {
                index.entry((t.kind.clone(), s, v)).or_default().push(n);
            }
        }
    }
    let consistent = |members: &[usize]| COPY_SOURCES.iter().all(|s| {
        let mut said = members.iter().filter_map(|&m| id_of(titles[m], s));
        let first = said.next();
        said.all(|v| Some(v) == first)
    });
    let mut parent: Vec<usize> = (0..titles.len()).collect();
    fn root(parent: &mut [usize], mut n: usize) -> usize {
        while parent[n] != n {
            parent[n] = parent[parent[n]];
            n = parent[n];
        }
        n
    }
    for (n, t) in titles.iter().enumerate() {
        // What `find_title` would take for one title, asked with this item's ids.
        let mut one: Vec<usize> = COPY_SOURCES.iter().filter_map(|s| index.get(&(t.kind.clone(), *s, id_of(t, s)?))).flatten().copied().collect();
        one.sort_unstable();
        one.dedup();
        if one.len() < 2 || !consistent(&one) {
            continue;
        }
        for m in one {
            let (a, b) = (root(&mut parent, n), root(&mut parent, m));
            parent[a] = b;
        }
    }
    let mut groups: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for n in 0..titles.len() {
        let r = root(&mut parent, n);
        groups.entry(r).or_default().push(n);
    }
    let mut out: Vec<Vec<&Item>> = groups
        .into_values()
        .filter(|g| g.len() > 1 && consistent(g))
        .map(|g| {
            let mut members: Vec<&Item> = g.into_iter().map(|n| titles[n]).collect();
            members.sort_by(|a, b| a.id.cmp(&b.id));
            members
        })
        .collect();
    out.sort_by(|a, b| a[0].id.cmp(&b[0].id));
    out
}

/// What a set of files that are one thing amounts to: the bytes spent on a second copy in the same resolution class
/// (everything but the largest of each class), whether there is such a copy at all, and the classes present.
fn weigh(files: &[&Item]) -> (i64, bool, Vec<Option<Class>>) {
    let mut by_class: BTreeMap<Option<Class>, Vec<i64>> = BTreeMap::new();
    for f in files {
        by_class.entry(class(f.width, f.height)).or_default().push(f.size_bytes.unwrap_or(0));
    }
    let mut wasted = 0;
    let mut twice = false;
    for sizes in by_class.values().filter(|s| s.len() > 1) {
        twice = true;
        wasted += sizes.iter().sum::<i64>() - sizes.iter().max().copied().unwrap_or(0);
    }
    (wasted, twice, by_class.into_keys().rev().collect())
}

fn labels(classes: &[Option<Class>]) -> Vec<&'static str> {
    classes.iter().map(|c| c.map_or("unknown", Class::label)).collect()
}

fn file_json(f: &Item) -> Value {
    json!({ "id": f.id, "library_id": f.library_id, "resolution": class(f.width, f.height).map(Class::label), "codec": codec(f.video_codec.as_deref()),
            "size_bytes": f.size_bytes, "path": f.path, "season": f.season, "episode": f.number })
}

/// Copies of one title. The same thing twice (one resolution class) is space spent twice and counted; two versions
/// (a 4K library beside a 1080p one) are often deliberate, listed, and never counted as waste. A show's copies are its
/// episodes: through the show's own copies, or twice in one show, matched by season and episode number.
fn copies(items: &[Item], lib: &Library) -> Vec<Finding> {
    let mut out = vec![];
    let groups = copy_groups(items);
    let mut push = |twice: bool, wasted: i64, subject: &str, title: String, files: &[&Item], ev: Value| {
        let mut f = finding(if twice { "copies" } else { "versions" }, subject, subject, title, files, ev);
        if twice {
            f.wasted_bytes = Some(wasted);
        }
        out.push(f);
    };
    for g in groups.iter().filter(|g| g[0].kind == "Movie") {
        let (wasted, twice, classes) = weigh(g);
        let ev = json!({ "files": g.iter().map(|f| file_json(f)).collect::<Vec<_>>(), "resolutions": labels(&classes) });
        push(twice, wasted, &g[0].id, g[0].name.clone(), g, ev);
    }
    // Every show is its own group unless it has copies, so an episode twice in one show is found too.
    let mut show_of: BTreeMap<&str, &str> = BTreeMap::new();
    for g in groups.iter().filter(|g| g[0].kind == "Series") {
        for s in g {
            show_of.insert(s.id.as_str(), g[0].id.as_str());
        }
    }
    let mut slots: BTreeMap<&str, BTreeMap<(i64, i64), Vec<&Item>>> = BTreeMap::new();
    for e in items.iter().filter(|i| i.kind == "Episode" && i.is_file()) {
        let (Some(series), Some(n)) = (e.series_id.as_deref(), e.number) else { continue };
        let show = show_of.get(series).copied().unwrap_or(series);
        slots.entry(show).or_default().entry((e.season_number(), n)).or_default().push(e);
    }
    for (show, slots) in slots {
        let (mut twice_files, mut twice_slots, mut examples, mut wasted) = (vec![], 0, vec![], 0);
        let (mut version_files, mut version_slots, mut version_classes) = (vec![], 0, std::collections::BTreeSet::new());
        for ((season, episode), files) in slots.into_iter().filter(|(_, f)| f.len() > 1) {
            let (w, twice, classes) = weigh(&files);
            if twice {
                twice_slots += 1;
                wasted += w;
                if examples.len() < COPY_EXAMPLES {
                    examples.push(json!({ "season": season, "episode": episode, "files": files.iter().map(|f| file_json(f)).collect::<Vec<_>>() }));
                }
                twice_files.extend(files);
            } else {
                version_slots += 1;
                version_classes.extend(classes);
                version_files.extend(files);
            }
        }
        if twice_slots > 0 {
            push(true, wasted, show, lib.title(show), &twice_files, json!({ "episodes": twice_slots, "examples": examples }));
        }
        if version_slots > 0 {
            let classes: Vec<Option<Class>> = version_classes.into_iter().rev().collect();
            push(false, 0, show, lib.title(show), &version_files, json!({ "episodes": version_slots, "resolutions": labels(&classes) }));
        }
    }
    out
}

/// How sharp a picture a file claims, coarsely: finer steps (1440p, 576p) only split a season into noise.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Class {
    Sd,
    P720,
    P1080,
    Uhd,
}

// ---------------------------------------------------------------- thresholds
//
// Every number a finding rests on is here, with why it is that number.

/// Resolution bands by width — a cropped film keeps its width (1920×800 is 1080p) — or by height, for 4:3 and
/// pillarboxed pictures that keep theirs (1440×1080 is 1080p). Low enough to forgive a crop of a few pixels.
const UHD_WIDTH: i64 = 3200;
const UHD_HEIGHT: i64 = 2000;
const P1080_WIDTH: i64 = 1800;
const P1080_HEIGHT: i64 = 1000;
const P720_WIDTH: i64 = 1200;
const P720_HEIGHT: i64 = 700;

/// A season whose numbers span more than this is numbered by date (S2024E0315) or from the show's first episode, and
/// the numbers between two of its files are not missing files. The longest real seasons run to a few hundred.
const GAP_SPAN_MAX: i64 = 1000;
/// How many of a show's doubled episodes a finding lists by name; the count says the rest.
const COPY_EXAMPLES: usize = 20;
/// A file is odd one out only in a season of at least this many files with a look…
const EPISODE_DRIFT_MIN_FILES: usize = 4;
/// …that at least this share of them have (in percent): in a season split four to two, nobody is the odd one.
const EPISODE_DRIFT_SHARE_PCT: usize = 75;
/// …and a number past this is a date (20240315), not a count: nothing between two dates is missing.
const EPISODE_NUMBER_MAX: i64 = 5000;

impl Class {
    pub fn label(self) -> &'static str {
        match self {
            Class::Sd => "SD",
            Class::P720 => "720p",
            Class::P1080 => "1080p",
            Class::Uhd => "4K",
        }
    }
}

/// The resolution class of a picture this size; `None` without one.
pub fn class(width: Option<i64>, height: Option<i64>) -> Option<Class> {
    let (w, h) = (width.unwrap_or(0), height.unwrap_or(0));
    if w <= 0 && h <= 0 {
        return None;
    }
    Some(if w >= UHD_WIDTH || h >= UHD_HEIGHT {
        Class::Uhd
    } else if w >= P1080_WIDTH || h >= P1080_HEIGHT {
        Class::P1080
    } else if w >= P720_WIDTH || h >= P720_HEIGHT {
        Class::P720
    } else {
        Class::Sd
    })
}

/// A dynamic range by family: HDR10 and HDR10+ are one look on a screen, and a season in both is not drift.
pub fn range(raw: Option<&str>) -> Option<&'static str> {
    let r = raw?.trim().to_ascii_uppercase();
    if r.starts_with("DOVI") {
        Some("Dolby Vision")
    } else if r.starts_with("HDR") || r == "HLG" {
        Some("HDR")
    } else if r == "SDR" {
        Some("SDR")
    } else {
        None
    }
}

/// A video codec as people name it: Jellyfin says "h265" in one file and "hevc" in the next.
pub fn codec(raw: Option<&str>) -> Option<String> {
    let c = raw?.trim().to_ascii_uppercase();
    Some(match c.as_str() {
        "" => return None,
        "H265" | "HEVC" => "HEVC".into(),
        "H264" | "AVC" => "H.264".into(),
        _ => c,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(super) fn episode(id: &str, series: &str, season: i64, number: i64) -> Item {
        Item {
            id: id.into(), kind: "Episode".into(), name: id.into(), library_id: Some("shows".into()), series_id: Some(series.into()), season: Some(season),
            number: Some(number), path: Some(format!("/media/shows/{series}/{id}.mkv")), size_bytes: Some(1_000_000_000), runtime_s: Some(1500),
            width: Some(1920), height: Some(1080), video_codec: Some("hevc".into()), video_range: Some("SDR".into()), ..Default::default()
        }
    }

    pub(super) fn show(id: &str, name: &str) -> Item {
        Item { id: id.into(), kind: "Series".into(), name: name.into(), library_id: Some("shows".into()), ..Default::default() }
    }

    fn of<'a>(found: &'a [Finding], kind: &str) -> Vec<&'a Finding> {
        found.iter().filter(|f| f.kind == kind).collect()
    }

    #[test]
    fn a_hole_in_a_season_is_the_numbers_missing_between_its_files() {
        let mut items = vec![show("lo", "Low Orbit")];
        items.extend([1, 2, 3, 6, 7, 9].map(|n| episode(&format!("e{n}"), "lo", 1, n)));
        let found = findings(&items);
        let gaps = of(&found, "gap");
        assert_eq!(gaps.len(), 1);
        let g = gaps[0];
        assert_eq!((g.key.as_str(), g.item_id.as_str(), g.title.as_str()), ("gap:lo:1", "lo", "Low Orbit"));
        assert_eq!(g.evidence["season"], 1);
        assert_eq!(g.evidence["missing"], serde_json::json!([[4, 5], [8, 8]]));
        assert_eq!((g.evidence["from"].as_i64(), g.evidence["to"].as_i64()), (Some(1), Some(9)));
        assert_eq!(g.libraries, ["shows"]);
    }

    #[test]
    fn a_file_of_two_episodes_leaves_no_gap() {
        let mut items = vec![show("lo", "Low Orbit"), episode("e1", "lo", 1, 1), episode("e3", "lo", 1, 3)];
        items[1].number_end = Some(2);
        assert!(of(&findings(&items), "gap").is_empty(), "S01E01-E02 holds episode 2");
    }

    #[test]
    fn specials_unnumbered_episodes_virtual_items_and_the_end_of_a_season_say_nothing() {
        let mut items = vec![show("lo", "Low Orbit"), episode("e1", "lo", 1, 1), episode("e2", "lo", 1, 2)];
        items.push(episode("sp1", "lo", 0, 1));
        items.push(episode("sp9", "lo", 0, 9));
        let mut unnumbered = episode("x", "lo", 1, 0);
        unnumbered.number = None;
        items.push(unnumbered);
        let mut virtual_one = episode("e9", "lo", 1, 9);
        virtual_one.path = None;
        virtual_one.size_bytes = None;
        items.push(virtual_one);
        assert!(of(&findings(&items), "gap").is_empty(), "{:?}", of(&findings(&items), "gap"));
    }

    #[test]
    fn a_season_numbered_by_date_is_not_a_season_of_holes() {
        let items = vec![show("ln", "Late Night"), episode("d1", "ln", 2024, 20240102), episode("d2", "ln", 2024, 20240315)];
        assert!(of(&findings(&items), "gap").is_empty());
    }

    #[test]
    fn an_episode_without_a_season_number_is_in_season_one_as_the_profile_counts_it() {
        let mut e3 = episode("e3", "lo", 1, 3);
        e3.season = None;
        let items = vec![show("lo", "Low Orbit"), episode("e1", "lo", 1, 1), e3];
        assert_eq!(of(&findings(&items), "gap")[0].evidence["missing"], serde_json::json!([[2, 2]]));
    }

    /// A season of `n` episodes numbered from 1, every one shaped by `f`.
    fn season(series: &str, season: i64, n: i64, f: impl Fn(&mut Item)) -> Vec<Item> {
        (1..=n).map(|e| {
            let mut it = episode(&format!("{series}-s{season}e{e}"), series, season, e);
            f(&mut it);
            it
        }).collect()
    }

    fn sd(it: &mut Item) {
        (it.width, it.height) = (Some(720), Some(576));
    }

    #[test]
    fn a_season_in_another_resolution_is_one_finding_for_the_show() {
        let mut items = vec![show("lo", "Low Orbit")];
        items.extend(season("lo", 1, 4, |e| (e.width, e.height) = (Some(1280), Some(720))));
        items.extend(season("lo", 2, 4, |_| {}));
        items.extend(season("lo", 3, 4, |_| {}));
        let found = findings(&items);
        let drift = of(&found, "season_drift");
        assert_eq!(drift.len(), 1);
        assert_eq!((drift[0].key.as_str(), drift[0].item_id.as_str()), ("season_drift:lo", "lo"));
        assert_eq!(drift[0].evidence["differs"], serde_json::json!(["resolution"]));
        let seasons = drift[0].evidence["seasons"].as_array().unwrap();
        assert_eq!(seasons.len(), 3, "every season, so the odd one is plain");
        assert_eq!((seasons[0]["season"].as_i64(), seasons[0]["resolution"].as_str(), seasons[0]["episodes"].as_i64()), (Some(1), Some("720p"), Some(4)));
        assert_eq!(seasons[1]["resolution"], "1080p");
        assert!(of(&found, "episode_drift").is_empty(), "every season is even within itself");
    }

    #[test]
    fn a_change_of_codec_or_range_counts_between_seasons_not_between_episodes() {
        let mut items = vec![show("lo", "Low Orbit")];
        items.extend(season("lo", 1, 4, |e| e.video_codec = Some("h264".into())));
        items.extend(season("lo", 2, 4, |e| e.video_range = Some("HDR10".into())));
        let found = findings(&items);
        assert_eq!(of(&found, "season_drift")[0].evidence["differs"], serde_json::json!(["range", "codec"]));

        let mut mixed = vec![show("bb", "Big Buck Bunny Shorts")];
        mixed.extend(season("bb", 1, 6, |e| if e.number == Some(2) { e.video_codec = Some("avc".into()) }));
        mixed.extend(season("bb", 2, 6, |e| e.video_range = Some(if e.number == Some(1) { "HDR10Plus" } else { "HDR10" }.into())));
        let found = findings(&mixed);
        assert!(of(&found, "episode_drift").is_empty() && of(&found, "season_drift").len() == 1, "one H.264 episode is harmless; HDR10+ is HDR: {found:?}");
        assert_eq!(of(&found, "season_drift")[0].evidence["differs"], serde_json::json!(["range"]));
    }

    #[test]
    fn one_file_unlike_its_season_is_a_finding_of_its_own() {
        let mut items = vec![show("lo", "Low Orbit")];
        items.extend(season("lo", 1, 8, |e| if e.number == Some(5) { sd(e) }));
        let found = findings(&items);
        let odd = of(&found, "episode_drift");
        assert_eq!(odd.len(), 1);
        assert_eq!((odd[0].key.as_str(), odd[0].item_id.as_str(), odd[0].title.as_str()), ("episode_drift:lo-s1e5", "lo-s1e5", "Low Orbit"));
        assert_eq!((odd[0].evidence["resolution"].as_str(), odd[0].evidence["season_resolution"].as_str()), (Some("SD"), Some("1080p")));
        assert_eq!((odd[0].evidence["season"].as_i64(), odd[0].evidence["episode"].as_i64()), (Some(1), Some(5)));
        assert!(of(&found, "season_drift").is_empty(), "one season has nothing to differ from");
    }

    #[test]
    fn a_season_with_no_clear_look_singles_nobody_out() {
        let mut items = vec![show("lo", "Low Orbit")];
        items.extend(season("lo", 1, 6, |e| if e.number.unwrap() <= 2 { sd(e) }));
        items.extend(season("lo", 2, 3, |e| if e.number == Some(1) { sd(e) }));
        assert!(of(&findings(&items), "episode_drift").is_empty(), "four of six is no rule, and three files are too few to have one");
    }

    pub(super) fn film(id: &str, name: &str) -> Item {
        Item {
            id: id.into(), kind: "Movie".into(), name: name.into(), library_id: Some("films".into()), year: Some(2008), path: Some(format!("/media/films/{id}.mkv")),
            size_bytes: Some(4_000_000_000), runtime_s: Some(5400), width: Some(1920), height: Some(1080), video_codec: Some("h264".into()),
            video_range: Some("SDR".into()), provider_ids: [("Tmdb".to_string(), format!("{id}-tmdb"))].into(), ..Default::default()
        }
    }

    fn at_bitrate(mut it: Item, mbps: f64, w: i64, h: i64, codec: &str) -> Item {
        it.bitrate = Some((mbps * 1_000_000.0) as i64);
        (it.width, it.height, it.video_codec) = (Some(w), Some(h), Some(codec.into()));
        it
    }

    #[test]
    fn a_file_far_too_thin_for_its_resolution_is_named_with_its_numbers() {
        let items = vec![
            at_bitrate(film("bbb", "Big Buck Bunny"), 1.1, 3840, 2160, "hevc"),
            at_bitrate(film("sin", "Sintel"), 1.4, 1920, 800, "h264"),
            at_bitrate(film("tos", "Tears of Steel"), 1.6, 1920, 800, "h264"),
            at_bitrate(film("ele", "Elephants Dream"), 0.9, 1920, 1080, "hevc"),
            at_bitrate(film("cos", "Cosmos Laundromat"), 0.7, 1920, 1080, "av1"),
            at_bitrate(film("old", "Caminandes"), 0.3, 720, 576, "h264"),
            at_bitrate(film("mp2", "Glass Half"), 0.5, 1920, 1080, "mpeg2video"),
        ];
        let found = findings(&items);
        let thin: Vec<&str> = of(&found, "thin").iter().map(|f| f.item_id.as_str()).collect();
        assert_eq!(thin, ["bbb", "cos", "sin"], "4K HEVC at 1.1, AV1 1080p at 0.7, H.264 1080p at 1.4 — not SD, not a codec without a rule");
        let bbb = of(&found, "thin")[0];
        assert_eq!(bbb.key, "thin:bbb");
        assert_eq!((bbb.evidence["resolution"].as_str(), bbb.evidence["codec"].as_str()), (Some("4K"), Some("HEVC")));
        assert_eq!((bbb.evidence["bitrate_bps"].as_i64(), bbb.evidence["threshold_bps"].as_i64()), (Some(1_100_000), Some(4_000_000)));
    }

    #[test]
    fn a_file_without_a_bitrate_is_measured_by_its_size_and_one_without_size_or_runtime_is_not_judged() {
        let mut measured = film("bbb", "Big Buck Bunny");
        (measured.size_bytes, measured.runtime_s) = (Some(600_000_000), Some(5400)); // 0.89 Mbps
        let mut no_runtime = at_bitrate(film("sin", "Sintel"), 0.5, 1920, 1080, "h264");
        no_runtime.runtime_s = None;
        let mut no_size = at_bitrate(film("tos", "Tears of Steel"), 0.5, 1920, 1080, "h264");
        no_size.size_bytes = None;
        let found = findings(&[measured, no_runtime, no_size]);
        let thin = of(&found, "thin");
        assert_eq!(thin.len(), 1);
        assert_eq!((thin[0].item_id.as_str(), thin[0].evidence["bitrate_bps"].as_i64()), ("bbb", Some(888_888)));
    }

    #[test]
    fn thin_episodes_are_one_finding_per_season() {
        let mut items = vec![show("lo", "Low Orbit")];
        items.extend(season("lo", 1, 6, |e| {
            e.bitrate = Some(if e.number.unwrap() <= 4 { 600_000 + e.number.unwrap() * 10_000 } else { 2_000_000 });
        }));
        items.extend(season("lo", 2, 3, |e| e.bitrate = Some(2_000_000)));
        let found = findings(&items);
        let thin = of(&found, "thin");
        assert_eq!(thin.len(), 1);
        let t = thin[0];
        assert_eq!((t.key.as_str(), t.item_id.as_str(), t.title.as_str()), ("thin:lo:1", "lo", "Low Orbit"));
        assert_eq!((t.evidence["season"].as_i64(), t.evidence["files"].as_i64(), t.evidence["of"].as_i64()), (Some(1), Some(4), Some(6)));
        assert_eq!((t.evidence["lowest_bps"].as_i64(), t.evidence["highest_bps"].as_i64()), (Some(610_000), Some(640_000)));
        assert_eq!(t.evidence["episodes"].as_array().unwrap().len(), 4);
    }

    fn speaking<'a>(langs: &'a [&'a str]) -> impl Fn(&mut Item) + 'a {
        move |e: &mut Item| e.audio_languages = langs.iter().map(|l| l.to_string()).collect()
    }

    #[test]
    fn a_dub_that_covers_some_seasons_and_not_others_is_named_by_language() {
        let mut items = vec![show("lo", "Low Orbit")];
        items.extend(season("lo", 1, 4, speaking(&["jpn", "eng", "und"])));
        items.extend(season("lo", 2, 4, speaking(&["jpn", "eng", "und"])));
        items.extend(season("lo", 3, 4, speaking(&["jpn"])));
        items.extend(season("lo", 4, 4, |e| e.audio_languages = if e.number.unwrap() <= 2 { vec!["jpn".into(), "eng".into()] } else { vec!["jpn".into()] }));
        let found = findings(&items);
        let dub = of(&found, "dub");
        assert_eq!(dub.len(), 1, "jpn is everywhere and und is no language: {dub:?}");
        assert_eq!((dub[0].key.as_str(), dub[0].item_id.as_str()), ("dub:lo:eng", "lo"));
        assert_eq!(dub[0].evidence, serde_json::json!({ "language": "eng", "full": [1, 2], "none": [3], "partial": [{ "season": 4, "episodes": 2, "of": 4 }] }));
    }

    #[test]
    fn a_dub_that_thins_out_but_never_stops_says_nothing() {
        let mut items = vec![show("lo", "Low Orbit")];
        items.extend(season("lo", 1, 4, speaking(&["jpn", "eng"])));
        items.extend(season("lo", 2, 4, |e| e.audio_languages = if e.number == Some(1) { vec!["jpn".into(), "eng".into()] } else { vec!["jpn".into()] }));
        let mut special = episode("sp", "lo", 0, 1);
        special.audio_languages = vec!["jpn".into()];
        items.push(special);
        assert!(of(&findings(&items), "dub").is_empty(), "no season is without it, and the specials are not a season");
    }

    #[test]
    fn a_film_or_show_jellyfin_never_identified_has_no_provider_id_at_all() {
        let mut home = film("hv", "Home Video 2019");
        home.provider_ids.clear();
        let mut anime = show("an", "Big Buck Bunny: The Series");
        anime.provider_ids = [("AniDB".to_string(), "17".to_string())].into();
        let mut unknown_show = show("us", "Sintel Shorts");
        unknown_show.provider_ids.clear();
        let mut episode_alone = episode("e1", "us", 1, 1);
        episode_alone.provider_ids.clear();
        let found = findings(&[home, film("bbb", "Big Buck Bunny"), anime, unknown_show, episode_alone]);
        let keys: Vec<&str> = of(&found, "unidentified").iter().map(|f| f.key.as_str()).collect();
        assert_eq!(keys, ["unidentified:hv", "unidentified:us"], "an AniDB id is an identification; an episode is its show's business");
        assert_eq!(of(&found, "unidentified")[0].evidence["type"], "Movie");
    }

    fn ids(mut it: Item, pairs: &[(&str, &str)]) -> Item {
        it.provider_ids = pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
        it
    }

    fn in_library(mut it: Item, library: &str) -> Item {
        it.library_id = Some(library.into());
        it
    }

    #[test]
    fn the_same_film_twice_in_one_resolution_is_space_spent_twice() {
        let mut small = in_library(ids(film("bbb-b", "Big Buck Bunny"), &[("Tmdb", "10378")]), "films-2");
        small.size_bytes = Some(3_000_000_000);
        let items = [ids(film("bbb-a", "Big Buck Bunny"), &[("Tmdb", "10378"), ("Imdb", "tt1254207")]), small, film("sin", "Sintel")];
        let found = findings(&items);
        let copies = of(&found, "copies");
        assert_eq!(copies.len(), 1);
        let c = copies[0];
        assert_eq!((c.key.as_str(), c.item_id.as_str(), c.title.as_str()), ("copies:bbb-a", "bbb-a", "Big Buck Bunny"), "the lowest id stands for them, as everywhere");
        assert_eq!(c.wasted_bytes, Some(3_000_000_000), "all but the largest");
        assert_eq!(c.libraries, ["films", "films-2"]);
        assert_eq!(c.evidence["files"].as_array().unwrap().len(), 2);
        assert!(of(&found, "versions").is_empty());
    }

    #[test]
    fn a_4k_copy_beside_a_1080p_one_is_two_versions_and_wastes_nothing() {
        let uhd = in_library(at_bitrate(ids(film("bbb-4k", "Big Buck Bunny"), &[("Tmdb", "10378")]), 20.0, 3840, 2160, "hevc"), "films-4k");
        let items = [ids(film("bbb", "Big Buck Bunny"), &[("Tmdb", "10378")]), uhd];
        let found = findings(&items);
        assert!(of(&found, "copies").is_empty());
        let v = of(&found, "versions");
        assert_eq!((v.len(), v[0].key.as_str(), v[0].wasted_bytes), (1, "versions:bbb", None));
        assert_eq!(v[0].evidence["resolutions"], serde_json::json!(["4K", "1080p"]));
    }

    #[test]
    fn ids_that_lead_to_different_titles_are_no_copies() {
        let items = [ids(film("a", "Sintel"), &[("Tmdb", "45745"), ("Imdb", "tt1727587")]), ids(film("b", "Sintel"), &[("Tmdb", "99999"), ("Imdb", "tt1727587")])];
        assert!(findings(&items).iter().all(|f| f.kind != "copies" && f.kind != "versions"), "two TMDB ids: nothing is sure");
        let film_and_show = [ids(film("f", "Low Orbit"), &[("Imdb", "tt0000001")]), ids(show("s", "Low Orbit"), &[("Imdb", "tt0000001")])];
        assert!(findings(&film_and_show).iter().all(|f| f.kind != "copies" && f.kind != "versions"), "a film is never a copy of a show");
    }

    #[test]
    fn episodes_are_copies_through_their_shows_ids_and_their_numbers() {
        let mut items = vec![ids(show("lo-a", "Low Orbit"), &[("Tvdb", "70001")]), in_library(ids(show("lo-b", "Low Orbit"), &[("Tvdb", "70001")]), "shows-2")];
        items.extend(season("lo-a", 1, 3, |_| {}));
        items.extend(season("lo-b", 1, 2, |e| e.library_id = Some("shows-2".into())));
        // …and an episode in the first show's own folder twice, Jellyfin not having merged the two
        let mut twice = episode("lo-a-again", "lo-a", 1, 3);
        twice.size_bytes = Some(400_000_000);
        items.push(twice);
        let found = findings(&items);
        let c = of(&found, "copies");
        assert_eq!(c.len(), 1, "{found:?}");
        assert_eq!((c[0].key.as_str(), c[0].item_id.as_str()), ("copies:lo-a", "lo-a"));
        assert_eq!(c[0].evidence["episodes"], 3, "S01E01 and E02 in both shows, E03 twice in one");
        assert_eq!(c[0].wasted_bytes, Some(2 * 1_000_000_000 + 400_000_000));
        assert_eq!(c[0].libraries, ["shows", "shows-2"]);
    }

    #[test]
    fn a_show_kept_in_4k_beside_its_1080p_self_is_two_versions() {
        let mut items = vec![ids(show("lo", "Low Orbit"), &[("Tvdb", "70001")]), in_library(ids(show("lo-4k", "Low Orbit"), &[("Tvdb", "70001")]), "shows-4k")];
        items.extend(season("lo", 1, 4, |_| {}));
        items.extend(season("lo-4k", 1, 4, |e| (e.width, e.height, e.library_id) = (Some(3840), Some(2160), Some("shows-4k".into()))));
        let found = findings(&items);
        assert!(of(&found, "copies").is_empty());
        let v = of(&found, "versions");
        assert_eq!((v.len(), v[0].key.as_str(), v[0].evidence["episodes"].as_i64()), (1, "versions:lo", Some(4)));
    }

    /// Library health never defines "the same title" a third way: whatever `watchlist::find_title` takes for one title is
    /// one title here, and ids it refuses are no copies here either.
    #[test]
    fn copies_are_exactly_the_titles_find_title_treats_as_one() {
        let c = crate::db::rusqlite::Connection::open_in_memory().unwrap();
        for m in crate::db::MIGRATIONS {
            c.execute_batch(m).unwrap();
        }
        let items = vec![
            ids(film("m1", "Big Buck Bunny"), &[("Tmdb", "10378"), ("Imdb", "tt1254207")]),
            ids(film("m2", "Big Buck Bunny"), &[("Tmdb", "10378")]),
            ids(film("m3", "Big Buck Bunny"), &[("Imdb", "tt1254207")]),
            ids(film("m4", "Sintel"), &[("Tmdb", "45745"), ("Imdb", "tt1727587")]),
            ids(film("m5", "Sintel"), &[("Tmdb", "99999"), ("Imdb", "tt1727587")]),
            ids(film("m6", "Tears of Steel"), &[("Tmdb", "133701")]),
            ids(show("s1", "Low Orbit"), &[("Tvdb", "70001"), ("Tmdb", "500")]),
            ids(show("s2", "Low Orbit"), &[("Tvdb", "70001")]),
        ];
        for i in &items {
            c.execute(
                "INSERT INTO items(id, type, name, provider_ids, removed, updated_at) VALUES (?1, ?2, ?3, ?4, 0, 0)",
                crate::db::rusqlite::params![i.id, i.kind, i.name, serde_json::to_string(&i.provider_ids).unwrap()],
            )
            .unwrap();
        }
        crate::pipeline::rebuild_external(&c).unwrap();
        let groups = copy_groups(&items);
        let group_of = |id: &str| groups.iter().find(|g| g.iter().any(|i| i.id == id)).map(|g| { let mut v: Vec<&str> = g.iter().map(|i| i.id.as_str()).collect(); v.sort(); v });
        for i in &items {
            let get = |k: &str| i.provider_ids.get(k).cloned();
            let t = crate::watchlist::Title { kind: i.kind.clone(), tmdb_id: get("Tmdb"), tvdb_id: get("Tvdb"), imdb_id: get("Imdb"), title: i.name.clone(), year: None };
            let one: Vec<String> = c
                .prepare("SELECT DISTINCT x.item_id FROM item_external x JOIN items i ON i.id = x.item_id AND i.type = ?1 WHERE (x.source = 'Tmdb' AND x.value = ?2) OR (x.source = 'Tvdb' AND x.value = ?3) OR (x.source = 'Imdb' AND x.value = ?4) ORDER BY 1")
                .unwrap()
                .query_map(crate::db::rusqlite::params![t.kind, t.tmdb_id, t.tvdb_id, t.imdb_id], |r| r.get(0))
                .unwrap()
                .map(Result::unwrap)
                .collect();
            match crate::watchlist::find_title(&c, &t).unwrap() {
                Some(chosen) if one.len() > 1 => {
                    let g = group_of(&i.id).unwrap_or_else(|| panic!("find_title takes {one:?} for one title, health found no copies of {}", i.id));
                    assert!(one.iter().all(|x| g.contains(&x.as_str())) && g.contains(&chosen.as_str()), "{one:?} are one title to find_title, {g:?} here");
                }
                Some(_) => assert!(group_of(&i.id).is_none(), "{} has no copy to find_title", i.id),
                None => {}
            }
        }
        assert_eq!(group_of("m4"), None, "two TMDB ids for one IMDb id: nothing is sure");
        assert_eq!(group_of("s2"), Some(vec!["s1", "s2"]));
    }

    #[test]
    fn a_rescan_finds_the_same_findings_and_a_replaced_file_changes_only_its_own() {
        let mut items = vec![show("lo", "Low Orbit"), at_bitrate(film("bbb", "Big Buck Bunny"), 1.1, 3840, 2160, "hevc")];
        items.extend([1, 2, 5].map(|n| episode(&format!("e{n}"), "lo", 1, n)));
        let first = findings(&items);
        assert_eq!(first, findings(&items), "the same library reads the same");
        let mut more = items.clone();
        more.push(film("sin", "Sintel"));
        let kept = |found: &[Finding], key: &str| found.iter().find(|f| f.key == key).cloned();
        assert_eq!(kept(&findings(&more), "gap:lo:1"), kept(&first, "gap:lo:1"), "a title added elsewhere changes nothing here");
        let mut replaced = items.clone();
        replaced[1].size_bytes = Some(9_000_000_000);
        let again = findings(&replaced);
        assert_ne!(kept(&again, "thin:bbb").unwrap().fingerprint, kept(&first, "thin:bbb").unwrap().fingerprint, "a new file is a new look");
        assert_eq!(kept(&again, "gap:lo:1"), kept(&first, "gap:lo:1"));
    }

    // ------------------------------------------------------------ stored

    fn migrated() -> crate::db::rusqlite::Connection {
        let c = crate::db::rusqlite::Connection::open_in_memory().unwrap();
        for m in crate::db::MIGRATIONS {
            c.execute_batch(m).unwrap();
        }
        c
    }

    /// Two copies of Big Buck Bunny in two libraries, and a season of Low Orbit with episode 2 missing.
    fn stocked() -> crate::db::rusqlite::Connection {
        let c = migrated();
        c.execute_batch(
            r#"INSERT INTO items(id, library_id, type, name, production_year, provider_ids, path, size_bytes, runtime_s, width, height, video_codec, removed, updated_at) VALUES
                 ('bbb-a', 'films',   'Movie', 'Big Buck Bunny', 2008, '{"Tmdb":"10378"}', '/films/a.mkv', 4000000000, 5400, 1920, 1080, 'h264', 0, 0),
                 ('bbb-b', 'films-2', 'Movie', 'Big Buck Bunny', 2008, '{"Tmdb":"10378","Tvdb":""}', '/films2/b.mkv', 3000000000, 5400, 1920, 1080, 'h264', 0, 0),
                 ('gone',  'films',   'Movie', 'Sintel', 2010, NULL, '/films/s.mkv', 1, 60, 1920, 1080, 'h264', 1, 0),
                 ('lo',    'shows',   'Series', 'Low Orbit', 2020, '{"Tvdb":"70001"}', NULL, NULL, NULL, NULL, NULL, NULL, 0, 0);
               INSERT INTO items(id, library_id, type, name, series_id, parent_index_number, index_number, path, size_bytes, runtime_s, width, height, video_codec, removed, updated_at) VALUES
                 ('lo1', 'shows', 'Episode', 'One',   'lo', 1, 1, '/shows/1.mkv', 500000000, 1500, 1920, 1080, 'hevc', 0, 0),
                 ('lo3', 'shows', 'Episode', 'Three', 'lo', 1, 3, '/shows/3.mkv', 500000000, 1500, 1920, 1080, 'hevc', 0, 0);"#,
        )
        .unwrap();
        c
    }

    fn stored(c: &crate::db::rusqlite::Connection) -> Vec<(String, i64)> {
        c.prepare("SELECT key, found_at FROM health_findings ORDER BY key").unwrap().query_map([], |r| Ok((r.get(0)?, r.get(1)?))).unwrap().map(Result::unwrap).collect()
    }

    #[test]
    fn a_recompute_stores_what_it_finds_keeps_when_each_was_first_found_and_forgets_what_is_gone() {
        let mut c = stocked();
        assert_eq!(recompute(&mut c, 100).unwrap(), 2);
        assert_eq!(stored(&c), [("copies:bbb-a".to_string(), 100), ("gap:lo:1".to_string(), 100)], "a removed title is no finding");
        let libs: Vec<String> = c.prepare("SELECT library_id FROM health_libraries WHERE key = 'copies:bbb-a' ORDER BY 1").unwrap().query_map([], |r| r.get(0)).unwrap().map(Result::unwrap).collect();
        assert_eq!(libs, ["films", "films-2"]);
        let wasted: Option<i64> = c.query_row("SELECT wasted_bytes FROM health_findings WHERE key = 'copies:bbb-a'", [], |r| r.get(0)).unwrap();
        assert_eq!(wasted, Some(3_000_000_000));

        c.execute("INSERT INTO items(id, library_id, type, name, series_id, parent_index_number, index_number, path, size_bytes, updated_at) VALUES ('lo2', 'shows', 'Episode', 'Two', 'lo', 1, 2, '/shows/2.mkv', 1, 0)", []).unwrap();
        recompute(&mut c, 200).unwrap();
        assert_eq!(stored(&c), [("copies:bbb-a".to_string(), 100)], "the hole is filled; the copies were found at the first look");
        let left: i64 = c.query_row("SELECT COUNT(*) FROM health_libraries WHERE key = 'gap:lo:1'", [], |r| r.get(0)).unwrap();
        assert_eq!(left, 0);
    }

    /// `item_external` is rebuilt when the library is read, but health does not lean on it: on a server with no Sonarr,
    /// Radarr or Seerr, and nothing in that table at all, copies are still copies.
    #[test]
    fn copies_are_found_on_a_server_with_nothing_connected() {
        let mut c = stocked();
        c.execute("DELETE FROM item_external", []).unwrap();
        let services: i64 = c.query_row("SELECT COUNT(*) FROM services", [], |r| r.get(0)).unwrap();
        assert_eq!(services, 0);
        recompute(&mut c, 100).unwrap();
        assert!(stored(&c).iter().any(|(k, _)| k == "copies:bbb-a"));
    }

    fn dismissed(c: &crate::db::rusqlite::Connection) -> Vec<String> {
        c.prepare(&format!("SELECT f.key FROM health_findings f WHERE {DISMISSED_SQL} ORDER BY 1")).unwrap().query_map([], |r| r.get(0)).unwrap().map(Result::unwrap).collect()
    }

    #[test]
    fn a_dismissal_outlives_a_rescan_and_lifts_when_the_file_changes() {
        let mut c = stocked();
        recompute(&mut c, 100).unwrap();
        assert!(dismiss(&c, "copies:bbb-a", Some("the 4K one is coming"), "alice", 110).unwrap());
        assert!(!dismiss(&c, "copies:nothing", None, "alice", 110).unwrap(), "only a finding there is can be dismissed");
        assert_eq!(dismissed(&c), ["copies:bbb-a"]);
        recompute(&mut c, 200).unwrap();
        assert_eq!(dismissed(&c), ["copies:bbb-a"], "the same copies, still dismissed");
        let note: Option<String> = c.query_row("SELECT note FROM health_dismissed WHERE key = 'copies:bbb-a'", [], |r| r.get(0)).unwrap();
        assert_eq!(note.as_deref(), Some("the 4K one is coming"));

        c.execute("UPDATE items SET size_bytes = 3100000000 WHERE id = 'bbb-b'", []).unwrap();
        recompute(&mut c, 300).unwrap();
        assert!(dismissed(&c).is_empty(), "a replaced file is looked at again");
        assert!(stored(&c).iter().any(|(k, _)| k == "copies:bbb-a"), "…because it is still a copy");

        assert!(dismiss(&c, "copies:bbb-a", None, "alice", 310).unwrap());
        assert!(undismiss(&c, "copies:bbb-a").unwrap());
        assert!(!undismiss(&c, "copies:bbb-a").unwrap());
        assert!(dismissed(&c).is_empty());
    }

    #[test]
    fn a_look_for_metadata_changes_waits_for_the_first_library_read() {
        let mut c = stocked();
        assert_eq!(after_changes(&mut c, 100).unwrap(), None, "until a library read has brought where multi-episode files end");
        assert!(stored(&c).is_empty());
        recompute(&mut c, 200).unwrap();
        assert_eq!(after_changes(&mut c, 300).unwrap(), Some(2));
    }

    /// A library of 500 shows of 50 episodes and 5,000 films, a few of everything wrong in it.
    pub(super) fn generated() -> crate::db::rusqlite::Connection {
        let mut c = migrated();
        let tx = c.transaction().unwrap();
        {
            let mut ins = tx
                .prepare(
                    "INSERT INTO items(id, library_id, type, name, series_id, parent_index_number, index_number, production_year, path, size_bytes, runtime_s,
                                       bitrate, width, height, video_codec, video_range, audio_languages, provider_ids, removed, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 2020, ?8, ?9, ?10, ?11, ?12, ?13, ?14, 'SDR', ?15, ?16, 0, 0)",
                )
                .unwrap();
            for f in 0..5000i64 {
                let tmdb = if f % 97 == 0 { f + 1 } else { f }; // every 97th film is a copy of the next
                let ids = if f % 211 == 0 { "{}".to_string() } else { format!(r#"{{"Tmdb":"{tmdb}"}}"#) };
                let (w, h, br) = if f % 50 == 0 { (3840, 2160, 2_000_000) } else { (1920, 1080, 6_000_000) };
                ins.execute(params![format!("film{f:05}"), "films", "Movie", format!("Film {f}"), None::<String>, None::<i64>, None::<i64>, format!("/films/{f}.mkv"),
                    4_000_000_000i64, 6000, br, w, h, "hevc", r#"["eng"]"#, ids]).unwrap();
            }
            for s in 0..500i64 {
                let series = format!("show{s:04}");
                ins.execute(params![series, "shows", "Series", format!("Show {s}"), None::<String>, None::<i64>, None::<i64>, None::<String>, None::<i64>, None::<i64>,
                    None::<i64>, None::<i64>, None::<i64>, None::<String>, None::<String>, format!(r#"{{"Tvdb":"{s}"}}"#)]).unwrap();
                for season in 1..=5i64 {
                    for n in 1..=10i64 {
                        if s % 13 == 0 && season == 2 && n == 4 {
                            continue; // a hole
                        }
                        let (w, h) = if s % 17 == 0 && season == 1 { (1280, 720) } else { (1920, 1080) };
                        let langs = if s % 19 == 0 && season == 5 { r#"["jpn"]"# } else { r#"["jpn","eng"]"# };
                        ins.execute(params![format!("{series}-{season}-{n}"), "shows", "Episode", format!("Episode {n}"), series, season, n, format!("/shows/{s}/{season}/{n}.mkv"),
                            600_000_000i64, 1500, 3_000_000, w, h, "hevc", langs, None::<String>]).unwrap();
                    }
                }
            }
        }
        tx.commit().unwrap();
        c
    }

    /// `cargo test --release health::tests::at_scale -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn at_scale() {
        let mut c = generated();
        let started = std::time::Instant::now();
        let n = recompute(&mut c, 100).unwrap();
        let first = started.elapsed();
        let started = std::time::Instant::now();
        recompute(&mut c, 200).unwrap();
        println!("recompute over 25,000 episodes and 5,000 films: {n} findings, {first:?} first, {:?} again", started.elapsed());
        assert!(first < std::time::Duration::from_secs(2));
    }

    #[test]
    fn a_picture_is_classed_by_its_width_or_its_height() {
        assert_eq!(class(Some(1920), Some(800)), Some(Class::P1080), "a scope film is cropped, not smaller");
        assert_eq!(class(Some(1440), Some(1080)), Some(Class::P1080), "4:3 HD is pillarboxed, not 720p");
        assert_eq!(class(Some(3840), Some(1600)), Some(Class::Uhd));
        assert_eq!(class(Some(3200), Some(1300)), Some(Class::Uhd), "the 4K band starts at 3200");
        assert_eq!(class(Some(3199), Some(1300)), Some(Class::P1080));
        assert_eq!(class(Some(1800), Some(750)), Some(Class::P1080), "the 1080p band starts at 1800");
        assert_eq!(class(Some(1799), Some(750)), Some(Class::P720));
        assert_eq!(class(Some(1200), Some(500)), Some(Class::P720), "the 720p band starts at 1200");
        assert_eq!(class(Some(1199), Some(500)), Some(Class::Sd));
        assert_eq!(class(Some(960), Some(720)), Some(Class::P720), "4:3 720p");
        assert_eq!(class(Some(720), Some(576)), Some(Class::Sd));
        assert_eq!(class(None, None), None);
        assert_eq!(class(Some(0), Some(0)), None);
    }

    #[test]
    fn ranges_and_codecs_are_compared_by_family() {
        assert_eq!(range(Some("HDR10")), range(Some("HDR10Plus")));
        assert_eq!(range(Some("HLG")), Some("HDR"));
        assert_eq!(range(Some("DOVIWithHDR10")), Some("Dolby Vision"));
        assert_eq!(range(Some("SDR")), Some("SDR"));
        assert_eq!(range(None), None);
        assert_eq!(codec(Some("h265")).as_deref(), Some("HEVC"));
        assert_eq!(codec(Some("HEVC")).as_deref(), Some("HEVC"));
        assert_eq!(codec(Some("avc")).as_deref(), Some("H.264"));
        assert_eq!(codec(Some("mpeg2video")).as_deref(), Some("MPEG2VIDEO"));
        assert_eq!(codec(Some(" ")), None);
        assert_eq!(range(Some("Unknown")), None, "a range nobody named is no evidence");
    }
}
