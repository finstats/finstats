// FinUI: tabs. Several views of one thing, one shown at a time under a row of tabs: real tab panels, so a screen reader
// hears "tab, 2 of 4" and the arrows move between them. Each panel is drawn when it is first shown.

import { h } from '../../core.js';
import { nextTab } from './plan.js';

let seq = 0;
/** tabs({ label, tabs: [{ key, label, count, disabled, panel }], value, onChange, fill }): `panel` a node or a function
 *  that draws it; `count` a number beside the name; `fill` stretches the tabs across. */
export function tabs({ label, tabs: list, value = null, onChange = () => {}, fill = false }) {
  const id = `fui-tabs-${++seq}`;
  let at = Math.max(0, list.findIndex((t) => t.key === value));
  if (list[at] && list[at].disabled) at = Math.max(0, list.findIndex((t) => !t.disabled));
  const off = list.map((t) => !!t.disabled);
  const buttons = list.map((t, i) => h('button', { type: 'button', class: 'fui-tabs__tab', role: 'tab', id: `${id}-tab-${i}`, 'aria-controls': `${id}-panel`, disabled: t.disabled || null, dataset: { key: t.key } },
    h('span', null, t.label), t.count != null ? h('span', { class: 'fui-tabs__count' }, String(t.count)) : null));
  const panel = h('div', { class: 'fui-tabs__panel', role: 'tabpanel', id: `${id}-panel`, tabindex: '0' });
  const drawn = new Map();
  const show = (i, focus = false) => {
    at = i;
    buttons.forEach((b, k) => { b.setAttribute('aria-selected', String(k === i)); b.tabIndex = k === i ? 0 : -1; });
    panel.setAttribute('aria-labelledby', `${id}-tab-${i}`);
    if (!drawn.has(i)) { const p = list[i].panel; drawn.set(i, typeof p === 'function' ? p() : p); }
    panel.replaceChildren(drawn.get(i) || '');
    if (focus) buttons[i].focus();
  };
  buttons.forEach((b, i) => b.addEventListener('click', () => { if (i !== at) { show(i); onChange(list[i].key); } }));
  const bar = h('div', { class: ['fui-tabs__list', fill && 'fui-tabs__list--fill'], role: 'tablist', 'aria-label': label }, buttons);
  bar.addEventListener('keydown', (e) => { const i = nextTab(at, e.key, off); if (i === null || i === at) return; e.preventDefault(); show(i, true); onChange(list[i].key); });
  show(at);
  return h('div', { class: 'fui-tabs' }, bar, panel);
}

const said = (t) => h('p', null, t);
export const meta = {
  name: 'tabs',
  purpose: 'Shows one of several views of the same thing at a time, under a row of tabs.',
  use: 'Parts of one page that are read separately: a title’s Overview, Plays and Files; a person’s Activity and Watchlist. count says how many each holds.',
  avoid: 'Switching how one view is drawn (segmented: Chart or Table). Steps to go through in order (steps). Navigating between pages (top-bar, sections).',
  variants: ['plain', 'with counts', 'one unavailable', 'stretched across'],
  states: ['selected', 'disabled', 'focus-visible'],
  a11y: 'role="tablist" of role="tab" buttons, each aria-selected and aria-controls its role="tabpanel"; the arrows move and select, Home and End go to the ends, an unavailable tab is stepped over; only the selected tab is in the tab order.',
  props: { 'tabs({ label, tabs, value, onChange, fill })': 'tabs: [{ key, label, count, disabled, panel }]' },
  playground: {
    controls: [
      { key: 'counts', label: 'Counts', on: true },
      { key: 'off', label: 'One unavailable' },
      { key: 'fill', label: 'Stretched across' },
    ],
    render: (o) => h('div', { class: 'fui-tabs__demo' }, tabs({ label: 'Big Buck Bunny', fill: o.fill, tabs: [
      { key: 'overview', label: 'Overview', panel: () => said('A short film about a large rabbit, 2008.') },
      { key: 'plays', label: 'Plays', count: o.counts ? 31 : null, panel: () => said('31 plays by 5 people.') },
      { key: 'files', label: 'Files', count: o.counts ? 2 : null, disabled: o.off, panel: () => said('1080p HEVC and 4K HEVC.') },
      { key: 'people', label: 'Cast & crew', count: o.counts ? 6 : null, panel: () => said('Directed by Sacha Goedegebure.') },
    ] })),
  },
};
