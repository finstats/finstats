// Search (Ctrl+Space, or the menu's own): FinUI's search run against finstats: a page, a user, anything in the library, or
// someone who is in it. Where it lives (the menu, or the page) is shell.js' business; this only answers the words.

import { h, icon, debounce } from './dom.js';
import { api, isAbort } from './api.js';
import { poster, avatar } from './components.js';
import { pageCommands } from './shell.js';
import { watchToggle, watchable } from './watchlist.js';
import { searchPanel } from '../finui/components/search/search.js';

const TYPE = { Movie: 'Movie', Series: 'Series', MusicAlbum: 'Album', Audio: 'Track', Episode: 'Episode' };
// "Actor · 12 titles", "Director · 1 title", "Actor and director · 4 titles"
const credit = (p) => [p.is_actor && p.is_director ? 'Actor and director' : p.is_director ? 'Director' : 'Actor',
  p.titles ? `${p.titles} ${p.titles === 1 ? 'title' : 'titles'}` : null].filter(Boolean).join(' · ');

/** A search over finstats. `onPick(row)` opens what was chosen (every row has an `href`); `onEscape()` gives up. */
export function finstatsSearch({ onPick, onEscape }) {
  const pages = pageCommands();
  let abort = null;
  // Every typed word has to be in the page's name, in any order.
  const pageRows = (q) => pages.filter((p) => !q || q.split(/\s+/).filter(Boolean).every((w) => p.label.toLowerCase().includes(w)))
    .map((p) => ({ label: p.label, href: p.href, thumb: h('span', { class: 'fui-search__icon' }, icon(p.icon, 15)) }));
  const ask = debounce(async (q, paint) => {
    if (abort) abort.abort();
    abort = new AbortController();
    try {
      const data = await api.get('/search', { q, limit: 12 }, { signal: abort.signal });
      paint([
        { title: 'Pages', rows: pageRows(q.toLowerCase()) },
        { title: 'Users', rows: (data.users || []).map((u) => ({ label: u.name, href: `/users/${u.id}`, thumb: avatar(u.id, u.name, { size: 24 }) })) },
        { title: 'Library', rows: (data.items || []).map((it) => ({ label: it.name, href: `/items/${it.id}`,
          sub: [TYPE[it.type] || it.type, it.sub || it.year].filter(Boolean).join(' · '), thumb: poster(it.image_item_id || it.id, it.name, { w: 120, cls: 'fui-poster--xs' }),
          keep: watchable(it.type) ? watchToggle({ item_id: it.id }, { compact: true, name: it.name, keepFocus: true }) : null })) },
        { title: 'Cast and crew', rows: (data.people || []).map((p) => ({ label: p.name, href: `/people/${p.id}`, sub: credit(p),
          thumb: poster(p.has_image ? p.id : null, p.name, { w: 120, cls: 'fui-poster--xs' }) })) },
      ], `Nothing matches “${q}”.`);
    } catch (e) {
      if (isAbort(e) || e.status === 401) return;
      paint([{ title: 'Pages', rows: pageRows(q.toLowerCase()) }], 'Search is unavailable right now.');
    }
  }, 150);
  const panel = searchPanel({ placeholder: 'Search titles, people and pages', label: 'Search', onPick, onEscape,
    run: (q, paint) => {
      if (!q) { ask.cancel(); if (abort) abort.abort(); paint([{ title: 'Pages', rows: pageRows('') }]); return; }
      paint([{ title: 'Pages', rows: pageRows(q.toLowerCase()) }], 'Searching…');
      ask(q, paint);
    } });
  const destroy = panel.destroy;
  panel.destroy = () => { ask.cancel(); if (abort) abort.abort(); destroy(); };
  return panel;
}
