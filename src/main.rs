mod access;
mod api;
mod appearance;
mod artwork;
mod arr;
mod audit;
mod auth;
mod backup;
mod browse;
mod capacity;
mod card;
mod changelog;
mod channels;
mod collector;
mod db;
mod downloads;
mod fuzzy;
mod geo;
mod finui;
mod finmotion;
mod groups;
mod health;
mod i18n;
mod ical;
mod import;
mod jellyfin;
mod jobs;
mod keys;
mod licenses;
mod live;
mod locate;
mod mail;
mod media;
mod network;
mod notify;
mod outbound;
mod pipeline;
mod playback;
mod profile;
mod public;
mod recap;
mod recent;
mod relink;
mod schedule;
mod security;
mod seerr;
mod services;
mod socket;
mod state;
mod stats;
mod story;
mod streamystats;
mod sync;
mod tautulli;
mod timeline;
mod trash;
mod upgrade;
mod watchlist;
mod welcome;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};

use anyhow::{Context, Result, bail};
use rand::RngCore;
use tokio::sync::Notify;

use state::{AppState, CollectorStatus, JfConfig, Settings, Tasks};

const USAGE: &str = "FinStats: playback statistics for Jellyfin

USAGE:
    FinStats                            Run the server
    finstats import-jellystat <file>    Import a Jellystat backup (.jsonl / .json), then exit
    finstats import-streamystats <file> Import a Streamystats backup (.json), then exit
    finstats backup                     Write a backup into <data dir>/backups, then exit
    finstats restore <file> [--with-settings]
                                        Merge a FinStats backup's history into the database, then exit;
                                        --with-settings also brings back its settings and permissions, never wider than here
    finstats relink                     Re-attach history to renamed items now, then exit (also runs after every sync)
    finstats --version

ENVIRONMENT:
    FINSTATS_DATA_DIR      Where the database and caches live   (default: ./data)
    FINSTATS_BIND          Address to listen on                  (default: 0.0.0.0:8080)
    FINSTATS_TRUST_PROXY   The reverse proxy's address or range (10.0.0.2, 172.18.0.0/16, …) whose X-Forwarded-For
                           is believed; 1 believes any peer on this machine or a private network, one hop
    FINSTATS_GEOIP_DB      A city database (.mmdb) to place addresses with (default: newest in <data dir>/geoip)
    FINSTATS_PUBLIC_IP_URL Where to look up this network's public address (a service answering it as plain text)
    FINSTATS_ALLOW_LIBRARY_SHRINK  Set to 1 to accept a library read that empties or nearly empties a library
    FINSTATS_SKIP_PREUPDATE_BACKUP Set to 1 to upgrade without copying the database first
    JELLYFIN_URL           Optional: skip the setup wizard…
    JELLYFIN_API_KEY       …together with an API key
    TZ                     Timezone used for \"per day\" and \"hour of day\" statistics
    RUST_LOG               Log filter                            (default: finstats=info)";

/// SQLite's own complaint about a directory it may not write to is "unable to open database file",
/// and it arrives after the pool has waited half a minute. Say what is wrong, at once.
fn ensure_writable(data_dir: &std::path::Path) -> Result<()> {
    let probe = data_dir.join(format!(".finstats-write-test-{}", std::process::id()));
    match std::fs::write(&probe, b"") {
        Ok(()) => {
            let _ = std::fs::remove_file(&probe);
            Ok(())
        }
        Err(e) => bail!(
            "FinStats cannot write to its data directory {dir} ({e}).\n\n\
             It belongs to another user. This usually happens when Docker created the folder itself, as root.\n\
             Fix it with:   sudo chown -R 1000:1000 <your data folder>\n\
             or let the container start as root (no --user / user: setting): it then fixes this by itself\n\
             and drops to an unprivileged user before FinStats runs. PUID and PGID choose that user.",
            dir = data_dir.display()
        ),
    }
}

/// Why FinStats will not run as root (`euid` 0) over a data folder that belongs to `owner`: everything it wrote would
/// belong to root, and FinStats, running as the folder's owner, could no longer write it (a `backups/` made by
/// `docker exec finstats finstats backup`). `None` when that is not the case.
fn as_root_over(euid: u32, owner: u32) -> Option<String> {
    (euid == 0 && owner != 0).then(|| {
        format!(
            "FinStats does not run as root over a data folder that belongs to user {owner}: what it wrote would belong to\n\
             root, and FinStats could no longer write it. Run it as that user. With Docker:\n\
             \n    docker exec -u {owner} finstats finstats <command>\n"
        )
    })
}

/// The effective user id in the text of `/proc/self/status` (its `Uid:` line: real, effective, saved, filesystem).
fn effective_uid_in(status: &str) -> Option<u32> {
    status.lines().find_map(|l| l.strip_prefix("Uid:"))?.split_whitespace().nth(1)?.parse().ok()
}

/// What FinStats was asked to do; everything but `--version` and `--help`, which need nothing at all.
enum Command {
    Serve,
    ImportJellystat(PathBuf),
    ImportStreamystats(PathBuf),
    Backup,
    /// The file, and whether the settings and permissions come back too.
    Restore(PathBuf, bool),
    Relink,
}

fn command(args: &[String]) -> Result<Command> {
    let file = |cmd: &str| args.get(1).map(PathBuf::from).with_context(|| format!("usage: finstats {cmd} <file>"));
    Ok(match args.first().map(String::as_str) {
        None | Some("serve") => Command::Serve,
        Some(cmd @ "import-jellystat") => Command::ImportJellystat(file(cmd)?),
        Some(cmd @ "import-streamystats") => Command::ImportStreamystats(file(cmd)?),
        Some("backup") => Command::Backup,
        Some("restore") => {
            let usage = || anyhow::anyhow!("usage: finstats restore <file> [--with-settings]");
            let (flags, files): (Vec<&String>, Vec<&String>) = args[1..].iter().partition(|a| a.starts_with("--"));
            let with_settings = match flags.as_slice() {
                [] => false,
                [flag] if flag.as_str() == "--with-settings" => true,
                _ => return Err(usage()),
            };
            match files.as_slice() {
                [file] => Command::Restore(PathBuf::from(file), with_settings),
                _ => return Err(usage()),
            }
        }
        Some("relink") => Command::Relink,
        Some(other) => bail!("unknown command `{other}`\n\n{USAGE}"),
    })
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("--version" | "-V") => {
            println!("FinStats {}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
        Some("--help" | "-h" | "help") => {
            println!("{USAGE}");
            return Ok(());
        }
        _ => {}
    }

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "finstats=info".into()))
        .with_target(false)
        .init();

    // What was asked is settled before anything is opened: a mistyped command, or one missing its file, used to
    // create the data folder and open (and so migrate) the database before saying it did not know what to do.
    let command = command(&args)?;
    let data_dir = PathBuf::from(std::env::var("FINSTATS_DATA_DIR").unwrap_or_else(|_| "data".into()));
    // For the user FinStats runs as alone: everything in it holds the Jellyfin key, a secret or everybody's history, and a
    // folder nobody else may enter keeps whatever is made inside it (SQLite's own files included) from them.
    std::fs::create_dir_all(&data_dir).with_context(|| format!("creating data directory {}", data_dir.display()))?;
    {
        use std::os::unix::fs::MetadataExt;
        let euid = std::fs::read_to_string("/proc/self/status").ok().as_deref().and_then(effective_uid_in);
        if let (Some(euid), Ok(meta)) = (euid, std::fs::metadata(&data_dir))
            && let Some(why) = as_root_over(euid, meta.uid())
        {
            bail!(why);
        }
    }
    if let Err(e) = db::private_dir(&data_dir) {
        tracing::warn!("could not make {} private to the user FinStats runs as ({e}); other accounts on this machine may read it", data_dir.display());
    }
    ensure_writable(&data_dir)?;
    let db = db::Db::open(&data_dir.join("finstats.db"))?;
    db::private_file(&data_dir.join("finstats.db"));

    match command {
        Command::Serve => {}
        Command::ImportJellystat(file) => {
            let started = std::time::Instant::now();
            let res = import::run(&db, &file, None)?;
            println!(
                "Imported {} plays ({} skipped as duplicates), {} users, {} libraries, {} items, {} seasons, {} episodes in {:.1}s",
                res.plays_imported, res.plays_skipped, res.users, res.libraries, res.items, res.seasons, res.episodes,
                started.elapsed().as_secs_f64()
            );
            return Ok(());
        }
        Command::ImportStreamystats(file) => {
            let started = std::time::Instant::now();
            let res = streamystats::run(&db, &file, None)?;
            println!(
                "Imported {} plays ({} already present, {} marked watched but never played) and {} users from {} sessions in {:.1}s",
                res.plays_imported, res.plays_skipped, res.marked_watched, res.users, res.sessions_read, started.elapsed().as_secs_f64()
            );
            return Ok(());
        }
        Command::Backup => {
            let made = backup::export(&db, &backup::dir(&data_dir), None)?;
            println!("Wrote {} ({} plays, {} rows, {:.1} MB)", backup::dir(&data_dir).join(&made.name).display(), made.plays, made.rows, made.size_bytes as f64 / 1e6);
            return Ok(());
        }
        Command::Restore(file, with_settings) => {
            let r = backup::restore(&db, &file, with_settings, None)?;
            println!("Restored {} plays ({} already present), {} timeline events, {} other rows; settings restored: {}", r.plays_imported, r.plays_skipped, r.events, r.other_rows, r.settings_restored);
            return Ok(());
        }
        Command::Relink => {
            let r = relink::relink_orphans(&*db.conn()?, Settings::load(&*db.conn()?)?.merge_window_s)?;
            println!(
                "Re-linked {} title plays and {} episode plays; cleaned {} names; removed {} plays that had become duplicates",
                r.titles, r.episodes, r.names_cleaned, r.duplicates_removed
            );
            return Ok(());
        }
    }

    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .max_blocking_threads(8)
        .enable_all()
        .build()?
        .block_on(serve(db, data_dir))
}

async fn serve(db: db::Db, data_dir: PathBuf) -> Result<()> {
    // A Tautulli backup waits for its wires only while somebody is drawing them: it holds every Plex user's tokens.
    if tautulli::sweep(&data_dir, std::time::Duration::ZERO) {
        tracing::info!("removed a Tautulli backup left waiting for its wires before this start");
    }
    if api::sweep_uploads(&data_dir) > 0 {
        tracing::info!("removed what an import or a restore stopped half way before this start left in the data folder");
    }
    let (config, settings, device_id) = db
        .call(|c| {
            let device_id = match db::get_setting(c, "device_id")? {
                Some(id) => id,
                None => {
                    let mut raw = [0u8; 16];
                    rand::rng().fill_bytes(&mut raw);
                    let id = hex::encode(raw);
                    db::set_setting(c, "device_id", &id)?;
                    id
                }
            };
            let stored = match (db::get_setting(c, "jellyfin_url")?, db::get_setting(c, "jellyfin_api_key")?) {
                (Some(url), Some(api_key)) => Some(JfConfig {
                    url,
                    api_key,
                    server_name: db::get_setting(c, "server_name")?.unwrap_or_else(|| "Jellyfin".into()),
                    server_version: db::get_setting(c, "server_version")?.unwrap_or_default(),
                    from_env: false,
                }),
                _ => None,
            };
            c.execute("DELETE FROM sessions WHERE expires_at <= ?1", [db::now()])?;
            network::set_manual(c, &Settings::load(c)?.home_addresses)?;
            network::reclassify_at_start(c)?;
            let settings_now = Settings::load(c)?;
            let began = db::now();
            // Re-linked plays keep their times, so the start-up regroup would not find the titles they moved onto.
            for title in relink::relink_orphans(c, settings_now.merge_window_s)?.moved_to {
                groups::detect(c, settings_now.group_window_s, Some(&title))?;
            }
            groups::regroup_at_start(c, settings_now.group_window_s, began)?;
            Ok((stored, Settings::load(c)?, device_id))
        })
        .await?;

    // Environment wins over the wizard, so a compose file stays the source of truth.
    let env = |k: &str| std::env::var(k).ok().map(|v| v.trim().to_string()).filter(|v| !v.is_empty());
    let config = match (env("JELLYFIN_URL"), env("JELLYFIN_API_KEY")) {
        (Some(url), Some(api_key)) => Some(JfConfig {
            url: jellyfin::normalize_url(&url)?,
            api_key,
            server_name: config.as_ref().map(|c| c.server_name.clone()).unwrap_or_else(|| "Jellyfin".into()),
            server_version: config.as_ref().map(|c| c.server_version.clone()).unwrap_or_default(),
            from_env: true,
        }),
        (Some(_), None) | (None, Some(_)) => bail!("JELLYFIN_URL and JELLYFIN_API_KEY must be set together"),
        _ => config,
    };

    let setup_code = config.is_none().then(auth::new_setup_code);
    let trust_proxy = auth::Proxies::parse(&env("FINSTATS_TRUST_PROXY").unwrap_or_default())?;
    if let Some(said) = trust_proxy.describe() {
        tracing::info!("{said}");
    }
    let app = Arc::new(AppState {
        db,
        data_dir,
        http: jellyfin::http_client(),
        device_id,
        trust_proxy,
        config: RwLock::new(config),
        settings: RwLock::new(settings),
        settings_write: tokio::sync::Mutex::new(()),
        tasks: Tasks::new(),
        live: RwLock::new(vec![]),
        collector: RwLock::new(CollectorStatus::default()),
        login_attempts: Mutex::new(Default::default()),
        setup_code: setup_code.clone(),
        geo: Default::default(),
        services: Default::default(),
        service_health: Default::default(),
        services_http: services::Http::new(),
        downloads: Default::default(),
        wishes: Default::default(),
        downloads_watched: Mutex::new(0),
        downloads_wake: Notify::new(),
        live_feed: Default::default(),
        public_cards: Default::default(),
        card_permits: tokio::sync::Semaphore::new(2),
        jf_jobs: Mutex::new(Default::default()),
        notify_targets: Default::default(),
        notify_wake: Notify::new(),
        wake: Notify::new(),
        halt: Notify::new(),
        halt_reason: Mutex::new(None),
    });

    services::reload(&app).await?;
    notify::reload(&app).await?;

    // Before the collector: a play that begins in the first second is looked at like any other.
    if geo::load(&app) {
        security::check(&app, None).await;
    }

    // One question about this network's public address, on an install that has never had an answer,
    // and never again unless the owner asks in Settings. It must not hold up the server.
    tokio::spawn({
        let app = app.clone();
        async move { network::refresh_if_unknown(&app).await }
    });

    tokio::spawn(collector::run(app.clone()));
    tokio::spawn(downloads::run(app.clone()));
    tokio::spawn(live::run(app.clone()));
    tokio::spawn(notify::run(app.clone()));
    tokio::spawn(sync::scheduler(app.clone()));
    tokio::spawn(jobs::run(app.clone()));
    tokio::spawn(api::prune_image_cache(app.clone()));

    let bind = env("FINSTATS_BIND").unwrap_or_else(|| "0.0.0.0:8080".into());
    let listener = tokio::net::TcpListener::bind(&bind).await.with_context(|| format!("listening on {bind}"))?;
    match app.config.read().unwrap().as_ref() {
        Some(c) => tracing::info!("FinStats {} on http://{bind}, connected to {}", env!("CARGO_PKG_VERSION"), c.url),
        None => tracing::info!(
            "FinStats {} on http://{bind}; open it in a browser to finish setup. Setup code: {}",
            env!("CARGO_PKG_VERSION"),
            setup_code.as_deref().unwrap_or_default()
        ),
    }

    let serve_app = app.clone();
    let proxies = app.trust_proxy.clone();
    serve_http(listener, api::router(app.clone()), Limits::default(), move |ip| proxies.trusts(ip), async move {
        let ctrl_c = tokio::signal::ctrl_c();
        let mut term = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()).expect("SIGTERM handler");
        // A fail-closed halt (`request_halt`) is a third way out, next to Ctrl-C and SIGTERM: FinStats
        // refused to apply something from Jellyfin and is stopping so the operator can look.
        tokio::select! {
            _ = ctrl_c => tracing::info!("shutting down"),
            _ = term.recv() => tracing::info!("shutting down"),
            _ = serve_app.halt.notified() => {}
        }
    })
    .await;
    // A clean stop returns 0; a fail-closed halt returns non-zero and says why, so a process manager
    // surfaces it (a restart loop stops as soon as the operator sets FINSTATS_ALLOW_LIBRARY_SHRINK=1 or
    // fixes Jellyfin). The refused change was never applied: the database is exactly as it was.
    if let Some(reason) = app.halt_reason() {
        tracing::error!("FinStats stopped without applying a change it did not trust: {reason}");
        eprintln!("
FinStats halted: {reason}

Nothing was changed. Check Jellyfin (an upgrade may have changed its API),
then restart. To allow it through once (for example after really emptying a library),
set FINSTATS_ALLOW_LIBRARY_SHRINK=1.
");
        std::process::exit(70);
    }
    Ok(())
}

/// What one connection may hold, and how many may be held at once.
#[derive(Clone, Copy)]
struct Limits {
    /// A request's head must have arrived this long after the connection began waiting for it: on a new connection, and
    /// between two requests on one kept alive, which is how long an idle connection is kept.
    head_within: std::time::Duration,
    /// The most connections served at once; the next waits in the listen queue until one closes.
    connections: usize,
    /// How long a request's body may take before it must keep up `body_rate`.
    body_grace: std::time::Duration,
    /// The fewest bytes a second a body must have averaged once its grace is over.
    body_rate: u64,
    /// The most connections one address (an IPv6 /64) holds at once, a trusted proxy's apart.
    per_peer: usize,
    /// How long one write may wait for the client to read.
    write_within: std::time::Duration,
}

impl Default for Limits {
    /// Fifteen seconds is far more than any browser takes to send a head, and a browser opens a new connection when an
    /// idle one was closed. Five hundred connections is a household's tabs many times over, and well inside the open
    /// files a container is given (1,024 at the least), leaving room for the database and every outbound read.
    fn default() -> Self {
        Limits { head_within: std::time::Duration::from_secs(15), connections: 512, body_grace: std::time::Duration::from_secs(15), body_rate: 1024, per_peer: 64, write_within: std::time::Duration::from_secs(30) }
    }
}

/// One connection's place in its address's share ([`Limits::per_peer`]), given back when the connection ends.
struct PeerShare {
    shares: Arc<Mutex<HashMap<std::net::IpAddr, usize>>>,
    key: std::net::IpAddr,
}

impl PeerShare {
    /// A place for one more connection from `ip` (an IPv6 address counts as its /64), or `None` when it holds `most`.
    fn take(shares: &Arc<Mutex<HashMap<std::net::IpAddr, usize>>>, ip: std::net::IpAddr, most: usize) -> Option<PeerShare> {
        let key = auth::attempt_bucket(ip);
        let mut held = shares.lock().unwrap();
        let count = held.entry(key).or_default();
        if *count >= most {
            return None;
        }
        *count += 1;
        Some(PeerShare { shares: shares.clone(), key })
    }
}

impl Drop for PeerShare {
    fn drop(&mut self) {
        let mut held = self.shares.lock().unwrap();
        if let Some(count) = held.get_mut(&self.key) {
            *count -= 1;
            if *count == 0 {
                held.remove(&self.key);
            }
        }
    }
}

/// A request body that must have averaged `rate` bytes a second once `grace` is over, or it ends in an error (and the
/// connection with it). hyper times only the head: a body promised and sent a byte at a time held its connection for good.
struct MinRate {
    inner: hyper::body::Incoming,
    started: tokio::time::Instant,
    got: u64,
    grace: std::time::Duration,
    rate: u64,
    timer: std::pin::Pin<Box<tokio::time::Sleep>>,
}

impl MinRate {
    fn new(inner: hyper::body::Incoming, grace: std::time::Duration, rate: u64) -> MinRate {
        let started = tokio::time::Instant::now();
        MinRate { inner, started, got: 0, grace, rate, timer: Box::pin(tokio::time::sleep_until(started + grace)) }
    }

    /// When the body must have sent more than it has.
    fn due(&self) -> tokio::time::Instant {
        self.started + self.grace + std::time::Duration::from_secs_f64(self.got as f64 / self.rate.max(1) as f64)
    }
}

impl http_body::Body for MinRate {
    type Data = axum::body::Bytes;
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn poll_frame(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
        use std::task::Poll;
        match std::pin::Pin::new(&mut self.inner).poll_frame(cx) {
            Poll::Ready(Some(Ok(frame))) => {
                if let Some(data) = frame.data_ref() {
                    self.got = self.got.saturating_add(data.len() as u64);
                    let due = self.due();
                    self.timer.as_mut().reset(due);
                }
                return Poll::Ready(Some(Ok(frame)));
            }
            Poll::Ready(other) => return Poll::Ready(other.map(|r| r.map_err(Into::into))),
            Poll::Pending => {}
        }
        match self.timer.as_mut().poll(cx) {
            Poll::Ready(()) => Poll::Ready(Some(Err("the request body came too slowly".into()))),
            Poll::Pending => Poll::Pending,
        }
    }

    fn is_end_stream(&self) -> bool {
        self.inner.is_end_stream()
    }

    fn size_hint(&self) -> http_body::SizeHint {
        self.inner.size_hint()
    }
}

/// A connection whose writes may wait for the client at most `within` each: an answer nobody reads is given up rather
/// than holding the connection while it waits.
struct WriteWithin {
    io: tokio::net::TcpStream,
    within: std::time::Duration,
    waiting: Option<std::pin::Pin<Box<tokio::time::Sleep>>>,
}

impl WriteWithin {
    /// What a write that could not go on answers: still waiting, or past its time.
    fn stalled<T>(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<std::io::Result<T>> {
        let within = self.within;
        let timer = self.waiting.get_or_insert_with(|| Box::pin(tokio::time::sleep(within)));
        match timer.as_mut().poll(cx) {
            std::task::Poll::Ready(()) => std::task::Poll::Ready(Err(std::io::ErrorKind::TimedOut.into())),
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }

    fn went<T>(&mut self, done: std::task::Poll<std::io::Result<T>>, cx: &mut std::task::Context<'_>) -> std::task::Poll<std::io::Result<T>> {
        match done {
            std::task::Poll::Pending => self.stalled(cx),
            ready => {
                self.waiting = None;
                ready
            }
        }
    }
}

impl tokio::io::AsyncRead for WriteWithin {
    fn poll_read(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>, buf: &mut tokio::io::ReadBuf<'_>) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.io).poll_read(cx, buf)
    }
}

impl tokio::io::AsyncWrite for WriteWithin {
    fn poll_write(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>, buf: &[u8]) -> std::task::Poll<std::io::Result<usize>> {
        let done = std::pin::Pin::new(&mut self.io).poll_write(cx, buf);
        self.went(done, cx)
    }

    fn poll_write_vectored(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>, bufs: &[std::io::IoSlice<'_>]) -> std::task::Poll<std::io::Result<usize>> {
        let done = std::pin::Pin::new(&mut self.io).poll_write_vectored(cx, bufs);
        self.went(done, cx)
    }

    fn is_write_vectored(&self) -> bool {
        self.io.is_write_vectored()
    }

    fn poll_flush(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<std::io::Result<()>> {
        let done = std::pin::Pin::new(&mut self.io).poll_flush(cx);
        self.went(done, cx)
    }

    fn poll_shutdown(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.io).poll_shutdown(cx)
    }
}

/// How long a stop waits for the requests under way to finish.
const DRAIN_WITHIN: std::time::Duration = std::time::Duration::from_secs(20);

/// Serve `app` on `listener` until `shutdown`, then let the requests under way finish (at most [`DRAIN_WITHIN`]).
/// `axum::serve` sets no timer, so a connection that never finished its request head, or sat idle after one, was kept for
/// good, and enough of them left no file to open: FinStats stopped answering, reading Jellyfin and writing its database.
async fn serve_http(listener: tokio::net::TcpListener, app: axum::Router, limits: Limits, trusted: impl Fn(std::net::IpAddr) -> bool, shutdown: impl std::future::Future<Output = ()>) {
    use hyper_util::rt::{TokioIo, TokioTimer};
    let permits = Arc::new(tokio::sync::Semaphore::new(limits.connections));
    let shares: Arc<Mutex<HashMap<std::net::IpAddr, usize>>> = Default::default();
    let (stop, stopping) = tokio::sync::watch::channel(false);
    let mut builder = hyper::server::conn::http1::Builder::new();
    builder.timer(TokioTimer::new()).header_read_timeout(limits.head_within);
    tokio::pin!(shutdown);
    loop {
        let permit = tokio::select! {
            p = permits.clone().acquire_owned() => p.expect("the semaphore is never closed"),
            _ = &mut shutdown => break,
        };
        let (stream, peer) = tokio::select! {
            accepted = listener.accept() => match accepted {
                Ok(a) => a,
                Err(e) => {
                    // Out of files, most likely: wait a moment rather than spin on it.
                    tracing::warn!("could not accept a connection: {e}");
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    continue;
                }
            },
            _ = &mut shutdown => break,
        };
        // A trusted proxy carries everybody's connections; anybody else past their share is closed at once.
        let share = match trusted(peer.ip()) {
            true => None,
            false => match PeerShare::take(&shares, peer.ip(), limits.per_peer) {
                Some(share) => Some(share),
                None => continue,
            },
        };
        let app = app.clone();
        let service = hyper::service::service_fn(move |mut req: axum::http::Request<hyper::body::Incoming>| {
            req.extensions_mut().insert(axum::extract::ConnectInfo(peer));
            let req = req.map(|body| axum::body::Body::new(MinRate::new(body, limits.body_grace, limits.body_rate)));
            tower_service::Service::call(&mut app.clone(), req)
        });
        let io = WriteWithin { io: stream, within: limits.write_within, waiting: None };
        let conn = builder.serve_connection(TokioIo::new(io), service).with_upgrades();
        let mut stopping = stopping.clone();
        tokio::spawn(async move {
            let _permit = permit;
            let _share = share;
            tokio::pin!(conn);
            tokio::select! {
                _ = conn.as_mut() => return,
                _ = stopping.wait_for(|s| *s) => conn.as_mut().graceful_shutdown(),
            }
            let _ = conn.await;
        });
    }
    drop(listener);
    let _ = stop.send(true);
    // Every connection gives its permit back as it ends: once all are back, nothing is under way.
    let all = u32::try_from(limits.connections).unwrap_or(u32::MAX);
    if tokio::time::timeout(DRAIN_WITHIN, permits.acquire_many(all)).await.is_err() {
        tracing::warn!("stopping with requests still under way after {}s", DRAIN_WITHIN.as_secs());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddr;

    fn parse(args: &[&str]) -> Result<Command> {
        command(&args.iter().map(|a| a.to_string()).collect::<Vec<_>>())
    }

    /// A server on a free port with these limits, answering `ok` at `/` with the address it saw, the length of what was
    /// sent at `/body`, and 64 MB at `/big`.
    async fn listening(limits: Limits) -> SocketAddr {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let app = axum::Router::new()
            .route("/", axum::routing::get(|axum::extract::ConnectInfo(peer): axum::extract::ConnectInfo<SocketAddr>| async move { format!("ok {}", peer.ip()) }))
            .route("/body", axum::routing::post(|body: axum::body::Bytes| async move { format!("got {}", body.len()) }))
            .route("/big", axum::routing::get(|| async { vec![0u8; 64 << 20] }));
        tokio::spawn(serve_http(listener, app, limits, |_| false, std::future::pending()));
        addr
    }

    /// Limits for a test: short enough to see in a second or two.
    fn quick() -> Limits {
        let ms = std::time::Duration::from_millis;
        Limits { head_within: ms(300), connections: 8, body_grace: ms(300), body_rate: 1024, per_peer: 8, write_within: ms(300) }
    }

    /// A connection to `addr` from `from`, a loopback address of the test's choosing.
    async fn connect_from(from: &str, addr: SocketAddr) -> tokio::net::TcpStream {
        let socket = tokio::net::TcpSocket::new_v4().unwrap();
        socket.bind(format!("{from}:0").parse().unwrap()).unwrap();
        socket.connect(addr).await.unwrap()
    }

    /// Whether the server closes this connection within `within` (a read that ends, or fails).
    async fn closed_within(stream: &mut tokio::net::TcpStream, within: std::time::Duration) -> bool {
        use tokio::io::AsyncReadExt;
        let mut buf = [0u8; 512];
        loop {
            match tokio::time::timeout(within, stream.read(&mut buf)).await {
                Err(_) => return false,
                Ok(Ok(0) | Err(_)) => return true,
                Ok(Ok(_)) => {}
            }
        }
    }

    /// A connection that sends half a request head, or nothing at all, or finishes one request and then sits there, is
    /// closed once a head is overdue. Without a timer every such socket stayed open for good, and a few hundred of them
    /// took FinStats offline: no file left to open a database or reach Jellyfin with.
    #[tokio::test]
    async fn a_connection_that_never_finishes_a_request_is_closed() {
        use tokio::io::AsyncWriteExt;
        let addr = listening(quick()).await;
        let mut half = tokio::net::TcpStream::connect(addr).await.unwrap();
        half.write_all(b"G").await.unwrap();
        let mut silent = tokio::net::TcpStream::connect(addr).await.unwrap();
        let mut kept = tokio::net::TcpStream::connect(addr).await.unwrap();
        kept.write_all(b"GET / HTTP/1.1\r\nHost: x\r\n\r\n").await.unwrap();
        let wait = std::time::Duration::from_secs(3);
        assert!(closed_within(&mut half, wait).await, "half a request head kept its connection");
        assert!(closed_within(&mut silent, wait).await, "a connection that said nothing was kept");
        assert!(closed_within(&mut kept, wait).await, "an idle connection was kept alive for good");
    }

    /// A head that promises a body and then sends it a byte at a time, or not at all, is closed once the body falls behind
    /// the slowest rate any real client sends at. Only the head had a timer, and 512 such requests took every connection
    /// FinStats would serve.
    #[tokio::test]
    async fn a_request_whose_body_never_comes_is_closed() {
        use tokio::io::AsyncWriteExt;
        let addr = listening(quick()).await;
        let mut slow = tokio::net::TcpStream::connect(addr).await.unwrap();
        slow.write_all(b"POST /body HTTP/1.1\r\nHost: x\r\nContent-Length: 10\r\n\r\n{").await.unwrap();
        assert!(closed_within(&mut slow, std::time::Duration::from_secs(3)).await, "a body that never came kept its connection");
    }

    /// A body that keeps up the rate arrives whole, however long it takes past the grace.
    #[tokio::test]
    async fn a_body_that_keeps_up_arrives_whole() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let addr = listening(Limits { body_rate: 4096, ..quick() }).await;
        let mut stream = tokio::net::TcpStream::connect(addr).await.unwrap();
        stream.write_all(b"POST /body HTTP/1.1\r\nHost: x\r\nConnection: close\r\nContent-Length: 4000\r\n\r\n").await.unwrap();
        for _ in 0..8 {
            stream.write_all(&[b'a'; 500]).await.unwrap();
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        let mut buf = vec![];
        tokio::time::timeout(std::time::Duration::from_secs(3), stream.read_to_end(&mut buf)).await.unwrap().unwrap();
        assert!(String::from_utf8_lossy(&buf).ends_with("got 4000"), "{}", String::from_utf8_lossy(&buf));
    }

    /// One address holds no more than its share of the connections: past it a new one is closed at once, and another
    /// address is still served. From one host, 520 connections held the server's whole 512.
    #[tokio::test]
    async fn one_address_holds_no_more_than_its_share() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let addr = listening(Limits { head_within: std::time::Duration::from_secs(10), per_peer: 2, ..quick() }).await;
        let _a = connect_from("127.0.0.1", addr).await;
        let _b = connect_from("127.0.0.1", addr).await;
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        let mut third = connect_from("127.0.0.1", addr).await;
        assert!(closed_within(&mut third, std::time::Duration::from_secs(2)).await, "a third connection from one address was kept");
        let mut other = connect_from("127.0.0.2", addr).await;
        other.write_all(b"GET / HTTP/1.1\r\nHost: x\r\nConnection: close\r\n\r\n").await.unwrap();
        let mut buf = vec![];
        tokio::time::timeout(std::time::Duration::from_secs(3), other.read_to_end(&mut buf)).await.expect("another address waited").unwrap();
        assert!(String::from_utf8_lossy(&buf).ends_with("ok 127.0.0.2"), "{}", String::from_utf8_lossy(&buf));
    }

    /// A trusted proxy carries everybody's connections, so it is not held to one address's share.
    #[tokio::test]
    async fn a_trusted_proxy_is_not_held_to_one_share() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let app = axum::Router::new().route("/", axum::routing::get(|| async { "ok" }));
        tokio::spawn(serve_http(listener, app, Limits { head_within: std::time::Duration::from_secs(10), per_peer: 1, ..quick() }, |ip| ip.is_loopback(), std::future::pending()));
        let _a = tokio::net::TcpStream::connect(addr).await.unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        let mut second = tokio::net::TcpStream::connect(addr).await.unwrap();
        assert!(!closed_within(&mut second, std::time::Duration::from_millis(500)).await, "the proxy's second connection was closed");
    }

    /// An answer nobody reads is not written for ever: a client that stops reading is let go once a write has waited too
    /// long, rather than holding its connection while the whole answer waits.
    #[tokio::test]
    async fn an_answer_nobody_reads_is_given_up() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let addr = listening(quick()).await;
        let mut stream = tokio::net::TcpStream::connect(addr).await.unwrap();
        stream.write_all(b"GET /big HTTP/1.1\r\nHost: x\r\n\r\n").await.unwrap();
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        let mut buf = vec![0u8; 1 << 20];
        let mut total = 0usize;
        while let Ok(Ok(n)) = tokio::time::timeout(std::time::Duration::from_secs(2), stream.read(&mut buf)).await {
            if n == 0 {
                break;
            }
            total += n;
        }
        assert!(total < 64 << 20, "the whole answer was still being written after the client stopped reading");
    }

    /// Past the most connections served at once the next waits for one to close, rather than each taking a file until
    /// none is left for anything else.
    #[tokio::test]
    async fn no_more_connections_are_served_at_once_than_the_limit() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let addr = listening(Limits { head_within: std::time::Duration::from_secs(10), connections: 2, ..quick() }).await;
        let first = tokio::net::TcpStream::connect(addr).await.unwrap();
        let _second = tokio::net::TcpStream::connect(addr).await.unwrap();
        let mut third = tokio::net::TcpStream::connect(addr).await.unwrap();
        third.write_all(b"GET / HTTP/1.1\r\nHost: x\r\nConnection: close\r\n\r\n").await.unwrap();
        let mut buf = vec![];
        let early = tokio::time::timeout(std::time::Duration::from_millis(300), third.read_to_end(&mut buf)).await;
        assert!(early.is_err(), "a third connection was served beside two");
        drop(first);
        tokio::time::timeout(std::time::Duration::from_secs(3), third.read_to_end(&mut buf)).await.expect("one closed, and still the third waited").unwrap();
        let answer = String::from_utf8_lossy(&buf);
        assert!(answer.starts_with("HTTP/1.1 200") && answer.ends_with("ok 127.0.0.1"), "{answer}");
    }

    /// The live channel is a WebSocket: a connection served here must still be handed over when it asks to be upgraded.
    #[tokio::test]
    async fn a_websocket_is_still_handed_over() {
        use futures_util::{SinkExt, StreamExt};
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let app = axum::Router::new().route(
            "/ws",
            axum::routing::get(|ws: axum::extract::ws::WebSocketUpgrade| async move {
                ws.on_upgrade(|mut socket| async move {
                    if let Some(Ok(msg)) = socket.recv().await {
                        let _ = socket.send(msg).await;
                    }
                })
            }),
        );
        tokio::spawn(serve_http(listener, app, quick(), |_| false, std::future::pending()));
        let (mut ws, _) = tokio_tungstenite::connect_async(format!("ws://{addr}/ws")).await.unwrap();
        // Past the head's time limit: an upgraded connection is no longer waiting for a head.
        tokio::time::sleep(std::time::Duration::from_millis(600)).await;
        ws.send(tokio_tungstenite::tungstenite::Message::text("hello")).await.unwrap();
        let back = ws.next().await.unwrap().unwrap();
        assert_eq!(back.into_text().unwrap().as_str(), "hello");
    }

    #[test]
    fn what_was_asked_is_known_before_anything_is_opened() {
        assert!(matches!(parse(&[]).unwrap(), Command::Serve));
        assert!(matches!(parse(&["serve"]).unwrap(), Command::Serve));
        assert!(matches!(parse(&["backup"]).unwrap(), Command::Backup));
        assert!(matches!(parse(&["relink"]).unwrap(), Command::Relink));
        assert!(matches!(parse(&["import-jellystat", "a.jsonl"]).unwrap(), Command::ImportJellystat(p) if p == std::path::Path::new("a.jsonl")));
        assert!(matches!(parse(&["import-streamystats", "b.json"]).unwrap(), Command::ImportStreamystats(p) if p == std::path::Path::new("b.json")));
        assert!(matches!(parse(&["restore", "c.jsonl.gz"]).unwrap(), Command::Restore(p, false) if p == std::path::Path::new("c.jsonl.gz")));
    }

    /// The command line restores history only, as the web restore does; the settings and permissions only when asked.
    /// It always brought them back, and it is the command FinStats itself tells people to run.
    #[test]
    fn a_restore_from_the_command_line_brings_settings_back_only_when_asked() {
        assert!(matches!(parse(&["restore", "c.jsonl.gz", "--with-settings"]).unwrap(), Command::Restore(p, true) if p == std::path::Path::new("c.jsonl.gz")));
        assert!(matches!(parse(&["restore", "--with-settings", "c.jsonl.gz"]).unwrap(), Command::Restore(p, true) if p == std::path::Path::new("c.jsonl.gz")));
        assert!(parse(&["restore", "c.jsonl.gz", "--with-setting"]).is_err(), "a mistyped flag was taken for nothing");
        assert!(USAGE.contains("restore <file> [--with-settings]"));
    }

    /// `docker exec finstats finstats backup` runs as root, past the entrypoint that drops to the data folder's owner,
    /// and left a `backups/` the server could not write to. As root over somebody else's data folder, FinStats refuses
    /// and says how to run it as that user; root over a folder of root's own changes no ownership and is let be.
    #[test]
    fn root_is_refused_over_somebody_else_s_data_folder() {
        let said = as_root_over(0, 1000).expect("root over a folder of user 1000 was let through");
        assert!(said.contains("-u 1000") && said.contains("root"), "{said}");
        assert_eq!(as_root_over(1000, 1000), None);
        assert_eq!(as_root_over(1000, 0), None, "not root: the usual error about writing says the rest");
        assert_eq!(as_root_over(0, 0), None);
        assert_eq!(effective_uid_in("Name:\tfinstats\nUid:\t1000\t0\t0\t0\nGid:\t1000\t1000\t1000\t1000\n"), Some(0), "the effective id is the second");
        assert_eq!(effective_uid_in("Name:\tfinstats\n"), None);
    }

    #[test]
    fn a_command_it_cannot_carry_out_is_refused_with_the_usage() {
        let err = parse(&["bakcup"]).err().expect("an unknown command was accepted").to_string();
        assert!(err.contains("unknown command `bakcup`") && err.contains("USAGE:"), "{err}");
        for cmd in ["import-jellystat", "import-streamystats"] {
            let err = parse(&[cmd]).err().unwrap_or_else(|| panic!("{cmd} without a file was accepted")).to_string();
            assert_eq!(err, format!("usage: finstats {cmd} <file>"));
        }
        let err = parse(&["restore", "--with-settings"]).err().expect("restore without a file was accepted").to_string();
        assert_eq!(err, "usage: finstats restore <file> [--with-settings]");
    }

    #[test]
    fn the_usage_names_every_variable_finstats_reads() {
        for entry in std::fs::read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/src")).unwrap().flatten().filter(|e| e.path().extension().is_some_and(|x| x == "rs")) {
            let (file, text) = (entry.file_name().to_string_lossy().into_owned(), std::fs::read_to_string(entry.path()).unwrap());
            for (i, _) in text.match_indices("env::var(\"") {
                let name: String = text[i + 10..].chars().take_while(|c| *c != '"').collect();
                if name.starts_with("FINSTATS_") || name.starts_with("JELLYFIN_") {
                    assert!(USAGE.contains(&name), "{file} reads {name}, which the usage never mentions");
                }
            }
        }
    }
}
