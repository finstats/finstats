// A page that is one section on screen at a time: `/base/:section`, a sticky list of sections on the
// left (a row of chips on a phone), and the old one-page anchors forwarded to the section that holds
// them. Settings and Server are built on it. A section is `{ key, label, sub, group, icon, visible }`
// plus whatever the page renders it with; this module only decides which one is open and draws the list.

import { navigate } from './router.js';
export { reveal, sectionNav, sectionLayout } from '../finui/components/sections/sections.js';

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

