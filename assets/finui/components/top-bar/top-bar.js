// FinUI: top-bar. A site's bar across the top of every page: its name, its sections as links (the one you are in
// marked), and on the right what belongs to every page: a theme switch, an account. It stays as the page scrolls.

import { h } from '../../core.js';
import { button } from '../button/button.js';
import { say } from '../../i18n.js';

/** topBar({ brand: { name, mark, href }, links: [{ href, label, key, name }], current, lead, actions, variant, label }) → the
 *  bar, with .setCurrent(key) to mark another section. On a phone its links scroll sideways rather than wrap. `mark`: a
 *  picture before the name. A link whose `label` is an icon gives its words as `name`. `lead`: a control just before the
 *  links, such as a search. `variant: 'solid'`: an app's own bar, opaque, the links on the right and the open one lifted. */
export function topBar({ brand = null, links = [], current = null, lead = null, actions = null, variant = null, label = say('top_bar.label', 'Sections') }) {
  const items = links.map((l) => h('a', { class: 'fui-top-bar__link', href: l.href, 'aria-label': l.name || null, title: l.name || null, dataset: l.key ? { key: l.key } : {} }, l.label));
  const el = h('header', { class: ['fui-top-bar', variant === 'solid' && 'fui-top-bar--solid'] },
    brand ? h('a', { class: 'fui-top-bar__brand', href: brand.href || '/' }, brand.mark || null, brand.name) : null,
    lead ? h('div', { class: 'fui-top-bar__lead' }, lead) : null,
    h('nav', { class: 'fui-top-bar__nav', 'aria-label': label }, items),
    actions ? h('div', { class: 'fui-top-bar__actions' }, actions) : null);
  el.setCurrent = (key) => {
    for (const a of items) {
      const on = !!key && a.dataset.key === key;
      a.classList.toggle('is-active', on);
      if (on) a.setAttribute('aria-current', 'page'); else a.removeAttribute('aria-current');
    }
  };
  el.setCurrent(current);
  return el;
}

export const meta = {
  name: 'top-bar',
  purpose: 'Carries a site’s name and its sections across the top of every page, the one you are in marked.',
  use: 'A website or a small app of a few sections: its name links home, each section is a link, and actions holds what every page needs (a theme switch, sign in).',
  avoid: 'An app of many pages (desktop-nav and mobile-nav). Actions that belong to one page.',
  variants: ['with actions', 'a section marked', 'on a phone (the links scroll)', 'solid: an app’s own bar, the links on the right'],
  states: ['the open section (aria-current="page")'],
  a11y: 'A <header> holding a <nav> of links; the open one says aria-current="page".',
  props: { 'topBar({ brand, links, current, lead, actions, variant, label })': 'brand: { name, mark, href }; links: [{ href, label, key, name }]; variant: \'solid\' → the bar, with .setCurrent(key)' },
  playground: {
    controls: [
      { key: 'current', label: 'Open section', choices: [['guide', 'Guide'], ['blog', 'Blog'], ['none', 'None']] },
      { key: 'actions', label: 'Actions' },
      { key: 'solid', label: 'Solid' },
    ],
    render: (o) => h('div', { class: 'fui-top-bar__demo' }, topBar({ brand: { name: 'Northwind', href: '#' }, current: o.current === 'none' ? null : o.current, variant: o.solid ? 'solid' : null,
      links: [{ href: '#', label: 'Guide', key: 'guide' }, { href: '#', label: 'Blog', key: 'blog' }, { href: '#', label: 'About', key: 'about' }],
      actions: o.actions ? button({ size: 'sm', variant: 'primary', href: '#' }, 'Sign in') : null })),
  },
};
