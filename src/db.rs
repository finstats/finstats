use std::path::Path;

use anyhow::{Context, Result, bail};
use r2d2_sqlite::SqliteConnectionManager;
pub use r2d2_sqlite::rusqlite;
use rusqlite::{Connection, OptionalExtension, params};

pub type Pool = r2d2::Pool<SqliteConnectionManager>;

#[derive(Clone)]
pub struct Db {
    pool: Pool,
}

pub(crate) const MIGRATIONS: &[&str] = &[
    // 1 — initial schema
    r#"
    CREATE TABLE settings (
        key   TEXT PRIMARY KEY,
        value TEXT NOT NULL
    ) WITHOUT ROWID;

    CREATE TABLE sessions (
        token_hash TEXT PRIMARY KEY,
        user_id    TEXT NOT NULL,
        user_name  TEXT NOT NULL,
        is_admin   INTEGER NOT NULL,
        created_at INTEGER NOT NULL,
        expires_at INTEGER NOT NULL,
        ip         TEXT,
        user_agent TEXT
    ) WITHOUT ROWID;

    CREATE TABLE users (
        id               TEXT PRIMARY KEY,
        name             TEXT NOT NULL,
        is_admin         INTEGER NOT NULL DEFAULT 0,
        is_disabled      INTEGER NOT NULL DEFAULT 0,
        image_tag        TEXT,
        last_login_at    INTEGER,
        last_activity_at INTEGER,
        removed          INTEGER NOT NULL DEFAULT 0,
        updated_at       INTEGER NOT NULL
    ) WITHOUT ROWID;

    CREATE TABLE libraries (
        id              TEXT PRIMARY KEY,
        name            TEXT NOT NULL,
        collection_type TEXT,
        image_tag       TEXT,
        removed         INTEGER NOT NULL DEFAULT 0,
        updated_at      INTEGER NOT NULL
    ) WITHOUT ROWID;

    CREATE TABLE items (
        id                  TEXT PRIMARY KEY,
        library_id          TEXT,
        type                TEXT NOT NULL,
        name                TEXT NOT NULL,
        series_id           TEXT,
        season_id           TEXT,
        series_name         TEXT,
        index_number        INTEGER,
        parent_index_number INTEGER,
        album               TEXT,
        album_artist        TEXT,
        runtime_s           INTEGER,
        production_year     INTEGER,
        premiere_date       TEXT,
        date_created        INTEGER,
        community_rating    REAL,
        official_rating     TEXT,
        genres              TEXT,
        overview            TEXT,
        image_tag           TEXT,
        backdrop_tag        TEXT,
        container           TEXT,
        path                TEXT,
        size_bytes          INTEGER,
        bitrate             INTEGER,
        video_codec         TEXT,
        width               INTEGER,
        height              INTEGER,
        video_range         TEXT,
        audio_codec         TEXT,
        audio_channels      INTEGER,
        removed             INTEGER NOT NULL DEFAULT 0,
        updated_at          INTEGER NOT NULL
    ) WITHOUT ROWID;
    CREATE INDEX idx_items_library ON items(library_id, type);
    CREATE INDEX idx_items_series  ON items(series_id);
    CREATE INDEX idx_items_created ON items(date_created);

    CREATE TABLE playbacks (
        id                  INTEGER PRIMARY KEY,
        source              TEXT NOT NULL,           -- 'live' | 'jellystat'
        source_id           TEXT UNIQUE,             -- de-duplication key for imports
        active              INTEGER NOT NULL DEFAULT 0,
        user_id             TEXT NOT NULL,
        user_name           TEXT NOT NULL,
        item_id             TEXT NOT NULL,
        item_name           TEXT NOT NULL,
        item_type           TEXT NOT NULL,
        series_id           TEXT,
        series_name         TEXT,
        season_id           TEXT,
        season_number       INTEGER,
        episode_number      INTEGER,
        library_id          TEXT,
        started_at          INTEGER NOT NULL,
        ended_at            INTEGER NOT NULL,
        duration_s          INTEGER NOT NULL,        -- time actually spent playing
        paused_s            INTEGER NOT NULL DEFAULT 0,
        position_s          INTEGER,
        runtime_s           INTEGER,
        client              TEXT,
        device_name         TEXT,
        device_id           TEXT,
        app_version         TEXT,
        remote_ip           TEXT,
        play_method         TEXT,
        container           TEXT,
        bitrate             INTEGER,
        video_codec         TEXT,
        width               INTEGER,
        height              INTEGER,
        video_range         TEXT,
        bit_depth           INTEGER,
        audio_codec         TEXT,
        audio_channels      INTEGER,
        audio_language      TEXT,
        subtitle_codec      TEXT,
        subtitle_language   TEXT,
        transcode           TEXT                      -- JSON, null when not transcoding
    );
    CREATE INDEX idx_pb_ended  ON playbacks(ended_at);
    CREATE INDEX idx_pb_user   ON playbacks(user_id, ended_at);
    CREATE INDEX idx_pb_item   ON playbacks(item_id, ended_at);
    CREATE INDEX idx_pb_series ON playbacks(series_id, ended_at);
    CREATE INDEX idx_pb_active ON playbacks(active) WHERE active = 1;

    CREATE TABLE devices (
        device_id    TEXT NOT NULL,
        user_id      TEXT NOT NULL,
        device_name  TEXT,
        client       TEXT,
        app_version  TEXT,
        last_ip      TEXT,
        first_seen   INTEGER NOT NULL,
        last_seen    INTEGER NOT NULL,
        PRIMARY KEY (device_id, user_id)
    ) WITHOUT ROWID;

    CREATE TABLE server_events (
        id             INTEGER PRIMARY KEY,           -- Jellyfin activity log id
        date           INTEGER NOT NULL,
        name           TEXT NOT NULL,
        overview       TEXT,
        short_overview TEXT,
        type           TEXT,
        severity       TEXT,
        user_id        TEXT,
        item_id        TEXT
    );
    CREATE INDEX idx_events_date ON server_events(date);
    "#,
    // 2 — what happens *during* a play, Jellyfin's own played flags, richer item details
    r#"
    ALTER TABLE playbacks ADD COLUMN pause_count INTEGER NOT NULL DEFAULT 0;
    ALTER TABLE playbacks ADD COLUMN seek_count INTEGER NOT NULL DEFAULT 0;
    ALTER TABLE playbacks ADD COLUMN start_position_s INTEGER;
    ALTER TABLE playbacks ADD COLUMN is_local INTEGER;          -- NULL = unknown

    CREATE TABLE playback_events (
        id          INTEGER PRIMARY KEY,
        playback_id INTEGER NOT NULL REFERENCES playbacks(id) ON DELETE CASCADE,
        at          INTEGER NOT NULL,
        kind        TEXT NOT NULL,   -- start | pause | resume | seek | audio | subtitle | transcode | stop
        position_s  INTEGER,
        detail      TEXT
    );
    CREATE INDEX idx_pbe_playback ON playback_events(playback_id, id);

    CREATE TABLE user_items (
        user_id        TEXT NOT NULL,
        item_id        TEXT NOT NULL,
        played         INTEGER NOT NULL DEFAULT 0,
        is_favorite    INTEGER NOT NULL DEFAULT 0,
        play_count     INTEGER NOT NULL DEFAULT 0,
        last_played_at INTEGER,
        PRIMARY KEY (user_id, item_id)
    ) WITHOUT ROWID;
    CREATE INDEX idx_user_items_item ON user_items(item_id);

    ALTER TABLE items ADD COLUMN provider_ids TEXT;   -- JSON object
    ALTER TABLE items ADD COLUMN studios TEXT;        -- JSON array of names
    ALTER TABLE items ADD COLUMN bit_depth INTEGER;
    ALTER TABLE items ADD COLUMN framerate REAL;

    ALTER TABLE devices ADD COLUMN last_user_name TEXT;
    "#,
    // 3 — Jellystat has no item type, so imported Live TV channels were guessed to be films.
    //     A channel is not in the library and has neither a container nor a runtime.
    r#"
    UPDATE playbacks SET item_type = 'TvChannel'
    WHERE source = 'jellystat' AND item_type = 'Movie' AND container IS NULL AND runtime_s IS NULL
      AND NOT EXISTS (SELECT 1 FROM items i WHERE i.id = playbacks.item_id);
    "#,
    // 4 — re-linking renamed items looks titles up by name, once per orphaned item.
    r#"
    CREATE INDEX idx_items_type_name ON items(type, name COLLATE NOCASE);
    CREATE INDEX idx_items_series_episode ON items(series_id, parent_index_number, index_number);
    "#,
    // 5 — what each non-admin user has been granted, on top of the defaults in settings
    r#"
    CREATE TABLE user_permissions (
        user_id     TEXT PRIMARY KEY,
        permissions TEXT NOT NULL,          -- JSON array of permission keys
        updated_at  INTEGER NOT NULL
    ) WITHOUT ROWID;
    "#,
    // 6 — episodes a user marked as seen by hand, for what was watched while nothing was recording
    r#"
    CREATE TABLE manual_seen (
        user_id    TEXT NOT NULL,
        item_id    TEXT NOT NULL,
        created_at INTEGER NOT NULL,
        PRIMARY KEY (user_id, item_id)
    ) WITHOUT ROWID;
    "#,
    // 7 — group watching: plays that were watched together share a group_id (the lowest play id in it)
    r#"
    ALTER TABLE playbacks ADD COLUMN group_id INTEGER;
    CREATE INDEX idx_pb_group ON playbacks(group_id) WHERE group_id IS NOT NULL;
    CREATE INDEX idx_pb_item_start ON playbacks(item_id, started_at);
    "#,
    // 8 — cast and crew of films and shows, for "most watched people" in the recap. Forgetting when
    //     the library was last read makes the next start read it once more, so people arrive without
    //     waiting for Jellyfin's next scan.
    r#"
    CREATE TABLE item_people (
        item_id   TEXT NOT NULL,
        person_id TEXT NOT NULL,
        kind      TEXT NOT NULL,            -- Actor | Director
        name      TEXT NOT NULL,
        role      TEXT,
        sort      INTEGER NOT NULL,         -- billing order within the title
        has_image INTEGER NOT NULL DEFAULT 0,
        PRIMARY KEY (item_id, person_id, kind)
    ) WITHOUT ROWID;
    CREATE INDEX idx_item_people_person ON item_people(person_id, kind);
    DELETE FROM settings WHERE key = 'library_synced_at';
    "#,
    // 9 — addresses that count as "at home" although they are public: this network's own public
    //     address (looked up, every one ever seen) and any the owner adds by hand
    r#"
    CREATE TABLE home_addresses (
        ip         TEXT PRIMARY KEY,
        source     TEXT NOT NULL,           -- lookup | manual
        first_seen INTEGER NOT NULL,
        last_seen  INTEGER NOT NULL
    ) WITHOUT ROWID;
    "#,
    // 10 — "recently added" walks the library newest first and stops after a few dozen rows
    r#"
    CREATE INDEX idx_items_added ON items(date_created DESC) WHERE removed = 0 AND date_created IS NOT NULL;
    "#,
    // 11 — where people watch from: the address of a sign-in (NULL = not looked at yet, '' = none in the
    //      text), a place per address from the local geolocation database, and what looked wrong
    r#"
    ALTER TABLE server_events ADD COLUMN remote_ip TEXT;
    CREATE INDEX idx_events_ip ON server_events(type, date) WHERE remote_ip <> '';
    CREATE INDEX idx_pb_ip ON playbacks(remote_ip) WHERE remote_ip IS NOT NULL;

    CREATE TABLE ip_locations (
        ip           TEXT PRIMARY KEY,        -- as written in playbacks / server_events
        country_code TEXT,                    -- all NULL: a public address the database does not know
        country      TEXT,
        region       TEXT,
        city         TEXT,
        latitude     REAL,
        longitude    REAL,
        timezone     TEXT,
        looked_up_at INTEGER NOT NULL
    ) WITHOUT ROWID;

    CREATE TABLE security_alerts (
        id          INTEGER PRIMARY KEY,
        kind        TEXT NOT NULL,            -- impossible_travel | new_country
        severity    TEXT NOT NULL,            -- high | medium
        user_id     TEXT NOT NULL,
        user_name   TEXT NOT NULL,
        at          INTEGER NOT NULL,         -- when it happened, not when it was noticed
        dedupe      TEXT NOT NULL UNIQUE,     -- a rescan never reports the same thing twice
        details     TEXT NOT NULL,            -- JSON
        created_at  INTEGER NOT NULL,
        resolved_at INTEGER,
        resolved_by TEXT,
        note        TEXT,
        muted       INTEGER NOT NULL DEFAULT 0 -- "these two places are fine for this person"
    );
    CREATE INDEX idx_alerts_open ON security_alerts(resolved_at, at);
    CREATE INDEX idx_alerts_user ON security_alerts(user_id, at);
    "#,
    // 12 — which languages a file can be played in: every audio and subtitle track, not just the first.
    //      JSON arrays of the codes Jellyfin reports (ISO 639-2, "und" for a track without one), in track
    //      order. Filled by the library read, so forget when it last ran and it runs again now.
    r#"
    ALTER TABLE items ADD COLUMN audio_languages TEXT;
    ALTER TABLE items ADD COLUMN subtitle_languages TEXT;
    DELETE FROM settings WHERE key = 'library_synced_at';
    "#,
    // 13 — connections to the services around Jellyfin (Sonarr, Radarr, Seerr, torrent clients). Their keys
    //      and passwords live here and nowhere else: never in the settings blob, never in a backup.
    //      AUTOINCREMENT: an id is never handed out twice, so rows of a deleted service cannot be adopted.
    //      `item_external` is the library's provider ids turned sideways (one id can belong to several
    //      items: the same film in an HD and a 4K library), so that a TMDB or TVDB id finds its items by index.
    r#"
    CREATE TABLE services (
        id                   INTEGER PRIMARY KEY AUTOINCREMENT,
        kind                 TEXT NOT NULL,           -- sonarr | radarr | seerr | qbittorrent | transmission | deluge
        name                 TEXT NOT NULL,
        url                  TEXT NOT NULL,
        username             TEXT,
        secret               TEXT NOT NULL,           -- API key or password
        accept_invalid_certs INTEGER NOT NULL DEFAULT 0,
        enabled              INTEGER NOT NULL DEFAULT 1,
        created_at           INTEGER NOT NULL,
        version              TEXT,
        last_ok_at           INTEGER,
        last_error           TEXT
    );

    CREATE TABLE item_external (
        item_id TEXT NOT NULL,
        source  TEXT NOT NULL,                        -- Tmdb | Tvdb | Imdb, as Jellyfin spells them
        value   TEXT NOT NULL,
        PRIMARY KEY (item_id, source)
    ) WITHOUT ROWID;
    CREATE INDEX idx_item_external ON item_external(source, value);
    "#,
    // 14 — what Sonarr and Radarr expect: episodes about to air, films about to be released. One row per
    //      instance, title and kind of release; the same episode in two Sonarrs is folded when it is read.
    //      An episode has a moment (`at`, UTC). A film has a *day*: Radarr gives midnight UTC, which as a
    //      moment would be the evening before in every zone west of Greenwich.
    r#"
    CREATE TABLE upcoming (
        service_id   INTEGER NOT NULL REFERENCES services(id) ON DELETE CASCADE,
        kind         TEXT NOT NULL,              -- episode | movie
        external_id  INTEGER NOT NULL,           -- Sonarr's episode id, Radarr's movie id
        release      TEXT NOT NULL,              -- air | cinema | digital | physical
        at           INTEGER,                    -- episodes
        day          TEXT,                       -- films: YYYY-MM-DD
        series_title TEXT,
        title        TEXT NOT NULL,
        season       INTEGER,
        episode      INTEGER,
        finale       TEXT,                       -- season | series | midseason
        year         INTEGER,
        tvdb_id      INTEGER,
        tmdb_id      INTEGER,
        imdb_id      TEXT,
        arr_media_id INTEGER NOT NULL,           -- Sonarr's series id, Radarr's movie id: where the poster is
        has_file     INTEGER NOT NULL DEFAULT 0,
        item_id      TEXT,                       -- the series or film in the library, once it is there
        PRIMARY KEY (service_id, kind, external_id, release)
    ) WITHOUT ROWID;
    CREATE INDEX idx_upcoming_at ON upcoming(at);
    CREATE INDEX idx_upcoming_day ON upcoming(day);
    "#,
    // 15 — what people asked for in Seerr. Seerr purges a request when its media is removed, so a request that
    //      disappears is kept and marked (`removed_at`): "asked for 14, watched 9" should not shrink.
    //      `status` and `media_status` are Seerr's own numbers; `available_at` is worked out by finstats.
    //      `user_id` is the Jellyfin user behind Seerr's, when Seerr says who that is; `item_id` and
    //      `arr_*` say where the poster is: in the library, or still only in Sonarr or Radarr.
    r#"
    CREATE TABLE requests (
        service_id        INTEGER NOT NULL REFERENCES services(id) ON DELETE CASCADE,
        request_id        INTEGER NOT NULL,
        media_type        TEXT NOT NULL,             -- movie | tv
        tmdb_id           INTEGER,
        tvdb_id           INTEGER,
        imdb_id           TEXT,
        title             TEXT,                      -- NULL until it has been looked up
        year              INTEGER,
        seasons           TEXT NOT NULL DEFAULT '[]', -- JSON array of season numbers; empty for a film
        is_4k             INTEGER NOT NULL DEFAULT 0,
        status            INTEGER NOT NULL,          -- 1 pending, 2 approved, 3 declined, 4 failed, 5 completed
        media_status      INTEGER NOT NULL,          -- 1 unknown, 2 pending, 3 processing, 4 partially available, 5 available, 6+ gone
        requested_at      INTEGER NOT NULL,
        updated_at        INTEGER NOT NULL,
        media_added_at    INTEGER,                   -- Seerr's own note of when it arrived
        seen_available_at INTEGER,                   -- when finstats first saw it available
        available_at      INTEGER,
        removed_at        INTEGER,
        seerr_user_id     INTEGER,
        seerr_user_name   TEXT,
        jellyfin_user_id  TEXT,
        jellyfin_username TEXT,
        jellyfin_media_id TEXT,
        user_id           TEXT,
        item_id           TEXT,
        arr_service_id    INTEGER,
        arr_media_id      INTEGER,
        looked_up_at      INTEGER,                   -- the last attempt to find its title
        PRIMARY KEY (service_id, request_id)
    ) WITHOUT ROWID;
    CREATE INDEX idx_requests_user ON requests(user_id, requested_at);
    CREATE INDEX idx_requests_when ON requests(requested_at);
    CREATE INDEX idx_requests_item ON requests(item_id);
    "#,
    // 16 — what actually came in: Sonarr's and Radarr's history, the grabbed / imported / failed events.
    //      One row per event, kept by the id the instance gave it, so reading twice changes nothing.
    r#"
    CREATE TABLE grabs (
        service_id  INTEGER NOT NULL REFERENCES services(id) ON DELETE CASCADE,
        history_id  INTEGER NOT NULL,
        event       TEXT NOT NULL,              -- grabbed | imported | failed
        at          INTEGER NOT NULL,
        media_type  TEXT NOT NULL,              -- tv | movie
        title       TEXT,                       -- the show or film
        source      TEXT,                       -- what the release was called
        season      INTEGER,
        tvdb_id     INTEGER,
        tmdb_id     INTEGER,
        size_bytes  INTEGER,
        quality     TEXT,
        indexer     TEXT,
        protocol    TEXT,
        client      TEXT,
        download_id TEXT,
        PRIMARY KEY (service_id, history_id)
    ) WITHOUT ROWID;
    CREATE INDEX idx_grabs_at ON grabs(at);
    "#,
    // 17 — clear out the `transcode` events 1.5.0 and earlier wrote every second. A play whose
    //      client settled back to direct play after transcoding had its kept record forced to
    //      "Transcode" *after* each comparison, so every reading that followed looked like a
    //      change: one event per poll, all identical, filling the Activity page. The collector no
    //      longer writes them; this drops the ones already written, keeping the first of each run
    //      so a play that really did switch back and forth still reads as it happened.
    r#"
    DELETE FROM playback_events WHERE id IN (
        SELECT id FROM (
            SELECT id, detail, LAG(detail) OVER (PARTITION BY playback_id ORDER BY at, id) AS before
            FROM playback_events WHERE kind = 'transcode'
        ) WHERE IFNULL(detail, '') = IFNULL(before, '')
    );
    "#,
    // 18 — notifications: where finstats may send what it finds, what it found, and how each sending went.
    //      `notify_targets` holds the address *and* the secret of a destination (a Discord webhook URL is
    //      itself the credential), so like `services` it never leaves this table and is never backed up.
    //      An event is written once and deduped by `dedupe`, exactly like `security_alerts`, so deriving
    //      the same thing twice adds nothing; `private` holds the addresses and coordinates a destination
    //      only gets when its owner switched "include addresses" on. Delivery is separate from the event:
    //      one row per destination, retried on its own clock, so a webhook that is down loses nothing.
    r#"
    CREATE TABLE notify_targets (
        id                   INTEGER PRIMARY KEY AUTOINCREMENT,
        kind                 TEXT NOT NULL,                -- webhook | discord | ntfy | gotify
        name                 TEXT NOT NULL,
        url                  TEXT NOT NULL,                -- never sent back to a browser
        secret               TEXT NOT NULL DEFAULT '',
        topic                TEXT,                         -- ntfy
        owner_id             TEXT REFERENCES users(id) ON DELETE CASCADE,  -- NULL = the server's own
        events               TEXT NOT NULL DEFAULT '[]',   -- JSON array of event kinds this one wants
        with_addresses       INTEGER NOT NULL DEFAULT 0,
        min_severity         TEXT NOT NULL DEFAULT 'info',
        accept_invalid_certs INTEGER NOT NULL DEFAULT 0,
        enabled              INTEGER NOT NULL DEFAULT 1,
        created_at           INTEGER NOT NULL,
        last_ok_at           INTEGER,
        last_error           TEXT
    );
    CREATE TABLE notify_events (
        id         INTEGER PRIMARY KEY AUTOINCREMENT,
        kind       TEXT NOT NULL,
        severity   TEXT NOT NULL,                -- info | warn | alert
        at         INTEGER NOT NULL,             -- when the thing itself happened
        created_at INTEGER NOT NULL,             -- when finstats noticed
        dedupe     TEXT NOT NULL UNIQUE,
        user_id    TEXT,
        user_name  TEXT,
        title      TEXT NOT NULL,
        body       TEXT NOT NULL,
        link       TEXT,                         -- a path inside finstats, joined with public_url when sent
        data       TEXT NOT NULL DEFAULT '{}',   -- what any destination may be told
        private    TEXT NOT NULL DEFAULT '{}',   -- addresses and places: only with "include addresses"
        historic   INTEGER NOT NULL DEFAULT 0    -- found long after it happened: recorded, never sent
    );
    CREATE INDEX idx_notify_events_at ON notify_events(at);
    CREATE TABLE notify_deliveries (
        event_id  INTEGER NOT NULL REFERENCES notify_events(id) ON DELETE CASCADE,
        target_id INTEGER NOT NULL REFERENCES notify_targets(id) ON DELETE CASCADE,
        state     TEXT NOT NULL,                 -- queued | sent | failed
        attempts  INTEGER NOT NULL DEFAULT 0,
        next_at   INTEGER NOT NULL,
        sent_at   INTEGER,
        error     TEXT,
        PRIMARY KEY (event_id, target_id)
    ) WITHOUT ROWID;
    CREATE INDEX idx_notify_due ON notify_deliveries(state, next_at);
    "#,
    // 19 — what a destination needs beyond an address, a token and the one field beside them. Only mail
    // has any (the sender, and a user name that is often not the sender), and it is not a secret: the
    // password stays in `secret`.
    r#"
    ALTER TABLE notify_targets ADD COLUMN options TEXT NOT NULL DEFAULT '{}';
    "#,
    // 20 — which tracker a play came from is a filter on the Activity page, and the page asks on
    //      every load which trackers the history holds at all. Three seeks rather than three scans.
    r#"
    CREATE INDEX idx_pb_source ON playbacks(source);
    "#,
    // 21 — a seek's origin as a number. The label ("27:12 → 33:10") stays for the timeline; the number
    //      is what says whether a seek went backwards, which the rewind heatmap asks of every seek. The
    //      left side of the label is the position playback was expected at, in m:ss or h:mm:ss.
    //      `backup::restore` runs the same backfill for a file from before the column existed.
    r#"
    ALTER TABLE playback_events ADD COLUMN from_s INTEGER;
    UPDATE playback_events SET from_s = (
      WITH c(t) AS (SELECT substr(detail, 1, instr(detail, ' → ') - 1))
      SELECT CASE WHEN length(t) - length(replace(t, ':', '')) = 2
                  THEN CAST(substr(t, 1, instr(t, ':') - 1) AS INTEGER) * 3600
                     + CAST(substr(t, instr(t, ':') + 1, 2) AS INTEGER) * 60 + CAST(substr(t, -2) AS INTEGER)
                  ELSE CAST(substr(t, 1, instr(t, ':') - 1) AS INTEGER) * 60 + CAST(substr(t, -2) AS INTEGER) END
      FROM c)
    WHERE kind = 'seek' AND detail LIKE '%:__ → %';
    CREATE INDEX idx_pbe_kind ON playback_events(kind, playback_id, position_s);
    "#,
    // 22 — API keys, and finstats' own audit log, in one migration: a long-lived credential without a
    //      record of what it did would be a step backwards. A key is stored as the hash of its token,
    //      like a session, and is never part of a backup; the audit log is, being finstats' own data.
    //      Ids are never reused (AUTOINCREMENT) so an audit row keeps pointing at the right key.
    r#"
    CREATE TABLE api_keys (
        id           INTEGER PRIMARY KEY AUTOINCREMENT,
        token_hash   TEXT NOT NULL UNIQUE,
        user_id      TEXT NOT NULL,
        name         TEXT NOT NULL,
        scope        TEXT NOT NULL,              -- full | calendar
        created_at   INTEGER NOT NULL,
        expires_at   INTEGER,                    -- NULL = until revoked
        last_used_at INTEGER,
        last_used_ip TEXT,
        revoked_at   INTEGER
    );
    CREATE INDEX idx_api_keys_user ON api_keys(user_id);
    CREATE TABLE audit (
        id        INTEGER PRIMARY KEY AUTOINCREMENT,
        at        INTEGER NOT NULL,
        kind      TEXT NOT NULL,
        user_id   TEXT,                          -- who did it; NULL for a failed sign-in of an unknown name, or the scheduler
        user_name TEXT,                          -- as typed, for a failed sign-in
        ip        TEXT,
        key_id    INTEGER,                       -- through which key, when a key did it
        target    TEXT,                          -- what it was about: a user id, a backup name, a key id, a play id
        detail    TEXT NOT NULL DEFAULT '{}',    -- JSON; never a secret
        outcome   TEXT NOT NULL DEFAULT 'ok'     -- ok | failed | refused
    );
    CREATE INDEX idx_audit_at ON audit(at);
    CREATE INDEX idx_audit_user ON audit(user_id, at);
    "#,
    // 24 — Public profiles (2.0): what a person chose to publish, and the link that reaches it. The token
    //      is stored as it is, not hashed: it is a link its owner is shown again, and it opens nothing but
    //      what the owner published. Not part of a backup, so a restore never brings an old link back.
    r#"
    CREATE TABLE public_profiles (
        user_id      TEXT PRIMARY KEY,
        token        TEXT NOT NULL UNIQUE,
        published    INTEGER NOT NULL DEFAULT 0,
        display_name TEXT NOT NULL DEFAULT '',
        show_avatar  INTEGER NOT NULL DEFAULT 0,
        sections     TEXT NOT NULL DEFAULT '{}',   -- {"totals","habits","recap","recent"}: bools, all off
        created_at   INTEGER NOT NULL,
        updated_at   INTEGER NOT NULL
    ) WITHOUT ROWID;
    "#,
    // 25 — An import asks, for every row, whether this person already has this play of this title. The
    //      title index alone made that read every play of the title by anybody — quadratic in a big import.
    "CREATE INDEX idx_pb_user_item ON playbacks(user_id, item_id, started_at);",
    // 26 — …and the same by where a play ended, for the other half of the rule: two trackers agree about the end.
    "CREATE INDEX idx_pb_user_item_end ON playbacks(user_id, item_id, ended_at);",
    // 27 — Re-linking sweeps only the titles it moved plays onto. Before, it swept the whole history at every
    //      start and library read; an install coming from before that sweep existed (2.0.0) is swept once, here.
    crate::relinked_duplicates_sql!("COALESCE((SELECT json_extract(value, '$.merge_window_s') FROM settings WHERE key = 'settings'), 600)"),
    // user_version 27 (the labels above run one ahead of it) — The order one person's sessions were made in.
    //      `created_at` is whole seconds and the table has no rowid, so among sign-ins of one second the one
    //      that gave way to the limit was picked by its token's hash, not its age.
    "ALTER TABLE sessions ADD COLUMN seq INTEGER NOT NULL DEFAULT 0;",
    // user_version 28 — How each of finstats' own jobs last ended (2.0.4). An interval trigger counts from the
    //      last run, and a start must not forget it: every job used to run again at every start. Re-readable,
    //      so not part of a backup.
    r#"
    CREATE TABLE task_runs (
        task        TEXT PRIMARY KEY,
        state       TEXT NOT NULL,            -- ok | error
        started_at  INTEGER,
        finished_at INTEGER,
        message     TEXT,
        error       TEXT
    ) WITHOUT ROWID;
    "#,
    // user_version 29 — A person's portrait has an image tag of its own (2.0.4): the cache names a picture after it, and
    //      a portrait replaced in Jellyfin re-saves the person, not the titles they are in.
    "ALTER TABLE item_people ADD COLUMN image_tag TEXT;",
    // user_version 30 — Watchlists (2.1.0): the films and shows somebody means to watch. An entry names its title by
    //      the item in the library or, when it is not there yet, by its provider ids, and keeps enough of it (kind,
    //      title, year, ids) to be read without the library. Somebody's own and not Jellyfin's to give back, so it is
    //      part of a backup. AUTOINCREMENT: an id a page still holds never comes to mean another entry.
    r#"
    CREATE TABLE watchlist (
        id       INTEGER PRIMARY KEY AUTOINCREMENT,
        user_id  TEXT NOT NULL,
        kind     TEXT NOT NULL,             -- Movie | Series
        item_id  TEXT,                      -- the title in the library, once it has been; kept after it is removed
        tmdb_id  TEXT,
        tvdb_id  TEXT,
        imdb_id  TEXT,
        title    TEXT NOT NULL,
        year     INTEGER,
        added_at INTEGER NOT NULL
    );
    CREATE INDEX idx_watchlist_user ON watchlist(user_id, added_at);
    CREATE UNIQUE INDEX idx_watchlist_item ON watchlist(user_id, item_id) WHERE item_id IS NOT NULL;
    "#,
    // user_version 31 — Whether an entry's title was missing from the library at the last look, and when it came (2.1.1).
    //      An arrival is a title that was not here and now is: a new item alone also means a replaced file (a new path
    //      is a new id), which announced "now on the server" for titles that never left.
    r#"
    ALTER TABLE watchlist ADD COLUMN missing INTEGER NOT NULL DEFAULT 1;
    ALTER TABLE watchlist ADD COLUMN arrived_at INTEGER;
    UPDATE watchlist SET missing = NOT EXISTS (SELECT 1 FROM items i WHERE i.id = watchlist.item_id AND i.removed = 0);
    "#,
    // user_version 32 — A title's original-language name, beside the one Jellyfin shows (2.1.2): Plex, Tautulli or an old
    //      library may know Squid Game only as 오징어 게임, and a title is found by either. Filled by the next library read.
    "ALTER TABLE items ADD COLUMN original_title TEXT;",
];

/// One look at the file before anything opens it for real. The pool retries a connection that fails for its whole
/// 30-second timeout and then says only "timed out waiting for connection", and its first connection switches the
/// file to WAL — a write — before any check could run. So a file that is damaged, is not a database, or is another
/// program's database is refused here, at once, by name, having only been read.
fn preflight(path: &Path) -> Result<()> {
    let shown = path.display();
    let recover = "Nothing was changed. If this is finstats' database and it was damaged, stop finstats and put the newest \
                   copy from pre-update-backups/ (or restore one from backups/) in its place.";
    match std::fs::metadata(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => bail!("{shown} cannot be read ({e}). Nothing was changed."),
        Ok(m) if m.is_dir() => bail!("{shown} is a folder, not a database. Nothing was changed."),
        Ok(m) if m.len() == 0 => return Ok(()), // an empty file is a new database, as SQLite itself treats it
        Ok(_) => {}
    }
    // SQLite opens a file it may not write read-only without a word, and the first write fails much later.
    if let Err(e) = std::fs::OpenOptions::new().read(true).write(true).open(path) {
        bail!("{shown} cannot be written ({e}). Nothing was changed. It must belong to the user finstats runs as, with write permission.");
    }
    let read = (|| -> rusqlite::Result<(i64, Vec<String>)> {
        let c = Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX)?;
        let version: i64 = c.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        let mut stmt = c.prepare("SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name")?;
        let tables = stmt.query_map([], |r| r.get::<_, String>(0))?.collect::<rusqlite::Result<Vec<_>>>()?;
        Ok((version, tables))
    })();
    match read {
        Err(e) => bail!("{shown} is damaged or is not a database ({e}). {recover}"),
        Ok((0, tables)) if !tables.is_empty() => bail!(
            "{shown} is another program's database (it holds {}, which finstats did not make), so finstats will not write into it. \
             Nothing was changed. Point FINSTATS_DATA_DIR at a folder of finstats' own.",
            tables.iter().take(5).map(|t| format!("`{t}`")).collect::<Vec<_>>().join(", ")
        ),
        Ok(_) => Ok(()),
    }
}

impl Db {
    pub fn open(path: &Path) -> Result<Self> {
        preflight(path)?;
        let manager = SqliteConnectionManager::file(path).with_init(|c| {
            c.execute_batch(&format!(
                "PRAGMA journal_mode = WAL;
                 PRAGMA synchronous = NORMAL;
                 PRAGMA foreign_keys = ON;
                 PRAGMA busy_timeout = 15000;
                 PRAGMA temp_store = MEMORY;
                 PRAGMA cache_size = -8000;
                 PRAGMA journal_size_limit = {JOURNAL_LIMIT};"
            ))?;
            // Every transaction takes the write lock as it begins, waiting its turn under busy_timeout, so one that reads
            // before it writes can never be refused its write by a commit in between (SQLITE_BUSY_SNAPSHOT). The one
            // transaction that only reads — a backup's consistent snapshot — asks for a deferred one by name.
            c.set_transaction_behavior(rusqlite::TransactionBehavior::Immediate);
            Ok(())
        });
        let pool = r2d2::Pool::builder()
            .max_size(6)
            .min_idle(Some(1))
            .build(manager)
            .context("opening database")?;
        let db = Db { pool };
        let running = env!("CARGO_PKG_VERSION");
        refuse_downgrade(&*db.conn()?, running)?;
        back_up_before_update(&*db.conn()?, path, running)?;
        db.migrate()?;
        record_version(&*db.conn()?, running)?;
        Ok(db)
    }

    /// A migrated database that exists only for the length of a test.
    #[cfg(test)]
    pub fn open_in_memory() -> Result<Self> {
        let manager = SqliteConnectionManager::memory().with_init(|c| c.execute_batch("PRAGMA foreign_keys = ON;"));
        // One connection: every caller must see the same in-memory database.
        let db = Db { pool: r2d2::Pool::builder().max_size(1).build(manager)? };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> Result<()> {
        let mut conn = self.pool.get()?;
        let current: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        for (i, sql) in MIGRATIONS.iter().enumerate().skip(current as usize) {
            let tx = conn.transaction()?;
            tx.execute_batch(sql)?;
            tx.pragma_update(None, "user_version", (i + 1) as i64)?;
            tx.commit()?;
            tracing::info!("applied database migration {}", i + 1);
        }
        Ok(())
    }

    /// Run blocking database work off the async runtime.
    pub async fn call<T, F>(&self, f: F) -> Result<T>
    where
        T: Send + 'static,
        F: FnOnce(&mut Connection) -> Result<T> + Send + 'static,
    {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get().context("database pool exhausted")?;
            f(&mut conn)
        })
        .await
        .context("database task panicked")?
    }

    /// Synchronous access for code that is already on a blocking thread.
    pub fn conn(&self) -> Result<r2d2::PooledConnection<SqliteConnectionManager>> {
        self.pool.get().context("database pool exhausted")
    }
}

/// Settings key: the newest finstats version that has opened this database.
const VERSION_KEY: &str = "app_version";

/// `1.2.3` as a comparable triple; a pre-release or build suffix (`-rc.1`, `+abc`) is ignored.
fn parse_version(v: &str) -> Option<(u64, u64, u64)> {
    let mut parts = v.trim().split(['-', '+']).next()?.split('.').map(|p| p.parse::<u64>().ok());
    let triple = (parts.next()??, parts.next()??, parts.next()??);
    parts.next().is_none().then_some(triple)
}

/// A database only moves forward. An older finstats knows nothing about the tables and columns a newer one added,
/// would skip the migrations without a word and write rows the newer one then misreads, so it refuses to start
/// instead. Checked before anything is written; a stored version nobody can read counts as newer.
fn refuse_downgrade(conn: &Connection, running: &str) -> Result<()> {
    let schema: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    if schema == 0 {
        return Ok(()); // brand new: there is no settings table yet
    }
    // No entry: last opened by a release from before versions were recorded (1.0.4 or older).
    let stored = get_setting(conn, VERSION_KEY)?;
    // Unreadable is refused like newer, since it may be, but said as what it is: only a hand edit makes it.
    if let Some(s) = stored.as_deref().filter(|s| parse_version(s).is_none()) {
        bail!(
            "this database records the last finstats to open it as [{s}], which is not a version finstats can read,\n\
             so this finstats {running} cannot tell whether the database is newer than it. Nothing was changed.\n\
             It is the app_version setting. Write the version that last ran here as three numbers, without quotes,\n\
             and never an older one than that:   sqlite3 finstats.db \"UPDATE settings SET value='x.y.z' WHERE key='app_version'\""
        )
    }
    let newer = stored.as_deref().is_some_and(|s| match (parse_version(s), parse_version(running)) {
        (Some(theirs), Some(ours)) => theirs > ours,
        _ => true,
    });
    if !newer && schema as usize <= MIGRATIONS.len() {
        return Ok(());
    }
    let theirs = stored.map(|s| format!("finstats {s}")).unwrap_or_else(|| "a newer finstats".into());
    bail!(
        "this database was last used by {theirs}, and this is the older finstats {running}.\n\n\
         An older version cannot safely open a newer database, so it will not start. Nothing was changed.\n\
         Fix it by running that version or a newer one again (with Docker: the image tag you used before),\n\
         or start this version on an empty data folder and bring your history back from one of the files in\n\
         the old folder's backups/ directory:   finstats restore <file>"
    )
}

/// Whether opening this database is an *update* worth snapshotting first: a populated database (it has
/// a schema, so it holds data) that a different finstats version is now opening, or that still has
/// migrations to run. A brand-new database has nothing to protect, and the same version restarting is
/// not an update.
fn is_update(schema: i64, stored: Option<&str>, running: &str, migrations_len: usize) -> bool {
    schema > 0 && ((schema as usize) < migrations_len || stored != Some(running))
}

/// Kept apart from the exportable JSON backups in `backups/`: these are byte-for-byte copies of the
/// whole database — the library and the secrets included — for going back locally if an upgrade breaks
/// something, so they are never served over the API.
const PRE_UPDATE_DIR: &str = "pre-update-backups";
const PRE_UPDATE_KEEP: usize = 3;

/// The most the write-ahead journal keeps on disk once it has been checkpointed (64 MB).
const JOURNAL_LIMIT: i64 = 64 * 1024 * 1024;

/// Before a newer finstats touches an older database, copy the whole thing, so that nothing an upgrade
/// might break — a migration, or the new binary writing rows the old one cannot — can lose the user's
/// data beyond recovery. They can go back to the copy and report the bug without having lost anything.
/// The copy is a consistent full snapshot (`VACUUM INTO`), taken before any migration runs. Missing the
/// copy is only fatal when migrations are pending (the risky case); a plain version bump warns and goes on.
fn back_up_before_update(conn: &Connection, path: &Path, running: &str) -> Result<()> {
    let schema: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    if schema == 0 {
        return Ok(()); // brand new: no settings table yet, and no data to protect
    }
    let stored = get_setting(conn, VERSION_KEY)?;
    if !is_update(schema, stored.as_deref(), running, MIGRATIONS.len()) {
        return Ok(());
    }
    let from = stored.as_deref().unwrap_or("an unknown earlier version");
    let pending = (schema as usize) < MIGRATIONS.len();
    if std::env::var("FINSTATS_SKIP_PREUPDATE_BACKUP").is_ok_and(|v| v == "1" || v.eq_ignore_ascii_case("true")) {
        tracing::warn!("update detected ({from} -> {running}) but FINSTATS_SKIP_PREUPDATE_BACKUP is set; not backing up first");
        return Ok(());
    }
    let dir = path.parent().unwrap_or_else(|| Path::new(".")).join(PRE_UPDATE_DIR);
    let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let dst = dir.join(format!("finstats-{}-{stamp}.db", stored.as_deref().unwrap_or("pre-1.0.5")));
    match snapshot_into(conn, &dir, &dst) {
        Ok(()) => {
            tracing::info!(
                "update detected ({from} -> {running}); backed up the database to {} before upgrading. If anything looks wrong after this update, stop finstats, replace {} with that file, and start the previous version — then report the bug.",
                dst.display(),
                path.display()
            );
            prune_snapshots(&dir, PRE_UPDATE_KEEP);
            Ok(())
        }
        Err(e) if pending => bail!(
            "could not back up the database before applying migrations ({from} -> {running}): {e:#}\n\n\
             finstats will not run migrations without a safety copy, so nothing was changed. Free up disk\n\
             space (the copy needs about as much room as the database) or fix the permissions on {}, then\n\
             start again. To upgrade without a copy anyway, set FINSTATS_SKIP_PREUPDATE_BACKUP=1.",
            dir.display()
        ),
        Err(e) => {
            tracing::warn!("could not back up before the update ({from} -> {running}): {e:#}; continuing, since there is no schema change this time");
            Ok(())
        }
    }
}

/// A consistent copy of the whole database to `dst` (which must not already exist). `VACUUM INTO`
/// writes a compact, fully-committed snapshot without needing the file closed or a special build feature.
///
/// Written to `<dst>.part` and renamed only once complete: VACUUM INTO writes straight into the file it is given, so a
/// start killed while copying used to leave an empty file under a snapshot's own name, which counted toward the
/// three kept. Whatever such a start left behind is removed first.
fn snapshot_into(conn: &Connection, dir: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for e in entries.flatten() {
            if e.file_name().to_str().is_some_and(|n| n.starts_with("finstats-") && n.contains(".db.part")) {
                let _ = std::fs::remove_file(e.path());
            }
        }
    }
    let part = dst.with_file_name(format!("{}.part", dst.file_name().and_then(|n| n.to_str()).context("backup path is not valid UTF-8")?));
    let written = part.to_str().context("backup path is not valid UTF-8")?;
    let copied = conn.execute_batch(&format!("VACUUM INTO '{}'", written.replace('\'', "''")));
    if let Err(e) = copied {
        let _ = std::fs::remove_file(&part);
        return Err(e.into());
    }
    std::fs::rename(&part, dst).with_context(|| format!("naming {}", dst.display()))?;
    Ok(())
}

/// Keep only the newest `keep` snapshots; a full copy is large, and the point is recovery from the last
/// upgrade or two, not a museum.
fn prune_snapshots(dir: &Path, keep: usize) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    let mut snaps: Vec<_> = entries.filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x == "db")).collect();
    if snaps.len() <= keep {
        return;
    }
    snaps.sort_by_key(|p| std::fs::metadata(p).and_then(|m| m.modified()).ok());
    for old in &snaps[..snaps.len() - keep] {
        let _ = std::fs::remove_file(old);
    }
}

/// Remember the newest version that has opened this database; never lowers it.
fn record_version(conn: &Connection, running: &str) -> Result<()> {
    if get_setting(conn, VERSION_KEY)?.as_deref() != Some(running) {
        set_setting(conn, VERSION_KEY, running)?;
    }
    Ok(())
}

pub fn get_setting(conn: &Connection, key: &str) -> Result<Option<String>> {
    Ok(conn
        .query_row("SELECT value FROM settings WHERE key = ?1", [key], |r| r.get(0))
        .optional()?)
}

pub fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO settings(key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

/// LAN, loopback, link-local, CGNAT (Tailscale & friends) and IPv6 ULA count as local.
///
/// The unspecified addresses (`0.0.0.0`, `::`) and the rest of `0.0.0.0/8` count too, and not as a
/// nicety: `connect()` to `0.0.0.0` reaches this machine, so a "public" destination spelled that way
/// is loopback under another name — which is exactly what `notify::must_be_public` is holding shut.
pub fn is_local_ip(ip: &str) -> Option<bool> {
    use std::net::IpAddr;
    let v4_local = |v4: std::net::Ipv4Addr| {
        v4.is_private() || v4.is_loopback() || v4.is_link_local() || v4.octets()[0] == 0 || (v4.octets()[0] == 100 && (v4.octets()[1] & 0xC0) == 64)
    };
    match ip.trim().parse::<IpAddr>().ok()? {
        IpAddr::V4(v4) => Some(v4_local(v4)),
        IpAddr::V6(v6) => Some(match v6.to_ipv4_mapped() {
            Some(v4) => v4_local(v4),
            None => v6.is_loopback() || v6.is_unspecified() || (v6.segments()[0] & 0xfe00) == 0xfc00 || (v6.segments()[0] & 0xffc0) == 0xfe80,
        }),
    }
}


pub fn now() -> i64 {
    chrono::Utc::now().timestamp()
}

/// Jellyfin ids show up with and without dashes depending on the endpoint.
pub fn norm_id(id: &str) -> String {
    id.chars().filter(|c| *c != '-').map(|c| c.to_ascii_lowercase()).collect()
}

/// Parse an ISO-8601 timestamp (as emitted by Jellyfin / Jellystat) to unix seconds.
pub fn parse_ts(s: &str) -> Option<i64> {
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|d| d.timestamp())
        .ok()
        .or_else(|| {
            // Jellyfin sometimes omits the offset; treat as UTC.
            chrono::NaiveDateTime::parse_from_str(s.trim_end_matches('Z'), "%Y-%m-%dT%H:%M:%S%.f")
                .ok()
                .map(|d| d.and_utc().timestamp())
        })
}

pub use rusqlite::types::Value as SqlValue;

#[cfg(test)]
mod tests {
    use super::*;

    fn db_at(schema: usize, version: Option<&str>) -> Connection {
        let c = Connection::open_in_memory().unwrap();
        if schema > 0 {
            c.execute_batch(MIGRATIONS[0]).unwrap();
            c.pragma_update(None, "user_version", schema as i64).unwrap();
        }
        if let Some(v) = version {
            set_setting(&c, VERSION_KEY, v).unwrap();
        }
        c
    }

    #[test]
    fn keys_and_an_audit_log_arrive_together() {
        // A long-lived credential without a record of what it did would be a step backwards, so the
        // two tables are one migration.
        let c = Connection::open_in_memory().unwrap();
        for m in MIGRATIONS {
            c.execute_batch(m).unwrap();
        }
        c.execute_batch(
            "INSERT INTO api_keys(token_hash, user_id, name, scope, created_at) VALUES ('h', 'u1', 'laptop', 'full', 1);
             INSERT INTO audit(at, kind, user_id, user_name, ip, key_id, target, detail, outcome) VALUES (1, 'key_created', 'u1', 'alice', '192.168.1.10', 1, '1', '{}', 'ok');",
        )
        .unwrap();
        let n: i64 = c.query_row("SELECT COUNT(*) FROM audit a JOIN api_keys k ON k.id = a.key_id", [], |r| r.get(0)).unwrap();
        assert_eq!(n, 1);
        for idx in ["idx_api_keys_user", "idx_audit_at", "idx_audit_user"] {
            let t = if idx.starts_with("idx_api") { "api_keys" } else { "audit" };
            let found: i64 = c.query_row(&format!("SELECT COUNT(*) FROM pragma_index_list('{t}') WHERE name = '{idx}'"), [], |r| r.get(0)).unwrap();
            assert_eq!(found, 1, "{idx}");
        }
    }

    #[test]
    fn an_install_that_never_swept_for_relinked_duplicates_is_swept_once() {
        // Re-linking now sweeps only the titles it moved plays onto. An install coming from before the sweep existed
        // (2.0.0) may hold duplicates it never looked for, so the whole history is swept once, here, with the install's
        // own merge window — and with the default one when the settings never named it.
        let last = 25; // the sweep (user_version 26), whatever comes after it
        for (window, kept) in [(Some(30), vec![1, 2, 3]), (None, vec![1, 3])] {
            let c = Connection::open_in_memory().unwrap();
            for m in &MIGRATIONS[..last] {
                c.execute_batch(m).unwrap();
            }
            if let Some(w) = window {
                c.execute("INSERT INTO settings(key, value) VALUES ('settings', ?1)", [format!("{{\"merge_window_s\": {w}}}")]).unwrap();
            }
            c.execute_batch(
                "INSERT INTO playbacks(id, source, source_id, user_id, user_name, item_id, item_name, item_type, started_at, ended_at, duration_s) VALUES
                   (1, 'live',      NULL,   'u1', 'alice', 'i1', 'Big Buck Bunny', 'Movie', 1000, 4600, 3600),
                   (2, 'jellystat', 'js:1', 'u1', 'alice', 'i1', 'Big Buck Bunny', 'Movie', 1100, 4700, 3600),
                   (3, 'jellystat', 'js:2', 'u1', 'alice', 'i1', 'Big Buck Bunny', 'Movie', 90000, 93600, 3600);",
            )
            .unwrap();
            c.execute_batch(MIGRATIONS[last]).unwrap();
            let left: Vec<i64> = c.prepare("SELECT id FROM playbacks ORDER BY id").unwrap().query_map([], |r| r.get(0)).unwrap().map(Result::unwrap).collect();
            assert_eq!(left, kept, "window {window:?}: 100 s apart is the same play at the default 600, two plays at 30");
        }
    }

    #[test]
    fn a_seek_recorded_before_its_origin_was_kept_gets_one_from_its_label() {
        // A seek used to keep where it came from only in its label ("27:12 → 33:10"), the left side
        // being the position playback was expected at. The rewind heatmap asks of every seek whether
        // it went backwards, so the origin becomes a number once, here, for every seek already kept.
        let c = Connection::open_in_memory().unwrap();
        let last = 20; // migration 21 (numbered from 1), whatever comes after it
        for m in &MIGRATIONS[..last] {
            c.execute_batch(m).unwrap();
        }
        c.execute_batch(
            "INSERT INTO playbacks(id, source, user_id, user_name, item_id, item_name, item_type, started_at, ended_at, duration_s)
               VALUES (1, 'live', 'u1', 'alice', 'i1', 'Big Buck Bunny', 'Movie', 100, 700, 600);
             INSERT INTO playback_events(playback_id, at, kind, position_s, detail) VALUES
               (1, 100, 'seek', 1990, '27:12 → 33:10'),
               (1, 200, 'seek', 300, '1:02:03 → 5:00'),
               (1, 300, 'seek', 5, '0:40 → 0:05'),
               (1, 400, 'pause', 5, NULL),
               (1, 500, 'seek', 9, 'garbled');",
        )
        .unwrap();
        c.execute_batch(MIGRATIONS[last]).unwrap();
        let from: Vec<Option<i64>> = c
            .prepare("SELECT from_s FROM playback_events ORDER BY id").unwrap()
            .query_map([], |r| r.get(0)).unwrap().collect::<Result<_, _>>().unwrap();
        assert_eq!(from, [Some(27 * 60 + 12), Some(3600 + 2 * 60 + 3), Some(40), None, None]);
        let indexed: i64 = c.query_row("SELECT COUNT(*) FROM pragma_index_list('playback_events') WHERE name = 'idx_pbe_kind'", [], |r| r.get(0)).unwrap();
        assert_eq!(indexed, 1, "an aggregate over one kind of event must not read every event");
    }

    #[test]
    fn an_address_that_reaches_this_machine_is_never_public() {
        // The unspecified forms are the point: `connect()` to either reaches this machine, so a
        // notification destination spelled that way would be loopback wearing a public face.
        for local in ["0.0.0.0", "::", "0:0:0:0:0:0:0:0", "0.1.2.3", "127.0.0.1", "10.0.0.1", "192.168.1.10",
                      "172.16.0.1", "169.254.169.254", "100.64.0.1", "::1", "fd00::1", "fe80::1", "::ffff:127.0.0.1", "::ffff:0.0.0.0"] {
            assert_eq!(is_local_ip(local), Some(true), "{local} should be local");
        }
        for public in ["8.8.8.8", "1.1.1.1", "203.0.113.7", "99.64.0.1", "101.64.0.1", "2001:4860:4860::8888", "::ffff:8.8.8.8"] {
            assert_eq!(is_local_ip(public), Some(false), "{public} should be public");
        }
        for nonsense in ["", "localhost", "not-an-ip", "999.1.1.1"] {
            assert_eq!(is_local_ip(nonsense), None, "{nonsense:?}");
        }
    }

    #[test]
    fn an_update_is_a_populated_database_a_different_version_now_opens() {
        let n = MIGRATIONS.len();
        // Brand new: nothing to protect, whatever the versions say.
        assert!(!is_update(0, None, "1.7.0", n));
        assert!(!is_update(0, Some("1.6.3"), "1.7.0", n));
        // The same version, fully migrated, restarting: not an update.
        assert!(!is_update(n as i64, Some("1.7.0"), "1.7.0", n));
        // A different (newer) binary opening a populated database: an update.
        assert!(is_update(n as i64, Some("1.6.3"), "1.7.0", n));
        // A pending migration is an update even without a version change (a dev adding one).
        assert!(is_update((n - 1) as i64, Some("1.7.0"), "1.7.0", n));
        // A database from before versions were recorded: unknown past, back it up to be safe.
        assert!(is_update(n as i64, None, "1.7.0", n));
    }

    #[test]
    fn versions_compare_as_numbers_not_text() {
        assert!(parse_version("1.10.0") > parse_version("1.9.12"));
        assert_eq!(parse_version("2.0.0-rc.1+build5"), Some((2, 0, 0)));
        assert_eq!(parse_version(" 1.0.4\n"), Some((1, 0, 4)));
        for bad in ["", "1.0", "1.0.0.1", "one.two.three", "v1.0.0"] {
            assert_eq!(parse_version(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn an_older_version_refuses_a_newer_database() {
        let n = MIGRATIONS.len();
        // New, from before versions were recorded, the same version, an upgrade: all start.
        for (schema, stored) in [(0, None), (n, None), (n, Some("1.0.4")), (n - 1, Some("1.0.3")), (n, Some("0.9.12"))] {
            assert!(refuse_downgrade(&db_at(schema, stored), "1.0.4").is_ok(), "{schema} {stored:?}");
        }
        // A newer patch, minor or major, and a schema from the future: none start.
        for (schema, stored) in [(n, Some("1.0.5")), (n, Some("1.1.0")), (n, Some("2.0.0")), (n, Some("1.10.0")), (n + 1, None), (n + 1, Some("1.0.4"))] {
            let err = refuse_downgrade(&db_at(schema, stored), "1.0.4").expect_err(&format!("{schema} {stored:?}")).to_string();
            assert!(err.contains("older finstats 1.0.4") && err.contains("finstats restore"), "{err}");
        }
        assert!(refuse_downgrade(&db_at(n, Some("1.1.0")), "1.0.4").unwrap_err().to_string().contains("last used by finstats 1.1.0"));
    }

    #[test]
    fn a_version_nobody_can_read_is_named_as_unreadable_not_as_newer() {
        // Written by hand with JSON quotes, the install refused to start saying it had been "last used by
        // finstats "2.0.1", and this is the older finstats 2.0.1" — true of neither.
        let n = MIGRATIONS.len();
        for stored in ["\"2.0.1\"", "next", ""] {
            let err = refuse_downgrade(&db_at(n, Some(stored)), "2.0.1").expect_err(stored).to_string();
            assert!(!err.contains("older finstats"), "{err}");
            assert!(err.contains(&format!("[{stored}]")) && err.contains("not a version finstats can read"), "{err}");
            assert!(err.contains("Nothing was changed") && err.contains("key='app_version'") && err.contains("without quotes"), "{err}");
        }
    }

    #[test]
    fn an_update_snapshots_the_whole_database_before_migrating() {
        let dir = std::env::temp_dir().join(format!("finstats-preupdate-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("finstats.db");
        {
            let db = Db::open(&path).unwrap();
            let c = db.conn().unwrap();
            set_setting(&c, "marker", "keep-me").unwrap();
            // Pretend an older finstats last opened it, so the next open is an update.
            set_setting(&c, VERSION_KEY, "0.1.0").unwrap();
        }
        // Opening as the current (newer) version detects the update and snapshots first.
        let _db = Db::open(&path).unwrap();
        let snaps: Vec<_> = std::fs::read_dir(dir.join("pre-update-backups"))
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "db"))
            .collect();
        assert_eq!(snaps.len(), 1, "exactly one pre-update snapshot");
        // The snapshot is a real database that still holds the data, and it captured the OLD version —
        // i.e. it was taken before the upgrade rewrote anything.
        let snap = Connection::open(&snaps[0]).unwrap();
        assert_eq!(get_setting(&snap, "marker").unwrap().as_deref(), Some("keep-me"));
        assert_eq!(get_setting(&snap, VERSION_KEY).unwrap().as_deref(), Some("0.1.0"));
        // A plain restart of the same version does not pile up another snapshot.
        drop(_db);
        let _db = Db::open(&path).unwrap();
        let count = std::fs::read_dir(dir.join("pre-update-backups")).unwrap().filter_map(|e| e.ok()).filter(|e| e.path().extension().is_some_and(|x| x == "db")).count();
        assert_eq!(count, 1, "a restart of the same version must not snapshot again");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_transaction_holds_the_write_lock_from_its_first_statement() {
        // A deferred transaction that reads before it writes cannot write at all once anybody else has committed in
        // between — SQLITE_BUSY_SNAPSHOT, which no busy timeout waits out. Under the collector's steady writes that
        // failed the group detection after an import (so the import said it failed) and could fail any read-then-write.
        let dir = std::env::temp_dir().join(format!("finstats-txlock-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let db = Db::open(&dir.join("finstats.db")).unwrap();
        let mut mine = db.conn().unwrap();
        let other = db.conn().unwrap();
        other.busy_timeout(std::time::Duration::ZERO).unwrap();
        let tx = mine.transaction().unwrap();
        let _: i64 = tx.query_row("SELECT COUNT(*) FROM playbacks", [], |r| r.get(0)).unwrap();
        assert!(other.execute("INSERT INTO settings(key, value) VALUES ('between', 'x')", []).is_err(), "another connection committed inside a transaction that had begun");
        tx.execute("INSERT INTO settings(key, value) VALUES ('mine', 'x')", []).unwrap();
        tx.commit().unwrap();
        drop((mine, other, db));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_journal_is_kept_to_a_size_after_a_big_write() {
        // SQLite reuses the write-ahead journal and never shrinks it on its own: after a 120,000-play import it
        // stayed 95 MB for good. journal_size_limit truncates it to this size whenever it is reset.
        let dir = std::env::temp_dir().join(format!("finstats-journal-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let db = Db::open(&dir.join("finstats.db")).unwrap();
        for _ in 0..3 {
            let limit: i64 = db.conn().unwrap().query_row("PRAGMA journal_size_limit", [], |r| r.get(0)).unwrap();
            assert_eq!(limit, JOURNAL_LIMIT, "every pooled connection");
        }
        drop(db);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_damaged_or_foreign_file_is_refused_at_once_and_left_as_it_was() {
        let dir = std::env::temp_dir().join(format!("finstats-preflight-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let refused = |path: &Path, why: &str| {
            let before = std::fs::read(path).ok();
            let started = std::time::Instant::now();
            let err = format!("{:#}", Db::open(path).err().unwrap_or_else(|| panic!("{why}: opened")));
            assert!(started.elapsed() < std::time::Duration::from_secs(5), "{why}: took {:?} to refuse — a supervisor sees a hang", started.elapsed());
            assert!(err.contains(&path.display().to_string()) && err.contains("Nothing was changed"), "{why}: the message names neither the file nor what happened to it: {err}");
            assert_eq!(std::fs::read(path).ok(), before, "{why}: the file was changed");
            let wal = path.with_extension("db-wal");
            assert!(!wal.exists() || std::fs::metadata(&wal).unwrap().len() == 0, "{why}: a journal was left beside it");
        };
        // Random bytes, and a finstats database cut in half — what a failed copy or a full disk leaves.
        let garbage = dir.join("garbage.db");
        std::fs::write(&garbage, (0..300_000u32).map(|i| (i.wrapping_mul(2_654_435_761) >> 13) as u8).collect::<Vec<_>>()).unwrap();
        refused(&garbage, "random bytes");
        let cut = dir.join("cut.db");
        drop(Db::open(&cut).unwrap());
        let whole = std::fs::read(&cut).unwrap();
        std::fs::write(&cut, &whole[..whole.len() / 3]).unwrap();
        let _ = std::fs::remove_file(cut.with_extension("db-wal"));
        let _ = std::fs::remove_file(cut.with_extension("db-shm"));
        refused(&cut, "a database cut short");
        // Somebody else's database, which the data folder was pointed at by mistake: never written into.
        let foreign = dir.join("foreign.db");
        Connection::open(&foreign).unwrap().execute_batch("CREATE TABLE notes(id INTEGER PRIMARY KEY, body TEXT); INSERT INTO notes(body) VALUES ('mine');").unwrap();
        refused(&foreign, "another program's database");
        // Read-only (a restored copy that kept the wrong permissions): it can be read, and nothing can be recorded.
        let locked = dir.join("locked.db");
        drop(Db::open(&locked).unwrap());
        let mut perms = std::fs::metadata(&locked).unwrap().permissions();
        perms.set_readonly(true);
        std::fs::set_permissions(&locked, perms).unwrap();
        if std::fs::OpenOptions::new().write(true).open(&locked).is_err() {
            refused(&locked, "a read-only file"); // (not provable as root, who may write anything)
        }
        // A new install still starts: no file, or an empty one.
        drop(Db::open(&dir.join("new.db")).unwrap());
        std::fs::write(dir.join("empty.db"), b"").unwrap();
        drop(Db::open(&dir.join("empty.db")).unwrap());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_snapshot_cut_short_is_never_kept_under_a_snapshots_name() {
        // VACUUM INTO writes straight into the file it is given, so a start killed while copying left an empty
        // `finstats-<version>-<stamp>.db` (and its journal) — which SQLite calls a valid empty database, and which
        // counted toward the three kept, pushing a real one out.
        let dir = std::env::temp_dir().join(format!("finstats-snapcut-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("pre-update-backups")).unwrap();
        let path = dir.join("finstats.db");
        {
            let db = Db::open(&path).unwrap();
            let c = db.conn().unwrap();
            set_setting(&c, "marker", "keep-me").unwrap();
            set_setting(&c, VERSION_KEY, "0.1.0").unwrap();
        }
        // What a killed start left: the copy it was writing, and that copy's journal.
        let snaps = dir.join("pre-update-backups");
        std::fs::write(snaps.join("finstats-0.1.0-20200101-000000.db.part"), b"").unwrap();
        std::fs::write(snaps.join("finstats-0.1.0-20200101-000000.db.part-journal"), [0u8; 512]).unwrap();
        drop(Db::open(&path).unwrap());
        let names: Vec<String> = std::fs::read_dir(&snaps).unwrap().flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect();
        assert_eq!(names.len(), 1, "what a killed snapshot left is still there: {names:?}");
        assert!(names[0].ends_with(".db") && !names[0].contains(".part"), "{names:?}");
        let snap = Connection::open(snaps.join(&names[0])).unwrap();
        assert_eq!(get_setting(&snap, "marker").unwrap().as_deref(), Some("keep-me"), "the snapshot kept does not hold the database");
        drop(snap);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn opening_records_the_version_and_a_refusal_changes_nothing() {
        let dir = std::env::temp_dir().join(format!("finstats-version-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("finstats.db");
        let running = env!("CARGO_PKG_VERSION");

        let db = Db::open(&path).unwrap();
        assert_eq!(get_setting(&db.conn().unwrap(), VERSION_KEY).unwrap().as_deref(), Some(running));
        set_setting(&db.conn().unwrap(), VERSION_KEY, "999.0.0").unwrap();
        drop(db);

        assert!(Db::open(&path).is_err());
        let c = Connection::open(&path).unwrap();
        assert_eq!(get_setting(&c, VERSION_KEY).unwrap().as_deref(), Some("999.0.0"));
        drop(c);

        // An upgrade moves the mark forward.
        Connection::open(&path).unwrap().execute("UPDATE settings SET value = '0.1.0' WHERE key = ?1", [VERSION_KEY]).unwrap();
        let db = Db::open(&path).unwrap();
        assert_eq!(get_setting(&db.conn().unwrap(), VERSION_KEY).unwrap().as_deref(), Some(running));
        drop(db);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
