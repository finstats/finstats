//! Everything in one library, a page at a time: every film, show or album, not only what arrived lately. The library is
//! the same for everyone (as Recently added and the library's make-up are), so nothing here is scoped to the caller, and
//! nothing here is about plays.

use anyhow::Result;
use serde_json::{Value, json};

use axum::Json;
use axum::extract::{Path, Query, State};
use serde::Deserialize;

use crate::auth::AuthUser;
use crate::db::rusqlite::{Connection, params_from_iter, types::Value as SqlValue};
use crate::state::{ApiResult, App};

/// Which titles a page lists.
#[derive(Debug, Clone, Default)]
pub struct Browse {
    pub library_id: String,
    pub kind: Option<String>,
    pub q: Option<String>,
    pub sort: Option<String>,
    pub dir: Option<String>,
    pub page: i64,
}

/// Titles in one page: enough to fill a screen of posters twice, so scrolling asks again rarely.
pub const PAGE_SIZE: i64 = 60;
/// What a library holds as titles of their own; episodes, seasons and tracks belong to one of these.
const TITLE_TYPES: &str = "'Movie', 'Series', 'MusicAlbum', 'MusicVideo', 'Video', 'Book', 'AudioBook'";
/// What a show weighs: its episodes' files.
const SIZE: &str = "COALESCE(t.size_bytes, (SELECT SUM(e.size_bytes) FROM items e WHERE e.series_id = t.id AND e.removed = 0))";
const SORTS: [(&str, &str); 4] = [("name", "t.name COLLATE NOCASE"), ("year", "t.production_year"), ("added", "t.date_created"), ("size", SIZE)];

/// The WHERE over `items t`, with the type filter or without it.
fn filter(b: &Browse, with_kind: bool) -> (String, Vec<SqlValue>) {
    let mut sql = format!("t.library_id = ? AND t.removed = 0 AND t.type IN ({TITLE_TYPES})");
    let mut args: Vec<SqlValue> = vec![crate::db::norm_id(&b.library_id).into()];
    if let Some(k) = b.kind.as_deref().filter(|k| with_kind && !k.is_empty()) {
        sql.push_str(" AND t.type = ?");
        args.push(k.to_string().into());
    }
    for like in crate::stats::like_words(b.q.as_deref()) {
        sql.push_str(" AND t.name LIKE ? ESCAPE '\\'");
        args.push(like.into());
    }
    (sql, args)
}

/// One page of titles, in `order`.
fn titles_sql(b: &Browse, order: &str) -> (String, Vec<SqlValue>) {
    let (wh, mut args) = filter(b, true);
    args.push(PAGE_SIZE.into());
    args.push(((b.page.max(1) - 1) * PAGE_SIZE).into());
    let sql = format!(
        "SELECT t.id, t.name, t.type, t.production_year, t.date_created, t.image_tag IS NOT NULL, t.runtime_s, t.album_artist, {SIZE},
                CASE WHEN t.type = 'Series' THEN (SELECT COUNT(*) FROM items e WHERE e.series_id = t.id AND e.type = 'Episode' AND e.removed = 0
                    AND (e.path IS NOT NULL OR e.size_bytes IS NOT NULL) AND COALESCE(e.parent_index_number, 1) > 0) END
         FROM items t WHERE {wh} ORDER BY {order} LIMIT ? OFFSET ?"
    );
    (sql, args)
}

/// A page of a library's titles, with how many there are and which kinds the library holds (whatever the filter).
pub fn titles(conn: &Connection, b: &Browse) -> Result<Value> {
    let (wh, args) = filter(b, true);
    let total: i64 = conn.query_row(&format!("SELECT COUNT(*) FROM items t WHERE {wh}"), params_from_iter(args.iter()), |r| r.get(0))?;
    let (all, args) = filter(&Browse { q: None, ..b.clone() }, false);
    let types: Vec<Value> = conn
        .prepare(&format!("SELECT t.type, COUNT(*) FROM items t WHERE {all} GROUP BY t.type ORDER BY 2 DESC, 1"))?
        .query_map(params_from_iter(args.iter()), |r| Ok(json!({ "type": r.get::<_, String>(0)?, "count": r.get::<_, i64>(1)? })))?
        .collect::<Result<_, _>>()?;
    let order = crate::stats::order_by(&SORTS, b.sort.as_deref(), b.dir.as_deref(), "t.name COLLATE NOCASE, t.id");
    let (sql, args) = titles_sql(b, &order);
    let items: Vec<Value> = conn
        .prepare(&sql)?
        .query_map(params_from_iter(args.iter()), |r| {
            let id: String = r.get(0)?;
            Ok(json!({
                "id": id, "name": r.get::<_, String>(1)?, "type": r.get::<_, String>(2)?, "year": r.get::<_, Option<i64>>(3)?,
                "date_created": r.get::<_, Option<i64>>(4)?, "has_image": r.get::<_, bool>(5)?, "image_item_id": id,
                "runtime_s": r.get::<_, Option<i64>>(6)?, "album_artist": r.get::<_, Option<String>>(7)?, "size_bytes": r.get::<_, Option<i64>>(8)?,
                "episodes": r.get::<_, Option<i64>>(9)?,
            }))
        })?
        .collect::<Result<_, _>>()?;
    Ok(json!({ "total": total, "page": b.page.max(1), "page_size": PAGE_SIZE, "types": types, "items": items }))
}

#[derive(Deserialize)]
pub struct TitlesQuery {
    #[serde(rename = "type")]
    kind: Option<String>,
    q: Option<String>,
    sort: Option<String>,
    dir: Option<String>,
    page: Option<i64>,
}

/// `GET /api/libraries/{id}/titles?type=&q=&sort=&dir=&page=`: every title in a library, a page at a time.
pub async fn get_titles(State(app): State<App>, _user: AuthUser, Path(id): Path<String>, Query(q): Query<TitlesQuery>) -> ApiResult {
    let b = Browse { library_id: id, kind: q.kind, q: q.q, sort: q.sort, dir: q.dir, page: q.page.unwrap_or(1) };
    Ok(Json(app.db.call(move |c| titles(c, &b)).await?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn library() -> Connection {
        let c = Connection::open_in_memory().unwrap();
        for m in crate::db::MIGRATIONS {
            c.execute_batch(m).unwrap();
        }
        c.execute_batch(
            "INSERT INTO items(id, library_id, type, name, production_year, date_created, size_bytes, runtime_s, image_tag, removed, updated_at) VALUES
               ('bbb', 'films', 'Movie', 'Big Buck Bunny', 2008, 300, 4000000000, 600, 'tag1', 0, 0),
               ('sin', 'films', 'Movie', 'sintel', 2010, 100, 2000000000, 888, NULL, 0, 0),
               ('tos', 'films', 'Movie', 'Tears of Steel', 2012, 200, 3000000000, 734, 'tag2', 0, 0),
               ('gone', 'films', 'Movie', 'Elephants Dream', 2006, 50, 1000, 600, NULL, 1, 0),
               ('lo', 'shows', 'Series', 'Low Orbit', 2020, 400, NULL, NULL, 'tag3', 0, 0),
               ('lo-s1', 'shows', 'Season', 'Season 1', 2020, 400, NULL, NULL, NULL, 0, 0),
               ('cos', 'films', 'Video', 'Cosmos Laundromat', 2015, 500, 500000000, 720, NULL, 0, 0);
             INSERT INTO items(id, library_id, type, name, series_id, parent_index_number, index_number, size_bytes, path, removed, updated_at) VALUES
               ('lo1', 'shows', 'Episode', 'One', 'lo', 1, 1, 700000000, '/s/1.mkv', 0, 0),
               ('lo2', 'shows', 'Episode', 'Two', 'lo', 1, 2, 800000000, '/s/2.mkv', 0, 0),
               ('lo3', 'shows', 'Episode', 'Three', 'lo', 1, 3, NULL, NULL, 0, 0);",
        )
        .unwrap();
        c
    }

    fn names(v: &Value) -> Vec<String> {
        v["items"].as_array().unwrap().iter().map(|i| i["name"].as_str().unwrap().to_string()).collect()
    }

    fn of(library: &str) -> Browse {
        Browse { library_id: library.into(), page: 1, ..Default::default() }
    }

    #[test]
    fn a_library_lists_its_titles_by_name_and_nothing_else() {
        let c = library();
        let v = titles(&c, &of("films")).unwrap();
        assert_eq!(names(&v), ["Big Buck Bunny", "Cosmos Laundromat", "sintel", "Tears of Steel"], "a title Jellyfin removed is not here, and case does not decide the order");
        assert_eq!(v["total"], 4);
        assert_eq!(v["types"], json!([{ "type": "Movie", "count": 3 }, { "type": "Video", "count": 1 }]));
        let shows = titles(&c, &of("shows")).unwrap();
        assert_eq!(names(&shows), ["Low Orbit"], "a show, not its seasons or episodes");
        let lo = &shows["items"][0];
        assert_eq!((lo["episodes"].as_i64(), lo["size_bytes"].as_i64()), (Some(2), Some(1_500_000_000)), "its episodes that are files, and what they weigh");
        let bbb = &v["items"][0];
        assert_eq!((bbb["id"].as_str(), bbb["year"].as_i64(), bbb["has_image"].as_bool(), bbb["image_item_id"].as_str()), (Some("bbb"), Some(2008), Some(true), Some("bbb")));
        assert_eq!(v["items"][2]["has_image"], false);
    }

    #[test]
    fn titles_are_sorted_filtered_and_paged_on_the_server() {
        let c = library();
        let sorted = |sort: &str, dir: &str| names(&titles(&c, &Browse { sort: Some(sort.into()), dir: Some(dir.into()), ..of("films") }).unwrap());
        assert_eq!(sorted("added", "desc"), ["Cosmos Laundromat", "Big Buck Bunny", "Tears of Steel", "sintel"]);
        assert_eq!(sorted("year", "asc"), ["Big Buck Bunny", "sintel", "Tears of Steel", "Cosmos Laundromat"]);
        assert_eq!(sorted("size", "desc"), ["Big Buck Bunny", "Tears of Steel", "sintel", "Cosmos Laundromat"]);
        assert_eq!(sorted("name", "desc"), ["Tears of Steel", "sintel", "Cosmos Laundromat", "Big Buck Bunny"]);
        assert_eq!(sorted("t.id; DROP TABLE items", "asc"), ["Big Buck Bunny", "Cosmos Laundromat", "sintel", "Tears of Steel"], "an unknown sort is by name");
        let found = titles(&c, &Browse { q: Some("of tears".into()), ..of("films") }).unwrap();
        assert_eq!((names(&found), found["total"].as_i64()), (vec!["Tears of Steel".to_string()], Some(1)), "every word, in any order");
        let videos = titles(&c, &Browse { kind: Some("Video".into()), ..of("films") }).unwrap();
        assert_eq!(names(&videos), ["Cosmos Laundromat"]);
        assert_eq!(videos["types"].as_array().unwrap().len(), 2, "the types a library holds, whatever the filter");
        let nothing = titles(&c, &Browse { kind: Some("Episode".into()), ..of("shows") }).unwrap();
        assert_eq!(nothing["total"], 0, "an episode is not a title of its own here");
    }

    #[test]
    fn a_page_is_sixty_titles_and_the_next_one_follows() {
        let c = library();
        let mut tx = String::from("INSERT INTO items(id, library_id, type, name, removed, updated_at) VALUES ");
        tx.push_str(&(0..130).map(|n| format!("('m{n:03}', 'big', 'Movie', 'Film {n:03}', 0, 0)")).collect::<Vec<_>>().join(", "));
        c.execute_batch(&tx).unwrap();
        let page = |p: i64| titles(&c, &Browse { page: p, ..of("big") }).unwrap();
        let (one, three) = (page(1), page(3));
        assert_eq!((one["total"].as_i64(), one["page_size"].as_i64(), one["items"].as_array().unwrap().len()), (Some(130), Some(PAGE_SIZE), 60));
        assert_eq!(names(&three), ["Film 120", "Film 121", "Film 122", "Film 123", "Film 124", "Film 125", "Film 126", "Film 127", "Film 128", "Film 129"]);
        assert_eq!(page(9)["items"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn a_library_is_read_by_its_index() {
        let c = library();
        let (sql, args) = titles_sql(&of("films"), "t.name COLLATE NOCASE");
        let plan: String = c
            .prepare(&format!("EXPLAIN QUERY PLAN {sql}"))
            .unwrap()
            .query_map(crate::db::rusqlite::params_from_iter(args.iter()), |r| r.get::<_, String>(3))
            .unwrap()
            .map(Result::unwrap)
            .collect::<Vec<_>>()
            .join(" | ");
        assert!(plan.contains("idx_items_library"), "{plan}");
    }
}
