import { h, icon, num, bytes, durEl, relEl, relTime, dateTime, duration, debounce, mount } from '../dom.js';
import { api, soft, imgItem } from '../api.js';
import { readDays, saveDays, can } from '../state.js';
import { replaceQuery } from '../router.js';
import { pageHeader, card, filterBar, dataView, sk, emptyState, topList, poster, segmented, setBusy, errorState } from '../components.js';
import { activityCard, libraryInsights } from '../widgets.js';
import { KINDS as HEALTH_KINDS } from './health.js';
import { button } from '../../finui/components/button/button.js';
import { mediaCard, mediaGrid } from '../../finui/components/media-card/media-card.js';

const KIND = { movies: 'Movies', tvshows: 'Shows', music: 'Music', musicvideos: 'Music videos', homevideos: 'Home videos', books: 'Books', boxsets: 'Collections', mixed: 'Mixed' };
const KIND_ICON = { movies: 'film', tvshows: 'play', music: 'activity' };

function counts(l) {
  if (l.collection_type === 'tvshows') return `${num(l.series_count)} series · ${num(l.episode_count)} episodes`;
  if (l.collection_type === 'music') return `${num(l.item_count)} tracks`;
  return `${num(l.item_count)} ${l.item_count === 1 ? 'item' : 'items'}`;
}

// ---------------------------------------------------------------- /libraries
/**
 * The picture Jellyfin shows for a library on its home screen. `fallback` (the plain type icon on
 * the cards) takes its place when Jellyfin has none; without a fallback it simply disappears.
 */
function libraryArt(l, w, cls, fallback = null) {
  if (l.removed) return fallback; // Jellyfin no longer has it, so there is nothing to ask for
  return h('img', { class: cls, src: imgItem(l.id, w), alt: '', loading: 'lazy', decoding: 'async',
    onError: (e) => { if (fallback) e.target.replaceWith(fallback); else e.target.remove(); } });
}

// Loaders are shared with the prefetcher, so a prefetched view has exactly the address the page asks for.
const loadLibraries = (days, signal) => api.get('/libraries', { days }, { signal });
const loadLibrary = (id, days, signal) => api.get(`/libraries/${id}`, { days }, { signal });
const loadMakeup = (libraryId, signal) => soft(api.get('/library/insights', libraryId ? { library_id: libraryId } : null, { signal }));
export const prefetchLibraries = ({ query, signal }) => [() => loadLibraries(readDays(query), signal), () => loadMakeup(null, signal)];
// Library health is for those who may see the server; for anyone else the page asks nothing about it.
const loadHealth = (libraryId, signal) => soft(api.get('/library/health', { library_id: libraryId }, { signal }));
export const prefetchLibrary = ({ params, query, signal }) => [() => loadLibrary(params.id, readDays(query), signal), () => loadMakeup(params.id, signal),
  ...(can('see_server') ? [() => loadHealth(params.id, signal)] : [])];

export function librariesPage(ctx) {
  ctx.title('Libraries');
  let days = readDays(ctx.query);
  const view = h('div');
  const dv = dataView({
    container: view, signal: ctx.signal,
    skeleton: () => h('div', { class: 'lib-grid' }, [0, 1, 2].map(() => h('div', { class: 'lib-card' }, sk.line('40%', 16), sk.line('60%'), sk.block(48)))),
    fetch: () => loadLibraries(days, ctx.signal),
    render: (data) => {
      const libs = (data.libraries || []).slice().sort((a, b) => (a.removed - b.removed) || (b.watch_s || 0) - (a.watch_s || 0));
      if (!libs.length) return emptyState('No libraries yet', 'Libraries appear after the first sync with Jellyfin. You can start one from Settings → Tasks.');
      return h('div', { class: 'lib-grid' }, libs.map((l) => h('a', { class: ['lib-card', l.removed && 'is-dim'], href: `/libraries/${l.id}` },
        h('div', { class: 'lib-head' }, libraryArt(l, 300, 'lib-art', h('span', { class: 'lib-icon' }, icon(KIND_ICON[l.collection_type] || 'library', 16))),
          h('div', null, h('div', { class: 'lib-name' }, l.name), h('div', { class: 'lib-kind' }, KIND[l.collection_type] || 'Library', l.removed ? ' · removed from Jellyfin' : ''))),
        h('div', { class: 'lib-counts mono' }, counts(l), l.size_bytes ? ` · ${bytes(l.size_bytes)}` : ''),
        h('dl', { class: 'lib-stats' },
          h('div', null, h('dt', null, 'Watch time'), h('dd', null, durEl(l.watch_s))),
          h('div', null, h('dt', null, 'Plays'), h('dd', { class: 'mono' }, num(l.plays))),
          h('div', null, h('dt', null, 'Last played'), h('dd', { class: 'mono', title: l.last_played_at ? dateTime(l.last_played_at) : '' }, l.last_played_at ? relTime(l.last_played_at) : 'Never'))))));
    },
  });
  ctx.root.append(pageHeader('Libraries', 'What’s on the server and how much of it gets watched'),
    filterBar({ days, signal: ctx.signal, onDays: (v) => { days = v; saveDays(v); replaceQuery({ days }); dv.load(); } }), view, makeupSlot(ctx));
  dv.load();
}

/** Library make-up loads once per page: it has no time range, so range changes never refetch it. */
function makeupSlot(ctx, libraryId) {
  const slot = h('div', { class: 'makeup' });
  dataView({
    container: slot, signal: ctx.signal,
    skeleton: () => [sk.line('240px', 18), sk.tiles(3), h('div', { class: 'grid-3' }, sk.cardRows(4), sk.cardRows(4), sk.cardRows(4))],
    fetch: () => loadMakeup(libraryId, ctx.signal),
    render: (d) => libraryInsights(d, { scoped: !!libraryId }),
  }).load();
  return slot;
}

// ---------------------------------------------------------------- /libraries/:id
export function libraryPage(ctx) {
  const id = ctx.params.id;
  ctx.title('Library');
  let days = readDays(ctx.query);
  const headerSlot = h('div', null, pageHeader(sk.line('200px', 26)));
  const view = h('div', { class: 'stack' });
  const dv = dataView({
    container: view, signal: ctx.signal,
    skeleton: () => [sk.cardBlock(260), sk.cardRows(5)],
    fetch: () => loadLibrary(id, days, ctx.signal),
    render: (d) => {
      const l = d.library;
      ctx.title(l.name);
      headerSlot.replaceChildren(pageHeader(l.name, [KIND[l.collection_type] || 'Library', counts(l), l.size_bytes ? bytes(l.size_bytes) : null, l.removed ? 'removed from Jellyfin' : null].filter(Boolean).join(' · '), libraryArt(l, 480, 'lib-art lib-art-lg')));
      return [
        h('div', { class: 'fui-stat-tile__grid fui-stat-tile__grid--three' },
          h('div', { class: 'fui-stat-tile' }, h('div', { class: 'fui-stat-tile__label' }, 'Watch time'), h('div', { class: 'fui-stat-tile__value' }, durEl(l.watch_s, ''))),
          h('div', { class: 'fui-stat-tile' }, h('div', { class: 'fui-stat-tile__label' }, 'Plays'), h('div', { class: 'fui-stat-tile__value' }, num(l.plays))),
          h('div', { class: 'fui-stat-tile' }, h('div', { class: 'fui-stat-tile__label' }, 'Last played'), h('div', { class: 'fui-stat-tile__value' }, l.last_played_at ? relEl(l.last_played_at, '') : 'Never'))),
        activityCard({ daily: d.daily, bucket: d.bucket }),
        card({ title: 'Most watched', sub: 'By watch time', body: topList(d.top) }),
        card({ title: 'Recently added', body: itemGrid(d.recently_added),
          actions: button({ href: `/libraries/${encodeURIComponent(id)}/titles`, size: 'sm', variant: 'ghost' }, 'Everything in it', icon('chevronRight', 13)) }),
      ];
    },
  });
  ctx.root.append(h('a', { class: 'back-link', href: '/libraries' }, icon('chevronLeft', 14), 'Libraries'), headerSlot,
    filterBar({ days, signal: ctx.signal, onDays: (v) => { days = v; saveDays(v); replaceQuery({ days }); dv.load(); } }), view, healthSlot(ctx, id), makeupSlot(ctx, id));
  dv.load();
}

/** "7 things to look at": this library's share of Library health, leading to it. Nothing at all when there is nothing,
 *  or for someone who may not see the server. */
function healthSlot(ctx, libraryId) {
  if (!can('see_server')) return null;
  const slot = h('div');
  dataView({
    container: slot, signal: ctx.signal,
    skeleton: () => h('div'),
    fetch: () => loadHealth(libraryId, ctx.signal),
    render: (d) => {
      if (!d || !d.total) return null;
      const kinds = d.kinds.filter((k) => k.count).map((k) => `${num(k.count)} ${(HEALTH_KINDS[k.kind] || [k.kind])[0].toLowerCase()}`);
      return card({ cls: 'health-card', title: 'Library health', body: h('p', null,
        h('a', { href: `/server/health?library_id=${encodeURIComponent(libraryId)}` }, `${num(d.total)} ${d.total === 1 ? 'thing' : 'things'} to look at`), `: ${kinds.join(', ')}.`) });
    },
  }).load();
  return slot;
}

export function itemGrid(items) {
  if (!items || !items.length) return h('div', { class: 'fui-empty--chart fui-empty--chart-sm' }, 'Nothing added yet.');
  return mediaGrid(items.map((it) => mediaCard({ href: `/items/${it.id}`, poster: poster(it.image_item_id || it.id, it.name, { w: 300, cls: 'fui-poster--grid' }),
    name: it.name, sub: [it.sub || it.year, it.date_created ? 'added ' + relTime(it.date_created) : null].filter(Boolean).join(' · ') })));
}

// ---------------------------------------------------------------- /libraries/:id/titles
// Everything a library holds, not only what arrived lately: a grid of posters by name, searched and sorted on the server,
// sixty at a time, the next sixty arriving as the end of the grid comes into view (or at the press of a button).

const TITLE_KIND = { Movie: 'Films', Series: 'Shows', MusicAlbum: 'Albums', MusicVideo: 'Music videos', Video: 'Videos', Book: 'Books', AudioBook: 'Audiobooks' };
// Each sort in the direction one means by it: names from A, the rest from the most.
const TITLE_SORTS = [{ value: 'name', label: 'Name', dir: 'asc' }, { value: 'year', label: 'Year', dir: 'desc' }, { value: 'added', label: 'Added', dir: 'desc' }, { value: 'size', label: 'Size', dir: 'desc' }];
const titlesFilter = (query) => {
  const sort = TITLE_SORTS.find((x) => x.value === query.get('sort')) || TITLE_SORTS[0];
  return { q: query.get('q') || '', type: query.get('type') || '', sort: sort.value, dir: sort.dir };
};
const loadTitles = (libraryId, f, page, signal) => api.get(`/libraries/${encodeURIComponent(libraryId)}/titles`, { q: f.q, type: f.type, sort: f.sort, dir: f.dir, page: page > 1 ? page : '' }, { signal });
export const prefetchLibraryTitles = ({ params, query, signal }) => [() => loadLibrary(params.id, readDays(query), signal), () => loadTitles(params.id, titlesFilter(query), 1, signal)];

function titleSub(t) {
  if (t.type === 'Series') return [t.year, t.episodes != null ? `${num(t.episodes)} episode${t.episodes === 1 ? '' : 's'}` : null].filter(Boolean).join(' · ');
  if (t.type === 'MusicAlbum') return [t.album_artist, t.year].filter(Boolean).join(' · ');
  return [t.year, t.runtime_s ? duration(t.runtime_s) : null].filter(Boolean).join(' · ');
}
const titleCard = (t) => h('li', null, mediaCard({ href: `/items/${t.id}`, poster: poster(t.has_image ? t.image_item_id : null, t.name, { w: 300, cls: 'fui-poster--grid' }), name: t.name, sub: titleSub(t) }));

export function libraryTitlesPage(ctx) {
  const id = ctx.params.id;
  const f = titlesFilter(ctx.query);
  ctx.title('Library');
  const headerSlot = h('div', null, pageHeader(sk.line('200px', 26)));
  const back = h('a', { class: 'back-link', href: `/libraries/${encodeURIComponent(id)}` }, icon('chevronLeft', 14), 'Library');
  const view = h('div', { class: 'stack' });
  let observer = null;
  ctx.onCleanup(() => observer && observer.disconnect());

  loadLibrary(id, readDays(ctx.query), ctx.signal).then((d) => {
    ctx.title(`Everything in ${d.library.name}`);
    back.lastChild.textContent = d.library.name;
    headerSlot.replaceChildren(pageHeader(d.library.name, 'Everything in it'));
  }).catch(() => {});

  function apply() {
    replaceQuery({ q: f.q, type: f.type, sort: f.sort === 'name' ? '' : f.sort });
    dv.load();
  }

  const search = h('input', { class: 'fui-field__input fui-field__input--search', type: 'search', placeholder: 'Find a title…', value: f.q, 'aria-label': 'Find a title in this library', autocomplete: 'off' });
  const onSearch = debounce(() => { f.q = search.value.trim(); apply(); }, 250);
  search.addEventListener('input', onSearch);
  ctx.onCleanup(() => onSearch.cancel());
  const sortPick = h('div', { class: 'titles-sort' }, segmented({ label: 'Sort by', size: 'sm', value: f.sort, options: TITLE_SORTS,
    onChange: (v) => { const x = TITLE_SORTS.find((o) => o.value === v); f.sort = x.value; f.dir = x.dir; apply(); } }));
  const kindSlot = h('div', { class: 'titles-kinds' });
  const count = h('p', { class: 'result-count', 'aria-live': 'polite' });

  const dv = dataView({
    container: view, signal: ctx.signal,
    skeleton: () => h('ul', { class: 'fui-media-card__grid titles-grid' }, Array.from({ length: 12 }, () => h('li', null, sk.block(180)))),
    fetch: () => loadTitles(id, f, 1, ctx.signal),
    render: (d) => {
      if (observer) observer.disconnect();
      count.textContent = `${num(d.total)} title${d.total === 1 ? '' : 's'}`;
      mount(kindSlot, d.types.length > 1 ? segmented({ label: 'Kind', size: 'sm', value: f.type,
        options: [{ value: '', label: 'All' }, ...d.types.map((t) => ({ value: t.type, label: `${TITLE_KIND[t.type] || t.type} (${num(t.count)})` }))],
        onChange: (v) => { f.type = v; apply(); } }) : null);
      if (!d.items.length) return emptyState(f.q || f.type ? 'Nothing matches' : 'Nothing here yet', f.q || f.type ? 'No title in this library has every word you typed.' : 'Titles appear here after finstats reads the library.');
      const grid = h('ul', { class: 'fui-media-card__grid titles-grid' }, d.items.map(titleCard));
      let page = 1, shown = d.items.length, busy = false;
      const more = button({ variant: 'ghost', class: 'titles-more', onClick: () => next() }, 'Show more');
      const err = h('div');
      const tail = h('div', { class: 'titles-tail' }, more, err);
      const paintMore = () => { more.hidden = shown >= d.total; more.lastChild.textContent = `Show ${num(Math.min(d.page_size, d.total - shown))} more`; };
      async function next() {
        if (busy || shown >= d.total) return;
        busy = true; setBusy(more, true, 'Loading…'); mount(err);
        try {
          const n = await loadTitles(id, f, page + 1, ctx.signal);
          page += 1; shown += n.items.length;
          grid.append(...n.items.map(titleCard));
          if (!n.items.length) shown = d.total;
        } catch (e) {
          if (!ctx.signal.aborted) mount(err, errorState(e, next));
        } finally { busy = false; setBusy(more, false); paintMore(); }
      }
      paintMore();
      // The end of the grid coming into view asks for the next page; the button is there for a keyboard.
      if ('IntersectionObserver' in window) {
        observer = new IntersectionObserver((es) => { if (es.some((e) => e.isIntersecting)) next(); }, { rootMargin: '600px 0px' });
        observer.observe(tail);
      }
      return [grid, tail];
    },
  });

  ctx.root.append(back, headerSlot,
    h('div', { class: 'filters' }, h('div', { class: 'fui-field__search titles-search' }, icon('search', 14), search), sortPick, kindSlot), count, view);
  dv.load();
}
