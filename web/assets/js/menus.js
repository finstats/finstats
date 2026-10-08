// FinStats' context menus (FinUI's context-menu): wherever a title, a person or a library is a link (a poster on the
// dashboard, a row of a list, a name in a table), a right-click, a long-press or the menu key offers what can be done
// to it. One resolver for the whole app, reading the link itself: a page draws nothing for it, and a new page's links
// have a menu the day they are written. The link's `data-type` (a Jellyfin type) is what says a title is a film or a
// show, the two things a watchlist holds; a link without one is offered everything else.
import { state } from './state.js';
import { navigate, onRouteChange } from './router.js';
import { watchMenuItem } from './watchlist.js';
import { attachContextMenu, closeMenu } from '../finui/components/context-menu/context-menu.js';

/** The `dataset` that tells the menu what kind of title a link opens; nothing when that is not known. */
export const typed = (type) => (type ? { type } : null);

/** The `dataset` of a card for a title the library does not have yet (Coming up): no link to read, so it names the title
 *  itself: `t` is what a watchlist knows it by, `{kind, tmdb_id, tvdb_id, title, year}`. */
export function notHere(t) {
  const d = { notHere: '1', kind: t.kind, name: t.title, tmdb: t.tmdb_id, tvdb: t.tvdb_id, year: t.year };
  return Object.fromEntries(Object.entries(d).filter(([, v]) => v != null && v !== '').map(([k, v]) => [k, String(v)]));
}

const KINDS = { items: 'title', users: 'person', people: 'person', libraries: 'library' };

/** What a link leads to, or null when it is not one of FinStats' own things. */
export function linkTarget(a) {
  const href = a.getAttribute('href');
  if (!href || !href.startsWith('/') || href.startsWith('//')) return null;
  const path = href.split(/[?#]/)[0];
  const m = path.match(/^\/(items|libraries|people)\/([^/]+)\/?$/) || path.match(/^\/(users)\/([^/]+)(?:\/[a-z]+)?\/?$/);
  return m ? { kind: KINDS[m[1]], section: m[1], id: decodeURIComponent(m[2]), path } : null;
}

function nameOf(a) {
  const named = a.getAttribute('aria-label') || (a.querySelector('[class*="name"], [class*="title"]') || a).textContent;
  return String(named || '').replace(/\s+/g, ' ').trim().slice(0, 80);
}

async function copyText(text) {
  try { await navigator.clipboard.writeText(text); return; } catch { /* not a secure origin: the old way */ }
  const t = document.createElement('textarea');
  t.value = text; t.setAttribute('readonly', ''); t.className = 'sr-only';
  document.body.append(t); t.select();
  try { document.execCommand('copy'); } finally { t.remove(); }
}

function itemsFor(a, target) {
  const { kind, section, id, path } = target;
  const here = section === 'users' ? `/users/${encodeURIComponent(id)}` : path;
  const url = new URL(here, location.origin).href;
  const items = [
    { label: 'Open', icon: kind === 'person' ? 'user' : kind === 'library' ? 'library' : 'film', onSelect: () => navigate(here) },
    kind === 'person' && section === 'users' ? { label: 'Timeline', icon: 'activity', onSelect: () => navigate(`${here}/timeline`) } : null,
    kind === 'person' && section === 'users' && state.user && state.user.id === id ? { label: 'Watchlist', icon: 'bookmark', onSelect: () => navigate(`${here}/watchlist`) } : null,
    { label: 'Open in a new tab', icon: 'external', href: here, newTab: true },
  ];
  if (kind === 'title') {
    const keep = (a.dataset.type === 'Movie' || a.dataset.type === 'Series') ? watchMenuItem({ item_id: id }) : null;
    const jf = state.user && state.user.jellyfin_details;
    if (keep || jf) items.push({ separator: true }, keep, jf ? { label: 'Open in Jellyfin', icon: 'play', href: jf + encodeURIComponent(id), newTab: true } : null);
  }
  items.push({ separator: true }, { label: 'Copy link', icon: 'link', onSelect: () => copyText(url) });
  return items.filter(Boolean);
}

/** Its page on TMDB and TVDB, in the forms a title's own page links them (`stats.rs`). */
function elsewhere(d) {
  const film = d.kind === 'Movie';
  return [
    d.tmdb ? { label: 'Open on TMDB', icon: 'external', href: `https://www.themoviedb.org/${film ? 'movie' : 'tv'}/${encodeURIComponent(d.tmdb)}`, newTab: true } : null,
    d.tvdb && !film ? { label: 'Open on TVDB', icon: 'external', href: `https://www.thetvdb.com/dereferrer/series/${encodeURIComponent(d.tvdb)}`, newTab: true } : null,
  ];
}

function itemsNotHere(d) {
  const t = { kind: d.kind, tmdb_id: d.tmdb || null, tvdb_id: d.tvdb || null, title: d.name, year: d.year ? Number(d.year) : null };
  const upcoming = state.user && state.user.features && state.user.features.upcoming;
  return [
    upcoming ? { label: 'In Pipeline', icon: 'calendar', onSelect: () => navigate('/pipeline?tab=upcoming') } : null,
    upcoming ? { separator: true } : null,
    watchMenuItem(t),
    ...elsewhere(d),
    { separator: true },
    { label: 'Copy title', icon: 'copy', onSelect: () => copyText(d.kind === 'Movie' && d.year ? `${d.name} (${d.year})` : d.name) },
  ].filter(Boolean);
}

/** Once, for the whole app. */
export function installMenus() {
  attachContextMenu(document, (el) => {
    if (!state.user || !el.closest || el.closest('input, textarea, select, [contenteditable], .fui-context-menu')) return null;
    const a = el.closest('a[href]');
    const target = a && linkTarget(a);
    if (!target) {
      const card = !a && el.closest('[data-not-here]');
      return card && card.dataset.name ? { items: itemsNotHere(card.dataset), from: card, label: `Actions for ${card.dataset.name}` } : null;
    }
    const name = nameOf(a);
    return { items: itemsFor(a, target), from: a, label: name ? `Actions for ${name}` : 'Actions' };
  });
  onRouteChange(() => closeMenu({ instant: true }));
}
