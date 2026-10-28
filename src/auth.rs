//! Sign-in with Jellyfin credentials.
//!
//! FinStats never stores passwords. A login is forwarded to Jellyfin's
//! `AuthenticateByName`; on success we mint our own opaque session token (stored
//! hashed) and immediately end the Jellyfin session the check created.

use std::collections::HashMap;
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
/// The session's cookie on https: the `__Host-` prefix makes a browser take it only from this host, Secure, for the whole
/// path, so a host on a sibling name cannot set one of its own for this one to read first.
const HOST_COOKIE_NAME: &str = "__Host-finstats_session";
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

/// The feed's own extractor: a calendar key in the query string, or a key of any scope in the header, and never
/// the cookie: a subscribed calendar cannot send one, and a browser must not open a feed by accident.
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

/// What a session token begins with: begun on https, or on plain http. Tokens from before have neither (they are hex).
const BEGUN_ON_HTTPS: &str = "h";
const BEGUN_ON_HTTP: &str = "p";

/// Every value a cookie of this name was sent with.
fn cookies_named(headers: &HeaderMap, name: &str) -> Vec<String> {
    headers
        .get_all(COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(';'))
        .filter_map(|kv| {
            let (k, v) = kv.trim().split_once('=')?;
            (k == name && !v.is_empty()).then(|| v.to_string())
        })
        .collect()
}

/// The session a browser presents: the `__Host-` cookie when there is one; else the plain one, unless there are two of
/// those, which only another host's cookie for the whole domain beside ours could make, and then neither. A session
/// begun on https is never taken under the plain name, which any host of the domain may set: before the browser held
/// a `__Host-` cookie, or once it had signed out, a sibling's tossed cookie was the only one sent, and won.
pub(crate) fn cookie_token(headers: &HeaderMap) -> Option<String> {
    if let Some(own) = cookies_named(headers, HOST_COOKIE_NAME).into_iter().next() {
        return Some(own);
    }
    let plain = cookies_named(headers, COOKIE_NAME);
    (plain.len() == 1).then(|| plain.into_iter().next()).flatten().filter(|t| !t.starts_with(BEGUN_ON_HTTPS))
}

/// `Authorization: Bearer <key>`: `None` when there is no such header at all (the cookie may be there),
/// `Some(Err)` when there is one that is not a bearer token, which is never quietly ignored.
pub fn bearer_token(headers: &HeaderMap) -> Option<Result<String, ApiError>> {
    let raw = headers.get(AUTHORIZATION)?.to_str().ok().map(str::trim).unwrap_or_default();
    let bad = || ApiError::new(StatusCode::UNAUTHORIZED, crate::i18n::t("server.auth.bearer_form", &[]));
    let Some((scheme, rest)) = raw.split_once(char::is_whitespace) else { return Some(Err(bad())) };
    let token = rest.trim();
    Some(if scheme.eq_ignore_ascii_case("bearer") && !token.is_empty() { Ok(token.to_string()) } else { Err(bad()) })
}

/// The header wins over the cookie: it is an explicit act of this request where a cookie is ambient,
/// so a script sending a revoked key gets its 401 rather than quietly succeeding on a browser's session.
/// Another scheme (`Basic`, which a proxy's own sign-in passes along) beside a cookie is not FinStats' and is left alone;
/// keys are `Bearer` only.
pub fn pick_credential(headers: &HeaderMap) -> Result<Presented, ApiError> {
    let not_bearer = headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split_whitespace().next())
        .is_some_and(|scheme| !scheme.eq_ignore_ascii_case("bearer"));
    if not_bearer && let Some(cookie) = cookie_token(headers) {
        return Ok(Presented::Cookie(cookie));
    }
    match bearer_token(headers) {
        Some(t) => Ok(Presented::Bearer(t?)),
        None => cookie_token(headers).map(Presented::Cookie).ok_or_else(|| ApiError::new(StatusCode::UNAUTHORIZED, crate::i18n::t("server.auth.sign_in_to_continue", &[]))),
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

/// A session token to its user **as they are now**, exactly like a key: a session lives thirty days, and
/// one minted for an administrator whom Jellyfin has since demoted, disabled or deleted must not go on
/// acting for who they were. Only before the users read has written their row at all (the wizard signs
/// its administrator in first) is the session what Jellyfin said at sign-in.
pub(crate) fn resolve_session_in(c: &Connection, settings: &crate::state::Settings, token: &str, now: i64) -> anyhow::Result<Option<AuthUser>> {
    let row = c
        .query_row(
            "SELECT s.user_id, s.user_name, COALESCE(u.is_admin, s.is_admin), COALESCE(u.is_disabled, 0), COALESCE(u.removed, 0)
             FROM sessions s LEFT JOIN users u ON u.id = s.user_id WHERE s.token_hash = ?1 AND s.expires_at > ?2",
            params![hash_token(token), now],
            |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, i64>(2)? != 0, r.get::<_, i64>(3)? != 0, r.get::<_, i64>(4)? != 0)),
        )
        .optional()?;
    let Some((id, name, is_admin, disabled, removed)) = row else { return Ok(None) };
    if disabled || removed {
        return Ok(None);
    }
    let mut user = AuthUser { id, name, is_admin, perms: Perms::default(), credential: Credential::Session, ip: None };
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
        Credential::Key { scope: KeyScope::Calendar, .. } => Err(ApiError::new(StatusCode::FORBIDDEN, crate::i18n::t("server.auth.calendar_key_only", &[]))),
        _ => Ok(()),
    }
}

/// The address a request came from, when the listener recorded one.
fn request_ip(parts: &Parts, app: &App) -> Option<IpAddr> {
    parts.extensions.get::<ConnectInfo<SocketAddr>>().map(|ConnectInfo(peer)| client_ip(app, &parts.headers, *peer))
}

/// The same resolution for a connection that outlives its request (the live channel), which keeps what it presented
/// and asks again: a session signed out or a key revoked must close it as it would refuse a request.
pub(crate) async fn resolve_presented(presented: Presented, ip: Option<IpAddr>, app: &App) -> Result<AuthUser, ApiError> {
    resolve(presented, ip, app).await
}

/// A presented credential to its user: the one place a request proves who it is.
async fn resolve(presented: Presented, ip: Option<IpAddr>, app: &App) -> Result<AuthUser, ApiError> {
    let unauthorized = || ApiError::new(StatusCode::UNAUTHORIZED, crate::i18n::t("server.auth.sign_in_to_continue", &[]));
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

/// Which peers' `X-Forwarded-For` (and `X-Forwarded-Host`, `X-Forwarded-Proto`) is believed: `FINSTATS_TRUST_PROXY`.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum Proxies {
    /// Nobody's (unset, `0`, `false`): every request is the address it came from.
    #[default]
    Off,
    /// `1` or `true`, as it always was: a peer on this machine or a private network is a proxy, and only the last entry,
    /// the one it wrote, is believed. A peer from outside is itself, so a port reachable beside the proxy picks nothing.
    Local,
    /// A list of addresses and ranges (`172.18.0.0/16, 10.0.0.2`): those peers are proxies, and the header is walked
    /// from the right past every hop that is one of them.
    Named(Vec<(IpAddr, u8)>),
}

impl Proxies {
    pub fn parse(raw: &str) -> anyhow::Result<Proxies> {
        let raw = raw.trim();
        if raw.is_empty() || raw == "0" || raw.eq_ignore_ascii_case("false") {
            return Ok(Proxies::Off);
        }
        if raw == "1" || raw.eq_ignore_ascii_case("true") {
            return Ok(Proxies::Local);
        }
        let mut nets = vec![];
        for entry in raw.split([',', ' ']).map(str::trim).filter(|e| !e.is_empty()) {
            let (addr, bits) = entry.split_once('/').map_or((entry, None), |(a, b)| (a, Some(b)));
            let addr: IpAddr = canonical(addr.parse().map_err(|_| anyhow::anyhow!("FINSTATS_TRUST_PROXY: `{entry}` is not an address or a range of addresses"))?);
            let max = if addr.is_ipv4() { 32 } else { 128 };
            let bits = match bits {
                None => max,
                Some(b) => b.parse::<u8>().ok().filter(|b| *b <= max).ok_or_else(|| anyhow::anyhow!("FINSTATS_TRUST_PROXY: `{entry}` has no valid prefix length"))?,
            };
            nets.push((addr, bits));
        }
        Ok(Proxies::Named(nets))
    }

    /// Whether any peer is believed at all.
    pub fn any(&self) -> bool {
        *self != Proxies::Off
    }

    /// Whether this peer is a proxy whose forwarded headers are believed.
    pub fn trusts(&self, peer: IpAddr) -> bool {
        let peer = canonical(peer);
        match self {
            Proxies::Off => false,
            Proxies::Local => db::is_local_ip(&peer.to_string()).unwrap_or(false),
            Proxies::Named(nets) => nets.iter().any(|(net, bits)| within(peer, *net, *bits)),
        }
    }

    /// What the start-up log says about it.
    pub fn describe(&self) -> Option<String> {
        match self {
            Proxies::Off => None,
            Proxies::Local => Some("FINSTATS_TRUST_PROXY=1: X-Forwarded-For is believed only from a peer on this machine or a private network, one hop; name the proxy's address (or range) instead to be exact".into()),
            Proxies::Named(nets) => Some(format!(
                "X-Forwarded-For is believed from {}",
                nets.iter().map(|(a, b)| format!("{a}/{b}")).collect::<Vec<_>>().join(", ")
            )),
        }
    }
}

/// An IPv4 address written inside IPv6 is that address.
fn canonical(ip: IpAddr) -> IpAddr {
    match ip {
        IpAddr::V6(v6) => v6.to_ipv4_mapped().map_or(ip, IpAddr::V4),
        v4 => v4,
    }
}

fn within(ip: IpAddr, net: IpAddr, bits: u8) -> bool {
    let mask = |width: u32| if bits == 0 { 0 } else { u128::MAX << (width - u32::from(bits)) };
    match (ip, net) {
        (IpAddr::V4(a), IpAddr::V4(n)) => (u128::from(u32::from(a)) ^ u128::from(u32::from(n))) & mask(32) & u128::from(u32::MAX) == 0,
        (IpAddr::V6(a), IpAddr::V6(n)) => (u128::from(a) ^ u128::from(n)) & mask(128) == 0,
        _ => false,
    }
}

/// One `X-Forwarded-For` entry: an address, with or without a port (`203.0.113.9:51234`, `[2001:db8::7]:443`).
fn forwarded_entry(raw: &str) -> Option<IpAddr> {
    let raw = raw.trim().trim_matches('"');
    if let Ok(ip) = raw.parse::<IpAddr>() {
        return Some(canonical(ip));
    }
    raw.parse::<SocketAddr>().ok().map(|s| canonical(s.ip()))
}

/// The address a request came from. Without a believed proxy, the peer. Behind one, `X-Forwarded-For` read from the
/// right: every entry before the proxy's own came from the client, which may put anything there, a header line of its own
/// included (some proxies add a line rather than extend the client's, so the last line is the proxy's).
pub fn client_ip(app: &App, headers: &HeaderMap, peer: SocketAddr) -> IpAddr {
    let peer_ip = canonical(peer.ip());
    let proxies = &app.trust_proxy;
    if !proxies.trusts(peer_ip) {
        warn_unbelieved(proxies, headers, peer_ip);
        return peer_ip;
    }
    let lines: Vec<&str> = headers.get_all("x-forwarded-for").iter().filter_map(|v| v.to_str().ok()).collect();
    let mut entries = lines.iter().flat_map(|l| l.split(',')).collect::<Vec<_>>().into_iter().rev();
    if *proxies == Proxies::Local {
        return entries.next().and_then(forwarded_entry).unwrap_or(peer_ip);
    }
    let mut hop = peer_ip;
    for entry in entries {
        let Some(ip) = forwarded_entry(entry) else { return hop };
        hop = ip;
        if !proxies.trusts(hop) {
            break;
        }
    }
    hop
}

/// Once per run: with no proxy believed, a peer on this network sent `X-Forwarded-For`, which is a reverse proxy in front
/// of FinStats, and every request through it then counts as the proxy's (one sign-in limit for the whole internet).
fn warn_unbelieved(proxies: &Proxies, headers: &HeaderMap, peer: IpAddr) {
    static WARNED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if *proxies == Proxies::Off
        && headers.contains_key("x-forwarded-for")
        && db::is_local_ip(&peer.to_string()).unwrap_or(false)
        && !WARNED.swap(true, std::sync::atomic::Ordering::Relaxed)
    {
        tracing::warn!(
            "a request from {peer} carried X-Forwarded-For, but FINSTATS_TRUST_PROXY is not set: every request through that proxy counts as the proxy's own address, sign-in limit included. Set FINSTATS_TRUST_PROXY to the proxy's address"
        );
    }
}

impl FromRequestParts<App> for AuthUser {
    type Rejection = ApiError;

    /// Every route but the calendar feed: the cookie or a full key. Grants are read on every
    /// request, so a change by an administrator applies at once, to a key as much as to a session.
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
        let in_address = in_query.is_some();
        let presented = match in_query {
            Some(token) => Presented::Bearer(token),
            None => match bearer_token(&parts.headers) {
                Some(t) => Presented::Bearer(t?),
                None => return Err(ApiError::new(StatusCode::UNAUTHORIZED, crate::i18n::t("server.auth.feed_needs_key", &[]))),
            },
        };
        let user = resolve(presented, request_ip(parts, app), app).await?;
        // An address is kept by whatever syncs the calendar and by every log on the way, so only a key that opens the
        // feed alone may be put in one; any other key is sent in the header, as everywhere else.
        if in_address && !matches!(user.credential, Credential::Key { scope: KeyScope::Calendar, .. }) {
            return Err(ApiError::new(StatusCode::FORBIDDEN, crate::i18n::t("server.auth.feed_key_in_address", &[])));
        }
        Ok(CalendarKey(user))
    }
}

impl FromRequestParts<App> for Manager {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, app: &App) -> Result<Self, Self::Rejection> {
        let user = AuthUser::from_request_parts(parts, app).await?;
        if !user.perms.manage {
            return Err(ApiError::not_permitted("manage FinStats"));
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
    let name = if secure { HOST_COOKIE_NAME } else { COOKIE_NAME };
    let mut c = format!("{name}={token}; Path=/; HttpOnly; SameSite=Lax; Max-Age={max_age}");
    if secure {
        c.push_str("; Secure");
    }
    HeaderValue::from_str(&c).expect("cookie is ascii")
}

/// The session's cookie set, and on https the plain one from before the `__Host-` name cleared beside it.
fn set_session(resp: &mut Response, token: &str, max_age: i64, secure: bool) {
    resp.headers_mut().append(SET_COOKIE, session_cookie(token, max_age, secure));
    if secure {
        resp.headers_mut().append(SET_COOKIE, session_cookie("", 0, false));
    }
}

/// Whether the page that signs in is on https: its proxy says so, or the browser does in the `Origin` every sign-in (a POST)
/// carries. Either can only ever make a cookie stricter, so neither needs a trusted proxy.
fn is_https(headers: &HeaderMap) -> bool {
    let said = |name: &str| headers.get(name).and_then(|v| v.to_str().ok()).map(str::to_ascii_lowercase);
    said("x-forwarded-proto").is_some_and(|v| v == "https") || said("origin").is_some_and(|v| v.starts_with("https://"))
}

async fn user_json(app: &App, id: &str, name: &str, is_admin: bool, perms: Perms) -> Value {
    let uid = id.to_string();
    // `welcomed`: whether they have been through the welcome, on any browser.
    let (has_image, welcomed) = app
        .db
        .call(move |c| {
            let image = c.query_row("SELECT image_tag IS NOT NULL FROM users WHERE id = ?1", [&uid], |r| r.get::<_, bool>(0)).optional()?.unwrap_or(false);
            Ok((image, crate::welcome::welcomed(c, &uid)?))
        })
        .await
        .unwrap_or((false, false));
    // `features`: which optional pages have something behind them (a connected Sonarr, a Seerr, …). `jellyfin_details`:
    // a title's page in Jellyfin is this followed by its id, for a link offered where the answer had none to give (a menu).
    // Only the address an administrator gave for people to open Jellyfin at: the one FinStats reads it at is often inside
    // the house, and only administrators are shown that.
    let jellyfin_details = Some(app.settings().jellyfin_public_url.trim().to_string()).filter(|u| !u.is_empty()).map(|b| crate::jellyfin::web_link(&b, ""));
    // `public_url`: the address FinStats is reached at, for a link made in the page (a calendar feed's); the settings
    // that hold it are a manager's to read.
    let public_url = Some(app.settings().public_url.trim_end_matches('/').to_string()).filter(|u| !u.is_empty());
    json!({ "id": id, "name": name, "is_admin": is_admin, "has_image": has_image, "welcomed": welcomed, "permissions": perms, "features": crate::services::features(app),
            "jellyfin_details": jellyfin_details, "public_url": public_url })
}

/// What Jellyfin has just said about somebody it let in, written to their users row: a session reads that
/// row live, so it must never be older than the sign-in it serves. Without this, a row the users read has
/// not reached yet (written by a Jellystat import from the file's own word for who administers Jellyfin)
/// would make a session an administrator's that Jellyfin never did.
/// Answers whether the row changed (or was new), which is when that person's open connections must look again.
pub(crate) fn remember_sign_in(c: &Connection, user_id: &str, user_name: &str, is_admin: bool, policy: &crate::access::Policy, now: i64) -> anyhow::Result<bool> {
    let changed = c.execute(
        "INSERT INTO users(id, name, is_admin, is_disabled, removed, updated_at) VALUES (?1, ?2, ?3, 0, 0, ?4)
         ON CONFLICT(id) DO UPDATE SET name = excluded.name, is_admin = excluded.is_admin, is_disabled = 0, removed = 0
         WHERE users.name IS NOT excluded.name OR users.is_admin IS NOT excluded.is_admin OR users.is_disabled <> 0 OR users.removed <> 0",
        params![user_id, user_name, is_admin, now],
    )?;
    // What of the library Jellyfin lets them see, from the same answer: a change there is a change for their connections too.
    let access = crate::access::store(c, user_id, policy)?;
    Ok(changed > 0 || access)
}

/// The most sign-ins one person keeps open at once: every device they use, many times over. A session is a row kept
/// for thirty days, and somebody with a password could otherwise sign in in a loop and fill the table and the disk.
pub(crate) const SESSIONS_PER_USER: usize = 30;

/// A new session, written, and that person's oldest beyond [`SESSIONS_PER_USER`] let go, never the new one. Answers how
/// many were let go.
#[allow(clippy::too_many_arguments)]
pub(crate) fn store_session(c: &Connection, hash: &str, user_id: &str, user_name: &str, is_admin: bool, now: i64, ip: Option<&str>, ua: Option<&str>) -> anyhow::Result<usize> {
    c.execute(
        "INSERT INTO sessions(token_hash, user_id, user_name, is_admin, created_at, expires_at, ip, user_agent, seq)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, (SELECT COALESCE(MAX(seq), 0) + 1 FROM sessions WHERE user_id = ?2))",
        params![hash, user_id, user_name, is_admin, now, now + SESSION_TTL_S, ip, ua],
    )?;
    Ok(c.execute(
        "DELETE FROM sessions WHERE user_id = ?1 AND token_hash <> ?2 AND token_hash NOT IN (
           SELECT token_hash FROM sessions WHERE user_id = ?1 AND token_hash <> ?2 ORDER BY created_at DESC, seq DESC LIMIT ?3)",
        params![user_id, hash, SESSIONS_PER_USER as i64 - 1],
    )?)
}

#[allow(clippy::too_many_arguments)]
async fn start_session(
    app: &App,
    headers: &HeaderMap,
    ip: IpAddr,
    user_id: &str,
    user_name: &str,
    is_admin: bool,
    policy: crate::access::Policy,
    perms: Perms,
) -> ApiResult<Response> {
    let mut raw = [0u8; 32];
    rand::rng().fill_bytes(&mut raw);
    let secure = is_https(headers);
    let token = format!("{}{}", if secure { BEGUN_ON_HTTPS } else { BEGUN_ON_HTTP }, hex::encode(raw));
    let hash = hash_token(&token);
    let ua =headers.get("user-agent").and_then(|v| v.to_str().ok()).map(|s| s.chars().take(200).collect::<String>());
    let (uid, uname, ip_s) = (user_id.to_string(), user_name.to_string(), ip.to_string());
    let changed = app
        .db
        .call(move |c| {
            let now = db::now();
            c.execute("DELETE FROM sessions WHERE expires_at <= ?1", [now])?;
            let row = remember_sign_in(c, &uid, &uname, is_admin, &policy, now)?;
            let let_go = store_session(c, &hash, &uid, &uname, is_admin, now, Some(&ip_s), ua.as_deref())?;
            Ok(row || let_go > 0)
        })
        .await?;
    // What Jellyfin just said about this person may have changed their row, and their oldest session may have given way to
    // this one: then their own connections look again, and nobody else's. A sign-in that changed nothing asks nothing.
    if changed {
        app.live_feed.recheck_person(user_id);
    }
    let body = Json(json!({ "user": user_json(app, user_id, user_name, is_admin, perms).await }));
    let mut resp = body.into_response();
    set_session(&mut resp, &token, SESSION_TTL_S, secure);
    Ok(resp)
}

/// The most addresses (and /48s) the limiter remembers at once. Past it a new one is refused until a window ends, never
/// an old one forgotten: forgetting the oldest let a guesser with more addresses than this take them in turn, each
/// forgotten before its tenth try.
const ATTEMPTS_REMEMBERED: usize = 10_000;
/// Tries per IPv6 /48 in a window, across all its /64s: a network is often given a whole /48, 65,536 /64s.
const WIDE_ATTEMPTS: u32 = 5 * MAX_ATTEMPTS;
/// Failed sign-ins a minute on the whole server past which nobody's attempt is forwarded until the minute has passed. A
/// household mistypes a few; this many is somebody guessing from more addresses than any limit per address can see.
const FAILURES_PER_MINUTE: usize = 60;

/// One address's (or one /48's) attempts in its window.
#[derive(Clone, Copy)]
struct Bucket {
    tries: u32,
    since: i64,
    /// Whether this window's first failure has been written to the audit log; the rest are not.
    audited: bool,
}

/// What the sign-in limiter remembers.
#[derive(Default)]
pub struct Attempts {
    buckets: HashMap<IpAddr, Bucket>,
    /// The same, per IPv6 /48.
    wide: HashMap<IpAddr, Bucket>,
    /// When each failure of the last minute happened, server-wide.
    failures: std::collections::VecDeque<i64>,
}

impl Attempts {
    /// How many addresses and /48s are remembered now.
    #[cfg(test)]
    pub fn remembered(&self) -> usize {
        self.buckets.len().max(self.wide.len())
    }

    fn forget_old(&mut self, now: i64) {
        self.buckets.retain(|_, b| now - b.since < ATTEMPT_WINDOW_S);
        self.wide.retain(|_, b| now - b.since < ATTEMPT_WINDOW_S);
        while self.failures.front().is_some_and(|t| now - t >= 60) {
            self.failures.pop_front();
        }
    }
}

/// Whose attempts an address counts as: an IPv4 address, or a whole IPv6 /64, which one machine holds and may
/// take any address in. An IPv4 address written inside IPv6 is that address.
fn attempt_bucket(ip: IpAddr) -> IpAddr {
    match ip {
        IpAddr::V6(v6) => match v6.to_ipv4_mapped() {
            Some(v4) => IpAddr::V4(v4),
            None => IpAddr::V6(std::net::Ipv6Addr::from(u128::from(v6) & !((1u128 << 64) - 1))),
        },
        v4 => v4,
    }
}

/// The IPv6 /48 an address is in, if it is IPv6.
fn wide_bucket(ip: IpAddr) -> Option<IpAddr> {
    match attempt_bucket(ip) {
        IpAddr::V6(v6) => Some(IpAddr::V6(std::net::Ipv6Addr::from(u128::from(v6) & !((1u128 << 80) - 1)))),
        IpAddr::V4(_) => None,
    }
}

fn too_many(wait_s: i64) -> ApiError {
    ApiError::new(StatusCode::TOO_MANY_REQUESTS, crate::i18n::t("server.auth.too_many_attempts", &[("count", ((wait_s.max(1) + 59) / 60).into())]))
}

/// Room for one more try in `map` under `key`, at most `max` a window: refused when spent, or when the map is full and
/// `key` is new. Nothing is counted here.
fn room(map: &HashMap<IpAddr, Bucket>, key: IpAddr, max: u32, now: i64) -> Result<(), ApiError> {
    match map.get(&key) {
        Some(b) if b.tries >= max => Err(too_many(ATTEMPT_WINDOW_S - (now - b.since))),
        Some(_) => Ok(()),
        None if map.len() >= ATTEMPTS_REMEMBERED => Err(too_many(ATTEMPT_WINDOW_S)),
        None => Ok(()),
    }
}

/// Sliding-window limiter against password guessing through FinStats: per address (an IPv6 /64), per IPv6 /48, and the
/// failures of the whole server in the last minute.
fn check_rate_limit(app: &App, ip: IpAddr) -> Result<(), ApiError> {
    let now = db::now();
    let mut a = app.login_attempts.lock().unwrap();
    a.forget_old(now);
    if a.failures.len() >= FAILURES_PER_MINUTE {
        return Err(too_many(60 - (now - a.failures.front().copied().unwrap_or(now))));
    }
    let key = attempt_bucket(ip);
    room(&a.buckets, key, MAX_ATTEMPTS, now)?;
    let wide = wide_bucket(ip);
    if let Some(w) = wide {
        room(&a.wide, w, WIDE_ATTEMPTS, now)?;
        a.wide.entry(w).or_insert(Bucket { tries: 0, since: now, audited: false }).tries += 1;
    }
    a.buckets.entry(key).or_insert(Bucket { tries: 0, since: now, audited: false }).tries += 1;
    Ok(())
}

/// A sign-in that worked is not a guess: it gives back its own attempt and nothing more. Clearing the address's count
/// here let anybody allowed in guess at any account without a limit, a sign-in of their own between every nine.
fn signed_in(app: &App, ip: IpAddr) {
    let mut a = app.login_attempts.lock().unwrap();
    if let Some(b) = a.buckets.get_mut(&attempt_bucket(ip)) {
        b.tries = b.tries.saturating_sub(1);
    }
    if let Some(b) = wide_bucket(ip).and_then(|w| a.wide.get_mut(&w)) {
        b.tries = b.tries.saturating_sub(1);
    }
}

/// A sign-in Jellyfin refused: counted against the whole server's minute. Answers whether it is this address's first
/// failure of its window, the one written to the audit log.
fn failed(app: &App, ip: IpAddr) -> bool {
    let now = db::now();
    let mut a = app.login_attempts.lock().unwrap();
    a.failures.push_back(now);
    match a.buckets.get_mut(&attempt_bucket(ip)) {
        Some(b) if !b.audited => {
            b.audited = true;
            true
        }
        Some(_) => false,
        None => true,
    }
}

/// Whether a sign-in as `name` may be put to Jellyfin: `Some(false)` when the name is nobody who may use FinStats (no such
/// user, disabled or gone in Jellyfin, or not let in), `None` while the users have never been read and so nobody is
/// known (a fresh install's first sign-ins, the wizard's own among them), which Jellyfin alone decides.
pub(crate) fn may_try(c: &Connection, settings: &crate::state::Settings, name: &str) -> anyhow::Result<Option<bool>> {
    let read: bool = c.query_row("SELECT EXISTS(SELECT 1 FROM task_runs WHERE task = 'sync_users' AND state = 'ok')", [], |r| r.get(0))?;
    if !read {
        return Ok(None);
    }
    let mut stmt = c.prepare("SELECT id, is_admin FROM users WHERE name = ?1 COLLATE NOCASE AND is_disabled = 0 AND removed = 0")?;
    let found: Vec<(String, bool)> = stmt.query_map([name], |r| Ok((r.get(0)?, r.get(1)?)))?.collect::<Result<_, _>>()?;
    for (id, is_admin) in found {
        let grants = if is_admin { vec![] } else { stored_grants(c, &id)? };
        if effective(is_admin, &grants, settings).is_some() {
            return Ok(Some(true));
        }
    }
    Ok(Some(false))
}

/// How long Jellyfin typically takes to turn a password down (it hashes what it was sent), so that a name refused without
/// asking it is answered no sooner: an answer that came at once would tell which names are somebody.
#[derive(Clone, Copy)]
pub(crate) struct FailureTime(std::time::Duration);

impl Default for FailureTime {
    fn default() -> Self {
        FailureTime(Self::FIRST_GUESS)
    }
}

impl FailureTime {
    /// Before Jellyfin has turned anybody down: about what a Jellyfin on modest hardware takes.
    pub const FIRST_GUESS: std::time::Duration = std::time::Duration::from_millis(400);
    /// No refusal waits longer than this, however slow one answer was.
    pub const LONGEST: std::time::Duration = std::time::Duration::from_secs(5);

    /// One more refusal Jellyfin took this long over: a running average, each new one a tenth of it.
    pub fn saw(&mut self, took: std::time::Duration) {
        let took = took.min(Self::LONGEST);
        self.0 = (self.0 * 9 + took) / 10;
    }

    pub fn typical(&self) -> std::time::Duration {
        self.0
    }
}

static FAILURE_TIME: std::sync::Mutex<FailureTime> = std::sync::Mutex::new(FailureTime(FailureTime::FIRST_GUESS));

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
        .ok_or_else(|| ApiError::new(StatusCode::CONFLICT, crate::i18n::t("server.auth.not_connected", &[])))?;
    let username = body.username.trim();
    if username.is_empty() {
        return Err(ApiError::bad_request(crate::i18n::t("server.auth.enter_username", &[])));
    }
    let ip = client_ip(&app, &headers, peer);
    check_rate_limit(&app, ip)?;
    // The name as typed: it is the administrator's own server, and Jellyfin's log says the same. Once per address and
    // window: a guesser's every try was a row more, and Jellyfin's own log has each.
    let failure = |app: &App| {
        if failed(app, ip) {
            let who = Actor { user_id: None, user_name: Some(username.chars().take(100).collect()), ip: Some(ip), key_id: None };
            audit::record(app, audit::Entry::new("sign_in_failed", who).outcome("failed"));
        }
        ApiError::new(StatusCode::UNAUTHORIZED, crate::i18n::t("server.auth.wrong_password", &[]))
    };

    // Only a name that may use FinStats is put to Jellyfin, which counts every failure against the account (and locks it),
    // judges its "remote access" on FinStats' own address and logs FinStats as the one trying. Anybody else is told what a
    // wrong password is told, no sooner than Jellyfin would have told them.
    let (settings, typed) = (app.settings(), username.to_string());
    if app.db.call(move |c| may_try(c, &settings, &typed)).await? == Some(false) {
        let typical = FAILURE_TIME.lock().unwrap().typical();
        let jitter = rand::Rng::random_range(&mut rand::rng(), 0.85..1.15);
        tokio::time::sleep(typical.mul_f64(jitter)).await;
        return Err(failure(&app));
    }

    let jf = app.jellyfin_anonymous(jf.base());
    let asked = std::time::Instant::now();
    let auth = match jf.authenticate(username, &body.password).await {
        Ok(a) => a,
        Err(AuthError::InvalidCredentials) => {
            FAILURE_TIME.lock().unwrap().saw(asked.elapsed());
            return Err(failure(&app));
        }
        Err(AuthError::Other(e)) => {
            tracing::warn!("login could not be checked against Jellyfin: {e:#}");
            return Err(ApiError::new(StatusCode::BAD_GATEWAY, crate::i18n::t("server.auth.unreachable", &[])));
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
            crate::i18n::t("server.auth.no_access", &[]),
        ));
    };
    signed_in(&app, ip);
    tracing::info!("{} signed in from {ip}", auth.user_name);
    audit::record(&app, audit::Entry::new("sign_in", actor));
    start_session(&app, &headers, ip, &auth.user_id, &auth.user_name, auth.is_admin, auth.policy, perms).await
}

/// `POST /api/auth/logout`: ends every session the browser sent, the `__Host-` cookie and each plain one. With two plain
/// cookies (one tossed in beside ours) `cookie_token` takes neither, and a sign-out that ended nothing answered `ok`.
pub async fn logout(State(app): State<App>, ConnectInfo(peer): ConnectInfo<SocketAddr>, headers: HeaderMap) -> ApiResult<Response> {
    let mut hashes: Vec<String> = [HOST_COOKIE_NAME, COOKIE_NAME].iter().flat_map(|n| cookies_named(&headers, n)).map(|t| hash_token(&t)).collect();
    hashes.dedup();
    hashes.truncate(4);
    if !hashes.is_empty() {
        let ip = client_ip(&app, &headers, peer);
        let ended = app
            .db
            .call(move |c| {
                let mut ended = vec![];
                for hash in hashes {
                    let who: Option<(String, String)> = c
                        .query_row("DELETE FROM sessions WHERE token_hash = ?1 RETURNING user_id, user_name", [hash], |r| Ok((r.get(0)?, r.get(1)?)))
                        .optional()?;
                    if let Some((id, name)) = who {
                        audit::record_quietly(c, &audit::Entry::new("sign_out", Actor { user_id: Some(id.clone()), user_name: Some(name), ip: Some(ip), key_id: None }));
                        ended.push(id);
                    }
                }
                Ok(ended)
            })
            .await?;
        // The tab signing out is not the only one this session opened: every live connection of it closes now. Only that
        // person's, and only when a session ended, or anybody could make every open connection read its access again.
        for user_id in ended {
            app.live_feed.recheck_person(&user_id);
        }
    }
    let mut resp = Json(json!({ "ok": true })).into_response();
    set_session(&mut resp, "", 0, is_https(&headers));
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
    // asking. Never who, never what: no names, no titles, not even how many sessions there are.
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
        return Err(ApiError::new(StatusCode::CONFLICT, crate::i18n::t("server.auth.already_set_up", &[])));
    }
    Ok(())
}

/// Letters and digits nobody misreads: no 0/O, 1/I/L.
const SETUP_ALPHABET: &[u8] = b"ABCDEFGHJKMNPQRSTUVWXYZ23456789";

/// A fresh install's one-time setup code, `XXXX-XXXX`, written to the log at start. Until setup is finished the wizard is
/// open to whoever reaches the port, who could otherwise point FinStats at a Jellyfin of their own and be its
/// administrator, or use the test to look around the network; the log is the owner's alone.
pub fn new_setup_code() -> String {
    let mut rng = rand::rng();
    let pick = |rng: &mut rand::rngs::ThreadRng| SETUP_ALPHABET[rand::Rng::random_range(rng, 0..SETUP_ALPHABET.len())] as char;
    let half = |rng: &mut rand::rngs::ThreadRng| (0..4).map(|_| pick(rng)).collect::<String>();
    format!("{}-{}", half(&mut rng), half(&mut rng))
}

/// Whether `given` is the setup code, however it was typed (case, dash, spaces), compared in constant time.
fn setup_code_matches(expected: Option<&str>, given: &str) -> bool {
    let plain = |s: &str| s.chars().filter(char::is_ascii_alphanumeric).map(|c| c.to_ascii_uppercase()).collect::<String>();
    let Some(expected) = expected.map(plain).filter(|e| !e.is_empty()) else { return false };
    let given = plain(given);
    expected.len() == given.len() && expected.bytes().zip(given.bytes()).fold(0u8, |acc, (a, b)| acc | (a ^ b)) == 0
}

fn ensure_setup_code(app: &App, given: &str) -> Result<(), ApiError> {
    if setup_code_matches(app.setup_code.as_deref(), given) {
        return Ok(());
    }
    Err(ApiError::new(StatusCode::PRECONDITION_REQUIRED, crate::i18n::t("server.auth.setup_code", &[])))
}

#[derive(Deserialize)]
pub struct SetupTestBody {
    #[serde(default)]
    url: String,
    #[serde(default)]
    code: String,
}

pub async fn setup_test(
    State(app): State<App>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(body): Json<SetupTestBody>,
) -> ApiResult {
    ensure_unconfigured(&app)?;
    check_rate_limit(&app, client_ip(&app, &headers, peer))?;
    ensure_setup_code(&app, &body.code)?;
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
    #[serde(default)]
    code: String,
}

/// Held from the last look at whether FinStats is set up to the config being written: two people finishing the wizard
/// at once both found it not set up, and the second overwrote the first's Jellyfin.
static SETUP_FINISHING: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

pub async fn setup(
    State(app): State<App>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(body): Json<SetupBody>,
) -> ApiResult<Response> {
    ensure_unconfigured(&app)?;
    let ip = client_ip(&app, &headers, peer);
    check_rate_limit(&app, ip)?;
    ensure_setup_code(&app, &body.code)?;
    let url = jellyfin::normalize_url(&body.url).map_err(|e| ApiError::bad_request(e.to_string()))?;
    let jf = app.jellyfin_anonymous(&url);
    let info = jf.public_info().await.map_err(|e| ApiError::new(StatusCode::BAD_GATEWAY, e.to_string()))?;

    let auth = match jf.authenticate(body.username.trim(), &body.password).await {
        Ok(a) => a,
        Err(AuthError::InvalidCredentials) => {
            return Err(ApiError::new(StatusCode::UNAUTHORIZED, crate::i18n::t("server.auth.wrong_password", &[])));
        }
        Err(AuthError::Other(e)) => return Err(ApiError::new(StatusCode::BAD_GATEWAY, e.to_string())),
    };
    if !auth.is_admin {
        jf.logout(&auth.access_token).await;
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            crate::i18n::t("server.auth.not_admin", &[]),
        ));
    }
    let key = jf.ensure_api_key(&auth.access_token).await;
    jf.logout(&auth.access_token).await;
    let api_key = key.map_err(|e| ApiError::new(StatusCode::BAD_GATEWAY, e.to_string()))?;

    // Two people finishing the wizard at once: the second finds it set up.
    let _finishing = SETUP_FINISHING.lock().await;
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
    signed_in(&app, ip);
    tracing::info!("setup completed by {}", auth.user_name);
    audit::record(&app, audit::Entry::new("setup_completed", Actor { user_id: Some(auth.user_id.clone()), user_name: Some(auth.user_name.clone()), ip: Some(ip), key_id: None }));
    app.wake.notify_waiters();

    start_session(&app, &headers, ip, &auth.user_id, &auth.user_name, true, auth.policy, Perms::ALL).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::Settings;

    /// A proxy adds the address it saw to the end of `X-Forwarded-For` and keeps whatever the client
    /// sent in front of it, so only the last entry is the proxy's word. Reading the first let anybody
    /// pick a new address per sign-in attempt, and never meet the limit.
    #[test]
    fn behind_a_proxy_the_address_is_the_one_the_proxy_saw() {
        let mut app = crate::state::test_app();
        std::sync::Arc::get_mut(&mut app).unwrap().trust_proxy = Proxies::parse("1").unwrap();
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", "10.0.0.7, 203.0.113.9".parse().unwrap());
        let peer: SocketAddr = "127.0.0.1:40000".parse().unwrap();
        assert_eq!(client_ip(&app, &headers, peer).to_string(), "203.0.113.9");
        headers.insert("x-forwarded-for", "203.0.113.9".parse().unwrap());
        assert_eq!(client_ip(&app, &headers, peer).to_string(), "203.0.113.9");
        headers.insert("x-forwarded-for", "203.0.113.9, not-an-address".parse().unwrap());
        assert_eq!(client_ip(&app, &headers, peer).to_string(), "127.0.0.1", "a last entry that is not an address is not trusted");
        // Some proxies add a line of their own rather than extending the client's, and the client's then comes first.
        let mut headers = HeaderMap::new();
        headers.append("x-forwarded-for", "198.51.100.1".parse().unwrap());
        headers.append("x-forwarded-for", "203.0.113.9".parse().unwrap());
        assert_eq!(client_ip(&app, &headers, peer).to_string(), "203.0.113.9", "the client's own line was believed");
        // `1` believes a peer on this machine or this network, never one from outside: with the port open as well as the
        // proxy, anybody else picked their own address with the header.
        let outside: SocketAddr = "198.51.100.20:40000".parse().unwrap();
        assert_eq!(client_ip(&app, &headers, outside).to_string(), "198.51.100.20");
        // An entry with a port is the address before it.
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", "203.0.113.9:51234".parse().unwrap());
        assert_eq!(client_ip(&app, &headers, peer).to_string(), "203.0.113.9");
        headers.insert("x-forwarded-for", "[2001:db8::7]:443".parse().unwrap());
        assert_eq!(client_ip(&app, &headers, peer).to_string(), "2001:db8::7");
    }

    /// Named proxies are believed and nobody else: the header is read from the right, past every hop that is a proxy
    /// named, and the first that is not is the client. Anything before it is whatever the client wrote.
    #[test]
    fn only_a_named_proxy_is_believed_and_its_hops_are_walked_from_the_right() {
        let mut app = crate::state::test_app();
        std::sync::Arc::get_mut(&mut app).unwrap().trust_proxy = Proxies::parse("172.18.0.0/16, 10.0.0.2").unwrap();
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", "198.51.100.1, 203.0.113.9, 10.0.0.2".parse().unwrap());
        let proxy: SocketAddr = "172.18.0.5:40000".parse().unwrap();
        assert_eq!(client_ip(&app, &headers, proxy).to_string(), "203.0.113.9", "two proxies, then the client");
        let other: SocketAddr = "192.168.1.10:40000".parse().unwrap();
        assert_eq!(client_ip(&app, &headers, other).to_string(), "192.168.1.10", "a peer outside the list is itself, whatever it says");
        headers.insert("x-forwarded-for", "10.0.0.2".parse().unwrap());
        assert_eq!(client_ip(&app, &headers, proxy).to_string(), "10.0.0.2", "nothing but proxies: the last one");
        assert!(Proxies::parse("10.0.0.0/33").is_err() && Proxies::parse("proxy.lan").is_err(), "a list FinStats cannot read is refused at start");
        assert!(!Proxies::parse("").unwrap().any() && !Proxies::parse("0").unwrap().any() && !Proxies::parse("false").unwrap().any());
        assert!(Proxies::parse("true").unwrap().any());
    }

    /// Signing out with a cookie that opens nothing asks no open connection to re-read its access: anybody could do it
    /// in a loop, signed out, and each request cost a database read for every tab open anywhere.
    #[tokio::test]
    async fn a_sign_out_that_ends_no_session_asks_nobody_to_look_again() {
        let app = crate::state::test_app();
        let peer: SocketAddr = "192.0.2.7:4000".parse().unwrap();
        let with = |token: &str| {
            let mut h = HeaderMap::new();
            h.insert(axum::http::header::COOKIE, format!("{COOKIE_NAME}={token}").parse().unwrap());
            h
        };
        let (alice, bob) = (app.live_feed.seats.take("ua", None), app.live_feed.seats.take("ub", None));
        let before = app.live_feed.rechecks();
        logout(State(app.clone()), ConnectInfo(peer), with("not-a-session")).await.unwrap();
        assert!(!asked(&alice).await && !asked(&bob).await);
        let hash = hash_token("a-real-one");
        app.db.call(move |c| Ok(c.execute("INSERT INTO sessions(token_hash, user_id, user_name, is_admin, created_at, expires_at) VALUES (?1, 'ua', 'alice', 0, 1, 9999999999)", [hash])?)).await.unwrap();
        logout(State(app.clone()), ConnectInfo(peer), with("a-real-one")).await.unwrap();
        assert!(asked(&alice).await, "the connections of the session that ended look");
        assert!(!asked(&bob).await, "and nobody else's");
        assert_eq!(app.live_feed.rechecks(), before, "not everybody's");
    }

    /// A session begun on https is marked so in its token, and one begun on http too, so the plain cookie name can refuse
    /// the first.
    #[tokio::test]
    async fn a_session_token_says_whether_it_was_begun_on_https() {
        let app = crate::state::test_app();
        let ip: IpAddr = "192.0.2.7".parse().unwrap();
        let token = |resp: Response| resp.headers().get(SET_COOKIE).unwrap().to_str().unwrap().split(';').next().unwrap().to_string();
        let mut https = HeaderMap::new();
        https.insert("origin", "https://stats.example.com".parse().unwrap());
        assert!(token(start_session(&app, &https, ip, "u1", "alice", false, crate::access::policy(&json!({ "Policy": { "EnableAllFolders": true } })), Perms::default()).await.unwrap()).starts_with("__Host-finstats_session=h"));
        assert!(token(start_session(&app, &HeaderMap::new(), ip, "u1", "alice", false, crate::access::policy(&json!({ "Policy": { "EnableAllFolders": true } })), Perms::default()).await.unwrap()).starts_with("finstats_session=p"));
    }

    /// A sign-out ends every session it was sent, two plain cookies included: answering `ok` while ending none left the
    /// browser signed in to whichever one won next.
    #[tokio::test]
    async fn a_sign_out_ends_every_session_it_was_sent() {
        let app = crate::state::test_app();
        let peer: SocketAddr = "192.0.2.7:4000".parse().unwrap();
        for t in ["p-one", "p-two"] {
            let hash = hash_token(t);
            app.db.call(move |c| Ok(c.execute("INSERT INTO sessions(token_hash, user_id, user_name, is_admin, created_at, expires_at) VALUES (?1, 'ua', 'alice', 0, 1, 9999999999)", [hash])?)).await.unwrap();
        }
        let mut h = HeaderMap::new();
        h.insert(COOKIE, HeaderValue::from_static("finstats_session=p-one; finstats_session=p-two"));
        logout(State(app.clone()), ConnectInfo(peer), h).await.unwrap();
        let left: i64 = app.db.call(|c| Ok(c.query_row("SELECT COUNT(*) FROM sessions", [], |r| r.get(0))?)).await.unwrap();
        assert_eq!(left, 0);
    }

    /// A name that may not use FinStats is answered as a wrong password, and Jellyfin hears nothing of it.
    #[tokio::test]
    async fn a_name_that_may_not_sign_in_never_reaches_jellyfin() {
        let app = crate::state::test_app();
        let jellyfin = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", jellyfin.local_addr().unwrap());
        *app.config.write().unwrap() = Some(JfConfig { url, api_key: "k".into(), server_name: "Jellyfin".into(), server_version: String::new(), from_env: false });
        app.db.call(|c| Ok(c.execute_batch("INSERT INTO users(id, name, is_admin, updated_at) VALUES ('u2', 'bob', 0, 1);
                                             INSERT INTO task_runs(task, state, started_at, finished_at) VALUES ('sync_users', 'ok', 1, 2);")?)).await.unwrap();
        let peer: SocketAddr = "192.0.2.7:4000".parse().unwrap();
        for name in ["bob", "mallory"] {
            let body = Json(LoginBody { username: name.into(), password: "x".into() });
            let err = login(State(app.clone()), ConnectInfo(peer), HeaderMap::new(), body).await.expect_err("refused");
            assert_eq!((err.0, err.1), (StatusCode::UNAUTHORIZED, crate::i18n::t("server.auth.wrong_password", &[])), "{name}");
        }
        let asked = tokio::time::timeout(std::time::Duration::from_millis(50), jellyfin.accept()).await;
        assert!(asked.is_err(), "Jellyfin was asked about a name that may not sign in");
    }

    /// Whether a connection was asked to read its access again.
    async fn asked(t: &crate::live::Ticket) -> bool {
        tokio::time::timeout(std::time::Duration::from_millis(20), t.recheck.notified()).await.is_ok()
    }

    /// A sign-in asks that person's own connections to look again only when it changed something they hold: their row
    /// (what Jellyfin just said about them), or an older session that gave way. Every sign-in asked every connection on
    /// the server, so somebody signing in in a loop with their own password made every open tab ask the database again.
    #[tokio::test]
    async fn a_sign_in_asks_its_own_connections_to_look_again_only_when_it_changed_something() {
        let app = crate::state::test_app();
        let (headers, ip): (HeaderMap, IpAddr) = (HeaderMap::new(), "192.0.2.7".parse().unwrap());
        let (alice, bob) = (app.live_feed.seats.take("u1", None), app.live_feed.seats.take("u2", None));
        let everybody = app.live_feed.rechecks();
        let sign_in = || start_session(&app, &headers, ip, "u1", "alice", false, crate::access::policy(&json!({ "Policy": { "EnableAllFolders": true } })), Perms::default());
        sign_in().await.unwrap();
        assert!(asked(&alice).await, "a row written for the first time");
        sign_in().await.unwrap();
        assert!(!asked(&alice).await, "nothing changed");
        for _ in 2..SESSIONS_PER_USER {
            sign_in().await.unwrap();
        }
        assert!(!asked(&alice).await, "thirty sessions, none given way");
        sign_in().await.unwrap();
        assert!(asked(&alice).await, "the oldest session gave way to the thirty-first");
        assert!(!asked(&bob).await);
        assert_eq!(app.live_feed.rechecks(), everybody, "nobody else was asked");
    }

    /// A sign-in of one's own takes back that attempt alone. Clearing the address's count let anybody with an account
    /// guess nine passwords, sign in as themselves, and guess nine more, for ever, at any account.
    #[test]
    fn a_sign_in_of_ones_own_does_not_forgive_the_guesses_before_it() {
        let app = crate::state::test_app();
        let ip: IpAddr = "192.0.2.7".parse().unwrap();
        for _ in 0..MAX_ATTEMPTS - 1 {
            check_rate_limit(&app, ip).unwrap();
        }
        check_rate_limit(&app, ip).unwrap();
        signed_in(&app, ip);
        check_rate_limit(&app, ip).unwrap();
        assert!(check_rate_limit(&app, ip).is_err(), "the guesses before a sign-in were forgotten");
    }

    /// One machine on IPv6 holds a whole /64 and may sign in from any address in it, so the limit is the /64's; an IPv4
    /// address written inside IPv6 is that address.
    #[test]
    fn the_sign_in_limit_counts_an_ipv6_network_as_one_address() {
        let app = crate::state::test_app();
        let ip = |s: &str| s.parse::<IpAddr>().unwrap();
        for n in 0..MAX_ATTEMPTS {
            assert!(check_rate_limit(&app, ip(&format!("2001:db8::{:x}", n + 1))).is_ok());
        }
        assert!(check_rate_limit(&app, ip("2001:db8::ffff")).is_err(), "a new address in the same /64 began again");
        assert!(check_rate_limit(&app, ip("2001:db8:0:1::1")).is_ok(), "the next /64 is somebody else");
        for _ in 0..MAX_ATTEMPTS {
            let _ = check_rate_limit(&app, ip("192.0.2.7"));
        }
        assert!(check_rate_limit(&app, ip("::ffff:192.0.2.7")).is_err());
    }

    /// A flood of new addresses fills the limiter, and then a new one is refused rather than an old one forgotten:
    /// letting the oldest go meant ten thousand and one /64s, taken in turn, each forgotten before its tenth try, guessed
    /// without any limit.
    #[test]
    fn a_full_limiter_refuses_newcomers_rather_than_forgetting_the_spent() {
        let app = crate::state::test_app();
        let spent: IpAddr = "192.0.2.7".parse().unwrap();
        for _ in 0..MAX_ATTEMPTS {
            check_rate_limit(&app, spent).unwrap();
        }
        for n in 0..ATTEMPTS_REMEMBERED as u32 {
            let _ = check_rate_limit(&app, IpAddr::V4(std::net::Ipv4Addr::from(0x0a00_0000 + n)));
        }
        assert!(app.login_attempts.lock().unwrap().remembered() <= ATTEMPTS_REMEMBERED, "it grew without end");
        assert!(check_rate_limit(&app, spent).is_err(), "an address that spent its tries was forgotten");
        assert!(check_rate_limit(&app, "198.51.100.1".parse().unwrap()).is_err(), "a newcomer was let in by forgetting another");
    }

    /// One network is often given a whole IPv6 /48 (65,536 /64s), so the /48 has a limit of its own, a few households wide.
    #[test]
    fn an_ipv6_48_has_a_wider_limit_of_its_own() {
        let app = crate::state::test_app();
        for n in 0..WIDE_ATTEMPTS {
            assert!(check_rate_limit(&app, format!("2001:db8:1:{:x}::1", n + 1).parse().unwrap()).is_ok(), "{n}");
        }
        assert!(check_rate_limit(&app, "2001:db8:1:ffff::1".parse().unwrap()).is_err(), "a fresh /64 in a spent /48");
        assert!(check_rate_limit(&app, "2001:db8:2::1".parse().unwrap()).is_ok(), "the next /48 is somebody else");
    }

    /// Past so many failures a minute on the whole server, nobody's attempt is forwarded until the minute has passed:
    /// the limit per address does nothing against a guesser with thousands of addresses.
    #[test]
    fn so_many_failures_a_minute_anywhere_stop_every_attempt() {
        let app = crate::state::test_app();
        for n in 0..FAILURES_PER_MINUTE as u32 {
            let ip = IpAddr::V4(std::net::Ipv4Addr::from(0x0a00_0000 + n));
            check_rate_limit(&app, ip).unwrap();
            failed(&app, ip);
        }
        assert!(check_rate_limit(&app, "198.51.100.1".parse().unwrap()).is_err());
    }

    /// A failed sign-in is written to the audit log once per address and window: each try wrote a row, and a guesser's
    /// every try was a row more in the log.
    #[test]
    fn a_failure_is_audited_once_per_address_and_window() {
        let app = crate::state::test_app();
        let ip: IpAddr = "192.0.2.7".parse().unwrap();
        check_rate_limit(&app, ip).unwrap();
        assert!(failed(&app, ip), "the first");
        check_rate_limit(&app, ip).unwrap();
        assert!(!failed(&app, ip), "the second, in the same window");
        let other: IpAddr = "192.0.2.8".parse().unwrap();
        check_rate_limit(&app, other).unwrap();
        assert!(failed(&app, other), "another address's first");
    }

    /// A session cookie is Secure whenever the page signing in is on https. A proxy says so with `X-Forwarded-Proto`, but
    /// many never send it (nginx does not unless told to), and then a cookie good for thirty days went out without Secure.
    /// The browser says it too: a sign-in is a POST, which always carries the page's own `Origin`.
    #[test]
    fn a_session_cookie_is_secure_whenever_the_page_is_on_https() {
        let mut h = HeaderMap::new();
        assert!(!is_https(&h));
        h.insert("origin", "https://stats.example.com".parse().unwrap());
        assert!(is_https(&h), "a page on https, behind a proxy that says nothing");
        h.insert("origin", "http://192.168.1.10:8080".parse().unwrap());
        assert!(!is_https(&h), "a page on plain http: a Secure cookie would never be kept");
        h.insert("x-forwarded-proto", "https".parse().unwrap());
        assert!(is_https(&h));
    }

    /// The setup code is eight letters and digits nobody misreads, and is taken however it is typed.
    #[test]
    fn the_setup_code_is_taken_however_it_is_typed_and_nothing_else_is() {
        let code = new_setup_code();
        assert_eq!(code.len(), 9);
        assert_eq!(&code[4..5], "-");
        assert!(code.bytes().filter(|b| *b != b'-').all(|b| SETUP_ALPHABET.contains(&b)), "{code}");
        assert_ne!(new_setup_code(), new_setup_code());
        let right = Some("7KQ2-M9XD");
        for typed in ["7KQ2-M9XD", "7kq2m9xd", " 7KQ2 M9XD "] {
            assert!(setup_code_matches(right, typed), "{typed:?}");
        }
        for typed in ["", "7KQ2-M9XE", "7KQ2", "7KQ2-M9XD-7KQ2"] {
            assert!(!setup_code_matches(right, typed), "{typed:?}");
        }
        assert!(!setup_code_matches(None, ""), "no code at all opens nothing");
    }

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
    fn a_session_follows_the_user_as_jellyfin_has_them_now() {
        let c = conn();
        c.execute("INSERT INTO sessions(token_hash, user_id, user_name, is_admin, created_at, expires_at) VALUES (?1, 'u1', 'alice', 1, 1, 9999999999)", [hash_token("alice-cookie")]).unwrap();
        assert_eq!(resolve_session_in(&c, &open(), "alice-cookie", 1000).unwrap().unwrap().perms, Perms::ALL);
        c.execute_batch("UPDATE users SET is_admin = 0 WHERE id = 'u1'").unwrap();
        let demoted = resolve_session_in(&c, &open(), "alice-cookie", 1000).unwrap().expect("a demoted administrator may still sign in");
        assert!(!demoted.is_admin && demoted.perms == Perms::default(), "demoted in Jellyfin, the session signed in as an administrator demotes with them");
        c.execute_batch("UPDATE users SET is_disabled = 1 WHERE id = 'u1'").unwrap();
        assert!(resolve_session_in(&c, &open(), "alice-cookie", 1000).unwrap().is_none(), "disabled in Jellyfin");
        c.execute_batch("UPDATE users SET is_disabled = 0, removed = 1 WHERE id = 'u1'").unwrap();
        assert!(resolve_session_in(&c, &open(), "alice-cookie", 1000).unwrap().is_none(), "removed from Jellyfin");
        // The wizard signs its administrator in before the first read of the users has written their row:
        // until there is one, the session is what Jellyfin said at sign-in.
        c.execute("INSERT INTO sessions(token_hash, user_id, user_name, is_admin, created_at, expires_at) VALUES (?1, 'u9', 'carol', 1, 1, 9999999999)", [hash_token("carol-cookie")]).unwrap();
        let first = resolve_session_in(&c, &open(), "carol-cookie", 1000).unwrap().expect("no users row yet");
        assert!(first.is_admin && first.name == "carol");
    }

    #[test]
    fn one_person_keeps_a_bounded_number_of_sessions_and_the_oldest_give_way() {
        let c = conn();
        for n in 0..(SESSIONS_PER_USER as i64 + 15) {
            store_session(&c, &hash_token(&format!("t{n}")), "u2", "bob", false, 1000 + n, None, None).unwrap();
        }
        store_session(&c, &hash_token("alice"), "u1", "alice", true, 5000, None, None).unwrap();
        let bob: i64 = c.query_row("SELECT COUNT(*) FROM sessions WHERE user_id = 'u2'", [], |r| r.get(0)).unwrap();
        assert_eq!(bob, SESSIONS_PER_USER as i64, "one person's sign-ins are not bounded");
        let newest = SESSIONS_PER_USER as i64 + 14;
        assert!(resolve_session_in(&c, &open(), &format!("t{newest}"), 2000).unwrap().is_some(), "the newest was dropped");
        assert!(resolve_session_in(&c, &open(), "t0", 2000).unwrap().is_none(), "the oldest is still open");
        assert!(resolve_session_in(&c, &open(), "alice", 6000).unwrap().is_some(), "somebody else's session gave way");
    }

    /// A sign-in loop fits dozens of sessions into one second, and `created_at` is whole seconds: which of
    /// them gave way was decided by the token's hash, so the oldest could outlive the limit.
    #[test]
    fn within_one_second_it_is_still_the_oldest_session_that_gives_way() {
        let c = conn();
        for n in 0..(SESSIONS_PER_USER + 20) {
            store_session(&c, &hash_token(&format!("s{n}")), "u2", "bob", false, 1000, None, None).unwrap();
        }
        let open_now = |t: &str| resolve_session_in(&c, &open(), t, 1000).unwrap().is_some();
        let kept: Vec<usize> = (0..SESSIONS_PER_USER + 20).filter(|n| open_now(&format!("s{n}"))).collect();
        assert_eq!(kept, (20..SESSIONS_PER_USER + 20).collect::<Vec<_>>(), "the newest thirty, in the order they were made");
    }

    #[test]
    fn signing_in_writes_down_what_jellyfin_just_said_about_them() {
        let c = conn();
        // A Jellystat import writes a row for somebody the users read has not reached yet, with the
        // file's word for whether they administer Jellyfin. The file is not Jellyfin.
        c.execute_batch("INSERT INTO users(id, name, is_admin, updated_at) VALUES ('u3', 'dave', 1, 0);").unwrap();
        remember_sign_in(&c, "u3", "dave", false, &every_library(), 1000).unwrap();
        c.execute("INSERT INTO sessions(token_hash, user_id, user_name, is_admin, created_at, expires_at) VALUES (?1, 'u3', 'dave', 0, 1, 9999999999)", [hash_token("dave-cookie")]).unwrap();
        assert!(!resolve_session_in(&c, &open(), "dave-cookie", 1000).unwrap().unwrap().is_admin, "an imported row crowned somebody Jellyfin did not");
        // Jellyfin has just let them in, so they are there and not disabled, whatever the last read said.
        c.execute_batch("UPDATE users SET removed = 1, is_disabled = 1 WHERE id = 'u2'").unwrap();
        remember_sign_in(&c, "u2", "bob", false, &every_library(), 1000).unwrap();
        let bob: (i64, i64, i64) = c.query_row("SELECT is_admin, is_disabled, removed FROM users WHERE id = 'u2'", [], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))).unwrap();
        assert_eq!(bob, (0, 0, 0));
        // And somebody the read has never seen gets a row of their own.
        remember_sign_in(&c, "u9", "carol", true, &every_library(), 1000).unwrap();
        let carol: (String, i64) = c.query_row("SELECT name, is_admin FROM users WHERE id = 'u9'", [], |r| Ok((r.get(0)?, r.get(1)?))).unwrap();
        assert_eq!(carol, ("carol".to_string(), 1));
        // What of the library Jellyfin lets them see comes with them, so somebody new is not shut out until the users read.
        let films = crate::access::policy(&json!({ "Policy": { "EnableAllFolders": false, "EnabledFolders": ["films"] } }));
        remember_sign_in(&c, "u8", "erin", false, &films, 1000).unwrap();
        let erin = crate::access::Access::of(&c, "u8", false).unwrap();
        assert!(erin.may_see_library("films") && !erin.may_see_library("adult"));
    }

    fn every_library() -> crate::access::Policy {
        crate::access::policy(&json!({ "Policy": { "EnableAllFolders": true } }))
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

    /// A host on a sibling name may set a cookie for the whole domain, sent first: a victim then worked inside the
    /// attacker's account, adding their webhooks and keys to it. On https the session is a `__Host-` cookie, which only
    /// this host can set, and it wins; two cookies of the plain name are one too many to choose between.
    #[test]
    fn a_cookie_another_host_could_have_set_does_not_win() {
        let with = |cookies: &'static str| {
            let mut h = HeaderMap::new();
            h.insert(COOKIE, HeaderValue::from_static(cookies));
            cookie_token(&h)
        };
        assert_eq!(with("finstats_session=tossed; __Host-finstats_session=mine").as_deref(), Some("mine"));
        assert_eq!(with("finstats_session=tossed; finstats_session=mine"), None);
        assert_eq!(with("finstats_session=mine").as_deref(), Some("mine"), "plain http, and sessions from before");
        // A session begun on https says so in its token, and is never taken under the plain name: before the browser holds
        // a `__Host-` cookie (or once it signed out), a sibling's tossed cookie was the only one sent, and won.
        assert_eq!(with("finstats_session=h0123abcd"), None, "an https session under the plain name");
        assert_eq!(with("__Host-finstats_session=h0123abcd").as_deref(), Some("h0123abcd"));
        assert_eq!(with("finstats_session=p0123abcd").as_deref(), Some("p0123abcd"), "a session begun on http");
        let set = session_cookie("t", 60, true);
        assert!(set.to_str().unwrap().starts_with("__Host-finstats_session=t; Path=/;") && set.to_str().unwrap().contains("Secure"));
        assert!(session_cookie("t", 60, false).to_str().unwrap().starts_with("finstats_session=t;"));
    }

    #[test]
    fn the_header_beats_the_cookie() {
        let mut h = HeaderMap::new();
        h.insert(COOKIE, HeaderValue::from_static("finstats_session=cookie-token"));
        assert_eq!(pick_credential(&h).unwrap(), Presented::Cookie("cookie-token".into()));
        h.insert(AUTHORIZATION, HeaderValue::from_static("Bearer fs_abc"));
        assert_eq!(pick_credential(&h).unwrap(), Presented::Bearer("fs_abc".into()), "an explicit header, not the ambient cookie");
        h.insert(AUTHORIZATION, HeaderValue::from_static("Bearer"));
        assert_eq!(pick_credential(&h).unwrap_err().0, StatusCode::UNAUTHORIZED, "a key sent wrong never falls back to the cookie");
        // A proxy's own sign-in (nginx's auth_basic) passes `Authorization: Basic` along with every request: that is not
        // FinStats' to judge, and the cookie beside it is the browser's.
        h.insert(AUTHORIZATION, HeaderValue::from_static("Basic dXNlcjpwYXNz"));
        assert_eq!(pick_credential(&h).unwrap(), Presented::Cookie("cookie-token".into()));
        let mut basic_alone = HeaderMap::new();
        basic_alone.insert(AUTHORIZATION, HeaderValue::from_static("Basic dXNlcjpwYXNz"));
        assert_eq!(pick_credential(&basic_alone).unwrap_err().0, StatusCode::UNAUTHORIZED);
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

    /// Before Jellyfin is asked, the name typed must be somebody who may use FinStats: Jellyfin counts every failure
    /// against the account (and locks it), judges "remote access" on FinStats' own address, and logs FinStats as the one
    /// guessing. Until the users have been read once, nobody is known, so everybody is asked about as before.
    #[test]
    fn only_a_name_that_may_use_finstats_is_put_to_jellyfin() {
        let c = conn();
        let closed = Settings::default();
        assert_eq!(may_try(&c, &closed, "nobody").unwrap(), None, "no users read yet: Jellyfin decides, as for the wizard's first sign-in");
        c.execute_batch("INSERT INTO task_runs(task, state, started_at, finished_at) VALUES ('sync_users', 'ok', 1, 2)").unwrap();
        assert_eq!(may_try(&c, &closed, "nobody").unwrap(), Some(false), "a name that is nobody");
        assert_eq!(may_try(&c, &closed, "ALICE").unwrap(), Some(true), "an administrator, however the name is typed");
        assert_eq!(may_try(&c, &closed, "bob").unwrap(), Some(false), "not allowed in");
        assert_eq!(may_try(&c, &open(), "bob").unwrap(), Some(true), "sign-in open to everybody");
        c.execute_batch("UPDATE users SET is_disabled = 1 WHERE id = 'u1'").unwrap();
        assert_eq!(may_try(&c, &closed, "alice").unwrap(), Some(false), "disabled in Jellyfin");
        c.execute_batch("UPDATE users SET is_disabled = 0, removed = 1 WHERE id = 'u1'").unwrap();
        assert_eq!(may_try(&c, &closed, "alice").unwrap(), Some(false), "gone from Jellyfin");
    }

    /// A name refused without asking Jellyfin waits about as long as one Jellyfin turned down, or how fast the answer came
    /// would tell which names are somebody.
    #[test]
    fn a_refusal_takes_about_as_long_as_jellyfin_takes_to_refuse() {
        let mut took = FailureTime::default();
        assert_eq!(took.typical(), FailureTime::FIRST_GUESS);
        for _ in 0..50 {
            took.saw(std::time::Duration::from_millis(900));
        }
        let t = took.typical().as_millis();
        assert!((850..=950).contains(&t), "{t}");
        took.saw(std::time::Duration::from_secs(60));
        assert!(took.typical() <= FailureTime::LONGEST, "one slow answer does not hold every refusal for a minute");
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

#[cfg(test)]
mod me_tests {
    use super::*;

    #[tokio::test]
    async fn me_says_where_a_title_opens_in_jellyfin() {
        let app = crate::state::test_app();
        let me = || user_json(&app, "u1", "alice", false, Perms::default());
        assert_eq!(me().await["jellyfin_details"], Value::Null, "no Jellyfin yet, no link");
        // The address FinStats reads Jellyfin at is often one inside the house, which only administrators are shown; a
        // link is offered once Settings names the address people open Jellyfin at.
        *app.config.write().unwrap() = Some(JfConfig { url: "http://192.168.1.10:8096".into(), api_key: "k".into(), server_name: "Jellyfin".into(), server_version: String::new(), from_env: false });
        assert_eq!(me().await["jellyfin_details"], Value::Null, "the internal address was told to everyone signed in");
        app.update_settings(|s| {
            s.jellyfin_public_url = "https://jellyfin.example.com/".into();
            Ok(())
        })
        .await
        .unwrap();
        // The page of a title is this followed by its id: one rule for the link, `jellyfin::web_link`'s.
        assert_eq!(me().await["jellyfin_details"], json!(crate::jellyfin::web_link("https://jellyfin.example.com/", "")));
        assert_eq!(me().await["jellyfin_details"], json!("https://jellyfin.example.com/web/#/details?id="));
    }

    /// Everybody signed in is told the address FinStats is reached at, so a calendar feed's link is made with it. It was
    /// read from the settings, which only a manager may read: anybody else got a 403 and a link that works only at home.
    #[tokio::test]
    async fn me_says_the_address_finstats_is_reached_at() {
        let app = crate::state::test_app();
        let me = || user_json(&app, "u1", "alice", false, Perms::default());
        assert_eq!(me().await["public_url"], Value::Null, "none set, none said");
        app.update_settings(|s| {
            s.public_url = "https://stats.example.com/".into();
            Ok(())
        })
        .await
        .unwrap();
        assert_eq!(me().await["public_url"], json!("https://stats.example.com"), "without the slash a path is put after");
    }
}
