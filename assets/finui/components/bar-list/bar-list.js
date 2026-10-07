// FinUI: bar-list. A ranked list where each row's length is its amount: the name and the value in words, a thin bar under
// them from the same start, longest first. Rows may lead somewhere.

import { h } from '../../core.js';
import { compact } from '../chart/plan.js';

/** barList({ items: [{ name, value, href, note }], format, max }): `max` rows, the rest summed into "And N more". */
export function barList({ items, format = compact, max = 8 }) {
  const sorted = [...items].sort((a, b) => b.value - a.value);
  const shown = sorted.length > max ? sorted.slice(0, max - 1) : sorted;
  const rest = sorted.slice(shown.length);
  const top = Math.max(1, ...shown.map((i) => i.value), rest.reduce((n, i) => n + i.value, 0));
  const row = (it, other = false) => {
    const fill = h('span', { class: 'fui-bar-list__fill' });
    fill.style.width = `${Math.max(1, (it.value / top) * 100)}%`;
    const name = it.href ? h('a', { class: 'fui-bar-list__name', href: it.href }, it.name) : h('span', { class: 'fui-bar-list__name' }, it.name);
    return h('li', { class: ['fui-bar-list__row', other && 'is-other'] },
      h('div', { class: 'fui-bar-list__line' }, name, it.note ? h('span', { class: 'fui-bar-list__note' }, it.note) : null, h('span', { class: 'fui-bar-list__value' }, format(it.value))),
      h('span', { class: 'fui-bar-list__track', 'aria-hidden': 'true' }, fill));
  };
  return h('ol', { class: 'fui-bar-list' }, shown.map((it) => row(it)), rest.length ? row({ name: `And ${rest.length} more`, value: rest.reduce((n, i) => n + i.value, 0) }, true) : null);
}

const CLIENTS = [['Jellyfin Web', 412], ['Swiftfin', 238], ['Findroid', 190], ['Infuse', 96], ['Kodi', 41], ['Roku', 22], ['Android TV', 18]];
export const meta = {
  name: 'bar-list',
  purpose: 'Ranks things by an amount, each row’s bar as long as its share of the largest.',
  use: 'The clients people watch on, the most watched titles, the busiest people: a list read top-down, with the amount beside each name. max keeps it short; the rest are summed into one row.',
  avoid: 'Amounts over time (bar-chart, line-chart). Parts of one whole that should add to 100% (donut-chart).',
  variants: ['plain', 'with links', 'with a note per row', 'more than fit (And N more)'],
  states: [],
  a11y: 'An ordered list: rank, name and value are text, read in order; the bar is decoration (aria-hidden).',
  props: { 'barList({ items, format, max })': 'items: [{ name, value, href, note }]' },
  playground: {
    controls: [
      { key: 'links', label: 'Rows lead somewhere' },
      { key: 'notes', label: 'A note per row' },
      { key: 'short', label: 'At most five' },
    ],
    render: (o) => h('div', { class: 'fui-bar-list__demo' }, barList({ max: o.short ? 5 : 8, format: (v) => `${v} plays`,
      items: CLIENTS.map(([name, value], i) => ({ name, value, href: o.links ? '#' : null, note: o.notes ? `${7 - i} people` : null })) })),
  },
};
