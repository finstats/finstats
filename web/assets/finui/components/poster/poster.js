// finui: poster. A title's picture at one of a few sizes, standing in with its initials when there is none or it fails
// to load. It is given an address, not an id: where pictures come from is the app's business.

import { h, mount } from '../../core.js';
import { initials } from '../../format.js';

/** poster(src, name, { cls }): src may be null; cls is a size (fui-poster--sm, --md, --lg, --grid, --still…). */
export function poster(src, name, { cls = '' } = {}) {
  const box = h('span', { class: 'fui-poster ' + cls, 'aria-hidden': 'true' });
  const fallback = () => mount(box, h('span', { class: 'fui-poster__fallback' }, initials(name)));
  if (!src) { fallback(); return box; }
  box.append(h('img', { src, alt: '', loading: 'lazy', decoding: 'async', onError: fallback }));
  return box;
}

export const meta = {
  name: 'poster',
  purpose: 'Shows a title’s picture, or its initials when it has none.',
  use: 'Beside a title’s name, at the size the row allows: --xs and --sm in tables, --md and --np in cards, --lg on a title’s page, --grid in a grid, --still for a 16:9 picture.',
  avoid: 'A poster as the only name of a title: it is aria-hidden, and the name is written beside it.',
  variants: ['xs', 'sm', 'md', 'np', 'lg', 'grid', 'still', 'placeholder'],
  states: ['loading (lazily)', 'failed → initials'],
  a11y: 'aria-hidden with an empty alt: decoration beside the name.',
  props: { 'poster(src, name, { cls })': 'src: an address or null; name gives the initials' },
  examples: [
    { name: 'Sizes, with initials for want of a picture', render: () => h('div', { class: 'fui-poster__demo' }, poster(null, 'Big Buck Bunny', { cls: 'fui-poster--sm' }), poster(null, 'Sintel', { cls: 'fui-poster--md' }), poster(null, 'Tears of Steel', { cls: 'fui-poster--lg' })) },
  ],
};
