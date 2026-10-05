// FinUI: drawer. A panel that slides in from an edge over the page — the modal's dialog, held to a side: details of a
// row, a filter with many parts, a phone's menu. Esc, its ×, or a press on the page behind closes it.

import { h, icon } from '../../core.js';
import { button } from '../button/button.js';
import { openModal } from '../modal/modal.js';

let seq = 0;
/** openDrawer({ title, body, side, foot, onClose }) → the modal's { close, body, dialog }. `side`: 'right', 'left' or
 *  'bottom'; `foot` stays at the bottom while the body scrolls. */
export function openDrawer({ title, body, side = 'right', foot = null, onClose }) {
  return openModal({ title, onClose, labelId: `fui-drawer-${++seq}`, cls: `fui-drawer fui-drawer--${side}`,
    body: [h('div', { class: 'fui-drawer__body' }, body), foot ? h('div', { class: 'fui-drawer__foot' }, foot) : null] });
}

/** The panel as it looks open, held in a page (the gallery shows it this way). */
export function drawerPanel({ title, side = 'right', foot = null }, ...children) {
  return h('div', { class: ['fui-drawer', 'fui-drawer--held', `fui-drawer--${side}`], role: 'group', 'aria-label': title },
    h('div', { class: 'fui-drawer__head' }, h('h2', { class: 'fui-drawer__title' }, title), button({ variant: 'icon', 'aria-label': 'Close' }, icon('x', 16))),
    h('div', { class: 'fui-drawer__body' }, children), foot ? h('div', { class: 'fui-drawer__foot' }, foot) : null);
}

export const meta = {
  name: 'drawer',
  purpose: 'Slides a panel in from an edge over the page, for something that needs room but not a page of its own.',
  use: 'The details of a row without leaving the list, a filter of many parts, a menu on a phone (from the left or the bottom). foot holds its actions, in sight while the body scrolls.',
  avoid: 'A question that needs an answer before anything else (modal). Something people come back to (a page with an address).',
  variants: ['from the right', 'from the left', 'from the bottom', 'with actions at its foot'],
  states: ['open', 'closing'],
  a11y: 'The modal’s dialog: aria-modal, named by its title, Tab kept inside, Esc closes it and gives the focus back to what opened it.',
  props: {
    'openDrawer({ title, body, side, foot, onClose })': '→ { close, body, dialog }',
    'drawerPanel({ title, side, foot }, ...children)': 'the panel, held in a page',
  },
  playground: {
    controls: [
      { key: 'side', label: 'From', choices: [['right', 'The right'], ['left', 'The left'], ['bottom', 'The bottom']] },
      { key: 'foot', label: 'Actions at its foot', on: true },
    ],
    render: (o) => h('div', { class: ['fui-drawer__demo', `is-${o.side}`] },
      button({ size: 'sm', onClick: () => openDrawer({ title: 'Big Buck Bunny', side: o.side, body: h('p', null, '2008 · 10 minutes · seen by 5 people'), foot: o.foot ? button({ variant: 'primary' }, 'Open the title') : null }) }, 'Open it'),
      drawerPanel({ title: 'Big Buck Bunny', side: o.side, foot: o.foot ? [button({ variant: 'ghost' }, 'Close'), button({ variant: 'primary' }, 'Open the title')] : null },
        h('p', null, '2008 · 10 minutes'), h('p', { class: 'muted' }, 'Seen by 5 people, last on Friday.'))),
  },
};
