//! The year as a story (2.0): one vertical 1080×1920 card per chapter of the recap, the shape phone
//! stories use, drawn by the same means as the profile cards (`card.rs`).
//!
//! Every card is drawn from a `StoryYear`, never from the recap's JSON: a typed, name-free copy of the
//! year that holds only what may leave finstats. It has no field for a companion, a rank among other
//! people or an app, so a card cannot carry one however the recap grows. The same type is what a
//! published profile shows (`public.rs`), so a card someone downloads in the app and the one a
//! stranger sees under a link can never say different things.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::card::{ACCENT, ACCENT_HI, BG, Canvas, FAINT, HEAT_EMPTY, HEAT_RAMP, MUTED, Posters, Style, TEXT, TILE, thousands};
use crate::public::{Persona, Title};

pub const W: u32 = 1080;
pub const H: u32 = 1920;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Totals {
    pub plays: i64,
    pub watch_s: i64,
    pub active_days: i64,
    pub titles: i64,
    pub movies: i64,
    pub episodes: i64,
    pub tracks: i64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Named {
    pub name: String,
    pub watch_s: i64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Records {
    pub longest_streak_days: Option<i64>,
    /// The day with the most watched, and how much.
    pub biggest_day: Option<(String, i64)>,
    /// Most episodes of one show in a day: the show and the count.
    pub biggest_binge: Option<(String, i64)>,
    /// Watched on the most separate days: the title and the days.
    pub most_rewatched: Option<(String, i64)>,
    /// The oldest title watched: its name and year.
    pub oldest: Option<(String, i64)>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Together {
    pub evenings: i64,
    pub together_s: i64,
    pub share: f64,
    pub top_title: Option<Title>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Finished {
    pub count: i64,
    pub series: Vec<Title>,
    pub dropped_count: i64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Asked {
    pub made: i64,
    pub available: i64,
    pub watched: i64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Versus {
    pub year: i64,
    pub plays: i64,
    pub watch_s: i64,
}

/// One year, as much of it as may be shown to anyone.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StoryYear {
    /// "2025", or "The last 12 months".
    pub label: String,
    /// Whose year it is, as the card says it: set by the caller — a published profile's chosen name,
    /// the server's name for the server's year, empty in the app (a login name never goes on a card).
    pub whose: String,
    pub totals: Totals,
    pub top_series: Vec<Title>,
    pub top_movies: Vec<Title>,
    pub top_tracks: Vec<Title>,
    pub genres: Vec<Named>,
    pub persona: Option<Persona>,
    /// Watch seconds per hour of the day and per weekday (Monday first).
    pub hours: Vec<i64>,
    pub weekdays: Vec<i64>,
    /// Watch seconds per local day, `YYYY-MM-DD`.
    pub days: Vec<(String, i64)>,
    pub records: Records,
    pub together: Option<Together>,
    pub finished: Option<Finished>,
    pub asked: Option<Asked>,
    pub versus: Option<Versus>,
}

/// The chapters a story can have, in the order it tells them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Chapter {
    Year,
    Numbers,
    Shows,
    Films,
    Music,
    Genres,
    Persona,
    Rhythm,
    Days,
    Records,
    Together,
    Finished,
    Asked,
    Versus,
}

impl Chapter {
    pub const ALL: [Chapter; 14] = [
        Chapter::Year, Chapter::Numbers, Chapter::Shows, Chapter::Films, Chapter::Music, Chapter::Genres, Chapter::Persona,
        Chapter::Rhythm, Chapter::Days, Chapter::Records, Chapter::Together, Chapter::Finished, Chapter::Asked, Chapter::Versus,
    ];
    pub fn key(self) -> &'static str {
        match self {
            Chapter::Year => "year",
            Chapter::Numbers => "numbers",
            Chapter::Shows => "shows",
            Chapter::Films => "films",
            Chapter::Music => "music",
            Chapter::Genres => "genres",
            Chapter::Persona => "persona",
            Chapter::Rhythm => "rhythm",
            Chapter::Days => "days",
            Chapter::Records => "records",
            Chapter::Together => "together",
            Chapter::Finished => "finished",
            Chapter::Asked => "asked",
            Chapter::Versus => "versus",
        }
    }
    pub fn parse(s: &str) -> Option<Chapter> {
        Chapter::ALL.into_iter().find(|c| c.key() == s)
    }
}

fn int(v: &Value) -> i64 {
    v.as_i64().unwrap_or(0)
}

fn text(v: &Value) -> Option<String> {
    v.as_str().filter(|s| !s.is_empty()).map(str::to_string)
}

/// A title from the recap's lists: its poster only while the library still has it.
fn title(v: &Value) -> Option<Title> {
    Some(Title {
        name: text(&v["name"])?,
        sub: text(&v["sub"]),
        plays: int(&v["plays"]),
        watch_s: int(&v["watch_s"]),
        image: if v["item_exists"] == false { None } else { text(&v["image_item_id"]) },
    })
}

fn titles(v: &Value) -> Vec<Title> {
    v.as_array().map(|a| a.iter().filter_map(title).take(5).collect()).unwrap_or_default()
}

impl StoryYear {
    /// The year as `recap::build` answers it, copied field by field into what may leave finstats.
    /// Anything the recap has that is not a field here — companions, the rank, the apps — stays behind.
    pub fn from_recap(r: &Value, whose: &str) -> Option<StoryYear> {
        if r["empty"] == true || int(&r["totals"]["plays"]) == 0 {
            return None;
        }
        let t = &r["totals"];
        let rec = &r["records"];
        let pair = |v: &Value, name: &str, n: &str| Some((text(&v[name])?, int(&v[n]))).filter(|(_, n)| *n > 0);
        let series_of = |v: &Value| -> Vec<Title> {
            v.as_array().map(|a| a.iter().filter_map(|s| Some(Title { name: text(&s["name"])?, sub: text(&s["finished_on"]), plays: int(&s["episodes"]), watch_s: 0, image: text(&s["image_item_id"]) })).take(4).collect()).unwrap_or_default()
        };
        Some(StoryYear {
            label: match r["year"].as_i64() {
                Some(y) => y.to_string(),
                None => "The last 12 months".into(),
            },
            whose: whose.trim().to_string(),
            totals: Totals {
                plays: int(&t["plays"]), watch_s: int(&t["watch_s"]), active_days: int(&t["active_days"]), titles: int(&t["distinct_items"]),
                movies: int(&t["movies"]), episodes: int(&t["episodes"]), tracks: int(&t["tracks"]),
            },
            top_series: titles(&r["top_series"]),
            top_movies: titles(&r["top_movies"]),
            top_tracks: titles(&r["top_tracks"]),
            genres: r["top_genres"].as_array().map(|a| a.iter().filter_map(|g| Some(Named { name: text(&g["name"])?, watch_s: int(&g["watch_s"]) })).take(6).collect()).unwrap_or_default(),
            persona: r["persona"].as_object().and_then(|p| Some(Persona { title: text(p.get("title")?)?, line: text(p.get("line")?).unwrap_or_default() })),
            hours: r["hours"].as_array().map(|a| a.iter().map(int).collect()).unwrap_or_default(),
            weekdays: r["weekdays"].as_array().map(|a| a.iter().map(int).collect()).unwrap_or_default(),
            days: r["days"].as_array().map(|a| a.iter().filter_map(|d| Some((text(&d["date"])?, int(&d["watch_s"])))).collect()).unwrap_or_default(),
            records: Records {
                longest_streak_days: rec["longest_streak"]["days"].as_i64(),
                biggest_day: pair(&rec["biggest_day"], "date", "watch_s"),
                biggest_binge: pair(&rec["biggest_binge"], "series_name", "episodes"),
                most_rewatched: pair(&rec["most_rewatched"], "name", "plays"),
                oldest: pair(&rec["oldest_title"], "name", "year"),
            },
            together: r["together"].as_object().map(|g| Together {
                evenings: g.get("evenings").map(int).unwrap_or(0),
                together_s: g.get("together_s").map(int).unwrap_or(0),
                share: g.get("share").and_then(Value::as_f64).unwrap_or(0.0),
                top_title: g.get("top_title").and_then(|t| Some(Title { name: text(&t["name"])?, sub: None, plays: int(&t["evenings"]), watch_s: 0, image: text(&t["image_item_id"]) })),
            }).filter(|g| g.evenings > 0),
            finished: r["finished"].as_object().map(|f| Finished {
                count: f.get("count").map(int).unwrap_or(0),
                series: f.get("series").map(series_of).unwrap_or_default(),
                dropped_count: f.get("dropped_count").map(int).unwrap_or(0),
            }).filter(|f| f.count > 0),
            asked: r["requests"].as_object().map(|q| Asked {
                made: q.get("made").map(int).unwrap_or(0),
                available: q.get("available").map(int).unwrap_or(0),
                watched: q.get("watched").map(int).unwrap_or(0),
            }).filter(|q| q.made > 0),
            versus: r["versus"].as_object().map(|v| Versus { year: v.get("year").map(int).unwrap_or(0), plays: v.get("plays").map(int).unwrap_or(0), watch_s: v.get("watch_s").map(int).unwrap_or(0) }),
        })
    }

    /// The chapters that have something to say, in order.
    pub fn chapters(&self) -> Vec<Chapter> {
        Chapter::ALL
            .into_iter()
            .filter(|ch| match ch {
                Chapter::Year | Chapter::Numbers => true,
                Chapter::Shows => !self.top_series.is_empty(),
                Chapter::Films => !self.top_movies.is_empty(),
                Chapter::Music => !self.top_tracks.is_empty(),
                Chapter::Genres => !self.genres.is_empty(),
                Chapter::Persona => self.persona.is_some(),
                Chapter::Rhythm => self.hours.iter().sum::<i64>() > 0,
                Chapter::Days => !self.days.is_empty(),
                Chapter::Records => self.records != Records::default(),
                Chapter::Together => self.together.is_some(),
                Chapter::Finished => self.finished.is_some(),
                Chapter::Asked => self.asked.is_some(),
                Chapter::Versus => self.versus.is_some(),
            })
            .collect()
    }

    /// What the opening card shows: series and films in turn.
    fn favourites(&self) -> Vec<&Title> {
        let mut out = vec![];
        for i in 0..3 {
            out.extend(self.top_series.get(i));
            out.extend(self.top_movies.get(i));
        }
        out.truncate(3);
        out
    }

    fn drawn(&self, ch: Chapter) -> Vec<&Title> {
        match ch {
            Chapter::Year => self.favourites(),
            Chapter::Shows => self.top_series.iter().collect(),
            Chapter::Films => self.top_movies.iter().collect(),
            Chapter::Music => self.top_tracks.iter().collect(),
            Chapter::Together => self.together.iter().filter_map(|t| t.top_title.as_ref()).collect(),
            Chapter::Finished => self.finished.iter().flat_map(|f| f.series.iter()).collect(),
            _ => vec![],
        }
    }

    /// The posters a chapter draws.
    pub fn poster_ids(&self, ch: Chapter) -> Vec<String> {
        self.drawn(ch).into_iter().filter_map(|t| t.image.clone()).collect()
    }

    /// "Alice's 2025", "Home's 2025", or "2025 in review".
    fn heading(&self) -> String {
        if self.whose.is_empty() { format!("{} in review", self.label) } else { format!("{}’s {}", self.whose, self.label) }
    }
}

// ---------------------------------------------------------------- drawing

const X: f64 = 64.0;
const WIDE: f64 = W as f64 - 2.0 * X;

fn hours(secs: i64) -> String {
    thousands(secs / 3600)
}

fn plural(n: i64, one: &str, many: &str) -> String {
    format!("{} {}", thousands(n), if n == 1 { one } else { many })
}

/// The frame every chapter shares: the accent band, the chapter's word standing faint behind, whose
/// year it is, and the footer.
fn frame(c: &mut Canvas, y: &StoryYear, word: &str) {
    c.push(format!(r#"<rect width="{W}" height="{H}" fill="{BG}"/><rect width="{W}" height="10" fill="{ACCENT}"/>"#));
    c.text(X - 8.0, 420.0, Style(230.0, 800, "#242424"), word);
    c.text_fit(X, 150.0, Style(38.0, 600, MUTED), &y.heading(), Some(WIDE));
    c.text(X, 1840.0, Style(30.0, 600, FAINT), &format!("finstats · {}", y.label));
}

fn headline(c: &mut Canvas, top: f64, s: &str) -> f64 {
    let n = c.wrap(X, top, Style(76.0, 700, TEXT), s, WIDE, 3);
    top + n as f64 * 95.0
}

fn big_number(c: &mut Canvas, top: f64, value: &str, unit: &str) -> f64 {
    c.text(X, top, Style(220.0, 800, ACCENT_HI), value);
    c.text_fit(X, top + 80.0, Style(46.0, 500, MUTED), unit, Some(WIDE));
    top + 140.0
}

/// A ranked list: the first as a large poster, the rest as rows.
fn ranked(c: &mut Canvas, list: &[Title], posters: &Posters, unit: &str, what: &str) {
    let Some(first) = list.first() else { return };
    headline(c, 560.0, &format!("Your top {what}"));
    c.poster(X, 780.0, 420.0, 630.0, first.image.as_ref().and_then(|i| posters.get(i)));
    c.text(X + 460.0, 830.0, Style(40.0, 700, ACCENT_HI), "#1");
    c.wrap(X + 460.0, 900.0, Style(48.0, 700, TEXT), &first.name, WIDE - 460.0, 3);
    c.text_fit(X + 460.0, 1100.0, Style(34.0, 400, MUTED), &format!("{} · {}", plural(first.plays, "play", "plays"), unit_of(first.watch_s, unit)), Some(WIDE - 460.0));
    for (i, t) in list.iter().enumerate().skip(1).take(4) {
        let top = 1460.0 + (i - 1) as f64 * 90.0;
        c.text(X, top + 50.0, Style(36.0, 700, FAINT), &format!("#{}", i + 1));
        c.text_fit(X + 90.0, top + 50.0, Style(38.0, 600, TEXT), &t.name, Some(WIDE - 360.0));
        c.text(X + WIDE - 250.0, top + 50.0, Style(32.0, 400, MUTED), &unit_of(t.watch_s, unit));
    }
}

fn unit_of(secs: i64, unit: &str) -> String {
    if unit == "minutes" { format!("{} min", thousands(secs / 60)) } else { format!("{} h", hours(secs)) }
}

fn bars(c: &mut Canvas, x: f64, top: f64, w: f64, h: f64, values: &[i64], labels: &[(usize, String)]) {
    let max = values.iter().copied().max().unwrap_or(0).max(1);
    let n = values.len().max(1) as f64;
    let gap = 6.0;
    let bw = (w - gap * (n - 1.0)) / n;
    let peak = values.iter().enumerate().max_by_key(|(_, v)| **v).map(|(i, _)| i);
    for (i, v) in values.iter().enumerate() {
        let bh = (h * *v as f64 / max as f64).max(4.0);
        let fill = if Some(i) == peak { ACCENT_HI } else { "#5e4ba8" };
        c.push(format!(r#"<rect x="{:.1}" y="{:.1}" width="{bw:.1}" height="{bh:.1}" rx="4" fill="{fill}"/>"#, x + i as f64 * (bw + gap), top + h - bh));
    }
    for (i, l) in labels {
        c.push(format!(r#"<text x="{:.1}" y="{:.1}" font-size="26" fill="{FAINT}" text-anchor="middle">{}</text>"#, x + *i as f64 * (bw + gap) + bw / 2.0, top + h + 40.0, crate::public::html_escape(l)));
    }
}

/// "28 February", for a day the recap gives as `2025-02-28`.
fn long_day(d: &str) -> String {
    chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").map_or_else(|_| d.to_string(), |d| d.format("%-d %B").to_string())
}

const WEEKDAYS: [&str; 7] = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];

pub fn svg(ch: Chapter, y: &StoryYear, posters: &Posters) -> String {
    let mut c = Canvas::new();
    let poster_of = |t: &Title| t.image.as_ref().and_then(|i| posters.get(i));
    match ch {
        Chapter::Year => {
            frame(&mut c, y, "YEAR");
            let next = big_number(&mut c, 760.0, &hours(y.totals.watch_s), "hours of watching");
            c.text_fit(X, next + 60.0, Style(40.0, 400, MUTED), &format!("{} · {} · {}", plural(y.totals.plays, "play", "plays"), plural(y.totals.titles, "title", "titles"), plural(y.totals.active_days, "day", "days")), Some(WIDE));
            for (i, t) in y.favourites().into_iter().enumerate() {
                c.poster(X + i as f64 * 330.0, 1150.0, 292.0, 438.0, poster_of(t));
                c.wrap(X + i as f64 * 330.0, 1640.0, Style(30.0, 600, TEXT), &t.name, 292.0, 2);
            }
        }
        Chapter::Numbers => {
            frame(&mut c, y, "NUMBERS");
            headline(&mut c, 560.0, "The year in numbers");
            let t = &y.totals;
            let mut tiles = vec![("Plays", thousands(t.plays)), ("Hours", hours(t.watch_s)), ("Titles", thousands(t.titles)), ("Days", thousands(t.active_days))];
            if t.episodes > 0 { tiles.push(("Episodes", thousands(t.episodes))) }
            if t.movies > 0 { tiles.push(("Films", thousands(t.movies))) }
            if t.tracks > 0 { tiles.push(("Songs", thousands(t.tracks))) }
            for (i, (label, value)) in tiles.into_iter().take(6).enumerate() {
                let (cx, cy) = (X + (i % 2) as f64 * 490.0, 760.0 + (i / 2) as f64 * 330.0);
                c.push(format!(r#"<rect x="{cx}" y="{cy}" width="462" height="290" rx="16" fill="{TILE}"/>"#));
                c.text(cx + 40.0, cy + 80.0, Style(34.0, 500, MUTED), label);
                c.text_fit(cx + 40.0, cy + 210.0, Style(110.0, 800, TEXT), &value, Some(390.0));
            }
        }
        Chapter::Shows => { frame(&mut c, y, "SHOWS"); ranked(&mut c, &y.top_series, posters, "hours", "shows") }
        Chapter::Films => { frame(&mut c, y, "FILMS"); ranked(&mut c, &y.top_movies, posters, "hours", "films") }
        Chapter::Music => { frame(&mut c, y, "MUSIC"); ranked(&mut c, &y.top_tracks, posters, "minutes", "songs") }
        Chapter::Genres => {
            frame(&mut c, y, "GENRES");
            let top = headline(&mut c, 560.0, &format!("Mostly {}", y.genres[0].name));
            let max = y.genres.iter().map(|g| g.watch_s).max().unwrap_or(1).max(1);
            for (i, g) in y.genres.iter().enumerate() {
                let gy = top + 120.0 + i as f64 * 160.0;
                c.text_fit(X, gy, Style(40.0, 600, TEXT), &g.name, Some(WIDE - 200.0));
                c.text(X + WIDE - 180.0, gy, Style(34.0, 400, MUTED), &format!("{} h", hours(g.watch_s)));
                let w = (WIDE * g.watch_s as f64 / max as f64).max(8.0);
                c.push(format!(r#"<rect x="{X}" y="{:.0}" width="{WIDE}" height="28" rx="14" fill="{TILE}"/><rect x="{X}" y="{:.0}" width="{w:.0}" height="28" rx="14" fill="{}"/>"#, gy + 30.0, gy + 30.0, if i == 0 { ACCENT_HI } else { "#5e4ba8" }));
            }
        }
        Chapter::Persona => {
            frame(&mut c, y, "YOU");
            if let Some(p) = &y.persona {
                c.text(X, 760.0, Style(44.0, 500, MUTED), "This year you were");
                c.wrap(X, 900.0, Style(120.0, 800, ACCENT_HI), &p.title, WIDE, 3);
                c.wrap(X, 1300.0, Style(46.0, 400, TEXT), &p.line, WIDE, 4);
            }
        }
        Chapter::Rhythm => {
            frame(&mut c, y, "RHYTHM");
            let peak_h = y.hours.iter().enumerate().max_by_key(|(_, v)| **v).map_or(0, |(i, _)| i);
            headline(&mut c, 560.0, &format!("Most of it around {peak_h:02}:00"));
            let labels: Vec<(usize, String)> = (0..24).step_by(6).map(|h| (h, format!("{h:02}"))).collect();
            bars(&mut c, X, 820.0, WIDE, 360.0, &y.hours, &labels);
            if y.weekdays.iter().sum::<i64>() > 0 {
                let peak_d = y.weekdays.iter().enumerate().max_by_key(|(_, v)| **v).map_or(0, |(i, _)| i);
                c.text_fit(X, 1360.0, Style(48.0, 700, TEXT), &format!("{}s were busiest", WEEKDAYS[peak_d]), Some(WIDE));
                let labels: Vec<(usize, String)> = (0..7).map(|d| (d, WEEKDAYS[d][..3].to_string())).collect();
                bars(&mut c, X, 1420.0, WIDE, 220.0, &y.weekdays, &labels);
            }
        }
        Chapter::Days => {
            frame(&mut c, y, "DAYS");
            headline(&mut c, 560.0, &format!("{} with something on", plural(y.totals.active_days, "day", "days")));
            calendar(&mut c, &y.days, y.label.parse().ok(), 820.0);
            if let Some(d) = y.records.longest_streak_days {
                c.text_fit(X, 1560.0, Style(46.0, 600, TEXT), &format!("Longest streak: {}", plural(d, "day", "days")), Some(WIDE));
            }
        }
        Chapter::Records => {
            frame(&mut c, y, "RECORDS");
            headline(&mut c, 560.0, "Records worth bragging about");
            let r = &y.records;
            let mut rows: Vec<(&str, String)> = vec![];
            if let Some(d) = r.longest_streak_days { rows.push(("Longest streak", plural(d, "day", "days"))) }
            if let Some((day, secs)) = &r.biggest_day { rows.push(("Biggest day", format!("{} h on {}", hours(*secs), long_day(day)))) }
            if let Some((show, n)) = &r.biggest_binge { rows.push(("Biggest binge", format!("{} of {show}", plural(*n, "episode", "episodes")))) }
            if let Some((name, n)) = &r.most_rewatched { rows.push(("Most rewatched", format!("{name}, on {}", plural(*n, "day", "days")))) }
            if let Some((name, year)) = &r.oldest { rows.push(("Oldest title", format!("{name} ({year})"))) }
            for (i, (label, value)) in rows.iter().enumerate() {
                let ry = 800.0 + i as f64 * 190.0;
                c.text(X, ry, Style(32.0, 500, MUTED), label);
                c.wrap(X, ry + 60.0, Style(46.0, 700, TEXT), value, WIDE, 2);
            }
        }
        Chapter::Together => {
            frame(&mut c, y, "TOGETHER");
            if let Some(g) = &y.together {
                let next = big_number(&mut c, 780.0, &thousands(g.evenings), if g.evenings == 1 { "evening in company" } else { "evenings in company" });
                c.text_fit(X, next + 60.0, Style(40.0, 400, MUTED), &format!("{} h together · {}% of your watching", hours(g.together_s), (g.share * 100.0).round() as i64), Some(WIDE));
                if let Some(t) = &g.top_title {
                    c.text(X, 1160.0, Style(34.0, 500, FAINT), "What brought people together most");
                    c.poster(X, 1200.0, 300.0, 450.0, poster_of(t));
                    c.wrap(X + 340.0, 1270.0, Style(50.0, 700, TEXT), &t.name, WIDE - 340.0, 3);
                    c.text(X + 340.0, 1500.0, Style(36.0, 400, MUTED), &plural(t.plays, "evening", "evenings"));
                }
            }
        }
        Chapter::Finished => {
            frame(&mut c, y, "FINISHED");
            if let Some(f) = &y.finished {
                headline(&mut c, 560.0, &format!("{} seen to the end", plural(f.count, "show", "shows")));
                for (i, t) in f.series.iter().take(4).enumerate() {
                    let (px, py) = (X + (i % 2) as f64 * 490.0, 780.0 + (i / 2) as f64 * 520.0);
                    c.poster(px, py, 280.0, 420.0, poster_of(t));
                    c.wrap(px, py + 470.0, Style(32.0, 600, TEXT), &t.name, 440.0, 1);
                }
                if f.dropped_count > 0 {
                    c.text_fit(X, 1800.0 - 40.0, Style(36.0, 400, MUTED), &format!("and {} left for later", plural(f.dropped_count, "show", "shows")), Some(WIDE));
                }
            }
        }
        Chapter::Asked => {
            frame(&mut c, y, "ASKED");
            if let Some(q) = &y.asked {
                let next = big_number(&mut c, 780.0, &thousands(q.made), if q.made == 1 { "request" } else { "requests" });
                c.text_fit(X, next + 80.0, Style(50.0, 600, TEXT), &format!("{} arrived", thousands(q.available)), Some(WIDE));
                c.text_fit(X, next + 160.0, Style(50.0, 600, TEXT), &format!("{} watched", thousands(q.watched)), Some(WIDE));
            }
        }
        Chapter::Versus => {
            frame(&mut c, y, "VERSUS");
            if let Some(v) = &y.versus {
                let (now, then) = (y.totals.watch_s, v.watch_s);
                let change = if then > 0 { ((now - then) as f64 / then as f64 * 100.0).round() as i64 } else { 0 };
                let word = if change >= 0 { format!("{change}% more than {}", v.year) } else { format!("{}% less than {}", -change, v.year) };
                headline(&mut c, 560.0, &word);
                let max = now.max(then).max(1) as f64;
                for (i, (label, secs)) in [(y.label.clone(), now), (v.year.to_string(), then)].into_iter().enumerate() {
                    let by = 860.0 + i as f64 * 300.0;
                    c.text(X, by, Style(40.0, 600, if i == 0 { TEXT } else { MUTED }), &label);
                    c.push(format!(r#"<rect x="{X}" y="{:.0}" width="{:.0}" height="90" rx="16" fill="{}"/>"#, by + 30.0, (WIDE * secs as f64 / max).max(12.0), if i == 0 { ACCENT_HI } else { "#4b3f80" }));
                    c.text(X, by + 190.0, Style(40.0, 400, MUTED), &format!("{} h", hours(secs)));
                }
            }
        }
    }
    c.finish_sized(W, H)
}

/// The year as a calendar: a column per week, a row per weekday, in the grid's colours.
fn calendar(c: &mut Canvas, days: &[(String, i64)], year: Option<i32>, top: f64) {
    use chrono::{Datelike, NaiveDate};
    let parsed: Vec<(NaiveDate, i64)> = days.iter().filter_map(|(d, s)| Some((NaiveDate::parse_from_str(d, "%Y-%m-%d").ok()?, *s))).collect();
    // A calendar year opens on its own first week, so an empty January reads as empty, not as missing.
    let Some(first) = year.and_then(|y| NaiveDate::from_ymd_opt(y, 1, 1)).or_else(|| parsed.iter().map(|(d, _)| *d).min()) else { return };
    let start = first - chrono::Duration::days(first.weekday().num_days_from_monday() as i64);
    let mut sorted: Vec<i64> = parsed.iter().map(|(_, s)| *s).filter(|s| *s > 0).collect();
    sorted.sort_unstable();
    // The 95th percentile tops the scale, as on the recap page: one marathon should not wash out the year.
    let cap = sorted.get(sorted.len().saturating_sub(1) * 95 / 100).copied().unwrap_or(1).max(1);
    let (cell, gap) = (15.0, 3.0);
    let by_day: std::collections::HashMap<NaiveDate, i64> = parsed.iter().copied().collect();
    let last = year.and_then(|y| NaiveDate::from_ymd_opt(y, 12, 31)).or_else(|| parsed.iter().map(|(d, _)| *d).max()).unwrap_or(first);
    for d in first.iter_days().take_while(|d| *d <= last) {
        let secs = by_day.get(&d).copied().unwrap_or(0);
        let week = (d - start).num_days() / 7;
        if week > 52 {
            continue;
        }
        let x = X + week as f64 * (cell + gap);
        let y = top + d.weekday().num_days_from_monday() as f64 * (cell + gap) * 2.0;
        let fill = if secs <= 0 { HEAT_EMPTY } else { HEAT_RAMP[((secs.min(cap) as f64 / cap as f64 * HEAT_RAMP.len() as f64 - 1e-9).floor() as usize).min(HEAT_RAMP.len() - 1)] };
        c.push(format!(r#"<rect x="{x:.1}" y="{y:.1}" width="{cell}" height="{:.1}" rx="3" fill="{fill}"/>"#, cell * 2.0 + gap));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn title(name: &str, id: &str) -> Value {
        json!({ "id": id, "name": name, "sub": "2008", "image_item_id": id, "plays": 3, "watch_s": 5400, "episodes": 2, "item_exists": true })
    }

    /// A recap as `recap::build` answers it, with everything the story must leave out filled in.
    fn recap() -> Value {
        let mut days = vec![];
        for d in 1..=28 {
            days.push(json!({ "date": format!("2025-02-{d:02}"), "plays": 1, "watch_s": d * 600 }));
        }
        json!({
            "year": 2025, "empty": false,
            "scope": { "user_id": "u-alice-0001", "user_name": "alice.login", "server_name": "Home" },
            "totals": { "plays": 1203, "watch_s": 412 * 3600, "distinct_items": 300, "movies": 80, "episodes": 1100, "tracks": 23, "active_days": 200 },
            "rank": { "position": 1, "of": 4, "share": 0.5 },
            "top_series": [title("Sintel Stories", "s1"), title("Glass Garden", "s2")],
            "top_movies": [title("Big Buck Bunny", "m1")],
            "top_tracks": [],
            "top_genres": [{ "name": "Drama", "plays": 40, "watch_s": 90000 }, { "name": "Animation", "plays": 10, "watch_s": 30000 }],
            "persona": { "key": "night_owl", "title": "Night owl", "line": "Most of it after 22:00" },
            "hours": (0..24).map(|h| h * 100).collect::<Vec<i64>>(), "weekdays": [1, 2, 3, 4, 5, 6, 7],
            "days": days,
            "records": {
                "longest_streak": { "days": 12, "from": "2025-02-01", "to": "2025-02-12" },
                "biggest_day": { "date": "2025-02-28", "watch_s": 16800, "plays": 1 },
                "biggest_binge": { "date": "2025-03-01", "series_name": "Sintel Stories", "episodes": 6, "watch_s": 10000 },
                "most_rewatched": { "name": "Big Buck Bunny", "plays": 4 },
                "oldest_title": { "name": "Steamboat Days", "year": 1928 },
            },
            "together": { "evenings": 12, "together_s": 40000, "share": 0.1, "top_title": { "id": "m1", "name": "Big Buck Bunny", "image_item_id": "m1", "evenings": 3 },
                          "companions": [{ "user_id": "u-bob-0002", "user_name": "bob", "evenings": 9, "together_s": 30000 }] },
            "finished": { "count": 2, "series": [{ "id": "s1", "name": "Sintel Stories", "image_item_id": "s1", "episodes": 10, "finished_on": "2025-11-02" }], "dropped_count": 1, "dropped": [] },
            "requests": { "made": 5, "available": 4, "watched": 3, "top": [] },
            "versus": { "year": 2024, "plays": 900, "watch_s": 300 * 3600, "active_days": 150 },
            "clients": [{ "name": "Jellyfin Web", "plays": 3, "watch_s": 100 }],
            "people": { "actors": [], "directors": [] },
        })
    }

    fn story() -> StoryYear {
        StoryYear::from_recap(&recap(), "Alice").unwrap()
    }

    #[test]
    fn the_story_copies_the_year_and_nobody_else() {
        let s = story();
        assert_eq!(s.label, "2025");
        assert_eq!(s.whose, "Alice");
        assert_eq!(s.totals.plays, 1203);
        assert_eq!(s.top_series[0].name, "Sintel Stories");
        assert_eq!(s.records.biggest_binge, Some(("Sintel Stories".into(), 6)));
        assert_eq!(s.together.as_ref().map(|t| t.evenings), Some(12));
        let text = serde_json::to_string(&s).unwrap();
        for leak in ["bob", "u-bob-0002", "alice.login", "u-alice-0001", "Jellyfin Web", "position", "rank"] {
            assert!(!text.contains(leak), "{leak} reached the story: {text}");
        }
        assert!(StoryYear::from_recap(&json!({ "year": 2025, "empty": true }), "").is_none(), "an empty year has no story");
        for ch in s.chapters() {
            let card = svg(ch, &s, &Posters::new());
            assert!(!card.contains("bob") && !card.contains("alice.login"), "{ch:?} names somebody");
        }
    }

    #[test]
    fn a_chapter_with_nothing_to_say_is_not_offered() {
        let mut s = story();
        assert!(s.chapters().contains(&Chapter::Shows) && !s.chapters().contains(&Chapter::Music), "no music was played");
        s.together = None;
        s.asked = None;
        s.versus = None;
        s.persona = None;
        let ch = s.chapters();
        for gone in [Chapter::Together, Chapter::Asked, Chapter::Versus, Chapter::Persona] {
            assert!(!ch.contains(&gone), "{gone:?} offered with nothing to say");
        }
        assert_eq!(ch[0], Chapter::Year, "the story always opens on the year");
    }

    #[test]
    fn every_chapter_renders_a_1080_by_1920_png() {
        let s = story();
        for ch in s.chapters() {
            let png = crate::card::render_png(&svg(ch, &s, &Posters::new())).unwrap();
            let (w, h) = (u32::from_be_bytes(png[16..20].try_into().unwrap()), u32::from_be_bytes(png[20..24].try_into().unwrap()));
            assert_eq!((w, h), (W, H), "{ch:?}");
        }
    }

    #[test]
    fn titles_are_escaped_on_every_chapter() {
        let mut r = recap();
        r["top_series"][0]["name"] = json!("Tom & <Jerry>");
        r["top_movies"][0]["name"] = json!("Tom & <Jerry>");
        r["finished"]["series"][0]["name"] = json!("Tom & <Jerry>");
        r["together"]["top_title"]["name"] = json!("Tom & <Jerry>");
        r["top_genres"][0]["name"] = json!("Tom & <Jerry>");
        let s = StoryYear::from_recap(&r, "<script>").unwrap();
        for ch in s.chapters() {
            let svg = svg(ch, &s, &Posters::new());
            assert!(!svg.contains("<Jerry>") && !svg.contains("<script>"), "{ch:?}");
        }
    }

    #[test]
    fn a_chapter_draws_only_the_posters_it_shows() {
        let s = story();
        assert_eq!(s.poster_ids(Chapter::Shows), ["s1", "s2"]);
        assert_eq!(s.poster_ids(Chapter::Films), ["m1"]);
        assert!(s.poster_ids(Chapter::Numbers).is_empty());
        let with = svg(Chapter::Shows, &s, &Posters::from([("s1".to_string(), vec![0xFF, 0xD8, 0xFF, 0xE0])]));
        assert!(with.contains("data:image/jpeg;base64,"), "the poster is embedded");
    }
}
