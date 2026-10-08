# finstats HTTP API

All endpoints live under `/api` and speak JSON. Authentication is a session
cookie (`finstats_session`, HttpOnly, SameSite=Lax) issued by `POST /api/auth/login`, **or** an API key
sent as `Authorization: Bearer fs_…` (see *v1.10*). Either resolves to the same user with the same
permissions; a request carrying both is judged on the header alone.

## Conventions

- **Timestamps** are unix seconds (UTC). **Durations** are seconds.
- **Dates** in time series are `YYYY-MM-DD` in the server's local timezone (`TZ` env).
- **IDs** are Jellyfin IDs: 32 lowercase hex chars, no dashes.
- **Errors**: non-2xx status with `{"error": "human readable message"}`.
  `401` = not logged in (also an expired, revoked or malformed API key), `403` = not allowed (also a
  calendar-scoped key anywhere but the feed), `409` = wrong state (e.g. already configured).
- **Common filters** (query string) on every `/api/stats/*`, `/api/activity`, `/api/users*`,
  `/api/libraries*`, `/api/items/*` endpoint:
  - `days`: integer window ending now. `0` or absent = all time.
  - `user_id`: restrict to these users: one id, or several separated by commas
    (`user_id=a1b2,c3d4`). Absent or empty = everybody.
  - `library_id`: restrict to one library.
- **A filter that names several values** uses commas, and naming nothing means all of them,
  so `?type=` and no `type` at all are the same request. Values are de-duplicated and at most 50
  are read, so no caller can grow the SQL by repeating or padding them.
- **Non-admin users** are always scoped to their own `user_id` server-side, whichever and however
  many the URL names: a user filter can narrow what somebody sees and never widen it. They also
  never see IP addresses, and get `403` on admin endpoints (marked 🔒).

## Bootstrap & auth

| Method | Path | Body | Response |
|---|---|---|---|
| GET | `/api/status` | – | `{configured, version, server_name?}` plus how the collector is listening (public) |
| POST | `/api/setup/test` | `{url}` | `{server_name, version, id}` (public, only while unconfigured) |
| POST | `/api/setup` | `{url, username, password}` | `{user}`; must be a Jellyfin admin. Creates a Jellyfin API key named `finstats`, stores config, logs in, kicks off first sync. Only while unconfigured. |
| POST | `/api/auth/login` | `{username, password}` | `{user}`; `401` bad credentials, `403` user login disabled, `429` too many attempts |
| POST | `/api/auth/logout` | – | `{ok: true}` |
| GET | `/api/auth/me` | – | `{user}` or `401` |

`user` = `{id, name, is_admin, has_image}`.

## Live

`GET /api/now-playing` → `{sessions: [Session]}`

```jsonc
Session = {
  "key": "…",                 // stable while this play lasts
  "user_id": "…", "user_name": "…",
  "item_id": "…", "item_name": "…", "item_type": "Episode",
  "series_id": "…"|null, "series_name": "…"|null,
  "season_number": 1|null, "episode_number": 4|null,
  "image_item_id": "…",       // best item to ask /api/img for (series poster for episodes)
  "position_s": 512, "runtime_s": 1440,
  "is_paused": false,
  "started_at": 1790000000, "watched_s": 498,
  "client": "Jellyfin Web", "device_name": "Firefox", "app_version": "10.11.0",
  "remote_ip": "…"|null,      // null for non-admins
  "play_method": "DirectPlay" | "DirectStream" | "Transcode",
  "video": "HEVC 1080p HDR10"|null, "audio": "EAC3 5.1"|null, "subtitle": "eng (subrip)"|null,
  "container": "mkv"|null, "bitrate": 8500000|null,
  "transcode": null | {
    "video_codec": "h264", "audio_codec": "aac", "container": "ts",
    "is_video_direct": false, "is_audio_direct": false,
    "hw_accel": "nvenc"|null, "reasons": ["ContainerNotSupported"], "progress": 0.42|null
  }
}
```

## Stats

`GET /api/stats/overview`
```jsonc
{
  "totals":   {"plays": 0, "watch_s": 0, "active_users": 0, "distinct_items": 0},
  "previous": {"plays": 0, "watch_s": 0, "active_users": 0, "distinct_items": 0} | null, // same-length window before; null when days=0
  "daily": [ {"date": "2026-01-31", "plays": 3, "watch_s": 5400,
              "by_type": {"Movie": [1, 5000], "Episode": [2, 400], "Audio": [0,0], "Other": [0,0]}} ], // [plays, watch_s]; gap-free, oldest first
  "library": {"movies": 0, "series": 0, "episodes": 0, "tracks": 0, "size_bytes": 0, "users": 0}
}
```
When the window is longer than 120 days `daily` is bucketed per ISO week; each entry then
carries the Monday date and `"bucket": "week"` is set at top level (otherwise `"day"`).

`GET /api/stats/top?kind=<kind>&limit=10`: `kind` ∈ `movies | series | music | users | clients | devices | libraries`
```jsonc
{"rows": [ {"id": "…"|null, "name": "…", "sub": "2019"|null, "plays": 12, "watch_s": 3600,
            "users": 3,               // distinct users (absent for kind=users)
            "image_item_id": "…"|null, "last_played": 1790000000} ]}
```
Sorted by `watch_s` desc unless `&sort=plays`.

`GET /api/stats/heatmap` → `{"plays": [[24 ints] × 7], "watch_s": [[24 ints] × 7]}`: outer index 0 = Monday, inner = hour of day (server TZ).

`GET /api/stats/playback`
```jsonc
{
  "methods":           [{"name": "DirectPlay", "plays": 0, "watch_s": 0}],
  "transcode_reasons": [Bucket], "hw_accel": [Bucket],
  "video_codecs": [Bucket], "audio_codecs": [Bucket], "resolutions": [Bucket],
  "video_ranges": [Bucket], "containers": [Bucket], "audio_channels": [Bucket],
  "clients": [Bucket], "subtitles": [Bucket]   // subtitle language, "None" when off
}
// Bucket = {"name": "hevc", "plays": 0, "watch_s": 0}; sorted by plays desc, max 12, tail folded into "Other"
```

## Activity

`GET /api/activity?page=1&per_page=50&q=&method=&type=&item_id=&series_id=&source=&deleted=` (+ common filters)

`method`, `type` and `source` each take several comma-separated values: `type=Movie,Episode` is films
and episodes and no music. `type` also takes `Other`, which is anything that is not `Movie`,
`Episode` or `Audio`, so `type=Movie,Other` means either, and naming all four is no filter at all.
`source` is `live`, `jellystat` or `streamystats`; anything else, an empty value included, is ignored,
so an unknown tracker means all of them. `sources` in the answer lists which of the three this caller's history actually came from,
in that order. They are scoped to whose plays they may see, and deliberately *not* narrowed by the window or
the other filters, so a filter built from it does not appear and disappear as the days change. The UI
offers the filter only when there is more than one to choose between.

`deleted=1` lists the trash instead (2.2.0): the plays a manager deleted in the last 30 days, scoped and filtered
exactly like the list itself (without `see_everyone`, one's own), newest deletion first, each row with `deleted_at` and
`purge_at` (when it goes for good). The normal answer carries `in_trash` (how many of the plays this list would show are
in the trash) for `manage` only; a play in the trash is out of every other answer (totals, charts, titles, people,
recap, public profiles) until it comes back.
```jsonc
{"total": 2918, "page": 1, "per_page": 50, "sources": ["live", "jellystat"], "in_trash": 0, "rows": [Play]}

Play = {
  "id": 123, "source": "live" | "jellystat" | "streamystats", "active": false,
  "user_id": "…", "user_name": "…",
  "item_id": "…", "item_name": "…", "item_type": "Episode",
  "series_id": null, "series_name": null, "season_number": null, "episode_number": null,
  "image_item_id": "…", "item_exists": true,     // false → item no longer in library, don't link
  "started_at": 0, "ended_at": 0, "duration_s": 0, "paused_s": 0,
  "position_s": 0|null, "runtime_s": 0|null, "completion": 0.93|null,
  "client": "…", "device_name": "…", "app_version": "…", "remote_ip": "…"|null,
  "play_method": "…", "container": "mkv"|null,
  "video": "HEVC 1080p SDR"|null, "audio": "AAC 2.0 jpn"|null, "subtitle": "eng"|null
}
```

`GET /api/activity/{id}` → `Play` plus
`{"device_id", "bitrate", "video_codec", "width", "height", "video_range", "bit_depth", "audio_codec", "audio_channels", "audio_language", "subtitle_codec", "subtitle_language", "transcode": {…same as Session.transcode, plus "bitrate","width","height","audio_channels"} | null}`

🔒 `PUT /api/activity/{id}` `{"deleted": true | false}` → `{"ok": true, "id": 123, "deleted": true, "deleted_at": 0|null, "purge_at": 0|null}`
(`manage`). Moves a finished play into the trash, or back out of it; it is removed for good 30 days after it was
deleted (`purge_at`), with its timeline. Asking for the state it is already in changes nothing and answers it (a second
delete keeps the first `deleted_at`). `404` for no such play, a play still running, or (to restore) one that is not in
the trash. Whoever watched with it is grouped again without it, and with it once it is back. `DELETE` answers `405`.

## Users

`GET /api/users` (non-admin: only self)
```jsonc
{"users": [ {"id","name","is_admin","is_disabled","removed","has_image",
             "last_login_at","last_activity_at","plays","watch_s","last_played_at",
             "last_item_name","last_client"} ]}
```

`GET /api/users/{id}`
```jsonc
{
  "user": {…as above},
  "totals": {"plays","watch_s","distinct_items","movies","episodes","tracks"},
  "daily": [...same as overview.daily],  "bucket": "day",
  "heatmap": {"plays": [[…]], "watch_s": [[…]]},
  "top_series": [TopRow], "top_movies": [TopRow],
  "clients": [Bucket], "methods": [Bucket],
  "devices": [{"device_id","device_name","client","app_version","plays","last_seen"}],
  "ips": [{"ip","plays","first_seen","last_seen","is_local"}]   // 🔒 admins only, else []
}
```

## Libraries & items

`GET /api/libraries`
```jsonc
{"libraries": [ {"id","name","collection_type","removed","item_count","series_count","episode_count",
                 "size_bytes","plays","watch_s","last_played_at"} ]}
```

`GET /api/libraries/{id}` → `{"library": {…}, "top": [TopRow], "recently_added": [ItemCard], "daily": [...], "bucket": "day"}`

`ItemCard = {"id","name","type","year","sub","image_item_id","date_created"}`

`GET /api/items/{id}`
```jsonc
{
  "item": {"id","name","type","year","overview":null,"genres":[…],"community_rating","official_rating",
           "runtime_s","premiere_date","date_created","library_id","library_name","removed",
           "series_id","series_name","season_number","episode_number",
           "container","size_bytes","bitrate","video":"HEVC 1080p"|null,"audio":null,"path":"…"|null /* 🔒 */,
           "has_image": true /* own primary image; an episode without one shows its series' poster */, "has_backdrop": true},
  "totals": {"plays","watch_s","users","last_played_at"},
  "watchers": [{"user_id","user_name","plays","watch_s","last_played_at"}],
  "seasons": [ {"id","name","season_number","episodes":[{"id","name","episode_number","runtime_s","plays","watch_s"}]} ], // Series only, else []
  "daily": [...], "bucket": "day"
}
```
Recent plays for an item come from `/api/activity?item_id=` (episode/movie) or `?series_id=`.

`GET /api/search?q=att&limit=12` → `{"items": [ItemCard], "users": [{"id","name"}], "people": [Person]}`: items limited to Movie/Series/MusicAlbum/Audio top-level hits.
`Person = {"id","name","has_image","is_actor","is_director","titles"}`: cast and crew of titles that are still in the library (at most 8; `titles` counts those
titles, and breaks ties: more first). Matched like titles, by name only. Open to everyone signed in; `users` needs `see_everyone`.

## Images

- `GET /api/img/item/{id}?kind=primary|backdrop&w=300`: proxied + cached from Jellyfin. `404` when Jellyfin has none.
- `GET /api/img/user/{id}?w=96`

A picture finstats knows the Jellyfin image tag of is cached under that tag and answered with `ETag` and
`Cache-Control: private, no-cache`: the browser asks again every time and gets `304` while the picture is the same.
A poster replaced in Jellyfin has a new tag, and so is fetched and sent again. The tags are read by the library read
and, in between, by the task `sync_changes` (Metadata changes, below).
Without a known tag (a person's portrait, say) a picture is cached as before: `private, max-age=604800`, no `ETag`.

Both send long-lived `Cache-Control`. Use as `<img loading="lazy">` with an `onerror` fallback.

## Admin 🔒

`GET /api/settings`
```jsonc
{"jellyfin_url","server_name","server_version",
 "allow_user_login": false,        // let non-admin Jellyfin users sign in and see their own stats
 "active_interval_s": 1,           // session polling while something plays, 1..60 (replaced poll_interval_s in 0.10)
 "idle_interval_s": 5,             // …and while nothing does, 1..60
 "follow_jellyfin_scan": true,     // since 2.0.4 only the default of the library read's triggers (see Scheduling)
 "sync_interval_h": 6,             // …the same, when not following Jellyfin's scan
 "merge_window_s": 600,            // resume the same play if it restarts within this window
 "min_play_s": 0,                  // stats ignore plays shorter than this
 "public_url": ""}                 // where finstats answers from outside; only used to put a link in notifications (administrators only, like the access keys)
```
`PUT /api/settings`: partial object of the mutable keys above (not `jellyfin_url`/`server_*`) → full settings.

`GET /api/tasks`
```jsonc
{"tasks": [ {"id": "sync_users" | "sync_libraries" | "sync_events" | "import",
             "state": "idle" | "running" | "ok" | "error",
             "message": "Fetching items 4,500 / 13,900", "progress": 0.32|null,
             "started_at","finished_at","error": null} ],
 "collector": {"connected": true, "last_poll_at": 0, "active_sessions": 1, "error": null},
 "db": {"size_bytes","plays","items","oldest_play_at"}}
```
`POST /api/tasks/{id}/run` (not for `import`, `import_streamystats` or `restore`) → `202 {ok:true}`; `409` if already running.

### Scheduling (2.0.4)

Every job that runs by itself carries triggers, the way Jellyfin's scheduled tasks do. `GET /api/tasks` answers
`"time_zone": "Europe/London"` (the zone times of day are in: finstats' `TZ`) and, on every task:

```jsonc
{"schedulable": true,              // false for import, import_streamystats and restore: each needs a file
 "runnable": true,                 // POST /api/tasks/{id}/run starts it
 "triggers": [{"type": "after_scan"}, {"type": "interval", "every_s": 604800}],
 "custom": false,                  // false: these are the defaults, from the settings before 2.0.4
 "next_at": 0 | null,              // the soonest trigger that can be known in advance
 "can": {"after_scan": true, "limit": true}}
```

A trigger is one of `{"type": "daily", "at_min": 180}` (minute of the day, 0..1439), `{"type": "weekly", "day": 0, "at_min": 180}`
(day 0 = Sunday .. 6), `{"type": "interval", "every_s": 3600}` (300 .. 30 days, counted from the end of the last run),
`{"type": "startup"}` and `{"type": "after_scan"}` (when Jellyfin's *Scan Media Library* has finished since the job last started;
only `sync_libraries`, `sync_userdata`, `sync_changes`). Any of them may carry `"limit_s"` (60 .. 7 days): a run that takes longer is
stopped and fails with that reason (not for `backup` or `geoip`, which cannot be stopped half way). At most 16 per job, no duplicates.

| | |
|---|---|
| `PUT /api/tasks/{id}/triggers` | `{"triggers": [...]}` → the task as above. An empty list: only by hand. `400` for a trigger the job cannot take, `404` for an unknown job. `manage`. Audited as `task_schedule_changed`. |
| `DELETE /api/tasks/{id}/triggers` | Back to the job's defaults. |

The triggers live in the settings as `"schedules": {"backup": [...]}` and `PUT /api/settings` checks them by the same rules.
A job without an entry runs on its defaults, which is what `follow_jellyfin_scan`, `sync_interval_h`, `backup_every_d` and
`geoip_download` say; those settings are no longer shown, and decide nothing else. Defaults: users, server log, server
details, Sonarr/Radarr calendars and history every 15 minutes, Seerr requests every 5, metadata changes every hour, the library
read and watched flags after Jellyfin's scan plus every 7 days (or every `sync_interval_h` hours when not following it), backups
every `backup_every_d` days, the GeoIP download daily (it fetches only when a newer month is out) when `geoip_download` was on.
A library read due while Jellyfin is scanning waits for the scan to end. When each job last finished is kept (`task_runs`), so a
start does not run everything again.

**Metadata changes** (`sync_changes`, 2.0.4): per library,
`GET /Items?ParentId=…&MinDateLastSaved=…` with the library read's own fields plus `People`, from the last look (else the last
library read) minus ten minutes. Each title is stored as the library read stores it (names, overviews, genres, ratings, file
details, image tags), and the cast and crew of films and shows are replaced. It never marks anything removed: only a whole
library read may. Then the people: `GET /Items?IncludeItemTypes=Person&Recursive=true&MinDateLastSaved=…` (ids and portrait tags,
1,000 a page, since after a scan Jellyfin has re-saved thousands), because replacing a portrait re-saves the person and none of the
titles they are in; the first look after upgrading reads every person once (settings key `portraits_read` marks it done).
A portrait's tag is kept on every `item_people` row of that person (migration 29), so `/api/img/item/{person id}`
is cached under it like a poster. The UI asks for pictures with `v=3` (`v=2`, an earlier step of 2.0.4, still let a portrait be kept for a week).

`POST /api/import/jellystat`: the **raw request body** is the `.jsonl` (or legacy `.json`) backup
(`Content-Type: application/octet-stream`; can be hundreds of MB, so use XHR for upload progress).
Returns `202 {ok:true}` once the upload is stored; parsing continues as task `import`
(poll `/api/tasks`). On finish the task `message` summarises, and `result` holds
`{"plays_imported","plays_skipped","users","libraries","items","seasons","episodes","item_info"}`.

`POST /api/import/streamystats`: the same, for a Streamystats backup (its **Settings → Backup &
Import → Download Backup**, one `.json` document). Task `import_streamystats`; `result` holds
`{"sessions_read","plays_imported","plays_skipped","marked_watched","users","unreadable_rows"}`,
where `marked_watched` counts rows that were never a play: Jellyfin reported the item watched and
Streamystats wrote a row as long as the whole runtime for a viewing nobody saw. Those are never
imported. A Streamystats export carries no libraries, items or users of its own.

**Tautulli** (Plex's history) imports in two steps, because its people are not Jellyfin's: nobody is
guessed, the owner wires each Plex user to a Jellyfin user. All four need `manage`.

`POST /api/import/tautulli`: the **raw request body** is the backup (Tautulli's `.db`, or the `.zip`
holding it). Nothing is imported: the file waits for its wires (one at a time; a new upload replaces
it) and the answer is the board:

```jsonc
{"board": {
  "plex_users": [{"id": 101, "name": "Robin", "plays": 12, "first_at": 0 | null, "last_at": 0 | null}],
  // plays: viewings that would come in (films and episodes); a user with none is still listed
  "jellyfin_users": [{"id": "…", "name": "alice", "has_image": true}]
}}
```

Never a Plex token or e-mail address: the backup holds both, and they are never read. `400` for an
empty file or one that is not a Tautulli database; `409` while an import or a restore runs.

`GET /api/import/tautulli` → `{"board": … | null}`: the board of a backup still waiting for its wires.
`DELETE /api/import/tautulli` → `{ok:true}`: put it away, removing the file.

`POST /api/import/tautulli/run` with `{"wires": [{"plex_user_id": 101, "jellyfin_user_id": "…"}]}` →
`202 {ok:true}`, then task `import_tautulli`, whose `result` holds
`{"plays_imported","plays_skipped","not_wired","other_media","users_wired"}`. `not_wired`: viewings
of Plex users without a wire, which stay behind; `other_media`: music, clips, photos and Live TV,
which are not imported. A Plex user may have one wire; several may go into one Jellyfin user. `400`
for no wire, a Plex user wired twice, or either end naming somebody who is not there; `409` when no
backup is waiting. The file is removed when the import ends, whatever its outcome, and also at
start-up and after six hours of waiting.

**Titles the library does not have under that name.** All three need `manage`.

`GET /api/library/missing` → every film, video and episode plays point at that the library lacks (usually from an
import, whose server named things differently): films by name first, then episodes by show, season and number:

```jsonc
{"missing": [{"id": "plex:5", "item_type": "Movie" | "Video" | "Episode", "name": "Star Wars: Episode V - The Empire Strikes Back (1980)",
              "series_name": null, "season": null, "episode": null, "plays": 2, "last_at": 0,
              "sources": ["tautulli"],      // where its plays came from, in the order Activity's tracker filter lists them
              "suggestions": [ /* up to 3 candidates, likeliest first; see below */ ]}]}
```

`GET /api/library/missing/{id}/candidates?q=` → `{"candidates": [{"id", "item_type", "name", "year", "series_name", "season",
"episode"}]}`, at most 8: without `q`, worked out from its names (fuzzy, either way round, by Jellyfin's original title
too, and for an episode by its own name wherever a guide moved it); with `q`, every film and episode matching what was
typed. `404` when nothing is missing under that id.

`POST /api/library/locate` with `{"from": "plex:5", "to": "<item id>"}` → `{"moved": 2}`: the plays of `from` become plays
of `to`, named, typed and placed as the library has it (a film may be located as a show's special, and the other way
round), under the same rule for a play the history already has as re-linking. The choice is kept, so plays arriving under
`from` again (a re-import) are attached by it. `404` when `from` is not missing or `to` is not in the library, `400`
when `to` is a whole show or season. Recorded in the audit log as `title_located`.

**One import runs at a time**, any kind: they write to the same tables, so whichever is not running
answers `409` while another is.

Re-importing the same backup is safe: a play is recognised by the tracker's own id for it and,
failing that, by the same person watching the same item with either end of the play within
`merge_window_s` of one already here. That second rule is what lets a Jellystat export and a
Streamystats export of the same evenings both be imported without counting anything twice.

`GET /api/events?page=1&per_page=50&q=&type=` → Jellyfin server activity log
```jsonc
{"total", "page", "per_page",
 "rows": [{"id","date","name","overview","type","severity","user_id","user_name","item_id"}]}
```

## Light status (any signed-in user)

`GET /api/summary` → `{"active_sessions": 1, "plays_total": 2918, "last_sync_at": 0, "collector_ok": true, "version": "0.1.0"}`. Cheap; poll for the status bar.

---

# v0.2 additions

## Richer plays

The collector now keeps a timeline per live play and counts interruptions.

- `Play` rows (list + detail) gain `"pause_count": 0`, `"seek_count": 0`, and (admins only, else `null`)
  `"is_local": true|false|null` (LAN / remote, derived from the IP).
- `GET /api/activity/{id}` additionally returns `"start_position_s": 0|null` (where playback resumed from) and
  ```jsonc
  "events": [ {"at": 1790000000, "kind": "start"|"pause"|"resume"|"seek"|"audio"|"subtitle"|"transcode"|"stop",
               "position_s": 512|null,
               "detail": null | "12:40 → 31:05" | "EAC3 5.1 eng" | "Off" | "Transcode: ContainerNotSupported"} ]
  ```
  Oldest first. Empty for imported plays: a tracker stores one row per play and nothing of what happened during it.

## `GET /api/stats/insights` (common filters)

```jsonc
{
  "concurrency": {"peak": 4, "peak_at": 1790000000|null, "peak_transcodes": 2,
                  "series": [{"date": "2026-01-31", "peak": 3}], "bucket": "day"|"week"},   // gap-free, same bucketing as overview.daily
  "network": [Bucket],            // names "Local" / "Remote" / "Unknown"; [] for non-admins
  "data_bytes": 1234567890,       // estimated bytes sent to clients (stream bitrate × time watched)
  "genres": [Bucket],             // by watch time; episodes count towards their series' genres; max 12 + "Other"
  "client_methods": [ {"client": "Jellyfin Web", "direct_play": 10, "direct_stream": 2, "transcode": 30, "watch_s": 0} ], // plays; sorted by total desc, max 12
  "completion": [ {"name": "Under 10%", "plays": 0}, {"name": "10–50%", ...}, {"name": "50–90%", ...}, {"name": "Finished (90%+)", ...} ], // movies + episodes; fixed order
  "behaviour": {"plays_measured": 120, "avg_pauses": 1.4, "avg_seeks": 0.8, "resumed_share": 0.31}, // live plays only; plays_measured = 0 → hide
  "failed_logins": [ {"date": 0, "overview": "…", "user_name": null} ]   // 🔒 newest 10 in window; [] for non-admins
}
```

## `GET /api/library/insights?library_id=`: what the library is made of (no time window)

```jsonc
{
  "totals": {"files": 0, "size_bytes": 0, "runtime_s": 0, "movies": 0, "series": 0, "episodes": 0, "tracks": 0},
  "resolutions": [LibBucket], "video_codecs": [LibBucket], "video_ranges": [LibBucket],
  "containers": [LibBucket], "audio_codecs": [LibBucket],
  "genres": [LibBucket],          // movies + series, size_bytes omitted (0)
  "decades": [LibBucket],         // name "1990s", oldest first
  "added": [ {"month": "2026-01", "count": 12} ],   // last 24 months, gap-free, oldest first
  "largest": [LibItem],           // 15 biggest movies / series (series = sum of episodes)
  "unwatched": {"count": 0, "size_bytes": 0, "items": [LibItem]}   // never played by anyone, per finstats history AND Jellyfin's own played flags; 25 biggest
}
// LibBucket = {"name", "count", "size_bytes"};   sorted by count desc (decades/added excepted), max 12 + "Other"
// LibItem   = {"id","name","type","year","size_bytes","date_created","image_item_id"}
```

## `GET /api/libraries/{id}/titles?type=&q=&sort=&dir=&page=`: everything in a library

Every film, show, album, music video, video, book or audiobook in one library, sixty a page, not only what arrived
lately. Episodes, seasons and tracks belong to one of these and are not titles of their own here. The library is the
same for everyone, so this is open to anyone signed in and says nothing about plays.

```jsonc
{ "total": 1246, "page": 1, "page_size": 60,
  "types": [ {"type": "Movie", "count": 1240}, {"type": "Video", "count": 6} ],   // what the library holds, whatever the filter
  "items": [ {"id", "name", "type", "year", "date_created", "has_image", "image_item_id", "runtime_s", "album_artist",
              "size_bytes",          // a show's is its episodes' files
              "episodes"} ] }        // a show's episodes that are files, specials left out; null for anything else
```

`type` narrows to one kind, `q` keeps the titles whose name has every word typed. `sort` is `name` (default, case
ignored), `year`, `added` or `size`; anything else is by name.

## `GET /api/server` 🔒: the Jellyfin server itself

```jsonc
{
  "fetched_at": 0|null,           // null → not fetched yet (run task sync_server)
  "info": {"server_name","version","operating_system","architecture","has_update_available","has_pending_restart",
           "transcoding_temp_path","cache_path","program_data_path","log_path","encoder_location"} | null,
  "storage": [ {"label": "Shows" | "Program data" | "Cache" | "Transcodes" | …, "path", "free_bytes", "used_bytes", "kind": "library"|"system"} ], // [] on servers older than 10.11
  "plugins": [ {"name","version","status","description"} ],
  "scheduled_tasks": [ {"name","category","state","last_result": "Completed"|"Failed"|…|null,"last_run_at": 0|null,"last_duration_s": 0|null} ],
  "devices": [ {"device_id","name","app","app_version","last_user_id","last_user_name","last_seen"} ]   // newest first
}
```
New task ids in `/api/tasks`: `sync_server` (server info, plugins, tasks, devices) and `sync_userdata` (per-user played / favourite flags). Both runnable via `POST /api/tasks/{id}/run`.

## Small additions to existing responses

- `GET /api/users/{id}` gains `"genres": [Bucket]` and `"jellyfin": {"played_movies": 0, "played_episodes": 0, "favorites": 0} | null` (Jellyfin's own flags; covers history from before finstats).
- `GET /api/items/{id}` → `item` gains `"studios": ["…"]`, `"external": [{"label": "IMDb", "url": "https://…"}]`, `"bit_depth"`, `"framerate"`; top level gains
  `"played_by": [{"user_id","user_name","last_played_at": 0|null,"is_favorite": false}]` (Jellyfin's played flags; admins see everyone, others only themselves).

---

# v0.3: Recap (the year in review)

`GET /api/recap?year=2026`: a recap is personal; everyone, administrators included, only ever gets their **own**.
`year` is a calendar year (server TZ) or `last12` (the last 12 full months plus the current one).
Default: the year whose recap is "ready": the current year during December, otherwise the previous year
(2026 becomes the default in December 2026 and stays it until December 2027). If that year has no plays
(a new install), the newest year that has.

```jsonc
{
  "years": [2026, 2025],                 // years that have plays in this scope, newest first
  "year": 2026 | "last12", "from": 1790000000, "to": 1790000000,   // [from, to)
  "scope": {"user_id": "…", "user_name": "…", "server_name": "…"},          // always the signed-in user
  "empty": false,                        // true → nothing was played in this period; all lists empty

  "totals": {"plays", "watch_s", "distinct_items", "movies", "episodes", "tracks",   // plays per type
             "series_count", "active_days"},
  "rank": {"position": 2, "of": 8, "share": 0.31} | null,  // by watch time among users that played anything (no names)

  "top_series": [RecapTitle],  "top_movies": [RecapTitle],  "top_tracks": [RecapTitle],   // up to 5 each, by watch time
  "top_genres": [Bucket],                // up to 6, by watch time, no "Other"
  // RecapTitle = {"id","name","sub","image_item_id","plays","watch_s","episodes": 12 /* distinct episodes, series only, else null */, "item_exists": true}

  "months": [ {"month": "2026-01", "watch_s", "plays",
               "top": {"id","name","image_item_id","watch_s"} | null} ],   // every month of the period, oldest first; top = most watched series-or-movie
  "hours": [24 × watch_s], "weekdays": [7 × watch_s /* 0 = Monday */],

  "persona": {"key": "night_owl" | "early_bird" | "weekend_warrior" | "binge_watcher" | "movie_buff" | "music_lover" | "creature_of_habit",
              "title": "Night owl", "line": "43% of the watching happened after 22:00"},

  "records": {                           // any entry may be null
    "biggest_day":    {"date": "2026-03-14", "watch_s", "plays"},
    "biggest_binge":  {"date", "series_id", "series_name", "image_item_id", "episodes", "watch_s"},
    "longest_streak": {"days": 9, "from": "2026-02-01", "to": "2026-02-09"},
    "longest_play":   {"item_id", "name", "image_item_id", "duration_s", "date"},
    "most_rewatched": {"id", "name", "type", "image_item_id", "plays"},      // a movie or episode played on ≥ 2 different days
    "first_play":     {"item_id", "name", "image_item_id", "at": 1790000000},
    "oldest_title":   {"id", "name", "year": 1957, "image_item_id"}
  },

  "discovery": {"new_series": 14,                                 // shows whose first ever play falls in the period
                "one_and_done": [{"id","name","image_item_id"}],  // up to 5 shows with exactly one episode started, ever
                "finished_movies": 31, "finished_episodes": 402}, // ≥ 90% complete

  "clients": [Bucket]                    // top 3
}
```

---

# v0.4: Patch notes

`GET /api/changelog` (any signed-in user): `CHANGELOG.md`, compiled into the binary and parsed.

```jsonc
{
  "current": "0.4.0",                       // the running version
  "releases": [                             // newest first
    {"version": "0.4.0", "date": "2026-09-19" | null, "summary": "One paragraph." | null,
     "started": "2026-09-17" | null,        // the first day, for a release made over several; `date` is the day it was released
     "groups": [ {"kind": "Added" | "Changed" | "Performance" | "Stability" | "Fixed" | "Removed", "items": ["One change. May contain **bold** and `code`."]} ]}
  ]
}
```

---

# v0.5: Permissions

`user` (from `/api/auth/login`, `/api/auth/me`, `/api/setup`) gains
`"permissions": {"see_everyone": false, "see_network": false, "see_server": false, "manage": false}`, all `true` for
Jellyfin administrators. They are evaluated on every request, so a change applies at once.

What each one gates, server-side:

| Permission | Effect |
|---|---|
| *(none)* | Every list and statistic is pinned to the caller's own `user_id` (a `user_id` filter is ignored); `/api/users` returns only them; `/api/users/{other}` → `403`. |
| `see_everyone` | `user_id` filters are honoured, `/api/users` and `/api/users/{id}` for anyone, all sessions in `/api/now-playing`, users in `/api/search`, everyone in `watchers` / `played_by`, the lists in `/api/stats/files`. |
| `see_network` | `remote_ip`, `device_id`, `is_local` in plays and sessions; `ips` on user pages; `network` in insights; IP search in `/api/activity?q=`. Otherwise `null` / `[]`. |
| `see_server` | `/api/server`, `/api/events`, `failed_logins` in insights, `item.path`. Otherwise `403` / `[]` / `null`. |
| `manage` | `/api/settings`, `/api/tasks*`, `/api/import/*`, `PUT /api/activity/{id}`, `in_trash` in `/api/activity`. Otherwise `403` / `null`. |
| *Jellyfin administrator* | Not a permission but the account flag: `/api/audit`, everyone's keys in `/api/keys` (and revoking them), `/api/permissions*`, backups, connections. |

`/api/recap` is never widened: it is always the caller's own. `PUT /api/settings` rejects `allow_user_login` and
`default_permissions` with `403` unless the caller is a Jellyfin administrator.

## Managing permissions: Jellyfin administrators only (🔒 `403` for everyone else, including `manage`)

`GET /api/permissions`
```jsonc
{"available": [{"key": "sign_in" | "see_everyone" | "see_network" | "see_server" | "see_downloads" | "notify" | "manage", "label": "…", "description": "…"}],
 "defaults": ["sign_in"],                  // what every non-admin has; "sign_in" here = sign-in is open to everyone
 "users": [{"id","name","is_admin","is_disabled","has_image","permissions": ["see_everyone"]}]}   // own grants only, defaults not included
```
`PUT /api/permissions/defaults` `{"permissions": [...]}` → `{"defaults": [...]}`
`PUT /api/permissions/users/{id}` `{"permissions": [...]}` → `{"permissions": [...]}`; `400` for an unknown key or an
administrator (they already have everything), `404` for an unknown user. Effective = defaults ∪ own grants.

---

# v0.6: Profiles, show progress, manual marks, streaks

`GET /api/users/{id}/shows`: own id, or anyone's with `see_everyone` (`403` otherwise). All time; no filters.
```jsonc
{
  "editable": true,                       // true only on your own profile
  "streaks": {"longest": {"days": 29, "from": "2026-04-17", "to": "2026-05-15"} | null,
              "current": {"days": 3, "since": "2026-09-17" | null, "includes_today": true},   // still alive if it reached yesterday
              "active_days": 189},
  "shows": [ {"id", "name", "year", "removed", "image_item_id", "last_played_at": 0|null,
              "total": 26, "seen": 25, "started": 0,
              "seasons": [ {"season_number": 1, "total": 12, "seen": 12,
                            "episodes": [ {"id", "episode_number": 1|null, "name",
                                           "state": "seen" | "started" | "none",
                                           "source": "played" | "jellyfin" | "manual" | null} ]} ]} ]
}
```
Only episodes that exist as files count: Jellyfin's virtual items (missing or unaired episodes) and specials
(season 0) are left out, so an announced season does not drag a finished show below 100%. For a series that has
left the library its removed episodes are used instead. An episode is `seen` when a recorded play reached 80%
(`played`), Jellyfin has it marked as played (`jellyfin`), or the user marked it (`manual`), in that order of
precedence; `started` = played, but not that far. Shows with nothing seen or started are omitted.
The same `streaks` object is also part of `GET /api/users/{id}`.

`POST /api/me/seen` `{"item_ids": ["…"], "seen": true|false}` → `{"changed": 12}`: marks episodes as seen for the
caller only (1–5000 ids; non-episodes are ignored). Marks live in finstats alone: nothing is written to Jellyfin,
and `seen: false` only removes manual marks, never a recorded play.

`Play` rows were already carrying `position_s` and `runtime_s`; the Activity table now shows them as the stop position.

---

# v0.6.1: Recap for administrators

`GET /api/recap?year=&user_id=`: `user_id` is honoured for **Jellyfin administrators** only and selects one other
user's recap (`scope` then names them). Everyone else always gets their own, whatever permissions they hold
(`see_everyone` included); the parameter is ignored rather than refused. The whole server's year came in 2.0 (`scope=server`, below).

---

# v0.7: Group watching

Plays of one item by at least two different users that start within `group_window_s` (setting, default 60, 5–600) of
each other and overlap for at least two minutes share a `group_id`. Jellyfin's sessions do not expose SyncPlay groups,
so this is inferred; it is recomputed for an item whenever one of its plays ends, after an import, at start-up, and for
the whole history when the setting changes.

- `Play` rows gain `"group_id": 123|null` and `"group_size": 3|null` (distinct people).
- `GET /api/activity/{id}` gains `"watched_with": [{"user_id","user_name"}]`: the other people in the group.

`GET /api/stats/groups` (common filters; without `see_everyone` only groups the caller was part of; with a `user_id`
filter only that person's groups)
```jsonc
{
  "totals": {"sessions": 150, "together_s": 0, "person_s": 0, "people": 3},
  // together_s: per session, how long at least two people were watching (the second-longest stay), summed.
  // person_s: everyone's watch time in those sessions, summed.
  "companions": [ {"members": [{"user_id","user_name","has_image"}], "sessions", "together_s", "last_at"} ],  // by exact set of people, top 8
  "titles":     [ {"id","name","image_item_id","sessions","together_s"} ],                                   // series or movie, top 8
  "recent":     [ {"item_id","item_name","item_type","series_id","series_name","season_number","episode_number","image_item_id",
                   "started_at","together_s","members": [{"user_id","user_name","has_image","duration_s"}]} ]   // newest 10
}
```
`GET/PUT /api/settings` gains `"group_window_s": 60`.

`Session` (from `GET /api/now-playing`) gains `"group": {"size": 2, "with": [{"user_id","user_name"}]}` when other
people are playing the same title right now, having started within `group_window_s` or being within
`max(group_window_s, 30)` seconds of the same position; absent otherwise. It is computed before the list is narrowed
to the caller, so someone without `see_everyone` still sees who they are watching with. `size` counts everybody
(the stream's own person included); `with` names at most six of the others, so a crowd on one title
costs a few names per stream instead of the whole crowd.

---

# v0.7.5: Forgiving search

`GET /api/search?q=` matches word by word instead of by exact phrase: every word of `q` has to be found in the title
(any order; case, accents, punctuation and a leading article ignored; one slip allowed in words of 4–7 letters, two in
longer ones; music also matches on its artist). Results are ranked, best first. The `q` filters of `/api/activity` and
`/api/events` are word-by-word as well: each word must occur in at least one searched column.

---

# v0.8: Recap with days, people, rewatches

`GET /api/recap` gains, all within the same period and scope as the rest of the response:

```jsonc
{
  "totals": { …, "movie_watch_s": 0, "episode_watch_s": 0, "track_watch_s": 0 },
  "genres": {"count": 16, "plays": 480, "watch_s": 0} | null,   // distinct genres; plays and time of titles that have any genre
                                                                  // (top_genres[0].watch_s / genres.watch_s = the top genre's share)
  "people": {                                                     // most watched cast and crew, 5 each, by watch time
    "actors":    [{"id": "…", "name": "…", "has_image": true, "plays": 0, "watch_s": 0, "titles": 3, "top_title": "…"}],
    "directors": [ … same shape … ]
  },
  "rewatch": {"sittings": 0, "rewatches": 0, "share": 0.12} | null,
  "days": [{"date": "2026-03-14", "plays": 3, "watch_s": 0}]      // only days with plays, oldest first, server TZ
}
```

- **People** come from the films and shows themselves (an episode counts towards its show's cast). Only actors (the
  first 12 billed per title) and directors are kept. A person's portrait is `GET /api/img/item/{person id}`.
  They are read with the library, in a second, smaller request per library (`IncludeItemTypes=Movie,Series`,
  `Fields=People`); until the first library read after upgrading, both lists are empty.
- A **sitting** is one film or episode on one local day (plays of at least 5 minutes). A **rewatch** is a sitting with
  something that already had one in the period: `rewatches = sittings − distinct titles`, `share = rewatches / sittings`.
  Picking a play up again on the same day is not a rewatch.

---

# v0.8.1: People

`GET /api/items/{id}` gains `"people": [{"id","name","kind": "Actor"|"Director","role": "…"|null,"has_image": true}]`:
directors first, then the cast in billing order. An episode or season answers with its show's. Empty until the
library has been read by 0.8.0 or newer.

`GET /api/people/{id}` (common filters): one actor or director. `404` for an id nobody in the library carries.

```jsonc
{
  "person": {"id": "…", "name": "…", "has_image": true, "is_actor": true, "is_director": false, "titles": 13},
  "totals": {"plays": 0, "watch_s": 0, "users": 0, "titles_watched": 0, "last_played_at": 0|null},
  "titles": [{"id","name","type": "Movie"|"Series","year","removed": false,"kinds": "Actor"|"Director"|"Actor,Director",
              "role": "…"|null,"plays": 0,"watch_s": 0,"last_played_at": 0|null}],   // every title they are in; most watched first
  "watchers": [{"user_id","user_name","plays","watch_s","last_played_at"}]
}
```

Everything counted is within the caller's scope, exactly like `/api/items/{id}`: without `see_everyone` the totals,
per-title figures and `watchers` cover the caller's own plays only. A play of an episode counts towards its show's
people; a play counts once even when the person both acts in and directs the title.

---

# v0.9: Sorting the paginated lists

`GET /api/activity` and `GET /api/events` take `sort` and `dir` (`asc` | `desc`, default `desc`). Rows without a value
for the sorted column come last in either direction, and the default order is always the tiebreaker, so pages stay stable.
An unknown `sort` is ignored (default order), never an error.

| Endpoint | `sort` values | Default order |
|---|---|---|
| `/api/activity` | `when`, `user`, `title` (show name for episodes), `watched`, `progress` (the same rule as `completion`), `client`, `method`, `ip` (ignored without `see_network`) | newest first |
| `/api/events` | `when`, `event`, `type`, `user` | newest first |

Every other table is sorted in the browser (`web/assets/js/tables.js`); those endpoints are unchanged.

---

# v0.9.1: Home network

`is_local` on plays and on a user's address list now means "a private address **or** a known home address". Home
addresses are this network's public IP (looked up, every one ever seen) plus any added by hand; changing either
re-decides `is_local` for the whole history.

`GET/PUT /api/settings` gain `"public_ip_lookup": true` and `"home_addresses": ["203.0.113.7"]` (IP addresses only, at
most 50; anything else is a `400`). The response also carries, read-only:

```jsonc
"known_home_addresses": [{"ip": "203.0.113.7", "source": "lookup"|"manual", "first_seen": 0, "last_seen": 0}],
"public_ip_services": ["https://checkip.amazonaws.com", "…"]      // who is asked, in order
```

---

# v0.10: Backups, and the polling intervals

`poll_interval_s` is gone from the settings. In its place: `"active_interval_s": 1` (while something plays) and
`"idle_interval_s": 5` (while nothing does), both 1..60. New: `"backup_every_d": 7` (0 = off, 0..365) and `"backup_keep": 5` (1..100).

All of the following are for **Jellyfin administrators** (`403` otherwise): a backup is everyone's history, and a restore
can bring permissions back. `{name}` must look exactly like `finstats-backup-YYYYMMDD-HHMMSS.jsonl.gz`; anything else is a `404`.

| | |
|---|---|
| `GET /api/backups` | `{"backups": [{"name","size_bytes","created_at"}], "deleted": [{"name","size_bytes","created_at","deleted_at","purge_at"}], "scheduled": true, "keep": 5, "next_at": 0\|null}`, newest first; `deleted` is the trash, newest deletion first, and never counts toward `keep`. `scheduled` and `next_at` come from the `backup` task's triggers (2.0.4; `every_d` is gone). |
| `POST /api/backups` | Start writing one now. `202`; progress is task `backup` in `/api/tasks`. `409` while one is running. |
| `GET /api/backups/{name}` | The file (`application/gzip`, `Content-Disposition: attachment`), streamed. |
| `PUT /api/backups/{name}` | `{"deleted": true\|false}` → `{"ok": true, "name", "deleted", "deleted_at": 0\|null, "purge_at": 0\|null}`. Moves the file into the trash (`<data>/backups/deleted/`) or back; it is removed for good at `purge_at`, 30 days on. A file in the trash cannot be downloaded or restored from (`404`). `404` when there is no such file where it is asked to move from, `409` when restoring it would replace a backup that has taken its name since. `DELETE` answers `405`. Backups older than `keep` are still removed for good, not moved to the trash. |
| `POST /api/backups/{name}/restore?settings=true` | Restore a stored backup. `202`; task `restore`. |
| `POST /api/backups/restore?settings=true` | The same from an uploaded file: raw request body, no size limit. |

`settings` (default `true`) also restores the settings and the permissions; `false` merges history only. The finished
`restore` task carries `result: {"plays_imported","plays_skipped","events","other_rows","settings_restored","from_version"}`.

**The file** is gzip-compressed JSON Lines. Line 1: `{"finstats_backup": 1, "app_version", "created_at", "server_name", "counts": {table: rows}}`.
Every other line: `{"t": "<table>", "r": {column: value}}` for `settings` (the one settings row), `playbacks`, `playback_events`,
`manual_seen`, `user_permissions`, `home_addresses`, `server_events`, `devices`, `security_alerts`, `audit`, `watchlist`. Rows are matched by column name in both
directions, so backups move between versions. Never in it: the Jellyfin address and API key, sessions, the library.
Restoring merges: a play already present (same `source_id`, or same user, item and start) is skipped with its timeline;
restored plays are never `active`, and groups, local/remote and library links are worked out again afterwards. A watchlist
entry is merged by its title per person (the same item, or the same kind and any one id), keeping the older `added_at`.
CLI: `finstats backup`, `finstats restore <file>`.

# v1.1: Timeline

`GET /api/users/{id}/timeline?before=&limit=24&libraries=`: own id, or anyone's with `see_everyone` (`403` otherwise; `404`
for an unknown user). All time, newest first; `min_play_s` applies.
```jsonc
{
  "user": {"id", "name", "has_image"},
  "libraries": [{"id", "name", "collection_type"}],   // the ones this person has played from, for the filter
  "next": "1789675170.3245" | null,                    // pass as `before` for the next page; null = the history ends here
  "stops": [ {"kind": "season"|"album"|"item", "type": "Episode", "id": "<series, track or item id>", "name": "The Rookery",
              "sub": "Season 2" | "<year>" | "<album artist>" | null, "image_item_id",   // the season's poster if it has one, else the show's
              "from": 0, "to": 0,                        // start of the oldest play, end of the newest
              "plays": 4, "titles": 4, "watch_s": 5520, "active": false,   // titles = different episodes/tracks; active = still playing
              "episode_from": 1, "episode_to": 4} ]    // only when the episodes are an unbroken run, else null
}
```
A stop is a run of plays that follow each other and belong together: episodes of one season of one show, tracks of one album,
or one title played again. Anything else in between starts a new stop, so a show can appear many times. A page never ends in
the middle of a stop, and the stops are the same whatever `limit` (1..60) is. `libraries` is a comma-separated list of library
ids (absent = all); ids that are not ids and cursors that are not cursors are a `400`.

# v1.1.2: Recently added

`GET /api/library/recent?limit=30` (1..60), for everyone signed in: the library is the same for all, nothing here is about plays.
```jsonc
{ "items": [
  {"kind": "episodes", "type": "Episode", "id": "<series id>", "name": "The Rookery", "sub": "Season 4" | "3 seasons" | "Specials" | null,
   "image_item_id",                       // the season's poster when it is one season with a poster, else the show's
   "added_at": 0,                         // the newest of them
   "day": "2026-09-18",                   // the local day they were added, which is what folds them
   "episodes": 3, "seasons": 1,
   "episode_number": 7 | null, "episode_name": "…" | null},   // only when it is a single episode
  {"kind": "item", "type": "Movie" | "MusicAlbum" | "Video" | "MusicVideo" | "Book" | "AudioBook", "id", "name",
   "sub": "<album artist>" | "<year>" | null, "image_item_id", "added_at": 0, "day": "2026-09-18"}
] }
```
Newest first. Episodes of one show added on the same local day are one entry, however many seasons they span, so a season pack
or a whole imported show takes one place. Series, seasons and single tracks are never entries; removed items are left out.

---

# v1.2: Security, places, impossible travel

Addresses get a place (country, city, a city-centre coordinate) from a city database read locally (`<data dir>/geoip/*.mmdb` or
`FINSTATS_GEOIP_DB`); nothing is asked of anyone per lookup. Everything here needs **both** `see_network` and `see_everyone`
(`403` otherwise); the writes also need `manage`. Without a database the reads still answer, with `"database": null` and empty lists.

`GET /api/security?days=&user_id=`
```jsonc
{ "database": {"kind": "DBIP-City-Lite", "built_at": 0, "dbip": true, "file": "dbip-city-lite-2026-09.mmdb"} | null,
  "can_manage": true, "home_known": true, "open_alerts": 2, "addresses_without_place": 0, "days": 30,
  "places": [ {"label": "Paris, France", "home": false,        // home: every play from the home network, placed where its public address is
               "city", "region", "country", "country_code": "FR", "latitude": 48.85, "longitude": 2.35,
               "plays": 3, "watch_s": 0, "sign_ins": 1, "addresses": 1, "last_seen": 0,
               "users": [{"id", "name", "plays", "sign_ins"}]} ],
  "countries": [{"code": "FR", "name": "France", "plays": 3, "users": 1}],
  "failed": [ {"label", "country_code", "latitude", "longitude", "attempts": 9, "last_at": 0, "events": ["Failed login attempt from admin"]} ],
                                                               // failed sign-ins from outside: only with `see_server`, never with `user_id`
  "now_playing": [ {"user_id", "user_name", "item_name", "series_name", "device_name", "home": true, "place", "latitude", "longitude"} ] }
```
Places count plays in the window plus successful sign-ins (`AuthenticationSucceeded` in Jellyfin's activity log).

`GET /api/security/alerts?status=open|resolved|all&user_id=&page=&per_page=` (default `open`, 25 per page, at most 100)
```jsonc
{ "total": 3, "open": 2, "page": 1, "per_page": 25,
  "rows": [ {"id": 1, "kind": "impossible_travel" | "new_country", "severity": "high" | "medium",
             "user_id", "user_name", "has_image", "at": 0,
             "resolved_at": 0 | null, "resolved_by": "alice" | "finstats" | null, "note": "…" | null, "muted": false,
             "details": {
               // impossible_travel
               "from": {"place", "country_code", "latitude", "longitude", "at", "until", "what", "ip", "home"}, "to": {…},
               "pair": "home|40.7,-74.0", "distance_km": 5570, "gap_s": 1200, "overlap": false, "speed_kmh": 16711 | null,
               // new_country
               "to": {…}, "country": "France", "country_code": "FR", "known": 1 } } ] }
```
A *sighting* is a play (for as long as it ran) or a sign-in / new session in the activity log. `impossible_travel`: two sightings of one
person at least `travel_min_km` apart that overlap (`overlap`, no speed) or would need more than `travel_speed_kmh`; one alert per pair
of places per day. `new_country`: the first sighting in a country once the person has a history. Alerts found more than 30 days after
the fact (an import, the first database) are filed as resolved by `finstats`. Alerts are part of backups.

- `POST /api/security/alerts/{id}/resolve` `{ "note": "…"?, "mute": false }` → `{ "ok": true, "also_resolved": 0 }`. `mute` (impossible travel
  only) also resolves the open alerts for the same two places and stops that pair from reporting for this person again.
- `POST /api/security/alerts/{id}/reopen` → `{ "ok": true }` (clears the note and the mute).
- `POST /api/security/alerts/resolve-all` → `{ "ok": true, "resolved": 5 }`.
- `POST /api/security/database` (`manage`) → `202`; downloads DB-IP's free city database as task `geoip` (`409` while it runs, `400` when
  the file is set with `FINSTATS_GEOIP_DB`).

`GET/PUT /api/settings` gain `"geoip_download": false` (fetch that file now and monthly), `"travel_speed_kmh": 900` (100..5000) and
`"travel_min_km": 500` (50..5000). Read-only in the response:
```jsonc
"geoip": {"database": {…} | null, "folder": "/data/geoip", "from_env": false, "source": "https://download.db-ip.com/free/dbip-city-lite-YYYY-MM.mmdb.gz"}
```

---

# v1.2.2: Languages

Every audio and subtitle track of a file is kept, not only the first: ISO 639-2 codes as Jellyfin reports them, lower case, each once,
in track order, `"und"` for a track without a language. Filled by the library read (and by a Jellystat import for files it describes; a Streamystats export describes none).

- `GET /api/items/{id}`, for a film, episode or other file: `item.audio_languages: ["jpn","eng"] | null` and `item.subtitle_languages`.
  A series or a season has no tracks of its own and gets
  `item.language_coverage: {"episodes": 26, "audio": [{"code": "jpn", "episodes": 26}, {"code": "eng", "episodes": 13}], "subtitles": [...]}`
  over its episodes that exist as files (absent when there are none). Each row of `seasons[].episodes[]` gains `audio_languages`.
- `GET /api/library/insights` gains `audio_languages` and `subtitle_languages`: `[{"name": "jpn", "count": 29, "size_bytes": 0}]`, video
  files that have a track in the language (a file with two languages counts in both), at most 12 and then `"Other"`.

---

# v1.3: Connections (Sonarr, Radarr, Seerr)

finstats reads from these services and never changes anything in them. **Jellyfin administrators only** (`403` for everyone else, including
`manage`). A key or password is write-only: the API says `has_secret`, never the value, and none of this is part of a backup.

`GET /api/services`
```jsonc
{ "services": [ {"id": 1, "kind": "sonarr", "label": "Sonarr", "name": "Sonarr", "url": "http://192.168.1.10:8989",
                 "has_secret": true, "accept_invalid_certs": false, "enabled": true,
                 "version": "4.0.9" | null, "last_ok_at": 0 | null, "last_error": "Sonarr refused the API key" | null} ],
  "kinds": [ {"key": "sonarr" | "radarr" | "seerr", "label", "what", "example"} ] }
```
Every write answers with the same list.

- `POST /api/services/test` `{kind, url, secret?, accept_invalid_certs?}` or `{id, …}` (whatever is left out is taken from the stored
  connection, so a key need not be retyped) → `{"ok": true, "app": "Sonarr", "version": "4.0.9"}`, or `502` with a sentence that says what is wrong.
- `POST /api/services`: the same body plus `name?`. The connection is tested first and only saved when it answers (`502` otherwise). A missing
  name becomes the kind's, then "Radarr 2". At most 20.
- `PUT /api/services/{id}`: any of `name`, `url`, `secret`, `accept_invalid_certs`, `enabled`; the kind never changes. Changing what
  is needed to connect tests again. Pointing a connection at another host, port or base path throws away everything read from the old one.
- `DELETE /api/services/{id}`: also removes everything that was read from it.

Addresses: `http://` or `https://`, a base path is kept (`/sonarr`), no `user:password@`, `?` or `#` (`400`). finstats follows no redirect to a
service: a redirect is reported as the error it is. `accept_invalid_certs` switches certificate verification off for that one connection.

`GET /api/auth/me` → `user.features: {"upcoming": true, "requests": true}`: which optional pages have a connected service behind them.

## Upcoming (Sonarr, Radarr)

Read every 15 minutes (task `sync_upcoming`, which `POST /api/tasks/sync_upcoming/run` starts by hand): monitored episodes and film releases from
7 days back to 90 days ahead. For everyone signed in: a calendar is about the library, like Recently added.

`GET /api/upcoming?days=14&user_id=&mine=`: `days` 1..90 from today (local days); `user_id` is whose shows "follow" refers to (the caller's own
without `see_everyone`, whatever is asked); `mine=true` keeps only what that person follows.
```jsonc
{ "days": 14, "user_id": "<whose>", "entries": [
  {"kind": "episode" | "movie", "release": "air" | "cinema" | "digital" | "physical",
   "day": "2026-09-25",                 // local day; a film only has a day (Radarr's midnight UTC is not a moment)
   "at": 0 | null,                      // episodes: when it airs
   "series_title": "Low Orbit" | null, "title": "Re-entry", "season": 3, "episode": 10, "finale": "season" | "series" | "midseason" | null, "year": 2026 | null,
   "has_file": false,                   // already downloaded
   "item_id": "<series or film in the library>" | null,
   "poster": {"item_id": "…"} | {"service_id": 1, "media_id": 15},   // the second kind is served by /api/img/arr
   "you_follow": true,                  // that person played an episode of the show in the last 120 days (never true for a film)
   "followers": 2, "follower_names": ["alice", "bob"]                  // only with `see_everyone`; otherwise the keys are absent
  } ] }
```
The same episode in two Sonarrs, or the same film in an HD and a 4K Radarr, is one entry (on disk if it is anywhere). A `physical` release of a
film that is already on disk is left out, because a disc date for a copy that arrived weeks ago is no news; its other dates, and the disc date of a film
that is not here yet, are listed as before. Switched-off connections say nothing. Without `see_everyone` there is no count either: on a small server a number is a name.

- `GET /api/img/arr/{service_id}/{media_id}?w=`: the poster of a title that is not in the library yet, proxied from Sonarr or Radarr and cached on
  disk. Both ids are numbers; only posters of titles finstats itself lists are served (what is on a calendar, what somebody asked for, and, for
  people with `see_downloads`, what is downloading now), and the browser never talks to TMDB.
- `GET /api/items/{id}` gains `item.upcoming` for a series or film with something due in the next 90 days: entries as above, without any of the
  keys about people.

## Requests (Seerr)

Read every 5 minutes (task `sync_requests`), and a pass that has nothing to read costs one row: it asks Seerr for its
newest-modified request, and stops there when that is one it already knows. What is still on its way is looked at again
every quarter hour (a request's own record does not always change when its media arrives), and everything is listed once
a day. Everyone signed in sees **their own** requests; other people's need `see_everyone`, and
without it no count, name or "somebody else watched it" is sent either. A request Seerr cannot tie to a Jellyfin user belongs to nobody and
is only visible to those who may see everyone.

`GET /api/requests?status=&user_id=&q=&sort=&dir=&page=&per_page=`: `status`: `open` | `arrived` | `declined` | all (default); `q` matches the
title; `sort` is one of `when` (default), `title`, `user`, `state`, `arrived`, `watched`; at most 100 per page.
```jsonc
{ "rows": [ {"id": "3:41",                       // <connection>:<Seerr request>
             "media_type": "movie" | "tv", "title": "Low Orbit" | null, "year": 2021, "tmdb_id": 871002,
             "seasons": [1, 2], "is_4k": false,
             "state": "pending" | "approved" | "processing" | "partial" | "available" | "declined" | "failed" | "removed",
             "requested_at": 0, "available_at": 0 | null, "arrived_after_s": 7200 | null,
             "user_id": "…" | null, "user_name": "alice" | null, "has_image": true,
             "item_id": "…" | null, "poster": {"item_id": "…"} | {"service_id": 2, "media_id": 21} | null,
             "watched": true, "plays": 2, "first_play_at": 0 | null,          // by the person who asked, after they asked
             "watched_by_anyone": true, "plays_by_anyone": 3}                 // only with `see_everyone`
          ],
  "total": 42, "page": 1, "per_page": 25 }
```
"Watched" counts plays of the requested title that started at or after the request, are longer than `min_play_s` (at least two minutes), and
for a series only in the seasons that were asked for.

`GET /api/requests/summary?user_id=&days=` (`days` 0 = all time)
```jsonc
{ "days": 0,
  "totals": {"requests": 42, "titles": 39,      // one film asked for in HD and in 4K is one title
             "open": 4, "arrived": 30, "watched": 21, "watched_by_anyone": 24},   // the last only with `see_everyone`
  "median_arrive_s": 7200,
  "trend": [{"month": "2026-09", "arrived": 6, "median_s": 5400}],               // at most 12 months, by the month it was asked in
  "never_played": [ /* rows as above: arrived over two weeks ago, nobody has watched it, one line per title */ ],
  "people": [{"user_id": "…" | null, "user_name": "alice", "requests": 12, "arrived": 10, "watched": 7}]   // only for `see_everyone`, and only when not asking about one person
}
```

`GET /api/items/{id}` gains `item.request`: the oldest request for that title, but only the caller's own unless they have `see_everyone`;
otherwise the key is absent, "arrived after" included.

## Downloads (Sonarr and Radarr)

Live, from memory: the queues of every connected Sonarr and Radarr. They already talk to the download client, whichever it is, and report a
torrent and a usenet download the same way, so finstats reads them rather than each client's own API. Nothing is stored. Needs the
permission **`see_downloads`** (`403` without it; Jellyfin administrators always have it).

`GET /api/downloads?live=1`: `live=1` means "a page is showing this": the snapshot is then refreshed every 5 seconds for the next 20.
While nobody is looking it is read every 60 seconds if there is anything in the queue, and every 5 minutes if there is not. Opening the page,
connecting a service or a read of Seerr all refresh it at once. The prefetcher never asks for it.
```jsonc
{ "rows": [ {"key": "<download id>:<service>" | "arr:<service>:<title>:<sub>",
             "title": "Low Orbit", "sub": "Season 3 · 3 episodes" | "2026" | null,
             "state": "failed" | "importing" | "downloading" | "stalled" | "queued" | "paused" | "checking" | "unknown",
             "progress": 0.66, "size": 9000000000, "left": 3000000000, "eta_s": 1300,
             "down_bps": 8400000,                              // worked out from what moved since the last reading, 0 when it cannot be
             "release": "Low.Orbit.S03.1080p.WEB-DL" | null,   // what the release is called
             "client": "qBittorrent" | null,                   // the client Sonarr or Radarr handed it to
             "protocol": "torrent" | "usenet" | null, "service_name": "Sonarr" | null,
             "error": "One file was not imported" | null,         // Sonarr's or Radarr's words, with any login or key in an address blanked
             "poster": {"item_id": "…"} | {"service_id": 1, "media_id": 12} | null,
             "item_id": "…" | null,
             "requested_by": {"user_id": "…" | null, "user_name": "maria"} | null} ],
  "totals": {"down_bps": 0, "downloading": 3, "queued": 1, "importing": 1, "failed": 0},
  "at": 0, "sources": 2,
  "problems": [{"service": "Radarr", "error": "…"}] }
```
Several records with one download id are one row (a season pack, and the episodes it holds); the same id in two instances is two rows,
because each is waiting for its own copy. Worst first: what needs attention is on top. At most 500 rows.

**Without `see_downloads`** a person still learns how far their *own* request has got: rows of `GET /api/requests` and
`item.request` of `GET /api/items/{id}` carry `"download": {"state": "downloading", "progress": 0.66, "eta_s": 1300}` when something in the
queue is that title, and nothing else: no release name, no client, no speed, no other download.

`GET/PUT /api/permissions` gain `see_downloads`.

`GET /api/downloads/history?days=30` (1..3650, `see_downloads`): Sonarr's and Radarr's own history, read every 15 minutes (task
`sync_grabs`; the first read goes back a year, afterwards only what is new). Only `grabbed`, `imported` and `failed` events are kept;
renames, deletions and ignores say nothing about what arrived.
```jsonc
{ "days": 30,
  "totals": {"imported": 144, "grabbed": 154, "failed": 10, "size_bytes": 0},
  "daily": [{"day": "2026-09-20", "imported": 3, "size_bytes": 0, "failed": 1}],       // gap-free local days
  "indexers": [{"name": "A Tracker", "count": 64, "size_bytes": 0}],                   // imports only, at most 12 each
  "quality": [...], "clients": [...], "protocols": [...],
  "failures": [{"at": 0, "title": "Low Orbit", "source": "Low.Orbit.S03E08.1080p", "indexer": "A Tracker", "media_type": "tv"}] }
```

---

# v1.5: Live session tracking

The collector is *told* when something starts instead of asking for it: finstats keeps one WebSocket open to Jellyfin's `/socket`. There
is no setting: it is how the collector works, and `active_interval_s` / `idle_interval_s` are what it asks at, plus the fallback for a
socket that is not carrying. Nothing else about the API changes: the same rows, the same `/api/now-playing`, only sooner.

Each transport does the half it is good at. **Nothing playing:** finstats listens and asks for nothing at all. Jellyfin sends a session
list when something changes and nothing in between, so a quiet server is a quiet socket and not a broken one. **Something playing:**
finstats reads `/Sessions` every `active_interval_s`, because a pause, a seek or a track change is only as sharp as the gap between two
sightings, and because the push carries no `ActiveWithinSeconds`; asking is what ends a play whose client vanished. **Everything paused:**
back to listening after three readings in a row of it, since a frozen position is nothing for a server to report; a single `/Sessions` read stands as a net
(after a minute of silence, and in any case every five minutes), and anyone starting again is pushed and answered within about a
second. Silence means *nothing heard and nothing asked*: a push, a read of finstats' own and a fresh subscription all start the minute
again, the read the net itself calls for included, and two reads are never closer together than five seconds. So a pause that follows
three minutes of playing asks for nothing at all for the next minute, however long ago the last push was. Measured from the last push
alone, as an earlier attempt did, the clock is already stale the moment the subscription comes back on, since it is off for the whole of a
play: a pause after a minute of one asked at once, and again, and again, 2,625 times in eight seconds until a push happened along. A server that answers the subscription with
nothing at all is believed as long as it answers keep-alives: one read says where things stand, and that is the mode. So `transport` reads `poll` while
something is actually running and `socket` otherwise, with `socket_live` true throughout.

Jellyfin is subscribed to whenever nothing is actually running (idle or all-paused alike) and never while something is, which is the one
moment its pushes would be a second copy of what is already being asked for. Jellyfin normally answers `SessionsStart` at once, whether or not anything is loaded. A server that does
not is polled meanwhile, but not written off: the connection is kept and stays subscribed, `SessionsStart` is re-sent every five minutes,
and the first real push settles it. Neither that silence nor an app left open with nothing playing is ever evidence against the socket. The two never
run at once: finstats sends Jellyfin `SessionsStop` for as long as it is polling (otherwise the same list would arrive twice, and the
pushed copy is not compressed) and subscribes again on the pass where the last play ends.

A socket that closes, says nothing at all for 90 s (not even an answer to a keep-alive), or never answers `SessionsStart` with a first
session list drops finstats back to polling at `active_interval_s` / `idle_interval_s` on the next pass, and it keeps trying to reconnect.

`GET /api/tasks` → `collector` gains:
```jsonc
{ "connected": true, "last_poll_at": 0, "active_sessions": 1, "error": null,
  "transport": "socket" | "poll",        // how the list arrived last
  "socket_error": "Jellyfin said nothing for 90s" | null,   // why the socket is not carrying; null while it is
  "socket_live": true,                   // the socket is open and carrying, whatever brought the last list
  "session_mode": "idle_socket" | "playing_poll" | "paused_socket" | "fallback",
  "socket_subscribed": true }            // Jellyfin is being asked to push session lists right now
```
`GET /api/status` carries the same picture, unauthenticated, so that what the collector is doing can be checked from
outside without reading a log:
```jsonc
{ "configured": true, "server_name": "Home Cinema", "version": "1.5.0",
  "session_mode": "idle_socket" | "playing_poll" | "paused_socket" | "fallback",
  "socket_connected": true,        // a WebSocket that is open and answering
  "socket_subscribed": true,       // SessionsStart sent on *this* connection, no SessionsStop after it
  "poll_interval_s": null,         // the beat actually being asked at; null while nothing is asked for
  "sessions_requests_last_min": 0, // /Sessions reads in the last 60 s, counted where they go out
  "mode_since": "2026-09-22T18:16:27Z" }
```
These always hold, and finstats logs a warning about itself if they ever stop: listening means connected, subscribed and
`poll_interval_s: null`; `playing_poll` means *not* subscribed and a beat that is running; `fallback` always has a beat. A
safety read is not a beat, so it does not appear. `sessions_requests_last_min` is about 60 while something plays and at most 2
while listening, counted at the one gate every `/Sessions` read passes through, so it is what a packet capture counts and not
what the collector believes it asked for; that gate also refuses more than 2 reads in a second or 70 in a minute, whatever asks
it, and says so in the log at most once a minute. Shape only (no names, no titles, not even how many sessions there are),
and answered from memory: no request to Jellyfin, no query.

`GET /api/summary` gains `collector_live` (`socket_live`), for the status bar.

`POST /api/settings/public-ip` 🔒: look this network's public address up **now**, and answer like `GET /api/settings`. This is the only
thing that asks after the first answer: the lookup no longer runs on the 15-minute timer, only once at start-up on an install that has
never learned an address, when `public_ip_lookup` is switched on, and when this is called. `400` when the setting is off.

`GET /api/outbound` 🔒: every destination finstats can reach, for the **Outbound connections** card. Read from what is already kept;
nothing is recorded for it. Hosts (with ports) only: never a path, never a key.
```jsonc
{ "destinations": [
    {"id": "jellyfin",   "what": "Your Jellyfin server", "hosts": ["jellyfin.example:8096"], "why": "…",
     "state": "always" | "on" | "off",
     "last_at": 0 | null,                  // when it last answered, as far as something already recorded knows
     "error": "…" | null},
    {"id": "public_ip",  "hosts": ["checkip.amazonaws.com", "…"], "state": "on", …},
    {"id": "geoip",      "hosts": ["download.db-ip.com"], "state": "off", …},
    {"id": "service:3",  "what": "Radarr 4K (Radarr)", "hosts": ["nas:7878"], "state": "on", …},
    {"id": "notify:1",   "what": "Household (Discord)", "hosts": ["discord.com"], "state": "on", …} ],   // v1.6: the ones finstats *sends* to
  "reachable": 2, "total": 4 }
```


---

# v1.6: Notifications

Where what finstats finds is sent. A destination belongs either to the **server** (Jellyfin administrators) or to **one
person** (anybody with `notify`), and finstats sends nothing at all until one exists: no destination, no request.

Every route here needs `notify`, which administrators always have. A person sees and may touch only their own
destinations; an administrator sees every one. **A destination's address is never given back**: a Discord webhook URL
carries its own token, so the answer holds the host and a hint (`shown`), and editing without a new `url` keeps the
stored one, the way a service's API key does.

Nine kinds of destination. Eight are one POST of JSON (`webhook`, `discord`, `slack`, `telegram`, `ntfy`, `gotify`,
`pushover`, `pushbullet`); `email` is the one that is not, and goes over SMTP. Three of them (`telegram`, `pushover`,
`pushbullet`) are always reached at their own service's address, which the catalogue gives as `fixed_url` and which
finstats fills in rather than asking for.

`GET /api/notifications`
```jsonc
{ "targets": [
    {"id": 3, "kind": "webhook" | "discord" | "slack" | "telegram" | "email" | "ntfy" | "gotify" | "pushover" | "pushbullet",
     "label": "Discord", "name": "Household",
     "shown": "discord.com/…",            // host, and the topic, chat or mailbox where there is one; never the URL
     "scope": "server" | "me", "owner_id": "…" | null, "owner_name": "bob" | null,   // "me": a personal destination, not necessarily the caller's
     "topic": "finstats-abc" | null,      // whatever that kind calls it: topic, chat id, user key, mailbox
     "options": {"from": "finstats@example.com"},   // what that kind needs beyond those; only email has any
     "has_secret": true,
     "events": ["travel", "new_items"],   // the kinds it asked for
     "with_addresses": false,             // IP addresses and coordinates only when this is on (and, on a personal one, its owner has see_network)
     "min_severity": "info" | "warn" | "alert",
     "accept_invalid_certs": false, "enabled": true, "created_at": 0,
     "last_ok_at": 0 | null, "last_error": "…" | null} ],
  "catalogue": {
    "events": [{"key": "travel", "label": "Impossible travel", "what": "…", "severity": "alert",
                "group": "security" | "housekeeping" | "library" | "playback", "group_label": "Security",
                "personal": true}],       // true = it is about a person, so a personal destination needs the right to see it
    "channels": [{"key": "ntfy", "label": "ntfy", "what": "…",
                  "example": "https://ntfy.sh",        // empty when the address is fixed
                  "fixed_url": null | "https://api.telegram.org",   // filled in, never asked for
                  "needs_topic": true, "topic_label": "Topic" | null, "topic_help": "…", "topic_example": "finstats-abc",
                  "secret_label": "Access token (optional)" | null, "secret_required": false,
                  "extras": [{"key": "from", "label": "From address", "help": "…", "example": "…", "required": true}]}],
    "severities": ["info", "warn", "alert"],
    "groups": [{"key": "security", "label": "Security"}] },
  "public_url": "https://finstats.example",   // empty: messages carry no link
  "can_add_server": true, "max_own": 5 }
```

`POST /api/notifications/targets` → `{"target": {…}}`
```jsonc
{"scope": "server" | "me",          // "server" needs to be an administrator; default: "server" for them, "me" otherwise
 "kind": "discord", "name": "Household",
 "url": "https://discord.com/api/webhooks/…",   // required on create unless the kind has a fixed_url; a plain webhook
                                    // may carry a query, the others may not. Email is smtps:// (465) or smtp:// (587)
 "secret": "…",                     // the bot token, the application token, the mail password; optional Bearer for a webhook or ntfy
 "topic": "finstats-abc",           // the field beside the address: ntfy topic, Telegram chat id, Pushover user key, mailbox
 "options": {"from": "finstats@example.com", "username": "finstats@example.com"},   // email only
 "events": ["travel", "new_country", "request_available"],
 "with_addresses": false, "min_severity": "info", "accept_invalid_certs": false, "enabled": true}
```
`PUT /api/notifications/targets/{id}`: the same object, every key optional; what is left out keeps what is stored, and
`kind` and `scope` never change. `DELETE /api/notifications/targets/{id}` → `{"ok": true}`.

`POST /api/notifications/targets/{id}/test` → `{"ok": true}` or `{"ok": false, "error": "Gotify refused the token"}`:
one message down the same path as every other, so a test that arrives proves the real thing works.

`GET /api/notifications/history?limit=50` (max 200): what has been sent lately. A person sees only what went to their own
destinations.
```jsonc
{ "events": [
    {"id": 91, "kind": "request_available", "label": "A request is watchable", "severity": "info", "at": 0,
     "user_name": "alice" | null, "title": "Ready to watch: Winterline", "body": "…",
     "historic": false,               // found more than six hours late: recorded, never sent
     "deliveries": [{"target_id": 3, "target": "Household", "channel": "discord",
                     "state": "queued" | "sent" | "failed", "attempts": 1, "sent_at": 0 | null, "error": null}]} ] }
```

**What goes out.** A webhook is posted this JSON; Discord and Slack get a card each, ntfy and Gotify their own JSON
(title in the body, never in a header, because a title is a film title), Telegram plain text with no `parse_mode` (a film
title is not markup), Pushover and Pushbullet what they take; email is a plain-text letter whose subject is the title:
```jsonc
{"event": "travel", "severity": "alert", "at": 0, "title": "Impossible travel: alice",
 "body": "Oslo, Norway and London, United Kingdom, 1160 km apart, 40 minutes apart.",
 "link": "https://finstats.example/security" | null,      // only when public_url is set
 "user": "alice" | null,
 "fields": [{"label": "Person", "value": "alice"}],       // addresses and places appear here only with with_addresses
 "source": "finstats/1.6.0"}
```

**The rules the server keeps**, all of them tested:
- an event is written once (deduped like a security alert) and delivered per destination, retried on its own clock
  (30 s, 2 min, 10 min, 1 h, then given up on; a `Retry-After` is honoured), at most 20 messages a minute per destination;
- nothing found more than six hours after it happened is ever sent, and no destination is sent anything that happened
  before it existed, so adding one cannot replay a year;
- a personal destination carries exactly what its owner may see in the app: their own rows always, somebody else's play
  or request needs `see_everyone`, somebody else's places need `see_network` as well, and the server's own business needs
  `see_server`;
- a personal destination's address must not resolve into a private or loopback range, checked when it is saved and again
  before every send. An administrator's may point anywhere;
- mail is encrypted or it does not go: `smtps://` from the first byte, `smtp://` must upgrade with STARTTLS. There is no
  third option, and no path by which the password is sent in the clear. A certificate of your own making is the same
  administrator-only switch (`accept_invalid_certs`) every other connection has.


## `GET /api/jellyfin/jobs` 🔒: what your Jellyfin is doing right now

Needs *see the server*. Jellyfin's own scheduled tasks, read live (at most one read of Jellyfin every 3 seconds however
many people are watching), with the hidden ones included: "what is running" must not leave something out because
Jellyfin's dashboard does not draw it. finstats only ever reads this; there is no way in the code to start, stop or
change a task on Jellyfin.

```jsonc
{ "jobs": [
    {"id": "…", "key": "MediaSegmentDetect", "name": "Detect and Analyze Media Segments", "category": "Library",
     "state": "Running" | "Idle" | "Cancelling", "running": true, "hidden": false,
     "progress": 68.4 | null,               // only while it runs
     "eta_s": 420 | null,                   // only ever a measured estimate; see below
     "watching_since": 0 | null,            // when finstats first saw this run, which may be long after it began
     "unchanged_for_s": 0 | null,           // how long the percentage has been standing still
     "what": "Goes through your episodes looking for the parts a player can offer to skip …",
     "known": true,                         // false: the sentence is Jellyfin's own, or says there is none
     "description": "…",                    // Jellyfin's own words, whatever they are
     "schedule": ["every day at 03:00", "every 12 hours"],
     "next_at": 0 | null,                   // only from an interval trigger, which is measured from the last run
     "last_result": "Completed" | "Failed" | "Aborted" | "Cancelled" | null,
     "last_error": "…" | null, "last_run_at": 0 | null, "last_duration_s": 200 | null} ],
  "running": 1, "fetched_at": 0,
  "error": "Jellyfin did not answer" }       // only when it did not; the jobs are then the last thing it said
```

**`eta_s` is an estimate, and the shape of the answer says so.** Jellyfin reports a percentage and never when the
current run started, so finstats times the run by watching it. The rate is measured against the most recent reading far
enough back to say anything (at least 0.5% ago and at least 5 s ago, within a 15-minute memory), so a job that speeds up
or slows down is described by the pace it has now rather than the one it averaged, and a job creeping a percent every
few minutes is still measurable at all.

**The watching is the backend's, and starts before the first call.** The task lists finstats reads anyway (the
library-scan check every 5 minutes, the server details every 15) notice a run, and from then on finstats reads the whole
list itself every 10 seconds for as long as anything runs (2.0.4), whether or not a page is open. So a page opened on a
running job is answered with an `eta_s` measured over the last minutes, not one it has to watch into being. Idle, nothing
extra is asked of Jellyfin. This endpoint's own read is still at most every 3 s, and a page reading it spares the
backend's. A job Jellyfin reports at 0% the whole time (a library scan's first phase does) has no rate to measure,
and stays `null` until the percentage moves.

**When nothing has been measured, `eta_s` is `null` and stays `null`.** There is a tempting number to put there (how
long the last run took, applied to the fraction that is left), and it is a guess: it knows nothing about how much of
*this* run has already happened, it does not move while the job does not, and on a page it is indistinguishable from an
estimate that was earned. finstats does not offer it. The page shows a cycling ellipsis in place of the number, and
`last_duration_s` sits beside it for anyone who wants to judge for themselves. The page writes "ETA" in front of the
ellipsis, so the dots read as an estimate that cannot be given yet rather than as a page still loading.
`unchanged_for_s` is what makes a slow job legible rather than a broken page, and `watching_since` is when finstats
started watching, not when Jellyfin started the job.

Running jobs come first, then whatever ran most recently.

---

# v1.6.4: Licences

`GET /api/licenses` (any signed-in user): what finstats is built on, and the licence each part is under. It is the
same for every caller and cannot change while the process runs, so it is rendered once at the first call and handed out
unchanged after that; it runs to about half a megabyte of licence text (≈ 55 KB over the wire, compressed).

```jsonc
{
  "version": "1.6.4",                        // the running version
  "components": [
    {"name": "finstats", "version": "1.6.4", "license": "GPL-3.0-only",
     "repository": "https://github.com/finstats/finstats",
     "notices": [253],                       // indices into "notices" below; may be empty
     "kind": "app" | "bundled" | "crate"}
  ],
  "notices": [
    {"file": "LICENSE-MIT", "text": "Permission is hereby granted, free of charge…"}
  ]
}
```

**The texts are deduplicated, not summarised.** Hundreds of crates ship the same MIT wording, so each distinct text
appears once in `notices` and every component points at the ones it carries. Nothing is retyped from memory: each text
is a licence file as its own project wrote it, read out of the crate sources by `tools/make-third-party.py` and
compiled in as `THIRD-PARTY.json`. `kind` separates the three halves: `app` is finstats itself under the GPL, `bundled`
is what is shipped or read but is not a crate (the fonts, the map outlines, the city database), and `crate` is the
generated dependency list. A component with an empty `notices` has no licence file to show; its SPDX `license` is then
all there is to say.

`cargo test` fails while `THIRD-PARTY.json` does not cover every package in `Cargo.lock`, so a dependency cannot be
added without its licence being recorded.

---

# v1.8: Playback insights

What the timeline of every play adds up to. Pure reads over `playbacks` and `playback_events`; nothing new is collected.

## `GET /api/items/{id}` gains `insights` and per-episode `users` / `finished`

A film or episode answers where its plays stopped:

```jsonc
"insights": {
  "runtime_s": 7020,             // the axis: the item's runtime, or the longest a play reported
  "bucket_s": 120,               // the grid: 30 s for a short episode, 120 s for a film, never more than 60 buckets
  "plays": 41,
  "measured": 9,                 // plays that know where they stopped (finstats' own, Streamystats' with a runtime): the stop is that
  "estimated": 32,               // the rest (Jellystat keeps how long it ran): the stop is taken as that, from 0:00
  "curve": [1.0, 0.98, …],       // share still watching at each bucket edge, from the start to the runtime (n + 1 points)
  "rewinds": [0, 0, 3, …],       // per bucket: seeks that landed there from further on (n counts)
  "subtitles": [0, 2, …]         // per bucket: plays whose first subtitle change, to a track, happened there
} | null
```

`null` for anything but a film or an episode, under three plays in scope, or without a runtime to draw on. The stop rule
is the one `completion` has always used, made explicit: `position_s` when finstats saw the play end, else `duration_s`.
The two are never mixed silently: `measured` and `estimated` always travel with the curve. Rewinds and subtitle
switch-ons exist only for plays finstats recorded itself. A seek shorter than 20 s was never recorded, so a short rewind
is invisible by design.

A show's `seasons[].episodes[]` gain `"users": 3` (distinct people who started the episode, in scope) and
`"finished": 2` (plays that stopped at 90 % of the runtime or later). "Everyone quits episode three" is people who
never press play on episode four, which `users` in episode order shows.

`GET /api/activity/{id}` events gain `"from_s": 1632 | null`: a seek's origin as a number beside its label (the
position playback was expected at when it jumped; `position_s` is where it landed). Filled once for every seek already
kept, and by a restore for a backup from before it existed.

## `GET /api/stats/files` (common filters) 🔒 *see everyone*

Files worth a look, from everyone's plays in the window. Without *see everyone* every list is `[]` (200): from one
person's own plays they would be noise, and a title in "files nobody gets into" is a fact about other people's viewing.

```jsonc
{
  "broken": [                    // started three times or more, never past thirty seconds; newest tried first
    {"id": "…", "name": "Broken Reel", "type": "Movie", "series_id": null, "series_name": null,
     "plays": 3, "users": 2, "longest_s": 12, "last_tried_at": 1790000000, "clients": ["Jellyfin Web", "Kodi"]}
  ],
  "rewound": [                   // backwards seeks per play, from plays finstats recorded; two plays and three rewinds at least
    {"id": "…", "name": "Mumbled", "type": "Movie", "series_id": null, "series_name": null,
     "plays": 2, "rewinds": 4, "per_play": 2.0, "hot_s": 1260}   // hot_s: the minute they cluster in
  ],
  "subtitled": [                 // plays whose first subtitle change was to a track within ten minutes; two at least
    {"id": "…", "name": "Clear", "type": "Movie", "series_id": null, "series_name": null,
     "plays": 3, "switched_on": 2, "share": 0.67, "typical_s": 350}
  ]
}
```

The broken list is built **without the minimum play length**: that setting is exactly what a broken file's plays never
reach. Jellyfin's activity log carries no playback errors, so the plays are the only witness. Movies and episodes only,
up to 25 broken and 15 of each of the others.

---

# v1.9: Watched together

`GET /api/stats/groups` (common filters) answers what the **Together** page draws, alongside the keys the dashboard and
profile cards have always read. Nothing new is collected: the sessions are the groups `group_id` already marks.

```jsonc
{
  "totals": { "sessions": 12, "together_s": 61200, "person_s": 130000, "people": 4,
              "watch_s": 400000,            // everything the scoped people watched in the window, in company or not
              "share": 0.31 },              // their time in group sessions ÷ watch_s; null when nothing was watched
  "previous": { "sessions": 9, "together_s": 40000, "people": 3, "share": 0.25 } | null,   // the same window right before; null for all time
  "bucket": "day" | "week",
  "series": [ { "date": "2026-09-20", "together_s": 5400, "alone_s": 7200 } ],   // gap-free, like every series
  "pairs": [ { "members": [{"user_id": "…", "user_name": "alice", "has_image": true}, {"…": "bob"}],
              "sessions": 5, "together_s": 21000, "last_at": 1790000000,
              "top_title": {"id": "…", "name": "Low Orbit", "image_item_id": "…", "sessions": 3} | null } ],   // at most 20
  "people": [ { "user_id": "…", "user_name": "alice", "has_image": true, "together_s": 21000, "alone_s": 60000, "total_s": 81000, "share": 0.259 } ],
  "companions": […], "titles": […], "recent": […]   // as before; a recent session also carries its "group_id"
}
```

Two rules. **A pair's time together is the shorter of the two stays**, which is theirs alone. A session's own
`together_s` (the second-longest stay) describes any two of its members, so an evening of three counts for each of its
three pairs. **A session belongs whole to the bucket it started in**; `alone_s` is what the scoped people watched in that
bucket minus their time inside sessions that started in it, never below zero.

Scoping is the caller's, as everywhere: without *see everyone* the pairs are the caller's own, `people` is the caller
alone (a companion's name is theirs to see; a companion's time alone is not), and `previous` and `series` follow the same
pin. `min_play_s` applies to `watch_s` and the series' totals, not to the sessions themselves, which are at least two
minutes long by construction.

# v1.10: API keys, calendar feed, audit

## API keys

A key is a second credential for the same person: `fs_` + 64 hex characters (67 in all), shown **once** when it is
made and stored only as a hash. It is sent as `Authorization: Bearer fs_…` (scheme case-insensitive, spacing
tolerant) and resolves to exactly the `user` a session would: name, administrator flag and permissions are read live
on every request, so a demoted administrator's key demotes with them and a person who may no longer sign in has no
working keys. A session cookie is read the same way, and one of somebody disabled or deleted in Jellyfin
is `401`. **Header beats cookie**: a request carrying both is judged on the header, and an invalid header is `401`
even with a valid cookie beside it. A key is refused (`401`) once revoked or past its expiry.

Two scopes. `full` opens everything its holder may see. `calendar` opens `GET /api/calendar.ics` and nothing else
(`403` everywhere else); it exists so a phone's calendar can hold a credential that cannot read a single statistic.

| Method | Path | Body | Response |
|---|---|---|---|
| GET | `/api/keys` | – | `{keys: [{id, name, scope, user_id, user_name, has_image, created_at, expires_at, last_used_at, last_used_ip, mine}]}`: the caller's live keys; a Jellyfin administrator's list holds everyone's. Never the key itself. |
| POST | `/api/keys` | `{name (1–60), scope: "full"\|"calendar", expires_in_d?: 1..3650}` | `201 {id, key, name, scope, created_at, expires_at}`, the only time `key` is ever answered. At most 20 live keys per person (`409`). |
| DELETE | `/api/keys/{id}` | – | `{ok: true}`; a soft revoke that stops the key on its next request. Own keys, or anyone's for an administrator; `404` otherwise, and for a key already revoked. |

Both writes need a **session**: a request authenticated by a key cannot mint or revoke keys (`403`), so a leaked key
has no successors. `last_used_at`/`last_used_ip` are written at most once a minute per key.

## The calendar feed

`GET /api/calendar.ics?key=fs_…&days=90&mine=1`: a subscribable iCalendar of what Sonarr and Radarr have coming, the
same rows as `/api/upcoming` for the same person, as far ahead as `days` (1–90, default 90). `mine=1` (or `true`)
keeps it to the shows and films the caller follows.

**This is the one place a credential is accepted in the address.** A subscribed calendar can send no header, so the
feed reads `?key=` (a Bearer header works too); it **never** reads the cookie, so a link cannot open a feed in a
browser that happens to be signed in. A `calendar`-scoped key is meant for it; a `full` key opens it as well.

The answer is `text/calendar; charset=utf-8`, `Cache-Control: private, no-store`, CRLF line ends, folded at 75 octets:

```
BEGIN:VCALENDAR / VERSION:2.0 / PRODID:-//finstats//EN / CALSCALE:GREGORIAN / METHOD:PUBLISH / X-WR-CALNAME:finstats: coming up
BEGIN:VEVENT
UID:{service_id}-{episode|movie}-{external_id}-{release}@finstats     // the upcoming row's own key, stable across syncs
DTSTAMP:…Z
DTSTART:20260921T141320Z            // an episode: its moment, and DURATION:PT1H
DTSTART;VALUE=DATE:20260921         // a film: the whole day, DTEND the next day
SUMMARY:Low Orbit S03E10 · Re-entry   // a film: "Title (2026) · In cinemas" / "· On disc"
CATEGORIES:Episode | Film
DESCRIPTION:Season finale. \n Already here. \n You watch this. \n <public_url>/items/{id}   // what applies; the link only when public_url is set and the title is in the library
URL:<public_url>/items/{id}
END:VEVENT
```

The feed **names nobody by construction**: the query behind it carries no user name or count, only whether the
caller follows the title, because a subscribed calendar syncs through somebody's cloud.

## The audit log: Jellyfin administrators only (🔒)

Every write path in finstats leaves a row: who did it, from where, through which key if any, to what, and how it
went. Reads leave none. `GET /api/audit` pages through it.

| Query | Meaning |
|---|---|
| `page`, `per_page` | as `/api/events` |
| `q` | word-by-word over the user name, the target and the detail |
| `kind` | one of the kinds below; an unknown one answers an empty page |
| `user_id` | rows by one person |
| `sort`, `dir` | `when` (default, newest first), `kind`, `user`, `outcome`; anything else falls back to `when` |

```jsonc
{ "total": 412, "page": 1, "per_page": 50,
  "rows": [ { "id": 9, "at": 1790000000, "kind": "setting_changed", "user_id": "…", "user_name": "alice", "has_image": true,
              "ip": "192.168.1.10", "key_id": null, "key_name": null, "target": null,
              "detail": { "changed": [ { "key": "min_play_s", "from": 60, "to": 90 } ] }, "outcome": "ok" } ],
  "kinds": ["sign_in", "setting_changed", "…"] }   // the kinds present, for the filter
```

Kinds: `sign_in`, `sign_in_failed` (the name as typed, outcome `failed`), `sign_in_refused` (a valid password but no
right to sign in), `sign_out`, `setup_completed`, `key_created`, `key_revoked`, `key_used` (a key's first use),
`setting_changed` (only the keys that changed, with `from` and `to`; settings hold no secret), `permissions_changed`,
`service_added|changed|removed` (kind and name, never the address or key), `target_added|changed|removed` (name,
channel, whose), `backup_made` (no actor when the schedule wrote it) / `backup_restored` / `backup_deleted` /
`backup_downloaded` / `backup_undeleted`, `task_run`, `import_started` / `import_finished` (plays imported and skipped, or the error),
`play_deleted` / `play_undeleted` (title, person, start), `trash_purged` (no actor; how many plays and backups went,
nothing else), `alert_resolved` / `alert_reopened`. A row is kept a year, is written even when the action it records
failed (`outcome: "failed"`), and never fails the action for not being written. The `audit` table is part of
backups; `api_keys` is not.

# v1.11: Public profiles and shareable cards

The first answers given without an account. They are built by `public.rs` from what one person published and nothing
else, and each of them answers the same `404 {"error": "Profile not found"}` when the server switch (`public_profiles`,
off by default, Jellyfin administrators only) is off, the link is unknown or was reset, the profile is unpublished or was
taken down, or its owner was removed, disabled or may no longer sign in. Every figure counts only plays that **ended at
least a day before** (`active = 0`, `ended_at <= now − 86400`), recent plays included. Responses carry
`X-Robots-Tag: noindex, nofollow`.

## Without an account

| Method | Path | Response |
|---|---|---|
| GET | `/u/{token}` | The page, HTML, with its link preview (`og:title`, `og:description`, `og:image` = the profile card; absolute when `public_url` is set). |
| GET | `/u/{token}/card.png?kind=profile\|recap` | A 1200×630 PNG. `profile` needs `totals` or `habits` published, `recap` needs `recap`. `Cache-Control: public, max-age=3600`. |
| GET | `/api/public/{token}` | `{name, avatar, totals?, habits?, recap?, recent?}`; an unpublished section is absent, not empty. |
| GET | `/api/public/{token}/img/{item_id}?w=` | A poster, only for an `image` the answer lists. |
| GET | `/api/public/{token}/avatar` | The owner's picture, only when `avatar` is true. |

`totals`: `{plays, watch_s, movies, episodes, tracks, top_series, top_movies, top_tracks}`, each list up to five
`{name, sub, plays, watch_s, image}`. `habits`: `{longest_streak_days, active_days, heatmap: {plays, watch_s}, genres: [{name,
watch_s}]}`. The grid is `[weekday, Monday = 0][hour]`, the shape `charts.js` draws; there is no current streak.
`recap` (the ready year): `{year, plays, watch_s, active_days, persona: {title, line}|null, top_series, top_movie,
top_genre, longest_streak_days}`, never the rank among other people, the apps or the records. `recent`: up to ten
`{day: "YYYY-MM-DD", name, sub, image}`. `name` is the name the owner typed, `""` for none; never the Jellyfin login name.

## The owner's side (a session; a key is refused `403`)

| Method | Path | Body | Response |
|---|---|---|---|
| GET | `/api/me/public-profile` | – | `{server_enabled, published, url, display_name, show_avatar, sections: {totals, habits, recap, recent}}`; `url` is `null` until the first save. |
| PUT | `/api/me/public-profile` | `{published, display_name (≤ 60), show_avatar, sections}` | The same; the first save mints the link. `409` while the server switch is off. |
| POST | `/api/me/public-profile/reset` | – | The same, with a new `url`; the old link is gone. `404` before the first save. |
| GET | `/api/public-profiles` 🔒 *Jellyfin administrators* | – | `{enabled, profiles: [{user_id, user_name, display_name, published, sections, created_at, updated_at}]}` |
| DELETE | `/api/public-profiles/{user_id}` 🔒 *Jellyfin administrators* | – | `{ok: true}`: unpublished, the owner's choices kept. `404` when it was not published. |

`/api/auth/me` gains `user.features.public_profiles`. Audit kinds: `profile_published`, `profile_changed`,
`profile_unpublished`, `profile_link_reset`. `public_profiles` is not in backups: a restore never brings a link back.

# v2.0: Recap 2026

## `GET /api/recap` grows

Parameters gain `scope`: `user` (default) or `server`. `server` is the whole server's year, **Jellyfin administrators
only** (`403` for anyone else, whatever they were granted; `400` for an unknown scope): every title and total and the
persona of the house, with `rank`, `clients` and `together.companions` taken out and `scope.kind = "server"`.

New keys, each `null` when there is nothing to say:

| Key | Shape |
|---|---|
| `together` | `{evenings, together_s, share, top_title: {id, name, image_item_id, evenings} \| null, companions: [{user_id, user_name, has_image, evenings, together_s}] /* ≤ 3, a person's year only */, people_in_company /* the server's year only */}` |
| `finished` | `{series: [{id, name, image_item_id, episodes, finished_on}], count, dropped: [{id, name, image_item_id, seen, total}], dropped_count}`, a person's year only. Finished: every file episode seen, the last inside the year. Dropped: begun in the year, under half seen, nothing played for 60 days before the year closed (or before now). |
| `requests` | `{made, available, watched, top: [{title, year, item_id, image_item_id}]}`: Seerr requests made in the year; `watched` counts those played by the requester after they arrived. `null` when no request was ever recorded. |
| `versus` | `{year, plays, watch_s, active_days}`: the calendar year before; `null` for `last12` or when that year had no plays. |
| `story` | The chapters that have a card, in order: keys of `year, numbers, shows, films, music, genres, persona, rhythm, days, records, together, finished, asked, versus`. |

## The year as cards

| Method | Path | Response |
|---|---|---|
| GET | `/api/recap/cards/{chapter}?year=&user_id=&scope=` | A 1080×1920 PNG of one chapter, for whoever may open that year. `404` for a chapter the year has no card for. |
| GET | `/api/recap/cards.zip?…` | Every card, `01-year.png` onwards, as one stored ZIP (`Content-Disposition: attachment; filename="finstats-2025.zip"`). |
| GET | `/u/{token}/recap/{chapter}` | The same for a published year, without an account (`Cache-Control: public, max-age=3600`). |
| GET | `/u/{token}/recap.zip` | …and all of them. Both `404` exactly like the rest of `/u/` when the year is not published. |

A card is drawn from a typed copy of the year that has no field for another person, a rank or an app: it never names the
people someone watched with. In the app a card carries no name at all; a published one carries the owner's chosen name, the
server's year the server's name. `GET /api/public/{token}`'s `recap` is now that copy (`{label, whose, totals, top_series,
top_movies, top_tracks, genres, persona, hours, weekdays, days, records, together, finished, asked, versus}`), with `story`
beside it listing its cards.

## Notifications

A new event kind, `recap_ready` (Library group, the person it is about): in December, once per person who watched that year,
"Your 2025 in review is ready" with a link to `/recap?year=2025`.

# v2.0.2: Open in Jellyfin

`GET /api/items/{id}` gains `item.jellyfin_link`: Jellyfin's web app on that title's page
(`{base}/web/#/details?id={id}`), or `null` for a title Jellyfin no longer has. `base` is the setting
`jellyfin_public_url` (where people open Jellyfin, which is often not the address finstats connects to) or,
while it is empty, the address finstats connects to. `jellyfin_public_url` is an http(s) address naming a host
(`400` otherwise) and, being a link everyone follows, only a Jellyfin administrator may set it (`403` for anyone else).


# v2.1: Watchlist

## Your own list

Every endpoint acts on the caller and takes no `user_id`: nobody (not `see_everyone`, not a Jellyfin administrator)
reads or changes somebody else's list. A session or a full key may use them; a calendar key is refused `403`. Nothing is
written anywhere but finstats' own database, and nothing is audit-logged.

| Method | Path | Body | Response |
|---|---|---|---|
| GET | `/api/me/watchlist` | – | `{user: {id, name, has_image}, entries: [Entry]}`, newest first. |
| POST | `/api/me/watchlist` | `{item_id}`, or `{kind, tmdb_id?, tvdb_id?, imdb_id?, title, year?}` | `{id, created}`: `201` when added, `200` when it was on the list already. |
| DELETE | `/api/me/watchlist/{id}` | – | `{ok: true}`; `404` when the entry is not the caller's. |
| GET | `/api/me/watchlist/keys` | – | `{entries: [{id, kind, item_ids, tmdb_id, tvdb_id, imdb_id}]}`, newest first. |

`/api/auth/me` gains `user.features.watchlist: true`; a page offers the list only where it is there.

Only films and shows go on a list: `kind` is `Movie` or `Series`, and an `item_id` that is anything else, or that the
library does not have, is `404`. A title outside the library needs at least one id (TMDB and TVDB ids are digits and may
be sent as numbers, an IMDb id is `tt` and digits) and a name of at most 300 characters; `year`, when given, is
1870–2200. Anything else is `400` with a sentence saying why. If the ids already name a title in the library, the entry
is attached to it at once. A list holds at most 1,000 titles; one more is `400`.

Adding what is already there (the same item, a copy of it in another library, or the same kind with any one of its ids)
answers the entry that is there. `keys` is what a page needs to show the toggle without a request per poster: `item_ids`
is every item in the library that shares one of the entry's ids (a film in an HD and a 4K library is one title), plus the
item it was added from.

`Entry`: `{id, kind, title, year, added_at, item_id, tmdb_id, tvdb_id, imdb_id, state, progress, request, next, poster}`.
`title` and `year` are the snapshot kept with the entry; `item_id` is the title in the library now (the lowest id of its
copies), `null` when it is not there. `state` is worked out when the list is read and never stored; the first that holds:

| `state` | When |
|---|---|
| `watched` | Seen by the profile's rule: a play that reached 80%, Jellyfin's played flag, or a mark by hand. A show when every episode on disk (no specials) is. Stays on the list until its owner removes it. |
| `started` | In the library and begun: a film played but not that far, a show with some episodes seen or begun. |
| `on_server` | In the library, not begun. |
| `requested` | Open in Seerr (pending, approved, processing or partly there). |
| `coming_up` | Sonarr or Radarr has a date in the next 90 days. |
| `left_library` | It was in the library and is not any more. |
| `not_on_server` | None of these. |

`progress` is `{seen, total}` for a show somebody has begun, else `null`; of several copies the one furthest along
counts. `request` is `{by_you: true}` for the caller's own request, and `{by_you: false, user_name}` for somebody else's,
which is only ever sent to somebody with `see_everyone`: without it, another person's request is not mentioned at all,
exactly as on the Pipeline page. `next` is `{day, at, release, season, episode}` from the calendar, `null` without one.
`poster` is `{item_id}` for a title the library has or had, else `{service_id, media_id}` for one on the calendar or in a
request the caller may see (served by `/api/img/arr/…`), else `null`.

Watchlists are part of a backup (they are somebody's own and Jellyfin cannot give them back), and a restore merges them
per person and title, keeping the older date, and attaches them to the library as it is.

## `GET /api/upcoming` entries carry their ids

Every entry gains `tvdb_id` and `tmdb_id` (the show's for an episode, the film's for a release; `null` where the service
has none), so a title that is not in the library yet can be put on a watchlist by them.

## Notifications: `watchlist_available`

A new event kind (Library group): a film or show somebody put on their watchlist has arrived in the library, once per
person and title, only for a title that came after it was put on the list, and nothing older than the last few hours.
It is **only ever somebody's own**: it goes to that person's own destinations and to no other, a server destination
and an administrator's included, and it is left out of `GET /api/notifications/history` for everyone but that person.
It is ticked on no destination until its owner ticks it. The catalogue in `GET /api/notifications` marks such kinds
`own_only: true`; the page offers them only to a destination of one's own.

## Library health 🔒 *see server details*

What is wrong with a file only shows beside its neighbours. finstats works the findings out after every library read
and every look for metadata changes, never on a request, and only reports: nothing here changes anything in Jellyfin.
Nothing in an answer is about plays or people; the one name is who dismissed a finding.

`GET /api/library/health?library_id=` →

```jsonc
{
  "computed_at": 1767225600,        // null until the first library read of 2.2.0 has worked them out
  "total": 12,                      // to look at, dismissed ones left out
  "wasted_bytes": 3000000000,       // what copies of the same thing spend twice, dismissed ones left out
  "kinds": [ {"kind": "gap", "count": 3, "dismissed": 1, "wasted_bytes": 0} ],  // every kind, in the order below
  "libraries": [ {"id": "…", "name": "Shows", "count": 7} ]   // every library with something to look at, whatever library_id says
}
```

`GET /api/library/health/findings?kind=&library_id=&dismissed=&sort=&dir=&page=` → one page of 25:
`{"total", "page", "page_size", "items": [Finding]}`. `dismissed=1` lists the ones set aside instead of the ones to look at.
`sort` is `title` (default), `wasted`, `found` or `kind`; anything else is the default order.

```jsonc
// Finding
{
  "key": "gap:{series id}:1",       // kind and subjects: the same at every recompute
  "kind": "gap", "item_id": "…",    // where a click leads: the film, the show, or the episode
  "title": "Low Orbit", "year": 2020, "image_item_id": "…", "removed": false,
  "libraries": [{"id": "…", "name": "Shows"}],
  "evidence": { … },                // by kind, below
  "wasted_bytes": null,             // copies only
  "found_at": 1767225600,           // first found; a rescan keeps it
  "dismissed": false, "note": null, "dismissed_at": null, "dismissed_by": null,
  "jellyfin_link": "https://jellyfin.example.com/web/#/details?id=…"
}
```

| `kind` | `evidence` |
|---|---|
| `gap` | `{season, from, to, files, missing: [[4, 5], [8, 8]]}`: episode numbers missing between a season's lowest and highest file. A file of several episodes counts whole; season 0 and episodes without a number are left out, and nothing is said about the end of a season. |
| `season_drift` | `{differs: ["resolution", "range", "codec"], seasons: [{season, episodes, resolution, range, codec}]}`: each season's usual look; one finding per show. |
| `episode_drift` | `{season, episode, differs, resolution?, season_resolution?, range?, season_range?}`: a file unlike its own season (a season of at least 4 files, 75% of them alike). Codec mixes inside a season are not findings. |
| `copies` | a film: `{files: [File], resolutions}`; a show: `{episodes, examples: [{season, episode, files: [File]}]}` (at most 20): the same thing twice in one resolution class. `wasted_bytes` is everything but the largest copy. |
| `versions` | a film: `{files: [File], resolutions: ["4K", "1080p"]}`; a show: `{episodes, resolutions}`: copies in different classes, often deliberate; never counted as waste. |
| `thin` | a film: `{resolution, codec, bitrate_bps, threshold_bps}`; a season: `{season, files, of, lowest_bps, highest_bps, episodes: [{id, episode, resolution, codec, bitrate_bps, threshold_bps}]}`. Only the first version of an item is stored, so an item with several is judged by its first. |
| `dub` | `{language, full: [1, 2], none: [3], partial: [{season, episodes, of}]}`: an audio language that covers some seasons fully and others not at all. |
| `unidentified` | `{type, year, path}`: a film or show with no provider id at all. |

`File` = `{id, library_id, resolution, codec, size_bytes, path, season, episode}`. Resolution classes are `4K`, `1080p`,
`720p` and `SD`, by width (a cropped 1920×800 is 1080p) or height (a pillarboxed 1440×1080 is too). Copies follow the
rule the watchlist and Pipeline use: films or shows of one type that share a TMDB, TVDB or IMDb id, unless the ids
among them lead to different titles; a show's episodes are copies through it, by season and episode number.

`POST /api/library/health/dismiss` with `{"key", "note"}` → `{"ok": true}`; `404` when there is no such finding, `400`
for a note over 500 characters. `POST /api/library/health/undismiss` with `{"key"}` → `{"ok": true}`; `404` when it was
not dismissed. Both need *manage finstats* **and** *see server details*. A dismissal holds while the values behind the
finding do: a replaced file or a re-encoded season brings it back if it is still true. Recorded in the audit log as
`finding_dismissed` and `finding_undismissed`. Dismissals travel with a backup and a restore keeps one this database
already has; the findings themselves are worked out again and are not in a backup.

# v2.2: Appearance

How finstats looks is each person's own: a FinUI preset's code as FinUI create makes it
(`https://finui.finstats.no/create/`), one letter or digit per choice, such as `0101`.

`GET /api/me/appearance` → `{"finui_preset": "0101"}` (`""`: FinUI as it ships). `PUT /api/me/appearance` with
`{"finui_preset"}` sets the caller's and nobody else's, and answers the same shape; `""` takes it back; `400` for a code that
names no option. Anyone signed in; recorded in the audit log as `appearance_changed`. It travels in a backup, and a restore
keeps a choice this install already has for the same person.

`GET /assets/finui.css` (no sign-in needed, as before) answers in the look of whoever's session cookie asks: FinUI's
stylesheet, then the preset's tokens (FinUI's own generated file of each chosen option,
`/assets/finui/p/{axis}/{option}.css`, in axis order, after a comment naming the code) with an ETag of its own and
`Cache-Control: private, no-cache`, `Vary: Cookie`. Signed out, or nothing chosen: FinUI as it ships. The choices and their
names are `/assets/finui/create/presets.json`, FinUI's file.

## A title's page in Jellyfin, from anywhere

`/api/auth/me` gains `user.jellyfin_details`: a title's page in Jellyfin is this followed by its id.
It reads like `"https://jellyfin.example.com/web/#/details?id="` and comes from the address an administrator set for "Open in Jellyfin", else the one
finstats connects to; `null` before Jellyfin is set up. The context menu offers "Open in Jellyfin" on any link to a title
with it, where the answer that drew the link had no `jellyfin_link` to give.
