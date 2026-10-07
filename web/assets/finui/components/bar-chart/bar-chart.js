// FinUI: bar-chart. Amounts per category as columns from one baseline (stacked when there are several series) with
// round steps up the side, thin columns with a rounded end at the top and a gap between stacked parts, a tooltip per
// column and the same numbers as a table.

import { h, s } from '../../core.js';
import { niceTicks, linear, stack, compact } from '../chart/plan.js';
import { chartFigure, legend, tableTwin, tone, foldSeries, sized, hoverable } from '../chart/chart.js';

const r2 = (n) => Math.round(n * 100) / 100;
/** A column part, its top corners rounded when it is the top of its stack. */
function part(x, y0, y1, w, round) {
  const top = Math.min(y0, y1), bottom = Math.max(y0, y1), hgt = bottom - top, r = round ? Math.min(4, w / 2, hgt) : 0;
  return `M${r2(x)} ${r2(bottom)}V${r2(top + r)}${r ? `Q${r2(x)} ${r2(top)} ${r2(x + r)} ${r2(top)}H${r2(x + w - r)}Q${r2(x + w)} ${r2(top)} ${r2(x + w)} ${r2(top + r)}` : `H${r2(x + w)}`}V${r2(bottom)}Z`;
}

/** barChart({ title, sub, categories, series: [{ name, values }], format, height }) */
export function barChart({ title = null, sub = null, categories, series, format = compact, height = 220 }) {
  const all = foldSeries(series);
  const stacks = stack(all.map((x) => x.values));
  const totals = categories.map((_, i) => all.reduce((n, x) => n + (x.values[i] || 0), 0));
  const box = h('div', { class: 'fui-bar-chart__box' });
  const figure = chartFigure({ title, sub, legend: all.length > 1 ? legend(all.map((x) => ({ name: x.name }))) : null, plot: box,
    table: tableTwin({ caption: title || 'The numbers', head: ['', ...all.map((x) => x.name), ...(all.length > 1 ? ['Total'] : [])], rows: categories.map((c, i) => [c, ...all.map((x) => format(x.values[i] || 0)), ...(all.length > 1 ? [format(totals[i])] : [])]) }) });
  let spec = null;
  hoverable(box, () => spec, `${title || 'A bar chart'}: ${categories.length} columns, the highest ${format(Math.max(...totals, 0))}`);
  sized(box, (width) => {
    const ticks = niceTicks(0, Math.max(...totals, 0), 4);
    const m = { top: 8, right: 4, bottom: 22, left: Math.max(28, 8 + 7 * Math.max(...ticks.map((t) => format(t).length))) };
    const pw = width - m.left - m.right, ph = height - m.top - m.bottom;
    const y = linear([0, ticks[ticks.length - 1]], [m.top + ph, m.top]);
    const band = pw / Math.max(1, categories.length), bw = Math.max(2, Math.min(24, band * 0.62));
    const every = Math.max(1, Math.ceil(categories.length / Math.max(1, Math.floor(pw / 46))));
    const columns = categories.map((c, i) => {
      const x = m.left + i * band + (band - bw) / 2;
      const parts = stacks.map((st, k) => {
        const { from, to } = st[i];
        if (to <= from) return null;
        const topmost = !stacks.slice(k + 1).some((above) => above[i].to > above[i].from);
        // A 2 px gap of the ground between stacked parts: each part above the first stops short of the one under it.
        const base = y(from) - (from > 0 ? 2 : 0);
        const p = s('path', { class: 'fui-chart__bar', d: part(x, base, y(to), bw, topmost) });
        p.style.fill = tone(k);
        return p;
      });
      return s('g', { class: 'fui-chart__bar-group' }, parts);
    });
    const svg = s('svg', { class: 'fui-chart__plot', width: String(width), height: String(height), viewBox: `0 0 ${width} ${height}`, 'aria-hidden': 'true' },
      ticks.map((t) => s('line', { class: t ? 'fui-chart__grid' : 'fui-chart__axis', x1: String(m.left), x2: String(width - m.right), y1: String(r2(y(t))), y2: String(r2(y(t))) })),
      ticks.map((t) => s('text', { class: 'fui-chart__tick', x: String(m.left - 6), y: String(r2(y(t)) + 3.5), 'text-anchor': 'end' }, format(t))),
      categories.map((c, i) => (i % every ? null : s('text', { class: 'fui-chart__tick', x: String(r2(m.left + i * band + band / 2)), y: String(height - 6), 'text-anchor': 'middle' }, c))),
      columns);
    spec = {
      count: categories.length,
      indexAt: (e) => { const r = svg.getBoundingClientRect(); return Math.floor((e.clientX - r.left - m.left) / band); },
      mark: (i) => { svg.classList.toggle('is-pointing', i >= 0); columns.forEach((g, k) => g.querySelectorAll('.fui-chart__bar').forEach((b) => b.classList.toggle('is-on', k === i))); },
      at: (i) => {
        const r = svg.getBoundingClientRect();
        return { rect: { left: r.left + m.left + i * band, top: r.top + y(totals[i]), width: band, height: 1, bottom: r.top + y(totals[i]) }, title: categories[i],
          rows: all.map((x, k) => ({ color: tone(k), value: format(x.values[i] || 0), label: x.name })).filter((_, k) => all.length > 1 || k === 0),
          total: all.length > 1 ? { value: format(totals[i]), label: 'in all' } : null };
      },
    };
    box.replaceChildren(svg);
  });
  return figure;
}

const DAYS = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'];
const hours = (v) => `${Number(v.toFixed(1))}h`;
export const meta = {
  name: 'bar-chart',
  purpose: 'Compares amounts per category as columns from one baseline, stacked when there are several series.',
  use: 'Watch time per day, plays per hour, what each library holds. Four series at most; a fifth folds into Other. format says each value (hours, plays).',
  avoid: 'Change over a long run of time (line-chart). Parts of one whole (donut-chart, or a stacked bar). Two measures of different scale on one chart: draw two.',
  variants: ['one series', 'stacked', 'many columns (the labels thin out)'],
  states: ['a column pointed at or arrowed to (the others step back)'],
  a11y: 'A figure named by its title; the plot is one tab stop whose arrows move a tooltip from column to column; every number is in a table read instead of the picture.',
  props: { 'barChart({ title, sub, categories, series, format, height })': 'series: [{ name, values }]' },
  playground: {
    controls: [
      { key: 'stacked', label: 'Several series, stacked', on: true },
      { key: 'month', label: 'A month of days' },
    ],
    render: (o) => {
      const n = o.month ? 30 : 7, cats = o.month ? Array.from({ length: n }, (_, i) => String(i + 1)) : DAYS;
      const at = (seed) => Array.from({ length: n }, (_, i) => ((i * 7 + seed * 3) % 5) + 1 + (i % 6 === 5 ? 2 : 0));
      const series = o.stacked ? [{ name: 'Films', values: at(1) }, { name: 'Episodes', values: at(2) }, { name: 'Music', values: at(3).map((v) => Math.round(v / 3 * 10) / 10) }] : [{ name: 'Watch time', values: at(1) }];
      return h('div', { class: 'fui-bar-chart__demo' }, barChart({ title: 'Watch time', sub: o.month ? 'Each day of September' : 'Each day this week', categories: cats, series, format: hours }));
    },
  },
};
