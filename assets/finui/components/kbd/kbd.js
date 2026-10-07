// FinUI: kbd. A keyboard shortcut written as keys: Ctrl + K, each key in its own cap. On a Mac the modifier keys are
// written as a Mac's keyboard prints them.

import { h } from '../../core.js';

const MAC = { Ctrl: '⌘', Control: '⌘', Alt: '⌥', Option: '⌥', Shift: '⇧', Enter: '↩', Backspace: '⌫' };
/** kbd(shortcut, { mac }): "Ctrl+K" or ["Ctrl", "K"]. `mac` writes it a Mac's way; by default the browser's own. */
export function kbd(shortcut, { mac = typeof navigator !== 'undefined' && /Mac|iPhone|iPad/.test(navigator.platform || '') } = {}) {
  const keys = Array.isArray(shortcut) ? shortcut : String(shortcut).split('+').map((k) => k.trim()).filter(Boolean);
  return h('span', { class: 'fui-kbd' }, keys.map((k, i) => [i && !mac ? h('span', { class: 'fui-kbd__plus', 'aria-hidden': 'true' }, '+') : null, h('kbd', { class: 'fui-kbd__key' }, mac ? (MAC[k] || k) : k)]));
}

export const meta = {
  name: 'kbd',
  purpose: 'Writes a keyboard shortcut as the keys to press.',
  use: 'Beside what a shortcut does: in a menu item’s hint, in a tooltip, in help. A Mac’s modifier keys are written as its keyboard prints them.',
  avoid: 'A shortcut that only works in one browser. Saying a key that does nothing on the page.',
  variants: ['one key', 'a combination', 'on a Mac'],
  states: [],
  a11y: 'Each key is a <kbd>, read as the key’s name.',
  props: { 'kbd(shortcut, { mac })': 'shortcut: "Ctrl+K" or ["Ctrl", "K"]' },
  playground: {
    controls: [
      { key: 'keys', label: 'Shortcut', choices: [['Ctrl+Space', 'Search'], ['Shift+F10', 'A menu'], ['Esc', 'Back']] },
      { key: 'mac', label: 'On a Mac' },
    ],
    render: (o) => h('span', { class: 'fui-kbd__demo' }, kbd(o.keys, { mac: o.mac }), h('span', null, { 'Ctrl+Space': 'opens search', 'Shift+F10': 'opens the menu of what has the focus', Esc: 'steps back a page' }[o.keys])),
  },
};
