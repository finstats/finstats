// FinUI: theme-switch. Light · Device · Dark as one switch with three stops: a click on a third of it, a drag of the knob
// or the arrow keys pick one, and the knob carries the icon of what is chosen. Which theme is kept, and where, is the
// app's: it gives the value and is told the choice.

import { h, icon, mount } from '../../core.js';

// The stops, left to right: Device sits in the middle, the one that is neither.
const THEME_STOPS = ['light', 'device', 'dark'];
const THEME_NAME = { device: 'Device', light: 'Light', dark: 'Dark' };
const THEME_ICON = { device: 'monitor', light: 'sun', dark: 'moon' };

/**
 * The theme as a switch with three stops, like the two-stop ones in Settings: a click on a third of it, a drag of the
 * knob or the arrow keys pick one. The knob carries the icon of what is chosen, and a new icon moves in when it
 * changes (CSS, and not at all with reduced motion). A slider to assistive technology: "Theme: Dark".
 */
export function themeSwitch({ value = 'device', onChange = () => {} } = {}) {
  const knob = h('span', { class: 'fui-theme-switch__knob' });
  const el = h('div', { class: 'fui-theme-switch', role: 'slider', tabindex: '0', 'aria-label': 'Theme', 'aria-valuemin': '0', 'aria-valuemax': '2', 'aria-orientation': 'horizontal' },
    h('span', { class: 'fui-theme-switch__marks', 'aria-hidden': 'true' }, THEME_STOPS.map(() => h('span', { class: 'fui-theme-switch__mark' }))), knob);
  let at = Math.max(0, THEME_STOPS.indexOf(value));
  function paint(animate) {
    const t = THEME_STOPS[at];
    el.dataset.at = String(at);
    el.style.setProperty('--at', String(at));
    el.setAttribute('aria-valuenow', String(at));
    el.setAttribute('aria-valuetext', THEME_NAME[t]);
    el.title = `Theme: ${THEME_NAME[t]}`;
    if (knob.dataset.icon !== THEME_ICON[t]) {
      knob.dataset.icon = THEME_ICON[t];
      mount(knob, icon(THEME_ICON[t], 13, animate ? 'fui-theme-switch__icon is-in' : 'fui-theme-switch__icon'));
    }
  }
  function choose(next) {
    next = Math.max(0, Math.min(2, next));
    knob.style.translate = '';
    if (next === at) return;
    at = next;
    onChange(THEME_STOPS[at]);
    paint(true);
  }
  // Where on the track a pointer is, as a stop: 0, 1 or 2.
  const stopAt = (clientX) => { const r = el.getBoundingClientRect(); return Math.max(0, Math.min(2, Math.floor(((clientX - r.left) / r.width) * 3))); };
  let drag = null;
  el.addEventListener('pointerdown', (e) => {
    if (e.button !== 0) return;
    const r = el.getBoundingClientRect(), k = knob.getBoundingClientRect();
    // Where the knob sits now (the track's border and the knob's inset are a pixel each), and how far it can go.
    drag = { x: e.clientX, from: k.left - r.left - 2, max: r.width - k.width - 4, moved: false };
    el.setPointerCapture(e.pointerId);
  });
  el.addEventListener('pointermove', (e) => {
    if (!drag) return;
    const dx = e.clientX - drag.x;
    if (!drag.moved && Math.abs(dx) < 3) return;
    drag.moved = true;
    el.classList.add('is-dragging');
    // The knob follows the pointer between the two ends; it lands on the nearest stop when let go.
    knob.style.translate = `${Math.max(0, Math.min(drag.max, drag.from + dx)).toFixed(1)}px 0`;
  });
  const letGo = (e) => {
    if (!drag) return;
    const d = drag; drag = null;
    el.classList.remove('is-dragging');
    if (!d.moved) return choose(stopAt(e.clientX));
    const k = knob.getBoundingClientRect();
    choose(stopAt(k.left + k.width / 2));
    knob.style.translate = '';
  };
  el.addEventListener('pointerup', letGo);
  el.addEventListener('pointercancel', () => { drag = null; el.classList.remove('is-dragging'); knob.style.translate = ''; });
  el.addEventListener('keydown', (e) => {
    const step = { ArrowLeft: -1, ArrowDown: -1, ArrowRight: 1, ArrowUp: 1 }[e.key];
    if (step) { e.preventDefault(); choose(at + step); }
    else if (e.key === 'Home') { e.preventDefault(); choose(0); }
    else if (e.key === 'End') { e.preventDefault(); choose(2); }
  });
  paint(false);
  return el;
}

export const meta = {
  name: 'theme-switch',
  purpose: 'Chooses the theme: light, the device’s, or dark.',
  use: 'Once, where the app keeps its own controls (the sidebar). onChange stores the choice; the app applies it.',
  avoid: 'Two of them on a page. A theme switch inside a theme frame of the gallery: it changes the whole page.',
  variants: ['light', 'device', 'dark'],
  states: ['dragging', 'a new icon moves in (not with reduced motion)'],
  a11y: 'role="slider" with aria-valuetext ("Dark"); arrows, Home and End move it; it is in the tab order.',
  props: { 'themeSwitch({ value, onChange })': "value: 'light' | 'device' | 'dark'" },
  playground: {
    controls: [{ key: 'value', label: 'Set to', choices: [['device', 'Device'], ['light', 'Light'], ['dark', 'Dark']] }],
    render: (o) => h('div', { class: 'fui-theme-switch__demo' }, themeSwitch({ value: o.value })),
  },
};
