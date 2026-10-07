# Security model

**Signing in.** finstats has no accounts of its own. A sign-in is checked against your Jellyfin
server every time, and the Jellyfin session that check creates is ended immediately. Passwords are
never stored or logged. finstats then issues its own session: a random 256-bit token, stored only
as a hash, sent as an `HttpOnly; SameSite=Lax` cookie (and `Secure` when a proxy reports HTTPS).
A session follows the person as Jellyfin has them now, not as they were when they signed in: once
finstats has read Jellyfin's users (every 15 minutes, or at once from Settings), an administrator
Jellyfin demoted is no longer one here, and somebody it disabled or deleted is signed out. One person
keeps at most 30 sessions; the oldest is signed out at the next sign-in. Attempts are rate limited to
10 per 5 minutes per address.

**API keys.** Anyone signed in may make keys for themselves under Settings → API keys, for a script,
a dashboard or a phone's calendar. A key is a second credential for the same person and nothing
more: it is resolved at the one place a session is, reads that person's name, administrator flag and
permissions live on every request, and so can never open more than its maker could at that moment:
lose the right to sign in and every key you hold stops with it. Keys are shown once and stored
hashed; they can be given an expiry, are revoked with one click (an administrator can revoke
anyone's), and a request made with a key can neither make nor revoke keys, so a leaked key has no
successors. A key travels in the `Authorization` header, with one exception, next.

**Public profiles (the one read without an account).** Everything else finstats answers needs a
session or a key. The exception is `/u/<link>` and the API behind it, and it is closed until a Jellyfin
administrator switches public profiles on (Settings → Public profile) and a person then publishes theirs.
A published page is built by its own code, which starts from nothing and adds only the sections its owner
switched on (never from the signed-in statistics with parts taken away), and it can never carry a device,
a client or app, a play method, an IP address, a place, a file path, another person, the Jellyfin login
name or when somebody was last seen. Every figure on it counts only plays that ended at least a day
before, so watching a page tells a stranger nothing about who is at home now. Posters are served only for
the titles the page shows. The link is 22 random characters; resetting it, unpublishing, an administrator
taking it down, the owner losing the right to sign in and the server switch each close it, and all of them
answer the same "not found" as a link that never existed. Publishing needs a session (a key cannot make
anything public), and every publish, change, reset and take-down is in the audit log. Pages ask search
engines not to index them; a chat app that already fetched a card may keep it for up to an hour.

**Who sees what.** Jellyfin administrators see everything. Other users can only sign in when you
allow it, for everyone or person by person, and then start with their own statistics only: no
other people's activity, no IP addresses, no device ids, no file paths, no server log, no settings.
An administrator can grant more under Settings → Access:

| Permission | Opens up |
|---|---|
| Sign in | Using finstats at all, when sign-in is not open to everyone. |
| See everyone's activity | Other people's statistics and history, the Users page, every live stream. |
| See network details | IP addresses, device ids, local versus remote. |
| See the server | The Server page, the server log, failed sign-ins, file paths. |
| Manage finstats | Settings, tasks, the Jellystat import, deleting plays. |

Grants for everyone and grants for one person add up; there are no "deny" rules to reason about.
All of it is enforced by the server on every request, not by hiding things in the interface, and
is read fresh each time, so taking a permission away works immediately, including locking
someone out. Only a Jellyfin administrator can change permissions: someone who may *manage*
finstats still cannot open sign-in to everyone, change the defaults or grant anything, so nobody
can promote themselves. The year recap is personal: you get your own. A Jellyfin administrator can open another
person's, and the whole server's year, which ranks and names nobody; no permission grants either to anyone
else. The year's cards (2.0) are drawn from a copy of the year that has no field for another person, a rank
or an app, so a card that leaves finstats (downloaded, or shown on a published profile) can never name the
people somebody watched with; in the app a card does not even carry its owner's login name.

**Talking to Jellyfin.** During setup finstats creates its own API key, named `finstats`, which
you can revoke at any time under Dashboard → API Keys. It only reads: it never modifies your
server and never starts a library scan. The key is stored in `data/finstats.db`, so protect the
data folder like you would protect Jellyfin's own.

**Setup window.** Until setup is completed, anyone who can reach the port can open the wizard,
but finishing it requires a Jellyfin administrator's credentials.

**The web interface** loads nothing from third parties: fonts and scripts are bundled, posters are
proxied from your own Jellyfin, and a strict Content-Security-Policy is sent with every response.
Requests that change anything are refused when their `Origin` does not match. A request without an
`Origin` carries either a cookie a browser attaches only to a navigation, or an API key no browser
adds on its own.

**The calendar feed** (`/api/calendar.ics`) is the one address finstats accepts a credential in:
a subscribed calendar can send no header. So the feed reads `?key=`, never the cookie (a link can
not open it in a browser that happens to be signed in), and the key meant for it has the `calendar`
scope, which opens the feed and refuses everything else. The feed itself names nobody: the question
it asks the database carries no user name or count. Treat the address like the key it holds; revoke
the key and the address is dead.

**The container** runs finstats as an unprivileged user (1000:1000, or `PUID`:`PGID`). It starts as root for one
step only: making the data directory belong to that user, because Docker creates a missing bind-mount folder as root.
It then drops privileges with `su-exec` and cannot get them back; finstats itself never runs as root. Start the
container with `--user` and even that step is skipped.

**Backups** (`data/backups`, and whatever you download from **Settings → Backups**) hold the full viewing history
with IP addresses, the permissions, the settings and the audit log. They never contain the Jellyfin address or API
key, nor any sign-in session or API key, so a leaked backup exposes history but grants no access. Only Jellyfin administrators can list,
download, delete or restore them, and a backup's file name is checked against the exact pattern finstats generates
before it touches the disk. Restoring validates the settings it brings back the same way the settings page does.

**The audit log** (Server → Audit, Jellyfin administrators only) is finstats' record of itself: every sign-in and
failed attempt with the address it came from, every setting or permission changed and what it changed to, every
key made, first used or revoked, every connection or destination added, changed or removed (its kind and name,
never its address or secret), every backup made, downloaded, deleted or restored, every import and its result,
every play deleted and every alert resolved, with who did it, from where, through which key if any, and whether it
worked. Reads leave no trace. A row is kept a year, is written even when the action failed, and its absence can
never stop an action. It is part of backups.

**No telemetry, and two outside requests: one you can switch off, one that is off until you switch it on.** finstats talks to your Jellyfin server and, by
default, to a public "what is my IP" service (`checkip.amazonaws.com`, falling back to Cloudflare's `cdn-cgi/trace`, by name and by `1.1.1.1`,
then `api.ipify.org` and `icanhazip.com`; several because ad-blocking DNS often blocks such services) **once**, the first time it needs to
know, and after that only when you press *Look up now*. It needs the answer to tell plays from your own household's public address apart
from remote ones. The request is a bare `GET` with `User-Agent: finstats` and `Accept: text/plain`: no version, no
identifiers, nothing about your server or users. What the service necessarily learns is that *something* at your
address asked. Turn off **Settings → Home network → Recognise my own public address** and finstats makes no
connections other than to Jellyfin; `FINSTATS_PUBLIC_IP_URL` points the lookup at a service of your own instead.

The second is the geolocation database behind the **Security** page. Looking an address up never leaves the machine: finstats
reads a city database file (`.mmdb`) in its data folder. Getting that file is the only part that can touch the network, and it
is off by default. With a trigger on the **Geolocation database** task (Settings → Tasks), or the *Download* button, finstats
fetches DB-IP's free "IP to City Lite" file from `download.db-ip.com`, on each trigger only when a newer month is out: a plain `GET` of a public file with
`User-Agent: finstats`, carrying no address of yours, no version and no identifiers. What DB-IP necessarily learns is that
something at your address downloaded its public file. Leave it off and put a file into `<data>/geoip/` yourself (DB-IP's, MaxMind's
GeoLite2-City, or any other in that format; `FINSTATS_GEOIP_DB` names one elsewhere) and finstats asks nobody. (Before 2.0.4 this
was the switch *Keep the database up to date*; an install that had it on keeps a daily trigger.) The map is drawn
from outlines bundled with finstats; no map tiles or map service are involved, so no coordinate ever leaves the browser.

**Notifications: the one thing finstats sends rather than reads.** Under **Settings → Notifications** you can give finstats somewhere to
say what it finds: a Discord or Slack channel, a Telegram chat, a mailbox, Pushover, Pushbullet, an ntfy topic, a Gotify server, or a
webhook of your own. Until you add a destination, nothing is sent anywhere: no destination, no request. A destination is told only the kinds of event ticked for it, and nothing that happened before it
existed, so adding one cannot replay a year of history at you.

- **Addresses and places stay out of the message** unless you switch *Include IP addresses and places* on for that destination. Without it,
  an impossible-travel message says "Oslo, Norway and London, United Kingdom" and never the addresses behind them. This is worth a thought
  for a destination somebody else runs: a Discord webhook means Discord holds whatever the message says.
- **A destination's address is a password.** A Discord webhook URL carries its own token, so finstats stores the address the way it stores an
  API key: never shown again, never written to a log, never part of a backup, and never sent back to the browser; the page shows the host.
- **Who may add one.** Only a Jellyfin administrator can add a destination for the server. Anybody else needs the *Be sent notifications*
  permission, their destination may only point at a public address (not something inside your network), and it is sent only what that person
  can already see in finstats: their own requests and alerts, somebody else's plays only if they may see everyone's activity, addresses only
  if they may see network details.
- **No redirect is followed**, for the same reason as everywhere else: a token must not travel to wherever a redirect points. The three
  services whose address is their own (Telegram, Pushover, Pushbullet) are reached at that address and no other, so a token issued by one
  of them can never be posted to a look-alike host.
- **Mail is encrypted or it does not go.** `smtps://` is encrypted from the first byte; `smtp://` starts plain and must upgrade with
  STARTTLS before anything is said. There is no third setting, and no path by which your mail password is sent in the clear.

**Settings → System → Outbound connections** lists both of the outside requests above, your Jellyfin, every Sonarr, Radarr or Seerr you have connected,
and every notification destination: what each is for, whether it is switched on, and when it last answered or last took a message. It is built
from what finstats already keeps, so the list itself learns nothing; it exists so that the paragraphs above are something you can check rather
than something you have to believe.

**What the Security page is, and is not.** A place is the centre of a city or of an ISP's region, never a household, and it can be
hundreds of kilometres off; a VPN or a phone on mobile data looks like a trip. Alerts (*impossible travel*, *new country*) are a
reason to look, not proof. The page needs both *see network details* and *see everyone's activity*; resolving alerts needs *manage*.

**The services you connect.** Under **Settings → Connections** a Jellyfin administrator can point finstats at Sonarr, Radarr and Seerr. These are
requests to addresses *you* enter, normally on your own network; finstats contacts nothing on its own account, and the two outside requests above
stay the only ones it makes by itself. (Notification destinations are addresses you enter too; see below.) Your download client is not connected to finstats at all: Sonarr and Radarr already talk to it, and finstats reads their
queues instead.
- **Read-only.** finstats sends `GET` to these services and nothing else: there is no code in it that could approve a request, start a search,
  or add, pause or remove a download.
- **As rarely as is useful, and as small as they allow.** Seerr is asked every five minutes, but a pass with nothing to read is a single
  row: finstats asks for the newest-changed request, recognises it, and stops. Sonarr's and Radarr's queues are read every five seconds
  only while a page is showing them, every minute while something is in them, and every five minutes while there is not. Every read also
  asks for a compressed answer (`Accept-Encoding: gzip, br`), which these services and Jellyfin all give.
- **Keys and passwords** are stored in finstats' database next to the Jellyfin key, are never sent back to the browser (the settings page only
  learns that one is stored), never written to a log and never part of a backup. They travel in headers or request bodies, never in an
  address (finstats' own calendar feed is the one address that carries a key, above, and that key opens nothing else).
- **No redirect is followed.** A key would travel along a redirect to wherever it points, so finstats reports a redirect as an error and asks for
  the final address instead.
- **Certificates are verified.** A service with a self-signed certificate needs "Accept a self-signed certificate" switched on for that one
  connection; finstats then does not verify who answers there. Plain `http://` on your own network needs no switch.
