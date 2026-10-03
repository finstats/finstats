// App-wide state: server status, signed-in user, and the global filters.

import { store } from './dom.js';
import { api, clearViewCache } from './api.js';

export const state = {
  status: null, // {configured, version, server_name?}
  user: null,   // {id, name, is_admin, has_image, permissions: {see_everyone, see_network, see_server, manage}}
};

export const isAdmin = () => !!(state.user && state.user.is_admin);
/** What the signed-in user may see or do. The server enforces all of it; this only decides what to draw. */
export const can = (perm) => !!(state.user && (state.user.is_admin || (state.user.permissions && state.user.permissions[perm])));

export const RANGES = [
  { value: 7, label: '7d', long: 'Last 7 days' },
  { value: 30, label: '30d', long: 'Last 30 days' },
  { value: 90, label: '90d', long: 'Last 90 days' },
  { value: 365, label: '1y', long: 'Last year' },
  { value: 0, label: 'All', long: 'All time' },
];

const validDays = (v) => RANGES.some((r) => r.value === v);

/** Range comes from the URL first, then the last choice, then 30 days. */
export function readDays(query) {
  const q = query && query.get('days');
  if (q != null && q !== '' && validDays(Number(q))) return Number(q);
  const saved = Number(store.get('finstats.days', '30'));
  return validDays(saved) ? saved : 30;
}
export function saveDays(days) { store.set('finstats.days', String(days)); }

export const rangeLong = (days) => (RANGES.find((r) => r.value === days) || RANGES[1]).long;

// Small cache so every user combobox doesn't refetch the list.
let usersCache = null;
let usersAt = 0;
export async function userList(signal) {
  if (usersCache && Date.now() - usersAt < 60000) return usersCache;
  const data = await api.get('/users', null, { signal });
  usersCache = (data.users || []).slice().sort((a, b) => a.name.localeCompare(b.name));
  usersAt = Date.now();
  return usersCache;
}
export function resetCaches() { usersCache = null; clearViewCache(); }

// ---- "new version" hint: a dot on the Patch notes tab until the notes have been opened
const SEEN_KEY = 'finstats.seenVersion';
const versionSubs = new Set();
export function hasUnseenVersion() {
  const v = state.status && state.status.version;
  if (!v) return false;
  try { return localStorage.getItem(SEEN_KEY) !== v; } catch { return false; }
}
export function markVersionSeen(v) {
  try { if (v) localStorage.setItem(SEEN_KEY, v); } catch { /* private mode: the dot just stays */ }
  versionSubs.forEach((fn) => fn());
}
/** The server was updated while this tab stayed open. */
export function noteRunningVersion(v) {
  if (!v || !state.status || state.status.version === v) return;
  state.status.version = v;
  versionSubs.forEach((fn) => fn());
}
export function onVersionSeen(fn) { versionSubs.add(fn); return () => versionSubs.delete(fn); }

// ---- the mobile menu: one of FinUI mobile-nav's styles, this browser's choice (a phone picks its own menu). Nothing stored
// is the tab bar; a stored key FinUI does not know reads as the tab bar too.
const MOBILE_NAV_KEY = 'finstats.mobileNav';
const mobileNavSubs = new Set();
export const mobileNavChoice = () => store.get(MOBILE_NAV_KEY) || '';
export function setMobileNav(key) {
  store.set(MOBILE_NAV_KEY, key);
  mobileNavSubs.forEach((fn) => fn(key));
}
export function onMobileNavChange(fn) { mobileNavSubs.add(fn); return () => mobileNavSubs.delete(fn); }

// ---- the desktop menu: one of FinUI desktop-nav's styles, the side a sidebar sits on and what Pinned holds, all this
// browser's (like the phone's menu). Nothing stored is today's sidebar on the left, with the primary pages pinned.
const DESKTOP_NAV_KEY = 'finstats.desktopNav', DESKTOP_SIDE_KEY = 'finstats.desktopNavSide', PINS_KEY = 'finstats.desktopNavPins';
const desktopNavSubs = new Set();
export const desktopNavChoice = () => ({
  style: store.get(DESKTOP_NAV_KEY) || '',
  side: store.get(DESKTOP_SIDE_KEY) || 'left',
  pins: (() => { try { return JSON.parse(store.get(PINS_KEY) || 'null'); } catch { return null; } })(),
});
/** Change any of style, side and pins; the menu is drawn again for a style or a side, never for a pin (it redraws itself). */
export function setDesktopNav({ style, side, pins }) {
  if (style !== undefined) store.set(DESKTOP_NAV_KEY, style);
  if (side !== undefined) store.set(DESKTOP_SIDE_KEY, side);
  if (pins !== undefined) store.set(PINS_KEY, JSON.stringify(pins));
  if (style !== undefined || side !== undefined) desktopNavSubs.forEach((fn) => fn());
}
export function onDesktopNavChange(fn) { desktopNavSubs.add(fn); return () => desktopNavSubs.delete(fn); }

// ---- where search opens: in the menu (it grows out of the menu's own search into the menu) or in the page (in the page's
// place), this browser's choice like the menus. Nothing stored is the menu.
const SEARCH_MODE_KEY = 'finstats.searchMode';
export const searchMode = () => (store.get(SEARCH_MODE_KEY) === 'page' ? 'page' : 'menu');
export const setSearchMode = (mode) => store.set(SEARCH_MODE_KEY, mode === 'page' ? 'page' : 'menu');

// ---- theme: 'device' (nothing stored, app.css follows the device), 'light' or 'dark' — this browser's
// choice. theme.js applies the stored one before the first paint; this changes it afterwards.
const THEME_KEY = 'finstats.theme';
export const THEMES = ['device', 'light', 'dark'];
export function themeChoice() {
  const t = store.get(THEME_KEY);
  return t === 'light' || t === 'dark' ? t : 'device';
}
/** Apply a choice, and keep it (Device is the absence of one). */
export function setTheme(t) {
  if (t === 'light' || t === 'dark') store.set(THEME_KEY, t); else store.remove(THEME_KEY);
  applyTheme();
}
export function applyTheme() {
  const t = themeChoice(), root = document.documentElement;
  if (t === 'device') delete root.dataset.theme; else root.dataset.theme = t;
  // the browser's own bar: each tag carries one scheme's colour; a choice switches the other off
  for (const m of document.querySelectorAll('meta[name="theme-color"]')) {
    m.dataset.media ??= m.media;
    const light = m.dataset.media.includes('light');
    m.media = t === 'device' ? m.dataset.media : (t === 'light') === light ? 'all' : 'not all';
  }
}
