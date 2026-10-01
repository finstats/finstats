// FinUI: tooltip. One floating box for the whole page, beside whatever the pointer or the keyboard is on: a chart's
// column, a dot on a map. It moves to stay on screen, and a scroll hides it.

import { h } from '../../core.js';

let tipEl = null;
function tipNode() {
  if (!tipEl) {
    tipEl = h('div', { class: 'fui-tooltip', role: 'tooltip', hidden: true });
    document.body.append(tipEl);
    window.addEventListener('scroll', hideTip, { passive: true });
  }
  return tipEl;
}
export function showTip(rect, content) {
  const el = tipNode();
  el.replaceChildren(content);
  el.hidden = false;
  const tw = el.offsetWidth, th = el.offsetHeight;
  let x = rect.left + rect.width / 2 - tw / 2;
  let y = rect.top - th - 8;
  if (y < 8) y = rect.bottom + 8;
  x = Math.max(8, Math.min(x, window.innerWidth - tw - 8));
  el.style.transform = `translate(${Math.round(x)}px, ${Math.round(y)}px)`;
}
export function hideTip() { if (tipEl) tipEl.hidden = true; }
/** Is a tooltip on screen? Anything may have hidden it since it was shown: a scroll hides every one. */
export const tipShown = () => !!tipEl && !tipEl.hidden;

export function tipRows(title, rows, total) {
  return h('div', null,
    h('div', { class: 'fui-tooltip__title' }, title),
    total ? h('div', { class: 'fui-tooltip__row' }, h('span', { class: 'fui-tooltip__key' }), h('strong', null, total.value), h('span', null, total.label)) : null,
    rows.map((r) => h('div', { class: 'fui-tooltip__row' },
      h('span', { class: 'fui-tooltip__key', style: { background: r.color } }),
      h('strong', null, r.value), h('span', null, r.label))));
}

export const meta = {
  name: 'tooltip',
  purpose: 'Shows the numbers behind one mark of a chart, beside it, while it is pointed at or focused.',
  use: 'Charts and the map: showTip(rect, content) on pointer or focus, hideTip() on leave. tipRows(title, rows, total) for a title and coloured rows.',
  avoid: 'Anything that cannot be had another way: every chart has a table twin, and the tooltip only repeats what the table says.',
  variants: ['rows with colour keys', 'with a total'],
  states: ['shown', 'hidden (a scroll hides it)'],
  a11y: 'role="tooltip"; the same numbers are in the chart’s table for screen readers and keyboards.',
  props: { 'showTip(rect, content)': 'rect: the mark’s DOMRect', 'hideTip()': '', 'tipShown()': 'is one on screen', 'tipRows(title, rows, total)': 'rows: [{ color, value, label }]' },
  examples: [
    { name: 'Point at the button', render: () => { const b = h('button', { type: 'button', class: 'fui-tooltip__demo', onPointerenter: (e) => showTip(e.currentTarget.getBoundingClientRect(), tipRows('Tue 14 Oct', [{ color: 'var(--series-1)', value: '2h 10m', label: 'Movies' }, { color: 'var(--series-2)', value: '48m', label: 'Episodes' }], { value: '2h 58m', label: 'in all' })), onPointerleave: hideTip, onFocus: (e) => showTip(e.currentTarget.getBoundingClientRect(), tipRows('Tue 14 Oct', [{ color: 'var(--series-1)', value: '2h 10m', label: 'Movies' }])), onBlur: hideTip }, 'A column of a chart'); return b; } },
  ],
};
