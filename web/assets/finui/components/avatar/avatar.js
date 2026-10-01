// FinUI: avatar. A person's picture in a circle, or their initials. Given an address, not an id.

import { h, mount } from '../../core.js';
import { initials } from '../../format.js';

/** avatar(src, name, { size }): src may be null. */
export function avatar(src, name, { size = 28 } = {}) {
  const box = h('span', { class: 'fui-avatar', style: { width: size + 'px', height: size + 'px', fontSize: Math.max(10, size * 0.38) + 'px' }, 'aria-hidden': 'true' });
  const fallback = () => mount(box, initials(name));
  if (!src) { fallback(); return box; }
  box.append(h('img', { src, alt: '', loading: 'lazy', decoding: 'async', onError: fallback }));
  return box;
}

export const meta = {
  name: 'avatar',
  purpose: 'Shows who: a person’s picture in a circle, or their initials.',
  use: 'Beside a person’s name, in a list, a table or a header, at the size of the text around it.',
  avoid: 'An avatar as the only name of a person: it is aria-hidden, and the name is written beside it.',
  variants: ['any size', 'picture', 'initials'],
  states: ['failed → initials'],
  a11y: 'aria-hidden: decoration beside the name.',
  props: { 'avatar(src, name, { size })': 'src: an address or null' },
  examples: [
    { name: 'Initials at three sizes', render: () => h('div', { class: 'fui-avatar__demo' }, avatar(null, 'alice', { size: 20 }), avatar(null, 'bob builder', { size: 28 }), avatar(null, 'carol', { size: 44 })) },
  ],
};
