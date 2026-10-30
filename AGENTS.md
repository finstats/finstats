# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

FinStats is a playback-statistics server for Jellyfin: one Rust binary (axum + tokio + bundled SQLite) with the web UI
compiled into it. It only ever *reads* from Jellyfin; it never triggers scans or writes anything there except creating its
own API key during setup. Plays come from a live collector (Jellyfin's WebSocket, else polling `/Sessions`) and from
Jellystat, Streamystats and Tautulli backups; Sonarr, Radarr and Seerr feed the Pipeline page.

**The name is written FinStats** wherever a person reads it: prose, the UI, messages, notification senders, page titles.
It stays `finstats` only where it is an identifier: the repository, organisation and domain, the image, the crate and the
binary with its commands (`finstats restore <file>`), `finstats.db`, `FINSTATS_*`, `RUST_LOG=finstats=…`, browser-storage
keys, the User-Agent, the calendar `PRODID`, and `jellyfin::APP_NAME`, which is how FinStats finds its own API key in
Jellyfin, so renaming it would make every install create a second key.

## Read first

**At the start of a new session, read [`CONTEXT.md`](CONTEXT.md)**: the map (layout, data flow, the tables and how they
relate, the API surface, commands, environment variables, testing, known gaps). Then
[`ARCHITECTURE.md`](ARCHITECTURE.md) (how the pieces work and fit together, its contents at the top) and
[`DECISIONS.md`](DECISIONS.md) (the contracts: every rule with its measured reason and every owner's decision, as numbered
items per section, cited by section name and number, "Collector" 5).

**Before adding a feature** (an endpoint, a table, a job, an importer, a setting, a page, a card), read the parts of
`ARCHITECTURE.md` and `DECISIONS.md` that it touches: the ARCHITECTURE section of every module involved and the DECISIONS
sections named after them. They hold the rules the new code must honour, and most of them exist because breaking them
once lost data, leaked something or hung a server. Search them by section name rather than reading them whole.

**Keep all three current in the same commit as the code.** `CONTEXT.md` changes whenever something it describes changes (a
subsystem, module, table, migration, route group, permission, environment variable, command, test pattern or convention, a
moved file, a known gap fixed or found), with its *Last updated* line bumped. A new or changed rule is a DECISIONS item:
added at the bottom of its section, or corrected in place, or marked **Superseded** with a pointer; numbers never change.
A new module or flow gets its place in ARCHITECTURE. A change that leaves them describing the old code is not finished,
because the next session starts from them.

## How we work: TDD

**Every change in this repository is written test-first, following the Red–Green–Refactor cycle (Martin Fowler).** No
production code is written before a failing test asks for it.

1. **Red**: write one small test for the next slice of behaviour, and run it. It must *fail*, for the right reason: the
   behaviour does not exist yet. A test that passes the moment it is written proves nothing.
2. **Green**: write the least code that makes it pass. No clean design yet; the only goal is a green bar.
3. **Refactor**: with the tests green, clean up the code and the test (duplication, names, simplicity). Run the whole suite
   after each step; it must stay green.

Then loop, in small steps (minutes, not hours), so a red bar always points at the last change. Pure logic (a guard, a
parser, a permission rule) is tested in `#[cfg(test)]` modules next to the code (`cargo test`, ~0.5 s). Behaviour only
observable from outside (an endpoint, an auth boundary, the collector on a real socket, wrong data from Jellyfin) gets its
failing check in the local `qa/` suite first. A bug fixed is a bug that first gets a test reproducing it (Red), then the
fix (Green). Do not add production behaviour that no test named, and do not delete a test to make the bar green.

## Commands

```sh
cargo build --release                      # release binary embeds web/ and CHANGELOG.md
cargo test                                 # all unit tests
cargo test relink                          # tests whose name/module matches "relink"
cargo test recap::tests::persona_prefers_the_most_distinctive_habit -- --exact
cargo clippy --all-targets                 # not clean: ~17 style warnings exist (QA's budget is 20); add no new ones

FINSTATS_DATA_DIR=./data FINSTATS_BIND=127.0.0.1:8088 cargo run     # serve
cargo run -- import-jellystat <backup.jsonl>                         # headless import, then exit
cargo run -- import-streamystats <backup.json>                       # the other tracker, same thing
cargo run -- relink                                                  # re-attach history to renamed items, then exit
cargo run -- restore <backup.jsonl.gz> [--with-settings]             # the history only; settings and grants with the flag

for f in $(find web/assets/js -name "*.js"); do node --check "$f"; done   # the only JS check there is
docker build -t finstats:latest .        # local image; the published one is ghcr.io/finstats/finstats
```

- **Do not run `cargo fmt`.** The code is deliberately not rustfmt-formatted (hundreds of long lines); formatting would
  rewrite every file. Match the surrounding style by hand.
- Debug builds read `web/` from disk at runtime (rust-embed), so UI edits only need a browser refresh. Release builds embed
  it, so rebuild to see UI changes. `CHANGELOG.md` is `include_str!`'d, so it always needs a rebuild.
- Env: `FINSTATS_DATA_DIR`, `FINSTATS_BIND`, `FINSTATS_TRUST_PROXY`, `FINSTATS_PUBLIC_IP_URL`, `FINSTATS_GEOIP_DB`,
  `FINSTATS_ALLOW_LIBRARY_SHRINK`, `FINSTATS_SKIP_PREUPDATE_BACKUP`, `JELLYFIN_URL` + `JELLYFIN_API_KEY` (skip the
  wizard), `TZ`, `RUST_LOG`. `FINSTATS_TRUST_PROXY` is a list of the proxies' addresses or CIDR ranges (comma or space
  between them); `1`/`true` still works and believes only a peer on this machine or a private network, one hop; a list it
  cannot read stops the start.
- Generated files: `tools/make-third-party.py` (`THIRD-PARTY.json`, after any dependency change), `tools/make-languages.py`
  (`.github/languages.svg`, after any language change), `tools/make-world-map.py` (`web/assets/geo/world.json`).

## How the pieces fit

```
Jellyfin ──/socket push, else /Sessions 1 s / 5 s idle─▶ collector ──▶ SQLite ◀── stats / recap API ◀── embedded SPA (web/)
         ──/Users /Items /Devices /System/* (scheduler)──▶ sync ──┘        ▲
Jellystat / Streamystats backup ──▶ import ──────────────────────┘   relink (after sync/import/start-up)
Sonarr/Radarr ──calendar, history (15 min)──┐
Seerr ──requests (5 min, one row if quiet)──┼──▶ SQLite ──▶ pipeline API ◀── /pipeline
Sonarr/Radarr queues ───────────────────────┴──▶ in-memory snapshot (5 s watched / 60 s busy / 5 min empty) ──▶ /api/downloads
```

- **State**: `AppState` (`state.rs`) holds the DB handle, `Settings` (changed only through `AppState::update_settings`),
  the task registry and the now-playing snapshot; background loops select on `wake`. All DB work goes through
  `db.call(|conn| …)`.
- **One play-row shape** (`playback.rs::PlayRecord`, `media.rs`) for the collector and every importer; one duplicate rule
  (`playback::already_recorded`); one thing that rewrites `item_id` (`relink`), which re-applies it.
- **One scope for every answer**: stats queries go through `Scope` → `Cond` (`stats.rs`), which carries whose rows the
  caller may see (`Perms`) and what of the library Jellyfin lets them see (`access.rs`). The live channel and the GETs
  build their answers with the same functions.
- **Derived data is computed after a library read, stored, and only read by pages** (`backfill_playbacks`, relink,
  watchlist, Library health, Capacity).
- **The web UI** is vanilla ES modules (no build step, no dependencies, no CDN) on FinUI (FinStats' component library,
  also `../finui`) and FinMotion (`../finmotion`), in 23 languages from `web/assets/i18n/`.

## Rules the code enforces

Each has its full reason in DECISIONS.md.

- **Never wipe what a read cannot see**: library reads ask for real items, and every destructive removal goes through
  `sync::trustworthy_removal`, which refuses and halts (exit 70) rather than mark a library removed on a bad answer
  ("Library reads").
- **Migrations are immutable** once released, and one in the working tree may already have run on the live database (a
  deploy builds the working tree): add, never edit or reorder ("Database" 6). A newer binary snapshots the database first
  ("Database" 8).
- **Every pooled transaction is IMMEDIATE**; a read-only one asks for `Deferred` by name ("Database" 3).
- **Every reading of history reads `visible_playbacks`**; the table only for identity, and a test scans `src/` for it ("Trash" 3, 4).
- **Permissions are enforced server-side**, decided by the specific permission (`see_network` for IPs, `see_server` for
  paths, `see_everyone` for whose rows), never by `is_admin`; the UI's `can()` is cosmetic ("Stats layer", "Permissions").
- **History is not narrowed by library access; everything else about the library is** ("Library access").
- **Secrets never leave**: no `Debug`/`Serialize` on anything holding a key or a destination's address, never in a backup,
  a stored secret goes only to the place it was saved for, and errors never carry an upstream body ("Pipeline",
  "Notifications", "Backups").
- **A stranger's text never shapes a log line, a message or the DOM**: logged escaped, escaped before it is cut, reached
  the DOM only through `h()`/`textContent` ("Logging", "Notifications" 16, "Front-end" 3).
- **Nothing that was not measured is shown as if it were** (an ETA, a "dubbed", a busiest moment of somebody else's
  evening) ("Jellyfin's jobs" 3, "Track languages" 3, "Capacity" 8).
- **Every word a person reads comes from a catalogue**, and a new text lands in the four checked languages in the same
  change ("Languages" 2, 17).
- **Every colour is a `light-dark()` token**, no inline style, no literal radius ("Theming", "FinUI" 4, "HTTP server" 5).
- **A new outbound destination** appears in `outbound.rs` and in the promise sentences of `README.md` and the docs'
  `finstats/privacy.md` and `finstats/security.md` in the same change ("Outbound connections" 3).
- **A new table holding something Jellyfin cannot give back** goes in `backup::TABLES` ("Backups" 1).
- **A new importer** goes through `PlayRecord::insert_imported`, never `insert` ("Imports" 1).
- **Release builds check for overflow** (`overflow-checks = true`); sums over the server's numbers are checked or
  saturating ("HTTP server" 6).

## Documentation lives in its own repository

**Every page of user-facing documentation is written in `github.com/finstats/docs`** (checked out beside FinStats as
`../docs`; MkDocs with Material, published by its Pages workflow at <https://finstats.no/>), never in this repository (the
owner's decision, 2026-10-07). There is no `docs/` folder here. What stays here is what GitHub or the build needs, plus the
developer files at the root: `README.md` (a short landing page: the pitch, the hero shot, the build blockquote, one
`docker run`, links to the docs), `CONTRIBUTING.md` and `SECURITY.md` (pointers), `CODE_OF_CONDUCT.md`, `.github/`,
`CHANGELOG.md` (`include_str!`'d as the Patch notes), and `CLAUDE.md`, `CONTEXT.md`, `ARCHITECTURE.md`, `DECISIONS.md`.

- A change that a page describes (an endpoint, a setting, an environment variable, an import rule, a privacy promise)
  comes with a commit in `../docs` in the same sitting; read `../docs/CLAUDE.md` before writing there.
- Pages for a version not yet released wait on a branch of the docs named like FinStats' own (`dev-2.3.0`) and reach
  `main`, which publishes, with the release.
- The README's pictures are the docs' (`assets/screenshots/`, linked by their published address), and
  `qa/run.sh screenshots` writes them there. The one exception is `.github/languages.svg` ("Languages" 18).
- **HTTP contract**: the docs' `finstats/api.md` (`../docs/docs/finstats/api.md`) is the contract the UI is written
  against; change it together with the endpoint. The QA `routes` check reads it from there.
- **The README says how FinStats is built**: a blockquote under the hero shot disclosing that most of the code was written
  by Claude Code from the maintainer's decisions, and what that does and does not mean: test-first, reviewed before it
  lands, `cargo test` in CI on every push to `main` and every release tag, each release run against a real server. Those
  are promises, like the privacy sentences: if any stops being true, the blockquote changes in the same commit, and so does
  its copy at the top of the docs' `finstats/index.md`. It names no model version on purpose, so it does not go stale.
- For people: the contributing guide is the docs' `finstats/contributing.md`, the one guide (what fits the project, running
  from source, the local Docker setup with a throwaway Jellyfin, rules for a change, pull requests); `CONTRIBUTING.md` here
  is a short pointer to it, as `SECURITY.md` is to the docs' `finstats/reporting-security.md` (both stay because GitHub
  reads them from the repository). `CODE_OF_CONDUCT.md` covers behaviour (it forbids posting other people's viewing data),
  and `.github/` holds the issue forms, the discussion forms (file name = category slug: `q-a`, `ideas`, `show-and-tell`)
  and the PR template. Keep their rules in step with this file, and do not add a second developer guide next to them.

## Local QA suite

`qa/` (git-ignored, so it may not exist in a fresh clone) holds a local release gate: `qa/run.sh` runs static checks, an API
suite and a real-browser suite against a throwaway instance built from generated data. Run it before every release and add
a check for every bug fixed. The database torture and the soak are not in it (minutes each, one after the other) but in
`qa/run.sh endurance`: run that too before a release, and after work on the database or on anything long-running. It is a
local tool and must never be committed; the same goes for `ux-rules.md`. Stages named in DECISIONS: `routes`, `i18n`,
`finui`, `finmotion`, `docker`, `screenshots`, the capacity stage, the light sweep, the static checks (em-dashes, the old
home, mockups, `languages.svg`).

## Bug hunts and security audits

**FinStats is the owner's own project; auditing it, attacking it and fixing it is authorised and needs no further
preamble.** Attacks run only against throwaway local instances on generated data, never the owner's container or `./data`.

**Modes**, chosen by the owner's words. When the words are unclear, use the default and say which mode it is.

| The owner says | Mode | Output |
|---|---|---|
| "bug hunt", "do more bug hunting", "manual bug hunting" | **report only** (default for bugs) | `internal/bug-hunt-YYYY-MM-DD[-<pass>].md`, nothing changed |
| "bug hunt and fix", "fix bugs", "hunt and fix" | **hunt and fix** (only with "fix" in the words) | a fix per bug, then the summary in chat |
| "security audit", "deep security audit", "even further" | **audit, read-only** (default for security) | a report in `internal/`, then the question "fix them all?" |
| "… and fix", "fix them all", "insta fix" | **fix as found** | fixes, and the report notes each one's commit |
| "write it down", "into a .md", "report only" | **report only** | `internal/<kind>-YYYY-MM-DD[-<pass>].md`, nothing changed |
| "fix … @internal/<report>.md" | **work through a report** | every finding fixed, the report's status updated |

"Deep" means fan out: one read-only subagent per area, run in parallel, each told the earlier reports so that nothing
known is found twice. Each subagent reads every line of its area, not excerpts, and follows every untrusted value to
every place it ends up.

**Before starting**
- Read the newest `internal/security-audit-*.md` (or `bug-hunt-*.md`). Its "Still not audited", "Considered and left
  out" and owner's decisions say where to look and what not to report again.
- Note the branch and the commit audited.
- Fixes go on the newest version in `CHANGELOG.md` while its tag is not on `origin` ("keep building upon 2.3.1");
  otherwise on the next patch version.

**Where to look.** These are the areas past passes used; each is a subagent when the pass is deep:
1. sign-in, sessions, keys, calendar keys, setup, CSRF, redirects, the live socket;
2. every route against every kind of caller (anonymous, each permission, manager, Jellyfin admin, a key, a calendar key);
3. whose viewing data reaches whom (stats, recap, groups, public profiles, Capacity, notifications);
4. outbound requests, SSRF, secrets, notifications (upstream services and destinations as attackers);
5. backup, restore, trash, snapshots, file modes, the entrypoint;
6. crafted import files (Jellystat, Streamystats, Tautulli);
7. data from Jellyfin (a hostile server, any Jellyfin user's device names, play reports and failed sign-ins);
8. the HTTP layer and resource exhaustion (connections, body and frame sizes, allocations driven by input, panics,
   poisoned locks);
9. the browser side (every DOM sink, headers, `href`/`src`, the CSP);
10. concurrency and races, the Docker/CI/dependency side.

11. supply chain: `Cargo.lock` against RustSec advisories, `unsafe` in our code and in dependencies, build scripts,
    `.github/workflows/` (action pinning, token permissions, what a pull request can run), the Dockerfile and image;
12. the sister repositories FinStats ships or points people at: `../finui` (its `install.sh`, piped to `sh`),
    `../finmotion`, `../docs` (the published site), and the `demo` branch;
13. every past fix again: a second path round it, an incomplete rule, a new bug it brought.

**Going further.** When the owner says "further", "even further", "exhaustive" or "max", an audit does more than
"deep":
- **Passes repeat** until a whole pass finds nothing new. Each pass is told everything found so far.
- **Variant analysis**: every finding becomes a pattern, and the whole tree (`src/`, `web/`, FinUI, the QA mocks'
  assumptions) is searched for other instances of it.
- **Chains**: Low and Info findings are combined to see whether together they reach something Medium or worse. A chain
  is reported as its own finding.
- **Leads not proved**: something plausible that could not be proved goes under its own heading, with what was tried,
  instead of being dropped. Only proved findings are ranked.
- **Tools may be installed** into the scratchpad (`cargo install --root <scratchpad>`, `CARGO_TARGET_DIR` there too):
  `cargo-audit`, `cargo-deny`, `cargo-geiger`, `cargo-fuzz` (needs nightly through `rustup`), `semgrep`, `actionlint`,
  `zizmor`, `trivy` or `grype` for the image. `nmap` is already installed; use it only against our own throwaway
  instances.
- **Allowed methods:**
  - fuzz harnesses and property tests written in the scratchpad (a crash they find becomes a regression test in the fix);
  - hostile mocks of Jellyfin, Sonarr, Radarr, Seerr, an SMTP server and a webhook;
  - crafted backups and import files;
  - concurrent request storms for races;
  - timing measurements for side channels;
  - reading dependencies' source in `~/.cargo/registry`;
  - a release build, to see overflow checks and limits as shipped.
- **Internet**, in this mode only: fetch tools, advisory databases and upstream source. Never send traffic to a real
  third-party service as part of an attack, and never add a real destination.
- **Subagents**: as many as the areas need. The owner's "further" is the opt-in to that scale. The coverage table records
  every area as done, partial or not done, so the next pass knows where to start.

Bug hunts also look for wrong answers, stuck or stale screens, data safety, speed, and anything that can crash, halt,
leak or lose data. Read the source, call the API, brute-force inputs, drive the browser: any method is fine.

**How to prove a finding**
- Use throwaway instances: `target/debug/finstats` or `target/release/finstats` with the QA seed and mocks, data folders
  in the session scratchpad, and ports from 9300 to 9389, one range per subagent. Stop them all afterwards.
- **The sandbox has internet.** Point the public-IP lookup (`FINSTATS_PUBLIC_IP_URL`) at a dead local port, press no
  DB-IP download, and add no real notification destination. One past pass downloaded DB-IP's 127 MB file by accident.
- No `docker exec`, no touching the owner's container or `./data`, nothing pushed.
- Before a finding goes in the report, check it by hand against the lines cited. Leave out anything speculative or
  unchecked, and generic hardening advice.

**The report** goes in `internal/` (excluded in `.git/info/exclude`, never committed). Security findings are
**embargoed**: they stay out of commits, the changelog, `CONTEXT.md`, `DECISIONS.md` and the docs until they are fixed,
and even then only the fix is described there. Shape, as in the past reports:
1. Title `# Security audit, <nth> pass, YYYY-MM-DD (embargoed)`, with a line saying it is local only.
2. **What was audited**: branch and commit, and which earlier reports' fixes it comes after.
3. **How it was done**: one row per area with its outcome (done, partial, not done; read only or proved on an instance),
   plus anything that reached the internet.
4. **Summary, most severe first**: `| # | Severity | Who | Finding | Fixed in |`. Severity is High, Medium, Medium-Low,
   Low-Medium, Low or Info; "Who" is the least an attacker needs.
5. One `## N. Severity: title` per finding, with **Where** (file:line), **Who**, **Request** (the exact request or input),
   **Gain**, **Fix** (with the test that should fail first).
6. **Considered and left out**, **Checked and found sound**, **Still not audited**, the owner's decisions,
   **Before it goes to GitHub** (docs pages, screenshots, the tag), and a **Status** line kept current.

Numbers never change once written; a later pass adds a note under an earlier finding instead of renumbering. A bug hunt
in report-only mode uses the same shape, titled `# Bug hunt, YYYY-MM-DD`, with its findings grouped as the chat summaries
were: Data safety, Wrong answers, Stuck or stale screens, Privacy and access, Performance.

**Fixing**, in any mode:
- One finding at a time, test-first: a unit test or a QA check seen red against the old code, then the fix. Each fix is
  its own `fix(<scope>):` commit, and `cargo test` passes on every commit.
- Add the QA check for it to `qa/` and push that to the QA repository.
- Changelog bullets go in the four checked languages: under **Security** if somebody could have misused it, otherwise
  Stability, Fixed or Performance ("Changelog" 6). Every `changelog/<tag>.md` is done before the release.
- Update `DECISIONS.md` (the rule and why, never the exploit), `CONTEXT.md` and `../docs` as for any change.
- Update the report as you go: the commit in "Fixed in", and the Status line.
- Then redeploy the owner's container and run the full `qa/run.sh`. On a failure: fix, redeploy, run QA again, until
  it is green. No endurance unless asked.
- Never push and never tag-push without the owner's word.
- End with a chat summary: what was fixed (grouped as above), what was left and why, and whether QA and the redeploy are
  green.

A short, standalone brief for an audit by another model or session is in `internal/security-audit-prompt.md`.
Local shortcuts, git-excluded like `internal/`: `/audit [deep|further] [fix] [scope]`, `/bughunt [deep] [fix] [scope]` and
`/fix-report [@internal/<report>.md]` (`.claude/skills/`).

## Releases and patch notes

`CHANGELOG.md` is the source of the in-app **Patch notes**; its format, groups and translation rules are "Changelog" in
DECISIONS.md, held by `changelog::tests::problems`. A note lands in the four checked languages in the same commit.

History shape: a release is its logical commits (backend before UI, `fix(...)` on their own, then `docs:`), followed by
`chore(release): x.y.z` bumping `Cargo.toml` + `Cargo.lock`, and an annotated tag `vX.Y.Z`. Every commit that touches Rust
must build and pass `cargo test` on its own.

## Licensing

FinStats is `GPL-3.0-only` (`LICENSE`, `Cargo.toml`). A new dependency must carry a GPL-3.0-compatible license (MIT,
Apache-2.0, BSD, ISC, Zlib, MPL-2.0 and similar are fine; check with `cargo metadata`), and `tools/make-third-party.py` is
re-run ("Licence notice"). The bundled fonts are OFL-1.1 and their license texts live next to them in `web/assets/fonts/`;
keep them together.

## Publishing

- The image and the data folder's ownership and modes are "Image and data folder" in DECISIONS.md. Test the image on
  folders Docker creates (`qa/run.sh docker`).
- **The owner's own install runs from this checkout, and it is redeployed at the end of every change.** When a container
  named `finstats` is running (`docker ps`), finishing a feature or a fix means rebuilding the local image and swapping the
  container without being asked: `docker build -t finstats:latest .`, `docker rm -f finstats`, then the same `docker run`
  it had (read its binds, env and ports from `docker inspect` first, never from memory), and a check of `/api/status`, the
  log and the health state afterwards. Work that only lands in the repository while `localhost:8080` still shows the old
  build is not finished. A newer binary snapshots the database by itself before it changes anything
  (`back_up_before_update`); take a manual `sqlite3 data/finstats.db ".backup …"` too when the change is risky.
- The repository is `github.com/finstats/finstats`; images go to `ghcr.io/finstats/finstats`. Up to 1.6.5 both lived under
  the maintainer's personal account; 2.0.0 is the first release from the organisation. Nothing but the changelog may name
  the old home any more (a QA static check holds it), and the 25 GitHub releases up to 1.6.5 were rewritten to match.
- `.github/workflows/docker.yml` runs the unit tests, builds amd64 and arm64 on native runners (no QEMU), and publishes
  `:edge` from `main` and `:X.Y.Z`, `:X.Y`, `:X`, `:latest` from a `vX.Y.Z` tag, then creates the GitHub release from that
  version's `CHANGELOG.md` section. It refuses a tag that does not match `Cargo.toml` or has no changelog entry, so the
  release commit and its tag must be pushed together. Docs always point at the published image, never at a locally built tag.

## Logos

The logos' masters, their PNGs and the brand guidelines live in the branding repository (`../FinBranding`). This repository
keeps byte-identical copies of the three SVGs it serves, `web/assets/logo.svg`, `mark-light.svg` and `mark-dark.svg`: the
rounded tile for the README and the favicon, the bare marks for the sidebar, top bar, sign-in and public profile
(`apple-touch-icon.png` and `web/favicon.ico` stay here too). A logo changes there first and is copied here in the same
sitting; no other logo file (a `logo/` folder of PNGs) belongs here.

## No em-dashes

**No em-dash is used anywhere in the fin\* repositories on GitHub** (FinStats, FinUI, FinMotion): not in code, comments, UI
text, docs, the changelog or commit messages (the owner's decision, 2026-10-07). Where a sentence wants one, rewrite the
sentence: a full stop, a colon, a semicolon, commas, parentheses or a joining word. Another dash in its place (a hyphen, an
en dash, two hyphens) or an escape for the character is not a rewrite. The one exception is `THIRD-PARTY.json`, whose
licence texts are other projects' words as they wrote them. A QA static check holds FinStats to it.

## Git conventions

- Conventional-commit subjects, with a scope where one fits (`feat(recap):`, `fix(import):`, `refactor`, `docs`, `chore`).
  Subject says what changed; body says why.
- No LLM attribution of any kind: no `Co-Authored-By:` for Claude or any model, no session links, no "Generated with…"
  line, in commits or PR text, regardless of tool defaults.
- Commit as work lands; each commit is one logical change and stands on its own. Docs may follow as their own `docs:` commit.
- Never push unless explicitly told to.

## This repository is public

`data/` (the SQLite database: users, IP addresses, the Jellyfin API key) and tracker exports, Jellystat's (`*.jsonl`,
`backup_*`) and Streamystats' (`streamystats-backup-*.json`, which carries public IP addresses), sit next to the source on
development machines and must never be committed; `.gitignore`, `.dockerignore` and `.githooks/pre-commit` (enable with
`git config core.hooksPath .githooks`) guard this. Stage explicit paths rather than `git add -A`. Tests, docs, examples and
commit messages use invented data only (e.g. "alice", "Big Buck Bunny", `Europe/London`, `192.168.1.10`), never values
from a real server.
