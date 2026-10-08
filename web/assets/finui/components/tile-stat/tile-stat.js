// FinUI: tile-stat. A square tile with an icon on a colour, and beside it a name with its figure in large numbers: an app
// and how much it was used, a person and how much they watched. The tile alone marks a kind of thing at a glance.

import { h, icon } from '../../core.js';

/** The colours a tile may be, each a token: the accent, good, critical and the second and fourth series, or neutral. */
const TONES = ['accent', 'good', 'critical', 'series-2', 'series-4', 'neutral'];

/** tile({ icon, tone, size, label }): a square of a tone with an icon on it; `label` names it for a screen reader. */
export function tile({ icon: name, tone = 'neutral', size = 48, label = null }) {
  const el = h('span', { class: ['fui-tile-stat__tile', `fui-tile-stat__tile--${TONES.includes(tone) ? tone : 'neutral'}`], role: label ? 'img' : null, 'aria-label': label, 'aria-hidden': label ? null : 'true' }, icon(name, Math.round(size * 0.55)));
  el.style.width = el.style.height = `${size}px`;
  return el;
}

/** tileStat({ picture, name, figure, unit, href }): picture is a node (a tile, an avatar); figure the number; unit the small
 *  word after it ("plays"). With `href` the name is a link. */
export function tileStat({ picture, name, figure, unit = null, href = null }) {
  return h('div', { class: 'fui-tile-stat' }, picture,
    h('div', { class: 'fui-tile-stat__text' },
      href ? h('a', { class: 'fui-tile-stat__name', href }, name) : h('span', { class: 'fui-tile-stat__name' }, name),
      h('span', { class: 'fui-tile-stat__figure' }, figure, unit ? h('span', { class: 'fui-tile-stat__unit' }, unit) : null)));
}

/** tileStats(items): a grid of them that wraps. */
export const tileStats = (items) => h('div', { class: 'fui-tile-stat__grid' }, items);

export const meta = {
  name: 'tile-stat',
  purpose: 'Shows one thing as a coloured tile or a picture, its name, and a figure in large numbers beside it.',
  use: 'How much each app or person was used, a grid of them; the tile alone marks a kind of app in a list.',
  avoid: 'A ranking that needs its order read (rank-list). A figure that needs its trend (stat-tile).',
  variants: ['with a tile', 'with an avatar', 'a tile alone', 'in a grid'],
  states: [],
  a11y: 'The tile is decoration beside its name unless it is given a label; the figure is text.',
  props: { 'tile({ icon, tone, size, label })': `tone: ${TONES.join(' | ')}`, 'tileStat({ picture, name, figure, unit, href })': 'one', 'tileStats(items)': 'a grid' },
  playground: {
    controls: [{ key: 'unit', label: 'A unit after the figure', on: true }],
    render: (o) => tileStats([['globe', 'accent', 'Web browser', '1,745'], ['smartphone', 'good', 'Phone', '572'], ['tv', 'series-4', 'Living room TV', '223'], ['music', 'critical', 'Music player', '17']]
      .map(([i, tone, name, n]) => tileStat({ picture: tile({ icon: i, tone }), name, figure: n, unit: o.unit ? 'plays' : null }))),
  },
};
