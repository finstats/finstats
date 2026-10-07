// FinUI: breadcrumb. Where a page sits: the places above it as links, itself last and not a link. A long trail keeps its
// first place and its last ones, and the rest wait behind an ellipsis that brings them back.

import { h, icon } from '../../core.js';
import { collapse } from './plan.js';

/** breadcrumb({ items: [{ label, href }], max }): the last item is the page itself. */
export function breadcrumb({ items, max = 4 }) {
  const list = h('ol', { class: 'fui-breadcrumb__list' });
  const draw = (all) => list.replaceChildren(...(all ? items : collapse(items, max)).map((it, i, shown) => {
    const last = i === shown.length - 1;
    const sep = last ? null : icon('chevronRight', 12, 'fui-breadcrumb__sep');
    if (it === null) return h('li', { class: 'fui-breadcrumb__item' }, h('button', { type: 'button', class: 'fui-breadcrumb__more', 'aria-label': 'Show every place', onClick: () => draw(true) }, '…'), sep);
    return h('li', { class: 'fui-breadcrumb__item' }, last || !it.href ? h('span', { class: 'fui-breadcrumb__here', 'aria-current': last ? 'page' : null }, it.label) : h('a', { class: 'fui-breadcrumb__link', href: it.href }, it.label), sep);
  }));
  draw(false);
  return h('nav', { class: 'fui-breadcrumb', 'aria-label': 'Breadcrumb' }, list);
}

const TRAIL = [{ label: 'Home', href: '#' }, { label: 'Libraries', href: '#' }, { label: 'Shows', href: '#' }, { label: 'Low Orbit', href: '#' }, { label: 'Season 2', href: '#' }, { label: 'Episode 5' }];
export const meta = {
  name: 'breadcrumb',
  purpose: 'Says where a page sits: the places above it, each a link, and the page itself last.',
  use: 'Pages deep in a hierarchy: an episode in a season in a show in a library. max keeps a long trail to its first place and its last ones.',
  avoid: 'A flat site (the top-bar says where you are). The history of where somebody has been (that is the browser’s Back).',
  variants: ['short', 'long, shortened', 'shortened, then shown whole'],
  states: ['the page itself (aria-current="page")'],
  a11y: 'A <nav> named Breadcrumb holding an ordered list; the page itself says aria-current="page"; the ellipsis is a button named "Show every place".',
  props: { 'breadcrumb({ items, max })': 'items: [{ label, href }], the last the page itself' },
  playground: {
    controls: [
      { key: 'long', label: 'A long trail', on: true },
      { key: 'short', label: 'Shortened' , on: true },
    ],
    render: (o) => breadcrumb({ items: o.long ? TRAIL : [TRAIL[0], TRAIL[1], { label: 'Shows' }], max: o.short ? 4 : 99 }),
  },
};
