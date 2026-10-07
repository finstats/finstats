// FinUI: hero. The opening of a landing page: what it is, in a large title and a line or two under it, what to do
// next, and beside it the thing itself to look at. Once per page, at the top.

import { h } from '../../core.js';
import { button } from '../button/button.js';
import { card } from '../card/card.js';

/** hero({ title, lede, actions, aside, children }): `children` go under the actions (a picker, an install line). */
export function hero({ title, lede = null, actions = null, aside = null, children = [] }) {
  return h('section', { class: ['fui-hero', aside && 'fui-hero--aside'] },
    h('div', { class: 'fui-hero__copy' },
      h('h1', { class: 'fui-hero__title' }, title),
      lede ? h('p', { class: 'fui-hero__lede' }, lede) : null,
      actions ? h('div', { class: 'fui-hero__actions' }, actions) : null,
      children),
    aside ? h('div', { class: 'fui-hero__aside' }, aside) : null);
}

export const meta = {
  name: 'hero',
  purpose: 'Opens a landing page: what it is in a large title, a line or two under it, what to do next, and the thing itself beside it.',
  use: 'Once, at the top of a page that introduces something: a project’s site, a feature’s first screen. aside holds what it is about, shown rather than told.',
  avoid: 'A page that does a job (a dashboard opens on its numbers, with page-header). A second one further down.',
  variants: ['with something beside it', 'with actions', 'with more under the actions'],
  states: [],
  a11y: 'The page’s one <h1>. The actions are buttons or links, in reading order after the words.',
  props: { 'hero({ title, lede, actions, aside, children })': 'children go under the actions' },
  playground: {
    controls: [
      { key: 'aside', label: 'Something beside it', on: true },
      { key: 'actions', label: 'Actions', on: true },
    ],
    render: (o) => hero({
      title: 'Every evening on your server, counted.',
      lede: 'Who watched what, for how long and on what: read from Jellyfin, kept on your own machine.',
      actions: o.actions ? [button({ variant: 'primary' }, 'Get started'), button({}, 'Read the docs')] : null,
      aside: o.aside ? card({ title: 'Last night', body: h('p', null, '3 people, 4 films, 6h 12m.') }) : null,
    }),
  },
};
