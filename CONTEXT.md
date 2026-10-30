# CONTEXT.md

> **Last updated:** 2026-10-30 (backup placement ownership and atomic undo-delete), on branch `dev` (crate version 2.3.1, local only; schema `user_version` 42; newest pushed tag `v2.2.0`).
>
> A briefing for whoever starts work here next, a person or an AI assistant: what FinStats is, how the pieces fit,
> where things belong, which rules hold, and what is missing. It is the distilled map. `ARCHITECTURE.md` explains how
> the pieces work, `DECISIONS.md` holds every rule with its reason and the owner's decisions, and `CLAUDE.md` says how
> we work; the published docs at <https://finstats.no/> (repository `../docs`) are the user-facing manual and the HTTP
> contract. When this file and the code disagree, the code wins, and this file should be fixed in the same change.

---

## 1. Project overview

FinStats is a self-hosted **playback-statistics server for Jellyfin**. It watches what people play, keeps the history in
SQLite, and serves a web UI with dashboards, per-person profiles, a yearly recap, group-watching detection, library
health, security alerts, notifications and a "Pipeline" view of Sonarr / Radarr / Seerr. It can import history from
Jellystat, Streamystats and Tautulli (Plex).

It **only reads from Jellyfin**. The one write it ever makes there is creating its own API key during setup.

| Aspect | Choice |
|---|---|
| Shape | One Rust binary (`finstats`) with the web UI embedded; one data folder |
| Language | Rust, edition 2024, `GPL-3.0-only` |
| HTTP server | `axum 0.8` (HTTP/1 only, `ws`) served through `hyper` directly (15 s for a request head, then 1 KB/s for its body, 30 s per write, 512 connections, 64 per public address), `tower-http` gzip compression |
| Async | `tokio` multi-thread runtime with **2 workers, 8 blocking threads** |
| Database | SQLite, bundled, through `r2d2` + `r2d2_sqlite` (pool of 6), WAL; rusqlite re-exported as `db::rusqlite` |
| Outbound HTTP | `reqwest 0.12`, rustls, `gzip` + `brotli` decoding |
| Jellyfin push | `tokio-tungstenite 0.30` WebSocket to Jellyfin's `/socket` |
| Mail | `lettre` (SMTPS or STARTTLS only) |
| Image cards | `resvg` rasterising hand-built SVG, Inter TTFs in `fonts/card/` |
| GeoIP | `maxminddb` (mmap of a private copy) |
| Errors / logs | `anyhow` internally, `ApiError` at the HTTP edge; `tracing` |
| Frontend | Vanilla ES modules, **no build step, no npm dependencies, no CDN**, served from the binary via `rust-embed` |
| Component library | FinUI (`web/assets/finui/`, its own repo `../finui`) |
| Motion | FinMotion (`web/assets/finmotion/`, its own repo `../finmotion`) |
| Packaging | Docker image `ghcr.io/finstats/finstats` (amd64 + arm64), Alpine runtime |
| Users | Jellyfin users sign in with their Jellyfin credentials; Jellyfin administrators are FinStats administrators |

**Naming:** written **FinStats** wherever a person reads it; `finstats` only as an identifier (crate, binary, image,
`finstats.db`, `FINSTATS_*`, storage keys, `jellyfin::APP_NAME`).

---

## 2. Repository layout

```
finstats/
├── Cargo.toml / Cargo.lock      Crate manifest; release profile thin-LTO + strip + overflow-checks. No dev-dependencies.
├── src/                          The whole server. Flat: one module per subsystem (see §3 and §7).
│   └── finui/preset.rs           Decodes a FinUI preset code into token-file overlays.
├── web/                          Embedded UI (debug builds read it from disk; release builds embed it)
│   ├── index.html                SPA shell (finui.css, finmotion.css, app.css, theme.js, main.js)
│   ├── public.html               Public profile page /u/:token (server-templated, outside the app)
│   ├── favicon.ico
│   └── assets/
│       ├── app.css               FinStats' own layout rules (no colour tokens of its own)
│       ├── simple.css, simple-tokens.css   Simple mode only
│       ├── js/                   The application (see §8)
│       │   ├── pages/            One module per FinStats-layout page
│       │   ├── settings/         One module per Settings section
│       │   ├── simple/           Simple mode (Tautulli-like layout) and its pages/
│       │   └── public/           Public profile page script
│       ├── i18n/                 UI catalogues, one JSON per language (24; en-GB the source)
│       ├── finui/                FinUI copy: registry.json, tokens.css, base.css, core.js, components/, create/, p/
│       ├── finmotion/            FinMotion copy: registry.json, core/, parts/, components/
│       ├── fonts/                Bundled woff2 families + their OFL licences (preset-selectable)
│       ├── geo/world.json        Pre-projected Natural Earth map for the Security page
│       └── logo.svg, mark-*.svg, apple-touch-icon.png
├── fonts/card/                   Inter TTFs for server-rendered PNG cards (resvg cannot read woff2)
├── CHANGELOG.md                  Patch notes source (en-GB); include_str!'d, parsed by changelog.rs
├── changelog/                    Translations of CHANGELOG.md, one per language (embedded)
├── THIRD-PARTY.json              Generated licence notice for every crate in Cargo.lock
├── tools/                        make-third-party.py (licence notice), make-world-map.py (world.json), make-languages.py (README's languages picture)
├── Dockerfile, docker-entrypoint.sh, docker-compose.yml, .dockerignore
├── .github/                      workflows/docker.yml (test, build, publish, release), issue/discussion/PR templates
├── .githooks/pre-commit          Refuses data, DBs, tracker exports, secrets, files > 2 MB
├── README.md                     Landing page only (docs live in ../docs)
├── CONTRIBUTING.md, SECURITY.md  Pointers to the docs site
├── CODE_OF_CONDUCT.md, LICENSE
├── CLAUDE.md                     How we work (for AI assistants): TDD, commands, git, releases
├── ARCHITECTURE.md               How the pieces work and fit together
├── DECISIONS.md                  Every rule with its reason, numbered per section
└── CONTEXT.md                    This file

Local only, never committed (git-ignored or excluded):
├── data/                         A real install's data folder (real users, IPs, Jellyfin key). Never read into docs/tests.
├── qa/                           The local release gate (its own private repository), see §11
├── internal/                     Private dev log and notes
├── ux-rules.md                   Third-party UX reference copy
└── target/
```

Sibling checkouts the work depends on: `../docs` (MkDocs site, the API contract `docs/finstats/api.md`), `../finui`,
`../finmotion`, `../FinBranding` (logo masters).

---

## 3. Architecture

### 3.1 Data flow

```
                 /socket push (SessionsStart), else GET /Sessions on a 1 s / 5 s beat
Jellyfin ───────────────────────────────────────────────▶ collector.rs ──┐
         ─── /Users /Items /Devices /System/* /ScheduledTasks (scheduler)▶ sync.rs ──┤
Jellystat / Streamystats / Tautulli backup ─▶ import.rs, streamystats.rs, tautulli.rs ─┤
FinStats' own backup ─▶ backup.rs (restore) ───────────────────────────────────────────┤
                                                                                       ▼
                                                             SQLite (data/finstats.db)
                                                                                       │
            relink.rs / locate.rs (re-attach orphaned plays) ◀── after sync, import, start-up
                                                                                       │
            stats.rs, groups.rs, recap.rs, profile.rs, … (read-only queries) ◀─────────┤
                                                                                       ▼
                    api.rs router (/api/*) ──▶ embedded SPA (web/)   ◀── live.rs WebSocket (/api/live)
                                           ──▶ /u/:token public pages + PNG cards (public.rs, card.rs)

Sonarr / Radarr ── calendar, history (15 min) ─┐
Seerr ─────────── requests (5 min) ────────────┼─▶ SQLite ─▶ pipeline.rs ─▶ /pipeline
Sonarr / Radarr queues ─▶ downloads.rs (in-memory snapshot, 5 s / 60 s / 5 min) ─▶ /api/downloads, live channel

notify.rs ── events + per-destination deliveries ──▶ channels.rs (8 HTTP channels) / mail.rs (SMTP)
```

### 3.2 Processes inside the binary

`main.rs` parses the CLI **before** touching the data folder, opens the DB (`Db::open`: preflight, pool, downgrade
refusal, pre-update snapshot, migrations, upgrade steps), then in `serve` spawns these long-lived loops:

| Loop | Module | What it does |
|---|---|---|
| Collector | `collector::run` | Owns the Jellyfin socket; records plays |
| Scheduler | `sync::scheduler` | Once a minute (or on `wake`): runs due jobs; 15-min housekeeping beat |
| Jellyfin jobs | `jobs::run` | Re-reads Jellyfin's scheduled tasks every 10 s while one runs (for ETAs) |
| Downloads | `downloads::run` | Reads Sonarr/Radarr queues into memory |
| Live channel | `live::run` | Once a second while anyone listens: diff in-memory state, push to browsers |
| Notifications | `notify::run` | Sends queued deliveries with backoff |
| Image cache prune | `api::prune_image_cache` | Daily |
| Public IP | `network::refresh_if_unknown` | One-shot at start-up if never learned |

Shutdown is graceful on Ctrl-C / SIGTERM, or on `AppState::request_halt` (data-integrity guard), which exits with
**code 70** and a reason on stderr.

### 3.3 Layers and where logic lives

- **Handlers are thin.** Most modules end with axum handlers that build a `Scope` or check a permission, then call pure
  functions over a `&Connection`. Business logic lives in the subsystem module (`stats.rs`, `watchlist.rs`, …), not in
  `api.rs`. `api.rs` holds the router, middleware, shared extractors and the admin handlers (settings, tasks,
  permissions, backups, imports, image proxy, cards), some with inline SQL.
- **Pure core, effectful shell.** Decision logic is written as pure functions and tested directly: `collector::Decide`,
  `schedule::due`, `security::detect`, `health::findings`, `timeline::fold`, `recent::fold`, `groups::fold_sessions`,
  `notify::wanted_by` / `message`, `channels::payload`, `capacity::*`. The effectful code around them is small.
- **Database access:** always `app.db.call(move |conn| …)` (pooled connection on `spawn_blocking`, returns
  `anyhow::Result`). `db.conn()` only when already on a blocking thread (imports, restore). Never block the 2 tokio workers.
- **Shared state:** `state.rs` `AppState` (shared as `App = Arc<AppState>`): DB handle, Jellyfin config, `Settings`, the
  in-memory task registry (`Tasks`), the live now-playing snapshot, collector status, downloads snapshot, notify fan-out,
  the live `Feed`, `card_permits` (`Semaphore(2)`), and `wake: Notify` that background loops `select!` on.
- **Settings:** one JSON blob (`settings` row `settings`), `#[serde(default)]`. **Every change goes through
  `AppState::update_settings`** (one lock: read, change, validate, store, publish, then `live_feed.recheck()`). An
  unreadable blob is an error naming the bad key, never a silent reset.
- **Background tasks** register in `Tasks` (`try_start`, `try_start_alone`, `update`, `finish`) with ids from `TASK_IDS`;
  `try_start` panics on an unknown id, and tests hold `RUNNABLE`, `HISTORY_WRITERS` and `schedule::SCHEDULED` to it.

---

## 4. Domain model

### 4.1 Conventions

- **Jellyfin ids** (users, items, libraries, series, seasons, people, devices) are TEXT normalised by `db::norm_id`
  (no dashes, lowercase), everywhere.
- **FinStats' own rows** use `INTEGER PRIMARY KEY`; `AUTOINCREMENT` where an id must never be reused (`services`,
  `notify_targets`, `notify_events`, `api_keys`, `audit`, `watchlist`).
- **Time** is unix seconds. Exception: `upcoming.day` is a `YYYY-MM-DD` film release day.
- **JSON in TEXT columns** is common (`items.provider_ids`, `items.genres`, `*_languages`, `playbacks.transcode`,
  `notify_events.data/private`, `audit.detail`, `user_permissions.permissions`, `*_findings.evidence`, …).
- **Soft deletes:** `users`, `libraries`, `items` carry `removed`; `playbacks.deleted_at` is the trash;
  `requests.removed_at` marks a Seerr purge.
- **Relationships are mostly by convention, not FOREIGN KEY.** Plays must outlive their item: an orphaned play (item
  re-id'd by Jellyfin) is re-attached later by `relink` / `locate`. Declared FKs (with `ON DELETE CASCADE`):
  `playback_events → playbacks`, `upcoming / requests / grabs → services`, `notify_deliveries → notify_events, notify_targets`,
  `notify_targets.owner_id → users`.

### 4.2 Entity relationships

```
LIBRARY (re-read from Jellyfin; not in backups)
  libraries ──< items  (films, shows, seasons, episodes, tracks in one table, told apart by `type`;
                        series_id / season_id point back into items)
  items ──< item_people (item_id, person_id, kind)   no people table; person_id is Jellyfin's
  items ──< item_external (item_id, source) = provider_ids turned sideways; one id may map to several items (HD + 4K)
  users ──< user_items >── items   Jellyfin's played / favourite flags
  users ──< visible_items >── items   what Jellyfin lists for somebody it hides titles from (access.rs)

HISTORY (backed up)
  users ──< playbacks >── items        (item_id may be ORPHANED until relink/locate)
               │  group_id → playbacks.id (self: lowest id of a group-watch)
               │  remote_ip → ip_locations.ip ; device_id → devices
               ├──< playback_events  (FK, cascade)   pause / seek / track / transcode timeline, live plays only
  views: visible_playbacks (deleted_at IS NULL), trashed_playbacks
  manual_seen (user_id, item_id)       located (from_id = orphaned item id → to_id = items.id)
  devices (device_id, user_id)          server_events (Jellyfin activity log)

PEOPLE'S OWN / AUTH
  users ──< sessions (token_hash)   ──< api_keys (token_hash UNIQUE, scope full|calendar)
        ──1 user_permissions        ──1 public_profiles (token)   ──1 appearance (finui_preset)
        ──< watchlist (item_id? or tmdb/tvdb/imdb ids)            audit (FinStats' own write log)

PIPELINE (not in backups)
  services ==< upcoming   (calendar)      item_id → items
           ==< requests   (Seerr)         user_id → users, item_id → items
           ==< grabs      (download history)

NOTIFICATIONS (not in backups)
  notify_targets (owner_id NULL = server's) ==< notify_deliveries >== notify_events (dedupe UNIQUE)

SECURITY / NETWORK
  ip_locations (ip)   home_addresses (ip, lookup|manual)   security_alerts (dedupe UNIQUE)

FINDINGS
  health_findings (key) ──< health_libraries (key, library_id)   capacity_findings (key)
  health_dismissed (key, fingerprint) → health_findings.key OR capacity_findings.key

MISC
  settings (key, value)   task_runs (task: when each scheduled job last finished)
```

### 4.3 Tables

| Table | PK | Key columns | Notes |
|---|---|---|---|
| **libraries** | id | name, collection_type, removed | A library leaving Jellyfin takes its titles with it |
| **items** | id | library_id, type, name, original_title, series_id, season_id, index_number, index_number_end, parent_index_number, album, album_artist, runtime_s, date_created, provider_ids, image_tag, backdrop_tag, path, size_bytes, codecs, width/height, audio/subtitle_languages, removed | A file exists when `path`/`size_bytes` are set |
| **item_people** | (item_id, person_id, kind) | name, role, sort, image_tag | Actors (first 12) and directors of films/shows only |
| **item_external** | (item_id, source) | value | Rebuilt from `provider_ids` by `pipeline::rebuild_external` |
| **user_items** | (user_id, item_id) | played, is_favorite, play_count | Jellyfin's flags |
| **users** | id | name, is_admin, is_disabled, image_tag, removed, access_all, access_libraries, access_hides, visible_for, remote_access | Mirror of Jellyfin users; `access_*` is their library access from Jellyfin's policy (NULL = unread, opens nothing); `remote_access` is Jellyfin's switch for connecting from outside, applied to every request |
| **visible_items** | (user_id, item_id) | | Titles Jellyfin lists for a person something hides titles from; counts only while `users.visible_for` is the policy it was read under. Not in backups. New in 2.3.1 |
| **playbacks** | id | source (`live`/`jellystat`/`streamystats`/`tautulli`), source_id UNIQUE, active, user_id, item_id, item_type, series_id, season_id, library_id, started_at, ended_at, duration_s, paused_s, position_s, runtime_s, remote_ip, is_local, play_method, transcode, group_id, deleted_at | Denormalised names kept so a play reads without the library |
| **playback_events** | id | playback_id (FK), at, kind, position_s, from_s, detail | Live plays only |
| **devices** | (device_id, user_id) | client, app_version, last_ip, first/last_seen | |
| **server_events** | id (Jellyfin's) | date, type, severity, user_id, remote_ip | Contains no playback errors |
| **manual_seen** | (user_id, item_id) | | `POST /api/me/seen` |
| **located** | from_id | to_id, at, by | Owner's choices in Unlinked media; consulted before any relink rule |
| **sessions** | token_hash | user_id, is_admin (fallback only), expires_at, seq | sha256 of token; max 30 per person |
| **api_keys** | id | token_hash, user_id, name, scope, expires_at, last_used_*, revoked_at | `fs_` + 64 hex |
| **user_permissions** | user_id | permissions (JSON) | Grants only add |
| **public_profiles** | user_id | token, published, display_name, sections | |
| **appearance** | user_id | finui_preset | No row = FinUI defaults |
| **welcomed** | user_id | at | Who has had the welcome, once a person; in backups. New in 2.3.0 |
| **watchlist** | id | user_id, kind, item_id?, tmdb/tvdb/imdb, title, year, added_at, missing, arrived_at | Max 1,000 per person |
| **audit** | id | at, kind, user_id, key_id, target, detail, outcome | |
| **services** | id | kind (sonarr/radarr/seerr), url, secret, accept_invalid_certs, enabled | Ids never reused; holds secrets |
| **upcoming** | (service_id, kind, external_id, release) | at / day, ids, has_file, item_id | |
| **requests** | (service_id, request_id) | status, media_status, user_id, item_id, available_at, removed_at | |
| **grabs** | (service_id, history_id) | event, at, quality, indexer, download_id | |
| **notify_targets** | id | kind, url (a credential), secret, topic, owner_id, events, with_addresses, options | |
| **notify_events** | id | kind, dedupe UNIQUE, at, user_id, data, private, historic | Thinned after 30 days |
| **notify_deliveries** | (event_id, target_id) | state, attempts, next_at | One retry clock per destination |
| **ip_locations** | ip | country, city, lat/long | All-NULL row = looked up, no place |
| **home_addresses** | ip | source | |
| **security_alerts** | id | kind, user_id, dedupe UNIQUE, details, resolved_at, muted | |
| **health_findings** | key | kind, item_id, evidence, fingerprint, wasted_bytes | Recomputed, never edited |
| **health_libraries** | (key, library_id) | | |
| **health_dismissed** | key | fingerprint | Holds while the fingerprint matches |
| **capacity_findings** | key | kind, evidence, fingerprint | New in 2.3.0 |
| **task_runs** | task | state, started_at, finished_at | |
| **settings** | key | value | Key/value store (see 4.5) |

### 4.4 Views and indexes that matter

- **`visible_playbacks`** is the source for every reading of history; **`trashed_playbacks`** for the trash. The table
  `playbacks` is read directly only for *identity* (dedupe, source_id, relink/locate, collector deletes, address reads).
  `db::tests::every_read_of_history_leaves_the_trash_out` scans every string literal in `src/` and fails on any other
  read; exceptions are listed with reasons in `IDENTITY_READS`.
- Dedupe indexes `(user_id, item_id, started_at)` and `(user_id, item_id, ended_at)`; partial indexes such as
  `idx_items_added`, `idx_items_type_added`, `idx_playbacks_trash`, `idx_pb_active`, `idx_pb_group`; `idx_pbe_kind`;
  `idx_item_external`. Several query plans are pinned by tests (see §11.1).

### 4.5 Settings

The `Settings` blob (`state.rs`) groups: access (`allow_user_login`, `default_permissions`), collector
(`active_interval_s`, `idle_interval_s`), history rules (`merge_window_s` 600, `min_play_s`, `group_window_s` 60),
network (`public_ip_lookup`, `home_addresses`), security (`travel_speed_kmh`, `travel_min_km`), backups
(`backup_every_d`, `backup_keep`), scheduling (`schedules`: job → triggers; legacy `follow_jellyfin_scan`,
`sync_interval_h`, `geoip_download` only seed defaults), links (`public_url`, `jellyfin_public_url`, `public_profiles`).

Other `settings` keys: `app_version` (downgrade guard), `upgrade_steps`, `update_copy`, `backups_requested` (a manager's
backups, pruned apart; not exported), `jellyfin_url`,
`jellyfin_api_key`, `device_id` (secrets), `library_synced_at`, `changes_checked_at`, `health_computed_at`, `capacity`,
`groups_detected`, `network_classified` (re-derive-once markers), `service:{id}:*` cursors.

### 4.6 Backups

gzip JSON Lines (`finstats-backup-YYYYMMDD-HHMMSS.jsonl.gz`, in UTC, trying up to 1,024 earlier seconds on name collisions, never written over an existing file; placement and undo-delete require hard-link support): header, the settings blob, then rows tagged by table and
matched **by column name** both ways. `backup::TABLES`: playbacks, playback_events, manual_seen, user_permissions,
home_addresses, server_events, devices, security_alerts, audit, watchlist, located, health_dismissed, appearance, welcomed.

Never exported: secrets (Jellyfin URL/key, sessions, api_keys, services, notify_targets), the library (re-readable),
Pipeline tables, derived tables, public_profiles, library access (`visible_items`, `users.access_*`). Restore reads the whole
file first without a lock (refused past a hundred times its size, at least 64 MB, at most 64 GB), then **merges** a batch at a
time (dedupe through `already_recorded`, remaps event ids, never undeletes, keeps plays and their numbers within an import's
bounds), brings settings only with `settings=true` (CLI `finstats restore <file> --with-settings`) and then never wider than
this install has them (`narrow_settings`, `narrow_grants`); without settings it leaves out home addresses, mutes and located
titles the library lacks, and marks audit rows `restored_from`. A restored live play gets `source_id = 'restored:…'`, so
**seen live is `source = 'live' AND source_id IS NULL`**. Then it re-derives groups, `is_local`, library links. A backup a
manager runs by hand is pruned in its own pool. Restore preflight checks parsing, the header and size; a SQL failure
rolls back only its batch, leaving earlier batches. An interrupted valid backup can be retried; invalid rows need correcting.
Separately, `pre-update-backups/` holds full
`VACUUM INTO` copies taken before a newer binary migrates (newest 3 of FinStats' own kept, never served).

---

## 5. API surface

`/api/*` is a nested router behind `same_origin`; unknown `/api` paths answer JSON 404. Everything else non-asset falls
back to `index.html` (SPA). Layers, outermost first: gzip compression (skips `application/gzip` and fonts; the UI's own files
are gzipped once and kept), `i18n::middleware` (the `finstats_lang` cookie, else Accept-Language; nothing FinStats speaks is
`i18n::DEFAULT`, en-US), `security_headers` (`harden`: CSP, `Cross-Origin-Opener-Policy`, `Permissions-Policy`; on `/api/*`
also `Cross-Origin-Resource-Policy: same-origin` and `no-store` unless the answer says otherwise).

The authoritative contract is `../docs/docs/finstats/api.md`; change it with the endpoint (a QA check reads it).

| Group | Methods | Who | Module |
|---|---|---|---|
| `/status` | GET | **anyone** (shape only, from memory) | auth |
| `/setup/test`, `/setup` | POST | anyone, only while unconfigured, with the one-time code from the log (428 without it), rate-limited; needs a Jellyfin admin | auth |
| `/auth/login`, `/auth/logout`, `/auth/me` | POST, POST, GET | anyone / session (logout ends every session the browser sent) | auth |
| `/summary`, `/now-playing`, `/stats/*`, `/library/{insights,recent}`, `/libraries*`, `/items/{id}`, `/people/{id}`, `/search`, `/users*`, `/users/{id}/{shows,timeline}`, `/activity`, `/stats/groups` | GET | AuthUser, narrowed by `Scope` (library, title, person pages and search also by library access: hidden = 404) | stats, groups, browse, recent, profile, timeline |
| `/activity/{id}` | PUT `{deleted}` | Manager | stats + trash |
| `/live` | GET (WebSocket) | cookie or Bearer, Origin-checked, never a calendar key | live |
| `/recap`, `/recap/cards/{chapter}`, `/recap/cards.zip` | GET | AuthUser (`whose_year`) | recap, story |
| `/me/seen`, `/me/watchlist*`, `/me/appearance`, `/me/public-profile*` | various | caller only | profile, watchlist, appearance, public |
| `/keys*` | GET/POST/DELETE | AuthUser; writes need a session | keys |
| `/calendar.ics` | GET | CalendarKey (a calendar key in `?key=`, any key as Bearer, never cookie) | ical |
| `/upcoming`, `/requests*` | GET | AuthUser, pinned scope | pipeline |
| `/downloads*` | GET | DownloadsViewer | downloads, pipeline |
| `/notifications*` | various | AuthUser + `notify`/ownership inside | notify |
| `/security*` | GET / POST | `see_network`+`see_everyone`; writes Manager (the database download also waits `by_hand`'s minute) | security |
| `/server`, `/jellyfin/jobs`, `/events`, `/library/health*`, `/server/capacity` | GET | ServerViewer (others' log lines and a device's last user need `see_everyone`, addresses `see_network`) | stats, jobs, health, capacity |
| `/library/health/{dismiss,undismiss}` | POST | Manager + `see_server` | health |
| `/settings`, `/settings/public-ip`, `/tasks*` | various | Manager (access keys, backup count and interval, merge window and the users / activity log / backup schedules admin-only, the access keys by a session only; `jellyfin_url` and the data folder told to admins only; home addresses and travel rules need the Security page's permissions; work by hand once a minute for a manager, `api::by_hand`) | api |
| `/import/*` | various | Manager + `see_everyone` (an upload past 4 GB is refused; one Tautulli upload at a time; cancelling a board too) | api, tautulli |
| `/library/missing*`, `/library/locate` | various | Manager + `see_everyone` | api, locate |
| `/permissions*`, `/backups*`, `/services*`, `/audit`, `/outbound`, `/public-profiles*` | various | JellyfinAdmin (permissions, a restore and a service's create / update / test need a session, never a key) | api, services, audit, outbound, public |
| `/img/item/{id}`, `/img/user/{id}`, `/img/arr/{svc}/{media}` | GET | AuthUser (item/user: only what FinStats has, `artwork::known`, and what the caller may see, `artwork::may_show`; arr: only ids FinStats listed) | api, artwork |
| `/public/{token}*`, `/u/{token}*` | GET | **anyone** with the token; every failure is the same 404 | public, card, story |

**Conventions**

- JSON in, JSON out. Handlers return `ApiResult<T = Json<Value>>`, usually `json!` built from `row_json` rows; public
  profiles and story cards use **typed structs** on purpose (nothing added to a query can leak).
- Errors: `ApiError(StatusCode, String)` → `{"error": "..."}`. `anyhow`/rusqlite errors log at ERROR and answer 500
  with a fixed message (never internals). Helpers `bad_request`, `not_found`, `forbidden`, `not_permitted`. Also used:
  401, 409 (already running / name taken), 429 (sign-in limit; work by hand within its minute), 502 (upstream unreachable),
  202 (task started).
- Writes (non-GET) must pass the Origin check (`same_origin`; `X-Forwarded-Host` only from a peer `FINSTATS_TRUST_PROXY`
  names; the scheme is compared too when a believed proxy sends `X-Forwarded-Proto` or the page is on `public_url`'s host).
- CSP: `default-src 'self'; img-src 'self' data:; style-src 'self'; script-src 'self'; object-src 'none'; frame-ancestors 'none'; …`.
  The UI may not use inline `<style>` or `style=""`.
- Static assets: ETag + `no-cache` (fonts are immutable). `/assets/finui.css` is per-person (`private`, `Vary: Cookie`).
  Images: ETag from the Jellyfin image tag, `private, no-cache`, 304 before any read.
- Paginated lists sort server-side through `stats::order_by` (whitelisted keys, stable tiebreaker).
- Query structs must **not** use `#[serde(flatten)]` (serde_urlencoded then passes numbers as strings → 400).

---

## 6. Auth and permissions

- **Sign-in:** `POST /auth/login` forwards to Jellyfin's `AuthenticateByName`, logs that Jellyfin session out at once,
  and mints FinStats' own session (random token, stored as sha256; cookie `finstats_session`, or `__Host-finstats_session`
  on https, which wins and which a sibling host cannot set; HttpOnly, SameSite=Lax, 30 days, Secure when `X-Forwarded-Proto`
  or the sign-in's `Origin` says https). A token begins `h` or `p` (begun on https or http), and an `h` token under the plain
  name is refused. Rate limit 10 attempts / 5 min per address (an IPv6 /64 counting as one), 50 per IPv6 /48, and nothing
  forwarded past 60 failures a minute server-wide (attempts put to Jellyfin only, those under way included); at 10,000 remembered addresses a new one is refused; a failure is audited
  once per address and window; a sign-in that works gives back only its own attempt. Once a users read has succeeded, a
  name that may not use FinStats is answered as a wrong password without asking Jellyfin (`may_try`), no sooner than
  Jellyfin would; with Jellyfin's `EnableRemoteAccess` off, a sign-in from outside the home network is refused (403).
  `FINSTATS_TRUST_PROXY` names the proxies whose `X-Forwarded-*` is believed (`auth::Proxies`), the header walked from the
  right past them. Every sign-in rewrites the user row from Jellyfin's answer, its library policy included.
- **Setup:** until configured, `main.rs` writes a one-time code to the log (`auth::new_setup_code`, kept in
  `AppState.setup_code`); `/setup/test` and `/setup` refuse without it (428) and count against the sign-in limit, and
  `SETUP_FINISHING` holds the last check and the write together.
- **One resolution point:** `auth::resolve`. A `Bearer fs_…` header beats the cookie; a malformed header is 401 and never
  falls back. Sessions and keys are both resolved against the **live** `users` row and live grants (`auth::effective`),
  so a demotion, disable or delete in Jellyfin takes effect immediately. A users read that does not say somebody is enabled
  keeps them out. A non-`Bearer` `Authorization` header beside a cookie (a proxy's Basic sign-in) is ignored.
- **What outlives a key needs a session** (`keys::needs_a_session` / `only_a_session`, 403 to a key): minting and revoking
  keys, permissions, the access settings, a restore, a service's create / update / test, notification destinations.
- **Perms:** `see_everyone`, `see_network`, `see_server`, `see_downloads`, `notify`, `manage` (+ `sign_in`). Jellyfin
  admins get `Perms::ALL`. Others get nothing unless `allow_user_login` or `sign_in`; then their grants ∪
  `default_permissions`. Grants only add; no denies.
- **Extractors:** `AuthUser`, `Manager`, `ServerViewer`, `DownloadsViewer`, `JellyfinAdmin` (the only one that may edit
  permissions), `CalendarKey`. A calendar-scoped key is refused by `AuthUser` everywhere.
- **Library access is Jellyfin's, not a permission** (`access.rs`): each person sees the libraries Jellyfin opens to them and,
  inside them, only the titles Jellyfin lists for them under their rating cap, tags and unrated rules (`visible_items`, read
  by `sync::read_visible` through `/Items?UserId=` only for people something hides titles from). Administrators see
  everything; an unread policy opens nothing. `Scope` carries `access` and `viewer`; applied to library pages and titles,
  Recently added, a title's and a person's page, search, library insights, pictures, the watchlist, Coming up and the
  calendar feed, and arrivals told to personal destinations. History is not narrowed; Library health and Capacity are
  whole for `see_server`.
- **In queries, decide by the specific permission, never by `is_admin`**: `see_everyone` for whose rows,
  `see_network` for IPs/devices/`is_local`, `see_server` for file paths and failed sign-ins. The recap is the documented
  exception (own year for everyone; any user or the server for Jellyfin admins), and its rank is left out without
  `see_everyone`.
- **UI** mirrors this with `can('perm')` from `state.js`; it is cosmetic. Every rule is enforced server-side.
- **A manager is not an administrator** (`api::Changer`): settings whose value changes are judged key by key, so a form may
  send what it may not change, unchanged. Work started by hand (a job, the public-address lookup, a regroup, the location
  database) goes through `api::by_hand`: a manager once a minute, an administrator not held.
- **Audit:** `audit.rs` records FinStats' own write paths (`KINDS`). A new write path gets a kind and one `record`.

---

## 7. Subsystems in brief

**Jellyfin client (`jellyfin.rs`).** Every request asks `Accept: application/json; profile="PascalCase"`; all JSON
access assumes PascalCase. Library reads pass `CollapseBoxSetItems=false` and `ExcludeLocationTypes=Virtual`: anything
a read does not return gets marked removed, so a query that hides items is a data-loss bug.

**Collector (`collector.rs`, `socket.rs`).** Session lists come pushed (socket) or polled; `tick()` cannot tell which.
`Decide::see` picks the mode: anything running → poll at `active_interval_s`; nothing loaded or everything paused for
3 readings → listen on the socket. A safety-net read (`Net::take_read`) fires after silence; every `/Sessions` read in
the process goes through one rate gate. `/api/status` publishes `session_mode`, socket flags, poll interval and reads
per minute. A play row is inserted on first sight (`active=1`), saved every 30 s or on events, merged with a restart
within `merge_window_s`, and closed when it vanishes (then `groups::detect`). Time between sightings belongs to the state
the play was already in, so a pause never becomes watch time.

**One play shape (`playback.rs`, `media.rs`).** `PlayRecord` is produced by the collector and all three importers.
`playback::already_recorded` is the one duplicate rule: same `source_id`; within one source the very same second; across
sources either end within `merge_window_s`. Importers call `insert_imported`, never `insert`. `STOP_S`, `PLAY_FRAC`,
`SEEN_AT` (0.8) are the one rule for where a play stopped and whether it counts as seen.

**Library sync and scheduling (`sync.rs`, `schedule.rs`).** Every paged Jellyfin read stops on a short page or a page
with no new id (`page_adds_ids`, a fresh set per query). Jobs: `sync_users`, `sync_libraries`, `sync_events`,
`sync_server`, `sync_userdata`, `sync_changes`, `backup`, `geoip`, Pipeline syncs. Triggers per job (daily, weekly,
interval, start-up, after Jellyfin's library scan), chosen by the pure `schedule::due`; last runs in `task_runs`.
**`trustworthy_removal`** refuses a read (a library's items, the users, a person's played/favourite flags) that comes back empty or catastrophically shrunk: data kept, task failed,
process halts (exit 70); `FINSTATS_ALLOW_LIBRARY_SHRINK=1` overrides. After a read, `backfill_playbacks` fills library
ids / runtimes / S·E numbers, runs `relink`, `pipeline::link` and `watchlist::resolve`; health and capacity recompute
beside it, never inside it.

**Relink and locate (`relink.rs`, `locate.rs`).** Jellyfin ids come from file paths, so a rename orphans plays.
`relink_orphans` re-attaches them by: owner's `located` choice, provider id, exact/original name, cleaned name, episodes by
series + S/E; only when unambiguous. Every move goes through `move_plays`, which re-applies the duplicate rule. `locate`
is the owner choosing by hand (fuzzy candidates).

**Imports.** Jellystat (`import.rs`: `ActivityDateInserted` is the *end*), Streamystats (`streamystats.rs`: one-moment rows
are ends; `isInferred` rows are not plays), Tautulli (`tautulli.rs`: upload, then a wiring board maps Plex users to
Jellyfin users; titles get stand-in `plex:` ids that relink attaches later). All: one IMMEDIATE transaction,
`insert_imported`, `import::finalize`, `groups::detect`; one history writer at a time, its slot held by `api::Running` (given
back as "interrupted" if the request is dropped); a row at most `import::ROW_MAX` (8 MB, `RowBudget`); no play before 1990 or
past tomorrow. Uploads stop at `api::UPLOAD_MAX`
(4 GB), every line at `import::LINE_MAX` (64 MB), the older Jellystat document is streamed (`legacy_rows`). A Tautulli
backup is opened defensively (`SQLITE_DBCONFIG_DEFENSIVE`, no trusted schema) and every table it reads must be a plain
table (`tautulli::READ`) without a generated or hidden column, it is read whole before the write lock is taken, its zip entry
is inflated no further than its size (`inflate`), and one upload arrives at a time, each in a part of its own, swept by prefix.
It is opened `immutable=1` (no `-wal`/`-shm` left), refused when its ids repeat, and its text read to 1,000 characters.
Season, episode and end numbers from every producer are kept only within 0 to `i32::MAX` (`playback::bounded_index`); an
imported play's start and end are both bounded and in order. An upload cut off half way takes its file with it.

**Stats (`stats.rs`).** See the pattern in §9.2. Also playback insights (where viewers stop), concurrency, splits,
search (`fuzzy.rs`, scored in Rust).

**Groups (`groups.rs`).** Plays of one item by ≥ 2 users starting within `group_window_s` of each other's own start (never
chained start to start), or ending within it at nearly the same measured place (SyncPlay with a late joiner), and
overlapping ≥ 2 min share `group_id`; somebody with more than one play overlapping a group is left out of it (`RULE` 4);
clustered without the write lock, written under it. Without `see_everyone` a group is only the people the caller's play met
(`met_by`, `GROUP_SIZE_MET`). `mark_live` does the same for streams playing now; somebody without `see_everyone` is told
only of companions who started with them, and a stream following one of the same title by the same person names nobody.

**Recap and story (`recap.rs`, `story.rs`).** One person's year (or the server's); story cards drawn from the typed
`StoryYear`, which has no field for another person's name, a rank or an app. No rank without `see_everyone`.

**Profile, timeline, recent, watchlist.** `profile.rs` (show progress; seen = play ≥ 80 %, Jellyfin's flag, or a manual
mark), `timeline.rs` (folded stops, `(started_at, id)` cursor), `recent.rs` (episodes of one show on one day fold),
`watchlist.rs` (caller-only, state computed at read time; copies of one title are one).

**Public profiles (`public.rs`, `card.rs`).** `/u/:token` without an account. Built from the owner's id alone, typed
structs, only plays that ended before the last local midnight a day ago (so a page changes once a day), uniform 404. PNG
cards via resvg: the owner's own two at a time, eight waiting at most; published cards drawn in en-US whoever asks, in a lane
of their own (one at a time, own cache and queue). An answer is kept a minute per link and language (`public::Answers`, worked
out once while concurrent requests wait), keyed by what the owner shows and a fingerprint of the plays it counts (kept 5 s,
so a change shows within seconds). Posters only through `servable_images` (`listed_images` ∩ `artwork::known`).

**Library health and capacity (`health.rs`, `capacity.rs`).** Pure rules over item rows (gaps, drift, thin bitrates,
dubs, unidentified, copies/versions; disks, runway, transcodes) with named-constant thresholds; findings stored after
library reads; dismissals hold while the fingerprint matches. A disk's growth counts a file that took a removed one's place
(same title and library, `take_places`, within `REPLACED_WITHIN_S` of the removed row's last `updated_at`, or any time
when an import wrote it with 0) as the difference only. Sums use checked or saturating arithmetic. Without `see_everyone`,
Capacity's busiest moments are days, and only of at least three streams.

**Network, geo, security (`network.rs`, `geo.rs`, `security.rs`).** Local = private range or a home address; always via
`network::classify`. GeoIP maps a private copy of a MaxMind-format city DB (opt-in DB-IP download). Pure `detect()` finds
impossible travel and new countries; `bursts()` failed sign-ins, per account (the whole name typed) and 15-minute window:
one warning, one alert at twenty. The activity log is read from its newest row not in the future.

**Pipeline (`services.rs`, `arr.rs`, `seerr.rs`, `downloads.rs`, `pipeline.rs`).** GET-only clients that follow **no
redirect** (`X-Api-Key` would follow). `Service` is neither `Debug` nor `Serialize`. Download clients are deliberately
not connected; queues are read from Sonarr/Radarr. Every upstream body is read through `services::body_at_most`, capped
after unpacking (16 MB for a service's JSON, 8 MB for a queue, request or history page, 256 MB for Jellyfin's, 20 MB for a
picture, 1 GB / 2 GB for the GeoIP file), and a service's JSON is parsed within `services::Limits` (lists cut, too many values
refused). Without admin, a queue's problem is a kind of failure (`server.downloads.problem.*`), never the service's words;
a key in those words is blanked by its shape as well as its name. Download row errors also hide URL hosts, ports and
paths from everyone except a Jellyfin administrator.

**Notifications (`notify.rs`, `channels.rs`, `mail.rs`).** Events written once (`dedupe`), one delivery per destination
with backoff (30 s, 2 min, 10 min, 1 h, then given up), sent side by side (four destinations at once, 5 s a message, 30 s a
letter; three timeouts in a row pause a destination ten minutes). `wanted_by` is the whole permission rule, checked at raise and send
time (a job's or a connection's failure needs `manage` too). `message()` reads the `private` bag (addresses) only with the
per-destination switch. A stored secret goes only to the scheme, host and port it was saved for; destinations are made or
changed only by a session; deliveries go the most severe first; Discord and Slack need `https`; an administrator never sees
another person's topic or user key nor changes its certificate checks; text is escaped before it is cut. A personal
destination may not point at the household's public address either, and is
sent through `services::Http::public` (its resolver refuses any non-public answer, no proxy) and mail connects to the
address that was checked (`mail::Account::connect_to`), so a name cannot be checked at one address and reached at another. Channels: Discord, Telegram,
Slack, email, ntfy, Gotify, Pushover, Pushbullet, webhook.

**Outbound registry (`outbound.rs`).** Every destination FinStats can reach. A new outbound destination must be added
there and to the privacy sentences in `README.md` and the docs' `privacy.md` / `security.md` in the same change.

**Live channel (`live.rs`, `live.js`).** One WebSocket per tab; topics `now_playing`, `downloads`, `summary`, each frame
built by the same function its GET uses, per person, pushed only on change. Permissions re-checked every 30 s and on
`Feed::recheck` (everybody), `recheck_person` or `recheck_key` (close 4401); 20 per person, oldest closed with 4429; a frame
at most 4 KB. The fallback is the old polling.

**Trash (`trash.rs`).** Deleting a play or a backup moves it to the trash for 30 days; `purge` runs on housekeeping.

**Smaller modules.** `upgrade.rs` (one-off data steps beyond migrations, recorded in `upgrade_steps`), `changelog.rs`
(Patch notes, per language), `licenses.rs` (shipped licence notice), `finui.rs` / `finmotion.rs` / `appearance.rs`
(assembled stylesheets, per-person preset), `browse.rs` (a library's titles grid), `access.rs` (library access from Jellyfin's
policy, `Access`, `visible_items`), `artwork.rs` (image cache names and ETags by image tag, `known`, `may_show`; one fetch per
missing picture, a failure remembered 30 s), `ical.rs` (hand-written RFC 5545 feed), `welcome.rs` (who has had the welcome), `keys.rs`, `audit.rs`, `i18n.rs`.

---

## 8. Frontend

### 8.1 Boot

1. `theme.js` (classic, blocking, in `<head>`): sets `data-theme`, `data-layout="simple"` (+ writes Simple mode's
   stylesheets), `<html lang>` and preloads the language catalogue, all from `localStorage`.
2. `main.js`: `await loadLanguage()`, then imports `simple/app.js` or `app.js` (FinStats layout).
3. `app.js` / `simple/app.js`: register routes `route(path, page, { perm?, prefetch?, bare? })`, set the layout shell.
4. `boot.js` (shared): 401 handler, `/status`, `/auth/me`, prefetch warm-up, router start.

### 8.2 Core modules (`web/assets/js/`)

| Module | Role |
|---|---|
| `dom.js` | `h()` element builder (from FinUI), icons, all formatters (durations, bytes, dates via `Intl`), `safeHttps`, `store` |
| `api.js` | `api.get/post/put/del` (prefix `/api`, throw `ApiError`), `soft()`, view cache (cleared by any non-GET), `Accept-Language` |
| `router.js` | History-API router; guard (setup / login / perm); `ctx = {params, query, signal, root, onCleanup, every, title}`; refuses a parameter that decodes to a separator, `.`, `..`, `?` or `#` (loaders encode parameters into API paths) |
| `state.js` | `state.user`, `can(perm)`, ranges, all `finstats.*` localStorage keys (theme, layout, language, menus, …; `welcomed()` is the server's `user.welcomed`; the date and time formats are `dom.js`'s) |
| `components.js` | `dataView()` (stale-while-revalidate page loader), `setBusy`, `openModal`, filters, plays table, FinUI re-exports |
| `tables.js` | Re-export of FinUI data-table (`dataTable`, `plainTable`, `chartTable`, `sortable`) |
| `charts.js` | Hand-rolled SVG charts; colours by entity (`TYPES`), as `'var(--series-n)'` |
| `widgets.js` | Larger shared cards (activity, heatmap, now playing, insights, …) |
| `live.js` | `follow(topic, {poll, every, onData, signal})`: WebSocket with polling fallback |
| `when.js` | Pure: a date or time in a chosen format (`writeDate`, `writeTime`); `dom.js`'s `clockTime` / `calendarDate` use it |
| `welcome.js` | The welcome (`/welcome`) and the same card at the end of setup: language, formats, colours, layout; Skip keeps `DEFAULTS`; `finstatsWelcome()` in the console (`boot.js`) opens it again |
| `nowplaying.js` | Now-playing view with a 1 s ticker clock (polls only correct, never repaint) |
| `prefetch.js` | Warm-up and on-intent prefetch, calling the page's own loaders |
| `shell.js`, `sections.js`, `menus.js`, `search.js` | Layout chrome (FinUI menus, status bar, Esc), section pages, context menus, search |
| `i18n.js` | `t(key, vars)`, `tParts`, `tOr`, `lang()`, `LANGUAGES` (24, five `checked`), `pickLanguage` (nothing it speaks, or a bare `en`, is en-US), `loadLanguage` (with en-GB beside an unchecked language), `setLanguage` |
| Feature helpers | `watchlist.js`, `worldmap.js`, `notifications.js`, `connections.js`, `downloads.js`, `upcoming.js`, `playmodal.js`, `changelog-text.js` |
| `simple/*` | Simple mode shell, helpers (`platform.js`, `profile.js`, `history.js`) and pages |
| `public/profile.js` | Public page; no state, api.js, router or prefetch |

### 8.3 Pages

FinStats layout: `/` dashboard, `/recap`, `/activity`, `/together`, `/users` (`see_everyone`), `/users/:id` (+
`/timeline`, `/watchlist`), `/libraries` (+ `/:id`, `/:id/titles`), `/items/:id`, `/people/:id`, `/playback`,
`/pipeline`, `/server/:section` (`see_server`: overview, jobs, devices, plugins, capacity, health, log, audit),
`/security` (`see_network`), `/settings/:section` (+ `/settings/tasks/:task`), `/changelog`, `/licenses`, `/setup`,
`/login`, `/welcome`. Settings sections live in `settings/*.js` and export
`{ key, label, sub, group, icon, visible, entries, render(slot, store) }`.

Simple mode: `/`, `/history`, `/libraries(/:id)`, `/users(/:id)`, `/items/:id`, `/graphs`, plus shared settings, sign-in,
licences and its own changelog, loaded lazily. It never loads the FinStats layout's pages, shell or FinMotion.

### 8.4 Shape of a page module

```js
const loadPerson = (id, days, signal) => api.get(`/people/${id}`, { days }, { signal });
export const prefetchPerson = ({ params, query, signal }) => [() => loadPerson(params.id, readDays(query), signal)];

export default function personPage(ctx) {
  const view = h('div', { class: 'stack' });
  const dv = dataView({ container: view, signal: ctx.signal,
    skeleton: () => [sk.tiles(3)],
    fetch: () => loadPerson(ctx.params.id, days, ctx.signal),      // GETs fired synchronously = cache key
    render: (d) => [ /* cards; null for a card whose data is missing */ ] });
  ctx.root.append(view);
  dv.load();
}
// app.js: route('/people/:id', personPage, { prefetch: prefetchPerson });
```

### 8.5 FinUI and FinMotion

- **FinUI** (`web/assets/finui/`): `registry.json` lists foundation files (`tokens.css`, `base.css`, `core.js`,
  `format.js`, `i18n.js`; `core.js`'s `h()`/`s()` set an address only when it is http(s) or relative, never `srcdoc`, a change
  that must stay in step with `../finui`, where it is already) and components (`components/<name>/` with `<name>.js` exporting a builder + `meta`,
  `<name>.css` styling only `fui-<name>`, often a pure `plan.js`). The server concatenates every CSS file in registry
  order into `/assets/finui.css`, then appends the person's preset overlay (`create/presets.json` axes; token files
  `p/<axis>/<option>.css`; a preset code is one base-36 digit per axis).
- **FinMotion** (`web/assets/finmotion/`): springs (`--spring-settle|snap|drift|glide`) paced by FinUI's Motion
  choice; parts that enhance FinUI components by their classes; `motion(root)` called once. Served as `/assets/finmotion.css`.
- **Both are developed here and copied to their own repositories** (`../finui`, `../finmotion`) in the same sitting; QA
  fails while the copies differ. FinUI imports nothing from outside `finui/`; FinMotion imports nothing from FinUI.

### 8.6 CSS

Load order: `finui.css` (tokens + components + preset), `finmotion.css`, `app.css`, then in Simple mode
`simple-tokens.css` and `simple.css`. Every colour is a FinUI token defined as `light-dark(light, dark)` in
`finui/tokens.css`; a colour literal anywhere else fails `cargo test`.

---

## 9. Patterns and conventions (read before writing code)

### 9.1 Backend rules

1. **Test first** (Red, Green, Refactor). No production behaviour without a test that asked for it. A bug fix starts
   with a test that reproduces it. Unit tests sit in `#[cfg(test)] mod tests` beside the code; behaviour visible only from
   outside gets a check in `qa/` first.
2. **Migrations are append-only.** Add an entry to `db::MIGRATIONS`; never edit or reorder a released one (a SHA-256 over
   the released prefix is pinned). One-off data fixes go in `upgrade::STEPS`.
3. **Read history from `visible_playbacks`**, not `playbacks` (unless it is an identity read, listed in `IDENTITY_READS`).
4. **Every transaction is IMMEDIATE** (pool init). A read-only snapshot asks for `Deferred` by name.
5. **Import rusqlite as `db::rusqlite`.**
6. **Settings change only through `AppState::update_settings`.**
7. **Library reads go through the caller's `Access`** (`Scope.access`; `access.rs`): a title or library Jellyfin hides is the
   same 404 as an unknown one. History is not narrowed by it.
8. **A read never wipes what it cannot see.** Removals go through `trustworthy_removal` / `whole_set_vanished`; count
   what was stored, not what was returned.
9. **Anything that rewrites `playbacks.item_id` re-applies the duplicate rule** (`relink::move_plays`). Seen live is
   `source = 'live' AND source_id IS NULL`, never `source` alone.
10. **Sticky values go above a diff** (e.g. "once a transcode, always a transcode" before `diff_events`).
11. **Every `/Sessions` read goes through the gate**, and a timed read re-arms the clock that called for it.
12. **Pure where possible**: decisions as pure functions with tests; DB and network at the edges. Sums over numbers from
    Jellyfin or a file are checked or saturating (release builds check for overflow).
13. **Secrets never serialise.** Types holding them (`Service`, `notify::Target`) are neither `Debug` nor `Serialize`;
    JSON is built by hand with `has_secret`; errors name a status and a kind, never an upstream body.
14. **New outbound destination** → `outbound.rs` + README / docs privacy sentences, same change.
15. **New table holding something Jellyfin cannot give back** → `backup::TABLES`.
16. **New notification kind** → a `Kind` arm and one `raise`; **new channel** → a `Channel` arm, a `payload` arm, where its
    token goes (and `notify::clean_url` / `clean_topic` / `Target::shown` if it has special host or field rules).
17. **New importer** → `PlayRecord::insert_imported`, one IMMEDIATE transaction, `import::finalize`, task id in
    `HISTORY_WRITERS`, docs page in `../docs`.
18. **Test mocks must answer the real query** (a mock that ignored `IncludeItemTypes` hid a halt-loop bug).
19. **Do not run `cargo fmt`.** The code is deliberately not rustfmt-formatted; match the surrounding style by hand.
20. **Clippy:** add no new warnings.
21. **Dependencies** must be GPL-3.0-compatible; re-run `tools/make-third-party.py` after any change to `Cargo.lock`.

### 9.2 Adding a statistic

```rust
pub async fn my_stat(State(app): State<App>, user: AuthUser, Query(q): Query<FilterQuery>) -> ApiResult {
    Ok(Json(scoped(&app, &user, &q, |c, scope| {
        let cond = scope.cond().with_raw("p.item_type = 'Movie'");   // users pinned by permission, window, min_play_s
        Ok(json!({ "rows": rows_json(c, &format!(
            "SELECT … FROM visible_playbacks p {} GROUP BY … ORDER BY {}",
            cond.sql(), order_by(&SORTS, sort, dir, "watch_s DESC")), &cond.args)? }))
    }).await?))
}
// Several independent parts: resolve once, then run up to five side by side.
let scope = resolved(&app, &user, &q).await?;
let (a, b) = tokio::try_join!(apart(app.clone(), scope.clone(), |c, s| totals(c, &s.cond())),
                              apart(app.clone(), scope.clone(), |c, s| daily(c, s, &s.cond())))?;
```

- "Last N days" = N full local days; bucket in SQL with `'localtime'`; in Rust use `LocalDays`.
- `row_json` maps columns by name; integer columns in `BOOL_COLS` become booleans, text in `JSON_COLS` is parsed.
- Blank `remote_ip`/devices without `see_network`, paths without `see_server`.
- At most five concurrent queries per page (pool of 6; the collector keeps one).

### 9.3 Frontend rules

1. API and user strings reach the DOM only through `h()` / `textContent`; never `innerHTML` with data. A route parameter put
   into an API path is encoded.
2. Pass possibly-absent nodes through `h()` (native `append(null)` prints "null").
3. Everything a page starts dies with `ctx.signal` (fetches, `follow`, listeners on `window`/`document`, timers via
   `ctx.every`). `stickyFilters` throws without a signal.
4. Each page exports `prefetchX(ctx)` beside the loader its `dataView` uses; loaders fire GETs synchronously.
5. Every card hides itself when its data is missing (older servers, non-admins, imported plays).
6. Tables go through `tables.js`; paginated lists sort on the server (`server: {key, dir, onSort}`).
7. No inline styles (CSP); `el.style.x` from JS is fine. Every colour is a token, reached as `'var(--x)'`.
8. Live data uses `follow()`, never a bare `setInterval`.
9. Overlays stop propagation of their own Esc; `shell.js` handles Esc globally.
10. Every word a person reads comes from `t('key')`. A new key lands in the four checked catalogues (en-GB, en-US, nb-NO,
    nn-NO) in the same change; the other nineteen say it in en-GB until translated (see `DECISIONS.md`, Languages).
11. Content stays visible: no folds; use navigation and jump lists instead.

### 9.4 Writing rules (all repositories)

- **No em-dash anywhere** (code, comments, UI, docs, changelog, commit messages). Rewrite the sentence; no substitute
  dash. Only `THIRD-PARTY.json` is exempt. A QA check enforces it.
- Tests, docs, examples and commit messages use invented data only (alice, bob, Big Buck Bunny, `192.168.1.10`).
- Commits: conventional subjects with a scope (`feat(recap):`, `fix(import):`); body says why; one logical change each;
  every commit touching Rust builds and passes `cargo test`; **no LLM attribution lines**; stage explicit paths; never
  push without being asked (a pushed `v*` tag publishes images and a GitHub release).
- Documentation pages belong in `../docs` (a change that a page describes gets a docs commit in the same sitting);
  unreleased pages wait on the docs branch named like this one (`dev-2.3.0`).

---

## 10. Commands

```sh
# Build and run
cargo build --release                                   # embeds web/, CHANGELOG.md, changelog/*.md
FINSTATS_DATA_DIR=./data FINSTATS_BIND=127.0.0.1:8088 cargo run        # serve (debug reads web/ from disk)
cargo run -- import-jellystat <backup.jsonl>
cargo run -- import-streamystats <backup.json>
cargo run -- backup | restore <file> [--with-settings] | relink | --version | --help   # restore: history only by default

# Tests and checks
cargo test                                              # all unit tests (~0.5 s; CI runs cargo test --locked)
cargo test relink                                       # tests whose path matches
cargo test recap::tests::persona_prefers_the_most_distinctive_habit -- --exact
cargo clippy --all-targets                              # warning budget in qa/baseline.json; add none
for f in $(find web/assets/js -name "*.js"); do node --check "$f"; done   # the only JS check

# Local release gate (git-ignored qa/)
qa/run.sh                                               # everyday gate (~70 s)
qa/run.sh endurance                                     # torture + soak, before a release
qa/run.sh <stage>                                       # static, api, browser, live, socket, docker, finui, finmotion, i18n, coverage, screenshots, serve, …

# Generated files
cargo fetch && python3 tools/make-third-party.py        # after any dependency change
python3 tools/make-languages.py                         # after a catalogue, changelog translation or language changes (--check: QA)
python3 tools/make-world-map.py <ne_50m.geojson> web/assets/geo/world.json
node ../finui/tools/build-site.mjs <dir>                # then copy p/, create/presets.json, fonts/ into web/assets/finui

# Docker
docker build -t finstats:latest .
```

The owner's own install runs as a container named `finstats` built from this checkout; finishing a change means
rebuilding the image and swapping the container (read its run arguments from `docker inspect`, never from memory), then
checking `/api/status`, the log and the health state. A command inside the container runs as the data folder's owner
(`docker exec -u <PUID> finstats finstats <command>`): FinStats refuses to run as root over a folder another user owns.

---

## 11. Testing strategy

### 11.1 Unit tests (`cargo test`)

- Live in `#[cfg(test)] mod tests { use super::*; … }` at the end of each module; no `tests/` folder, no fixture files.
- Test names are behaviour sentences (`a_play_in_the_trash_is_out_of_history_and_still_there_for_identity`), often with
  a doc comment stating the rule held.
- **Databases:** `db::Db::open_in_memory()` (pool of 1, all migrations) and `state::test_app()` (a full `AppState` over it,
  for `#[tokio::test]` handler tests). Some tests run `MIGRATIONS` on a raw in-memory connection. Fixtures are local
  helpers inserting rows with SQL literals and invented data.
- **Recurring patterns:**
  - `EXPLAIN QUERY PLAN` assertions pin index use (dedupe ranges, `recent`'s `+i.type`, trash purge, watchlist,
    pipeline, browse, health, stats rows that must not scan `playbacks`).
  - Repository-wide scans: reads of `playbacks` vs the trash; FinUI colour literals, radii, class ownership, token lists;
    `card.rs` colours vs dark tokens; licence notice vs `Cargo.lock` and fonts; every catalogue's keys, placeholders and
    plurals vs en-GB; changelog translations vs `CHANGELOG.md`; released migrations' hash.
  - Pure simulations (the collector walking minutes of simulated time in microseconds).
  - Contract tests between modules (e.g. health's copy groups vs `watchlist::find_title`).
- The changelog test requires the top `CHANGELOG.md` entry to equal the `Cargo.toml` version.

### 11.2 The QA suite (`qa/`, local and private)

Git-ignored and kept in its own private repository (address only in `qa/README.md`). It builds a release binary and
Rust tools (`qa/tool`: `qa-seed` invented data, `qa-mock` Jellyfin / breakable Jellyfin / arr / notify sinks,
`qa-check` static rules, `qa-load`), starts throwaway instances on fixed ports, and runs stages:

| Stage | Covers |
|---|---|
| `static` | build, unit tests, clippy budget, JS parse/imports, no inline style / innerHTML / em-dash, versions vs tags, routes in `api.md`, privacy scan, commit hygiene |
| `api` | every endpoint: shapes, permissions (auth-boundary sweep three ways), hostile input, headers, time budgets |
| `escalation` | a non-admin trying to become one |
| `browser` | every page at 1440 and 390 px, as admin and as a user without permissions, plus journeys (puppeteer-core) |
| `live`, `socket` | the browser WebSocket; collector transport scenarios against a misbehaving mock |
| `badjellyfin` | wrong Jellyfin data keeps the DB and halts with 70 |
| `firstrun`, `cli`, `geo`, `docker` | wizard, CLI, GeoIP, the image on fresh folders like a stranger's machine |
| `finui`, `finmotion` | those repos' QA, and that the copies here match |
| `endurance` | `kill -9` torture, full disk, damaged DBs, soak for leaks |
| `coverage` | instrumented Rust + browser JS coverage, ratcheted in `qa/baseline.json` |
| `screenshots` | retakes the docs' screenshots from showcase data |

Never commit `qa/`. Add a check there for every bug fixed that unit tests cannot reach.

---

## 12. Environment variables

| Variable | Default | Meaning |
|---|---|---|
| `FINSTATS_DATA_DIR` | `data` (image: `/data`) | Database, caches, backups, uploads, geoip |
| `FINSTATS_BIND` | `0.0.0.0:8080` | Listen address |
| `FINSTATS_TRUST_PROXY` | off | The proxies (addresses or CIDR ranges, comma or space separated) whose `X-Forwarded-For`, `-Host` and `-Proto` are believed, read from the right past them; `1`/`true` = a loopback or private-network peer, one hop; an unreadable list stops the start |
| `FINSTATS_GEOIP_DB` | newest `.mmdb` in `<data>/geoip` | Fixed city database (also disables the download job) |
| `FINSTATS_PUBLIC_IP_URL` | built-in services | Plain-text public-IP lookup URL |
| `FINSTATS_ALLOW_LIBRARY_SHRINK` | off | Let an emptying/shrinking library read through (clears a halt loop) |
| `FINSTATS_SKIP_PREUPDATE_BACKUP` | off | Skip the `VACUUM INTO` snapshot before an upgrade |
| `JELLYFIN_URL` + `JELLYFIN_API_KEY` | unset | Configure Jellyfin without the wizard (both or neither) |
| `TZ` | process local | Local-day bucketing (the image ships tzdata) |
| `RUST_LOG` | `finstats=info` | Log filter |
| `PUID` / `PGID` | 1000 / 1000 | Container user (entrypoint only; it sets `umask 077` and changes only the data folder and FinStats' own entries) |

`main.rs`'s `USAGE` text must name every variable FinStats reads (a test checks literal `env::var("…")` calls).

---

## 13. Releases

- `CHANGELOG.md`: `## [x.y.z] - YYYY-MM-DD` (or `- first to last`), an `x.y.0` opens with a short title line, groups
  `### Added | Changed | Security | Performance | Stability | Fixed | Removed`, one-line bullets (`**bold**`, `*italic*`, `` `code` ``;
  no links). **Every entry must also be added to `changelog/en-US.md`, `nb-NO.md`, `nn-NO.md`** with the same groups,
  bullet counts and code spans, or `cargo test` fails. The other nineteen `changelog/<tag>.md` may follow within the
  series, but `qa/run.sh i18n` fails until every one tells every release the source does, so none ships behind.
- History shape: logical commits (backend before UI, `fix(...)` on their own, then `docs:`), then
  `chore(release): x.y.z` bumping `Cargo.toml` + `Cargo.lock`, then an annotated tag `vX.Y.Z`.
- CI (`.github/workflows/docker.yml`): on push to `main`, tags, PRs. `test` (cargo test, node --check, tag vs version vs
  changelog), `build` (amd64 + arm64 natively), `publish` (`:edge` from main; `:X.Y.Z`, `:X.Y`, `:X`, `:latest` from a
  tag), `release` (GitHub release from that changelog section).
- Bug fixes after an `x.y.0` get their own `x.y.1`; otherwise the owner prefers larger releases.

---

## 14. Known gaps and constraints

**Constraints that are deliberate**

- FinStats never writes to Jellyfin (except creating its API key) and never connects download clients.
- No frontend build step or JS dependencies; the only JS check is `node --check`.
- No HTTP/2; 2 tokio worker threads; a 6-connection pool. Heavy work belongs in `db.call` / `spawn_blocking`.
- The code is not rustfmt-formatted, on purpose.
- `/api/status` is unauthenticated by the owner's decision (shape only, plus the server's name and version).
- Jellyfin's `/socket` cannot be asked for PascalCase; `socket.rs` re-cases keys and checks the first push against `/Sessions`.
- Jellyfin's activity log carries no playback errors; broken files are inferred from plays.
- Releases up to 1.0.4 predate the downgrade guard and cannot be stopped by it.

**Open issues found while writing this file (2026-10-12)**

- **Nineteen of the 23 languages are unchecked translations.** Only en-GB, en-US, nb-NO and nn-NO are read by the owner;
  the README, the docs and Settings → Appearance → Language say so. A person's own Recap cards draw Japanese, Korean,
  Chinese and Hindi readers' cards in en-GB, because the card font (Inter) has no CJK or Devanagari letters.
- **Notification messages stay English** (they are sent outside any request, to external services), and stored texts
  written by scheduled runs (a task's error) are in en-GB.
- **CI's `node --check` covers only `web/assets/js/*.js` and `pages/*.js`**, not `settings/`, `simple/`, `public/`,
  FinUI or FinMotion (the QA static stage covers everything).
- Libraries (unlike users) are still marked removed by timestamp rather than by id.

**Open findings from the 2.3.0 bug hunt, left for 2.3.1 (2026-10-26)**

- `capacity::recompute` reads the whole play history each time it runs (after every read of the server's details), where
  only the plays since its last run can change the busiest moments.
- `capacity::busiest_of` joins two evenings into one spell while a play is stuck active between them.
- The live channel: a tab's own drift can hide behind a reset of the shared course. (A key revoked, a sign-in or a
  sign-out now rechecks only the connections it concerns, and a topic taken away at a recheck comes back on the same
  connection when it is allowed again.)

**Left with the owner after the 2.3.1 security passes (2026-10-28)**

- **No `Host` allowlist against DNS rebinding.** A page on a name the attacker controls, re-pointed at FinStats' address,
  is same-origin to itself; FinStats does not refuse unknown `Host` headers, because a default allowlist would break LAN
  hostnames, bare IPs and proxies. It could be an opt-in (`FINSTATS_ALLOWED_HOSTS`), which the owner may want.
- **No per-name sign-in limit below Jellyfin's lockout.** Jellyfin's failure counter resets only on a successful sign-in, so
  a window in FinStats would delay a lockout without preventing it, and would let anyone keep a person out of FinStats by
  typing their name. The per-address, per-/48 and server-wide limits and `may_try` stand instead.
- **Published cards are always drawn in en-US** (`i18n::DEFAULT`), not in the owner's language: FinStats stores no language
  for a person, and keeping one would need a migration.
- On plain http every other web service on the same host or address, on any port, receives the session cookie; only the
  `__Host-` cookie on https is FinStats' alone (the docs say to serve it on its own hostname over https).

---

## 15. Keeping this file current

Update `CONTEXT.md` in the same change whenever you add a subsystem, table, route group, environment variable, command
or convention, or fix one of the gaps in §14. Bump the **Last updated** line. Keep it a map: link to `ARCHITECTURE.md`, `DECISIONS.md`, the
code and the docs for detail rather than copying them, and do not list how many files a folder holds.
