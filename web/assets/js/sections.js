// A page that is one section on screen at a time: `/base/:section`, a sticky list of sections on the
// left (a row of chips on a phone), and the old one-page anchors forwarded to the section that holds
// them. Settings and Server are built on it. A section is `{ key, label, sub, group, icon, visible }`
// plus whatever the page renders it with; this module only decides which one is open and draws the list.

import { h, icon } from './dom.js';
import { navigate } from './router.js';

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

/** The section the address names, or null after forwarding: bare `/base` and an unknown key go to the
 *  first section the caller may see; an old anchor (`/base#jobs`) goes to the section `legacy` maps it to. */
export function pickSection(ctx, base, visible, legacy = {}) {
  const key = ctx.params.section;
  const section = key ? visible.find((s) => s.key === key) : null;
  if (section) return section;
  const anchor = location.hash.slice(1);
  const known = legacy[anchor] && visible.some((s) => s.key === legacy[anchor]) ? legacy[anchor] : null;
  const to = known || visible[0].key;
  navigate(`${base}/${to}${anchor && anchor !== to && known ? '#' + anchor : ''}`, { replace: true, scroll: false });
  return null;
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
  return h('nav', { class: 'section-nav', 'aria-label': label }, groups.map((g) => h('div', { class: 'section-group' },
    g.name ? h('p', { class: 'section-group-title' }, g.name) : null,
    g.items.map((s) => h('a', { class: ['section-link', s.key === current && 'is-active'], href: `${base}/${s.key}`, 'aria-current': s.key === current ? 'page' : null },
      icon(s.icon, 15), h('span', null, s.label))))));
}

/** The two-column layout: the list, and a slot for the open section. */
export function sectionLayout(nav, slot) {
  return h('div', { class: 'sections' }, nav, slot);
}
