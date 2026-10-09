// Simple mode's Libraries and a library's page. The list is every library in a table: what it holds, what was played last
// and how much. A library's page has its global figures, the people who watch it most, what was watched in it lately,
// and its own history.

import { h, icon, num, relTime } from '../../dom.js';
import { api } from '../../api.js';
import { can } from '../../state.js';
import { dataView, poster, avatar, sk, emptyState } from '../../components.js';
import { dataTable } from '../../tables.js';
import { tile, tileStat, tileStats } from '../../../finui/components/tile-stat/tile-stat.js';
import { shelf } from '../../../finui/components/shelf/shelf.js';
import { mediaCard } from '../../../finui/components/media-card/media-card.js';
import { libraryIcon, longDuration } from '../platform.js';
import { SPANS, profileHead, box, globalStats } from '../profile.js';
import { historyList } from '../history.js';

const TYPE = { movies: 'Movie', tvshows: 'Show', music: 'Artist', musicvideos: 'Music video', books: 'Book', homevideos: 'Video', boxsets: 'Collection' };
const KIND_ICON = { Movie: 'film', Episode: 'tv', Audio: 'music' };

export const loadLibraries = (signal) => api.get('/libraries', { days: 0 }, { signal });
export const prefetchLibraries = ({ signal }) => [() => loadLibraries(signal)];

/** What a library holds, in the three columns a classic stats server gives it: titles, then seasons or albums, then episodes or tracks. */
function holds(l) {
  if (l.collection_type === 'tvshows' || l.series_count) return [l.series_count, l.season_count, l.episode_count];
  if (l.collection_type === 'music' || l.track_count) return [l.artist_count, l.album_count, l.track_count];
  return [l.movie_count || l.item_count, null, null];
}

function lastPlayed(l) {
  if (!l.last_item_id) return h('span', { class: 'muted' }, 'n/a');
  const words = l.last_series_name ? `${l.last_series_name} - ${l.last_item_name}` : l.last_item_name;
  return h('span', { class: 'simple-title' }, h('span', { class: 'simple-kind', 'aria-hidden': 'true' }, icon(KIND_ICON[l.last_item_type] || 'play', 14)), h('a', { href: `/items/${l.last_item_id}` }, words));
}

function table(libs) {
  const th = (label, opts = {}) => h('th', opts, label);
  const cell = (n) => h('td', { dataset: { sort: String(n || 0) } }, n ? num(n) : '');
  return dataTable(h('table', { class: 'fui-data-table fui-data-table--hover simple-table' },
    h('thead', null, h('tr', null, th('', { 'data-nosort': '' }), th('Library Name'), th('Library Type'), th('Total Movies / TV Shows / Artists'), th('Total Seasons / Albums'),
      th('Total Episodes / Tracks'), th('Last Accessed'), th('Last Played'), th('Total Plays'), th('Total Played Duration'))),
    h('tbody', null, libs.map((l) => {
      const [a, b, c] = holds(l);
      return h('tr', null,
        h('td', { class: 'simple-table__icon' }, icon(libraryIcon(l.collection_type), 22)),
        h('td', null, h('a', { href: `/libraries/${l.id}` }, l.name)),
        h('td', null, TYPE[l.collection_type] || l.collection_type || ''),
        cell(a), cell(b), cell(c),
        h('td', { dataset: { sort: String(l.last_played_at || 0) } }, l.last_played_at ? relTime(l.last_played_at) : 'never'),
        h('td', null, lastPlayed(l)),
        cell(l.plays),
        h('td', { dataset: { sort: String(l.watch_s || 0) } }, longDuration(l.watch_s)));
    }))));
}

export function librariesPage(ctx) {
  ctx.title('Libraries');
  const body = h('div', { id: 'simple-libraries-table' });
  dataView({
    container: body, signal: ctx.signal, skeleton: () => sk.cardRows(5),
    fetch: () => loadLibraries(ctx.signal),
    render: (d) => {
      const libs = (d.libraries || []).filter((l) => !l.removed);
      return libs.length ? [table(libs), h('div', { class: 'simple-table__foot' }, h('span', { class: 'simple-table__count' }, `Showing ${num(libs.length)} of ${num(libs.length)} libraries`))]
        : h('p', { class: 'simple-empty' }, 'No libraries yet.');
    },
  }).load();
  ctx.root.append(h('div', { class: 'simple-panel-head' }, h('h1', { class: 'simple-page-title' }, icon('library', 22), 'All Libraries')), h('div', { class: 'simple-panel' }, body));
}

/** A library's page, in one go: its row over each span, who watches it (with see_everyone), and what was watched in it. */
export function loadLibrary(id, signal) {
  const o = { signal };
  return Promise.all([
    ...SPANS.map(([days]) => api.get('/libraries', { days }, o)),
    can('see_everyone') ? api.get('/stats/top', { days: 0, kind: 'users', library_id: id, limit: 24, sort: 'plays' }, o) : null,
    api.get('/activity', { days: 0, library_id: id, page: 1, per_page: 30 }, o),
  ]).then(([today, week, month, ever, users, recent]) => {
    const mine = (d) => (d.libraries || []).find((l) => l.id === id) || null;
    return { library: mine(ever), spans: [today, week, month, ever].map(mine), users, recent };
  });
}
export const prefetchLibrary = ({ params, signal }) => [() => loadLibrary(params.id, signal)];

export function libraryPage(ctx) {
  const id = ctx.params.id;
  const history = ctx.query.get('tab') === 'history';
  const head = h('div');
  const body = h('div');
  const tabs = (name) => profileHead({ picture: tile({ icon: libraryIcon(name.type), tone: 'neutral', size: 80 }), name: name.text,
    tabs: [{ label: 'Profile', href: `/libraries/${id}`, current: !history }, { label: 'History', href: `/libraries/${id}?tab=history`, current: history }] });
  ctx.root.append(head, body);
  if (history) {
    // The header needs only the library's name; the list asks for its own plays.
    api.get('/libraries', { days: 0 }, { signal: ctx.signal }).then((d) => {
      const l = (d.libraries || []).find((x) => x.id === id);
      if (l) { ctx.title(l.name); head.replaceChildren(tabs({ text: l.name, type: l.collection_type })); }
    }).catch(() => {});
    const list = historyList(ctx, { id: 'simple-library-history', fixed: { library_id: id } });
    body.append(h('div', { class: 'simple-panel-head' }, h('h2', { class: 'simple-page-title' }, 'History'), h('div', { class: 'simple-head__tools' }, list.tools)), list.el);
    return;
  }
  dataView({
    container: body, signal: ctx.signal, skeleton: () => sk.cardRows(6),
    fetch: () => loadLibrary(id, ctx.signal),
    render: (d) => {
      if (!d.library) return emptyState('No such library', 'It may have been removed from Jellyfin.');
      ctx.title(d.library.name);
      head.replaceChildren(tabs({ text: d.library.name, type: d.library.collection_type }));
      const people = d.users && d.users.rows.length ? box({ id: 'simple-user-stats', title: 'User Stats', icon: 'users',
        body: tileStats(d.users.rows.map((u) => tileStat({ picture: avatar(u.id, u.name, { size: 80, hasImage: u.has_image }), name: u.name, href: `/users/${u.id}`, figure: num(u.plays), unit: u.plays === 1 ? 'play' : 'plays' }))) }) : null;
      const seen = new Set();
      const watched = ((d.recent && d.recent.rows) || []).filter((p) => { const k = p.series_id || p.item_id; if (seen.has(k)) return false; seen.add(k); return true; }).slice(0, 18);
      const recent = watched.length ? box({ id: 'simple-library-recent', title: 'Recently Watched', icon: 'repeat',
        body: shelf({ label: 'Recently watched', items: watched.map((p) => mediaCard({ href: `/items/${p.series_id || p.item_id}`, poster: poster(p.image_item_id, p.series_name || p.item_name, { w: 240, cls: 'fui-poster--grid' }),
          name: p.series_name || p.item_name, sub: relTime(p.ended_at) })) }) }) : null;
      return [box({ id: 'simple-global', title: 'Global Stats', icon: 'trendUp', body: globalStats(d.spans) }), people, recent].filter(Boolean);
    },
  }).load();
}
