# How Tautulli data is imported

Tautulli keeps what Plex played. FinStats reads its database backup directly and brings that history
over to Jellyfin. The backup is the `.db` file, or the `.zip` holding one, that **Backup Database**
(Settings → Import & Backups) saves in the `backups` folder of Tautulli's data folder
(`/config/backups` in Docker). The file is saved where Tautulli runs, not downloaded, so copy it to the computer you
upload from.
The whole import is one transaction: it either fully succeeds or changes nothing.

## You say who is who

Nothing in a Plex history shares an id with Jellyfin: not the people and not the titles. So the
upload does not import anything yet. It turns the Tautulli card into a **wiring board**: every Plex
user on the left, every Jellyfin user on the right. Connect each Plex user to who they are on
Jellyfin by dragging a wire from one to the other, or by clicking one and then the other. With the
keyboard, press Enter on a Plex user and use the arrow keys to choose.

- A Plex user **without a wire is not imported**. Somebody who used Plex and never Jellyfin simply
  stays behind.
- **Several Plex users can go into one Jellyfin user**, for somebody who had two Plex accounts. One
  Plex user goes to one Jellyfin user.
- Pull a wire out into empty space (or press Delete on its plug) to unplug it. **Start over** puts
  the board away and removes the upload.

## What the fields mean

Learned from a real backup rather than from documentation:

| Tautulli | How FinStats reads it |
|---|---|
| `session_history` + `_metadata` + `_media_info` | One row each per stretch of playing, joined by `id`. |
| `reference_id` | A resumed play continues the row it resumed, and every row of it carries the first one's id here. **One viewing is one play**: from its first start to its last stop, the time it ran without the time it was paused. |
| A track, then an episode, under one `reference_id` | Plex plays a show's theme while the show is open, and Tautulli chains the episode onto it. A viewing is one reference *and one title*, so the episode comes in on its own. |
| `paused_counter` | Seconds paused. Not counted as watched. |
| `view_offset` | Where playback was when the row ended (milliseconds). Taken as where the play stopped, beside the runtime (`duration`). |
| `transcode_decision` | `direct play`, `copy` (a direct stream) or `transcode`. A transcode that copied both picture and sound is a direct stream, the same rule as for a play FinStats watches itself. |
| `guid` | `plex://…` or `local://…`: Plex's own ids, with no TMDB, TVDB or IMDb id beside them. |
| An empty string where a number goes | Tautulli writes one where it has no number (a film's episode number, a missing width), and writes some numbers as text. Both are read for what they say. |

## How titles are found

With no provider ids to go by, a film is matched to your library by its **title and year**, and an
episode by its **show and its season and episode number**, by the same rule FinStats uses to follow a
title that was renamed. A match must be unambiguous, and it is forgiving where catalogues differ:

- A name matches Jellyfin's title **or its original-language title**, so a show Plex knew as
  오징어 게임 is found as Squid Game, and a film Plex knew as *Im Westen nichts Neues* as *All Quiet on
  the Western Front*.
- Case, accents, punctuation (a hyphen for a dash), a year written into the name, as in "JoJo's Bizarre
  Adventure (2012)", and a leading "The" do not matter.
- The year may be one off, as catalogues often disagree by a year about when a film came out.

A title known by another name altogether (*Fast & Furious 7* where Jellyfin has *Furious 7*) is
not guessed at. Those are listed under **Settings → Unlinked media**, each with where it most
likely is: press **Locate**, pick the title, and its plays attach. FinStats remembers the choice, so importing
the same backup again attaches it by itself. A play of something not on your server is kept
under its name, and attached the moment the title arrives.

**Music, clips, photos and Live TV are not imported.** A track's name alone does not say which
track it is.

## The backup holds Plex's keys

Tautulli's `users` table carries every Plex user's access tokens and e-mail address. FinStats never
reads those columns: the board shows names and play counts only. The uploaded file is removed once
you import or start over, at the next start, and after six hours of waiting for its wires.

## What imported history cannot have

Tautulli stores one row per stretch of playing, so imported plays have no timeline of pauses, skips
and track switches, no pause or skip counts, and no resume point. Everything FinStats records
itself does.

## Importing alongside the others

Nothing is counted twice: a play is recognised by Tautulli's own id for it, so importing the same
backup again is safe, and a viewing already here from another tracker is recognised by the same
person, the same title and either end of the play.

There is no command-line import for Tautulli, because somebody has to say who is who.
