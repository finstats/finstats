// Now playing, live, as both layouts draw it: one loader and one clock. The list is pushed over the live channel when it
// changes and asked for every 5 s only while that is not there; between two answers only the ticker moves a clock.

import { h, mount, clock } from './dom.js';
import { api, isAbort } from './api.js';
import { follow } from './live.js';

/**
 * The live "Now playing" block. The server is asked every few seconds; in between, the clock and
 * the bar of every playing session advance locally once a second so they move like a player does.
 *
 * Clients report their position to Jellyfin only every ten seconds or so, which means a poll often
 * returns a position that is a little *behind* what we are already showing. Snapping to it would
 * make the clock stutter backwards, so the local clock is kept unless the server disagrees in a way
 * that means something: it is ahead, the viewer paused, or the gap is too big to be reporting lag
 * (a skip).
 */
const BEHIND_OK_S = 15; // the server may trail us by this much: that is just reporting lag
const AHEAD_OK_S = 3;   // and lead us by this much: rounding and poll timing, not news
/** `list(sessions)` draws the cards, each carrying `data-np-key`; `parts(card)` finds in one the clock to move and the bar
 *  to fill: `{ clock, fill, bar, also }`, any of them absent; `also(session, position)` is said on the same beat, for what
 *  else on a card follows the position or the latest answer. Both layouts draw their own cards on this one clock. */
export function nowPlayingView({ list, parts, cls = 'np-live' }) {
  const root = h('div', { class: cls });
  let live = new Map(); // key -> { s, pos (whole seconds), shape, clockEl, fillEl, barEl }
  // Everything about a session except the numbers that change every second: if this is the same,
  // the card on screen is still right and only its clock needs attention.
  const shapeOf = (s) => JSON.stringify({ ...s, position_s: 0, watched_s: 0, transcode: s.transcode ? { ...s.transcode, progress: 0 } : null });
  function paint(l) {
    const pos = l.s.runtime_s ? Math.min(l.pos, l.s.runtime_s) : l.pos;
    if (l.clockEl) l.clockEl.textContent = l.s.runtime_s ? `${clock(pos)} / ${clock(l.s.runtime_s)}` : clock(pos);
    if (l.fillEl && l.s.runtime_s) {
      const pr = Math.max(0, Math.min(1, pos / l.s.runtime_s));
      l.fillEl.style.width = pr * 100 + '%';
      if (l.barEl) l.barEl.setAttribute('aria-valuenow', String(Math.round(pr * 100)));
    }
    if (l.also) l.also(l.s, pos);
  }
  // The ticker is the only thing that moves a clock: exactly one second per beat, so the display
  // is a metronome. A poll landing between two beats must never nudge the number off the beat.
  const timer = setInterval(() => { for (const l of live.values()) if (!l.s.is_paused) { l.pos += 1; paint(l); } }, 1000);

  function update(sessions) {
    const next = new Map();
    // An empty view has drawn nothing yet, not even "nothing is playing": zero cards is not "the same cards".
    let sameCards = sessions.length === live.size && root.hasChildNodes();
    const corrected = [];
    for (const s of sessions) {
      const old = live.get(s.key);
      const shape = shapeOf(s);
      const server = Math.floor(s.position_s ?? 0);
      let pos = server;
      if (old && old.shape === shape && !s.is_paused) {
        const drift = server - old.pos; // > 0: the server is ahead of what we show
        if (drift <= AHEAD_OK_S && -drift <= BEHIND_OK_S) pos = old.pos; // close enough: keep counting, stay on the beat
      }
      if (!old || old.shape !== shape) sameCards = false;
      const l = { s, pos, shape, clockEl: old && old.clockEl, fillEl: old && old.fillEl, barEl: old && old.barEl, also: old && old.also };
      if (old && pos !== old.pos) corrected.push(l); // a skip, a pause, or real drift (a throttled background tab)
      next.set(s.key, l);
    }
    live = next;
    if (!sameCards) {
      // Someone started, stopped, paused or changed quality: draw the cards again.
      mount(root, list(sessions));
      for (const l of live.values()) {
        const card = root.querySelector(`[data-np-key="${CSS.escape(l.s.key)}"]`);
        const found = card ? parts(card) : {};
        l.clockEl = found.clock || null;
        l.barEl = found.bar || null;
        l.fillEl = found.fill || null;
        l.also = found.also || null;
        paint(l);
      }
    } else {
      // Even a correction waits for the beat: set the clock one short and let the next tick land
      // on the right second, so the number only ever changes in time with the others. A paused clock
      // has no beat coming (the ticker skips it), so a paused viewer who scrubbed is shown at once.
      for (const l of corrected) { if (l.s.is_paused) paint(l); else l.pos -= 1; }
    }
  }
  return { el: root, update, destroy: () => clearInterval(timer) };
}

/** Feed `paint(data)` with `/now-playing` for as long as the page is open; `failed(error)` when an answer cannot be had. */
export function followNowPlaying(ctx, { paint, failed = () => {} }) {
  // An answer asked for before the live channel last spoke is older than what it said, and the channel speaks only on
  // change: painted after it, a play that had ended kept its card until something else happened.
  let pushes = 0;
  async function load() {
    const asked = pushes;
    try {
      const d = await api.get('/now-playing', null, { signal: ctx.signal });
      if (asked === pushes) paint(d);
    } catch (e) {
      if (isAbort(e) || e.status === 401) return;
      failed(e);
    }
  }
  load();
  follow('now_playing', { poll: load, every: 5000, onData: (d) => { pushes += 1; paint(d); }, signal: ctx.signal });
}
