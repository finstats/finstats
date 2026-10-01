// finui: button. One control in five looks — the default, primary for the one thing a view is for, ghost for the quiet
// ones, danger for what cannot be undone, and an icon alone — at two sizes. A link that looks like a button is still a
// link (`href`), and a file picker's label can look like one (`tag: 'label'`).

import { h, icon } from '../../core.js';

/**
 * button({ variant, size, tone, block, disabled, href, tag, ...props }, ...children)
 * Everything else in props is h()'s: type, onClick, aria-*, title, class (added to finui's own).
 */
export function button({ variant = 'default', size = 'md', tone = null, block = false, disabled = false, href = null, tag = null, class: extra = null, ...props } = {}, ...children) {
  const cls = ['fui-button', variant !== 'default' && `fui-button--${variant}`, size === 'sm' && 'fui-button--sm', tone === 'danger' && 'fui-button--danger-text',
    block && 'fui-button--block', disabled && tag === 'label' && 'fui-button--disabled', ...(Array.isArray(extra) ? extra : [extra])];
  if (href != null) return h('a', { ...props, href, class: cls }, children);
  if (tag === 'label') return h('label', { ...props, class: cls }, children);
  return h('button', { type: 'button', ...props, disabled, class: cls }, children);
}

const say = () => {};
export const meta = {
  name: 'button',
  purpose: 'A control that does something when pressed, or a link that looks like one.',
  use: 'Primary for the one action a view is for, and at most one of it. Default for the rest, ghost where buttons stand in a row or beside content, danger only for what cannot be undone. A link that leads somewhere is a link (href), even when it looks like a button.',
  avoid: 'A button that only navigates (use href). A row of primary buttons. An icon button without aria-label: the icon is hidden from screen readers, so the label is its only name.',
  variants: ['default', 'primary', 'ghost', 'danger', 'icon', 'size sm', 'tone danger', 'block'],
  states: ['hover', 'focus-visible', 'disabled (a native button)', 'aria-disabled (ghost and icon: shown, not pressable)', 'busy (setBusy)'],
  a11y: 'A native <button> (type="button" unless given) or <a>, so Enter and Space work and it is in the tab order. The focus ring is base.css’ :focus-visible. An icon button carries aria-label.',
  props: {
    variant: "'default' | 'primary' | 'ghost' | 'danger' | 'icon'",
    size: "'md' | 'sm'",
    tone: "'danger': red text on a default or ghost button",
    block: 'true: the whole width of its container',
    href: 'makes it a link',
    tag: "'label': a file picker's label (with disabled, it looks and acts disabled)",
    '...children': 'what it says: icon() and text, as h() takes them',
  },
  examples: [
    { name: 'Variants', render: () => h('div', { class: 'fui-button__demo' },
      button({ variant: 'primary', onClick: say }, icon('plus', 14), 'Make a key'),
      button({ onClick: say }, icon('refresh', 14), 'Read again'),
      button({ variant: 'ghost', onClick: say }, 'Cancel'),
      button({ variant: 'danger', onClick: say }, icon('trash', 14), 'Delete the backup'),
      button({ variant: 'icon', 'aria-label': 'Search', onClick: say }, icon('search', 18))) },
    { name: 'Small, and quiet in a row', render: () => h('div', { class: 'fui-button__demo' },
      button({ size: 'sm', onClick: say }, icon('check', 13), 'Resolve…'),
      button({ size: 'sm', variant: 'ghost', onClick: say }, icon('refresh', 13), 'Reopen'),
      button({ size: 'sm', variant: 'ghost', tone: 'danger', onClick: say }, 'Revoke'),
      button({ size: 'sm', variant: 'ghost', 'aria-disabled': 'true', 'aria-label': 'Previous page' }, icon('chevronLeft', 14))) },
    { name: 'A link that looks like a button', render: () => h('div', { class: 'fui-button__demo' },
      button({ href: '/finui/core' }, icon('layers', 14), 'Core', icon('chevronRight', 14)),
      button({ variant: 'primary', size: 'md', disabled: true }, 'Saving…')) },
    { name: 'The whole width', render: () => button({ variant: 'primary', block: true, onClick: say }, 'Sign in') },
  ],
};
