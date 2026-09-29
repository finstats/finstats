//! Periodic copies of Jellyfin's users, libraries/items and server activity log.

use std::time::Duration;

use anyhow::{Result, anyhow};
use serde_json::{Value, json};

use crate::db::{self, norm_id, parse_ts, rusqlite::Connection, rusqlite::params};
use crate::jellyfin::Jellyfin;
use crate::media::{Streams, int, ticks_to_s};
use crate::state::App;

const PAGE: usize = 500;
/// Library kinds that only reference items living in other libraries.
const SKIPPED_COLLECTIONS: [&str; 2] = ["boxsets", "playlists"];

/// Whether a read is a trustworthy basis for removing what it did not return. `items_page` turns
/// anything it cannot parse into an empty list, so a Jellyfin upgrade that changed the response shape,
/// or an error dressed as `200 {"Items":[]}`, would otherwise silently wipe a library. But this guard
/// exists for that "Jellyfin broke my program" case, **not** for ordinary churn — a small library
/// genuinely losing most of its items (a handful of clips whose files went) must not be mistaken for a
/// fault, or the guard turns a normal day into an outage. So only a *clearly* broken read is refused:
/// a library big enough to matter read back completely empty, or a big one gutted to almost nothing.
/// Everything else applies as before. `FINSTATS_ALLOW_LIBRARY_SHRINK=1` waves even a refused one
/// through — after genuinely emptying a large library, say.
const EMPTY_FLOOR: i64 = 200; // a fully-empty read is only alarming once a library is at least this big
const WIPE_FLOOR: i64 = 1000; // ...and a partial gutting only when it would remove at least this many
pub(crate) fn trustworthy_removal(seen: usize, current: i64) -> bool {
    if current <= 0 {
        return true; // nothing stored yet: a first sync, or a genuinely new set.
    }
    let seen = seen as i64;
    if seen == 0 {
        // A sizeable library read back completely empty is the classic broken read; a tiny one going
        // empty is just a tiny one going empty.
        return current < EMPTY_FLOOR;
    }
    // Otherwise refuse only a large, near-total wipe: a four-figure loss that leaves under a twentieth
    // of the library standing. A big library dropping to a handful is a fault; ordinary churn is not.
    let would_remove = current - seen;
    !(would_remove >= WIPE_FLOOR && seen.saturating_mul(20) < current)
}

/// A whole set gone at once — Jellyfin listing zero libraries, or zero users, where finstats holds
/// some — is a broken read (an auth failure, an API change), never a normal day: you do not lose every
/// library at once. Unlike the item guard there is no "big enough to matter" floor, because a set going
/// entirely empty is the catastrophe whatever its size. Losing *some* (one library of five) is normal.
fn whole_set_vanished(seen: usize, current: i64) -> bool {
    current > 0 && seen == 0
}

fn allow_shrink() -> bool {
    std::env::var("FINSTATS_ALLOW_LIBRARY_SHRINK").is_ok_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
}

fn opt_str(v: &Value) -> Option<String> {
    v.as_str().map(str::trim).filter(|s| !s.is_empty()).map(str::to_string)
}

/// Runs a task unless it is already running. Returns false in that case.
/// The tasks that read from Jellyfin. (`services::TASKS` are the ones that read from Sonarr and friends.)
pub const TASKS: [&str; 6] = ["sync_users", "sync_libraries", "sync_events", "sync_server", "sync_userdata", "sync_changes"];

pub fn spawn(app: &App, id: &'static str) -> bool {
    spawn_within(app, id, None)
}

/// `spawn`, stopped once it has run for `limit` (`schedule::within`): the time limit of the trigger that started it.
pub fn spawn_within(app: &App, id: &'static str, limit: Option<Duration>) -> bool {
    if !TASKS.contains(&id) {
        return false;
    }
    let Some(jf) = app.jellyfin() else { return false };
    if !app.tasks.try_start(id, "Starting…") {
        return false;
    }
    let app = app.clone();
    tokio::spawn(async move {
        let outcome = crate::schedule::within(limit, async { match id {
            "sync_users" => sync_users(&app, &jf).await,
            "sync_libraries" => {
                let done = sync_libraries(&app, &jf).await;
                announce_new_items(&app).await;
                // An episode arriving is also how a request becomes watchable.
                crate::seerr::check_available(&app).await;
                done
            }
            "sync_events" => {
                let done = sync_events(&app, &jf).await;
                // New sign-ins are new sightings, and several failed ones in a row are news of their own.
                crate::security::check(&app, None).await;
                crate::security::check_sign_ins(&app).await;
                done
            }
            "sync_server" => sync_server(&app, &jf).await,
            "sync_userdata" => sync_userdata(&app, &jf).await,
            "sync_changes" => {
                let done = sync_changes(&app, &jf).await;
                announce_new_items(&app).await;
                done
            }
            other => Err(anyhow!("unknown task {other}")),
        } })
        .await;
        if let Err(e) = &outcome {
            crate::notify::task_failed(&app, id, &format!("{e:#}")).await;
        }
        app.tasks.finish(id, outcome.map(|m| (m, None)));
    });
    true
}

/// After a library read: what arrived, as notifications. Never a reason for the read to fail.
async fn announce_new_items(app: &App) {
    let bus_app = app.clone();
    let done = app.db.call(move |c| {
        let bus = crate::notify::Fanout::of(c, &bus_app)?;
        crate::recent::announce(c, &bus)
    }).await;
    match done {
        Ok(n) if n > 0 => app.notify_wake.notify_one(),
        Ok(_) => {}
        Err(e) => tracing::warn!("could not announce what arrived: {e:#}"),
    }
}

/// In December, each person's year in review is ready: say so, once. Outside December it reads nothing.
async fn announce_ready_year(app: &App) {
    let bus_app = app.clone();
    let done = app.db.call(move |c| {
        let today: String = c.query_row("SELECT date('now', 'localtime')", [], |r| r.get(0))?;
        let today = chrono::NaiveDate::parse_from_str(&today, "%Y-%m-%d")?;
        if chrono::Datelike::month(&today) != 12 {
            return Ok(0);
        }
        let bus = crate::notify::Fanout::of(c, &bus_app)?;
        crate::recap::announce_ready(c, &bus, today, bus_app.settings().min_play_s)
    }).await;
    match done {
        Ok(n) if n > 0 => app.notify_wake.notify_one(),
        Ok(_) => {}
        Err(e) => tracing::warn!("could not announce the year in review: {e:#}"),
    }
}

/// Housekeeping that is not a job of its own: the server's name and version, a GeoIP file dropped in by hand,
/// whether every connection still answers, the audit log's age.
const HOUSEKEEPING_EVERY_S: i64 = 900;
const SCAN_CHECK_EVERY_S: i64 = 300;

/// Write a backup in the background and thin out old ones. Returns false when one is already being written.
pub fn run_backup(app: &App, actor: Option<crate::audit::Actor>) -> bool {
    const ID: &str = "backup";
    if !app.tasks.try_start(ID, "Writing backup") {
        return false;
    }
    let app = app.clone();
    let automatic = actor.is_none();
    tokio::task::spawn_blocking(move || {
        let dir = crate::backup::dir(&app.data_dir);
        let outcome = crate::backup::export(&app.db, &dir, Some((&app.tasks, ID))).map(|made| {
            let removed = crate::backup::prune(&dir, app.settings().backup_keep.clamp(1, 100) as usize);
            tracing::info!("{} backup written: {} ({} plays){}", if automatic { "automatic" } else { "manual" }, made.name, made.plays,
                if removed > 0 { format!("; removed {removed} old") } else { String::new() });
            (format!("{} plays, {:.1} MB", made.plays, made.size_bytes as f64 / 1e6), serde_json::to_value(&made).ok())
        });
        if let Ok(c) = app.db.conn() {
            let who = actor.unwrap_or_default();
            let entry = match &outcome {
                Ok((_, made)) => crate::audit::Entry::new("backup_made", who).target(made.as_ref().and_then(|m| m["name"].as_str()).unwrap_or("").to_string())
                    .detail(serde_json::json!({ "trigger": if automatic { "schedule" } else { "request" }, "plays": made.as_ref().and_then(|m| m["plays"].as_i64()), "size_bytes": made.as_ref().and_then(|m| m["size_bytes"].as_i64()) })),
                Err(e) => crate::audit::Entry::new("backup_made", who).detail(serde_json::json!({ "trigger": if automatic { "schedule" } else { "request" }, "error": format!("{e:#}") })).outcome("failed"),
            };
            crate::audit::record_quietly(&c, &entry);
        }
        if let Err(e) = &outcome {
            tracing::error!("backup failed: {e:#}");
            let (app, message) = (app.clone(), format!("{e:#}"));
            tokio::spawn(async move { crate::notify::backup_failed(&app, &message).await });
        }
        app.tasks.finish(ID, outcome);
    });
    true
}

/// Runs every job on its triggers (`schedule`), looking once a minute or when woken. finstats only ever *reads*
/// from Jellyfin; it never starts a scan there — "after Jellyfin's library scan" waits for Jellyfin's own.
pub async fn scheduler(app: App) {
    let mut last_housekeeping = 0i64;
    let mut last_scan_check = 0i64;
    // Jellyfin's "Scan Media Library": (running now, when it last finished), from the last read of its task list.
    let mut scan: Option<(bool, Option<i64>)> = None;
    // A library read due while Jellyfin scans waits for the scan to end: read now, it would be half old, half new.
    let mut deferred_library = false;
    let newest_backup = crate::backup::newest_at(&crate::backup::dir(&app.data_dir));
    let known = app
        .db
        .call(move |c| {
            let library_read = db::get_setting(c, "library_synced_at")?.and_then(|v| v.parse().ok());
            Ok(crate::schedule::seed(crate::schedule::load_runs(c)?, library_read, newest_backup))
        })
        .await;
    match known {
        Ok(runs) => app.tasks.remember(runs),
        Err(e) => tracing::warn!("could not read when the jobs last ran: {e:#}"),
    }
    let mut saved = String::new();
    let mut from = db::now();
    let mut startup = true;
    loop {
        let now = db::now();
        let jf = app.jellyfin();
        if let Some(jf) = &jf {
            if now - last_housekeeping >= HOUSEKEEPING_EVERY_S {
                last_housekeeping = now;
                refresh_server_info(&app).await;
                announce_ready_year(&app).await;
                crate::geo::pick_up(&app).await;
                crate::services::check_all(&app).await;
                // A year of finstats' own audit log is enough to answer "who changed this in spring".
                let _ = app.db.call(|c| crate::audit::thin(c, db::now())).await;
            }
            if now - last_scan_check >= SCAN_CHECK_EVERY_S {
                last_scan_check = now;
                // One read, two answers: whether a scan is on, and where every running job has got to.
                // Timing a run costs nothing once the list is in hand, and it is what lets the Jellyfin
                // jobs card open on an estimate instead of on an ellipsis.
                scan = match jf.scheduled_tasks().await {
                    Ok(tasks) => {
                        crate::jobs::observe(&app, &tasks);
                        crate::jellyfin::scan_status(&tasks)
                    }
                    Err(_) => None, // Jellyfin unreachable; the collector already reports that
                };
            }
        }
        let scanning = matches!(scan, Some((true, _)));
        let look = crate::schedule::Look { from, to: now, startup, scan_done: match scan { Some((false, done)) => done, _ => None } };
        for (task, limit) in crate::schedule::due(&app.settings(), &app.tasks.snapshot(), &look, &chrono::Local) {
            start_scheduled(&app, task, limit, scanning, &mut deferred_library).await;
        }
        if deferred_library && !scanning && spawn(&app, "sync_libraries") {
            deferred_library = false;
            tracing::info!("Jellyfin's library scan is over; reading the library");
        }
        // What each job last did, kept for the next start — written only when something finished since.
        let snapshot = app.tasks.snapshot();
        let finished = serde_json::to_string(&snapshot.iter().map(|t| (t.id, t.finished_at)).collect::<Vec<_>>()).unwrap_or_default();
        if finished != saved {
            match app.db.call(move |c| crate::schedule::save_runs(c, &snapshot)).await {
                Ok(_) => saved = finished,
                Err(e) => tracing::warn!("could not keep when the jobs last ran: {e:#}"),
            }
        }
        (from, startup) = (now, false);
        tokio::select! {
            _ = app.wake.notified() => { last_scan_check = 0; }
            _ = tokio::time::sleep(Duration::from_secs(60)) => {}
        }
    }
}

/// Starts one job a trigger set off, if there is anything for it to do: a job that reads from a service nobody
/// connected, or a backup of a database without a single play, is skipped rather than run to say so.
async fn start_scheduled(app: &App, task: &'static str, limit_s: Option<i64>, scanning: bool, deferred_library: &mut bool) {
    let limit = limit_s.map(|s| Duration::from_secs(s.max(0) as u64));
    match task {
        "sync_libraries" if scanning => {
            if !*deferred_library {
                tracing::info!("the library read waits for Jellyfin's library scan to finish");
            }
            *deferred_library = true;
        }
        "backup" => {
            let has_plays = app.db.call(|c| Ok(c.query_row("SELECT EXISTS(SELECT 1 FROM playbacks)", [], |r| r.get::<_, bool>(0))?)).await.unwrap_or(false);
            if has_plays {
                run_backup(app, None);
            }
        }
        "geoip" => {
            crate::geo::download_if_stale(app);
        }
        "sync_upcoming" | "sync_grabs" if crate::services::enabled(app, crate::services::Kind::is_arr).is_empty() => {}
        "sync_requests" if !crate::seerr::connected(app) => {}
        t if TASKS.contains(&t) => {
            spawn_within(app, t, limit);
        }
        t => {
            crate::services::spawn_within(app, t, limit);
        }
    }
}

async fn refresh_server_info(app: &App) {
    let Some(jf) = app.jellyfin() else { return };
    let Ok(info) = jf.system_info().await else { return };
    let (name, version) = (opt_str(&info["ServerName"]), opt_str(&info["Version"]));
    {
        let mut cfg = app.config.write().unwrap();
        if let Some(c) = cfg.as_mut() {
            if let Some(n) = &name {
                c.server_name = n.clone();
            }
            if let Some(v) = &version {
                c.server_version = v.clone();
            }
        }
    }
    let _ = app
        .db
        .call(move |c| {
            if let Some(n) = name {
                db::set_setting(c, "server_name", &n)?;
            }
            if let Some(v) = version {
                db::set_setting(c, "server_version", &v)?;
            }
            Ok(())
        })
        .await;
}

// ---------------------------------------------------------------- users

/// One read of Jellyfin's users, written down. `Some(known)` = refused: the read came back empty where
/// finstats knows people, which is a broken read, not everybody deleted.
pub(crate) fn store_users(c: &mut Connection, users: &[Value], shrink_ok: bool, now: i64) -> Result<Option<i64>> {
    let tx = c.transaction()?;
    {
        let mut stmt = tx.prepare(
            "INSERT INTO users(id, name, is_admin, is_disabled, image_tag, last_login_at, last_activity_at, removed, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0, ?8)
             ON CONFLICT(id) DO UPDATE SET name = excluded.name, is_admin = excluded.is_admin,
                is_disabled = excluded.is_disabled, image_tag = excluded.image_tag,
                last_login_at = excluded.last_login_at, last_activity_at = excluded.last_activity_at,
                removed = 0, updated_at = excluded.updated_at",
        )?;
        for u in users {
            let Some(id) = u["Id"].as_str() else { continue };
            stmt.execute(params![
                norm_id(id),
                u["Name"].as_str().unwrap_or("Unknown"),
                u["Policy"]["IsAdministrator"].as_bool().unwrap_or(false),
                u["Policy"]["IsDisabled"].as_bool().unwrap_or(false),
                opt_str(&u["PrimaryImageTag"]),
                u["LastLoginDate"].as_str().and_then(parse_ts),
                u["LastActivityDate"].as_str().and_then(parse_ts),
                now,
            ])?;
        }
    }
    // The same guard as libraries and items: an empty /Users where finstats knows people is a
    // broken read, not everyone deleted — do not mark them all removed.
    // Counted by the ids it could read, not by entries: a reshaped answer is everybody in keys finstats
    // does not know, and that is a broken read too.
    let seen: Vec<String> = users.iter().filter_map(|u| u["Id"].as_str()).map(norm_id).collect();
    let current: i64 = tx.query_row("SELECT COUNT(*) FROM users WHERE removed = 0", [], |r| r.get(0))?;
    if !shrink_ok && whole_set_vanished(seen.len(), current) {
        tx.rollback()?;
        return Ok(Some(current));
    }
    // Removed = not in this read, by id. Comparing `updated_at` with a clock in whole seconds could not
    // tell a row this read wrote from one the read before it wrote in the same second, and a session now
    // ends with its user's removal, so a deleted user must be seen as deleted by the read that missed them.
    tx.execute("UPDATE users SET removed = 1 WHERE removed = 0 AND id NOT IN (SELECT value FROM json_each(?1))", [serde_json::to_string(&seen)?])?;
    tx.commit()?;
    Ok(None)
}

async fn sync_users(app: &App, jf: &Jellyfin) -> Result<String> {
    app.tasks.update("sync_users", "Fetching users", None);
    let users = jf.users().await?;
    let count = users.iter().filter(|u| u["Id"].is_string()).count();
    let shrink_ok = allow_shrink();
    let refused = app.db.call(move |c| store_users(c, &users, shrink_ok, db::now())).await?;
    if let Some(current) = refused {
        let msg = format!(
            "Jellyfin returned {count} user(s) where finstats knows {current}. Refusing to mark the missing ones removed — this looks like a Jellyfin change or a bad read. Nothing was changed."
        );
        app.request_halt(msg.clone());
        return Err(anyhow!(msg));
    }
    Ok(format!("{count} users"))
}

// ---------------------------------------------------------------- libraries & items

/// `false` when there was nothing to store: an entry without an id.
pub fn upsert_item(conn: &Connection, library_id: &str, it: &Value, now: i64) -> Result<bool> {
    let Some(id) = it["Id"].as_str() else { return Ok(false) };
    let source = &it["MediaSources"][0];
    let streams = Streams::extract(&source["MediaStreams"], &Value::Null);
    let genres = it["Genres"].as_array().filter(|g| !g.is_empty()).map(|g| Value::Array(g.clone()).to_string());
    let provider_ids = it["ProviderIds"].as_object().filter(|p| !p.is_empty()).map(|_| it["ProviderIds"].to_string());
    let studios = it["Studios"]
        .as_array()
        .map(|a| a.iter().filter_map(|x| x["Name"].as_str()).collect::<Vec<_>>())
        .filter(|a| !a.is_empty())
        .map(|a| json!(a).to_string());
    let framerate = source["MediaStreams"]
        .as_array()
        .and_then(|a| a.iter().find(|x| x["Type"].as_str() == Some("Video")))
        .and_then(|v| v["AverageFrameRate"].as_f64().or_else(|| v["RealFrameRate"].as_f64()))
        .map(|f| (f * 1000.0).round() / 1000.0);
    conn.prepare_cached(
        "INSERT INTO items(id, library_id, type, name, series_id, season_id, series_name, index_number, parent_index_number,
            album, album_artist, runtime_s, production_year, premiere_date, date_created, community_rating, official_rating,
            genres, overview, image_tag, backdrop_tag, container, path, size_bytes, bitrate,
            video_codec, width, height, video_range, audio_codec, audio_channels, provider_ids, studios, bit_depth, framerate, audio_languages, subtitle_languages, removed, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23,
            ?24, ?25, ?26, ?27, ?28, ?29, ?30, ?31, ?33, ?34, ?35, ?36, ?37, ?38, 0, ?32)
         ON CONFLICT(id) DO UPDATE SET library_id = excluded.library_id, type = excluded.type, name = excluded.name,
            series_id = excluded.series_id, season_id = excluded.season_id, series_name = excluded.series_name,
            index_number = excluded.index_number, parent_index_number = excluded.parent_index_number,
            album = excluded.album, album_artist = excluded.album_artist, runtime_s = excluded.runtime_s,
            production_year = excluded.production_year, premiere_date = excluded.premiere_date,
            date_created = excluded.date_created, community_rating = excluded.community_rating,
            official_rating = excluded.official_rating, genres = excluded.genres, overview = excluded.overview,
            image_tag = excluded.image_tag, backdrop_tag = excluded.backdrop_tag, container = excluded.container,
            path = excluded.path, size_bytes = excluded.size_bytes, bitrate = excluded.bitrate,
            video_codec = excluded.video_codec, width = excluded.width, height = excluded.height,
            video_range = excluded.video_range, audio_codec = excluded.audio_codec,
            audio_channels = excluded.audio_channels, provider_ids = excluded.provider_ids, studios = excluded.studios,
            bit_depth = excluded.bit_depth, framerate = excluded.framerate,
            audio_languages = excluded.audio_languages, subtitle_languages = excluded.subtitle_languages, removed = 0, updated_at = excluded.updated_at",
    )?
    .execute(params![
        norm_id(id),
        library_id,
        it["Type"].as_str().unwrap_or("Unknown"),
        it["Name"].as_str().unwrap_or("Unknown"),
        it["SeriesId"].as_str().map(norm_id),
        it["SeasonId"].as_str().map(norm_id),
        opt_str(&it["SeriesName"]),
        it["IndexNumber"].as_i64(),
        it["ParentIndexNumber"].as_i64(),
        opt_str(&it["Album"]),
        opt_str(&it["AlbumArtist"]),
        ticks_to_s(&it["RunTimeTicks"]),
        it["ProductionYear"].as_i64(),
        opt_str(&it["PremiereDate"]),
        it["DateCreated"].as_str().and_then(parse_ts),
        it["CommunityRating"].as_f64(),
        opt_str(&it["OfficialRating"]),
        genres,
        opt_str(&it["Overview"]),
        opt_str(&it["ImageTags"]["Primary"]),
        opt_str(&it["BackdropImageTags"][0]),
        opt_str(&source["Container"]).or_else(|| opt_str(&it["Container"])),
        opt_str(&it["Path"]).or_else(|| opt_str(&source["Path"])),
        int(&source["Size"]),
        int(&source["Bitrate"]),
        streams.video_codec,
        streams.width,
        streams.height,
        streams.video_range,
        streams.audio_codec,
        streams.audio_channels,
        now,
        provider_ids,
        studios,
        streams.bit_depth,
        framerate,
        crate::media::track_languages(&source["MediaStreams"], "Audio"),
        crate::media::track_languages(&source["MediaStreams"], "Subtitle"),
    ])?;
    Ok(true)
}

/// How many actors per title are kept: the billed cast, not every walk-on.
const MAX_ACTORS: usize = 12;

/// Replaces what is known about one title's actors and directors with Jellyfin's current answer.
pub fn store_people(conn: &Connection, it: &Value) -> Result<()> {
    let Some(item_id) = it["Id"].as_str().map(norm_id) else { return Ok(()) };
    conn.prepare_cached("DELETE FROM item_people WHERE item_id = ?1")?.execute([&item_id])?;
    let Some(people) = it["People"].as_array() else { return Ok(()) };
    let mut actors = 0usize;
    for (sort, p) in people.iter().enumerate() {
        let kind = match p["Type"].as_str() {
            Some("Actor") if actors < MAX_ACTORS => {
                actors += 1;
                "Actor"
            }
            Some("Director") => "Director",
            _ => continue,
        };
        let (Some(person_id), Some(name)) = (p["Id"].as_str().map(norm_id), opt_str(&p["Name"])) else { continue };
        conn.prepare_cached(
            "INSERT OR IGNORE INTO item_people(item_id, person_id, kind, name, role, sort, has_image, image_tag) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        )?
        .execute(params![item_id, person_id, kind, name, opt_str(&p["Role"]), sort as i64, opt_str(&p["PrimaryImageTag"]).is_some(), opt_str(&p["PrimaryImageTag"])])?;
    }
    Ok(())
}

/// Plays recorded before their item was known get their library (and type details) filled in.
/// Everything the library can tell a play that the play did not already know. Runs after every
/// library read as well as after an import, because history is often imported first: a tracker that
/// records no item type (neither Jellystat nor Streamystats does) leaves rows guessed or unknown,
/// and an episode has no season or episode number until something knows the library.
pub fn backfill_playbacks(conn: &Connection) -> Result<Vec<String>> {
    conn.execute_batch(
        "UPDATE playbacks SET library_id = COALESCE(
                (SELECT library_id FROM items WHERE items.id = playbacks.item_id),
                (SELECT library_id FROM items WHERE items.id = playbacks.series_id))
         WHERE library_id IS NULL;
         UPDATE playbacks SET runtime_s = (SELECT runtime_s FROM items WHERE items.id = playbacks.item_id)
         WHERE runtime_s IS NULL;

         UPDATE playbacks SET
            season_number  = (SELECT parent_index_number FROM items WHERE items.id = playbacks.item_id),
            episode_number = (SELECT index_number FROM items WHERE items.id = playbacks.item_id)
         WHERE item_type = 'Episode' AND episode_number IS NULL;

         -- Only for history that was imported: a play finstats watched itself was typed by the
         -- session as it happened, and that is the better answer if the two ever disagree.
         UPDATE playbacks SET item_type = (SELECT type FROM items WHERE items.id = playbacks.item_id)
         WHERE source <> 'live' AND item_type <> 'Episode'
           AND EXISTS (SELECT 1 FROM items WHERE items.id = playbacks.item_id AND items.type <> playbacks.item_type);

         -- Jellystat keeps the series' name where an episode's should be; the library knows the episode's.
         UPDATE playbacks SET item_name = (SELECT name FROM items WHERE items.id = playbacks.item_id)
         WHERE source <> 'live' AND item_type = 'Episode' AND item_name = series_name
           AND EXISTS (SELECT 1 FROM items WHERE items.id = playbacks.item_id AND items.name <> '' AND items.name <> playbacks.item_name);",
    )?;
    let relinked = crate::relink::relink_orphans(conn, crate::state::Settings::load(conn)?.merge_window_s)?;
    // New titles may be what Sonarr, Radarr or a request were waiting for.
    crate::pipeline::link(conn)?;
    // …and what somebody put on their watchlist before it was here.
    crate::watchlist::resolve(conn)?;
    Ok(relinked.moved_to)
}

/// The libraries Jellyfin lists; `Some(known)` when the read is refused (`whole_set_vanished`).
pub(crate) fn store_libraries(c: &mut Connection, lib_rows: &[Value], shrink_ok: bool, started: i64) -> Result<Option<i64>> {
    let tx = c.transaction()?;
    for l in lib_rows {
        tx.execute(
            "INSERT INTO libraries(id, name, collection_type, image_tag, removed, updated_at) VALUES (?1, ?2, ?3, ?4, 0, ?5)
             ON CONFLICT(id) DO UPDATE SET name = excluded.name, collection_type = excluded.collection_type,
                image_tag = excluded.image_tag, removed = 0, updated_at = excluded.updated_at",
            params![l["id"].as_str(), l["name"].as_str(), l["collection_type"].as_str(), l["image_tag"].as_str(), started],
        )?;
    }
    // The same guard as items, at the level above: a Jellyfin that lists no libraries where
    // finstats knows several is a broken read, not an emptied server.
    let current: i64 = tx.query_row("SELECT COUNT(*) FROM libraries WHERE removed = 0", [], |r| r.get(0))?;
    if !shrink_ok && whole_set_vanished(lib_rows.len(), current) {
        tx.rollback()?;
        return Ok(Some(current));
    }
    tx.execute("UPDATE libraries SET removed = 1 WHERE updated_at < ?1", [started])?;
    // A library that is gone is never read again, so nothing else would ever say its titles are gone too.
    // Reversible like any removal: a library that comes back is read, and its titles with it.
    tx.execute("UPDATE items SET removed = 1 WHERE removed = 0 AND library_id IN (SELECT id FROM libraries WHERE removed = 1)", [])?;
    tx.commit()?;
    Ok(None)
}

async fn sync_libraries(app: &App, jf: &Jellyfin) -> Result<String> {
    const ID: &str = "sync_libraries";
    app.tasks.update(ID, "Fetching libraries", None);
    let started = db::now();
    let folders = jf.virtual_folders().await?;

    let mut libraries: Vec<(String, String)> = vec![];
    let lib_rows: Vec<Value> = folders
        .iter()
        .filter(|f| !SKIPPED_COLLECTIONS.contains(&f["CollectionType"].as_str().unwrap_or("")))
        .filter_map(|f| {
            let id = norm_id(f["ItemId"].as_str()?);
            let name = f["Name"].as_str().unwrap_or("Library").to_string();
            libraries.push((id.clone(), name.clone()));
            Some(json!({ "id": id, "name": name, "collection_type": f["CollectionType"], "image_tag": f["PrimaryImageItemId"] }))
        })
        .collect();

    let seen_libs = lib_rows.len();
    let shrink_ok = allow_shrink();
    let refused = app.db.call(move |c| store_libraries(c, &lib_rows, shrink_ok, started)).await?;
    if let Some(current) = refused {
        let plural = if seen_libs == 1 { "y" } else { "ies" };
        let msg = format!(
            "Jellyfin listed {seen_libs} librar{plural} where finstats knows {current}. Refusing to mark the missing ones removed — this looks like a Jellyfin change or a bad read, not an emptied server. Nothing was changed."
        );
        app.request_halt(msg.clone());
        return Err(anyhow!(msg));
    }

    let mut total_items = 0usize;
    let lib_count = libraries.len();
    for (n, (lib_id, lib_name)) in libraries.into_iter().enumerate() {
        // How many of this library's items the read actually saw. The guard below rests on it, so it
        // is named for what it means and never reused: a second cursor called `start` for the
        // cast-and-crew pass used to shadow it, and the guard then compared a library's *shows*
        // against its *items* — on a television library, a handful against thousands, so every read
        // looked like a gutted library and halted the install.
        let mut seen = 0usize;
        // What of it could be stored: the guard's count. `seen` is the page cursor, and an answer whose
        // items finstats cannot read (no `Id`) is a full page that stored nothing.
        let mut stored = 0usize;
        let mut total = 0usize;
        loop {
            let (items, reported) = jf.items_page(&lib_id, seen, PAGE).await?;
            if seen == 0 {
                total = reported;
            }
            let got = items.len();
            if got == 0 {
                break;
            }
            let lib = lib_id.clone();
            stored += app
                .db
                .call(move |c| {
                    let now = db::now();
                    let tx = c.transaction()?;
                    let mut stored = 0usize;
                    for it in &items {
                        stored += usize::from(upsert_item(&tx, &lib, it, now)?);
                    }
                    tx.commit()?;
                    Ok(stored)
                })
                .await?;
            seen += got;
            total_items += got;
            let within = if total > 0 { seen as f64 / total as f64 } else { 1.0 };
            app.tasks.update(
                ID,
                format!("{lib_name}: {seen} of {} items", total.max(seen)),
                Some((n as f64 + within.min(1.0)) / lib_count.max(1) as f64),
            );
            if got < PAGE {
                break;
            }
        }
        // Cast and crew, for films and shows only. A failure here never fails the library read.
        let mut people_at = 0usize;
        loop {
            let items = match jf.people_page(&lib_id, people_at, PAGE).await {
                Ok(items) => items,
                Err(e) => {
                    tracing::warn!("reading cast and crew of {lib_name} failed: {e:#}");
                    break;
                }
            };
            let got = items.len();
            if got == 0 {
                break;
            }
            app.tasks.update(ID, format!("{lib_name}: cast and crew"), None);
            app.db
                .call(move |c| {
                    let tx = c.transaction()?;
                    for it in &items {
                        store_people(&tx, it)?;
                    }
                    tx.commit()?;
                    Ok(())
                })
                .await?;
            people_at += got;
            if got < PAGE {
                break;
            }
        }

        // Only after a library was read completely is "not seen" proof of removal — and only if the
        // read is trustworthy. `seen` is how many of this library's items the read above actually
        // saw; if that is a fraction of what finstats holds, the read is broken, not the library
        // empty. Refuse, keep the data, and halt so the operator can look (`trustworthy_removal`).
        let lib = lib_id.clone();
        let shrink_ok = allow_shrink();
        let refused = app
            .db
            .call(move |c| {
                let current: i64 = c.query_row("SELECT COUNT(*) FROM items WHERE library_id = ?1 AND removed = 0", [&lib], |r| r.get(0))?;
                if !shrink_ok && !trustworthy_removal(stored, current) {
                    return Ok(Some(current));
                }
                c.execute("UPDATE items SET removed = 1 WHERE library_id = ?1 AND updated_at < ?2 AND removed = 0", params![lib, started])?;
                Ok(None)
            })
            .await?;
        if let Some(current) = refused {
            let msg = format!(
                "Jellyfin returned {stored} readable item(s) for library “{lib_name}” but finstats holds {current}. Refusing to mark {} items removed — this looks like a Jellyfin change or a bad read, not a deletion. The library was left exactly as it was.",
                current - stored as i64
            );
            app.request_halt(msg.clone());
            return Err(anyhow!(msg));
        }
    }

    app.tasks.update(ID, "Linking plays to libraries", Some(1.0));
    let window = app.settings().group_window_s;
    app.db
        .call(move |c| {
            // Re-linked plays keep their times, so nothing else would look at the titles they moved onto again.
            for title in backfill_playbacks(c)? {
                crate::groups::detect(c, window, Some(&title))?;
            }
            db::set_setting(c, "library_synced_at", &started.to_string())
        })
        .await?;
    Ok(format!("{total_items} items in {lib_count} libraries"))
}

/// How far back each look reaches past the last one: the two clocks may differ a little, and an item saved while
/// the last look was being answered must not fall between the two.
const CHANGES_OVERLAP_S: i64 = 600;
/// People asked for in one page: ids and tags only, so a page this big is small.
const PEOPLE_PAGE: usize = 1000;

/// Where the next look for changes starts: from the last look, else from the last library read (which read
/// everything), with the overlap. `None` before the library was ever read — that read will bring everything.
fn changes_from(last_look: Option<i64>, library_read: Option<i64>) -> Option<i64> {
    let from = last_look.max(library_read)?;
    Some(from - CHANGES_OVERLAP_S)
}

/// One page of what Jellyfin saved in a library since the last look, stored as the library read stores an item —
/// names, overviews, genres, ratings, file details, pictures — and cast and crew for films and shows. Returns how
/// many titles were stored. Nothing is ever marked removed here: that takes a whole library read.
pub fn store_changes(c: &Connection, library_id: &str, items: &[Value], now: i64) -> Result<usize> {
    let mut stored = 0;
    for it in items {
        if !upsert_item(c, library_id, it, now)? {
            continue;
        }
        stored += 1;
        // As the library read keeps them: the cast of films and shows (`people_page`), never of every episode.
        if matches!(it["Type"].as_str(), Some("Movie" | "Series")) {
            store_people(c, it)?;
        }
    }
    Ok(stored)
}

/// Where the look at people starts: where the look at titles does, once every person has been read (`portraits_read`).
/// Until then — the first look after 2.0.4 added portrait tags — every person is read once, since a portrait replaced
/// before the last look would otherwise wait for the next library read. It is a mark of a *finished* read, not "some
/// tag is known": a look that read a few people must not stand in for one that read them all.
fn people_from(from: i64, portraits_known: bool) -> i64 {
    if portraits_known { from } else { 0 }
}

/// The portraits of people Jellyfin saved since the last look, written onto every title they are in. Returns how many
/// people's portraits changed; a person finstats does not know is passed over.
pub fn store_portraits(c: &Connection, people: &[Value]) -> Result<usize> {
    let mut stmt = c.prepare_cached(
        "UPDATE item_people SET image_tag = ?2, has_image = ?3 WHERE person_id = ?1 AND (image_tag IS NOT ?2 OR has_image != ?3)",
    )?;
    let mut changed = 0;
    for p in people {
        let Some(id) = p["Id"].as_str().map(norm_id) else { continue };
        let tag = opt_str(&p["ImageTags"]["Primary"]);
        let has = tag.is_some();
        changed += usize::from(stmt.execute(params![id, tag, has])? > 0);
    }
    Ok(changed)
}

/// Everything Jellyfin changed since the last look — an edited title, a new poster, the cast refreshed — library by
/// library, without waiting for a library scan: editing metadata in Jellyfin starts none.
async fn sync_changes(app: &App, jf: &Jellyfin) -> Result<String> {
    const ID: &str = "sync_changes";
    let started = db::now();
    let (from, libraries) = app
        .db
        .call(|c| {
            let at = |key: &str| -> Result<Option<i64>> { Ok(db::get_setting(c, key)?.and_then(|v| v.parse().ok())) };
            let from = changes_from(at("changes_checked_at")?, at("library_synced_at")?);
            let mut stmt = c.prepare("SELECT id, name FROM libraries WHERE removed = 0 ORDER BY name")?;
            let libraries = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?.collect::<Result<Vec<_>, _>>()?;
            Ok((from, libraries))
        })
        .await?;
    let Some(from) = from else { return Ok("Waiting for the first library read".into()) };
    let mut changed = 0usize;
    for (lib_id, lib_name) in libraries {
        let mut looked = 0usize;
        loop {
            let items = jf.changed_page(&lib_id, from, looked, PAGE).await?;
            let got = items.len();
            let lib = lib_id.clone();
            changed += app
                .db
                .call(move |c| {
                    let tx = c.transaction()?;
                    let n = store_changes(&tx, &lib, &items, db::now())?;
                    tx.commit()?;
                    Ok(n)
                })
                .await?;
            looked += got;
            if got < PAGE {
                break;
            }
            app.tasks.update(ID, format!("{lib_name}: {looked} changed items"), None);
        }
    }
    // People are in no library, and a replaced portrait re-saves the person and nothing else. After a scan Jellyfin
    // has re-saved thousands of them, so this pages through ids and tags only.
    let known = app.db.call(|c| Ok(db::get_setting(c, "portraits_read")?.is_some())).await?;
    let since = people_from(from, known);
    let mut portraits = 0usize;
    let mut looked = 0usize;
    loop {
        let people = jf.changed_people_page(since, looked, PEOPLE_PAGE).await?;
        let got = people.len();
        portraits += app.db.call(move |c| {
            let tx = c.transaction()?;
            let n = store_portraits(&tx, &people)?;
            tx.commit()?;
            Ok(n)
        }).await?;
        looked += got;
        if got < PEOPLE_PAGE {
            break;
        }
        app.tasks.update(ID, format!("People: {looked} looked at"), None);
    }
    let window = app.settings().group_window_s;
    app.db
        .call(move |c| {
            // A title that arrived since the library read may be one a renamed file became: the same tail as a read.
            for title in backfill_playbacks(c)? {
                crate::groups::detect(c, window, Some(&title))?;
            }
            if !known {
                db::set_setting(c, "portraits_read", &started.to_string())?;
            }
            db::set_setting(c, "changes_checked_at", &started.to_string())
        })
        .await?;
    Ok(changes_said(changed, portraits))
}

/// "3 titles and 1 portrait changed".
fn changes_said(titles: usize, portraits: usize) -> String {
    let count = |n: usize, one: &str| format!("{n} {one}{}", if n == 1 { "" } else { "s" });
    match (titles, portraits) {
        (0, 0) => "Nothing changed".into(),
        (t, 0) => format!("{} changed", count(t, "title")),
        (0, p) => format!("{} changed", count(p, "portrait")),
        (t, p) => format!("{} and {} changed", count(t, "title"), count(p, "portrait")),
    }
}

// ---------------------------------------------------------------- server activity log

async fn sync_events(app: &App, jf: &Jellyfin) -> Result<String> {
    const ID: &str = "sync_events";
    app.tasks.update(ID, "Fetching server activity", None);
    let newest: Option<i64> = app.db.call(|c| Ok(c.query_row("SELECT MAX(date) FROM server_events", [], |r| r.get(0))?)).await?;
    let min_date = newest
        .and_then(|t| chrono::DateTime::from_timestamp(t - 60, 0))
        .map(|d| d.to_rfc3339_opts(chrono::SecondsFormat::Secs, true));

    let mut start = 0usize;
    let mut added = 0usize;
    loop {
        let (entries, total) = jf.activity_log(start, PAGE, min_date.as_deref()).await?;
        let got = entries.len();
        if got == 0 {
            break;
        }
        added += app
            .db
            .call(move |c| {
                let tx = c.transaction()?;
                let mut n = 0;
                {
                    let mut stmt = tx.prepare(
                        "INSERT OR IGNORE INTO server_events(id, date, name, overview, short_overview, type, severity, user_id, item_id, remote_ip)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                    )?;
                    for e in &entries {
                        let (Some(id), Some(date)) = (e["Id"].as_i64(), e["Date"].as_str().and_then(parse_ts)) else { continue };
                        let user = e["UserId"].as_str().map(norm_id).filter(|u| u.chars().any(|ch| ch != '0'));
                        n += stmt.execute(params![
                            id,
                            date,
                            e["Name"].as_str().unwrap_or(""),
                            opt_str(&e["Overview"]),
                            opt_str(&e["ShortOverview"]),
                            opt_str(&e["Type"]),
                            opt_str(&e["Severity"]),
                            user,
                            e["ItemId"].as_str().map(norm_id),
                            e["ShortOverview"].as_str().and_then(crate::security::event_ip).unwrap_or_default(),
                        ])?;
                    }
                }
                tx.commit()?;
                Ok(n)
            })
            .await?;
        start += got;
        app.tasks.update(ID, format!("{start} of {} entries", total.max(start)), Some(start as f64 / total.max(start) as f64));
        // Cap the very first backfill; the log can be enormous on old servers.
        if got < PAGE || start >= 20_000 {
            break;
        }
    }
    Ok(format!("{added} new entries"))
}

// ---------------------------------------------------------------- server details & devices

fn folder(label: &str, kind: &str, f: &Value) -> Option<Value> {
    let (free, used) = (f["FreeSpace"].as_i64()?, f["UsedSpace"].as_i64()?);
    (free >= 0 && used >= 0 && free + used > 0).then(|| json!({ "label": label, "path": f["Path"], "free_bytes": free, "used_bytes": used, "kind": kind }))
}

async fn sync_server(app: &App, jf: &Jellyfin) -> Result<String> {
    const ID: &str = "sync_server";
    app.tasks.update(ID, "Reading server details", None);
    let info = jf.system_info().await?;
    let storage_raw = jf.storage().await;
    let plugins = jf.plugins().await.unwrap_or_default();
    let tasks = jf.scheduled_tasks().await.unwrap_or_default();
    crate::jobs::observe(app, &tasks);
    app.tasks.update(ID, "Reading devices", Some(0.6));
    let devices = jf.devices().await.unwrap_or_default();

    let mut storage: Vec<Value> = vec![];
    if let Some(st) = &storage_raw {
        for lib in st["Libraries"].as_array().map(Vec::as_slice).unwrap_or_default() {
            let name = lib["Name"].as_str().unwrap_or("Library");
            storage.extend(lib["Folders"].as_array().map(Vec::as_slice).unwrap_or_default().iter().filter_map(|f| folder(name, "library", f)));
        }
        for (key, label) in [("ProgramDataFolder", "Program data"), ("CacheFolder", "Cache"), ("TranscodingTempFolder", "Transcodes"), ("InternalMetadataFolder", "Metadata"), ("LogFolder", "Logs")] {
            storage.extend(folder(label, "system", &st[key]));
        }
    }
    let snapshot = json!({
        "fetched_at": db::now(),
        "info": {
            "server_name": info["ServerName"], "version": info["Version"],
            "operating_system": opt_str(&info["OperatingSystemDisplayName"]).or_else(|| opt_str(&info["OperatingSystem"])),
            "architecture": info["SystemArchitecture"],
            "has_update_available": info["HasUpdateAvailable"].as_bool().unwrap_or(false),
            "has_pending_restart": info["HasPendingRestart"].as_bool().unwrap_or(false),
            "transcoding_temp_path": info["TranscodingTempPath"], "cache_path": info["CachePath"],
            "program_data_path": info["ProgramDataPath"], "log_path": info["LogPath"],
            "encoder_location": info["EncoderLocation"],
        },
        "storage": storage,
        "plugins": plugins.iter().map(|p| json!({ "name": p["Name"], "version": p["Version"], "status": p["Status"], "description": p["Description"] })).collect::<Vec<_>>(),
        "scheduled_tasks": tasks.iter().map(|t| {
            let last = &t["LastExecutionResult"];
            let (start, end) = (last["StartTimeUtc"].as_str().and_then(parse_ts), last["EndTimeUtc"].as_str().and_then(parse_ts));
            json!({
                "name": t["Name"], "category": t["Category"], "state": t["State"],
                "last_result": last["Status"], "last_run_at": end.or(start),
                "last_duration_s": start.zip(end).map(|(s, e)| (e - s).max(0)),
            })
        }).collect::<Vec<_>>(),
    });

    let device_count = devices.len();
    app.db
        .call(move |c| {
            let now = db::now();
            let tx = c.transaction()?;
            db::set_setting(&tx, "server_info", &snapshot.to_string())?;
            {
                let mut stmt = tx.prepare(
                    "INSERT INTO devices(device_id, user_id, device_name, client, app_version, last_user_name, first_seen, last_seen)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)
                     ON CONFLICT(device_id, user_id) DO UPDATE SET device_name = excluded.device_name, client = excluded.client,
                        app_version = excluded.app_version, last_user_name = excluded.last_user_name,
                        last_seen = MAX(devices.last_seen, excluded.last_seen)",
                )?;
                for d in &devices {
                    let (Some(id), Some(user)) = (d["Id"].as_str(), d["LastUserId"].as_str()) else { continue };
                    stmt.execute(params![
                        id,
                        norm_id(user),
                        opt_str(&d["CustomName"]).or_else(|| opt_str(&d["Name"])),
                        opt_str(&d["AppName"]),
                        opt_str(&d["AppVersion"]),
                        opt_str(&d["LastUserName"]),
                        d["DateLastActivity"].as_str().and_then(parse_ts).unwrap_or(now),
                    ])?;
                }
            }
            tx.commit()?;
            Ok(())
        })
        .await?;
    Ok(format!("{device_count} devices, {} plugins", plugins.len()))
}

// ---------------------------------------------------------------- per-user played & favourite flags

/// Jellyfin remembers what each user has finished or favourited, including everything from
/// before finstats existed. That is what makes "never watched" trustworthy.
async fn sync_userdata(app: &App, jf: &Jellyfin) -> Result<String> {
    const ID: &str = "sync_userdata";
    let users: Vec<(String, String)> = app
        .db
        .call(|c| {
            let mut stmt = c.prepare("SELECT id, name FROM users WHERE removed = 0 ORDER BY name")?;
            let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?.collect::<Result<_, _>>()?;
            Ok(rows)
        })
        .await?;
    let mut total = 0usize;
    let count = users.len().max(1);
    for (n, (user_id, name)) in users.into_iter().enumerate() {
        app.tasks.update(ID, format!("Reading what {name} has watched"), Some(n as f64 / count as f64));
        let mut rows: Vec<Value> = vec![];
        for (filter, types) in [("IsPlayed", "Movie,Episode"), ("IsFavorite", "Movie,Series,Episode")] {
            let mut start = 0;
            loop {
                let page = jf.user_items_page(&user_id, filter, types, start, 1000).await?;
                let got = page.len();
                rows.extend(page);
                start += got;
                if got < 1000 {
                    break;
                }
            }
        }
        total += rows.len();
        let uid = user_id.clone();
        app.db
            .call(move |c| {
                let tx = c.transaction()?;
                tx.execute("DELETE FROM user_items WHERE user_id = ?1", [&uid])?;
                {
                    let mut stmt = tx.prepare(
                        "INSERT INTO user_items(user_id, item_id, played, is_favorite, play_count, last_played_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                         ON CONFLICT(user_id, item_id) DO UPDATE SET played = MAX(played, excluded.played), is_favorite = MAX(is_favorite, excluded.is_favorite)",
                    )?;
                    for it in &rows {
                        let Some(id) = it["Id"].as_str() else { continue };
                        let ud = &it["UserData"];
                        stmt.execute(params![
                            uid,
                            norm_id(id),
                            ud["Played"].as_bool().unwrap_or(false),
                            ud["IsFavorite"].as_bool().unwrap_or(false),
                            ud["PlayCount"].as_i64().unwrap_or(0),
                            ud["LastPlayedDate"].as_str().and_then(parse_ts),
                        ])?;
                    }
                }
                tx.commit()?;
                Ok(())
            })
            .await?;
    }
    Ok(format!("{total} played or favourite items"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_first_look_at_people_reads_everybody_once() {
        assert_eq!(people_from(10_000, true), 10_000);
        assert_eq!(people_from(10_000, false), 0, "no portrait's tag known yet: every person, once");
    }

    #[test]
    fn a_portrait_replaced_in_jellyfin_reaches_every_title_the_person_is_in() {
        let c = Connection::open_in_memory().unwrap();
        for m in crate::db::MIGRATIONS {
            c.execute_batch(m).unwrap();
        }
        for movie in ["m1", "m2"] {
            store_people(&c, &json!({ "Id": movie, "People": [{ "Id": "p1", "Name": "alice", "Type": "Actor", "PrimaryImageTag": "old" }] })).unwrap();
        }
        let tags = |c: &Connection| c.prepare("SELECT DISTINCT image_tag FROM item_people WHERE person_id = 'p1'").unwrap()
            .query_map([], |r| r.get::<_, Option<String>>(0)).unwrap().map(Result::unwrap).collect::<Vec<_>>();
        assert_eq!(tags(&c), [Some("old".to_string())], "the library read keeps the portrait's tag");
        let answer = [json!({ "Id": "P1", "Type": "Person", "ImageTags": { "Primary": "new" } }), json!({ "Id": "p9", "ImageTags": { "Primary": "x" } })];
        assert_eq!(store_portraits(&c, &answer).unwrap(), 1, "one person finstats knows");
        assert_eq!(tags(&c), [Some("new".to_string())], "on every title");
        assert_eq!(store_portraits(&c, &answer).unwrap(), 0, "the same answer twice changes nothing");
        store_portraits(&c, &[json!({ "Id": "p1", "ImageTags": {} })]).unwrap();
        let has: i64 = c.query_row("SELECT MAX(has_image) FROM item_people WHERE person_id = 'p1'", [], |r| r.get(0)).unwrap();
        assert_eq!((tags(&c), has), (vec![None], 0), "a portrait removed in Jellyfin");
    }

    #[test]
    fn the_first_look_for_changes_starts_where_the_last_library_read_did() {
        assert_eq!(changes_from(None, None), None, "the library read will bring everything");
        assert_eq!(changes_from(None, Some(10_000)), Some(10_000 - CHANGES_OVERLAP_S));
        assert_eq!(changes_from(Some(20_000), Some(10_000)), Some(20_000 - CHANGES_OVERLAP_S));
        // A library read after the last look read everything again.
        assert_eq!(changes_from(Some(10_000), Some(20_000)), Some(20_000 - CHANGES_OVERLAP_S));
    }

    #[test]
    fn a_title_edited_in_jellyfin_is_stored_with_its_cast_and_nothing_is_removed() {
        let c = Connection::open_in_memory().unwrap();
        for m in crate::db::MIGRATIONS {
            c.execute_batch(m).unwrap();
        }
        upsert_item(&c, "films", &json!({ "Id": "m1", "Name": "Big Buck Bunny", "Type": "Movie", "Overview": "old", "ImageTags": { "Primary": "p-old" } }), 100).unwrap();
        upsert_item(&c, "films", &json!({ "Id": "m2", "Name": "Sintel", "Type": "Movie" }), 100).unwrap();
        store_people(&c, &json!({ "Id": "m1", "People": [{ "Id": "a1", "Name": "alice", "Type": "Actor" }] })).unwrap();
        let answer = [
            json!({ "Id": "M1", "Name": "Big Buck Bunny (Director's Cut)", "Type": "Movie", "Overview": "new", "ImageTags": { "Primary": "p-new" },
                    "People": [{ "Id": "b2", "Name": "bob", "Type": "Director" }] }),
            json!({ "Id": "e1", "Name": "Pilot", "Type": "Episode", "People": [{ "Id": "c3", "Name": "carol", "Type": "Actor" }] }),
        ];
        assert_eq!(store_changes(&c, "films", &answer, 200).unwrap(), 2);
        let (name, overview, tag): (String, String, String) =
            c.query_row("SELECT name, overview, image_tag FROM items WHERE id = 'm1'", [], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))).unwrap();
        assert_eq!((name.as_str(), overview.as_str(), tag.as_str()), ("Big Buck Bunny (Director's Cut)", "new", "p-new"));
        let people: Vec<String> = c.prepare("SELECT name FROM item_people WHERE item_id = 'm1'").unwrap().query_map([], |r| r.get(0)).unwrap().map(Result::unwrap).collect();
        assert_eq!(people, ["bob"], "the cast is Jellyfin's current one");
        let episode_people: i64 = c.query_row("SELECT COUNT(*) FROM item_people WHERE item_id = 'e1'", [], |r| r.get(0)).unwrap();
        assert_eq!(episode_people, 0, "cast and crew are kept for films and shows only, as the library read keeps them");
        let library: String = c.query_row("SELECT library_id FROM items WHERE id = 'e1'", [], |r| r.get(0)).unwrap();
        assert_eq!(library, "films", "a title that arrived since the read is stored under its library");
        let removed: i64 = c.query_row("SELECT removed FROM items WHERE id = 'm2'", [], |r| r.get(0)).unwrap();
        assert_eq!(removed, 0, "a title the look did not return is not gone: only a whole library read can say that");
    }

    #[test]
    fn a_user_missing_from_the_read_is_removed_even_by_a_read_in_the_same_second() {
        let mut c = Connection::open_in_memory().unwrap();
        for m in crate::db::MIGRATIONS {
            c.execute_batch(m).unwrap();
        }
        let user = |id: &str, name: &str| json!({ "Id": id, "Name": name, "Policy": { "IsAdministrator": false, "IsDisabled": false } });
        let removed = |c: &Connection, id: &str| c.query_row("SELECT removed FROM users WHERE id = ?1", [id], |r| r.get::<_, i64>(0)).unwrap();
        assert_eq!(store_users(&mut c, &[user("a1", "alice"), user("b2", "bob")], false, 100).unwrap(), None);
        // bob is deleted in Jellyfin and the next read lands in the same second as the last: comparing
        // whole-second timestamps, bob's row looked as fresh as alice's and was never marked removed.
        assert_eq!(store_users(&mut c, &[user("a1", "alice")], false, 100).unwrap(), None);
        assert_eq!((removed(&c, "a1"), removed(&c, "b2")), (0, 1), "the read did not say bob is gone");
        store_users(&mut c, &[user("a1", "alice"), user("b2", "bob")], false, 101).unwrap();
        assert_eq!(removed(&c, "b2"), 0, "back in Jellyfin, back in finstats");
        assert_eq!(store_users(&mut c, &[], false, 102).unwrap(), Some(2), "an empty read is refused, not everybody removed");
        assert_eq!(removed(&c, "a1"), 0);
        // A Jellyfin that changed shape under an upgrade answers everybody, in keys finstats cannot read:
        // that is an empty read too, not everybody deleted.
        let reshaped = [json!({ "id": "a1", "name": "alice" }), json!({ "id": "b2", "name": "bob" })];
        assert_eq!(store_users(&mut c, &reshaped, false, 103).unwrap(), Some(2), "nobody readable is refused like nobody at all");
        assert_eq!((removed(&c, "a1"), removed(&c, "b2")), (0, 0));
    }

    /// The library guard counts what a read stored, and an item without an id stores nothing: a page of
    /// items in keys finstats cannot read must count as an empty read, not a full one.
    #[test]
    fn an_item_finstats_cannot_read_is_not_counted_as_read() {
        let c = Connection::open_in_memory().unwrap();
        for m in crate::db::MIGRATIONS {
            c.execute_batch(m).unwrap();
        }
        assert!(upsert_item(&c, "lib1", &json!({ "Id": "m1", "Name": "Big Buck Bunny", "Type": "Movie" }), 100).unwrap());
        assert!(!upsert_item(&c, "lib1", &json!({ "id": "m2", "name": "Sintel", "type": "Movie" }), 100).unwrap());
    }

    /// Items are marked removed library by library, as each is read; a library Jellyfin no longer lists is
    /// never read again, so its titles have to go with it, or they stay in search, on the shelf and in
    /// the totals for ever, and their plays are never re-linked to where the files went.
    #[test]
    fn the_titles_of_a_library_deleted_in_jellyfin_go_with_it() {
        let mut c = Connection::open_in_memory().unwrap();
        for m in crate::db::MIGRATIONS {
            c.execute_batch(m).unwrap();
        }
        let lib = |id: &str, name: &str| json!({ "id": id, "name": name, "collection_type": "movies" });
        assert_eq!(store_libraries(&mut c, &[lib("films", "Films"), lib("old", "Old films")], false, 100).unwrap(), None);
        for (item, library) in [("m1", "films"), ("m2", "old")] {
            upsert_item(&c, library, &json!({ "Id": item, "Name": item, "Type": "Movie" }), 100).unwrap();
        }
        assert_eq!(store_libraries(&mut c, &[lib("films", "Films")], false, 200).unwrap(), None);
        let removed = |id: &str| c.query_row("SELECT removed FROM items WHERE id = ?1", [id], |r| r.get::<_, i64>(0)).unwrap();
        assert_eq!((removed("m1"), removed("m2")), (0, 1));
    }

    /// Jellystat keeps the series' name where an episode's should be (`NowPlayingItemName`); the library knows
    /// the episode's. A play finstats recorded itself was named by the session and is left alone.
    #[test]
    fn an_imported_episode_play_is_called_what_the_library_calls_the_episode() {
        let c = Connection::open_in_memory().unwrap();
        for m in crate::db::MIGRATIONS {
            c.execute_batch(m).unwrap();
        }
        c.execute_batch(
            "INSERT INTO items(id, type, name, series_id, updated_at) VALUES ('s1', 'Series', 'Big Buck Bunny', NULL, 1), ('e1', 'Episode', 'The Big Meadow', 's1', 1);
             INSERT INTO playbacks(source, user_id, user_name, item_id, item_name, item_type, series_id, series_name, started_at, ended_at, duration_s) VALUES
               ('jellystat', 'u1', 'alice', 'e1', 'Big Buck Bunny', 'Episode', 's1', 'Big Buck Bunny', 100, 700, 600),
               ('live',      'u1', 'alice', 'e1', 'Big Buck Bunny', 'Episode', 's1', 'Big Buck Bunny', 900, 1500, 600);",
        )
        .unwrap();
        backfill_playbacks(&c).unwrap();
        let names: Vec<(String, String)> = c.prepare("SELECT source, item_name FROM playbacks ORDER BY id").unwrap()
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?))).unwrap().map(Result::unwrap).collect();
        assert_eq!(names, [("jellystat".to_string(), "The Big Meadow".to_string()), ("live".to_string(), "Big Buck Bunny".to_string())]);
    }

    #[test]
    fn re_linking_a_renamed_item_does_not_leave_the_same_evening_twice() {
        let c = Connection::open_in_memory().unwrap();
        for m in crate::db::MIGRATIONS {
            c.execute_batch(m).unwrap();
        }
        // The track was re-added in Jellyfin, so it has a new id: `old-id` is gone and `new-id` is
        // what is there now. A Jellystat import was linked to it long ago; a Streamystats import of
        // the same evening still carries the old id, so nothing matched it when it was written.
        c.execute_batch(
            "INSERT INTO items(id, type, name, runtime_s, removed, updated_at) VALUES
                ('new-id', 'Audio', 'A track', 210, 0, 0),
                ('old-id', 'Audio', 'A track', 210, 1, 0);
             INSERT INTO playbacks(id, source, source_id, user_id, user_name, item_id, item_name, item_type, started_at, ended_at, duration_s) VALUES
                (1, 'jellystat',    'js:1', 'u1', 'alice', 'new-id', 'A track', 'Audio', 100, 310, 210),
                (2, 'streamystats', 'ss:1', 'u1', 'alice', 'old-id', 'A track', 'Audio', 100, 310, 210);",
        )
        .unwrap();
        // Re-linked onto the item that is really there, which the library read then regroups: moved plays keep their
        // times, so nothing else would look at that title again.
        assert_eq!(backfill_playbacks(&c).unwrap(), ["new-id"]);
        // …and not counted twice.
        let rows: Vec<(i64, String, String)> = c
            .prepare("SELECT id, source, item_id FROM playbacks ORDER BY id")
            .unwrap()
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .unwrap()
            .map(Result::unwrap)
            .collect();
        assert_eq!(rows, [(1, "jellystat".to_string(), "new-id".to_string())]);
    }

    #[test]
    fn a_library_read_tells_imported_plays_what_they_were_watching() {
        let c = Connection::open_in_memory().unwrap();
        for m in crate::db::MIGRATIONS {
            c.execute_batch(m).unwrap();
        }
        // History imported before finstats had ever read the library: one tracker guessed the type
        // wrong, another could not tell at all, and neither knew which episode this was.
        c.execute_batch(
            "INSERT INTO items(id, type, name, runtime_s, parent_index_number, index_number, updated_at) VALUES
                ('i1', 'Audio', 'Big Buck Bunny', 210, NULL, NULL, 0),
                ('e1', 'Episode', 'The Big Meadow', 1500, 2, 5, 0);
             -- Days apart on purpose: these are four separate viewings, not one evening seen by
             -- several trackers, which `drop_relinked_duplicates` would rightly take apart.
             INSERT INTO playbacks(source, user_id, user_name, item_id, item_name, item_type, started_at, ended_at, duration_s) VALUES
                ('streamystats', 'u1', 'alice', 'i1', 'Big Buck Bunny', 'Unknown', 100, 400, 300),
                ('jellystat', 'u1', 'alice', 'i1', 'Big Buck Bunny', 'Movie', 100000, 100300, 300),
                ('streamystats', 'u1', 'alice', 'e1', 'The Big Meadow', 'Episode', 200000, 201500, 1500),
                ('live', 'u1', 'alice', 'i1', 'Big Buck Bunny', 'Movie', 300000, 300300, 300);",
        )
        .unwrap();
        backfill_playbacks(&c).unwrap();
        let ty = |start: i64| -> String { c.query_row("SELECT item_type FROM playbacks WHERE started_at = ?1", [start], |r| r.get(0)).unwrap() };
        assert_eq!(ty(100), "Audio");
        assert_eq!(ty(100000), "Audio");
        // A play finstats watched itself was typed by the session at the time; that stands.
        assert_eq!(ty(300000), "Movie");
        // And the episode now knows which one it is.
        let se: (i64, i64) = c.query_row("SELECT season_number, episode_number FROM playbacks WHERE item_id = 'e1'", [], |r| Ok((r.get(0)?, r.get(1)?))).unwrap();
        assert_eq!(se, (2, 5));
    }

    #[test]
    fn keeps_the_billed_cast_and_directors_and_replaces_them_on_the_next_read() {
        let conn = Connection::open_in_memory().unwrap();
        for m in crate::db::MIGRATIONS {
            conn.execute_batch(m).unwrap();
        }
        let mut people: Vec<Value> = (0..20).map(|n| json!({ "Id": format!("a{n}"), "Name": format!("Actor {n}"), "Type": "Actor", "Role": "Extra" })).collect();
        people[0]["PrimaryImageTag"] = json!("tag");
        people.push(json!({ "Id": "d1", "Name": "Jane Doe", "Type": "Director" }));
        people.push(json!({ "Id": "w1", "Name": "A Writer", "Type": "Writer" }));
        // The same person acting in and directing a title is two credits.
        people.push(json!({ "Id": "a0", "Name": "Actor 0", "Type": "Director" }));
        store_people(&conn, &json!({ "Id": "AB-CD", "People": people })).unwrap();

        let count = |sql: &str| -> i64 { conn.query_row(sql, [], |r| r.get(0)).unwrap() };
        assert_eq!(count("SELECT COUNT(*) FROM item_people WHERE kind = 'Actor'"), MAX_ACTORS as i64);
        assert_eq!(count("SELECT COUNT(*) FROM item_people WHERE kind = 'Director'"), 2);
        assert_eq!(count("SELECT COUNT(*) FROM item_people WHERE item_id = 'abcd'"), MAX_ACTORS as i64 + 2);
        assert_eq!(count("SELECT has_image FROM item_people WHERE person_id = 'a0' AND kind = 'Actor'"), 1);

        store_people(&conn, &json!({ "Id": "AB-CD", "People": [{ "Id": "d2", "Name": "John Roe", "Type": "Director" }] })).unwrap();
        assert_eq!(count("SELECT COUNT(*) FROM item_people"), 1);
    }

    #[test]
    fn losing_every_library_or_user_at_once_is_always_a_broken_read() {
        assert!(whole_set_vanished(0, 5), "five libraries down to none is a broken read");
        assert!(whole_set_vanished(0, 1), "even one set going to zero is caught");
        assert!(!whole_set_vanished(4, 5), "losing one of five is normal");
        assert!(!whole_set_vanished(0, 0), "nothing held, nothing lost");
        assert!(!whole_set_vanished(3, 3), "unchanged");
    }

    #[test]
    fn only_a_clearly_broken_read_is_refused_ordinary_shrink_is_normal() {
        // The guard is for the "Jellyfin broke my program" case — a stocked library that reads back
        // empty, or a big one gutted to almost nothing — not for ordinary churn. A real install losing
        // most of a small library (37 clips down to 3 as their files go) must NOT be mistaken for a
        // fault, or the guard turns a normal day into an outage.
        assert!(trustworthy_removal(3, 37), "a 37-item library down to 3 is ordinary churn, not a fault");
        assert!(trustworthy_removal(0, 37), "a small library reading empty is allowed, not a crash");
        assert!(trustworthy_removal(100, 1244), "8% of a mid library surviving is allowed");
        // First sync (nothing stored) and genuinely-new sets are always fine.
        assert!(trustworthy_removal(0, 0));
        assert!(trustworthy_removal(4000, 0));
        // Ordinary churn on a big library is fine.
        assert!(trustworthy_removal(4800, 5000), "4% churn is normal");
        assert!(trustworthy_removal(600, 5000), "12% surviving is allowed");
        // Clearly broken: a sizeable library read empty, or gutted to under 5% with a big loss.
        assert!(!trustworthy_removal(0, 16743), "a stocked library read empty is a broken read");
        assert!(!trustworthy_removal(0, 200), "at the empty-floor, an empty read is refused");
        assert!(!trustworthy_removal(3, 16743), "thousands down to three is a broken read");
        assert!(!trustworthy_removal(200, 4773), "under 5% of a big library surviving is refused");
        // But an empty read of a set below the floor, and a shrink that removes fewer than the wipe
        // floor however small the survivor, are treated as ordinary.
        assert!(trustworthy_removal(0, 199), "just below the empty-floor is allowed");
        assert!(trustworthy_removal(1, 900), "removing under the wipe-floor is ordinary, even to one");
    }
}
