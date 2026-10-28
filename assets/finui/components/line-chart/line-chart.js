// FinUI: line-chart. A measure over time as a 2 px line (a wash of its colour under it when there is one series) with
// round steps up the side, a dot ringed in the ground at the end, and a crosshair that a pointer and the arrow keys move
// along, the tooltip saying every series at that moment. A limit is a dashed line in the critical colour, named at its
// left end, that the axis always reaches, and a line that goes past it is drawn in that colour above it; marks are
// moments shaded the plot's height in a status colour (when a limit was hit, when something was held back), named in
// the legend and the tooltip.

import { h, s } from '../../core.js';
import { niceTicks, linear, linePath, compact, runs } from '../chart/plan.js';
import { chartFigure, legend, tableTwin, tone, foldSeries, sized, hoverable } from '../chart/chart.js';
import { say } from '../../i18n.js';

const r2 = (n) => Math.round(n * 100) / 100;
const MARK_TONES = { critical: 'var(--critical)', warning: 'var(--warning)' };
let clips = 0;
/** lineChart({ title, sub, labels, series: [{ name, values, colour }], format, area, height, limit: { value, label }, marks: [{ name, tone, values }] }) */
export function lineChart({ title = null, sub = null, labels, series, format = compact, area = null, height = 220, limit = null, marks = [] }) {
  const all = foldSeries(series);
  // A series may name its own colour (a total that is nobody's, say); else it has its place's.
  const hue = (k) => all[k].colour || tone(k);
  const wash = area ?? all.length === 1;
  const box = h('div', { class: 'fui-line-chart__box' });
  const limitName = limit ? `${limit.label || say('line_chart.limit', 'Limit')} (${format(limit.value)})` : null;
  // With a limit or marks, the legend names the line too, so every key says what it is.
  const keys = [
    ...(all.length > 1 || limit || marks.length ? all.map((x, k) => ({ name: x.name, colour: hue(k) })) : []),
    ...(limit ? [{ name: limitName, colour: MARK_TONES.critical }] : []),
    ...marks.map((mk) => ({ name: mk.name, colour: MARK_TONES[mk.tone] || MARK_TONES.warning })),
  ];
  const figure = chartFigure({ title, sub, legend: keys.length ? legend(keys) : null, plot: box,
    table: tableTwin({ caption: title || say('chart.numbers', 'The numbers'), head: ['', ...all.map((x) => x.name), ...marks.map((mk) => mk.name), ...(limit ? [limitName] : [])],
      rows: labels.map((l, i) => [l, ...all.map((x) => format(x.values[i] || 0)), ...marks.map((mk) => format(mk.values[i] || 0)), ...(limit ? [format(limit.value)] : [])]) }) });
  let spec = null;
  const named = title || say('line_chart.name', 'A line chart');
  hoverable(box, () => spec, say('line_chart.aria', `${named}: ${labels[0]} to ${labels[labels.length - 1]}`, { title: named, from: labels[0], to: labels[labels.length - 1] }));
  sized(box, (width) => {
    // The axis reaches the limit even when nothing has come near it, and goes past it when something did.
    const top = Math.max(0, ...all.flatMap((x) => x.values), limit ? limit.value : 0);
    const ticks = niceTicks(0, top, 4, { whole: all.every((x) => x.values.every((v) => Number.isInteger(v || 0))) && (!limit || Number.isInteger(limit.value)) });
    const m = { top: 10, right: 10, bottom: 22, left: Math.max(28, 8 + 7 * Math.max(...ticks.map((t) => format(t).length))) };
    const pw = width - m.left - m.right, ph = height - m.top - m.bottom;
    const y = linear([0, ticks[ticks.length - 1]], [m.top + ph, m.top]);
    const x = linear([0, Math.max(1, labels.length - 1)], [m.left, m.left + pw]);
    const every = Math.max(1, Math.ceil(labels.length / Math.max(1, Math.floor(pw / 56))));
    const cross = s('line', { class: 'fui-chart__cross', y1: String(m.top), y2: String(m.top + ph), visibility: 'hidden' });
    const dots = all.map((_, k) => { const d = s('circle', { class: 'fui-chart__dot', r: '4', visibility: 'hidden' }); d.style.fill = hue(k); return d; });
    // Above the limit: what is drawn there again, clipped to it, in the critical colour.
    const clip = limit ? `fui-line-chart-over-${++clips}` : null;
    const over = limit ? s('clipPath', { id: clip }, s('rect', { x: String(m.left - 6), y: '0', width: String(pw + 12), height: String(Math.max(0, r2(y(limit.value)))) })) : null;
    const lines = all.map((ser, k) => {
      const pts = ser.values.map((v, i) => [x(i), y(v || 0)]);
      const line = s('path', { class: 'fui-chart__line', d: linePath(pts) }); line.style.stroke = hue(k);
      const under = wash ? s('path', { class: 'fui-chart__area', d: `${linePath(pts)}L${r2(x(labels.length - 1))} ${r2(y(0))}L${r2(x(0))} ${r2(y(0))}Z` }) : null;
      if (under) under.style.fill = hue(k);
      const past = limit && ser.values.some((v) => (v || 0) > limit.value) ? s('path', { class: 'fui-chart__line fui-chart__over', d: linePath(pts), 'clip-path': `url(#${clip})` }) : null;
      const end = s('circle', { class: 'fui-chart__dot', r: '4', cx: String(r2(pts[pts.length - 1][0])), cy: String(r2(pts[pts.length - 1][1])) }); end.style.fill = hue(k);
      return [under, line, past, end];
    });
    // A mark's runs span the plot's height, half a step either side of their first and last moment.
    const half = labels.length > 1 ? pw / (labels.length - 1) / 2 : pw / 2;
    const bands = marks.flatMap((mk) => runs(mk.values).map(([a, b]) => {
      const x0 = Math.max(m.left, x(a) - half), x1 = Math.min(m.left + pw, x(b) + half);
      return s('rect', { class: `fui-chart__band fui-chart__band--${mk.tone === 'critical' ? 'critical' : 'warning'}`, x: String(r2(x0)), y: String(m.top), width: String(r2(Math.max(2, x1 - x0))), height: String(ph) });
    }));
    const limitLine = limit ? [
      s('line', { class: 'fui-chart__limit', x1: String(m.left), x2: String(width - m.right), y1: String(r2(y(limit.value))), y2: String(r2(y(limit.value))) }),
      // Named at its left end, where a line over time is oldest and least often in the way.
      s('text', { class: 'fui-chart__limit-label', x: String(m.left + 4), y: String(r2(y(limit.value)) - 5), 'text-anchor': 'start' }, limitName),
    ] : [];
    const svg = s('svg', { class: 'fui-chart__plot', width: String(width), height: String(height), viewBox: `0 0 ${width} ${height}`, 'aria-hidden': 'true' },
      over ? s('defs', null, over) : null,
      bands,
      ticks.map((t) => s('line', { class: t ? 'fui-chart__grid' : 'fui-chart__axis', x1: String(m.left), x2: String(width - m.right), y1: String(r2(y(t))), y2: String(r2(y(t))) })),
      ticks.map((t) => s('text', { class: 'fui-chart__tick', x: String(m.left - 6), y: String(r2(y(t)) + 3.5), 'text-anchor': 'end' }, format(t))),
      labels.map((l, i) => (i % every ? null : s('text', { class: 'fui-chart__tick', x: String(r2(x(i))), y: String(height - 6), 'text-anchor': i === 0 ? 'start' : i === labels.length - 1 ? 'end' : 'middle' }, l))),
      limitLine, lines, cross, dots);
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
        const rows = [
          ...all.map((ser, k) => ({ color: hue(k), value: format(ser.values[i] || 0), label: ser.name })),
          ...marks.filter((mk) => mk.values[i] > 0).map((mk) => ({ color: MARK_TONES[mk.tone] || MARK_TONES.warning, value: format(mk.values[i]), label: mk.name })),
          ...(limit ? [{ color: MARK_TONES.critical, value: format(limit.value), label: limit.label || say('line_chart.limit', 'Limit') }] : []),
        ];
        return { rect: { left: r.left + x(i) - 1, top: r.top + y(hi), width: 2, height: 1, bottom: r.top + y(hi) }, title: labels[i], rows };
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
  variants: ['one series with a wash', 'several series', 'a year of months', 'a limit, gone past, with the moments it was hit'],
  states: ['a moment pointed at or arrowed to (the crosshair and its dots)'],
  a11y: 'A figure named by its title; the plot is one tab stop whose arrows move the crosshair and its tooltip; every number is in a table read instead of the picture.',
  props: { 'lineChart({ title, sub, labels, series, format, area, height, limit, marks })': 'series: [{ name, values, colour }] (colour: a token of its own, else its place\'s); area: the wash (one series by default); limit: { value, label }, a dashed line in the critical colour the axis always reaches, a line past it drawn in that colour; marks: [{ name, tone: critical | warning, values }], the moments a value is above 0 shaded the plot\'s height' },
  playground: {
    controls: [
      { key: 'several', label: 'Several series' },
      { key: 'wash', label: 'A wash under it', on: true },
      { key: 'limit', label: 'A limit' },
    ],
    render: (o) => {
      const at = (seed, scale) => MONTHS.map((_, i) => Math.round((40 + 30 * Math.sin((i + seed) / 2) + i * 6) * scale));
      const series = o.several ? [{ name: 'Films', values: at(1, 1) }, { name: 'Episodes', values: at(3, 1.4) }] : [{ name: 'Plays', values: at(1, 2) }];
      // Past the limit in the summer, held back in the months after.
      const limit = o.limit ? { value: o.several ? 110 : 190, label: 'Limit' } : null;
      const marks = o.limit ? [
        { name: 'Over the limit', tone: 'critical', values: MONTHS.map((_, i) => series.some((x) => x.values[i] > limit.value) ? 1 : 0) },
        { name: 'Held back', tone: 'warning', values: MONTHS.map((_, i) => (i === 10 ? 3 : 0)) },
      ] : [];
      return h('div', { class: 'fui-line-chart__demo' }, lineChart({ title: 'Plays', sub: 'Each month of the last year', labels: MONTHS, series, area: o.wash, limit, marks }));
    },
  },
};
