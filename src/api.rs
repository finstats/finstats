//! Router, admin endpoints, image proxy and the embedded web UI.

use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use axum::body::Body;
use axum::extract::{DefaultBodyLimit, Path, Query, Request, State};
use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE, HOST, ORIGIN};
use axum::http::{HeaderValue, Method, StatusCode, Uri};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use futures_util::StreamExt;
use rust_embed::RustEmbed;
use serde::Deserialize;
use serde_json::{Value, json};
use tokio::io::AsyncWriteExt;
use tower_http::compression::CompressionLayer;
use tower_http::compression::predicate::{DefaultPredicate, NotForContentType, Predicate};

use crate::artwork::{self, Picture};
use crate::audit::{self, Actor};
use crate::auth::{self, AuthUser, JellyfinAdmin, Manager};
use crate::state::{ApiError, ApiResult, App, Settings};
use crate::db::rusqlite::OptionalExtension;
use crate::{changelog, db, groups, import, pipeline, profile, recap, recent, security, services, stats, streamystats, sync, timeline};

#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/web"]
struct WebAssets;

pub fn router(app: App) -> Router {
    let api = Router::new()
        .route("/status", get(auth::status))
        .route("/setup/test", post(auth::setup_test))
        .route("/setup", post(auth::setup))
        .route("/auth/login", post(auth::login))
        .route("/auth/logout", post(auth::logout))
        .route("/auth/me", get(auth::me))
        .route("/summary", get(stats::summary))
        .route("/now-playing", get(stats::now_playing))
        .route("/stats/overview", get(stats::overview))
        .route("/stats/top", get(stats::top_handler))
        .route("/stats/heatmap", get(stats::heatmap_handler))
        .route("/stats/playback", get(stats::playback))
        .route("/stats/insights", get(stats::insights))
        .route("/stats/files", get(stats::file_signals))
        .route("/stats/groups", get(groups::groups))
        .route("/library/insights", get(stats::library_insights))
        .route("/library/recent", get(recent::recently_added))
        .route("/server", get(stats::server))
        .route("/jellyfin/jobs", get(crate::jobs::jobs))
        .route("/recap", get(recap::recap))
        .route("/recap/cards/{chapter}", get(story_card))
        .route("/recap/cards.zip", get(story_zip))
        .route("/changelog", get(changelog::changelog))
        .route("/licenses", get(crate::licenses::licenses))
        .route("/activity", get(stats::activity))
        .route("/activity/{id}", get(stats::activity_detail))
        .route("/activity/{id}", delete(stats::activity_delete))
        .route("/users", get(stats::users))
        .route("/users/{id}", get(stats::user_detail))
        .route("/users/{id}/shows", get(profile::shows))
        .route("/users/{id}/timeline", get(timeline::timeline))
        .route("/me/seen", post(profile::mark_seen))
        .route("/me/watchlist", get(crate::watchlist::list_mine).post(crate::watchlist::add_mine))
        .route("/me/watchlist/keys", get(crate::watchlist::keys_mine))
        .route("/me/watchlist/{id}", delete(crate::watchlist::remove_mine))
        .route("/libraries", get(stats::libraries))
        .route("/libraries/{id}", get(stats::library_detail))
        .route("/items/{id}", get(stats::item_detail))
        .route("/people/{id}", get(stats::person_detail))
        .route("/search", get(stats::search))
        .route("/events", get(stats::events))
        .route("/audit", get(crate::audit::audit))
        .route("/me/public-profile", get(crate::public::get_mine).put(crate::public::put_mine))
        .route("/me/public-profile/reset", post(crate::public::reset_mine))
        .route("/public-profiles", get(crate::public::list))
        .route("/public-profiles/{id}", delete(crate::public::take_down))
        // Read without an account: what one person published, to anyone holding the link, and nothing else.
        .route("/public/{token}", get(crate::public::read))
        .route("/public/{token}/img/{item_id}", get(public_item_image))
        .route("/public/{token}/avatar", get(public_avatar))
        .route("/keys", get(crate::keys::list).post(crate::keys::create))
        .route("/keys/{id}", delete(crate::keys::revoke))
        .route("/calendar.ics", get(crate::ical::feed))
        .route("/security", get(security::overview))
        .route("/security/alerts", get(security::alerts))
        .route("/security/alerts/resolve-all", post(security::resolve_all))
        .route("/security/alerts/{id}/resolve", post(security::resolve))
        .route("/security/alerts/{id}/reopen", post(security::reopen))
        .route("/security/database", post(security::download_database))
        .route("/img/item/{id}", get(item_image))
        .route("/img/user/{id}", get(user_image))
        .route("/img/arr/{service_id}/{media_id}", get(arr_image))
        .route("/upcoming", get(pipeline::upcoming))
        .route("/requests", get(pipeline::requests))
        .route("/requests/summary", get(pipeline::requests_summary))
        .route("/downloads", get(crate::downloads::downloads))
        .route("/downloads/history", get(pipeline::download_history))
        .route("/notifications", get(crate::notify::list))
        .route("/notifications/targets", post(crate::notify::create))
        .route("/notifications/targets/{id}", axum::routing::put(crate::notify::update).delete(crate::notify::remove))
        .route("/notifications/targets/{id}/test", post(crate::notify::test))
        .route("/notifications/history", get(crate::notify::history))
        .route("/settings", get(get_settings).put(put_settings))
        .route("/settings/public-ip", post(lookup_public_ip))
        .route("/outbound", get(crate::outbound::outbound))
        .route("/permissions", get(get_permissions))
        .route("/permissions/defaults", axum::routing::put(put_default_permissions))
        .route("/permissions/users/{id}", axum::routing::put(put_user_permissions))
        .route("/services", get(services::list).post(services::create))
        .route("/services/test", post(services::test_connection))
        .route("/services/{id}", axum::routing::put(services::update).delete(services::remove))
        .route("/tasks", get(get_tasks))
        .route("/tasks/{id}/run", post(run_task))
        .route("/tasks/{id}/triggers", axum::routing::put(put_triggers).delete(reset_triggers))
        // Backups run to hundreds of MB and are streamed to disk, never buffered.
        .route("/import/jellystat", post(import_jellystat).layer(DefaultBodyLimit::disable()))
        .route("/import/streamystats", post(import_streamystats).layer(DefaultBodyLimit::disable()))
        .route("/backups", get(list_backups).post(create_backup))
        .route("/backups/restore", post(restore_upload).layer(DefaultBodyLimit::disable()))
        .route("/backups/{name}", get(download_backup).delete(delete_backup))
        .route("/backups/{name}/restore", post(restore_stored))
        .fallback(|| async { ApiError::not_found("Endpoint") })
        .layer(middleware::from_fn_with_state(app.clone(), same_origin));

    Router::new()
        .nest("/api", api)
        .route("/u/{token}", get(public_page))
        .route("/u/{token}/card.png", get(public_card))
        .route("/u/{token}/recap/{chapter}", get(public_story_card))
        .route("/u/{token}/recap.zip", get(public_story_zip))
        .fallback(static_handler)
        .layer(middleware::from_fn(security_headers))
        // A backup is gzip already. Compressing it again gains nothing, and the doubly-encoded stream
        // broke off in browsers: downloads failed without a word.
        .layer(CompressionLayer::new().gzip(true).compress_when(DefaultPredicate::new().and(NotForContentType::new("application/gzip"))))
        .with_state(app)
}

// ---------------------------------------------------------------- middleware

/// Browsers attach `Origin` to cross-site writes. Refuse any write whose origin is not us.
/// (SameSite=Lax cookies already cover this; this is the second lock. A `Bearer` key needs neither:
/// no browser adds an `Authorization` header on its own.)
///
/// Behind a reverse proxy the browser's `Origin` is the public name while `Host` has been rewritten to
/// the internal one, so a proxy's `X-Forwarded-Host` is what the origin should match. But that header
/// is trusted **only** when `FINSTATS_TRUST_PROXY` is set — the same gate `client_ip` uses. Without a
/// trusted proxy a client sets `X-Forwarded-Host` itself, and honouring it would let any client vouch
/// for its own foreign `Origin` (`Origin: https://evil` + `X-Forwarded-Host: evil` sailed straight
/// through), which is the whole lock undone by one header.
async fn same_origin(State(app): State<App>, req: Request, next: Next) -> Response {
    let write = !matches!(*req.method(), Method::GET | Method::HEAD | Method::OPTIONS);
    let h = req.headers();
    let origin = h.get(ORIGIN).and_then(|v| v.to_str().ok());
    let host = h.get(HOST).and_then(|v| v.to_str().ok());
    let forwarded = h.get("x-forwarded-host").and_then(|v| v.to_str().ok());
    if write && cross_site(origin, host, forwarded, app.trust_proxy) {
        return ApiError::new(StatusCode::FORBIDDEN, "Cross-site request refused").into_response();
    }
    next.run(req).await
}

/// The same-origin decision, pulled out so it can be tested without a live request. A write is
/// cross-site when it carries both an `Origin` and a `Host` and the origin's host matches neither the
/// real `Host` nor — **only behind a trusted proxy** — the `X-Forwarded-Host`. A request with no
/// `Origin` (a native client, curl) is not judged here: it carries either the SameSite=Lax cookie, which a
/// browser attaches to a cross-site navigation only and never to a cross-site `fetch`, or a `Bearer` key,
/// which no browser adds on its own — a cross-site page cannot set `Authorization` without a CORS
/// preflight, and finstats answers none.
fn cross_site(origin: Option<&str>, host: Option<&str>, forwarded: Option<&str>, trust_proxy: bool) -> bool {
    let (Some(origin), Some(host)) = (origin, host) else { return false };
    let origin_host = origin.split_once("://").map(|(_, h)| h).unwrap_or(origin);
    // A client can set X-Forwarded-Host itself; only a trusted proxy's copy may vouch for the origin,
    // or `Origin: https://evil` + `X-Forwarded-Host: evil` would defeat the whole check.
    let forwarded = trust_proxy.then_some(forwarded).flatten();
    origin_host != host && Some(origin_host) != forwarded
}

async fn security_headers(req: Request, next: Next) -> Response {
    let mut resp = next.run(req).await;
    let h = resp.headers_mut();
    h.insert("x-content-type-options", HeaderValue::from_static("nosniff"));
    h.insert("x-frame-options", HeaderValue::from_static("DENY"));
    h.insert("referrer-policy", HeaderValue::from_static("same-origin"));
    h.insert(
        "content-security-policy",
        HeaderValue::from_static(
            "default-src 'self'; img-src 'self' data: blob:; style-src 'self'; script-src 'self'; \
             object-src 'none'; base-uri 'self'; frame-ancestors 'none'; form-action 'self'",
        ),
    );
    resp
}

// ---------------------------------------------------------------- web UI

async fn static_handler(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    // The public page is a template with holes in it; it is only ever served filled, from `/u/{token}`.
    if let Some(file) = WebAssets::get(path).filter(|_| !path.is_empty() && path != "public.html") {
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        // Fonts never change; everything else revalidates so upgrades show up immediately.
        let cache = if path.starts_with("assets/fonts/") { "public, max-age=31536000, immutable" } else { "no-cache" };
        let etag = format!("\"{}\"", hex::encode(&file.metadata.sha256_hash()[..8]));
        return Response::builder()
            .header(CONTENT_TYPE, mime.as_ref())
            .header(CACHE_CONTROL, cache)
            .header("etag", etag)
            .body(Body::from(file.data.into_owned()))
            .unwrap();
    }
    if path.starts_with("assets/") {
        return StatusCode::NOT_FOUND.into_response();
    }
    // Client-side routes: /users/abc, /settings, …
    match WebAssets::get("index.html") {
        Some(index) => Response::builder()
            .header(CONTENT_TYPE, "text/html; charset=utf-8")
            .header(CACHE_CONTROL, "no-cache")
            .body(Body::from(index.data.into_owned()))
            .unwrap(),
        None => (StatusCode::NOT_FOUND, "finstats was built without its web UI").into_response(),
    }
}

// ---------------------------------------------------------------- images

const WIDTHS: [u32; 6] = [64, 96, 160, 300, 480, 1280];
const IMAGE_TTL: Duration = Duration::from_secs(7 * 86_400);
const MISSING_TTL: Duration = Duration::from_secs(86_400);

#[derive(Deserialize)]
struct ImageQuery {
    kind: Option<String>,
    w: Option<u32>,
}

fn sniff(bytes: &[u8]) -> &'static str {
    match bytes {
        [0xFF, 0xD8, ..] => "image/jpeg",
        [0x89, b'P', b'N', b'G', ..] => "image/png",
        [b'R', b'I', b'F', b'F', _, _, _, _, b'W', b'E', b'B', b'P', ..] => "image/webp",
        [b'G', b'I', b'F', ..] => "image/gif",
        _ => "application/octet-stream",
    }
}

fn image_response(bytes: Vec<u8>) -> Response {
    Response::builder()
        .header(CONTENT_TYPE, sniff(&bytes))
        .header(CACHE_CONTROL, "private, max-age=604800")
        .body(Body::from(bytes))
        .unwrap()
}

/// Proxy + disk cache, so browsers never need to reach Jellyfin themselves (it is often
/// only reachable from the Docker network) and posters cost Jellyfin one resize each.
async fn cached_image(app: &App, cache_name: String, jf_path: String, width: u32) -> ApiResult<Response> {
    let jf = app.jellyfin();
    cached(app, cache_name, "Jellyfin", jf.map(|jf| async move { Ok(jf.image(&jf_path, width).await?.map(|(bytes, _)| bytes)) })).await
}

/// The disk cache around any image source: a fresh file is served as it is, a miss is remembered (as an empty
/// file) so grids of poster-less titles stay cheap, and when the source is down a stale poster beats a broken one.
/// `fetch` is `None` when there is nowhere to ask; it is only polled when the cache has no answer.
async fn cached<F>(app: &App, cache_name: String, source: &str, fetch: Option<F>) -> ApiResult<Response>
where
    F: std::future::Future<Output = anyhow::Result<Option<Vec<u8>>>>,
{
    let dir: PathBuf = app.data_dir.join("cache").join("img");
    let file = dir.join(&cache_name);
    if let Ok(meta) = tokio::fs::metadata(&file).await {
        let age = meta.modified().ok().and_then(|m| SystemTime::now().duration_since(m).ok()).unwrap_or(Duration::MAX);
        if meta.len() == 0 && age < MISSING_TTL {
            return Err(ApiError::not_found("Image"));
        }
        if meta.len() > 0 && age < IMAGE_TTL {
            if let Ok(bytes) = tokio::fs::read(&file).await {
                return Ok(image_response(bytes));
            }
        }
    }
    let fetch = fetch.ok_or_else(|| ApiError::not_found("Image"))?;
    let fetched = fetch.await;
    tokio::fs::create_dir_all(&dir).await.ok();
    match fetched {
        Ok(Some(bytes)) => {
            tokio::fs::write(&file, &bytes).await.ok();
            Ok(image_response(bytes))
        }
        Ok(None) => {
            tokio::fs::write(&file, b"").await.ok();
            Err(ApiError::not_found("Image"))
        }
        Err(e) => {
            if let Ok(bytes) = tokio::fs::read(&file).await {
                if !bytes.is_empty() {
                    return Ok(image_response(bytes));
                }
            }
            tracing::debug!("image fetch failed: {e:#}");
            Err(ApiError::new(StatusCode::BAD_GATEWAY, format!("Could not load the image from {source}")))
        }
    }
}

fn pick_width(w: Option<u32>) -> u32 {
    let want = w.unwrap_or(300);
    WIDTHS.iter().copied().find(|x| *x >= want).unwrap_or(1280)
}

fn valid_id(id: &str) -> Result<String, ApiError> {
    let id = db::norm_id(id);
    if id.len() == 32 && id.chars().all(|c| c.is_ascii_hexdigit()) { Ok(id) } else { Err(ApiError::not_found("Image")) }
}

async fn item_image(State(app): State<App>, _user: AuthUser, Path(id): Path<String>, Query(q): Query<ImageQuery>, headers: axum::http::HeaderMap) -> ApiResult<Response> {
    let pic = if q.kind.as_deref() == Some("backdrop") { Picture::Backdrop } else { Picture::Primary };
    current_picture(&app, pic, valid_id(&id)?, pick_width(q.w), &headers).await
}

async fn user_image(State(app): State<App>, _user: AuthUser, Path(id): Path<String>, Query(q): Query<ImageQuery>, headers: axum::http::HeaderMap) -> ApiResult<Response> {
    current_picture(&app, Picture::User, valid_id(&id)?, pick_width(q.w.or(Some(96))), &headers).await
}

/// A picture through the cache, under the tag finstats last read for it (`artwork`): a poster replaced in
/// Jellyfin has a new tag and so a new file. With a tag the browser is handed it as the ETag and asks again
/// every time — a 304, before anything is read, when nothing changed — instead of keeping whatever it had for a week.
async fn current_picture(app: &App, pic: Picture, id: String, width: u32, headers: &axum::http::HeaderMap) -> ApiResult<Response> {
    let tag = tag_of(app, pic, &id).await?;
    let Some(etag) = tag.as_deref().map(artwork::etag) else { return fetch_picture(app, pic, &id, width, None).await };
    let fresh = artwork::not_modified(headers.get(axum::http::header::IF_NONE_MATCH).and_then(|v| v.to_str().ok()), &etag);
    let mut r = if fresh { StatusCode::NOT_MODIFIED.into_response() } else { fetch_picture(app, pic, &id, width, tag.as_deref()).await? };
    r.headers_mut().insert(CACHE_CONTROL, HeaderValue::from_static("private, no-cache"));
    if let Ok(v) = HeaderValue::from_str(&etag) {
        r.headers_mut().insert(axum::http::header::ETAG, v);
    }
    Ok(r)
}

/// A picture from the cache or Jellyfin, under the tag finstats knows for it.
async fn picture(app: &App, pic: Picture, id: String, width: u32) -> ApiResult<Response> {
    let tag = tag_of(app, pic, &id).await?;
    fetch_picture(app, pic, &id, width, tag.as_deref()).await
}

async fn tag_of(app: &App, pic: Picture, id: &str) -> ApiResult<Option<String>> {
    let id = id.to_string();
    Ok(app.db.call(move |c| artwork::tag_of(c, pic, &id)).await?)
}

async fn fetch_picture(app: &App, pic: Picture, id: &str, width: u32, tag: Option<&str>) -> ApiResult<Response> {
    let path = match pic {
        Picture::Primary => format!("/Items/{id}/Images/Primary"),
        Picture::Backdrop => format!("/Items/{id}/Images/Backdrop"),
        Picture::User => format!("/Users/{id}/Images/Primary"),
    };
    cached_image(app, artwork::cache_name(pic, id, width, tag), path, width).await
}

/// A poster on a published profile, for a reader without an account: only an item the page itself shows.
async fn public_item_image(State(app): State<App>, Path((token, id)): Path<(String, String)>, Query(q): Query<ImageQuery>) -> ApiResult<Response> {
    let id = valid_id(&id)?;
    let (_, a) = crate::public::resolve(&app, token).await?;
    if !crate::public::listed_images(&a).contains(&id) {
        return Err(ApiError::not_found("Image"));
    }
    public_image(picture(&app, Picture::Primary, id, pick_width(q.w)).await?)
}

/// The owner's picture, when they chose to show it.
async fn public_avatar(State(app): State<App>, Path(token): Path<String>) -> ApiResult<Response> {
    let (p, _) = crate::public::resolve(&app, token).await?;
    if !p.show_avatar {
        return Err(ApiError::not_found("Image"));
    }
    public_image(picture(&app, Picture::User, valid_id(&p.user_id)?, 96).await?)
}

/// Anyone may keep a published poster, but not for long: a profile taken down should stop showing soon.
fn public_image(mut r: Response) -> ApiResult<Response> {
    r.headers_mut().insert(CACHE_CONTROL, HeaderValue::from_static("public, max-age=3600"));
    Ok(crate::public::noindex(r))
}

/// `/u/{token}`: the published page. Its preview (the title and the card chat apps show) is filled in
/// here, because a link preview is read by a bot that runs no script.
async fn public_page(State(app): State<App>, Path(token): Path<String>) -> Response {
    let found = crate::public::resolve(&app, token.clone()).await;
    let page = WebAssets::get("public.html").map(|f| String::from_utf8_lossy(&f.data).into_owned());
    let (Ok((_, a)), Some(page)) = (found, page) else {
        let body = "<!doctype html><meta charset=utf-8><title>Not found</title><p>There is no profile here.</p>";
        return crate::public::noindex((StatusCode::NOT_FOUND, [(CONTENT_TYPE, "text/html; charset=utf-8")], body).into_response());
    };
    let base = app.settings().public_url.trim_end_matches('/').to_string();
    let html = crate::public::fill_page(&page, &a, &base, &token);
    crate::public::noindex(([(CONTENT_TYPE, "text/html; charset=utf-8"), (CACHE_CONTROL, "no-cache")], html).into_response())
}

#[derive(Deserialize)]
struct CardQuery {
    kind: Option<String>,
}

/// `/u/{token}/card.png`: the published profile as a picture, for a link preview or to post.
async fn public_card(State(app): State<App>, Path(token): Path<String>, Query(q): Query<CardQuery>) -> ApiResult<Response> {
    let kind = crate::card::Kind::parse(q.kind.as_deref()).ok_or_else(crate::public::not_found)?;
    let (_, a) = crate::public::resolve(&app, token.clone()).await?;
    if !kind.available(&a) {
        return Err(crate::public::not_found());
    }
    let key = crate::card::key(&token, kind, &a);
    let ids = crate::card::poster_ids(kind, &a);
    let png = draw(&app, key, ids, move |posters| crate::card::svg(kind, &a, posters)).await?;
    let r = Response::builder()
        .header(CONTENT_TYPE, "image/png")
        .header(CACHE_CONTROL, "public, max-age=3600")
        .body(Body::from(png.as_ref().clone()))
        .unwrap();
    Ok(crate::public::noindex(r))
}

/// A card from the memory cache, or drawn: at most two at once (`card_permits`), its posters fetched
/// through the image cache, rasterised off the async threads. Every card goes through here — the
/// published ones and the recap's story — so none of them can skip the limit.
pub(crate) async fn draw<F>(app: &App, key: String, poster_ids: Vec<String>, svg: F) -> ApiResult<std::sync::Arc<Vec<u8>>>
where
    F: FnOnce(&crate::card::Posters) -> String,
{
    if let Some(png) = app.public_cards.get(&key) {
        return Ok(png);
    }
    let _permit = app.card_permits.acquire().await.map_err(anyhow::Error::from)?;
    if let Some(png) = app.public_cards.get(&key) {
        return Ok(png);
    }
    let mut posters = crate::card::Posters::new();
    for id in poster_ids {
        if let Some(bytes) = poster_bytes(app, &id).await {
            posters.insert(id, bytes);
        }
    }
    let svg = svg(&posters);
    let png = tokio::task::spawn_blocking(move || crate::card::render_png(&svg)).await.map_err(anyhow::Error::from)??;
    Ok(app.public_cards.put(key, png))
}

/// The published year, one chapter at a time, to anyone holding the link — only when the year is published.
async fn public_story_card(State(app): State<App>, Path((token, chapter)): Path<(String, String)>) -> ApiResult<Response> {
    let ch = crate::story::Chapter::parse(&chapter).ok_or_else(crate::public::not_found)?;
    let (_, a) = crate::public::resolve(&app, token).await?;
    let story = a.recap.filter(|y| y.chapters().contains(&ch)).ok_or_else(crate::public::not_found)?;
    let png = draw_story(&app, &story, ch).await?;
    let r = Response::builder()
        .header(CONTENT_TYPE, "image/png")
        .header(CACHE_CONTROL, "public, max-age=3600")
        .body(Body::from(png.as_ref().clone()))
        .unwrap();
    Ok(crate::public::noindex(r))
}

async fn public_story_zip(State(app): State<App>, Path(token): Path<String>) -> ApiResult<Response> {
    let (_, a) = crate::public::resolve(&app, token).await?;
    let story = a.recap.ok_or_else(crate::public::not_found)?;
    Ok(crate::public::noindex(zip_response(&story_files(&app, &story).await?, &story.label)))
}

/// One chapter of the caller's year as a 1080×1920 card, for the same people who may open that year.
async fn story_card(State(app): State<App>, user: AuthUser, Path(chapter): Path<String>, Query(q): Query<recap::RecapQuery>) -> ApiResult<Response> {
    let ch = crate::story::Chapter::parse(&chapter).ok_or_else(|| ApiError::not_found("Chapter"))?;
    let story = recap::story_for(&app, &user, &q).await?;
    if !story.chapters().contains(&ch) {
        return Err(ApiError::not_found("Chapter"));
    }
    let png = draw_story(&app, &story, ch).await?;
    Ok(Response::builder()
        .header(CONTENT_TYPE, "image/png")
        .header(CACHE_CONTROL, "private, max-age=300")
        .body(Body::from(png.as_ref().clone()))
        .unwrap())
}

/// Every chapter's card, in order, in one ZIP.
async fn story_zip(State(app): State<App>, user: AuthUser, Query(q): Query<recap::RecapQuery>) -> ApiResult<Response> {
    let story = recap::story_for(&app, &user, &q).await?;
    Ok(zip_response(&story_files(&app, &story).await?, &story.label))
}

pub(crate) async fn story_files(app: &App, story: &crate::story::StoryYear) -> ApiResult<Vec<(String, Vec<u8>)>> {
    let mut files = vec![];
    for (i, ch) in story.chapters().into_iter().enumerate() {
        files.push((format!("{:02}-{}.png", i + 1, ch.key()), draw_story(app, story, ch).await?.as_ref().clone()));
    }
    Ok(files)
}

pub(crate) fn zip_response(files: &[(String, Vec<u8>)], label: &str) -> Response {
    let slug: String = label.to_lowercase().chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '-' }).collect();
    Response::builder()
        .header(CONTENT_TYPE, "application/zip")
        .header("content-disposition", format!("attachment; filename=\"finstats-{}.zip\"", slug.trim_matches('-')))
        .header(CACHE_CONTROL, "private, max-age=300")
        .body(Body::from(crate::card::zip(files)))
        .unwrap()
}

pub(crate) async fn draw_story(app: &App, story: &crate::story::StoryYear, ch: crate::story::Chapter) -> ApiResult<std::sync::Arc<Vec<u8>>> {
    use sha2::{Digest, Sha256};
    let said = serde_json::to_vec(story).map_err(anyhow::Error::from)?;
    let key = format!("story:{}:{}", ch.key(), hex::encode(&Sha256::digest(&said)[..12]));
    let story = story.clone();
    draw(app, key, story.poster_ids(ch), move |posters| crate::story::svg(ch, &story, posters)).await
}

/// A poster's bytes through the same cache the pages use; a card without one draws an empty frame.
async fn poster_bytes(app: &App, id: &str) -> Option<Vec<u8>> {
    let id = valid_id(id).ok()?;
    let r = picture(app, Picture::Primary, id, 480).await.ok()?;
    axum::body::to_bytes(r.into_body(), 16 << 20).await.ok().map(|b| b.to_vec())
}

/// The poster of a title that is not in the library yet: only Sonarr or Radarr has it. Both path segments
/// are numbers, the upstream path is a constant, and only posters of titles finstats itself has listed are
/// served, so this is no window into everything Sonarr and Radarr know.
async fn arr_image(State(app): State<App>, user: AuthUser, Path((service_id, media_id)): Path<(i64, i64)>, Query(q): Query<ImageQuery>) -> ApiResult<Response> {
    let width = if q.w.unwrap_or(250) <= 250 { 250 } else { 500 };
    let svc = services::all(&app).iter().find(|s| s.id == service_id && s.enabled && s.kind.is_arr()).cloned().ok_or_else(|| ApiError::not_found("Image"))?;
    // Only posters of titles finstats itself has listed: what is on the calendar, what somebody asked for, and
    // what is downloading now (for the people who may see that). Anything else and the proxy would be a way to
    // walk through the whole Arr library by counting upwards.
    let listed = media_id > 0
        && ((user.perms.see_downloads && crate::downloads::in_queue(&app, service_id, media_id))
            || app.db.call(move |c| pipeline::poster_is_listed(c, service_id, media_id, &user)).await?);
    if !listed {
        return Err(ApiError::not_found("Image"));
    }
    let worker = app.clone();
    cached(&app, format!("arr-{service_id}-{media_id}-{width}"), svc.kind.label(), Some(async move {
        let bytes = services::get_bytes(&worker, &svc, &crate::arr::poster_path(media_id, width), 6 * 1024 * 1024).await?;
        // Whatever comes back is only passed on if it is an image.
        Ok(bytes.filter(|b| sniff(b) != "application/octet-stream"))
    }))
    .await
}

/// Drop cached images nobody has refreshed in a month.
pub async fn prune_image_cache(app: App) {
    loop {
        let dir = app.data_dir.join("cache").join("img");
        let _ = tokio::task::spawn_blocking(move || {
            let Ok(entries) = std::fs::read_dir(&dir) else { return };
            for e in entries.flatten() {
                let old = e.metadata().and_then(|m| m.modified()).ok().and_then(|m| SystemTime::now().duration_since(m).ok()).is_some_and(|a| a > Duration::from_secs(30 * 86_400));
                if old {
                    let _ = std::fs::remove_file(e.path());
                }
            }
        })
        .await;
        tokio::time::sleep(Duration::from_secs(86_400)).await;
    }
}

// ---------------------------------------------------------------- settings & tasks

fn settings_json(app: &App) -> Value {
    let cfg = app.config.read().unwrap().clone();
    let mut v = serde_json::to_value(app.settings()).unwrap_or_else(|_| json!({}));
    if let (Some(obj), Some(c)) = (v.as_object_mut(), cfg) {
        obj.insert("jellyfin_url".into(), json!(c.url));
        obj.insert("server_name".into(), json!(c.server_name));
        obj.insert("server_version".into(), json!(c.server_version));
        obj.insert("configured_by_env".into(), json!(c.from_env));
    }
    v
}

/// The settings plus what is only read: the home addresses finstats knows, and who it would ask.
async fn settings_response(app: &App) -> ApiResult {
    let mut v = settings_json(app);
    let known = app.db.call(|c| crate::network::list(c)).await?;
    if let Some(obj) = v.as_object_mut() {
        obj.insert("known_home_addresses".into(), json!(known));
        obj.insert("public_ip_services".into(), json!(crate::network::services()));
        obj.insert("geoip".into(), security::status_json(app));
    }
    Ok(Json(v))
}

async fn get_settings(State(app): State<App>, Manager(_): Manager) -> ApiResult {
    settings_response(&app).await
}

/// Ask a public service for this network's address, now, because somebody pressed the button.
/// It is the only thing that asks after the first answer: there is no timer behind this.
async fn lookup_public_ip(State(app): State<App>, Manager(_): Manager) -> ApiResult {
    if !app.settings().public_ip_lookup {
        return Err(ApiError::bad_request("Turn on \"Recognise my own public address\" first"));
    }
    crate::network::refresh(&app).await;
    settings_response(&app).await
}

async fn put_settings(State(app): State<App>, Manager(user): Manager, Json(patch): Json<Value>) -> ApiResult {
    let Some(patch) = patch.as_object() else {
        return Err(ApiError::bad_request("Expected a JSON object"));
    };
    // Who gets in and what everyone may see is an administrator's call: a manager must not be able to promote themselves.
    if !user.is_admin && ACCESS_KEYS.iter().any(|k| patch.contains_key(*k)) {
        return Err(ApiError::forbidden());
    }
    let (before, next) = app
        .update_settings(|s| {
            let mut merged = serde_json::to_value(&*s).map_err(anyhow::Error::from)?;
            let target = merged.as_object_mut().expect("the settings are an object");
            for (k, v) in patch {
                if target.contains_key(k) {
                    target.insert(k.clone(), v.clone());
                }
            }
            *s = serde_json::from_value(merged).map_err(|e| ApiError::bad_request(format!("Invalid settings: {e}")))?;
            Ok(())
        })
        .await?;
    // What changed, for the audit log: the blob holds no secret, so the diff is the whole story.
    let changed = crate::audit::changed_keys(&serde_json::to_value(&before).unwrap_or_default(), &serde_json::to_value(&next).unwrap_or_default());
    let regroup = (next.group_window_s != before.group_window_s).then_some(next.group_window_s);
    let homes = (next.home_addresses != before.home_addresses).then(|| next.home_addresses.clone());
    let homes_changed = homes.is_some();
    let lookup_switched_on = next.public_ip_lookup && !before.public_ip_lookup;
    let rules_changed = (next.travel_speed_kmh, next.travel_min_km) != (before.travel_speed_kmh, before.travel_min_km);
    if regroup.is_some() || homes.is_some() {
        app.db
            .call(move |c| {
                // A different window means different groups, for the whole history.
                if let Some(window) = regroup {
                    groups::detect(c, window, None)?;
                }
                // Home addresses decide which plays were local, for the whole history.
                if let Some(list) = &homes {
                    crate::network::set_manual(c, list)?;
                    crate::network::reclassify(c)?;
                }
                Ok(())
            })
            .await?;
    }
    app.wake.notify_waiters();
    if lookup_switched_on {
        crate::network::refresh(&app).await;
    }
    if rules_changed || homes_changed {
        security::check(&app, None).await;
    }
    if !changed.is_empty() {
        crate::audit::record(&app, crate::audit::Entry::new("setting_changed", crate::audit::Actor::from(&user)).detail(json!({ "changed": changed })));
    }
    settings_response(&app).await
}

async fn get_tasks(State(app): State<App>, Manager(_): Manager) -> ApiResult {
    let data_dir = app.data_dir.clone();
    let dbinfo = app
        .db
        .call(move |c| {
            let (plays, oldest): (i64, Option<i64>) = c.query_row("SELECT COUNT(*), MIN(started_at) FROM playbacks", [], |r| Ok((r.get(0)?, r.get(1)?)))?;
            let items: i64 = c.query_row("SELECT COUNT(*) FROM items WHERE removed = 0", [], |r| r.get(0))?;
            let size: u64 = ["finstats.db", "finstats.db-wal"].iter().filter_map(|f| std::fs::metadata(data_dir.join(f)).ok()).map(|m| m.len()).sum();
            Ok(json!({ "size_bytes": size, "plays": plays, "items": items, "oldest_play_at": oldest }))
        })
        .await?;
    let collector = app.collector.read().unwrap().clone();
    let settings = app.settings();
    let tasks: Vec<Value> = app.tasks.snapshot().iter().map(|t| task_json(t, &settings)).collect();
    Ok(Json(json!({ "tasks": tasks, "collector": collector, "db": dbinfo, "time_zone": time_zone() })))
}

/// One job as the Tasks section shows it: how it last went, and when it runs (`schedule`).
fn task_json(t: &crate::state::TaskState, settings: &Settings) -> Value {
    use crate::schedule;
    let schedulable = schedule::SCHEDULED.contains(&t.id);
    let triggers = if schedulable { schedule::effective(t.id, settings) } else { vec![] };
    let mut v = serde_json::to_value(t).unwrap_or_default();
    if let Some(o) = v.as_object_mut() {
        o.insert("schedulable".into(), json!(schedulable));
        o.insert("next_at".into(), json!(schedule::next_at(&triggers, db::now(), t.finished_at, &chrono::Local)));
        o.insert("triggers".into(), json!(triggers));
        o.insert("custom".into(), json!(settings.schedules.contains_key(t.id)));
        o.insert("runnable".into(), json!(RUNNABLE.contains(&t.id)));
        o.insert("can".into(), json!({ "after_scan": schedule::AFTER_SCAN.contains(&t.id), "limit": schedule::takes_limit(t.id) }));
    }
    v
}

/// The zone times of day are in: finstats' own, which is the process's `TZ`.
fn time_zone() -> String {
    std::env::var("TZ").ok().filter(|z| !z.trim().is_empty()).unwrap_or_else(|| chrono::Local::now().format("UTC%:z").to_string())
}

/// `PUT /api/tasks/{id}/triggers` with `{"triggers": [...]}`: the job's own schedule. An empty list means by hand only.
async fn put_triggers(State(app): State<App>, Manager(user): Manager, Path(id): Path<String>, Json(body): Json<Value>) -> ApiResult {
    let Some(id) = crate::state::TASK_IDS.into_iter().find(|t| *t == id) else { return Err(ApiError::not_found("Task")) };
    let triggers: Vec<crate::schedule::Trigger> =
        serde_json::from_value(body["triggers"].clone()).map_err(|e| ApiError::bad_request(format!("Invalid triggers: {e}")))?;
    crate::schedule::validate(id, &triggers).map_err(ApiError::bad_request)?;
    let detail = json!({ "triggers": triggers });
    save_schedule(&app, &user, id, move |s| _ = s.schedules.insert(id.to_string(), triggers), detail).await
}

/// `DELETE /api/tasks/{id}/triggers`: back to the job's defaults.
async fn reset_triggers(State(app): State<App>, Manager(user): Manager, Path(id): Path<String>) -> ApiResult {
    let Some(id) = crate::state::TASK_IDS.into_iter().find(|t| *t == id) else { return Err(ApiError::not_found("Task")) };
    save_schedule(&app, &user, id, |s| _ = s.schedules.remove(id), json!({ "reset": true })).await
}

async fn save_schedule(app: &App, user: &AuthUser, id: &'static str, change: impl FnOnce(&mut Settings), detail: Value) -> ApiResult {
    app.update_settings(|s| {
        change(s);
        Ok(())
    })
    .await?;
    // The scheduler looks again now: a trigger due in the next minute should not wait for the one after.
    app.wake.notify_waiters();
    audit::record(app, audit::Entry::new("task_schedule_changed", Actor::from(user)).target(id).detail(detail));
    let settings = app.settings();
    let t = app.tasks.snapshot().into_iter().find(|t| t.id == id).ok_or_else(|| ApiError::not_found("Task"))?;
    Ok(Json(task_json(&t, &settings)))
}

/// Every task that can be started by hand. `Tasks::try_start` panics on an id it does not know, so a test
/// holds this list against `TASK_IDS`.
const RUNNABLE: [&str; 11] = ["sync_users", "sync_libraries", "sync_events", "sync_server", "sync_userdata", "sync_changes", "sync_upcoming", "sync_requests", "sync_grabs", "backup", "geoip"];
/// The two runnable jobs that are neither a Jellyfin read nor a service read.
#[cfg(test)]
const OWN_RUNNERS: [&str; 2] = ["backup", "geoip"];

async fn run_task(State(app): State<App>, Manager(user): Manager, Path(id): Path<String>) -> ApiResult<Response> {
    let Some(id) = RUNNABLE.into_iter().find(|t| *t == id) else { return Err(ApiError::not_found("Task")) };
    let started = match id {
        "backup" => sync::run_backup(&app, Some(Actor::from(&user))),
        "geoip" => {
            if std::env::var("FINSTATS_GEOIP_DB").is_ok_and(|p| !p.trim().is_empty()) {
                return Err(ApiError::bad_request("The database is set with FINSTATS_GEOIP_DB; replace that file instead"));
            }
            crate::geo::spawn_download(&app)
        }
        // One way in: the Jellyfin reads, then the ones that read from connected services.
        _ => sync::spawn(&app, id) || services::spawn(&app, id),
    };
    if !started {
        return Err(ApiError::new(StatusCode::CONFLICT, "That task is already running"));
    }
    audit::record(&app, audit::Entry::new("task_run", Actor::from(&user)).target(id));
    Ok((StatusCode::ACCEPTED, Json(json!({ "ok": true }))).into_response())
}

// ---------------------------------------------------------------- permissions

/// Settings only a Jellyfin administrator may write: who gets in, what everyone may see, the address
/// finstats puts in the messages it sends, and whether anything may be read without an account.
const ACCESS_KEYS: [&str; 5] = ["allow_user_login", "default_permissions", "public_url", "public_profiles", "jellyfin_public_url"];

const PERMISSION_INFO: [(&str, &str, &str); 7] = [
    ("sign_in", "Sign in", "May use finstats and sees their own statistics and recap."),
    ("see_everyone", "See everyone's activity", "Other people's statistics and history, the Users page and every live stream."),
    ("see_network", "See network details", "IP addresses, device ids and whether a play was local or remote."),
    ("see_server", "See the server", "The Server page, the server log, failed sign-ins and file paths."),
    ("see_downloads", "See what is downloading", "The download queue with speeds, torrent names and which client. Without it, people still see how far their own request has got."),
    ("notify", "Be sent notifications", "May add destinations of their own (a webhook, Discord, ntfy or Gotify) and is sent what they are already allowed to see. Their destinations must point at a public address."),
    ("manage", "Manage finstats", "Settings, tasks, the Jellystat import and deleting plays. Cannot change permissions."),
];

#[derive(Deserialize)]
struct PermissionsBody {
    #[serde(default)]
    permissions: Vec<String>,
}

fn clean_permissions(list: Vec<String>) -> Result<Vec<String>, ApiError> {
    if let Some(bad) = list.iter().find(|p| !auth::GRANTABLE.contains(&p.as_str())) {
        return Err(ApiError::bad_request(format!("Unknown permission `{bad}`")));
    }
    // Stored in a fixed order, without duplicates.
    Ok(auth::GRANTABLE.iter().filter(|g| list.iter().any(|p| p == *g)).map(|g| g.to_string()).collect())
}

fn defaults_as_keys(s: &Settings) -> Vec<String> {
    let mut out: Vec<String> = if s.allow_user_login { vec![auth::SIGN_IN.to_string()] } else { vec![] };
    out.extend(s.default_permissions.iter().cloned());
    out
}

async fn get_permissions(State(app): State<App>, JellyfinAdmin(_): JellyfinAdmin) -> ApiResult {
    let users = app
        .db
        .call(|c| {
            let mut stmt = c.prepare(
                "SELECT u.id, u.name, u.is_admin, u.is_disabled, u.removed, (u.image_tag IS NOT NULL), COALESCE(p.permissions, '[]')
                 FROM users u LEFT JOIN user_permissions p ON p.user_id = u.id
                 WHERE u.removed = 0 ORDER BY u.is_admin DESC, u.name COLLATE NOCASE",
            )?;
            let rows = stmt
                .query_map([], |r| {
                    let raw: String = r.get(6)?;
                    Ok(json!({
                        "id": r.get::<_, String>(0)?, "name": r.get::<_, String>(1)?, "is_admin": r.get::<_, bool>(2)?,
                        "is_disabled": r.get::<_, bool>(3)?, "has_image": r.get::<_, bool>(5)?,
                        "permissions": serde_json::from_str::<Value>(&raw).unwrap_or_else(|_| json!([])),
                    }))
                })?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })
        .await?;
    let available: Vec<Value> = PERMISSION_INFO.iter().map(|(key, label, description)| json!({ "key": key, "label": label, "description": description })).collect();
    Ok(Json(json!({ "available": available, "defaults": defaults_as_keys(&app.settings()), "users": users })))
}

async fn put_default_permissions(State(app): State<App>, JellyfinAdmin(user): JellyfinAdmin, Json(body): Json<PermissionsBody>) -> ApiResult {
    let keys = clean_permissions(body.permissions)?;
    let (_, next) = app
        .update_settings(|s| {
            s.allow_user_login = keys.iter().any(|k| k == auth::SIGN_IN);
            s.default_permissions = keys.into_iter().filter(|k| k != auth::SIGN_IN).collect();
            Ok(())
        })
        .await?;
    audit::record(&app, audit::Entry::new("permissions_changed", Actor::from(&user)).target("defaults").detail(json!({ "permissions": defaults_as_keys(&next) })));
    Ok(Json(json!({ "defaults": defaults_as_keys(&next) })))
}

async fn put_user_permissions(State(app): State<App>, JellyfinAdmin(user): JellyfinAdmin, Path(id): Path<String>, Json(body): Json<PermissionsBody>) -> ApiResult {
    let id = db::norm_id(&id);
    let keys = clean_permissions(body.permissions)?;
    let stored = keys.clone();
    let (actor, target, granted) = (Actor::from(&user), id.clone(), keys.clone());
    let found = app
        .db
        .call(move |c| {
            let is_admin: Option<bool> = c.query_row("SELECT is_admin FROM users WHERE id = ?1", [&id], |r| r.get(0)).optional()?;
            let Some(is_admin) = is_admin else { return Ok(None) };
            if is_admin {
                return Ok(Some(false));
            }
            if stored.is_empty() {
                c.execute("DELETE FROM user_permissions WHERE user_id = ?1", [&id])?;
            } else {
                c.execute(
                    "INSERT INTO user_permissions(user_id, permissions, updated_at) VALUES (?1, ?2, ?3)
                     ON CONFLICT(user_id) DO UPDATE SET permissions = excluded.permissions, updated_at = excluded.updated_at",
                    db::rusqlite::params![id, serde_json::to_string(&stored)?, db::now()],
                )?;
            }
            Ok(Some(true))
        })
        .await?;
    match found {
        None => Err(ApiError::not_found("User")),
        Some(false) => Err(ApiError::bad_request("Jellyfin administrators already have every permission")),
        Some(true) => {
            audit::record(&app, audit::Entry::new("permissions_changed", actor).target(target).detail(json!({ "permissions": granted })));
            Ok(Json(json!({ "permissions": keys })))
        }
    }
}

// ---------------------------------------------------------------- importing history

// ---------------------------------------------------------------- backups
// Jellyfin administrators only: a backup is everyone's history, and restoring one can bring permissions back.

#[derive(Deserialize)]
struct RestoreQuery {
    /// Also restore settings and permissions (default: yes).
    settings: Option<bool>,
}

fn backup_path(app: &App, name: &str) -> ApiResult<PathBuf> {
    if !crate::backup::valid_name(name) {
        return Err(ApiError::not_found("Backup"));
    }
    let path = crate::backup::dir(&app.data_dir).join(name);
    if path.is_file() { Ok(path) } else { Err(ApiError::not_found("Backup")) }
}

async fn list_backups(State(app): State<App>, JellyfinAdmin(_): JellyfinAdmin) -> ApiResult {
    let dir = crate::backup::dir(&app.data_dir);
    let backups = tokio::task::spawn_blocking(move || crate::backup::list(&dir)).await.map_err(anyhow::Error::from)?;
    let s = app.settings();
    // When the next one is written is the backup task's schedule (Settings → Tasks), measured from its last run.
    let triggers = crate::schedule::effective("backup", &s);
    let last = app.tasks.snapshot().into_iter().find(|t| t.id == "backup").and_then(|t| t.finished_at);
    let next_at = crate::schedule::next_at(&triggers, db::now(), last, &chrono::Local);
    Ok(Json(json!({ "backups": backups, "scheduled": !triggers.is_empty(), "keep": s.backup_keep, "next_at": next_at })))
}

async fn create_backup(State(app): State<App>, JellyfinAdmin(user): JellyfinAdmin) -> ApiResult<Response> {
    if !sync::run_backup(&app, Some(Actor::from(&user))) {
        return Err(ApiError::new(StatusCode::CONFLICT, "A backup is already being written"));
    }
    Ok((StatusCode::ACCEPTED, Json(json!({ "ok": true }))).into_response())
}

async fn download_backup(State(app): State<App>, JellyfinAdmin(user): JellyfinAdmin, Path(name): Path<String>) -> ApiResult<Response> {
    let path = backup_path(&app, &name)?;
    // The whole history leaving: worth a line, though it is a read.
    audit::record(&app, audit::Entry::new("backup_downloaded", Actor::from(&user)).target(name.clone()));
    let file = tokio::fs::File::open(&path).await.map_err(anyhow::Error::from)?;
    let len = file.metadata().await.map_err(anyhow::Error::from)?.len();
    // Streamed from disk in 64 KB pieces: a large history never has to fit in memory.
    let stream = futures_util::stream::unfold(file, |mut f| async move {
        let mut buf = vec![0u8; 64 * 1024];
        match tokio::io::AsyncReadExt::read(&mut f, &mut buf).await {
            Ok(0) => None,
            Ok(n) => { buf.truncate(n); Some((Ok::<_, std::io::Error>(buf), f)) }
            Err(e) => Some((Err(e), f)),
        }
    });
    Ok(Response::builder()
        .header(CONTENT_TYPE, "application/gzip")
        .header("content-length", len)
        .header("content-disposition", format!("attachment; filename=\"{name}\""))
        .header(CACHE_CONTROL, "no-store")
        .body(Body::from_stream(stream))
        .unwrap())
}

async fn delete_backup(State(app): State<App>, JellyfinAdmin(user): JellyfinAdmin, Path(name): Path<String>) -> ApiResult {
    let path = backup_path(&app, &name)?;
    tokio::fs::remove_file(&path).await.map_err(anyhow::Error::from)?;
    audit::record(&app, audit::Entry::new("backup_deleted", Actor::from(&user)).target(name.clone()));
    Ok(Json(json!({ "ok": true })))
}

/// How an import ended, for the audit log; written from the worker thread.
fn import_finished(app: &App, actor: Actor, tracker: &str, outcome: Result<(u64, u64), String>) {
    let Ok(c) = app.db.conn() else { return };
    let entry = match outcome {
        Ok((imported, skipped)) => audit::Entry::new("import_finished", actor).target(tracker).detail(json!({ "plays_imported": imported, "plays_skipped": skipped })),
        Err(e) => audit::Entry::new("import_finished", actor).target(tracker).detail(json!({ "error": e })).outcome("failed"),
    };
    audit::record_quietly(&c, &entry);
}

/// Run a restore in the background, then load what it may have changed (the settings) into the running app.
fn spawn_restore(app: &App, path: PathBuf, with_settings: bool, remove_after: bool, actor: Actor, source: String) {
    let worker = app.clone();
    tokio::task::spawn_blocking(move || {
        // A restore that brings settings is a change to them like any other: nothing else may write the blob over
        // it, or hold the one it replaces in memory, until the restored one is the one in use.
        let _settings = with_settings.then(|| worker.settings_write.blocking_lock());
        let outcome = crate::backup::restore(&worker.db, &path, with_settings, Some((&worker.tasks, "restore")));
        if remove_after {
            let _ = std::fs::remove_file(&path);
        }
        if let Ok(c) = worker.db.conn() {
            let entry = match &outcome {
                Ok(r) => audit::Entry::new("backup_restored", actor).target(source).detail(json!({ "plays_imported": r.plays_imported, "plays_skipped": r.plays_skipped, "settings_restored": r.settings_restored })),
                Err(e) => audit::Entry::new("backup_restored", actor).target(source).detail(json!({ "error": format!("{e:#}") })).outcome("failed"),
            };
            audit::record_quietly(&c, &entry);
        }
        if outcome.is_ok() {
            if with_settings {
                match worker.db.conn().and_then(|c| Settings::load(&c)) {
                    Ok(loaded) => *worker.settings.write().unwrap() = loaded,
                    Err(e) => tracing::warn!("the restored settings could not be loaded; the ones in use stay until a restart: {e:#}"),
                }
            }
            worker.wake.notify_waiters();
        }
        worker.tasks.finish(
            "restore",
            outcome.map(|r| (format!("Restored {} plays ({} already present)", r.plays_imported, r.plays_skipped), serde_json::to_value(&r).ok())),
        );
    });
}

async fn restore_stored(State(app): State<App>, JellyfinAdmin(user): JellyfinAdmin, Path(name): Path<String>, Query(q): Query<RestoreQuery>) -> ApiResult<Response> {
    let path = backup_path(&app, &name)?;
    if !app.tasks.try_start_alone("restore", "Reading backup", &HISTORY_WRITERS) {
        return Err(ApiError::new(StatusCode::CONFLICT, "A restore or an import is already running"));
    }
    spawn_restore(&app, path, q.settings.unwrap_or(true), false, Actor::from(&user), name);
    Ok((StatusCode::ACCEPTED, Json(json!({ "ok": true }))).into_response())
}

async fn restore_upload(State(app): State<App>, JellyfinAdmin(user): JellyfinAdmin, Query(q): Query<RestoreQuery>, req: Request) -> ApiResult<Response> {
    if !app.tasks.try_start_alone("restore", "Receiving backup", &HISTORY_WRITERS) {
        return Err(ApiError::new(StatusCode::CONFLICT, "A restore or an import is already running"));
    }
    let path = app.data_dir.join("restore-upload.tmp");
    let received = async {
        let mut file = tokio::fs::File::create(&path).await?;
        let mut stream = req.into_body().into_data_stream();
        let mut total = 0u64;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| anyhow::anyhow!("The upload was interrupted: {e}"))?;
            total += chunk.len() as u64;
            file.write_all(&chunk).await?;
        }
        file.flush().await?;
        anyhow::Ok(total)
    }
    .await;
    match received {
        Ok(n) if n > 0 => {}
        other => {
            let _ = tokio::fs::remove_file(&path).await;
            let msg = other.map(|_| "The uploaded file was empty".to_string()).unwrap_or_else(|e| format!("{e:#}"));
            app.tasks.finish("restore", Err(anyhow::anyhow!(msg.clone())));
            return Err(ApiError::bad_request(msg));
        }
    }
    spawn_restore(&app, path, q.settings.unwrap_or(true), true, Actor::from(&user), "upload".into());
    Ok((StatusCode::ACCEPTED, Json(json!({ "ok": true }))).into_response())
}

/// The trackers finstats can take history from, each with its own task so each Settings card can
/// watch its own import. Only one runs at a time whichever it is: they write to the same tables.
/// The two imports and the restore. Each holds the database in one long transaction: only one of them at a time.
const HISTORY_WRITERS: [&str; 3] = ["import", "import_streamystats", "restore"];

async fn import_jellystat(State(app): State<App>, Manager(user): Manager, req: Request) -> ApiResult<Response> {
    let path = receive(&app, req, "import", "jellystat-upload.tmp").await?;
    let actor = Actor::from(&user);
    audit::record_now(&app, audit::Entry::new("import_started", actor.clone()).target("jellystat")).await;
    let worker = app.clone();
    tokio::task::spawn_blocking(move || {
        let outcome = import::run(&worker.db, &path, Some(&worker.tasks));
        let _ = std::fs::remove_file(&path);
        import_finished(&worker, actor, "jellystat", outcome.as_ref().map(|r| (r.plays_imported, r.plays_skipped)).map_err(|e| format!("{e:#}")));
        worker.tasks.finish("import", outcome.map(|r| (format!("Imported {} plays ({} already present)", r.plays_imported, r.plays_skipped), serde_json::to_value(&r).ok())));
    });
    Ok((StatusCode::ACCEPTED, Json(json!({ "ok": true }))).into_response())
}

async fn import_streamystats(State(app): State<App>, Manager(user): Manager, req: Request) -> ApiResult<Response> {
    let path = receive(&app, req, "import_streamystats", "streamystats-upload.tmp").await?;
    let actor = Actor::from(&user);
    audit::record_now(&app, audit::Entry::new("import_started", actor.clone()).target("streamystats")).await;
    let worker = app.clone();
    tokio::task::spawn_blocking(move || {
        let outcome = streamystats::run(&worker.db, &path, Some(&worker.tasks));
        let _ = std::fs::remove_file(&path);
        import_finished(&worker, actor, "streamystats", outcome.as_ref().map(|r| (r.plays_imported, r.plays_skipped)).map_err(|e| format!("{e:#}")));
        worker.tasks.finish(
            "import_streamystats",
            outcome.map(|r| {
                let mut msg = format!("Imported {} plays ({} already present)", r.plays_imported, r.plays_skipped);
                if r.marked_watched > 0 {
                    msg.push_str(&format!(", {} marked watched but never played", r.marked_watched));
                }
                (msg, serde_json::to_value(&r).ok())
            }),
        );
    });
    Ok((StatusCode::ACCEPTED, Json(json!({ "ok": true }))).into_response())
}

/// Stream an uploaded export to disk. One import at a time, whichever tracker it came from.
async fn receive(app: &App, req: Request, task: &'static str, name: &str) -> ApiResult<std::path::PathBuf> {
    if !app.tasks.try_start_alone(task, "Receiving backup", &HISTORY_WRITERS) {
        return Err(ApiError::new(StatusCode::CONFLICT, "An import or a restore is already running"));
    }
    let path = app.data_dir.join(name);
    let received = async {
        let mut file = tokio::fs::File::create(&path).await?;
        let mut stream = req.into_body().into_data_stream();
        let mut total = 0u64;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| anyhow::anyhow!("The upload was interrupted: {e}"))?;
            total += chunk.len() as u64;
            file.write_all(&chunk).await?;
        }
        file.flush().await?;
        anyhow::Ok(total)
    }
    .await;

    match received {
        Ok(0) => {
            let _ = tokio::fs::remove_file(&path).await;
            app.tasks.finish(task, Err(anyhow::anyhow!("The uploaded file was empty")));
            Err(ApiError::bad_request("The uploaded file was empty"))
        }
        Ok(total) => {
            tracing::info!("received a history export ({:.1} MB)", total as f64 / 1e6);
            Ok(path)
        }
        Err(e) => {
            let _ = tokio::fs::remove_file(&path).await;
            let msg = format!("{e:#}");
            app.tasks.finish(task, Err(e));
            Err(ApiError::bad_request(msg))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_jellyfin_administrator_lets_profiles_be_public() {
        assert!(ACCESS_KEYS.contains(&"public_profiles"));
    }

    #[test]
    fn only_a_jellyfin_administrator_says_where_people_open_jellyfin() {
        // Every "Open in Jellyfin" button points there: a manager must not be able to send everyone elsewhere.
        assert!(ACCESS_KEYS.contains(&"jellyfin_public_url"));
    }

    #[test]
    fn both_ways_of_importing_history_and_the_restore_have_a_task_of_their_own() {
        // `Tasks::try_start` panics on an id it does not know, and these are started from their own
        // Settings cards rather than through `RUNNABLE`, so nothing else checks them.
        for id in HISTORY_WRITERS {
            assert!(crate::state::TASK_IDS.contains(&id), "`{id}` would panic in Tasks::try_start");
            let tasks = crate::state::Tasks::new();
            assert!(tasks.try_start_alone(id, "Receiving backup", &HISTORY_WRITERS));
            assert!(!tasks.try_start_alone(id, "Receiving backup", &HISTORY_WRITERS), "`{id}` must refuse a second run");
            // Each one refuses while another is going: they write to the same tables.
            assert!(HISTORY_WRITERS.iter().all(|other| !tasks.try_start_alone(other, "Receiving backup", &HISTORY_WRITERS)));
        }
    }

    #[test]
    fn every_task_that_can_be_started_is_known_and_has_exactly_one_runner() {
        for id in RUNNABLE {
            assert!(crate::state::TASK_IDS.contains(&id), "`{id}` would panic in Tasks::try_start");
            assert_eq!(sync::TASKS.contains(&id) as u8 + services::TASKS.contains(&id) as u8 + OWN_RUNNERS.contains(&id) as u8, 1, "`{id}` needs one runner");
        }
        for id in sync::TASKS.iter().chain(services::TASKS.iter()) {
            assert!(RUNNABLE.contains(id), "`{id}` cannot be started by hand");
        }
        // As in Jellyfin, every job that runs by itself also runs when somebody presses Run.
        for id in crate::schedule::SCHEDULED {
            assert!(RUNNABLE.contains(&id), "`{id}` is scheduled but cannot be started by hand");
        }
    }

    #[test]
    fn a_write_is_cross_site_only_when_its_origin_is_not_us() {
        let us = "finstats.example";
        // Same origin: allowed. (`Host` carries no scheme; `Origin` does.)
        assert!(!cross_site(Some("https://finstats.example"), Some(us), None, false));
        assert!(!cross_site(Some("http://finstats.example"), Some(us), None, false));
        // No Origin at all (a native client): not judged here — a cookie a browser only attaches to a
        // navigation, or a Bearer key no browser adds on its own.
        assert!(!cross_site(None, Some(us), None, false));
        // A foreign origin is refused.
        assert!(cross_site(Some("https://evil.example"), Some(us), None, false));
    }

    #[test]
    fn a_client_supplied_forwarded_host_cannot_vouch_for_a_foreign_origin() {
        let us = "finstats.example";
        // The bug: without a trusted proxy, `Origin: evil` + `X-Forwarded-Host: evil` must still be
        // refused — a client sets that header itself, so honouring it lets any origin vouch for itself.
        assert!(cross_site(Some("https://evil.example"), Some(us), Some("evil.example"), false));
        // Behind a trusted proxy the browser's Origin is the public name and Host is the internal one,
        // so the proxy's X-Forwarded-Host is exactly what the origin should match: allowed.
        assert!(!cross_site(Some("https://finstats.example"), Some("127.0.0.1:8080"), Some("finstats.example"), true));
        // …but even trusted, the forwarded host must actually match the origin.
        assert!(cross_site(Some("https://evil.example"), Some("127.0.0.1:8080"), Some("finstats.example"), true));
    }
}
