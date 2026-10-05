// FinUI: donut-chart. Parts of one whole as slices of a ring, a gap of the ground between each, the whole said in the
// middle and every part named in the legend with its value and its share — four parts at most, the rest as Other.

import { h, s } from '../../core.js';
import { slice, compact } from '../chart/plan.js';
import { chartFigure, legend, tableTwin, tone, hoverable } from '../chart/chart.js';

/** donutChart({ title, sub, parts: [{ name, value }], total, format, size }): `total` { value, label } in the middle. */
export function donutChart({ title = null, sub = null, parts, total = null, format = compact, size = 168 }) {
  const sorted = [...parts].sort((a, b) => b.value - a.value);
  const kept = sorted.length > 4 ? [...sorted.slice(0, 3), { name: 'Other', value: sorted.slice(3).reduce((n, p) => n + p.value, 0) }] : sorted;
  const sum = kept.reduce((n, p) => n + p.value, 0) || 1;
  const share = (v) => `${Math.round((v / sum) * 100)}%`;
  const c = size / 2, outer = c - 2, inner = outer - Math.max(12, size * 0.15);
  // A gap of 2 px of ground at the outer edge between slices.
  const gap = kept.length > 1 ? 2 / outer : 0;
  let a = 0;
  const arcs = kept.map((p, i) => {
    const a0 = a, a1 = a + (p.value / sum) * Math.PI * 2; a = a1;
    const d = p.value >= sum ? slice(c, c, outer, inner, 0, Math.PI * 1.999) : slice(c, c, outer, inner, a0 + gap / 2, Math.max(a0 + gap / 2 + 0.001, a1 - gap / 2));
    const path = s('path', { class: 'fui-chart__bar', d }); path.style.fill = tone(i);
    return { path, mid: (a0 + a1) / 2 };
  });
  const svg = s('svg', { class: 'fui-chart__plot fui-donut-chart__svg', width: String(size), height: String(size), viewBox: `0 0 ${size} ${size}`, 'aria-hidden': 'true' }, arcs.map((x) => x.path));
  const spec = {
    count: kept.length,
    indexAt: (e) => { const r = svg.getBoundingClientRect(); let ang = Math.atan2(e.clientX - r.left - c, -(e.clientY - r.top - c)); if (ang < 0) ang += Math.PI * 2; let acc = 0; return kept.findIndex((p) => (acc += (p.value / sum) * Math.PI * 2) >= ang); },
    mark: (i) => { svg.classList.toggle('is-pointing', i >= 0); arcs.forEach((x, k) => x.path.classList.toggle('is-on', k === i)); },
    at: (i) => { const r = svg.getBoundingClientRect(), m = arcs[i].mid; const px = r.left + c + Math.sin(m) * (outer - 6), py = r.top + c - Math.cos(m) * (outer - 6);
      return { rect: { left: px, top: py, width: 1, height: 1, bottom: py }, title: kept[i].name, rows: [{ color: tone(i), value: format(kept[i].value), label: share(kept[i].value) }] }; },
  };
  const ring = h('div', { class: 'fui-donut-chart__ring' }, svg, total ? h('div', { class: 'fui-donut-chart__middle' }, h('span', { class: 'fui-donut-chart__total' }, total.value), h('span', { class: 'fui-donut-chart__label' }, total.label)) : null);
  hoverable(ring, () => spec, `${title || 'A donut chart'}: ${kept.map((p) => `${p.name} ${share(p.value)}`).join(', ')}`);
  return chartFigure({ title, sub, cls: 'fui-donut-chart', plot: h('div', { class: 'fui-donut-chart__body' }, ring, legend(kept.map((p) => ({ name: p.name, value: `${format(p.value)} · ${share(p.value)}` })))),
    table: tableTwin({ caption: title || 'The parts', head: ['', 'Value', 'Share'], rows: kept.map((p) => [p.name, format(p.value), share(p.value)]) }) });
}

const TB = (v) => `${(v / 1024).toFixed(1)} TB`;
export const meta = {
  name: 'donut-chart',
  purpose: 'Shows the parts of one whole as slices of a ring, the whole said in its middle.',
  use: 'What a library is made of, how plays split between ways of playing, storage by kind. Four parts at most; the smallest fold into Other. The legend names each with its value and share.',
  avoid: 'Comparing amounts that are not parts of one whole (bar-chart). Many parts (bar-list). Changes over time.',
  variants: ['with the whole in the middle', 'two parts', 'more than four (Other)'],
  states: ['a slice pointed at or arrowed to (the others step back)'],
  a11y: 'A figure named by its title; the ring is one tab stop whose arrows move between slices; its name says every share, and the table repeats the numbers.',
  props: { 'donutChart({ title, sub, parts, total, format, size })': 'parts: [{ name, value }]; total: { value, label }' },
  playground: {
    controls: [
      { key: 'total', label: 'The whole in the middle', on: true },
      { key: 'many', label: 'More than four parts' },
    ],
    render: (o) => {
      const parts = [{ name: 'Films', value: 6200 }, { name: 'Shows', value: 7800 }, { name: 'Music', value: 300 }, ...(o.many ? [{ name: 'Photos', value: 420 }, { name: 'Books', value: 260 }, { name: 'Games', value: 90 }] : [{ name: 'Other', value: 160 }])];
      return donutChart({ title: 'Storage', sub: 'What each kind takes', parts, format: TB, total: o.total ? { value: TB(parts.reduce((n, p) => n + p.value, 0)), label: 'of 20 TB' } : null });
    },
  },
};
