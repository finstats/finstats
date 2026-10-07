// FinUI: page-header. A page's name, a line under it, and whatever belongs on its right: chips, a picture.

import { h } from '../../core.js';

export function pageHeader(title, sub, right) {
  return h('header', { class: 'fui-page-header' },
    h('div', null, h('h1', { class: 'fui-page-header__title' }, title), sub ? h('p', { class: 'fui-page-header__sub' }, sub) : null),
    right ? h('div', { class: 'fui-page-header__right' }, right) : null);
}

/** sectionHeader(title, sub, right): the same, one level down: a part of the page, under its <h1>. */
export function sectionHeader(title, sub, right) {
  return h('header', { class: 'fui-page-header fui-page-header--section' },
    h('div', null, h('h2', { class: 'fui-page-header__title' }, title), sub ? h('p', { class: 'fui-page-header__sub' }, sub) : null),
    right ? h('div', { class: 'fui-page-header__right' }, right) : null);
}

export const meta = {
  name: 'page-header',
  purpose: 'Names the page and says in a line what it shows.',
  use: 'At the top of every page: an <h1> title, a sub line, and on the right what describes the whole page (chips, artwork).',
  avoid: 'Buttons that act on one card (they belong to that card). A second <h1>.',
  variants: ['title', 'with a sub line', 'with something on the right', 'a section (sectionHeader, an <h2>)'],
  states: [],
  a11y: 'The page’s one <h1>; sectionHeader is an <h2> under it.',
  props: { 'pageHeader(title, sub, right)': 'title and sub are text or nodes; right is a node', 'sectionHeader(title, sub, right)': 'the same for a part of the page' },
  playground: {
    controls: [
      { key: 'sub', label: 'A line under it', on: true },
      { key: 'right', label: 'Something on the right' },
      { key: 'section', label: 'A section of the page' },
    ],
    render: (o) => (o.section ? sectionHeader : pageHeader)('Libraries', o.sub ? 'What’s on the server and how much of it gets watched' : null, o.right ? h('span', { class: 'fui-page-header__demo-chip' }, 'Update available') : null),
  },
};
