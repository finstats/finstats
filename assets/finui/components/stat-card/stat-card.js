// FinUI: stat-card. One statistic as a card: a picture of what leads it, the statistic's name and the measure it is ranked
// by, and the rows under it, the first one large. A picture of the leader may lie dimmed behind the whole card.

import { h } from '../../core.js';

/** statCard({ title, column, art, picture, rows, ranked, label }): rows are [{ href, name, sub, value }]; `column` names
 *  the measure (Plays, Users); `art` is an image address drawn dimmed behind it; `picture` a node on the left (a poster,
 *  an avatar, a tile). `ranked: false` drops the numbers, for a list of facts rather than a ranking. */
export function statCard({ title, column = null, art = null, picture = null, rows = [], ranked = true }) {
  return h('section', { class: 'fui-stat-card' },
    art ? h('img', { class: 'fui-stat-card__art', src: art, alt: '', loading: 'lazy', decoding: 'async', onError: (e) => e.target.remove() }) : null,
    picture ? h('div', { class: 'fui-stat-card__picture' }, picture) : null,
    h('div', { class: 'fui-stat-card__body' },
      h('div', { class: 'fui-stat-card__head' }, h('h3', { class: 'fui-stat-card__title' }, title), column ? h('span', { class: 'fui-stat-card__column' }, column) : null),
      h(ranked ? 'ol' : 'ul', { class: 'fui-stat-card__rows' }, rows.map((r, i) => h('li', { class: 'fui-stat-card__row' },
        ranked ? h('span', { class: 'fui-stat-card__rank' }, String(i + 1)) : null,
        h('span', { class: 'fui-stat-card__name' }, r.href ? h('a', { href: r.href }, r.name) : r.name, r.sub ? h('span', { class: 'fui-stat-card__sub' }, r.sub) : null),
        h('span', { class: 'fui-stat-card__value' }, r.value))))));
}

/** statCards(cards): a grid of them that fits as many to a row as the width allows. */
export const statCards = (cards) => h('div', { class: 'fui-stat-card__grid' }, cards);

export const meta = {
  name: 'stat-card',
  purpose: 'Shows one statistic as a card: what leads it in a picture, the measure, and the ranked rows beneath.',
  use: 'A home page of statistics: the most watched shows, the most active people, a library’s counts (ranked: false).',
  avoid: 'More than about ten rows (a table sorts them). A single figure (stat-tile).',
  variants: ['ranked', 'a list of facts (ranked: false)', 'with a picture', 'with art behind it'],
  states: ['the first row is large, its value in the accent'],
  a11y: 'A section named by its heading; the ranking is an ordered list, the art decoration.',
  props: { 'statCard({ title, column, art, picture, rows, ranked })': 'rows: [{ href, name, sub, value }]', 'statCards(cards)': 'a grid' },
  playground: {
    controls: [{ key: 'ranked', label: 'Ranked', on: true }, { key: 'picture', label: 'A picture', on: true }],
    render: (o) => statCards([
      statCard({ title: 'Most watched films', column: o.ranked ? 'Plays' : null, ranked: o.ranked, picture: o.picture ? h('span', { class: 'fui-stat-card__demo-poster', 'aria-hidden': 'true' }) : null,
        rows: [['Big Buck Bunny', '31'], ['Sintel', '18'], ['Tears of Steel', '9'], ['Cosmos Laundromat', '4']].map(([name, value]) => ({ href: '#', name, value })) }),
      statCard({ title: 'Most popular shows', column: 'Users', ranked: o.ranked, picture: o.picture ? h('span', { class: 'fui-stat-card__demo-poster', 'aria-hidden': 'true' }) : null,
        rows: [['The Rookery', '6'], ['Low Orbit', '5'], ['Harbour Street', '3']].map(([name, value]) => ({ href: '#', name, value })) }),
    ]),
  },
};
