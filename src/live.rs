//! The live channel: one WebSocket per browser tab, from FinStats to its own pages, carrying what changes on its own
//! (who is playing, what is downloading, the status bar's numbers). Page loads stay on HTTP; this only tells a page that
//! something it already shows has changed.

use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::extract::ws::rejection::WebSocketUpgradeRejection;
use axum::extract::ws::{CloseFrame, Message, WebSocket, WebSocketUpgrade};
use axum::extract::{ConnectInfo, State};
use axum::http::header::{HOST, ORIGIN};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use serde_json::{Value, json};
use tokio::sync::watch;

use crate::auth::{AuthUser, Presented};
use crate::state::{ApiError, App};

/// What a client may subscribe to. More (jobs, tasks, notifications) fit the same frame.
pub const TOPICS: [&str; 3] = ["now_playing", "downloads", "summary"];

/// The version every frame carries, per topic. A frame's `data` is exactly what the topic's GET answers.
pub const FRAME_V: u32 = 1;

/// How far ahead of the clock a page shows the server may be before a push corrects it (`AHEAD_OK_S` in `widgets.js`).
const AHEAD_OK_S: i64 = 3;
/// And how far behind: clients report to Jellyfin about every 10 s, so a position trails by that much (`BEHIND_OK_S`).
const BEHIND_OK_S: i64 = 15;

/// A message from the page.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMsg {
    Subscribe { topics: Vec<String> },
    Unsubscribe { topics: Vec<String> },
    /// Whether the tab is on screen. A hidden tab keeps its subscriptions but stops vouching for anybody looking.
    Visible { visible: bool },
    /// Does this connection still carry? Answered `{"type": "pong"}` at once. A path that died silently closes nothing
    /// on the page's side, and on a quiet evening nothing is pushed: only an answer it asked for tells the two apart.
    Ping,
}

/// One frame of a topic: `{"type": topic, "v": 1, "data": …}`.
pub fn frame(topic: &str, data: &Value) -> String {
    json!({ "type": topic, "v": FRAME_V, "data": data }).to_string()
}

/// Everything a now-playing card draws, minus the numbers that move every second (`shapeOf` in `widgets.js`): the
/// position, the time watched and how far a transcode has got. Two readings of the same shape need no push.
pub fn shape(sessions: &[Value]) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    for s in sessions {
        let mut s = s.clone();
        s["position_s"] = json!(0);
        s["watched_s"] = json!(0);
        if s["transcode"].is_object() {
            s["transcode"]["progress"] = json!(0);
        }
        s.to_string().hash(&mut h);
    }
    h.finish()
}

/// Where a page last drew a stream's clock: a position, when, and whether it was counting.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shown {
    pub pos: i64,
    pub at: i64,
    pub paused: bool,
}

/// Whether a stream's position has left the course its clock is on: a seek, or drift the page would correct. A page's
/// clock runs on from the last position it was given, one second per second while playing; a reading within the
/// page's own tolerance of that is not news.
pub fn off_course(shown: &Shown, pos: i64, now: i64) -> bool {
    let expected = if shown.paused { shown.pos } else { shown.pos + (now - shown.at) };
    let drift = pos - expected;
    drift > AHEAD_OK_S || -drift > BEHIND_OK_S
}

/// The clocks pages were last given, by stream key.
#[derive(Debug, Default)]
pub struct Course(HashMap<String, Shown>);

impl Course {
    /// Remember what was just pushed.
    pub fn set(&mut self, sessions: &[Value], now: i64) {
        self.0 = sessions
            .iter()
            .filter_map(|s| Some((s["key"].as_str()?.to_string(), Shown { pos: s["position_s"].as_i64().unwrap_or(0), at: now, paused: s["is_paused"].as_bool().unwrap_or(false) })))
            .collect();
    }

    /// Whether any stream in this reading has left its course.
    pub fn moved(&self, sessions: &[Value], now: i64) -> bool {
        sessions.iter().any(|s| {
            let (Some(key), Some(pos)) = (s["key"].as_str(), s["position_s"].as_i64()) else { return false };
            self.0.get(key).is_some_and(|shown| off_course(shown, pos, now))
        })
    }
}

/// What one page was last sent of now playing: the shape of its cards and where their clocks were set. A page is sent
/// now playing again only when its own answer is news by these, never because the server's picture moved: somebody
/// else's pause is not news to a person who sees only their own streams, and a frame sent just then would say when it
/// happened.
#[derive(Debug, Default)]
pub struct Drawn {
    shape: Option<u64>,
    course: Course,
}

impl Drawn {
    pub fn news(&self, own: &[Value], now: i64) -> bool {
        self.shape != Some(shape(own)) || self.course.moved(own, now)
    }

    pub fn set(&mut self, own: &[Value], now: i64) {
        self.shape = Some(shape(own));
        self.course.set(own, now);
    }
}

/// Whether a connection vouches for somebody looking at the queue: only a visible tab subscribed to it, the rule
/// `visibleOnly` held for the poll it replaces. A forgotten background tab must not keep Sonarr and Radarr on 5 s.
pub fn stamps(subscribed: bool, visible: bool) -> bool {
    subscribed && visible
}

/// How many live connections one person may hold at once: a tab each, and a person rarely has more than a few. Beyond it
/// the oldest gives way, as the oldest session does: a page frozen in a browser's back/forward cache, or a connection
/// that died somewhere on the way without a word, would otherwise hold a seat that nobody is using.
pub const PER_PERSON: usize = 20;

/// One connection's seat: its number, and how it is told that a newer connection of the same person took its place.
pub struct Ticket {
    pub id: u64,
    pub gone: Arc<tokio::sync::Notify>,
}

/// One person's connections, oldest first: each one's number and how it is told to go.
type Held = std::collections::VecDeque<(u64, Arc<tokio::sync::Notify>)>;

/// Who holds which connections.
#[derive(Default)]
pub struct Seats {
    held: Mutex<HashMap<String, Held>>,
    next: std::sync::atomic::AtomicU64,
}

impl Seats {
    /// A seat for this person. One who already holds [`PER_PERSON`] has their oldest told to go.
    pub fn take(&self, user_id: &str) -> Ticket {
        let id = self.next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let gone = Arc::new(tokio::sync::Notify::new());
        let mut held = self.held.lock().unwrap();
        let mine = held.entry(user_id.to_string()).or_default();
        if mine.len() >= PER_PERSON
            && let Some((_, oldest)) = mine.pop_front()
        {
            oldest.notify_one(); // kept as a permit if that connection is not listening at this instant
        }
        mine.push_back((id, gone.clone()));
        Ticket { id, gone }
    }

    /// The connection has ended. A seat already given up to a newer one changes nothing.
    pub fn leave(&self, user_id: &str, id: u64) {
        let mut held = self.held.lock().unwrap();
        if let Some(mine) = held.get_mut(user_id) {
            mine.retain(|(n, _)| *n != id);
            if mine.is_empty() {
                held.remove(user_id);
            }
        }
    }

    /// Connections open now.
    pub fn open(&self) -> usize {
        self.held.lock().unwrap().values().map(|m| m.len()).sum()
    }
}

// ---------------------------------------------------------------- the feed: one latest value per topic

/// Ping a quiet connection this often: on an evening nobody watches nothing is pushed, and a proxy closes a socket that
/// says nothing (nginx after 60 s by default).
const PING_EVERY: Duration = Duration::from_secs(25);
/// A connection that has answered nothing, not even a ping, for this long is gone, and its task ends.
const SILENT_FOR: Duration = Duration::from_secs(80);
/// Re-read who a connection speaks for at least this often, whatever else happens: a socket lives for hours, and a
/// session signed out, a key revoked or a demotion read from Jellyfin must reach it as it reaches a request.
const RESOLVE_EVERY: Duration = Duration::from_secs(30);
/// The status bar's numbers also come from the database (plays, the last library read), which nothing in memory
/// announces: they are asked again this often while somebody shows them, and sent only when they differ.
const SUMMARY_EVERY: Duration = Duration::from_secs(30);
/// How often a visible tab on the queue stamps it as watched: inside the queue's own `WATCHING_FOR_S` (20 s).
const STAMP_EVERY: Duration = Duration::from_secs(10);
/// How often the watcher looks at what the collector and the queue last wrote.
const LOOK_EVERY: Duration = Duration::from_secs(1);
/// What a page may send: four kinds of short message.
const MESSAGE_MAX: usize = 4096;
/// A page that stops reading must not hold its connection's task: a send that cannot be written within this long ends
/// the connection. Without it a reader who never read pinned the task, its socket and its buffers for good, a seat given
/// way included (the task that would have closed it was the one waiting).
const SEND_WITHIN: Duration = Duration::from_secs(10);
/// Close codes: the credential no longer opens FinStats.
const CLOSE_SIGNED_OUT: u16 = 4401;
/// …and: this person opened more than [`PER_PERSON`], and this was the oldest. The page polls rather than coming back.
const CLOSE_REPLACED: u16 = 4429;

/// What changed, by topic: a latest value, never a queue, so a slow page only ever gets the newest state.
pub struct Feed {
    /// Everybody's streams with groups marked, as the last change left them. Each connection narrows its own.
    now_playing: watch::Sender<Arc<Vec<Value>>>,
    downloads: watch::Sender<u64>,
    summary: watch::Sender<u64>,
    /// Somebody's access may have changed (a permission, a sign-out, a key, a users read): re-read yours now.
    auth: watch::Sender<u64>,
    pub seats: Seats,
    /// When a refusal was last logged, so a reconnecting script cannot flood the log.
    refused_logged: Mutex<Option<Instant>>,
    /// When a connection last gave way to a newer one, likewise.
    replaced_logged: Mutex<Option<Instant>>,
}

impl Default for Feed {
    fn default() -> Self {
        Feed {
            now_playing: watch::Sender::new(Arc::new(vec![])),
            downloads: watch::Sender::new(0),
            summary: watch::Sender::new(0),
            auth: watch::Sender::new(0),
            seats: Seats::default(),
            refused_logged: Mutex::new(None),
            replaced_logged: Mutex::new(None),
        }
    }
}

impl Feed {
    /// Somebody's access may have changed: every open connection re-reads its own before it says anything more.
    pub fn recheck(&self) {
        self.auth.send_modify(|n| *n += 1);
    }

    fn bump(topic: &watch::Sender<u64>) {
        topic.send_modify(|n| *n += 1);
    }

    fn refused(&self, why: &str) {
        if Self::once_a_minute(&self.refused_logged) {
            tracing::warn!("live channel: refused a connection ({why}); further refusals this minute are not logged");
        }
    }

    fn noted(&self, what: &str) {
        if Self::once_a_minute(&self.replaced_logged) {
            tracing::info!("live channel: {what} (at most once a minute in this log)");
        }
    }

    fn once_a_minute(last: &Mutex<Option<Instant>>) -> bool {
        let mut last = last.lock().unwrap();
        let due = last.is_none_or(|t| t.elapsed() >= Duration::from_secs(60));
        if due {
            *last = Some(Instant::now());
        }
        due
    }
}

/// The one task that notices change: what the collector and the queue last wrote, read from memory once a second.
/// It asks Jellyfin, Sonarr and Radarr for nothing, and does nothing while no page is listening.
pub async fn run(app: App) {
    let mut course = Course::default();
    let (mut shown_shape, mut shown_keys, mut queue_at, mut collector) = (None, None, None, None);
    let mut tick = tokio::time::interval(LOOK_EVERY);
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    loop {
        tick.tick().await;
        let feed = &app.live_feed;
        if feed.seats.open() == 0 {
            // Nobody listening: the next connection is sent everything on subscribing anyway.
            (shown_shape, shown_keys, queue_at, collector) = (None, None, None, None);
            continue;
        }
        let now = crate::db::now();
        let marked = crate::stats::marked_live(&app);
        let sh = shape(&marked);
        if shown_shape != Some(sh) || course.moved(&marked, now) {
            course.set(&marked, now);
            shown_shape = Some(sh);
            let keys: Vec<String> = marked.iter().filter_map(|s| s["key"].as_str().map(str::to_string)).collect();
            // Somebody beginning or ending changes the status bar's count; a pause does not.
            if shown_keys.as_ref() != Some(&keys) {
                shown_keys = Some(keys);
                Feed::bump(&feed.summary);
            }
            feed.now_playing.send_replace(Arc::new(marked));
        }
        let at = app.downloads.read().unwrap().at;
        if queue_at != Some(at) {
            queue_at = Some(at);
            Feed::bump(&feed.downloads);
        }
        let c = app.collector.read().unwrap().clone();
        let state = (c.connected, c.socket_live, c.active_sessions);
        if collector != Some(state) {
            collector = Some(state);
            Feed::bump(&feed.summary);
        }
    }
}

// ---------------------------------------------------------------- one connection

/// A seat held for as long as its connection lives, given back however the connection ends, the upgrade failing
/// before it began included.
struct Seat {
    app: App,
    user_id: String,
    id: u64,
}

impl Drop for Seat {
    fn drop(&mut self) {
        self.app.live_feed.seats.leave(&self.user_id, self.id);
    }
}

/// Which topics this person may have.
fn allowed(user: &AuthUser) -> Vec<&'static str> {
    TOPICS.into_iter().filter(|t| *t != "downloads" || user.perms.see_downloads).collect()
}

/// `GET /api/live`: the upgrade. Checked like a write (the `Origin` must be ours, `X-Forwarded-Host` only behind a
/// trusted proxy), because a browser attaches the session cookie to a cross-site WebSocket as readily as to a page; then
/// the cookie or a `Bearer` key, never a key in the address, and never a calendar key.
pub async fn connect(
    State(app): State<App>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    ws: Result<WebSocketUpgrade, WebSocketUpgradeRejection>,
) -> Response {
    let header = |name: &str| headers.get(name).and_then(|v| v.to_str().ok());
    if crate::api::cross_site(header(ORIGIN.as_str()), header(HOST.as_str()), header("x-forwarded-host"), app.trust_proxy) {
        app.live_feed.refused("another site's page");
        return ApiError::new(StatusCode::FORBIDDEN, "Cross-site request refused").into_response();
    }
    let ip = crate::auth::client_ip(&app, &headers, peer);
    let presented = match crate::auth::pick_credential(&headers) {
        Ok(p) => p,
        Err(e) => return e.into_response(),
    };
    let user = match crate::auth::resolve_presented(presented.clone(), Some(ip), &app).await.and_then(|u| crate::auth::refuse_calendar_key(&u).map(|()| u)) {
        Ok(u) => u,
        Err(e) => return e.into_response(),
    };
    let ws = match ws {
        Ok(ws) => ws,
        Err(_) => return ApiError::new(StatusCode::UPGRADE_REQUIRED, "This address is a WebSocket").into_response(),
    };
    let ticket = app.live_feed.seats.take(&user.id);
    let seat = Seat { app: app.clone(), user_id: user.id.clone(), id: ticket.id };
    let gone = ticket.gone;
    ws.max_message_size(MESSAGE_MAX).on_upgrade(move |socket| serve(app, socket, user, presented, seat, gone))
}

/// What one connection has asked for, and what it was last sent, so nothing is sent twice.
struct Tab {
    user: AuthUser,
    topics: HashSet<&'static str>,
    visible: bool,
    sent: HashMap<&'static str, String>,
    /// Now playing's own measure of news (see [`Drawn`]); the other topics go by `sent`.
    drawn: Drawn,
}

impl Tab {
    /// This topic's answer for this person now: the very function its GET calls.
    async fn answer(&self, app: &App, topic: &str, marked: &[Value]) -> Option<Value> {
        match topic {
            "now_playing" => Some(crate::stats::now_playing_for(marked, &self.user)),
            "downloads" => Some(crate::downloads::answer(app, &self.user)),
            "summary" => crate::stats::summary_for(app, &self.user).await.inspect_err(|e| tracing::debug!("live channel: summary: {e:#}")).ok(),
            _ => None,
        }
    }

    /// The frame to send for a topic, or `None` when the page already has exactly this.
    async fn update(&mut self, app: &App, topic: &'static str, marked: &[Value]) -> Option<String> {
        if !self.topics.contains(topic) {
            return None;
        }
        let data = self.answer(app, topic, marked).await?;
        if topic == "now_playing" {
            let own = data["sessions"].as_array().map(Vec::as_slice).unwrap_or_default();
            let now = crate::db::now();
            if self.sent.contains_key(topic) && !self.drawn.news(own, now) {
                return None;
            }
            self.drawn.set(own, now);
        }
        let f = frame(topic, &data);
        (self.sent.get(topic) != Some(&f)).then(|| {
            self.sent.insert(topic, f.clone());
            f
        })
    }

    /// Send this topic whole the next time round, whatever was sent before: a new subscription, or a tab shown again
    /// whose clocks ran slow while it was hidden (a browser throttles a hidden tab's timers to once a minute).
    fn resend(&mut self, topic: &'static str) {
        self.sent.remove(topic);
    }
}

/// Send one message, or give up after [`SEND_WITHIN`]: false means the connection is done.
async fn send(socket: &mut WebSocket, msg: Message) -> bool {
    matches!(tokio::time::timeout(SEND_WITHIN, socket.send(msg)).await, Ok(Ok(())))
}

async fn serve(app: App, mut socket: WebSocket, user: AuthUser, presented: Presented, _seat: Seat, gone: Arc<tokio::sync::Notify>) {
    let feed = &app.live_feed;
    let (mut np, mut dl, mut sum, mut auth) = (feed.now_playing.subscribe(), feed.downloads.subscribe(), feed.summary.subscribe(), feed.auth.subscribe());
    let mut tab = Tab { topics: HashSet::new(), visible: false, sent: HashMap::new(), drawn: Drawn::default(), user };
    let hello = json!({ "type": "hello", "v": FRAME_V, "topics": allowed(&tab.user) }).to_string();
    if !send(&mut socket, Message::Text(hello.into())).await {
        return;
    }
    let every = |d: Duration| {
        let mut i = tokio::time::interval_at(tokio::time::Instant::now() + d, d);
        i.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        i
    };
    let (mut ping, mut resolve, mut backstop, mut stamp) = (every(PING_EVERY), every(RESOLVE_EVERY), every(SUMMARY_EVERY), every(STAMP_EVERY));
    let mut heard = Instant::now();
    loop {
        // What to send this time round, worked out after the select so the socket is free to send it.
        let mut due: Vec<&'static str> = vec![];
        let (mut recheck, mut fresh) = (false, false);
        tokio::select! {
            msg = socket.recv() => {
                let Some(Ok(msg)) = msg else { break };
                heard = Instant::now();
                match msg {
                    Message::Close(_) => break,
                    Message::Text(text) => match serde_json::from_str::<ClientMsg>(&text) {
                        Ok(ClientMsg::Subscribe { topics }) => {
                            let may = allowed(&tab.user);
                            for t in TOPICS.into_iter().filter(|t| topics.iter().any(|x| x == t)) {
                                if may.contains(&t) {
                                    tab.topics.insert(t);
                                    tab.resend(t); // the whole state on subscribing, whatever was sent before
                                    due.push(t);
                                    fresh = true;
                                } else {
                                    let denied = json!({ "type": "denied", "v": FRAME_V, "topic": t }).to_string();
                                    if !send(&mut socket, Message::Text(denied.into())).await {
                                        return;
                                    }
                                }
                            }
                            if stamps(tab.topics.contains("downloads"), tab.visible) {
                                crate::downloads::watched(&app, crate::db::now());
                            }
                        }
                        Ok(ClientMsg::Unsubscribe { topics }) => tab.topics.retain(|t| !topics.iter().any(|x| x == t)),
                        Ok(ClientMsg::Ping) => {
                            let pong = json!({ "type": "pong", "v": FRAME_V }).to_string();
                            if !send(&mut socket, Message::Text(pong.into())).await {
                                return;
                            }
                        }
                        Ok(ClientMsg::Visible { visible }) => {
                            if visible && !tab.visible {
                                if tab.topics.contains("now_playing") {
                                    tab.resend("now_playing");
                                }
                                due.extend(["now_playing", "summary"]);
                            }
                            tab.visible = visible;
                            if stamps(tab.topics.contains("downloads"), tab.visible) {
                                crate::downloads::watched(&app, crate::db::now());
                            }
                        }
                        Err(_) => {}
                    },
                    _ => {}
                }
            }
            Ok(()) = np.changed() => due.push("now_playing"),
            Ok(()) = dl.changed() => due.push("downloads"),
            Ok(()) = sum.changed() => due.push("summary"),
            Ok(()) = auth.changed() => recheck = true,
            _ = resolve.tick() => recheck = true,
            // A hidden tab shows no status bar: it is asked again when it is shown.
            _ = backstop.tick() => if tab.visible { due.push("summary") },
            _ = stamp.tick() => {
                if stamps(tab.topics.contains("downloads"), tab.visible) {
                    crate::downloads::watched(&app, crate::db::now());
                }
            }
            _ = gone.notified() => {
                app.live_feed.noted("one person's oldest connection gave way to a newer one");
                let bye = CloseFrame { code: CLOSE_REPLACED, reason: "a newer tab took this seat".into() };
                let _ = send(&mut socket, Message::Close(Some(bye))).await;
                break;
            }
            _ = ping.tick() => {
                if heard.elapsed() > SILENT_FOR || !send(&mut socket, Message::Ping(Default::default())).await {
                    break;
                }
            }
        }
        if recheck {
            match crate::auth::resolve_presented(presented.clone(), tab.user.ip, &app).await {
                Ok(user) if crate::auth::refuse_calendar_key(&user).is_ok() => {
                    let lost: Vec<&'static str> = tab.topics.iter().copied().filter(|t| !allowed(&user).contains(t)).collect();
                    let before = std::mem::replace(&mut tab.user, user);
                    for t in lost {
                        tab.topics.remove(t);
                        let denied = json!({ "type": "denied", "v": FRAME_V, "topic": t }).to_string();
                        if !send(&mut socket, Message::Text(denied.into())).await {
                            return;
                        }
                    }
                    // What this person may see may have changed: every answer again, sent only where it differs. The
                    // same grants answer the same, so the half-minute re-read costs nothing more when nothing changed.
                    if before.perms != tab.user.perms {
                        due.extend(TOPICS);
                        fresh = true;
                    }
                }
                Err(e) if e.0.is_server_error() => tracing::debug!("live channel: could not re-read a connection's user: {}", e.1),
                _ => {
                    let bye = CloseFrame { code: CLOSE_SIGNED_OUT, reason: "signed out".into() };
                    let _ = send(&mut socket, Message::Close(Some(bye))).await;
                    break;
                }
            }
        }
        // A subscription or a re-read is answered from a fresh reading: while nobody listened the watcher kept nothing.
        // Otherwise the watcher's value is read without marking it seen: `changed()` marks what it wakes for, and a
        // value marked here, on a pass that woke for something else, was never sent. Somebody beginning or ending
        // announces now playing and the status bar in one instant, and a page that took the status bar first lost the
        // play until the next change.
        let marked = if fresh {
            if due.contains(&"now_playing") {
                np.borrow_and_update();
            }
            Arc::new(crate::stats::marked_live(&app))
        } else {
            np.borrow().clone()
        };
        due.sort_unstable();
        due.dedup();
        for topic in due {
            if let Some(f) = tab.update(&app, topic, &marked).await
                && !send(&mut socket, Message::Text(f.into())).await
            {
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stream(key: &str, pos: i64) -> Value {
        json!({ "key": key, "user_id": "u1", "user_name": "alice", "item_id": "i1", "item_name": "Big Buck Bunny", "position_s": pos,
                "watched_s": pos, "is_paused": false, "play_method": "DirectPlay", "audio": "AAC 2.0", "subtitle": null,
                "transcode": { "video_codec": "h264", "progress": 0.1 } })
    }

    /// A page is sent now playing when what *it* shows changed: its own streams' cards or clocks. Somebody else's pause
    /// moves the server's picture, and must not reach a person who sees only their own streams, even as a frame with
    /// nothing new in it: when frames arrive would say when other people press play.
    #[test]
    fn a_page_hears_of_now_playing_only_when_its_own_answer_changed() {
        let mut drawn = Drawn::default();
        let mine = [stream("k1", 100)];
        assert!(drawn.news(&mine, 1_000), "nothing drawn yet");
        drawn.set(&mine, 1_000);
        assert!(!drawn.news(&[stream("k1", 110)], 1_010), "ten seconds on, on course");
        assert!(drawn.news(&[stream("k1", 400)], 1_010), "a seek");
        let mut paused = stream("k1", 110);
        paused["is_paused"] = json!(true);
        assert!(drawn.news(&[paused], 1_010), "a pause");
        assert!(drawn.news(&[], 1_010), "it ended");
        assert!(drawn.news(&[stream("k1", 110), stream("k2", 5)], 1_010), "another of one's own began");
    }

    #[test]
    fn the_shape_ignores_only_what_moves_every_second() {
        let a = shape(&[stream("k1", 100)]);
        let mut later = stream("k1", 101);
        later["watched_s"] = json!(101);
        later["transcode"]["progress"] = json!(0.2);
        assert_eq!(shape(&[later]), a, "a second later is the same card");

        for (field, value) in [("is_paused", json!(true)), ("item_id", json!("i2")), ("play_method", json!("Transcode")), ("audio", json!("AC3 5.1")),
                               ("subtitle", json!("English")), ("group", json!({ "size": 2, "with": [{ "user_id": "u2", "user_name": "bob" }] }))] {
            let mut changed = stream("k1", 100);
            changed[field] = value;
            assert_ne!(shape(&[changed]), a, "{field} is news");
        }
        assert_ne!(shape(&[stream("k1", 100), stream("k2", 5)]), a, "somebody else began");
        assert_ne!(shape(&[]), a, "it ended");
    }

    #[test]
    fn a_position_is_news_only_off_the_course_the_page_counts_along() {
        let playing = Shown { pos: 100, at: 1_000, paused: false };
        assert!(!off_course(&playing, 110, 1_010), "on course");
        assert!(!off_course(&playing, 100, 1_010), "a report ten seconds old is reporting lag, not a seek");
        assert!(!off_course(&playing, 113, 1_010), "a little ahead is rounding");
        assert!(off_course(&playing, 114, 1_010), "further ahead is a skip forward");
        assert!(off_course(&playing, 94, 1_010), "a rewind");
        let paused = Shown { paused: true, ..playing };
        assert!(!off_course(&paused, 100, 1_300), "a paused clock stays put however long");
        assert!(off_course(&paused, 400, 1_300), "scrubbing while paused");
    }

    #[test]
    fn the_course_follows_what_was_last_pushed() {
        let mut course = Course::default();
        course.set(&[stream("k1", 100)], 1_000);
        assert!(!course.moved(&[stream("k1", 105)], 1_005));
        assert!(course.moved(&[stream("k1", 600)], 1_005), "a seek");
        assert!(!course.moved(&[stream("k2", 600)], 1_005), "a stream never pushed is a new shape, not a seek");
        course.set(&[stream("k1", 600)], 1_005);
        assert!(!course.moved(&[stream("k1", 605)], 1_010), "the new course");
    }

    #[test]
    fn only_a_visible_subscribed_tab_vouches_for_somebody_watching_the_queue() {
        assert!(stamps(true, true));
        assert!(!stamps(true, false), "a hidden tab, like visibleOnly's paused poll");
        assert!(!stamps(false, true), "a page that does not show the queue");
        assert!(!stamps(false, false));
    }

    #[test]
    fn a_page_speaks_four_kinds_of_message_and_nothing_else() {
        let read = |s: &str| serde_json::from_str::<ClientMsg>(s).ok();
        assert_eq!(read(r#"{"type":"subscribe","topics":["now_playing","summary"]}"#), Some(ClientMsg::Subscribe { topics: vec!["now_playing".into(), "summary".into()] }));
        assert_eq!(read(r#"{"type":"unsubscribe","topics":["downloads"]}"#), Some(ClientMsg::Unsubscribe { topics: vec!["downloads".into()] }));
        assert_eq!(read(r#"{"type":"visible","visible":false}"#), Some(ClientMsg::Visible { visible: false }));
        assert_eq!(read(r#"{"type":"ping"}"#), Some(ClientMsg::Ping));
        assert_eq!(read(r#"{"type":"delete_everything"}"#), None);
        assert_eq!(read("not json"), None);
    }

    #[test]
    fn a_frame_is_the_topic_its_version_and_the_get_body() {
        let f: Value = serde_json::from_str(&frame("summary", &json!({ "plays_total": 3 }))).unwrap();
        assert_eq!(f, json!({ "type": "summary", "v": 1, "data": { "plays_total": 3 } }));
    }

    #[tokio::test]
    async fn the_oldest_connection_gives_way_to_the_twenty_first_and_a_closed_one_frees_its_seat() {
        let seats = Seats::default();
        let gone = |t: &Ticket| {
            let t = t.gone.clone();
            async move { tokio::time::timeout(std::time::Duration::from_millis(20), t.notified()).await.is_ok() }
        };
        let tickets: Vec<Ticket> = (0..PER_PERSON).map(|_| seats.take("u1")).collect();
        assert_eq!(seats.open(), PER_PERSON);
        let newest = seats.take("u1");
        assert!(gone(&tickets[0]).await, "the oldest is told to go");
        assert!(!gone(&tickets[1]).await && !gone(&newest).await, "and nobody else");
        assert_eq!(seats.open(), PER_PERSON, "still twenty");
        let other = seats.take("u2");
        assert!(!gone(&tickets[1]).await, "somebody else's connection costs alice nothing");
        seats.leave("u1", tickets[0].id); // the one told to go, closing: already given up, nothing changes
        assert_eq!(seats.open(), PER_PERSON + 1);
        for t in tickets.iter().skip(1).chain([&newest]) {
            seats.leave("u1", t.id);
        }
        seats.leave("u2", other.id);
        assert_eq!(seats.open(), 0, "every seat given back: nothing left behind");
    }
}
