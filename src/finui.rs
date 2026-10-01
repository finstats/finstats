//! finui: finstats' own component registry, in `web/assets/finui`. Components are source in this repository — a
//! module and its CSS each, described by the module's own `meta` and listed in `registry.json` — and finstats serves
//! their styles as one stylesheet, `/assets/finui.css`: the foundation (tokens, base) and every component's CSS in the
//! registry's order, read through the same embed as every other file. One request on the critical path and no build
//! step; the files stay one per component. The tests below hold the registry to what is on disk.

use serde::Deserialize;

/// What `registry.json` says.
#[derive(Deserialize)]
#[allow(dead_code)]
pub struct Registry {
    pub name: String,
    pub license: String,
    pub prefix: String,
    /// Paths under `finui/`, served first: the tokens, the base styles, the core modules.
    pub foundation: Vec<String>,
    pub components: Vec<Component>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct Component {
    pub name: String,
    pub category: String,
    pub files: Vec<String>,
    /// Every token its CSS reads.
    pub tokens: Vec<String>,
    /// Other finui components (or "core") it is built from.
    pub requires: Vec<String>,
}

fn file(path: &str) -> anyhow::Result<String> {
    let f = crate::api::WebAssets::get(&format!("assets/finui/{path}")).ok_or_else(|| anyhow::anyhow!("finui/{path} is listed but not there"))?;
    Ok(String::from_utf8(f.data.into_owned())?)
}

/// The registry, read through the embed.
pub fn registry() -> anyhow::Result<Registry> {
    Ok(serde_json::from_str(&file("registry.json")?)?)
}

/// Every CSS file the registry lists, foundation first, each once, in order.
pub fn stylesheet() -> anyhow::Result<String> {
    let r = registry()?;
    let mut out = String::from("/* finui — finstats' components: web/assets/finui, in registry.json's order. */\n");
    for path in r.foundation.iter().chain(r.components.iter().flat_map(|c| c.files.iter())).filter(|p| p.ends_with(".css")) {
        out.push('\n');
        out.push_str(file(path)?.trim());
        out.push('\n');
    }
    Ok(out)
}

/// `/assets/finui.css`, with its ETag. A release build reads the embed once; a debug build reads the files again
/// on every request, so an edit shows on the next refresh as it does for every other file.
pub fn served() -> anyhow::Result<(std::sync::Arc<String>, String)> {
    fn tagged(css: String) -> (std::sync::Arc<String>, String) {
        use sha2::Digest;
        let etag = format!("\"{}\"", hex::encode(&sha2::Sha256::digest(css.as_bytes())[..8]));
        (std::sync::Arc::new(css), etag)
    }
    if cfg!(debug_assertions) {
        return Ok(tagged(stylesheet()?));
    }
    static ONCE: std::sync::OnceLock<(std::sync::Arc<String>, String)> = std::sync::OnceLock::new();
    if let Some(v) = ONCE.get() {
        return Ok(v.clone());
    }
    Ok(ONCE.get_or_init(|| tagged(stylesheet().unwrap_or_default())).clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, BTreeSet};
    use std::path::{Path, PathBuf};

    fn web() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("web")
    }
    fn finui() -> PathBuf {
        web().join("assets/finui")
    }
    fn read(p: &Path) -> String {
        std::fs::read_to_string(p).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
    }
    /// Every file under a folder, as paths relative to it.
    fn files_under(dir: &Path) -> Vec<String> {
        let mut out = vec![];
        let mut todo = vec![dir.to_path_buf()];
        while let Some(d) = todo.pop() {
            for e in std::fs::read_dir(&d).unwrap().flatten() {
                let p = e.path();
                if p.is_dir() {
                    todo.push(p);
                } else {
                    out.push(p.strip_prefix(dir).unwrap().to_string_lossy().replace('\\', "/"));
                }
            }
        }
        out.sort();
        out
    }
    fn listed(r: &Registry) -> Vec<String> {
        r.foundation.iter().cloned().chain(r.components.iter().flat_map(|c| c.files.iter().cloned())).collect()
    }
    /// The CSS of a file without its comments, which may name a colour or a class in passing.
    fn uncommented(css: &str) -> String {
        let mut out = String::new();
        let mut rest = css;
        while let Some(at) = rest.find("/*") {
            out.push_str(&rest[..at]);
            rest = rest[at..].find("*/").map_or("", |end| &rest[at + end + 2..]);
        }
        out + rest
    }
    /// The JS of a file without its comments.
    fn js_code(js: &str) -> String {
        uncommented(js).lines().filter(|l| !l.trim_start().starts_with("//")).collect::<Vec<_>>().join("\n")
    }

    #[test]
    fn the_registry_is_finui_s_and_carries_the_repository_s_licence() {
        let r = registry().expect("web/assets/finui/registry.json");
        assert_eq!((r.name.as_str(), r.prefix.as_str()), ("finui", "fui-"));
        let cargo = read(&Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"));
        assert!(cargo.contains(&format!("license = \"{}\"", r.license)), "finui is under {}, finstats under something else", r.license);
        let names: BTreeSet<&str> = r.components.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names.len(), r.components.len(), "a component is listed twice");
        for c in &r.components {
            for need in &c.requires {
                assert!(need == "core" || names.contains(need.as_str()), "{} requires {need}, which finui does not have", c.name);
            }
        }
    }

    #[test]
    fn every_listed_file_exists_and_every_file_is_listed() {
        let r = registry().unwrap();
        let listed: BTreeSet<String> = listed(&r).into_iter().collect();
        let on_disk: BTreeSet<String> = files_under(&finui()).into_iter().filter(|f| f != "registry.json").collect();
        let missing: Vec<_> = listed.difference(&on_disk).collect();
        let unlisted: Vec<_> = on_disk.difference(&listed).collect();
        assert!(missing.is_empty(), "listed in registry.json but not there: {missing:?}");
        assert!(unlisted.is_empty(), "in finui/ but not in registry.json: {unlisted:?}");
    }

    #[test]
    fn the_stylesheet_is_every_listed_css_file_once_and_in_order() {
        let r = registry().unwrap();
        let css = stylesheet().unwrap();
        let mut at = 0;
        for f in listed(&r).iter().filter(|f| f.ends_with(".css")) {
            let body = read(&finui().join(f));
            let found = css[at..].find(body.trim()).unwrap_or_else(|| panic!("{f} is not in finui.css, or not after what comes before it"));
            at += found + body.trim().len();
            assert_eq!(css.matches(body.trim()).count(), 1, "{f} is in finui.css twice");
        }
        assert!(css.starts_with("/* finui"), "finui.css says what it is");
    }

    #[test]
    fn finui_imports_nothing_from_outside_finui() {
        let root = finui().canonicalize().unwrap();
        for f in files_under(&finui()).iter().filter(|f| f.ends_with(".js")) {
            let path = finui().join(f);
            let code = js_code(&read(&path));
            for spec in code.split(['\'', '"', '`']).skip(1).step_by(2).filter(|s| s.starts_with('.') || s.starts_with('/')) {
                let follows_import = code.contains(&format!("from '{spec}'")) || code.contains(&format!("import('{spec}')")) || code.contains(&format!("from \"{spec}\""));
                if !follows_import {
                    continue;
                }
                let target = if spec.starts_with('/') { web().join(spec.trim_start_matches('/')) } else { path.parent().unwrap().join(spec) };
                let target = target.canonicalize().unwrap_or_else(|_| panic!("{f} imports {spec}, which is not there"));
                assert!(target.starts_with(&root), "{f} imports {spec}: finui imports nothing from outside finui");
            }
        }
    }

    /// A colour is a token, and every token is `light-dark(light, dark)`, in `tokens.css` alone: a literal anywhere else
    /// is right in one theme and wrong in the other.
    #[test]
    fn no_colour_literal_outside_the_tokens() {
        let assets = web().join("assets");
        let mut found = vec![];
        for f in files_under(&assets).iter().filter(|f| (f.ends_with(".css") || f.ends_with(".js")) && f.as_str() != "finui/tokens.css") {
            let text = read(&assets.join(f));
            let code = if f.ends_with(".css") { uncommented(&text) } else { js_code(&text) };
            for (n, line) in code.lines().enumerate() {
                let hex = line.match_indices('#').any(|(i, _)| {
                    let run: String = line[i + 1..].chars().take_while(char::is_ascii_hexdigit).collect();
                    let next = line[i + 1 + run.len()..].chars().next();
                    matches!(run.len(), 3 | 4 | 6 | 8) && !next.is_some_and(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
                        && (f.ends_with(".css") || line[..i].ends_with(['\'', '"', ' ', ':']))
                });
                let function = ["rgb(", "rgba(", "hsl(", "hsla(", "light-dark("].iter().any(|k| line.contains(k));
                if hex || function {
                    found.push(format!("{f}:{}: {}", n + 1, line.trim()));
                }
            }
        }
        assert!(found.is_empty(), "colours that are not tokens:\n{}", found.join("\n"));
    }

    /// A class `fui-x`, `fui-x--variant` or `fui-x__part` is styled by component x's CSS and nowhere else.
    #[test]
    fn a_finui_class_is_styled_by_its_own_component_only() {
        let r = registry().unwrap();
        let mut owner: BTreeMap<String, String> = BTreeMap::new();
        for c in &r.components {
            for f in c.files.iter().filter(|f| f.ends_with(".css")) {
                for class in fui_classes(&uncommented(&read(&finui().join(f)))) {
                    let base = class.split("--").next().unwrap().split("__").next().unwrap().to_string();
                    assert_eq!(base, format!("fui-{}", c.name), "{f} styles .{class}, which is not {}'s", c.name);
                    owner.insert(class, c.name.clone());
                }
            }
        }
        for f in files_under(&web().join("assets")).iter().filter(|f| f.ends_with(".css") && !f.starts_with("finui/components/")) {
            let found = fui_classes(&uncommented(&read(&web().join("assets").join(f))));
            assert!(found.is_empty(), "{f} styles finui's classes: {found:?}");
        }
    }

    fn fui_classes(css: &str) -> BTreeSet<String> {
        css.match_indices(".fui-").map(|(i, _)| css[i + 1..].chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_').collect()).collect()
    }

    #[test]
    fn every_token_a_component_reads_is_listed_and_defined() {
        let r = registry().unwrap();
        let tokens = uncommented(&read(&finui().join("tokens.css")));
        let defined: BTreeSet<String> = tokens.match_indices("--").filter(|(i, _)| tokens[i + 2..].split(':').next().is_some_and(|n| !n.contains([' ', ')', ',', ';']))).map(|(i, _)| tokens[i..].split(':').next().unwrap().to_string()).collect();
        for c in &r.components {
            let mut read_here = BTreeSet::new();
            for f in c.files.iter().filter(|f| f.ends_with(".css")) {
                let css = uncommented(&read(&finui().join(f)));
                read_here.extend(css.match_indices("var(--").map(|(i, _)| css[i + 4..].chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '-').collect::<String>()));
            }
            let listed: BTreeSet<String> = c.tokens.iter().cloned().collect();
            assert_eq!(read_here, listed, "{}: the tokens its CSS reads and the tokens registry.json lists", c.name);
            for t in &listed {
                assert!(defined.contains(t), "{} reads {t}, which tokens.css does not define", c.name);
            }
        }
    }

    /// The server-drawn profile cards stay dark, and are drawn in the dark side of the same tokens the pages use.
    #[test]
    fn the_server_drawn_cards_are_the_dark_side_of_the_tokens() {
        let tokens = read(&finui().join("tokens.css"));
        let dark = |name: &str| -> String {
            let line = tokens.lines().find(|l| l.trim_start().starts_with(&format!("{name}:"))).unwrap_or_else(|| panic!("{name} is not a token"));
            let inner = line.split_once("light-dark(").unwrap_or_else(|| panic!("{name} is not light-dark()")).1;
            let mut depth = 0;
            let comma = inner.char_indices().find(|&(_, c)| { depth += (c == '(') as i32 - (c == ')') as i32; c == ',' && depth == 0 }).unwrap().0;
            inner[comma + 1..].split(')').next().unwrap().trim().to_lowercase()
        };
        use crate::card::*;
        let pairs = [("--bg", BG), ("--bg-2", TILE), ("--text", TEXT), ("--text-muted", MUTED), ("--text-faint", FAINT), ("--accent", ACCENT), ("--accent-hi", ACCENT_HI), ("--heat-0", HEAT_EMPTY)];
        for (token, card) in pairs.into_iter().chain(HEAT_RAMP.iter().enumerate().map(|(i, c)| (["--heat-1", "--heat-2", "--heat-3", "--heat-4", "--heat-5", "--heat-6"][i], *c))) {
            assert_eq!(dark(token), card.to_lowercase(), "card.rs and {token} must change together");
        }
    }
}
