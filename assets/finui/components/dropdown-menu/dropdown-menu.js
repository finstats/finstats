// FinUI: dropdown-menu. A button that opens a menu of actions under it (the context menu's, with its keys, its
// type-ahead and its way back), named by what it is for and marked open while it is.

import { h, icon } from '../../core.js';
import { button } from '../button/button.js';
import { openMenu } from '../context-menu/context-menu.js';

/** dropdownMenu({ label, items, icon, iconOnly, variant, size }) → the button. `items` as openMenu takes them, or a
 *  function that answers them when it opens. */
export function dropdownMenu({ label, items, icon: iconName = null, iconOnly = false, variant = 'default', size = null }) {
  const btn = button({ variant: iconOnly ? 'icon' : variant, size, 'aria-haspopup': 'menu', 'aria-expanded': 'false', 'aria-label': iconOnly ? label : null, class: 'fui-dropdown-menu' },
    iconName ? icon(iconName, 14) : null, iconOnly ? null : h('span', null, label), iconOnly ? null : icon('chevronDown', 13, 'fui-dropdown-menu__caret'));
  const open = () => {
    const r = btn.getBoundingClientRect();
    btn.setAttribute('aria-expanded', 'true');
    openMenu({ items: typeof items === 'function' ? items() : items, at: { x: Math.round(r.left), y: Math.round(r.bottom + 4) }, from: btn, label, onClose: () => btn.setAttribute('aria-expanded', 'false') });
  };
  btn.addEventListener('click', () => (btn.getAttribute('aria-expanded') === 'true' ? null : open()));
  btn.addEventListener('keydown', (e) => { if (e.key === 'ArrowDown') { e.preventDefault(); open(); } });
  return btn;
}

const ITEMS = () => [{ label: 'Rename', icon: 'log' }, { label: 'Copy link', icon: 'link' }, { separator: true }, { label: 'Remove', icon: 'trash', danger: true }];
export const meta = {
  name: 'dropdown-menu',
  purpose: 'Opens a menu of actions under the button that names them.',
  use: 'Actions on one thing that do not each earn a button: a row’s “More”, a sort order, an export. It is the context menu’s menu, opened by a button: the same keys, type-ahead and way back.',
  avoid: 'Choosing a value that stays shown (picker, segmented, combobox). Navigating a site (top-bar, sections).',
  variants: ['a button with words', 'an icon alone', 'primary', 'small'],
  states: ['open (aria-expanded)', 'closed'],
  a11y: 'A button with aria-haspopup="menu" and aria-expanded; Arrow Down, Enter or Space opens the menu with its first item focused; Esc and Tab close it and give the focus back.',
  props: { 'dropdownMenu({ label, items, icon, iconOnly, variant, size })': 'items: as openMenu takes them, or a function' },
  playground: {
    controls: [
      { key: 'iconOnly', label: 'An icon alone' },
      { key: 'primary', label: 'Primary' },
      { key: 'small', label: 'Small' },
    ],
    render: (o) => dropdownMenu({ label: o.iconOnly ? 'More' : 'Actions', icon: o.iconOnly ? 'menu' : null, iconOnly: o.iconOnly, variant: o.primary ? 'primary' : 'default', size: o.small ? 'sm' : null, items: ITEMS }),
  },
};
