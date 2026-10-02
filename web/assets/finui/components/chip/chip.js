// FinUI: chip. A small rounded thing that stands for one value: a genre, a filter in force, a choice that is on or off.
// Unlike a badge it can be pressed — to filter by it, to switch it, or to take it away.

import { h, icon } from '../../core.js';

/** chip(text, title): a value. The plain form stays as it was in components.js. */
export function chip(text, title) { return h('span', { class: 'fui-chip', title }, text); }

/** chipButton({ onClick, mono, title, class }, ...children): a chip that does something, like showing only its kind. */
export function chipButton({ onClick, mono = false, title = null, class: extra = null } = {}, ...children) {
  return h('button', { type: 'button', class: ['fui-chip', 'fui-chip--button', mono && 'mono', extra], title, onClick }, children);
}

/** chipToggle({ pressed, onChange, class }, ...children): a chip that is on or off. */
export function chipToggle({ pressed = false, onChange, class: extra = null } = {}, ...children) {
  const el = h('button', { type: 'button', class: ['fui-chip', 'fui-chip--toggle', extra], 'aria-pressed': String(pressed),
    onClick: () => { const on = el.getAttribute('aria-pressed') !== 'true'; el.setAttribute('aria-pressed', String(on)); if (onChange) onChange(on); } }, children);
  return el;
}

/** removableChip({ label, onRemove, removeLabel }): a filter in force, with its own way out. */
export function removableChip({ label, onRemove, removeLabel = 'Remove' }) {
  return h('span', { class: 'fui-chip fui-chip--removable' }, label, h('button', { type: 'button', class: 'fui-chip__x', 'aria-label': removeLabel, onClick: onRemove }, icon('x', 12)));
}

/** chipSet(...chips): a row that wraps. */
export const chipSet = (...chips) => h('div', { class: 'fui-chip__set' }, chips);

export const meta = {
  name: 'chip',
  purpose: 'One value in a small rounded frame: a genre, a filter in force, an option that is on or off.',
  use: 'Several short values in a row (chipSet). A filter that is applied, with its × to take it off. Options that switch independently (chipToggle, aria-pressed).',
  avoid: 'One of several mutually exclusive choices (segmented). A state the user cannot change (badge). Long sentences.',
  variants: ['plain', 'mono', 'button', 'toggle (pressed / not)', 'removable', 'group', 'in a set'],
  states: ['hover (button)', 'aria-pressed (toggle)', 'focus-visible'],
  a11y: 'Pressable chips are <button>s; a toggle says aria-pressed; the × of a removable chip has its own aria-label.',
  props: { 'chip(text, title)': 'a value', 'chipButton({ onClick, mono, title })': 'a value that acts', 'chipToggle({ pressed, onChange })': 'on or off', 'removableChip({ label, onRemove, removeLabel })': 'a filter in force', 'chipSet(...chips)': 'a wrapping row' },
  playground: {
    controls: [
      { key: 'kind', label: 'Kind', choices: [['value', 'Values'], ['button', 'One to press'], ['toggle', 'Toggles'], ['removable', 'A filter in force'], ['group', 'A group']] },
      { key: 'mono', label: 'Mono' },
    ],
    render: (o) => {
      const m = o.mono ? 'mono' : null;
      if (o.kind === 'button') return chipSet(chipButton({ title: 'Show only this kind', mono: o.mono, onClick: () => {} }, 'AuthenticationFailed'));
      if (o.kind === 'toggle') return chipSet(chipToggle({ pressed: true, class: m }, 'Films'), chipToggle({ pressed: false, class: m }, 'Shows'), chipToggle({ pressed: true, class: m }, 'Music'));
      if (o.kind === 'removable') return chipSet(removableChip({ label: o.mono ? 'Codec: HEVC' : 'Kind: Movies', onRemove: () => {}, removeLabel: 'Remove the filter' }));
      if (o.kind === 'group') return h('span', { class: ['fui-chip fui-chip--group', m] }, icon('users', 13), 'Watched together');
      return chipSet(...['Drama', 'Science Fiction', 'HEVC'].map((t) => h('span', { class: ['fui-chip', m] }, t)));
    },
  },
};
