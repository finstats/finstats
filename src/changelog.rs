//! Patch notes. `CHANGELOG.md` is the source: it is compiled into the binary and parsed into the structure the
//! "Patch notes" tab renders. Each other language FinStats speaks may have its own copy (`changelog/<tag>.md`, embedded),
//! the same releases, groups and bullets in its words, and the tab reads the one the page asked in (`i18n::current`).
//! The languages the owner checks (`i18n::CHECKED`) tell every release; another may lag behind, and a release it has not
//! told yet is shown as the source tells it. The headings (`## [x.y.z] - date`, `### Added`) stay as the source writes
//! them: they are structure, not words.

use std::collections::HashMap;

use axum::Json;
use rust_embed::RustEmbed;
use serde::Serialize;
use serde_json::{Value, json};

use crate::auth::AuthUser;

const SOURCE: &str = include_str!("../CHANGELOG.md");

/// The source in the other languages FinStats speaks.
#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/changelog"]
struct Translations;

/// The source told in `lang`, if it has been.
fn translation(lang: &str) -> Option<String> {
    Translations::get(&format!("{lang}.md")).map(|f| String::from_utf8_lossy(&f.data).into_owned())
}

/// The releases in `lang`: each as the language tells it, the source's where it does not (yet), in the source's order.
fn told(source: Vec<Release>, translated: Vec<Release>) -> Vec<Release> {
    let mut by_version: HashMap<String, Release> = translated.into_iter().map(|r| (r.version.clone(), r)).collect();
    source.into_iter().map(|r| by_version.remove(&r.version).unwrap_or(r)).collect()
}

/// The Patch notes in `lang`.
fn releases_for(lang: &str) -> Vec<Release> {
    match translation(lang) {
        Some(text) if lang != crate::i18n::LOCALES[0] => told(parse(SOURCE), parse(&text)),
        _ => parse(SOURCE),
    }
}

#[derive(Debug, Serialize, PartialEq)]
pub struct Release {
    pub version: String,
    /// The day it was released.
    pub date: Option<String>,
    /// The first day of work, for a release made over several (`## [x.y.z] - first to last`).
    pub started: Option<String>,
    pub summary: Option<String>,
    pub groups: Vec<Group>,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct Group {
    /// "Added", "Changed", "Fixed", "Removed"…
    pub kind: String,
    pub items: Vec<String>,
}

/// Reads `## [version] - date` (or `- first to last`) sections with `### Kind` groups of `- ` bullets.
/// Anything before the first release heading (the file's own introduction) is ignored.
pub fn parse(source: &str) -> Vec<Release> {
    let mut releases: Vec<Release> = vec![];
    for line in source.lines() {
        let trimmed = line.trim_end();
        if let Some(heading) = trimmed.strip_prefix("## ") {
            let (version, date) = match heading.split_once(" - ") {
                Some((v, d)) => (v, Some(d.trim())),
                None => (heading, None),
            };
            let (started, date) = match date.and_then(|d| d.split_once(" to ")) {
                Some((first, last)) => (Some(first.trim().to_string()), Some(last.trim().to_string())),
                None => (None, date.map(str::to_string)),
            };
            let version = version.trim().trim_start_matches('[').trim_end_matches(']').to_string();
            releases.push(Release { version, date, started, summary: None, groups: vec![] });
            continue;
        }
        let Some(release) = releases.last_mut() else { continue };
        if let Some(kind) = trimmed.strip_prefix("### ") {
            release.groups.push(Group { kind: kind.trim().to_string(), items: vec![] });
        } else if let Some(item) = trimmed.trim_start().strip_prefix("- ") {
            match release.groups.last_mut() {
                Some(group) => group.items.push(item.trim().to_string()),
                None => release.groups.push(Group { kind: "Changed".into(), items: vec![item.trim().to_string()] }),
            }
        } else if !trimmed.trim().is_empty() {
            let text = trimmed.trim();
            match release.groups.last_mut().and_then(|g| g.items.last_mut()) {
                // A wrapped bullet continues on the next line.
                Some(item) => {
                    item.push(' ');
                    item.push_str(text);
                }
                None => match &mut release.summary {
                    Some(s) => {
                        s.push(' ');
                        s.push_str(text);
                    }
                    None => release.summary = Some(text.to_string()),
                },
            }
        }
    }
    releases
}

pub async fn changelog(_user: AuthUser) -> Json<Value> {
    Json(json!({ "current": env!("CARGO_PKG_VERSION"), "releases": releases_for(crate::i18n::current()) }))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The groups a release may use; the Patch notes page gives each its own icon.
    const KINDS: [&str; 7] = ["Added", "Changed", "Security", "Performance", "Stability", "Fixed", "Removed"];

    fn is_day(d: &str) -> bool {
        d.len() == 10 && d.char_indices().all(|(i, c)| if i == 4 || i == 7 { c == '-' } else { c.is_ascii_digit() })
    }

    /// Everything wrong with a parsed changelog, one sentence each.
    fn problems(releases: &[Release]) -> Vec<String> {
        let mut out = vec![];
        for r in releases {
            let v = &r.version;
            if !(v.split('.').count() == 3 && v.split('.').all(|p| p.parse::<u32>().is_ok())) {
                out.push(format!("bad version heading: {v}"));
            }
            let days = [r.started.as_deref(), r.date.as_deref()];
            if r.date.is_none() || !days.iter().flatten().all(|d| is_day(d)) {
                out.push(format!("release {v} needs a YYYY-MM-DD date, or two joined by \" to \""));
            } else if r.started.as_deref().is_some_and(|s| Some(s) > r.date.as_deref()) {
                out.push(format!("release {v} ends before it starts"));
            }
            if !r.groups.iter().any(|g| !g.items.is_empty()) {
                out.push(format!("release {v} has no notes"));
            }
            for g in r.groups.iter().filter(|g| !KINDS.contains(&g.kind.as_str())) {
                out.push(format!("unknown group {:?} in {v}", g.kind));
            }
            // The app folds releases by minor series and headlines each fold with its x.y.0 summary.
            if v.ends_with(".0") && !r.summary.as_deref().is_some_and(|s| s.len() >= 8) {
                out.push(format!("release {v} opens a series and needs a one-sentence summary: it is that series' headline in Patch notes"));
            }
        }
        out
    }

    #[test]
    fn parses_releases_groups_and_wrapped_bullets() {
        let r = parse("# Notes\nintro is ignored\n\n## [1.1.0] - 2026-01-02\n\nA summary\nover two lines.\n\n### Added\n- one\n- two that\n  wraps\n\n### Fixed\n- three\n\n## [1.0.0]\n- loose bullet\n");
        assert_eq!(r.len(), 2);
        assert_eq!((r[0].version.as_str(), r[0].date.as_deref()), ("1.1.0", Some("2026-01-02")));
        assert_eq!(r[0].summary.as_deref(), Some("A summary over two lines."));
        assert_eq!(r[0].groups[0], Group { kind: "Added".into(), items: vec!["one".into(), "two that wraps".into()] });
        assert_eq!(r[0].groups[1].items, vec!["three"]);
        assert_eq!((r[1].date.as_deref(), r[1].groups[0].kind.as_str()), (None, "Changed"));
    }

    /// A release worked on over several days says so; `date` stays the day it was released.
    #[test]
    fn a_release_may_span_several_days() {
        let r = parse("## [2.0.0] - 2026-09-25 to 2026-09-28\n### Added\n- one\n\n## [1.0.0] - 2026-01-02\n### Added\n- two\n");
        assert_eq!((r[0].started.as_deref(), r[0].date.as_deref()), (Some("2026-09-25"), Some("2026-09-28")));
        assert_eq!((r[1].started.as_deref(), r[1].date.as_deref()), (None, Some("2026-01-02")));
    }

    /// A speed-up is not a fix and neither is a crash that can no longer happen: each has a group
    /// of its own. A range is two real days, the first no later than the last.
    #[test]
    fn the_rules_know_performance_and_stability_and_read_a_range() {
        let ok = "## [2.0.0] - 2026-09-25 to 2026-09-28\nTitle line\n### Performance\n- quicker\n### Stability\n- steadier\n";
        assert_eq!(problems(&parse(ok)), Vec::<String>::new());
        assert_eq!(problems(&parse("## [2.0.1] - 2026-09-28 to 2026-09-25\n### Fixed\n- one\n")), vec!["release 2.0.1 ends before it starts"]);
        assert_eq!(problems(&parse("## [2.0.1] - 2026-9-25 to 2026-09-28\n### Fixed\n- one\n")), vec!["release 2.0.1 needs a YYYY-MM-DD date, or two joined by \" to \""]);
        assert_eq!(problems(&parse("## [2.0.1] - 2026-09-28\n### Speed\n- one\n")), vec!["unknown group \"Speed\" in 2.0.1"]);
    }

    /// Something somebody could have misused is a group of its own, not a fix that starts with "Security:".
    #[test]
    fn security_is_a_group_of_its_own() {
        assert_eq!(problems(&parse("## [2.3.1] - 2026-10-26\n### Security\n- closed\n")), Vec::<String>::new());
    }

    /// The shipped file must be well-formed and must describe the version being built,
    /// so a release can't go out without its notes.
    #[test]
    fn the_shipped_changelog_covers_this_version() {
        let releases = parse(SOURCE);
        assert_eq!(releases.first().map(|r| r.version.as_str()), Some(env!("CARGO_PKG_VERSION")), "add a CHANGELOG.md entry for this version");
        assert_eq!(problems(&releases), Vec::<String>::new());
    }

    /// `code` spans, which are names (an endpoint, a setting, a file) and never translated.
    fn code_spans(s: &str) -> Vec<&str> {
        s.split('`').skip(1).step_by(2).collect()
    }

    /// A translation tells the source's releases, bullet for bullet: for every release it tells, the same day, groups
    /// and number of notes, a title where the source has one, the same `code` and as much **bold**, and no release the
    /// source does not have. The languages the owner checks tell every release: a release cannot ship in one of them and
    /// not the others.
    #[test]
    fn every_translation_tells_the_releases_the_source_tells() {
        let source = parse(SOURCE);
        let by_version: HashMap<&str, &Release> = source.iter().map(|r| (r.version.as_str(), r)).collect();
        for tag in crate::i18n::LOCALES.iter().skip(1) {
            let Some(text) = translation(tag) else {
                assert!(!crate::i18n::CHECKED.contains(tag), "{tag} is checked and has no changelog/{tag}.md");
                continue;
            };
            let these = parse(&text);
            if crate::i18n::CHECKED.contains(tag) {
                assert_eq!(these.iter().map(|r| &r.version).collect::<Vec<_>>(), source.iter().map(|r| &r.version).collect::<Vec<_>>(), "{tag}: not the same releases");
            }
            for t in &these {
                let v = &t.version;
                let Some(s) = by_version.get(v.as_str()) else { panic!("{tag}: a release {v} the source does not have") };
                assert_eq!((&t.date, &t.started), (&s.date, &s.started), "{tag} {v}: another day");
                assert_eq!(t.summary.is_some(), s.summary.is_some(), "{tag} {v}: a title where the source has none, or none where it has one");
                assert_eq!(t.groups.iter().map(|g| (&g.kind, g.items.len())).collect::<Vec<_>>(), s.groups.iter().map(|g| (&g.kind, g.items.len())).collect::<Vec<_>>(), "{tag} {v}: other groups or another number of notes");
                for (sg, tg) in s.groups.iter().zip(&t.groups) {
                    for (si, ti) in sg.items.iter().zip(&tg.items) {
                        assert_eq!(code_spans(ti), code_spans(si), "{tag} {v}: `code` is a name and stays as it is\n  source: {si}\n  {tag}: {ti}");
                        assert_eq!(ti.matches("**").count(), si.matches("**").count(), "{tag} {v}: bold where the source has none\n  source: {si}\n  {tag}: {ti}");
                    }
                }
                for line in t.summary.iter().chain(t.groups.iter().flat_map(|g| &g.items)) {
                    assert!(!line.contains('\u{2014}'), "{tag} {v}: an em-dash: {line}");
                }
            }
        }
    }

    #[test]
    fn the_changelog_is_read_in_the_language_asked_for() {
        assert_eq!(releases_for("en-GB"), parse(SOURCE));
        assert_eq!(releases_for("xx-YY"), parse(SOURCE), "a language with no translation reads the source");
        assert_eq!(releases_for("nb-NO"), parse(&translation("nb-NO").unwrap()));
    }

    #[test]
    fn a_release_a_language_has_not_told_yet_is_the_source_s() {
        let source = parse("## [1.1.0] - 2026-01-02\nTitle\n### Added\n- new\n\n## [1.0.0] - 2026-01-01\n### Fixed\n- old\n");
        let partial = parse("## [1.0.0] - 2026-01-01\n### Fixed\n- gammel\n");
        let r = told(source, partial);
        assert_eq!(r.iter().map(|x| (x.version.as_str(), x.groups[0].items[0].as_str())).collect::<Vec<_>>(), [("1.1.0", "new"), ("1.0.0", "gammel")]);
    }
}
