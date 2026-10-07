//! Which picture of a title or a person is the current one. Jellyfin names every image with a tag that
//! changes when the image does, and finstats keeps it on the row (`items.image_tag` / `backdrop_tag`,
//! `users.image_tag`). The image cache names its copy after that tag and the browser is handed it as the
//! ETag, so a poster replaced in Jellyfin is fetched again as soon as finstats has read the new tag,
//! instead of the old one being served from the disk for a week and from the browser for another.
//!
//! The tags themselves are refreshed by the library read and, in between, by `sync_changes`: one read of what
//! Jellyfin saved since the last look, because replacing a poster saves the item but starts no library scan.

use anyhow::Result;
use crate::db::{rusqlite::Connection, rusqlite::OptionalExtension};

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
        // A title's poster, else a person's portrait: people are no titles, and keep theirs with the cast they are in.
        Picture::Primary => "SELECT COALESCE((SELECT image_tag FROM items WHERE id = ?1),
                                             (SELECT image_tag FROM item_people WHERE person_id = ?1 AND image_tag IS NOT NULL LIMIT 1))",
        Picture::Backdrop => "SELECT backdrop_tag FROM items WHERE id = ?1",
        Picture::User => "SELECT image_tag FROM users WHERE id = ?1",
    };
    Ok(c.prepare_cached(sql)?.query_row([id], |r| r.get::<_, Option<String>>(0)).optional()?.flatten())
}

/// The ETag a picture under this tag is sent with.
pub fn etag(tag: &str) -> String {
    format!("\"{}\"", safe_tag(tag))
}

/// `If-None-Match` against the ETag a picture would be sent with.
pub fn not_modified(if_none_match: Option<&str>, etag: &str) -> bool {
    if_none_match.is_some_and(|h| h.split(',').map(str::trim).any(|t| t == "*" || t.trim_start_matches("W/") == etag))
}

#[cfg(test)]
mod tests {
    use super::*;

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
        // A person is no title: their portrait's tag is kept with the cast they are in.
        c.execute("INSERT INTO item_people(item_id, person_id, kind, name, sort, has_image, image_tag) VALUES ('m1', 'p1', 'Actor', 'alice', 0, 1, 'face-1')", []).unwrap();
        assert_eq!(tag_of(&c, Picture::Primary, "p1").unwrap().as_deref(), Some("face-1"));
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
}
