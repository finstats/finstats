# Implementation decisions

The rules the code must honour, each with the reason it exists: what broke without it, what was measured, or what the
owner decided. [`ARCHITECTURE.md`](ARCHITECTURE.md) explains how the pieces fit; this file is the contract behind each
piece. Newest at the bottom of each section.

Items are cited by section name and number ("Collector" 4, "Backups" 7), so a number never changes: an item the code has
outgrown is corrected in place, or marked **Superseded** with a pointer to what replaced it. A bare number in parentheses,
"(3.)", is an item of the same section. A change that adds or alters a rule adds or corrects its item in the same commit.

Contents:

- Foundation: Settings and state · Database · HTTP server · Logging
- Jellyfin: Jellyfin client · Library reads · Scheduling · Jellyfin's jobs · Pictures · Track languages
- Plays: Collector · Live channel · Play rows · Imports · Jellystat import · Streamystats import · Tautulli import ·
  Duplicate rule · Relink · Locate
- Answers: Stats layer · Library access · Playback insights · Groups · Search · Profiles · Watchlist · Timeline · Browse ·
  Recently added · Recap · Library health · Capacity
- People: Auth · Permissions · Audit · Public profiles
- Network: Local and remote · Security
- Around Jellyfin: Pipeline · Outbound connections · Notifications
- Data: Backups · Trash
- Languages · Changelog
- Web UI: Front-end · Tables · Data loading and prefetch · Settings and Server pages · Navigation · Theming · FinUI ·
  FinMotion · Simple mode · Formats and the welcome · Now playing
- Shipping: Image and data folder · Licence notice

## Settings and state (`state.rs`)

1. `Settings` is one JSON blob in the `settings` table, `#[serde(default)]` so old installs load. A blob that still cannot
   be read is an error naming the key at fault, never "all defaults": that reset every setting silently, and the next save
   wrote the defaults back.
2. **Every change goes through `AppState::update_settings`**, which reads, changes, validates, stores and publishes under
   one lock: each write is the whole blob, and two at once lost one.
3. Background loops select on `AppState::wake` (a `Notify`).

## Database (`db.rs`, `upgrade.rs`)

1. All DB work goes through `db.call(|conn| …)` (r2d2 pool + `spawn_blocking`). rusqlite is re-exported as `db::rusqlite`:
   import it from there, not from the crate, to stay on the version `r2d2_sqlite` uses.
2. A page whose queries over the history need nothing from each other runs them side by side: `stats::resolved` once,
   then `apart` per part under `tokio::try_join!`. Each is one core's work in SQLite, and in sequence the page took their
   sum. At most five at once, so the collector keeps a connection of the six.
3. **Every `transaction()` on a pooled connection is IMMEDIATE** (set in the pool's init): a deferred one that reads
   before it writes is refused its write outright once anybody else commits in between (`SQLITE_BUSY_SNAPSHOT`, which
   `busy_timeout` does not wait out), and the collector commits all the time. A transaction that only reads (the backup's
   snapshot) asks for `Deferred` by name, or it would hold the write lock for the whole export.
4. `journal_size_limit` (64 MB) is set in the pool's init: SQLite never shrinks its WAL on its own.
5. **`preflight` looks at the file once before the pool touches it**: the pool retries a bad file for its 30-second
   timeout, and its first connection switches the file to WAL (a write). A damaged file, another program's database
   (tables, `user_version` 0), a folder or a file FinStats may not write is refused there, in milliseconds, naming it and
   saying nothing was changed.
6. **Migrations** are the `MIGRATIONS` array, applied by index against `PRAGMA user_version`. Released migrations are
   immutable (deployed databases ran them): add an entry, never edit or reorder one. A migration in the working tree may
   already have run on a live database (a deploy builds the working tree, not HEAD): once it could have run, add a new one
   instead of changing it.
7. **`refuse_downgrade`**: the settings key `app_version` holds the newest version that has opened the database; a binary
   older than that, or one with fewer migrations than `user_version`, bails before writing anything. Releases up to 1.0.4
   predate the check and cannot be stopped. The key is not part of backups on purpose.
8. **A newer binary snapshots the database before it upgrades it (`back_up_before_update`).** Between `refuse_downgrade`
   and the migrations, `is_update` asks whether a populated database is being opened by a *different* version, or still
   has migrations to run; if so a full `VACUUM INTO` copy (the library and the secrets included, unlike the exportable
   JSON backups) is written to `<data>/pre-update-backups/` **before** anything is changed, so a migration or binary that
   corrupts data can be rolled back (stop FinStats, put the copy in place of `finstats.db`, run the old version). A
   brand-new database and a same-version restart snapshot nothing.
9. The newest `PRE_UPDATE_KEEP` (3) snapshots are kept; they are never served over the API. Pruning counts and removes
   only FinStats' own copies (a `.db` the owner put in that folder is left alone); a copy's name takes only the digits and
   dots of the version it records.
10. A failed snapshot is fatal only when migrations are pending (the risky case); a plain version bump warns and
    continues. `FINSTATS_SKIP_PREUPDATE_BACKUP=1` turns it off.
11. A snapshot is written to `<name>.db.part` and renamed when complete, and what a killed start left is swept first:
    `VACUUM INTO` writes straight into its target, and an empty file under a snapshot's name is a valid empty database to
    SQLite that counted toward the three kept.
12. **Upgrade steps (2.3.0)** are what a version needs done once to a database an older FinStats wrote, beside its
    migrations: a migration changes tables, a step does the rest (a job made due so something new is worked out at the
    first start, a value filled in). `Db::open` runs them after `migrate`, oldest version first, and records each by name
    (`upgrade_steps`) once run, like a migration, never judged by the version that last opened the database: a
    development build of that version had already opened it. A brand-new database is recorded as having had every step
    and runs none; a step that fails is not recorded and runs again at the next start. A step's name is never reused.
13. **Every reading of history reads the view `visible_playbacks`**, never the table, except for identity (see "Trash" 3).

## HTTP server (`main.rs`, `api.rs`)

1. **The server is hyper, driven directly (`main.rs::serve_http`), not `axum::serve`**, which sets no timer: a connection
   that sent half a request head, or nothing, or sat idle after one, was kept for good, and a few hundred of them used up
   the open files, so FinStats stopped answering, reading Jellyfin and writing its database. `Limits`: the head must arrive
   within 15 s (which also bounds an idle kept-alive connection) and at most 512 connections are served at once, the next
   waiting in the listen queue; a stop lets requests under way finish for at most `DRAIN_WITHIN` (20 s). It brings
   `hyper`, `hyper-util` and `tower-service` as direct dependencies, all already in `Cargo.lock`.
2. The response compression layer skips `application/gzip`: re-compressing a backup broke the download in browsers.
   Anything served pre-compressed needs the same exemption (`compress_when`); fonts are skipped too.
3. The web UI's own files are gzipped once and kept until the next release (`gzipped`, `Vary: Accept-Encoding`, sent as
   they are to a browser that does not ask for gzip): compressing every module and the 450 KB world map again at each
   request ran on the async threads.
4. `api.rs` adds an Origin check on writes. It compares the scheme too where FinStats knows how it is reached (a believed
   proxy's `X-Forwarded-Proto`, or a page on the host of `public_url`); `X-Forwarded-Host` counts only from a believed peer.
5. `harden`'s headers go on every answer: a strict CSP (`img-src 'self' data:`, no `blob:`; `style-src 'self'`: the UI
   must not use inline `<style>` or `style=""`; `el.style.x` from JS is fine), `Cross-Origin-Opener-Policy: same-origin` (a
   page that opened FinStats could later turn its tab into a sign-in page of its own) and a `Permissions-Policy` denying
   the camera, microphone, location and the like. `/api/*` answers add `Cross-Origin-Resource-Policy: same-origin`, and
   `no-store` when they set no `Cache-Control` themselves.
6. **Release builds check for overflow** (`[profile.release] overflow-checks = true`): a number from Jellyfin or an import
   that overflows panics the one task reading it rather than wrapping into a loop that never ends (one episode number far
   below zero once hung every library read). Sums over the server's numbers are checked or saturating all the same.
7. **A request's body and a response's writes have timers too, and one address holds at most its share of the
   connections** (`Limits`): once its first 15 s are over a body must have averaged 1 KB a second (`MinRate`), or it ends
   in an error and closes its connection; a write that waits 30 s for the client to read gives the connection up
   (`WriteWithin`); and one address (an IPv6 /64) holds at most 64 connections, a believed proxy's apart, since it carries
   everybody's. The head's timer alone let a body promised and sent a byte at a time hold its connection for good, and
   one host could hold all 512. A rate rather than a deadline, so a 4 GB upload over a slow line still arrives.

## Logging

1. **Names a stranger chose never write a log line of their own**: a device's, a person's or a title's name is logged
   with its control characters escaped; what a service says it is goes into its last error quoted, short and on one line.
2. The Jellyfin connection and a sign-in's answer have no `Debug` (they hold the API key and an access token).

## Jellyfin client (`jellyfin.rs`, `socket.rs`)

1. Every request asks for `Accept: application/json; profile="PascalCase"`: 10.x servers answer PascalCase and newer ones
   camelCase by default. All JSON access in the codebase assumes PascalCase keys.
2. Jellyfin ids are normalised with `db::norm_id` (no dashes, lowercase) everywhere.
3. **Compression is reqwest's `gzip` + `brotli` features**: every outbound read asks for `Accept-Encoding: gzip, br` and
   is decoded transparently (Jellyfin, Sonarr, Radarr and Seerr all compress JSON when asked, and JSON is nearly everything
   FinStats reads). The features are the whole mechanism: `default-features = false` without them means no header goes
   out and nothing is decoded, so the QA mocks answer compressed and a check holds the header.
4. Two exceptions to (3.): `geo.rs` pins `Accept-Encoding: identity` on the `.mmdb.gz` download (a gzip *body* it unpacks
   itself; the header also keeps `content_length` honest, and setting it at all turns reqwest's own decoding off), and the
   WebSocket carries none (frame compression is `permessage-deflate`, which tungstenite does not implement).
5. The socket's handshake cannot ask for `profile="PascalCase"`, so `socket.rs` re-cases keys and holds the first pushed
   snapshot against one real `/Sessions` read before a row is written from it.
6. `jellyfin::APP_NAME` is how FinStats finds its own API key in Jellyfin; it stays `finstats`, or every install would
   create a second key.

## Library reads (`sync.rs`)

1. **Library reads must ask for real items.** `items_page` passes `CollapseBoxSetItems=false` (otherwise servers with
   "group movies into collections" return the BoxSet *instead of* its films, which then get flagged removed) and
   `ExcludeLocationTypes=Virtual` (missing and unaired placeholders). Anything a read does not return is marked
   `removed`, so a query that silently hides items is a data-loss bug, not a cosmetic one.
2. **A read never wipes what it cannot see.** Marking rows `removed` is destructive (they vanish from every page and
   stat), and `items_page` turns anything it cannot parse into an empty list, so a Jellyfin that changes shape under an
   upgrade, or answers `200 {"Items":[]}`, must not read as "the library was emptied". `sync::trustworthy_removal(seen, current)` gates every destructive removal (items, libraries, users): a read that comes back empty, or a catastrophic
   shrink of a sizeable set, is refused. The data is kept, the sync fails (task + notification), and
   `AppState::request_halt` asks the process to stop cleanly (exit 70, a reason on stderr; `notify_one`, because the
   scheduler can ask before the server is waiting), so the operator pins a version or pushes a fix rather than finding a
   wiped install. Ordinary churn still applies; `FINSTATS_ALLOW_LIBRARY_SHRINK=1` waves a genuine emptying through (and
   clears a halt loop).
3. The count shown to the guard is what the read could *store* (entries with an `Id`), never the raw length of the
   answer: an answer in unknown keys is an empty read.
4. A library that leaves Jellyfin's list takes its titles with it (`store_libraries`), since it is never read again.
5. **`sync_libraries` reads a library twice (every item, then the cast and crew of films and shows only), and only the
   first count may reach `trustworthy_removal`.** The two cursors were both called `start`, the second shadowing the
   first, so the guard compared a library's *shows* with its *items*: equal on a film library, 223 against 16,744 on a
   television one, 0 against 4,773 on music. Every read of such a library looked gutted, so the guard refused it and
   halted (correctly, on a reading that was never true), and the install restarted and did it again every two and a half
   minutes, with no library read ever completing. Nothing was wrongly removed (`updated_at` decides that, not this count,
   and too small a count only ever refuses), but nothing was read either. The cursors are now `seen` and `people_at`.
6. **A QA mock must answer the query it stands in for**: `mock-jellyfin-breakable.mjs` ignored `IncludeItemTypes`, so
   both passes of (5.) looked identical to it and a stage written for exactly that class of fault passed throughout.
7. The cast-and-crew pass (`Fields=People`, a second, small `/Items` read per library) fills `item_people`: actors (first
   12 billed) and directors of films and shows only. Never add `People` to the main item read.
8. **What the library can tell a play lives in `sync::backfill_playbacks`**, not in an importer, because history is
   usually imported before FinStats has ever read the library. It runs after every library read and after an import, and
   fills `library_id`, `runtime_s`, season/episode numbers and, for imported rows only, the item type neither tracker
   records. A live row keeps the type the session gave it.
9. After a library read and a look for metadata changes, derived findings (Library health, Capacity) are worked out
   **beside `backfill_playbacks`, never inside it**: imports and restores run that too and change no item.
10. **Metadata changes (`sync_changes`)** reads, per library, what Jellyfin saved since the last look
    (`jellyfin::changed_query`: the library read's own question plus `People` and `MinDateLastSaved`, from `changes_from`
    with a ten-minute overlap) and stores it as the library read does (`store_changes`), cast included (editing metadata in
    Jellyfin starts no scan), but never marks anything removed.
11. `sync_changes` then reads the **people** saved since (`people_changed_query`, 1,000 a page) and writes their portrait
    tags onto every title they are in (`store_portraits`, `item_people.image_tag`, migration 29): replacing a portrait
    re-saves the person and none of their titles, and `artwork::tag_of` falls back to that tag for a person's id. Before
    that, a replaced portrait stayed a week on disk and a week in the browser.
12. `sync::store_users` keeps out a user whose read does not say they are enabled (a missing `IsDisabled` fails closed, as
    a missing `IsAdministrator` does), and marks removed whoever the read did not return, by id: a whole-second
    `updated_at` comparison missed a deletion read in the same second.
13. **The guard counts only titles a library read stored** (`sync::held_by_reads`, `updated_at > 0`; a title only an import
    brought has 0). Counted with imported titles, a backup of invented ones twenty times a library's size made every read
    look broken and halted FinStats at every start, with the blame on Jellyfin.

## Scheduling (`schedule.rs`, `sync::scheduler`, Settings → Tasks, 2.0.4)

1. Every job that runs by itself carries triggers the way Jellyfin's scheduled tasks do (daily, weekly, interval,
   start-up), plus one Jellyfin has no need for, **after Jellyfin's library scan** (`AFTER_SCAN` jobs only), each with an
   optional time limit (`within`; not for `backup`/`geoip`, which cannot be stopped half way).
2. The owner's lists live in `Settings::schedules`; a job without one runs on `defaults`, which is exactly what the
   settings before 2.0.4 said (`follow_jellyfin_scan` + `sync_interval_h`, `backup_every_d`, `geoip_download`). Those are
   no longer shown and decide nothing else: **one place schedules a job** (owner's decision).
3. The scheduler looks once a minute (or on `wake`); `schedule::due` is the whole decision and is pure. A time of day fires
   when it falls in the stretch since the last look (so a time missed while stopped is not caught up); an interval counts
   from the end of the last run.
4. **When each job last finished is kept** (`task_runs`, migration 28, loaded into `Tasks::remember` and `seed`ed from
   `library_synced_at` and the newest backup), or every start would run everything again.
5. A library read due mid-scan waits for the scan to end (`deferred_library`).
6. Housekeeping that is not a job (server name, a dropped-in GeoIP file, `services::check_all`, the audit log's age) stays
   on a 15-minute beat.
7. The public-IP lookup is **not** a job: it runs once at start-up on an install that has never learned an address, and
   otherwise only when somebody asks (`POST /api/settings/public-ip`, or switching the setting on).
8. Every scheduled job can also be run by hand (`api::RUNNABLE`; a test holds it against `TASK_IDS`).
9. **Making a library read due**: since 2.0.4 that means deleting the `task_runs` row of `sync_libraries` as well as
   `library_synced_at`; the setting alone no longer makes a read due.
10. The `/ScheduledTasks` read (every 5 min) and `sync_server`'s both pass their task list to `jobs::observe` (see
    "Jellyfin's jobs" 5).

## Jellyfin's jobs (`jobs.rs`, `GET /api/jellyfin/jobs`, the Server page)

1. Jellyfin's scheduled tasks say what the code is called ("Detect and Analyze Media Segments"); `explain` says what it
   does to the server, matching Jellyfin's **key first** (a name is in the server's language) and a keyword in the name
   second, falling back to Jellyfin's own `Description`. A sentence FinStats does not have is never invented.
2. **A run is timed by watching it**: the API carries a percentage and never a start time for the run in progress, so
   `Run` keeps the recent readings and `eta_s` works out the rest from the rate they moved at, measured against the *most
   recent* reading far enough back to say anything (0.5% and 5 s, within `WINDOW_S`), never the average of the whole run.
3. **Nothing that was not measured is offered**: with no rate, `eta_s` is `None` and the page shows a cycling ellipsis
   rather than a number. The tempting fallback (the last run's duration for the fraction left) put "about 2 minutes left"
   on screen for a quarter of an hour (1.6.1 made it age, 1.6.2 removed it): it knows nothing about how much of *this* run
   has happened and reads exactly like an earned estimate. A job Jellyfin reports at 0% throughout has no rate, and the
   page keeps its cycling "ETA…" until the percentage moves.
4. `unchanged_for_s` is published so a slow job reads as slow rather than as a stuck page; a percentage that goes
   backwards is the next run, not this one going back.
5. The watch `eta_s` needs is fed by lists FinStats already has (`jobs::observe`), never by a request made for it, which
   is the only reason the jobs card can open on an ETA.
6. `schedule` prints a time of day as a time, never a countdown (a daily trigger is in the *server's* local zone, which
   FinStats cannot know); only an interval trigger, measured from the last run, produces `next_at`.
7. The read is live but never more often than `MIN_GAP_S` (3 s) however many people watch, hidden tasks included, and
   read-only: nothing in the code can start or stop a task on Jellyfin.
8. **The estimate is the backend's (2.0.4)**: `jobs::run` reads the list itself every `BUSY_EVERY_S` (10 s) while
   `Watch::next_look` knows of a run, so a page opened on a running job is served an `eta_s` it did not have to watch into
   being (before, only a page's own 3-s reads and the five-minute scan check timed a run). Idle it asks nothing; the
   scheduler's read notices the next run.

## Pictures (`artwork.rs`)

1. **Pictures are cached under their image tag.** `/api/img/item|user` looks up the tag FinStats holds (`items.image_tag`
   / `backdrop_tag`, `users.image_tag`), names the disk-cache file after it (`cache_name`; an untagged picture keeps the
   old name) and sends it as the ETag with `private, no-cache`, so a 304 answers before anything is read. Keyed on
   anything but the tag, a replaced poster was served from the disk for a week and from the browser for another.
2. Replacing a poster in Jellyfin saves the item but starts no scan; the tags come from the library read and from
   `sync_changes` ("Library reads" 10, 11).
3. Which ids may be fetched at all is `known`: what FinStats has, a play's item counting only for a play it recorded
   itself (an imported id is the file's to say, and counts once a library read has it; a restored one is imported too).
4. Which of those a person may be shown is `may_show` (`access.rs`: a library they may open, a title they may see, a
   person in one of those); anything else is the same 404 as an unknown id.
5. A picture missing from the cache is fetched by one request while the others wait for the file it writes (`Fetches`);
   one the source failed to give is not asked for again for `FAILED_TTL` (30 s), a stale copy served meanwhile.

## Track languages (`media.rs`)

1. `items.audio_languages` / `subtitle_languages` are JSON arrays from `media::track_languages` (every track, each code
   once, `und` for an untagged one), written by both item producers (`sync::upsert_item`, the import's `jf_item_info`).
2. A series or season has none of its own: `item_detail` answers `language_coverage` (episodes per language, files only;
   the count is `media::language_counts`, shared with Library health), which is what shows a dub that stops half way.
3. FinStats never claims "dubbed": it does not know a title's original language, so it lists the languages and lets the
   reader decide.
4. Names come from the browser (`languageName` in `dom.js`: `Intl.DisplayNames` plus the bibliographic codes it lacks),
   so no language table is shipped.
5. On the title page the hero names at most four languages per kind, one line each that never wraps, and the Languages
   card (`#languages`, only when the hero could not say it all) lists every one with its episodes: a long show carries two
   dozen subtitle languages, and listing them in the hero pushed the artwork down a screen.

## Collector (`collector.rs`, `socket.rs`)

1. **Two transports, one collector.** The session list arrives either by asking (`GET /Sessions` on the
   `active_interval_s` / `idle_interval_s` timers) or by being told (Jellyfin's `/socket` with `SessionsStart`); `tick()`
   cannot tell which and must not learn.
2. **There is no setting**: the socket is spawned for whatever Jellyfin is configured, always, and being told is how
   FinStats collects. 1.4.0's `live_socket` switch was removed in 1.5.0, so `active_interval_s` / `idle_interval_s` are
   only what the asking half asks at, and the fallback for a socket that is not carrying.
3. **Each half does what it is good at.** `collector::Decide::see` reads one session list (pushed or asked for, it decides
   the same) and answers `Mode::Poll` or `Mode::Listen`. **Anything running** → poll at `active_interval_s`, because a
   pause or a seek is only as sharp as the gap between two sightings and the push carries no `ActiveWithinSeconds`.
   **Nothing loaded anywhere** → listen. **Everything loaded is paused** for `PAUSE_DEBOUNCE` readings in a row → listen,
   because a frozen position is exactly what a server has nothing to push about, and one 1 s poll per second of it asks the
   same question 3,600 times an hour. Any session running again (a resume, or somebody new) takes it straight back, within
   about a second.
4. `session_mode` calls the pause debounce `playing_poll` and keeps `fallback` for a socket that is genuinely unusable: an
   early attempt reported every pause as a fault for three seconds.
5. **`should_subscribe(playing, settling) = playing == 0 && !settling`, never the collector's own mode.** It keeps the
   subscription off for the whole debounce, and the transition itself sends the frame and waits for `Handle::settled`
   before publishing, so nothing is ever subscribed and polling at once. Listening needs the socket to have proved itself,
   and the proof is the list a subscription brings: an early attempt unsubscribed before it could arrive, so the fallback
   latched on for good (connected, never subscribed, polling for ever on exactly the idle server the socket is for).
   Anything that gates the subscription on something the subscription itself produces is that bug again.
6. **One at a time**: `SessionsStart`'s `"0,1500"` asks Jellyfin to look every 1.5 s and send what differs, which is
   silence on an idle server and a list every 1.5 s during a play, so while polling the pushes would be the same list a
   second time, uncompressed. `Handle::listen(false)` sends `SessionsStop` for the duration (the connection stays open and
   answering `KeepAlive`); `listen(true)` on the pass where the last play ends. A reconnect mid-play subscribes once for
   the proof and goes quiet again.
7. **The safety net is `Net`/`safety_due`**: a one-shot read after `PAUSED_SAFETY_SILENCE` of silence with something
   paused, or `SAFETY_EVERY` regardless. An early attempt restarted its wait on every push, so a client that kept reporting
   itself while paused meant the net never fell.
8. **Silence is nothing heard *and* nothing asked, and the attempt re-arms the clock that called for it**: `Net` is reset
   by a push, by any `/Sessions` read of FinStats' own (the beat during a play included) and by a fresh subscription, with
   `SAFETY_MIN_GAP` as a floor under all of it. Measured from the last push alone, as an early attempt did, it broke: since
   the subscription is off for the whole of a play, the first pause after a minute of one was already "silent", and the
   read that followed reset nothing, so 2,625 of them went out in eight seconds until a push happened along. Anything that
   makes a repeat depend on a clock the repeat does not touch is that bug again.
9. `Net::take_read` takes the gate slot *and* re-arms both clocks in **one call**, so a read that does not restart the
   wait it answers is not expressible (1.5.2); `Net::read` is private.
10. `Pass` (one object holding `Decide`, `Net` and the subscription as the wire has it) is what `run()` goes through, which
    also lets a test walk seven simulated minutes in microseconds
    (`three_minutes_of_playing_then_a_pause_is_almost_no_reads_at_all`) instead of sitting through two real ones.
11. **`sessions_slot`** makes **every** `/Sessions` read in the process, the socket's consistency check included, take a
    slot from one gate: `READS_PER_S`, `READS_PER_MIN`, refusals counted and logged WARN at most once a minute.
    `/api/status` publishes what the gate counted (`sessions_requests_last_min`), and `disagrees` holds a listening mode to
    `READS_WHILE_LISTENING`, because every word of the status was true throughout the storm of (8.).
12. With no socket at all, paused sessions are polled at `PAUSED_POLL_S` rather than every second. In the fallback the
    beat slows (`FALLBACK_IDLE_S`, `PAUSED_POLL_S`) but never below the owner's own interval: it only ever slows the beat,
    never hurries it.
13. **`attribute()` holds the invariant the whole thing rests on**: the gap between two sightings belongs to the state the
    play was *already* in, so however rarely a paused play is looked at, none of it becomes watch time.
14. **Silence is normal, not death.** Jellyfin answers `SessionsStart` with the list as it stands and then sends nothing
    until something changes, which on an evening when nobody watches is never. So two clocks, never one: `subscribed` (no
    session list within `SUBSCRIBE_MAX` of being asked) and `heard` (nothing at all, whether a list, a `KeepAlive` answer or
    a pong, within `lost_after(period)`, two unanswered keepalives = gone). Judging the connection by how lately a *list*
    arrived made 1.4.0 drop the socket every 15 s and poll right through, on exactly the idle server it was meant to spare.
15. **A verdict is never kept, and an absent list proves nothing.** A Jellyfin normally answers `SessionsStart` at once
    whether or not anything is loaded (measured: `0.0s after subscribing, 0 loaded`), but one was once seen not to (cause
    never established; most likely a server still starting). So after `SUBSCRIBE_MAX` FinStats says so once, the collector
    polls meanwhile, and the connection is kept, stays subscribed and re-sends `SessionsStart` every `PROBE_EVERY`; the
    first real push settles it. An earlier attempt concluded "cannot push" from that one silence and lost the socket
    altogether.
16. The `SUBSCRIBE_MAX` deadline runs from `listening_since`, not the handshake, or a socket kept quiet on purpose would be
    mistaken for a server that does not speak this.
17. `End::Unsupported` is only for a *shape* mismatch, which is real disproof; every other end uses the backoff.
    `looks_like_sessions` answers *what* differed for the log, and compares only sessions with a `NowPlayingItem` that both
    lists saw: an app open with nothing playing is in the push and not in `/Sessions?ActiveWithinSeconds=300`, which says
    nothing about the socket.
18. The collector's `on_socket` is the socket saying it carries, not a recent snapshot, and a list that arrives while the
    poller sleeps is *kept* (`pending`), because with push-on-change there may not be another for hours.
19. Any doubt falls back to polling on the next pass, because a play that is never seen cannot be backfilled.
20. **What it is doing is published, not inferred.** `publish()` writes the whole picture under one lock: `session_mode`
    (`idle_socket` | `playing_poll` | `paused_socket` | `fallback`), `socket_connected`, `socket_subscribed`,
    `poll_interval_s`, `mode_since`. It does so twice a pass, so nothing can be read half-applied while a pass blocks for a
    minute on a push. The two wire facts come from the socket task itself (`Handle::connected`/`subscribed`, atomics set
    where the frames are actually written), never from what the collector *asked* for.
21. `disagrees()` is the machine marking its own work: every rule is something a packet capture would contradict, and a
    disagreement is logged WARN only once it outlives `SETTLE`, because asking the socket to stop and the stop reaching the
    wire are two moments.
22. All of it is on `GET /api/status`, unauthenticated by the owner's decision: shape only, never a name, a title or even a
    count, answered from memory with no request and no query.
23. This replaced the one-a-minute reconcile read: `Source` has no `Reconcile` any more, and `CollectorStatus` swapped
    `last_reconcile_at` for `socket_live` (the socket carries, whatever brought the last list; `transport` then says which
    half we are in, and `/api/summary`'s `collector_live` follows `socket_live`).
24. **At crowd scale** (the capacity stage in `qa/`), the collector finds ended plays with a set (`ended_keys`) and closes a
    pass's ended plays in one transaction, detecting each title once (`close_ended`); the plays a pass sees begin, the
    progress it saves and the devices it saw are likewise one transaction each (`start_plays`, `save_progress`,
    `remember_devices`), never one awaited call per play: a crowd of 100,000 took 13 s to record that way, and one pass ten.

## Live channel (`live.rs`, `live.js`, `GET /api/live`, 2.3.0)

1. One WebSocket per tab, from FinStats to its own pages, carrying what changes on its own (`now_playing`, `downloads`,
   `summary`). Page loads stay on HTTP (`dataView`'s cache, prefetch, ETags and the API contract all rest on plain GETs);
   the socket only says that something a page already shows has changed. `/api/now-playing`, `/api/downloads` and
   `/api/summary` stay as they are, for scripts and as the fallback.
2. **One function per topic, two transports**: a frame's `data` is built by the very function the GET calls
   (`stats::now_playing_for` over `stats::marked_live`, which marks groups before anything is narrowed;
   `downloads::answer`; `stats::summary_for`), for the person the connection speaks for; never one snapshot broadcast to
   everyone.
3. **The Jellyfin side is not touched**: `live::run` reads what the collector and the queue last wrote, from memory, once
   a second while anybody listens, and asks Jellyfin, Sonarr and Radarr for nothing (`stats::marked_live` makes one read of
   the history, `groups::replayed`, when a title has more than one person on it).
4. **Push on change**: `now_playing` when `live::shape` changes (everything a card draws but position, time watched and
   transcode progress: the UI's `shapeOf`) or a position leaves the course the page counts along (`off_course`, the UI's
   own 3 s / 15 s), never for a clock moving on. A connection sends a topic only when its answer differs from what it last
   sent. Each topic is a `watch` (a latest value, not a queue).
5. **The downloads heartbeat**: a visible tab subscribed to `downloads` stamps the queue through `downloads::watched` every
   10 s, exactly as `?live=1` does; a hidden or unsubscribed one does not (`live::stamps`, the rule `visibleOnly` held).
6. **Permissions stay live**: the upgrade checks `Origin` with `api::cross_site` (a WebSocket upgrade is a GET, so
   `same_origin` does not see it), takes the cookie or a `Bearer` key, and never a key in the address nor a calendar key.
7. Each connection re-resolves what it presented every 30 s and at once when asked: `Feed::recheck` (every connection) for
   what changes everybody's access (`update_settings`, the users read, a restore), `Feed::recheck_person` and
   `Feed::recheck_key` for what changes one person's or one key's (their permissions, a sign-in that changed their row or
   let an older session go, a sign-out, a key revoked), since a loop of one's own sign-ins or keys otherwise made every
   open tab on the server ask the database again. A credential that no longer opens FinStats closes it with 4401.
8. A frame, like a message, is held to 4 KB: a frame is read whole before the message is measured, so each connection
   could be made to hold 16 MB.
9. **One person holds at most 20 (`PER_PERSON`), the oldest giving way** with 4429, as the oldest session does: a page in a
   browser's back/forward cache keeps its socket until the browser lets it go, and refusing the newest instead locked out
   somebody loading page after page in four tabs. `live.js` also gives the socket up on `pagehide`.
10. **The fallback is today's polling**: `follow(topic, {poll, every, onData, signal})` runs the page's old poll (same
    endpoint, interval and `visibleOnly`) whenever the socket is not live and stops it when the socket is back,
    reconnecting with backoff (1 s to 15 s); nobody is told. `/api/status` says nothing about it.
11. axum's `ws` brings tokio-tungstenite 0.29 beside the Jellyfin socket's 0.30.
12. **A connection keeps what its page asked for, not only what it was allowed** (`Tab.wanted`): a topic refused, or taken
    away at a recheck, is sent whole again at the recheck that allows it. The page is told `denied` once and never asks
    again, so a grant given back while it was open left it standing still until a reload.

## Play rows (`playback.rs`, `media.rs`)

1. **One play-row shape, four producers.** `playback.rs::PlayRecord` + `media.rs` (stream/transcode extraction, labels,
   `effective_play_method`) are shared by the live `collector.rs`, the Jellystat `import.rs`, the Streamystats
   `streamystats.rs` and the Tautulli `tautulli.rs`, so every source produces identical columns. A fifth tracker is a
   reader, not new architecture.
2. The collector inserts a row the moment a play is first seen (`active = 1`), refreshes it every 30 s, counts only
   un-paused time, and merges a restart within `merge_window_s` into the same row: only a row it saw itself, never one a
   backup brought back.
3. Consecutive sightings are diffed into `playback_events` (pause/seek/track/transcode timeline), at most
   `EVENTS_PER_PLAY` a play, since a client may pause and jump every second for as long as it likes.
4. **An event is a change, so the two sides of `diff_events` must be the same kind of value.** "Once a transcode, always a
   transcode" is applied to the new reading *before* the diff, never after: applied after, the kept record says
   `Transcode` while every following reading says what the client settled back to, and each one is a change. FinStats
   wrote one `transcode` event per second for the rest of the play, reading `DirectPlay: <reasons>`, a line that
   contradicts itself (migration 17 clears them). Anything sticky that the diff also reads belongs above the diff.
5. **Seen live is `source = 'live' AND source_id IS NULL`**, since a restored live play carries
   `restored:<user>:<start>:<end>` (`playback::restored_id`). A restored play counts as imported whatever source it claims.
6. **A stop is `position_s` where the play has one and `duration_s` otherwise** (FinStats' own plays and Streamystats rows
   that kept their runtime know where they stopped; Jellystat keeps a length, not a place, so the play is taken to have
   started at 0:00). The rule is `playback::STOP_S` / `STOP_MEASURED` / `PLAY_FRAC`, with `SEEN_AT` (0.8) for "seen", and
   every page uses it: the curve once trusted only FinStats' own positions while the activity list and the profile
   trusted Streamystats' too, so one play finished on one page and stopped twenty minutes in on another.
7. A season, episode or end number is kept only if Jellyfin could hold it (0 to `i32::MAX`, `playback::index_number` /
   `bounded_index`), in every producer, the library read included.

## Imports (`import.rs`, `api.rs`)

1. A new importer goes through `PlayRecord::insert_imported`, never `insert`, and ends in `import::finalize`.
2. An import skips a play whose start or end is before 1990 or past tomorrow, or that ends before it starts
   (`insert_imported`).
3. The four history writers hold their slot through `api::Running`, which says "interrupted" when dropped: hyper drops a
   handler whose connection closed, and nothing else gave the slot back. An upload that ends half way takes its file with
   it (`Running::with_file`).
4. **A row is read through `import::one_row` over a `RowBudget` reader, at most `ROW_MAX` (8 MB)**, as `LINE_MAX` (64 MB)
   holds a line: `[0,0,…]` is 32 bytes of memory for every two of file, so one row could run the process out of memory
   inside the transaction. Uploads stop at `api::UPLOAD_MAX` (4 GB).
5. Importing history and the Tautulli board need `see_everyone` (`may_locate`), since an import writes other people's plays.
6. **A file is read through before the write lock is taken** (`import::Spool`): Jellystat's and Streamystats' rows worth
   writing are kept aside beside the upload as they are read, with no lock held, and only those are written, in one
   IMMEDIATE transaction, once the file has been read to the end (Tautulli's reader already read first). Read inside the
   transaction, a valid head and gigabytes of rows of no known kind kept every other write waiting until the end. A spool
   is removed with the import, and at start with the uploads (`api::sweep_uploads`).

## Jellystat import (`import.rs`, the docs' `finstats/imports/jellystat.md`)

Learned from real exports; also documented at the top of `import.rs`.

1. `ActivityDateInserted` is the *end* of a play.
2. For episodes `NowPlayingItemId` is the series and `EpisodeId` the episode.
3. `PositionTicks` is unreliable and not imported; completion = watched ÷ runtime for imported rows.
4. There is no item type, so Live TV is inferred (video, not in library, no container → `TvChannel`).
5. The import is a single transaction and streams a multi-hundred-MB file line by line; the older Jellystat document is
   streamed too (`legacy_rows`).
6. A `jf_library_items` row for a library a read has already filled comes in as removed: a title that read did not see is
   gone, so it names the plays it had and is never one more live title ("Library reads" 13).

## Streamystats import (`streamystats.rs`, the docs' `finstats/imports/streamystats.md`)

1. Its backup is **sessions only** (no items, users or libraries) and holds three kinds of row that must be read
   differently, none of them flagged as such.
2. **A row carries one moment or two, and which decides what the moment means.** Two (`startTime` < `endTime`) is a play
   Streamystats watched, and the first is the real start. One (`startTime` == `endTime`) is a play it imported from
   Jellystat, and that moment is the **end**: `ActivityDateInserted` copied into both fields (1,799 of 3,167 rows in the
   file this was learned from). Read as starts, every one of those evenings moves forward by the length of the film,
   invisible until every chart is quietly wrong.
3. `isInferred` (or an `inferred:` id) is **not a play at all**: Jellyfin reported the item watched, so Streamystats wrote
   a row as long as the whole runtime for a viewing nobody saw. It is counted (`marked_watched`) and never imported.
4. `itemId` is the item in both kinds (Streamystats re-links renamed items); `mediaSourceId` is *not* an item id.
5. **Where the two kinds carry data is opposite, and reading the wrong one is worse than reading nothing.** A play it
   watched has no source media at all (`videoCodec`, `resolution*`, `audioCodec`, `videoRangeType` empty in every row) but
   does carry its transcode target. A play from Jellystat carries the whole session in `rawData` (read with
   `Streams::extract`) while its flat `transcoding*` columns are a copy of the *source* with `transcodeReasons: ["Unknown"]`; stored as a transcode that reads as a file transcoded into itself for reasons unknown, so those are read
   only from `rawData.TranscodingInfo`, and a third of them have none and get no transcode.
6. `positionTicks` counts only where the row also keeps the `runtimeTicks` it is a position in.
7. `isActive` is `true` on nearly every row of a backup and means nothing.
8. **Nothing is invented to fill a gap**: an item neither the row nor the library can type is left `Unknown`, not guessed.
9. The file is walked with a `DeserializeSeed` rather than loaded, in one transaction, each row through `import::one_row`
   ("Imports" 4).

## Tautulli import (`tautulli.rs`, the docs' `finstats/imports/tautulli.md`, 2.1.2)

1. It is Plex's history, so nothing in it shares an id with Jellyfin, and nothing is guessed.
2. **Two steps.** `POST /api/import/tautulli` only stores the backup (`.db`, or the zip Tautulli's download gives; `unpack`
   reads the one `.db` out of the central directory, stored or deflated, its entry inflated no further than its size,
   `inflate`) as `tautulli-upload.db` and answers the **wiring board** (`settings/wiring.js`: Plex users left, Jellyfin
   users right; the owner drags a cable from each, or clicks, or uses Enter and the arrows). `POST …/run` with the wires
   imports.
3. A Plex user **without a wire is not imported**, and several may go into one Jellyfin user (`check_wires`: one wire per
   Plex user). Every wire starts unplugged: no matching by name (owner's decisions).
4. The backup's `users` table holds every Plex user's tokens and e-mail: `PLEX_USERS_SQL` reads ids, names and counts only.
   The file is removed after the import (whatever its outcome), on cancel, at start-up and after six hours (`sweep`).
5. **Titles carry no provider id** (`plex://`, `local://`), so a play is written under a stand-in id (`plex:<rating key>`,
   its show `plex:<grandparent>`) with the names Jellyfin would give it, a film as `Title (2008)` so the year travels, and
   `relink` attaches it during the import's own `finalize`, or at any later library read when the title arrives: one
   matching rule, not two.
6. **A viewing is one `reference_id` and one title**: a resumed play chains its rows by `reference_id`, but Plex plays a
   show's theme while the show is open and Tautulli chains the episode onto that track; grouping by reference alone counted
   those episodes as music. `source_id` is the viewing's first row (`tautulli:<id>`), which is its reference everywhere else.
7. **Read every value leniently** (`num`, `text`): Tautulli writes `''` where it has no number and some numbers as text,
   and a strict read failed a whole real import on the first film without an episode number.
8. Music, clips, photos and Live TV are counted (`other_media`) and not imported. Learned from a real backup: 1,390 rows,
   1,031 viewings, 721 films and episodes, every one of them imported.
9. **The file is somebody else's**: `open` reads it defensively (`SQLITE_DBCONFIG_DEFENSIVE`, no trusted schema),
   untrusted, from plain tables only (`tautulli::READ`; no view, virtual table, or generated or hidden column, which runs
   an expression on every row read: 200 MB a row from an 8 KB file), opened `immutable=1` (a WAL-mode backup otherwise
   leaves `-wal`/`-shm` beside it, which `sweep` also removes), and refused when the ids its three history tables are
   joined on repeat (a few MB of rows under one id made the join their product and ran the process out of memory).
10. A text is read to at most `TEXT_MAX` (1,000) characters and a blob not at all; the board counts every Plex user's
    viewings in one pass; `run` reads the file whole before taking FinStats' write lock; and one upload arrives and is
    unpacked at a time (`TautulliArriving`), each in a part of its own, swept by prefix.
11. Cancelling a waiting board needs `see_everyone`, as the board does.

## Duplicate rule (`playback::already_recorded`)

1. **One rule for a play the history already has**, shared by every importer and the backup restore. `source_id` stops a
   file being imported twice and says nothing about the same evening arriving from another tracker under another id. So:
   its own id first, else the same person, the same item, and one of the play's **two ends** close enough.
2. **Between** sources that is `merge_window_s` at *either* end, because the trackers disagree about the *start*
   (Jellystat keeps only the end, so the start is derived from the seconds played and every minute paused moves it later)
   while all three agree about the end. Measured on real history (3,162 plays a Streamystats export and a Jellystat import
   held in common): either end recognised 2,750, the start alone 2,527, and the start's misses ran in an unbroken smear
   past ten minutes where the ends fall off a cliff inside one.
3. **Within** one source it is the very same second and nothing wider: a tracker never exports the same play twice, so a
   second row of the same item minutes later is a restart the viewer really made, and a window there would silently drop
   it, on an import and, worse, on a restore of FinStats' own backup.
4. The same person's same two seconds from the same source are the same play whatever title it is under, because
   re-linking rewrites `item_id` and nothing else: a backup from before a re-link restored every moved play a second time
   otherwise.
5. **Both statements of the rule (`SAME_PLAY_SQL`, `RELINKED_DUPLICATES_SQL`) are index ranges** over `(user_id, item_id, started_at)` and `…ended_at` (migration 25), and a test holds their plans: written as `ABS(started_at - ?) <= window`
   no index could narrow them, a 120,000-play import took five minutes, and the sweep held the write lock for 25 s at every
   start and library read.

## Relink (`relink.rs`)

1. Jellyfin ids derive from the path, so a renamed file orphans its plays; `relink.rs` re-attaches orphaned plays to
   renamed items after every library read, import and start-up.
2. The rules, in order, only when unambiguous: the owner's `located` choice ("Locate" 5); a provider-id match; the name as
   written against Jellyfin's name **or its original-language name** (`items.original_title`, migration 32; Plex and
   Tautulli often know a title only by that: 오징어 게임 for Squid Game); the name cleaned as search cleans it
   (`title_key`: `fuzzy::normalize`, no year in the name, no leading article; "- " and "– " are one); episodes by series +
   S/E number; a track only on the same album by the same artist where both are known (half the albums there are have an
   "Intro").
3. A year may be one off (catalogues disagree: Kingsman is 2014 to Plex and 2015 to Jellyfin), and of several within one
   the exact year wins (`pick`); two off is a remake.
4. The cleaned index (`Keys`) is read once per re-link and only when a name as written found nothing, because re-linking
   runs at every start and after every library read.
5. **Re-linking is the only thing that rewrites `item_id`, so it is the only thing that can turn an imported play into a
   duplicate of one already here**: the `already_recorded` check ran before the id moved. `relink::relink_orphans`
   therefore re-applies the rule (`playback::drop_relinked_duplicates`) to each title it moved plays onto, in one savepoint
   with the move (`move_plays`), and takes `merge_window_s` as an argument so none of its three callers can forget to.
6. Only an imported row is ever removed, never one the collector recorded (its row carries a timeline no import can have),
   and rows of one source are never compared with each other. A real history had 363 such pairs, 189 rows, mostly music,
   music being what gets re-added and renamed. **Anything that rewrites which item a play points at re-applies the rule**,
   or it re-creates that bug.
7. It must stay cheap when there is nothing to do, since it runs at every start and after every library read and nearly
   always finds nothing: `ORPHANS_SQL` steps from title to title through the title index and returns at once when no title
   is orphaned; only the plays of orphaned titles are read. Sweeping the whole history, finding orphans by grouping every
   play, took 5 of the 8 seconds of a start on a million plays.
8. Migration 26 swept every install once, with its own merge window, for those coming from before the sweep existed. The
   rule's text is one macro (`relinked_duplicates_sql!`) so the statement and the migration cannot drift.
9. The duplicate sweep never keeps a deleted row over a live one (the keeper fragment of `relinked_duplicates_sql!`'s second
   arm); the one-argument arm is migration 26's text, byte for byte.
10. Groups are detected again for the titles re-linking moved plays onto ("Groups" 9).

## Locate (`locate.rs`, Settings → Unlinked media, 2.1.2)

1. `relink` attaches only what a name rule finds without guessing; the rest (Plex's "Star Wars: Episode V - The Empire
   Strikes Back" is Jellyfin's "The Empire Strikes Back"; TVDB moved an episode into another show's specials) is listed by
   `missing()` (orphans through `relink::ORPHANS_SQL`, films and episodes only).
2. Suggestions come from `candidates()`: `fuzzy::Query` scored **both ways round** (search's own direction needs every
   typed word in the title, which a longer Plex name never is), the original title too, and an episode by its own name
   anywhere (`title_key`), so a moved special is found.
3. The library is read once per request (`Library`).
4. The owner picks; `locate()` moves the plays through `relink::move_plays` (the same savepoint and duplicate rule) by
   `move_onto`, which also gives them the target's type and place. A film may be located as an episode (anime films are
   often a show's special), but never as a whole show.
5. The choice is kept in `located` (migration 33), which `relink_orphans` consults **before any rule**, so a re-import
   attaches by itself. It is the owner's and not Jellyfin's to give back, so it is in `backup::TABLES`, restored before the
   restore's own re-link.
6. Unlinked media is not narrowed by library access.

## Stats layer (`stats.rs`)

1. Every query goes through `Scope` → `Cond`: the time window ("last N days" = N full local days, so chart buckets and
   totals agree), user/library filters, `min_play_s`, and the rule that **non-admins without `see_everyone` are
   force-scoped to their own `user_id`, and IPs/device ids/file paths are only sent with `see_network`/`see_server`,
   enforced server-side**. A library removed from Jellyfin stays listed while it has history, and for them that is their
   own history.
2. In `stats.rs` decide by the specific permission (`scope.perms.see_network` for IPs, `see_server` for paths,
   `see_everyone` for whose rows), never by `is_admin`.
3. `row_json` maps SQL rows to JSON by column name (`BOOL_COLS` / `JSON_COLS` decide bool and JSON columns), so adding a
   field is usually just adding a column to a SELECT.
4. Local-time bucketing relies on SQLite's `'localtime'` and the process `TZ` (the Docker image ships tzdata for this).
   Rust that needs the local day of many moments asks through `LocalDays`, once per quarter hour (every offset and clock
   change falls on one), never once per play.
5. Do not use `#[serde(flatten)]` in `Query` structs: serde_urlencoded then hands numbers over as strings and every
   numeric filter 400s.
6. Paginated lists sort on the server: the endpoint whitelists `sort` via `stats::order_by`.
7. Graphs' one query of its own is `stats::breakdown` (`/api/stats/split`): plays by app, person, source or sent
   resolution, day or month, each split by kind of media or by how it played; the day's rule is shared with
   `overview.daily` through `periods`; the concurrency series says each day's most of each way of playing.

## Library access (`access.rs`, migration 41, 2.3.1)

1. **Each person sees of the library what Jellyfin lets them see, and nothing more** (owner's decision, 2026-10-28).
   Jellyfin keeps it in each user's policy: every library or the ones named, and inside them a rating cap, blocked and
   allowed tags and kinds of unrated item.
2. `access::policy` reads it from the users read and from every sign-in (`remember_sign_in` takes a `Policy`) into
   `users.access_all`, `access_libraries` and `access_hides`.
3. FinStats compares the libraries itself; what hides titles inside them is Jellyfin's own judgement (a rating's number
   comes from Jellyfin's tables for its country, a tag from the title or any folder above it), so FinStats does not redo
   it. For somebody something hides titles from, `sync::read_visible` asks Jellyfin `/Items?UserId=` (ids only, per
   library) which titles it lists for them and keeps them in `visible_items`, which counts only while `users.visible_for`
   is the policy it was read under.
4. It is read when a policy changed (after the users read) and again after library reads and metadata looks; the upgrade
   step `library-access-first-read` makes the first start read the users at once.
5. **Jellyfin administrators see everything, and anything not yet read opens nothing**: a policy FinStats cannot read is
   no library at all, never every library.
6. `Scope` carries it (`access`, `viewer`), resolved once with the rest of the request. It narrows a library's titles,
   Recently added, the libraries list and pages, a title's page and a person's page (a title or library Jellyfin hides
   answers the same 404 as one that does not exist), search (titles and people), library insights, the overview's library
   counts, pictures (`artwork::may_show`), the watchlist (a hidden title reads as not on the server, its id is refused like
   an unknown one, its arrival is told to nobody it is hidden from), Coming up and the calendar feed (a title the library
   does not have yet belongs to no library and stays), and what a personal destination is told of an arrival
   (`notify::owner_sees_title`).
7. **History is not narrowed**: a person's own plays of a title they can no longer open are still theirs, and
   `see_everyone` still sees everybody's.
8. Library health and Capacity stay whole for `see_server`; Unlinked media and `/users/{id}/shows` are not narrowed.
9. None of it is in a backup: Jellyfin gives it back.
10. **Library access is not a permission**: what of the library a person sees is Jellyfin's to say, for every permission
    alike, and only a Jellyfin administrator sees all of it.
11. **Sonarr's and Radarr's posters and the watchlist's next episode follow it too**: `pipeline::poster_is_listed` serves a
    poster only for a calendar or request row whose title the caller may see, and `watchlist::listing` reads the calendar
    through `pipeline::seen_by`. Counting the services' small ids upwards told which hidden shows had dates, and a list
    named a hidden show's next episode by day, season and number.

## Playback insights (`stats.rs`, 2.0)

1. Where a title loses its viewers, drawn from where each play stopped ("Play rows" 6). Measured and estimated stops are
   never mixed silently: every curve carries `measured` and `estimated`.
2. The grid (`bucket_width`) keeps any runtime to sixty points; under `MIN_CURVE_PLAYS` (3) or without a runtime there is
   no curve, not a thin one.
3. Events exist only for live plays, so rewinds (a seek whose `from_s` is past its `position_s`) and subtitle switch-ons (a
   play's *first* subtitle change, to a track; the state before it is never an event, so a later language change is not a
   second switch-on) are counted over those.
4. `from_s` came in migration 21, backfilled from the seek label in SQL; `playback::backfill_seek_origins` is the same
   statement for a restored backup from before it, and a test holds both to the same answer.
5. A show's episodes carry `users` and `finished`, because "everyone quits episode three" is a fact *between* episodes.
6. `GET /api/stats/files` builds three lists from everyone's plays and answers empty lists without `see_everyone`.
7. The broken-file list uses `Scope::cond_any_length()` (the minimum play length is precisely what a broken file never
   reaches) and reads nothing from `server_events`: Jellyfin's activity log carries no playback errors (every Error row on a
   real install was a failed sign-in).
8. Server-wide event queries go through `idx_pbe_kind`, and a test pins the plan.
9. A seek under `SEEK_TOLERANCE_S` was never recorded, so a short rewind is invisible by design.

## Groups (`groups.rs`, `/together`)

1. Inferred, because `/Sessions` exposes no SyncPlay groups: plays of one item by ≥ 2 different users starting within
   `group_window_s` (default 60; real data shows a third of genuine groups start 6–60 s apart) **of each other's own start,
   pair by pair**, **or ending within it at nearly the same measured place** (`max(window, 30)` s; SyncPlay with a late
   joiner, who starts minutes after the one waiting paused at 0:00), and overlapping ≥ 2 min, share `playbacks.group_id`
   (= lowest play id in the group).
2. Rule 3 (2.3.1): pair by pair, never chained. Chaining a start to the start before it let anybody keep a title playing
   back to back all evening and put everyone who played it into one group.
3. **Somebody with more than one play overlapping a group's span is left out of it** (`RULE` 4): one person keeping a title
   in rotation from device after device, a new play every minute, was otherwise grouped with everyone who played it
   meanwhile, names and whole watch times.
4. `groups_detected` notes the `RULE`, so a new rule regroups the whole history once.
5. The fold is pure and reusable (2.0): `sessions_for` (a `Window` with an end, so the recap can ask for one person and one
   year) → `fold_sessions` → `pairs`, `per_bucket`, `people_shares`, and the three lists the cards always read; `answer`
   assembles `GET /api/stats/groups` from them.
6. **A pair's time together is the shorter of the two stays** (an evening of three counts for each of its three pairs; a
   session's own `together_s` describes any two of its members), and **a session belongs whole to the bucket it started
   in** (`alone_s` = scoped watch time in the bucket minus grouped time, never below zero). "Time together" is the
   second-longest stay in a session.
7. Without `see_everyone` the pairs must include the caller and `people` is the caller alone: a companion's name is theirs
   to see, a companion's time alone is not.
8. **Without `see_everyone` a group is only the people the caller's own play met** (`groups::met_by`, `sessions_met_by`;
   a play's companions and its group size through `stats::play_select`'s `GROUP_SIZE_MET`, `MET_SQL` held to
   `MIN_OVERLAP_S` by a test), for the time it met them: a group is joined through anybody in it, so its whole membership
   told them of people their play never ran beside.
9. `detect()` re-runs per item when a play ends and for the titles re-linking moved plays onto, fully after an import or a
   restore and when the setting changes.
10. **Any Jellyfin user can make a title's history long**, so `cluster` compares a play with at most `PEERS` others, leaves
    out at once a play too short to overlap anybody, and `detect` clusters on a plain read (`plan`) and takes the write
    lock only to write what changed: in a transaction, one person's 40,000 plays in a row held the lock for six seconds at
    every play's end.
11. `detect` writes only the plays whose group changed: resetting every group to NULL and writing it back rewrote every
    grouped play of the history at each start, import and play end.
12. **A start regroups only what the last run left behind** (`regroup_at_start`): plays it was still recording when it
    stopped were all saved after that run began, so the key `groups_detected` notes version, window and when, and the next
    start regroups titles with a play ended since, and everything only for another version or window. Regrouping the whole
    history at every start took 24 s on ten million plays.
13. Running streams are grouped separately by `mark_live` (same title, different users, starts within the window *or*
    positions within `max(window, 30)` s), before `/api/now-playing` narrows the list to the caller.
14. **A position anybody may seek to proves nothing to somebody who may see only themselves**: for them `now_playing_for`
    answers `group_seen_alone` as `group`, companions who started with them, and none while they have the title playing
    more than once (playing a title from many devices along its length told them who was watching it). A stream that
    follows one of the same title by the same person names nobody and is named to nobody (`groups::replayed`, the one
    history read `marked_live` makes, only when a title has more than one person on it).
15. **Measured at crowd scale** (the capacity stage in `qa/`): `mark_live` pairs streams of one title only, on plain values
    read once, and names at most `COMPANIONS_NAMED` companions (the size is exact); comparing every stream with every other
    and listing every companion took 40 s at 10,000 streams and 23.7 GB at 25,000.
16. A deleted play's `group_id` is cleared and its title regrouped ("Trash" 1).

## Search (`fuzzy.rs`)

1. `/api/search` scores every library title in Rust instead of using `LIKE`: normalised (case, accents, punctuation,
   leading article), every typed word must match some word of the title (exact > prefix > substring > typo; typos only for
   words of 4+ letters, with swapped letters as one slip). ~70 ms over 5.5k titles; the search (`search.js`) debounces.
2. The same query also scores the ~10k names in `item_people` (`people` in the answer): bare names first, details only for
   the best 60, because grouping and counting every person's titles up front took 350 ms.
3. The Activity and Server-log `q` filters stay in SQL, but word by word too.
4. What a person types goes through `Query::typed`: nothing past `TYPED_CHARS` (200) or `TYPED_WORDS` (12) is answered,
   since 400 KB of one letter cost tens of seconds of a pooled connection per request. `Query::new` is for titles read from
   the library.

## Profiles (`profile.rs`)

1. Show progress counts only episodes that exist as files (`path`/`size_bytes` set; the sync asks Jellyfin to exclude
   virtual items), and season 0 is skipped.
2. "Seen" merges three sources in order: a recorded play ≥ 80%, Jellyfin's played flag (`user_items`), a manual mark
   (`manual_seen`, written via `POST /api/me/seen` for the caller only; FinStats never writes to Jellyfin).
3. Streaks are all-time and share `recap::longest_run`.
4. The Cast & crew row on `/items/:id` and the person pages (`/people/:id`, `stats::person_detail`) read `item_people` and
   are scoped like any other stats query.

## Watchlist (`watchlist.rs`, `/users/:id/watchlist`, 2.1)

1. One person's films and shows to watch, **their own and nobody else's**: every endpoint (`/api/me/watchlist*`) acts on
   the caller and takes no `user_id`, so no permission, not even a Jellyfin administrator's, opens another's list (a
   calendar key is refused by `AuthUser` like everywhere).
2. An entry names its title by the item (`item_id`) or, not in the library yet, by provider ids, and keeps a snapshot
   (kind, title, year, ids) so it reads without the library.
3. **Copies are one title**: items sharing an id and a type are the HD and 4K film; the lowest id stands for them
   (Pipeline's rule, not `relink.rs`'s, since nothing here rewrites history), and ids that lead to *different* titles
   attach to nothing (`find_title`).
4. `resolve` runs at the end of `backfill_playbacks`, so every library read, metadata look, import and restore attaches
   waiting entries, follows a renamed title, keeps attached snapshots current and merges every entry of one title into the
   oldest `added_at`: all of them, before the survivor moves, since one may hold the item, and the unique `(user_id, item_id)`
   index refusing it would fail the whole library read.
5. **State is worked out at read time and never stored** (`listing`): watched by the profile's one reading of "seen"
   (`profile::films` / `episodes`, one `PLAY_FRAC` and `verdict`), started, on the server, requested (someone else's
   request only with `see_everyone`, Pipeline's rule), coming up (`pipeline::entries_for`), left the library, not on the
   server.
6. At most `MAX_ENTRIES` (1,000) per person. In `backup::TABLES`, merged per person and title on restore.
7. The page and every toggle (a film's or show's page, and an episode's or season's for its show; Pipeline's Upcoming rows;
   the dashboard's Coming up and Recently added cards; search with Ctrl+Enter) read `/api/me/watchlist/keys` once per
   visit (`watchlist.js`), never a request per poster.
8. On a dashboard poster the toggle shows on hover or keyboard focus, and always where `(hover: none) and (pointer:
   coarse)`, not `(hover: hover)` for the opposite, which headless browsers do not report, so the QA journey could not see
   the rule work.
9. None of it shows on a server whose `/auth/me` lacks `features.watchlist`.
10. Its notification, `watchlist_available`, is `notify::Need::Owner`: the owner's own destinations only, never a server's,
    and left out of every other person's notification history, administrators' included.
11. **An arrival is a title that was missing and is here now** (`missing`, `arrived_at`, migration 31, noted by `resolve`
    at every read), never just a new item: a replaced file is a new path and so a new Jellyfin id with a fresh
    `DateCreated`, and judged by the item alone every Radarr upgrade said "now on the server".

## Timeline (`timeline.rs`, `/users/:id/timeline`)

1. One person's plays, newest first, folded by the pure `fold()`: plays that follow each other with the same key (series +
   season, album + artist, else the item) are one stop.
2. Pages use a `(started_at, id)` cursor and only give out a stop once the play after it has been read, so a stop is never
   split and the stops do not depend on the page size.
3. In the UI (`pages/timeline.js`) the column count comes from a `ResizeObserver`: rows alternate direction by being
   `direction: rtl` (their cards reset to `ltr`), a bend joins each row to the next, and below 620 px it becomes one
   straight line (`.is-line`). DOM order stays chronological, so keyboard and screen readers follow the trail.

## Browse (`browse.rs`, `GET /api/libraries/{id}/titles`, 2.3.0)

1. Every film, show or album in one library, a page at a time, filtered, searched and sorted on the server.
2. Scoped to what Jellyfin lets the caller see ("Library access"): a library Jellyfin keeps them out of is a 404, and inside
   one only the titles listed for them. Nothing here is about plays.

## Recently added (`recent.rs`, the dashboard shelf)

1. Episodes of one show added on the same local day fold into one entry (pure `fold()`); everything else is its own.
2. It must stay cheap because the dashboard loads it: the query walks the partial index `idx_items_added` newest first and
   `fold()` stops reading at the first row older than the last entry's day (about 50 rows for 30 entries).
3. The `+i.type` in its SQL is deliberate: without it the planner picks the type index, sorts every episode and takes
   ~270 ms on a 23k-episode library; a test asserts the plan.
4. Scoped to what Jellyfin lets the caller see, still walking its date index; its notification goes to a personal
   destination only for a title the owner may see (`notify::owner_sees_title`).
5. The shelf scrolls by hand (`shelf()` in `pages/dashboard.js`): one glide towards a target for Shift+wheel, keys and the
   arrows. No CSS scroll-snap on a row like this: a wheel notch shorter than half a card springs back, so Firefox users
   could barely move it.

## Recap (`recap.rs`, `story.rs`)

1. One person's year: the caller's own, or, for a Jellyfin administrator only (`is_admin`, not a permission), the user in
   `user_id`, or the whole server's (`scope=server`, 2.0), which goes through `server_edition` and so ranks nobody and
   names nobody (companions, rank and apps taken out). `whose_year` is the one place that rule lives; the page and the
   cards both go through it.
2. The recap ignores permissions otherwise (own for everyone, any one user for Jellyfin administrators), except its `rank`:
   **the rank is left out altogether without `see_everyone`** (`rank_for`): a position watched while one's own total grows
   an hour at a time tells each other person's yearly watch time to within the hour.
3. It excludes Live TV item types and defaults to the year that is "ready": the current year in December, otherwise the
   previous one.
4. 2.0 added `together` (the group fold for one person and one year; companions named, at most three), `finished` (by
   `profile::episodes`, the same "seen" as show progress; dropped = begun this year, under half seen, nothing for
   `DROPPED_QUIET_S`), `requests` (null when no request was ever recorded) and `versus`.
5. Its "most watched people" read `item_people` ("Library reads" 7).
6. **The year as a story (`story.rs`)**: every card is drawn from `StoryYear`, a typed copy with no field for a companion,
   a rank or an app, so a card cannot name anybody however the recap grows. Never draw a card from the recap's JSON.
7. `/api/recap` answers `story` (which chapters have cards), so the page never recomputes it.
8. `recap::announce_ready` tells each person who watched in December, once (`notify:recap:{year}:{user}`).
9. Cards (Recap, public profile) are drawn in a language their font has letters for (`i18n::drawable`: Japanese, Korean,
   Chinese and Hindi draw in en-GB).
10. **One person's Together chapter names only who their own play met** (`together` reads `groups::sessions_met_by` for a
    person, `sessions_for` only for the whole server's year, which names nobody), as the Together page does without
    `see_everyone` ("Groups" 8): the whole group, joined through somebody else, named people their play never ran beside.

## Library health (`health.rs`, Server → Library health, `/server/health`, 2.2.0)

1. A linter for the library: every rule (gaps, season and episode drift, copies and versions, thin files, dubs that stop,
   never identified) is a pure function over `Item` rows, and every threshold is a named constant with its reason.
2. Resolution classes are coarse on purpose (4K, 1080p, 720p, SD) and go by width *or* height, so a scope 1920×800 and a
   pillarboxed 1440×1080 are both 1080p.
3. Findings are worked out **after a library read and after a look for metadata changes, beside `backfill_playbacks` and
   never inside it**, and a page only reads what was stored (`health_findings`, `health_libraries`, migration 35). The
   look for changes waits until a library read has computed once (`health_computed_at`).
4. **A gap needs `index_number_end`** (migration 34): "S01E01-E02" is numbered 1, and without where it ends every such file
   reads as a hole. Migration 34 forced one library read to bring it ("Scheduling" 9).
5. Copies use the rule the watchlist and Pipeline use (items of one type sharing a provider id, unless the ids among them
   lead to different titles), read from `items.provider_ids` with `rebuild_external`'s own filter, and a test holds them
   to `watchlist::find_title`. A show's episodes are copies through it, by season and episode number.
6. The same thing twice in one class counts as waste; two classes are versions and never do.
7. The dub rule is the Languages card's (`media::language_counts`, lifted out of `item_detail` under a test that pinned the
   card's answer).
8. Every finding has a stable key (kind and subject ids) and a fingerprint of what it read; a dismissal (`health_dismissed`,
   in `backup::TABLES`; a restore keeps one already here) holds while the fingerprint does, so a replaced file is looked at
   again.
9. Reading needs `see_server`; dismissing needs `manage` **and** `see_server` (`may_dismiss`), and is audited. Nothing in an
   answer is about plays or people. It is the whole library for `see_server`, not narrowed by library access.

## Capacity (`capacity.rs`, Server → Capacity, `/server/capacity`, 2.3.0)

1. **Will this server cope, from what FinStats already holds**, never "your server can handle N", which it cannot know.
2. Every rule is a pure function over plain rows, recomputed after a library read and after a read of the server's details
   (`recompute`, stored in `capacity_findings` and the `capacity` setting); a page only reads what was stored.
3. The busiest moments come from the plays (Tautulli's are Plex's, another server, and left out).
4. A disk is the library folders on one device Jellyfin names, or at one free space without a device, less the folders
   inside Jellyfin's own (its Collections).
5. **A disk's growth is each library's on it, from a month after that library's first file** (its first month is its
   filling: a library copied over last week is not "full within a month").
6. **A file that took another's place adds only the difference** (`take_places`: a removed row of the same title in the same
   library, last seen within `REPLACED_WITHIN_S` of the new file's arrival, or at any time when an import wrote it): an
   upgrade from Radarr or Sonarr, and a file Jellyfin gave a new id, read as all new otherwise.
7. Findings are dismissed with Library health's `health_dismissed`, and a finding no longer found takes its dismissal along.
8. **Without `see_everyone` a busiest moment is its day** (`by_day: true`, the moment at local midnight) and only moments of
   at least `FEWEST_TOLD` (3) streams are listed: on a household of two or three, "one stream at 21:14" that was not one's
   own is somebody else's evening, which the stats pages keep from them.
9. Otherwise it is the whole library for `see_server`, not narrowed by library access.

## Auth (`auth.rs`, `keys.rs`)

1. Login forwards credentials to Jellyfin's `AuthenticateByName`, immediately logs that Jellyfin session out, and mints an
   own opaque session token (stored hashed, HttpOnly SameSite=Lax cookie).
2. **Two credentials, one resolution point**: `auth::resolve` reads `Authorization: Bearer fs_…` first (`looks_like_key`
   gates before any query; an invalid header never falls back to the cookie: header beats cookie), else the cookie, and
   both end in the same `AuthUser`, which carries `credential` (`Session` | `Key{id, scope}`) and `ip`.
3. A key (`keys.rs`, `api_keys`, sha256 of `fs_` + 64 hex, shown once) is resolved against the **live** user row and the
   live grants through `effective`, never a snapshot, so a lost `sign_in` or a demotion reaches every key at once.
   `touch_key` writes `last_used_*` at most once a minute.
4. **Until FinStats is set up, the wizard asks for a one-time code written to the log at start** (`auth::new_setup_code`,
   `AppState.setup_code`; 428 without it, and both setup routes count against the sign-in limit), or whoever reached the
   port first could point it at a Jellyfin of their own and be its administrator.
5. **The limiter has three tiers**: 10 tries in five minutes per address (an IPv6 /64 counting as one), 50 per IPv6 /48
   (`WIDE_ATTEMPTS`), and past 60 failed sign-ins a minute on the whole server (`FAILURES_PER_MINUTE`) nobody's attempt is
   forwarded until the minute has passed. **Only attempts put to Jellyfin count there, those still waiting on it
   included** (`forward` takes a place in the same step as the look at whether there is one, `Forwarded::refused` makes it
   a failure, and dropping it gives it back): counted, names refused here let anybody with six addresses stop everybody
   signing in at no cost, and looked at apart from taking a place, 500 guesses at once all reached Jellyfin. Home
   addresses get no way round it (the owner, 2026-10-30).
6. **Full (`ATTEMPTS_REMEMBERED`, 10,000), the limiter refuses a new address** until a window ends: letting the oldest go
   let a guesser with more /64s than that take them in turn and guess without a limit.
7. A failure is audited once per address and window (`sign_in_failed`), not once per try.
8. **Only a name that may use FinStats is put to Jellyfin** (`may_try`): once a users read has succeeded, a name that is
   nobody, disabled, removed or not let in is answered as a wrong password (401) without asking Jellyfin, and no sooner
   than Jellyfin typically takes to turn one down (`FailureTime`, a running average, with jitter), so the time does not
   tell which names exist. Jellyfin counts every failure against the account and locks it, judges its remote-access rule
   on FinStats' own address and logs FinStats as the one guessing.
9. Before the first users read (the wizard's own sign-in) Jellyfin alone decides, and the 403 "not given access" is only
   seen then; a new Jellyfin user signs in after the next users read.
10. **Jellyfin's "allow remote connections" is applied to the real address** (`kept_home`): Jellyfin reads the switch from
    the address asking it, which is FinStats', so with `EnableRemoteAccess` off a sign-in from outside the home network is
    refused (403 `server.auth.home_only`). **And on every request after it**: the switch is kept on the users row
    (`remote_access`, migration 42, from every users read and sign-in), and `resolve_session_in` and `resolve_key_in` refuse
    such a person from outside (401), the live socket at its next look. Checked at sign-in alone, a cookie or key carried
    out of the house opened everything from anywhere. The address it is judged by is `judged_ip`: none at all when a peer
    FinStats does not believe as a proxy sends `X-Forwarded-For`, since every request is then that proxy's private
    address, home for everybody; the rule fails closed then.
11. **`FINSTATS_TRUST_PROXY` names the proxies** (`auth::Proxies`): a list of addresses and CIDR ranges whose
    `X-Forwarded-For`, `-Host` and `-Proto` are believed, the header walked from the right past every listed hop (ports
    stripped). `1`/`true` believes only a loopback or private-network peer, one hop, and says so at start; unset believes
    nobody and logs one WARN when a peer on this network sends the header. Believed from any peer, as it once was, anybody
    beside the proxy chose their own address for the limiter, the audit log and a key's last use.
12. One person keeps at most `SESSIONS_PER_USER` (30) sessions, the oldest giving way at the next sign-in
    (`store_session`), or a password in a loop fills the table.
13. A sign-in that works gives back its own attempt and nothing more (`signed_in`): clearing the count let anybody with an
    account guess without a limit.
14. **On https the session is `__Host-finstats_session`** (only this host may set it, so a sibling name cannot toss in a
    cookie of its own); the plain name is still read on http and for older sessions, but not when two of it arrive
    (`cookie_token`).
15. **A token says where it was begun** (`h` on https, `p` on http), and an `h` token under the plain name is refused:
    while the browser held no `__Host-` cookie, a sibling's tossed plain one was the only one sent.
16. A sign-out ends every session the browser sent, both names.
17. An `Authorization` header that is not `Bearer` beside a cookie is left alone (a proxy's own Basic sign-in, nginx
    `auth_basic`, otherwise refused every cookie request), while a key sent wrong still never falls back to the cookie.
18. **A session is resolved like a key** (`resolve_session_in` joins `users`; `sessions.is_admin` is only the fallback
    before the users read has written the row, which the wizard's own sign-in is): an administrator demoted, disabled or
    deleted in Jellyfin used to keep full access for the session's 30 days.
19. Because the session trusts the row, every sign-in writes Jellyfin's fresh answer into it (`remember_sign_in`), or a row
    a Jellystat import wrote from the file's `IsAdministrator` could crown somebody Jellyfin did not.
20. `KeyScope::Calendar` opens only `/api/calendar.ics`: the `AuthUser` extractor refuses it with 403 everywhere, and the
    feed's own `CalendarKey` extractor takes `?key=` or the header and never the cookie (`ical.rs`).
    `pipeline::entries_for` carries no name by construction, since a subscribed calendar syncs through somebody's cloud.
21. Minting or revoking through a key is refused (`keys::only_a_session`), and so is whatever else outlives a key
    (`keys::needs_a_session`, 403): editing permissions, the access settings in `put_settings`, a restore (which can bring
    grants back) and adding, changing or testing a service; a leaked key revoked must take everything it could do with it.
    That includes switching off what would notice it: changing `ADMIN_SETTINGS` or `SECURITY_SETTINGS` (sent unchanged,
    as a whole form is, they pass), the schedule of `ADMIN_SCHEDULES` (`watched_schedule`), muting an alert or resolving
    them all, moving a backup to the trash and removing a notification destination.
22. `/auth/me`'s `jellyfin_details` is null unless `jellyfin_public_url` is set, and a title's `jellyfin_link` and Library
    health's links fall back to the address FinStats reads Jellyfin at only for administrators.
23. `api_keys` is never in a backup.

## Permissions (`auth.rs`, `api.rs`)

1. `AuthUser.perms` (`Perms`: `see_everyone`, `see_network`, `see_server`, `see_downloads`, `notify`, `manage`) is rebuilt on
   every request from `user_permissions` ∪ `Settings.default_permissions`; `sign_in` (or `allow_user_login`) gates access
   at all; Jellyfin admins always get `Perms::ALL`. Grants only add; there are no denies. The grantable names are
   `auth::GRANTABLE`.
2. Extractors: `AuthUser` (anyone signed in), `ServerViewer`, `Manager`, and `JellyfinAdmin`, the only one allowed to edit
   permissions; `put_settings` refuses the access keys from anyone else, so a manager cannot self-promote.
3. **A manager is not an administrator either** (`Changer`, 2.3.1): `ADMIN_SETTINGS` (how many backups are kept, how often
   one is written, since `backup_every_d` is the backup's schedule while the job has no triggers and 0 stops it, and the
   merge window, at most an hour) and the schedules of `ADMIN_SCHEDULES` (the users read, which carries Jellyfin's
   disables to sessions and keys; the activity log; the backup) are an administrator's, judged by whether the value
   changes, so a form may send them unchanged.
4. `SECURITY_SETTINGS` (home addresses, the travel rules) take what reading the alerts takes; the settings name the
   household's addresses only with `see_network`; `jellyfin_url` and the data folder are an administrator's to be told; the
   Tasks section counts a manager's own plays and streams without `see_everyone`.
5. **`api::by_hand(user, what)` is the one gate for work started by hand**: a manager starts each at most once a minute
   however it is asked for (a job, the public-address switch and button, a new group window, which regroups the whole
   history, and the location database download, which also needs `security::gate`). An administrator is not held by it,
   so a manager cannot keep them from it.
6. Test notifications are counted per sender (`test_too_soon`).
7. The UI mirrors this with `can('perm')` from `state.js`; it is cosmetic, and every rule is enforced server-side.

## Audit (`audit.rs`)

1. The one record of FinStats' own write paths (`KINDS`; `Actor::from(&AuthUser)`). `record` is fire-and-forget and never
   the caller's error; `record_now` is for a row that must land before what it announces, such as an import's transaction.
2. A new write path gets a kind and one `record`, never a second log.
3. `audit` is in backups; a restored row says `detail.restored_from`.

## Public profiles (`public.rs`, `card.rs`, `/u/:token`, 2.0)

1. **The one read without an account, and it never goes through `AuthUser` or `Scope`.** A `Published` names one person and
   the sections they switched on; `answer` builds each of those from that id alone and returns typed structs, never
   `row_json`, so a column added to a shared query cannot reach a stranger.
2. Devices, clients, play methods, addresses, places, paths, other people and the login name are never produced.
3. **Every section counts only plays that ended a day ago or more** (`DELAY_S`; `recap::build` takes `until` for it): the
   first version delayed only the recent list, and the totals, grid and streak then moved the moment somebody pressed play.
4. The cut-off is the last local midnight at least `DELAY_S` ago (`counted_until`), so the page changes once a day: moving
   with every request, it told a stranger polling it each play's end to the minute.
5. An answer is kept per link **and language** (`Answers`), and **requests arriving together work it out once**
   (`Answers::get_or`, single-flight per token and language, the others waiting off the database); the fingerprint of the
   plays it counts is kept `PRINT_TTL_S` (5 s), so a change reaches the answer within seconds, not on the next request.
6. **Published cards (`card.png`, `recap/{chapter}`, `recap.zip`) are drawn in `i18n::DEFAULT` (en-US) whoever asks**, sent
   without `Vary: Accept-Language`, in a lane of their own (`public_lane`: its own cache, one permit, its own queue of at
   most `CARDS_WAITING`, else 503): drawn in the asker's language, in the owners' cache and for the same two permits, a link
   holder asking in turn in many languages made the owner's own recap cards answer 503.
7. `lookup` answers `None` for every reason there is nothing (switch off, unknown or reset token, unpublished, owner
   removed, disabled or without `sign_in` via `auth::effective`), and all of them are the same 404.
8. The image routes and the cards' posters serve only `servable_images` (`listed_images` that `artwork::known` also
   allows): an imported play's id is whatever the file said, and FinStats fetched it with the administrator's key for anyone.
9. The page preview fills its `{{holes}}` in one pass, so a display name holding `{{url}}` is shown as written.
10. Cards are SVG built from the `PublicProfile` alone and rasterised with resvg using Inter TTFs in `fonts/card/` (the web
    woff2 cannot be read; no system font, and `<image>` may only be the card's own `data:`), cached by link + kind + a hash
    of what they say, two drawn at once.
11. `charts.js` cannot run server-side, so the card's grid copies its colours; change them together. The server-rendered
    cards stay dark.
12. The page (`web/public.html` + `js/public/profile.js`) is outside the app: no state, api.js, router or prefetch, so
    nothing there can send a stranger to /login.
13. `public_profiles` is not in backups.

## Local and remote (`network.rs`)

1. `is_local` = a private range (`db::is_local_ip`) or a row in `home_addresses`: this network's own public IP, plus the
   manual `home_addresses` setting.
2. The public IP is looked up **once** from a plain-text service (the only non-Jellyfin request FinStats makes by default;
   setting `public_ip_lookup`, override `FINSTATS_PUBLIC_IP_URL`), at start-up when no `lookup` row exists, or when somebody
   presses the button ("Scheduling" 7).
3. Always go through `network::classify(conn, ip)`. After the set changes call `network::reclassify`, which re-decides the
   whole history and notes what it decided against (`network_classified`: version and home addresses); a start re-decides
   only when that differs (`reclassify_at_start`), never out of habit.
4. The lookup must stay anonymous (no version, no ids in the request), and the docs' privacy claims must stay true to it.
5. **For who Jellyfin keeps home, home is a private network or the looked-up public address only**
   (`home_for_who_is_kept_home`), never a `manual` home address: those decide how plays are counted, and a manager who may
   set them could otherwise open the house to any address they liked, which is Jellyfin's decision ("Permissions" 3).

## Security (`geo.rs`, `security.rs`, `worldmap.js`, `/security`)

1. `geo.rs` reads a MaxMind-format city database through a memory map (`Geo` in `AppState`, swapped whole when a newer file
   appears) of a **private copy** (`private_copy`: copied into `<data>/geoip`, opened, its name removed), never of the file
   itself: an owner who copies a newer file over the old one truncates what is mapped, and the next lookup is SIGBUS,
   which kills the process.
2. The file used is `FINSTATS_GEOIP_DB`, else the newest `.mmdb` in `<data>/geoip/`. Lookups never leave the machine.
3. The only network use is the opt-in download of DB-IP's monthly file (task `geoip`, no trigger by default; a trigger
   fires `download_if_stale`), which must stay as anonymous as the public-IP lookup; the docs' privacy claims must stay
   true to both.
4. Every distinct address gets one row in `ip_locations`, keyed by the spelling stored in `playbacks`/`server_events` (an
   all-NULL row = looked up, no place); a changed database empties the table and places everything again (`refresh_all`).
5. `server_events.remote_ip` is parsed from the log text by `event_ip` (the first word that is an address, because the
   label follows the server's language; `''` = none).
6. Plays from the home network are placed where the home's public address is (`home_place`).
7. Alerts come from the pure `detect()` over one person's whole history, so a rescan finds the same ones and `dedupe` keeps
   them single: travel is keyed by the later sighting, a pair of places reports once a day, a muted pair never, and
   anything found more than 30 days late is filed as resolved.
8. `scan` runs when a play begins (that user; one scan of a person at a time and once more for any asked meanwhile,
   `check_person`), after the log sync, at start-up and when the rules change.
9. **Failed sign-ins**: Jellyfin writes no user id on one, only a line ending in the name typed, so an account is a user
   whose name is **the whole of** what follows Jellyfin's own words (`SIGN_IN_TEMPLATES` in English, learned from what the
   lines share in another language; a name that merely ended the line filed "x alice" on alice). Names that are no account
   are whatever the one knocking typed and name nobody.
10. Failures are counted per account (and, for the names that are none, server-wide) and `BURST_WINDOW_S` (15 min) window,
    all of them: one warning, and one alert once the window holds twenty (`burst_dedupe`:
    `notify:signins:user:<name>:<window>`, plus `:alert`), so a long guessing session is at most two notices a window
    rather than thousands of warnings that could never be an alert (`scan_sign_ins`).
11. Every read of the activity log reads all that is new (`read_more_events`), from the newest row that is not in the
    future (one row dated years ahead, from a restore, stopped the log being read again); only the first read is capped,
    since the log answers newest first and a flood left unread hid the sign-in before it for good.
12. The location database download (`POST /api/security/database`) needs what reading the page needs (`security::gate`),
    waits the `geoip` job's minute for a manager (`by_hand`), and places plays again only when a file was loaded that was
    not before.
13. Reads need `see_network` **and** `see_everyone`, writes also `manage`; failed sign-ins additionally `see_server`.
14. The map (`worldmap.js`) is hand-drawn SVG over `web/assets/geo/world.json` (Natural Earth, pre-projected by
    `tools/make-world-map.py`; Mercator, because an equal-area projection leans the north and the owner read it as a tilted
    map; the formula there and in `worldmap.js` must match). No tiles, no map service.
15. The plain wheel scrolls the page; Ctrl+wheel, the buttons and the keys zoom.
16. **Drawing the coastline is what costs** (445 kB of path): a gesture only transforms the picture already drawn
    (`.wm-stage`, twice the frame each way so a drag uncovers land) and the viewBox is set once it settles (`draw`).
    Nothing is ever animated inside the SVG (the playing-now pulse is HTML above it), because that re-records the whole
    world every frame.
17. Hover is worked out from the dots' positions (`hover`), one tooltip for everything under the pointer, anchored on the
    top circle.

## Pipeline (`services.rs`, `arr.rs`, `seerr.rs`, `downloads.rs`, `pipeline.rs`, `/pipeline`)

1. Connections (Sonarr, Radarr, Seerr; several of a kind) live in `services`, secrets and all, and are `JellyfinAdmin`-only.
2. **Download clients are deliberately not connected**: Sonarr and Radarr already talk to them and report a torrent and a
   usenet download the same way, so FinStats reads their queues: three APIs to keep up with instead of six, and nothing
   for the owner to set up twice. What that gives up is ratio and peers; the speed is worked out from what moved between
   two readings (`downloads::speeds`).
3. Services get **their own HTTP clients that follow no redirect**: reqwest drops `Authorization`/`Cookie` across hosts
   but not `X-Api-Key`, and a 307 replays a POST body.
4. `Service` is neither `Serialize` nor `Debug`; its JSON is hand-built with `has_secret`; errors name a status and a kind
   of failure, never an upstream body.
5. Everything is `GET`, so nothing in the code could change anything there.
6. An id is never reused, and pointing a connection at another host/port/base path forgets its rows.
7. Certificates are verified unless the owner switches that off per connection.
8. Tasks: `sync_upcoming` + `sync_grabs` (15 min), `sync_requests` (5 min), all through `services::spawn`; `api::RUNNABLE`
   is the one way in, and a test holds it against `TASK_IDS`.
9. `item_external` turns `items.provider_ids` into indexed rows. **One id may belong to several items** (HD and 4K), so
   joins go id → every item → plays, unlike `relink.rs`, which refuses ambiguity because it rewrites history.
10. Film releases are stored as a *day* (Radarr's midnight UTC is the evening before west of Greenwich); episodes keep
    their moment.
11. Seerr is read newest-modified-first with an overlap on *its* clock, plus a re-read of everything still open (a media
    status change does not touch the request); that one runs every 15 min and only while something *is* open.
12. Only a whole listing may set `removed_at` (Seerr purges; the history must not shrink).
13. Every pass starts with a `take=1` probe (`newest_change`): a newest `updatedAt` no newer than the cursor means nothing
    was created or changed, and the pass ends there, which keeps a five-minute cadence from being most of the traffic
    FinStats makes.
14. Users link by `jellyfinUserId`, then Jellyfin user name, never a display name or e-mail.
15. `downloads.rs` is the only live part: its own loop and `Notify`, 5 s while a page says `?live=1`, otherwise 60 s while
    anything is in the queue (a request page shows how far along it is) and 5 min while it is empty (`wait_s`); an empty
    queue has nothing to go out of date. Opening the page, connecting a service or a read of Seerr all wake the loop. No DB
    work per tick; the snapshot is in memory only.
16. `fold()` turns queue records into downloads by `downloadId` + service: a season pack is one row, the same id in two
    instances is two rows, a record without an id stands for itself.
17. Scoping goes through `stats::pinned_user`: own requests for everyone; others' need `see_everyone` (and then no follower
    *counts* either, since on a small server a number is a name); the queue needs `see_downloads`; own-request progress
    (`state`, `progress`, `eta_s` and nothing else) is always allowed.
18. A queue row's `error` is the service's own words passed through `downloads::redact`: Sonarr and Radarr quote the client
    or indexer they could not reach, `user:pass@` and `?apikey=` included, and every indexer spells its key differently, so
    a query value or a path part shaped like a key (sixteen letters and digits or more, or a long hexadecimal one) is
    blanked whatever it is called.
19. **Why a service's queue could not be read is the administrator's, in the service's own words** (they name its internal
    address and, for a redirect, where it points); anybody else who may see the queue is told the service and a kind of
    failure (`problem_for`: `server.downloads.problem.{timeout,refused,failed}`).
20. **An answer is built no larger than FinStats reads of it** (`services::parse_within`, off the request threads): two bytes
    of `{}` are a value of their own, so a few kilobytes of gzip became millions of records. Each list keeps its first
    entries and skips the rest unbuilt, an answer of more values than its `Limits` is refused, and the byte caps are 16 MB
    (`JSON_MAX`), 8 MB for a queue page, a page of requests or of history.
21. A queue's time left and total speed saturate rather than overflow.
22. The poster proxy `/img/arr/{service}/{media}` serves only ids FinStats itself has listed.
23. None of these tables are in `backup::TABLES`: they are re-readable, and `services` holds secrets.
24. **A stored secret goes only to the place it was saved for** (`services::same_place`: scheme, host and port): an edit or
    a test naming another server and no secret is refused at the secret's field, for a Sonarr, Radarr or Seerr connection
    as for a notification destination.

## Outbound connections (`outbound.rs`, `GET /api/outbound`, Settings card)

1. One row per destination FinStats can reach (Jellyfin, the public-IP services, DB-IP, each `services` row, each
   `notify_targets` row), with whether it is on and when it last answered.
2. Built entirely from what is already kept (collector status, `home_addresses`, the `.mmdb` on disk, `service_health`, a
   destination's `last_ok_at`): nothing is recorded for it, and hosts are shown without paths or keys.
3. A new outbound destination must appear here, and in the promise sentences in `README.md` and the docs'
   `finstats/privacy.md` and `finstats/security.md`, in the same change that adds it.

## Notifications (`notify.rs`, `channels.rs`, `mail.rs`, `/api/notifications*`, Settings card)

1. The only thing FinStats *sends*. An **event** is raised where the thing is noticed and written once (`raise_in` is
   `INSERT OR IGNORE` on `dedupe`, exactly like `security_alerts`, so re-deriving the same thing announces nothing twice).
2. **Delivery is a separate row per destination** with its own attempts and clock (`backoff`: 30 s → 2 min → 10 min → 1 h,
   then given up on; `Retry-After` honoured; `PER_MINUTE` per destination), so a webhook that is down delays nothing else.
3. **Destinations are sent to side by side** (`SENDING_AT_ONCE`, four), each one's in its own order, and a message is given
   `SEND_TIMEOUT` (5 s; a letter 30 s, each exchange 10 s). A timeout leaves that destination's other messages for the next
   batch, and `TIMEOUTS_TO_PAUSE` (3) in a row leave it alone for `PAUSE_S` (ten minutes). Sent one at a time with reqwest's
   fifteen seconds and five attempts, one that took the connection and never answered held up everybody's.
4. `notify::run` is its own loop, woken by `notify_wake` and otherwise asleep until the next retry; an install with no
   destination never wakes. Deliveries due go out the most severe first (`due`).
5. **Two guards make adding a destination safe**, both needed: anything found more than `HISTORIC_S` (6 h) after it
   happened is stored `historic = 1` and never queued, and `wanted_by` refuses anything that happened before that
   destination's `created_at`.
6. History is thinned at `KEEP_S` (30 days), which is safe only because every source either re-derives a short window
   (`recent::announce`, `seerr::announce_available`, `security::scan_sign_ins`) or raises once, when the thing itself is
   first written (`security::file_alerts`).
7. **What may be said is one pure function**: `message(event, with_addresses, public_url)`. An event carries two bags:
   `data`, which any destination may be told, and `private` (addresses, coordinates), which `message` reads *only* with the
   per-destination switch, so the rule holds by construction, and a test asserts no address appears in any kind of message
   without it. The link is `public_url` + the event's path; empty setting, no link.
8. **`wanted_by` is the whole permission rule in one pure place.** A server destination (`owner_id IS NULL`) is not
   filtered; a personal one is checked against its owner's `Perms` (`auth::effective`): own rows always; somebody else's
   play or request needs `see_everyone`; somebody else's *places* need `see_network` too (`security::gate`'s rule);
   somebody else's failed sign-ins `see_server` as well (the Security page's); a job's or a connection's failure needs
   `see_server` and `manage` (it names a person, or a service's internal address, which the pages show only to who may
   manage FinStats).
9. The owner is read live (disabled or removed in Jellyfin = told nothing) and **asked again at send time**
   (`deliverable`): a delivery can wait an hour between tries, and "include addresses" is the owner's wish, which on a
   personal destination also takes `see_network`.
10. Managing a personal destination needs the grantable `notify`, and its host must not resolve into a private range nor to
    the household's own public address (`must_be_public`, checked on save **and** before every send, because a public name
    can be re-pointed later; `services::remember_home` keeps the set in memory, read again wherever `home_addresses` is
    written, since from inside, the public address loops back to the router and whatever it forwards). A name that cannot
    be looked up is refused in the same words as an address inside (`server.notify.not_public_or_unknown`), so a refusal
    does not say which internal names exist, and an IPv6 address is checked as the address it is. An administrator's may
    point anywhere.
11. Discord and Slack addresses must be `https` (the address is the token), and a destination's name may hold no control
    character, since it is written to the log.
12. **Somebody else's personal destination is theirs**: an administrator may switch it off, choose what it is told or remove
    it, never change its address, secret, field, options, certificate checks or addresses switch (`describe`), nor read its
    ntfy topic or Pushover user key (`shown` names only the start of a topic to anybody but its owner).
13. A destination is made or changed only by a session, never a key (`keys::only_a_session`), since it outlives the key; and
    a test is sent no more often a minute than anything else, counted per sender (`test_too_soon`), so removing a
    destination and adding it again starts no fresh minute.
14. **A destination's address is a credential** (a Discord webhook URL carries its token), so `Target` is neither
    `Serialize` nor `Debug`, the API answers `shown` (host, plus the topic, chat or mailbox that says which one it is) and
    never the URL, editing without a `url` keeps the stored one, and `notify_targets` is not in `backup::TABLES`. A stored
    secret goes only where it was saved for ("Pipeline" 24).
15. `channels.rs` holds every payload shape and the POST; it reuses `services::Http` (no redirect followed while holding a
    token) and publishes to ntfy and Gotify in their JSON form, never through headers, because a title is a film title, the
    same reason Telegram is sent with no `parse_mode`.
16. Part of what a message says is a stranger's (a device's name, a name tried at a sign-in), so Slack is sent it escaped
    (`slack_text`), Discord as escaped Markdown with `allowed_mentions` empty (`markdown_text`). **Escaped first and cut
    after**, to each platform's limits without splitting an escape (the fields of one embed share a budget; the description
    takes what is left of Discord's 6,000): cut first, a name full of Markdown doubled when escaped and Discord refused
    every alert naming it.
17. A new title (`NewItems`) goes to a personal destination only when its owner may see it (`owner_sees_title`,
    `access.rs`), and a watchlist arrival is never told of a title hidden from its owner.
18. **Nine kinds of destination, one model under them**: an address, a token, one field beside them, and `options` for
    what is left (migration 19; only mail has any). A channel may fix its own address (`Channel::fixed_url`: Telegram,
    Pushover, Pushbullet are reached at their own service and nowhere else, so a token can never be posted to a look-alike
    host) and names that one field itself (`topic_label`: an ntfy topic, a chat id, a Pushover user key, a mailbox), each
    checked the way its own service writes it.
19. **Email is the one that is not a POST of JSON**, so it is its own module (`mail.rs`, `lettre`): `smtps://` is encrypted
    from the first byte, `smtp://` must upgrade with STARTTLS, and there is no third option, so no path by which a password
    is sent in the clear. `payload()` answering `None` says "not an HTTP request at all"; `Channel::is_mail` is the same
    fact where it is easier to read.
20. Anything new here is a `Channel` arm, a payload and where its token goes: `notify.rs` should not have to change for one.
21. Producers live where the thing is noticed: `security.rs` (alerts, and `bursts` of failed sign-ins), `sync.rs` (a failed
    job, a failed backup, what arrived), `services.rs` and `collector.rs` (a connection that stopped answering, Jellyfin
    included, and plays beginning and ending), `seerr.rs` (a request that became watchable). Adding a kind of event means a
    `Kind` arm and one `raise`, never a second way out.
22. **A mail server is read no further than any reply could be**: `mail::deliver` opens the connection itself and hands
    lettre a stream (`Capped`) that ends after 1 MiB over the whole letter, certificates included, and lets no read wait
    longer than 10 s; for `smtps://` the TLS goes inside it (`Encrypted`). lettre grows a reply until it sees a line's end,
    so a server that never sent one grew FinStats' memory as fast as it could send. A Test goes under the same
    `MAIL_TIMEOUT` (or `SEND_TIMEOUT`) as the queue's own sending.

## Backups (`backup.rs`)

1. gzip JSON Lines, one row per line tagged with its table, matched *by column name* both ways so files move between
   versions. A new table that holds something Jellyfin cannot give back must be added to `backup::TABLES`.
2. Secrets (Jellyfin URL/API key, sessions, API keys) and the library are never exported; a test asserts the key is absent.
3. Restore merges (dedupe on `source_id` or user + item + start), remaps timeline rows to the new play ids, and re-derives
   groups, `is_local` and library links.
4. **A restore reads the whole file before it takes the write lock** (`each_line`, without a lock: a file wrong further down
   writes nothing, and one that unpacks to far more than its size is refused, `server.backup.too_large`: a hundred times, at
   least 64 MB, never past 64 GB, blank lines counted), **then writes it a batch at a time** (`BATCH_ROWS` / `BATCH_BYTES`,
   each its own transaction), so the collector writes between two batches: in one transaction a few MB of gzipped blank
   lines held the write lock for most of a minute and the collector lost plays. A restore stopped between batches is a
   partial merge, and restoring again completes it.
5. Settings and grants come back only when asked for (`settings=true` on the page, `finstats restore <file>
   --with-settings` on the command line, which is history only by default like the page), and **never open FinStats wider
   than it is**: `narrow_settings` makes who may sign in, `default_permissions` and public profiles what both here and the
   backup say, and keeps this install's `public_url` and `jellyfin_public_url` unless it has none; `narrow_grants` narrows a
   person's grants to what both hold, and somebody without a row here gets only those of the backup's that
   `default_permissions` already gives everyone, names filtered by `auth::GRANTABLE`. Last month's backup gave back every
   grant taken away since, and an older or crafted one let everybody in with every permission.
6. **A restore of the history alone changes nothing about what watches over it**: home addresses come back only with the
   settings (only those typed in), an alert comes back unmuted, an audit row says `detail.restored_from`, a located title
   only where the library has it, and a play only within an import's bounds (1990 to tomorrow, ending after it starts, at
   most `EVENTS_PER_PLAY` events, its numbers clamped by `bound_play`); server log rows and devices from outside those
   years are left out too.
7. **A restored play counts as imported whatever source it claims** ("Play rows" 5), everywhere it matters
   (`artwork::known`, the duplicate sweep, the collector's resume), or a crafted backup opened the picture proxy for any id,
   outlived the sweep and became the start of somebody's real evening.
8. The `backup` task's triggers write one (skipped while there is no play); endpoints are `JellyfinAdmin`-only and names go
   through `valid_name`.
9. **Names are in UTC** (in local time the hour the clocks go back named two backups the same), and a file is written as
   `<name>.part` and put in place without replacing anything (`put_in_place`). A `.part` nobody has written to for 15
   minutes (`STALE_PART`) was a killed backup and is removed; a fresher one may be `finstats backup` running beside the server.
10. **A backup a manager runs by hand is pruned in its own pool** (settings key `backups_requested`, not exported), each
    pool keeping `backup_keep`: counted together, a few runs a minute apart pushed out every restore point the administrator had.

## Trash (`trash.rs`, migration 37, 2.2.0)

1. **Nothing a person deletes goes at once.** A play is deleted by `PUT /api/activity/{id}` `{"deleted": true}`
   (`manage`), which sets `playbacks.deleted_at`, clears its `group_id` and regroups the title (`groups::detect`, again on
   the way back). A backup by `PUT /api/backups/{name}`, which moves the file to `<data>/backups/deleted/<name>.<unix>` (out
   of `list`, `newest`, `prune`'s count, the download and the restore) and back only when nothing has taken its name
   (`409`). `DELETE` is gone from both.
2. After `trash::KEEP_S` (30 days) `trash::purge` removes them for good on the 15-minute housekeeping beat, one look at the
   partial index `idx_playbacks_trash` (a test pins the plan), touching only file names `backup::in_trash` recognises;
   `trash_purged` is audited by counts alone. `prune` stays a hard delete: what `backup_keep` lets go was never chosen by
   anyone.
3. **Every reading of history reads the view `visible_playbacks`** (`SELECT * … WHERE deleted_at IS NULL`, flattened by the
   planner onto the table's indexes), the trash listing `trashed_playbacks`, and **the table itself only for identity**
   (`already_recorded`, `source_id`, relink and locate, the collector's deletes, the address reads): a play in the trash is
   still that play, so an import or a restore does not bring it back as a new one, and relinking moves it with its title.
4. `db::tests::every_read_of_history_leaves_the_trash_out` scans every string literal in `src/` and fails on a read of
   `playbacks` that names neither `deleted_at` nor an entry of `IDENTITY_READS` (each with its reason; a stale entry fails
   too): a new query over the table is a decision, not a habit.
5. The duplicate sweep never keeps a deleted row over a live one ("Relink" 9), the collector never resumes a play in the
   trash, and a restore carries `deleted_at` but keeps the state of a row already here: it never undeletes. A restored
   deletion is no later than the restore, and the purge day is added without wrapping.

## Languages (`i18n.rs`, `web/assets/js/i18n.js`, `web/assets/i18n/`, 2.3.0)

1. **FinStats speaks 23 languages, and every word a person reads comes from a catalogue**: one JSON file per BCP 47 tag
   (`i18n::LOCALES` and `LANGUAGES` in `i18n.js` are the same list, and `theme.js` repeats the tags). **British English is
   the source.**
2. **Four are checked by the owner, who reads them (`i18n::CHECKED`: en-GB, en-US, nb-NO, nn-NO), and are whole in every
   change.** The other nineteen are translations nobody here can check: they may lag behind and say what they lack in
   British English (the page loads en-GB beside them, `i18n::say` falls back the same way); the README's picture shows how
   far each has come, and the Settings list and the README say they are unchecked.
3. **Synnmørsk (`nn-NO-synnmore`)**, the owner's dialect, was a language of its own in 2.3.0 and was removed in 2.3.1
   (owner's decision): a browser that chose it, or a cookie or header that names it, reads Nynorsk (`i18n::pick`,
   `languageOf` in `i18n.js` and `theme.js` alike).
4. Every catalogue answers to the source (`i18n::tests`): no key the source lacks, the same `{placeholders}`, plural objects
   of exactly the CLDR forms the language has (`one`/`few`/`many`/`other` for Russian, Ukrainian and Polish, `other` alone
   for Japanese, Korean and Chinese; `i18n::plural` is the server's copy of the rules), no em-dash, each English spelled its
   own way.
5. A key says where it is used (`pages.watchlist.empty_title`, `settings.sections.backups.sub`, `server.not_found.play`); a
   sentence is one key, never glued from translated pieces, because Norwegian orders words differently.
6. **The browser side**: `t(key, vars)` (numbers in `vars` are written the language's way), `tParts` when a placeholder is
   an element, `tOr(key, fallback)` for an id from the server the catalogue may not know, `lang()` for every `Intl` call and
   `localeCompare` (never `undefined`, `'en'` or `'en-US'`).
7. **The language is the browser's** (Settings → Appearance → Language, `finstats.language`; nothing stored follows
   `navigator.languages`). A browser asking only for languages FinStats does not speak, or for an `en` that names no
   country, gets **American English** (`i18n::DEFAULT`, `DEFAULT` in `i18n.js`, `theme.js`), while British English stays
   the source, what a catalogue lacks is said in, and what work outside a request (a test, a scheduled run) speaks.
   `theme.js` decides the same way to preload the file and set `<html lang>`.
8. `i18n.js` keeps the language in the `finstats_lang` cookie, which `i18n::asked` prefers to `Accept-Language`, because a
   picture or a download the browser fetches by itself carries the browser's own languages. The cookie is every tab's, so
   a page's own requests say `X-FinStats-Language` and its socket `?lang=`, which beat it. Choosing another reloads.
9. `main.js` puts the catalogue in use **before** importing either layout, so a module may say `t()` at its top level; the
   public profile does the same before its own imports.
10. FinUI's components speak through `finui/i18n.js` (`say(key, english)`, `locale()`), fed the catalogue's `finui` section.
11. Numbers follow the language everywhere (FinUI's `num`, `dom.js`'s formatters), and FinUI's table reader parses them
    back with that language's separators, its units and "5 of 26", which `useCatalogue` hands FinUI as `units` (the
    catalogue's `units` and `pages.person.of_total`). Tables read numbers as `num()` prints them, never with the browser's
    own separators.
12. A word beside a number goes through a plural key, never `n === 1 ? a : b`; one that is the word alone beside a number
    the page draws (`…_word`, `unit…`) may leave `{count}` out.
13. **The server side**: `api.js` sends the page's language as `Accept-Language` and `X-FinStats-Language`;
    `i18n::middleware` keeps it for the request (`i18n::current`, a task-local; `i18n::pick` is `pickLanguage`'s rule);
    `i18n::t` says a catalogue text in it (the same files, `include_str!`'d); `i18n::carry` hands it to work spawned from a
    request, and `i18n::within` to a blocking thread (`Db::call` and `api.rs`'s `blocking` do, so `db.call` and imports keep it).
14. Errors and everything else the server answers in words go through it (`server.*` keys); `ApiError::not_found("Thing")`
    finds `server.not_found.thing` by itself. Tests run outside a request and so in en-GB, which is why en-GB keeps the
    code's English exactly.
15. Ids, kinds and states in answers never change with the language: the pages translate them by id
    (``t(`pages.audit.kind.${k}`)``).
16. Two guards in `i18n::pages_tests` hold the pages to it: every literal key a page asks for has an en-GB text, and no
    English sentence or capitalised label is left in `web/assets/js` or FinUI's components outside
    `t()`/`tParts()`/`say()` (`IDS` and `ENGLISH_ON_PURPOSE` list the exceptions, each with its reason).
17. **A new text is a key in the four checked catalogues in the same change** (the others may follow, and say it in English
    until they do), written in Bokmål and Nynorsk by the glossary the existing catalogues set (play = avspilling/avspeling,
    watch time = visningstid/visingstid, watchlist = Min liste / Lista mi, Recap = Tilbakeblikk, Pipeline = På vei/På veg);
    the owner reads Norwegian and checks it.
18. The README's **Languages** section is a picture drawn like a translation platform's status widget
    (`.github/languages.svg`, a bar per language, in GitHub's light or dark), worked out from the catalogues and the
    changelogs by `tools/make-languages.py`, which redraws both; run it whenever a language changes, and the QA static stage
    runs it with `--check`, which fails while either is out of date. It is the one README picture kept here rather than in
    the docs, because the tool draws it from this repository.
19. The local QA stage `i18n` runs `i18n.js`'s rules in node; the browser stage pins its pages to en-GB (it overrides
    `navigator.languages`, since headless Chromium says en-US whatever `--lang` says) and sweeps every page in Bokmål and
    Nynorsk for a missing key.

## Changelog (`changelog.rs`, `CHANGELOG.md`, `changelog/<tag>.md`)

1. `CHANGELOG.md` is the source of the in-app **Patch notes** tab (`changelog.rs` parses it); `changelog/<tag>.md`
   (embedded) is the same file in another language, answered by `Accept-Language`.
2. **A note lands in the four checked languages in the same commit**; another language may tell a release later in the
   series, and shows the source's until it does, but **not later than the release itself**: the QA stage `i18n` fails while
   any `changelog/<tag>.md` lacks a release `CHANGELOG.md` has.
3. A test holds every translation to the source release by release (the same headings byte for byte, groups, number of
   bullets, `code` and bold).
4. A test fails if the top entry's version differs from `Cargo.toml`, or an entry lacks a date/notes/known group, so a
   version bump and its changelog entry land together. The rules live in one test function (`changelog::tests::problems`).
5. Format: `## [x.y.z] - YYYY-MM-DD`, or `- YYYY-MM-DD to YYYY-MM-DD` for a release made over several days (`date` stays
   the release day, `started` the first; the page shows both), then `### Added | Changed | Security | Performance |
   Stability | Fixed | Removed` with one-line bullets (`**bold**`, `*italic*` and `` `code` `` are rendered; a Markdown link
   is not, so name a doc as `code`).
6. **Security** is something somebody could have misused (its own group since 2.3.1, never a fix that begins
   "Security:"); **Performance** is quicker or lighter; **Stability** is something that can no longer crash, halt, leak or
   lose data; **Fixed** is something that gave a wrong answer. A speed-up filed as a fix reads as a bug nobody had.
7. An `x.y.0` entry opens with a **short title line** and nothing else ("Notifications", "WebSocket session tracking"),
   which the app shows as that whole series' headline (`changelog.js` takes the summary's first sentence, or all of it when
   there is no full stop, so a title stays whole); a test enforces that an `x.y.0` has one.
8. Notes are terse and factual, in the shape of a GitHub changelog: what changed, and the fact that makes it make sense.
   Nothing a release did not do.

## Front-end (`web/`)

1. No build step, no dependencies, no CDN: vanilla ES modules served from the binary, fonts bundled. `dom.js` (`h()`
   element builder, icon map, formatters), `api.js`, `router.js` (History API; the server returns `index.html` for any
   non-`/api`, non-`/assets` path), `components.js`, `charts.js` (hand-rolled SVG), `pages/*.js`. The only JS check is
   `node --check`.
2. Every word a person reads comes from `t()` ("Languages").
3. API and user strings reach the DOM only via `h()`/`textContent`, never `innerHTML` with data.
4. Fetches are aborted, timers cleared and listeners on `window` or `document` removed on route change (`ctx.signal`): a
   listener left behind holds its whole page, and `stickyFilters` refuses to be drawn without one.
5. Every new card hides itself when its data is missing (older servers, non-admins, imported plays).
6. Native `el.append(null)` prints the text "null", so pass possibly-absent nodes through `h()` or filter them first.
7. **A route parameter never reaches another endpoint than its page's own**: `router.js` refuses a parameter that decodes
   to a separator, `.` or `..`, or the start of a query or fragment, and every loader that puts one into an API path encodes
   it (a link to `/items/..%2Fbackups` made the browser ask `/api/backups`).
8. **FinUI's `h()` and `s()` set an address attribute (`href`, `src` and the like) only when it is http(s) or relative**,
   read as the browser reads it, and never `srcdoc`, so a `javascript:` value from an API is not held back by the CSP
   alone. The change lives in `finui/core.js` and is in `../finui` too.
9. Each screen is held to the UX patterns from <https://uxgoodpatterns.com>. A generated copy, `ux-rules.md`, may sit in the
   working tree for reference; it is someone else's work, is git-ignored and must never be committed.
10. In the status bar every item is one small box (18 px, `--radius-sm`) and every link hovers as that box; the `·`
    between two items is an element of its own (`.sb-sep`), laid out between them so flexbox centres it, never an item's
    `::before` (inside a link its hover lit the dot up, and placed by hand it was never quite centred). It hides with the
    item after it (`:has`).
11. A long value in a `facts()` grid (a file path) takes `{ wide: true }` and the whole row, or it runs down a 190 px cell.
12. Signing out lives in Settings → Account (`settings/account.js`), not in the sidebar.

## Tables (`tables.js`)

1. Tables go through `tables.js`: `dataTable()` / `plainTable()` / `chartTable()` wrap a built `<table>` in its scroll box
   and make every header a sort button (`sortable()` for tables without a scroll box, like the bar lists). A new table
   that skips this helper is a bug.
2. Sorting reads cell meaning (durations, sizes, %, `<time>`, switches); `data-sort` overrides, `data-nosort` opts a header
   out, `data-pin` keeps a row on top.
3. Paginated lists must not be sorted in the browser: they pass `server: {key, dir, onSort}` with `data-key` headers, and
   the endpoint whitelists `sort` via `stats::order_by`.

## Data loading and prefetch (`components.js`, `prefetch.js`)

1. Pages load through `dataView()`, which is stale-while-revalidate: it records the GET URLs a page's `fetch()` issues
   (they must be fired synchronously when `fetch()` is called), uses them as the cache key, paints a remembered result at
   once, refreshes behind it and only re-renders when the JSON differs.
2. Skeletons appear only after 150 ms and then stay at least 300 ms.
3. The cache is in-memory per tab and is cleared by `resetCaches()` (sign-out) and by any non-GET request.
4. **Prefetch** fills that same cache before a page is asked for: a warm-up about a second after the first page
   (top-level pages, then each library and the 12 most active people, one at a time while the browser is idle; at most
   every 2 min; not on `saveData`/2g), and on intent (pointer resting 90 ms on a link, focus, touch). Titles and people are
   fetched on intent only, never in bulk.
5. It only works because page and prefetcher call the *same loader*: every page module exports `prefetchX(ctx) → [() =>
   load…]` next to the loader its `dataView` uses, and `main.js` passes it as `route(…, { prefetch })`. A page whose
   `fetch` is written inline cannot be prefetched, and a loader copied into `prefetch.js` would drift from the page's URLs
   and silently miss.
6. Clearing the cache bumps a generation and aborts what is on its way, so an answer from before a sign-out or a write is
   never stored. Pages join a prefetch already in flight (`shareRequestsOf`); only the prefetcher's requests can be
   joined, because a page's signal dies with the page.

## Settings and Server pages (`pages/settings.js`, `settings/*.js`, `pages/server.js`, `sections.js`)

1. **One section on screen at a time**, on the shell in `sections.js` (`pickSection` forwards bare and legacy addresses,
   `sectionNav` draws the sticky list, `reveal` marks a row a link pointed at): `/settings/:section` and
   `/server/:section`, a sticky list on the left (a row of chips on a phone), and on Settings a finder in the header (`/`
   focuses it) built from each section's `entries`.
2. A section is a module exporting `{ key, label, sub, group, icon, visible, entries, render(slot, store) }`;
   `settings/common.js` holds the store (`/settings` and `/tasks`, fetched once per visit, tasks polled only if a section
   subscribes) and the two shapes a setting takes: `toggleRow` (saves itself) and `numberForm` (one `Save` per section,
   bottom right).
3. A setting is one row (label, one line of help of at most ~150 characters, control on the right), and a QA check holds
   that budget: the page it replaced was fourteen cards and six hundred words of help in one column.
4. Bare `/settings` and the old anchors (`/settings#backups`) forward to the section that holds them (`LEGACY`); a new
   section's card id stays a valid anchor.
5. Sections hide themselves (`visible`) rather than explaining why they are empty.
6. **Tasks (`settings/tasks.js`) is the one section with pages below it**: `/settings/tasks` lists every job Jellyfin-style
   (a row per job, its last run in words, a round Run button; the whole row is the link) and `/settings/tasks/:task` is one
   job's schedule with Jellyfin's *Add trigger* dialog. Esc goes back from a schedule to the list (`escTarget` in
   `shell.js`), and hovering the Run button lights the button, never the row (the row reads as "open").
7. Tasks repaints from what the rows *say* (`said`), never from the raw answer: a job that is due has a `next_at` of "now",
   which moves every second.
8. Backups, Security and Collection link to their job's schedule; none of them holds a timing control of its own any more.
9. The licences page is reached from a button in Settings; the route itself is open to anyone signed in. Settings → System
   links to FinUI's site.

## Navigation (`shell.js`, `search.js`, `menus.js`, FinUI `desktop-nav`, `mobile-nav`, `search`, `context-menu`)

1. **Esc is handled globally in `shell.js`** (steps back out of `/libraries/:id`, `/users/:id`, `/items/:id`), going back
   through history only onto an entry of FinStats' own: every entry the router writes carries `history.state.depth`, and a
   tab opened on a title has none behind it. Overlays must keep calling `stopPropagation()` on their own Esc, and
   `openModal` keeps a stack: only the dialog on top answers keys.
2. **The menus are FinUI's `desktop-nav` and `mobile-nav`, each in the style its browser chose** (Settings → Appearance;
   owner's decision: kept by the browser, `state.js`, never the account).
3. Above 820 px, one of eight: Sidebar (today's, the default), Grouped, Rail that opens, Search first, Dock, Command bar,
   Pinned, Coloured tiles. The Rail shows icons until the pointer or the keyboard's focus comes (`:focus-visible`, never a
   link a click left focused), then the whole sidebar *over* the page (`STYLES[].opens`); closed, its list scrolls without
   drawing a bar.
4. The six that are a side may sit on the right (*Menu on the right*; `sideOf` answers none for the dock and the command
   bar). Each style says which edge of the window it keeps (`STYLES[].edge` and `size`); `shell.js` lays the page beside it
   (`.app.nav-left|right|dock|command`, `--desktop-nav-size`).
5. The dock sends the status bar to the top, so the version and the repository link stay on screen; it tucks itself away
   while the page scrolls down and comes back scrolling up, at the top, with the pointer near the bottom edge or the focus
   in it (`dockShown`).
6. Pinned's pins are the browser's too (`finstats.desktopNavPins`; nothing stored is the primary pages).
7. Below 820 px, one of five (Tab bar, the default, Peek, Full screen, Thumb arc, Address bar), offered only at that width;
   the three that keep the bottom of the screen take the status bar's place (`.app.has-mobile-dock`,
   `--mobile-nav-space`).
8. Pages carry `group` and `primary` in `navItems`, which both menus read. The rules that are not drawing (styles, sides,
   groups, pins, the arc's rings, the dock's hiding, the search's flight) are each component's `plan.js`, tested in FinUI's
   repository.
9. A menu takes Esc in the capture phase, before `shell.js`' own Esc steps back a page. The hand-built sidebar and the
   phone's slide-in drawer are gone.
10. **Search is never a dialog.** It lives in the menu or in the page, as this browser chose (Settings → Appearance →
    *Search opens*; `finstats.searchMode`, nothing stored is the menu). In the menu, each style holds it in its own place
    (`openSearch(panel)` on `desktop-nav` and `mobile-nav`: a sidebar widens over the page, the rail opens all the way, the
    dock rises into a card with the field where the dock was, the command bar's name turns into the field, a phone's sheet,
    screen or arc card holds it, and the address bar keeps the field under the thumb). In the page, it takes `.content`'s
    place in `.page-search` while the menu stays.
11. Either way `grow()` plays it out of the control that opened it (the menu's search, the command bar, the top bar's search
    on a phone, or for Ctrl+Space the menu's own) and `shrink()` back into it, a place that was not there opening out of
    the control (`cut`) rather than sliding in. `search.js` answers the words (pages at once, `/api/search` when it comes);
    a press anywhere else, Esc or a page chosen puts it away. The modal palette it replaced is gone.
12. **Esc gives the keyboard's focus back to what had it when search opened** (the control pressed, or the place Ctrl+Space
    was pressed in, never the menu's search for a shortcut) and lets a click's focus go (handed back after a key it would be
    the keyboard's, and a link a click left focused in the rail would open it). A rail holds itself open while the focus is
    in it, and `desktop-nav` moves the focus before `shrink` measures where it closes to (and again once it has, the search
    laid back over the menu having hidden it).
13. `grow` and `shrink` both hold the host's own transition (nested `hold`) *before* they measure: a rail measured
    mid-transition is still as wide as the search, and the close shrank the shape into itself, an empty panel until it
    vanished.
14. **Context menus (2.2.0) come from the link, not the page.** One resolver for the whole app (`installMenus`,
    `attachContextMenu(document, …)`): any `<a>` to `/items/:id`, `/users/:id`, `/people/:id` or `/libraries/:id` has a
    menu: right-click, a long-press on a touch screen (`plan.js` `held`: 500 ms, 10 px), the menu key or Shift+F10. Shift
    with a right-click, a field and anything else are left to the browser.
15. What a title is comes from the link's `data-type` (`typed()`, a Jellyfin type), set where the answer that drew it knows:
    only a film or a show is offered the watchlist (`watchMenuItem`, the toggle's own add and remove), and a link without a
    type is offered everything else.
16. A card with no link (a film in Coming up the library does not have yet) names its title in `data-` attributes instead
    (`notHere`, the watchlist's own `{kind, tmdb_id, tvdb_id, title, year}`) and is offered the watchlist by its ids, its
    page on TMDB or TVDB (the forms `stats.rs` links) and Pipeline.
17. "Open in Jellyfin" is `user.jellyfin_details` plus the id (`/auth/me`; `jellyfin::web_link`'s rule).
18. The menu grows out of the pressed point (`place` answers its corner and `transform-origin`), the first item takes the
    focus, Esc and Tab hand it back, and Esc never reaches `shell.js`.
19. **A scroll carries the menu with its element, and only its leaving the screen closes it**: closing at any scroll shut a
    menu opened from the keyboard in the frame it opened, as focusing a poster far along a shelf scrolled it into view.

## Theming

1. The look is Obsidian's in the dark and washi paper in the light (cream ground, ink text, a vermilion seal for the
   accent, nando for quantities), via FinUI's tokens (`web/assets/finui/tokens.css`).
2. Categorical chart colours follow the entity (Movie/Episode/Audio/Other), never rank.
3. **Every colour is a token, and every token is `light-dark(light, dark)`**: one declaration, two themes. A colour literal
   anywhere else (a CSS rule, a chart's `fill`, a `style.background` in JS) is right in one theme and wrong in the other;
   the QA light sweep measures text contrast on every page and fails on it (`finui::tests::no_colour_literal_outside_the_tokens`
   holds every stylesheet). JS reaches a token as the string `'var(--x)'` through `style`, never an SVG presentation attribute.
4. With nothing chosen the device decides (`color-scheme: light dark`). The sidebar's theme switch (`themeSwitch` in
   `shell.js`: three stops Light · Device · Dark, a `role="slider"`; click, drag or keys; its knob's icon animates in, never
   with reduced motion) keeps the choice in `localStorage` (`finstats.theme`, absent = Device), which `theme.js` (a classic,
   blocking script in both pages' `<head>`) puts on `<html data-theme>` before the first paint.

## FinUI (`web/assets/finui/`, `finui.rs`, `appearance.rs`)

1. **FinStats' own component library**, written "FinUI" wherever a person reads it (lowercase only in paths, the registry's
   name and the `fui-` prefix).
2. Each component is a module with its `meta` and its CSS, listed in `registry.json` with the tokens its CSS reads (a test
   holds the list); a `fui-x` class is styled by component x alone, and its modules import nothing from outside `finui/`.
3. `/assets/finui.css` is every CSS file in registry order, built by the server: one request, no build step.
4. **Every value a theme may change is a token**: colours, but also every corner (`--radius*`; a test refuses a literal
   radius), `--density`, `--card-shadow`, `--icon-stroke`, `--on-accent`.
5. **FinUI is also a repository of its own, `github.com/finstats/finui`** (checked out as `../finui`): the components are
   developed here, where they are used, and copied there as they change. Its gallery, **FinUI create** (`create/`: presets,
   `preset.js`, the page) and the installer (`curl -fsSL https://finui.finstats.no/install.sh | sh -s -- <code>`, built
   with the site by its pages workflow) live there only; FinStats serves no gallery.
6. **Each person's FinStats wears the preset they chose** (Settings → Appearance, `appearance.rs`, table `appearance`,
   `/api/me/appearance`, in `backup::TABLES`). FinStats keeps FinUI's `create/presets.json` and the per-option token files
   FinUI's site build generates (`finui/p/<axis>/<option>.css`), and `/assets/finui.css` answers in the look of the session
   cookie that asks (`appearance::of_request`; `private`, `Vary: Cookie`): the stylesheet followed by the files the code
   names in axis order, exactly what FinUI's `install.sh` does, so FinStats has no copy of how a choice becomes tokens.
   `finui::preset::overlay` only decodes the code against the files; an unknown code is refused at save.
7. There is no look for everyone: one person's choice never reaches another (owner's decision).
8. **A style is a whole look in one choice** (`presets.json` `styles`: a key, a label, a line and its picks by option key;
   FinUI create's first picker). FinStats' Appearance offers none (owner's decision: FinStats ships one look, and a style
   reaches it only as the code FinUI create makes), and it is nothing but a code: Washi, every default, is saved as no
   preset at all.
9. FinStats' own chrome reads FinUI's tokens where a choice should reach it: the menu's open page is `--selected-*` (the
   Menu axis), as FinUI's section list is.
10. **Presets choose fonts too** (axes Font, Heading and Mono; Heading is the token `--font-heading`, the text's own by
    default): every family a preset may name is bundled in `web/assets/fonts` beside its OFL licence, which `licenses.rs`
    lists (a test holds every licence there to the notice), and a font option's file carries its `@font-face` rules, so a
    page fetches only the fonts it was told to.
11. **Icons move once on hover** (FinUI's `animated-icon`): `main.js` calls `animateWithin(document.body)`, which animates
    every `icon()` inside something pressable as it is drawn (an icon among words stays still), so no page asks for it. A
    busy button (`setBusy` in `components.js`) whose icon is `refresh` keeps that icon turning instead of showing a spinner
    and lets it finish its turn when done; it is the only icon that moves for longer than one hover.
12. To bring new options over: build FinUI's site (`node ../finui/tools/build-site.mjs <dir>`) and copy its `p/`,
    `create/presets.json` and FinUI's `fonts/`.
13. The QA stage `finui` runs that repository's QA (its checks and tests live in `../finui/qa`, never in FinUI), serves its
    site, walks the gallery and create, and fails when a file of `web/assets/finui` differs from the repository's (`p/` from
    its built site): copy a change across in the same sitting.

## FinMotion (`web/assets/finmotion/`, `finmotion.rs`)

1. **FinMotion is how FinUI moves, and FinUI is not an animation library** (the owner's words): FinUI's components are still
   and complete on their own, and nothing of motion goes back into FinUI but its animated icons.
2. FinMotion is a repository of its own (`github.com/finstats/finmotion`, checked out as `../finmotion`, its own CLAUDE.md)
   put on top of FinUI, and **FinStats always wears it**: `/assets/finmotion.css` is its stylesheets in its registry's
   order, served as one like `finui.css` and linked after it, and `main.js` calls `motion()` once.
3. Everything moves on four springs (`--spring-settle | -snap | -drift | -glide`), paced by the person's Motion choice
   (FinUI's `--ease` read as `--fm-pace`) and stilled by reduced motion.
4. Each of its parts finds a FinUI component by FinUI's own classes and moves what FinUI draws, so FinUI needs no change for
   it. What a person does moves at once, while an entrance or a flourish waits for an `fm-` class a page adds (`fm-roll`,
   `fm-arrive`, `fm-develop`, `fm-light`, `fm-gathers`, `fm-film`), because FinStats redraws its pages.
5. Developed in its own repository and copied here; the QA stage `finmotion` runs its QA (`../finmotion/qa`) and fails while
   `web/assets/finmotion` differs from it.

## Simple mode (`js/simple/`, `simple.css`, `simple-tokens.css`)

1. **A second layout of FinStats**, laid out the way a classic Plex stats server is, for people who come from one: a solid
   FinUI `top-bar` instead of the menu (search, Home, Libraries, Users with `see_everyone`, History, Graphs, and a gear for
   My profile, Settings, the theme and signing out), and only those pages: Home (Activity, watch statistics, library
   statistics, recently added), Libraries and a library, Users and a person, a title, History, Graphs, Settings. Pages
   FinStats has and it does not (Recap, Together, Pipeline, Security, Server and the rest) are no page there.
2. **It is chosen per browser** (Settings → Appearance → Layout, `finstats.layout` in `state.js`, never the account; nothing
   stored is FinStats) and **loads only its own half**: `theme.js` marks `<html data-layout="simple">` before the first
   paint and writes its two stylesheets after the others, and `main.js` is a bootstrap that imports `simple/app.js` or
   `app.js` (the FinStats layout, unchanged), both ending in `boot.js`. Simple mode never fetches FinStats' pages,
   `shell.js`, `widgets.js` or FinMotion, and imports the pages it shares (sign-in, Settings, licences) only when opened.
3. Its Patch notes are a page of its own (`simple/pages/changelog.js`: every release, each kind named once over a plain list,
   a version list to jump by), drawing the words through `changelog-text.js` as the FinStats layout's page does.
4. **It wears FinStats' own colours**, the person's FinUI preset in the light and the dark like every other page (owner's
   decision: an orange look of its own was too close to the stats server it imitates). `simple-tokens.css` holds only what
   makes it read like a classic stats page (IBM Plex Sans, squarer corners, no card shadow) and its stillness (FinUI's
   Motion: Off and FinMotion's springs set to nothing on the root), and no colour.
5. Its charts put the kinds of media in FinStats' order, so films, episodes and music wear their colours there too.
6. **Everything it asks goes through the same `Scope`**: its cards and tables read the FinStats layout's endpoints
   (`sort=users`, `watched=1`, `parts=`, the peak split, the heatmap's `by_type`, a library's and a person's last play, a
   session's `is_local`), and Most popular and Most active users are drawn only with `see_everyone`.
7. Graphs has the set a classic stats page draws (Media Type, Stream Type, Play Totals) and the one query of its own
   ("Stats layer" 7).
8. Now playing runs on `nowplaying.js`, the clock and loader both layouts share, with its own stream cards.
9. Its components are FinUI's (`stat-card`, `stream-card`, `tile-stat`, `stat-figure`, the solid `top-bar`), developed here
   and copied to `../finui`.
10. History, a library's, a person's and a title's History tab are one server-sorted table (`simple/history.js`).
11. Its label is **Tautulli-style** (owner's decision, 2026-10-26: a comparison, never the bare name, which would read as
    Tautulli built in or endorsed), and its help line says it is not affiliated with Tautulli; no Tautulli logo, colours or
    code, and the mark is FinStats'. In the code it stays Simple mode (`simple/`, `finstats.layout=simple`).

## Formats and the welcome (`when.js`, `welcome.js`, `welcome.rs`)

1. **Date and time formats (Settings → Appearance, 2.3.0)** are this browser's (`finstats.dateFormat`,
   `finstats.timeFormat`; nothing stored is the language's own way, which writes every date and time exactly as before a
   format could be chosen).
2. `when.js` is pure (`writeDate`, `writeTime`, the seven date and four time patterns, tested in the QA stage `i18n`).
   `dom.js`'s formatters (`dateTime`, `dateOnly`, `shortStamp`, `timeOfDay`, `dayLabelYear`) and every page with a clock of
   its own go through `clockTime(d, fallback)` / `calendarDate(d, fallback)`, the fallback being the page's own `Intl`
   formatter. A new place that writes a time of day or a day with its year goes through them too; a date written with a
   month's name ("28 February", "Oct 2026") stays the language's. The server writes no dates a person reads.
3. **The welcome (`/welcome?next=`, 2.3.0).** The first time somebody signs in, on whichever browser, the router's guard
   sends them to a card in setup's own look (`ol.wizard`): language (spoken at once through `loadLanguage`, in place), date
   and time formats, colours (FinUI's `choiceGroup`: Light, Device, Dark, each with a line saying what it does, worn at
   once), layout with a picture of each (`web/assets/welcome/*.png`, 480×300 from the screenshots of the dashboard and of
   the Tautulli-style home).
4. Skip keeps `DEFAULTS` (en-GB, `dd/mm/yyyy`, `HH:mm`, the device's colours, FinStats). Setup goes on into the same steps in
   the same card once its sign-in succeeds, its two steps shown done.
5. Everything it asks is the browser's, but being welcomed is the person's (owner's decision, 2026-10-26): the table
   `welcomed` (migration 40, in `backup::TABLES`), `user.welcomed` on `/auth/me`, and `PUT /api/me/welcomed`, which the card
   awaits before it reloads.
6. QA's instances start with everybody welcomed; the welcome's journeys sign in somebody new.
7. `finstatsWelcome()` in `boot.js` opens it again from the browser's console; first added for trying it by hand, it is kept
   in the release (owner's decision, 2026-10-26).

## Now playing (`widgets.js`, `nowplaying.js`)

1. "Now playing" (`nowPlayingView` in `widgets.js`) is polled every 5 s, but **only the 1 s ticker moves a clock** (+1 whole
   second per beat). A poll never repaints it; it only corrects the position when that means something (pause, server > 3 s
   ahead or > 15 s behind; clients report to Jellyfin roughly every 10 s), and even then by setting it one short so the
   change lands on the next beat. Painting from the poll is what made the clock stutter.

## Image and data folder (`Dockerfile`, `docker-entrypoint.sh`, `main.rs`, `db.rs`)

1. The image has no `USER` line on purpose: `docker-entrypoint.sh` starts as root only to make the data directory belong to
   `PUID:PGID` (default 1000:1000; Docker creates a missing bind-mount folder as root, which is what broke 1.0.0 on fresh
   machines), then `su-exec`s to that user; with `--user` it changes nothing.
2. As root the entrypoint changes only the data folder itself and the entries FinStats keeps there (`chown -h`, following
   no link, leaving a file of another name alone), narrows the folder to its owner and starts FinStats with `umask 077`.
   FinStats itself never runs as root.
3. `main.rs::ensure_writable` fails fast with the fix, and **run as root over a data folder another user owns, FinStats
   refuses** (`as_root_over`) and names the command to use instead, `docker exec -u <PUID> finstats finstats <command>`:
   `docker exec` runs as root past the entrypoint, and a `backups/` it made could no longer be written by the server. Root
   over a folder of root's own is let be.
4. **What FinStats writes is its user's alone**: the data folder is narrowed to 0700 at every start, the database to 0600,
   and backups, uploads, unpacked Tautulli backups and pre-update copies are written 0600 in folders of 0700
   (`db::private_dir`, `private_file`, `private_new_file`).
5. Test the image on folders Docker creates (`qa/run.sh docker`), not on a data folder that already exists on the dev machine.
6. The Docker image ships tzdata ("Stats layer" 4).

## Licence notice (`licenses.rs`, `THIRD-PARTY.json`, `GET /api/licenses`, `/licenses`)

1. **The notice is shipped, not summarised.** `tools/make-third-party.py` walks `cargo metadata` (non-dev, every platform:
   a notice true only on the machine that generated it would be wrong on the others) and reads each crate's own `LICENSE`
   files out of the sources cargo has unpacked. Identical texts are stored once (hundreds of crates ship the same MIT
   wording), and every crate points at the ones it carries.
2. **Nothing in it is retyped**: each text is a licence file as its project wrote it, and the bundled half (`BUNDLED` in
   `licenses.rs`: FinStats' own GPL, every font in `web/assets/fonts` (a test holds each licence there to the notice), the
   map, DB-IP's database) is `include_str!` of the file that really sits on disk.
3. Re-run the generator whenever a dependency is added, removed or bumped: a test holds the file against every package in
   `Cargo.lock`, so a dependency whose licence was never recorded fails `cargo test`.
4. The answer is rendered once into a `OnceLock`: half a megabyte, the same for everyone, unchanging while the process runs.
