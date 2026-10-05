// FinUI: line-chart. A measure over time as a 2 px line — a wash of its colour under it when there is one series — with
// round steps up the side, a dot ringed in the ground at the end, and a crosshair that a pointer and the arrow keys move
// along, the tooltip saying every series at that moment.

import { h, s } from '../../core.js';
import { niceTicks, linear, linePath, compact } from '../chart/plan.js';
import { chartFigure, legend, tableTwin, tone, foldSeries, sized, hoverable } from '../chart/chart.js';

const r2 = (n) => Math.round(n * 100) / 100;
/** lineChart({ title, sub, labels, series: [{ name, values }], format, area, height }) */
export function lineChart({ title = null, sub = null, labels, series, format = compact, area = null, height = 220 }) {
  const all = foldSeries(series);
  const wash = area ?? all.length === 1;
  const box = h('div', { class: 'fui-line-chart__box' });
  const figure = chartFigure({ title, sub, legend: all.length > 1 ? legend(all.map((x) => ({ name: x.name }))) : null, plot: box,
    table: tableTwin({ caption: title || 'The numbers', head: ['', ...all.map((x) => x.name)], rows: labels.map((l, i) => [l, ...all.map((x) => format(x.values[i] || 0))]) }) });
  let spec = null;
  hoverable(box, () => spec, `${title || 'A line chart'}: ${labels[0]} to ${labels[labels.length - 1]}`);
  sized(box, (width) => {
    const top = Math.max(0, ...all.flatMap((x) => x.values));
    const ticks = niceTicks(0, top, 4);
    const m = { top: 10, right: 10, bottom: 22, left: Math.max(28, 8 + 7 * Math.max(...ticks.map((t) => format(t).length))) };
    const pw = width - m.left - m.right, ph = height - m.top - m.bottom;
    const y = linear([0, ticks[ticks.length - 1]], [m.top + ph, m.top]);
    const x = linear([0, Math.max(1, labels.length - 1)], [m.left, m.left + pw]);
    const every = Math.max(1, Math.ceil(labels.length / Math.max(1, Math.floor(pw / 56))));
    const cross = s('line', { class: 'fui-chart__cross', y1: String(m.top), y2: String(m.top + ph), visibility: 'hidden' });
    const dots = all.map((_, k) => { const d = s('circle', { class: 'fui-chart__dot', r: '4', visibility: 'hidden' }); d.style.fill = tone(k); return d; });
    const lines = all.map((ser, k) => {
      const pts = ser.values.map((v, i) => [x(i), y(v || 0)]);
      const line = s('path', { class: 'fui-chart__line', d: linePath(pts) }); line.style.stroke = tone(k);
      const under = wash ? s('path', { class: 'fui-chart__area', d: `${linePath(pts)}L${r2(x(labels.length - 1))} ${r2(y(0))}L${r2(x(0))} ${r2(y(0))}Z` }) : null;
      if (under) under.style.fill = tone(k);
      const end = s('circle', { class: 'fui-chart__dot', r: '4', cx: String(r2(pts[pts.length - 1][0])), cy: String(r2(pts[pts.length - 1][1])) }); end.style.fill = tone(k);
      return [under, line, end];
    });
    const svg = s('svg', { class: 'fui-chart__plot', width: String(width), height: String(height), viewBox: `0 0 ${width} ${height}`, 'aria-hidden': 'true' },
      ticks.map((t) => s('line', { class: t ? 'fui-chart__grid' : 'fui-chart__axis', x1: String(m.left), x2: String(width - m.right), y1: String(r2(y(t))), y2: String(r2(y(t))) })),
      ticks.map((t) => s('text', { class: 'fui-chart__tick', x: String(m.left - 6), y: String(r2(y(t)) + 3.5), 'text-anchor': 'end' }, format(t))),
      labels.map((l, i) => (i % every ? null : s('text', { class: 'fui-chart__tick', x: String(r2(x(i))), y: String(height - 6), 'text-anchor': i === 0 ? 'start' : i === labels.length - 1 ? 'end' : 'middle' }, l))),
      lines, cross, dots);
    spec = {
      count: labels.length,
      indexAt: (e) => { const r = svg.getBoundingClientRect(); return Math.round(((e.clientX - r.left - m.left) / Math.max(1, pw)) * (labels.length - 1)); },
      // Left, the crosshair and its dots are put back as they were drawn: hidden, and nowhere.
      mark: (i) => {
        const on = i >= 0, set = (el, attrs) => Object.entries(attrs).forEach(([k, v]) => (v == null ? el.removeAttribute(k) : el.setAttribute(k, v)));
        set(cross, { visibility: on ? 'visible' : 'hidden', x1: on ? String(r2(x(i))) : null, x2: on ? String(r2(x(i))) : null });
        dots.forEach((d, k) => set(d, { visibility: on ? 'visible' : 'hidden', cx: on ? String(r2(x(i))) : null, cy: on ? String(r2(y(all[k].values[i] || 0))) : null }));
      },
      at: (i) => {
        const r = svg.getBoundingClientRect(), hi = Math.max(...all.map((ser) => ser.values[i] || 0));
        return { rect: { left: r.left + x(i) - 1, top: r.top + y(hi), width: 2, height: 1, bottom: r.top + y(hi) }, title: labels[i], rows: all.map((ser, k) => ({ color: tone(k), value: format(ser.values[i] || 0), label: ser.name })) };
      },
    };
    box.replaceChildren(svg);
  });
  return figure;
}

const MONTHS = ['Nov', 'Dec', 'Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct'];
export const meta = {
  name: 'line-chart',
  purpose: 'Shows how a measure changes over time as a line, with a crosshair that says every series at a moment.',
  use: 'Plays per month, watch time per week, bandwidth over a day. One series gets a wash of its colour under it; several a legend. Four at most.',
  avoid: 'A few categories to compare (bar-chart). Two measures of different scale on one chart: draw two, or index both to 100.',
  variants: ['one series with a wash', 'several series', 'a year of months'],
  states: ['a moment pointed at or arrowed to (the crosshair and its dots)'],
  a11y: 'A figure named by its title; the plot is one tab stop whose arrows move the crosshair and its tooltip; every number is in a table read instead of the picture.',
  props: { 'lineChart({ title, sub, labels, series, format, area, height })': 'series: [{ name, values }]; area: the wash (one series by default)' },
  playground: {
    controls: [
      { key: 'several', label: 'Several series' },
      { key: 'wash', label: 'A wash under it', on: true },
    ],
    render: (o) => {
      const at = (seed, scale) => MONTHS.map((_, i) => Math.round((40 + 30 * Math.sin((i + seed) / 2) + i * 6) * scale));
      const series = o.several ? [{ name: 'Films', values: at(1, 1) }, { name: 'Episodes', values: at(3, 1.4) }] : [{ name: 'Plays', values: at(1, 2) }];
      return h('div', { class: 'fui-line-chart__demo' }, lineChart({ title: 'Plays', sub: 'Each month of the last year', labels: MONTHS, series, area: o.wash }));
    },
  },
};
