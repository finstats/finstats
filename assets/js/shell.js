// App chrome: the desktop and the mobile menu (FinUI's, each in the style this browser chose), Obsidian-style status bar,
// scroll-to-top, and the two layouts the router can ask for.

import { h, icon, num, relTime, dateTime, mount, logo } from './dom.js';
import { api, isAbort } from './api.js';
import { state, hasUnseenVersion, onVersionSeen, noteRunningVersion, can, themeChoice, setTheme, mobileNavChoice, onMobileNavChange, desktopNavChoice, setDesktopNav, onDesktopNavChange, searchMode } from './state.js';
import { hasPipeline } from './pages/pipeline.js';
import { navigate, onRouteChange, canGoBack } from './router.js';
import { avatar } from './components.js';
import { finstatsSearch } from './search.js';
import { grow, shrink } from '../finui/components/search/search.js';
import { themeSwitch as themeSwitchOf } from '../finui/components/theme-switch/theme-switch.js';
import { button } from '../finui/components/button/button.js';
import { mobileNav, STYLES, styleOf } from '../finui/components/mobile-nav/mobile-nav.js';
import { desktopNav, STYLES as DESKTOP_STYLES, styleOf as desktopStyleOf, sideOf } from '../finui/components/desktop-nav/desktop-nav.js';
import { cornerOf as desktopCornerOf } from '../finui/components/desktop-nav/plan.js';
import { cornerOf as mobileCornerOf } from '../finui/components/mobile-nav/plan.js';
import { toTop as toTopButton } from '../finui/components/to-top/to-top.js';

let shell = null; // {el, content, setActive, destroy, userId}
let bare = null;


/** The theme switch in the menus: FinUI's, given the choice this browser keeps. */
const themeSwitch = () => themeSwitchOf({ value: themeChoice(), onChange: setTheme });

function navItems() {
  const me = state.user;
  return [
    { href: '/', label: 'Dashboard', icon: 'home', group: 'You', exact: true, primary: true },
    { href: '/recap', label: 'Recap', icon: 'recap', group: 'You' },
    { href: `/users/${me.id}`, label: 'My profile', icon: 'user', group: 'You', primary: true },
    { href: '/activity', label: 'Activity', icon: 'activity', group: 'Watching', primary: true },
    { href: '/together', label: 'Together', icon: 'together', group: 'Watching' },
    can('see_everyone') ? { href: '/users', label: 'Users', icon: 'users', group: 'Library', not: `/users/${me.id}` } : null,
    { href: '/libraries', label: 'Libraries', icon: 'library', group: 'Library', also: ['/items'], primary: true },
    { href: '/playback', label: 'Playback', icon: 'sliders', group: 'Watching' },
    hasPipeline() ? { href: '/pipeline', label: 'Pipeline', icon: 'layers', group: 'Library' } : null,
    can('see_server') ? { href: '/server', label: 'Server', icon: 'server', group: 'Server' } : null,
    can('see_network') && can('see_everyone') ? { href: '/security', label: 'Security', icon: 'shield', group: 'Server' } : null,
    { href: '/settings', label: 'Settings', icon: 'settings', group: 'Server' },
    { href: '/changelog', label: 'Patch notes', icon: 'tag', group: 'Server', dot: hasUnseenVersion },
  ].filter(Boolean);
}

export function pageCommands() {
  return navItems().map((n) => ({ label: n.label, href: n.href, icon: n.icon }));
}

function buildShell() {
  const me = state.user;
  const items = navItems();
  // Who is signed in and the theme, at the foot of the mobile menu.
  const meFoot = () => [
    h('a', { class: 'me', href: `/users/${me.id}` }, avatar(me.id, me.name, { size: 26, hasImage: me.has_image }),
      h('span', { class: 'me-text' }, h('span', { class: 'me-name' }, me.name), h('span', { class: 'me-role' }, me.is_admin ? 'Administrator' : 'Viewer'))),
    themeSwitch()];
  // ---- the desktop menu: FinUI's desktop-nav in the style and on the side this browser chose (Settings → Appearance, on a
  // wide screen). The app lays its page beside whatever edge the style keeps (STYLES: a side, the bottom or the top), and the
  // dock sends the status bar to the top.
  const desktopSlot = h('div', { class: 'desktop-nav' });
  const corners = { desktop: null, mobile: null };   // what keeps the bottom-right corner, for to-top (below)
  let desktop = null, here = null;
  const pagesOf = () => items.map((n) => ({ key: n.href, href: n.href, label: n.label, icon: n.icon, group: n.group, primary: !!n.primary, dot: n.dot }));
  function buildDesktop() {
    if (desktop) desktop.destroy();
    const chosen = desktopNavChoice();
    const style = desktopStyleOf(chosen.style), side = sideOf(style, chosen.side);
    const { edge, size } = DESKTOP_STYLES.find((x) => x.key === style);
    const bar = parseFloat(getComputedStyle(document.documentElement).getPropertyValue('--statusbar-h')) || 26;
    desktop = desktopNav({ style, side, current: here, pages: pagesOf(), onSearch: () => openSearch(desktop.searchControl), keys: 'Ctrl Space',
      brand: { href: '/', mark: brandMark(), name: 'FinStats' },
      me: { href: `/users/${me.id}`, avatar: avatar(me.id, me.name, { size: 26, hasImage: me.has_image }), name: me.name, role: me.is_admin ? 'Administrator' : 'Viewer' },
      theme: themeSwitch(), pins: chosen.pins, onPins: (pins) => setDesktopNav({ pins }),
      insets: edge === 'bottom' ? { top: bar, bottom: 0 } : edge === 'top' ? { top: 0, bottom: bar } : { top: 0, bottom: bar } });
    mount(desktopSlot, desktop.el);
    for (const c of ['nav-left', 'nav-right', 'nav-dock', 'nav-command']) el.classList.remove(c);
    el.classList.add(side ? `nav-${side}` : edge === 'bottom' ? 'nav-dock' : 'nav-command');
    el.style.setProperty('--desktop-nav-size', `${size}px`);
    // The corner beside the menu: the dock leaves it free (the status bar went to the top), a side on the right keeps it.
    corners.desktop = [desktopCornerOf(style, side), edge === 'bottom' ? null : { bottom: bar }];
    placeToTop();
  }
  const stopDesktop = onDesktopNavChange(buildDesktop);

  // ---- the mobile menu: FinUI's mobile-nav in the style this browser chose (Settings → Appearance, on a phone). The styles
  // that open from a button in the top bar (Peek, Full screen) hand it over; the others bring their own at the bottom.
  const menuSlot = h('span', { class: 'topbar-menu' });
  const topbar = h('header', { class: 'topbar' }, menuSlot, h('a', { class: 'brand', href: '/' }, brandMark(), h('span', { class: 'brand-name' }, 'FinStats'), h('span', { class: 'sb-demo-mark', title: 'Everything here is invented. Nothing can be changed.' }, 'Demo')),
    button({ variant: 'icon', type: 'button', class: 'topbar-search', 'aria-label': 'Search', 'aria-keyshortcuts': 'Control+Space', onClick: (e) => openSearch(e.currentTarget) }, icon('search', 18)));
  const mobileSlot = h('div', { class: 'mobile-nav' });
  let mobile = null;
  function buildMobile() {
    if (mobile) mobile.destroy();
    const style = styleOf(mobileNavChoice());
    mobile = mobileNav({ style, current: here, onSearch: (from) => openSearch(from), onSearchEnd: () => dropSearch(), foot: h('div', { class: 'mobile-nav-foot' }, meFoot()),
      pages: pagesOf() });
    mount(menuSlot, mobile.trigger);
    mount(mobileSlot, mobile.el);
    const space = STYLES.find((x) => x.key === style).space;
    el.classList.toggle('has-mobile-dock', space > 0);
    el.style.setProperty('--mobile-nav-space', `${space}px`);
    // A menu that keeps the bottom takes the status bar's place; without one, the status bar is what is there.
    const bar = parseFloat(getComputedStyle(document.documentElement).getPropertyValue('--statusbar-h')) || 26;
    corners.mobile = [mobileCornerOf(style), space > 0 ? null : { bottom: bar }];
    placeToTop();
  }
  const stopMobile = onMobileNavChange(buildMobile);
  const stopDots = onVersionSeen(() => { if (mobile) mobile.repaint(); if (desktop) desktop.repaint(); });

  const content = h('div', { class: 'content', id: 'content' });
  const pageSearch = h('div', { class: 'page-search' });
  const main = h('main', { class: 'main', tabindex: -1 }, pageSearch, content);

  // ---- search: FinUI's, never a dialog. In the menu, the menu holds it (it grows out of the menu's own search into the
  // place the style has); in the page, it takes the page's place under the menu. The choice is this browser's (state.js).
  let searching = null;   // { panel, where: 'desktop' | 'mobile' | 'page', from, back }
  const wide = () => !matchMedia('(max-width: 820px)').matches;
  function openSearch(from) {
    if (searching) { searching.panel.focus(); return; }
    // What had the keyboard's focus (a control pressed, or the place Ctrl+Space was pressed in) is where Esc gives it back;
    // a click's focus is let go (desktop-nav's rule: handed back after a key, it is the keyboard's).
    const back = document.activeElement !== document.body && document.activeElement.matches(':focus-visible') ? document.activeElement : null;
    const panel = finstatsSearch({ onPick: (row) => { endSearch(false); navigate(row.href); }, onEscape: () => endSearch(true) });
    if (searchMode() === 'menu' && wide()) { searching = { panel, where: 'desktop', from }; desktop.openSearch(panel); return; }
    if (searchMode() === 'menu') { searching = { panel, where: 'mobile', from }; mobile.openSearch(panel, from); return; }
    searching = { panel, where: 'page', from, back };
    grow({ from, host: pageSearch, bar: panel.bar, list: panel.list, fresh: true, show: () => { pageSearch.replaceChildren(panel.el); el.classList.add('is-page-searching'); } });
    window.scrollTo(0, 0);
    panel.focus();
  }
  /** Close it, played back into what opened it; `restore` gives that the focus (Esc), a page chosen does not. */
  function endSearch(restore) {
    const s = searching;
    if (!s) return;
    searching = null;
    const done = s.where === 'desktop' ? desktop.closeSearch(restore) : s.where === 'mobile' ? mobile.closeSearch(restore)
      : shrink({ to: s.from, host: pageSearch, bar: s.panel.bar, list: s.panel.list, gone: true,
        show: () => el.classList.add('is-page-searching'), hide: () => el.classList.remove('is-page-searching') }).then(() => pageSearch.replaceChildren());
    done.then(() => {
      s.panel.destroy();
      // Back on what had the focus, quietly: a tooltip popping up as the search goes reads as a glitch (desktop-nav's own
      // rule). Not the menu's search for Ctrl+Space: a rail holds itself open while the focus is in it.
      const to = s.back;
      if (restore && s.where === 'page' && to && to.isConnected) {
        to.dataset.quiet = '';
        const loud = () => { delete to.dataset.quiet; to.removeEventListener('blur', loud); to.removeEventListener('pointerenter', loud); };
        to.addEventListener('blur', loud);
        to.addEventListener('pointerenter', loud);
        to.focus({ preventScroll: true });
      }
    });
  }
  /** The mobile menu let its search go by itself (its scrim, a handle). */
  function dropSearch() { if (searching) { searching.panel.destroy(); searching = null; } }
  // A press anywhere else gives up, as Esc does: the search is part of the page, not a dialog that holds it.
  const onPress = (e) => {
    if (!searching || searching.panel.el.contains(e.target) || e.target.closest('.fui-mobile-nav, .topbar-search, .fui-desktop-nav__search-btn, .fui-desktop-nav__trigger')) return;
    endSearch(false);
  };
  document.addEventListener('pointerdown', onPress, true);

  // ---- status bar
  const sbDot = h('span', { class: 'sb-dot' });
  const sbStream = h('a', { href: '/', class: 'sb-item sb-link' }, sbDot, h('span', null, 'connecting…'));
  const sbPlays = h('span', { class: 'sb-item' });
  const sbSync = h('span', { class: 'sb-item sb-sync' });
  // The demo says so on every page (demo.js answers the API from files taken from invented data).
  const sbDemo = h('span', { class: 'sb-item sb-right sb-demo', title: 'Everything here is invented: the people, the titles and the addresses. Nothing can be changed.' }, h('span', { class: 'sb-demo-mark' }, 'Demo'), 'invented data, read-only');
  const sbRepo = h('a', { class: 'sb-item sb-link', href: 'https://github.com/finstats/finstats', target: '_blank', rel: 'noopener noreferrer', title: 'FinStats on GitHub' }, icon('github', 12), 'Repo');
  const version = state.status && state.status.version;
  const sbVerText = h('span', null, version ? 'v' + version : '');
  const sbVer = h('a', { class: 'sb-item sb-link', href: '/changelog', title: 'Patch notes', hidden: !version }, sbVerText);
  const sep = () => h('span', { class: 'sb-sep', 'aria-hidden': 'true' }, '·');
  const statusbar = h('footer', { class: 'statusbar', role: 'status', 'aria-label': 'Collector status' }, sbStream, sep(), sbPlays, sep(), sbSync, sbDemo, sep(), sbRepo, sep(), sbVer);

  let sbTimer = null, sbAbort = null, dead = false;
  async function pollSummary() {
    if (dead) return;
    if (!document.hidden) {
      sbAbort = new AbortController();
      try {
        const sum = await api.get('/summary', null, { signal: sbAbort.signal });
        const ok = !!sum.collector_ok;
        sbDot.className = 'sb-dot ' + (ok ? 'ok' : 'bad');
        const n = sum.active_sessions || 0;
        sbStream.lastChild.textContent = ok ? `${num(n)} streaming` : 'collector offline';
        sbStream.title = !ok ? 'FinStats can’t reach Jellyfin right now. Plays are not being recorded.'
          : sum.collector_live ? 'Jellyfin is pushing what is playing · go to now playing' : 'Go to now playing';
        sbPlays.textContent = `${num(sum.plays_total)} plays`;
        sbSync.textContent = sum.last_sync_at ? `synced ${relTime(sum.last_sync_at)}` : 'not synced yet';
        sbSync.title = sum.last_sync_at ? 'Last library sync: ' + dateTime(sum.last_sync_at) : '';
        if (sum.version) { sbVerText.textContent = 'v' + sum.version; sbVer.hidden = false; noteRunningVersion(sum.version); }
      } catch (e) {
        if (!isAbort(e) && e.status !== 401) {
          sbDot.className = 'sb-dot bad';
          sbStream.lastChild.textContent = 'FinStats unreachable';
        }
      }
    }
    if (!dead) sbTimer = setTimeout(pollSummary, 10000);
  }
  pollSummary();

  // ---- scroll to top: FinUI's to-top, resting clear of whatever keeps the bottom-right corner in this layout (the menus'
  // corners and the status bar, told by buildDesktop and buildMobile), the phone's or the desktop's by the width.
  const toTop = toTopButton({ onTop: () => main.focus({ preventScroll: true }) });
  const narrow = matchMedia('(max-width: 820px)');
  const placeToTop = () => toTop.place((narrow.matches ? corners.mobile : corners.desktop) || []);
  narrow.addEventListener('change', placeToTop);

  const el = h('div', { class: 'app' }, topbar, desktopSlot, main, statusbar, toTop.el, mobileSlot);
  buildDesktop();
  buildMobile();

  return {
    /** Ctrl+Space: search, grown from the menu's own search on a wide screen, from the top bar's on a phone. */
    search() { openSearch(wide() ? desktop.searchControl : topbar.querySelector('.topbar-search')); },
    el, content, userId: me.id + ':' + me.is_admin + ':' + JSON.stringify(me.permissions || {}) + JSON.stringify(me.features || {}),
    setActive(path) {
      if (searching) endSearch(false);
      const open = items.find((n) => (n.exact ? path === n.href : n.not && (path === n.not || path.startsWith(n.not + '/')) ? false : path === n.href || path.startsWith(n.href + '/') || (n.also || []).some((p) => path.startsWith(p))));
      here = open ? open.href : null;
      desktop.setCurrent(here);
      mobile.close();
      mobile.setCurrent(here);
    },
    destroy() {
      dead = true;
      clearTimeout(sbTimer);
      if (sbAbort) sbAbort.abort();
      toTop.destroy();
      narrow.removeEventListener('change', placeToTop);
      document.removeEventListener('pointerdown', onPress, true);
      if (searching) { searching.panel.destroy(); searching = null; }
      stopMobile();
      stopDesktop();
      desktop.destroy();
      stopDots();
      mobile.destroy();
      el.remove();
    },
  };
}

function brandMark() {
  return logo(24);
}

/** Router hook: returns the element pages render into. */
export function layout(kind) {
  const appRoot = document.getElementById('app');
  if (kind === 'bare') {
    if (shell) { shell.destroy(); shell = null; }
    if (!bare) { bare = h('main', { class: 'bare' }); }
    if (!bare.isConnected) mount(appRoot, bare);
    return bare;
  }
  const key = state.user.id + ':' + state.user.is_admin + ':' + JSON.stringify(state.user.permissions || {}) + JSON.stringify(state.user.features || {});
  if (shell && shell.userId !== key) { shell.destroy(); shell = null; }
  if (!shell) {
    shell = buildShell();
    mount(appRoot, shell.el);
  }
  return shell.content;
}

onRouteChange((path) => { if (shell) shell.setActive(path); });

// ---- Esc steps back out of a detail page
// Overlays (modal, search, dropdown) swallow their own Esc before it gets here.
const appPath = () => window.finstatsDemo.app(location.pathname);
let herePath = appPath();
let cameFrom = null;
onRouteChange(() => {
  if (appPath() !== herePath) { cameFrom = herePath; herePath = appPath(); }
});

/** Where Esc leads from this page; null on top-level pages. `back` = prefer the real history entry. */
function escTarget(path) {
  if (/^\/libraries\/[^/]+/.test(path)) return { up: '/libraries' };
  if (/^\/users\/[^/]+\/(timeline|watchlist)$/.test(path)) return { up: path.replace(/\/(timeline|watchlist)$/, '') };
  if (/^\/users\/[^/]+/.test(path)) return can('see_everyone') && path !== `/users/${state.user.id}` ? { up: '/users' } : null; // your own profile is a top-level page
  if (/^\/items\/[^/]+/.test(path)) return { up: '/libraries', back: true };     // reached from anywhere, so return to wherever that was
  if (/^\/people\/[^/]+/.test(path)) return { up: '/libraries', back: true };
  if (/^\/settings\/tasks\/[^/]+/.test(path)) return { up: '/settings/tasks' };   // a job's schedule, back to the list
  return null;
}

document.addEventListener('keydown', (e) => {
  if (e.key !== 'Escape' || e.defaultPrevented || e.metaKey || e.ctrlKey || e.altKey || e.shiftKey || !state.user) return;
  // Esc belongs to whatever you are using: leave fields and keyboard-driven widgets (charts, rows) alone.
  const el = document.activeElement;
  if (el && el !== document.body && !el.matches('a, button, main')) { if (el.matches('input, textarea')) el.blur(); return; }
  const target = escTarget(appPath());
  if (!target) return;
  e.preventDefault();
  // Going back through history restores the list exactly as it was left (scroll position, filters).
  // Only when the entry behind this one is FinStats' own: a title opened in a new tab has none.
  if (canGoBack() && cameFrom !== null && (target.back || cameFrom === target.up)) history.back();
  else navigate(target.up);
});

// Global shortcut for search.
document.addEventListener('keydown', (e) => {
  // Ctrl+Space. `code` rather than `key`, so it is the same physical shortcut on every keyboard layout.
  if (e.ctrlKey && !e.metaKey && !e.altKey && !e.shiftKey && e.code === 'Space') {
    if (!state.user || !shell) return;
    e.preventDefault();
    shell.search();
  }
});
