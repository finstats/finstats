// FinUI: chart. What every FinUI chart shares: a figure with its title, a legend once there are two series (the key
// beside the words, never the words in the series' colour), the same numbers as a table for screen readers, a series'
// colour by its place (four at most: a fifth folds into Other), drawing at the width it is given, and a tooltip that a
// pointer and the arrow keys both reach.

import { h } from '../../core.js';
import { showTip, hideTip, tipRows } from '../tooltip/tooltip.js';
import { compact } from './plan.js';

export { compact };
/** A series' colour, by its place. A chart shows at most four: the rest are folded into Other before they get here. */
export const tone = (i) => `var(--series-${(i % 4) + 1})`;
/** Series folded to four: the fourth and on become Other. */
export function foldSeries(series, other = 'Other') {
  if (series.length <= 4) return series;
  const rest = series.slice(3);
  return [...series.slice(0, 3), { name: other, values: rest[0].values.map((_, i) => rest.reduce((n, s) => n + (s.values[i] || 0), 0)) }];
}

/** The key: a swatch of each series' colour beside its name in the text's own colour, and its value when given. */
export function legend(items) {
  return h('ul', { class: 'fui-chart__legend' }, items.map((it, i) => {
    const key = h('span', { class: 'fui-chart__key', 'aria-hidden': 'true' });
    key.style.background = it.colour || tone(i);
    return h('li', null, key, h('span', null, it.name), it.value != null ? h('span', { class: 'fui-chart__legend-value' }, it.value) : null);
  }));
}

/** The numbers as a table, for whoever cannot see the chart: `head` the column names, `rows` the cells. */
export const tableTwin = ({ caption, head, rows }) => h('table', { class: 'sr-only' }, h('caption', null, caption),
  h('thead', null, h('tr', null, head.map((c) => h('th', { scope: 'col' }, c)))),
  h('tbody', null, rows.map((r) => h('tr', null, r.map((c, i) => (i ? h('td', null, c) : h('th', { scope: 'row' }, c)))))));

/** A chart's figure: its title (and a line under it), the legend, the plot, the table twin. */
export function chartFigure({ title = null, sub = null, legend: key = null, plot, table = null, cls = null }) {
  return h('figure', { class: ['fui-chart', cls] },
    title || sub ? h('figcaption', { class: 'fui-chart__head' }, title ? h('span', { class: 'fui-chart__title' }, title) : null, sub ? h('span', { class: 'fui-chart__sub' }, sub) : null) : null,
    key, plot, table);
}

/** `draw(width)` into `box` now and whenever its width changes. */
export function sized(box, draw, fallback = 560) {
  let last = 0;
  const run = (w) => { const width = Math.round(w || fallback); if (width === last || width < 40) return; last = width; draw(width); };
  run(box.clientWidth || fallback);
  if (typeof ResizeObserver === 'function') new ResizeObserver((es) => run(es[0].contentRect.width)).observe(box);
}

/** Points along a plot that a pointer and the keys move between. `target` is the plot's HTML box (one tab stop, named
 *  by `label`), and `spec()` answers the drawing as it stands: { count, at(i) → { rect, title, rows, total }, mark(i)
 *  (−1: none), indexAt(event) }. Wired once, so a redraw at another width only answers a new spec. */
export function hoverable(target, spec, label) {
  let i = -1;
  const show = (k) => {
    const sp = spec(); if (!sp || !sp.count) return;
    i = Math.max(0, Math.min(sp.count - 1, k));
    sp.mark(i);
    const t = sp.at(i);
    showTip(t.rect, tipRows(t.title, t.rows, t.total || null));
  };
  const leave = () => { const sp = spec(); i = -1; if (sp) sp.mark(-1); hideTip(); };
  target.classList.add('fui-chart__target');
  target.setAttribute('tabindex', '0');
  target.setAttribute('role', 'img');
  if (label) target.setAttribute('aria-label', label);
  target.addEventListener('pointermove', (e) => { const sp = spec(); const k = sp ? sp.indexAt(e) : -1; if (k >= 0 && k !== i) show(k); });
  target.addEventListener('pointerleave', leave);
  target.addEventListener('blur', leave);
  target.addEventListener('focus', () => { const sp = spec(); show(i < 0 && sp ? sp.count - 1 : i); });
  target.addEventListener('keydown', (e) => {
    const sp = spec(); if (!sp) return;
    const k = { ArrowRight: i + 1, ArrowLeft: i - 1, Home: 0, End: sp.count - 1 }[e.key];
    if (k === undefined) return;
    e.preventDefault(); show(k);
  });
}

export const meta = {
  name: 'chart',
  purpose: 'Holds what every FinUI chart shares: its figure and title, a legend, the table twin, colours by place, drawing at its width, a tooltip for pointer and keys.',
  use: 'Building a chart FinUI does not have: chartFigure around an SVG drawn at sized()’s width, legend() for two series or more, tableTwin() with the same numbers, hoverable() for the tooltip. The chart components are built of these.',
  avoid: 'A second axis on one chart (two charts). Colour as the only way to tell series apart (the legend is always there). A fifth series (foldSeries makes it Other).',
  variants: ['a legend', 'a legend with values', 'a title and a line under it'],
  states: [],
  a11y: 'A <figure> named by its <figcaption>; the plot is one tab stop whose arrows move a tooltip along it; every number is in a table screen readers read instead of the picture.',
  props: {
    'chartFigure({ title, sub, legend, plot, table, cls })': '', 'legend(items)': 'items: [{ name, value, colour }]', 'tableTwin({ caption, head, rows })': '',
    'tone(i)': "'var(--series-n)'", 'foldSeries(series, other)': 'four at most', 'sized(box, draw, fallback)': 'draw(width) now and on resize', 'hoverable(target, spec, label)': 'spec() → { count, at, mark, indexAt }', 'compact(n)': '1,284 · 12.9K',
  },
  playground: {
    controls: [
      { key: 'values', label: 'Values in the legend', on: true },
      { key: 'sub', label: 'A line under the title', on: true },
    ],
    render: (o) => chartFigure({ title: 'What was watched', sub: o.sub ? 'The last 30 days' : null,
      legend: legend([['Films', '42h'], ['Episodes', '31h'], ['Music', '6h'], ['Other', '2h']].map(([name, value]) => ({ name, value: o.values ? value : null }))),
      plot: h('p', { class: 'muted' }, 'The plot goes here.') }),
  },
};
