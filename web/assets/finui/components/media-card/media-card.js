// finui: media-card. A title as a poster with its name and a line under it, the whole card a link; in a grid that fits as
// many as the width allows.

import { h } from '../../core.js';

/** mediaCard({ href, poster, name, sub }): poster is a node (finui's poster, size --grid). */
export function mediaCard({ href, poster, name, sub }) {
  return h('a', { class: 'fui-media-card', href }, poster, h('span', { class: 'fui-media-card__name' }, name), h('span', { class: 'fui-media-card__sub' }, sub));
}

/** mediaGrid(cards, { class }): a list of cards. */
export const mediaGrid = (cards, { class: extra = null } = {}) => h('ul', { class: ['fui-media-card__grid', extra] }, cards.map((c) => h('li', null, c)));

export const meta = {
  name: 'media-card',
  purpose: 'Shows a title as its poster, its name and a line under it, the whole card leading to it.',
  use: 'A grid of titles: what was added, everything in a library.',
  avoid: 'A row of titles that scrolls sideways (the dashboard’s shelf). A card with buttons inside it: the card is one link.',
  variants: ['in a grid'],
  states: ['hover and focus outline the poster'],
  a11y: 'One link per card, named by the title; the poster is decoration.',
  props: { 'mediaCard({ href, poster, name, sub })': 'one card', 'mediaGrid(cards, { class })': 'a grid of them' },
  examples: [
    { name: 'A grid', render: () => mediaGrid(['Big Buck Bunny', 'Sintel', 'Tears of Steel', 'Cosmos Laundromat'].map((n, i) => mediaCard({ href: '#', poster: h('span', { class: 'fui-media-card__demo-poster', 'aria-hidden': 'true' }), name: n, sub: String(2008 + i * 2) }))) },
  ],
};
