// FinUI: facts. A few things about one thing, label over value, in a grid: what a file is, where it lives, how big.

import { h } from '../../core.js';

/** A grid of label/value pairs. `{ wide: true }` gives a pair the whole row: a long value (a file path) squeezed into
 *  one cell broke onto a line every few characters beside a row left empty. */
export function facts(pairs) {
  return h('dl', { class: 'fui-facts' }, pairs.filter(Boolean).map(([k, v, opts]) =>
    h('div', { class: ['fui-facts__item', opts && opts.wide && 'fui-facts__item--wide'] }, h('dt', null, k), h('dd', { class: opts && opts.mono ? 'mono' : '' }, v == null || v === '' ? '–' : v))));
}

export const meta = {
  name: 'facts',
  purpose: 'Lists label and value pairs about one thing, in a grid that wraps.',
  use: 'The details of one title, play, server or file. { wide: true } for a long value (a path) that needs the whole row; { mono: true } for a number or an address.',
  avoid: 'A table of many rows of the same shape (dataTable). Prose.',
  variants: ['grid', 'wide item', 'mono value', 'missing value (–)'],
  states: [],
  a11y: 'A <dl> of <dt> and <dd>, read as term and description.',
  props: { 'facts(pairs)': 'pairs: [label, value, { wide, mono }]; a falsy pair is left out; an empty value prints –' },
  playground: {
    controls: [
      { key: 'wide', label: 'A long value, the whole row', on: true },
      { key: 'mono', label: 'Numbers in mono', on: true },
      { key: 'missing', label: 'A value not known' },
    ],
    render: (o) => facts([['Resolution', '1080p HEVC'], ['Size', '4.1 GB', { mono: o.mono }], ['Library', 'Films'], ['Added', o.missing ? null : '3 days ago'],
      o.wide ? ['Path', h('span', { class: 'mono' }, '/media/films/Big Buck Bunny (2008)/Big Buck Bunny.mkv'), { wide: true }] : null]),
  },
};
