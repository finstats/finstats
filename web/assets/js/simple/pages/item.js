// Simple mode's page of a title: where it sits (library, show, season), the title over its art with its poster, its facts
// (studio, aired, runtime, rating), its stars, overview, cast and genres; a show's seasons and every episode; how much it
// was watched and by whom; and its own history.

import { h, icon, num, relTime } from '../../dom.js';
import { api, imgItem, isAbort } from '../../api.js';
import { can } from '../../state.js';
import { dataView, poster, avatar, sk, emptyState } from '../../components.js';
import { breadcrumb } from '../../../finui/components/breadcrumb/breadcrumb.js';
import { mediaCard, mediaGrid } from '../../../finui/components/media-card/media-card.js';
import { tileStat, tileStats } from '../../../finui/components/tile-stat/tile-stat.js';
import { statFigure, statFigures, durationMeasure } from '../../../finui/components/stat-figure/stat-figure.js';
import { box } from '../profile.js';
import { historyList } from '../history.js';

export const loadItem = (id, signal) => api.get(`/items/${encodeURIComponent(id)}`, { days: 0 }, { signal });
export const prefetchItem = ({ params, signal }) => [() => loadItem(params.id, signal)];

/** A rating out of ten, as five stars: whole, half or empty. */
function stars(rating) {
  if (rating == null) return null;
  const halves = Math.round(Math.max(0, Math.min(10, rating)));
  return h('span', { class: 'simple-stars', role: 'img', 'aria-label': `Rated ${(rating / 2).toFixed(1)} of 5` },
    [0, 1, 2, 3, 4].map((i) => h('span', { class: ['simple-star', halves >= i * 2 + 2 ? 'is-full' : halves === i * 2 + 1 ? 'is-half' : null] }, '★')));
}

const fact = (label, value) => (value ? h('span', { class: 'simple-hero__fact' }, h('span', null, label), ' ', h('strong', null, value)) : null);

function hero(it, people) {
  const actors = people.filter((p) => p.kind === 'Actor').slice(0, 6);
  const art = it.has_backdrop ? it.id : it.series_id;
  const shown = it.type === 'Episode' && !it.has_image && it.series_id ? it.series_id : it.id;
  return h('section', { class: 'simple-hero' },
    art ? h('img', { class: 'simple-hero__art', src: imgItem(art, 1280, 'backdrop'), alt: '', onError: (e) => e.target.remove() }) : null,
    h('div', { class: 'simple-hero__poster' }, poster(shown, it.name, { w: 400 })),
    h('div', { class: 'simple-hero__text' },
      h('h1', { class: 'simple-hero__title' }, it.name),
      h('div', { class: 'simple-hero__facts' },
        fact('Studio', (it.studios || [])[0]), fact('Aired', it.year ? String(it.year) : null), fact('Runtime', it.runtime_s ? `${Math.round(it.runtime_s / 60)} mins` : null),
        fact('Rated', it.official_rating), stars(it.community_rating)),
      it.overview ? h('p', { class: 'simple-hero__overview' }, it.overview) : null,
      h('div', { class: 'simple-hero__lists' },
        actors.length ? h('div', null, h('h2', null, 'Starring'), h('ul', { class: 'simple-hero__cast' }, actors.map((p) => h('li', null, p.name)))) : null,
        (it.genres || []).length ? h('div', null, h('h2', null, 'Genres'), h('ul', null, it.genres.map((g) => h('li', null, g)))) : null)));
}

function trail(it) {
  const items = [];
  if (it.library_id) items.push({ label: it.library_name || 'Library', href: `/libraries/${it.library_id}` });
  if (it.series_id && it.type !== 'Series') items.push({ label: it.series_name || 'Show', href: `/items/${it.series_id}` });
  items.push({ label: it.type === 'Episode' && it.episode_number != null ? `${it.name}` : it.name });
  return breadcrumb({ items });
}

/** A show's seasons as posters, and every episode under them, with how often each was played. */
function seasons(list) {
  if (!list.length) return [];
  // Each season's card jumps to its episodes on this page. By its place in the list, since a season may have no number.
  const cards = mediaGrid(list.map((s, i) => mediaCard({ href: `#season-${i}`, poster: poster(s.id, s.name, { w: 240, cls: 'fui-poster--grid' }), name: s.name, sub: `${s.episodes.length} episodes` })));
  cards.addEventListener('click', (e) => {
    const a = e.target.closest('a[href^="#season-"]');
    const target = a && document.getElementById(a.getAttribute('href').slice(1));
    if (!target) return;
    e.preventDefault();   // a bare #fragment is a history entry, and the router re-mounts the page on popstate
    target.scrollIntoView({ behavior: matchMedia('(prefers-reduced-motion: reduce)').matches ? 'auto' : 'smooth', block: 'start' });
  });
  const episodes = h('div', { id: 'simple-episodes' }, list.map((s, i) => h('div', { class: 'simple-season', id: `season-${i}` },
    h('h3', null, s.name),
    h('ol', { class: 'simple-season__list' }, s.episodes.map((e) => h('li', { class: 'simple-episode' },
      h('span', { class: 'simple-episode__n' }, e.episode_number != null ? String(e.episode_number) : ''),
      h('a', { href: `/items/${e.id}` }, e.name),
      h('span', { class: 'simple-episode__plays' }, e.plays ? `${num(e.plays)} ${e.plays === 1 ? 'play' : 'plays'}` : ''),
      h('span', { class: 'simple-episode__runtime' }, e.runtime_s ? `${Math.round(e.runtime_s / 60)} mins` : '')))))));
  return [box({ id: 'simple-seasons', title: 'Season List', icon: 'layers', body: cards }), box({ id: 'simple-episode-box', title: 'Episode List', icon: 'tv', body: episodes })];
}

export default function itemPage(ctx) {
  const id = ctx.params.id;
  const history = ctx.query.get('tab') === 'history';
  const head = h('div');
  const body = h('div');
  ctx.root.append(head, body);
  let drawn = null;
  const top = (d) => {
    if (drawn) return;
    drawn = d;
    ctx.title(d.item.name);
    head.replaceChildren(h('div', { class: 'simple-crumbs' }, trail(d.item)), hero(d.item, d.people || []),
      h('nav', { class: 'simple-tabs simple-tabs--page', 'aria-label': 'Views' },
        h('a', { href: `/items/${id}`, 'aria-current': history ? null : 'page' }, 'Profile'), h('a', { href: `/items/${id}?tab=history`, 'aria-current': history ? 'page' : null }, 'History')));
  };
  if (history) {
    // A show's history is its episodes'; anything else is its own plays. One answer names the page and picks the list.
    const missing = () => body.replaceChildren(emptyState('No such title', 'It may have been removed from Jellyfin.'));
    loadItem(id, ctx.signal).then((d) => {
      if (ctx.signal.aborted) return;
      if (!d || !d.item) return missing();
      top(d);
      const list = historyList(ctx, { id: 'simple-item-history', fixed: d.item.type === 'Series' ? { series_id: id } : { item_id: id } });
      body.append(h('div', { class: 'simple-panel-head' }, h('h2', { class: 'simple-page-title' }, 'History'), h('div', { class: 'simple-head__tools' }, list.tools)), list.el);
    }).catch((e) => { if (!ctx.signal.aborted && !isAbort(e)) missing(); });
    return;
  }
  dataView({
    container: body, signal: ctx.signal, skeleton: () => sk.cardRows(6),
    // An address of no title is said to be one, as its History tab and the library and person pages say it.
    fetch: () => loadItem(id, ctx.signal).catch((e) => { if (e.status === 404) return null; throw e; }),
    render: (d) => {
      if (!d || !d.item) return emptyState('No such title', 'It may have been removed from Jellyfin.');
      top(d);
      const t = d.totals || {};
      const figures = box({ id: 'simple-global', title: 'Global Stats', icon: 'trendUp', body: statFigures([
        statFigure({ label: 'All time', measures: [[[num(t.plays || 0), t.plays === 1 ? 'play' : 'plays']], durationMeasure(t.watch_s)] }),
        can('see_everyone') ? statFigure({ label: 'Watched by', measures: [[[num(t.users || 0), t.users === 1 ? 'person' : 'people']]] }) : null,
        statFigure({ label: 'Last played', measures: [[[t.last_played_at ? relTime(t.last_played_at) : 'never', null]]] }),
      ].filter(Boolean)) });
      const watchers = can('see_everyone') && (d.watchers || []).length ? box({ id: 'simple-user-stats', title: 'User Stats', icon: 'users',
        body: tileStats(d.watchers.map((w) => tileStat({ picture: avatar(w.user_id, w.user_name, { size: 80, hasImage: w.has_image }), name: w.user_name, href: `/users/${w.user_id}`, figure: num(w.plays), unit: w.plays === 1 ? 'play' : 'plays' }))) }) : null;
      return [...seasons(d.seasons || []), figures, watchers].filter(Boolean);
    },
  }).load();
}
