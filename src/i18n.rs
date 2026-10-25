//! The words of the interface, in each language FinStats speaks (`web/assets/i18n/<BCP 47 tag>.json`). British English
//! (`en-GB`) is the source every other catalogue answers to: the same keys, the same placeholders, the plural forms its
//! language has. A key reads where it is used (`nav.dashboard`, `auth.login.title`); a placeholder is `{name}`; a text
//! that depends on a number is an object of the CLDR plural categories that language has (`one`, `other`), picked with
//! `Intl.PluralRules` and filled from `{count}`. `$meta` says which language a file is and what it falls back to. The
//! tests below hold the catalogues to each other.
//!
//! The page says which language it reads in `Accept-Language` (`api.js` sends the one this browser chose), and
//! `middleware` keeps it for the request (`current()`), so whatever answers in words answers in that one.

use axum::extract::Request;
use axum::http::header::ACCEPT_LANGUAGE;
use axum::middleware::Next;
use axum::response::Response;

/// Every language FinStats speaks; the first is the source.
pub const LOCALES: [&str; 24] = [
    "en-GB", "en-US", "nb-NO", "nn-NO", "nn-NO-synnmore", "da-DK", "de-DE", "es-ES", "fi-FI", "fr-FR", "hi-IN", "is-IS", "it-IT", "ja-JP", "ko-KR",
    "nl-NL", "pl-PL", "pt-BR", "ru-RU", "sv-SE", "tr-TR", "uk-UA", "zh-CN", "zh-TW",
];
/// The languages the owner reads and has checked: complete in every change. The others are translations nobody here
/// can check; they may lag behind, and what one lacks is said in British English until it has it.
#[allow(dead_code)] // read by the tests that hold the catalogues and the changelogs, and by the README's picture
pub const CHECKED: [&str; 5] = ["en-GB", "en-US", "nb-NO", "nn-NO", "nn-NO-synnmore"];
/// What a reader is given when nothing they ask for is a language FinStats speaks, and for English that names no country.
pub const DEFAULT: &str = "en-US";

/// The catalogue a language tag (lower case) asks for, if FinStats speaks that language: the tag as written, else its
/// language (`no` is Bokmål, `en` the default and British where it names another country, `pt` Brazilian, Chinese by its
/// script or region).
fn language_of(tag: &str) -> Option<&'static str> {
    if let Some(l) = LOCALES.iter().find(|l| l.eq_ignore_ascii_case(tag)) { return Some(l) }
    let mut parts = tag.split('-');
    let lang = parts.next().unwrap_or("");
    match lang {
        "nb" | "no" => Some("nb-NO"),
        "nn" => Some("nn-NO"),
        "en" => Some(if parts.next().is_some() { "en-GB" } else { DEFAULT }),
        "zh" => Some(if parts.any(|p| matches!(p, "hant" | "tw" | "hk" | "mo")) { "zh-TW" } else { "zh-CN" }),
        _ => LOCALES.iter().find(|l| l.split('-').next() == Some(lang)).copied(),
    }
}

/// The language of `Accept-Language` that FinStats speaks, by weight then order (`language_of`). Nothing usable is the
/// default.
pub fn pick(header: &str) -> &'static str {
    let mut asked: Vec<(f32, usize, &'static str)> = vec![];
    for (i, part) in header.split(',').enumerate() {
        let mut bits = part.split(';');
        let tag = bits.next().unwrap_or("").trim().to_ascii_lowercase();
        let q = bits.filter_map(|b| b.trim().strip_prefix("q=")).next().map_or(Some(1.0), |q| q.trim().parse::<f32>().ok());
        let Some(q) = q.filter(|q| *q > 0.0) else { continue };
        if let Some(l) = language_of(&tag) { asked.push((q, i, l)); }
    }
    asked.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
    asked.first().map_or(DEFAULT, |a| a.2)
}

/// The plural form a whole number takes in `lang` (CLDR's rules for integers; `Intl.PluralRules` says the same).
pub fn plural(lang: &str, n: i64) -> &'static str {
    let n = n.unsigned_abs();
    let (ten, hundred) = (n % 10, n % 100);
    let few = (2..=4).contains(&ten) && !(12..=14).contains(&hundred);
    match lang.split('-').next().unwrap_or("") {
        "ja" | "ko" | "zh" => "other",
        "fr" | "pt" | "hi" => if n <= 1 { "one" } else { "other" },
        "is" => if ten == 1 && hundred != 11 { "one" } else { "other" },
        "ru" | "uk" => if ten == 1 && hundred != 11 { "one" } else if few { "few" } else { "many" },
        "pl" => if n == 1 { "one" } else if few { "few" } else { "many" },
        _ => if n == 1 { "one" } else { "other" },
    }
}

/// The language a card is drawn in: `lang`, unless its letters are not in the card's font (Inter: Latin, Greek and
/// Cyrillic), when the card is drawn in the source rather than in boxes.
pub fn drawable(lang: &str) -> &'static str {
    match lang.split('-').next().unwrap_or("") {
        "ja" | "ko" | "zh" | "hi" => LOCALES[0],
        _ => LOCALES.iter().copied().find(|l| *l == lang).unwrap_or(LOCALES[0]),
    }
}

/// What fills a placeholder: words as they are, or a number written the language's way (and, as `count`, the one that
/// picks a plural's form).
pub enum Arg {
    Text(String),
    Number(i64),
}
impl From<&str> for Arg { fn from(s: &str) -> Self { Arg::Text(s.to_string()) } }
impl From<String> for Arg { fn from(s: String) -> Self { Arg::Text(s) } }
impl From<&String> for Arg { fn from(s: &String) -> Self { Arg::Text(s.clone()) } }
impl From<i64> for Arg { fn from(n: i64) -> Self { Arg::Number(n) } }
impl From<i32> for Arg { fn from(n: i32) -> Self { Arg::Number(n.into()) } }
impl From<u32> for Arg { fn from(n: u32) -> Self { Arg::Number(n.into()) } }
impl From<u64> for Arg { fn from(n: u64) -> Self { Arg::Number(n as i64) } }
impl From<usize> for Arg { fn from(n: usize) -> Self { Arg::Number(n as i64) } }

/// The catalogues, the same files the pages read (embedded with them), parsed once. A file that is missing or does not
/// parse is an empty catalogue: the tests hold every one to be there and whole.
fn catalogue(lang: &str) -> &'static serde_json::Value {
    static ALL: std::sync::OnceLock<Vec<(&'static str, serde_json::Value)>> = std::sync::OnceLock::new();
    let all = ALL.get_or_init(|| {
        LOCALES.iter().map(|&tag| {
            let file = crate::api::WebAssets::get(&format!("assets/i18n/{tag}.json"));
            (tag, file.and_then(|f| serde_json::from_slice(&f.data).ok()).unwrap_or_else(|| serde_json::json!({})))
        }).collect()
    });
    all.iter().find(|(tag, _)| *tag == lang).map_or(&all[0].1, |(_, v)| v)
}

/// A whole number as `lang` writes it, as `Intl` does: 1,390 in English, 1.390 in German, 1 390 in Norwegian (a no-break
/// space; French a narrow one), Spanish and Polish grouping only from five digits, Hindi in lakhs and crores.
fn number(lang: &str, n: i64) -> String {
    let digits = n.unsigned_abs().to_string();
    let code = lang.split('-').next().unwrap_or("");
    let (sep, from) = match code {
        "de" | "it" | "pt" | "nl" | "da" | "tr" | "is" => ('.', 4),
        "es" => ('.', 5),
        "fr" => ('\u{202f}', 4),
        "nb" | "nn" | "sv" | "fi" | "ru" | "uk" => ('\u{a0}', 4),
        "pl" => ('\u{a0}', 5),
        _ => (',', 4),
    };
    let mut out = String::new();
    if digits.len() >= from {
        let len = digits.len();
        for (i, c) in digits.chars().enumerate() {
            let left = len - i;
            // Indian grouping: the last three digits, then pairs.
            let cut = if code == "hi" { left == 3 || (left > 3 && (left - 3).is_multiple_of(2)) } else { left.is_multiple_of(3) };
            if i > 0 && cut { out.push(sep); }
            out.push(c);
        }
    } else {
        out = digits;
    }
    if n < 0 { format!("-{out}") } else { out }
}

/// `key` in `lang`, `{name}` filled from `args`; a plural takes `plural`'s form for `count`. A language that does not have
/// the text yet says it in British English; a key with no text at all is the key itself.
pub fn say(lang: &str, key: &str, args: &[(&str, Arg)]) -> String {
    let lang = LOCALES.iter().copied().find(|l| *l == lang).unwrap_or(LOCALES[0]);
    if lang == LOCALES[0] { said(&[catalogue(lang)], lang, key, args) } else { said(&[catalogue(lang), catalogue(LOCALES[0])], lang, key, args) }
}

/// `key` from the first of `chain` that has it: the language's own, then the source's (whose rules then apply).
fn said(chain: &[&serde_json::Value], lang: &str, key: &str, args: &[(&str, Arg)]) -> String {
    let found = chain.iter().enumerate().find_map(|(i, cat)| {
        let mut v = *cat;
        for part in key.split('.') { v = v.get(part)?; }
        (v.is_string() || v.get("other").is_some_and(|o| o.is_string())).then_some((i, v))
    });
    let Some((i, v)) = found else { return key.to_string() };
    let lang = if i == 0 { lang } else { LOCALES[0] };
    let text = match v {
        serde_json::Value::String(s) => s.as_str(),
        _ => {
            let count = args.iter().find_map(|(n, a)| match a { Arg::Number(c) if *n == "count" => Some(*c), _ => None });
            let form = count.map_or("other", |c| plural(lang, c));
            v.get(form).or(v.get("other")).and_then(|x| x.as_str()).unwrap_or(key)
        }
    };
    let mut out = text.to_string();
    for (name, arg) in args {
        let value = match arg { Arg::Text(s) => s.clone(), Arg::Number(n) => number(lang, *n) };
        out = out.replace(&format!("{{{name}}}"), &value);
    }
    out
}

/// `key` in the language of the request being answered.
pub fn t(key: &str, args: &[(&str, Arg)]) -> String {
    say(current(), key, args)
}

tokio::task_local! {
    static LANG: &'static str;
}

/// The language of the request being answered; the source outside one.
pub fn current() -> &'static str {
    LANG.try_with(|l| *l).unwrap_or(LOCALES[0])
}

/// `fut` in the language of the request that starts it: work spawned from a request (an import, a task run by hand)
/// says what it says in the words of whoever started it, where a spawned future would otherwise have none.
pub fn carry<F: std::future::Future>(fut: F) -> impl std::future::Future<Output = F::Output> {
    LANG.scope(current(), fut)
}

/// `fut` in `lang`, for a future that runs where the request's language is no longer known (a WebSocket's own task).
pub fn speaking<F: std::future::Future>(lang: &'static str, fut: F) -> impl std::future::Future<Output = F::Output> {
    LANG.scope(lang, fut)
}

/// `f` in `lang`, on a thread with no request around it (the database's): the synchronous half of `carry`.
pub fn within<R>(lang: &'static str, f: impl FnOnce() -> R) -> R {
    LANG.sync_scope(lang, f)
}

/// The language a request asks for: the one chosen in FinStats (the `finstats_lang` cookie the pages keep, which a
/// picture or a download fetched by the browser itself carries too), else `Accept-Language`.
pub fn asked(headers: &axum::http::HeaderMap) -> &'static str {
    let chosen = headers.get_all(axum::http::header::COOKIE).iter().filter_map(|v| v.to_str().ok()).flat_map(|v| v.split(';'))
        .filter_map(|c| c.trim().strip_prefix("finstats_lang=")).find_map(|tag| LOCALES.iter().copied().find(|l| *l == tag));
    chosen.unwrap_or_else(|| pick(headers.get(ACCEPT_LANGUAGE).and_then(|v| v.to_str().ok()).unwrap_or("")))
}

/// Keeps the language a request asked for while it is answered.
pub async fn middleware(req: Request, next: Next) -> Response {
    let lang = asked(req.headers());
    LANG.scope(lang, next.run(req)).await
}

/// The pages hold up their side: every key they ask for has a text, and no English sentence is left in them outside one.
#[cfg(test)]
mod pages_tests {
    use super::*;
    use std::path::{Path, PathBuf};

    fn web() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("web/assets")
    }
    fn js_files(dir: &Path) -> Vec<PathBuf> {
        let mut out = vec![];
        let mut todo = vec![dir.to_path_buf()];
        while let Some(d) = todo.pop() {
            for e in std::fs::read_dir(&d).unwrap().flatten() {
                let p = e.path();
                if p.is_dir() { todo.push(p) } else if p.extension().is_some_and(|x| x == "js") { out.push(p) }
            }
        }
        out.sort();
        out
    }
    /// The interface's code: FinStats' modules and FinUI's components, not FinUI create or its gallery.
    fn interface() -> Vec<(String, String)> {
        let w = web();
        js_files(&w.join("js")).into_iter().chain(js_files(&w.join("finui/components")))
            .map(|p| (p.strip_prefix(&w).unwrap().to_string_lossy().replace('\\', "/"), std::fs::read_to_string(&p).unwrap()))
            .collect()
    }
    /// The code without comments and without FinUI's `meta` (its gallery's documentation, never shown in FinStats).
    fn code(js: &str) -> String {
        // FinUI's gallery: `meta`, and the demo data its pages are drawn with, after the component's last function.
        let js = match js.find("export const meta") { Some(at) => js[..at].rfind("\n}\n").map_or(&js[..at], |end| &js[..end + 3]), None => js };
        let mut out = String::new();
        let mut rest = js;
        while let Some(at) = rest.find("/*") {
            out.push_str(&rest[..at]);
            rest = rest[at..].find("*/").map_or("", |end| &rest[at + end + 2..]);
        }
        out.push_str(rest);
        out.lines().map(|l| match l.find("// ") { Some(at) if !l[..at].ends_with(':') && l[..at].matches('\'').count() % 2 == 0 && l[..at].matches('`').count() % 2 == 0 => &l[..at], _ => l }).collect::<Vec<_>>().join("\n")
    }
    /// Every string literal of a line, with what stands before it on that line.
    fn literals(line: &str) -> Vec<(String, String)> {
        let b = line.as_bytes();
        let mut out = vec![];
        let mut i = 0;
        while i < b.len() {
            let q = b[i];
            if q == b'\'' || q == b'"' || q == b'`' {
                let start = i;
                i += 1;
                let mut s = String::new();
                while i < b.len() && b[i] != q {
                    if b[i] == b'\\' { i += 1; }
                    if i < b.len() { s.push(b[i] as char); }
                    i += 1;
                }
                out.push((line[..start].to_string(), String::from_utf8_lossy(&line.as_bytes()[start + 1..i.min(b.len())]).into_owned()));
            }
            i += 1;
        }
        out
    }
    fn has_text(cat: &serde_json::Value, key: &str) -> bool {
        let mut v = cat;
        for part in key.split('.') {
            match v.get(part) { Some(x) => v = x, None => return false }
        }
        v.is_string() || v.get("other").is_some_and(|o| o.is_string())
    }
    fn has_branch(cat: &serde_json::Value, key: &str) -> bool {
        let mut v = cat;
        for part in key.trim_end_matches('.').split('.') {
            match v.get(part) { Some(x) => v = x, None => return false }
        }
        v.is_object()
    }

    #[test]
    fn every_key_the_pages_ask_for_has_a_text() {
        let cat = catalogue("en-GB");
        let mut missing = vec![];
        for (file, js) in interface() {
            for line in code(&js).lines() {
                for (before, lit) in literals(line) {
                    let b = before.trim_end();
                    let call = ["tParts(", "t(", "tr(", "say("].into_iter().find(|c| b.ends_with(c) && !b[..b.len() - c.len()].ends_with(|x: char| x.is_alphanumeric() || x == '.' || x == '_' || x == '$'));
                    let Some(call) = call else { continue };
                    if lit.contains(char::is_whitespace) || lit.is_empty() || lit.starts_with("${") { continue }
                    let key = if call == "say(" { format!("finui.{lit}") } else { lit.clone() };
                    let ok = match key.split_once("${") {
                        Some((stem, _)) => has_branch(cat, stem) || has_text(cat, stem.trim_end_matches('.')),
                        None => has_text(cat, &key),
                    };
                    if !ok { missing.push(format!("{file}: {key}")); }
                }
            }
        }
        assert!(missing.is_empty(), "keys with no text in en-GB:\n  {}", missing.join("\n  "));
    }

    /// Single words the code compares or sends rather than shows: ids from Jellyfin and the API (item, person and
    /// changelog kinds, play methods), key names, an HTTP header, and names that are the same in every language.
    const IDS: &[&str] = &[
        "Movie", "Series", "Season", "Episode", "Audio", "Video", "Book", "MusicAlbum", "MusicArtist", "TvChannel", "Trailer", "BoxSet", "Folder", "Playlist", "Photo",
        "Actor", "Director", "Writer", "Producer", "Composer", "GuestStar",
        "Added", "Changed", "Performance", "Stability", "Fixed", "Removed",
        "Transcode", "DirectPlay", "DirectStream", "Software", "Hardware", "None", "Other", "Unknown", "Remote", "Local",
        "Escape", "Enter", "Tab", "Home", "End", "Space", "Backspace", "Delete", "PageUp", "PageDown", "ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight", "Control", "Shift", "Meta", "Alt",
        "Active", "Restart", "Deleted", "Superceded", "Malfunctioned", "NotSupported", "Disabled", "Cancelling", "Running", "Idle", "Off",
        "Ctrl", "Esc", "Accept", "Mbps", "kbps", "Jellyfin", "Jellystat", "Streamystats", "Tautulli", "Sonarr", "Radarr", "Seerr", "Plex", "FinStats", "FinUI", "GitHub", "Discord", "Telegram", "Gotify", "Pushover", "Pushbullet",
    ];
    /// What may stay English in the code, each with why: names, and words the code reads rather than shows.
    const ENGLISH_ON_PURPOSE: &[&str] = &[
        // Each language by its own name, in i18n.js's list.
        "js/i18n.js:Norsk Bokmål", "js/i18n.js:Norsk Nynorsk", "js/i18n.js:Synnmørsk", "js/i18n.js:Dansk", "js/i18n.js:Deutsch", "js/i18n.js:Español",
        "js/i18n.js:Suomi", "js/i18n.js:Français", "js/i18n.js:Íslenska", "js/i18n.js:Italiano", "js/i18n.js:Nederlands",
        "js/i18n.js:Polski", "js/i18n.js:Русский", "js/i18n.js:Svenska", "js/i18n.js:Türkçe", "js/i18n.js:Українська",
        "js/i18n.js:Português (Brasil)",
        "js/state.js:not all", // a media query
        // The buttons and menus of Jellystat's, Streamystats' and Tautulli's own (English) pages, named in the import steps.
        "js/settings/import.js:Settings", "js/settings/import.js:Backup", "js/settings/import.js:Activity", "js/settings/import.js:Backups",
        "js/settings/import.js:Actions", "js/settings/import.js:Download",
    ];

    #[test]
    fn no_english_sentence_is_left_in_the_pages() {
        let mut found = vec![];
        for (file, js) in interface() {
            let whole = code(&js);
            let starts: Vec<usize> = std::iter::once(0).chain(whole.match_indices('\n').map(|(i, _)| i + 1)).collect();
            for (n, line) in whole.lines().enumerate() {
                if line.contains("console.") || line.trim_start().starts_with("import ") { continue }
                for (before, lit) in literals(line) {
                    let b = before.trim_end();
                    // FinUI's English beside its key, `say('key', '…'` (the call may run over lines).
                    let upto = &whole[..starts[n] + before.len()];
                    let mut from = upto.len().saturating_sub(400);
                    while !upto.is_char_boundary(from) { from -= 1; }
                    let tail = upto[from..].trim_end();
                    if tail.ends_with(',') && tail.rfind("say(").is_some_and(|at| { let call = &tail[at + 4..tail.len() - 1]; let c = call.trim(); c.len() >= 2 && (c.starts_with('\'') || c.starts_with('`')) && c.ends_with(c.chars().next().unwrap()) && !c[1..c.len() - 1].contains(['\'', '`', '(']) }) { continue }
                    // Words handed to the catalogue (FinUI's English fallback beside its key, a fallback for an id) are its.
                    if ["t(", "tParts(", "tr(", "say(", "tOr("].iter().any(|c| b.ends_with(c) && !b[..b.len() - c.len()].ends_with(|x: char| x.is_alphanumeric() || x == '.' || x == '_' || x == '$')) { continue }
                    if b.contains("say(") && b.ends_with(',') && b.matches("say(").count() > b.matches(')').count() { continue }
                    // Classes, selectors and attributes are not words.
                    if b.ends_with("class:") || b.ends_with("className =") || b.contains("querySelector") || b.contains("classList") || b.ends_with("closest(") || b.ends_with("matches(") || b.ends_with("rel:") || b.ends_with("noopener") || b.ends_with('?') && b.contains("rel:") || b.ends_with("Error(") { continue }
                    if IDS.contains(&lit.trim()) { continue }
                    // Addresses, selectors and CSS values are not words.
                    if lit.starts_with('/') || (lit.contains('[') && lit.contains(']')) || (lit.ends_with(')') && lit.split('(').next().is_some_and(|f| !f.is_empty() && f.chars().all(|c| c.is_ascii_lowercase() || c == '-'))) { continue }
                    // What a template says once its ${…} (however nested) are taken out.
                    let bare: String = {
                        let (mut out, mut depth, mut chars) = (String::new(), 0, lit.chars().peekable());
                        while let Some(c) = chars.next() {
                            if c == '$' && chars.peek() == Some(&'{') { chars.next(); depth += 1; out.push(' '); }
                            else if depth > 0 && c == '{' { depth += 1 }
                            else if depth > 0 && c == '}' { depth -= 1 }
                            else if depth == 0 { out.push(c) }
                        }
                        out
                    };
                    let words: Vec<&str> = bare.split_whitespace().filter(|w| !IDS.contains(w)).collect();
                    let worded = |w: &&str| w.chars().all(|c| c.is_alphabetic() || "’'.,:?!…".contains(c)) && w.chars().any(char::is_alphabetic) && !w.trim_end_matches(['.', ',', ':', '?', '!', '…']).contains('.');
                    let prose = words.len() >= 2 && words.iter().filter(|w| worded(w)).count() >= 2 && words.iter().any(|w| w.len() >= 3 && w.chars().next().is_some_and(char::is_lowercase) && worded(w));
                    let label = words.len() == 1 && worded(&words[0]) && words[0].len() >= 3 && words[0].chars().next().is_some_and(char::is_uppercase) && words[0].chars().skip(1).all(|c| c.is_lowercase() || "’'.:…".contains(c));
                    if (prose || label) && !ENGLISH_ON_PURPOSE.contains(&format!("{file}:{lit}").as_str()) {
                        found.push(format!("{file}:{}: {lit}", n + 1));
                    }
                }
            }
        }
        assert!(found.is_empty(), "{} English text(s) outside the catalogue (t(), tParts(), or FinUI's say()):\n  {}", found.len(), found.join("\n  "));
    }
}

#[cfg(test)]
mod say_tests {
    use super::*;

    #[test]
    fn a_key_is_its_text_in_the_language_asked_for() {
        assert_eq!(say("en-GB", "nav.dashboard", &[]), "Dashboard");
        assert_eq!(say("nb-NO", "nav.dashboard", &[]), "Oversikt");
        assert_eq!(say("nn-NO", "nav.my_profile", &[]), "Profilen min");
        assert_eq!(say("de-DE", "nav.dashboard", &[]), "Dashboard", "a language FinStats does not speak is the source");
    }

    #[test]
    fn norwegian_says_played_as_one_word_after_the_titles() {
        assert_eq!(say("nb-NO", "widgets.tiles.distinct_titles", &[]), "Ulike titler avspilt");
        assert_eq!(say("nn-NO", "widgets.tiles.distinct_titles", &[]), "Ulike titlar avspela");
        assert_eq!(say("nn-NO-synnmore", "widgets.tiles.distinct_titles", &[]), "Ulike titla avspela");
    }

    #[test]
    fn placeholders_are_filled_and_plurals_follow_the_count() {
        assert_eq!(say("en-GB", "auth.login.sub_on", &[("server", "Jellyfin".into())]), "Use your Jellyfin account on Jellyfin.");
        assert_eq!(say("nn-NO", "time.days_ago", &[("count", 1.into())]), "for 1 dag sidan");
        assert_eq!(say("nn-NO", "time.days_ago", &[("count", 3.into())]), "for 3 dagar sidan");
        assert_eq!(say("en-GB", "statusbar.plays", &[("count", 1390.into())]), "1,390 plays");
        assert_eq!(say("nb-NO", "statusbar.plays", &[("count", 1390.into())]), "1\u{a0}390 avspillinger");
    }

    #[test]
    fn the_shared_errors_speak_the_language_of_the_request() {
        use crate::state::ApiError;
        let rt = tokio::runtime::Builder::new_current_thread().build().unwrap();
        let (play, odd, perm, carried) = rt.block_on(LANG.scope("nb-NO", async {
            let carried = tokio::task::LocalSet::new().run_until(async { tokio::task::spawn_local(carry(async { current() })).await.unwrap() }).await;
            (ApiError::not_found("Play").1, ApiError::not_found("Gizmo").1, ApiError::not_permitted("see the server").1, carried)
        }));
        assert_eq!(play, "Fant ikke avspillingen");
        assert_eq!(odd, "Fant ikke Gizmo", "a thing with no sentence of its own is named in the general one");
        assert_eq!(perm, "Du har ikke tillatelse til å se serveren. En Jellyfin-administrator kan gi den under Innstillinger.");
        assert_eq!(carried, "nb-NO");
        assert_eq!(ApiError::not_found("Film or show").1, "Film or show not found", "outside a request: the source");
    }

    #[test]
    fn work_on_the_database_thread_keeps_the_request_s_language() {
        let rt = tokio::runtime::Builder::new_multi_thread().worker_threads(1).build().unwrap();
        let said = rt.block_on(LANG.scope("nn-NO", async {
            let db = crate::db::Db::open_in_memory().unwrap();
            db.call(|_| Ok(crate::state::ApiError::not_found("Play").1)).await.unwrap()
        }));
        assert_eq!(said, "Fann ikkje avspelinga");
    }

    #[test]
    fn a_key_with_no_text_is_the_key() {
        assert_eq!(say("en-GB", "no.such.key", &[]), "no.such.key");
        assert_eq!(say("en-GB", "nav.groups", &[]), "nav.groups");
    }
}

#[cfg(test)]
mod more_languages_tests {
    use super::*;

    #[test]
    fn every_language_finds_its_catalogue_from_what_a_browser_says() {
        for (asked, got) in [("de-DE", "de-DE"), ("de-AT", "de-DE"), ("de", "de-DE"), ("fr-CA", "fr-FR"), ("es-MX", "es-ES"), ("pt", "pt-BR"),
            ("pt-PT", "pt-BR"), ("it", "it-IT"), ("nl-BE", "nl-NL"), ("sv", "sv-SE"), ("da", "da-DK"), ("fi", "fi-FI"), ("is", "is-IS"),
            ("pl", "pl-PL"), ("ru", "ru-RU"), ("uk", "uk-UA"), ("tr", "tr-TR"), ("ja", "ja-JP"), ("ko", "ko-KR"), ("hi", "hi-IN"),
            ("zh", "zh-CN"), ("zh-CN", "zh-CN"), ("zh-Hans-CN", "zh-CN"), ("zh-SG", "zh-CN"), ("zh-TW", "zh-TW"), ("zh-HK", "zh-TW"),
            ("zh-Hant", "zh-TW"), ("zh-Hant-HK", "zh-TW"), ("no", "nb-NO"), ("en-US", "en-US"), ("en", "en-US"), ("en-AU", "en-GB")] {
            assert_eq!(pick(asked), got, "{asked}");
        }
    }

    #[test]
    fn a_count_takes_the_plural_form_its_language_gives_it() {
        let forms = |lang: &str, ns: &[i64]| ns.iter().map(|&n| plural(lang, n)).collect::<Vec<_>>();
        assert_eq!(forms("en-GB", &[0, 1, 2]), ["other", "one", "other"]);
        assert_eq!(forms("fr-FR", &[0, 1, 2]), ["one", "one", "other"]);
        assert_eq!(forms("pt-BR", &[0, 1, 2]), ["one", "one", "other"]);
        assert_eq!(forms("hi-IN", &[0, 1, 2]), ["one", "one", "other"]);
        assert_eq!(forms("is-IS", &[1, 11, 21, 2]), ["one", "other", "one", "other"]);
        assert_eq!(forms("ru-RU", &[1, 2, 5, 11, 21, 22, 25, 112]), ["one", "few", "many", "many", "one", "few", "many", "many"]);
        assert_eq!(forms("uk-UA", &[1, 3, 0, 14]), ["one", "few", "many", "many"]);
        assert_eq!(forms("pl-PL", &[1, 2, 5, 12, 22, 21]), ["one", "few", "many", "many", "few", "many"]);
        assert_eq!(forms("ja-JP", &[0, 1, 2]), ["other", "other", "other"]);
        assert_eq!(forms("zh-TW", &[1]), ["other"]);
    }

    #[test]
    fn a_text_a_language_has_not_yet_falls_back_to_british_english() {
        let source: serde_json::Value = serde_json::json!({ "a": { "b": "Hello {name}" }, "n": { "one": "{count} play", "other": "{count} plays" } });
        let partial: serde_json::Value = serde_json::json!({ "a": {} });
        assert_eq!(said(&[&partial, &source], "fr-FR", "a.b", &[("name", "Ada".into())]), "Hello Ada");
        assert_eq!(said(&[&partial, &source], "fr-FR", "n", &[("count", 2.into())]), "2 plays");
        assert_eq!(said(&[&partial, &source], "fr-FR", "x.y", &[]), "x.y");
    }

    #[test]
    fn numbers_are_written_as_each_language_writes_them() {
        assert_eq!(number("de-DE", 1_390), "1.390");
        assert_eq!(number("fr-FR", 1_390), "1\u{202f}390");
        assert_eq!(number("es-ES", 1_390), "1390", "Spanish groups from five digits");
        assert_eq!(number("es-ES", 12_390), "12.390");
        assert_eq!(number("pl-PL", 1_390), "1390");
        assert_eq!(number("ru-RU", 1_390), "1\u{a0}390");
        assert_eq!(number("ja-JP", 1_390), "1,390");
        assert_eq!(number("hi-IN", 12_345_678), "1,23,45,678", "Indian grouping");
    }

    #[test]
    fn a_card_is_drawn_in_a_language_its_font_has_letters_for() {
        for l in ["ja-JP", "ko-KR", "zh-CN", "zh-TW", "hi-IN"] { assert_eq!(drawable(l), "en-GB", "{l}"); }
        for l in ["ru-RU", "uk-UA", "is-IS", "pl-PL", "tr-TR", "nb-NO"] { assert_eq!(drawable(l), l); }
    }
}

#[cfg(test)]
mod pick_tests {
    use super::*;

    #[test]
    fn the_first_language_asked_for_that_finstats_speaks_is_the_one() {
        assert_eq!(pick("nb-NO,nb;q=0.9,en;q=0.8"), "nb-NO");
        assert_eq!(pick("nn"), "nn-NO");
        assert_eq!(pick("no"), "nb-NO", "Norwegian without saying which is Bokmål, as browsers mean it");
        assert_eq!(pick("en-US,en;q=0.9"), "en-US");
        assert_eq!(pick("en"), "en-US", "English without saying which is the default");
        assert_eq!(pick("en-AU"), "en-GB", "an English that names a country other than America is British");
        assert_eq!(pick("eo,nn;q=0.5"), "nn-NO", "a language FinStats does not speak is passed over");
        assert_eq!(pick("en;q=0.2,nb;q=0.9"), "nb-NO", "the weights decide, not the order");
        assert_eq!(pick("NB-no"), "nb-NO");
    }

    #[test]
    fn the_language_chosen_in_finstats_beats_the_browser_s() {
        use axum::http::{HeaderMap, HeaderValue};
        let h = |accept: &str, cookie: Option<&str>| {
            let mut m = HeaderMap::new();
            m.insert(ACCEPT_LANGUAGE, HeaderValue::from_str(accept).unwrap());
            if let Some(c) = cookie { m.insert(axum::http::header::COOKIE, HeaderValue::from_str(c).unwrap()); }
            m
        };
        // A picture or a download is fetched by the browser itself, with the browser's own languages.
        assert_eq!(asked(&h("en-US,en;q=0.9", Some("finstats_session=x; finstats_lang=nn-NO"))), "nn-NO");
        assert_eq!(asked(&h("en-US", None)), "en-US");
        assert_eq!(asked(&h("nb", Some("finstats_lang=xx-YY"))), "nb-NO", "a cookie naming a language FinStats does not speak is ignored");
        assert_eq!(asked(&HeaderMap::new()), "en-US");
    }

    #[test]
    fn synnmorsk_is_its_own_checked_language_that_only_its_own_tag_asks_for() {
        assert_eq!(pick("nn-NO-synnmore"), "nn-NO-synnmore");
        assert_eq!(pick("nn-no-synnmore,nn;q=0.8"), "nn-NO-synnmore");
        assert_eq!(pick("nn"), "nn-NO", "Nynorsk without saying which is Nynorsk, never the dialect");
        assert_eq!(pick("nn-NO"), "nn-NO");
        assert!(CHECKED.contains(&"nn-NO-synnmore"), "the owner reads it, so it is whole in every change");
        assert_eq!((plural("nn-NO-synnmore", 1), plural("nn-NO-synnmore", 2)), ("one", "other"));
        assert_eq!(number("nn-NO-synnmore", 12_345), "12\u{a0}345", "numbers as Norwegian writes them");
    }

    #[test]
    fn nothing_usable_is_american_english() {
        for header in ["", "*", "eo", "ar-EG;q=1", ";;,", "nb;q=0"] {
            assert_eq!(pick(header), "en-US", "{header:?}");
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Map, Value};
    use std::collections::{BTreeMap, BTreeSet};
    use std::path::{Path, PathBuf};

    use super::{CHECKED, LOCALES};
    /// CLDR's plural categories; an object whose keys are only these is one text in its forms.
    const CATEGORIES: [&str; 6] = ["zero", "one", "two", "few", "many", "other"];

    fn dir() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("web/assets/i18n")
    }
    fn load(tag: &str) -> Value {
        let p = dir().join(format!("{tag}.json"));
        let text = std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
        serde_json::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
    }
    fn is_plural(m: &Map<String, Value>) -> bool {
        m.contains_key("other") && m.keys().all(|k| CATEGORIES.contains(&k.as_str()))
    }
    /// Every text by its dotted key; a plural is one text whose forms are its value. `$meta` is not a text.
    fn texts(v: &Value) -> BTreeMap<String, Value> {
        fn walk(prefix: &str, v: &Value, out: &mut BTreeMap<String, Value>) {
            match v {
                Value::Object(m) if !is_plural(m) => {
                    for (k, x) in m {
                        if prefix.is_empty() && k == "$meta" { continue; }
                        walk(&if prefix.is_empty() { k.clone() } else { format!("{prefix}.{k}") }, x, out);
                    }
                }
                _ => { out.insert(prefix.to_string(), v.clone()); }
            }
        }
        let mut out = BTreeMap::new();
        walk("", v, &mut out);
        out
    }
    /// The strings of one text: itself, or each of its plural forms.
    fn strings(v: &Value) -> Vec<&str> {
        match v {
            Value::String(s) => vec![s.as_str()],
            Value::Object(m) => m.values().filter_map(Value::as_str).collect(),
            _ => vec![],
        }
    }
    /// `{name}` → name.
    fn placeholders(s: &str) -> BTreeSet<String> {
        let mut out = BTreeSet::new();
        let mut rest = s;
        while let Some(open) = rest.find('{') {
            let after = &rest[open + 1..];
            let Some(close) = after.find('}') else { break };
            let name = &after[..close];
            if !name.is_empty() && name.chars().all(|c| c.is_ascii_lowercase() || c == '_') { out.insert(name.to_string()); }
            rest = &after[close + 1..];
        }
        out
    }
    fn every_catalogue() -> Vec<(&'static str, Value)> {
        LOCALES.iter().map(|t| (*t, load(t))).collect()
    }

    #[test]
    fn the_catalogues_are_the_languages_finstats_speaks_and_say_which_they_are() {
        let on_disk: BTreeSet<String> = std::fs::read_dir(dir()).expect("web/assets/i18n").filter_map(|e| {
            let name = e.ok()?.file_name().into_string().ok()?;
            name.strip_suffix(".json").map(str::to_string)
        }).collect();
        assert_eq!(on_disk, LOCALES.iter().map(|t| t.to_string()).collect(), "a catalogue is missing, or one is not in LOCALES");
        for (tag, cat) in every_catalogue() {
            let meta = &cat["$meta"];
            assert_eq!(meta["tag"], tag, "{tag}.json names another language");
            assert!(meta["name"].as_str().is_some_and(|n| !n.is_empty()), "{tag}: $meta.name is the language in its own words");
            if tag == "en-GB" {
                assert!(meta.get("fallback").is_none(), "en-GB is the source; it falls back to nothing");
            } else {
                assert_eq!(meta["fallback"], "en-GB", "{tag} falls back to the source");
            }
        }
    }

    /// The languages the owner checks have every text British English has; the others may lag behind (what they lack is
    /// said in British English, and the README shows how far each has come), but none has a text the source does not.
    #[test]
    fn every_catalogue_has_the_keys_british_english_has() {
        let source = texts(&load("en-GB"));
        assert!(!source.is_empty());
        for (tag, cat) in every_catalogue() {
            let these = texts(&cat);
            let missing: Vec<_> = source.keys().filter(|k| !these.contains_key(*k)).collect();
            let extra: Vec<_> = these.keys().filter(|k| !source.contains_key(*k)).collect();
            assert!(extra.is_empty(), "{tag}: not in en-GB {extra:?}");
            assert!(missing.is_empty() || !CHECKED.contains(&tag), "{tag} is checked and must be whole: missing {missing:?}");
            for (k, v) in &these {
                assert_eq!(v.is_object(), source[k].is_object(), "{tag}: {k} is a plural in one catalogue and not in the other");
            }
        }
    }

    #[test]
    fn a_translation_keeps_the_placeholders_of_the_source() {
        let source = texts(&load("en-GB"));
        for (tag, cat) in every_catalogue() {
            for (k, v) in texts(&cat) {
                let Some(src) = source.get(&k) else { continue };
                let want: BTreeSet<String> = strings(src).iter().flat_map(|s| placeholders(s)).collect();
                for s in strings(&v) {
                    assert_eq!(placeholders(s), want, "{tag}: {k} = {s:?}");
                }
            }
        }
    }

    #[test]
    fn a_plural_has_the_forms_its_language_has_and_counts_with_count() {
        // The forms a whole number takes in each language (CLDR; `i18n::plural` picks among them), and the one some
        // languages also have for round millions, which a catalogue may give.
        let forms = |tag: &str| -> (BTreeSet<&str>, BTreeSet<&str>) {
            let need = match tag.split('-').next().unwrap() {
                "pl" | "ru" | "uk" => BTreeSet::from(["one", "few", "many", "other"]),
                "ja" | "ko" | "zh" => BTreeSet::from(["other"]),
                _ => BTreeSet::from(["one", "other"]),
            };
            let mut may = need.clone();
            if matches!(tag.split('-').next().unwrap(), "es" | "fr" | "it" | "pt") { may.insert("many"); }
            (need, may)
        };
        for (tag, cat) in every_catalogue() {
            let (need, may) = forms(tag);
            for (k, v) in texts(&cat) {
                let Value::Object(m) = &v else { continue };
                let has: BTreeSet<&str> = m.keys().map(String::as_str).collect();
                assert!(need.is_subset(&has) && has.is_subset(&may), "{tag}: {k} has {has:?}, its language needs {need:?}");
                assert!(m["other"].as_str().is_some_and(|s| s.contains("{count}")), "{tag}: {k} other says how many");
            }
        }
    }

    #[test]
    fn every_text_is_words_without_an_em_dash() {
        for (tag, cat) in every_catalogue() {
            for (k, v) in texts(&cat) {
                let ss = strings(&v);
                assert!(!ss.is_empty(), "{tag}: {k} is not a text");
                for s in ss {
                    assert!(!s.trim().is_empty(), "{tag}: {k} is empty");
                    assert!(!s.contains('\u{2014}'), "{tag}: {k} has an em-dash: {s:?}");
                    assert_eq!(s, s.trim(), "{tag}: {k} has space at an end");
                }
            }
        }
    }

    #[test]
    fn each_english_is_spelled_its_own_way() {
        // (British, American): a word of one found in the other's catalogue is the other's spelling leaking in.
        const PAIRS: [(&str, &str); 10] = [("colour", "color"), ("favourite", "favorite"), ("licence", "license"), ("recognise", "recognize"),
            ("organise", "organize"), ("cancelled", "canceled"), ("kilometre", "kilometer"), ("centre", "center"), ("grey", "gray"), ("film", "movie")];
        let words = |tag: &str| -> BTreeSet<String> {
            texts(&load(tag)).values().flat_map(|v| strings(v).into_iter().map(str::to_string).collect::<Vec<_>>())
                .flat_map(|s| s.split(|c: char| !c.is_alphabetic()).map(str::to_lowercase).collect::<Vec<_>>()).collect()
        };
        let (gb, us) = (words("en-GB"), words("en-US"));
        for (british, american) in PAIRS {
            assert!(!us.iter().any(|w| w.starts_with(british)), "en-US says {british}…; Americans write {american}");
            // To license is British too (only the noun differs), and British English says movie as well as film.
            if american != "license" && american != "movie" {
                assert!(!gb.iter().any(|w| w.starts_with(american)), "en-GB says {american}…; FinStats' British writes {british}");
            }
        }
    }
}
