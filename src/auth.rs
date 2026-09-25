//! Sign-in with Jellyfin credentials.
//!
//! finstats never stores passwords. A login is forwarded to Jellyfin's
//! `AuthenticateByName`; on success we mint our own opaque session token (stored
//! hashed) and immediately end the Jellyfin session the check created.

use std::net::{IpAddr, SocketAddr};

use axum::extract::{ConnectInfo, FromRequestParts, State};
use axum::http::header::{AUTHORIZATION, COOKIE, SET_COOKIE};
use axum::http::request::Parts;
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use rand::RngCore;
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::audit::{self, Actor};
use crate::db::rusqlite::{Connection, OptionalExtension, params};
use crate::db::{self};
use crate::jellyfin::{self, AuthError};
use crate::state::{ApiError, ApiResult, App, JfConfig};

const COOKIE_NAME: &str = "finstats_session";
const SESSION_TTL_S: i64 = 30 * 86_400;
/// An API key is `fs_` and 64 hex characters: recognisable in a log, greppable in a leak, and a token
/// of any other shape never reaches the database.
pub const KEY_PREFIX: &str = "fs_";
const KEY_LEN: usize = 3 + 64;
/// How often a key's last use is written down: once a minute, not once a request.
const KEY_TOUCH_S: i64 = 60;
const MAX_ATTEMPTS: u32 = 10;
const ATTEMPT_WINDOW_S: i64 = 300;

/// Permission keys an administrator can grant. `sign_in` lets one user in while sign-in
/// for everyone is off; the rest widen what a signed-in user may see or do.
pub const SIGN_IN: &str = "sign_in";
pub const GRANTABLE: [&str; 7] = [SIGN_IN, "see_everyone", "see_network", "see_server", "see_downloads", "notify", "manage"];

/// What this request may see and do. Jellyfin administrators always hold everything.
/// The year recap is not covered: it stays personal whatever is granted.
#[derive(Clone, Copy, Debug, Default, PartialEq, serde::Serialize)]
pub struct Perms {
    /// Other people's activity and statistics, the user list, every live stream.
    pub see_everyone: bool,
    /// IP addresses, device ids, local/remote.
    pub see_network: bool,
    pub see_downloads: bool,
    /// The Server page, the server log, failed sign-ins and file paths.
    pub see_server: bool,
    /// May have notification destinations of their own, and be sent what they are allowed to see.
    pub notify: bool,
    /// Settings, tasks, imports and deleting plays.
    pub manage: bool,
}

impl Perms {
    pub const ALL: Perms = Perms { see_everyone: true, see_network: true, see_server: true, see_downloads: true, notify: true, manage: true };

    pub fn from_keys<'a>(keys: impl IntoIterator<Item = &'a str>) -> Self {
        let mut p = Perms::default();
        for k in keys {
            match k {
                "see_everyone" => p.see_everyone = true,
                "see_network" => p.see_network = true,
                "see_downloads" => p.see_downloads = true,
                "see_server" => p.see_server = true,
                "notify" => p.notify = true,
                "manage" => p.manage = true,
                _ => {}
            }
        }
        p
    }
}

#[derive(Clone, Debug)]
pub struct AuthUser {
    pub id: String,
    pub name: String,
    /// A Jellyfin administrator. Only they can change who is allowed what.
    pub is_admin: bool,
    pub perms: Perms,
    /// What this request proved itself with: the cookie a browser carries, or a key a script sent.
    pub credential: Credential,
    /// Where the request came from, for the audit log.
    pub ip: Option<IpAddr>,
}

impl AuthUser {
    /// The key this request came through, if it did.
    pub fn key_id(&self) -> Option<i64> {
        match self.credential {
            Credential::Key { id, .. } => Some(id),
            Credential::Session => None,
        }
    }
}

/// What a key may open: everything its maker may, or the calendar feed alone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyScope {
    Full,
    Calendar,
}

impl KeyScope {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "full" => Some(KeyScope::Full),
            "calendar" => Some(KeyScope::Calendar),
            _ => None,
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            KeyScope::Full => "full",
            KeyScope::Calendar => "calendar",
        }
    }
}

/// How a request proved itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Credential {
    #[default]
    Session,
    Key { id: i64, scope: KeyScope },
}

/// What the headers presented, before anything is looked up.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Presented {
    Bearer(String),
    Cookie(String),
}

/// The feed's own extractor: a key in the query string or the header, of any scope, and never the
/// cookie — a subscribed calendar cannot send one, and a browser must not open a feed by accident.
pub struct CalendarKey(pub AuthUser);

/// May change settings, run tasks, import and delete plays.
pub struct Manager(#[allow(dead_code)] pub AuthUser);
/// May see the server page and the server log.
pub struct ServerViewer(#[allow(dead_code)] pub AuthUser);
/// A Jellyfin administrator: the only one who may hand out permissions.
/// May see what is downloading: the queue, speeds, torrent names and which client.
pub struct DownloadsViewer(#[allow(dead_code)] pub AuthUser);

pub struct JellyfinAdmin(#[allow(dead_code)] pub AuthUser);

/// A non-admin's own grants, as stored. Administrators never have a row.
pub fn stored_grants(conn: &db::rusqlite::Connection, user_id: &str) -> anyhow::Result<Vec<String>> {
    let raw: Option<String> = conn.query_row("SELECT permissions FROM user_permissions WHERE user_id = ?1", [user_id], |r| r.get(0)).optional()?;
    Ok(raw.and_then(|r| serde_json::from_str(&r).ok()).unwrap_or_default())
}

/// `None` = this user may not sign in at all.
pub(crate) fn effective(is_admin: bool, grants: &[String], settings: &crate::state::Settings) -> Option<Perms> {
    if is_admin {
        return Some(Perms::ALL);
    }
    if !settings.allow_user_login && !grants.iter().any(|g| g == SIGN_IN) {
        return None;
    }
    Some(Perms::from_keys(grants.iter().chain(settings.default_permissions.iter()).map(String::as_str)))
}

fn hash_token(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}

fn cookie_token(headers: &HeaderMap) -> Option<String> {
    headers.get_all(COOKIE).iter().filter_map(|v| v.to_str().ok()).flat_map(|v| v.split(';')).find_map(|kv| {
        let (k, v) = kv.trim().split_once('=')?;
        (k == COOKIE_NAME && !v.is_empty()).then(|| v.to_string())
    })
}

/// `Authorization: Bearer <key>`: `None` when there is no such header at all (the cookie may be there),
/// `Some(Err)` when there is one that is not a bearer token — which is never quietly ignored.
pub fn bearer_token(headers: &HeaderMap) -> Option<Result<String, ApiError>> {
    let raw = headers.get(AUTHORIZATION)?.to_str().ok().map(str::trim).unwrap_or_default();
    let bad = || ApiError::new(StatusCode::UNAUTHORIZED, "Use `Authorization: Bearer <key>`");
    let Some((scheme, rest)) = raw.split_once(char::is_whitespace) else { return Some(Err(bad())) };
    let token = rest.trim();
    Some(if scheme.eq_ignore_ascii_case("bearer") && !token.is_empty() { Ok(token.to_string()) } else { Err(bad()) })
}

/// The header wins over the cookie: it is an explicit act of this request where a cookie is ambient,
/// so a script sending a revoked key gets its 401 rather than quietly succeeding on a browser's session.
pub fn pick_credential(headers: &HeaderMap) -> Result<Presented, ApiError> {
    match bearer_token(headers) {
        Some(t) => Ok(Presented::Bearer(t?)),
        None => cookie_token(headers).map(Presented::Cookie).ok_or_else(|| ApiError::new(StatusCode::UNAUTHORIZED, "Sign in to continue")),
    }
}

pub fn looks_like_key(s: &str) -> bool {
    s.len() == KEY_LEN && s.starts_with(KEY_PREFIX) && s[KEY_PREFIX.len()..].bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// Whether a key's last use is worth writing down again.
pub fn touch_due(last_used_at: Option<i64>, now: i64) -> bool {
    last_used_at.is_none_or(|t| now - t >= KEY_TOUCH_S)
}

/// A new key for a user: its row id and the one time the token is ever seen in the clear.
pub fn mint_key(c: &Connection, user_id: &str, name: &str, scope: KeyScope, expires_at: Option<i64>, now: i64) -> anyhow::Result<(i64, String)> {
    let mut raw = [0u8; 32];
    rand::rng().fill_bytes(&mut raw);
    let token = format!("{KEY_PREFIX}{}", hex::encode(raw));
    c.execute(
        "INSERT INTO api_keys(token_hash, user_id, name, scope, created_at, expires_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![hash_token(&token), user_id, name, scope.as_str(), now, expires_at],
    )?;
    Ok((c.last_insert_rowid(), token))
}

/// A session token to its user, as it was at sign-in; grants and settings read now.
pub(crate) fn resolve_session_in(c: &Connection, settings: &crate::state::Settings, token: &str, now: i64) -> anyhow::Result<Option<AuthUser>> {
    let user = c
        .query_row(
            "SELECT user_id, user_name, is_admin FROM sessions WHERE token_hash = ?1 AND expires_at > ?2",
            params![hash_token(token), now],
            |r| Ok(AuthUser { id: r.get(0)?, name: r.get(1)?, is_admin: r.get::<_, i64>(2)? != 0, perms: Perms::default(), credential: Credential::Session, ip: None }),
        )
        .optional()?;
    let Some(mut user) = user else { return Ok(None) };
    let grants = if user.is_admin { vec![] } else { stored_grants(c, &user.id)? };
    let Some(perms) = effective(user.is_admin, &grants, settings) else { return Ok(None) };
    user.perms = perms;
    Ok(Some(user))
}

/// A key to its user, **as they are now**: a key may live years and stands in for its maker as the
/// users table has them today, so a demoted administrator's key demotes and a removed or disabled
/// user's key dies. Then the same grants and settings as a session, so revoking a permission or
/// switching sign-in off revokes the key on the next request.
pub(crate) fn resolve_key_in(c: &Connection, settings: &crate::state::Settings, token: &str, now: i64) -> anyhow::Result<Option<AuthUser>> {
    if !looks_like_key(token) {
        return Ok(None);
    }
    let row = c
        .query_row(
            "SELECT k.id, k.scope, k.user_id, k.expires_at, k.revoked_at, u.name, u.is_admin, u.is_disabled, u.removed
             FROM api_keys k LEFT JOIN users u ON u.id = k.user_id WHERE k.token_hash = ?1",
            [hash_token(token)],
            |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?, r.get::<_, Option<i64>>(3)?, r.get::<_, Option<i64>>(4)?,
                    r.get::<_, Option<String>>(5)?, r.get::<_, Option<i64>>(6)?, r.get::<_, Option<i64>>(7)?, r.get::<_, Option<i64>>(8)?)),
        )
        .optional()?;
    let Some((id, scope, user_id, expires_at, revoked_at, name, is_admin, disabled, removed)) = row else { return Ok(None) };
    let (Some(name), Some(scope)) = (name, KeyScope::parse(&scope)) else { return Ok(None) };
    if revoked_at.is_some() || expires_at.is_some_and(|e| e <= now) || disabled.unwrap_or(0) != 0 || removed.unwrap_or(0) != 0 {
        return Ok(None);
    }
    let is_admin = is_admin.unwrap_or(0) != 0;
    let grants = if is_admin { vec![] } else { stored_grants(c, &user_id)? };
    let Some(perms) = effective(is_admin, &grants, settings) else { return Ok(None) };
    Ok(Some(AuthUser { id: user_id, name, is_admin, perms, credential: Credential::Key { id, scope }, ip: None }))
}

/// A calendar key opens the calendar feed and nothing else.
pub fn refuse_calendar_key(user: &AuthUser) -> Result<(), ApiError> {
    match user.credential {
        Credential::Key { scope: KeyScope::Calendar, .. } => Err(ApiError::new(StatusCode::FORBIDDEN, "This key only opens the calendar feed")),
        _ => Ok(()),
    }
}

/// The address a request came from, when the listener recorded one.
fn request_ip(parts: &Parts, app: &App) -> Option<IpAddr> {
    parts.extensions.get::<ConnectInfo<SocketAddr>>().map(|ConnectInfo(peer)| client_ip(app, &parts.headers, *peer))
}

/// A presented credential to its user: the one place a request proves who it is.
async fn resolve(presented: Presented, ip: Option<IpAddr>, app: &App) -> Result<AuthUser, ApiError> {
    let unauthorized = || ApiError::new(StatusCode::UNAUTHORIZED, "Sign in to continue");
    let settings = app.settings();
    let user = app
        .db
        .call(move |c| {
            let now = db::now();
            match presented {
                Presented::Cookie(token) => resolve_session_in(c, &settings, &token, now),
                Presented::Bearer(token) => {
                    let user = resolve_key_in(c, &settings, &token, now)?;
                    if let Some(u) = &user {
                        touch_key(c, u, ip, now)?;
                    }
                    Ok(user)
                }
            }
        })
        .await?
        .ok_or_else(unauthorized)?;
    Ok(AuthUser { ip, ..user })
}

/// Write a key's last use down, once a minute at most; its first use is an audit entry.
fn touch_key(c: &Connection, u: &AuthUser, ip: Option<IpAddr>, now: i64) -> anyhow::Result<()> {
    let Some(id) = u.key_id() else { return Ok(()) };
    let last: Option<i64> = c.query_row("SELECT last_used_at FROM api_keys WHERE id = ?1", [id], |r| r.get(0))?;
    if !touch_due(last, now) {
        return Ok(());
    }
    c.execute("UPDATE api_keys SET last_used_at = ?2, last_used_ip = ?3 WHERE id = ?1", params![id, now, ip.map(|i| i.to_string())])?;
    if last.is_none() {
        let actor = audit::Actor { user_id: Some(u.id.clone()), user_name: Some(u.name.clone()), ip, key_id: Some(id) };
        audit::record_quietly(c, &audit::Entry::new("key_used", actor).at(now).target(id.to_string()));
    }
    Ok(())
}

pub fn client_ip(app: &App, headers: &HeaderMap, peer: SocketAddr) -> IpAddr {
    if app.trust_proxy {
        if let Some(ip) = headers
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.split(',').next())
            .and_then(|v| v.trim().parse().ok())
        {
            return ip;
        }
    }
    peer.ip()
}

impl FromRequestParts<App> for AuthUser {
    type Rejection = ApiError;

    /// Every route but the calendar feed: the cookie or a full key. Grants are read on every
    /// request, so a change by an administrator applies at once — to a key as much as to a session.
    async fn from_request_parts(parts: &mut Parts, app: &App) -> Result<Self, Self::Rejection> {
        let user = resolve(pick_credential(&parts.headers)?, request_ip(parts, app), app).await?;
        refuse_calendar_key(&user)?;
        Ok(user)
    }
}

impl FromRequestParts<App> for CalendarKey {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, app: &App) -> Result<Self, Self::Rejection> {
        let in_query = parts.uri.query().and_then(|q| q.split('&').find_map(|kv| kv.strip_prefix("key=")).map(|v| v.trim().to_string())).filter(|v| !v.is_empty());
        let presented = match in_query {
            Some(token) => Presented::Bearer(token),
            None => match bearer_token(&parts.headers) {
                Some(t) => Presented::Bearer(t?),
                None => return Err(ApiError::new(StatusCode::UNAUTHORIZED, "This feed needs a key: add ?key=… to its address")),
            },
        };
        Ok(CalendarKey(resolve(presented, request_ip(parts, app), app).await?))
    }
}

impl FromRequestParts<App> for Manager {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, app: &App) -> Result<Self, Self::Rejection> {
        let user = AuthUser::from_request_parts(parts, app).await?;
        if !user.perms.manage {
            return Err(ApiError::not_permitted("manage finstats"));
        }
        Ok(Manager(user))
    }
}

impl FromRequestParts<App> for ServerViewer {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, app: &App) -> Result<Self, Self::Rejection> {
        let user = AuthUser::from_request_parts(parts, app).await?;
        if !user.perms.see_server {
            return Err(ApiError::not_permitted("see the server"));
        }
        Ok(ServerViewer(user))
    }
}

impl FromRequestParts<App> for DownloadsViewer {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, app: &App) -> Result<Self, Self::Rejection> {
        let user = AuthUser::from_request_parts(parts, app).await?;
        if !user.perms.see_downloads {
            return Err(ApiError::not_permitted("see what is downloading"));
        }
        Ok(DownloadsViewer(user))
    }
}

impl FromRequestParts<App> for JellyfinAdmin {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, app: &App) -> Result<Self, Self::Rejection> {
        let user = AuthUser::from_request_parts(parts, app).await?;
        if !user.is_admin {
            return Err(ApiError::forbidden());
        }
        Ok(JellyfinAdmin(user))
    }
}

fn session_cookie(token: &str, max_age: i64, secure: bool) -> HeaderValue {
    let mut c = format!("{COOKIE_NAME}={token}; Path=/; HttpOnly; SameSite=Lax; Max-Age={max_age}");
    if secure {
        c.push_str("; Secure");
    }
    HeaderValue::from_str(&c).expect("cookie is ascii")
}

fn is_https(headers: &HeaderMap) -> bool {
    headers.get("x-forwarded-proto").and_then(|v| v.to_str().ok()).is_some_and(|v| v.eq_ignore_ascii_case("https"))
}

async fn user_json(app: &App, id: &str, name: &str, is_admin: bool, perms: Perms) -> Value {
    let uid = id.to_string();
    let has_image = app
        .db
        .call(move |c| {
            Ok(c.query_row("SELECT image_tag IS NOT NULL FROM users WHERE id = ?1", [uid], |r| r.get::<_, bool>(0))
                .optional()?
                .unwrap_or(false))
        })
        .await
        .unwrap_or(false);
    // `features`: which optional pages have something behind them (a connected Sonarr, a Seerr, …).
    json!({ "id": id, "name": name, "is_admin": is_admin, "has_image": has_image, "permissions": perms, "features": crate::services::features(app) })
}

async fn start_session(
    app: &App,
    headers: &HeaderMap,
    ip: IpAddr,
    user_id: &str,
    user_name: &str,
    is_admin: bool,
    perms: Perms,
) -> ApiResult<Response> {
    let mut raw = [0u8; 32];
    rand::rng().fill_bytes(&mut raw);
    let token = hex::encode(raw);
    let hash = hash_token(&token);
    let ua = headers.get("user-agent").and_then(|v| v.to_str().ok()).map(|s| s.chars().take(200).collect::<String>());
    let (uid, uname, ip_s) = (user_id.to_string(), user_name.to_string(), ip.to_string());
    app.db
        .call(move |c| {
            let now = db::now();
            c.execute("DELETE FROM sessions WHERE expires_at <= ?1", [now])?;
            c.execute(
                "INSERT INTO sessions(token_hash, user_id, user_name, is_admin, created_at, expires_at, ip, user_agent)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![hash, uid, uname, is_admin, now, now + SESSION_TTL_S, ip_s, ua],
            )?;
            Ok(())
        })
        .await?;
    let body = Json(json!({ "user": user_json(app, user_id, user_name, is_admin, perms).await }));
    let mut resp = body.into_response();
    resp.headers_mut().insert(SET_COOKIE, session_cookie(&token, SESSION_TTL_S, is_https(headers)));
    Ok(resp)
}

/// Sliding-window limiter against password guessing through finstats.
fn check_rate_limit(app: &App, ip: IpAddr) -> Result<(), ApiError> {
    let now = db::now();
    let mut map = app.login_attempts.lock().unwrap();
    map.retain(|_, (_, start)| now - *start < ATTEMPT_WINDOW_S);
    let entry = map.entry(ip).or_insert((0, now));
    if entry.0 >= MAX_ATTEMPTS {
        let wait = (ATTEMPT_WINDOW_S - (now - entry.1)).max(1);
        return Err(ApiError::new(
            StatusCode::TOO_MANY_REQUESTS,
            format!("Too many sign-in attempts. Try again in {} minutes.", (wait + 59) / 60),
        ));
    }
    entry.0 += 1;
    Ok(())
}

fn clear_rate_limit(app: &App, ip: IpAddr) {
    app.login_attempts.lock().unwrap().remove(&ip);
}

#[derive(Deserialize)]
pub struct LoginBody {
    #[serde(default)]
    username: String,
    #[serde(default)]
    password: String,
}

pub async fn login(
    State(app): State<App>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(body): Json<LoginBody>,
) -> ApiResult<Response> {
    let jf = app
        .jellyfin()
        .ok_or_else(|| ApiError::new(StatusCode::CONFLICT, "finstats is not connected to Jellyfin yet"))?;
    let username = body.username.trim();
    if username.is_empty() {
        return Err(ApiError::bad_request("Enter your Jellyfin username"));
    }
    let ip = client_ip(&app, &headers, peer);
    check_rate_limit(&app, ip)?;

    let jf = app.jellyfin_anonymous(jf.base());
    let auth = match jf.authenticate(username, &body.password).await {
        Ok(a) => a,
        Err(AuthError::InvalidCredentials) => {
            // The name as typed: it is the administrator's own server, and Jellyfin's log says the same.
            let who = Actor { user_id: None, user_name: Some(username.chars().take(100).collect()), ip: Some(ip), key_id: None };
            audit::record(&app, audit::Entry::new("sign_in_failed", who).outcome("failed"));
            return Err(ApiError::new(StatusCode::UNAUTHORIZED, "Wrong username or password"));
        }
        Err(AuthError::Other(e)) => {
            tracing::warn!("login could not be checked against Jellyfin: {e:#}");
            return Err(ApiError::new(StatusCode::BAD_GATEWAY, "Could not reach Jellyfin to check your sign-in"));
        }
    };
    jf.logout(&auth.access_token).await;

    let uid = auth.user_id.clone();
    let grants = if auth.is_admin { vec![] } else { app.db.call(move |c| stored_grants(c, &uid)).await? };
    let actor = Actor { user_id: Some(auth.user_id.clone()), user_name: Some(auth.user_name.clone()), ip: Some(ip), key_id: None };
    let Some(perms) = effective(auth.is_admin, &grants, &app.settings()) else {
        audit::record(&app, audit::Entry::new("sign_in_refused", actor).outcome("refused"));
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            "You haven't been given access to finstats. A Jellyfin administrator can allow you in Settings.",
        ));
    };
    clear_rate_limit(&app, ip);
    tracing::info!("{} signed in from {ip}", auth.user_name);
    audit::record(&app, audit::Entry::new("sign_in", actor));
    start_session(&app, &headers, ip, &auth.user_id, &auth.user_name, auth.is_admin, perms).await
}

pub async fn logout(State(app): State<App>, ConnectInfo(peer): ConnectInfo<SocketAddr>, headers: HeaderMap) -> ApiResult<Response> {
    if let Some(token) = cookie_token(&headers) {
        let hash = hash_token(&token);
        let ip = client_ip(&app, &headers, peer);
        app.db
            .call(move |c| {
                let who: Option<(String, String)> = c
                    .query_row("DELETE FROM sessions WHERE token_hash = ?1 RETURNING user_id, user_name", [hash], |r| Ok((r.get(0)?, r.get(1)?)))
                    .optional()?;
                if let Some((id, name)) = who {
                    audit::record_quietly(c, &audit::Entry::new("sign_out", Actor { user_id: Some(id), user_name: Some(name), ip: Some(ip), key_id: None }));
                }
                Ok(())
            })
            .await?;
    }
    let mut resp = Json(json!({ "ok": true })).into_response();
    resp.headers_mut().insert(SET_COOKIE, session_cookie("", 0, is_https(&headers)));
    Ok(resp)
}

pub async fn me(State(app): State<App>, user: AuthUser) -> ApiResult {
    Ok(Json(json!({ "user": user_json(&app, &user.id, &user.name, user.is_admin, user.perms).await })))
}

// ---------------------------------------------------------------- first-run setup

pub async fn status(State(app): State<App>) -> Json<Value> {
    let cfg = app.config.read().unwrap().clone();
    // How the collector is listening, so it can be checked from outside without reading a log. Shape
    // only: which of the four things it is doing, what is true on the wire, and how often it is
    // asking. Never who, never what — no names, no titles, not even how many sessions there are.
    // One read of memory: no request to Jellyfin, no query, nothing that could make this slow.
    let c = app.collector.read().unwrap().clone();
    Json(json!({
        "configured": cfg.is_some(),
        "version": env!("CARGO_PKG_VERSION"),
        "server_name": cfg.map(|c| c.server_name),
        "session_mode": c.session_mode,
        "socket_connected": c.socket_connected,
        "socket_subscribed": c.socket_subscribed,
        "poll_interval_s": c.poll_interval_s,
        // Counted at the gate every `/Sessions` read passes through, so it is what a packet capture
        // would count and not what the collector believes it asked for. Listening should be nearly
        // nothing; a play is one a second.
        "sessions_requests_last_min": crate::collector::sessions_reads_last_min(),
        "mode_since": (c.mode_since > 0).then(|| chrono::DateTime::from_timestamp(c.mode_since, 0).unwrap_or_default().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)),
    }))
}

fn ensure_unconfigured(app: &App) -> Result<(), ApiError> {
    if app.is_configured() {
        return Err(ApiError::new(StatusCode::CONFLICT, "finstats is already set up"));
    }
    Ok(())
}

#[derive(Deserialize)]
pub struct SetupTestBody {
    #[serde(default)]
    url: String,
}

pub async fn setup_test(State(app): State<App>, Json(body): Json<SetupTestBody>) -> ApiResult {
    ensure_unconfigured(&app)?;
    let url = jellyfin::normalize_url(&body.url).map_err(|e| ApiError::bad_request(e.to_string()))?;
    let info = app
        .jellyfin_anonymous(&url)
        .public_info()
        .await
        .map_err(|e| ApiError::new(StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(json!({ "server_name": info.server_name, "version": info.version, "id": info.id, "url": url })))
}

#[derive(Deserialize)]
pub struct SetupBody {
    #[serde(default)]
    url: String,
    #[serde(default)]
    username: String,
    #[serde(default)]
    password: String,
}

pub async fn setup(
    State(app): State<App>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(body): Json<SetupBody>,
) -> ApiResult<Response> {
    ensure_unconfigured(&app)?;
    let ip = client_ip(&app, &headers, peer);
    check_rate_limit(&app, ip)?;
    let url = jellyfin::normalize_url(&body.url).map_err(|e| ApiError::bad_request(e.to_string()))?;
    let jf = app.jellyfin_anonymous(&url);
    let info = jf.public_info().await.map_err(|e| ApiError::new(StatusCode::BAD_GATEWAY, e.to_string()))?;

    let auth = match jf.authenticate(body.username.trim(), &body.password).await {
        Ok(a) => a,
        Err(AuthError::InvalidCredentials) => {
            return Err(ApiError::new(StatusCode::UNAUTHORIZED, "Wrong username or password"));
        }
        Err(AuthError::Other(e)) => return Err(ApiError::new(StatusCode::BAD_GATEWAY, e.to_string())),
    };
    if !auth.is_admin {
        jf.logout(&auth.access_token).await;
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            "That account is not a Jellyfin administrator. Set finstats up with an administrator account.",
        ));
    }
    let key = jf.ensure_api_key(&auth.access_token).await;
    jf.logout(&auth.access_token).await;
    let api_key = key.map_err(|e| ApiError::new(StatusCode::BAD_GATEWAY, e.to_string()))?;

    // Re-check under the lock-free flag: two people finishing the wizard at once.
    ensure_unconfigured(&app)?;
    let cfg = JfConfig {
        url,
        api_key,
        server_name: info.server_name.unwrap_or_else(|| "Jellyfin".into()),
        server_version: info.version.unwrap_or_default(),
        from_env: false,
    };
    let stored = cfg.clone();
    app.db
        .call(move |c| {
            db::set_setting(c, "jellyfin_url", &stored.url)?;
            db::set_setting(c, "jellyfin_api_key", &stored.api_key)?;
            db::set_setting(c, "server_name", &stored.server_name)?;
            db::set_setting(c, "server_version", &stored.server_version)?;
            Ok(())
        })
        .await?;
    *app.config.write().unwrap() = Some(cfg);
    clear_rate_limit(&app, ip);
    tracing::info!("setup completed by {}", auth.user_name);
    audit::record(&app, audit::Entry::new("setup_completed", Actor { user_id: Some(auth.user_id.clone()), user_name: Some(auth.user_name.clone()), ip: Some(ip), key_id: None }));
    app.wake.notify_waiters();

    start_session(&app, &headers, ip, &auth.user_id, &auth.user_name, true, Perms::ALL).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::Settings;

    fn grants(keys: &[&str]) -> Vec<String> {
        keys.iter().map(|k| k.to_string()).collect()
    }

    #[test]
    fn administrators_hold_everything_and_need_no_grant() {
        assert_eq!(effective(true, &[], &Settings::default()), Some(Perms::ALL));
    }

    #[test]
    fn nobody_else_gets_in_unless_allowed() {
        let closed = Settings::default();
        assert_eq!(effective(false, &[], &closed), None);
        // Holding a viewing permission is not an invitation: sign-in is its own grant.
        assert_eq!(effective(false, &grants(&["see_everyone"]), &closed), None);
        assert_eq!(effective(false, &grants(&["sign_in"]), &closed), Some(Perms::default()));
    }

    #[test]
    fn defaults_and_personal_grants_add_up() {
        let open = Settings { allow_user_login: true, default_permissions: grants(&["see_everyone"]), ..Settings::default() };
        assert_eq!(effective(false, &[], &open), Some(Perms { see_everyone: true, ..Perms::default() }));
        let p = effective(false, &grants(&["see_network", "bogus"]), &open).unwrap();
        assert_eq!(p, Perms { see_everyone: true, see_network: true, ..Perms::default() });
        assert!(!p.manage && !p.see_server);
    }

    #[test]
    fn unknown_default_permissions_are_rejected() {
        let s = Settings { default_permissions: grants(&["see_everyone", "root"]), ..Settings::default() };
        assert!(s.validate().is_err());
        assert!(Settings { default_permissions: grants(&["manage"]), ..Settings::default() }.validate().is_ok());
    }
}

#[cfg(test)]
mod key_tests {
    use super::*;
    use crate::db::rusqlite::Connection;
    use crate::state::Settings;

    fn conn() -> Connection {
        let c = Connection::open_in_memory().unwrap();
        for m in crate::db::MIGRATIONS {
            c.execute_batch(m).unwrap();
        }
        c.execute_batch(
            "INSERT INTO users(id, name, is_admin, updated_at) VALUES ('u1', 'alice', 1, 1), ('u2', 'bob', 0, 1);
             INSERT INTO user_permissions(user_id, permissions, updated_at) VALUES ('u2', '[\"see_everyone\"]', 1);",
        )
        .unwrap();
        c
    }
    fn open() -> Settings {
        Settings { allow_user_login: true, ..Default::default() }
    }

    #[test]
    fn a_key_resolves_to_the_same_permissions_as_a_session() {
        let c = conn();
        c.execute("INSERT INTO sessions(token_hash, user_id, user_name, is_admin, created_at, expires_at) VALUES (?1, 'u2', 'bob', 0, 1, 9999999999)", [hash_token("cookie-token")]).unwrap();
        let (_, key) = mint_key(&c, "u2", "laptop", KeyScope::Full, None, 1000).unwrap();
        let by_session = resolve_session_in(&c, &open(), "cookie-token", 1000).unwrap().expect("the session is good");
        let by_key = resolve_key_in(&c, &open(), &key, 1000).unwrap().expect("the key is good");
        assert_eq!((by_key.id.as_str(), by_key.name.as_str(), by_key.is_admin, by_key.perms), (by_session.id.as_str(), by_session.name.as_str(), by_session.is_admin, by_session.perms));
        assert!(by_key.perms.see_everyone && !by_key.perms.manage);
        assert_eq!(by_session.credential, Credential::Session);
        assert!(matches!(by_key.credential, Credential::Key { scope: KeyScope::Full, .. }));
        assert_eq!(by_key.key_id(), Some(1));
        // An administrator's key is an administrator: everything, read live from the users table.
        let (_, admin) = mint_key(&c, "u1", "script", KeyScope::Full, None, 1000).unwrap();
        assert_eq!(resolve_key_in(&c, &open(), &admin, 1000).unwrap().unwrap().perms, Perms::ALL);
        c.execute_batch("UPDATE users SET is_admin = 0 WHERE id = 'u1'").unwrap();
        assert_eq!(resolve_key_in(&c, &open(), &admin, 1000).unwrap().unwrap().perms, Perms::default(), "demoted, the key demotes with them");
    }

    #[test]
    fn a_calendar_key_is_refused_everywhere_but_the_feed() {
        let c = conn();
        let (_, key) = mint_key(&c, "u2", "phone", KeyScope::Calendar, None, 1000).unwrap();
        let user = resolve_key_in(&c, &open(), &key, 1000).unwrap().unwrap();
        assert!(matches!(user.credential, Credential::Key { scope: KeyScope::Calendar, .. }));
        assert_eq!(refuse_calendar_key(&user).unwrap_err().0, StatusCode::FORBIDDEN);
        let (_, full) = mint_key(&c, "u2", "laptop", KeyScope::Full, None, 1000).unwrap();
        assert!(refuse_calendar_key(&resolve_key_in(&c, &open(), &full, 1000).unwrap().unwrap()).is_ok());
    }

    #[test]
    fn an_expired_revoked_or_unknown_key_is_refused() {
        let c = conn();
        let (id, expiring) = mint_key(&c, "u2", "a", KeyScope::Full, Some(2000), 1000).unwrap();
        assert!(resolve_key_in(&c, &open(), &expiring, 1999).unwrap().is_some());
        assert!(resolve_key_in(&c, &open(), &expiring, 2000).unwrap().is_none(), "expired on the second");
        let (rid, revoked) = mint_key(&c, "u2", "b", KeyScope::Full, None, 1000).unwrap();
        c.execute("UPDATE api_keys SET revoked_at = 1500 WHERE id = ?1", [rid]).unwrap();
        assert!(resolve_key_in(&c, &open(), &revoked, 1600).unwrap().is_none());
        assert!(resolve_key_in(&c, &open(), &format!("fs_{}", "0".repeat(64)), 1000).unwrap().is_none(), "unknown");
        assert!(resolve_key_in(&c, &open(), "not a key", 1000).unwrap().is_none(), "not even shaped like one");
        assert_ne!(id, rid);
    }

    #[test]
    fn a_key_of_someone_who_may_no_longer_sign_in_is_refused() {
        let c = conn();
        let (_, key) = mint_key(&c, "u2", "laptop", KeyScope::Full, None, 1000).unwrap();
        assert!(resolve_key_in(&c, &open(), &key, 1000).unwrap().is_some());
        let closed = Settings { allow_user_login: false, ..Default::default() };
        assert!(resolve_key_in(&c, &closed, &key, 1000).unwrap().is_none(), "sign-in switched off, no sign_in grant: the key dies with the access");
        c.execute_batch("UPDATE user_permissions SET permissions = '[\"sign_in\"]' WHERE user_id = 'u2'").unwrap();
        assert!(resolve_key_in(&c, &closed, &key, 1000).unwrap().is_some(), "granted sign_in, the key lives");
        c.execute_batch("UPDATE users SET is_disabled = 1 WHERE id = 'u2'").unwrap();
        assert!(resolve_key_in(&c, &open(), &key, 1000).unwrap().is_none(), "disabled in Jellyfin");
        c.execute_batch("UPDATE users SET is_disabled = 0, removed = 1 WHERE id = 'u2'").unwrap();
        assert!(resolve_key_in(&c, &open(), &key, 1000).unwrap().is_none(), "removed from Jellyfin");
    }

    #[test]
    fn the_header_beats_the_cookie() {
        let mut h = HeaderMap::new();
        h.insert(COOKIE, HeaderValue::from_static("finstats_session=cookie-token"));
        assert_eq!(pick_credential(&h).unwrap(), Presented::Cookie("cookie-token".into()));
        h.insert(AUTHORIZATION, HeaderValue::from_static("Bearer fs_abc"));
        assert_eq!(pick_credential(&h).unwrap(), Presented::Bearer("fs_abc".into()), "an explicit header, not the ambient cookie");
        h.insert(AUTHORIZATION, HeaderValue::from_static("Basic dXNlcjpwYXNz"));
        assert_eq!(pick_credential(&h).unwrap_err().0, StatusCode::UNAUTHORIZED, "a wrong header never falls back to the cookie");
        assert_eq!(pick_credential(&HeaderMap::new()).unwrap_err().0, StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn only_a_token_shaped_like_a_key_reaches_the_database() {
        let hex64 = "a".repeat(64);
        assert!(looks_like_key(&format!("fs_{hex64}")));
        assert!(!looks_like_key(&format!("fs_{}", "a".repeat(63))));
        assert!(!looks_like_key(&format!("fs_{}", "A".repeat(64))), "lowercase hex only");
        assert!(!looks_like_key(&hex64), "no prefix");
        assert!(!looks_like_key(&format!("fs_{}", "g".repeat(64))));
        assert!(!looks_like_key(""));
    }

    #[test]
    fn a_key_is_touched_at_most_once_a_minute() {
        assert!(touch_due(None, 1000));
        assert!(!touch_due(Some(1000 - 30), 1000));
        assert!(touch_due(Some(1000 - 61), 1000));
    }

    #[test]
    fn a_token_is_stored_only_as_a_hash() {
        let c = conn();
        let (_, key) = mint_key(&c, "u2", "laptop", KeyScope::Full, None, 1000).unwrap();
        assert!(looks_like_key(&key));
        let plain: i64 = c.query_row("SELECT COUNT(*) FROM api_keys WHERE token_hash = ?1", [&key], |r| r.get(0)).unwrap();
        let hashed: i64 = c.query_row("SELECT COUNT(*) FROM api_keys WHERE token_hash = ?1", [hash_token(&key)], |r| r.get(0)).unwrap();
        assert_eq!((plain, hashed), (0, 1));
    }

    #[test]
    fn bearer_scheme_is_case_insensitive_and_spacing_tolerant() {
        let h = |v: &'static str| { let mut m = HeaderMap::new(); m.insert(AUTHORIZATION, HeaderValue::from_static(v)); m };
        assert_eq!(bearer_token(&h("bearer x")).unwrap().unwrap(), "x");
        assert_eq!(bearer_token(&h("BEARER   x  ")).unwrap().unwrap(), "x");
        assert!(bearer_token(&h("Basic x")).unwrap().is_err());
        assert!(bearer_token(&h("Bearer")).unwrap().is_err());
        assert!(bearer_token(&h("Bearer   ")).unwrap().is_err());
        assert!(bearer_token(&HeaderMap::new()).is_none(), "no header at all is not an error: the cookie may be there");
    }
}
