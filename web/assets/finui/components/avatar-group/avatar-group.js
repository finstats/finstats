// FinUI: avatar-group. Several people as a row of overlapping faces — as many as fit, and how many more — named in
// words for those who cannot see them.

import { h } from '../../core.js';
import { avatar } from '../avatar/avatar.js';
import { fit } from './plan.js';

/** avatarGroup({ people: [{ name, src }], max, size }) */
export function avatarGroup({ people, max = 4, size = 28 }) {
  const { shown, more } = fit(people, max);
  const names = people.map((p) => p.name);
  const said = names.length <= 3 ? names.join(', ') : `${names.slice(0, 2).join(', ')} and ${names.length - 2} more`;
  return h('span', { class: 'fui-avatar-group', role: 'img', 'aria-label': said, title: names.join(', ') },
    shown.map((p) => avatar(p.src || null, p.name, { size })),
    more ? (() => { const m = h('span', { class: 'fui-avatar-group__more', 'aria-hidden': 'true' }, `+${more}`); Object.assign(m.style, { width: `${size}px`, height: `${size}px`, fontSize: `${Math.max(10, size * 0.36)}px` }); return m; })() : null);
}

const PEOPLE = ['alice', 'bob', 'carol', 'dave', 'erin', 'frank', 'grace'].map((name) => ({ name }));
export const meta = {
  name: 'avatar-group',
  purpose: 'Shows several people as a row of overlapping faces, as many as fit and how many more.',
  use: 'Who watched together, who follows a show, who is in a room. max sets the places; the last says +N when there are more.',
  avoid: 'A list to read names in (rank-list, a table). One person (avatar).',
  variants: ['a few', 'more than fit', 'small, large'],
  states: [],
  a11y: 'One image named by the people it shows ("alice, bob and 5 more"); the faces themselves are hidden from screen readers.',
  props: { 'avatarGroup({ people, max, size })': 'people: [{ name, src }]' },
  playground: {
    controls: [
      { key: 'many', label: 'More than fit', on: true },
      { key: 'large', label: 'Large' },
    ],
    render: (o) => h('span', { class: 'fui-avatar-group__demo' }, avatarGroup({ people: o.many ? PEOPLE : PEOPLE.slice(0, 3), max: 4, size: o.large ? 40 : 28 }), h('span', null, o.many ? 'watched together on Friday' : 'watch this')),
  },
};
