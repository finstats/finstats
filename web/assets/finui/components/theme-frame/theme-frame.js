// FinUI: theme-frame. What it holds, drawn in one theme whatever the page around it is in — light or dark — or at a
// phone's width: a gallery's example, a look being tried. light-dark() follows the colour scheme of the element that
// uses it, so a frame's scheme is all it takes.

import { h } from '../../core.js';
import { button } from '../button/button.js';

/** themeFrame({ scheme, phone, class }, ...children). `scheme`: 'light', 'dark', or null for the page's own. */
export function themeFrame({ scheme = null, phone = false, class: extra = null } = {}, ...children) {
  return h('div', { class: ['fui-theme-frame', scheme && `fui-theme-frame--${scheme}`, phone && 'fui-theme-frame--phone', extra] }, children);
}

export const meta = {
  name: 'theme-frame',
  purpose: 'Draws what it holds in one theme, whatever the page around it is in, or at a phone’s width.',
  use: 'Showing something in a theme the page is not in: examples side by side in light and dark, a look being tried, a screen at 360 px.',
  avoid: 'A card: a frame has no heading and holds nothing of the page’s own. A frame inside a frame.',
  variants: ['light', 'dark', 'the page’s theme', 'at a phone’s width'],
  states: [],
  a11y: 'A plain box: what it holds keeps its own roles. Colour scheme only, so contrast is the theme’s own.',
  props: { 'themeFrame({ scheme, phone, class }, ...children)': "scheme: 'light' | 'dark' | null (the page's); phone: 360 px wide" },
  playground: {
    controls: [
      { key: 'scheme', label: 'Theme', choices: [['dark', 'Dark'], ['light', 'Light'], ['page', 'The page’s']] },
      { key: 'phone', label: 'At a phone’s width' },
    ],
    render: (o) => themeFrame({ scheme: o.scheme === 'page' ? null : o.scheme, phone: o.phone },
      h('p', null, o.scheme === 'page' ? 'In the page’s own theme.' : `Always in the ${o.scheme} theme.`), button({ variant: 'primary' }, 'Read again')),
  },
};
