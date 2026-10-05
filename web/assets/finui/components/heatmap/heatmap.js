// FinUI: heatmap. How busy each moment was, as a grid of cells from quiet to busy in steps of one colour — the hours of
// each day of the week, the days of a year — with its rows and columns named and a key from less to more.

import { h, s } from '../../core.js';
import { level, compact } from '../chart/plan.js';
import { chartFigure, tableTwin, sized, hoverable } from '../chart/chart.js';

/** heatmap({ title, sub, rows, cols, values, format, every }): `values[row][col]`; `every` names every nth column. */
export function heatmap({ title = null, sub = null, rows, cols, values, format = compact, every = null }) {
  const max = Math.max(0, ...values.flat());
  const box = h('div', { class: 'fui-heatmap__box' });
  const key = h('div', { class: 'fui-heatmap__key', 'aria-hidden': 'true' }, h('span', null, 'Less'), [0, 1, 2, 3, 4, 5, 6].map((l) => { const c = h('span', { class: 'fui-heatmap__swatch' }); c.style.background = `var(--heat-${l})`; return c; }), h('span', null, 'More'));
  const figure = chartFigure({ title, sub, plot: h('div', { class: 'fui-heatmap__wrap' }, box, key),
    table: tableTwin({ caption: title || 'How busy', head: ['', ...cols], rows: rows.map((r, i) => [r, ...values[i].map((v) => format(v))]) }) });
  let spec = null;
  hoverable(box, () => spec, `${title || 'A heatmap'}: the busiest ${format(max)}`);
  sized(box, (width) => {
    const left = 8 + 7 * Math.max(...rows.map((r) => r.length)), top = 16;
    const cell = Math.max(6, Math.min(26, Math.floor((width - left) / cols.length)));
    const step = every || Math.max(1, Math.ceil(cols.length / Math.max(1, Math.floor((width - left) / 34))));
    const w = left + cell * cols.length, hgt = top + cell * rows.length;
    const cells = rows.map((_, ri) => cols.map((__, ci) => {
      const r = s('rect', { class: 'fui-heatmap__cell', x: String(left + ci * cell + 1), y: String(top + ri * cell + 1), width: String(cell - 2), height: String(cell - 2), rx: '2' });
      r.style.fill = `var(--heat-${level(values[ri][ci], max)})`;
      return r;
    }));
    const svg = s('svg', { class: 'fui-chart__plot fui-heatmap__svg', width: String(w), height: String(hgt), viewBox: `0 0 ${w} ${hgt}`, 'aria-hidden': 'true' },
      rows.map((r, i) => s('text', { class: 'fui-chart__tick', x: String(left - 6), y: String(top + i * cell + cell / 2 + 3.5), 'text-anchor': 'end' }, r)),
      cols.map((c, i) => (i % step ? null : s('text', { class: 'fui-chart__tick', x: String(left + i * cell + cell / 2), y: '11', 'text-anchor': 'middle' }, c))),
      cells);
    const n = rows.length * cols.length;
    spec = {
      count: n,
      indexAt: (e) => { const b = svg.getBoundingClientRect(); const ci = Math.floor((e.clientX - b.left - left) / cell), ri = Math.floor((e.clientY - b.top - top) / cell); return ci >= 0 && ri >= 0 && ci < cols.length && ri < rows.length ? ri * cols.length + ci : -1; },
      mark: (k) => cells.flat().forEach((c, i) => c.classList.toggle('is-on', i === k)),
      at: (k) => { const ri = Math.floor(k / cols.length), ci = k % cols.length, b = cells[ri][ci].getBoundingClientRect();
        return { rect: b, title: `${rows[ri]} ${cols[ci]}`, rows: [{ color: `var(--heat-${level(values[ri][ci], max)})`, value: format(values[ri][ci]), label: '' }] }; },
    };
    box.replaceChildren(svg);
  });
  return figure;
}

const DAYS = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'];
const HOURS = Array.from({ length: 24 }, (_, i) => String(i).padStart(2, '0'));
export const meta = {
  name: 'heatmap',
  purpose: 'Shows how busy each moment was as a grid of cells, from quiet to busy in steps of one colour.',
  use: 'When people watch: the hours of each weekday; a year of days. Each step is one of the --heat tokens, so a preset reaches them; the key says less and more.',
  avoid: 'Exact amounts to compare (bar-chart). A diverging measure, above and below a middle (that needs two hues).',
  variants: ['hours of the week', 'with every column named', 'sparse'],
  states: ['a cell pointed at or arrowed to (ringed)'],
  a11y: 'A figure named by its title; the grid is one tab stop whose arrows move a tooltip from cell to cell; the table holds every value.',
  props: { 'heatmap({ title, sub, rows, cols, values, format, every })': 'values[row][col]' },
  playground: {
    controls: [
      { key: 'busy', label: 'A busy week', on: true },
      { key: 'every', label: 'Every hour named' },
    ],
    render: (o) => {
      const values = DAYS.map((_, d) => HOURS.map((__, hh) => { const evening = hh >= 18 && hh <= 23 ? 6 : hh >= 12 ? 2 : hh < 2 ? 3 : 0; return o.busy ? Math.max(0, evening + ((d * 5 + hh * 3) % 4) - (d < 5 && hh < 17 ? 2 : 0)) : (hh === 21 && d % 2 ? 3 : 0); }));
      return h('div', { class: 'fui-heatmap__demo' }, heatmap({ title: 'When people watch', sub: 'Plays by hour and weekday', rows: DAYS, cols: HOURS, values, every: o.every ? 1 : null }));
    },
  },
};
