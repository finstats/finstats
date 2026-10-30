# FinStats architecture

FinStats is a playback-statistics server for Jellyfin: one Rust binary (axum + tokio + bundled SQLite) with the web UI
compiled into it. It only ever *reads* from Jellyfin; the one thing it writes there is its own API key, during setup.

This document explains how the pieces work and fit together. The item-by-item contracts (every rule, every constant and
the measured reason behind it, every owner's decision) are in [`DECISIONS.md`](DECISIONS.md), cited by section name and
number ("Collector" 4). [`CONTEXT.md`](CONTEXT.md) is the short map (layout, tables, routes, commands, known gaps); the
user-facing manual and the HTTP contract are in the docs repository (`../docs`, <https://finstats.no/>).

**Contents.** §1 The process · §2 Data flow · §3 The database · §4 Reading Jellyfin (§4.1 the client, §4.2 library
reads, §4.3 scheduling, §4.4 Jellyfin's jobs) · §5 Collecting plays (§5.1 two transports, §5.2 modes, §5.3 play rows) ·
§6 History from elsewhere (§6.1 the four producers, §6.2 one duplicate rule, §6.3 relink and locate) · §7 Answering
questions (§7.1 Scope, §7.2 library access, §7.3 insights, §7.4 groups, §7.5 search, §7.6 a person's pages, §7.7 recap,
§7.8 library health and capacity) · §8 People and credentials (§8.1 sign-in, §8.2 keys, §8.3 permissions, §8.4 public
profiles) · §9 Network and security · §10 Around Jellyfin: Pipeline · §11 Telling people (§11.1 notifications, §11.2 the
live channel) · §12 Keeping data (§12.1 backups, §12.2 the trash) · §13 Serving HTTP · §14 Languages · §15 The web UI
(§15.1 pages and data, §15.2 FinUI and FinMotion, §15.3 navigation, §15.4 Simple mode, §15.5 per-browser choices and the
welcome) · §16 Shipping

## 1. The process

One binary, `finstats`, serving by default and taking subcommands that run once and exit (`import-jellystat`,
`import-streamystats`, `relink`, `restore`, `backup`). `main.rs` checks the data folder (`ensure_writable`,
`as_root_over`), opens the database (§3), builds `AppState` and starts the loops:

| Loop | Module | Wakes on |
|---|---|---|
| collector | `collector.rs` + `socket.rs` | the socket, or its own poll timer (§5) |
| scheduler | `sync::scheduler` | once a minute, or `wake` (§4.3) |
| housekeeping | `sync.rs` | every 15 min: server name, a dropped-in GeoIP file, `services::check_all`, the audit log's age, `trash::purge` |
| Jellyfin's jobs | `jobs::run` | every `BUSY_EVERY_S` while a run is known (§4.4) |
| downloads | `downloads.rs` | 5 s / 60 s / 5 min, or its own `Notify` (§10) |
| notifications | `notify::run` | `notify_wake`, or the next retry (§11.1) |
| live channel | `live::run` | once a second while anybody listens (§11.2) |

`state.rs` holds `AppState`, shared as `App` (an `Arc`): the DB handle, the Jellyfin config, `Settings` (one JSON blob in
the `settings` table, changed only through `AppState::update_settings`), the task registry, the now-playing snapshot and
`wake`, a `Notify` the background loops select on. The HTTP server is hyper driven directly (§13).

## 2. Data flow

```
Jellyfin ──/socket push, else /Sessions 1 s / 5 s idle─▶ collector ──▶ SQLite ◀── stats / recap API ◀── embedded SPA (web/)
         ──/Users /Items /Devices /System/* (scheduler)──▶ sync ──┘        ▲
Jellystat / Streamystats backup ──▶ import ──────────────────────┘   relink (after sync/import/start-up)
Sonarr/Radarr ──calendar, history (15 min)──┐
Seerr ──requests (5 min, one row if quiet)──┼──▶ SQLite ──▶ pipeline API ◀── /pipeline
Sonarr/Radarr queues ───────────────────────┴──▶ in-memory snapshot (5 s watched / 60 s busy / 5 min empty) ──▶ /api/downloads
```

Tautulli backups enter beside the other importers (§6.1). Everything a page reads comes from SQLite or from an in-memory
snapshot; no page load makes a request to Jellyfin.

## 3. The database

Bundled SQLite in WAL mode behind an r2d2 pool; all work goes through `db.call(|conn| …)` (`spawn_blocking`). The pool's
init makes every `transaction()` IMMEDIATE and sets `journal_size_limit`. `Db::open` runs, in order:

1. `preflight`: one look at the file before the pool touches it.
2. `refuse_downgrade`: the settings key `app_version` against this binary and its migration count.
3. `back_up_before_update`: a `VACUUM INTO` copy in `<data>/pre-update-backups/` when a different version or pending
   migrations open a populated database.
4. `migrate`: the `MIGRATIONS` array in `db.rs`, by index against `PRAGMA user_version`.
5. Upgrade steps (`upgrade.rs`): one-time work a version needs that is not a table change, recorded by name in
   `upgrade_steps`.

History lives in `playbacks` (one row per play, from any of four producers, §6.1) and `playback_events` (the timeline of
plays FinStats saw itself). Every reading of history goes through the view `visible_playbacks` (`deleted_at IS NULL`);
the table is read directly only for identity (§12.2). Contracts: "Database".

## 4. Reading Jellyfin

### 4.1 The client

`jellyfin.rs` asks every request in PascalCase and with compression; ids are normalised by `db::norm_id`. Jellyfin's
WebSocket (`socket.rs`) cannot ask for PascalCase and re-cases keys itself. Contracts: "Jellyfin client".

### 4.2 Library reads

`sync_libraries` reads each library through `items_page` (twice: every item, then cast and crew of films and shows into
`item_people`), stores items, marks what the read did not return `removed` (behind `sync::trustworthy_removal`) and then
runs the derivations that depend on the library, in this order: `sync::backfill_playbacks` (library ids, runtimes,
season/episode numbers, the imported rows' item types), `relink` (§6.3), `pipeline::link`, `watchlist::resolve`; Library
health and Capacity recompute beside it, never inside (§7.8). `sync_changes` (metadata changes) reads what Jellyfin saved
since the last look, items and people, without removing anything. The users read (`sync_users`) stores users, their
Jellyfin policies (§7.2) and who was removed. All seven paged reads use `sync::page_adds_ids` to stop
when a page brings no new id, with a fresh set per query. Contracts: "Library reads", "Pictures", "Track languages".

### 4.3 Scheduling

Every job that runs by itself (`sync_users`, `sync_libraries`, `sync_events`, `sync_server`, `sync_userdata`,
`sync_changes`, `backup`, `geoip`, the Pipeline reads) carries triggers like Jellyfin's scheduled tasks (daily, weekly,
interval, start-up, and after Jellyfin's library scan), kept in `Settings::schedules` or falling back to `defaults`. The
scheduler looks once a minute or on `wake`, and `schedule::due` alone decides. When each job last finished is in
`task_runs`. Every job can also be run by hand (`api::RUNNABLE`). Settings → Tasks shows and edits it. Contracts:
"Scheduling".

### 4.4 Jellyfin's jobs

`jobs.rs` reads Jellyfin's scheduled tasks for the Server page: what each does (`explain`), when it runs next
(`schedule`), and for a run in progress how long is left (`eta_s`, from watching the percentage move). Fed by lists
FinStats already reads (`jobs::observe`) and by `jobs::run` while a run is known. Contracts: "Jellyfin's jobs".

## 5. Collecting plays

### 5.1 Two transports, one collector

The session list arrives by asking (`GET /Sessions`) or by being told (Jellyfin's `/socket`, `SessionsStart`). `tick()`
handles one list without knowing which. The socket is always spawned; there is no setting.

### 5.2 Modes

`collector::Decide::see` reads each list and chooses: poll at `active_interval_s` while anything runs (the sharpness of a
pause or seek is the gap between sightings), listen on the socket while nothing is loaded or everything has been paused
for a few readings. The subscription follows `should_subscribe`; a safety net (`Net`) reads `/Sessions` once after
silence; every `/Sessions` read in the process takes a slot from one gate (`sessions_slot`). `Pass` bundles `Decide`,
`Net` and the subscription, and `run()` goes through it. What the collector is doing is published on `/api/status`
(`session_mode`, socket flags, poll interval, reads a minute), and `disagrees()` checks it against the rules.

### 5.3 Play rows

A play gets a row the moment it is first seen (`active = 1`), is refreshed every 30 s, counts un-paused time only
(`attribute()`), merges a restart within `merge_window_s`, and diffs consecutive sightings into `playback_events`. When
it vanishes it is closed and its title regrouped (`groups::detect`, §7.4). Contracts: "Collector", "Play rows".

## 6. History from elsewhere

### 6.1 The four producers

`playback.rs::PlayRecord` and `media.rs` (stream and transcode extraction, labels, `effective_play_method`) are shared by
the collector and three importers, so every source writes identical columns: Jellystat (`import.rs`), Streamystats
(`streamystats.rs`), Tautulli (`tautulli.rs`, Plex's history, through a wiring board that maps Plex users to Jellyfin
users). A restore of FinStats' own backup (§12.1) writes the same rows. A fifth tracker is a reader, not new
architecture. All go through `PlayRecord::insert_imported`, one history writer at a time (`api::Running`), and end in
`import::finalize` (backfill, relink, groups). Contracts: "Jellystat import", "Streamystats import", "Tautulli import",
"Imports".

### 6.2 One duplicate rule

`playback::already_recorded` decides whether a play is already in the history, for every importer and the restore:
its own `source_id`, else the same person and item with one of the play's two ends close enough. Contracts: "Duplicate rule".

### 6.3 Relink and locate

Jellyfin ids derive from file paths, so a renamed file orphans its plays. `relink::relink_orphans` (at start-up, after
every library read and import) re-attaches orphans by rules that never guess, moving plays through `move_plays`, which
re-applies the duplicate rule. What no rule finds is listed in Settings → Unlinked media (`locate.rs`), where the owner
picks; the choice is kept in `located` and consulted first next time. Contracts: "Relink", "Locate".

## 7. Answering questions

### 7.1 Scope

Every stats query goes through `Scope` → `Cond` (`stats.rs`): the time window, user and library filters, `min_play_s`,
whose rows the caller may see, and what of the library they may see. `row_json` maps SQL rows to JSON by column name.
Independent queries for one page run side by side (`stats::resolved`, then `apart` under `tokio::try_join!`). Contracts:
"Stats layer".

### 7.2 Library access

`access.rs` gives each person what Jellyfin lets them see: the libraries in their policy and, inside them, the titles
Jellyfin lists for them (`sync::read_visible` → `visible_items`). `Scope` carries it (`access`, `viewer`). History is not
narrowed. Contracts: "Library access".

### 7.3 Playback insights

Where a title loses its viewers, from where each play stopped (`playback::STOP_S`), rewinds and subtitle switch-ons
(from events), and the files list (`GET /api/stats/files`). Contracts: "Playback insights".

### 7.4 Groups

`groups.rs` infers watching together (Jellyfin exposes no SyncPlay groups): plays of one title by several people that
start together, or end together at the same place, and overlap. `detect` writes `playbacks.group_id`; the fold
(`sessions_for` → `fold_sessions` → `pairs`, `per_bucket`, `people_shares`) feeds `/together`, the recap and
`GET /api/stats/groups`; `mark_live` does the same for streams playing now. Contracts: "Groups".

### 7.5 Search

`fuzzy.rs` scores every title and person in Rust (`/api/search`); Activity and the server log filter in SQL, word by
word. Contracts: "Search".

### 7.6 A person's pages

`profile.rs` (show progress, "seen", streaks), `timeline.rs` (one person's plays folded into stops), `watchlist.rs`
(their own list, worked out at read time), `browse.rs` (a library's titles), `recent.rs` (the dashboard's Recently added).
Contracts: "Profiles", "Watchlist", "Timeline", "Browse", "Recently added".

### 7.7 Recap

`recap.rs` builds one person's year (or the server's, `server_edition`); `story.rs` draws its cards from a typed copy
(`StoryYear`); `whose_year` decides whose year a caller may see. Contracts: "Recap".

### 7.8 Library health and capacity

`health.rs` (Server → Library health) lints the library with pure rules over `Item` rows; `capacity.rs` (Server →
Capacity) works out whether the server will cope, from the plays and the disks Jellyfin names. Both are computed after a
library read (and after a metadata look or a server read respectively), stored, and only read by the pages. Findings are
dismissed in `health_dismissed`. Contracts: "Library health", "Capacity".

## 8. People and credentials

### 8.1 Sign-in

`auth.rs` forwards a sign-in to Jellyfin's `AuthenticateByName`, logs that Jellyfin session out at once and mints its own
opaque session (stored hashed, HttpOnly SameSite=Lax cookie). Until FinStats is set up, the wizard needs a one-time code
from the log. A limiter guards sign-ins; a name that may not use FinStats is never put to Jellyfin.

### 8.2 Keys

`keys.rs`: `Authorization: Bearer fs_…` keys, stored as sha256, shown once. `auth::resolve` reads the header first, else
the cookie; both end in the same `AuthUser`, resolved against the live user row and grants (`effective`). A calendar key
opens only `/api/calendar.ics`.

### 8.3 Permissions

`AuthUser.perms` (`see_everyone`, `see_network`, `see_server`, `see_downloads`, `notify`, `manage`) is rebuilt every request
from `user_permissions` ∪ `Settings.default_permissions`; `sign_in` gates access; Jellyfin administrators get `Perms::ALL`.
Extractors: `AuthUser`, `ServerViewer`, `Manager`, `JellyfinAdmin`, `Changer`. The UI mirrors it with `can('perm')`, which
is cosmetic. `audit.rs` records FinStats' own write paths. Contracts: "Auth", "Permissions", "Audit".

### 8.4 Public profiles

`public.rs`, `card.rs`, `/u/:token`: the one read without an account, built from one person's id into typed structs, a
day late, cards rasterised with resvg. Contracts: "Public profiles".

## 9. Network and security

`network.rs` decides local or remote (`network::classify`): a private range, or one of the household's addresses
(`home_addresses`, its public address looked up once). `geo.rs` places addresses from a MaxMind-format database on disk
(`ip_locations`); `security.rs` finds impossible travel and new places in one person's history (`detect`), and bursts of
failed sign-ins in Jellyfin's activity log. `/security` draws them on a hand-drawn map (`worldmap.js`). Contracts:
"Local and remote", "Security".

## 10. Around Jellyfin: Pipeline

`services.rs` keeps connections to Sonarr, Radarr and Seerr (several of a kind). `arr.rs` reads calendars and history,
`seerr.rs` requests, `pipeline.rs` puts them together for `/pipeline` and the calendar feed (`ical.rs`). `downloads.rs`
is the only live part: Sonarr's and Radarr's queues, folded into downloads and kept in memory. Download clients are not
connected; their queues come through Sonarr and Radarr. `outbound.rs` lists every destination FinStats can reach.
Contracts: "Pipeline", "Outbound connections".

## 11. Telling people

### 11.1 Notifications

The only thing FinStats sends. An event is raised once where the thing is noticed (`raise_in`, deduplicated); delivery is
a row per destination with its own retries (`notify::run`). `message` decides what may be said; `wanted_by` who may be
told; `channels.rs` holds every payload (nine kinds of destination, mail in `mail.rs`). Contracts: "Notifications".

### 11.2 The live channel

`live.rs` / `live.js` / `GET /api/live`: one WebSocket per tab carrying `now_playing`, `downloads` and `summary` when they
change, each built by the same function the GET calls, for the connection's person. Page loads stay HTTP; without the
socket the pages poll as before (`follow`). Contracts: "Live channel".

## 12. Keeping data

### 12.1 Backups

`backup.rs`: gzip JSON Lines, one row per line tagged with its table, matched by column name both ways, of the tables in
`backup::TABLES`. A restore reads the whole file, then merges a batch at a time. Pre-update snapshots (§3) are a separate,
whole-database copy. Contracts: "Backups".

### 12.2 The trash

`trash.rs`: a deleted play (`playbacks.deleted_at`) or backup (moved to `backups/deleted/`) waits `trash::KEEP_S` (30
days) before `trash::purge` removes it. Contracts: "Trash".

## 13. Serving HTTP

`main.rs::serve_http` drives hyper directly with timers (head, body rate, writes), a connection cap and a share per address. `api.rs` builds the routes, the Origin
check on writes and `harden`'s headers; the response compression layer skips what is already compressed and the UI's
files are gzipped once (`gzipped`). Pictures go through `artwork.rs`. Contracts: "HTTP server".

## 14. Languages

`i18n.rs` (server) and `web/assets/js/i18n.js` (browser) speak 23 languages from one JSON catalogue per BCP 47 tag in
`web/assets/i18n/`, British English the source. The browser chooses; the server answers in the language each request
carries (`i18n::middleware`, `i18n::current`). Patch notes are translated per release (`changelog/<tag>.md`). Contracts:
"Languages".

## 15. The web UI

No build step, no dependencies, no CDN: vanilla ES modules served from the binary (`rust-embed`; read from disk in debug
builds), fonts bundled.

### 15.1 Pages and data

`main.js` (bootstrap) → `app.js` or `simple/app.js` → `boot.js`. `router.js` (History API; the server answers
`index.html` for any non-`/api`, non-`/assets` path), `dom.js` (`h()`, icons, formatters), `api.js`, `components.js`
(`dataView`, stale-while-revalidate), `charts.js` (hand-rolled SVG), `tables.js`, `prefetch.js`, `pages/*.js`.
Settings and Server are one section at a time (`sections.js`). Contracts: "Front-end".

### 15.2 FinUI and FinMotion

FinUI (`web/assets/finui/`, `finui.rs`) is FinStats' own component library, also a repository of its own (`../finui`);
`/assets/finui.css` is every component's CSS in registry order, in the look of the person's preset (`appearance.rs`).
FinMotion (`web/assets/finmotion/`, `finmotion.rs`, `../finmotion`) moves FinUI's components on four springs, linked
after it and started once by `main.js`. Contracts: "FinUI", "FinMotion".

### 15.3 Navigation

FinUI's `desktop-nav` and `mobile-nav` (a style each browser chooses), search in the menu or the page (FinUI `search`,
`search.js`), context menus from links (`menus.js`), Esc handled in `shell.js`. Contracts: "Navigation".

### 15.4 Simple mode

A second layout (`js/simple/`, `simple.css`, `simple-tokens.css`) laid out like a classic Plex stats server, labelled
Tautulli-style, chosen per browser, loading only its own half. Contracts: "Simple mode".

### 15.5 Per-browser choices and the welcome

Theme, layout, menu style, search mode, language and date/time formats (`when.js`) are the browser's; being welcomed
(`welcome.js`, `welcome.rs`, the `welcomed` table) is the person's. Contracts: "Formats and the welcome", "Theming".

## 16. Shipping

`Dockerfile` + `docker-entrypoint.sh` (fixes the data folder's owner, then drops to `PUID:PGID`);
`.github/workflows/docker.yml` tests, builds amd64 and arm64 and publishes to `ghcr.io/finstats/finstats`. The licence
notice (`licenses.rs`, `THIRD-PARTY.json`) and the patch notes (`changelog.rs`, `CHANGELOG.md`) are compiled in.
Contracts: "Image and data folder", "Licence notice", "Changelog".
