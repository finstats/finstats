// FinUI: skeleton. The shape of what is coming, in grey, while it comes: a line, a block, rows, tiles, a card. It shows
// only when loading is slow (dataView waits 150 ms) and then stays long enough to read as deliberate.

import { h } from '../../core.js';

export const sk = {
  line: (w = '60%', hgt = 12) => h('span', { class: 'fui-skeleton', style: { width: w, height: hgt + 'px' } }),
  block: (hgt = 120) => h('div', { class: 'fui-skeleton fui-skeleton--block', style: { height: hgt + 'px' } }),
  tiles: (n = 4) => h('div', { class: 'fui-stat-tile__grid' }, Array.from({ length: n }, () =>
    h('div', { class: 'fui-stat-tile' }, sk.line('45%', 11), sk.line('60%', 26), sk.line('35%', 10)))),
  rows: (n = 5) => h('div', { class: 'fui-skeleton__rows' }, Array.from({ length: Math.min(n, 5) }, () =>
    h('div', { class: 'fui-skeleton__row' }, h('span', { class: 'fui-skeleton fui-skeleton--thumb' }), h('div', { class: 'fui-skeleton__lines' }, sk.line('55%'), sk.line('30%', 10))))),
  tableRows: (n = 5) => h('div', { class: 'fui-skeleton__rows' }, Array.from({ length: Math.min(n, 5) }, () =>
    h('div', { class: 'fui-skeleton__row' }, sk.line('18%'), sk.line('34%'), sk.line('12%'), sk.line('14%')))),
  cardBlock: (hgt = 240, title = true) => h('section', { class: 'fui-card' }, title ? h('div', { class: 'fui-card__head' }, sk.line('28%', 14)) : null, h('div', { class: 'fui-card__body' }, sk.block(hgt))),
  cardRows: (n = 5) => h('section', { class: 'fui-card' }, h('div', { class: 'fui-card__head' }, sk.line('28%', 14)), h('div', { class: 'fui-card__body' }, sk.rows(n))),
};

export const meta = {
  name: 'skeleton',
  purpose: 'Holds the place of what is loading, in its shape, so the page does not jump when it arrives.',
  use: 'As a view’s first paint when loading is slow. Shaped like what comes: rows for a list, a block for a chart, tiles for tiles.',
  avoid: 'A skeleton that flashes for a moment (wait before showing one). A skeleton for something that failed (error) or is empty (empty).',
  variants: ['line', 'block', 'rows', 'table rows', 'tiles', 'card with a block', 'card with rows'],
  states: ['breathes; holds still with reduced motion'],
  a11y: 'Decorative: the view around it says it is loading where that matters.',
  props: { 'sk.line(width, height)': 'a line of text', 'sk.block(height)': 'a chart or a picture', 'sk.rows(n)': 'a list', 'sk.tableRows(n)': 'a table', 'sk.tiles(n)': 'a row of tiles', 'sk.cardBlock(height, title)': 'a card', 'sk.cardRows(n)': 'a card with a list' },
  playground: {
    controls: [
      { key: 'shape', label: 'Shape', choices: [['lines', 'Lines'], ['rows', 'A list'], ['table', 'A table'], ['tiles', 'Tiles'], ['card', 'A card']] },
      { key: 'more', label: 'More of it' },
    ],
    render: (o) => {
      const n = o.more ? 5 : 2;
      if (o.shape === 'rows') return sk.rows(n);
      if (o.shape === 'table') return sk.tableRows(n);
      if (o.shape === 'tiles') return sk.tiles(o.more ? 4 : 2);
      if (o.shape === 'card') return sk.cardRows(n);
      return h('div', null, Array.from({ length: n }, (_, i) => [sk.line(i ? '70%' : '40%', i ? 12 : 14), h('br')]), sk.block(o.more ? 140 : 90));
    },
  },
};
