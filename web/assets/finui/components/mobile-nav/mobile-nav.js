// FinUI: mobile-nav. The menu of a phone, in five styles the person chooses between: a tab bar with More, a sheet that
// peeks half way, the whole screen, an arc around the thumb, and an address bar that grows into the menu. Every style
// closes on Esc, on the scrim and on a page chosen, and gives the focus back to what opened it. Search lives in it: the
// menu's search asks the app (onSearch), which hands it a FinUI search to hold (openSearch), and it grows into the place
// the style has for it. Which style, which page is open and what sits at its foot are the app's.

import { h, icon } from '../../core.js';
import { STYLES, styleOf, split, arc } from './plan.js';
import { grow, shrink } from '../search/search.js';

export { STYLES, styleOf } from './plan.js';

const FOCUSABLE = 'a[href], button:not([disabled]), input, [tabindex]:not([tabindex="-1"])';
const dotOn = (p) => (typeof p.dot === 'function' ? !!p.dot() : !!p.dot);

/**
 * The menu. `pages`: [{ key, href, label, icon, primary, dot }] — `primary` asks for a place in a bar, `dot` (a boolean or
 * a function read on repaint) marks something new. `foot` is the app's own node (who is signed in, a theme switch).
 * `onSearch(from)`: the menu's search was pressed (from that button); without it there is none. `onSearchEnd()`: the menu
 * let its search go by itself (the scrim, a handle). `contained` keeps it inside its positioned parent (a gallery's frame).
 * Answers { el, trigger, open, close, setCurrent, repaint, destroy, openSearch(panel, from), closeSearch() }: `trigger`
 * is the button an app bar should carry (peek, full); the other styles bring their own and it is null.
 */
export function mobileNav({ style, pages, current = null, foot = null, onSearch = null, onSearchEnd = () => {}, label = 'Menu', contained = false }) {
  const kind = styleOf(style);
  const el = h('div', { class: ['fui-mobile-nav', `fui-mobile-nav--${kind}`, contained && 'fui-mobile-nav--contained'] });
  const links = [];        // every link drawn, with the page it opens
  const triggers = [];
  let isOpen = false, here = current, restoreTo = null;
  const hooks = { open: [], close: [], current: [] };

  // ---- the pieces every style is made of
  function link(p, cls = 'fui-mobile-nav__link', { size = 18 } = {}) {
    const dot = p.dot ? h('span', { class: 'fui-mobile-nav__dot' }, h('span', { class: 'sr-only' }, 'New')) : null;
    const a = h('a', { class: ['fui-mobile-nav__link', cls !== 'fui-mobile-nav__link' && cls], href: p.href }, icon(p.icon, size), h('span', { class: 'fui-mobile-nav__label' }, p.label), dot);
    a.addEventListener('click', () => close());
    links.push({ a, p, dot });
    return a;
  }
  /** The menu's search: drawn as a field, and a button — the search itself is the app's, and opens where this was. */
  function search({ placeholder = 'Search' } = {}) {
    if (!onSearch) return null;
    const b = h('button', { type: 'button', class: 'fui-mobile-nav__search' }, icon('search', 18), h('span', null, placeholder));
    b.addEventListener('click', () => onSearch(b));
    return b;
  }
  /** The handle of a sheet: a tap or a drag down closes it, a drag up calls `up`; the sheet follows the finger. */
  function grab(sheet, { up = null, down = () => close(true), tap = () => close(true) } = {}) {
    const b = h('button', { type: 'button', class: 'fui-mobile-nav__grab', 'aria-label': 'Close menu' }, h('span', { class: 'fui-mobile-nav__grab-bar' }));
    let y0 = null, dy = 0;
    b.addEventListener('pointerdown', (e) => { y0 = e.clientY; dy = 0; b.setPointerCapture(e.pointerId); sheet.classList.add('is-dragging'); });
    b.addEventListener('pointermove', (e) => { if (y0 == null) return; dy = e.clientY - y0; sheet.style.translate = `0 ${dy < 0 ? (up ? dy : dy / 4) : dy}px`; });
    const end = () => {
      if (y0 == null) return;
      y0 = null; sheet.classList.remove('is-dragging'); sheet.style.translate = '';
      if (dy > 70) down(); else if (dy < -50 && up) up(); else if (Math.abs(dy) < 5) tap();
    };
    b.addEventListener('pointerup', end);
    b.addEventListener('pointercancel', end);
    // A click that no pointer made (the keyboard): the pointer's own ends above, so this only answers Enter and Space.
    b.addEventListener('click', (e) => { if (e.detail === 0) tap(); });
    return b;
  }
  const footer = (cls = null) => (foot ? h('div', { class: ['fui-mobile-nav__foot', cls] }, foot) : null);
  const list = (ps, cls = null, opts) => h('nav', { class: ['fui-mobile-nav__list', cls], 'aria-label': 'Pages' }, ps.map((p) => link(p, 'fui-mobile-nav__link', opts)));
  function opener(cls, children, aria = 'Open menu') {
    const b = h('button', { type: 'button', class: ['fui-mobile-nav__trigger', cls], 'aria-label': aria, 'aria-expanded': 'false', 'aria-haspopup': 'dialog' }, children);
    b.addEventListener('click', () => (isOpen ? close(true) : open()));
    triggers.push(b);
    return b;
  }

  const scrim = h('div', { class: 'fui-mobile-nav__scrim' });
  scrim.addEventListener('click', () => close(true));
  const panel = h('div', { class: 'fui-mobile-nav__panel', role: 'dialog', 'aria-modal': 'true', 'aria-label': label, tabindex: '-1' });
  let dock = null, trigger = null;
  const { bar, rest } = split(pages);

  // ---- the five
  if (kind === 'tabs') {
    const more = opener('fui-mobile-nav__tab fui-mobile-nav__more', [icon('menu', 20), h('span', { class: 'fui-mobile-nav__label' }, 'More')], 'More pages');
    hooks.current.push(() => more.classList.toggle('is-current', rest.some((p) => p.key === here)));
    dock = h('nav', { class: 'fui-mobile-nav__dock fui-mobile-nav__bar', 'aria-label': 'Main' }, bar.map((p) => link(p, 'fui-mobile-nav__tab', { size: 20 })), more);
    panel.classList.add('fui-mobile-nav__sheet');
    panel.append(grab(panel), search(), list(rest), footer());
  } else if (kind === 'peek') {
    trigger = opener('fui-mobile-nav__menu', icon('menu', 20));
    const all = h('button', { type: 'button', class: 'fui-mobile-nav__all', 'aria-expanded': 'false' }, h('span', null, 'All pages'), icon('chevronDown', 16));
    const widen = (full) => { el.classList.toggle('is-full', full); all.setAttribute('aria-expanded', String(full)); };
    all.addEventListener('click', () => widen(!el.classList.contains('is-full')));
    hooks.close.push(() => widen(false));
    panel.classList.add('fui-mobile-nav__sheet');
    panel.append(grab(panel, { up: () => widen(true), down: () => (el.classList.contains('is-full') ? widen(false) : close(true)), tap: () => widen(!el.classList.contains('is-full')) }),
      search(),
      h('nav', { class: 'fui-mobile-nav__bar fui-mobile-nav__quick', 'aria-label': 'Most used' }, bar.map((p) => link(p, 'fui-mobile-nav__q', { size: 22 }))),
      all,
      h('div', { class: 'fui-mobile-nav__rest' }, list(rest), footer()));
  } else if (kind === 'full') {
    trigger = opener('fui-mobile-nav__menu', icon('menu', 20));
    const shut = h('button', { type: 'button', class: 'fui-mobile-nav__close', 'aria-label': 'Close menu' }, icon('x', 20));
    shut.addEventListener('click', () => close(true));
    panel.classList.add('fui-mobile-nav__screen');
    panel.append(h('div', { class: 'fui-mobile-nav__head' }, shut, h('span', { class: 'fui-mobile-nav__title' }, label)), search(), list(pages, 'fui-mobile-nav__list--large', { size: 20 }), footer());
  } else if (kind === 'arc') {
    dock = opener('fui-mobile-nav__dock fui-mobile-nav__fab', [h('span', { class: 'fui-mobile-nav__fab-open' }, icon('menu', 22)), h('span', { class: 'fui-mobile-nav__fab-close' }, icon('x', 22))]);
    const spots = arc(pages.length);
    const fan = h('nav', { class: 'fui-mobile-nav__arc', 'aria-label': 'Pages' }, pages.map((p, i) => {
      const a = link(p, 'fui-mobile-nav__spot', { size: 19 });
      a.style.setProperty('--x', `${spots[i].x}px`);
      a.style.setProperty('--y', `${spots[i].y}px`);
      a.style.setProperty('--i', String(i));
      return a;
    }));
    panel.classList.add('fui-mobile-nav__fan');
    panel.append(h('div', { class: 'fui-mobile-nav__card' }, search(), footer()), fan);
  } else {
    const where = h('span', { class: 'fui-mobile-nav__label' });
    const whereIcon = h('span', { class: 'fui-mobile-nav__where' });
    dock = opener('fui-mobile-nav__dock fui-mobile-nav__pill', [whereIcon, where, icon('search', 18)]);
    hooks.current.push(() => {
      const p = pages.find((x) => x.key === here);
      where.textContent = p ? p.label : label;
      whereIcon.replaceChildren(icon(p ? p.icon : 'menu', 18));
    });
    panel.classList.add('fui-mobile-nav__sheet');
    panel.append(grab(panel), footer('fui-mobile-nav__foot--top'), list(pages, 'fui-mobile-nav__list--pairs', { size: 16 }), search({ placeholder: 'Search or go to…' }));
  }

  // Where search lives: the style's own place — its sheet, its screen, the arc's card.
  const found = h('div', { class: 'fui-mobile-nav__found' });
  const host = kind === 'arc' ? panel.querySelector('.fui-mobile-nav__card') : panel;
  host.append(found);
  el.append(...[scrim, panel, dock].filter(Boolean));

  // ---- what the menu does
  let hosted = null, searchFrom = null, freshSearch = false;
  /** The open menu as it stands, laid over itself while the search morphs out of it or back into it. */
  function ghost() {
    const wrap = el.cloneNode(false);
    wrap.classList.remove('is-searching', 'is-instant', 'is-gone');
    wrap.setAttribute('aria-hidden', 'true');
    wrap.inert = true;
    const copy = host.cloneNode(true);
    copy.querySelector('.fui-mobile-nav__found')?.remove();
    for (const n of copy.querySelectorAll('[id]')) n.removeAttribute('id');
    wrap.append(kind === 'arc' ? panel.cloneNode(false).appendChild(copy).parentNode : copy);
    el.after(wrap);
    return wrap;
  }
  /** Hold the app's search, growing out of `from` (the menu's search, or the app's own button) — opening the menu if needed. */
  function openSearch(searchPanel, from) {
    if (hosted) return;
    hosted = searchPanel; searchFrom = from;
    const fresh = !isOpen;
    freshSearch = fresh;
    grow({ from, host, bar: searchPanel.bar, list: searchPanel.list, fresh, ghost: fresh ? null : ghost, show: () => {
      if (fresh) el.classList.add('is-instant');   // no slide: it grows out of the button instead
      found.replaceChildren(searchPanel.el);
      el.classList.add('is-searching');
      open(false);
    } });
    if (fresh) requestAnimationFrame(() => requestAnimationFrame(() => el.classList.remove('is-instant')));
    searchPanel.focus();
  }
  /** Let the search go, back into what opened it, and the menu with it. */
  function closeSearch(restore = true) {
    const p = hosted;
    if (!p) return Promise.resolve();
    hosted = null;
    // Opened from outside the menu, it closes back into that button and takes the menu with it, without a glimpse of the pages.
    const gone = freshSearch;
    const hide = () => { el.classList.remove('is-searching'); if (gone) el.classList.add('is-instant', 'is-gone'); };
    const show = () => { el.classList.add('is-searching'); el.classList.remove('is-gone'); };
    // Closing into the button, the menu goes too, without its own transitions (is-instant): the dimming fades with the
    // shrink rather than holding at full strength until it ends and then dropping in one frame.
    const still = matchMedia('(prefers-reduced-motion: reduce)').matches;
    const dimming = gone && !still ? scrim.animate([{ opacity: getComputedStyle(scrim).opacity }, { opacity: 0 }], { duration: 300, easing: 'ease-out', fill: 'forwards' }) : null;
    return shrink({ to: searchFrom, host, bar: p.bar, list: p.list, show, hide, gone, ghost: gone ? null : ghost }).then(() => {
      found.replaceChildren();
      close(false);
      if (dimming) dimming.cancel();
      el.classList.remove('is-gone');
      requestAnimationFrame(() => requestAnimationFrame(() => el.classList.remove('is-instant')));
      if (restore && searchFrom && searchFrom.isConnected) searchFrom.focus({ preventScroll: true });
    });
  }
  /** The menu closing under its search (the scrim, a handle, a page): the search goes too, and the app is told. */
  function dropSearch() {
    if (!hosted) return;
    hosted = null;
    el.classList.remove('is-searching');
    found.replaceChildren();
    onSearchEnd();
  }
  function paintTriggers() { for (const t of triggers) t.setAttribute('aria-expanded', String(isOpen)); }
  function open(focusPanel = true) {
    if (isOpen) return;
    isOpen = true;
    restoreTo = document.activeElement;
    el.classList.add('is-open');
    if (!contained) document.documentElement.classList.add('fui-mobile-nav--locked');
    paintTriggers();
    for (const fn of hooks.open) fn();
    // The panel, not the search: focusing a field raises a phone's keyboard over the menu nobody asked to type in.
    if (focusPanel) panel.focus({ preventScroll: true });
  }
  /** Close; `restore` gives the focus back to what opened it (Esc, the scrim, a handle), a page chosen does not. */
  function close(restore = false) {
    if (!isOpen) return;
    isOpen = false;
    el.classList.remove('is-open');
    document.documentElement.classList.remove('fui-mobile-nav--locked');
    dropSearch();
    for (const fn of hooks.close) fn();
    paintTriggers();
    if (restore) {
      const back = triggers.find((t) => t.isConnected && t.getClientRects().length) || (restoreTo && restoreTo.isConnected ? restoreTo : null);
      if (back) back.focus({ preventScroll: true });
    }
  }
  // Esc and Tab while open, before anything of the page's own: the page's Esc steps back a page, and must not here. A menu
  // in a frame (the gallery draws several) answers only the keys pressed inside it.
  const keys = contained ? el : document;
  const onKey = (e) => {
    if (!isOpen) return;
    if (e.key === 'Escape') { if (hosted) return; e.preventDefault(); e.stopPropagation(); close(true); return; }   // a search gives up by itself
    if (e.key !== 'Tab') return;
    const order = [...panel.querySelectorAll(FOCUSABLE), ...(!dock ? [] : dock.matches(FOCUSABLE) ? [dock] : dock.querySelectorAll(FOCUSABLE))].filter((x) => x.getClientRects().length && !x.closest('[hidden]'));
    if (!order.length) return;
    const at = order.indexOf(document.activeElement);
    if (e.shiftKey && at <= 0) { e.preventDefault(); order[order.length - 1].focus(); }
    else if (!e.shiftKey && (at === -1 || at === order.length - 1)) { e.preventDefault(); order[0].focus(); }
  };
  keys.addEventListener('keydown', onKey, true);

  function setCurrent(key) {
    here = key;
    for (const { a, p } of links) {
      const on = p.key === key;
      a.classList.toggle('is-current', on);
      if (on) a.setAttribute('aria-current', 'page'); else a.removeAttribute('aria-current');
    }
    for (const fn of hooks.current) fn();
  }
  function repaint() { for (const { p, dot } of links) if (dot) dot.hidden = !dotOn(p); }
  function destroy() { close(); keys.removeEventListener('keydown', onKey, true); el.remove(); if (trigger) trigger.remove(); }

  setCurrent(current);
  repaint();
  return { el, trigger, open, close, setCurrent, repaint, destroy, openSearch, closeSearch, get isOpen() { return isOpen; }, get searching() { return !!hosted; } };
}

const DEMO_PAGES = [
  { key: 'home', href: '#home', label: 'Dashboard', icon: 'home', primary: true }, { key: 'recap', href: '#recap', label: 'Recap', icon: 'recap' },
  { key: 'me', href: '#me', label: 'My profile', icon: 'user', primary: true }, { key: 'activity', href: '#activity', label: 'Activity', icon: 'activity', primary: true },
  { key: 'together', href: '#together', label: 'Together', icon: 'together' }, { key: 'libraries', href: '#libraries', label: 'Libraries', icon: 'library', primary: true },
  { key: 'playback', href: '#playback', label: 'Playback', icon: 'sliders' }, { key: 'settings', href: '#settings', label: 'Settings', icon: 'settings' },
  { key: 'notes', href: '#notes', label: 'Patch notes', icon: 'tag', dot: true },
];

export const meta = {
  name: 'mobile-nav',
  purpose: 'The menu of a phone, in five styles the person chooses between: Tab bar, Peek, Full screen, Thumb arc and Address bar.',
  use: 'Once, below the width where a sidebar fits. The app keeps the person’s choice, marks the page that is open (setCurrent) and puts trigger in its bar for the styles that have one.',
  avoid: 'Choosing the style for the person: every style is a whole menu, and the choice is theirs. A second menu on the same screen. Pages that are not pages (a sign-out): those belong in foot.',
  variants: STYLES.map((s) => `${s.label}: ${s.line}`),
  states: ['is-open', 'is-full: Peek drawn up to every page', 'is-current: the open page (aria-current="page")', 'a page with something new (dot)'],
  a11y: 'The open menu is a dialog (aria-modal) that keeps the Tab key inside it; Esc, the scrim and a handle close it and give the focus back to the button that opened it, which says aria-expanded. A link is a link, with aria-current on the open page. The search is labelled; Enter opens the first page it found.',
  props: {
    'mobileNav({ style, pages, current, foot, onSearch, label, contained })': "style: 'tabs' | 'peek' | 'full' | 'arc' | 'address' (anything else is the tab bar); pages: [{ key, href, label, icon, primary, dot }]",
    'STYLES': '[{ key, label, line, space }]: space is how many px of the bottom of the screen the closed menu keeps',
    '→ { el, trigger, open(), close(restore), setCurrent(key), repaint(), destroy() }': 'trigger: the button for an app bar, or null',
  },
  playground: {
    controls: [
      { key: 'style', label: 'Style', choices: STYLES.map((s) => [s.key, s.label]) },
      { key: 'open', label: 'Open', on: true },
    ],
    render: (o) => {
      const nav = mobileNav({ style: o.style, pages: DEMO_PAGES, current: 'home', contained: true, onSearch: () => {} });
      const frame = h('div', { class: 'fui-mobile-nav__demo' },
        h('div', { class: 'fui-mobile-nav__demo-bar' }, nav.trigger || h('span'), h('span', { class: 'fui-mobile-nav__demo-name' }, 'Dashboard')),
        h('div', { class: 'fui-mobile-nav__demo-page' }, h('span'), h('span'), h('span')), nav.el);
      if (o.open) requestAnimationFrame(() => nav.open());
      return frame;
    },
  },
};
