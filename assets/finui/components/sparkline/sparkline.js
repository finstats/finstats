// FinUI: sparkline. A measure's recent shape, small enough to sit beside its number: a 2 px line in the quiet colour, a
// wash under it, and the last value as a dot in the accent. No axis and no tooltip: the number beside it is the value.

import { h, s } from '../../core.js';
import { linear, linePath } from '../chart/plan.js';

const r2 = (n) => Math.round(n * 100) / 100;
/** sparkline({ values, label, width, height, area }): `label` says it for those who cannot see it. */
export function sparkline({ values, label, width = 96, height = 28, area = true }) {
  const lo = Math.min(...values, 0), hi = Math.max(...values, 1);
  const x = linear([0, Math.max(1, values.length - 1)], [3, width - 4]), y = linear([lo, hi], [height - 3, 3]);
  const pts = values.map((v, i) => [x(i), y(v)]);
  const last = pts[pts.length - 1] || [0, height / 2];
  return s('svg', { class: 'fui-sparkline', width: String(width), height: String(height), viewBox: `0 0 ${width} ${height}`, role: 'img', 'aria-label': label },
    area ? s('path', { class: 'fui-sparkline__area', d: `${linePath(pts)}L${r2(last[0])} ${height}L${r2(pts[0] ? pts[0][0] : 0)} ${height}Z` }) : null,
    s('path', { class: 'fui-sparkline__line', d: linePath(pts) }),
    s('circle', { class: 'fui-sparkline__now', cx: String(r2(last[0])), cy: String(r2(last[1])), r: '3' }));
}

const WEEKS = [12, 18, 15, 22, 19, 25, 21, 28, 24, 31, 27, 34];
export const meta = {
  name: 'sparkline',
  purpose: 'Shows a measure’s recent shape, small enough to sit beside its number.',
  use: 'Beside a stat tile’s value (stat-tile’s spark), in a table cell, in a list row: where it was going, not what it was. label says it in words.',
  avoid: 'Anything to read a value from (line-chart). Several series (it is one line).',
  variants: ['with a wash', 'a line alone', 'wider'],
  states: [],
  a11y: 'role="img" named by its label ("Plays each week, rising from 12 to 34"); the number it sits beside carries the value.',
  props: { 'sparkline({ values, label, width, height, area })': '' },
  playground: {
    controls: [
      { key: 'area', label: 'A wash under it', on: true },
      { key: 'wide', label: 'Wider' },
    ],
    render: (o) => h('span', { class: 'fui-sparkline__demo' }, h('span', null, h('strong', null, '34'), ' plays this week'), sparkline({ values: WEEKS, label: 'Plays each week, rising from 12 to 34', area: o.area, width: o.wide ? 180 : 96 })),
  },
};
