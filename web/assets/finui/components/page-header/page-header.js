// finui: page-header. A page's name, a line under it, and whatever belongs on its right: chips, a picture.

import { h } from '../../core.js';

export function pageHeader(title, sub, right) {
  return h('header', { class: 'fui-page-header' },
    h('div', null, h('h1', { class: 'fui-page-header__title' }, title), sub ? h('p', { class: 'fui-page-header__sub' }, sub) : null),
    right ? h('div', { class: 'fui-page-header__right' }, right) : null);
}

export const meta = {
  name: 'page-header',
  purpose: 'Names the page and says in a line what it shows.',
  use: 'At the top of every page: an <h1> title, a sub line, and on the right what describes the whole page (chips, artwork).',
  avoid: 'Buttons that act on one card (they belong to that card). A second <h1>.',
  variants: ['title', 'with a sub line', 'with something on the right'],
  states: [],
  a11y: 'The page’s one <h1>.',
  props: { 'pageHeader(title, sub, right)': 'title and sub are text or nodes; right is a node' },
  examples: [
    { name: 'A title and a line', render: () => pageHeader('Libraries', 'What’s on the server and how much of it gets watched') },
    { name: 'With chips on the right', render: () => pageHeader('Jellyfin', 'Jellyfin 10.11.0', h('span', { class: 'fui-page-header__demo-chip' }, 'Update available')) },
  ],
};
