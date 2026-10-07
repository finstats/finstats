// FinUI: rank-list. The most of something, in order: a number, a picture, a name and a line under it, and on the right
// the figure it was ranked by with a smaller one under it.

import { h } from '../../core.js';
import { avatar } from '../avatar/avatar.js';

/** rankList(rows): rows are { href, thumb, name, sub, value, note }; value is a node or text, note text. A row without a
 *  sub has no second line, so its name sits in the middle of the row beside its picture. */
export function rankList(rows) {
  return h('ol', { class: 'fui-rank-list' }, rows.map((r, i) => h('li', { class: 'fui-rank-list__row' },
    h('span', { class: 'fui-rank-list__rank mono' }, String(i + 1)),
    r.thumb || null,
    h('div', { class: 'fui-rank-list__main' },
      r.href ? h('a', { href: r.href, class: 'fui-rank-list__name' }, r.name) : h('span', { class: 'fui-rank-list__name' }, r.name),
      r.sub ? h('div', { class: 'fui-rank-list__sub' }, r.sub) : null),
    h('div', { class: 'fui-rank-list__nums' }, typeof r.value === 'string' ? h('span', { class: 'mono fui-rank-list__watch' }, r.value) : r.value,
      h('span', { class: 'mono fui-rank-list__plays' }, r.note)))));
}

export const meta = {
  name: 'rank-list',
  purpose: 'Lists the most of something in order, with the figure each was ranked by.',
  use: 'Most watched titles, people, libraries: ten or so rows, each a link to its own page.',
  avoid: 'A list that is not ranked (use a plain list). More than about fifteen rows (a table sorts).',
  variants: ['with posters', 'with avatars', 'without pictures'],
  states: [],
  a11y: 'An ordered list: the order is the ranking. The name is the link.',
  props: { 'rankList(rows)': '[{ href, thumb, name, sub, value, note }]' },
  playground: {
    controls: [
      { key: 'thumb', label: 'Pictures', on: true },
      { key: 'sub', label: 'A line under the name', on: true },
      { key: 'note', label: 'A note under the value', on: true },
    ],
    render: (o) => rankList([['Big Buck Bunny', '2008 · 4 users', '12h 4m', '31 plays'], ['Sintel', '2010 · 3 users', '6h 50m', '18 plays'], ['Tears of Steel', '2012 · 1 user', '1h 2m', '1 play']]
      .map(([name, sub, value, note]) => ({ href: '#', name, sub: o.sub ? sub : null, value, note: o.note ? note : null, thumb: o.thumb ? avatar(null, name, { size: 36 }) : null }))),
  },
};
