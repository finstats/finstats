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

/// Every language FinStats speaks; the first is the source, and what nothing else matches.
pub const LOCALES: [&str; 4] = ["en-GB", "en-US", "nb-NO", "nn-NO"];

/// The language of `Accept-Language` that FinStats speaks, by weight then order: a tag as written, else its
/// language (`nb`, `nn`, `no` for Bokmål; `en` is British unless it says America). Nothing usable is the source.
pub fn pick(header: &str) -> &'static str {
    let mut asked: Vec<(f32, usize, &'static str)> = vec![];
    for (i, part) in header.split(',').enumerate() {
        let mut bits = part.split(';');
        let tag = bits.next().unwrap_or("").trim().to_ascii_lowercase();
        let q = bits.filter_map(|b| b.trim().strip_prefix("q=")).next().map_or(Some(1.0), |q| q.trim().parse::<f32>().ok());
        let Some(q) = q.filter(|q| *q > 0.0) else { continue };
        let lang = tag.split('-').next().unwrap_or("");
        let found = LOCALES.iter().find(|l| l.eq_ignore_ascii_case(&tag)).copied().or(match lang {
            "nb" | "no" => Some("nb-NO"),
            "nn" => Some("nn-NO"),
            "en" => Some("en-GB"),
            _ => None,
        });
        if let Some(l) = found { asked.push((q, i, l)); }
    }
    asked.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
    asked.first().map_or(LOCALES[0], |a| a.2)
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

/// The catalogues, the same files the pages read, parsed once.
fn catalogue(lang: &str) -> &'static serde_json::Value {
    static ALL: std::sync::OnceLock<Vec<(&'static str, serde_json::Value)>> = std::sync::OnceLock::new();
    let all = ALL.get_or_init(|| {
        [
            ("en-GB", include_str!("../web/assets/i18n/en-GB.json")),
            ("en-US", include_str!("../web/assets/i18n/en-US.json")),
            ("nb-NO", include_str!("../web/assets/i18n/nb-NO.json")),
            ("nn-NO", include_str!("../web/assets/i18n/nn-NO.json")),
        ]
        .into_iter()
        .map(|(tag, text)| (tag, serde_json::from_str(text).unwrap_or_else(|e| panic!("{tag}.json: {e}"))))
        .collect()
    });
    all.iter().find(|(tag, _)| *tag == lang).map_or(&all[0].1, |(_, v)| v)
}

/// A whole number as `lang` writes it: 1,390 in English, 1 390 (a no-break space) in Norwegian, as `Intl` does.
fn number(lang: &str, n: i64) -> String {
    let digits = n.unsigned_abs().to_string();
    let sep = if lang.starts_with("en") { ',' } else { '\u{a0}' };
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) { out.push(sep); }
        out.push(c);
    }
    if n < 0 { format!("-{out}") } else { out }
}

/// `key` in `lang`, `{name}` filled from `args`; a plural takes its form from `count` (English, Bokmål and Nynorsk say
/// `one` for exactly 1 and `other` for anything else). A key with no text is the key itself.
pub fn say(lang: &str, key: &str, args: &[(&str, Arg)]) -> String {
    let lang = LOCALES.iter().copied().find(|l| *l == lang).unwrap_or(LOCALES[0]);
    let mut v = catalogue(lang);
    for part in key.split('.') {
        match v.get(part) { Some(x) => v = x, None => return key.to_string() }
    }
    let text = match v {
        serde_json::Value::String(s) => s.as_str(),
        serde_json::Value::Object(m) if m.get("other").is_some_and(|o| o.is_string()) => {
            let count = args.iter().find_map(|(n, a)| match a { Arg::Number(c) if *n == "count" => Some(*c), _ => None });
            let form = if count == Some(1) { "one" } else { "other" };
            m.get(form).or(m.get("other")).and_then(|x| x.as_str()).unwrap_or(key)
        }
        _ => return key.to_string(),
    };
    let mut out = text.to_string();
    for (name, arg) in args {
        let value = match arg { Arg::Text(s) => s.clone(), Arg::Number(n) => number(lang, *n) };
        out = out.replace(&format!("{{{name}}}"), &value);
    }
    out
}

/// `key` in the language of the request being answered.
#[allow(dead_code)] // the server's words move onto it next
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

/// Keeps the language a request asked for while it is answered.
pub async fn middleware(req: Request, next: Next) -> Response {
    let lang = pick(req.headers().get(ACCEPT_LANGUAGE).and_then(|v| v.to_str().ok()).unwrap_or(""));
    LANG.scope(lang, next.run(req)).await
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
    fn placeholders_are_filled_and_plurals_follow_the_count() {
        assert_eq!(say("en-GB", "auth.login.sub_on", &[("server", "Jellyfin".into())]), "Use your Jellyfin account on Jellyfin.");
        assert_eq!(say("nn-NO", "time.days_ago", &[("count", 1.into())]), "for 1 dag sidan");
        assert_eq!(say("nn-NO", "time.days_ago", &[("count", 3.into())]), "for 3 dagar sidan");
        assert_eq!(say("en-GB", "statusbar.plays", &[("count", 1390.into())]), "1,390 plays");
        assert_eq!(say("nb-NO", "statusbar.plays", &[("count", 1390.into())]), "1\u{a0}390 avspillinger");
    }

    #[test]
    fn a_key_with_no_text_is_the_key() {
        assert_eq!(say("en-GB", "no.such.key", &[]), "no.such.key");
        assert_eq!(say("en-GB", "nav.groups", &[]), "nav.groups");
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
        assert_eq!(pick("en"), "en-GB");
        assert_eq!(pick("en-AU"), "en-GB", "any English but America's is the source");
        assert_eq!(pick("de-DE,nn;q=0.5"), "nn-NO", "a language FinStats does not speak is passed over");
        assert_eq!(pick("en;q=0.2,nb;q=0.9"), "nb-NO", "the weights decide, not the order");
        assert_eq!(pick("NB-no"), "nb-NO");
    }

    #[test]
    fn nothing_usable_is_british_english() {
        for header in ["", "*", "de", "fr-FR;q=1", ";;,", "nb;q=0"] {
            assert_eq!(pick(header), "en-GB", "{header:?}");
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Map, Value};
    use std::collections::{BTreeMap, BTreeSet};
    use std::path::{Path, PathBuf};

    /// Every catalogue, the source first.
    const LOCALES: [&str; 4] = ["en-GB", "en-US", "nb-NO", "nn-NO"];
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
    fn the_catalogues_are_the_four_languages_and_say_which_they_are() {
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

    #[test]
    fn every_catalogue_has_the_keys_british_english_has() {
        let source = texts(&load("en-GB"));
        assert!(!source.is_empty());
        for (tag, cat) in every_catalogue() {
            let these = texts(&cat);
            let missing: Vec<_> = source.keys().filter(|k| !these.contains_key(*k)).collect();
            let extra: Vec<_> = these.keys().filter(|k| !source.contains_key(*k)).collect();
            assert!(missing.is_empty() && extra.is_empty(), "{tag}: missing {missing:?}, not in en-GB {extra:?}");
            for (k, v) in &source {
                assert_eq!(v.is_object(), these[k].is_object(), "{tag}: {k} is a plural in one catalogue and not in the other");
            }
        }
    }

    #[test]
    fn a_translation_keeps_the_placeholders_of_the_source() {
        let source = texts(&load("en-GB"));
        for (tag, cat) in every_catalogue() {
            for (k, v) in texts(&cat) {
                let want: BTreeSet<String> = strings(&source[&k]).iter().flat_map(|s| placeholders(s)).collect();
                for s in strings(&v) {
                    assert_eq!(placeholders(s), want, "{tag}: {k} = {s:?}");
                }
            }
        }
    }

    #[test]
    fn a_plural_has_the_forms_its_language_has_and_counts_with_count() {
        // English, Bokmål and Nynorsk all have one and other (CLDR); a language with more lists them here.
        let forms = |_tag: &str| BTreeSet::from(["one", "other"]);
        for (tag, cat) in every_catalogue() {
            for (k, v) in texts(&cat) {
                let Value::Object(m) = &v else { continue };
                assert_eq!(m.keys().map(String::as_str).collect::<BTreeSet<_>>(), forms(tag), "{tag}: {k}");
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
            // To license is British too: only the noun differs.
            if american != "license" {
                assert!(!gb.iter().any(|w| w.starts_with(american)), "en-GB says {american}…; FinStats' British writes {british}");
            }
        }
    }
}
