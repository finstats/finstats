// The live channel: one WebSocket per tab, from FinStats to this page, saying when something the page already shows has
// changed (what is playing, what is downloading, the status bar's numbers). Pages still load over HTTP; this replaces only
// the polls that kept them current.
//
// A page follows a topic with the poll it would otherwise run. While the socket is not live (blocked at a proxy, a
// server restarting, a network change) that poll runs exactly as before: the same endpoint, the same interval, paused
// while the tab is hidden. It stops again the moment the socket is back. Nobody is told: the page looks the same either way.

import { lang } from './i18n.js';

const RETRY_FIRST_MS = 1000;
const RETRY_MAX_MS = 15000;
// A path that died silently (a laptop woken, a proxy that dropped it) closes nothing here, and on a quiet evening nothing
// is pushed: so a visible page asks every so often, and when it is shown again, and a socket that does not answer in
// time is given up for the poll and a new connection.
const PING_EVERY_MS = 30000;
const PONG_WITHIN_MS = 10000;

const follows = new Set();          // { topic, poll, every, visibleOnly, onData, timer }
let ws = null;
let live = false;                   // the socket said hello and is carrying
let retryMs = RETRY_FIRST_MS;
let retryTimer = null;
let watchingVisibility = false;
let away = false;                   // the page was left and may sit in the back/forward cache
let gaveWay = false;                // a newer tab of this person took the seat: this one polls until it is shown again
let pinger = null;                  // the interval that asks whether the socket still carries
let pongDue = null;                 // the timer that gives it up when no answer came

// The tab's own language: the cookie is every tab's, and a socket carries no header of the page's.
const address = () => `${location.protocol === 'https:' ? 'wss' : 'ws'}://${location.host}/api/live?lang=${encodeURIComponent(lang())}`;
const send = (msg) => { if (live && ws) { try { ws.send(JSON.stringify(msg)); } catch { /* closing */ } } };
const topics = () => [...new Set([...follows].map((f) => f.topic))];

/** The poll a follow falls back to: `fn` every `every` ms, skipped while hidden when `visibleOnly`. Each start is a chain
 *  of its own, and a chain that is no longer the follow's stops: a poll still on its way when the socket came back and
 *  went again would otherwise schedule its next turn beside the new chain's, and the page would ask twice as often. */
function startPolling(f, at = f.every) {
  if (f.chain) return;
  const chain = {};
  f.chain = chain;
  const tick = async () => {
    if (f.chain !== chain) return;
    if (live || !follows.has(f)) { f.chain = null; return; }
    if (!(f.visibleOnly && document.hidden)) { try { await f.poll(); } catch (e) { if (e.name !== 'AbortError') console.warn(e); } }
    if (f.chain !== chain) return;
    if (!live && follows.has(f)) f.timer = setTimeout(tick, f.every); else f.chain = null;
  };
  f.timer = setTimeout(tick, at);
}
function stopPolling(f) { clearTimeout(f.timer); f.timer = null; f.chain = null; }

function stopPinging() { clearInterval(pinger); pinger = null; clearTimeout(pongDue); pongDue = null; }
/** Ask whether `sock` still carries; with no answer in time, give it up as if it had closed. */
function ping(sock) {
  if (ws !== sock || !live || pongDue) return;
  send({ type: 'ping' });
  pongDue = setTimeout(() => {
    pongDue = null;
    if (ws !== sock) return;
    ws = null;
    stopPinging();
    try { sock.close(); } catch { /* already gone */ }
    fallBack(true);   // nothing has been heard for a while: what the page shows may be stale already
    reconnectLater();
  }, PONG_WITHIN_MS);
}

/** Poll while the socket is not live; `now` when what the page shows may already be out of date. */
function fallBack(now = false) {
  live = false;
  for (const f of follows) startPolling(f, now ? 0 : f.every);
}

function connect() {
  if (ws || away || gaveWay || !follows.size) return;
  let sock;
  try { sock = new WebSocket(address()); } catch { fallBack(); reconnectLater(); return; }
  ws = sock;
  sock.onmessage = (e) => {
    // Anything heard is an answer: the socket carries.
    clearTimeout(pongDue); pongDue = null;
    let msg; try { msg = JSON.parse(e.data); } catch { return; }
    if (msg.type === 'pong') return;
    if (msg.type === 'hello') {
      live = true;
      retryMs = RETRY_FIRST_MS;
      for (const f of follows) stopPolling(f);
      send({ type: 'visible', visible: !document.hidden });
      send({ type: 'subscribe', topics: topics() });
      stopPinging();
      pinger = setInterval(() => { if (!document.hidden) ping(sock); }, PING_EVERY_MS);
      return;
    }
    // A topic this person may not have: their page's own GET says so; nothing to poll either.
    if (msg.type === 'denied') return;
    for (const f of follows) if (f.topic === msg.type) { try { f.onData(msg.data); } catch (err) { console.warn(err); } }
  };
  sock.onclose = (e) => {
    if (ws !== sock) return;
    ws = null;
    stopPinging();
    if (away) { live = false; return; }
    fallBack();
    // 4429: this person has more tabs listening than FinStats keeps, and this was the oldest. It asks, as before, and
    // does not take a seat back at its next page, which would only let the next oldest tab go.
    if (e.code === 4429) gaveWay = true;
    else reconnectLater();
  };
  sock.onerror = () => {}; // a close always follows
}

function reconnectLater() {
  if (retryTimer || !follows.size) return;
  // A little jitter, so a restarted server is not met by every tab in the same instant.
  const wait = retryMs * (0.8 + Math.random() * 0.4);
  retryMs = Math.min(retryMs * 2, RETRY_MAX_MS);
  retryTimer = setTimeout(() => { retryTimer = null; connect(); }, wait);
}

function watchVisibility() {
  if (watchingVisibility) return;
  watchingVisibility = true;
  // The app's own listeners, for as long as the tab lives: a hidden tab stops vouching that somebody watches the queue.
  document.addEventListener('visibilitychange', () => {
    send({ type: 'visible', visible: !document.hidden });
    // Shown again after a while (a laptop opened): make sure the socket survived it before trusting it.
    if (!document.hidden && ws) ping(ws);
  });
  // A page left for another may be kept, frozen, in the back/forward cache: its socket would stay open with it and hold one
  // of its person's seats until the browser let the page go. So it is given up as the page is left, and taken up again
  // if the page is shown once more.
  window.addEventListener('pagehide', () => {
    away = true;
    clearTimeout(retryTimer); retryTimer = null;
    for (const f of follows) stopPolling(f);
    stopPinging();
    if (ws) { const s = ws; ws = null; live = false; s.close(1001); }
  });
  window.addEventListener('pageshow', (e) => {
    if (!away && !e.persisted) return;
    away = false;
    gaveWay = false;
    retryMs = RETRY_FIRST_MS;
    fallBack();
    connect();
  });
}

/**
 * Keep `topic` current: `onData(data)` with what its GET would answer, pushed when it changes; `poll()` every `every` ms
 * whenever the socket is not live. Stops with `signal` (a page's `ctx.signal`) or the returned function.
 */
export function follow(topic, { poll, every, visibleOnly = true, onData, signal }) {
  const f = { topic, poll, every, visibleOnly, onData, timer: null, chain: null };
  follows.add(f);
  watchVisibility();
  const stop = () => {
    if (!follows.delete(f)) return;
    stopPolling(f);
    if (!topics().includes(topic)) send({ type: 'unsubscribe', topics: [topic] });
    if (!follows.size && ws) { const s = ws; ws = null; live = false; stopPinging(); s.close(); }
  };
  signal?.addEventListener('abort', stop, { once: true });
  if (live) send({ type: 'subscribe', topics: [topic] });
  else { startPolling(f); connect(); }
  return stop;
}
