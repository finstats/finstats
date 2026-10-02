// FinUI: sections. A page that shows one of its sections at a time: a sticky list of them on the left — a row of chips
// on a phone — and the open one beside it. Which section is open is the app's to decide (it knows the address).

import { h, icon } from '../../core.js';

const HIT_MS = 2400;

/** Scroll a row (or card) into view and mark it for a moment, so the eye lands where the link pointed. */
export function reveal(id, focus = false) {
  const el = document.getElementById(id);
  if (!el) return false;
  el.scrollIntoView({ block: 'center' });
  el.classList.add('is-hit');
  setTimeout(() => el.classList.remove('is-hit'), HIT_MS);
  if (focus) { const c = el.querySelector('input, textarea, select, button, a[href]'); if (c) c.focus({ preventScroll: true }); }
  return true;
}

/** The sticky list: sections in their groups, the open one marked. */
export function sectionNav(base, visible, current, label = 'Sections') {
  const groups = [];
  for (const s of visible) {
    const name = s.group || '';
    let g = groups.find((x) => x.name === name);
    if (!g) groups.push(g = { name, items: [] });
    g.items.push(s);
  }
  return h('nav', { class: 'fui-sections__nav', 'aria-label': label }, groups.map((g) => h('div', { class: 'fui-sections__group' },
    g.name ? h('p', { class: 'fui-sections__group-title' }, g.name) : null,
    g.items.map((s) => h('a', { class: ['fui-sections__link', s.key === current && 'is-active'], href: `${base}/${s.key}`, 'aria-current': s.key === current ? 'page' : null },
      icon(s.icon, 15), h('span', null, s.label))))));
}

/** The two-column layout: the list, and a slot for the open section. */
export function sectionLayout(nav, slot) {
  return h('div', { class: 'fui-sections' }, nav, slot);
}

const DEMO = [{ key: 'overview', label: 'Overview', icon: 'server' }, { key: 'jobs', label: 'Jobs', icon: 'clock' }, { key: 'log', label: 'Log', icon: 'log', group: 'History' }];
export const meta = {
  name: 'sections',
  purpose: 'Shows one section of a long page at a time, with the list of all of them to move between.',
  use: 'A page of many independent parts (Settings, Server, the gallery). Each section has its own address, so a link lands on it.',
  avoid: 'Folding sections open and closed: the list navigates instead. Two levels of lists.',
  variants: ['groups with titles', 'a row of chips on a phone'],
  states: ['the open section (aria-current="page")', 'is-hit: a row a link pointed at, marked for a moment (reveal)'],
  a11y: 'A <nav> of links; the open one says aria-current="page". reveal() can move the focus to the row it marks.',
  props: { 'sectionNav(base, visible, current, label)': 'visible: [{ key, label, icon, group }]', 'sectionLayout(nav, slot)': 'the two columns', 'reveal(id, focus)': 'scroll to and mark an element' },
  playground: {
    controls: [
      { key: 'open', label: 'Open', choices: [['jobs', 'Jobs'], ['overview', 'Overview'], ['log', 'Log']] },
      { key: 'groups', label: 'In groups', on: true },
    ],
    render: (o) => sectionLayout(sectionNav('#', DEMO.map((d) => ({ ...d, group: o.groups ? d.group : null })), o.open, 'Example sections'),
      h('div', { class: 'fui-sections__body' }, h('p', null, { overview: 'How the server is doing.', jobs: 'What Jellyfin is doing, live.', log: 'What happened, newest first.' }[o.open]))),
  },
};
