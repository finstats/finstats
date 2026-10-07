// FinUI: spinner. A small turning ring for something that will take a moment: a button that is busy, a page that boots.

import { h } from '../../core.js';

export function spinner(size = 14) { return h('span', { class: 'fui-spinner', style: { width: size + 'px', height: size + 'px' }, 'aria-hidden': 'true' }); }

export const meta = {
  name: 'spinner',
  purpose: 'Shows that something is happening and will finish soon.',
  use: 'Inside a busy button (setBusy puts it there) or where a whole view is starting. It takes the colour of the text around it.',
  avoid: 'Where the shape of what is coming is known (skeleton). Anything that takes longer than a few seconds without saying how far along it is.',
  variants: ['14 px (default)', 'any size'],
  states: ['turns slower with reduced motion'],
  a11y: 'aria-hidden: what is busy says so in words beside it ("Saving…").',
  props: { 'spinner(size = 14)': 'its width and height in pixels' },
  playground: {
    controls: [
      { key: 'size', label: 'Size', choices: [[14, 'Small'], [18, 'Medium'], [28, 'Large']] },
      { key: 'words', label: 'With words' },
    ],
    render: (o) => h('span', { class: 'fui-spinner__demo' }, spinner(o.size), o.words ? 'Saving…' : null),
  },
};
