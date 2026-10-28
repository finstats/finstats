// Simple mode's shell: FinUI's top bar, solid, across the top of every page. FinStats' name and mark on the left; on the
// right the search, the sections (Home as an icon) and a gear for settings, the patch notes, the theme and signing out. Below 820 px the
// sections fold into one menu button. Esc steps back out of a detail page, as it does in the FinStats layout.

import { h, icon, logo, mount, debounce } from '../dom.js';
import { api, imgItem, isAbort } from '../api.js';
import { t } from '../i18n.js';
import { state, can, resetCaches, themeChoice, setTheme } from '../state.js';
import { onRouteChange, navigate, canGoBack } from '../router.js';
import { topBar } from '../../finui/components/top-bar/top-bar.js';
import { dropdownMenu } from '../../finui/components/dropdown-menu/dropdown-menu.js';
import { button } from '../../finui/components/button/button.js';
import { poster } from '../../finui/components/poster/poster.js';

/** The sections the bar names, for this person: Users only for whoever may see everyone. */
export const sections = () => [
  { href: '/', key: 'home', label: icon('home', 16), name: t('simple.nav.home') },
  { href: '/libraries', key: 'libraries', label: t('simple.nav.libraries') },
  can('see_everyone') ? { href: '/users', key: 'users', label: t('simple.nav.users') } : null,
  { href: '/history', key: 'history', label: t('simple.nav.history') },
  { href: '/graphs', key: 'graphs', label: t('simple.nav.graphs') },
].filter(Boolean);

/** Which section a page belongs to: a title is part of its library's, a person of Users. */
function sectionOf(path) {
  if (path === '/') return 'home';
  const top = path.split('/')[1];
  if (top === 'items') return 'libraries';
  return ['libraries', 'users', 'history', 'graphs'].includes(top) ? top : null;
}

async function signOut() {
  try { await api.post('/auth/logout', {}, { quiet401: true }); } catch { /* signing out anyway */ }
  state.user = null;
  resetCaches();
  navigate('/login');
}

const THEME_LABEL = { device: t('theme.is_device'), light: t('theme.is_light'), dark: t('theme.is_dark') };

function gear() {
  const btn = dropdownMenu({ label: t('simple.menu'), icon: 'settings', iconOnly: true, items: () => [
    { label: t('nav.my_profile'), icon: 'user', href: `/users/${state.user.id}` },
    { label: t('nav.settings'), icon: 'settings', href: '/settings' },
    { label: t('nav.patch_notes'), icon: 'tag', href: '/changelog' },
    { separator: true },
    ...['device', 'light', 'dark'].map((th) => ({ label: THEME_LABEL[th], icon: th === 'light' ? 'sun' : th === 'dark' ? 'moon' : 'monitor', hint: themeChoice() === th ? '✓' : null, onSelect: () => setTheme(th) })),
    { separator: true },
    { label: t('auth.sign_out'), icon: 'logout', onSelect: signOut },
  ] });
  btn.classList.add('simple-gear');
  return btn;
}

function menuButton() {
  const btn = dropdownMenu({ label: t('simple.sections'), icon: 'menu', iconOnly: true,
    items: () => sections().map((s) => ({ label: s.name || s.label, href: s.href, icon: s.key === 'home' ? 'home' : null })) });
  btn.classList.add('simple-menu-button');
  return btn;
}

/** The search: a magnifier that opens a field in its place, with titles and people as they are typed. */
function search(signal) {
  const box = h('div', { class: 'simple-search' });
  const results = h('div', { class: 'simple-search__results', role: 'listbox', 'aria-label': t('search.results') });
  const field = h('input', { class: 'simple-search__field', type: 'search', placeholder: t('search.placeholder_simple'), 'aria-label': t('search.label'), autocomplete: 'off' });
  const open = button({ variant: 'icon', 'aria-label': t('search.label'), title: t('search.label'), class: 'simple-search__open' }, icon('search', 16));
  let asked = null;
  const close = () => { if (asked) asked.abort(); mount(box, open); results.replaceChildren(); field.value = ''; };
  const ask = debounce(async () => {
    const q = field.value.trim();
    if (asked) asked.abort();
    if (!q) { results.replaceChildren(); return; }
    asked = new AbortController();
    try {
      const d = await api.get('/search', { q, limit: 8 }, { signal: asked.signal });
      const rows = [
        ...(d.items || []).map((it) => h('a', { class: 'simple-search__row', href: `/items/${it.id}`, role: 'option' },
          poster(imgItem(it.image_item_id || it.id, 60), it.name), h('span', null, it.name), h('span', { class: 'simple-search__kind' }, it.year || it.type))),
        ...(can('see_everyone') ? d.users || [] : []).map((u) => h('a', { class: 'simple-search__row', href: `/users/${u.id}`, role: 'option' },
          icon('user', 16), h('span', null, u.name), h('span', { class: 'simple-search__kind' }, t('search.user')))),
      ];
      // Where the results hang on a phone, where they take the screen's width (simple.css): just under the field.
      results.style.setProperty('--simple-search-top', `${Math.round(field.getBoundingClientRect().bottom) + 8}px`);
      results.replaceChildren(...(rows.length ? rows : [h('div', { class: 'simple-search__none' }, t('search.no_match_short'))]));
    } catch (e) { if (!isAbort(e)) results.replaceChildren(h('div', { class: 'simple-search__none' }, e.message)); }
  }, 150);
  field.addEventListener('input', ask);
  field.addEventListener('keydown', (e) => {
    if (e.key === 'Escape') { e.preventDefault(); e.stopPropagation(); close(); open.focus(); }
    if (e.key === 'ArrowDown') { const first = results.querySelector('a'); if (first) { e.preventDefault(); first.focus(); } }
  });
  results.addEventListener('keydown', (e) => {
    const rows = [...results.querySelectorAll('a')], i = rows.indexOf(document.activeElement);
    if (e.key === 'ArrowDown' && i < rows.length - 1) { e.preventDefault(); rows[i + 1].focus(); }
    if (e.key === 'ArrowUp') { e.preventDefault(); (i > 0 ? rows[i - 1] : field).focus(); }
    if (e.key === 'Escape') { e.preventDefault(); e.stopPropagation(); close(); open.focus(); }
  });
  results.addEventListener('click', (e) => { if (e.target.closest('a')) setTimeout(close); });
  // Safari focuses no link it presses: the field lost the focus to nothing, the list closed, and the click found no link.
  results.addEventListener('mousedown', (e) => { if (e.target.closest('a')) e.preventDefault(); });
  box.addEventListener('focusout', (e) => { if (!box.contains(e.relatedTarget) && !results.contains(e.relatedTarget)) setTimeout(() => { if (!box.contains(document.activeElement)) close(); }, 0); }, { signal });
  open.addEventListener('click', () => { mount(box, [field, results]); field.focus(); });
  mount(box, open);
  return box;
}

let shell = null;
let bare = null;

function buildShell(key) {
  const life = new AbortController();
  const bar = topBar({ variant: 'solid', label: 'FinStats', brand: { href: '/', mark: logo(26), name: h('span', null, 'FinStats', ' ', h('span', { class: 'sb-demo-mark', title: 'Everything here is invented: the people, the titles and the addresses. Nothing can be changed.' }, 'Demo')) },
    lead: search(life.signal), links: sections(), actions: [menuButton(), gear()], current: sectionOf(location.pathname) });
  const content = h('main', { class: 'simple-main', id: 'main' });
  const el = h('div', { class: 'simple-app' }, bar, content);
  return { el, bar, content, key, destroy: () => life.abort() };
}

/** The router's layout: `bare` for the sign-in pages, otherwise the shell, built again when who is signed in changes. */
export function layout(kind) {
  const appRoot = document.getElementById('app');
  if (kind === 'bare') {
    if (shell) { shell.destroy(); shell = null; }
    if (!bare) bare = h('main', { class: 'bare' });
    if (!bare.isConnected) mount(appRoot, bare);
    return bare;
  }
  const key = state.user.id + ':' + state.user.is_admin + ':' + JSON.stringify(state.user.permissions || {});
  if (shell && shell.key !== key) { shell.destroy(); shell = null; }
  if (!shell) { shell = buildShell(key); mount(appRoot, shell.el); }
  return shell.content;
}

onRouteChange((path) => { if (shell) shell.bar.setCurrent(sectionOf(path)); });

// ---- Esc steps back out of a detail page, the FinStats layout's rule for the pages this one has.
let herePath = location.pathname;
let cameFrom = null;
onRouteChange(() => { if (location.pathname !== herePath) { cameFrom = herePath; herePath = location.pathname; } });

function escTarget(path) {
  if (/^\/libraries\/[^/]+/.test(path)) return { up: '/libraries' };
  if (/^\/users\/[^/]+/.test(path)) return can('see_everyone') ? { up: '/users' } : null;
  if (/^\/items\/[^/]+/.test(path)) return { up: '/libraries', back: true };
  if (/^\/settings\/tasks\/[^/]+/.test(path)) return { up: '/settings/tasks' };
  return null;
}

document.addEventListener('keydown', (e) => {
  if (e.key !== 'Escape' || e.defaultPrevented || e.metaKey || e.ctrlKey || e.altKey || e.shiftKey || !state.user) return;
  const el = document.activeElement;
  if (el && el !== document.body && !el.matches('a, button, main')) { if (el.matches('input, textarea')) el.blur(); return; }
  const target = escTarget(location.pathname);
  if (!target) return;
  e.preventDefault();
  if (canGoBack() && cameFrom !== null && (target.back || cameFrom === target.up)) history.back();
  else navigate(target.up);
});
