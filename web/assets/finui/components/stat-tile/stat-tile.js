// finui: stat-tile. One number that matters, with its label and, beside it, how it moved since the period before.

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
  examples: [
    { name: 'A row of tiles', render: () => h('div', { class: 'fui-stat-tile__grid' }, statTile({ label: 'Watch time', value: '42h', current: 42, previous: 36, vsLabel: 'vs last week' }), statTile({ label: 'Plays', value: '128', current: 128, previous: 140, vsLabel: 'vs last week' }), statTile({ label: 'People', value: '5', current: 5, previous: 5, vsLabel: 'vs last week' }), statTile({ label: 'Last played', value: 'just now', hint: 'Big Buck Bunny' })) },
    { name: 'Pressable, one pressed', render: () => h('div', { class: 'fui-stat-tile__grid' }, ['Gaps', 'Copies', 'Thin files'].map((l, i) => h('button', { type: 'button', class: ['fui-stat-tile fui-stat-tile--pressable', i === 2 && 'is-quiet'], 'aria-pressed': String(i === 0) }, h('span', { class: 'fui-stat-tile__label' }, l), h('span', { class: 'fui-stat-tile__value' }, String([3, 1, 0][i]))))) },
  ],
};
