# Patch notes

Everything that changed in finstats, newest first. This file is compiled into the
binary and is what the **Patch notes** tab in the app shows.

Format: `## [version] - date`, or `## [version] - first day to release day` for a release made
over several days, then `### Added`, `### Changed`, `### Performance`, `### Stability`, `### Fixed`
or `### Removed` with one bullet per change. Performance is something that got quicker or lighter,
Stability something that can no longer crash, halt, leak or lose data, Fixed something that gave a
wrong answer. An `x.y.0` release carries a short title line under its heading: that title is what
the app uses as the headline of the whole series.

## [2.1.0] - 2026-09-29 to 2026-10-01

Watchlist

### Added
- **A watchlist of your own**: put a film or a show on it from its page, from Upcoming, from Recently added or from search (Ctrl+Enter), and find it under **Watchlist** on your own profile.
- **Every entry says where it stands now** — on the server, "5 of 26 episodes", requested, "Coming up Friday", not on the server, left the library, or watched — worked out each time you look, with the same reading of "seen" as your show progress. Watched titles stay, in a group of their own, until you take them off.
- **A title that is not on your server yet** can go on the list straight from what Sonarr or Radarr is waiting for. It is attached to the title by itself when it arrives, and a renamed file takes its entry along.
- **"On your watchlist: now available"**: a notification for your own destinations when something you were waiting for arrives. Off until you tick it.
- **Nobody else sees your watchlist**, an administrator included, and its notification goes to your destinations only. Nothing is written to Jellyfin, Seerr, Sonarr or Radarr.
- Watchlists are part of a backup, and a restore merges them without doubling anything.

### Changed
- `/api/upcoming` entries carry the TVDB and TMDB ids of their title, and `/api/auth/me` says whether the server keeps watchlists.

### Stability
- Saving a Sonarr, Radarr or Seerr connection while somebody had **Settings → Outbound** open could leave both waiting on each other for good.
- **Settings that cannot be read stop finstats at start-up**, naming the setting at fault, instead of quietly resetting every setting to its default.
- **Two changes to the settings at once** — the settings page and a task's schedule, say — no longer undo each other.
- A pause, seek or track change seen while the database was busy is written on the next pass instead of lost.

### Fixed
- One failed read of Jellyfin's plugins, scheduled tasks or storage no longer empties that list on the Server page until the next read.
- **A Streamystats play stops where it stopped on every page**: a title's drop-off curve and an episode's "finished" went by its length instead, and counted it as estimated.

## [2.0.4] - 2026-09-29

### Added
- **Task scheduling, the way Jellyfin does it**: every job finstats does by itself has triggers — daily at a time, weekly on a day, on an interval, at start-up, or after Jellyfin's library scan — each with an optional time limit. Click a task to see and change its schedule; *Add trigger* opens the same dialog Jellyfin has.
- **Tasks is a section of its own** in Settings, with each job's last run in words ("Last ran 38 minutes ago, taking 6 minutes") and a Run button; it used to sit at the top of System.
- **Metadata changes**: every hour finstats reads what Jellyfin saved since the last look — names, overviews, genres, ratings, cast and crew, file details and pictures — so an edit in Jellyfin shows up without waiting for a library scan.
- Backup and the geolocation database can be run from Tasks too; Esc steps back from a task's schedule to the list.

### Changed
- **The theme is a switch with three stops**, Light · Device · Dark, where the button that cycled through them was: click a third of it, drag the knob or use the arrow keys. The knob carries the choice's icon, which moves in when it changes.
- **Signing out is under Settings → Account**, which says who is signed in here; the sidebar keeps the theme switch.
- **The old schedule settings are now triggers**: *Follow Jellyfin's library scan* with its fallback interval, *Back up every N days* and the GeoIP *Keep the database up to date* switch became the matching triggers on their tasks, with the values you had. Collection, Backups and Security link to the schedule instead.
- A start no longer runs every job again: when each one last finished is kept.
- A Jellyfin job without an estimate yet reads "about … left", the dots cycling where the time will go, instead of "ETA…".

### Performance
- **The Security map no longer lags**: dragging and zooming move the map already drawn and draw it once when the gesture settles, and the playing-now pulse no longer makes the whole map repaint 120 times a second while nobody touches it. A drag went from 34 to 60 frames a second on a slow computer.

### Fixed
- **A poster or a portrait replaced in Jellyfin shows up in finstats**: pictures were cached for a week on the server and another week in the browser, whatever Jellyfin had since. They are now cached under Jellyfin's image tag, so a new picture is a new file, and the browser checks that its copy is current each time it shows one. Replacing a portrait re-saves only the person, so Metadata changes reads the people too.
- **Server → Jobs shows an estimate as soon as the page opens**: the time left was measured only from readings the page itself made while open, so every visit started on "ETA…" until it had watched the percentage move. finstats now times a running Jellyfin job by itself, every 10 seconds while one runs, and the page is served the estimate. A job Jellyfin reports at 0% the whole time (a library scan's first phase) keeps its dots until it moves: nothing is guessed.
- **Map tooltips appear on the dot**: in Firefox the playing-now dot's tooltip could land far away, and where dots overlap (somebody playing at home) which tooltip showed depended on a pixel. One tooltip now names everything under the pointer.
- **The status bar has one hover shape**: every link — the streams, Repo, the version — lights up as the same small box, and the dots between the items sit centred in the gaps. Hovering the version used to light up the dot and the gap before it too.
- **A file path takes the whole width of its card**: on a title's File card, and for Jellyfin's folders on the Server page, a path was squeezed into one narrow cell and ran down nine lines beside empty space.
- A dialog fading out no longer swallows a click on the page under it.

## [2.0.3] - 2026-09-28

### Stability
- **Only one import or restore at a time**: two uploads posted together both passed the check, and a restore never looked at imports; the second waited out the database and failed after minutes of upload.
- **An address with a broken `%`-escape opens the not-found page**, instead of leaving a spinner for good.
- **A large page number in a list is an empty page**: it overflowed, and came back as the first page again.

### Fixed
- **Security:** the Server page gave every device's id to anybody with *see server details*; ids need *see network details* as well, as everywhere else.
- **Security:** two permission switches flipped before the first had saved sent the same starting list twice, and the second undid the first — a permission taken away stayed granted, a public-profile section switched off stayed published, while both switches showed what was chosen.
- **Tables sort numbers right in a German or Norwegian browser**: "1,204" was read as 1.204, and anything past a million as text.
- **A chart's Table view sorts dates, weekdays and positions by when**, not by their letters.
- **The download history names where imports came from**: Sonarr and Radarr name the indexer and protocol only on the grab, so those breakdowns were always empty. A failed season pack is one failure, not one an episode.
- **Deleting a play watched together** finds that title's groups again, instead of leaving a "group" of one.
- **Time together, for one person, is never more than they stayed**: the Together page credited them with the whole evening's.
- **An evening in company across midnight on New Year's Eve** (or the edge of "last N days") belongs whole to the window it started in, instead of two evenings alone.
- **Imported episode plays are called by the episode's name** where Jellystat kept only the series'.
- **Geolocation finds the city database beside the Country and ASN ones** `geoipupdate` writes, instead of switching itself off.
- **The most-rewound hot spot follows the page's filters**, like its counts.
- **"Your year is ready"** goes only to people whose recap has something in it.
- **Resolving a muted alert again keeps its pair muted**; only reopening undoes a mute.
- **An expired API key no longer counts toward the twenty.**
- **The session limit lets the oldest session go** even when many sign-ins land in the same second.
- **A home address found by the lookup stays** when the same one is taken off the typed list.
- **Pictures in the audit log and the keys list**, where there were only initials.
- **An administrator's notification list says whose a personal destination is**, instead of "Yours" on everybody's; **Recently sent** shows every destination of its last event.
- **Esc on a title or a person never leaves finstats**, and closes only the dialog on top.
- **A page is drawn even when a save cancels the prefetch it was waiting on.**
- **Now playing shows a paused viewer who scrubs** where they are now, not when they press play.
- **Removing a connection updates which pages exist**, as saving one does.
- **The geolocation database's build date keeps its year** in US English.

## [2.0.2] - 2026-09-28

### Stability
- **A halt is never lost**: when a library read was refused in the first moments after start-up, before the server was listening for it, the refusal was noted and finstats ran on instead of stopping.
- **A Jellyfin that answers in a shape finstats cannot read is refused**, not believed: every user, or every title of a library, arriving under keys it does not know (`id` for `Id`) passed the guard with a full count and was then marked removed.
- **A play's end is never dropped**: when the database was busy past its wait (an import holding it), a play that ended in that moment stayed marked as playing until the next restart. It is written on the next pass now.
- **Settings a restore cannot read are left alone**: a backup whose settings no longer fitted this version reset every setting on restore — merge window, permissions, home addresses, schedule.
- **A control character in a name or a title no longer breaks the cards**: the profile card, every story card that showed that title and the ZIP of them answered with an error.

### Fixed
- **Security:** behind a proxy, the sign-in limit counted the first `X-Forwarded-For` address, which is the one the client sends — a new made-up address per attempt, and the limit never applied. It is the last one now, the one the proxy wrote.
- **Security:** *Include IP addresses and places* sent addresses to a personal destination whose owner lacks *see network details*, which no page would show them. Somebody else's failed sign-ins now need *see server details* too, as on the Security page.
- **Notifications stop for somebody disabled or deleted in Jellyfin**: their personal destinations went on being sent everything their old permissions allowed.
- **A queued notification is asked again before it is sent**: a destination switched off, or an owner who had lost a permission, was still sent what was waiting to be retried.
- **A destination that always answers `Retry-After` is given up on** after five tries like any other, instead of being retried for thirty days.
- **Device ids on a person's page** are shown only with *see network details*, as everywhere else.
- **A published year waits a day for everything**: its finished and dropped shows followed every play, so pressing play on a dropped show changed the public page within seconds.
- **A requested show counts as watched** when one of its episodes is: every requested show read as never watched, in the Requests chapter and on the Asked card.
- **The recap's Together chapter leaves Live TV out**, like the rest of the year: a channel left on together could become the top title, and the share could pass 100%.
- **The Days card of "the last 12 months" ends on the newest day**: the grid dropped the most recent three or four weeks.
- **A show's new episodes are announced once per evening**: episodes either side of midnight UTC were announced twice on a server east or west of it.
- **Connecting Seerr does not announce old requests as ready to watch**: a request already available when first read, with no other record of when, read as arriving that moment.
- **The titles of a library deleted in Jellyfin go with it**: they stayed in search, on the Recently added shelf and in the totals.
- **A remux stays a direct stream** when a reading loses its transcoding details for a moment (a seek), instead of becoming a transcode for the rest of the play.
- **"Jellyfin is down" means reads failed in a row**: stray timeouts days apart on a server idle on the socket added up to it.
- **A Streamystats import shows its progress on its own card**, not over the Jellystat import's last result.
- **An address whose host starts with "web"** (`http://web:8096`) is read as a host, not cut short as Jellyfin's web client path.
- **A queue error blanks the whole login** of an address even when the password holds a slash.

## [2.0.1] - 2026-09-28

### Changed
- **A title's languages stay out of the way**: the header names the first three audio and subtitle languages, one line each, and a new **Languages** card lists every one — the complete ones together, the partial ones with how many episodes have them. A long show's two dozen subtitle languages used to push the artwork down a screen.

### Fixed
- **A title's poster stays beside its name**: when the text next to it ran longer than the poster — a long synopsis, many studios — the poster sank to the bottom of the header, under empty artwork.
- **A version finstats cannot read is called that**: an `app_version` edited by hand into something unreadable made finstats refuse to start as "the older finstats 2.0.1", which it was not. It still refuses, and now says which value it could not read and how to write it.

## [2.0.0] - 2026-09-25 to 2026-09-28

Recap 2026, public profiles and watching together

### Added
- **Who you watched with**, in the recap: evenings in company, the time spent in it, the title that brought people together most, and up to three companions, named in the app and linked to their profiles.
- **Shows seen to the end**, in the recap: the last episode of each seen that year, by the same reading of "seen" as the profile's progress bars — and the ones begun and left for later, under half seen with nothing played for two months.
- **What you asked for**, in the recap: Seerr requests made in the year, how many arrived and how many you then watched. Only where requests exist.
- **The year against the one before**: hours, plays and days, and how much more or less.
- **The year as a story**: every chapter as a 1080×1920 card, the shape phone stories use — the year, the numbers, top shows, films and music, genres, the persona, hours and days, every day of the year, records, company, shows finished, requests and the year before. Save one, or all of them as a ZIP, from "Share the year as cards".
- **A card never names anybody else**: cards are drawn from a copy of the year with no field for another person, a rank or an app. In the app a card carries no name at all.
- **The server's year**, for Jellyfin administrators: every title and total and the persona of the house, with nobody ranked or named. A switch on the recap page, `scope=server` in the API.
- **"Your year in review is ready"**: a notification in December, once to each person who watched that year.
- **Public profiles**: a person can publish part of their profile at a link anyone can open without an account — totals and top titles, streaks and when they watch, the whole year as its story of cards, the last ten plays — each part off until its owner switches it on under Settings → Public profile. Devices, addresses, apps, play methods, file paths and other people are never published, whatever is switched on, and the name shown is one the owner types, never their Jellyfin login.
- **Nothing on a published page is newer than a day**, recent plays and every total alike, so a page someone keeps an eye on never says who is watching right now.
- **Shareable cards**: every published profile comes with a 1200×630 picture, of the profile and of the year, that Discord, Signal, Mastodon and the rest show when the link is pasted, and that can be downloaded to post. Drawn on the server from what the page shows and nothing more.
- **A link you can take back**: 22 random characters, reset with one click; the old link then answers exactly like one that never existed.
- **Public profiles are off until an administrator allows them**: one switch for the whole server, a list of who publishes what, and a way to take a profile down. Publishing, changes, resets and take-downs are in the audit log, and a key cannot make anything public.
- **Together**, a page of its own: who watches with whom, how much, and how that has changed. Time together, evenings and the share of watch time spent in company, each against the window before; hours in company against hours alone per day or week; every pair of people with their evenings, time together, when they last watched and what they watch most; the titles watched together most; the recent evenings; and, for anyone who may see everyone, each person's time alone against their time in company.
- Every pair counts: an evening of three counts for each of its three pairs, where the card used to count only the exact set of people. A pair's time together is the shorter of the two stays.
- The **Watched together** card on the dashboard and on a profile links to the page, and a profile's card says what share of that person's watch time was in company.
- `GET /api/stats/groups` gains `pairs`, `series`, `people`, `previous`, `bucket`, and `watch_s` and `share` in its totals; the keys it had are unchanged.
- **Where people stop**, on every film and episode page: the share of plays still watching at each minute, from everyone at the start to whoever reached the end, with the rewinds and subtitle switch-ons of those plays marked on the same axis. The card says how many stops were measured (plays finstats recorded) and how many estimated (imported plays, from how long they ran), and needs three plays before it draws anything.
- **Who keeps watching**, on a show page: how many people started each episode, in order, and a **Finished** column per episode. "Everyone quits episode three" is people who never press play on episode four.
- **Files that never play**, on Playback: a film or episode started three times or more that never got past thirty seconds, with who tried and on which apps. Jellyfin's log carries no playback errors, so the plays are the witness; the minimum play length does not apply here, since it is exactly what those plays never reach.
- **Most rewound** and **Subtitles switched on**, on Playback: titles ranked by backwards skips per play with the minute they cluster in, and by the share of plays where subtitles went on within the first ten minutes. All three lists need *see everyone's activity*. Skips shorter than 20 seconds were never recorded, so a short replay of one line is invisible by design.
- A seek keeps where it came from as a number beside its label (`from_s` in a play's timeline). Every seek already recorded gets its number from the label, and so does a restored backup from before this.
- `GET /api/stats/files` for the three lists; `GET /api/items/{id}` gains `insights`, and a show's episodes gain `users` and `finished`.
- **API keys**: anyone signed in can make keys for themselves under Settings → API keys, for a script, a dashboard or a phone. A key is sent as `Authorization: Bearer fs_…`, is shown once, stored hashed, can carry an expiry, and is revoked with a click — an administrator sees and can revoke everyone's. It resolves to the same person with the same permissions as a sign-in, read live on every request, so losing the right to sign in stops every key at once. A key can neither make nor revoke keys.
- **A calendar feed**: `GET /api/calendar.ics?key=…` is what Sonarr and Radarr have coming, as an iCalendar a phone can subscribe to. A key with the *calendar* scope opens the feed and nothing else; the feed reads the key from the address because a subscribed calendar can send no header, never the cookie, and names nobody.
- **An audit log** under Server → Audit, for Jellyfin administrators: every sign-in and failed attempt with its address, every setting or permission changed and to what, every key made, used or revoked, every connection, destination, backup and import, every play deleted and alert resolved — who, from where, through which key, and whether it worked. Kept a year, part of backups. `GET /api/audit` pages through it.
- **Import from Streamystats**, its own section under Settings beside Jellystat. Take its **Settings → Backup & Import → Download Backup** file and drop it in: years of history arrive in a couple of seconds, and nothing you already have is counted again. `finstats import-streamystats <file>` does it headless, `POST /api/import/streamystats` through the API; one import runs at a time, either kind.
- **Ran both trackers? Import both files.** A play is recognised as one finstats already has by the tracker's own id for it and, failing that, by the same person watching the same item with either end of the play close to one in the history — so a Jellystat export and a Streamystats export of the same evenings can both be imported, in either order, and an evening finstats watched itself is not imported over either. "Close" is the Merge window under Settings → Collection.
- Imported rows say what they cannot say. A play Streamystats watched itself keeps nothing about the file — no codec, resolution, bitrate or container — so those stay empty rather than being filled with a guess, and the cards that would show them hide themselves. A play it had taken from Jellystat carries the whole session, and that is read instead: dated by the moment it ended, the only moment either tracker kept, and described as a transcode only when the session really was one.
- Rows Streamystats wrote because Jellyfin reported an item watched, for a viewing it never saw, are counted and skipped: they are as long as the whole film and would be watch time on an evening nobody watched. The result says how many there were.
- How Streamystats data is interpreted, field by field, learned from a real export rather than from documentation: `docs/streamystats-import.md`.
- **Which tracker each play came from**, as a filter on Activity: what finstats recorded itself, what came from Jellystat, what came from Streamystats. Only the ones your history actually holds are offered, and it is scoped like every other list, so somebody who may only see their own plays is only told where their own history came from.
- **The filters on Activity are dropdowns you tick**, so you can ask for films *and* episodes without music, two play methods, or two people at once, rather than one of each or all of them. Media type gained **Other** now that it can be combined.
- Several people at once, anywhere the user filter appears — the dashboard, playback, libraries, security and the rest — with the map and every statistic following. A user filter still only ever narrows what somebody may see: without *see everyone's activity* a request is pinned to the caller, whoever the address names and however many.
- **A light theme**: washi paper, with ink text and a vermilion accent. finstats follows the device's light or dark setting until you pick one.
- **A theme button beside Sign out** cycles Device, Light and Dark. The choice is kept in this browser and applied before the page is drawn, so it never flashes the other theme.
- **Open in Jellyfin**: every film, show and episode page has a button that opens that title's page in Jellyfin, in a new tab.
- **Jellyfin's address for people** under Settings → Jellyfin: where those buttons point, for when the address finstats connects to is not one a browser can reach. Empty keeps the address finstats connects to; only a Jellyfin administrator can set it.

### Changed
- **finstats has a new home**: `github.com/finstats/finstats`, and the image is `ghcr.io/finstats/finstats`. Change the image name in your `docker run` or compose file to keep getting updates; the data folder stays as it is.
- **Settings is one section on screen at a time.** The page was fourteen cards in one column and six hundred words of help. Now each section has its own address (`/settings/collection`), a list on the left moves between them, and on a phone the list wraps into chips. Links to the old anchors still land.
- **A setting is one row**: what it is, one line stating the rule, the control on the right. Numbers in a section save together with one button at the bottom; switches still save themselves. The status cards (Tasks, Outbound connections, Database) and the licences link live under **System**, and the two importers have a section of their own.
- **Find a setting** from the box in the header — `/` focuses it — by any word in its name; it says which section the setting lives in, and Enter lands on the row.
- **Settings is for everyone signed in**: the sections that manage the server stay with those who may manage it, and a person who was only allowed notifications can finally reach their own destinations.
- **The Server page is one section on screen at a time**, the way Settings is: Overview (system and storage), Jobs, Devices, Plugins, Log and Audit, each at its own address. Links to `/server#jobs` still land.
- **The server log is a section of the Server page** (`/server/log`) rather than a page of its own in the menu; `/events` forwards there with its filters.
- A notification about a failed job, a failed backup or a connection that stopped answering links to the settings section that holds it, rather than to the top of the page.
- What your library can tell an imported play — the kind of thing it was, which episode, its runtime, which library it belongs to — is filled in after **every** library read, not only at the moment of an import. History is usually imported before finstats has ever read the library, so those plays used to stay guessed or unknown for good.
- Restoring a finstats backup no longer adds a play that arrived from a tracker under a different id, and still keeps two genuine viewings of the same thing minutes apart.
- A play's details name whichever tracker brought it in, rather than calling everything imported a Jellystat play.

### Performance
- **A restart is near-instant on any history.** Every start worked out who watched together over the whole history and re-decided which plays were local, both to change nothing: 24 s and 5.5 s on ten million plays. A start now does only what the last run left behind, and all of it again only after an update or when the setting behind it changed.
- **A crowd arriving at once is recorded in seconds.** Each new play, each progress save and each device was written on its own, and each ended play closed with its own look at the title's whole history; a pass now writes each kind at once. 100,000 plays starting together take 3 s, where 25,000 ending together used to take over four minutes.
- **Now playing stays quick with thousands watching.** Finding who watches together compared every stream with every other and listed every companion by name: forty seconds at 10,000 streams, and gigabytes of memory beyond. Streams are now compared within a title, and each shows a few names and how many others.
- Working out who watched together rewrote every grouped play of the history each time finstats started, imported or saw a play end; it now writes only what changed. Finding which plays had ended no longer gets slower with the square of the number playing.
- **Recognising a play the history already has goes through an index**, so an import of 150,000 plays takes seconds, and re-linking after a library read steps through the titles instead of reading every play while holding the database.
- **Playback insights and the overview are quicker over all time**: their independent parts are worked out side by side, the concurrency chart looks up the local day once per quarter hour instead of once per play, and genres are counted per title. On a million plays: 4.1 s to 2.4 s, and 1.6 s to 1.1 s.

### Stability
- **A library of television no longer reads as a library that has been gutted.** Reading a library makes two passes — every item, then a smaller one for the cast and crew of films and shows only — and the check that decides whether a read can be trusted to remove what it did not see was looking at the second. On a library of television or music the two counts are nowhere near, so the check refused every read and stopped finstats, and the install restarted and did the same again. Nothing was ever wrongly removed, but no library read could finish. If your Shows or Music library has not been picking up new titles, this is why.
- An import could fail with "database is locked" when another write — a play being refreshed — landed between its first read and its first write. Both importers take the write lock as they begin, and the database journal no longer stays as large as the largest import ever made.
- **A database file that is damaged, cut short, somebody else's or read-only is refused at once**, naming the file and saying nothing was changed, instead of 30 silent seconds; another program's database is never written into.
- A backup or pre-update snapshot interrupted half way no longer leaves a file behind — one that could take a good snapshot's place among the three kept.
- **Overwriting the place database while finstats runs no longer crashes it**: finstats reads from a private copy of the file.
- A Jellyfin app that reported a new device id every time could make finstats' memory grow for as long as it ran; devices not seen for five minutes are now forgotten.
- One person keeps at most 30 signed-in sessions; the oldest is signed out at the next sign-in.

### Fixed
- **The same evening is no longer counted twice when an item has been renamed.** Re-linking history to a title that was re-added in Jellyfin could turn a play imported from a tracker into a duplicate of one already there, because the check that would have caught it ran before the id moved. It now runs again wherever ids are re-linked. Only imported plays are ever removed, never one finstats recorded itself, and how many went is in the log. The first start of 2.0 sweeps the whole history once for the ones already there.
- **A session now follows the person as Jellyfin has them now.** An administrator Jellyfin demoted kept full access to finstats for the rest of their 30-day session, and somebody disabled or deleted in Jellyfin stayed signed in. They now lose it as soon as finstats reads Jellyfin's users (every 15 minutes, or at once from Settings), as API keys always do.
- Signing in writes what Jellyfin says about the person into finstats' user list, so an imported tracker backup can no longer make somebody an administrator before the first read of the users corrects it.
- Somebody deleted in Jellyfin is marked removed even when two reads of the users fall in the same second.
- **Download errors never show a download client's password or an indexer's API key**, which Sonarr and Radarr can quote in them.
- Plays re-linked to a renamed title during a library read were not checked for watching together until the next restart; they now are, at once.

## [1.6.5] - 2026-09-24

### Added
- **A full backup is taken automatically before every upgrade.** The first time a newer finstats opens your database, it writes a complete copy of it — your whole history, settings and all — to `data/pre-update-backups/` *before* it changes anything. If an upgrade ever breaks something, your data is safe: stop finstats, put the copy back in place of `finstats.db`, start the version you were on, and report the bug — nothing is lost. The newest few copies are kept; set `FINSTATS_SKIP_PREUPDATE_BACKUP=1` to turn it off if disk space is tight.

## [1.6.4] - 2026-09-24

### Added
- **Five more places to send notifications**: Telegram, Slack, Pushover, Pushbullet and e-mail, beside the webhook, Discord, ntfy and Gotify destinations already there. Each asks only for what it needs — a chat id, a user key, a mailbox — and the three that live at one service (Telegram, Pushover, Pushbullet) can only ever be reached there, so a token cannot be posted to a look-alike host. Mail goes out over TLS from the first byte or `STARTTLS`, and there is no third option.
- **Licences**, under **Settings**: finstats' own GPL and every third-party licence it is built on — 253 Rust crates, the two bundled fonts, the map outlines and the geolocation database — each text in full, as its own project wrote it. Pick a component and its licence opens in the window beside the list; a crate that ships several files switches between them in place. The list is generated from the crates finstats is actually built from, so a dependency cannot be added without its licence being recorded.

## [1.6.3] - 2026-09-24

### Fixed
- The safety guard now also steps in when Jellyfin lists *no* libraries or *no* users at all — a broken read, never a normal day — which the 1.6.2 tuning had let through. Losing some libraries is still treated as ordinary; losing every one of them at once is not.

## [1.6.2] - 2026-09-24

### Fixed
- The data-integrity guard added in 1.6.1 was too eager: a library genuinely losing most of its items — a handful of clips whose files went, say — was mistaken for a broken read, so finstats refused to update it and stopped. It now steps in only for a *clearly* broken read: a library big enough to matter that reads back completely empty, or a large one gutted to almost nothing. Everyday changes apply as before, and the "Jellyfin returned nothing" case is still caught.

## [1.6.1] - 2026-09-24

### Added
- **finstats no longer trusts a Jellyfin read that would wipe your history.** If a library that finstats holds thousands of items for suddenly reads back empty or nearly so — a Jellyfin upgrade that changed its API, an error dressed as an empty result — finstats keeps the data it has, marks the sync failed (and notifies), and stops cleanly with a clear message rather than marking everything removed. Restart once Jellyfin is itself again, or set `FINSTATS_ALLOW_LIBRARY_SHRINK=1` if you really did empty a library. The same guard covers the library list and the user list.

### Fixed
- **Security:** an `X-Forwarded-Host` header sent by a client could vouch for a foreign `Origin` and get a cross-site write past the same-origin guard. That header is trusted only behind a trusted proxy now (`FINSTATS_TRUST_PROXY`), the same rule the client-address lookup already uses. The `SameSite=Lax` session cookie was, and stays, the first line of defence.
- **Security:** a personal notification destination could be aimed at `0.0.0.0` or `::`, which reach this machine — the check that keeps personal destinations pointed at public addresses now counts those as local, alongside loopback and the private ranges.
- The **Playback** page no longer fails to load for a library that contains a play with no recorded method (a row restored from a backup older than the column); such a play simply counts as a direct play.
- A very long word in a search or a filter is no longer a server error: SQLite refuses a `LIKE` pattern past 50,000 characters, so filter words are cut to a sane length before the query.
- The **Settings** page no longer scrolls sideways on a phone when a card reports a whole sentence back, such as the result of a restore.

## [1.6.0] - 2026-09-24

Notifications

### Added
- **Notifications** card under Settings: add a destination, tick what it should hear about, press Test. No destination means no request is made to anything.
- Four kinds of destination: a plain **webhook** (JSON, documented in `docs/api.md`, optional `Authorization` header), **Discord** (an embed through a channel webhook URL), **ntfy** (ntfy.sh or your own, to a topic) and **Gotify** (your own server, with an application token). Titles travel in the message rather than in a header.
- Eleven kinds of event, each a tick of its own: impossible travel, an account seen in a new country, a burst of failed sign-ins, a task that failed, a backup that could not be written, something that stopped answering (Jellyfin, Sonarr, Radarr or Seerr) and when it answers again, new titles in the library (folded per show and day, as the dashboard shelf folds them), a request that became watchable, and a play beginning or ending. The last two are off by default.
- A request that became watchable names the title, who asked, and how long they waited.
- Personal destinations: grant **Be sent notifications** and somebody can add one of their own, which is sent only what they may already see — their own requests and alerts always, other people's with *see everyone's activity*, addresses with *see network details*. A personal destination must point at a public address; only an administrator may aim one inside the network.
- **Include IP addresses and places**, per destination and off by default: a message about impossible travel otherwise names places only ("Oslo, Norway and London, United Kingdom").
- **Recently sent**, in the same card: what was said, to which destination, and what the other side answered when it did not arrive. A failed message is retried four times over about an hour, and a destination that is rate-limiting is waited for.
- **The address of finstats**, a setting for administrators, is what a notification's link is built from. Empty means messages carry no link.
- Jellyfin's own scheduled tasks on the Server page: what each one does in words rather than by the name of its code, and whether it is one of the heavy ones. A running task shows a progress bar, a percentage and how much is left; the rest show when they next run, when they last ran and whether that went well. Tasks Jellyfin hides from its own dashboard are included, and nothing in finstats can start or stop a task.
- How much of a task is left is measured from the rate its percentage has actually moved at, against the most recent reading far enough back to mean something rather than the average of the whole run, so a task that speeds up or slows down is described by the pace it has now.
- The task lists finstats already reads for other reasons — the library-scan check every 5 minutes, the server details every 15 — feed the same measurement, so a task that has been running an hour usually has an estimate the moment the page opens. Nothing extra is asked of Jellyfin for it.
- A running task with nothing measured yet shows **ETA** and a cycling ellipsis instead of a number, and one whose percentage has not moved for half a minute reads "68% for 4m" beside it. Nothing stands in for an estimate that was not measured.
- A list of every version beside the patch notes: one click goes to any release, it marks the one being read, and on a narrow screen it becomes a row of series above the notes.

### Changed
- Patch notes no longer fold. Every series stands open and the list of versions is how you get about, instead of one group open at a time and a click to reach any other.
- **Settings → Outbound connections** lists every notification destination alongside Jellyfin and the service connections: what it is, its host, whether it is switched on, and when it last took a message.
- Nothing about collection, history or any figure on screen changes. finstats still only ever *reads* from Jellyfin, Sonarr, Radarr and Seerr.

## [1.5.2] - 2026-09-22

### Changed
- A read of Jellyfin's session list and the reset of the clock that asked for it are one call, so the two can no longer come apart — the shape of the runaway-request fault found while 1.5.0 was being built.
- Proving that takes a fraction of a second instead of seven minutes against a real server, so it is checked every time anything changes. The same requests, the same history, the same figures on screen.

## [1.5.1] - 2026-09-22

### Fixed
- A transcode is recorded once, not once a second. When a play starts by transcoding and settles back to direct play, Jellyfin leaves the transcoding details on the session; each reading was compared against a value finstats had just overwritten, so every reading looked like a change and the Activity page filled with "Transcoding" lines reading "Direct play: …" underneath.
- Lines already written are cleaned up on start-up: where a run of them says the same thing, the first is kept. A play that needed transcoding still counts as a transcode.

## [1.5.0] - 2026-09-22

WebSocket session tracking

### Changed
- finstats keeps one connection open to Jellyfin from the moment it starts, and that is how it hears about a play. 1.4.0 shipped it as a setting, off by default, to prove itself against real servers; it is now always on. The same plays, pauses and skips, worked out by the same code.
- Nothing playing: finstats asks Jellyfin nothing at all. An evening when nobody is watching costs no requests.
- Something playing: the session list is read once a second. That is where a pause, a seek or a track change gets its sharpness, and it is what ends a play whose client vanished without saying goodbye.
- Everything loaded paused for three readings in a row: the asking stops and the connection carries it instead, where before a paused film cost the same question every second, around 16 MB an hour. A resume, or somebody else starting something, is answered within about a second, and paused time still never counts as watch time.
- Every outbound read asks for compressed answers and unpacks them (Jellyfin, Sonarr, Radarr and Seerr all compress when asked), which takes more than half off the two largest reads: the session list during a play, and the library read. The geolocation database is still downloaded as it is stored.
- The Jellyfin card in Settings says which of the two halves is running, and the live dot follows whether the connection is carrying rather than which half you are in.
- Only one of the two runs at a time: while finstats is asking it tells Jellyfin to stop sending, and asks it to start again the moment the last play ends. The connection stays open throughout.
- A Jellyfin that answers the subscription with nothing is believed while it answers keep-alives: finstats says so once, reads the session list itself meanwhile, and stays subscribed until the first push settles it. "Cannot push" and "was restarting when we asked" look alike, so silence is never a verdict.
- The fallback is untouched: a connection that closes, goes quiet or turns out not to speak this drops back to asking on a timer on the very next pass and keeps reconnecting, at the **While someone is watching, check every** and **While nothing is playing, check every** intervals.

### Added
- What the collector is doing, on `GET /api/status`: which of the four things it is at (`idle_socket`, `playing_poll`, `paused_socket`, `fallback`), whether the connection is open and subscribed, how often it is asking, how many session reads went out in the last minute, and when that last changed. Answered from memory — no request, no query — and it carries no names, titles or counts.
- Those fields are checked against each other on every pass, and a disagreement that lasts is written to the log.
- One limiter under every session read in the process: at most 2 a second and 70 a minute, with anything above that delayed, counted and logged at most once a minute. Normal watching stays well under it.

### Fixed
- Live updates really replace the polling. Switched on, 1.4.0 still asked for the session list every five seconds and reopened the connection every twenty to forty, because it judged the connection by how recently a list had arrived — and Jellyfin sends one when something *changes* and nothing in between. Liveness is now whether Jellyfin is answering at all, checked with a keep-alive about twice a minute.
- A session list that arrived while finstats was between passes was thrown away, on the assumption that another was a second and a half behind. It is kept and used.
- **Last checked** under Settings, and the Outbound connections card, no longer go stale while the connection is quiet: Jellyfin's answer to a keep-alive is what they show.

### Removed
- The **Let Jellyfin push what is playing** setting under Settings → Collection, and `live_socket` from the settings API.
- `socket_enabled` from the collector status: there is no "switched off" any more.
- `last_reconcile_at` from the collector status, replaced by `socket_live`.

## [1.4.1] - 2026-09-22

### Changed
- Seerr is only read in full when something has changed: every five minutes finstats asks for one row, the most recently changed request, and the pass ends there if it already knows it — about a kilobyte against about sixty for a page of fifty. Requests still on their way are looked at every quarter of an hour, everything is listed once a day, and a new request still appears within five minutes.
- The Sonarr and Radarr queues are read every five seconds while a page is showing them, every minute while something is in the queue and every five minutes while it is empty, instead of every minute around the clock. Opening the page, connecting a service or a new request in Seerr refreshes them at once.

## [1.4.0] - 2026-09-21

Optional WebSocket sessions, and everywhere finstats can reach

### Added
- **Let Jellyfin push what is playing** under Settings → Collection, off until you turn it on: one connection carries the session list, and about 17,000 requests a day become almost none. The plays, pauses, skips and groups are worked out by exactly the code that worked them out before. While it is live finstats still makes one ordinary request a minute *while something is playing*, and none when nothing is. A connection that closes, goes quiet for fifteen seconds or turns out not to speak it falls back to the timer on the next pass and keeps reconnecting.
- **Outbound connections**, a card in Settings for Jellyfin administrators: every destination finstats can reach — your Jellyfin, the "what is my IP" service, DB-IP's database, and each Sonarr, Radarr or Seerr — with what it is for, whether it is switched on, and when it last answered. Built from what finstats already knows, and it shows addresses only: never a key, never a base path.
- **Look up now** under Settings → Home network, for the day your public address changes.

### Changed
- The "what is my IP" lookup runs once, the first time finstats needs an address, instead of every fifteen minutes.
- The Jellyfin card in Settings says how the collector is being told, and, when the connection is switched on but not carrying, why not.

## [1.3.0] - 2026-09-20

Sonarr, Radarr and Seerr, on one page

### Added
- **Connections** under Settings, for Jellyfin administrators: **Sonarr**, **Radarr** and **Seerr** (or Jellyseerr/Overseerr), several of a kind, each tested before it is saved. Your download client needs no setup of its own, because Sonarr and Radarr already talk to it and finstats reads what they know.
- **Requests**: who asked for what, how long it took to arrive, and whether they ever watched it. Tiles for titles waiting and the typical wait, a chart of how that changed month by month, and lists of what arrived weeks ago and was never played, and who asks for the most.
- **Upcoming**: new episodes and film releases from Sonarr and Radarr as an agenda by day, marked with whether *you* watch that show and, for people who may see everyone, who else does — a show counts as watched when somebody played an episode of it in the last four months. A "Coming up" row on the dashboard, one per profile for the shows that person watches, and what is next for a title on its own page.
- **Downloads**: the live queue of every Sonarr and Radarr — what it is, how far along, how fast, what went wrong on import — with the person who asked for it beside it. A season pack is one line however many episodes it holds, and what needs attention is on top. Below it, what came in over a week, a month or a year by indexer, quality and download client, and which downloads failed. The dashboard shows the same list, shortened.
- **See what is downloading**, a new permission. Without it people still see how far their *own* request has got and how long is left, and nothing else: no release names, no speeds, no other downloads. Other people's requests need *See everyone's activity*.

### Changed
- finstats sends `GET` to these services and nothing else: there is no code in it that could approve a request, start a search, or add, pause or remove a download.
- Their API keys are stored in finstats' own database, never sent back to the browser, never written to a log and never part of a backup. No redirect is followed, so a key cannot travel somewhere you did not enter, and certificates are verified unless you switch that off for one connection.

## [1.2.2] - 2026-09-20

### Added
- Audio and subtitle languages on every film and episode ("Audio: Japanese · English").
- How far a dub goes: a show or season says how many of its episodes have each language ("English: 13 of 26 episodes"), and the episode list has an **Audio** column.
- **Audio languages** and **Subtitle languages** on library pages: how many files can be played in each.
- finstats re-reads the library once after this update to pick the languages up. It never says "dubbed", because Jellyfin does not give a title's original language, and a track without a language tag is listed as "Unknown".

## [1.2.1] - 2026-09-20

### Changed
- The pictures in the README are retaken at this version, with a new one of the **Security** page. Nothing in the app changes.

## [1.2.0] - 2026-09-20

Where people watch from, on a map

### Added
- **Security page**, for people who may see network details and everyone's activity: a world map with a dot for every place your users watch from, sized by how much happens there. Home is green, places away are purple, live streams pulse and failed sign-ins from outside are red. Drag to move, zoom with the buttons, the keys or Ctrl + wheel, and overlapping dots merge until you zoom in. Below it, every place with its people, plays, watch time and sign-ins, and the countries by plays.
- **Impossible travel**: an alert when one account is seen in two places no flight connects, or in two distant places at once, with both sightings, the distance, the time between them and the speed that would have taken. **Show on map** draws the trip.
- **New country**: the first time someone plays or signs in from a country they have not been seen in.
- Resolve an alert with a note, or mute a pair of places for that person, for a VPN or a phone whose carrier sits in the capital. Alerts found in old history, after an import or on a fresh database, are filed as resolved rather than flooding the list. Alerts and what you decided about them are part of backups.
- Places come from a city database read locally: no address is ever sent anywhere, and the map is drawn from outlines bundled with finstats rather than a map service. Download DB-IP's free database from the Security page or **Settings → Security**, let finstats refresh it monthly (off until you switch it on), or drop your own `.mmdb` (DB-IP, MaxMind GeoLite2-City) into `data/geoip/`. `FINSTATS_GEOIP_DB` names a file elsewhere.
- **Settings → Security**: the database in use, the monthly update, and how fast (900 km/h) and how far apart (500 km) two sightings must be to count as impossible travel. City databases are often a few hundred kilometres off, which is what the distance is for.

## [1.1.3] - 2026-09-20

### Added
- Ctrl+Space also searches the cast and crew of everything in your library, under **Cast and crew**, with their photo, whether they act or direct, and how many of your titles they are in; choosing one opens their page. As forgiving as the title search — words in any order, accents ignored, a slip of the finger allowed — and when several match equally well, the one in more of your titles comes first.

## [1.1.2] - 2026-09-20

### Added
- **Recently added** on the dashboard, above the Activity chart: the 30 newest arrivals as a row of posters you can scroll sideways, with arrows on a computer. Each says when it arrived and what it is — a film, an album, or "Season 4 · 3 episodes" — and a single new episode says which one, with the season's own poster when it has one.
- **Shift + mouse wheel** and the arrow keys move one poster, Page Up and Page Down a screenful, Home and End go to either end, and swiping works. The plain wheel still scrolls the page.
- New episodes of the same show that arrive on the same day share one entry, and a whole show added at once is one entry ("3 seasons · 60 episodes"), so one big import does not push everything else off the list.
- The row ignores the dashboard's time range and person, since it is about the library and not about plays, and hides itself while the library is empty.

## [1.1.1] - 2026-09-20

### Added
- Pages open at once: a moment after finstats has loaded it quietly fetches what you are likely to open next — the dashboard, your profile and timeline, Activity, Users, Libraries and each library, Playback, the server pages, the recap and the most active people — so opening one shows it immediately and it still refreshes behind the scenes.
- Films, shows and cast members are fetched on intent only, as there are thousands of them: resting the pointer on a link, touching it or reaching it with the keyboard fetches that one page.
- It waits until the page you opened has finished loading, works one page at a time while the browser is idle, pauses in a background tab, does nothing on a data-saver or very slow connection, and is forgotten when you sign out.
- A small **Repo** link with the GitHub mark in the status bar, next to the version.

### Fixed
- The **Timeline** forgot which libraries were switched off when the page was reloaded or opened from a bookmark, although the choice was in the address.

## [1.1.0] - 2026-09-20

A timeline of everything you have watched

### Added
- **Timeline**, next to **Overview** on every profile: your watching as one trail from today back to the first play finstats knows about, with an evening of episodes folded into a single stop. Each stop is a poster with what was watched and when — "Season 2 · Episodes 3-6", a film and whether it took more than one sitting, or an album and how many tracks — and opens the title.
- The trail follows the width of the window: three stops to a row on a wide screen, two on a narrower one, every other row running backwards with a bend joining it to the next, down to a single straight line on a phone. The first stop of each month carries the month.
- Older stops load by themselves as you scroll, all the way back to where the history starts.
- **Libraries** switches above the trail leave out what you do not want to see, such as music, and the choice is part of the address.
- Something that is still playing says **Playing now**.
- You see your own timeline; other people's need the "see everyone" permission.

## [1.0.5] - 2026-09-20

### Added
- Downgrade protection: the database remembers the newest finstats version that has opened it, and an older version refuses to start on it instead of quietly working on data it does not fully understand. The message says what to do. Versions up to 1.0.4 were released before this check existed.

## [1.0.4] - 2026-09-20

### Fixed
- In a play's details, the copy button next to a long **Device ID** had dropped onto a line of its own.
- On the Playback page the last column of **Which clients transcode** was cut off ("Transc…"); two-word headers wrap now.

### Changed
- The screenshots in the README are current again, having still shown version 0.5.0.

## [1.0.3] - 2026-09-20

### Added
- An episode page shows that episode's own picture, the one Jellyfin shows in its episode list, instead of the show's poster. An episode Jellyfin has no picture for keeps the poster.

## [1.0.2] - 2026-09-20

### Fixed
- In **Patch notes**, the v0.9 and v0.10 groups had no headline next to their version. A group's headline is now always the first sentence of what its first release was about, so a long introduction is not cut off mid-sentence.

## [1.0.1] - 2026-09-20

### Fixed
- The published image would not start on a fresh machine: "unable to open database file: /data/finstats.db", after hanging for half a minute. Docker creates a missing `data` folder as root and finstats runs as an ordinary user. The container now makes the folder its own and then drops to that user, so `docker run` works on the first try. For files owned by someone other than user 1000, set `PUID` and `PGID`; `--user` works as before.
- A data folder that really cannot be written is reported at once, with the command that fixes it, instead of 30 seconds later as a database error.

## [1.0.0] - 2026-09-20

Ready for everyone

### Added
- A ready-made image, now at `ghcr.io/finstats/finstats`, for 64-bit Intel/AMD and ARM machines (a Raspberry Pi 4 or 5 works): `:latest` is the newest release, a version tag stays where it is, and `:edge` is the development version.
- Every release appears on the project's GitHub releases page with these patch notes.

### Changed
- Version numbers now mean what they say: 1.x updates will not break your data, your backups or your settings.
- The Docker Compose file and all instructions use the published image. A self-built `finstats:latest` only needs the image name changed; the `data` folder carries over untouched.

## [0.10.2] - 2026-09-20

### Fixed
- On a phone, Settings and the page of a show could scroll sideways: a label meant only for screen readers escaped its table, and the episode lists and the cast row did the same. They scroll inside themselves now.
- The recap no longer scrolls sideways when a chapter's background word is wider than the page.
- A long "watching on…" line under **Now playing** ends in an ellipsis instead of being cut mid-letter.
- The poster in front of each show on a profile was a second, nameless link for screen readers; it is hidden from them and the title next to it is the link.

## [0.10.1] - 2026-09-20

### Fixed
- Downloading a backup failed in the browser ("the source file could not be read" in Firefox) with nothing on the page to say why: finstats was compressing a backup, which is already compressed, a second time, and the doubly-packed transfer broke off before the end. Backups are sent as they are, with their size up front, so the browser can show real progress.

## [0.10.0] - 2026-09-20

Backups, and moving to a new install

### Added
- A backup every week, newest five kept: every play with its pause-and-skip timeline, seen marks, permissions, home addresses, the server log and your settings, in one small file — a few hundred KB for thousands of plays. **Settings → Backups** lists them with a **Download** button each, makes one on demand, and sets how often and how many. They live in the `backups` folder of your data directory. A backup holds no Jellyfin API key and no sign-in session, but it is a complete viewing history with IP addresses, so keep it somewhere private.
- Restoring merges rather than overwrites: plays already there are skipped, so it is safe to do twice or into an instance that has been running for a while. Untick "Also restore settings and permissions" for the history only. Backups work across versions in both directions. From a terminal: `finstats backup` and `finstats restore <file>`.

### Changed
- Collection checks every second while someone is watching and every 5 seconds while nobody is, where it used to be every 5 seconds throughout, so pauses, skips and track changes are recorded to the second. Both are under **Settings → Collection**; a changed old interval needs setting again.

### Fixed
- An IP address in a play's details broke across two lines in the middle of the address. The Local/Remote mark moves underneath when there is no room.

## [0.9.1] - 2026-09-20

### Fixed
- People at home were shown as remote. A device on your own network that reaches Jellyfin through its public name, such as a reverse proxy or a domain, arrives with the household's public IP, and only private addresses counted as local. finstats now learns the public address and counts those plays as local for the whole history, and it remembers earlier addresses.

### Added
- **Settings → Home network**: the addresses finstats treats as home, adding your own (an earlier address, a second home, a VPN exit), or switching the lookup off. The lookup asks a plain "what is my IP" service every 15 minutes, trying several (Amazon, Cloudflare and others) because ad-blocking DNS such as Pi-hole often blocks them, and one of them needs no DNS at all. The request contains nothing about you or your server, and with the switch off finstats talks to nothing but Jellyfin. `FINSTATS_PUBLIC_IP_URL` sets a service of your own.

## [0.9.0] - 2026-09-20

Every table, in the order you want

### Added
- Every table sorts: click a header to sort by it, again to turn it around, a third time for the original order. Cell meaning is understood, so "3d 2h" sorts as time and "1.4 GB" as size, and empty cells go last. It covers the user list, watchers, episodes, devices, plugins, scheduled tasks, the table view of every chart, the permission matrix, and the bar lists (codecs, resolutions, clients and the rest), which get a slim header of their own.
- Activity and the Server log sort across all their pages rather than the rows on screen, and the order is kept in the address, so it survives a reload or a shared link.
- Tables of ten rows or more get a small filter field above them.

## [0.8.2] - 2026-09-20

### Changed
- The recap's year picker is the same switch as every other range picker in finstats (7d · 30d · 90d …).

## [0.8.1] - 2026-09-19

### Added
- Pages for actors and directors, from the recap's "Most watched people" or the new **Cast & crew** row on any film, show or episode: everything they are in on your server, how much has been watched and by whom, and what is still waiting. It follows the time range you pick, people who may only see their own statistics see only their own watching there, and Esc takes you back.

## [0.8.0] - 2026-09-19

The recap has a new look, and more to say

### Added
- **Most watched people**: the actors you spent the most time with and the directors behind what you watched, as a row of portraits. Cast and crew are read along with the library and appear after the first library read following this update.
- **Your year in days**: every day of the year as one square, brighter the more you watched, with your longest streak and biggest day above it. Hover a day, or walk the calendar with the arrow keys, to see what it held.
- **By the numbers**: plays, watch time, different titles, days watched, longest streak and how much was a **rewatch**, plus how the year splits between episodes, movies and music.
- The genre chapter says how many genres you touched and how large a share the top one took.

### Changed
- A new design: it opens on your year "replayed", the headline beside the posters that filled it, over a waveform of the year with one bar per week and the loudest lit, and you pick the year right above it. Every chapter has a headline with its key word in violet, a sentence that carries the figures, and the chapter's word standing large and faint behind it. It ends the way films do, with the credits: starring, directed by, screened on, running time.
- Hours, weekdays and months share one **Activity patterns** chapter with a switch between them, and the headline follows what you are looking at ("Friday took the crown").

### Fixed
- **Now playing** on the dashboard was left blank when nobody was watching, so the heading seemed to belong to the filters under it. Its "Nothing is playing right now" box is back.

## [0.7.10] - 2026-09-19

### Added
- Library artwork: each library card, and the library's own page, shows the picture Jellyfin uses for it, through finstats' image proxy like the posters. A library without one keeps its icon.

## [0.7.9] - 2026-09-19

### Changed
- The Genres card opens as the **list** again, with the radar as the second option on the switch. An existing choice is kept.

## [0.7.8] - 2026-09-19

### Added
- A radar view for the Genres card, on the dashboard and on profiles: one spoke per genre, switched between **Radar** and **List** in the card's corner, and the choice is remembered. Hovering a point, or using the arrow keys, shows that genre's watch time and share.

## [0.7.7] - 2026-09-19

### Fixed
- Movies that belong to a collection were shown as "No longer in library". With *group movies into collections* switched on, Jellyfin hands out the collection instead of the films inside it, so finstats never saw them — on one server that hid 556 of 1,238 items in the Movies library. finstats now asks for the films themselves, and the read this update triggers brings them back and re-attaches their plays.

## [0.7.6] - 2026-09-19

### Changed
- On profiles the **Shows** card moved down to just above Genres, so a long list of shows no longer pushes the watch time, activity chart and top titles out of sight. The day streaks stay at the top, and a show you have opened stays open when you change the time range.

## [0.7.5] - 2026-09-19

### Changed
- Search matches word by word, in any order, with anything in between, instead of needing the exact phrase, so "alya hides" finds *Alya Sometimes Hides Her Feelings in Russian*. Case, accents and punctuation do not matter, a leading "The" is ignored, and a slip of the finger is forgiven ("mentalsit" finds *The Mentalist*); words of three letters or fewer still have to be typed right. Better matches rank first, and music is also found by its artist.
- The search boxes on Activity and the Server log match word by word across all their columns: "alya opera" finds plays of that show in Opera.

## [0.7.4] - 2026-09-19

### Changed
- Patch notes are grouped by series: every `0.x` series is one fold holding its releases, with what the series was about, how many there are and when. Only the newest starts open, and a closed fold still shows which one you are running.

## [0.7.3] - 2026-09-19

### Fixed
- The Now playing clock hesitated every few seconds: the 5-second refresh was also repainting it, so whenever the server was a fraction ahead the number moved early and then stood still for a second. Only the once-a-second tick moves it now, by exactly one second (measured 999 to 1001 ms apart), and the refresh still corrects after a pause, a skip or real drift, landing on the tick.

## [0.7.2] - 2026-09-19

### Added
- Group watching, live: each Now playing card says "With maria" while people are watching the same thing together, where a group used to be recognised only once the plays had ended. Live, people count as together when they are on the same title and either started within the group window or are at nearly the same position, which still works if finstats was restarted mid-stream.

## [0.7.1] - 2026-09-19

### Changed
- Now playing counts every second: the clock ticks and the bar glides in between, instead of jumping each time the server is asked. The server is still asked every 5 seconds and stays the source of truth — the display re-syncs on a pause, a skip or when the server is ahead, but not for a position merely a few seconds stale, since clients report only every ten seconds or so. Paused streams stand still.

## [0.7.0] - 2026-09-19

Who watches together

### Added
- Group watching: when different people start the same title at the same time, finstats counts it as watching together. A **Watched together** card on the dashboard shows the groups, the titles they share and their hours together, each profile shows who that person watches with most, shared plays carry a small people mark in Activity, and the play details say who it was watched with. It works on your whole history, imported plays included.
- A setting for how close together the starts must be, 60 seconds by default: about a third of genuine group sessions start 6 to 60 seconds apart. People also have to keep watching alongside each other for a couple of minutes, so two people opening the same episode by coincidence is not a group.

### Changed
- Search opens with **Ctrl + Space** instead of Ctrl/⌘ + K.

## [0.6.1] - 2026-09-19

### Added
- Administrators can open another person's recap, at the top of the Recap page or with **Open recap** on their profile. It then reads about them by name instead of "you".

### Changed
- Everyone else still sees only their own recap, whatever permissions they hold, and there is still no recap of the whole server.

## [0.6.0] - 2026-09-19

Your profile, and where you are in every show

### Added
- Show progress on profiles: every series you have touched as a bar with one segment per episode — seen, started or not yet — openable season by season, sorted by how close you are to finishing, with separate views for finished shows and everything.
- **Mark as seen** on your own profile, per episode, season or show, for what you watched while nothing was recording. Jellyfin's own played marks are used as well. Marks stay in finstats and never change anything in Jellyfin; a recorded play cannot be unmarked, your own marks can.
- Day streaks on profiles: your longest run of days in a row with a play, your current streak, and how many days you have watched something.
- **My profile** in the sidebar.
- Activity shows where playback stopped (for example `44:15 / 45:00`) under the progress bar, and the date and time under "3h ago". Plays imported from Jellystat have no stop position, because Jellystat never recorded one.

### Changed
- Episodes that are missing or have not aired yet are no longer treated as part of your library. Jellyfin lists them as placeholders without a file, and they used to count towards episode totals.

## [0.5.1] - 2026-09-19

### Added
- A logo: a play button sliced into three chart bars, in the sidebar and on the sign-in screens, as a sharp SVG with a multi-size `favicon.ico` for browsers and bookmark bars that want one, plus a home-screen icon for phones and tablets.

## [0.5.0] - 2026-09-19

You decide who sees what

### Added
- Permissions under Settings → Access, for everyone at once or person by person: **see everyone's activity** (other people's statistics and history, the Users page, every live stream), **see network details** (IP addresses, device ids, local or remote), **see the server** (the Server page, the server log, failed sign-ins, file paths) and **manage finstats** (settings, tasks, import, deleting plays).
- Sign-in per person, to let a few people in without opening finstats to every Jellyfin user.
- Changes apply immediately, including taking access away, and everything is enforced by the server rather than by hiding buttons.

### Changed
- Jellyfin administrators still have everything and are the only ones who can change permissions; someone who may manage finstats cannot grant anything, to themselves or anyone else.
- The year recap stays personal whatever is granted.
- Nothing changes for existing installs until you grant something.

## [0.4.5] - 2026-09-19

### Changed
- finstats is licensed under the **GNU General Public License v3.0**. No earlier version was ever published, so nothing was released under different terms. The licenses of the bundled fonts ship alongside them.

## [0.4.4] - 2026-09-19

### Added
- Esc steps back: on an opened library, user or title it returns you to where you came from, with the list exactly as you left it. Anything open closes first (play details, search, a dropdown, the mobile menu), and Esc is left alone while you are typing or using a chart with the keyboard.

## [0.4.3] - 2026-09-19

### Changed
- finstats remembers what each page last showed: going back to a profile, a library or a time range you have already looked at paints it immediately, profile picture and all, while fresh numbers load behind it, and the page only redraws if something changed. What is remembered lives in the browser tab and is dropped when you sign out or change anything.
- Loading placeholders appear only when loading is actually slow. Before, every page showed them for at least a third of a second.

### Fixed
- Libraries that were deleted in Jellyfin and never had a play are no longer listed. A deleted library with history is still shown, marked as removed.

## [0.4.2] - 2026-09-19

### Fixed
- On the Users page the **Admin** tag sat on its own line and made that row taller than the others. Tags sit beside the name now.

## [0.4.1] - 2026-09-19

### Fixed
- The word "null" no longer appears under the dashboard title outside December and January, when there is no recap banner to show.

## [0.4.0] - 2026-09-19

See what changed without leaving the app

### Added
- **Patch notes** tab showing every release, with the version you are running marked.
- The version in the status bar links to it, and the tab shows a dot after an update until you have looked.

### Changed
- Re-attaching history to renamed items is far faster: 18 seconds to a third of a second with 666 orphaned titles, so start-up stays instant as history grows.

## [0.3.3] - 2026-09-19

### Changed
- The library is re-read right after Jellyfin's **Scan Media Library** task finishes, never on a separate timer and never while a scan is running. finstats has always been read-only towards Jellyfin, and it never starts a scan there.
- A weekly safety-net read covers servers that rely on real-time monitoring, where the scan task may never run.
- The old interval is now only a fallback, for when following is switched off or the server reports no scan task.

### Added
- **Follow Jellyfin's library scan** switch under Settings → Collection, on by default.

## [0.3.2] - 2026-09-19

### Fixed
- History survives renames. Jellyfin gives a renamed file a new id, which used to strand its earlier plays under the raw folder name (`Title (2010) [tmdbid-…] [imdbid-…]`) with no poster. Those plays are re-attached to the current library entry: by TMDB/IMDb/TVDB id when the old name carries one, otherwise by cleaned title and year, and episodes by series plus season and episode number. A match has to be unambiguous, so titles that were really deleted stay as they are.
- Live TV channels imported from Jellystat were counted as movies, because Jellystat records no item type. They are recognised as Live TV, and new Live TV plays are recorded with Jellyfin's own type.

### Changed
- Live TV is left out of the recap entirely: top lists, totals, persona, records and rank.

### Added
- `finstats relink` to re-attach history on demand. It also runs at start-up and after every library read.

## [0.3.1] - 2026-09-19

### Changed
- The recap is strictly personal: everyone, administrators included, sees only their own, and the "Everyone" view, the user switcher and the server chapter are gone.
- The recap opens on the year that is "ready": the current year during December, otherwise the previous one. Other years and "Last 12 months" remain one click away.

## [0.3.0] - 2026-09-19

Your year in review

### Added
- **Recap** tab, a scrolling story of a year: hours watched, top shows, movies, tracks and genres, and a viewing persona (night owl, early bird, weekend warrior, binge watcher, movie buff, music lover, creature of habit) with hour-of-day and weekday charts.
- The year month by month, with the title that defined each month.
- Records: biggest day, biggest binge, longest daily streak, longest single play, most rewatched title, first play of the year and the oldest title watched.
- Discovery: new shows started, movies and episodes finished, and the shows that got exactly one episode.
- A one-line banner on the dashboard in December and January when a recap is ready.

## [0.2.0] - 2026-09-19

More than Jellystat ever recorded

### Added
- A timeline for every play: start, pause, resume, skip, audio and subtitle switches, direct play turning into a transcode, and stop. Plus pause and skip counts, where playback resumed from, and whether the viewer was on the local network.
- **Server** tab: Jellyfin version, pending updates and restarts, disk usage per library (Jellyfin 10.11+), plugins, scheduled task results and every registered device.
- **Playback** insights: concurrent streams over time, which clients force transcodes, how far people get before stopping, local versus remote plays and viewing behaviour.
- What your library is made of: resolutions, codecs, dynamic range, containers, titles per decade, additions per month, the largest titles, and the titles nobody has ever watched, checked against both finstats' history and Jellyfin's own played flags.
- Dashboard: peak concurrent streams, estimated data streamed, share of remote plays, genres by watch time and failed sign-ins.
- Item pages: IMDb and TMDB links, studios, bit depth and frame rate, and who has the title marked as played in Jellyfin.
- User pages: genres, and the movies, episodes and favourites Jellyfin has on record.

### Fixed
- The time label on a now-playing card no longer spills outside the card.
- Long labels on the Playback page were cut off too early.
- A series that has left the library still shows its season and episode list.

## [0.1.0] - 2026-09-19

First release

### Added
- Live collection from Jellyfin's sessions: user, title, client, device, IP address, play method, transcode reasons and hardware acceleration, video, audio and subtitle details, and time watched versus time paused.
- Sign-in with your Jellyfin account. A two-step setup creates finstats' own API key, and passwords are never stored. Non-admin users can optionally sign in to see only their own statistics.
- Dashboard, Activity, Users, Libraries, Playback and Server log pages, a command palette (Ctrl/⌘ K) and a live status bar.
- Jellystat import: large backups stream to disk and import in seconds, and importing the same file twice is safe.
- A single small binary with an embedded web UI and one SQLite file, plus a Docker image.
