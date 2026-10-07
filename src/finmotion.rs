//! FinMotion: how FinUI moves, put on top of it (`github.com/finstats/finmotion`, a copy in `web/assets/finmotion`).
//! finstats always wears it: `/assets/finmotion.css` is every stylesheet its registry lists, in order (the springs,
//! then each part's), served as one, as `/assets/finui.css` is; `main.js` calls its `motion()`. The tests below hold the
//! registry to what is on disk.

use serde::Deserialize;

/// What `registry.json` says.
#[derive(Deserialize)]
#[allow(dead_code)]
pub struct Registry {
    pub name: String,
    pub license: String,
    pub prefix: String,
    /// What it moves: FinUI.
    pub on: String,
    /// Paths under `finmotion/`: the springs, moving from script, motion(), the list of parts.
    pub foundation: Vec<String>,
    pub parts: Vec<Part>,
    /// What it adds that FinUI has not (an odometer): its own components, after the parts.
    #[serde(default)]
    pub components: Vec<Part>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct Part {
    pub name: String,
    pub files: Vec<String>,
}

fn file(path: &str) -> anyhow::Result<String> {
    let f = crate::api::WebAssets::get(&format!("assets/finmotion/{path}")).ok_or_else(|| anyhow::anyhow!("finmotion/{path} is listed but not there"))?;
    Ok(String::from_utf8(f.data.into_owned())?)
}

/// The registry, read through the embed.
pub fn registry() -> anyhow::Result<Registry> {
    Ok(serde_json::from_str(&file("registry.json")?)?)
}

/// Every CSS file the registry lists (foundation, parts, components), each once, in order.
pub fn stylesheet() -> anyhow::Result<String> {
    let r = registry()?;
    let mut out = String::from("/* FinMotion: how FinUI moves, from web/assets/finmotion in registry.json's order. */\n");
    for path in r.foundation.iter().chain(r.parts.iter().chain(r.components.iter()).flat_map(|p| p.files.iter())).filter(|p| p.ends_with(".css")) {
        out.push('\n');
        out.push_str(file(path)?.trim());
        out.push('\n');
    }
    Ok(out)
}

/// `/assets/finmotion.css`, with its ETag: read once in a release build, again on every request in a debug build (an edit
/// shows on the next refresh, as for every other file). The same for everybody: a person's look is FinUI's preset, and
/// FinMotion reads its pace from that.
pub fn served() -> anyhow::Result<(std::sync::Arc<String>, String)> {
    let tag = |css: String| {
        use sha2::Digest;
        let etag = format!("\"{}\"", hex::encode(&sha2::Sha256::digest(css.as_bytes())[..8]));
        (std::sync::Arc::new(css), etag)
    };
    if cfg!(debug_assertions) {
        return Ok(tag(stylesheet()?));
    }
    static ONCE: std::sync::OnceLock<(std::sync::Arc<String>, String)> = std::sync::OnceLock::new();
    Ok(ONCE.get_or_init(|| tag(stylesheet().unwrap_or_default())).clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    fn dir() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("web/assets/finmotion")
    }
    fn listed(r: &Registry) -> Vec<String> {
        r.foundation.iter().chain(r.parts.iter().chain(r.components.iter()).flat_map(|p| p.files.iter())).cloned().collect()
    }

    #[test]
    fn the_registry_is_finmotion_s_on_finui_and_carries_the_repository_s_licence() {
        let r = registry().unwrap();
        assert_eq!((r.name.as_str(), r.prefix.as_str(), r.on.as_str()), ("finmotion", "fm-", "finui"));
        let cargo = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")).unwrap();
        assert!(cargo.contains(&format!("license = \"{}\"", r.license)), "FinMotion is under {}, finstats under something else", r.license);
    }

    #[test]
    fn every_listed_file_exists_and_every_file_is_listed() {
        let r = registry().unwrap();
        let want: std::collections::BTreeSet<String> = listed(&r).into_iter().chain(["registry.json".to_owned()]).collect();
        let mut on_disk = std::collections::BTreeSet::new();
        let mut stack = vec![dir()];
        while let Some(d) = stack.pop() {
            for e in std::fs::read_dir(&d).unwrap().flatten() {
                if e.path().is_dir() { stack.push(e.path()); } else { on_disk.insert(e.path().strip_prefix(dir()).unwrap().to_string_lossy().replace('\\', "/")); }
            }
        }
        assert_eq!(on_disk, want);
    }

    #[test]
    fn the_stylesheet_is_every_listed_css_file_once_and_in_order() {
        let r = registry().unwrap();
        let css = stylesheet().unwrap();
        let mut at = 0;
        for f in listed(&r).iter().filter(|f| f.ends_with(".css")) {
            let body = std::fs::read_to_string(dir().join(f)).unwrap();
            let found = css[at..].find(body.trim()).unwrap_or_else(|| panic!("{f} is not in the stylesheet, or not in order"));
            at += found + body.trim().len();
            assert_eq!(css.matches(body.trim()).count(), 1, "{f} is in it more than once");
        }
        assert!(css.contains("--spring-settle"), "the springs come with it");
    }
}
