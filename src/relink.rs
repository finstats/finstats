//! Re-attaching history to items that changed identity.
//!
//! Jellyfin derives an item's id from its path, so renaming a file or folder makes a "new"
//! item and leaves every earlier play pointing at an id that no longer exists. Those plays
//! also keep whatever name the item had at the time, often the raw folder name
//! (`Title (2010) [tmdbid-37799] [imdbid-tt1285016]`) from before metadata was fetched.
//!
//! After each library sync, orphaned plays are matched to the item that is there now:
//! by provider id when the old name carries one, otherwise by cleaned name (and year).
//! A match must be unambiguous; titles that were simply deleted stay as they are.

use anyhow::Result;

use std::collections::HashMap;

use crate::db::rusqlite::{Connection, OptionalExtension, params};

/// What a raw Jellyfin folder/file name tells us once the decorations are peeled off.
#[derive(Debug, Default, PartialEq)]
pub struct ParsedName {
    /// Without `[…id-…]` tags, but still with a trailing `(year)`.
    pub without_tags: String,
    /// Without tags and without the trailing `(year)`.
    pub title: String,
    pub year: Option<i64>,
    /// (json key in `provider_ids`, value), e.g. ("Tmdb", "37799").
    pub provider_ids: Vec<(&'static str, String)>,
}

pub fn parse_name(raw: &str) -> ParsedName {
    let mut out = ParsedName::default();
    let mut rest = String::new();
    let mut tail = raw;
    // Pull out every [key-value] / [key=value] tag Jellyfin understands.
    while let Some(open) = tail.find('[') {
        let Some(close) = tail[open..].find(']').map(|i| open + i) else { break };
        let inner = &tail[open + 1..close];
        let tag = inner.split_once(['-', '=']).and_then(|(k, v)| {
            let key = match k.trim().to_ascii_lowercase().as_str() {
                "tmdbid" | "tmdb" => "Tmdb",
                "imdbid" | "imdb" => "Imdb",
                "tvdbid" | "tvdb" => "Tvdb",
                _ => return None,
            };
            let v = v.trim();
            (!v.is_empty()).then(|| (key, v.to_string()))
        });
        rest.push_str(&tail[..open]);
        match tag {
            Some(t) => out.provider_ids.push(t),
            None => rest.push_str(&tail[open..=close]), // some other bracket; part of the title
        }
        tail = &tail[close + 1..];
    }
    rest.push_str(tail);
    out.without_tags = rest.split_whitespace().collect::<Vec<_>>().join(" ");

    out.title = out.without_tags.clone();
    if let Some(open) = out.without_tags.rfind('(') {
        let inside = out.without_tags[open + 1..].trim_end().trim_end_matches(')');
        if out.without_tags.trim_end().ends_with(')') && inside.len() == 4 {
            if let Ok(y) = inside.parse::<i64>() {
                if (1870..=2200).contains(&y) {
                    out.year = Some(y);
                    out.title = out.without_tags[..open].trim_end().to_string();
                }
            }
        }
    }
    out
}

/// A name as two catalogues are compared: without provider tags or a year written into it, folded like search folds
/// it (case, accents, punctuation: a hyphen and a dash are one), and without a leading article.
pub fn title_key(name: &str) -> String {
    let folded = crate::fuzzy::normalize(&parse_name(name).title);
    ["the ", "a ", "an "].iter().find_map(|a| folded.strip_prefix(a)).map(str::to_string).unwrap_or(folded)
}

/// Of the titles a name can mean, the one: the only one within a year of `year` (catalogues disagree by a year about
/// when a film came out) or, of several, the only one of that very year. Without a year, the only one there is.
fn pick(mut hits: Vec<(String, Option<i64>)>, year: Option<i64>) -> Option<String> {
    hits.sort();
    hits.dedup();
    let near: Vec<&(String, Option<i64>)> = hits.iter().filter(|(_, y)| match (year, y) { (Some(a), Some(b)) => (a - b).abs() <= 1, _ => true }).collect();
    match near.as_slice() {
        [(id, _)] => Some(id.clone()),
        [] => None,
        _ => match near.iter().filter(|(_, y)| year.is_some() && *y == year).collect::<Vec<_>>().as_slice() {
            [(id, _)] => Some(id.clone()),
            _ => None,
        },
    }
}

/// Every live title of a type by `title_key`, of its name and of its original title, read once per re-link and only
/// when a name as written found nothing.
#[derive(Default)]
struct Keys(HashMap<String, ByKey>);

/// Titles (id, year) by `title_key`.
type ByKey = HashMap<String, Vec<(String, Option<i64>)>>;

impl Keys {
    fn of(&mut self, conn: &Connection, item_type: &str) -> Result<&ByKey> {
        if !self.0.contains_key(item_type) {
            let mut map = ByKey::new();
            let rows: Vec<(String, String, Option<String>, Option<i64>)> = conn
                .prepare_cached("SELECT id, name, original_title, production_year FROM items WHERE removed = 0 AND type = ?1")?
                .query_map([item_type], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?
                .collect::<Result<_, _>>()?;
            for (id, name, original, year) in rows {
                for key in std::iter::once(title_key(&name)).chain(original.as_deref().map(title_key)) {
                    if !key.is_empty() {
                        map.entry(key).or_default().push((id.clone(), year));
                    }
                }
            }
            self.0.insert(item_type.to_string(), map);
        }
        Ok(&self.0[item_type])
    }
}

/// The one live item of `item_type` this name can only mean, if there is exactly one: by provider id when the name
/// carries one, then by the name as written (Jellyfin's or the original-language one it keeps beside it), and last by
/// the name cleaned of what two catalogues write differently.
fn find_current(conn: &Connection, keys: &mut Keys, item_type: &str, raw_name: &str, known_year: Option<i64>) -> Result<Option<String>> {
    let parsed = parse_name(raw_name);
    for (key, value) in &parsed.provider_ids {
        let hits: Vec<String> = conn
            .prepare_cached("SELECT id FROM items WHERE removed = 0 AND type = ?1 AND json_extract(provider_ids, '$.' || ?2) = ?3 LIMIT 2")?
            .query_map(params![item_type, key, value], |r| r.get(0))?
            .collect::<Result<_, _>>()?;
        if hits.len() == 1 {
            return Ok(hits.into_iter().next());
        }
    }
    // A year, when we have one, must agree within one: remakes share titles.
    let year = parsed.year.or(known_year);
    for name in [&parsed.title, &parsed.without_tags] {
        if name.is_empty() {
            continue;
        }
        let hits: Vec<(String, Option<i64>)> = conn
            .prepare_cached("SELECT id, production_year FROM items WHERE removed = 0 AND type = ?1 AND (name = ?2 COLLATE NOCASE OR original_title = ?2 COLLATE NOCASE)")?
            .query_map(params![item_type, name], |r| Ok((r.get(0)?, r.get(1)?)))?
            .collect::<Result<_, _>>()?;
        if let Some(id) = pick(hits, year) {
            return Ok(Some(id));
        }
    }
    let key = title_key(&parsed.title);
    if key.is_empty() {
        return Ok(None);
    }
    Ok(pick(keys.of(conn, item_type)?.get(&key).cloned().unwrap_or_default(), year))
}

#[derive(Debug, Default)]
pub struct Relinked {
    pub titles: usize,
    pub episodes: usize,
    pub names_cleaned: usize,
    /// Plays that re-linking turned into duplicates of plays already here, and so were taken out.
    pub duplicates_removed: usize,
    /// The titles plays were moved onto, each once. Moved plays keep their times, so nothing that looks for recent
    /// plays finds them: whoever asked for the re-link regroups these.
    pub moved_to: Vec<String>,
}

/// Every title plays point at that the library no longer has, stepping from one title to the next through the title
/// index (`idx_pb_item`): one lookup per title rather than a read of every play, because this runs at every start and
/// after every library read and nearly always finds nothing.
pub(crate) const ORPHANS_SQL: &str = "WITH RECURSIVE t(id) AS (
        SELECT (SELECT MIN(item_id) FROM playbacks)
        UNION ALL
        SELECT (SELECT MIN(item_id) FROM playbacks WHERE item_id > t.id) FROM t WHERE t.id IS NOT NULL)
    SELECT id FROM t WHERE id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM items live WHERE live.id = t.id AND live.removed = 0)";

/// `merge_window_s` is asked for rather than read here so that a caller cannot forget it: the one
/// thing that rewrites `item_id` is the one thing that can turn an imported play into a duplicate
/// of a play already here, and the rule has to be re-applied where the ids move.
pub fn relink_orphans(conn: &Connection, merge_window_s: i64) -> Result<Relinked> {
    let mut done = Relinked::default();
    let orphaned: Vec<String> = conn.prepare(ORPHANS_SQL)?.query_map([], |r| r.get(0))?.collect::<Result<_, _>>()?;
    if orphaned.is_empty() {
        return Ok(done);
    }
    // Only the plays of those titles are read from here on, through the title index.
    let orphaned = serde_json::to_string(&orphaned)?;
    let mut keys = Keys::default();
    const ORPHAN: &str = "p.item_id IN (SELECT value FROM json_each(?1))";

    // ---- films and other stand-alone titles
    let orphans: Vec<(String, String, String, Option<i64>)> = conn
        .prepare(&format!(
            "SELECT p.item_id, p.item_type, COALESCE(MAX(old.name), MAX(p.item_name)), MAX(old.production_year)
             FROM playbacks p LEFT JOIN items old ON old.id = p.item_id
             WHERE p.item_type IN ('Movie', 'Video', 'MusicVideo', 'Audio') AND {ORPHAN} GROUP BY p.item_id"
        ))?
        .query_map([&orphaned], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?
        .collect::<Result<_, _>>()?;
    for (old_id, item_type, name, year) in orphans {
        // The owner said where this one is (`locate`): that, before any rule.
        if let Some(to) = crate::locate::located(conn, &old_id)? {
            let (n, dropped) = crate::locate::move_onto(conn, merge_window_s, &old_id, &to)?;
            (done.titles, done.duplicates_removed) = (done.titles + n, done.duplicates_removed + dropped);
            done.moved_to.push(to);
            continue;
        }
        if let Some(new_id) = find_current(conn, &mut keys, &item_type, &name, year)? {
            let (n, dropped) = move_plays(conn, merge_window_s, &new_id, || {
                Ok(conn.execute(
                    "UPDATE playbacks SET item_id = ?1,
                            item_name  = (SELECT name FROM items WHERE id = ?1),
                            library_id = (SELECT library_id FROM items WHERE id = ?1),
                            runtime_s  = COALESCE((SELECT runtime_s FROM items WHERE id = ?1), runtime_s)
                     WHERE item_id = ?2",
                    params![new_id, old_id],
                )?)
            })?;
            done.titles += n;
            done.duplicates_removed += dropped;
            done.moved_to.push(new_id);
        } else {
            // Gone for good, but it can at least be called by its name.
            let cleaned = parse_name(&name).without_tags;
            if !cleaned.is_empty() && cleaned != name {
                done.names_cleaned += conn.execute("UPDATE playbacks SET item_name = ?1 WHERE item_id = ?2 AND item_name = ?3", params![cleaned, old_id, name])?;
            }
        }
    }

    // ---- episodes: find the series as it is now, then the same season/episode number in it
    let orphans: Vec<(String, Option<String>, Option<String>, Option<i64>, Option<i64>, Option<i64>)> = conn
        .prepare(&format!(
            "SELECT p.item_id, MAX(p.series_id), COALESCE(MAX(olds.name), MAX(p.series_name)), MAX(olds.production_year),
                    MAX(COALESCE(p.season_number, olde.parent_index_number)), MAX(COALESCE(p.episode_number, olde.index_number))
             FROM playbacks p LEFT JOIN items olde ON olde.id = p.item_id LEFT JOIN items olds ON olds.id = p.series_id
             WHERE p.item_type = 'Episode' AND {ORPHAN} GROUP BY p.item_id"
        ))?
        .query_map([&orphaned], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?)))?
        .collect::<Result<_, _>>()?;
    for (old_id, series_id, series_name, year, season, episode) in orphans {
        if let Some(to) = crate::locate::located(conn, &old_id)? {
            let (n, dropped) = crate::locate::move_onto(conn, merge_window_s, &old_id, &to)?;
            (done.episodes, done.duplicates_removed) = (done.episodes + n, done.duplicates_removed + dropped);
            done.moved_to.push(to);
            continue;
        }
        let (Some(season), Some(episode)) = (season, episode) else { continue };
        let series_alive: Option<String> = match &series_id {
            Some(id) => conn.query_row("SELECT id FROM items WHERE id = ?1 AND removed = 0", [id], |r| r.get(0)).optional()?,
            None => None,
        };
        let series_now = match (series_alive, series_name) {
            (Some(id), _) => Some(id),
            (None, Some(name)) => find_current(conn, &mut keys, "Series", &name, year)?,
            _ => None,
        };
        let Some(series_now) = series_now else { continue };
        let hits: Vec<String> = conn
            .prepare_cached("SELECT id FROM items WHERE removed = 0 AND type = 'Episode' AND series_id = ?1 AND parent_index_number = ?2 AND index_number = ?3 LIMIT 2")?
            .query_map(params![series_now, season, episode], |r| r.get(0))?
            .collect::<Result<_, _>>()?;
        if let [new_id] = hits.as_slice() {
            let (n, dropped) = move_plays(conn, merge_window_s, new_id, || {
                Ok(conn.execute(
                    "UPDATE playbacks SET item_id = ?1, series_id = ?2,
                            season_id   = (SELECT season_id FROM items WHERE id = ?1),
                            item_name   = (SELECT name FROM items WHERE id = ?1),
                            series_name = (SELECT name FROM items WHERE id = ?2),
                            library_id  = (SELECT library_id FROM items WHERE id = ?1),
                            season_number = ?3, episode_number = ?4,
                            runtime_s   = COALESCE((SELECT runtime_s FROM items WHERE id = ?1), runtime_s)
                     WHERE item_id = ?5",
                    params![new_id, series_now, season, episode, old_id],
                )?)
            })?;
            done.episodes += n;
            done.duplicates_removed += dropped;
            done.moved_to.push(new_id.clone());
        }
    }

    done.moved_to.sort();
    done.moved_to.dedup();
    if done.titles + done.episodes + done.names_cleaned > 0 {
        tracing::info!("re-linked {} title plays and {} episode plays to renamed items; cleaned {} names", done.titles, done.episodes, done.names_cleaned);
    }
    if done.duplicates_removed > 0 {
        tracing::info!("removed {} imported plays that re-linking had turned into duplicates of plays already here", done.duplicates_removed);
    }
    Ok(done)
}

/// Point one title's orphaned plays at `to`, and re-apply the rule for a play already here to `to`'s plays: the
/// check that would have caught a duplicate ran before the id moved, and only where plays landed can one have
/// appeared. One write, so a start killed between the two cannot leave a duplicate behind that no later pass would
/// look for, since once moved, nothing is orphaned any more. A savepoint, because an import or a restore calls this inside
/// its own transaction. Answers (plays moved, duplicates removed).
pub(crate) fn move_plays(conn: &Connection, merge_window_s: i64, to: &str, update: impl FnOnce() -> Result<usize>) -> Result<(usize, usize)> {
    conn.execute_batch("SAVEPOINT relink")?;
    let done = update().and_then(|moved| Ok((moved, crate::playback::drop_relinked_duplicates(conn, merge_window_s, to)?)));
    match done {
        Ok(v) => {
            conn.execute_batch("RELEASE relink")?;
            Ok(v)
        }
        Err(e) => {
            let _ = conn.execute_batch("ROLLBACK TO relink; RELEASE relink");
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn peels_provider_tags_and_year() {
        let p = parse_name("Big Buck Bunny (2008) [tmdbid-10378] [imdbid-tt1254207]");
        assert_eq!(p.without_tags, "Big Buck Bunny (2008)");
        assert_eq!(p.title, "Big Buck Bunny");
        assert_eq!(p.year, Some(2008));
        assert_eq!(p.provider_ids, vec![("Tmdb", "10378".to_string()), ("Imdb", "tt1254207".to_string())]);
    }

    #[test]
    fn leaves_ordinary_names_alone() {
        let p = parse_name("Blade Runner 2049");
        assert_eq!((p.title.as_str(), p.year, p.provider_ids.len()), ("Blade Runner 2049", None, 0));
        // Brackets that are not provider tags belong to the title; "(1)" is not a year.
        let p = parse_name("Some Show [Director's Cut] (1)");
        assert_eq!(p.title, "Some Show [Director's Cut] (1)");
        assert_eq!(parse_name("Show [tvdbid=12345]").provider_ids, vec![("Tvdb", "12345".to_string())]);
    }

    #[test]
    fn relinks_by_provider_id_then_name_and_never_guesses() {
        let conn = migrated();
        conn.execute_batch(
            "INSERT INTO items(id, type, name, production_year, provider_ids, library_id, updated_at) VALUES
                ('new1', 'Movie', 'Big Buck Bunny', 2008, '{\"Tmdb\":\"10378\"}', 'lib', 1),
                ('remakeA', 'Movie', 'Twins', 1988, NULL, 'lib', 1), ('remakeB', 'Movie', 'Twins', 2024, NULL, 'lib', 1),
                ('s-new', 'Series', 'Test Show', 2020, NULL, 'lib', 1);
             INSERT INTO items(id, type, name, series_id, parent_index_number, index_number, library_id, updated_at) VALUES ('e-new', 'Episode', 'Pilot', 's-new', 1, 1, 'lib', 1);
             INSERT INTO playbacks(id, source, user_id, user_name, item_id, item_name, item_type, started_at, ended_at, duration_s) VALUES
                (1, 'live', 'u', 'u', 'old1', 'Big Buck Bunny (2008) [tmdbid-10378] [imdbid-tt1254207]', 'Movie', 0, 10, 10),
                (2, 'live', 'u', 'u', 'old2', 'Twins', 'Movie', 100, 110, 10),
                (3, 'live', 'u', 'u', 'old3', 'Deleted Film (1999) [tmdbid-1]', 'Movie', 200, 210, 10);
             INSERT INTO playbacks(id, source, user_id, user_name, item_id, item_name, item_type, series_id, series_name, season_number, episode_number, started_at, ended_at, duration_s)
                VALUES (4, 'live', 'u', 'u', 'e-old', 'Episode 1', 'Episode', 's-old', 'Test Show', 1, 1, 300, 310, 10);",
        )
        .unwrap();
        // Every row here is one finstats recorded itself, so none of them can be a duplicate of
        // another tracker's, so re-linking them takes nothing away.
        let r = relink_orphans(&conn, 600).unwrap();
        assert_eq!((r.titles, r.episodes, r.names_cleaned, r.duplicates_removed), (1, 1, 1, 0));
        let get = |old: &str| -> (String, String) { conn.query_row("SELECT item_id, item_name FROM playbacks WHERE id = ?1", [old], |r| Ok((r.get(0)?, r.get(1)?))).unwrap() };
        assert_eq!(get("1"), ("new1".into(), "Big Buck Bunny".into()));
        assert_eq!(get("2").0, "old2", "two films called Twins and no year: ambiguous, leave it");
        assert_eq!(get("3"), ("old3".into(), "Deleted Film (1999)".into()));
        assert_eq!(get("4"), ("e-new".into(), "Pilot".into()));
    }

    fn migrated() -> Connection {
        let c = Connection::open_in_memory().unwrap();
        for m in crate::db::MIGRATIONS {
            c.execute_batch(m).unwrap();
        }
        c
    }

    /// Where the plays of `old` point after a re-link.
    fn now_at(c: &Connection, old_name: &str) -> String {
        c.query_row("SELECT item_id FROM playbacks WHERE item_name = ?1 OR series_name = ?1", [old_name], |r| r.get(0)).unwrap()
    }

    fn orphan_film(c: &Connection, name: &str) {
        c.execute("INSERT INTO playbacks(source, user_id, user_name, item_id, item_name, item_type, started_at, ended_at, duration_s) VALUES ('jellystat', 'u', 'u', ?1, ?2, 'Movie', 0, 10, 10)",
            params![format!("gone-{name}"), name]).unwrap();
    }

    fn orphan_episode(c: &Connection, show: &str, season: i64, episode: i64) {
        c.execute("INSERT INTO playbacks(source, user_id, user_name, item_id, item_name, item_type, series_id, series_name, season_number, episode_number, started_at, ended_at, duration_s)
                   VALUES ('jellystat', 'u', 'u', ?1, 'An episode', 'Episode', ?2, ?3, ?4, ?5, 0, 10, 10)",
            params![format!("gone-{show}-{season}-{episode}"), format!("gone-{show}"), show, season, episode]).unwrap();
    }

    /// The same title written another way: other punctuation ("-" for "–"), other case, accents, or a year Jellyfin put
    /// into the name itself, as in "JoJo's Bizarre Adventure (2012)".
    /// Plays in the trash move with their title, so an undo puts them back on the item that is really there: a title
    /// whose only play is in the trash is orphaned all the same, and moved like any other.
    #[test]
    fn plays_in_the_trash_move_with_their_title() {
        let conn = migrated();
        conn.execute_batch(
            "INSERT INTO items(id, type, name, production_year, provider_ids, library_id, updated_at) VALUES ('new1', 'Movie', 'Big Buck Bunny', 2008, '{\"Tmdb\":\"10378\"}', 'lib', 1);
             INSERT INTO playbacks(id, source, user_id, user_name, item_id, item_name, item_type, started_at, ended_at, duration_s, deleted_at) VALUES
                (1, 'live', 'u', 'u', 'old1', 'Big Buck Bunny (2008) [tmdbid-10378]', 'Movie', 0, 10, 10, 50),
                (2, 'live', 'u', 'u', 'old1', 'Big Buck Bunny (2008) [tmdbid-10378]', 'Movie', 100, 110, 10, NULL);",
        )
        .unwrap();
        relink_orphans(&conn, 600).unwrap();
        let moved: Vec<(String, Option<i64>)> = conn.prepare("SELECT item_id, deleted_at FROM playbacks ORDER BY id").unwrap().query_map([], |r| Ok((r.get(0)?, r.get(1)?))).unwrap().map(Result::unwrap).collect();
        assert_eq!(moved, [("new1".into(), Some(50)), ("new1".into(), None)], "the play in the trash stayed on the old id");
        // And a title whose only play is in the trash.
        conn.execute_batch("UPDATE playbacks SET item_id = 'old2' WHERE id = 1").unwrap();
        relink_orphans(&conn, 600).unwrap();
        assert_eq!(conn.query_row("SELECT item_id FROM playbacks WHERE id = 1", [], |r| r.get::<_, String>(0)).unwrap(), "new1");
    }

    #[test]
    fn a_title_is_found_however_its_name_is_punctuated_or_dated() {
        let c = migrated();
        c.execute_batch(
            "INSERT INTO items(id, type, name, production_year, updated_at) VALUES ('sw1', 'Movie', 'Star Wars: Episode I – The Phantom Menace', 1999, 1),
                ('poke', 'Movie', 'Pokémon: The First Movie', 1998, 1), ('jojo', 'Series', 'JoJo''s Bizarre Adventure (2012)', 2012, 1);
             INSERT INTO items(id, type, name, series_id, parent_index_number, index_number, updated_at) VALUES ('jojo11', 'Episode', 'Phantom Blood', 'jojo', 1, 1, 1);",
        )
        .unwrap();
        orphan_film(&c, "Star Wars: Episode I - The Phantom Menace (1999)");
        orphan_film(&c, "POKEMON: THE FIRST MOVIE");
        orphan_episode(&c, "JoJo's Bizarre Adventure", 1, 1);
        relink_orphans(&c, 600).unwrap();
        assert_eq!(now_at(&c, "Star Wars: Episode I – The Phantom Menace"), "sw1", "a hyphen for a dash");
        assert_eq!(now_at(&c, "Pokémon: The First Movie"), "poke", "case and accents");
        assert_eq!(now_at(&c, "JoJo's Bizarre Adventure (2012)"), "jojo11", "a year in the library's name");
    }

    /// Plex, Tautulli or an old library may know a title by its original-language name (오징어 게임 for Squid Game), which
    /// Jellyfin keeps beside its own.
    #[test]
    fn a_title_is_found_by_its_original_name() {
        let c = migrated();
        c.execute_batch(
            "INSERT INTO items(id, type, name, original_title, production_year, updated_at) VALUES
                ('squid', 'Series', 'Squid Game', '오징어 게임', 2021, 1),
                ('quiet', 'Movie', 'All Quiet on the Western Front', 'Im Westen nichts Neues', 2022, 1);
             INSERT INTO items(id, type, name, series_id, parent_index_number, index_number, updated_at) VALUES ('squid21', 'Episode', 'Bread and Lottery', 'squid', 2, 1, 1);",
        )
        .unwrap();
        orphan_episode(&c, "오징어 게임", 2, 1);
        orphan_film(&c, "Im Westen nichts Neues (2022)");
        relink_orphans(&c, 600).unwrap();
        assert_eq!(now_at(&c, "Squid Game"), "squid21");
        assert_eq!(now_at(&c, "All Quiet on the Western Front"), "quiet");
    }

    /// Release years differ by a year between catalogues (Plex has Kingsman in 2014, Jellyfin in 2015); two is a remake.
    #[test]
    fn a_year_one_off_is_the_same_title_and_a_name_two_titles_share_is_neither() {
        let c = migrated();
        c.execute_batch(
            "INSERT INTO items(id, type, name, production_year, updated_at) VALUES ('kings', 'Movie', 'Kingsman: The Secret Service', 2015, 1),
                ('twinsA', 'Movie', 'Twins', 1988, 1), ('twinsB', 'Movie', 'Twins', 2024, 1),
                ('rain1', 'Movie', 'Black Rain', 1989, 1), ('rain2', 'Movie', 'Black Rain', 1990, 1);",
        )
        .unwrap();
        orphan_film(&c, "Kingsman: The Secret Service (2014)");
        orphan_film(&c, "Twins (1990)");
        orphan_film(&c, "Black Rain (1989)");
        relink_orphans(&c, 600).unwrap();
        assert_eq!(now_at(&c, "Kingsman: The Secret Service"), "kings", "one year apart");
        assert_eq!(now_at(&c, "Twins (1990)"), "gone-Twins (1990)", "two years from either: neither");
        assert_eq!(now_at(&c, "Black Rain"), "rain1", "the exact year wins over the one beside it");
    }

    #[test]
    fn finding_what_is_orphaned_walks_the_titles_not_the_plays() {
        // It runs at every start and after every library read, and nearly always finds nothing. Grouping every play by
        // title to ask each group whether its title is still there took 3.3 s on a million plays; stepping from one
        // title to the next through the title index asks one question per title.
        let c = migrated();
        c.execute_batch(
            "INSERT INTO items(id, type, name, removed, updated_at) VALUES ('here', 'Movie', 'Sintel', 0, 0), ('gone', 'Movie', 'Old cut', 1, 0);
             INSERT INTO playbacks(source, user_id, user_name, item_id, item_name, item_type, started_at, ended_at, duration_s) VALUES
               ('live', 'u1', 'alice', 'here', 'Sintel', 'Movie', 1, 2, 1), ('live', 'u1', 'alice', 'gone', 'Old cut', 'Movie', 3, 4, 1),
               ('live', 'u1', 'alice', 'never', 'Lost', 'Movie', 5, 6, 1), ('live', 'u2', 'bob', 'never', 'Lost', 'Movie', 7, 8, 1);",
        )
        .unwrap();
        let found: Vec<String> = c.prepare(ORPHANS_SQL).unwrap().query_map([], |r| r.get(0)).unwrap().map(Result::unwrap).collect();
        assert_eq!(found, ["gone", "never"]);
        let plan: Vec<String> = c.prepare(&format!("EXPLAIN QUERY PLAN {ORPHANS_SQL}")).unwrap().query_map([], |r| r.get::<_, String>(3)).unwrap().map(Result::unwrap).collect();
        assert!(!plan.iter().any(|p| p.starts_with("SCAN playbacks")), "{plan:?}");
    }

    #[test]
    fn only_the_titles_plays_were_moved_to_are_swept() {
        // A duplicate can only appear where re-linking moved a play. Swept over the whole history instead, re-linking
        // read every imported play at every start and after every library read, under the write lock, to find nothing.
        let c = migrated();
        c.execute_batch(
            "INSERT INTO items(id, type, name, production_year, removed, updated_at) VALUES
                ('new1', 'Movie', 'Big Buck Bunny', 2008, 0, 0), ('i9', 'Movie', 'Sintel', 2010, 0, 0);
             INSERT INTO playbacks(id, source, source_id, user_id, user_name, item_id, item_name, item_type, started_at, ended_at, duration_s) VALUES
               (1, 'live',         NULL,   'u1', 'alice', 'new1', 'Big Buck Bunny',        'Movie', 1000, 4600, 3600),
               (2, 'jellystat',    'js:1', 'u1', 'alice', 'old1', 'Big Buck Bunny (2008)', 'Movie', 1000, 4600, 3600),
               -- a pair on a title nothing was moved to: not this pass's business
               (3, 'jellystat',    'js:2', 'u1', 'alice', 'i9',   'Sintel',                'Movie', 9000, 9900, 900),
               (4, 'streamystats', 'ss:2', 'u1', 'alice', 'i9',   'Sintel',                'Movie', 9000, 9900, 900);",
        )
        .unwrap();
        let r = relink_orphans(&c, 600).unwrap();
        assert_eq!((r.titles, r.duplicates_removed), (1, 1));
        // …and says where plays landed, so whoever asked can regroup those titles: moved plays keep their old times.
        assert_eq!(r.moved_to, ["new1"]);
        let left: Vec<i64> = c.prepare("SELECT id FROM playbacks ORDER BY id").unwrap().query_map([], |r| r.get(0)).unwrap().map(Result::unwrap).collect();
        assert_eq!(left, [1, 3, 4]);
    }
}
