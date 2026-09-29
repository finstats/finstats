// App chrome: file-tree style sidebar, Obsidian-style status bar, mobile drawer,
// scroll-to-top, and the two layouts the router can ask for.

import { h, icon, num, relTime, dateTime, mount, logo } from './dom.js';
import { api, isAbort } from './api.js';
import { state, resetCaches, hasUnseenVersion, onVersionSeen, noteRunningVersion, can, themeChoice, setTheme } from './state.js';
import { hasPipeline } from './pages/pipeline.js';
import { navigate, onRouteChange, canGoBack } from './router.js';
import { avatar } from './components.js';
import { openPalette } from './palette.js';

let shell = null; // {el, content, navLinks, destroy, userId}
let bare = null;

const THEME_NAME = { device: 'Device', light: 'Light', dark: 'Dark' };
const THEME_ICON = { device: 'monitor', light: 'sun', dark: 'moon' };
// The stops of the theme switch, left to right: Device sits in the middle, the one that is neither.
const THEME_STOPS = ['light', 'device', 'dark'];

/**
 * The theme as a switch with three stops, like the two-stop ones in Settings: a click on a third of it, a drag of the
 * knob or the arrow keys pick one. The knob carries the icon of what is chosen, and a new icon moves in when it
 * changes (CSS, and not at all with reduced motion). A slider to assistive technology: "Theme: Dark".
 */
function themeSwitch() {
  const knob = h('span', { class: 'theme-knob' });
  const el = h('div', { class: 'theme-switch', role: 'slider', tabindex: '0', 'aria-label': 'Theme', 'aria-valuemin': '0', 'aria-valuemax': '2', 'aria-orientation': 'horizontal' },
    h('span', { class: 'theme-marks', 'aria-hidden': 'true' }, THEME_STOPS.map(() => h('span', { class: 'theme-mark' }))), knob);
  let at = THEME_STOPS.indexOf(themeChoice());
  function paint(animate) {
    const t = THEME_STOPS[at];
    el.dataset.at = String(at);
    el.style.setProperty('--at', String(at));
    el.setAttribute('aria-valuenow', String(at));
    el.setAttribute('aria-valuetext', THEME_NAME[t]);
    el.title = `Theme: ${THEME_NAME[t]}`;
    if (knob.dataset.icon !== THEME_ICON[t]) {
      knob.dataset.icon = THEME_ICON[t];
      mount(knob, icon(THEME_ICON[t], 13, animate ? 'theme-icon is-in' : 'theme-icon'));
    }
  }
  function choose(next) {
    next = Math.max(0, Math.min(2, next));
    knob.style.translate = '';
    if (next === at) return;
    at = next;
    setTheme(THEME_STOPS[at]);
    paint(true);
  }
  // Where on the track a pointer is, as a stop: 0, 1 or 2.
  const stopAt = (clientX) => { const r = el.getBoundingClientRect(); return Math.max(0, Math.min(2, Math.floor(((clientX - r.left) / r.width) * 3))); };
  let drag = null;
  el.addEventListener('pointerdown', (e) => {
    if (e.button !== 0) return;
    const r = el.getBoundingClientRect(), k = knob.getBoundingClientRect();
    // Where the knob sits now (the track's border and the knob's inset are a pixel each), and how far it can go.
    drag = { x: e.clientX, from: k.left - r.left - 2, max: r.width - k.width - 4, moved: false };
    el.setPointerCapture(e.pointerId);
  });
  el.addEventListener('pointermove', (e) => {
    if (!drag) return;
    const dx = e.clientX - drag.x;
    if (!drag.moved && Math.abs(dx) < 3) return;
    drag.moved = true;
    el.classList.add('is-dragging');
    // The knob follows the pointer between the two ends; it lands on the nearest stop when let go.
    knob.style.translate = `${Math.max(0, Math.min(drag.max, drag.from + dx)).toFixed(1)}px 0`;
  });
  const letGo = (e) => {
    if (!drag) return;
    const d = drag; drag = null;
    el.classList.remove('is-dragging');
    if (!d.moved) return choose(stopAt(e.clientX));
    const k = knob.getBoundingClientRect();
    choose(stopAt(k.left + k.width / 2));
    knob.style.translate = '';
  };
  el.addEventListener('pointerup', letGo);
  el.addEventListener('pointercancel', () => { drag = null; el.classList.remove('is-dragging'); knob.style.translate = ''; });
  el.addEventListener('keydown', (e) => {
    const step = { ArrowLeft: -1, ArrowDown: -1, ArrowRight: 1, ArrowUp: 1 }[e.key];
    if (step) { e.preventDefault(); choose(at + step); }
    else if (e.key === 'Home') { e.preventDefault(); choose(0); }
    else if (e.key === 'End') { e.preventDefault(); choose(2); }
  });
  paint(false);
  return el;
}



function navItems() {
  const me = state.user;
  return [
    { href: '/', label: 'Dashboard', icon: 'home', exact: true },
    { href: '/recap', label: 'Recap', icon: 'recap' },
    { href: `/users/${me.id}`, label: 'My profile', icon: 'user' },
    { href: '/activity', label: 'Activity', icon: 'activity' },
    { href: '/together', label: 'Together', icon: 'together' },
    can('see_everyone') ? { href: '/users', label: 'Users', icon: 'users', not: `/users/${me.id}` } : null,
    { href: '/libraries', label: 'Libraries', icon: 'library', also: ['/items'] },
    { href: '/playback', label: 'Playback', icon: 'sliders' },
    hasPipeline() ? { href: '/pipeline', label: 'Pipeline', icon: 'layers' } : null,
    can('see_server') ? { href: '/server', label: 'Server', icon: 'server' } : null,
    can('see_network') && can('see_everyone') ? { href: '/security', label: 'Security', icon: 'shield' } : null,
    { href: '/settings', label: 'Settings', icon: 'settings' },
    { href: '/changelog', label: 'Patch notes', icon: 'tag', dot: hasUnseenVersion },
  ].filter(Boolean);
}

export function pageCommands() {
  return navItems().map((n) => ({ label: n.label, href: n.href, icon: n.icon }));
}

async function signOut(btn) {
  btn.disabled = true;
  try { await api.post('/auth/logout', {}, { quiet401: true }); } catch { /* signing out anyway */ }
  state.user = null;
  resetCaches();
  navigate('/login');
}

function buildShell() {
  const me = state.user;
  const items = navItems();
  const navLinks = items.map((n) => {
    const a = h('a', { class: 'nav-row', href: n.href }, icon(n.icon, 15), h('span', null, n.label));
    if (n.dot) {
      // "Something new here" — announced in text too, never by the dot alone.
      const dot = h('span', { class: 'nav-dot' }, h('span', { class: 'sr-only' }, 'New version'));
      const paint = () => { dot.hidden = !n.dot(); };
      paint();
      onVersionSeen(paint);
      a.append(dot);
    }
    a._item = n;
    return a;
  });

  const searchBtn = h('button', { type: 'button', class: 'search-btn', onClick: () => openPalette(), 'aria-keyshortcuts': 'Control+Space' },
    icon('search', 14), h('span', null, 'Search…'), h('kbd', null, 'Ctrl Space'));

  const logout = h('button', { type: 'button', class: 'icon-btn', 'aria-label': 'Sign out', title: 'Sign out' }, icon('logout', 15));
  logout.addEventListener('click', () => signOut(logout));

  const sidebar = h('aside', { class: 'sidebar', id: 'sidebar', 'aria-label': 'Main' },
    h('a', { class: 'brand', href: '/' }, brandMark(), h('span', { class: 'brand-name' }, 'finstats')),
    searchBtn,
    h('nav', { class: 'nav', 'aria-label': 'Pages' }, navLinks),
    h('div', { class: 'sidebar-foot' },
      h('a', { class: 'me', href: `/users/${me.id}` }, avatar(me.id, me.name, { size: 26, hasImage: me.has_image }),
        h('span', { class: 'me-text' }, h('span', { class: 'me-name' }, me.name), h('span', { class: 'me-role' }, me.is_admin ? 'Administrator' : 'Viewer'))),
      themeSwitch(), logout));

  const menuBtn = h('button', { type: 'button', class: 'icon-btn', 'aria-label': 'Open menu', 'aria-controls': 'sidebar', 'aria-expanded': 'false' }, icon('menu', 18));
  const topbar = h('header', { class: 'topbar' }, menuBtn, h('a', { class: 'brand', href: '/' }, brandMark(), h('span', { class: 'brand-name' }, 'finstats')),
    h('button', { type: 'button', class: 'icon-btn', 'aria-label': 'Search', onClick: () => openPalette() }, icon('search', 18)));
  const scrim = h('div', { class: 'scrim', hidden: true });
  const setDrawer = (open) => {
    el.classList.toggle('drawer-open', open);
    scrim.hidden = !open;
    menuBtn.setAttribute('aria-expanded', String(open));
  };
  menuBtn.addEventListener('click', () => setDrawer(!el.classList.contains('drawer-open')));
  scrim.addEventListener('click', () => setDrawer(false));

  const content = h('div', { class: 'content', id: 'content' });
  const main = h('main', { class: 'main', tabindex: -1 }, content);

  // ---- status bar
  const sbDot = h('span', { class: 'sb-dot' });
  const sbStream = h('a', { href: '/', class: 'sb-item sb-link' }, sbDot, h('span', null, 'connecting…'));
  const sbPlays = h('span', { class: 'sb-item' });
  const sbSync = h('span', { class: 'sb-item' });
  const sbRepo = h('a', { class: 'sb-item sb-right sb-link', href: 'https://github.com/finstats/finstats', target: '_blank', rel: 'noopener noreferrer', title: 'finstats on GitHub' }, icon('github', 12), 'Repo');
  const sbVer = h('a', { class: 'sb-item sb-link', href: '/changelog', title: 'Patch notes' }, state.status && state.status.version ? 'v' + state.status.version : '');
  const statusbar = h('footer', { class: 'statusbar', role: 'status', 'aria-label': 'Collector status' }, sbStream, sbPlays, sbSync, sbRepo, sbVer);

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
        sbStream.title = !ok ? 'finstats can’t reach Jellyfin right now. Plays are not being recorded.'
          : sum.collector_live ? 'Jellyfin is pushing what is playing · go to now playing' : 'Go to now playing';
        sbPlays.textContent = `${num(sum.plays_total)} plays`;
        sbSync.textContent = sum.last_sync_at ? `synced ${relTime(sum.last_sync_at)}` : 'not synced yet';
        sbSync.title = sum.last_sync_at ? 'Last library sync: ' + dateTime(sum.last_sync_at) : '';
        if (sum.version) { sbVer.textContent = 'v' + sum.version; noteRunningVersion(sum.version); }
      } catch (e) {
        if (!isAbort(e) && e.status !== 401) {
          sbDot.className = 'sb-dot bad';
          sbStream.lastChild.textContent = 'finstats unreachable';
        }
      }
    }
    if (!dead) sbTimer = setTimeout(pollSummary, 10000);
  }
  pollSummary();

  // ---- scroll to top (sits above the status bar)
  const toTop = h('button', { type: 'button', class: 'to-top', 'aria-label': 'Scroll to top', hidden: true }, icon('arrowUp', 16));
  toTop.addEventListener('click', () => {
    const reduce = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
    window.scrollTo({ top: 0, behavior: reduce ? 'auto' : 'smooth' });
    main.focus({ preventScroll: true });
  });
  let ticking = false;
  const onScroll = () => {
    if (ticking) return;
    ticking = true;
    requestAnimationFrame(() => {
      ticking = false;
      const show = window.scrollY > 400;
      if (show) { toTop.hidden = false; requestAnimationFrame(() => toTop.classList.add('is-visible')); }
      else { toTop.classList.remove('is-visible'); setTimeout(() => { if (window.scrollY <= 400) toTop.hidden = true; }, 160); }
    });
  };
  window.addEventListener('scroll', onScroll, { passive: true });

  const el = h('div', { class: 'app' }, topbar, sidebar, scrim, main, statusbar, toTop);

  return {
    el, content, userId: me.id + ':' + me.is_admin + ':' + JSON.stringify(me.permissions || {}) + JSON.stringify(me.features || {}),
    setActive(path) {
      setDrawer(false);
      for (const a of navLinks) {
        const n = a._item;
        const on = n.exact ? path === n.href : n.not && (path === n.not || path.startsWith(n.not + '/')) ? false : path === n.href || path.startsWith(n.href + '/') || (n.also || []).some((p) => path.startsWith(p));
        a.classList.toggle('is-active', on);
        if (on) a.setAttribute('aria-current', 'page'); else a.removeAttribute('aria-current');
      }
    },
    destroy() {
      dead = true;
      clearTimeout(sbTimer);
      if (sbAbort) sbAbort.abort();
      window.removeEventListener('scroll', onScroll);
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
// Overlays (modal, palette, dropdown) swallow their own Esc before it gets here.
let herePath = location.pathname;
let cameFrom = null;
onRouteChange(() => {
  if (location.pathname !== herePath) { cameFrom = herePath; herePath = location.pathname; }
});

/** Where Esc leads from this page; null on top-level pages. `back` = prefer the real history entry. */
function escTarget(path) {
  if (/^\/libraries\/[^/]+/.test(path)) return { up: '/libraries' };
  if (/^\/users\/[^/]+\/timeline$/.test(path)) return { up: path.replace(/\/timeline$/, '') };
  if (/^\/users\/[^/]+/.test(path)) return can('see_everyone') && path !== `/users/${state.user.id}` ? { up: '/users' } : null; // your own profile is a top-level page
  if (/^\/items\/[^/]+/.test(path)) return { up: '/libraries', back: true };     // reached from anywhere, so return to wherever that was
  if (/^\/people\/[^/]+/.test(path)) return { up: '/libraries', back: true };
  return null;
}

document.addEventListener('keydown', (e) => {
  if (e.key !== 'Escape' || e.defaultPrevented || e.metaKey || e.ctrlKey || e.altKey || e.shiftKey || !state.user) return;
  const drawer = document.querySelector('.drawer-open');
  if (drawer) { const scrim = document.querySelector('.scrim'); if (scrim) scrim.click(); return; }
  // Esc belongs to whatever you are using: leave fields and keyboard-driven widgets (charts, rows) alone.
  const el = document.activeElement;
  if (el && el !== document.body && !el.matches('a, button, main')) { if (el.matches('input, textarea')) el.blur(); return; }
  const target = escTarget(location.pathname);
  if (!target) return;
  e.preventDefault();
  // Going back through history restores the list exactly as it was left (scroll position, filters).
  // Only when the entry behind this one is finstats' own: a title opened in a new tab has none.
  if (canGoBack() && cameFrom !== null && (target.back || cameFrom === target.up)) history.back();
  else navigate(target.up);
});

// Global shortcut for the command palette.
document.addEventListener('keydown', (e) => {
  // Ctrl+Space. `code` rather than `key`, so it is the same physical shortcut on every keyboard layout.
  if (e.ctrlKey && !e.metaKey && !e.altKey && !e.shiftKey && e.code === 'Space') {
    if (!state.user) return;
    e.preventDefault();
    openPalette();
  }
});
