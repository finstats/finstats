import { h, icon, num, bytes, durEl, relEl, relTime, dateTime, duration, debounce, mount } from '../dom.js';
import { api, soft, imgItem } from '../api.js';
import { readDays, saveDays, can } from '../state.js';
import { replaceQuery } from '../router.js';
import { pageHeader, card, filterBar, dataView, sk, emptyState, topList, poster, segmented, setBusy, errorState, combobox } from '../components.js';
import { activityCard, libraryInsights, libraryInsightsTitle } from '../widgets.js';
import { KINDS as HEALTH_KINDS } from './health.js';
import { button } from '../../finui/components/button/button.js';
import { mediaCard, mediaGrid } from '../../finui/components/media-card/media-card.js';
import { t, tParts, lang } from '../i18n.js';

const KINDS = ['movies', 'tvshows', 'music', 'musicvideos', 'homevideos', 'books', 'boxsets', 'mixed'];
const kindLabel = (type) => (KINDS.includes(type) ? t(`pages.libraries.kind.${type}`) : t('pages.libraries.library'));
const KIND_ICON = { movies: 'film', tvshows: 'play', music: 'activity' };

function counts(l) {
  if (l.collection_type === 'tvshows') return `${t('pages.libraries.series', { count: l.series_count })} · ${t('terms.episode', { count: l.episode_count })}`;
  if (l.collection_type === 'music') return t('terms.track', { count: l.item_count });
  return t('pages.libraries.items', { count: l.item_count });
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
const loadLibrary = (id, days, signal) => api.get(`/libraries/${encodeURIComponent(id)}`, { days }, { signal });
// `libraries` is the comma list the make-up's picker holds; empty is every library.
const loadMakeup = (libraries, signal) => soft(api.get('/library/insights', libraries ? { library_id: libraries } : null, { signal }));
// The make-up picker's choice is in the address (`?libraries=a,b`); a library's own page starts on that library, so there
// every library at once is spelt `all`, and the page's own library is no `libraries` at all.
const pickedLibraries = (query, own = '') => {
  const asked = query && query.get('libraries');
  const v = asked != null ? asked : own;
  return v === 'all' ? '' : v;
};
const librariesInAddress = (v, own) => (v === own ? null : v === '' ? 'all' : v);
/** The range and the picker each change their own part of the address and keep the other's. */
function changeQuery(changes) {
  replaceQuery({ ...Object.fromEntries(new URLSearchParams(location.search)), ...changes });
}
export const prefetchLibraries = ({ query, signal }) => [() => loadLibraries(readDays(query), signal), () => loadMakeup(pickedLibraries(query), signal)];
// Library health is for those who may see the server; for anyone else the page asks nothing about it.
const loadHealth = (libraryId, signal) => soft(api.get('/library/health', { library_id: libraryId }, { signal }));
export const prefetchLibrary = ({ params, query, signal }) => [() => loadLibrary(params.id, readDays(query), signal), () => loadMakeup(pickedLibraries(query, params.id), signal),
  ...(can('see_server') ? [() => loadHealth(params.id, signal)] : [])];

export function librariesPage(ctx) {
  ctx.title(t('nav.libraries'));
  let days = readDays(ctx.query);
  const view = h('div');
  const dv = dataView({
    container: view, signal: ctx.signal,
    skeleton: () => h('div', { class: 'lib-grid' }, [0, 1, 2].map(() => h('div', { class: 'lib-card' }, sk.line('40%', 16), sk.line('60%'), sk.block(48)))),
    fetch: () => loadLibraries(days, ctx.signal),
    render: (data) => {
      const libs = (data.libraries || []).slice().sort((a, b) => (a.removed - b.removed) || (b.watch_s || 0) - (a.watch_s || 0));
      if (!libs.length) return emptyState(t('pages.libraries.empty_title'), t('pages.libraries.empty_sub'));
      return h('div', { class: 'lib-grid' }, libs.map((l) => h('a', { class: ['lib-card', l.removed && 'is-dim'], href: `/libraries/${l.id}` },
        h('div', { class: 'lib-head' }, libraryArt(l, 300, 'lib-art', h('span', { class: 'lib-icon' }, icon(KIND_ICON[l.collection_type] || 'library', 16))),
          h('div', null, h('div', { class: 'lib-name' }, l.name), h('div', { class: 'lib-kind' }, kindLabel(l.collection_type), l.removed ? ' · ' + t('pages.libraries.removed') : ''))),
        h('div', { class: 'lib-counts mono' }, counts(l), l.size_bytes ? ` · ${bytes(l.size_bytes)}` : ''),
        h('dl', { class: 'lib-stats' },
          h('div', null, h('dt', null, t('terms.watch_time')), h('dd', null, durEl(l.watch_s))),
          h('div', null, h('dt', null, t('pages.libraries.plays')), h('dd', { class: 'mono' }, num(l.plays))),
          h('div', null, h('dt', null, t('pages.libraries.last_played')), h('dd', { class: 'mono', title: l.last_played_at ? dateTime(l.last_played_at) : '' }, l.last_played_at ? relTime(l.last_played_at) : t('pages.libraries.never')))))));
    },
  });
  ctx.root.append(pageHeader(t('nav.libraries'), t('pages.libraries.sub')),
    filterBar({ days, signal: ctx.signal, onDays: (v) => { days = v; saveDays(v); changeQuery({ days }); dv.load(); } }), view, makeupSlot(ctx));
  dv.load();
}

/** Library make-up: it has no time range, so range changes never refetch it; its own picker of libraries does. The
 *  heading and the picker stay put while the cards below reload, so ticking several libraries keeps the list open. */
function makeupSlot(ctx, ownLibrary = '') {
  let picked = pickedLibraries(ctx.query, ownLibrary);
  const count = () => picked.split(',').filter(Boolean).length;
  const title = h('h2', { class: 'subsection-title' }, libraryInsightsTitle(count()));
  const body = h('div', { class: 'makeup' });
  const dv = dataView({
    container: body, signal: ctx.signal,
    skeleton: () => [sk.tiles(3), h('div', { class: 'grid-3' }, sk.cardRows(4), sk.cardRows(4), sk.cardRows(4))],
    fetch: () => loadMakeup(picked, ctx.signal),
    // An older server, or an answer that failed: the whole section goes, heading and picker with it.
    render: (d) => { section.hidden = !d; return libraryInsights(d); },
  });
  const picker = combobox({ value: picked, multiple: true, iconName: 'library', label: t('widgets.library.pick_label'),
    allLabel: t('widgets.library.all_libraries'), placeholder: t('widgets.library.all_libraries'),
    // A library Jellyfin no longer lists is still a page of its own, so it stays here, said to be removed, or the button
    // named it "All libraries".
    load: () => api.get('/libraries', { days: 0 }, { signal: ctx.signal }).then((d) => (d.libraries || [])
      .map((l) => ({ value: l.id, label: l.removed ? `${l.name} · ${t('pages.libraries.removed')}` : l.name, removed: l.removed }))
      .sort((a, b) => (a.removed - b.removed) || a.label.localeCompare(b.label, lang()))),
    onChange: (v) => {
      picked = v;
      changeQuery({ libraries: librariesInAddress(v, ownLibrary) });
      title.textContent = libraryInsightsTitle(count());
      dv.load();
    } });
  const section = h('section', { class: 'subsection stack' },
    h('div', { class: 'makeup-head' }, h('div', null, title, h('p', { class: 'subsection-sub' }, t('widgets.library.about_files'))), picker),
    body);
  dv.load();
  return section;
}

// ---------------------------------------------------------------- /libraries/:id
export function libraryPage(ctx) {
  const id = ctx.params.id;
  ctx.title(t('pages.libraries.library'));
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
      headerSlot.replaceChildren(pageHeader(l.name, [kindLabel(l.collection_type), counts(l), l.size_bytes ? bytes(l.size_bytes) : null, l.removed ? t('pages.libraries.removed') : null].filter(Boolean).join(' · '), libraryArt(l, 480, 'lib-art lib-art-lg')));
      return [
        h('div', { class: 'fui-stat-tile__grid fui-stat-tile__grid--three' },
          h('div', { class: 'fui-stat-tile' }, h('div', { class: 'fui-stat-tile__label' }, t('terms.watch_time')), h('div', { class: 'fui-stat-tile__value' }, durEl(l.watch_s, ''))),
          h('div', { class: 'fui-stat-tile' }, h('div', { class: 'fui-stat-tile__label' }, t('pages.libraries.plays')), h('div', { class: 'fui-stat-tile__value' }, num(l.plays))),
          h('div', { class: 'fui-stat-tile' }, h('div', { class: 'fui-stat-tile__label' }, t('pages.libraries.last_played')), h('div', { class: 'fui-stat-tile__value' }, l.last_played_at ? relEl(l.last_played_at, '') : t('pages.libraries.never')))),
        activityCard({ daily: d.daily, bucket: d.bucket }),
        card({ title: t('pages.libraries.most_watched'), sub: t('pages.libraries.by_watch_time'), body: topList(d.top) }),
        card({ title: t('pages.libraries.recently_added'), body: itemGrid(d.recently_added),
          actions: button({ href: `/libraries/${encodeURIComponent(id)}/titles`, size: 'sm', variant: 'ghost' }, t('pages.libraries.everything_in_it'), icon('chevronRight', 13)) }),
      ];
    },
  });
  ctx.root.append(h('a', { class: 'back-link', href: '/libraries' }, icon('chevronLeft', 14), t('nav.libraries')), headerSlot,
    filterBar({ days, signal: ctx.signal, onDays: (v) => { days = v; saveDays(v); changeQuery({ days }); dv.load(); } }), view, healthSlot(ctx, id), makeupSlot(ctx, id));
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
      return card({ cls: 'health-card', title: t('pages.libraries.health'), body: h('p', null, ...tParts('pages.libraries.health_line', {
        things: h('a', { href: `/server/health?library_id=${encodeURIComponent(libraryId)}` }, t('pages.libraries.things_to_look_at', { count: d.total })), kinds: kinds.join(', ') })) });
    },
  }).load();
  return slot;
}

export function itemGrid(items) {
  if (!items || !items.length) return h('div', { class: 'fui-empty--chart fui-empty--chart-sm' }, t('pages.libraries.nothing_added'));
  return mediaGrid(items.map((it) => mediaCard({ href: `/items/${it.id}`, poster: poster(it.image_item_id || it.id, it.name, { w: 300, cls: 'fui-poster--grid' }),
    name: it.name, sub: [it.sub || it.year, it.date_created ? t('pages.libraries.added', { when: relTime(it.date_created) }) : null].filter(Boolean).join(' · ') })));
}

// ---------------------------------------------------------------- /libraries/:id/titles
// Everything a library holds, not only what arrived lately: a grid of posters by name, searched and sorted on the server,
// sixty at a time, the next sixty arriving as the end of the grid comes into view (or at the press of a button).

const TITLE_KINDS = ['Movie', 'Series', 'MusicAlbum', 'MusicVideo', 'Video', 'Book', 'AudioBook'];
const titleKind = (type) => (TITLE_KINDS.includes(type) ? t(`pages.libraries.title_kind.${type}`) : type);
// Each sort in the direction one means by it: names from A, the rest from the most.
const TITLE_SORTS = [{ value: 'name', label: t('pages.libraries.sort.name'), dir: 'asc' }, { value: 'year', label: t('pages.libraries.sort.year'), dir: 'desc' }, { value: 'added', label: t('pages.libraries.sort.added'), dir: 'desc' }, { value: 'size', label: t('pages.libraries.sort.size'), dir: 'desc' }];
const titlesFilter = (query) => {
  const sort = TITLE_SORTS.find((x) => x.value === query.get('sort')) || TITLE_SORTS[0];
  return { q: query.get('q') || '', type: query.get('type') || '', sort: sort.value, dir: sort.dir };
};
const loadTitles = (libraryId, f, page, signal) => api.get(`/libraries/${encodeURIComponent(libraryId)}/titles`, { q: f.q, type: f.type, sort: f.sort, dir: f.dir, page: page > 1 ? page : '' }, { signal });
export const prefetchLibraryTitles = ({ params, query, signal }) => [() => loadLibrary(params.id, readDays(query), signal), () => loadTitles(params.id, titlesFilter(query), 1, signal)];

function titleSub(x) {
  if (x.type === 'Series') return [x.year, x.episodes != null ? t('terms.episode', { count: x.episodes }) : null].filter(Boolean).join(' · ');
  if (x.type === 'MusicAlbum') return [x.album_artist, x.year].filter(Boolean).join(' · ');
  return [x.year, x.runtime_s ? duration(x.runtime_s) : null].filter(Boolean).join(' · ');
}
const titleCard = (x) => h('li', null, mediaCard({ href: `/items/${x.id}`, poster: poster(x.has_image ? x.image_item_id : null, x.name, { w: 300, cls: 'fui-poster--grid' }), name: x.name, sub: titleSub(x) }));

export function libraryTitlesPage(ctx) {
  const id = ctx.params.id;
  const f = titlesFilter(ctx.query);
  ctx.title(t('pages.libraries.library'));
  const headerSlot = h('div', null, pageHeader(sk.line('200px', 26)));
  const back = h('a', { class: 'back-link', href: `/libraries/${encodeURIComponent(id)}` }, icon('chevronLeft', 14), t('pages.libraries.library'));
  const view = h('div', { class: 'stack' });
  let observer = null;
  ctx.onCleanup(() => observer && observer.disconnect());

  loadLibrary(id, readDays(ctx.query), ctx.signal).then((d) => {
    ctx.title(t('pages.libraries.everything_in', { name: d.library.name }));
    back.lastChild.textContent = d.library.name;
    headerSlot.replaceChildren(pageHeader(d.library.name, t('pages.libraries.everything_in_it')));
  }).catch(() => {});

  function apply() {
    replaceQuery({ q: f.q, type: f.type, sort: f.sort === 'name' ? '' : f.sort });
    dv.load();
  }

  const search = h('input', { class: 'fui-field__input fui-field__input--search', type: 'search', placeholder: t('pages.libraries.find_title'), value: f.q, 'aria-label': t('pages.libraries.find_title_aria'), autocomplete: 'off' });
  const onSearch = debounce(() => { f.q = search.value.trim(); apply(); }, 250);
  search.addEventListener('input', onSearch);
  ctx.onCleanup(() => onSearch.cancel());
  const sortPick = h('div', { class: 'titles-sort' }, segmented({ label: t('pages.libraries.sort_by'), size: 'sm', value: f.sort, options: TITLE_SORTS,
    onChange: (v) => { const x = TITLE_SORTS.find((o) => o.value === v); f.sort = x.value; f.dir = x.dir; apply(); } }));
  const kindSlot = h('div', { class: 'titles-kinds' });
  const count = h('p', { class: 'result-count', 'aria-live': 'polite' });

  const dv = dataView({
    container: view, signal: ctx.signal,
    skeleton: () => h('ul', { class: 'fui-media-card__grid titles-grid' }, Array.from({ length: 12 }, () => h('li', null, sk.block(180)))),
    fetch: () => loadTitles(id, f, 1, ctx.signal),
    render: (d) => {
      if (observer) observer.disconnect();
      count.textContent = t('terms.title', { count: d.total });
      mount(kindSlot, d.types.length > 1 ? segmented({ label: t('pages.libraries.kind_label'), size: 'sm', value: f.type,
        options: [{ value: '', label: t('pages.libraries.all') }, ...d.types.map((x) => ({ value: x.type, label: `${titleKind(x.type)} (${num(x.count)})` }))],
        onChange: (v) => { f.type = v; apply(); } }) : null);
      if (!d.items.length) return emptyState(f.q || f.type ? t('pages.libraries.no_match_title') : t('pages.libraries.nothing_here_title'), f.q || f.type ? t('pages.libraries.no_match_sub') : t('pages.libraries.nothing_here_sub'));
      const grid = h('ul', { class: 'fui-media-card__grid titles-grid' }, d.items.map(titleCard));
      let page = 1, shown = d.items.length, busy = false;
      const more = button({ variant: 'ghost', class: 'titles-more', onClick: () => next() }, t('pages.libraries.show_more'));
      const err = h('div');
      const tail = h('div', { class: 'titles-tail' }, more, err);
      const paintMore = () => { more.hidden = shown >= d.total; more.lastChild.textContent = t('pages.libraries.show_n_more', { count: Math.min(d.page_size, d.total - shown) }); };
      async function next() {
        if (busy || shown >= d.total) return;
        busy = true; setBusy(more, true, t('common.loading')); mount(err);
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
