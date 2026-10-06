// FinUI: search. A bar and its results that live in whatever opened them — a menu, or the page — never in a dialog over
// everything. The words go to the app (`run`), which paints groups of rows as they come; the arrows move through them,
// Enter opens one, Ctrl+Enter presses the control a row keeps beside it, Esc gives up. grow() and shrink() play it out
// of, and back into, the control that opened it.

import { h, icon } from '../../core.js';
import { cut, step, flat } from './plan.js';

export { cut, step, flat } from './plan.js';

let made = 0;
const mac = () => /Mac|iPhone|iPad/.test(navigator.platform || '');

/**
 * The search. `run(words, paint)` is called with the words as they change (and with '' at once); it calls
 * `paint(groups, note)` as often as it has something — groups: [{ title, rows: [{ label, sub, thumb, href, keep }] }], `note`
 * what to say while there is nothing (searching, nothing found). `onPick(row, event)` opens a row; `onEscape()` gives up.
 * Answers { el, bar, input, list, set(words), focus(), destroy() }.
 */
export function searchPanel({ run, onPick, onEscape = () => {}, placeholder = 'Search', label = 'Search', keys = true }) {
  const id = `fui-search-${++made}`;
  const input = h('input', { class: 'fui-search__input', type: 'search', placeholder, autocomplete: 'off', spellcheck: 'false', enterkeyhint: 'go',
    role: 'combobox', 'aria-expanded': 'true', 'aria-controls': `${id}-list`, 'aria-autocomplete': 'list', 'aria-label': label });
  const shut = h('button', { type: 'button', class: 'fui-search__close', 'aria-label': 'Close search' }, icon('x', 15));
  const bar = h('div', { class: 'fui-search__bar' }, icon('search', 16), input, h('kbd', { class: 'fui-search__esc', 'aria-hidden': 'true' }, 'Esc'), shut);
  const list = h('ul', { class: 'fui-search__list', id: `${id}-list`, role: 'listbox', 'aria-label': 'Results' });
  const keep = h('span', { hidden: true }, h('kbd', null, mac() ? '⌘' : 'Ctrl'), h('kbd', null, '↵'), ' to add to your watchlist');
  const foot = keys ? h('div', { class: 'fui-search__foot', 'aria-hidden': 'true' },
    h('span', null, h('kbd', null, '↑'), h('kbd', null, '↓'), ' to move'), h('span', null, h('kbd', null, '↵'), ' to open'), keep, h('span', null, h('kbd', null, 'Esc'), ' to close')) : null;
  const el = h('div', { class: 'fui-search' }, bar, list, foot);
  let rows = [], items = [], at = 0, asked = 0;

  function paint(groups, note) {
    rows = flat(groups);
    at = Math.min(at, Math.max(0, rows.length - 1));
    let i = 0;
    items = [];
    list.replaceChildren(...(rows.length ? groups.filter((g) => (g.rows || []).length).map((g) => [
      h('li', { class: 'fui-search__group', role: 'presentation' }, g.title),
      ...g.rows.map((r) => {
        const k = i++;
        const li = h('li', { class: 'fui-search__row', role: 'option', id: `${id}-${k}`, 'aria-selected': 'false' },
          r.thumb || null, h('span', { class: 'fui-search__text' }, h('span', { class: 'fui-search__label' }, r.label), r.sub ? h('span', { class: 'fui-search__sub' }, r.sub) : null), r.keep || null);
        li.addEventListener('pointermove', () => { if (at !== k) mark(k); });
        li.addEventListener('click', (e) => { if (!(r.keep && r.keep.contains(e.target))) onPick(r, e); });
        items.push(li);
        return li;
      })]).flat() : [h('li', { class: 'fui-search__none', role: 'presentation' }, note || 'Nothing found')]));
    mark(at);
  }
  function mark(k) {
    items[at]?.classList.remove('is-active'); items[at]?.setAttribute('aria-selected', 'false');
    at = k;
    const li = items[at];
    if (li) { li.classList.add('is-active'); li.setAttribute('aria-selected', 'true'); }
    // Named to assistive technology while the field has the focus, which is the only time it is read.
    if (li && document.activeElement === input) input.setAttribute('aria-activedescendant', li.id); else input.removeAttribute('aria-activedescendant');
    keep.hidden = !(rows[at] && rows[at].keep);
  }
  const ask = () => { const n = ++asked; at = 0; run(input.value.trim(), (groups, note) => { if (n === asked) paint(groups, note); }); };
  input.addEventListener('input', ask);
  input.addEventListener('focus', () => mark(at));
  input.addEventListener('keydown', (e) => {
    if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
      e.preventDefault();
      const k = step(at, e.key === 'ArrowDown' ? 1 : -1, items.length);
      if (k >= 0) { mark(k); items[k].scrollIntoView({ block: 'nearest' }); }
    } else if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) {
      e.preventDefault();
      const b = rows[at] && rows[at].keep && rows[at].keep.querySelector('button:not([hidden])');
      if (b) b.click();
    } else if (e.key === 'Enter') {
      e.preventDefault();
      if (rows[at]) onPick(rows[at], e);
    } else if (e.key === 'Escape') {
      e.preventDefault(); e.stopPropagation();
      onEscape();
    }
  });
  shut.addEventListener('click', () => onEscape());
  ask();
  return {
    el, bar, input, list,
    set(words) { input.value = words; ask(); },
    focus() { input.focus({ preventScroll: true }); },
    destroy() { asked += 1; el.remove(); },
  };
}

// ---------------------------------------------------------------- the motion
// One shape, never a reflow: the search is laid out at its end size from the first frame, and only its outline moves — a
// clip from exactly the shape it grows out of (the menu as it was, or the control that opened it, with its corners) to its
// own. What it grew out of is a snapshot laid over it (`ghost`), fading as it opens and back as it closes, so the menu and
// the search trade places without a cut. The curve is a sheet's: quick off the mark, long and soft into place.

const SHEET = 'cubic-bezier(.32, .72, 0, 1)';
const box = (n) => { const r = n.getBoundingClientRect(); return { x: r.left, y: r.top, w: r.width, h: r.height }; };
const shown = (n) => !!(n && n.isConnected && n.getClientRects().length);
const still = () => matchMedia('(prefers-reduced-motion: reduce)').matches;
/** The corner of a box, as a number of px, however its radius is written (a pill's 999px is half its height). */
const round = (n, b) => Math.min(parseFloat(getComputedStyle(n).borderTopLeftRadius) || 0, Math.min(b.w, b.h) / 2);
/** While the shape moves, the host's own CSS transitions (a rail's width) would move it a second way: off for the length. */
/** The shape at rest: a little past the box, so its shadow is never cut off (where the browser takes a negative inset). */
const OUT = typeof CSS !== 'undefined' && CSS.supports('clip-path', 'inset(-1px)') ? 64 : 0;
const whole = (r) => `inset(${-OUT}px round ${r + OUT}px)`;
/** An animation's end, or its length and a little more: a browser that stops drawing (a hidden tab) holds animations still,
 *  and what waits on them — a snapshot to remove, a search to take away — must not wait for ever. */
const ended = (a, ms) => Promise.race([a.finished.catch(() => {}), new Promise((r) => setTimeout(r, ms + 80))]);
/** Everything of the search but its bar: the results and the keys under them. */
const rest = (bar) => [...bar.parentElement.children].filter((n) => n !== bar);
/** Holds nest: a close begun while the open still runs must not keep the open's "none" as the value to put back, or the
 *  host loses its own transitions for good (a rail that opens shut without moving). The last to let go puts it back. */
const holds = new WeakMap();
const hold = (host) => {
  const h = holds.get(host) || { n: 0, was: host.style.transition };
  if (!h.n) { h.was = host.style.transition; host.style.transition = 'none'; }
  h.n++; holds.set(host, h);
  let done = false;
  return () => {
    if (done) return;
    done = true;
    if (--h.n === 0) { host.style.transition = h.was; holds.delete(host); }
  };
};

/**
 * Lay the search out (`show`) and grow it into place: out of the host as it was, or — `fresh`, a place that was not there
 * — out of the control `from`. `ghost()` lays a snapshot of what it grows out of over it (a menu's own, before `show`).
 */
export function grow({ from, host, bar, list, show, fresh = false, ghost = null }) {
  if (still()) { show(); return; }
  const start = fresh ? (shown(from) ? from : null) : host;
  if (!start) { show(); return; }
  const before = box(start), r0 = round(start, before);
  const g = ghost ? ghost() : null;
  const free = hold(host);
  show();
  const after = box(host), r1 = round(host, after);
  const shape = host.animate([{ clipPath: cut(before, after, r0) }, { clipPath: whole(r1) }], { duration: 440, easing: SHEET });
  ended(shape, 440).then(free);
  if (g) ended(g.animate([{ opacity: 1 }, { opacity: 0 }], { duration: 200, easing: 'ease-out', fill: 'forwards' }), 200).then(() => g.remove());
  for (const c of bar.children) c.animate([{ opacity: 0 }, { opacity: 1 }], { duration: 220, delay: 90, easing: 'ease-out', fill: 'backwards' });
  for (const n of rest(bar)) n.animate([{ opacity: 0, transform: 'translateY(6px)' }, { opacity: 1, transform: 'none' }], { duration: 300, delay: 120, easing: SHEET, fill: 'backwards' });
}

/**
 * Close it back into what it grew out of — the menu (`hide` brings it back; `ghost()` is its snapshot, laid over and faded
 * in as the shape arrives) or, `gone`, the control `to` — and take it away.
 */
export function shrink({ to, host, bar, list, show, hide, gone = false, ghost = null }) {
  if (still() || !shown(host)) { hide(); return Promise.resolve(); }
  const now = box(host), r1 = round(host, now);
  hide();
  const end = gone ? (shown(to) ? to : null) : host;
  const then = end ? box(end) : null, r0 = end ? round(end, then) : 0;
  const g = ghost ? ghost() : null;
  show();
  if (!then) { if (g) g.remove(); hide(); return Promise.resolve(); }
  const free = hold(host);
  for (const n of rest(bar)) n.animate([{ opacity: 1 }, { opacity: 0 }], { duration: 120, easing: 'ease-in', fill: 'forwards' });
  for (const c of bar.children) c.animate([{ opacity: 1 }, { opacity: 0 }], { duration: 110, easing: 'ease-in', fill: 'forwards' });
  if (g) g.animate([{ opacity: 0 }, { opacity: 0, offset: 0.35 }, { opacity: 1 }], { duration: 340, easing: 'ease-out', fill: 'both' });
  const shape = host.animate([{ clipPath: whole(r1) }, { clipPath: cut(then, now, r0) }], { duration: 340, easing: SHEET, fill: 'forwards' });
  return ended(shape, 340).then(() => {
    hide();
    if (g) g.remove();
    for (const n of [host, bar, ...rest(bar), ...bar.children]) if (n) for (const a of n.getAnimations()) a.cancel();
    free();
  });
}

const DEMO = [
  { title: 'Pages', rows: [{ label: 'Dashboard', icon: 'home' }, { label: 'Activity', icon: 'activity' }, { label: 'Libraries', icon: 'library' }] },
  { title: 'Library', rows: [{ label: 'Big Buck Bunny', sub: 'Film · 2008', icon: 'film' }, { label: 'Sintel', sub: 'Film · 2010', icon: 'film' }, { label: 'Low Orbit', sub: 'Series', icon: 'tv' }] },
  { title: 'People', rows: [{ label: 'alice', sub: 'On this server', icon: 'user' }, { label: 'Mira Vance', sub: 'Actor · 4 titles', icon: 'user' }] },
];
export const meta = {
  name: 'search',
  purpose: 'Search that lives where it was opened — in a menu or in the page — never as a dialog over everything.',
  use: 'One at a time. The app runs the words (pages at once, the rest when its answer comes) and decides where it lives; grow() plays it out of the control that opened it and shrink() back into it.',
  avoid: 'A dialog with a backdrop for it. Running the search in the component: it is given what it shows. Fetching per keystroke without the app’s own debounce.',
  variants: ['in a menu (desktop-nav, mobile-nav)', 'in the page'],
  states: ['is-active: the row Enter opens (aria-activedescendant)', 'nothing found, or a note while searching', 'a row that keeps a control: Ctrl+Enter presses it'],
  a11y: 'A combobox over a listbox: the arrows move aria-activedescendant through the rows, Enter opens, Esc gives up and the app returns the focus to what had it when search opened: the control pressed, or the place a shortcut was pressed in. Rows say aria-selected. Still with reduced motion.',
  props: {
    'searchPanel({ run, onPick, onEscape, placeholder, label, keys })': 'run(words, paint): paint(groups, note) as often as there is something',
    'grow({ from, host, bar, list, show })': 'lay it out and play it out of `from`',
    'shrink({ to, host, bar, list, show, hide }) → Promise': 'play it back into `to` and take it away',
  },
  playground: {
    controls: [{ key: 'hints', label: 'Keys at the foot', on: true }],
    render: (o) => {
      const p = searchPanel({ placeholder: 'Titles, people and pages', keys: o.hints, onPick: () => {},
        run: (q, paint) => paint(DEMO.map((g) => ({ title: g.title, rows: g.rows.filter((r) => r.label.toLowerCase().includes(q.toLowerCase()))
          .map((r) => ({ ...r, thumb: h('span', { class: 'fui-search__icon' }, icon(r.icon, 15)) })) })), `Nothing holds “${q}”.`) });
      return h('div', { class: 'fui-search__demo' }, p.el);
    },
  },
};
