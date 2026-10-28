//! Connections to the services around Jellyfin: Sonarr, Radarr and Seerr. Download clients are read through Sonarr's and
//! Radarr's queues, never connected.
//!
//! FinStats only ever *reads* from them. Everything that makes that safe lives here:
//!
//! - **Own HTTP clients that follow no redirect.** reqwest drops `Authorization` and `Cookie` when a redirect
//!   leaves the host, but not `X-Api-Key`, and a 307 replays a POST body (a password) wherever it points.
//!   A redirect is therefore an answer ("enter that address instead"), never something to follow.
//! - **Secrets stay put.** `Service` is neither `Serialize` nor `Debug`, its JSON is written by hand, keys
//!   travel in headers and bodies and never in a URL, and an error says what happened (status, kind of
//!   failure) without repeating what the other side sent.
//! - Several connections of one kind are normal (a 4K Radarr, an anime Sonarr). An id is never reused, and
//!   whatever was read from a service belongs to that id: pointing a connection at another address throws
//!   its rows away, because ids from one instance mean nothing in the next.
//!
//! Only Jellyfin administrators may add or change a connection: it is a secret plus an address FinStats will call.

use std::sync::Arc;
use std::time::Duration;

use anyhow::{Result, anyhow, bail};
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use reqwest::header::{ACCEPT, HeaderMap, HeaderValue, LOCATION};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::auth::JellyfinAdmin;
use crate::db::rusqlite::{Connection, OptionalExtension, params};
use crate::db;
use crate::i18n::t;
use crate::notify::{FormError, FormResult};
use crate::state::{ApiError, ApiResult, App};

const MAX_SERVICES: usize = 20;

// ---------------------------------------------------------------- words said later

/// A sentence kept as its key and what fills it, and said in the language of whoever reads it. A read of a service
/// mostly happens in the background, where nobody's language is known, and what went wrong is shown later, on a page,
/// to somebody who reads it in theirs.
#[derive(Clone, Debug, PartialEq)]
pub struct Words {
    key: &'static str,
    args: Vec<(&'static str, String)>,
}

impl Words {
    pub fn new(key: &'static str, args: Vec<(&'static str, String)>) -> Words {
        Words { key, args }
    }

    pub fn key(&self) -> &'static str {
        self.key
    }

    pub fn say(&self, lang: &str) -> String {
        let args: Vec<(&str, crate::i18n::Arg)> = self.args.iter().map(|(name, value)| (*name, value.as_str().into())).collect();
        crate::i18n::say(lang, self.key, &args)
    }
}

/// Said in the language of the request being answered, or the source outside one.
impl std::fmt::Display for Words {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.say(crate::i18n::current()))
    }
}

impl std::error::Error for Words {}

/// What went wrong, as it is kept to be shown: words FinStats can say in anybody's language, or a text as it was
/// written (an error from elsewhere, or one read back from the database).
#[derive(Clone, Debug, PartialEq)]
pub enum Said {
    Words(Words),
    Text(String),
}

impl Said {
    pub fn of(e: &anyhow::Error) -> Said {
        e.downcast_ref::<Words>().map_or_else(|| Said::Text(e.to_string()), |w| Said::Words(w.clone()))
    }

    pub fn say(&self, lang: &str) -> String {
        match self {
            Said::Words(w) => w.say(lang),
            Said::Text(s) => s.clone(),
        }
    }

    /// In the language of the request being answered.
    pub fn now(&self) -> String {
        self.say(crate::i18n::current())
    }

    /// In the source language: what the database keeps and a notification carries.
    pub fn source(&self) -> String {
        self.say(crate::i18n::LOCALES[0])
    }
}

impl From<&str> for Said {
    fn from(s: &str) -> Said {
        Said::Text(s.to_string())
    }
}

impl From<String> for Said {
    fn from(s: String) -> Said {
        Said::Text(s)
    }
}

/// The catalogue's `key` as an error that is said when it is shown.
pub fn words(key: &'static str, args: Vec<(&'static str, String)>) -> anyhow::Error {
    anyhow::Error::new(Words::new(key, args))
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    Sonarr,
    Radarr,
    Seerr,
}

impl Kind {
    pub const ALL: [Kind; 3] = [Kind::Sonarr, Kind::Radarr, Kind::Seerr];

    pub fn key(self) -> &'static str {
        match self {
            Kind::Sonarr => "sonarr",
            Kind::Radarr => "radarr",
            Kind::Seerr => "seerr",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Kind::Sonarr => "Sonarr",
            Kind::Radarr => "Radarr",
            Kind::Seerr => "Seerr",
        }
    }

    pub fn from_key(key: &str) -> Option<Kind> {
        Kind::ALL.into_iter().find(|k| k.key() == key)
    }

    fn example(self) -> &'static str {
        match self {
            Kind::Sonarr => "http://192.168.1.10:8989",
            Kind::Radarr => "http://192.168.1.10:7878",
            Kind::Seerr => "http://192.168.1.10:5055",
        }
    }

    fn what(self) -> String {
        t(&format!("server.services.what.{}", self.key()), &[])
    }

    pub fn is_arr(self) -> bool {
        matches!(self, Kind::Sonarr | Kind::Radarr)
    }

}

// Deliberately not `Debug` and not `Serialize`: there is no way to print or send one by accident.
pub struct Service {
    pub id: i64,
    pub kind: Kind,
    pub name: String,
    pub url: String,
    pub username: Option<String>,
    secret: String,
    pub accept_invalid_certs: bool,
    pub enabled: bool,
}

impl Service {
    pub fn secret(&self) -> &str {
        &self.secret
    }
}

/// How a connection is doing. Kept in memory (the live loop asks every few seconds); the database only
/// hears about it when it changes, so that the last known state survives a restart.
#[derive(Clone, Default, PartialEq)]
pub struct Health {
    pub version: Option<String>,
    pub last_ok_at: Option<i64>,
    /// Said in the reader's language where FinStats has the words; the database keeps the source's.
    pub last_error: Option<Said>,
}

pub struct Http {
    strict: reqwest::Client,
    /// For connections whose owner switched certificate checks off: a self-signed service on the LAN.
    lax: reqwest::Client,
    /// The same two for a personal notification destination, which may only ever reach a public host (`PublicOnly`).
    public_strict: reqwest::Client,
    public_lax: reqwest::Client,
}

impl Http {
    pub fn new() -> Self {
        let build = |lax: bool, public: bool| {
            let mut headers = HeaderMap::new();
            headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
            let mut b = reqwest::Client::builder()
                .default_headers(headers)
                .redirect(reqwest::redirect::Policy::none())
                .connect_timeout(Duration::from_secs(8))
                .timeout(Duration::from_secs(15))
                .pool_max_idle_per_host(2)
                .user_agent(concat!("finstats/", env!("CARGO_PKG_VERSION")))
                .danger_accept_invalid_certs(lax);
            if public {
                // A proxy would do its own lookup, and connect wherever that one says.
                b = b.dns_resolver(Arc::new(PublicOnly)).no_proxy();
            }
            b.build().expect("building http client")
        };
        Http { strict: build(false, false), lax: build(true, false), public_strict: build(false, true), public_lax: build(true, true) }
    }

    /// For a personal destination: a name is looked up once, refused when any address it answers is not public, and
    /// connected to at exactly what was checked. A check by one lookup and a connection by another is what a name that
    /// answers publicly first and inwards second gets past. An address written as digits is never looked up at all, so
    /// `notify::must_be_public` still holds those.
    pub fn public(&self, accept_invalid_certs: bool) -> &reqwest::Client {
        if accept_invalid_certs { &self.public_lax } else { &self.public_strict }
    }

    pub fn of(&self, svc: &Service) -> &reqwest::Client {
        self.client(svc.accept_invalid_certs)
    }

    /// The same two clients, for anything else that must not follow a redirect while holding a token:
    /// a notification destination, say.
    pub fn client(&self, accept_invalid_certs: bool) -> &reqwest::Client {
        if accept_invalid_certs { &self.lax } else { &self.strict }
    }
}

/// May a personal destination not be reached at `ip`? Inside this network, or one of the household's own public
/// addresses (`home`): FinStats counts those as home too, and from inside they loop back to the router and to whatever
/// it forwards.
pub fn refused(ip: &std::net::IpAddr, home: &[std::net::IpAddr]) -> bool {
    crate::db::is_local_ip(&ip.to_string()).unwrap_or(true) || home.contains(ip)
}

/// The household's own public addresses (`home_addresses`), kept in memory for the checks above, which run where no
/// database is at hand: a name is resolved inside reqwest.
static HOME: std::sync::RwLock<Vec<std::net::IpAddr>> = std::sync::RwLock::new(Vec::new());

pub fn home() -> Vec<std::net::IpAddr> {
    HOME.read().unwrap().clone()
}

/// Read the household's addresses again. Called wherever `home_addresses` is written (`network`).
pub fn remember_home(conn: &Connection) -> Result<()> {
    let list: Vec<String> = conn.prepare_cached("SELECT ip FROM home_addresses")?.query_map([], |r| r.get(0))?.collect::<Result<_, _>>()?;
    *HOME.write().unwrap() = list.iter().filter_map(|ip| ip.parse().ok()).collect();
    Ok(())
}

/// The resolver of `Http::public`.
struct PublicOnly;

impl reqwest::dns::Resolve for PublicOnly {
    fn resolve(&self, name: reqwest::dns::Name) -> reqwest::dns::Resolving {
        let host = name.as_str().to_string();
        Box::pin(async move {
            let found: Vec<std::net::SocketAddr> = tokio::net::lookup_host((host.as_str(), 0)).await?.collect();
            let home = home();
            if found.is_empty() || found.iter().any(|a| refused(&a.ip(), &home)) {
                return Err(crate::i18n::t("server.notify.not_public_or_unknown", &[("host", host.as_str().into())]).into());
            }
            Ok(Box::new(found.into_iter()) as reqwest::dns::Addrs)
        })
    }
}

// ---------------------------------------------------------------- addresses

/// What people type, made canonical, and nothing that could carry a secret or aim the request elsewhere:
/// no `user:password@`, no query, no fragment. A base path (`/sonarr`) stays.
pub fn clean_url(input: &str, kind: Kind) -> Result<String> {
    // Whatever scheme was typed must be http(s); a missing one is added below, and would otherwise hide another.
    let typed = input.trim().to_ascii_lowercase();
    if typed.contains("://") && !typed.starts_with("http://") && !typed.starts_with("https://") {
        bail!(t("server.services.http_only", &[]));
    }
    let what = t("server.services.like", &[("service", kind.label().into()), ("example", kind.example().into())]);
    let url = crate::jellyfin::normalize_url_of(input, &what)?;
    let parsed = reqwest::Url::parse(&url).map_err(|_| anyhow!(t("server.services.bad_url", &[])))?;
    if !matches!(parsed.scheme(), "http" | "https") || parsed.host_str().is_none() {
        bail!(t("server.services.http_only", &[]));
    }
    if !parsed.username().is_empty() || parsed.password().is_some() {
        bail!(t("server.services.no_login", &[]));
    }
    if parsed.query().is_some() || parsed.fragment().is_some() {
        bail!(t("server.services.no_query", &[]));
    }
    Ok(url)
}

/// Host, port and base path: when these change, it is another instance and its ids mean nothing here.
/// The same server: scheme, host and port. A stored secret is sent nowhere else without being typed again.
pub(crate) fn same_place(a: &str, b: &str) -> bool {
    let key = |u: &str| reqwest::Url::parse(u).ok().map(|p| (p.scheme().to_string(), p.host_str().map(str::to_ascii_lowercase), p.port_or_known_default()));
    key(a).is_some() && key(a) == key(b)
}

fn same_instance(a: &str, b: &str) -> bool {
    let key = |u: &str| reqwest::Url::parse(u).ok().map(|p| (p.host_str().map(str::to_ascii_lowercase), p.port_or_known_default(), p.path().trim_end_matches('/').to_string()));
    key(a).is_some() && key(a) == key(b)
}

// ---------------------------------------------------------------- talking

fn chain_mentions_certificate(e: &reqwest::Error) -> bool {
    let mut source: Option<&dyn std::error::Error> = Some(e);
    while let Some(s) = source {
        let text = s.to_string().to_ascii_lowercase();
        if text.contains("certificate") || text.contains("unknownissuer") || text.contains("self signed") || text.contains("self-signed") {
            return true;
        }
        source = s.source();
    }
    false
}

/// Why a request never got an answer, in words, and without the URL reqwest would print (it is in the sentence already).
pub fn explain(e: reqwest::Error, svc: &Service) -> anyhow::Error {
    let (service, url) = (svc.kind.label().to_string(), svc.url.clone());
    if chain_mentions_certificate(&e) {
        words("server.services.certificate", vec![("url", url)])
    } else if e.is_timeout() {
        words("server.services.timeout", vec![("service", service), ("url", url)])
    } else if e.is_connect() {
        words("server.services.connect", vec![("service", service), ("url", url)])
    } else if e.is_decode() {
        words("server.services.decode", vec![("service", service), ("url", url)])
    } else {
        words("server.services.unreachable", vec![("url", url), ("error", e.without_url().to_string())])
    }
}

/// An answer that is not the one asked for. Says the status and what it usually means, never the body.
pub fn refuse(status: StatusCode, headers: &HeaderMap, svc: &Service) -> anyhow::Error {
    let (service, url) = (svc.kind.label().to_string(), svc.url.clone());
    if status.is_redirection() {
        let to: Option<String> = headers.get(LOCATION).and_then(|v| v.to_str().ok()).map(|to| to.chars().filter(|c| !c.is_control()).take(200).collect());
        return match to {
            Some(to) => words("server.services.redirect", vec![("url", url), ("to", crate::downloads::redact(&to))]),
            None => words("server.services.redirect_elsewhere", vec![("url", url)]),
        };
    }
    match (status.as_u16(), ()) {
        (401 | 403, _) => words(KEY_REFUSED, vec![("service", service)]),
        (404, _) => words("server.services.not_found", vec![("url", url), ("path", svc.kind.key().to_string())]),
        _ => words("server.services.answered", vec![("service", service), ("status", status.to_string())]),
    }
}

/// The one refusal that is about the key rather than the address.
const KEY_REFUSED: &str = "server.services.key_refused";

/// `GET` on Sonarr, Radarr or Seerr. The key is a header; the query never carries it.
/// The most a service's JSON answer may be, unpacked. The largest real one (a whole Sonarr library) is a few megabytes.
pub const JSON_MAX: usize = 64 << 20;

/// An answer's body, read no further than `cap` bytes as unpacked: `None` past it. A service that went wrong (or
/// anybody between, on a connection that takes any certificate) could otherwise answer gigabytes, or a little gzip
/// that inflates into them, and every byte of it was held in memory.
pub async fn body_at_most(mut resp: reqwest::Response, cap: usize) -> reqwest::Result<Option<Vec<u8>>> {
    let mut out = Vec::new();
    while let Some(chunk) = resp.chunk().await? {
        if out.len() + chunk.len() > cap {
            return Ok(None);
        }
        out.extend_from_slice(&chunk);
    }
    Ok(Some(out))
}

pub async fn get_json(app: &App, svc: &Service, path: &str, query: &[(&str, String)]) -> Result<Value> {
    let resp = app.services_http.of(svc).get(format!("{}{path}", svc.url)).header("X-Api-Key", svc.secret()).query(query).send().await.map_err(|e| explain(e, svc))?;
    if !resp.status().is_success() {
        return Err(refuse(resp.status(), resp.headers(), svc));
    }
    let not_it = || words("server.services.decode", vec![("service", svc.kind.label().to_string()), ("url", svc.url.clone())]);
    let body = body_at_most(resp, JSON_MAX).await.map_err(|e| explain(e, svc))?.ok_or_else(not_it)?;
    serde_json::from_slice(&body).map_err(|_| not_it())
}

/// Bytes from Sonarr or Radarr (a poster). `None` for a 404 and for anything larger than `cap`.
pub async fn get_bytes(app: &App, svc: &Service, path: &str, cap: usize) -> Result<Option<Vec<u8>>> {
    let resp = app.services_http.of(svc).get(format!("{}{path}", svc.url)).header("X-Api-Key", svc.secret()).header(ACCEPT, "image/*").send().await.map_err(|e| explain(e, svc))?;
    if resp.status() == StatusCode::NOT_FOUND {
        return Ok(None);
    }
    if !resp.status().is_success() {
        return Err(refuse(resp.status(), resp.headers(), svc));
    }
    if resp.content_length().is_some_and(|n| n as usize > cap) {
        return Ok(None);
    }
    body_at_most(resp, cap).await.map_err(|e| explain(e, svc))
}

/// Does the address answer like the service it is meant to be, and does it accept the secret? → (what it is, its version)
pub async fn test(app: &App, svc: &Service) -> Result<(String, String)> {
    match svc.kind {
        Kind::Sonarr | Kind::Radarr => {
            let status = get_json(app, svc, "/api/v3/system/status", &[]).await?;
            let name = status["appName"].as_str().unwrap_or("");
            if !name.eq_ignore_ascii_case(svc.kind.label()) {
                let (url, service) = (svc.url.clone(), svc.kind.label().to_string());
                return Err(match name {
                    "" => words("server.services.answers_as_other", vec![("url", url), ("service", service)]),
                    app => words("server.services.answers_as", vec![("url", url), ("app", app.to_string()), ("service", service)]),
                });
            }
            Ok((name.to_string(), status["version"].as_str().unwrap_or("").to_string()))
        }
        Kind::Seerr => {
            // The status needs no key and says what this is; `auth/me` says whether the key is good.
            let status = get_json(app, svc, "/api/v1/status", &[]).await?;
            let Some(version) = status["version"].as_str() else { return Err(words("server.services.not_seerr", vec![("url", svc.url.clone())])) };
            get_json(app, svc, "/api/v1/auth/me", &[]).await?;
            Ok(("Seerr".into(), version.to_string()))
        }
    }
}

// ---------------------------------------------------------------- the registry

fn read(r: &crate::db::rusqlite::Row) -> crate::db::rusqlite::Result<Option<(Service, Health)>> {
    let kind: String = r.get(1)?;
    let Some(kind) = Kind::from_key(&kind) else { return Ok(None) }; // a kind from a newer FinStats
    Ok(Some((
        Service { id: r.get(0)?, kind, name: r.get(2)?, url: r.get(3)?, username: r.get(4)?, secret: r.get(5)?, accept_invalid_certs: r.get(6)?, enabled: r.get(7)? },
        Health { version: r.get(8)?, last_ok_at: r.get(9)?, last_error: r.get::<_, Option<String>>(10)?.map(Said::Text) },
    )))
}

fn load(conn: &Connection) -> Result<Vec<(Service, Health)>> {
    let mut stmt = conn.prepare("SELECT id, kind, name, url, username, secret, accept_invalid_certs, enabled, version, last_ok_at, last_error FROM services ORDER BY id")?;
    let rows = stmt.query_map([], read)?.collect::<Result<Vec<_>, _>>()?;
    Ok(rows.into_iter().flatten().collect())
}

/// Read the connections into memory: at start-up and after every change. Loops ask memory, not the database.
pub async fn reload(app: &App) -> Result<()> {
    let rows = app.db.call(|c| load(c)).await?;
    // One lock at a time: the outbound page holds the connections while it asks for each one's health, and
    // holding the health here while waiting for the connections was the other half of a deadlock.
    {
        let mut health = app.service_health.write().unwrap();
        health.retain(|id, _| rows.iter().any(|(s, _)| s.id == *id));
        for (svc, stored) in &rows {
            health.entry(svc.id).or_insert_with(|| stored.clone());
        }
    }
    *app.services.write().unwrap() = Arc::new(rows.into_iter().map(|(s, _)| Arc::new(s)).collect());
    Ok(())
}

pub fn all(app: &App) -> Arc<Vec<Arc<Service>>> {
    app.services.read().unwrap().clone()
}

pub fn enabled(app: &App, want: impl Fn(Kind) -> bool) -> Vec<Arc<Service>> {
    all(app).iter().filter(|s| s.enabled && want(s.kind)).cloned().collect()
}

/// Note how a request to a service went. Touches the database only when the picture changed.
pub async fn record(app: &App, id: i64, outcome: std::result::Result<Option<String>, Said>) {
    let now = db::now();
    let (before, after) = {
        let mut map = app.service_health.write().unwrap();
        let h = map.entry(id).or_default();
        let before = h.clone();
        match outcome {
            Ok(version) => {
                h.last_ok_at = Some(now);
                h.last_error = None;
                if version.is_some() {
                    h.version = version;
                }
            }
            Err(message) => h.last_error = Some(message),
        }
        (before, h.clone())
    };
    let error_now = after.last_error.clone();
    // Every success moves `last_ok_at`; that alone is not worth a write more than once in a while.
    let changed = before.last_error != after.last_error || before.version != after.version || after.last_ok_at.unwrap_or(0) - before.last_ok_at.unwrap_or(0) >= 900;
    if changed {
        let error = after.last_error.as_ref().map(Said::source);
        let _ = app.db.call(move |c| Ok(c.execute("UPDATE services SET version = ?1, last_ok_at = ?2, last_error = ?3 WHERE id = ?4", params![after.version, after.last_ok_at, error, id])?)).await;
    }
    // Going quiet, and coming back, are each worth one message, which goes outside FinStats in its own words.
    if before.last_error.is_some() != error_now.is_some()
        && let Some(svc) = all(app).iter().find(|s| s.id == id)
    {
        crate::notify::service_state(app, &svc.name, svc.kind.label(), error_now.as_ref().map(Said::source).as_deref()).await;
    }
}

/// The tasks that read from these services. `sync::spawn` has its own list, and insists on a Jellyfin connection.
pub const TASKS: [&str; 3] = ["sync_upcoming", "sync_requests", "sync_grabs"];

/// Runs a task unless it is already running (or is not one of ours). Returns false in that case.
pub fn spawn(app: &App, id: &str) -> bool {
    spawn_within(app, id, None)
}

/// `spawn`, stopped once it has run for `limit`: the time limit of the trigger that started it.
pub fn spawn_within(app: &App, id: &str, limit: Option<std::time::Duration>) -> bool {
    let Some(id) = TASKS.into_iter().find(|t| *t == id) else { return false };
    if !app.tasks.try_start(id, &t("server.services.starting", &[])) {
        return false;
    }
    let app = app.clone();
    // Run by hand, the task reports in the words of whoever pressed the button.
    tokio::spawn(crate::i18n::carry(async move {
        let work = app.clone();
        // `caught` spawns the run again, which would leave the language behind, so it is carried in there too.
        let outcome = crate::schedule::caught(crate::i18n::carry(async move { let app = work; crate::schedule::within(limit, async { match id {
            "sync_upcoming" => crate::arr::sync_upcoming(&app).await,
            "sync_requests" => crate::seerr::sync_requests(&app).await,
            "sync_grabs" => crate::arr::sync_grabs(&app).await,
            other => Err(anyhow!("unknown task {other}")),
        } })
        .await }))
        .await;
        if let Err(e) = &outcome {
            crate::notify::task_failed(&app, id, &format!("{e:#}")).await;
        }
        app.tasks.finish(id, outcome.map(|m| (m, None)));
    }));
    true
}

/// With the other small, regular jobs: is every connection still answering? This is what keeps the status
/// under Settings → Connections true for a service nothing else has asked lately.
pub async fn check_all(app: &App) {
    for svc in enabled(app, |_| true) {
        let outcome = test(app, &svc).await;
        if let Err(e) = &outcome {
            tracing::debug!("{} ({}) is not answering: {e}", svc.name, svc.kind.label());
        }
        record(app, svc.id, outcome.map(|(_, version)| Some(version)).map_err(|e| Said::of(&e))).await;
    }
}

/// What the UI may build on. Says nothing about which services, only which kinds of page have something behind them.
pub fn features(app: &App) -> Value {
    let list = all(app);
    let has = |want: fn(Kind) -> bool| list.iter().any(|s| s.enabled && want(s.kind));
    json!({
        "upcoming": has(Kind::is_arr), "requests": has(|k| k == Kind::Seerr), "downloads": has(Kind::is_arr),
        // Not a connection, but the same question: is there anything behind the page. Settings offers
        // "Public profile" only once an administrator has allowed them.
        "public_profiles": app.settings().public_profiles,
        // Always there from 2.1: said so that a page on an older server offers no watchlist it cannot keep.
        "watchlist": true,
    })
}

// ---------------------------------------------------------------- API (Jellyfin administrators)

fn service_json(svc: &Service, health: Option<&Health>) -> Value {
    let h = health.cloned().unwrap_or_default();
    json!({
        "id": svc.id, "kind": svc.kind.key(), "label": svc.kind.label(), "name": svc.name, "url": svc.url,
        "has_secret": !svc.secret.is_empty(), "accept_invalid_certs": svc.accept_invalid_certs, "enabled": svc.enabled,
        "version": h.version, "last_ok_at": h.last_ok_at, "last_error": h.last_error.as_ref().map(Said::now),
    })
}

fn list_json(app: &App) -> Value {
    let health = app.service_health.read().unwrap();
    let services: Vec<Value> = all(app).iter().map(|s| service_json(s, health.get(&s.id))).collect();
    let kinds: Vec<Value> = Kind::ALL
        .into_iter()
        .map(|k| json!({ "key": k.key(), "label": k.label(), "what": k.what(), "example": k.example() }))
        .collect();
    json!({ "services": services, "kinds": kinds })
}

// No `Debug`: it holds a secret on its way in.
#[derive(Deserialize)]
pub struct ServiceBody {
    /// For a test while editing: use what is stored for whatever is left out.
    id: Option<i64>,
    kind: Option<String>,
    name: Option<String>,
    url: Option<String>,
    secret: Option<String>,
    accept_invalid_certs: Option<bool>,
    enabled: Option<bool>,
}

/// The connection a request describes: its fields where given, the stored ones otherwise.
fn describe(body: ServiceBody, stored: Option<&Service>, taken: &[String]) -> std::result::Result<Service, FormError> {
    let kind = match (stored, body.kind.as_deref()) {
        (Some(s), _) => s.kind, // a connection never changes its kind
        (None, Some(k)) => Kind::from_key(k).ok_or_else(|| ApiError::bad_request(t("server.services.unknown_kind", &[("kind", k.into())])))?,
        (None, None) => return Err(ApiError::bad_request(t("server.services.say_kind", &[])).into()),
    };
    let url = match (body.url.as_deref(), stored) {
        (Some(u), _) => clean_url(u, kind).map_err(|e| FormError::at("url", format!("{e}")))?,
        (None, Some(s)) => s.url.clone(),
        (None, None) => return Err(FormError::at("url", t("server.services.enter_address", &[("service", kind.label().into())]))),
    };
    let secret = match (body.secret.filter(|s| !s.is_empty()), stored) {
        (Some(s), _) => s,
        // A stored key goes only where it was saved for: pointed at another server, it is typed again.
        (None, Some(s)) if !same_place(&url, &s.url) => return Err(FormError::at("secret", t("server.services.key_again", &[]))),
        (None, Some(s)) => s.secret.clone(),
        (None, None) => return Err(FormError::at("secret", t("server.services.enter_key", &[]))),
    };
    if secret.len() > 512 || secret.chars().any(char::is_control) {
        return Err(FormError::at("secret", t("server.services.not_a_key", &[])));
    }
    // Nothing FinStats connects to needs a user name any more; the column stays for databases that have one.
    let username = stored.and_then(|s| s.username.clone());
    let wanted: String = body.name.as_deref().map(str::trim).filter(|n| !n.is_empty()).map(|n| n.chars().filter(|c| !c.is_control()).take(60).collect()).unwrap_or_default();
    let name = if !wanted.is_empty() {
        wanted
    } else if let Some(s) = stored {
        s.name.clone()
    } else {
        // "Radarr", then "Radarr 2": several of a kind are normal.
        (1..).map(|n| if n == 1 { kind.label().to_string() } else { format!("{} {n}", kind.label()) }).find(|n| !taken.contains(n)).unwrap_or_default()
    };
    Ok(Service {
        id: stored.map(|s| s.id).unwrap_or(0),
        kind,
        name,
        url,
        username,
        secret,
        accept_invalid_certs: body.accept_invalid_certs.or(stored.map(|s| s.accept_invalid_certs)).unwrap_or(false),
        enabled: body.enabled.or(stored.map(|s| s.enabled)).unwrap_or(true),
    })
}

fn find(app: &App, id: i64) -> std::result::Result<Arc<Service>, ApiError> {
    all(app).iter().find(|s| s.id == id).cloned().ok_or_else(|| ApiError::not_found("Connection"))
}

fn names(app: &App) -> Vec<String> {
    all(app).iter().map(|s| s.name.clone()).collect()
}

/// The service did not answer as it should: a refused key is about the key, anything else about the address.
fn unreachable_service(e: anyhow::Error) -> FormError {
    let field = if e.downcast_ref::<Words>().is_some_and(|w| w.key() == KEY_REFUSED) { "secret" } else { "url" };
    FormError { error: ApiError::new(StatusCode::BAD_GATEWAY, format!("{e}")), field: Some(field) }
}

pub async fn list(State(app): State<App>, JellyfinAdmin(_): JellyfinAdmin) -> ApiResult {
    Ok(Json(list_json(&app)))
}

pub async fn test_connection(State(app): State<App>, JellyfinAdmin(_): JellyfinAdmin, Json(body): Json<ServiceBody>) -> FormResult {
    let stored = match body.id {
        Some(id) => Some(find(&app, id)?),
        None => None,
    };
    let svc = describe(body, stored.as_deref(), &[])?;
    let (what, version) = test(&app, &svc).await.map_err(unreachable_service)?;
    Ok(Json(json!({ "ok": true, "app": what, "version": version })))
}

pub async fn create(State(app): State<App>, JellyfinAdmin(user): JellyfinAdmin, Json(body): Json<ServiceBody>) -> FormResult {
    if all(&app).len() >= MAX_SERVICES {
        return Err(ApiError::bad_request(t("server.services.max", &[("count", MAX_SERVICES.into())])).into());
    }
    let mut body = body;
    body.id = None;
    let svc = describe(body, None, &names(&app))?;
    // Only what answers is saved: a typo in a key is found now, not in a log next week.
    let (_, version) = test(&app, &svc).await.map_err(unreachable_service)?;
    let now = db::now();
    let (kind, name) = (svc.kind.key(), svc.name.clone());
    app.db
        .call(move |c| {
            Ok(c.execute(
                "INSERT INTO services(kind, name, url, username, secret, accept_invalid_certs, enabled, created_at, version, last_ok_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?8)",
                params![svc.kind.key(), svc.name, svc.url, svc.username, svc.secret, svc.accept_invalid_certs, svc.enabled, now, version],
            )?)
        })
        .await?;
    // Kind and name, never the address or the key.
    crate::audit::record(&app, crate::audit::Entry::new("service_added", crate::audit::Actor::from(&user)).detail(json!({ "kind": kind, "name": name })));
    changed(&app).await?;
    Ok(Json(list_json(&app)))
}

pub async fn update(State(app): State<App>, JellyfinAdmin(user): JellyfinAdmin, Path(id): Path<i64>, Json(body): Json<ServiceBody>) -> FormResult {
    let stored = find(&app, id)?;
    let taken: Vec<String> = names(&app).into_iter().filter(|n| *n != stored.name).collect();
    let svc = describe(body, Some(&stored), &taken)?;
    let reconnect = svc.url != stored.url || svc.secret != stored.secret || svc.username != stored.username || svc.accept_invalid_certs != stored.accept_invalid_certs;
    let moved = !same_instance(&svc.url, &stored.url);
    let version = if reconnect && svc.enabled { Some(test(&app, &svc).await.map_err(unreachable_service)?.1) } else { None };
    let now = db::now();
    crate::audit::record(&app, crate::audit::Entry::new("service_changed", crate::audit::Actor::from(&user)).target(id.to_string()).detail(json!({ "kind": svc.kind.key(), "name": svc.name, "reconnected": reconnect, "moved": moved })));
    app.db
        .call(move |c| {
            let tx = c.transaction()?;
            tx.execute(
                "UPDATE services SET name = ?1, url = ?2, username = ?3, secret = ?4, accept_invalid_certs = ?5, enabled = ?6 WHERE id = ?7",
                params![svc.name, svc.url, svc.username, svc.secret, svc.accept_invalid_certs, svc.enabled, id],
            )?;
            if let Some(v) = &version {
                tx.execute("UPDATE services SET version = ?1, last_ok_at = ?2, last_error = NULL WHERE id = ?3", params![v, now, id])?;
            }
            if moved {
                forget(&tx, id)?;
            }
            tx.commit()?;
            Ok(())
        })
        .await?;
    app.service_health.write().unwrap().remove(&id);
    if moved {
        forget_posters(&app.data_dir.join("cache").join("img"), id);
    }
    changed(&app).await?;
    Ok(Json(list_json(&app)))
}

pub async fn remove(State(app): State<App>, JellyfinAdmin(user): JellyfinAdmin, Path(id): Path<i64>) -> ApiResult {
    let stored = find(&app, id)?;
    crate::audit::record(&app, crate::audit::Entry::new("service_removed", crate::audit::Actor::from(&user)).target(id.to_string()).detail(json!({ "kind": stored.kind.key(), "name": stored.name })));
    // Whatever was read from it goes with it (ON DELETE CASCADE).
    app.db.call(move |c| Ok(c.execute("DELETE FROM services WHERE id = ?1", [id])?)).await?;
    changed(&app).await?;
    Ok(Json(list_json(&app)))
}

/// Everything read from a connection, thrown away: it now points at another instance.
fn forget(conn: &Connection, id: i64) -> Result<()> {
    for table in CHILD_TABLES {
        let exists: Option<i64> = conn.query_row("SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1", [table], |r| r.get(0)).optional()?;
        if exists.is_some() {
            conn.execute(&format!("DELETE FROM {table} WHERE service_id = ?1"), [id])?;
        }
    }
    conn.execute("DELETE FROM settings WHERE key LIKE ?1", [format!("service:{id}:%")])?;
    Ok(())
}

/// The posters cached for a connection (`api::arr_image` names them `arr-{id}-{media}-{width}`), thrown away with the rest
/// when it points at another instance: kept, they were the old instance's pictures under the new one's media ids.
fn forget_posters(img_dir: &std::path::Path, id: i64) -> usize {
    let prefix = format!("arr-{id}-");
    let Ok(entries) = std::fs::read_dir(img_dir) else { return 0 };
    entries.flatten().filter(|e| e.file_name().to_str().is_some_and(|n| n.starts_with(&prefix))).filter(|e| std::fs::remove_file(e.path()).is_ok()).count()
}

/// Tables whose rows belong to one connection (`service_id … REFERENCES services ON DELETE CASCADE`).
const CHILD_TABLES: [&str; 3] = ["upcoming", "requests", "grabs"];

async fn changed(app: &App) -> Result<()> {
    reload(app).await?;
    app.wake.notify_waiters();
    app.downloads_wake.notify_waiters();
    Ok(())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// An answer is read no further than the most it may be, counted after it is unpacked: a few kilobytes of gzip from a
    /// service that went wrong (or anybody between, on a connection that takes any certificate) inflate into gigabytes.
    #[tokio::test]
    async fn an_answer_is_read_no_further_than_the_most_it_may_be() {
        use std::io::Write;
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let mut enc = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::best());
        enc.write_all(&vec![b' '; 8 << 20]).unwrap();
        let bomb = enc.finish().unwrap();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(async move {
            while let Ok((mut sock, _)) = listener.accept().await {
                let bomb = bomb.clone();
                tokio::spawn(async move {
                    let mut buf = [0u8; 2048];
                    let _ = sock.read(&mut buf).await;
                    let head = format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Encoding: gzip\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", bomb.len());
                    let _ = sock.write_all(head.as_bytes()).await;
                    let _ = sock.write_all(&bomb).await;
                });
            }
        });
        let http = Http::new();
        let get = || http.client(false).get(format!("http://127.0.0.1:{port}/")).send();
        assert!(body_at_most(get().await.unwrap(), 1 << 20).await.unwrap().is_none(), "eight megabytes read in full against a limit of one");
        assert_eq!(body_at_most(get().await.unwrap(), 8 << 20).await.unwrap().map(|b| b.len()), Some(8 << 20));
    }

    /// A personal destination's name is looked up once, and that one answer is both what is checked and what is connected
    /// to: checked by one lookup and connected by another, a name that answers publicly first and inwards second reached the LAN.
    #[tokio::test]
    async fn a_personal_destination_never_connects_to_a_name_that_answers_inwards() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let reached = tokio::spawn(async move { tokio::time::timeout(Duration::from_secs(2), listener.accept()).await.is_ok() });
        let http = Http::new();
        assert!(http.public(false).post(format!("http://localhost:{port}/")).send().await.is_err());
        assert!(!reached.await.unwrap(), "the request reached a loopback address");
        // The ordinary client, an administrator's, still may.
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let _ = http.client(false).post(format!("http://localhost:{port}/")).timeout(Duration::from_millis(300)).send().await;
        assert!(tokio::time::timeout(Duration::from_secs(1), listener.accept()).await.is_ok());
    }

    pub(crate) fn service(kind: Kind, url: &str) -> Service {
        Service { id: 1, kind, name: kind.label().into(), url: url.into(), username: None, secret: "hunter2-secret-key".into(), accept_invalid_certs: false, enabled: true }
    }

    /// A connection pointed at another instance keeps its id, so the posters cached for it were the old instance's
    /// pictures under the new one's media ids, for a week. They go with what else was read from it.
    #[test]
    fn a_moved_connection_leaves_no_poster_of_the_old_instance() {
        let dir = std::env::temp_dir().join(format!("finstats-arrposters-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        for name in ["arr-1-12-250", "arr-1-7-500", "arr-12-1-250", "item-abc-300"] {
            std::fs::write(dir.join(name), b"x").unwrap();
        }
        assert_eq!(forget_posters(&dir, 1), 2);
        let mut left: Vec<String> = std::fs::read_dir(&dir).unwrap().flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect();
        left.sort();
        assert_eq!(left, ["arr-12-1-250", "item-abc-300"]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn an_address_keeps_its_base_path_and_carries_nothing_else() {
        assert_eq!(clean_url("192.168.1.10:8989/", Kind::Sonarr).unwrap(), "http://192.168.1.10:8989");
        assert_eq!(clean_url(" https://media.example/sonarr/ ", Kind::Sonarr).unwrap(), "https://media.example/sonarr");
        assert_eq!(clean_url("http://nas:7878/web/", Kind::Radarr).unwrap(), "http://nas:7878");
        for bad in ["http://admin:secret@nas:8989", "http://nas:8989/?apikey=abc", "http://nas:8989/#x", "ftp://nas", "file:///etc/passwd", ""] {
            assert!(clean_url(bad, Kind::Sonarr).is_err(), "{bad} was accepted");
        }
    }

    #[test]
    fn another_host_port_or_base_path_is_another_instance() {
        assert!(same_instance("http://nas:8989", "http://NAS:8989/"));
        assert!(same_instance("https://media.example", "https://media.example:443"));
        assert!(!same_instance("http://nas:8989", "http://nas:8990"));
        assert!(!same_instance("http://nas/sonarr", "http://nas/sonarr4k"));
        assert!(!same_instance("http://nas:8989", "http://other:8989"));
    }

    #[test]
    fn what_the_ui_gets_never_contains_the_secret() {
        let svc = service(Kind::Radarr, "http://nas:7878");
        let text = service_json(&svc, None).to_string();
        assert!(!text.contains("hunter2"), "{text}");
        assert!(text.contains("\"has_secret\":true"));
    }

    #[test]
    fn an_answer_that_is_not_ours_is_explained_without_repeating_it() {
        let svc = service(Kind::Sonarr, "http://nas:8989");
        let mut headers = HeaderMap::new();
        headers.insert(LOCATION, HeaderValue::from_static("/login?returnUrl=%2F"));
        let text = refuse(StatusCode::FOUND, &headers, &svc).to_string();
        assert!(text.contains("redirect to /login") && text.contains("follows no redirects"), "{text}");
        assert!(refuse(StatusCode::UNAUTHORIZED, &HeaderMap::new(), &svc).to_string().contains("refused the API key"));
        assert!(refuse(StatusCode::NOT_FOUND, &HeaderMap::new(), &svc).to_string().contains("/sonarr"));
        for e in [StatusCode::FOUND, StatusCode::UNAUTHORIZED, StatusCode::BAD_GATEWAY] {
            assert!(!refuse(e, &headers, &svc).to_string().contains("hunter2"));
        }
    }

    /// A read in the background knows nobody's language; what went wrong is kept as words and said in the language of
    /// whoever looks, while the database and a notification keep the source's.
    #[test]
    fn a_refusal_is_kept_as_words_and_said_in_the_readers_language() {
        let svc = service(Kind::Sonarr, "http://nas:8989");
        let said = Said::of(&refuse(StatusCode::UNAUTHORIZED, &HeaderMap::new(), &svc));
        assert_eq!(said.source(), "Sonarr refused the API key");
        assert_eq!(said.say("nb-NO"), "Sonarr avviste API-nøkkelen");
        assert_eq!(Said::of(&anyhow!("disk full")).say("nn-NO"), "disk full", "words FinStats does not have stay as they were");
        assert_eq!(unreachable_service(refuse(StatusCode::UNAUTHORIZED, &HeaderMap::new(), &svc)).field, Some("secret"), "a refused key is the key's fault");
        assert_eq!(unreachable_service(refuse(StatusCode::NOT_FOUND, &HeaderMap::new(), &svc)).field, Some("url"));
    }

    /// A stored key goes only where it was saved for: a test or an edit naming another host took the stored key there,
    /// so a leaked administrator key read out every Sonarr, Radarr and Seerr key by pointing them at its own server.
    #[test]
    fn a_stored_key_is_not_sent_to_a_new_address() {
        let stored = service(Kind::Sonarr, "http://192.168.1.10:8989");
        let body = |url: &str, secret: Option<&str>| ServiceBody { id: Some(1), kind: None, name: None, url: Some(url.into()), secret: secret.map(str::to_string), accept_invalid_certs: None, enabled: None };
        for elsewhere in ["https://attacker.example", "http://192.168.1.10:9999", "https://192.168.1.10:8989"] {
            assert_eq!(describe(body(elsewhere, None), Some(&stored), &[]).err().expect("the stored key went to a new address").field, Some("secret"), "{elsewhere}");
            assert!(describe(body(elsewhere, Some("a-new-key")), Some(&stored), &[]).is_ok(), "a key typed again goes anywhere");
        }
        // The same place, under another base path, is still where the key was saved for.
        assert_eq!(describe(body("http://192.168.1.10:8989/sonarr", None), Some(&stored), &[]).ok().unwrap().secret(), "hunter2-secret-key");
    }

    #[test]
    fn a_second_connection_of_a_kind_gets_its_own_name() {
        let body = |name: Option<&str>| ServiceBody { id: None, kind: Some("radarr".into()), name: name.map(str::to_string), url: Some("nas:7878".into()), secret: Some("k".into()), accept_invalid_certs: None, enabled: None };
        let first = describe(body(None), None, &[]).ok().unwrap();
        assert_eq!((first.name.as_str(), first.enabled, first.accept_invalid_certs), ("Radarr", true, false));
        assert_eq!(describe(body(None), None, &["Radarr".into()]).ok().unwrap().name, "Radarr 2");
        assert_eq!(describe(body(Some("  4K  ")), None, &[]).ok().unwrap().name, "4K");
        // Editing without retyping the key keeps the key, and the kind never changes.
        let edited = describe(ServiceBody { id: Some(1), kind: Some("sonarr".into()), name: None, url: None, secret: None, accept_invalid_certs: Some(true), enabled: None }, Some(&first), &[]).ok().unwrap();
        assert!(edited.kind == Kind::Radarr && edited.secret() == "k" && edited.accept_invalid_certs);
        assert!(describe(ServiceBody { id: None, kind: Some("radarr".into()), name: None, url: Some("nas".into()), secret: None, accept_invalid_certs: None, enabled: None }, None, &[]).is_err());
    }

    /// The reason this module has its own client: a redirect must not be followed, or the key goes along.
    #[tokio::test]
    async fn a_redirect_is_reported_and_the_key_goes_nowhere_else() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let elsewhere = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let target = elsewhere.local_addr().unwrap();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let (mut sock, _) = listener.accept().await.unwrap();
            let mut buf = [0u8; 2048];
            let _ = sock.read(&mut buf).await;
            let _ = sock.write_all(format!("HTTP/1.1 307 Temporary Redirect\r\nLocation: http://{target}/api/v3/system/status\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").as_bytes()).await;
        });
        let svc = service(Kind::Sonarr, &format!("http://{addr}"));
        let http = Http::new();
        let resp = http.of(&svc).get(format!("{}/api/v3/system/status", svc.url)).header("X-Api-Key", svc.secret()).send().await.unwrap();
        assert_eq!(resp.status(), StatusCode::TEMPORARY_REDIRECT);
        assert!(refuse(resp.status(), resp.headers(), &svc).to_string().contains("redirect"));
        // Nobody ever knocks on the other door.
        assert!(tokio::time::timeout(Duration::from_millis(300), elsewhere.accept()).await.is_err(), "the redirect was followed");
    }
    #[test]
    fn a_server_says_it_keeps_watchlists_so_an_older_one_shows_no_toggle() {
        assert_eq!(features(&crate::state::test_app())["watchlist"], serde_json::json!(true));
    }

    /// The outbound page holds the connections and asks for each one's health; a reload must never hold the health
    /// while it waits for the connections, or the two wait for each other for good.
    #[test]
    fn a_reload_never_holds_the_health_while_it_waits_for_the_connections() {
        let app = crate::state::test_app();
        let other = app.clone();
        let reading = app.services.read().unwrap();
        let worker = std::thread::spawn(move || tokio::runtime::Runtime::new().unwrap().block_on(reload(&other)).unwrap());
        std::thread::sleep(Duration::from_millis(300));
        let free = app.service_health.try_read().is_ok();
        drop(reading);
        worker.join().unwrap();
        assert!(free, "the reload sat on the health while it waited for the connections");
    }
}
