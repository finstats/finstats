//! What of the library one person may see: what Jellyfin lets them see, and nothing more. Jellyfin keeps it in each user's
//! policy: every library or the ones named, and inside them a rating cap, blocked and allowed tags and kinds of unrated
//! item that hide titles. The libraries are a short list FinStats compares itself. What hides titles inside them is
//! Jellyfin's own judgement (a rating's number comes from Jellyfin's tables for the country it is set to, a tag from the
//! title or any folder above it), so FinStats does not redo it: it asks Jellyfin which titles that person is listed, and
//! keeps the list (`visible_items`). Jellyfin administrators see everything. Anything not yet read opens nothing.

use anyhow::Result;
use serde_json::{json, Value};

use crate::db::rusqlite::types::Value as SqlValue;
use crate::db::rusqlite::{params, Connection, OptionalExtension};
use crate::db::norm_id;

/// What one user's Jellyfin policy says about the library.
#[derive(Debug, Clone, PartialEq)]
pub struct Policy {
    /// Every library (Jellyfin's `EnableAllFolders`, and every administrator).
    pub all: bool,
    /// The libraries named, when not all of them.
    pub libraries: Vec<String>,
    /// What hides titles inside those libraries, as one stable text; `None` when nothing does.
    pub hides: Option<String>,
}

/// Read a user's policy from Jellyfin's `/Users` entry (or the `User` of a sign-in). A policy FinStats cannot read opens
/// nothing: a Jellyfin that answers in a shape it does not know must not open every library to everybody.
pub fn policy(user: &Value) -> Policy {
    let p = &user["Policy"];
    if p["IsAdministrator"].as_bool() == Some(true) {
        return Policy { all: true, libraries: vec![], hides: None };
    }
    let all = p["EnableAllFolders"].as_bool() == Some(true);
    let mut libraries: Vec<String> = if all { vec![] } else { p["EnabledFolders"].as_array().map(|a| a.iter().filter_map(Value::as_str).map(norm_id).filter(|s| !s.is_empty()).collect()).unwrap_or_default() };
    libraries.sort();
    libraries.dedup();
    let list = |k: &str| -> Vec<String> {
        let mut v: Vec<String> = p[k].as_array().map(|a| a.iter().filter_map(Value::as_str).map(|s| s.trim().to_lowercase()).filter(|s| !s.is_empty()).collect()).unwrap_or_default();
        v.sort();
        v.dedup();
        v
    };
    let (blocked, allowed, unrated) = (list("BlockedTags"), list("AllowedTags"), list("BlockUnratedItems"));
    let (rating, sub) = (p["MaxParentalRating"].as_i64(), p["MaxParentalSubRating"].as_i64());
    let hides = (rating.is_some() || sub.is_some() || !blocked.is_empty() || !allowed.is_empty() || !unrated.is_empty())
        .then(|| json!({ "rating": rating, "sub_rating": sub, "blocked_tags": blocked, "allowed_tags": allowed, "unrated": unrated }).to_string());
    Policy { all, libraries, hides }
}

/// Keep what Jellyfin just said about one user. A changed `hides` leaves the list read under the old one counting for
/// nothing until it is read again (`visible_for`). Answers whether anything changed, which is when that person's open
/// connections must look again.
pub fn store(c: &Connection, user_id: &str, p: &Policy) -> Result<bool> {
    let changed = c.execute(
        "UPDATE users SET access_all = ?2, access_libraries = ?3, access_hides = ?4
         WHERE id = ?1 AND (access_all IS NOT ?2 OR access_libraries IS NOT ?3 OR access_hides IS NOT ?4)",
        params![user_id, p.all, serde_json::to_string(&p.libraries)?, p.hides],
    )?;
    Ok(changed > 0)
}

/// One person whose list of titles is to be read: their id, the policy it is read under, and their libraries (`None`: all).
pub type ToList = (String, String, Option<Vec<String>>);

/// A user's stored access: every library, the ones named, what hides titles, and the policy their list was read under.
type Row = (Option<bool>, Option<String>, Option<String>, Option<String>);

/// The people whose list of titles must be read from Jellyfin: something hides titles from them. With `stale_only`, only
/// those whose list was read under another policy, or never.
pub fn to_list(c: &Connection, stale_only: bool) -> Result<Vec<ToList>> {
    let mut stmt = c.prepare(
        "SELECT id, access_hides, access_all, access_libraries FROM users
         WHERE removed = 0 AND is_admin = 0 AND access_hides IS NOT NULL AND (?1 = 0 OR visible_for IS NOT access_hides)
         ORDER BY id",
    )?;
    let rows = stmt.query_map([stale_only], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, Option<bool>>(2)?, r.get::<_, Option<String>>(3)?)))?;
    let mut out = vec![];
    for row in rows {
        let (id, hides, all, libs) = row?;
        let libs = if all == Some(true) { None } else { Some(libs.and_then(|l| serde_json::from_str(&l).ok()).unwrap_or_default()) };
        out.push((id, hides, libs));
    }
    Ok(out)
}

/// Keep the titles Jellyfin listed for one person, read under the policy `hides`. A title the library no longer has keeps
/// the answer it had, so that person's own history of it still opens; everything else is replaced by this list.
pub fn store_visible(c: &mut Connection, user_id: &str, hides: &str, ids: &[String]) -> Result<()> {
    let tx = c.transaction()?;
    // Read under a policy that has changed since: it says nothing about this one.
    let current: Option<Option<String>> = tx.query_row("SELECT access_hides FROM users WHERE id = ?1", [user_id], |r| r.get(0)).optional()?;
    if current.flatten().as_deref() != Some(hides) {
        return Ok(());
    }
    tx.execute("DELETE FROM visible_items WHERE user_id = ?1 AND item_id IN (SELECT id FROM items WHERE removed = 0)", [user_id])?;
    {
        let mut ins = tx.prepare("INSERT OR IGNORE INTO visible_items(user_id, item_id) VALUES (?1, ?2)")?;
        for id in ids {
            ins.execute(params![user_id, norm_id(id)])?;
        }
    }
    tx.execute("UPDATE users SET visible_for = ?2 WHERE id = ?1", params![user_id, hides])?;
    tx.commit()?;
    Ok(())
}

/// What of the library one person may see.
#[derive(Debug, Clone, PartialEq)]
pub struct Access {
    /// The libraries they may open; `None` is every one.
    libraries: Option<Vec<String>>,
    /// Inside them, only the titles Jellyfin listed for this user (`visible_items`).
    listed_for: Option<String>,
}

/// Nothing: a place that forgets to say whose access it is shows no library at all.
impl Default for Access {
    fn default() -> Self {
        Self::nothing()
    }
}

impl Access {
    /// Everything: a Jellyfin administrator, and work that is nobody's (a scheduled job).
    pub fn everything() -> Self {
        Access { libraries: None, listed_for: None }
    }

    /// What one signed-in person may see.
    pub fn for_user(c: &Connection, user: &crate::auth::AuthUser) -> Result<Self> {
        Self::of(c, &user.id, user.is_admin)
    }

    /// Nothing at all.
    pub fn nothing() -> Self {
        Access { libraries: Some(vec![]), listed_for: None }
    }

    pub fn is_everything(&self) -> bool {
        self.libraries.is_none() && self.listed_for.is_none()
    }

    /// The rule for one person, from what the users read last stored. An administrator sees everything whatever their row
    /// says (the wizard's first one signs in before any read has written it). Anybody else whose policy has not been read,
    /// or whose list of titles was read under another policy, sees nothing until it is.
    pub fn of(c: &Connection, user_id: &str, is_admin: bool) -> Result<Self> {
        if is_admin {
            return Ok(Self::everything());
        }
        let row: Option<Row> = c
            .query_row("SELECT access_all, access_libraries, access_hides, visible_for FROM users WHERE id = ?1", [user_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
            .optional()?;
        let Some((Some(all), libs, hides, read_for)) = row else { return Ok(Self::nothing()) };
        let libraries = if all { None } else { Some(libs.and_then(|l| serde_json::from_str::<Vec<String>>(&l).ok()).unwrap_or_default()) };
        let listed_for = match hides {
            None => None,
            Some(h) if read_for.as_deref() == Some(h.as_str()) => Some(user_id.to_string()),
            Some(_) => return Ok(Self::nothing()),
        };
        Ok(Access { libraries, listed_for })
    }

    pub fn may_see_library(&self, library_id: &str) -> bool {
        self.libraries.as_ref().is_none_or(|l| l.iter().any(|x| x == library_id))
    }

    /// A clause over the `items` row aliased `alias`, with its arguments, which every place that shows a title adds.
    /// `"1"` and nothing to bind for everything. The libraries are one bound list, the titles a lookup by key: both
    /// leave the query's own index to do its work.
    pub fn items_sql(&self, alias: &str) -> (String, Vec<SqlValue>) {
        let mut parts = vec![];
        let mut args: Vec<SqlValue> = vec![];
        if let Some(l) = &self.libraries {
            // `+`: a narrowing, never what the planner walks, or the shelf would leave its date index for the libraries'.
            parts.push(format!("+{alias}.library_id IN (SELECT value FROM json_each(?))"));
            args.push(serde_json::to_string(l).unwrap_or_else(|_| "[]".into()).into());
        }
        if let Some(u) = &self.listed_for {
            parts.push(format!("EXISTS (SELECT 1 FROM visible_items v WHERE v.user_id = ? AND v.item_id = {alias}.id)"));
            args.push(u.clone().into());
        }
        if parts.is_empty() { ("1".into(), args) } else { (format!("({})", parts.join(" AND ")), args) }
    }

    /// The same for the `libraries` row aliased `alias`.
    pub fn libraries_sql(&self, alias: &str) -> (String, Vec<SqlValue>) {
        match &self.libraries {
            None => ("1".into(), vec![]),
            Some(l) => (format!("{alias}.id IN (SELECT value FROM json_each(?))"), vec![serde_json::to_string(l).unwrap_or_else(|_| "[]".into()).into()]),
        }
    }

    /// Whether this person may see the title `id`. A title FinStats does not hold in its library (known only from plays)
    /// carries nothing of the library to hide, so it is not refused here; whatever shows its plays scopes them already.
    pub fn may_see_item(&self, c: &Connection, id: &str) -> Result<bool> {
        if self.is_everything() {
            return Ok(true);
        }
        let (clause, mut args) = self.items_sql("i");
        args.insert(0, id.to_string().into());
        let sql = format!("SELECT CASE WHEN EXISTS (SELECT 1 FROM items WHERE id = ?1) THEN EXISTS (SELECT 1 FROM items i WHERE i.id = ?1 AND {clause}) ELSE 1 END");
        Ok(c.query_row(&sql, crate::db::rusqlite::params_from_iter(args.iter()), |r| r.get(0))?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn conn() -> Connection {
        let c = Connection::open_in_memory().unwrap();
        for m in crate::db::MIGRATIONS {
            c.execute_batch(m).unwrap();
        }
        c.execute_batch(
            "INSERT INTO libraries(id, name, updated_at) VALUES ('films', 'Films', 0), ('adult', 'Late films', 0);
             INSERT INTO items(id, library_id, type, name, removed, updated_at) VALUES
               ('bbb', 'films', 'Movie', 'Big Buck Bunny', 0, 0),
               ('sin', 'films', 'Movie', 'Sintel', 0, 0),
               ('late', 'adult', 'Movie', 'Elephants Dream', 0, 0),
               ('gone', 'films', 'Movie', 'Cosmos Laundromat', 1, 0);
             INSERT INTO users(id, name, is_admin, updated_at) VALUES ('alice', 'alice', 1, 0), ('bob', 'bob', 0, 0), ('kid', 'kid', 0, 0);",
        )
        .unwrap();
        c
    }

    fn sees(c: &Connection, a: &Access) -> Vec<String> {
        let (clause, args) = a.items_sql("i");
        let mut stmt = c.prepare(&format!("SELECT i.id FROM items i WHERE {clause} ORDER BY i.id")).unwrap();
        stmt.query_map(crate::db::rusqlite::params_from_iter(args.iter()), |r| r.get(0)).unwrap().map(Result::unwrap).collect()
    }

    #[test]
    fn a_policy_names_its_libraries_and_what_hides_titles() {
        let p = policy(&json!({ "Policy": { "IsAdministrator": false, "EnableAllFolders": false, "EnabledFolders": ["F1F1-AAAA", "f1f1aaaa", "B2"], "MaxParentalRating": 10, "BlockUnratedItems": ["Movie"] } }));
        assert_eq!((p.all, p.libraries.clone()), (false, vec!["b2".to_string(), "f1f1aaaa".to_string()]), "ids as FinStats writes them, each once");
        assert!(p.hides.is_some(), "a rating cap hides titles");
        let open = policy(&json!({ "Policy": { "EnableAllFolders": true, "EnabledFolders": [], "BlockedTags": [], "AllowedTags": [], "BlockUnratedItems": [], "MaxParentalRating": null } }));
        assert_eq!(open, Policy { all: true, libraries: vec![], hides: None }, "Jellyfin's defaults hide nothing");
        let tagged = policy(&json!({ "Policy": { "EnableAllFolders": true, "BlockedTags": ["Horror"] } }));
        assert!(tagged.hides.is_some(), "a blocked tag hides titles");
        let admin = policy(&json!({ "Policy": { "IsAdministrator": true, "EnableAllFolders": false, "MaxParentalRating": 1 } }));
        assert_eq!(admin, Policy { all: true, libraries: vec![], hides: None }, "an administrator sees everything");
        assert_eq!(policy(&json!({ "Id": "x" })), Policy { all: false, libraries: vec![], hides: None }, "a policy that cannot be read opens nothing");
    }

    #[test]
    fn a_person_sees_only_the_libraries_jellyfin_opens_to_them() {
        let c = conn();
        assert_eq!(sees(&c, &Access::of(&c, "alice", true).unwrap()), ["bbb", "gone", "late", "sin"], "an administrator, whatever the row says");
        assert_eq!(sees(&c, &Access::of(&c, "bob", false).unwrap()), Vec::<String>::new(), "a policy never read opens nothing");
        assert_eq!(sees(&c, &Access::of(&c, "nobody", false).unwrap()), Vec::<String>::new(), "nor does a person with no row");
        store(&c, "bob", &Policy { all: false, libraries: vec!["films".into()], hides: None }).unwrap();
        let bob = Access::of(&c, "bob", false).unwrap();
        assert_eq!(sees(&c, &bob), ["bbb", "gone", "sin"]);
        assert!(bob.may_see_library("films") && !bob.may_see_library("adult"));
        assert!(!bob.may_see_item(&c, "late").unwrap() && bob.may_see_item(&c, "bbb").unwrap());
        assert!(bob.may_see_item(&c, "only-in-plays").unwrap(), "a title the library does not hold has nothing of it to hide");
        store(&c, "bob", &Policy { all: true, libraries: vec![], hides: None }).unwrap();
        assert!(Access::of(&c, "bob", false).unwrap().is_everything());
    }

    #[test]
    fn storing_a_policy_says_whether_it_changed_anything() {
        // A sign-in asks the person's open connections to look again only when something changed, this included.
        let c = conn();
        let films = Policy { all: false, libraries: vec!["films".into()], hides: None };
        assert!(store(&c, "bob", &films).unwrap(), "a first policy is a change");
        assert!(!store(&c, "bob", &films).unwrap(), "the same one again is not");
        assert!(store(&c, "bob", &Policy { hides: Some("rating<=10".into()), ..films }).unwrap(), "a rating cap is");
    }

    #[test]
    fn what_hides_titles_is_jellyfins_own_list_and_only_while_the_policy_holds() {
        let mut c = conn();
        let capped = Policy { all: true, libraries: vec![], hides: Some("{\"rating\":10}".into()) };
        store(&c, "kid", &capped).unwrap();
        assert_eq!(sees(&c, &Access::of(&c, "kid", false).unwrap()), Vec::<String>::new(), "not yet asked Jellyfin: nothing");
        assert_eq!(to_list(&c, true).unwrap().len(), 1, "and that list is due");
        c.execute("INSERT INTO visible_items(user_id, item_id) VALUES ('kid', 'gone')", []).unwrap();
        store_visible(&mut c, "kid", "{\"rating\":10}", &["BBB".into()]).unwrap();
        assert_eq!(sees(&c, &Access::of(&c, "kid", false).unwrap()), ["bbb", "gone"], "what Jellyfin listed, and a removed title keeps what it had");
        assert!(to_list(&c, true).unwrap().is_empty(), "read under the policy that holds");
        store(&c, "kid", &Policy { hides: Some("{\"rating\":13}".into()), ..capped.clone() }).unwrap();
        assert_eq!(sees(&c, &Access::of(&c, "kid", false).unwrap()), Vec::<String>::new(), "a changed policy: the old list counts for nothing");
        store_visible(&mut c, "kid", "{\"rating\":10}", &["sin".into()]).unwrap();
        assert_eq!(sees(&c, &Access::of(&c, "kid", false).unwrap()), Vec::<String>::new(), "nor does a list read under the old one");
    }
}
