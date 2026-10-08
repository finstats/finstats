//! A FinUI preset applied to finstats itself (Settings → Appearance). The preset is made at FinUI create
//! (finui.finstats.no/create) and is a code: one base-36 digit per axis of `finui/create/presets.json`. finstats
//! keeps FinUI's own generated files of each option's tokens (`finui/p/<axis>/<option>.css`, built by FinUI's
//! tools/build-site.mjs and copied here; the QA stage `finui` holds them to FinUI's) and does with them what FinUI's
//! install.sh does: the files the code names, in axis order, after the stylesheet, so a later choice sets a token last.
//! So there is no second copy of how a choice becomes tokens.

/// Every option file there is for an axis: `p/<axis>/<option>.css` for options 1…, none for the default.
fn option(axis: usize, option: usize) -> Option<String> {
    let f = crate::api::WebAssets::get(&format!("assets/finui/p/{axis}/{option}.css"))?;
    String::from_utf8(f.data.into_owned()).ok()
}

/// How many axes a code may spell: one folder of option files each.
fn axes() -> usize {
    (0..).take_while(|&a| option(a, 1).is_some()).count()
}

/// The `:root` blocks a code sets, after a comment that names it; "" for a code that changes nothing (or none at all),
/// and None for one that names an option there is no file of.
pub fn overlay(code: &str) -> Option<String> {
    if !code.chars().all(|c| c.is_ascii_digit() || c.is_ascii_lowercase()) || code.len() > axes() {
        return None;
    }
    let mut blocks = String::new();
    for (axis, c) in code.chars().enumerate() {
        let o = c.to_digit(36)? as usize;
        if o > 0 {
            blocks.push_str(&option(axis, o)?);
        }
    }
    if blocks.is_empty() {
        return Some(String::new());
    }
    Some(format!("\n/* FinUI preset {code}, set in Settings → Appearance: https://finui.finstats.no/create/?preset={code} */\n{blocks}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn presets() -> serde_json::Value {
        serde_json::from_str(&std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/web/assets/finui/create/presets.json")).unwrap()).unwrap()
    }

    #[test]
    fn the_option_files_are_the_options_presets_json_names() {
        let p = presets();
        let axes = p["axes"].as_array().unwrap();
        assert_eq!(super::axes(), axes.len(), "one folder of option files per axis");
        for (a, axis) in axes.iter().enumerate() {
            let n = axis["options"].as_array().unwrap().len();
            assert!(option(a, 0).is_none(), "{}: the default sets nothing, so it has no file", axis["key"]);
            for o in 1..n {
                assert!(option(a, o).is_some_and(|css| css.contains(":root {")), "{}: p/{a}/{o}.css", axis["key"]);
            }
            assert!(option(a, n).is_none(), "{}: a file for an option presets.json does not have", axis["key"]);
        }
    }

    #[test]
    fn no_code_or_a_code_of_defaults_changes_nothing() {
        assert_eq!(overlay(""), Some(String::new()));
        assert_eq!(overlay(&"0".repeat(super::axes())), Some(String::new()));
        assert_eq!(overlay("00"), Some(String::new()), "a short code leaves the later axes at their defaults");
    }

    #[test]
    fn a_code_that_names_no_option_is_refused() {
        let past = char::from_digit(presets()["axes"][0]["options"].as_array().unwrap().len() as u32, 36).unwrap();
        for bad in ["zz", "0Z", "0-1", "0.1", "é", " 01", &"0".repeat(super::axes() + 1), &past.to_string()] {
            assert_eq!(overlay(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn a_preset_is_the_files_its_code_names_in_axis_order_after_a_comment_naming_it() {
        let css = overlay("0101").unwrap();
        assert!(css.starts_with("\n/* FinUI preset 0101,"), "{css}");
        let blocks = format!("{}{}", option(1, 1).unwrap(), option(3, 1).unwrap());
        assert!(css.ends_with(&blocks), "{css}");
        assert_eq!(css.matches(":root {").count(), 2);
    }
}
