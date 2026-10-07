// FinUI: picker. One option out of a short list, each shown as what it looks like: a button naming the choice with its
// swatch, and a list that opens under it. Moving through the list previews each option; Enter or a click keeps one, and
// closing without choosing puts back what was there. A lock may hold the choice where it is.

import { h, icon, mount } from '../../core.js';
import { swatch } from '../swatch/swatch.js';

let seq = 0;
/** The nearest ancestor that scrolls: the room a list has to open into, before the window's. */
function room(el) {
  for (let p = el.parentElement; p; p = p.parentElement) {
    const o = getComputedStyle(p).overflowY;
    if (o === 'auto' || o === 'scroll') return p.getBoundingClientRect();
  }
  return { top: 0, bottom: innerHeight };
}

/**
 * picker({ label, options, value, onChange, onPreview, lock, custom, dataset }).
 * `options`: [{ key, label, blurb, mark }]; `mark()` draws its swatch. `value`: the index chosen, or −1 for none of
 * them (it reads `custom`, "Custom"). `onPreview(i)`: an option is being looked at (−1 … back to what was there).
 * `lock`: { label, on, onToggle(on) }, a button beside it that holds the choice. Answers { el, update({ value, locked }) }.
 */
export function picker({ label, options, value = 0, onChange = () => {}, onPreview = () => {}, lock = null, custom = 'Custom', dataset = {} }) {
  const id = `fui-picker-${++seq}`;
  let current = value, locked = !!(lock && lock.on), active = 0, before = 0;
  const valueEl = h('span', { class: 'fui-picker__value' });
  const markSlot = h('span', { class: 'fui-picker__mark', 'aria-hidden': 'true' });
  const btn = h('button', { type: 'button', class: 'fui-picker__button', 'aria-haspopup': 'listbox', 'aria-expanded': 'false', 'aria-controls': id, dataset },
    h('span', { class: 'fui-picker__text' }, h('span', { class: 'fui-picker__label' }, label), valueEl), markSlot);
  const lockBtn = lock ? h('button', { type: 'button', class: 'fui-picker__lock', 'aria-pressed': 'false', dataset, 'aria-label': lock.label || `Lock ${label.toLowerCase()}`, title: lock.title || null }) : null;
  const list = h('ul', { class: 'fui-picker__list', role: 'listbox', id, tabindex: -1, hidden: true, 'aria-label': label });
  const el = h('div', { class: 'fui-picker' }, btn, lockBtn, list);

  function show() {
    const o = options[current];
    valueEl.textContent = o ? o.label : custom;
    mount(markSlot, o && o.mark ? o.mark() : null);
    if (!lockBtn) return;
    lockBtn.setAttribute('aria-pressed', String(locked));
    el.classList.toggle('is-locked', locked);
    mount(lockBtn, icon(locked ? 'lock' : 'unlock', 14));
  }
  function draw() {
    mount(list, options.map((o, i) => h('li', { role: 'option', id: `${id}-${i}`, dataset: o.key ? { option: o.key } : {},
      class: ['fui-picker__option', i === active && 'is-active'], 'aria-selected': String(i === current),
      onPointerdown: (e) => { e.preventDefault(); choose(i); },
      onPointermove: () => { if (active !== i) move(i); } },
      o.mark ? h('span', { class: 'fui-picker__mark', 'aria-hidden': 'true' }, o.mark()) : null,
      o.blurb ? h('span', { class: 'fui-picker__about' }, h('span', { class: 'fui-picker__name' }, o.label), h('span', { class: 'fui-picker__blurb' }, o.blurb)) : h('span', { class: 'trunc' }, o.label),
      i === current ? icon('check', 14, 'fui-picker__check') : null)));
    list.setAttribute('aria-activedescendant', `${id}-${active}`);
    list.querySelector('.is-active')?.scrollIntoView({ block: 'nearest' });
  }
  // Moving only moves the highlight: the options stay the same nodes, so what is under the pointer is what it clicks.
  function move(i) {
    active = i;
    list.querySelectorAll('[role=option]').forEach((li, k) => li.classList.toggle('is-active', k === i));
    list.setAttribute('aria-activedescendant', `${id}-${i}`);
    list.querySelector('.is-active')?.scrollIntoView({ block: 'nearest' });
    onPreview(i);
  }
  function open() {
    before = current; active = Math.max(0, before);
    list.hidden = false; btn.setAttribute('aria-expanded', 'true'); el.classList.add('is-open');
    draw();
    // Upwards when there is no room below: a list at the foot of a panel was cut off there.
    const r = room(el);
    el.classList.toggle('is-up', list.getBoundingClientRect().bottom > r.bottom && btn.getBoundingClientRect().top - r.top > list.offsetHeight);
    list.focus();
    document.addEventListener('pointerdown', outside, true);
  }
  function close(refocus) {
    if (list.hidden) return;
    list.hidden = true; btn.setAttribute('aria-expanded', 'false'); el.classList.remove('is-open');
    document.removeEventListener('pointerdown', outside, true);
    if (refocus) btn.focus();
  }
  function choose(i) { close(true); current = i; show(); onChange(i); }
  function cancel(refocus) { close(refocus); onPreview(before); }
  const outside = (e) => { if (!el.contains(e.target)) cancel(false); };
  btn.addEventListener('click', () => (list.hidden ? open() : cancel(true)));
  lockBtn?.addEventListener('click', () => { locked = !locked; show(); lock.onToggle?.(locked); });
  list.addEventListener('keydown', (e) => {
    const last = options.length - 1;
    if (e.key === 'ArrowDown') { e.preventDefault(); move(Math.min(last, active + 1)); }
    else if (e.key === 'ArrowUp') { e.preventDefault(); move(Math.max(0, active - 1)); }
    else if (e.key === 'Home') { e.preventDefault(); move(0); }
    else if (e.key === 'End') { e.preventDefault(); move(last); }
    else if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); choose(active); }
    else if (e.key === 'Escape') { e.preventDefault(); e.stopPropagation(); cancel(true); }
    else if (e.key === 'Tab') cancel(false);
  });
  show();
  draw();   // filled from the start, hidden until opened
  return { el, update({ value: v = current, locked: l = locked } = {}) { current = v; locked = l; show(); } };
}

const LOOKS = [['Washi', 'Paper by day, Obsidian by night.', ['var(--bg-2)', 'var(--accent)', 'var(--series-1)']], ['Canopy', 'Greens, rounded and soft.', ['var(--series-2)', 'var(--series-3)', 'var(--series-4)']], ['Ledger', 'Ruled lines and square corners.', ['var(--text)', 'var(--series-4)', 'var(--series-1)']]];
export const meta = {
  name: 'picker',
  purpose: 'Picks one option out of a short list, each shown as what it looks like, and lets each be seen before it is kept.',
  use: 'Choosing a look: a palette, a radius, a font, a style. onPreview shows an option while it is pointed at or arrowed to; leaving the list without choosing puts the old one back.',
  avoid: 'A long list to search (combobox), or plain words with nothing to show (segmented, or a native select).',
  variants: ['with swatches', 'with a line under each name', 'with a lock', 'Custom (none of them)'],
  states: ['open (upwards when there is no room below)', 'an option active', 'locked'],
  a11y: 'A button with aria-haspopup="listbox" opens a listbox; aria-activedescendant follows the arrows, Home and End; Enter or Space keeps one, Esc puts the old one back and returns the focus. The lock is a toggle button with aria-pressed.',
  props: {
    'picker({ label, options, value, onChange, onPreview, lock, custom, dataset })': 'options: [{ key, label, blurb, mark() }]; lock: { label, on, onToggle(on) } → { el, update({ value, locked }) }',
  },
  playground: {
    controls: [
      { key: 'marks', label: 'Swatches', on: true },
      { key: 'blurbs', label: 'A line under each name' },
      { key: 'lock', label: 'A lock' },
      { key: 'custom', label: 'None of them chosen' },
    ],
    render: (o) => h('div', { class: 'fui-picker__demo' }, picker({ label: 'Style', value: o.custom ? -1 : 0,
      options: LOOKS.map(([name, line, colours]) => ({ key: name.toLowerCase(), label: name, blurb: o.blurbs ? line : null, mark: o.marks ? () => swatch(colours, { shape: 'bars' }) : null })),
      lock: o.lock ? { label: 'Lock the style', on: false } : null }).el),
  },
};
