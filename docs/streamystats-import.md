# How Streamystats data is imported

finstats reads Streamystats' own backup directly: the one file its
**Settings → Backup & Import → Backup & Restore → Download Backup** gives you. The file is walked
rather than loaded, so a backup of years of history costs no more memory than a small one, and the
whole import is one transaction: it either fully succeeds or changes nothing.

A Streamystats backup is **sessions only**. It holds no libraries, items or users, so an import
brings history and the names of the people in it, and nothing else. Titles, posters and runtimes
come from your own Jellyfin, as they do for everything else finstats shows.

## What the fields mean

Learned from a real export rather than from documentation.

| Streamystats field | How finstats reads it |
|---|---|
| `startTime` **and** `endTime` | Two different moments means a play Streamystats watched itself: the first is the real start, and whatever the pair leaves over `playDuration` is time the play was not running. |
| `startTime` **==** `endTime` | One moment, and it is the **end**: this is a play Streamystats imported from Jellystat, whose `ActivityDateInserted` it copied into both fields. Start = end − `playDuration`. |
| `playDuration` | Seconds the play was actually running, so it is what finstats counts as watched. How much of the rest was a pause is not recorded, and is not guessed. |
| `isInferred` / an `inferred:` id | **Not a play.** Jellyfin reported the item watched, so Streamystats wrote a row as long as the whole runtime for a viewing nobody saw. Counted and skipped: importing it would add watch time to an evening nobody watched. |
| `itemId` | The item, in both kinds of row (Streamystats re-links renamed items). `mediaSourceId` is *not* an item id and is not used. |
| `positionTicks` | Taken only where the row also keeps the `runtimeTicks` it is a position in. A row from Jellystat has no runtime, and its position is the one Jellystat happened to catch rather than where playback stopped. |
| `videoCodec`, `resolution*`, `audioCodec`, `videoRangeType` | Empty in every play Streamystats watched itself. Nothing is invented in their place: those plays simply have no file details. A play from Jellystat carries the whole session in `rawData`, and those are read from it. |
| `transcoding*`, `transcodeReasons` | Read for a play Streamystats watched. **Not** read for one from Jellystat: there they are a copy of the source codec and container with a placeholder reason, which stored as a transcode would read as a file transcoded into itself for reasons unknown. Those rows use the session's own `TranscodingInfo`, or have no transcode at all. |
| `isActive` | Ignored. It is `true` on nearly every row of a backup; nothing in a backup is playing now. |
| *(no item type exists)* | Episodes are recognised by `seriesId`; everything else by your library. An item neither can type is left **unknown** rather than guessed, and a later library read fills it in. |

## Importing both trackers

If you ran Jellystat and Streamystats together, import both files. Nothing is counted twice: a play
is recognised by the tracker's own id for it and, failing that, by the same person watching the same
item with either end of the play close to one already in the history. Either end, because the two
trackers disagree about when a play *began* (Jellystat keeps only the end, so the start has to be
worked back from the seconds played, and every minute the viewer spent paused moves it later), while
they agree about when it ended. "Close" is the **Merge window** under Settings → Collection.

The order does not matter, and neither does what finstats has already recorded itself.

## What imported history cannot have

Both trackers store one row per play, so imported plays have no timeline of pauses, skips and track
switches, no pause or skip counts, and no resume point. Everything finstats records itself does.

Plays Streamystats watched itself carry nothing about the file either: no codec, resolution,
bitrate, container or track language. Cards with nothing to show hide themselves rather than showing
a blank.

## Without the browser

```sh
docker run --rm -v "$PWD/data:/data" -v /path/to/backup.json:/backup.json:ro \
  ghcr.io/finstats/finstats:latest import-streamystats /backup.json
```
