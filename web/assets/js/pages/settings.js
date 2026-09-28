// Settings: one section on screen at a time (the shell is sections.js), and a finder that knows
// every setting by name. Each section is a module under settings/ (`{ key, label, sub, group, icon,
// visible, entries, render }`); this page only decides which one is open and gives it a slot.
// `/settings` itself lands on the first section the caller may see; the anchors the old one-page
// layout had (`/settings#backups`) are forwarded to the section that holds them now.

import { h, icon, mount } from '../dom.js';
import { navigate } from '../router.js';
import { pageHeader, errorState } from '../components.js';
import { reveal, pickSection, sectionNav, sectionLayout } from '../sections.js';
import { createStore } from '../settings/common.js';
import jellyfin from '../settings/jellyfin.js';
import access from '../settings/access.js';
import collection from '../settings/collection.js';
import network from '../settings/network.js';
import security from '../settings/security.js';
import services from '../settings/services.js';
import backups from '../settings/backups.js';
import importSection from '../settings/import.js';
import system from '../settings/system.js';
import keys from '../settings/keys.js';
import publicProfile from '../settings/public.js';

const SECTIONS = [jellyfin, access, collection, network, security, ...services, backups, importSection, system, keys, publicProfile];
// Where the one-page anchors went, so links and bookmarks from before still land.
const LEGACY = { connections: 'connections', security: 'security', notifications: 'notifications', outbound: 'system', backups: 'backups', 'import-jellystat': 'import', 'import-streamystats': 'import' };
// ---------------------------------------------------------------- the finder
const norm = (s) => String(s || '').toLowerCase().normalize('NFD').replace(/[\u0300-\u036f]/g, '').replace(/[’']/g, '');
const words = (s) => norm(s).split(/[^a-z0-9]+/).filter(Boolean);

/** Every typed word must begin some word of the entry; a match on the label counts for more than one on a hint. */
function score(q, e) {
  let total = 0;
  for (const w of q) {
    let best = 0;
    if (e.labelWords.some((x) => x.startsWith(w))) best = 2;
    else if (e.hintWords.some((x) => x.startsWith(w))) best = 1;
    if (!best) return 0;
    total += best;
  }
  return total;
}

function finder(visible, current, ctx) {
  const index = visible.flatMap((s) => [
    { section: s, id: null, label: s.label, where: s.sub, labelWords: words(s.label), hintWords: words(s.sub) },
    ...(s.entries || []).map((e) => ({ section: s, id: e.id, label: e.label, where: s.label, labelWords: words(e.label), hintWords: words(`${e.hint || ''} ${s.label}`) })),
  ]);
  const input = h('input', { class: 'input input-search settings-find', type: 'search', placeholder: 'Find a setting…', 'aria-label': 'Find a setting', autocomplete: 'off',
    role: 'combobox', 'aria-expanded': 'false', 'aria-controls': 'settings-hits', 'aria-autocomplete': 'list', 'aria-keyshortcuts': '/' });
  const list = h('ul', { class: 'settings-hits', id: 'settings-hits', role: 'listbox', 'aria-label': 'Matching settings', hidden: true });
  const wrap = h('div', { class: 'search-field settings-finder' }, icon('search', 14), input, h('kbd', { class: 'settings-find-key', 'aria-hidden': 'true' }, '/'), list);
  let hits = [], sel = -1;

  const go = (hit) => {
    close(); input.value = '';
    const url = `/settings/${hit.section.key}${hit.id ? '#' + hit.id : ''}`;
    if (hit.section.key === current) { history.pushState({ depth: ((history.state && history.state.depth) || 0) + 1 }, '', url); if (hit.id) reveal(hit.id, true); else window.scrollTo(0, 0); }
    else navigate(url);
  };
  function paint() {
    input.setAttribute('aria-expanded', String(hits.length > 0));
    list.hidden = !hits.length;
    mount(list, hits.map((e, i) => {
      const li = h('li', { class: 'settings-hit', role: 'option', id: `settings-hit-${i}`, 'aria-selected': String(i === sel) },
        h('span', { class: 'settings-hit-label' }, e.label), h('span', { class: 'settings-hit-where' }, e.where));
      li.addEventListener('mousedown', (ev) => ev.preventDefault()); // keep the focus in the field
      li.addEventListener('click', () => go(e));
      return li;
    }));
    if (sel >= 0) input.setAttribute('aria-activedescendant', `settings-hit-${sel}`); else input.removeAttribute('aria-activedescendant');
  }
  function close() { hits = []; sel = -1; paint(); }
  input.addEventListener('input', () => {
    const q = words(input.value);
    if (!q.length) { close(); return; }
    hits = index.map((e) => [score(q, e), e]).filter(([sc]) => sc > 0).sort((a, b) => b[0] - a[0]).slice(0, 8).map(([, e]) => e);
    sel = hits.length ? 0 : -1;
    paint();
  });
  input.addEventListener('keydown', (e) => {
    if (e.key === 'ArrowDown' && hits.length) { e.preventDefault(); sel = (sel + 1) % hits.length; paint(); }
    else if (e.key === 'ArrowUp' && hits.length) { e.preventDefault(); sel = (sel - 1 + hits.length) % hits.length; paint(); }
    else if (e.key === 'Enter' && hits.length) { e.preventDefault(); go(hits[Math.max(0, sel)]); }
    else if (e.key === 'Escape') { if (input.value || hits.length) { e.stopPropagation(); input.value = ''; close(); } else input.blur(); }
  });
  input.addEventListener('blur', () => setTimeout(close, 120));
  // "/" from anywhere on the page goes to the finder, the way it does on GitHub.
  const onKey = (e) => {
    if (e.key !== '/' || e.defaultPrevented || e.ctrlKey || e.metaKey || e.altKey) return;
    const t = e.target;
    if (t && (t.matches('input, textarea, select, [contenteditable]') || t.isContentEditable)) return;
    e.preventDefault(); input.focus(); input.select();
  };
  document.addEventListener('keydown', onKey);
  ctx.onCleanup(() => document.removeEventListener('keydown', onKey));
  return wrap;
}

// ---------------------------------------------------------------- the page
export default function settings(ctx) {
  const visible = SECTIONS.filter((s) => s.visible());
  const section = pickSection(ctx, '/settings', visible, LEGACY);
  if (!section) return;
  ctx.title(`${section.label} · Settings`);
  const store = createStore(ctx);
  const slot = h('div', { class: 'section-body stack' });
  ctx.root.append(pageHeader('Settings', section.sub, finder(visible, section.key, ctx)),
    sectionLayout(sectionNav('/settings', visible, section.key, 'Settings sections'), slot));
  section.render(slot, store)
    .then(() => { const id = location.hash.slice(1); if (id && !ctx.signal.aborted) reveal(id); })
    .catch((e) => { if (!ctx.signal.aborted && e.name !== 'AbortError' && e.status !== 401) mount(slot, errorState(e, () => section.render(slot, store))); });
}
