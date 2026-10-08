//! Plays whose title the library does not have (imported from a server that called it something else, or kept after
//! the title left), and the owner saying where each one is now.
//!
//! `relink` attaches what a name rule can find without guessing. What it cannot is listed here with suggestions worked
//! out the forgiving way (`fuzzy`, in both directions, and by the episode's own name for one a guide moved into another
//! season or show), and the owner picks. A choice moves the plays exactly as `relink` would (the same savepoint, the same
//! rule for a play the history already has) and is remembered (`located`), so a re-import of the same history attaches
//! by itself.

use anyhow::Result;
use serde::Serialize;

use crate::db::rusqlite::{Connection, OptionalExtension, params};
use crate::fuzzy::Query;
use crate::relink::{parse_name, title_key};

/// What a missing title can be, and be located as: something a play is of, never a whole show or season.
const PLAYABLE: [&str; 3] = ["Movie", "Video", "Episode"];

/// A title plays point at that the library does not have.
#[derive(Debug, Serialize, PartialEq, Clone)]
pub struct Missing {
    pub id: String,
    pub item_type: String,
    pub name: String,
    pub series_name: Option<String>,
    pub season: Option<i64>,
    pub episode: Option<i64>,
    pub plays: i64,
    pub last_at: i64,
    /// Where its plays came from, in the order the Activity filter lists trackers: "tautulli", "jellystat"…
    pub sources: Vec<String>,
}

/// A title in the library it might be.
#[derive(Debug, Serialize, PartialEq, Clone)]
pub struct Candidate {
    pub id: String,
    pub item_type: String,
    pub name: String,
    pub year: Option<i64>,
    pub series_name: Option<String>,
    pub season: Option<i64>,
    pub episode: Option<i64>,
}

#[derive(Debug, PartialEq)]
pub enum Refused {
    /// Nothing points at that id, or the library has it after all.
    NotMissing,
    /// No live title by that id.
    NoSuchTitle,
    /// A whole show or season, which no play is of.
    WrongKind,
}

/// Every missing film, video and episode, films by name first, then shows' episodes together and in order.
pub fn missing(conn: &Connection) -> Result<Vec<Missing>> {
    let orphans: Vec<String> = conn.prepare(crate::relink::ORPHANS_SQL)?.query_map([], |r| r.get(0))?.collect::<Result<_, _>>()?;
    if orphans.is_empty() {
        return Ok(vec![]);
    }
    let rows = conn
        .prepare(
            "SELECT p.item_id, MAX(p.item_type), MAX(p.item_name), MAX(p.series_name), MAX(p.season_number), MAX(p.episode_number), COUNT(*), MAX(p.ended_at),
                    group_concat(DISTINCT p.source)
             FROM visible_playbacks p WHERE p.item_id IN (SELECT value FROM json_each(?1)) AND p.item_type IN ('Movie', 'Video', 'Episode')
             GROUP BY p.item_id
             ORDER BY MAX(p.item_type) = 'Episode', COALESCE(MAX(p.series_name), MAX(p.item_name)) COLLATE NOCASE,
                      MAX(p.season_number), MAX(p.episode_number), p.item_id",
        )?
        .query_map([serde_json::to_string(&orphans)?], |r| {
            let mut sources: Vec<String> = r.get::<_, Option<String>>(8)?.unwrap_or_default().split(',').filter(|s| !s.is_empty()).map(str::to_string).collect();
            sources.sort_by_key(|s| crate::stats::SOURCES.iter().position(|k| k == s).unwrap_or(usize::MAX));
            Ok(Missing { id: r.get(0)?, item_type: r.get(1)?, name: r.get(2)?, series_name: r.get(3)?, season: r.get(4)?, episode: r.get(5)?, plays: r.get(6)?, last_at: r.get(7)?, sources })
        })?
        .collect::<Result<_, _>>()?;
    Ok(rows)
}

/// How alike two names are, either way round: the library's shorter name inside Plex's longer one counts as much as
/// the other way. A year written into a name is not part of it.
fn closeness(a: &str, b: &str) -> Option<i32> {
    let (a, b) = (parse_name(a).title, parse_name(b).title);
    let one = Query::new(&a).and_then(|q| q.score(&b));
    let two = Query::new(&b).and_then(|q| q.score(&a));
    one.max(two)
}

/// The better of a title's two names.
fn closest(name: &str, title: &str, original: Option<&str>) -> Option<i32> {
    closeness(name, title).max(original.and_then(|o| closeness(name, o)))
}

/// The library as suggestions are made from it, read once for any number of missing titles.
pub struct Library {
    films: Vec<(Candidate, Option<String>)>,
    series: Vec<(String, String, Option<String>)>,
    episodes: Vec<(Candidate, Option<String>, String)>,
}

impl Library {
    pub fn load(conn: &Connection) -> Result<Self> {
        let films = conn
            .prepare("SELECT id, type, name, original_title, production_year FROM items WHERE removed = 0 AND type IN ('Movie', 'Video')")?
            .query_map([], |r| {
                Ok((Candidate { id: r.get(0)?, item_type: r.get(1)?, name: r.get(2)?, year: r.get(4)?, series_name: None, season: None, episode: None }, r.get(3)?))
            })?
            .collect::<Result<_, _>>()?;
        let series = conn
            .prepare("SELECT id, name, original_title FROM items WHERE removed = 0 AND type = 'Series'")?
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
            .collect::<Result<_, _>>()?;
        let episodes = conn
            .prepare(
                "SELECT e.id, e.name, e.series_id, COALESCE(s.name, e.series_name), e.parent_index_number, e.index_number, e.production_year
                 FROM items e LEFT JOIN items s ON s.id = e.series_id WHERE e.removed = 0 AND e.type = 'Episode'",
            )?
            .query_map([], |r| {
                let name: String = r.get(1)?;
                let key = title_key(&name);
                Ok((Candidate { id: r.get(0)?, item_type: "Episode".into(), name, year: r.get(6)?, series_name: r.get(3)?, season: r.get(4)?, episode: r.get(5)? }, r.get(2)?, key))
            })?
            .collect::<Result<_, _>>()?;
        Ok(Library { films, series, episodes })
    }
}

/// Up to `n` titles `m` might be, likeliest first: by its names when `typed` is empty, else by what was typed.
pub fn candidates(lib: &Library, m: &Missing, typed: Option<&str>, n: usize) -> Vec<Candidate> {
    let typed = typed.map(str::trim).filter(|t| !t.is_empty()).and_then(Query::new);
    // Searched for, anything a play can be of: a film one server keeps as a film may be a show's special on another.
    let mut scored: Vec<(i32, &Candidate)> = match (&typed, m.item_type == "Episode") {
        (Some(_), _) => [films(lib, m, typed.as_ref()), episodes(lib, m, typed.as_ref())].concat(),
        (None, true) => episodes(lib, m, None),
        (None, false) => films(lib, m, None),
    };
    scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.name.cmp(&b.1.name)));
    scored.into_iter().take(n).map(|(_, c)| c.clone()).collect()
}

fn films<'a>(lib: &'a Library, m: &Missing, typed: Option<&Query>) -> Vec<(i32, &'a Candidate)> {
    let parsed = parse_name(&m.name);
    let mut out = vec![];
    for (c, original) in &lib.films {
        let score = match typed {
            Some(q) => q.score(&c.name).max(original.as_deref().and_then(|o| q.score(o))),
            None => closest(&parsed.title, &c.name, original.as_deref()),
        };
        let Some(score) = score else { continue };
        let year = match (parsed.year, c.year) {
            (Some(a), Some(b)) if a == b => 300,
            (Some(a), Some(b)) if (a - b).abs() == 1 => 150,
            _ => 0,
        };
        out.push((score + year, c));
    }
    out
}

fn episodes<'a>(lib: &'a Library, m: &Missing, typed: Option<&Query>) -> Vec<(i32, &'a Candidate)> {
    // The shows it might be, by name; an episode of one of them is likelier the more of it agrees.
    let show = m.series_name.clone().unwrap_or_default();
    let shows: std::collections::HashMap<&str, i32> = match typed {
        Some(_) => Default::default(),
        None => lib.series.iter().filter_map(|(id, name, original)| closest(&show, name, original.as_deref()).map(|s| (id.as_str(), s))).collect(),
    };
    let key = title_key(&m.name);
    let mut out = vec![];
    for (c, series_id, name_key) in &lib.episodes {
        let score = match typed {
            Some(q) => q.score(&format!("{} S{:02}E{:02} {}", c.series_name.as_deref().unwrap_or_default(), c.season.unwrap_or(0), c.episode.unwrap_or(0), c.name)),
            None => {
                // An episode keeps its name when a guide moves it into the specials, or into another show.
                let same_name = !key.is_empty() && *name_key == key;
                let in_show = series_id.as_deref().and_then(|s| shows.get(s)).copied();
                if !same_name && in_show.is_none() {
                    continue;
                }
                let numbers = m.episode.is_some() && (c.season, c.episode) == (m.season, m.episode);
                Some(in_show.unwrap_or(0) + if same_name { 5000 } else { closeness(&m.name, &c.name).unwrap_or(0) / 2 } + if numbers { 1500 } else { 0 })
            }
        };
        if let Some(score) = score {
            out.push((score, c));
        }
    }
    out
}

/// Say that the plays of `from` are of `to`: they move, and the choice is kept for whatever comes under `from` again.
/// How many plays moved, or why not.
pub fn locate(conn: &Connection, from: &str, to: &str, by: &str, merge_window_s: i64) -> Result<Result<usize, Refused>> {
    let from_type: Option<String> = conn.query_row("SELECT MAX(item_type) FROM playbacks WHERE item_id = ?1", [from], |r| r.get(0))?;
    let here: bool = conn.query_row("SELECT EXISTS(SELECT 1 FROM items WHERE id = ?1 AND removed = 0)", [from], |r| r.get(0))?;
    let Some(from_type) = from_type.filter(|_| !here) else { return Ok(Err(Refused::NotMissing)) };
    let Some(to_type): Option<String> = conn.query_row("SELECT type FROM items WHERE id = ?1 AND removed = 0", [to], |r| r.get(0)).optional()? else {
        return Ok(Err(Refused::NoSuchTitle));
    };
    if !PLAYABLE.contains(&from_type.as_str()) || !PLAYABLE.contains(&to_type.as_str()) {
        return Ok(Err(Refused::WrongKind));
    }
    let (moved, _) = move_onto(conn, merge_window_s, from, to)?;
    conn.execute(
        "INSERT INTO located(from_id, to_id, at, by) VALUES (?1, ?2, ?3, ?4) ON CONFLICT(from_id) DO UPDATE SET to_id = excluded.to_id, at = excluded.at, by = excluded.by",
        params![from, to, crate::db::now(), by],
    )?;
    Ok(Ok(moved))
}

/// Point the plays of `from` at the title `to`, named, typed and placed as the library has it (an episode with its show,
/// season and number, a film with none), through `relink`'s move, which re-applies the rule for a play the history already has.
/// (plays moved, duplicates removed).
pub(crate) fn move_onto(conn: &Connection, merge_window_s: i64, from: &str, to: &str) -> Result<(usize, usize)> {
    crate::relink::move_plays(conn, merge_window_s, to, || {
        Ok(conn.execute(
            "UPDATE playbacks SET item_id = i.id, item_name = i.name, item_type = i.type, library_id = COALESCE(i.library_id, playbacks.library_id),
                    runtime_s = COALESCE(i.runtime_s, playbacks.runtime_s),
                    series_id = CASE WHEN i.type = 'Episode' THEN i.series_id END,
                    series_name = CASE WHEN i.type = 'Episode' THEN COALESCE((SELECT s.name FROM items s WHERE s.id = i.series_id), i.series_name) END,
                    season_id = CASE WHEN i.type = 'Episode' THEN i.season_id END,
                    season_number = CASE WHEN i.type = 'Episode' THEN i.parent_index_number END,
                    episode_number = CASE WHEN i.type = 'Episode' THEN i.index_number END
             FROM items i WHERE i.id = ?1 AND playbacks.item_id = ?2",
            params![to, from],
        )?)
    })
}

/// The title the owner located `from` as, when it is still in the library.
pub fn located(conn: &Connection, from: &str) -> Result<Option<String>> {
    Ok(conn
        .query_row("SELECT l.to_id FROM located l JOIN items i ON i.id = l.to_id AND i.removed = 0 WHERE l.from_id = ?1", [from], |r| r.get(0))
        .optional()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn conn() -> Connection {
        let c = Connection::open_in_memory().unwrap();
        for m in crate::db::MIGRATIONS {
            c.execute_batch(m).unwrap();
        }
        c.execute_batch(
            "INSERT INTO items(id, type, name, original_title, production_year, runtime_s, library_id, updated_at) VALUES
               ('empire', 'Movie', 'The Empire Strikes Back', NULL, 1980, 7440, 'films', 1),
               ('jedi', 'Movie', 'Return of the Jedi', NULL, 1983, 7860, 'films', 1),
               ('asphalt', 'Movie', 'Asphalt Burning', 'Børning 3', 2020, 6000, 'films', 1),
               ('bunny', 'Movie', 'Big Buck Bunny', NULL, 2008, 600, 'films', 1),
               ('tour', 'Series', 'The Grand Tour (2016)', NULL, 2016, NULL, 'shows', 1),
               ('orbit', 'Series', 'Low Orbit', NULL, 2021, NULL, 'shows', 1);
             INSERT INTO items(id, type, name, series_id, series_name, season_id, parent_index_number, index_number, runtime_s, library_id, updated_at) VALUES
               ('tour101', 'Episode', 'The Holy Trinity', 'tour', 'The Grand Tour (2016)', 'tour-s1', 1, 1, 3600, 'shows', 1),
               ('tour005', 'Episode', 'A Massive Hunt', 'tour', 'The Grand Tour (2016)', 'tour-s0', 0, 5, 5400, 'shows', 1),
               ('orbit101', 'Episode', 'Pilot', 'orbit', 'Low Orbit', 'orbit-s1', 1, 1, 1800, 'shows', 1);",
        )
        .unwrap();
        c
    }

    fn play(c: &Connection, item: &str, name: &str, ty: &str, show: Option<(&str, i64, i64)>, at: i64) {
        let (series, season, episode) = show.map_or((None, None, None), |(s, n, e)| (Some(s), Some(n), Some(e)));
        c.execute(
            "INSERT INTO playbacks(source, user_id, user_name, item_id, item_name, item_type, series_id, series_name, season_number, episode_number, started_at, ended_at, duration_s)
             VALUES ('tautulli', 'u', 'alice', ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8 + 600, 600)",
            params![item, name, ty, series.map(|s| format!("plex:{s}")), series, season, episode, at],
        )
        .unwrap();
    }

    /// A Plex history as one comes in: films and an episode the library calls something else, a film it has, Live TV.
    fn history(c: &Connection) {
        play(c, "plex:5", "Star Wars: Episode V - The Empire Strikes Back (1980)", "Movie", None, 1000);
        play(c, "plex:5", "Star Wars: Episode V - The Empire Strikes Back (1980)", "Movie", None, 5000);
        play(c, "plex:7", "Børning 3 (2020)", "Movie", None, 2000);
        play(c, "plex:9", "A Massive Hunt", "Episode", Some(("The Not Very Grand Tour", 1, 1)), 3000);
        play(c, "bunny", "Big Buck Bunny", "Movie", None, 4000);
        c.execute("INSERT INTO playbacks(source, user_id, user_name, item_id, item_name, item_type, started_at, ended_at, duration_s) VALUES ('live', 'u', 'alice', 'gone-channel', 'NRK1', 'TvChannel', 1, 2, 1)", []).unwrap();
    }

    #[test]
    fn what_the_library_lacks_is_listed_with_its_plays_and_nothing_it_has() {
        let c = conn();
        history(&c);
        let got: Vec<(String, String, i64)> = missing(&c).unwrap().into_iter().map(|m| (m.id, m.item_type, m.plays)).collect();
        assert_eq!(got, vec![
            ("plex:7".into(), "Movie".into(), 1),
            ("plex:5".into(), "Movie".into(), 2),
            ("plex:9".into(), "Episode".into(), 1),
        ], "films by name, then shows; never a title the library has, nor a channel");
    }

    /// Each missing title says where its plays came from (Tautulli, Jellystat, FinStats itself), in the filter's order.
    #[test]
    fn a_missing_title_says_which_trackers_its_plays_came_from() {
        let c = conn();
        history(&c);
        c.execute("INSERT INTO playbacks(source, user_id, user_name, item_id, item_name, item_type, started_at, ended_at, duration_s) VALUES ('jellystat', 'u', 'alice', 'plex:5', 'The Empire Strikes Back', 'Movie', 7000, 8000, 1000)", []).unwrap();
        let empire = missing(&c).unwrap().into_iter().find(|m| m.id == "plex:5").unwrap();
        assert_eq!(empire.sources, ["jellystat", "tautulli"]);
    }

    fn first(c: &Connection, id: &str, typed: Option<&str>) -> Option<String> {
        let m = missing(c).unwrap().into_iter().find(|m| m.id == id).unwrap();
        candidates(&Library::load(c).unwrap(), &m, typed, 8).into_iter().next().map(|x| x.id)
    }

    #[test]
    fn where_a_missing_title_probably_is_is_suggested_however_it_was_named() {
        let c = conn();
        history(&c);
        assert_eq!(first(&c, "plex:5", None).as_deref(), Some("empire"), "the library's name inside Plex's longer one");
        assert_eq!(first(&c, "plex:7", None).as_deref(), Some("asphalt"), "by the original title Jellyfin keeps");
        assert_eq!(first(&c, "plex:9", None).as_deref(), Some("tour005"), "an episode a guide moved into the specials of another show, by its own name");
        assert_eq!(first(&c, "plex:5", Some("jedi")).as_deref(), Some("jedi"), "and anything can be searched for");
        assert_eq!(first(&c, "plex:5", Some("nothing like it at all")), None);
    }

    #[test]
    fn a_located_title_takes_its_plays_and_is_remembered_for_the_next_import() {
        let c = conn();
        history(&c);
        assert_eq!(locate(&c, "plex:9", "tour005", "u", 600).unwrap(), Ok(1));
        let moved: (String, String, Option<String>, Option<String>, Option<i64>, Option<i64>) = c
            .query_row("SELECT item_id, item_name, series_id, series_name, season_number, episode_number FROM playbacks WHERE item_name = 'A Massive Hunt'", [], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?))
            })
            .unwrap();
        assert_eq!(moved, ("tour005".into(), "A Massive Hunt".into(), Some("tour".into()), Some("The Grand Tour (2016)".into()), Some(0), Some(5)), "now a special of The Grand Tour");
        assert_eq!(located(&c, "plex:9").unwrap().as_deref(), Some("tour005"));
        assert!(!missing(&c).unwrap().iter().any(|m| m.id == "plex:9"));

        // The same history imported again: its play is attached by the choice, without asking.
        play(&c, "plex:9", "A Massive Hunt", "Episode", Some(("The Not Very Grand Tour", 1, 1)), 9000);
        crate::relink::relink_orphans(&c, 600).unwrap();
        let again: String = c.query_row("SELECT item_id FROM playbacks WHERE started_at = 9000", [], |r| r.get(0)).unwrap();
        assert_eq!(again, "tour005");
    }

    /// A film one server keeps as a film can be a special of a show on another (anime films often are), so a film may be
    /// located as an episode and an episode as a film, the play becoming what it now is. A whole show is never a play.
    #[test]
    fn a_film_may_be_located_as_an_episode_but_nothing_as_a_whole_show() {
        let c = conn();
        history(&c);
        assert_eq!(first(&c, "plex:7", Some("massive hunt")).as_deref(), Some("tour005"), "a search from a film finds episodes too");
        assert_eq!(locate(&c, "plex:7", "tour", "u", 600).unwrap(), Err(Refused::WrongKind), "a show is not a play");
        assert_eq!(locate(&c, "plex:7", "tour005", "u", 600).unwrap(), Ok(1));
        let (ty, series, episode): (String, Option<String>, Option<i64>) =
            c.query_row("SELECT item_type, series_id, episode_number FROM playbacks WHERE item_id = 'tour005'", [], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))).unwrap();
        assert_eq!((ty.as_str(), series.as_deref(), episode), ("Episode", Some("tour"), Some(5)), "the play is the episode it now is");
    }

    #[test]
    fn a_title_is_located_only_when_it_is_missing_and_the_title_is_there() {
        let c = conn();
        history(&c);
        assert_eq!(locate(&c, "plex:5", "nothing-here", "u", 600).unwrap(), Err(Refused::NoSuchTitle));
        assert_eq!(locate(&c, "bunny", "empire", "u", 600).unwrap(), Err(Refused::NotMissing), "the library has it");
        assert_eq!(locate(&c, "plex:404", "empire", "u", 600).unwrap(), Err(Refused::NotMissing), "nothing points there");
        assert_eq!(locate(&c, "plex:5", "empire", "u", 600).unwrap(), Ok(2));
        let names: Vec<String> = c.prepare("SELECT DISTINCT item_name FROM playbacks WHERE item_id = 'empire'").unwrap().query_map([], |r| r.get(0)).unwrap().map(Result::unwrap).collect();
        assert_eq!(names, ["The Empire Strikes Back"], "called what the library calls it");
    }
}
