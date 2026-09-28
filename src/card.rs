//! Shareable cards (2.0): a published profile drawn as a 1200×630 picture, the size chat apps and
//! social sites show under a pasted link.
//!
//! The card is SVG built here as text and rasterised with resvg, with Inter bundled (`fonts/card/`) so
//! it looks the same on every machine and never reads a system font. The browser's charts cannot run
//! here — `charts.js` builds DOM with tooltips and observers — so the one shape the card shares with
//! the page, the weekday × hour grid, is drawn from the same colours (`HEAT_EMPTY`, `HEAT_RAMP` in
//! `charts.js`; change them together). Only a `PublicProfile` goes in, so a card can say nothing the
//! published page does not.

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use anyhow::Result;
use resvg::usvg;

use crate::public::{PublicProfile, Title};

pub const W: u32 = 1200;
pub const H: u32 = 630;

/// Which card: the profile as a whole, or the year.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    Profile,
    Recap,
}

impl Kind {
    pub fn parse(s: Option<&str>) -> Option<Kind> {
        match s.unwrap_or("profile") {
            "profile" => Some(Kind::Profile),
            "recap" => Some(Kind::Recap),
            _ => None,
        }
    }
    /// A card is only drawn from sections that are published.
    pub fn available(self, a: &PublicProfile) -> bool {
        match self {
            Kind::Profile => a.totals.is_some() || a.habits.is_some(),
            Kind::Recap => a.recap.is_some(),
        }
    }
}

/// The items whose posters this card draws, in the order it draws them.
pub fn poster_ids(kind: Kind, a: &PublicProfile) -> Vec<String> {
    let drawn = match kind {
        Kind::Profile => shelf(a),
        Kind::Recap => recap_picks(a).into_iter().map(|(_, t)| t).collect(),
    };
    drawn.into_iter().filter_map(|t| t.image.clone()).collect()
}

/// A ZIP of files stored as they are: PNGs do not compress, so nothing is gained by deflating them, and
/// the format is then small enough to write by hand rather than take a dependency for.
pub fn zip(files: &[(String, Vec<u8>)]) -> Vec<u8> {
    let mut out = vec![];
    let mut dir = vec![];
    // 1980-01-01 00:00 in DOS form: the files are made now, and no reader needs to be told when.
    let (time, date): (u16, u16) = (0, 0x21);
    for (name, data) in files {
        let mut crc = flate2::Crc::new();
        crc.update(data);
        let (crc, size, offset) = (crc.sum(), data.len() as u32, out.len() as u32);
        let common = |b: &mut Vec<u8>| {
            b.extend(20u16.to_le_bytes()); // version needed: 2.0
            b.extend(0u16.to_le_bytes()); // flags
            b.extend(0u16.to_le_bytes()); // stored
            b.extend(time.to_le_bytes());
            b.extend(date.to_le_bytes());
            b.extend(crc.to_le_bytes());
            b.extend(size.to_le_bytes());
            b.extend(size.to_le_bytes());
            b.extend((name.len() as u16).to_le_bytes());
            b.extend(0u16.to_le_bytes()); // no extra field
        };
        out.extend(0x0403_4b50u32.to_le_bytes());
        common(&mut out);
        out.extend(name.as_bytes());
        out.extend(data);
        dir.extend(0x0201_4b50u32.to_le_bytes());
        dir.extend(20u16.to_le_bytes()); // made by
        common(&mut dir);
        dir.extend([0u8; 10]); // no comment, disk 0, no internal or external attributes
        dir.extend(offset.to_le_bytes());
        dir.extend(name.as_bytes());
    }
    let (at, len, n) = (out.len() as u32, dir.len() as u32, files.len() as u16);
    out.extend(dir);
    out.extend(0x0605_4b50u32.to_le_bytes());
    out.extend([0u8; 4]); // this disk, the directory's disk
    out.extend(n.to_le_bytes());
    out.extend(n.to_le_bytes());
    out.extend(len.to_le_bytes());
    out.extend(at.to_le_bytes());
    out.extend(0u16.to_le_bytes()); // no comment
    out
}

/// Rendered cards, newest last. A card takes a few hundred milliseconds to draw and a pasted link is
/// fetched by every chat app it lands in, so the same card is drawn once.
pub const CACHE_MAX: usize = 64;

#[derive(Default)]
pub struct Cache {
    cards: std::sync::Mutex<std::collections::VecDeque<(String, Arc<Vec<u8>>)>>,
}

impl Cache {
    pub fn get(&self, key: &str) -> Option<Arc<Vec<u8>>> {
        self.cards.lock().unwrap().iter().find(|(k, _)| k == key).map(|(_, png)| png.clone())
    }
    pub fn put(&self, key: String, png: Vec<u8>) -> Arc<Vec<u8>> {
        let png = Arc::new(png);
        let mut cards = self.cards.lock().unwrap();
        cards.retain(|(k, _)| *k != key);
        if cards.len() >= CACHE_MAX {
            cards.pop_front();
        }
        cards.push_back((key, png.clone()));
        png
    }
    #[cfg(test)]
    fn len(&self) -> usize {
        self.cards.lock().unwrap().len()
    }
}

/// What a card is drawn from: its link, its kind and everything it would say.
pub fn key(token: &str, kind: Kind, a: &PublicProfile) -> String {
    use sha2::{Digest, Sha256};
    let said = serde_json::to_vec(a).unwrap_or_default();
    format!("{token}:{kind:?}:{}", hex::encode(&Sha256::digest(&said)[..12]))
}

/// Posters by item id, as the image cache holds them (JPEG, PNG or WebP bytes).
pub type Posters = HashMap<String, Vec<u8>>;

pub fn svg(kind: Kind, a: &PublicProfile, posters: &Posters) -> String {
    let mut c = Canvas::new();
    c.push(format!(r#"<rect width="{W}" height="{H}" fill="{BG}"/><rect width="{W}" height="6" fill="{ACCENT}"/>"#));
    match kind {
        Kind::Profile => profile(&mut c, a, posters),
        Kind::Recap => recap(&mut c, a, posters),
    }
    c.text(64.0, 590.0, Style(22.0, 600, FAINT), "finstats");
    c.finish()
}

// The app's own tokens (`app.css`): Obsidian's dark theme.
pub(crate) const BG: &str = "#1e1e1e";
pub(crate) const TILE: &str = "#262626";
pub(crate) const TEXT: &str = "#dadada";
pub(crate) const MUTED: &str = "#a8a8a8";
pub(crate) const FAINT: &str = "#7d7d7d";
pub(crate) const ACCENT: &str = "#8a5cf5";
pub(crate) const ACCENT_HI: &str = "#a68af9";
// The weekday × hour grid's colours, as `charts.js` has them.
pub(crate) const HEAT_EMPTY: &str = "#2f2f2f";
pub(crate) const HEAT_RAMP: [&str; 6] = ["#3a3358", "#4b3f80", "#5e4ba8", "#7459d0", "#8f6ff0", "#b49dfb"];
pub(crate) const DAYS: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

/// Size, weight and colour of a line of text.
#[derive(Clone, Copy)]
pub(crate) struct Style(pub f64, pub u16, pub &'static str);

pub(crate) struct Canvas {
    body: String,
    clips: usize,
}

impl Canvas {
    pub(crate) fn new() -> Self {
        Canvas { body: String::new(), clips: 0 }
    }
    pub(crate) fn push(&mut self, s: impl AsRef<str>) {
        self.body.push_str(s.as_ref());
    }
    /// One line of text, cut to `max_w` pixels when given.
    pub(crate) fn text_fit(&mut self, x: f64, y: f64, st: Style, s: &str, max_w: Option<f64>) {
        let Style(size, weight, fill) = st;
        let s = match max_w {
            Some(w) => fit(s, w, size, weight),
            None => s.to_string(),
        };
        self.push(format!(r#"<text x="{x}" y="{y}" font-size="{size}" font-weight="{weight}" fill="{fill}">{}</text>"#, xml(&s)));
    }
    /// Word-wrapped into at most `lines` lines of `max_w`; the last one cut when it all does not fit.
    /// Answers how many lines it took.
    pub(crate) fn wrap(&mut self, x: f64, y: f64, st: Style, s: &str, max_w: f64, lines: usize) -> usize {
        let room = chars_in(max_w, st.0, st.1);
        let mut out: Vec<String> = vec![];
        let mut words = s.split_whitespace();
        while let Some(w) = words.next() {
            let full = out.len() == lines;
            let fits = out.last().is_some_and(|l| l.chars().count() + 1 + w.chars().count() <= room);
            if out.is_empty() || (!fits && !full) {
                out.push(w.to_string());
            } else {
                // Fits on this line, or there is no line left: the rest goes on the last one, which is cut.
                let last = out.last_mut().unwrap();
                last.push(' ');
                last.push_str(w);
                if !fits {
                    for rest in words.by_ref() {
                        last.push(' ');
                        last.push_str(rest);
                    }
                }
            }
        }
        for (i, line) in out.iter().enumerate() {
            self.text_fit(x, y + i as f64 * st.0 * 1.25, st, line, Some(max_w));
        }
        out.len()
    }
    pub(crate) fn text(&mut self, x: f64, y: f64, st: Style, s: &str) {
        self.text_fit(x, y, st, s, None);
    }
    /// A poster in a rounded frame, or the frame alone when there is no picture to put in it.
    pub(crate) fn poster(&mut self, x: f64, y: f64, w: f64, h: f64, bytes: Option<&Vec<u8>>) {
        self.push(format!(r#"<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="8" fill="{TILE}"/>"#));
        let Some(uri) = bytes.and_then(|b| data_uri(b)) else { return };
        self.clips += 1;
        let id = self.clips;
        self.push(format!(
            r#"<clipPath id="c{id}"><rect x="{x}" y="{y}" width="{w}" height="{h}" rx="8"/></clipPath><image x="{x}" y="{y}" width="{w}" height="{h}" preserveAspectRatio="xMidYMid slice" clip-path="url(#c{id})" href="{uri}"/>"#
        ));
    }
    pub(crate) fn finish(self) -> String {
        self.finish_sized(W, H)
    }
    pub(crate) fn finish_sized(self, w: u32, h: u32) -> String {
        format!(r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}" font-family="Inter">{}</svg>"#, self.body)
    }
}

fn shown_name(a: &PublicProfile) -> &str {
    if a.name.is_empty() { "A finstats profile" } else { &a.name }
}

/// Series and films taken in turn, then music: the posters row, as many as there is room for — six on
/// a card of its own, four beside the weekday grid.
fn shelf(a: &PublicProfile) -> Vec<&Title> {
    let Some(t) = &a.totals else { return vec![] };
    let mut out = vec![];
    for i in 0..5 {
        out.extend(t.top_series.get(i));
        out.extend(t.top_movies.get(i));
    }
    out.extend(t.top_tracks.iter());
    out.truncate(if a.habits.is_none() { 6 } else { 4 });
    out
}

fn recap_picks(a: &PublicProfile) -> Vec<(&'static str, &Title)> {
    let Some(r) = &a.recap else { return vec![] };
    [("Top show", r.top_series.first()), ("Top film", r.top_movies.first())].into_iter().filter_map(|(l, t)| t.map(|t| (l, t))).collect()
}

fn profile(c: &mut Canvas, a: &PublicProfile, posters: &Posters) {
    c.text_fit(64.0, 118.0, Style(60.0, 700, TEXT), shown_name(a), Some(1072.0));
    if let Some(t) = &a.totals {
        let line = format!("{} hours watched · {} plays", thousands(t.watch_s / 3600), thousands(t.plays));
        c.text(64.0, 168.0, Style(28.0, 400, MUTED), &line);
        let (pw, ph, gap) = (150.0, 225.0, 24.0);
        for (i, title) in shelf(a).into_iter().enumerate() {
            let x = 64.0 + i as f64 * (pw + gap);
            c.poster(x, 214.0, pw, ph, title.image.as_ref().and_then(|id| posters.get(id)));
            let n = c.wrap(x, 466.0, Style(17.0, 600, TEXT), &title.name, pw, 2);
            if let Some(sub) = &title.sub {
                c.text_fit(x, 466.0 + n as f64 * 21.0 + 2.0, Style(15.0, 400, FAINT), sub, Some(pw));
            }
        }
    }
    if let Some(hb) = &a.habits {
        let (x, y, w, row) = if a.totals.is_some() { (780.0, 232.0, 356.0, 14.0) } else { (64.0, 214.0, 1072.0, 30.0) };
        c.text(x, y - 18.0, Style(18.0, 600, FAINT), "When they watch");
        heat(c, x, y, w, row, &hb.heatmap.watch_s);
        let under = y + 7.0 * (row + 2.0) + 58.0;
        let streak = format!("Longest streak {} {}", thousands(hb.longest_streak_days), if hb.longest_streak_days == 1 { "day" } else { "days" });
        c.text(x, under, Style(24.0, 600, TEXT), &streak);
        c.text(x, under + 32.0, Style(20.0, 400, MUTED), &format!("{} days with something on", thousands(hb.active_days)));
    }
}

/// The weekday × hour grid, the way the page draws it: day labels on the left, 24 columns of cells.
fn heat(c: &mut Canvas, x: f64, y: f64, w: f64, row: f64, grid: &[Vec<i64>]) {
    let (label, gap) = (40.0, 2.0);
    let cw = (w - label - gap * 23.0) / 24.0;
    let max = grid.iter().flatten().copied().max().unwrap_or(0);
    for (d, name) in DAYS.iter().enumerate() {
        let cy = y + d as f64 * (row + gap);
        c.text(x, cy + row * 0.5 + 5.0, Style(13.0, 400, FAINT), name);
        for hr in 0..24 {
            let v = grid.get(d).and_then(|r| r.get(hr)).copied().unwrap_or(0);
            let fill = if v <= 0 || max <= 0 { HEAT_EMPTY } else { HEAT_RAMP[((v as f64 / max as f64 * HEAT_RAMP.len() as f64 - 1e-9).floor() as usize).min(HEAT_RAMP.len() - 1)] };
            c.push(format!(r#"<rect x="{:.1}" y="{cy:.1}" width="{:.1}" height="{row}" rx="3" fill="{fill}"/>"#, x + label + hr as f64 * (cw + gap), cw.max(1.0)));
        }
    }
    // The hours along the bottom, every third one, as the page has them.
    let base = y + 7.0 * (row + gap) + 14.0;
    for hr in (0..24).step_by(3) {
        c.push(format!(
            r#"<text x="{:.1}" y="{base:.1}" font-size="12" fill="{FAINT}" text-anchor="middle">{hr:02}</text>"#,
            x + label + hr as f64 * (cw + gap) + cw / 2.0
        ));
    }
}

fn recap(c: &mut Canvas, a: &PublicProfile, posters: &Posters) {
    let Some(r) = &a.recap else { return };
    c.text_fit(64.0, 170.0, Style(132.0, 700, ACCENT_HI), &r.label, Some(620.0));
    let whose = if a.name.is_empty() { "A year on finstats".to_string() } else { format!("{}’s year", a.name) };
    c.text_fit(64.0, 226.0, Style(34.0, 600, TEXT), &whose, Some(620.0));
    c.text(64.0, 330.0, Style(84.0, 700, TEXT), &thousands(r.totals.watch_s / 3600));
    c.text(64.0, 366.0, Style(24.0, 400, MUTED), "hours watched");
    let mut facts = vec![format!("{} plays", thousands(r.totals.plays)), format!("{} days", thousands(r.totals.active_days))];
    if let Some(d) = r.records.longest_streak_days {
        facts.push(format!("a {d}-day streak"));
    }
    c.text_fit(64.0, 414.0, Style(22.0, 400, MUTED), &facts.join(" · "), Some(620.0));
    if let Some(p) = &r.persona {
        c.text_fit(64.0, 480.0, Style(30.0, 700, ACCENT_HI), &p.title, Some(620.0));
        c.text_fit(64.0, 514.0, Style(20.0, 400, MUTED), &p.line, Some(620.0));
    } else if let Some(g) = r.genres.first() {
        c.text_fit(64.0, 480.0, Style(30.0, 700, ACCENT_HI), &format!("Mostly {}", g.name), Some(620.0));
    }
    let picks = recap_picks(a);
    let (pw, ph) = (200.0, 300.0);
    // Laid out from the right edge, so one poster does not leave a hole beside it.
    let n = picks.len();
    for (i, (label, t)) in picks.into_iter().enumerate() {
        let x = 1136.0 - (n - i) as f64 * (pw + 24.0) + 24.0;
        c.text(x, 122.0, Style(16.0, 600, FAINT), label);
        c.poster(x, 136.0, pw, ph, t.image.as_ref().and_then(|id| posters.get(id)));
        c.wrap(x, 466.0, Style(20.0, 600, TEXT), &t.name, pw, 2);
    }
}

/// Inter's average advance is a little over half the size; a bold face runs wider. Close enough to cut
/// a title before it runs off the card, which is all this is for.
fn chars_in(max_w: f64, size: f64, weight: u16) -> usize {
    (max_w / (size * if weight >= 600 { 0.56 } else { 0.52 })).floor() as usize
}

fn fit(s: &str, max_w: f64, size: f64, weight: u16) -> String {
    let room = chars_in(max_w, size, weight);
    if s.chars().count() <= room {
        return s.to_string();
    }
    let cut: String = s.chars().take(room.saturating_sub(1)).collect();
    format!("{}…", cut.trim_end())
}

fn xml(s: &str) -> String {
    crate::public::html_escape(s)
}

fn data_uri(b: &[u8]) -> Option<String> {
    let mime = match b {
        [0xFF, 0xD8, ..] => "image/jpeg",
        [0x89, b'P', b'N', b'G', ..] => "image/png",
        [b'R', b'I', b'F', b'F', _, _, _, _, b'W', b'E', b'B', b'P', ..] => "image/webp",
        [b'G', b'I', b'F', ..] => "image/gif",
        _ => return None,
    };
    Some(format!("data:{mime};base64,{}", base64(b)))
}

fn base64(b: &[u8]) -> String {
    const A: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(b.len().div_ceil(3) * 4);
    for chunk in b.chunks(3) {
        let n = (chunk[0] as u32) << 16 | (*chunk.get(1).unwrap_or(&0) as u32) << 8 | *chunk.get(2).unwrap_or(&0) as u32;
        for i in 0..4 {
            out.push(if i <= chunk.len() { A[(n >> (18 - 6 * i) & 63) as usize] as char } else { '=' });
        }
    }
    out
}

pub(crate) fn thousands(n: i64) -> String {
    let digits = n.unsigned_abs().to_string();
    let mut out = String::new();
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(ch);
    }
    if n < 0 { format!("-{out}") } else { out }
}

fn fonts() -> Arc<usvg::fontdb::Database> {
    static DB: OnceLock<Arc<usvg::fontdb::Database>> = OnceLock::new();
    DB.get_or_init(|| {
        let mut db = usvg::fontdb::Database::new();
        for face in [&include_bytes!("../fonts/card/Inter-Regular.ttf")[..], include_bytes!("../fonts/card/Inter-SemiBold.ttf"), include_bytes!("../fonts/card/Inter-Bold.ttf")] {
            db.load_font_data(face.to_vec());
        }
        Arc::new(db)
    })
    .clone()
}

/// Bundled fonts only, and an `<image>` may only be the data the card itself put there: never a path
/// on this machine, whatever ends up in the text.
fn options() -> usvg::Options<'static> {
    let mut o = usvg::Options { font_family: "Inter".into(), fontdb: fonts(), ..usvg::Options::default() };
    o.image_href_resolver.resolve_string = Box::new(|_, _| None);
    o
}

pub fn render_png(svg: &str) -> Result<Vec<u8>> {
    let tree = usvg::Tree::from_str(svg, &options())?;
    // As large as the SVG says it is: a wide card or a story card.
    let size = tree.size().to_int_size();
    let mut pixmap = resvg::tiny_skia::Pixmap::new(size.width(), size.height()).ok_or_else(|| anyhow::anyhow!("no pixmap"))?;
    resvg::render(&tree, resvg::tiny_skia::Transform::default(), &mut pixmap.as_mut());
    Ok(pixmap.encode_png()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::public::{Habits, Heat, Totals};

    fn t(name: &str, image: Option<&str>) -> Title {
        Title { name: name.into(), sub: Some("2008".into()), plays: 3, watch_s: 5400, image: image.map(str::to_string) }
    }

    fn profile() -> PublicProfile {
        PublicProfile {
            name: "Alice".into(),
            avatar: false,
            totals: Some(Totals {
                plays: 1203, watch_s: 412 * 3600 + 59, movies: 80, episodes: 1100, tracks: 23,
                top_series: vec![t("Sintel Stories", Some("s1"))], top_movies: vec![t("Big Buck Bunny", Some("m1"))], top_tracks: vec![],
            }),
            habits: Some(Habits { longest_streak_days: 23, active_days: 200, heatmap: Heat { plays: vec![vec![0; 24]; 7], watch_s: vec![vec![0; 24]; 7].into_iter().enumerate().map(|(d, mut r)| { r[20] = d as i64 * 600; r }).collect() }, genres: vec![] }),
            recap: crate::story::StoryYear::from_recap(&serde_json::json!({
                "year": 2025, "totals": { "plays": 500, "watch_s": 300 * 3600, "active_days": 150 },
                "top_series": [{ "name": "Sintel Stories", "sub": "2008", "plays": 3, "watch_s": 5400, "image_item_id": "s1" }],
                "top_genres": [{ "name": "Drama", "watch_s": 100 }],
                "records": { "longest_streak": { "days": 12 } },
            }), "Alice"),
            story: None,
            recent: None,
        }
    }

    #[test]
    fn the_zip_holds_every_card_intact() {
        let files = vec![("2025-year.png".to_string(), vec![1u8, 2, 3, 4, 5]), ("2025-days.png".to_string(), (0..=255u8).collect::<Vec<u8>>())];
        let z = zip(&files);
        let u16at = |i: usize| u16::from_le_bytes([z[i], z[i + 1]]) as usize;
        let u32at = |i: usize| u32::from_le_bytes([z[i], z[i + 1], z[i + 2], z[i + 3]]);
        let mut at = 0;
        for (name, data) in &files {
            assert_eq!(u32at(at), 0x0403_4b50, "a local header");
            let (crc, size, name_len, extra) = (u32at(at + 14), u32at(at + 18) as usize, u16at(at + 26), u16at(at + 28));
            assert_eq!(&z[at + 30..at + 30 + name_len], name.as_bytes());
            let body = &z[at + 30 + name_len + extra..at + 30 + name_len + extra + size];
            assert_eq!(body, &data[..], "stored as it is");
            let mut check = flate2::Crc::new();
            check.update(data);
            assert_eq!(crc, check.sum(), "{name}");
            at += 30 + name_len + extra + size;
        }
        assert_eq!(u32at(at), 0x0201_4b50, "the central directory follows the files");
        let end = z.len() - 22;
        assert_eq!(u32at(end), 0x0605_4b50, "and the end record closes it");
        assert_eq!((u16at(end + 8), u16at(end + 10)), (2, 2), "two entries");
        assert_eq!(u32at(end + 16) as usize, at, "the directory is where the end record says");
    }

    /// A name or a title is somebody else's text, and XML refuses most control characters outright:
    /// one of them in a Jellyfin title must not make every card that shows it fail.
    #[test]
    fn a_control_character_in_a_name_does_not_break_the_card() {
        let mut a = profile();
        a.name = "Al\u{1}ice".into();
        let png = render_png(&svg(Kind::Profile, &a, &Posters::new()));
        assert!(png.is_ok(), "{:?}", png.err());
    }

    #[test]
    fn a_card_asks_only_for_the_posters_it_draws() {
        let a = profile();
        assert_eq!(poster_ids(Kind::Profile, &a), ["s1", "m1"]);
        assert_eq!(poster_ids(Kind::Recap, &a), ["s1"]);
        let mut b = a.clone();
        b.totals = None;
        assert!(poster_ids(Kind::Profile, &b).is_empty(), "the grid alone has no posters");
    }

    #[test]
    fn the_cache_keeps_the_newest_and_forgets_the_oldest() {
        let c = Cache::default();
        for i in 0..CACHE_MAX + 1 {
            c.put(format!("k{i}"), vec![i as u8]);
        }
        assert!(c.get("k0").is_none(), "the oldest went first");
        assert_eq!(c.get(&format!("k{CACHE_MAX}")).as_deref().map(Vec::as_slice), Some(&[CACHE_MAX as u8][..]));
        assert_eq!(c.len(), CACHE_MAX);
        // The key changes with what the card would say, so a changed profile is never answered from memory.
        let a = profile();
        let mut b = a.clone();
        b.totals.as_mut().unwrap().plays += 1;
        assert_ne!(key("tok", Kind::Profile, &a), key("tok", Kind::Profile, &b));
        assert_ne!(key("tok", Kind::Profile, &a), key("tok", Kind::Recap, &a));
    }

    #[test]
    fn titles_and_names_are_escaped() {
        let mut a = profile();
        a.name = "<script>&\"x\"".into();
        a.totals.as_mut().unwrap().top_series[0].name = "Tom & <Jerry>".into();
        for kind in [Kind::Profile, Kind::Recap] {
            let s = svg(kind, &a, &Posters::new());
            assert!(!s.contains("<script>") && !s.contains("<Jerry>"), "{s}");
            assert!(s.contains("&lt;script&gt;&amp;&quot;x&quot;"), "{s}");
        }
        assert!(svg(Kind::Profile, &a, &Posters::new()).contains("Tom &amp; &lt;Jerry&gt;"));
    }

    #[test]
    fn a_long_title_is_cut_not_overflowing() {
        let mut a = profile();
        let long = "The Extraordinarily Long and Winding Chronicle of a Very Small Rabbit";
        a.totals.as_mut().unwrap().top_series[0].name = long.into();
        let s = svg(Kind::Profile, &a, &Posters::new());
        assert!(!s.contains(long), "the whole title was drawn");
        assert!(s.contains(">The") && s.contains("…</text>"), "cut with an ellipsis");
    }

    #[test]
    fn an_unpublished_section_draws_nothing() {
        let mut a = profile();
        a.totals = None;
        let s = svg(Kind::Profile, &a, &Posters::new());
        assert!(!s.contains("Sintel Stories") && !s.contains("hours watched"), "{s}");
        assert!(s.contains("23"), "the streak is published: {s}");
        a.habits = None;
        assert!(!Kind::Profile.available(&a));
        a.recap = None;
        assert!(!Kind::Recap.available(&a));
    }

    #[test]
    fn a_poster_is_embedded_and_a_missing_one_is_a_plain_tile() {
        let a = profile();
        let with = svg(Kind::Profile, &a, &Posters::from([("s1".to_string(), vec![0xFF, 0xD8, 0xFF, 0xE0])]));
        assert!(with.contains("data:image/jpeg;base64,/9j/4A=="), "{with}");
        let without = svg(Kind::Profile, &a, &Posters::new());
        assert!(!without.contains("data:image"), "{without}");
    }

    #[test]
    fn numbers_read_the_way_the_page_writes_them() {
        assert_eq!(thousands(1203), "1,203");
        assert_eq!(thousands(999), "999");
        assert_eq!(thousands(1_234_567), "1,234,567");
        assert!(svg(Kind::Profile, &profile(), &Posters::new()).contains("412"));
    }

    #[test]
    fn both_cards_render_to_a_1200_by_630_png() {
        for kind in [Kind::Profile, Kind::Recap] {
            let png = render_png(&svg(kind, &profile(), &Posters::new())).unwrap();
            assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
            let (w, h) = (u32::from_be_bytes(png[16..20].try_into().unwrap()), u32::from_be_bytes(png[20..24].try_into().unwrap()));
            assert_eq!((w, h), (W, H));
        }
    }

    #[test]
    fn text_is_drawn_with_the_bundled_font() {
        // With no font at all resvg drops every glyph silently and still answers a PNG: a card of empty boxes.
        let tree = usvg::Tree::from_str(&svg(Kind::Profile, &profile(), &Posters::new()), &options()).unwrap();
        let mut glyphs = 0;
        fn walk(g: &usvg::Group, n: &mut usize) {
            for c in g.children() {
                match c {
                    usvg::Node::Group(g) => walk(g, n),
                    usvg::Node::Text(t) => *n += t.flattened().children().len(),
                    _ => {}
                }
            }
        }
        walk(tree.root(), &mut glyphs);
        assert!(glyphs > 5, "no text was laid out: {glyphs}");
    }
}
