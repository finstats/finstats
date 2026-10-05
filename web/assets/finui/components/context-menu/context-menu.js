// FinUI: context-menu. What can be done to the thing under the pointer, in a menu that grows out of the very point
// that was pressed and folds back into it. A right-click, a long-press on a touch screen, or the keyboard's menu key
// (Shift+F10) opens it; Shift with a right-click is left to the browser, whose own menu is still a click away. One menu
// at a time, on the page's top layer of its own (appended to <body>, position: fixed).
//
// The rules that are not drawing — where it opens, the keys, type-ahead, what a long-press is — are plan.js.

import { h, icon } from '../../core.js';
import { place, step, edge, typeahead, held } from './plan.js';

let current = null;   // the menu on screen: { el, rows, items, from, close }
const reduced = () => typeof matchMedia === 'function' && matchMedia('(prefers-reduced-motion: reduce)').matches;
// Out of the point it is quick and lands softly (the search's sheet curve); back into it quicker still.
const GROW = { duration: 190, easing: 'cubic-bezier(.32,.72,0,1)' };
const FOLD = { duration: 120, easing: 'cubic-bezier(.4,0,1,1)' };

/** Is a menu on screen? */
export const menuOpen = () => !!current;

/** Close the menu on screen. `restore`: hand the focus back to what it was opened from (Esc, Tab, a choice). */
export function closeMenu({ restore = false, instant = false } = {}) {
  if (current) current.close({ restore, instant });
}

function row(it, i) {
  if (it.separator) return h('div', { class: 'fui-context-menu__separator', role: 'separator' });
  const inner = [it.icon ? icon(it.icon, 15) : h('span', { class: 'fui-context-menu__no-icon', 'aria-hidden': 'true' }), h('span', { class: 'fui-context-menu__label' }, it.label),
    it.hint ? h('span', { class: 'fui-context-menu__hint' }, it.hint) : null];
  const cls = ['fui-context-menu__item', it.danger && 'fui-context-menu__item--danger', it.later && 'is-waiting'];
  const common = { class: cls, role: 'menuitem', tabindex: '-1', dataset: { i: String(i) }, 'aria-disabled': it.disabled ? 'true' : null };
  // A link is a real link: a middle-click, a drag and the status bar's address all work on it as on any other.
  return it.href && !it.disabled
    ? h('a', { ...common, href: it.href, target: it.newTab ? '_blank' : null, rel: it.newTab ? 'noopener noreferrer' : null }, inner)
    : h('button', { ...common, type: 'button' }, inner);
}

/**
 * Open a menu. `items`: [{ label, icon, onSelect(), href, newTab, disabled, danger, hint }, { separator: true }, …]; an
 * item may instead be `{ label, later: Promise<item|null> }`, drawn waiting until it knows what it is (null takes it
 * away). `at`: the point it opens at, in the window's coordinates. `from`: the element it was opened from, which gets
 * the focus back. `label`: what a screen reader calls the menu. `onClose()`: it has gone, however it went.
 */
export function openMenu({ items, at, from = null, label = 'Actions', onClose = null }) {
  closeMenu({ instant: true });
  // An item a caller left out can leave two separators together, or one at an end: a line divides two things or none.
  const list = items.filter(Boolean).filter((it, i, all) => !(it.separator && (i === 0 || i === all.length - 1 || all[i - 1].separator)));
  const el = h('div', { class: 'fui-context-menu', role: 'menu', 'aria-label': label });
  let rows = list.map(row);
  el.append(...rows);
  el.style.visibility = 'hidden';
  document.body.append(el);
  const view = { w: document.documentElement.clientWidth, h: document.documentElement.clientHeight };
  const spot = place(at, { w: el.offsetWidth, h: el.offsetHeight }, view);
  Object.assign(el.style, { left: `${spot.left}px`, top: `${spot.top}px`, transformOrigin: spot.origin, visibility: '' });

  let index = -1, typed = '', typedAt = 0, closed = false;
  const opened = performance.now();
  const focus = (i) => { if (i < 0) return; index = i; rows[i].focus({ preventScroll: true }); };
  const choose = (i, e) => {
    const it = list[i];
    if (!it || it.separator || it.disabled || it.later) { e?.preventDefault(); return; }
    // A link follows itself; the menu goes once the click has been seen by whoever routes it.
    if (!it.href) e?.preventDefault();
    close({ restore: !it.href || it.newTab });
    if (it.onSelect) it.onSelect();
  };

  const outside = (e) => { if (!el.contains(e.target)) close({ restore: false }); };
  const gone = () => close({ restore: false, instant: true });
  // A scroll carries the menu with the thing it belongs to — a smooth scroll still under way when it opened, a row
  // scrolled by its arrows, the page by a wheel — and only once that thing has left the screen does the menu go.
  // Closing at any scroll shut a menu opened from the keyboard in the same frame, as the focus scrolled it into view.
  const anchor = from && from.isConnected ? from.getBoundingClientRect() : null;
  let follow = 0;
  const scrolled = (e) => {
    if (el.contains(e.target)) return;
    if (!anchor) { gone(); return; }
    if (follow) return;
    follow = requestAnimationFrame(() => {
      follow = 0;
      if (closed) return;
      const r = from.getBoundingClientRect();
      if (!from.isConnected || r.bottom < 0 || r.top > innerHeight || r.right < 0 || r.left > innerWidth) { gone(); return; }
      el.style.translate = `${Math.round(r.left - anchor.left)}px ${Math.round(r.top - anchor.top)}px`;
    });
  };
  const close = ({ restore = false, instant = false } = {}) => {
    if (closed) return;
    closed = true;
    if (onClose) onClose();
    if (current && current.el === el) current = null;
    document.removeEventListener('pointerdown', outside, true);
    window.removeEventListener('blur', gone);
    window.removeEventListener('resize', gone);
    window.removeEventListener('scroll', scrolled, true);
    cancelAnimationFrame(follow);
    if (restore && from && from.isConnected) from.focus({ preventScroll: true });
    if (instant || reduced()) { el.remove(); return; }
    el.style.pointerEvents = 'none';
    const a = el.animate([{ transform: 'none', opacity: 1 }, { transform: 'scale(.6)', opacity: 0 }], FOLD);
    const done = () => el.remove();
    a.finished.then(done, done);
    setTimeout(done, FOLD.duration + 100);   // an animation that never runs (a hidden tab) still leaves
  };

  el.addEventListener('keydown', (e) => {
    const k = e.key;
    if (k === 'Escape') { e.preventDefault(); e.stopPropagation(); close({ restore: true }); return; }
    if (k === 'Tab') { e.preventDefault(); close({ restore: true }); return; }
    if (k === 'ArrowDown' || k === 'ArrowUp') { e.preventDefault(); focus(step(list, index, k === 'ArrowDown' ? 1 : -1)); return; }
    if (k === 'Home' || k === 'End') { e.preventDefault(); focus(edge(list, k === 'End' ? 'last' : 'first')); return; }
    if (k === 'Enter' || k === ' ') {
      // A button answers Enter with a click of its own; a link does not answer the space bar.
      if (k === ' ') { e.preventDefault(); if (index >= 0) rows[index].click(); }
      return;
    }
    if (k.length === 1 && !e.ctrlKey && !e.metaKey && !e.altKey) {
      e.preventDefault();
      const now = Date.now();
      typed = now - typedAt < 600 ? typed + k : k; typedAt = now;
      focus(typeahead(list, typed.length > 1 ? index - 1 : index, typed));
    }
  });
  el.addEventListener('click', (e) => { const r = e.target.closest('[role="menuitem"]'); if (r) choose(Number(r.dataset.i), e); });
  // Under the pointer is where the keys go on from, as in any menu.
  el.addEventListener('pointermove', (e) => {
    const r = e.target.closest('[role="menuitem"]');
    if (r && r !== document.activeElement && r.getAttribute('aria-disabled') !== 'true') { index = Number(r.dataset.i); r.focus({ preventScroll: true }); }
  });
  el.addEventListener('contextmenu', (e) => e.preventDefault());

  document.addEventListener('pointerdown', outside, true);
  window.addEventListener('blur', gone);
  window.addEventListener('resize', gone);
  window.addEventListener('scroll', scrolled, true);

  // An item that has yet to learn what it is takes its place when it knows, or leaves it.
  list.forEach((it, i) => {
    if (!it.later) return;
    it.later.then((ready) => {
      if (closed) return;
      if (ready) {
        list[i] = ready; const fresh = row(ready, i); rows[i].replaceWith(fresh); rows[i] = fresh;
        if (index === i) fresh.focus({ preventScroll: true });
        // Known while the rows are still coming in: it comes in with them, not ahead of them.
        const left = 25 + i * 18 + 160 - (performance.now() - opened);
        if (left > 0 && !reduced()) fresh.animate([{ opacity: 0, transform: 'translateY(-3px)' }, { opacity: 1, transform: 'none' }], { duration: Math.min(160, left), delay: Math.max(0, left - 160), easing: 'ease-out', fill: 'backwards' });
      }
      else { list[i] = { separator: true, hidden: true }; rows[i].hidden = true; }
    }, () => { if (!closed) { list[i] = { ...it, later: null, disabled: true }; rows[i].setAttribute('aria-disabled', 'true'); rows[i].classList.remove('is-waiting'); } });
  });

  current = { el, from, close };
  focus(edge(list, 'first'));
  if (!reduced()) {
    el.animate([{ transform: 'scale(.6)', opacity: 0 }, { transform: 'none', opacity: 1 }], GROW);
    rows.forEach((r, i) => r.animate([{ opacity: 0, transform: 'translateY(-3px)' }, { opacity: 1, transform: 'none' }], { duration: 160, delay: 25 + i * 18, easing: 'ease-out', fill: 'backwards' }));
  }
  return { el, close };
}

/**
 * Give everything under `root` a context menu. `resolve(target)` answers what the element pressed offers —
 * `{ items, from, label }` — or null for the browser's own menu (text in a field, a picture to save). Answers a function
 * that takes it all away again.
 */
export function attachContextMenu(root, resolve) {
  let press = null, swallow = 0, keyed = 0;
  const at = (el) => { const r = el.getBoundingClientRect(); return { x: Math.round(r.left + Math.min(16, r.width / 2)), y: Math.round(r.bottom - Math.min(6, r.height / 2)) }; };
  const show = (target, point) => {
    const got = resolve(target);
    if (!got) return false;
    openMenu({ ...got, at: point || at(got.from || target) });
    return true;
  };
  const onContext = (e) => {
    if (e.shiftKey) return;
    // A long-press or a key has opened it already: Android sends this too after a long-press, and a browser may after
    // the menu key, at about the same moment.
    if (current && Date.now() - Math.max(press?.openedAt || 0, keyed) < 800) { e.preventDefault(); return; }
    // From the keyboard there is no pointer: it opens at the element that has the focus.
    const keyboard = e.button !== 2 && e.clientX === 0 && e.clientY === 0;
    if (show(e.target, keyboard ? null : { x: e.clientX, y: e.clientY })) e.preventDefault();
  };
  const onKey = (e) => {
    if (!(e.key === 'ContextMenu' || (e.key === 'F10' && e.shiftKey)) || e.ctrlKey || e.metaKey || e.altKey) return;
    if (show(e.target, null)) { e.preventDefault(); keyed = Date.now(); }
  };
  const onDown = (e) => {
    if (e.pointerType !== 'touch' || !e.isPrimary) return;
    clearTimeout(press?.timer);
    const start = { x: e.clientX, y: e.clientY, t: performance.now() }, target = e.target;
    press = { start, timer: setTimeout(() => {
      if (!press || press.start !== start) return;
      const { x, y } = press.last || start;
      if (!held({ ms: performance.now() - start.t, dx: x - start.x, dy: y - start.y })) return;
      if (show(target, { x: start.x, y: start.y })) { press.openedAt = Date.now(); swallow = Date.now(); }
    }, 500) };
  };
  const onMove = (e) => {
    if (!press || e.pointerType !== 'touch') return;
    press.last = { x: e.clientX, y: e.clientY };
    if (!held({ ms: Infinity, dx: e.clientX - press.start.x, dy: e.clientY - press.start.y })) { clearTimeout(press.timer); press = null; }
  };
  const onUp = () => { if (press) clearTimeout(press.timer); };
  // Lifting the finger after a long-press is not a tap on what was held.
  const onClick = (e) => { if (Date.now() - swallow < 900 && !(e.target.closest && e.target.closest('.fui-context-menu'))) { e.preventDefault(); e.stopPropagation(); swallow = 0; } };
  root.addEventListener('contextmenu', onContext);
  root.addEventListener('keydown', onKey);
  root.addEventListener('pointerdown', onDown, { passive: true });
  root.addEventListener('pointermove', onMove, { passive: true });
  root.addEventListener('pointerup', onUp, { passive: true });
  root.addEventListener('pointercancel', onUp, { passive: true });
  root.addEventListener('click', onClick, true);
  return () => {
    root.removeEventListener('contextmenu', onContext); root.removeEventListener('keydown', onKey);
    root.removeEventListener('pointerdown', onDown); root.removeEventListener('pointermove', onMove);
    root.removeEventListener('pointerup', onUp); root.removeEventListener('pointercancel', onUp);
    root.removeEventListener('click', onClick, true);
  };
}

export const meta = {
  name: 'context-menu',
  purpose: 'What can be done to the thing under the pointer, in a menu that grows out of the point that was pressed.',
  use: 'attachContextMenu(root, resolve) once, for a page or a whole app: resolve(target) answers { items, from, label } for what it knows and null for the rest. openMenu({ items, at, from }) opens one by hand.',
  avoid: 'The only way to an action: everything in a context menu is also somewhere a click or a tap finds without knowing to right-click. And never in a field, where the browser’s own menu is the one wanted.',
  variants: ['links and actions', 'with icons', 'a separator', 'a dangerous action', 'an item that waits to learn what it is'],
  states: ['opening (grows out of the point)', 'an item focused', 'closing (folds back into it)'],
  a11y: 'role="menu" with role="menuitem" rows; the first takes the focus; arrows, Home, End and type-ahead move; Enter or Space chooses; Esc and Tab close and hand the focus back. The keyboard’s menu key and Shift+F10 open it on the focused element.',
  props: {
    'attachContextMenu(root, resolve)': 'resolve(target) → { items, from, label } or null; answers a function that detaches it',
    'openMenu({ items, at, from, label, onClose })': 'items: [{ label, icon, onSelect, href, newTab, disabled, danger, hint, later }, { separator: true }]; at: { x, y }',
    'closeMenu({ restore })': '', 'menuOpen()': 'is one on screen',
  },
  playground: {
    controls: [
      { key: 'icons', label: 'Icons', on: true },
      { key: 'danger', label: 'A dangerous action' },
    ],
    render: (o) => {
      const items = () => [
        { label: 'Open', icon: o.icons ? 'external' : null },
        { label: 'Add to watchlist', icon: o.icons ? 'bookmark' : null },
        { label: 'Share', icon: o.icons ? 'share' : null },
        { separator: true },
        { label: 'Copy link', icon: o.icons ? 'copy' : null },
        o.danger ? { label: 'Remove', icon: o.icons ? 'trash' : null, danger: true } : null,
      ].filter(Boolean);
      const area = h('div', { class: 'fui-context-menu__demo', tabindex: '0' }, 'Right-click here, hold a finger on it, or focus it and press Shift+F10');
      attachContextMenu(area, () => ({ items: items(), from: area, label: 'Example' }));
      // Held open to look at, beside the real one on the area.
      const held = h('div', { class: 'fui-context-menu fui-context-menu--held', role: 'menu', 'aria-label': 'A context menu, held open' }, items().map(row));
      return h('div', { class: 'fui-context-menu__demo-row' }, held, area);
    },
  },
};
