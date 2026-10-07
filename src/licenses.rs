//! Third-party licences: what finstats is built on, and the licence each part is under.

use std::sync::OnceLock;

use axum::http::HeaderValue;
use axum::http::header::CONTENT_TYPE;
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};

use crate::auth::AuthUser;

const SOURCE: &str = include_str!("../THIRD-PARTY.json");

/// What is inside the binary but is not a crate, so nothing generated can know about it:
/// finstats itself, the fonts, the map, and the city database, which is not shipped at
/// all but is read by whoever switches it on, and asks to be credited.
/// Each licence file that really exists on disk is compiled in; nothing here is retyped.
const BUNDLED: &[(&str, &str, &str, &str, Option<&str>)] = &[
    ("finstats", env!("CARGO_PKG_VERSION"), "GPL-3.0-only", "https://github.com/finstats/finstats", Some(include_str!("../LICENSE"))),
    ("Inter", "", "OFL-1.1", "https://github.com/rsms/inter", Some(include_str!("../web/assets/fonts/LICENSE-Inter.txt"))),
    ("Inter (cards)", "4.1", "OFL-1.1", "https://github.com/rsms/inter", Some(include_str!("../fonts/card/LICENSE-Inter.txt"))),
    ("JetBrains Mono", "", "OFL-1.1", "https://github.com/JetBrains/JetBrainsMono", Some(include_str!("../web/assets/fonts/LICENSE-JetBrainsMono.txt"))),
    // The fonts a FinUI preset may choose (Settings → Appearance), Fontsource's variable builds.
    ("Geist", "", "OFL-1.1", "https://fontsource.org/fonts/geist", Some(include_str!("../web/assets/fonts/LICENSE-Geist.txt"))),
    ("Roboto", "", "OFL-1.1", "https://fontsource.org/fonts/roboto", Some(include_str!("../web/assets/fonts/LICENSE-Roboto.txt"))),
    ("Open Sans", "", "OFL-1.1", "https://fontsource.org/fonts/open-sans", Some(include_str!("../web/assets/fonts/LICENSE-OpenSans.txt"))),
    ("Source Sans 3", "", "OFL-1.1", "https://fontsource.org/fonts/source-sans-3", Some(include_str!("../web/assets/fonts/LICENSE-SourceSans3.txt"))),
    ("IBM Plex Sans", "", "OFL-1.1", "https://fontsource.org/fonts/ibm-plex-sans", Some(include_str!("../web/assets/fonts/LICENSE-IBMPlexSans.txt"))),
    ("Nunito", "", "OFL-1.1", "https://fontsource.org/fonts/nunito", Some(include_str!("../web/assets/fonts/LICENSE-Nunito.txt"))),
    ("Manrope", "", "OFL-1.1", "https://fontsource.org/fonts/manrope", Some(include_str!("../web/assets/fonts/LICENSE-Manrope.txt"))),
    ("DM Sans", "", "OFL-1.1", "https://fontsource.org/fonts/dm-sans", Some(include_str!("../web/assets/fonts/LICENSE-DMSans.txt"))),
    ("Plus Jakarta Sans", "", "OFL-1.1", "https://fontsource.org/fonts/plus-jakarta-sans", Some(include_str!("../web/assets/fonts/LICENSE-PlusJakartaSans.txt"))),
    ("Figtree", "", "OFL-1.1", "https://fontsource.org/fonts/figtree", Some(include_str!("../web/assets/fonts/LICENSE-Figtree.txt"))),
    ("Outfit", "", "OFL-1.1", "https://fontsource.org/fonts/outfit", Some(include_str!("../web/assets/fonts/LICENSE-Outfit.txt"))),
    ("Lexend", "", "OFL-1.1", "https://fontsource.org/fonts/lexend", Some(include_str!("../web/assets/fonts/LICENSE-Lexend.txt"))),
    ("Space Grotesk", "", "OFL-1.1", "https://fontsource.org/fonts/space-grotesk", Some(include_str!("../web/assets/fonts/LICENSE-SpaceGrotesk.txt"))),
    ("Work Sans", "", "OFL-1.1", "https://fontsource.org/fonts/work-sans", Some(include_str!("../web/assets/fonts/LICENSE-WorkSans.txt"))),
    ("Public Sans", "", "OFL-1.1", "https://fontsource.org/fonts/public-sans", Some(include_str!("../web/assets/fonts/LICENSE-PublicSans.txt"))),
    ("Lora", "", "OFL-1.1", "https://fontsource.org/fonts/lora", Some(include_str!("../web/assets/fonts/LICENSE-Lora.txt"))),
    ("Merriweather", "", "OFL-1.1", "https://fontsource.org/fonts/merriweather", Some(include_str!("../web/assets/fonts/LICENSE-Merriweather.txt"))),
    ("Source Serif 4", "", "OFL-1.1", "https://fontsource.org/fonts/source-serif-4", Some(include_str!("../web/assets/fonts/LICENSE-SourceSerif4.txt"))),
    ("Playfair Display", "", "OFL-1.1", "https://fontsource.org/fonts/playfair-display", Some(include_str!("../web/assets/fonts/LICENSE-PlayfairDisplay.txt"))),
    ("Fraunces", "", "OFL-1.1", "https://fontsource.org/fonts/fraunces", Some(include_str!("../web/assets/fonts/LICENSE-Fraunces.txt"))),
    ("EB Garamond", "", "OFL-1.1", "https://fontsource.org/fonts/eb-garamond", Some(include_str!("../web/assets/fonts/LICENSE-EBGaramond.txt"))),
    ("Fira Code", "", "OFL-1.1", "https://fontsource.org/fonts/fira-code", Some(include_str!("../web/assets/fonts/LICENSE-FiraCode.txt"))),
    ("Source Code Pro", "", "OFL-1.1", "https://fontsource.org/fonts/source-code-pro", Some(include_str!("../web/assets/fonts/LICENSE-SourceCodePro.txt"))),
    ("Roboto Mono", "", "OFL-1.1", "https://fontsource.org/fonts/roboto-mono", Some(include_str!("../web/assets/fonts/LICENSE-RobotoMono.txt"))),
    ("Geist Mono", "", "OFL-1.1", "https://fontsource.org/fonts/geist-mono", Some(include_str!("../web/assets/fonts/LICENSE-GeistMono.txt"))),
    ("Montserrat", "", "OFL-1.1", "https://fontsource.org/fonts/montserrat", Some(include_str!("../web/assets/fonts/LICENSE-Montserrat.txt"))),
    ("Raleway", "", "OFL-1.1", "https://fontsource.org/fonts/raleway", Some(include_str!("../web/assets/fonts/LICENSE-Raleway.txt"))),
    ("Rubik", "", "OFL-1.1", "https://fontsource.org/fonts/rubik", Some(include_str!("../web/assets/fonts/LICENSE-Rubik.txt"))),
    ("Mulish", "", "OFL-1.1", "https://fontsource.org/fonts/mulish", Some(include_str!("../web/assets/fonts/LICENSE-Mulish.txt"))),
    ("Noto Sans", "", "OFL-1.1", "https://fontsource.org/fonts/noto-sans", Some(include_str!("../web/assets/fonts/LICENSE-NotoSans.txt"))),
    ("Instrument Sans", "", "OFL-1.1", "https://fontsource.org/fonts/instrument-sans", Some(include_str!("../web/assets/fonts/LICENSE-InstrumentSans.txt"))),
    ("Red Hat Mono", "", "OFL-1.1", "https://fontsource.org/fonts/red-hat-mono", Some(include_str!("../web/assets/fonts/LICENSE-RedHatMono.txt"))),
    ("Inconsolata", "", "OFL-1.1", "https://fontsource.org/fonts/inconsolata", Some(include_str!("../web/assets/fonts/LICENSE-Inconsolata.txt"))),
    ("Victor Mono", "", "OFL-1.1", "https://fontsource.org/fonts/victor-mono", Some(include_str!("../web/assets/fonts/LICENSE-VictorMono.txt"))),
    ("Natural Earth", "1:50m", "Public domain", "https://www.naturalearthdata.com", None),
    ("DB-IP IP to City Lite", "", "CC-BY-4.0", "https://db-ip.com/db/lite.php", None),
];

fn crate_kind() -> String {
    "crate".into()
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Component {
    pub name: String,
    pub version: String,
    /// SPDX expression as the crate itself declares it ("MIT OR Apache-2.0").
    pub license: String,
    pub repository: String,
    /// Which of `Notices::notices` this component ships, by index.
    pub notices: Vec<usize>,
    /// "app", "bundled" or "crate". The generated file names none, so a crate it is.
    #[serde(default = "crate_kind")]
    pub kind: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Notice {
    pub file: String,
    pub text: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
pub struct Notices {
    pub components: Vec<Component>,
    pub notices: Vec<Notice>,
}

/// The whole notice: the generated crate list plus what is bundled by hand.
pub fn notice() -> Notices {
    let mut n: Notices = serde_json::from_str(SOURCE).expect("THIRD-PARTY.json is generated by tools/make-third-party.py and must parse");
    let mut head = vec![];
    for &(name, version, license, repository, text) in BUNDLED {
        let notices = match text {
            Some(text) => {
                n.notices.push(Notice { file: format!("{name} licence"), text: text.trim().to_string() });
                vec![n.notices.len() - 1]
            }
            None => vec![],
        };
        let kind = if name == env!("CARGO_PKG_NAME") { "app" } else { "bundled" };
        head.push(Component { name: name.into(), version: version.into(), license: license.into(), repository: repository.into(), notices, kind: kind.into() });
    }
    head.append(&mut n.components);
    n.components = head;
    n
}

/// The answer, rendered once: it is the same for every caller and cannot change while the
/// process runs, and it runs to half a megabyte of licence text.
fn body() -> &'static str {
    static BODY: OnceLock<String> = OnceLock::new();
    BODY.get_or_init(|| {
        let n = notice();
        serde_json::json!({ "version": env!("CARGO_PKG_VERSION"), "components": n.components, "notices": n.notices }).to_string()
    })
}

/// `GET /api/licenses`: finstats' own licence and every third-party one it is built on.
pub async fn licenses(_user: AuthUser) -> impl IntoResponse {
    ([(CONTENT_TYPE, HeaderValue::from_static("application/json"))], body())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn the_notice_lists_a_crate_with_its_licence() {
        let n = notice();
        let serde = n.components.iter().find(|c| c.name == "serde").expect("serde is a dependency and must be in the notice");
        assert!(serde.license.contains("MIT"), "serde's licence expression is missing: {:?}", serde.license);
        assert!(!serde.notices.is_empty(), "serde ships licence texts; the notice must carry them");
    }

    /// The fonts and the map are not crates, so nothing generated can know about them.
    #[test]
    fn the_notice_covers_what_is_bundled_and_finstats_itself() {
        let n = notice();
        let text = |name: &str| {
            let c = n.components.iter().find(|c| c.name == name).unwrap_or_else(|| panic!("{name} is shipped inside the binary and must be in the notice"));
            (c.license.clone(), c.notices.iter().map(|&i| n.notices[i].text.as_str()).collect::<Vec<_>>().join("\n"))
        };
        let (license, body) = text("finstats");
        assert_eq!(license, "GPL-3.0-only");
        assert!(body.contains("GNU GENERAL PUBLIC LICENSE"), "finstats' own licence text belongs here too");
        for font in ["Inter", "JetBrains Mono"] {
            let (license, body) = text(font);
            assert_eq!(license, "OFL-1.1");
            assert!(body.contains("SIL OPEN FONT LICENSE"), "{font} must carry the licence file that sits beside it");
        }
        assert_eq!(text("Natural Earth").0, "Public domain");
    }

    /// A font a FinUI preset may choose is shipped inside the binary like Inter: every licence that sits in
    /// web/assets/fonts is in the notice, as the font's own text.
    #[test]
    fn every_font_finstats_bundles_is_in_the_notice_with_its_licence() {
        let n = notice();
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("web/assets/fonts");
        for e in std::fs::read_dir(&dir).unwrap().flatten() {
            let file = e.file_name().to_string_lossy().to_string();
            let Some(_) = file.strip_prefix("LICENSE-").and_then(|f| f.strip_suffix(".txt")) else { continue };
            let text = std::fs::read_to_string(e.path()).unwrap();
            let found = n.components.iter().any(|c| c.license == "OFL-1.1" && c.notices.iter().any(|&i| n.notices[i].text == text.trim()));
            assert!(found, "{file} sits beside a bundled font and is not in the notice");
        }
    }

    /// The whole notice is the same for everybody and never changes while the process runs,
    /// so it is rendered once and handed out as it stands.
    #[test]
    fn the_answer_is_rendered_once_and_carries_the_running_version() {
        let v: serde_json::Value = serde_json::from_str(body()).expect("the answer must be JSON");
        assert_eq!(v["version"], env!("CARGO_PKG_VERSION"));
        assert_eq!(v["components"].as_array().map(|a| a.len()), Some(notice().components.len()));
        assert!(v["notices"].as_array().is_some_and(|a| a.len() > 50), "the licence texts themselves are the point of it");
        assert!(std::ptr::eq(body(), body()), "it is built once, not per request");
    }

    /// The gate on the generated half: Cargo.lock is the dependency graph as cargo resolved it,
    /// so a crate added, removed or bumped without re-running `tools/make-third-party.py` fails here.
    #[test]
    fn the_notice_covers_every_crate_in_the_lockfile() {
        let n = notice();
        let have: HashSet<(&str, &str)> = n.components.iter().map(|c| (c.name.as_str(), c.version.as_str())).collect();
        for (name, version) in lockfile(include_str!("../Cargo.lock")) {
            if name == env!("CARGO_PKG_NAME") {
                continue;
            }
            assert!(have.contains(&(name, version)), "{name} {version} is a dependency with no licence recorded: run tools/make-third-party.py");
        }
        for c in &n.components {
            assert!(!c.license.is_empty() || !c.notices.is_empty(), "{} is listed with no licence at all", c.name);
            for &i in &c.notices {
                assert!(n.notices.get(i).is_some_and(|t| !t.text.is_empty()), "{} points at licence text {i}, which is not there", c.name);
            }
        }
    }

    /// `[[package]]` blocks, as `name` and `version`.
    fn lockfile(source: &str) -> Vec<(&str, &str)> {
        fn value(line: &str) -> &str {
            line.split_once('"').map(|(_, r)| r.trim_end_matches('"')).unwrap_or("")
        }
        let mut out = vec![];
        let mut name = None;
        for line in source.lines() {
            if let Some(rest) = line.strip_prefix("name = ") {
                name = Some(value(rest));
            } else if let Some(rest) = line.strip_prefix("version = ")
                && let Some(name) = name.take()
            {
                out.push((name, value(rest)));
            }
        }
        out
    }
}
