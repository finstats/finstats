//! Which picture of a title or a person is the current one. Jellyfin names every image with a tag that
//! changes when the image does, and finstats keeps it on the row (`items.image_tag` / `backdrop_tag`,
//! `users.image_tag`). The image cache names its copy after that tag and the browser is handed it as the
//! ETag, so a poster replaced in Jellyfin is fetched again as soon as finstats has read the new tag —
//! instead of the old one being served from the disk for a week and from the browser for another.
//!
//! The tags themselves are refreshed by `sync_artwork`: one read of what Jellyfin saved since the last
//! look (`MinDateLastSaved`), because replacing a poster saves the item but starts no library scan.

use anyhow::Result;
use serde_json::Value;

use crate::db::{norm_id, rusqlite::Connection, rusqlite::OptionalExtension, rusqlite::params};

/// How far back each look reaches past the last one: the two clocks may differ a little, and an item saved
/// while the last look was being answered must not fall between the two.
pub const OVERLAP_S: i64 = 600;

/// The picture a URL asks for.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Picture {
    Primary,
    Backdrop,
    User,
}

/// The file the disk cache keeps a picture in. Without a tag it is the name the cache has always used.
pub fn cache_name(pic: Picture, id: &str, width: u32, tag: Option<&str>) -> String {
    let base = match pic {
        Picture::Primary => format!("item-{id}-primary-{width}"),
        Picture::Backdrop => format!("item-{id}-backdrop-{width}"),
        Picture::User => format!("user-{id}-{width}"),
    };
    match tag {
        Some(tag) => format!("{base}-{}", safe_tag(tag)),
        None => base,
    }
}

/// Jellyfin's tags are hex and pass as they are; anything else (a tag from an imported file) is hashed, so
/// no tag can name a path or an overlong file.
fn safe_tag(tag: &str) -> String {
    use sha2::{Digest, Sha256};
    if !tag.is_empty() && tag.len() <= 64 && tag.chars().all(|c| c.is_ascii_alphanumeric()) {
        tag.to_string()
    } else {
        hex::encode(&Sha256::digest(tag.as_bytes())[..12])
    }
}

/// The tag finstats last read for a picture, if it knows the row at all.
pub fn tag_of(c: &Connection, pic: Picture, id: &str) -> Result<Option<String>> {
    let sql = match pic {
        Picture::Primary => "SELECT image_tag FROM items WHERE id = ?1",
        Picture::Backdrop => "SELECT backdrop_tag FROM items WHERE id = ?1",
        Picture::User => "SELECT image_tag FROM users WHERE id = ?1",
    };
    Ok(c.prepare_cached(sql)?.query_row([id], |r| r.get::<_, Option<String>>(0)).optional()?.flatten())
}

/// Writes the image tags of an answer onto the rows finstats already has; returns how many changed. Nothing
/// else about a row is touched — least of all `updated_at`, which says whether the last library read saw it.
pub fn store_tags(c: &Connection, items: &[Value]) -> Result<usize> {
    let mut stmt = c.prepare_cached(
        "UPDATE items SET image_tag = ?2, backdrop_tag = ?3 WHERE id = ?1 AND (image_tag IS NOT ?2 OR backdrop_tag IS NOT ?3)",
    )?;
    let mut changed = 0;
    for it in items {
        let Some(id) = it["Id"].as_str() else { continue };
        let tag = |v: &Value| v.as_str().map(str::trim).filter(|s| !s.is_empty()).map(str::to_string);
        changed += stmt.execute(params![norm_id(id), tag(&it["ImageTags"]["Primary"]), tag(&it["BackdropImageTags"][0])])?;
    }
    Ok(changed)
}

/// Where the next look starts: from the last look, else from the last library read (which read every tag),
/// with the overlap. `None` before the library was ever read — that read will bring every tag.
pub fn look_from(last_look: Option<i64>, library_read: Option<i64>) -> Option<i64> {
    let from = last_look.max(library_read)?;
    Some(from - OVERLAP_S)
}

/// The ETag a picture under this tag is sent with.
pub fn etag(tag: &str) -> String {
    format!("\"{}\"", safe_tag(tag))
}

/// `If-None-Match` against the ETag a picture would be sent with.
pub fn not_modified(if_none_match: Option<&str>, etag: &str) -> bool {
    if_none_match.is_some_and(|h| h.split(',').map(str::trim).any(|t| t == "*" || t.trim_start_matches("W/") == etag))
}

/// The query for one page of what Jellyfin saved since `since` (unix seconds): ids and image tags only.
pub fn changed_query(since: i64, start: usize, limit: usize) -> Vec<(&'static str, String)> {
    let since = chrono::DateTime::from_timestamp(since, 0).unwrap_or_default().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    vec![
        ("MinDateLastSaved", since),
        ("Recursive", "true".into()),
        // The kinds of item the library read keeps; any other row would find nothing to update anyway.
        ("IncludeItemTypes", crate::jellyfin::ITEM_TYPES.into()),
        // Grouped into collections, a film would be answered as its BoxSet and never be looked at.
        ("CollapseBoxSetItems", "false".into()),
        ("EnableUserData", "false".into()),
        ("EnableImageTypes", "Primary,Backdrop".into()),
        ("ImageTypeLimit", "1".into()),
        ("SortBy", "DateCreated,SortName".into()),
        ("SortOrder", "Ascending".into()),
        ("StartIndex", start.to_string()),
        ("Limit", limit.to_string()),
        ("EnableTotalRecordCount", "false".into()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn db() -> Connection {
        let c = Connection::open_in_memory().unwrap();
        for m in crate::db::MIGRATIONS {
            c.execute_batch(m).unwrap();
        }
        c
    }

    #[test]
    fn a_replaced_poster_is_cached_under_a_new_name() {
        let id = "0123456789abcdef0123456789abcdef";
        let old = cache_name(Picture::Primary, id, 300, Some("aaa111"));
        let new = cache_name(Picture::Primary, id, 300, Some("bbb222"));
        assert_ne!(old, new, "the new poster would be served the old file");
        assert_ne!(cache_name(Picture::Backdrop, id, 300, Some("aaa111")), old);
        assert_ne!(cache_name(Picture::Primary, id, 480, Some("aaa111")), old);
        // Untagged, a picture keeps the name it always had, so an existing cache is not thrown away.
        assert_eq!(cache_name(Picture::Primary, id, 300, None), format!("item-{id}-primary-300"));
        assert_eq!(cache_name(Picture::Backdrop, id, 1280, None), format!("item-{id}-backdrop-1280"));
        assert_eq!(cache_name(Picture::User, id, 96, None), format!("user-{id}-96"));
    }

    #[test]
    fn a_tag_never_leads_out_of_the_cache_folder() {
        // A tag can come from an imported file, not only from Jellyfin.
        let name = cache_name(Picture::Primary, "abc", 300, Some("../../etc/passwd"));
        assert!(!name.contains('/') && !name.contains(".."), "{name}");
        assert_ne!(name, cache_name(Picture::Primary, "abc", 300, None));
        assert!(cache_name(Picture::Primary, "abc", 300, Some(&"f".repeat(4000))).len() < 200);
    }

    #[test]
    fn the_tag_is_read_from_the_row_the_picture_belongs_to() {
        let c = db();
        c.execute("INSERT INTO items(id, library_id, type, name, image_tag, backdrop_tag, updated_at) VALUES ('m1', 'lib', 'Movie', 'Big Buck Bunny', 'p1', 'b1', 1)", []).unwrap();
        c.execute("INSERT INTO users(id, name, is_admin, image_tag, updated_at) VALUES ('u1', 'alice', 0, 'u-tag', 1)", []).unwrap();
        assert_eq!(tag_of(&c, Picture::Primary, "m1").unwrap().as_deref(), Some("p1"));
        assert_eq!(tag_of(&c, Picture::Backdrop, "m1").unwrap().as_deref(), Some("b1"));
        assert_eq!(tag_of(&c, Picture::User, "u1").unwrap().as_deref(), Some("u-tag"));
        assert_eq!(tag_of(&c, Picture::Primary, "nobody-knows").unwrap(), None);
    }

    #[test]
    fn a_look_writes_the_new_tags_and_nothing_else() {
        let c = db();
        c.execute("INSERT INTO items(id, library_id, type, name, image_tag, backdrop_tag, removed, updated_at) VALUES ('m1', 'lib', 'Movie', 'Big Buck Bunny', 'old', 'b1', 0, 100)", []).unwrap();
        c.execute("INSERT INTO items(id, library_id, type, name, image_tag, removed, updated_at) VALUES ('m2', 'lib', 'Movie', 'Sintel', 's1', 1, 100)", []).unwrap();
        let answer = [
            json!({ "Id": "M1", "ImageTags": { "Primary": "new" }, "BackdropImageTags": ["b1"] }),
            json!({ "Id": "m2", "ImageTags": { "Primary": "s1" } }),
            json!({ "Id": "m9", "ImageTags": { "Primary": "x" } }),
        ];
        assert_eq!(store_tags(&c, &answer).unwrap(), 1, "only Big Buck Bunny's poster changed");
        let row = |id: &str| c.query_row("SELECT image_tag, backdrop_tag, removed, updated_at FROM items WHERE id = ?1", [id], |r| {
            Ok((r.get::<_, Option<String>>(0)?, r.get::<_, Option<String>>(1)?, r.get::<_, i64>(2)?, r.get::<_, i64>(3)?))
        }).unwrap();
        assert_eq!(row("m1"), (Some("new".into()), Some("b1".into()), 0, 100), "updated_at belongs to the library read");
        assert_eq!(row("m2").2, 1, "a look is not a library read: nothing comes back from removed");
        let known: i64 = c.query_row("SELECT COUNT(*) FROM items WHERE id = 'm9'", [], |r| r.get(0)).unwrap();
        assert_eq!(known, 0, "a title finstats does not have is left to the library read");
        assert_eq!(store_tags(&c, &answer).unwrap(), 0, "the same answer twice changes nothing");
        // A poster deleted in Jellyfin: the item comes back without one.
        store_tags(&c, &[json!({ "Id": "m1", "ImageTags": {} })]).unwrap();
        assert_eq!(row("m1").0, None);
        assert_eq!(row("m1").1, None);
    }

    #[test]
    fn the_first_look_starts_where_the_last_library_read_did() {
        assert_eq!(look_from(None, None), None, "the library read will bring every tag");
        assert_eq!(look_from(None, Some(10_000)), Some(10_000 - OVERLAP_S));
        assert_eq!(look_from(Some(20_000), Some(10_000)), Some(20_000 - OVERLAP_S));
        // A library read after the last look read every tag again.
        assert_eq!(look_from(Some(10_000), Some(20_000)), Some(20_000 - OVERLAP_S));
    }

    #[test]
    fn a_browser_holding_the_current_picture_is_told_so() {
        assert!(not_modified(Some("\"abc-300\""), "\"abc-300\""));
        assert!(not_modified(Some("W/\"abc-300\""), "\"abc-300\""));
        assert!(not_modified(Some("\"x\", \"abc-300\""), "\"abc-300\""));
        assert!(not_modified(Some("*"), "\"abc-300\""));
        assert!(!not_modified(Some("\"abd-300\""), "\"abc-300\""), "a replaced poster is sent again");
        assert!(!not_modified(None, "\"abc-300\""));
    }

    #[test]
    fn a_look_asks_for_what_was_saved_since_and_for_real_items_only() {
        let q = changed_query(1_790_685_895, 0, 500);
        let get = |k: &str| q.iter().find(|(key, _)| *key == k).map(|(_, v)| v.as_str());
        assert_eq!(get("MinDateLastSaved"), Some("2026-09-29T12:44:55Z"));
        assert_eq!(get("Recursive"), Some("true"));
        assert_eq!(get("CollapseBoxSetItems"), Some("false"), "a film in a collection would never be looked at");
        assert_eq!(get("EnableImageTypes"), Some("Primary,Backdrop"));
        assert_eq!(get("StartIndex"), Some("0"));
        assert_eq!(get("Limit"), Some("500"));
        assert!(get("IncludeItemTypes").is_some_and(|t| t.contains("Movie") && t.contains("Episode")));
        assert!(get("Fields").is_none(), "nothing but the id and the tags is needed");
    }
}
