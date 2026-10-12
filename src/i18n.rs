//! The words of the interface, in each language FinStats speaks (`web/assets/i18n/<BCP 47 tag>.json`). British English
//! (`en-GB`) is the source every other catalogue answers to: the same keys, the same placeholders, the plural forms its
//! language has. A key reads where it is used (`nav.dashboard`, `auth.login.title`); a placeholder is `{name}`; a text
//! that depends on a number is an object of the CLDR plural categories that language has (`one`, `other`), picked with
//! `Intl.PluralRules` and filled from `{count}`. `$meta` says which language a file is and what it falls back to. The
//! tests below hold the catalogues to each other.

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
