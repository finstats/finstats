// FinUI: stat-tile. One number that matters, with its label and, beside it, how it moved since the period before.

import { h, icon } from '../../core.js';

function deltaEl(cur, prev) {
  if (prev == null) return null;
  if (!prev && !cur) return h('span', { class: 'fui-stat-tile__delta fui-stat-tile__delta--flat' }, icon('minus', 13), 'No change');
  if (!prev) return h('span', { class: 'fui-stat-tile__delta fui-stat-tile__delta--up' }, icon('trendUp', 13), 'New');
  const d = (cur - prev) / prev;
  if (Math.abs(d) < 0.005) return h('span', { class: 'fui-stat-tile__delta fui-stat-tile__delta--flat' }, icon('minus', 13), 'No change');
  const up = d > 0;
  return h('span', { class: 'fui-stat-tile__delta ' + (up ? 'fui-stat-tile__delta--up' : 'fui-stat-tile__delta--down') }, icon(up ? 'trendUp' : 'trendDown', 13),
    `${up ? '+' : '−'}${Math.abs(d) >= 10 ? Math.round(Math.abs(d) * 100) : (Math.abs(d) * 100).toFixed(Math.abs(d) < 0.1 ? 1 : 0)}%`);
}

export function statTile({ label, value, title, current, previous, vsLabel, spark, hint }) {
  return h('div', { class: 'fui-stat-tile' },
    h('div', { class: 'fui-stat-tile__label' }, label),
    h('div', { class: 'fui-stat-tile__row' }, h('div', { class: 'fui-stat-tile__value', title }, value), spark ? h('div', { class: 'fui-stat-tile__spark' }, spark) : null),
    h('div', { class: 'fui-stat-tile__foot' }, previous != null ? [deltaEl(current, previous), h('span', { class: 'fui-stat-tile__vs' }, vsLabel)] : hint ? h('span', { class: 'fui-stat-tile__vs' }, hint) : h('span', { class: 'fui-stat-tile__vs' }, ' ')));
}

export const meta = {
  name: 'stat-tile',
  purpose: 'Shows one number with its label, and how it changed since the period before.',
  use: 'A row of the few numbers a page is about (fui-stat-tile__grid). --pressable when a tile also picks what is shown below it (aria-pressed). The grid --three for three, --quiet for figures that support rather than lead. Three, four, six or eight tiles keep their rows even: never one left alone under the others.',
  avoid: 'A tile for a sentence. More than six in a row. A delta without the period it compares with (vsLabel).',
  variants: ['plain', 'with a change', 'with a hint', 'pressable', 'quiet grid'],
  states: ['up, down, no change, new', 'pressed'],
  a11y: 'Text throughout; the change is said in words and a sign (+12%), not only by its colour and arrow.',
  props: { 'statTile({ label, value, title, current, previous, vsLabel, spark, hint })': 'current and previous make the change; spark is a small chart beside the value' },
  playground: {
    controls: [
      { key: 'change', label: 'How it moved', on: true },
      { key: 'spark', label: 'A sparkline' },
      { key: 'pressable', label: 'Pressable' },
    ],
    render: (o) => {
      const spark = o.spark ? h('span', { class: 'fui-stat-tile__demo-spark', 'aria-hidden': 'true' }) : null;
      const tile = (label, value, cur, prev) => statTile({ label, value, current: o.change ? cur : undefined, previous: o.change ? prev : undefined, vsLabel: 'vs last week', hint: o.change ? null : 'This week', spark });
      if (o.pressable) return h('div', { class: 'fui-stat-tile__grid' }, ['Gaps', 'Copies', 'Thin files'].map((l, i) => h('button', { type: 'button', class: ['fui-stat-tile fui-stat-tile--pressable', i === 2 && 'is-quiet'], 'aria-pressed': String(i === 0) }, h('span', { class: 'fui-stat-tile__label' }, l), h('span', { class: 'fui-stat-tile__value' }, String([3, 1, 0][i])))));
      return h('div', { class: 'fui-stat-tile__grid' }, tile('Watch time', '42h', 42, 36), tile('Plays', '128', 128, 140), tile('People', '5', 5, 5));
    },
  },
};
