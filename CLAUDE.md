# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

finstats is a playback-statistics server for Jellyfin: one Rust binary (axum + tokio + bundled SQLite)
with the web UI compiled into it. It only ever *reads* from Jellyfin; it never triggers scans or writes
anything there except creating its own API key during setup.

## How we work: TDD

**From now on, every change in this repository is written test-first, following the Red–Green–Refactor
cycle (Martin Fowler).** No production code is written before a failing test asks for it.

1. **Red** — write one small test for the next slice of behaviour, and run it. It must *fail*, for the
   right reason: the behaviour does not exist yet. A test that passes the moment it is written proves
   nothing — go back and make it demand something real.
2. **Green** — write the least code that makes that test pass. Do not reach for clean design yet; the
   only goal is a green bar. Run the tests.
3. **Refactor** — with the tests green, clean up what you just wrote (both the code and the test): remove
   duplication, improve names, simplify. Run the whole suite after each step; it must stay green.

Then loop: the next test drives the next slice. Keep the steps small — minutes, not hours — so a red bar
always points at the last thing you changed.

Practically here: unit tests are `#[cfg(test)]` modules next to the code (`cargo test`, ~0.5 s); pure
logic (a guard, a parser, a permission rule) is tested there first. Behaviour only observable from
outside — an endpoint, an auth boundary, the collector on a real socket, wrong data from Jellyfin — gets
its failing check in the local `qa/` suite first (see the QA section). A bug fixed is a bug that first
gets a test reproducing it (Red), then the fix (Green). Do not add production behaviour that no test
named, and do not delete a test to make the bar green.

## Commands

```sh
cargo build --release                      # release binary embeds web/ and CHANGELOG.md
cargo test                                 # all unit tests
cargo test relink                          # tests whose name/module matches "relink"
cargo test recap::tests::persona_prefers_the_most_distinctive_habit -- --exact
cargo clippy --all-targets                 # not clean: ~12 style warnings exist; add no new ones

FINSTATS_DATA_DIR=./data FINSTATS_BIND=127.0.0.1:8088 cargo run     # serve
cargo run -- import-jellystat <backup.jsonl>                         # headless import, then exit
cargo run -- import-streamystats <backup.json>                       # the other tracker, same thing
cargo run -- relink                                                  # re-attach history to renamed items, then exit

for f in $(find web/assets/js -name "*.js"); do node --check "$f"; done   # the only JS check there is
docker build -t finstats:latest .        # local image; the published one is ghcr.io/finstats/finstats
```

- **Do not run `cargo fmt`.** The code is deliberately not rustfmt-formatted (hundreds of long lines); formatting
  would rewrite every file. Match the surrounding style by hand.
- Debug builds read `web/` from disk at runtime (rust-embed), so UI edits only need a browser refresh.
  Release builds embed it — rebuild to see UI changes. `CHANGELOG.md` is `include_str!`'d, so it always needs a rebuild.
- Env: `FINSTATS_DATA_DIR`, `FINSTATS_BIND`, `FINSTATS_TRUST_PROXY`, `FINSTATS_PUBLIC_IP_URL`, `FINSTATS_GEOIP_DB`, `FINSTATS_ALLOW_LIBRARY_SHRINK`, `FINSTATS_SKIP_PREUPDATE_BACKUP`, `JELLYFIN_URL` + `JELLYFIN_API_KEY` (skip the wizard), `TZ`, `RUST_LOG`.

## Architecture

```
Jellyfin ──/socket push, else /Sessions 1 s / 5 s idle─▶ collector ──▶ SQLite ◀── stats / recap API ◀── embedded SPA (web/)
         ──/Users /Items /Devices /System/* (scheduler)──▶ sync ──┘        ▲
Jellystat / Streamystats backup ──▶ import ──────────────────────┘   relink (after sync/import/start-up)
Sonarr/Radarr ──calendar, history (15 min)──┐
Seerr ──requests (5 min, one row if quiet)──┼──▶ SQLite ──▶ pipeline API ◀── /pipeline
Sonarr/Radarr queues ───────────────────────┴──▶ in-memory snapshot (5 s watched / 60 s busy / 5 min empty) ──▶ /api/downloads
```

**State & DB access.** `state.rs` holds `AppState` (shared via `Arc` as `App`): DB handle, Jellyfin config,
`Settings` (one JSON blob in the `settings` table, `#[serde(default)]` so old installs load), the in-memory task
registry, the live now-playing snapshot, and a `Notify` (`wake`) that background loops select on. All DB work goes
through `db.call(|conn| …)` (r2d2 pool + `spawn_blocking`). A page whose queries over the history need nothing from each
other runs them side by side (`stats::resolved` once, then `apart` per part under `tokio::try_join!`): each is one
core's work in SQLite, and in sequence the page takes their sum — at most five at once, so the collector keeps a
connection of the six; rusqlite is re-exported as `db::rusqlite` — import it from
there, not from the crate, to stay on the version `r2d2_sqlite` uses. **Every `transaction()` on a pooled connection is
IMMEDIATE** (set in the pool's init): a deferred one that reads before it writes is refused its write outright once anybody
else commits in between (`SQLITE_BUSY_SNAPSHOT`, which `busy_timeout` does not wait out), and the collector commits all the
time. A transaction that only reads — the backup's snapshot — asks for `Deferred` by name, or it would hold the write lock
for the whole export. `journal_size_limit` (64 MB) is set there too: SQLite never shrinks its WAL on its own. **Before the
pool touches the file, `preflight` looks at it once**: the pool retries a bad file for its 30-second timeout and its first
connection switches the file to WAL (a write), so a damaged file, another program's database (tables, `user_version` 0), a
folder or a file finstats may not write is refused there, in milliseconds, naming it and saying nothing was changed.

**Migrations** are the `MIGRATIONS` array in `db.rs`, applied by index against `PRAGMA user_version`. Released
migrations are immutable — deployed databases have already run them. Add a new entry; never edit or reorder one.
`Db::open` also refuses a database from the future (`refuse_downgrade`): the settings key `app_version` holds the newest
version that has opened it, and a binary older than that, or one with fewer migrations than `user_version`, bails before
writing anything. Releases up to 1.0.4 predate the check and cannot be stopped. The key is not part of backups on purpose.

**A newer binary snapshots the database before it upgrades it (`back_up_before_update`).** On `Db::open`, between
`refuse_downgrade` and the migrations, `is_update` asks whether a populated database is being opened by a *different*
version, or still has migrations to run; if so a full `VACUUM INTO` copy — the library and the secrets included, unlike the
exportable JSON backups — is written to `<data>/pre-update-backups/` **before** anything is changed, so a migration or a new
binary that corrupts data can always be rolled back to (stop finstats, put the copy in place of `finstats.db`, run the old
version). A brand-new database and a same-version restart snapshot nothing. The newest `PRE_UPDATE_KEEP` (3) are kept; the
copies are never served over the API. A failed copy is fatal only when migrations are pending (the risky case) — a plain
version bump warns and continues. `FINSTATS_SKIP_PREUPDATE_BACKUP=1` turns it off. The copy is written to `<name>.db.part`
and renamed when complete, and what a killed start left is swept first: `VACUUM INTO` writes straight into its target, and an
empty file under a snapshot's name is a valid empty database to SQLite that counted toward the three kept.

**Library reads must ask for real items.** `items_page` passes `CollapseBoxSetItems=false` (otherwise servers with
"group movies into collections" return the BoxSet *instead of* its films, which then get flagged removed) and
`ExcludeLocationTypes=Virtual` (missing/unaired placeholders). Anything a read does not return is marked `removed`,
so a query that silently hides items is a data-loss bug, not a cosmetic one.

**Track languages.** `items.audio_languages` / `subtitle_languages` are JSON arrays from `media::track_languages` (every track, each code once,
`und` for an untagged one), written by both item producers (`sync::upsert_item`, the import's `jf_item_info`). A series or season has none of its
own: `item_detail` answers `language_coverage` (episodes per language, files only), which is what shows a dub that stops half way. finstats never
claims "dubbed": it does not know a title's original language, so it lists the languages and lets the reader decide. Names come from the browser
(`languageName` in `dom.js`: `Intl.DisplayNames` plus the bibliographic codes it lacks), so no language table is shipped.

**Jellyfin client (`jellyfin.rs`).** Every request asks for `Accept: application/json; profile="PascalCase"` because
10.x servers answer PascalCase and newer ones camelCase by default. All JSON access in the codebase assumes
PascalCase keys. Jellyfin ids are normalised with `db::norm_id` (no dashes, lowercase) everywhere.

**Compression (reqwest's `gzip` + `brotli` features).** Every outbound read asks for `Accept-Encoding: gzip, br` and is
decoded transparently — Jellyfin, Sonarr, Radarr and Seerr all compress JSON when asked, and JSON is nearly everything
finstats reads. The features are the whole mechanism: `default-features = false` without them means no header goes out
and nothing would be decoded, so the QA mocks answer compressed and a check holds the header. Two deliberate exceptions:
`geo.rs` pins `Accept-Encoding: identity` on the `.mmdb.gz` download (a gzip *body* it unpacks itself; the header also
keeps `content_length` honest, and setting it at all turns reqwest's own decoding off), and the WebSocket carries none —
frame compression is `permessage-deflate`, which tungstenite does not implement.

**Two transports, one collector (`collector.rs`, `socket.rs`).** The session list arrives either by asking
(`GET /Sessions` on the `active_interval_s` / `idle_interval_s` timers) or by being told (Jellyfin's `/socket` with
`SessionsStart`); `tick()` cannot tell which and must not learn. **There is no setting**: the socket is spawned for whatever
Jellyfin is configured, always, and being told is how finstats collects — 1.4.0's `live_socket` switch was removed in 1.5.0, so
`active_interval_s` / `idle_interval_s` are only what the asking half asks at, and the fallback for a socket that is not carrying.
**Each does the half it is good at.** `collector::Decide::see` reads one session list — pushed or asked for, it
decides the same — and answers `Mode::Poll` or `Mode::Listen`: **anything running** → poll at `active_interval_s`, because a pause or a
seek is only as sharp as the gap between two sightings and the push carries no `ActiveWithinSeconds`; **nothing loaded anywhere** →
listen; **everything loaded is paused**, for `PAUSE_DEBOUNCE` readings in a row → listen, because a frozen position is exactly what a
server has nothing to push about, and one 1 s poll per second of it asks the same question 3,600 times an hour. Any session running again
(a resume, or somebody new) takes it straight back, within about a second. Three rules hold the transitions together, all measured on the wire: `session_mode` calls the pause debounce
`playing_poll` and keeps `fallback` for a socket that is genuinely unusable (an early attempt reported every pause as a fault for three seconds);
`should_subscribe(playing, settling)` keeps the subscription off for the whole debounce, and the transition itself sends the frame and
waits for `Handle::settled` before publishing, so nothing is ever subscribed and polling at once; and the one-shot read is `Net`/`safety_due` —
after `PAUSED_SAFETY_SILENCE` of silence with something paused, or `SAFETY_EVERY` regardless (an early attempt restarted its wait on every push,
so a client that kept reporting itself while paused meant the net never fell). **Silence is nothing heard *and* nothing asked, and the
attempt re-arms the clock that called for it**: `Net` is reset by a push, by any `/Sessions` read of finstats' own — the beat
during a play included — and by a fresh subscription, with `SAFETY_MIN_GAP` as a floor under all of it. Measured from the last
push alone, as an early attempt did, it breaks: since the subscription is off for the whole of a play, the first pause after a minute of one was already "silent", and the read
that followed reset nothing, so 2,625 of them went out in eight seconds until a push happened along. Anything that makes a repeat depend on
a clock the repeat does not touch is that bug again. The two are **one call**: `Net::take_read` takes the gate slot *and* re-arms both clocks, so a read that
does not restart the wait it answers is not expressible (1.5.2); `Net::read` is private, and `Pass` — one object holding `Decide`, `Net` and
the subscription as the wire has it — is what `run()` goes through, which is also what lets a test walk seven simulated minutes of this in
microseconds (`three_minutes_of_playing_then_a_pause_is_almost_no_reads_at_all`) instead of sitting through two real ones. Under it all,
`sessions_slot` — **every** `/Sessions` read in the process, the socket's own consistency check included, takes a slot from one gate:
`READS_PER_S`, `READS_PER_MIN`, refusals counted and logged WARN at most once a minute. `/api/status` publishes what the gate counted (`sessions_requests_last_min`), and `disagrees` holds a listening mode to
`READS_WHILE_LISTENING`, because every word of the status was true throughout that storm. With no socket at all, paused sessions are polled at
`PAUSED_POLL_S` rather than every second.
`attribute()` holds the invariant the whole thing rests on: the gap between two sightings belongs to the state the play was *already* in,
so however rarely a paused play is looked at, none of it becomes watch time. **One at a time**: `SessionsStart`'s `"0,1500"` asks
Jellyfin to look every 1.5 s and send what differs — which is silence on an idle server and a list every 1.5 s during a play, so while
polling the pushes are the same list a second time, uncompressed. `Handle::listen(false)` sends `SessionsStop` for the duration (the
connection stays open and answering `KeepAlive`); `listen(true)` on the pass where the last play ends. **The rule is
`should_subscribe(playing) = playing == 0`, never the collector's own mode** — and that is the whole of it: listening needs the
socket to have proved itself, the proof is the list a subscription brings, and an early attempt unsubscribed before it could arrive,
so the fallback latched on for good (connected, never subscribed, polling for ever on exactly the idle server the socket is
for). Anything that gates the subscription on something the subscription itself produces is this bug again.
A verdict is never kept either. **An absent list proves nothing**: a Jellyfin normally answers `SessionsStart` at once whether or
not anything is loaded (measured: `0.0s after subscribing, 0 loaded`), but one was once seen not to, cause never established — most
likely a server still starting. So after `SUBSCRIBE_MAX` finstats says so once, the collector polls meanwhile, and the connection is
kept, stays subscribed and re-sends `SessionsStart` every `PROBE_EVERY`; the first real push settles it. An earlier attempt concluded "cannot push"
from that one silence and lost the socket altogether, which is why silence is never a verdict here. `End::Unsupported` is now only for a *shape* mismatch, which is real disproof; every
other end uses the backoff. `looks_like_sessions` answers *what* differed for the
log, and compares only sessions with a `NowPlayingItem` that both lists saw — an app open with nothing playing is in the
push and not in `/Sessions?ActiveWithinSeconds=300`, which says nothing about the socket. **What it is doing is published, not inferred**. `publish()` writes the whole picture — `session_mode`
(`idle_socket` | `playing_poll` | `paused_socket` | `fallback`), `socket_connected`, `socket_subscribed`, `poll_interval_s`,
`mode_since` — in one lock, twice a pass, so nothing can be read half-applied while a pass blocks for a minute on a push. The two wire
facts come from the socket task itself (`Handle::connected`/`subscribed`, atomics set where the frames are actually written), never from
what the collector *asked* for. `disagrees()` is the machine marking its own work: every rule is something a packet capture would
contradict, and a disagreement is logged WARN only once it outlives `SETTLE`, because asking the socket to stop and the stop reaching the
wire are two moments. All of it is on `GET /api/status`, unauthenticated by the owner's decision — shape only, never a name, a title or
even a count, and answered from memory with no request and no query. In the fallback the beat slows (`FALLBACK_IDLE_S`, `PAUSED_POLL_S`) but never below the owner's own
interval — it only ever slows the beat, never hurries it. A reconnect mid-play subscribes
once for the proof and goes quiet again, and the `SUBSCRIBE_MAX` deadline runs from `listening_since`, not the handshake, or a socket
kept quiet on purpose would be mistaken for a server that does not speak this. That replaced the one-a-minute reconcile read; `Source` has
no `Reconcile` any more, and `CollectorStatus` swapped `last_reconcile_at` for `socket_live` (the socket carries, whatever brought the
last list — `transport` then says which half we are in, and `/api/summary`'s `collector_live` follows `socket_live`).
**Silence is normal, not death**: Jellyfin answers `SessionsStart` with the list as it stands and then
sends nothing until something changes, which on an evening when nobody is watching is never. So two clocks, never one:
`subscribed` (no session list within `SUBSCRIBE_MAX` of being asked = this server does not speak it) and `heard` (nothing
at all — list, `KeepAlive` answer, pong — within `lost_after(period)`, two unanswered keepalives, = gone). Judging the connection by how lately a *list* arrived is
what made 1.4.0 drop the socket every 15 s and poll right through, on exactly the idle server it was meant to spare.
The collector's `on_socket` is likewise the socket saying it carries, not a recent snapshot, and a list that arrives
while the poller sleeps is *kept* (`pending`), because with push-on-change there may not be another for hours.
Any doubt falls back to polling on the next pass, because a play that is never seen cannot be backfilled.
The handshake cannot ask for `profile="PascalCase"` the way every HTTP read does, so `socket.rs` re-cases keys and
holds the first pushed snapshot against one real `/Sessions` read before a row is written from it.

**One play-row shape, three producers.** `playback.rs::PlayRecord` + `media.rs` (stream/transcode extraction, labels,
`effective_play_method`) are shared by the live `collector.rs`, the Jellystat `import.rs` and the Streamystats
`streamystats.rs`, so every source produces identical columns. A fourth tracker is a reader, not new architecture.
The collector inserts a row the moment a play is first seen (`active = 1`), refreshes
it every 30 s, counts only un-paused time, merges a restart within `merge_window_s` into the same row, and diffs
consecutive sightings into `playback_events` (pause/seek/track/transcode timeline). **An event is a change, so the two
sides of `diff_events` must be the same kind of value.** "Once a transcode, always a transcode" is applied to the new
reading *before* the diff, never after: applied after, the kept record says `Transcode` while every reading that follows
says what the client settled back to, and each one is a change — finstats wrote one `transcode` event per second for the
rest of the play, reading `DirectPlay: <reasons>`, a line that contradicts itself (migration 17 clears them). Anything
sticky that the diff also reads belongs above the diff.

**Jellystat import semantics** (learned from real exports; documented at the top of `import.rs` and in `docs/jellystat-import.md`):
`ActivityDateInserted` is the *end* of a play; for episodes `NowPlayingItemId` is the series and `EpisodeId` the
episode; `PositionTicks` is unreliable and not imported (completion = watched ÷ runtime for imported rows);
there is no item type, so Live TV is inferred (video, not in library, no container → `TvChannel`). The import is a
single transaction and streams a multi-hundred-MB file line by line.

**Streamystats import semantics (`streamystats.rs`, `docs/streamystats-import.md`).** Its backup is **sessions only** —
no items, users or libraries — and holds three kinds of row that must be read differently, none of them flagged as such.
**A row carries one moment or two, and which it is decides what the moment means**: two (`startTime` < `endTime`) is a
play Streamystats watched, and the first is the real start; one (`startTime` == `endTime`) is a play it imported from
Jellystat, and that moment is the **end** — `ActivityDateInserted` copied into both fields, 1,799 of 3,167 rows in the
file this was learned from. Reading those as starts moves every one of those evenings forward by the length of the film,
which is invisible until every chart is quietly wrong. `isInferred` (or an `inferred:` id) is **not a play at all**:
Jellyfin reported the item watched, so Streamystats wrote a row as long as the whole runtime for a viewing nobody saw —
counted (`marked_watched`) and never imported. `itemId` is the item in both kinds (Streamystats re-links renamed items);
`mediaSourceId` is *not* an item id. **Where the two kinds carry data is opposite, and reading the wrong one is worse
than reading nothing**: a play it watched has no source media at all (`videoCodec`, `resolution*`, `audioCodec`,
`videoRangeType` empty in every row) but does carry its transcode target; a play from Jellystat carries the whole session
in `rawData` (read with `Streams::extract`) while its own flat `transcoding*` columns are a copy of the *source* with
`transcodeReasons: ["Unknown"]`, which stored as a transcode reads as a file transcoded into itself for reasons unknown —
so those are read only from `rawData.TranscodingInfo`, and a third of them have none and get no transcode. `positionTicks`
counts only where the row also keeps the `runtimeTicks` it is a position in. `isActive` is `true` on nearly every row of a
backup and means nothing. **Nothing is invented to fill a gap**: an item neither the row nor the library can type is left
`Unknown`, not guessed. The file is walked with a `DeserializeSeed` rather than loaded, one transaction.

**One rule for a play the history already has (`playback::already_recorded`), shared by both importers and the backup
restore.** `source_id` stops a file being imported twice and says nothing about the same evening arriving from another
tracker under another id. So: its own id first, else the same person, the same item, and one of the play's **two ends**
close enough. **Between** sources that is `merge_window_s` at *either* end, because the trackers disagree about the
*start* — Jellystat keeps only the end, so the start is derived from the seconds played and every minute paused moves it
later — while all three agree about the end. Measured on real history (3,162 plays a Streamystats export and a Jellystat
import held in common): either end recognised 2,750, the start alone 2,527, and the start's misses ran in an unbroken
smear past ten minutes where the ends fall off a cliff inside one. **Within** one source it is the very same second and
nothing wider: a tracker never exports the same play twice, so a second row of the same item minutes later is a restart
the viewer really made, and a window there would silently drop it — on an import and, worse, on a restore of finstats'
own backup. A new importer goes through `PlayRecord::insert_imported`, never `insert`. **Both statements of the rule
(`SAME_PLAY_SQL`, `RELINKED_DUPLICATES_SQL`) are index ranges** over `(user_id, item_id, started_at)` and `…ended_at`
(migration 25), and a test holds their plans: written as `ABS(started_at - ?) <= window` no index could narrow them, a
120,000-play import took five minutes, and the sweep held the write lock for 25 s at every start and library read.

**Re-linking is the only thing that rewrites `item_id`, so it is the only thing that can turn an imported play into a
duplicate of one already here** — the `already_recorded` check ran before the id moved. `relink::relink_orphans` therefore
re-applies the rule (`playback::drop_relinked_duplicates`) to each title it moved plays onto, in one savepoint with the
move (`move_plays`), and takes `merge_window_s` as an argument so that none of its three callers can forget to. Only an
imported row is ever removed, never one the collector recorded (its row carries a timeline no import can have), and rows
of one source are never compared with each other. A real history had 363 such pairs, 189 rows, mostly music — music being
what gets re-added and renamed. **Anything that rewrites which item a play points at re-applies the rule, or it re-creates
that bug.** It runs at every start and after every library read and nearly always finds nothing, so it must stay cheap
when there is nothing: `ORPHANS_SQL` steps from title to title through the title index and it returns at once when no
title is orphaned; only the plays of orphaned titles are read. Sweeping the whole history instead, and finding orphans by
grouping every play, took 5 of the 8 seconds of a start on a million plays. Migration 26 swept every install once,
with its own merge window, for those coming from before the sweep existed; the rule's text is one macro
(`relinked_duplicates_sql!`) so the statement and the migration cannot drift.

**`sync_libraries` reads a library twice — every item, then the cast and crew of films and shows only — and only the
first count may be shown to `trustworthy_removal`.** The two cursors were both called `start`, the second shadowing the
first, so the guard compared a library's *shows* against its *items*: equal on a film library, 223 against 16,744 on a
television one and 0 against 4,773 on music. Every read of such a library read as a gutted library, so the guard refused
it and halted — correctly, on a reading that was never true — and the install restarted and did it again, every two and a
half minutes, with no library read ever completing. Nothing was wrongly removed (`updated_at` decides that, not this
count, and too small a count only ever refuses) but nothing was read either. The cursors are now `seen` and `people_at`.
**A QA mock must answer the query it is standing in for**: `mock-jellyfin-breakable.mjs` ignored `IncludeItemTypes`, so
both passes looked identical to it and a stage written for exactly this class of fault passed throughout.

**What the library can tell a play lives in `sync::backfill_playbacks`**, not in an importer, because history is usually
imported before finstats has ever read the library: it runs after every library read as well as after an import, and
fills `library_id`, `runtime_s`, season/episode numbers and — for imported rows only — the item type neither tracker
records. A live row keeps the type the session gave it.

**Sync scheduling (`sync.rs`).** Small reads (users, activity log, server details/devices) run every 15 min. The public-IP
lookup is **not** among them: it runs once at start-up on an install that has never learned an address, and otherwise only when
somebody asks (`POST /api/settings/public-ip`, or switching the setting on). The
expensive library read *follows Jellyfin's own "Scan Media Library" task* (`jellyfin::scan_status` over the 5-minute
`/ScheduledTasks` read): it runs after that task finishes, never mid-scan, with a weekly safety net; the
`sync_interval_h` timer is only a fallback (setting `follow_jellyfin_scan`). That read and `sync_server`'s both pass
their task list to `jobs::observe`, which is the only reason the Jellyfin jobs card can open on an ETA: the watch that
`eta_s` needs is fed by lists finstats already has, never by a request made for it. **A read never wipes what it cannot see.** Marking rows `removed` is destructive (they vanish from every page and stat) and
`items_page` turns anything it cannot parse into an empty list, so a Jellyfin that changes shape under an upgrade, or answers
`200 {"Items":[]}`, must not be read as "the library was emptied". `sync::trustworthy_removal(seen, current)` gates every
destructive removal (items, libraries, users): a read that comes back empty, or a catastrophic shrink of a sizeable set, is
refused — the data is kept, the sync fails (task + notification), and `AppState::request_halt` asks the process to stop cleanly
(exit 70, a reason on stderr) so the operator pins a version or pushes a fix rather than finding a wiped install. Ordinary churn
still applies; `FINSTATS_ALLOW_LIBRARY_SHRINK=1` waves a genuine emptying through (and clears a halt loop). After a library read, `backfill_playbacks` links plays to libraries and
`relink.rs` re-attaches orphaned plays to renamed items (Jellyfin ids derive from the path): provider-id match
first, then cleaned title + year, episodes by series + S/E number — only when unambiguous.

**Stats layer (`stats.rs`).** Every query goes through `Scope` → `Cond`: the time window ("last N days" = N full
local days, so chart buckets and totals agree), user/library filters, `min_play_s`, and the rule that **non-admins
without `see_everyone` are force-scoped to their own `user_id`, and IPs/device ids/file paths are only sent with `see_network`/`see_server` — enforced server-side**.
`row_json` maps SQL rows to JSON by column name (`BOOL_COLS` / `JSON_COLS` decide bool and JSON columns), so adding a
field is usually just adding a column to a SELECT. Local-time bucketing relies on SQLite's `'localtime'` and the
process `TZ` (the Docker image ships tzdata for this); Rust that needs the local day of many moments asks through
`LocalDays`, once per quarter hour (every offset and clock change falls on one), never once per play. Do not use `#[serde(flatten)]` in `Query` structs —
serde_urlencoded then hands numbers over as strings and every numeric filter 400s.

**Playback insights (`stats.rs`, 2.0).** Where a title loses its viewers, drawn from where each play stopped. **A stop is
`position_s` for a play finstats recorded and `duration_s` for an imported one** (a tracker keeps a length, not a place; the play is
taken to have started at 0:00), and the two are never mixed silently: every curve carries `measured` and `estimated`. The grid
(`bucket_width`) keeps any runtime to sixty points; under `MIN_CURVE_PLAYS` (3) or without a runtime there is no curve, not a thin
one. Events exist only for live plays, so rewinds (a seek whose `from_s` is past its `position_s`) and subtitle switch-ons (a play's
*first* subtitle change, to a track — the state before it is never an event, so a later language change is not a second switch-on)
are counted over those. `from_s` came in migration 21, backfilled from the seek label in SQL; `playback::backfill_seek_origins` is
the same statement for a restored backup from before it, and a test holds both to the same answer. A show's episodes carry `users`
and `finished` because "everyone quits episode three" is a fact *between* episodes. `GET /api/stats/files` builds three lists from
everyone's plays and answers empty lists without `see_everyone`; the broken-file list uses `Scope::cond_any_length()` — the minimum
play length is precisely what a broken file never reaches — and reads nothing from `server_events`, because Jellyfin's activity
log carries no playback errors (every Error row on a real install was a failed sign-in). Server-wide event queries go through
`idx_pbe_kind` and a test pins the plan. A seek under `SEEK_TOLERANCE_S` was never recorded, so a short rewind is invisible by design.

**Group watching (`groups.rs`, `/together`).** The fold is pure and reusable (2.0): `sessions_for` (a `Window` with an end, so the recap can
ask for one person and one year) → `fold_sessions` → `pairs`, `per_bucket`, `people_shares`, and the three lists the cards always read;
`answer` assembles `GET /api/stats/groups` from them. Two rules: **a pair's time together is the shorter of the two stays** (an evening of
three counts for each of its three pairs; a session's own `together_s` describes any two of its members), and **a session belongs whole to
the bucket it started in** (`alone_s` = scoped watch time in the bucket minus grouped time, never below zero). Without `see_everyone` the
pairs must include the caller and `people` is the caller alone: a companion's name is theirs to see, a companion's time alone is not.
Inferred, because `/Sessions` exposes no SyncPlay groups: plays of one item by ≥ 2
different users starting within `group_window_s` (default 60 — real data shows a third of genuine groups start 6–60 s
apart) and overlapping ≥ 2 min share `playbacks.group_id` (= lowest play id in the group). `detect()` re-runs per item
when a play ends and for the titles re-linking moved plays onto, fully after an import or a restore and when the setting
changes. **A start regroups only what the last run left behind** (`regroup_at_start`): plays it was still recording when it
stopped were all saved after that run began, so the key `groups_detected` notes version, window and when, and the next
start regroups titles with a play ended since — everything only for another version or window. Regrouping the whole
history at every start took 24 s on ten million plays. "Time together" is the
second-longest stay in a session. Running streams are grouped separately by `mark_live` (same title,
different users, starts within the window *or* positions within `max(window, 30)` s), before `/api/now-playing` narrows
the list to the caller. **Everything here is measured at crowd scale** (the capacity stage in `qa/`): `mark_live` pairs
streams of one title only, on plain values read once, and names at most `COMPANIONS_NAMED` companions (the size is exact) —
comparing every stream with every other and listing every companion took 40 s at 10,000 streams and 23.7 GB at 25,000.
`detect` writes only the plays whose group changed; resetting every group to NULL and writing it back rewrote every grouped
play of the history at each start, import and play end. The collector finds ended plays with a set (`ended_keys`) and closes
a pass's ended plays in one transaction, detecting each title once (`close_ended`); the plays a pass sees begin, the
progress it saves and the devices it saw are likewise one transaction each (`start_plays`, `save_progress`,
`remember_devices`), never one awaited call per play — a crowd of 100,000 took 13 s to record that way, and one pass ten.

**Search (`fuzzy.rs`).** `/api/search` scores every library title in Rust instead of using `LIKE`: normalised (case,
accents, punctuation, leading article), every typed word must match some word of the title (exact > prefix > substring
> typo; typos only for words of 4+ letters, with swapped letters as one slip). ~70 ms over 5.5k titles; the palette
debounces. The same query also scores the ~10k names in `item_people` (`people` in the answer): bare names first, details only for the best
60, because grouping and counting every person's titles up front took 350 ms. The Activity and Server-log `q` filters stay in SQL but are word-by-word too.

**Profiles (`profile.rs`).** Show progress counts only episodes that exist as files (`path`/`size_bytes` set; the sync
asks Jellyfin to exclude virtual items, and season 0 is skipped). "Seen" merges three sources in order: a recorded
play ≥ 80%, Jellyfin's played flag (`user_items`), a manual mark (`manual_seen`, written via `POST /api/me/seen` for
the caller only — finstats never writes to Jellyfin). Streaks are all-time and share `recap::longest_run`.

**Timeline (`timeline.rs`, `/users/:id/timeline`).** One person's plays, newest first, folded by the pure `fold()`: plays that follow
each other with the same key (series + season, album + artist, else the item) are one stop. Pages use a `(started_at, id)` cursor
and only give out a stop once the play after it has been read, so a stop is never split and the stops do not depend on the page
size. In the UI (`pages/timeline.js`) the column count comes from a `ResizeObserver`: rows alternate direction by being `direction: rtl`
(their cards reset to `ltr`), a bend joins each row to the next, and below 620 px it becomes one straight line (`.is-line`). DOM
order stays chronological, so keyboard and screen readers follow the trail.

**Recently added (`recent.rs`, the dashboard shelf).** Episodes of one show added on the same local day fold into one entry (pure
`fold()`), everything else is its own. It must stay cheap because the dashboard loads it: the query walks the partial index
`idx_items_added` newest first and `fold()` stops reading at the first row older than the last entry's day (about 50 rows for 30
entries). The `+i.type` in its SQL is deliberate: without it the planner picks the type index, sorts every episode and takes ~270 ms on
a 23k-episode library; a test asserts the plan. Not scoped to the caller: the library is the same for everyone.
The shelf scrolls by hand (`shelf()` in `pages/dashboard.js`): one glide towards a target for Shift+wheel, keys and the arrows. Do not
put CSS scroll-snap on a row like this: a wheel notch shorter than half a card springs back, so Firefox users could barely move it.

**Recap (`recap.rs`)** is one person's year: the caller's own, or, for a Jellyfin administrator only (`is_admin`, not a
permission), the user in `user_id`, or the whole server's (`scope=server`, 2.0) — which goes through `server_edition`, so it
ranks nobody and names nobody (companions, rank and apps taken out). `whose_year` is the one place that rule lives; the page
and the cards both go through it. It excludes Live TV item types and defaults to the year that is "ready": the current year in
December, otherwise the previous one. 2.0 added `together` (the group fold for one person and one year; companions named, at
most three), `finished` (by `profile::episodes`, the same "seen" as show progress; dropped = begun this year, under half seen,
nothing for `DROPPED_QUIET_S`), `requests` (null when no request was ever recorded) and `versus`. **The year as a story
(`story.rs`)**: every card is drawn from `StoryYear`, a typed copy with no field for a companion, a rank or an app, so a card
cannot name anybody however the recap grows — never draw a card from the recap's JSON. `/api/recap` answers `story` (which
chapters have cards) so the page never recomputes it. `recap::announce_ready` tells each person who watched in December, once
(`notify:recap:{year}:{user}`).
Its "most watched people" read `item_people`: actors (first 12 billed) and directors of films and shows only, filled by a
second, small `/Items` pass per library in `sync_libraries` (`Fields=People`) — never add `People` to the main item read.
The same table feeds the Cast & crew row on `/items/:id` and the person pages (`/people/:id`, `stats::person_detail`), which are
scoped like any other stats query.

**Auth (`auth.rs`).** Login forwards credentials to Jellyfin's `AuthenticateByName`, immediately logs that Jellyfin
session out, and mints an own opaque session token (stored hashed, HttpOnly SameSite=Lax cookie). **Two credentials, one
resolution point**: `auth::resolve` reads `Authorization: Bearer fs_…` first (`looks_like_key` gates before any query; an
invalid header never falls back to the cookie — header beats cookie), else the cookie, and both end in the same `AuthUser`,
which now carries `credential` (`Session` | `Key{id, scope}`) and `ip`. A key (`keys.rs`, `api_keys`, sha256 of `fs_`+64 hex,
shown once) is resolved against the **live** user row and the live grants through `effective`, never a snapshot, so a lost
`sign_in` or a demotion reaches every key at once. One person keeps at most `SESSIONS_PER_USER` (30) sessions, the oldest giving
way at the next sign-in (`store_session`), or a password in a loop fills the table. **A session is resolved the same way** (`resolve_session_in` joins `users`;
`sessions.is_admin` is only the fallback before the users read has written the row, which the wizard's own sign-in is): an
administrator demoted, disabled or deleted in Jellyfin used to keep full access for the session's 30 days. Because the
session now trusts the row, every sign-in writes Jellyfin's fresh answer into it (`remember_sign_in`), or a row a Jellystat
import wrote from the file's `IsAdministrator` could crown somebody Jellyfin did not. `sync::store_users` marks removed whoever
the read did not return, by id — a whole-second `updated_at` comparison missed a deletion read in the same second; `touch_key` writes `last_used_*` at most once a minute. `KeyScope::Calendar`
opens only `/api/calendar.ics`: the `AuthUser` extractor refuses it with 403 everywhere, and the feed's own `CalendarKey`
extractor takes `?key=` or the header and never the cookie (`ical.rs`; `pipeline::entries_for` carries no name by construction,
since a subscribed calendar syncs through somebody's cloud). Minting or revoking through a key is refused (`keys::only_a_session`).
`audit.rs` is the one record of finstats' own write paths (`KINDS`; `Actor::from(&AuthUser)`; `record` is fire-and-forget and
never the caller's error, `record_now` for a row that must land before what it announces — an import's transaction); a new
write path gets a kind and one `record`, never a second log. `api_keys` is never in a backup; `audit` is.

**Public profiles (`public.rs`, `card.rs`, `/u/:token`, 2.0) are the one read without an account, and they never go
through `AuthUser` or `Scope`.** A `Published` names one person and the sections they switched on; `answer` builds each
of those from that id alone and returns typed structs, never `row_json`, so a column added to a shared query cannot reach
a stranger. Devices, clients, play methods, addresses, places, paths, other people and the login name are never produced.
**Every section counts only plays that ended a day ago or more** (`DELAY_S`; `recap::build` takes `until` for it): the
first version delayed only the recent list, and the totals, grid and streak then moved the moment somebody pressed play.
`lookup` answers `None` for every reason there is nothing (switch off, unknown or reset token, unpublished, owner removed,
disabled or without `sign_in` via `auth::effective`) and all of them are the same 404. The image routes serve only
`listed_images`. Cards are SVG built from the `PublicProfile` alone and rasterised with resvg using Inter TTFs in
`fonts/card/` (the web woff2 cannot be read; no system font, and `<image>` may only be the card's own `data:`), cached by
link + kind + a hash of what they say, two drawn at once. `charts.js` cannot run server-side, so the card's grid copies
its colours; change them together. The page (`web/public.html` + `js/public/profile.js`) is outside the app: no
state, api.js, router or prefetch, so nothing there can send a stranger to /login. `public_profiles` is not in backups.

**Permissions.** `AuthUser.perms` (`Perms`: `see_everyone`, `see_network`, `see_server`, `see_downloads`, `notify`, `manage`) is rebuilt on every
request from `user_permissions` ∪ `Settings.default_permissions`; `sign_in` (or `allow_user_login`) gates access at
all; Jellyfin admins always get `Perms::ALL`. Grants only add, there are no denies. Extractors: `AuthUser` (anyone
signed in), `ServerViewer`, `Manager`, and `JellyfinAdmin` — the only one allowed to edit permissions, and
`put_settings` refuses the access keys from anyone else, so a manager cannot self-promote. In `stats.rs` decide by
the specific permission (`scope.perms.see_network` for IPs, `see_server` for paths, `see_everyone` for whose rows),
never by `is_admin`. The recap ignores permissions: own for everyone, any one user for Jellyfin administrators. The UI mirrors this with
`can('perm')` from `state.js`; it is cosmetic — every rule is enforced server-side. `api.rs` adds an Origin check on writes and a strict CSP
(`style-src 'self'` — the UI must not use inline `<style>`/`style=""`; `el.style.x` via JS is fine).

**Local vs remote (`network.rs`).** `is_local` = private range (`db::is_local_ip`) or a row in `home_addresses`: this network's own
public IP, looked up **once** from a plain-text service (the only non-Jellyfin request finstats makes by default; setting
`public_ip_lookup`, override `FINSTATS_PUBLIC_IP_URL`) — at start-up when no `lookup` row exists, or when somebody presses the button —
plus the manual `home_addresses` setting. Always go through
`network::classify(conn, ip)`; after the set changes call `network::reclassify`, which re-decides the whole history and
notes what it decided against (`network_classified`: version and home addresses); a start re-decides only when that differs
(`reclassify_at_start`), never out of habit. The lookup
must stay anonymous (no version, no ids in the request) and the docs' privacy claims must stay true to it.

**Security (`geo.rs`, `security.rs`, `/security`).** `geo.rs` reads a MaxMind-format city database through a memory map (`Geo` in `AppState`,
swapped whole when a newer file appears) — of a **private copy** (`private_copy`: copied into `<data>/geoip`, opened, its name removed),
never of the file itself, because an owner who copies a newer file over the old one truncates what is mapped and the next lookup is
SIGBUS, which kills the process. The file used is `FINSTATS_GEOIP_DB`, else the newest `.mmdb` in `<data>/geoip/`. Lookups never leave the machine; the
only network use is the opt-in download of DB-IP's monthly file (`geoip_download`, off by default, task `geoip`), which must stay as anonymous
as the public-IP lookup, and the docs' privacy claims must stay true to both. Every distinct address gets one row in `ip_locations`, keyed by the
spelling stored in `playbacks`/`server_events` (an all-NULL row = looked up, no place); a changed database empties the table and places everything
again (`refresh_all`). `server_events.remote_ip` is parsed from the log text by `event_ip` (first word that is an address, because the label
follows the server's language; `''` = none). Plays from the home network are placed where the home's public address is (`home_place`).
Alerts come from the pure `detect()` over one person's whole history, so a rescan finds the same ones and `dedupe` keeps them single: travel is
keyed by the later sighting, a pair of places reports once a day, a muted pair never, and anything found more than 30 days late is filed as
resolved. `scan` runs when a play begins (that user), after the log sync, at start-up and when the rules change. Reads need `see_network` **and**
`see_everyone`, writes also `manage`; failed sign-ins additionally `see_server`. The map (`worldmap.js`) is hand-drawn SVG over
`web/assets/geo/world.json` (Natural Earth, pre-projected by `tools/make-world-map.py`; Mercator, because an equal-area projection leans the north and the owner read it as a tilted map; the formula there and in `worldmap.js` must
match). No tiles, no map service. The plain wheel scrolls the page; Ctrl+wheel, the buttons and the keys zoom.

**Pipeline: the services around Jellyfin (`services.rs`, `arr.rs`, `seerr.rs`, `downloads.rs`, `pipeline.rs`, `/pipeline`).**
Connections (Sonarr, Radarr, Seerr; several of a kind) live in `services`, secrets and all, and are `JellyfinAdmin`-only. **Download clients are
deliberately not connected**: Sonarr and Radarr already talk to them and report a torrent and a usenet download the same way, so finstats reads
their queues — three APIs to keep up with instead of six, and nothing for the owner to set up twice. What that gives up is ratio and peers; the
speed is worked out from what moved between two readings (`downloads::speeds`). They get **their own HTTP clients that follow no redirect**: reqwest drops `Authorization`/`Cookie` across hosts but not
`X-Api-Key`, and a 307 replays a POST body. `Service` is neither `Serialize` nor `Debug`; its JSON is hand-built with `has_secret`; errors name a
status and a kind of failure, never an upstream body. Everything is `GET`, so nothing in the code could change anything there. An id is never reused and pointing a
connection at another host/port/base path forgets its rows. Certificates are verified unless the owner switches that off per connection.
Tasks: `sync_upcoming` + `sync_grabs` (15 min), `sync_requests` (5 min), all through `services::spawn`; `api::RUNNABLE` is the one way in and a
test holds it against `TASK_IDS`. `item_external` turns `items.provider_ids` into indexed rows — **one id may belong to several items** (HD and 4K),
so joins go id → every item → plays, unlike `relink.rs`, which refuses ambiguity because it rewrites history. Film releases are stored as a *day*
(Radarr's midnight UTC is the evening before west of Greenwich); episodes keep their moment. Seerr is read newest-modified-first with an overlap
on *its* clock, plus a re-read of everything still open (a media status change does not touch the request) — that one every 15 min and only while
something *is* open — and only a whole listing may set `removed_at` (Seerr purges; the history must not shrink). Every pass starts with a `take=1`
probe (`newest_change`): a newest `updatedAt` no newer than the cursor means nothing was created or changed, and the pass ends there, which is
what keeps a five-minute cadence from being most of the traffic finstats makes. Users link by `jellyfinUserId`, then Jellyfin user name — never a display name or e-mail.
`downloads.rs` is the only live part: its own loop and `Notify`, 5 s while a page says `?live=1`, otherwise 60 s while anything is in the queue
(a request page shows how far along it is) and 5 min while it is empty (`wait_s`) — an empty queue has nothing to go out of date, and opening the
page, connecting a service or a read of Seerr all wake the loop. No DB work per tick, the snapshot in memory only. `fold()` turns queue records into downloads by `downloadId` + service: a season pack is one row, the same id in two
instances is two rows, a record without an id stands for itself. Scoping goes through `stats::pinned_user`: own requests for everyone, others'
need `see_everyone` (and then no follower *counts* either — on a small server a number is a name), the queue needs `see_downloads`, while own-request
progress (`state`, `progress`, `eta_s` and nothing else) is always allowed. A queue row's `error` is the service's own words
passed through `downloads::redact`: Sonarr and Radarr quote the client or indexer they could not reach, `user:pass@` and
`?apikey=` included. The poster proxy `/img/arr/{service}/{media}` serves only ids finstats
itself has listed. None of these tables are in `backup::TABLES`: they are re-readable, and `services` holds secrets.

**Outbound connections (`outbound.rs`, `GET /api/outbound`, Settings card).** One row per destination finstats can reach —
Jellyfin, the public-IP services, DB-IP, each `services` row, each `notify_targets` row — with whether it is on and when it
last answered. Built entirely from what is already kept (collector status, `home_addresses`, the `.mmdb` on disk,
`service_health`, a destination's `last_ok_at`): nothing is recorded for it, and hosts are shown without paths or keys. A new
outbound destination must appear here, and in the promise sentences in `README.md` and `docs/security.md`, in the same change
that adds it.

**Jellyfin's own jobs (`jobs.rs`, `GET /api/jellyfin/jobs`, the Server page).** Jellyfin's scheduled tasks say what the
code is called ("Detect and Analyze Media Segments"); `explain` says what it does to the server, matching Jellyfin's
**key first** (a name is in the server's language) and a keyword in the name second, and falling back to Jellyfin's own
`Description` — a sentence finstats does not have is never invented. **A run is timed by watching it**: the API carries
a percentage and never a start time for the run in progress, so `Run` keeps the recent readings and `eta_s` works out
the rest from the rate they moved at — measured against the *most recent* reading that is far enough back to say
anything (0.5% and 5 s, within `WINDOW_S`), never the average of the whole run. **Nothing that was not measured is ever offered**: when there is no rate, `eta_s` is `None` and the page
shows a cycling ellipsis rather than a number. The tempting fallback — the last run's duration for the fraction left —
is what put "about 2 minutes left" on the screen for a quarter of an hour (1.6.1 made it age, 1.6.2 removed it): it
knows nothing about how much of *this* run has happened and reads exactly like an earned estimate. `unchanged_for_s` is published so a slow job reads as slow
rather than as a stuck page; a percentage that goes backwards is the next run, not this one going back. `schedule` prints a time of day as
a time, never a countdown — a daily trigger is in the *server's* local zone, which finstats cannot know — and only an
interval trigger, measured from the last run, produces `next_at`. The read is live but never more often than
`MIN_GAP_S` (3 s) however many people watch, hidden tasks included, and read-only like everything else: nothing in the
code can start or stop a task on Jellyfin.

**Notifications (`notify.rs`, `channels.rs`, `/api/notifications*`, Settings card).** The only thing finstats *sends*. An
**event** is raised where the thing is noticed and written once — `raise_in` is `INSERT OR IGNORE` on `dedupe`, exactly like
`security_alerts`, so re-deriving the same thing announces nothing twice — and **delivery is a separate row per destination**
with its own attempts and clock (`backoff`: 30 s → 2 min → 10 min → 1 h, then given up on; `Retry-After` honoured;
`PER_MINUTE` per destination), so a webhook that is down delays nothing else. `notify::run` is its own loop, woken by
`notify_wake` and otherwise asleep until the next retry; an install with no destination never wakes. **Two guards make adding a
destination safe**, both needed: anything found more than `HISTORIC_S` (6 h) after it happened is stored `historic = 1` and
never queued, and `wanted_by` refuses anything that happened before that destination's `created_at`. History is thinned at
`KEEP_S` (30 days), which is safe only because every source either re-derives a short window (`recent::announce`,
`seerr::announce_available`, `security::scan_sign_ins`) or raises once, when the thing itself is first written
(`security::file_alerts`).
**What may be said is one pure function**: `message(event, with_addresses, public_url)`. An event carries two bags — `data`,
which any destination may be told, and `private` (addresses, coordinates), which `message` reads *only* with the
per-destination switch — so the rule holds by construction rather than by care, and a test asserts no address appears in any
kind of message without it. The link is `public_url` + the event's path; empty setting, no link.
**`wanted_by` is the whole permission rule in one pure place**: a server destination (`owner_id IS NULL`) is not filtered; a
personal one is checked against its owner's `Perms` (`auth::effective`) — own rows always, somebody else's play or request
needs `see_everyone`, somebody else's *places* need `see_network` too (`security::gate`'s rule), the server's own business
needs `see_server`. Managing a personal destination needs the grantable `notify`, and its host must not resolve into a private
range (`must_be_public`, checked on save **and** before every send, because a public name can be re-pointed later); an
administrator's may point anywhere.
**A destination's address is a credential** (a Discord webhook URL carries its token), so `Target` is neither `Serialize` nor
`Debug`, the API answers `shown` (host, plus the topic, chat or mailbox that says which one it is) and never the URL, editing without a `url` keeps the stored one,
and `notify_targets` is not in `backup::TABLES`. `channels.rs` holds every payload shape and the POST; it reuses
`services::Http` (no redirect followed while holding a token) and publishes to ntfy and Gotify in their JSON form, never
through headers, because a title is a film title — the same reason Telegram is sent with no `parse_mode`.
**Nine kinds of destination, and one model under them**: an address, a token, one field beside them, and `options` for
what is left (migration 19; only mail has any). A channel may fix its own address (`Channel::fixed_url`: Telegram,
Pushover, Pushbullet are reached at their own service and nowhere else, so a token can never be posted to a look-alike
host) and names that one field itself (`topic_label`: an ntfy topic, a chat id, a Pushover user key, a mailbox), each
checked the way its own service writes it. **Email is the one that is not a POST of JSON**, so it is its own module
(`mail.rs`, `lettre`): `smtps://` is encrypted from the first byte, `smtp://` must upgrade with STARTTLS, and there is no
third option — no path by which a password is sent in the clear. `payload()` answering `None` is what says "not an HTTP
request at all"; `Channel::is_mail` is the same fact where it is easier to read. Anything new here is a `Channel` arm, a
payload and where its token goes: `notify.rs` should not have to change for one. Producers live where the thing is noticed: `security.rs` (alerts, and
`bursts` of failed sign-ins), `sync.rs` (a failed job, a failed backup, what arrived), `services.rs` and `collector.rs` (a
connection that stopped answering, Jellyfin included, and plays beginning and ending), `seerr.rs` (a request that became
watchable). Adding a kind of event means a `Kind` arm and one `raise` — never a second way out.

**Backups (`backup.rs`).** gzip JSON Lines, one row per line tagged with its table, matched *by column name* both ways so files move
between versions; a new table that holds something Jellyfin cannot give back must be added to `backup::TABLES`. Secrets (Jellyfin
URL/API key, sessions, API keys) and the library are never exported; a test asserts the key is absent. Restore merges (dedupe on `source_id`
or user+item+start), remaps timeline rows to the new play ids, and re-derives groups, `is_local` and library links. The scheduler
writes one when the newest file is older than `backup_every_d`; endpoints are `JellyfinAdmin`-only and names go through `valid_name`.
A file is written as `<name>.part` and renamed when complete; a `.part` found when the next one starts was a killed backup and is removed.

The response compression layer skips `application/gzip`: re-compressing a backup broke the download in browsers. Anything served
pre-compressed needs the same exemption.

**HTTP contract.** `docs/api.md` is the contract the UI is written against; change it together with the endpoint.

**The README says how finstats is built** — a blockquote under the hero shot disclosing that most of the code was
written by Claude Code from the maintainer's decisions, and what that does and does not mean: test-first, reviewed
before it lands, `cargo test` in CI on every push to `main` and every release tag, each release run against a real
server. Those are promises, like the privacy sentences: if any of them stops being true, the blockquote changes in
the same commit. It names no model version on purpose, so it does not go stale.

For people: `CONTRIBUTING.md` is the one guide (what fits the project, running from source, the local Docker setup with a throwaway
Jellyfin, rules for a change, pull requests); `SECURITY.md` covers private reporting, `CODE_OF_CONDUCT.md` behaviour (it forbids posting other people's viewing data), and
`.github/` holds the issue forms, the discussion forms (file name = category slug: `q-a`, `ideas`, `show-and-tell`) and the PR template. Keep their rules in step with this file, and do
not add a second developer guide next to them.

## Frontend (`web/`)

No build step, no dependencies, no CDN: vanilla ES modules served from the binary, fonts bundled. `dom.js` (`h()`
element builder, icon map, formatters), `api.js`, `router.js` (History API; the server returns `index.html` for any
non-`/api`, non-`/assets` path), `components.js`, `charts.js` (hand-rolled SVG), `pages/*.js`. Rules that hold
everywhere: API/user strings reach the DOM only via `h()`/`textContent` (never `innerHTML` with data); fetches are
aborted and timers cleared on route change; every new card must hide itself when its data is missing (older servers,
non-admins, imported plays). Native `el.append(null)` prints the text "null" — pass possibly-absent nodes through
`h()` or filter them first.

"Now playing" (`nowPlayingView` in `widgets.js`) is polled every 5 s, but **only the 1 s ticker moves a clock** (+1 whole
second per beat). A poll never repaints it; it only corrects the position when that means something (pause, server > 3 s
ahead or > 15 s behind — clients report to Jellyfin roughly every 10 s), and even then by setting it one short so the
change lands on the next beat. Painting from the poll is what made the clock stutter.

Tables go through `tables.js`: `dataTable()` / `plainTable()` / `chartTable()` wrap a built `<table>` in its scroll box and make every
header a sort button (`sortable()` for tables without a scroll box, like the bar lists). Sorting reads cell meaning (durations, sizes, %,
`<time>`, switches; `data-sort` overrides, `data-nosort` opts a header out, `data-pin` keeps a row on top). Paginated lists must not be
sorted in the browser: they pass `server: {key, dir, onSort}` with `data-key` headers, and the endpoint whitelists `sort` via
`stats::order_by`. A new table that skips this helper is a bug.

Pages load through `dataView()` (`components.js`), which is stale-while-revalidate: it records the GET URLs a page's
`fetch()` issues (they must be fired synchronously when `fetch()` is called), uses them as the cache key, paints a
remembered result at once, refreshes behind it and only re-renders when the JSON differs. Skeletons appear only after
150 ms and then stay at least 300 ms. The cache is in-memory per tab and is cleared by `resetCaches()` (sign-out) and
by any non-GET request. **Prefetch (`prefetch.js`).** Fills that same cache before a page is asked for: a warm-up about a second after the first page (top-level
pages, then each library and the 12 most active people, one at a time while the browser is idle; at most every 2 min; not on
`saveData`/2g), and on intent (pointer resting 90 ms on a link, focus, touch). Titles and people are fetched on intent only, never in bulk.
It only works because page and prefetcher call the *same loader*: every page module exports `prefetchX(ctx) → [() => load…]` next to
the loader its `dataView` uses, and `main.js` passes it as `route(…, { prefetch })`. A page whose `fetch` is written inline cannot be
prefetched, and a loader copied into `prefetch.js` would drift from the page's URLs and silently miss. Clearing the cache bumps a
generation and aborts what is on its way, so an answer from before a sign-out or a write is never stored. Pages join a prefetch
already in flight (`shareRequestsOf`); only the prefetcher's requests can be joined, because a page's signal dies with the page.
**Settings (`pages/settings.js`, `settings/*.js`) and Server (`pages/server.js`) are one section on screen at a time**, on the shell in
`sections.js` (`pickSection` forwards bare and legacy addresses, `sectionNav` draws the sticky list, `reveal` marks a row a link pointed at):
`/settings/:section` and `/server/:section`, a sticky list on the left (a row of chips on a phone), and on Settings a finder in the header
(`/` focuses it) built from each section's `entries`. A section is a module
exporting `{ key, label, sub, group, icon, visible, entries, render(slot, store) }`; `settings/common.js` holds the store (`/settings` and
`/tasks`, fetched once per visit, tasks polled only if a section subscribes) and the two shapes a setting takes: `toggleRow` (saves itself) and
`numberForm` (one `Save` per section, bottom right). A setting is one row — label, one line of help of at most ~150 characters, control on the
right — and a QA check holds that budget, because the page it replaced was fourteen cards and six hundred words of help in one column. Bare
`/settings` and the old anchors (`/settings#backups`) forward to the section that holds them (`LEGACY`); a new section's card id stays a valid
anchor. Sections hide themselves (`visible`) rather than explaining why they are empty.
Esc is handled globally in `shell.js` (steps back out of `/libraries/:id`, `/users/:id`,
`/items/:id`); overlays must keep calling `stopPropagation()` on their own Esc.
Each screen is held to the UX patterns from <https://uxgoodpatterns.com>. A generated copy, `ux-rules.md`, may sit in the working
tree for reference; it is someone else's work, is git-ignored and must never be committed. The look is Obsidian's in the dark and washi paper in the light (cream ground, ink text, a vermilion seal for
the accent, nando for quantities), via the tokens at the top of `app.css`; categorical chart colours follow the entity (Movie/Episode/Audio/Other), never rank.
**Every colour is a token, and every token is `light-dark(light, dark)`** — one declaration, two themes. A colour literal
anywhere else (a CSS rule, a chart's `fill`, a `style.background` in JS) is right in one theme and wrong in the other; the QA
light sweep measures text contrast on every page and fails on it. JS reaches a token as the string `'var(--x)'` through `style`
(never an SVG presentation attribute). With nothing chosen the device decides (`color-scheme: light dark`); the sidebar's
theme button cycles Device → Light → Dark and keeps the choice in `localStorage` (`finstats.theme`, absent = Device), which
`theme.js` — a classic, blocking script in both pages' `<head>` — puts on `<html data-theme>` before the first paint. The
server-rendered profile cards (`card.rs`) stay dark.

## Local QA suite

`qa/` (git-ignored, so it may not exist in a fresh clone) holds a local release gate: `qa/run.sh` runs static checks, an API suite
and a real-browser suite against a throwaway instance built from generated data. Run it before every release and add a check for
every bug fixed. It is a local tool and must never be committed; the same goes for `ux-rules.md`.

## Releases and patch notes

`CHANGELOG.md` is the single source for the in-app **Patch notes** tab (`changelog.rs` parses it). A test fails if
the top entry's version differs from `Cargo.toml`, or an entry lacks a date/notes/known group — so a version bump
and its changelog entry land together. Format: `## [x.y.z] - YYYY-MM-DD`, or `- YYYY-MM-DD to YYYY-MM-DD` for a release
made over several days (`date` stays the release day, `started` the first; the page shows both), then
`### Added | Changed | Performance | Stability | Fixed | Removed` with one-line bullets (`**bold**` and `` `code` `` are
rendered). **Performance** is quicker or lighter, **Stability** is something that can no longer crash, halt, leak or lose
data, **Fixed** is something that gave a wrong answer — a speed-up filed as a fix reads as a bug nobody had. The rules live
in one test function (`changelog::tests::problems`). An `x.y.0` entry opens with a **short title line**
and nothing else — "Notifications", "WebSocket session tracking" — which the app shows as that whole series' headline
(`changelog.js` takes the summary's first sentence, or all of it when there is no full stop, so a title stays whole);
a test enforces that an `x.y.0` has one. Notes are terse and factual, in the shape of a GitHub changelog: what
changed, and the fact that makes it make sense. Nothing a release did not do.

History shape: a release is its logical commits (backend before UI, `fix(...)` on their own, then `docs:`),
followed by `chore(release): x.y.z` bumping `Cargo.toml` + `Cargo.lock`, and an annotated tag `vX.Y.Z`. Every
commit that touches Rust must build and pass `cargo test` on its own.

## Licensing

finstats is `GPL-3.0-only` (`LICENSE`, `Cargo.toml`). A new dependency must carry a GPL-3.0-compatible license
(MIT, Apache-2.0, BSD, ISC, Zlib, MPL-2.0 and similar are fine; check with `cargo metadata`). The bundled fonts are
OFL-1.1 and their license texts live next to them in `web/assets/fonts/` — keep them together.

**The notice is shipped, not summarised (`licenses.rs`, `THIRD-PARTY.json`, `GET /api/licenses`, `/licenses`).**
`tools/make-third-party.py` walks `cargo metadata` (non-dev, every platform: a notice true only on the machine that
generated it would be wrong on the others) and reads each crate's own `LICENSE` files out of the sources cargo has
unpacked. Identical texts are stored once — hundreds of crates ship the same MIT wording — and every crate points at
the ones it carries. **Nothing in here is retyped**: each text is a licence file as its project wrote it, and the
bundled half (`BUNDLED` in `licenses.rs`: finstats' own GPL, the two fonts, the map, DB-IP's database) is
`include_str!` of the file that really sits on disk. Re-run the generator whenever a dependency is added, removed or
bumped: a test holds the file against every package in `Cargo.lock`, so a dependency whose licence was never recorded
fails `cargo test`. The answer is rendered once into a `OnceLock` — half a megabyte, the same for everyone, unchanging
while the process runs. The page reaches it from a button in Settings; the route itself is open to anyone signed in.

## Publishing

The image has no `USER` line on purpose: `docker-entrypoint.sh` starts as root only to make the data directory belong to `PUID:PGID`
(default 1000:1000; Docker creates a missing bind-mount folder as root, which is what broke 1.0.0 on fresh machines), then `su-exec`s
to that user; with `--user` it changes nothing. finstats itself never runs as root. `main.rs::ensure_writable` fails fast with the fix.
Test the image on folders Docker creates (`qa/run.sh docker`), not on a data folder that already exists on the dev machine.

**The owner's own install runs from this checkout, and it is redeployed at the end of every change.** When a container named `finstats`
is running (`docker ps`), finishing a feature or a fix means rebuilding the local image and swapping the container without being
asked: `docker build -t finstats:latest .`, `docker rm -f finstats`, then the same `docker run` it had (read its binds, env and ports
from `docker inspect` first, never from memory), and a check of `/api/status`, the log and the health state afterwards. Work that
only lands in the repository while `localhost:8080` still shows the old build is not finished. A newer binary snapshots the database
by itself before it changes anything (`back_up_before_update`); take a manual `sqlite3 data/finstats.db ".backup …"` too when the
change is risky.

The repository is `github.com/finstats/finstats`; images go to `ghcr.io/finstats/finstats`. Up to 1.6.5 both lived
under the maintainer's personal account; 2.0.0 is the first release from the organisation. Nothing but
`CHANGELOG.md` (history) may name the old home — a QA static check holds it. `.github/workflows/docker.yml` runs the unit
tests, builds amd64 and arm64 on native runners (no QEMU), and publishes `:edge` from `main` and `:X.Y.Z`, `:X.Y`, `:X`, `:latest` from a
`vX.Y.Z` tag, then creates the GitHub release from that version's `CHANGELOG.md` section. It refuses a tag that does not match
`Cargo.toml` or has no changelog entry, so the release commit and its tag must be pushed together. Docs always point at the published
image, never at a locally built tag.

## Git conventions

- Conventional-commit subjects, with a scope where one fits (`feat(recap):`, `fix(import):`, `refactor`, `docs`,
  `chore`). Subject says what changed; body says why.
- No LLM attribution of any kind: no `Co-Authored-By:` for Claude or any model, no session links, no
  "Generated with…" line, in commits or PR text, regardless of tool defaults.
- Commit as work lands; each commit is one logical change and stands on its own. Docs may follow as their own `docs:` commit.
- Never push unless explicitly told to.

## This repository is public

`data/` (the SQLite database: users, IP addresses, the Jellyfin API key) and tracker exports — Jellystat's
(`*.jsonl`, `backup_*`) and Streamystats' (`streamystats-backup-*.json`, which carries public IP addresses) —
sit next to the source on development machines and must never be committed; `.gitignore`,
`.dockerignore` and `.githooks/pre-commit` (enable with `git config core.hooksPath .githooks`) guard this. Stage
explicit paths rather than `git add -A`. Tests, docs, examples and commit messages use invented data only
(e.g. "alice", "Big Buck Bunny", `Europe/London`, `192.168.1.10`) — never values from a real server.
