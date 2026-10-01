// finui: rank-list. The most of something, in order: a number, a picture, a name and a line under it, and on the right
// the figure it was ranked by with a smaller one under it.

import { h } from '../../core.js';

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
  examples: [
    { name: 'The most watched', render: () => rankList([
      { href: '#', name: 'Big Buck Bunny', sub: '2008 · 4 users', value: '12h 4m', note: '31 plays' },
      { href: '#', name: 'Sintel', sub: '2010 · 3 users', value: '6h 50m', note: '18 plays' },
      { href: '#', name: 'Tears of Steel', sub: '2012 · 1 user', value: '1h 2m', note: '1 play' }]) },
    { name: 'People, with nothing under the name', render: () => rankList([
      { href: '#', name: 'alice', value: '20h', note: '40 plays' }, { href: '#', name: 'bob', value: '14h', note: '29 plays' }]) },
  ],
};
