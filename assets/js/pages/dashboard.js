import { h, icon, relEl, episodeCode } from '../dom.js';
import { api, soft } from '../api.js';
import { state, readDays, saveDays, can } from '../state.js';
import { replaceQuery } from '../router.js';
import { pageHeader, card, filterBar, dataView, sk, topList, playsTable, emptyState, poster } from '../components.js';
import { activityCard, heatmapCard, overviewTiles, nowPlayingSection, insightTiles, genresCard, failedLoginsCard, shelfRow } from '../widgets.js';
import { openPlayModal } from '../playmodal.js';
import { recapBanner } from './recap.js';
import { loadUpcoming, entryCard, entryTitle, entryName } from '../upcoming.js';
import { loadDownloads, downloadsList, nothingDownloading } from '../downloads.js';
import { follow } from '../live.js';
import { groupsCard } from '../widgets.js';
import { watchToggle, watchable } from '../watchlist.js';
import { button } from '../../finui/components/button/button.js';
import { typed } from '../menus.js';
import { t } from '../i18n.js';

// Coming up: the same loader for the card and the prefetcher. Nothing is asked when no Sonarr or Radarr is connected.
const hasComing = () => !!(state.user && state.user.features && state.user.features.upcoming);
const hasDownloads = () => !!(state.user && state.user.features && state.user.features.downloads && can('see_downloads'));
const loadComing = (signal) => soft(loadUpcoming({ days: 7 }, signal));

/** Everything the filter row scopes. Shared with the prefetcher, so both ask for exactly the same URLs. */
export async function loadDashboard({ days, userId }, signal) {
  const f = { days, user_id: userId };
  const o = { signal };
  const admin = can('see_everyone');
  const [overview, series, movies, users, heat, recent, insights, groups] = await Promise.all([
    api.get('/stats/overview', f, o),
    api.get('/stats/top', { ...f, kind: 'series', limit: 5 }, o),
    api.get('/stats/top', { ...f, kind: 'movies', limit: 5 }, o),
    admin && !userId ? api.get('/stats/top', { ...f, kind: 'users', limit: 5 }, o) : api.get('/stats/top', { ...f, kind: 'music', limit: 5 }, o),
    api.get('/stats/heatmap', f, o),
    api.get('/activity', { ...f, page: 1, per_page: 8 }, o),
    soft(api.get('/stats/insights', f, o)), // optional: the page works without it
    soft(api.get('/stats/groups', f, o)),
  ]);
  return { overview, series, movies, users, heat, recent, insights, groups };
}
const scopeOf = (query) => ({ days: readDays(query), userId: can('see_everyone') ? query.get('user_id') || '' : '' });
/** Newest arrivals in the libraries: the same for every range and person, so it loads apart from the filters. */
const loadRecent = (signal) => soft(api.get('/library/recent', { limit: 30 }, { signal }));
export const prefetchDashboard = ({ query, signal }) => [() => loadDashboard(scopeOf(query), signal), () => loadRecent(signal), ...(hasComing() ? [() => loadComing(signal)] : [])];

export default function dashboard(ctx) {
  ctx.title(t('nav.dashboard'));
  let { days, userId } = scopeOf(ctx.query);
  const admin = can('see_everyone'); // sees the whole server rather than only themselves

  // ---- now playing (live; not scoped by the filters below)
  const nowPlaying = nowPlayingSection(ctx);

  // ---- recently added: about the library, not the plays, so no filter applies and it loads on its own. The card is one node
  // that every render of the filtered view below slots in above the Activity chart. Hides itself on empty libraries.
  const shelfBody = h('div');
  const shelfArrows = h('span', { class: 'shelf-arrows', hidden: true });
  const shelfCard = card({ title: t('pages.dashboard.recent.title'), sub: t('pages.dashboard.recent.sub'), cls: 'shelf-section', actions: shelfArrows, body: shelfBody });
  shelfCard.hidden = true;
  dataView({
    container: shelfBody, signal: ctx.signal,
    skeleton: () => null,
    fetch: () => loadRecent(ctx.signal),
    render: (d) => {
      const items = (d && d.items) || [];
      shelfCard.hidden = !items.length;
      return items.length ? shelf(items, shelfArrows) : null;
    },
  }).load();

  // ---- downloading now: live, and only for people who may see it. Polled while this page is open.
  const dlBody = h('div');
  const dlCard = card({ title: t('pages.dashboard.downloads.title'), sub: t('pages.dashboard.downloads.sub'), cls: 'dl-card',
    actions: button({ variant: 'ghost', size: 'sm', href: '/pipeline?tab=downloads' }, t('pages.dashboard.downloads.all')), body: dlBody });
  dlCard.hidden = true;
  if (hasDownloads()) {
    const dlView = dataView({
      container: dlBody, signal: ctx.signal, skeleton: () => null,
      fetch: () => loadDownloads(ctx.signal),
      render: (d) => {
        dlCard.hidden = nothingDownloading(d);
        return nothingDownloading(d) ? null : downloadsList(d, { limit: 5 });
      },
    });
    dlView.load();
    follow('downloads', { poll: () => dlView.load(), every: 5000, onData: (d) => dlView.show(d), signal: ctx.signal });
  }

  // ---- coming up: what Sonarr and Radarr expect this week. About the calendar, not about plays: no filter applies. Hides
  // itself when nothing is connected or nothing is due.
  const comingBody = h('div');
  const comingArrows = h('span', { class: 'shelf-arrows', hidden: true });
  const comingCard = card({ title: t('pages.dashboard.coming.title'), sub: t('pages.dashboard.coming.sub'), cls: 'shelf-section', body: comingBody,
    actions: [comingArrows, button({ variant: 'ghost', size: 'sm', href: '/pipeline?tab=upcoming' }, t('pages.dashboard.coming.calendar'))] });
  comingCard.hidden = true;
  if (hasComing()) {
    dataView({
      container: comingBody, signal: ctx.signal,
      skeleton: () => null,
      fetch: () => loadComing(ctx.signal),
      render: (d) => {
        const list = ((d && d.entries) || []).slice(0, 30);
        comingCard.hidden = !list.length;
        // The toggle beside each card's link, as on Recently added and Pipeline's Upcoming.
        return list.length ? shelfRow(list.map((e) => [entryCard(e), watchToggle(entryTitle(e), { compact: true, name: entryName(e) })]), comingArrows, t('pages.dashboard.coming.aria')) : null;
      },
    }).load();
  }

  // ---- everything the filter row scopes
  const view = h('div', { class: 'stack' });
  const dv = dataView({
    container: view, signal: ctx.signal,
    skeleton: () => [sk.tiles(4), sk.cardBlock(260), h('div', { class: 'grid-3' }, sk.cardRows(5), sk.cardRows(5), sk.cardRows(5))],
    fetch: () => loadDashboard({ days, userId }, ctx.signal),
    render: ({ overview, series, movies, users, heat, recent, insights, groups }) => {
      const nothingYet = days === 0 && !userId && !(overview.totals && overview.totals.plays);
      if (nothingYet) {
        return [emptyState(t('pages.dashboard.empty.title'),
          can('manage') ? t('pages.dashboard.empty.manager')
                : t('pages.dashboard.empty.viewer'),
          can('manage') ? h('div', { class: 'fui-empty__buttons' },
            button({ variant: 'primary', href: '/settings/import#import-jellystat' }, icon('upload', 14), t('pages.dashboard.empty.import_jellystat')),
            button({ href: '/settings/import#import-streamystats' }, icon('upload', 14), t('pages.dashboard.empty.import_streamystats'))) : null), dlCard, shelfCard, comingCard];
      }
      const usersMode = admin && !userId;
      return [
        overviewTiles({ totals: overview.totals, previous: overview.previous, daily: overview.daily, days, scoped: !admin || !!userId }),
        insightTiles(insights),
        dlCard,
        shelfCard,
        comingCard,
        activityCard({ daily: overview.daily, bucket: overview.bucket }),
        h('div', { class: 'grid-3' },
          card({ title: t('pages.dashboard.top.series'), sub: t('widgets.by_watch_time'), body: topList(series.rows) }),
          card({ title: t('pages.dashboard.top.movies'), sub: t('widgets.by_watch_time'), body: topList(movies.rows) }),
          usersMode ? card({ title: t('pages.dashboard.top.users'), sub: t('widgets.by_watch_time'), body: topList(users.rows, { kind: 'users' }) })
                    : card({ title: t('pages.dashboard.top.music'), sub: t('widgets.by_watch_time'), body: topList(users.rows, { empty: t('pages.dashboard.top.no_music') }) })),
        insights ? h('div', { class: 'grid-heat' }, heatmapCard({ data: heat }), genresCard(insights.genres)) : heatmapCard({ data: heat }),
        // One person's companions only when one person is picked: several (`a,b`) are a group, not somebody to be with.
        groupsCard(groups, { forUser: (userId && !userId.includes(',') ? userId : null) || (admin ? null : state.user.id) }),
        insights ? failedLoginsCard(insights.failed_logins) : null,
        card({ title: t('pages.dashboard.recent_activity'), actions: button({ variant: 'ghost', size: 'sm', href: '/activity' }, t('pages.dashboard.view_all'), icon('chevronRight', 14)),
          cls: 'fui-card--flush', body: playsTable(recent.rows, { showUser: admin, onOpen: (p) => openPlayModal(p, { onDeleted: () => dv.load() }), empty: t('plays_table.empty_range') }) }),
      ];
    },
  });

  const sync = () => replaceQuery({ days, user_id: userId });
  const filters = filterBar({ days, userId, signal: ctx.signal,
    onDays: (v) => { days = v; saveDays(v); sync(); dv.load(); },
    onUser: (v) => { userId = v; sync(); dv.load(); } });

  // Native append() prints a missing node as the text "null", so absent pieces are filtered out.
  ctx.root.append(...[
    pageHeader(t('nav.dashboard'), state.status && state.status.server_name ? t('pages.dashboard.sub', { server: state.status.server_name }) : t('pages.dashboard.sub_default')),
    recapBanner(),
    nowPlaying.el,
    filters, view].filter(Boolean));

  nowPlaying.start();
  dv.load();
}

/** Recently added: one card per arrival. */
function shelf(items, arrows) {
  return shelfRow(items.map((it) => [h('a', { class: 'shelf-card', href: `/items/${it.id}`, dataset: typed(it.kind === 'episodes' ? 'Series' : it.type) },
    poster(it.image_item_id || it.id, it.name, { w: 300, cls: 'fui-poster--grid' }),
    h('span', { class: 'shelf-when' }, relEl(it.added_at, '')),
    h('span', { class: 'shelf-name' }, it.name),
    it.sub ? h('span', { class: 'shelf-sub' }, it.sub) : null,
    h('span', { class: 'shelf-sub' }, whatArrived(it))),
    // Beside the card's link, not inside it: a button in a link is two controls in one. Episodes fold into their show,
    // and the show is what goes on a list.
    it.kind === 'episodes' || watchable(it.type) ? watchToggle({ item_id: it.id }, { compact: true, name: it.name }) : null]), arrows, t('pages.dashboard.recent.aria'));
}

const ARRIVED = ['Movie', 'MusicAlbum', 'MusicVideo', 'Video', 'Book', 'AudioBook'];
function whatArrived(it) {
  if (it.kind !== 'episodes') return ARRIVED.includes(it.type) ? t(`pages.dashboard.arrived.${it.type}`) : it.type;
  if (it.episodes === 1) return it.episode_number != null ? t('pages.dashboard.arrived.episode_number', { number: it.episode_number }) : t('terms.episode', { count: 1 });
  return t('terms.episode', { count: it.episodes });
}

