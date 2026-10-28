import { h, icon, num, durEl, relEl, compact, duration, durationExact } from '../dom.js';
import { api, soft } from '../api.js';
import { readDays, saveDays, can, isAdmin, state } from '../state.js';
import { replaceQuery } from '../router.js';
import { pageHeader, card, filterBar, dataView, sk, avatar, emptyState, topList, chip, playsTable, statTile } from '../components.js';
import { activityCard, heatmapCard, genresCard, groupsCard } from '../widgets.js';
import { bucketList, methodsBar } from '../charts.js';
import { openPlayModal } from '../playmodal.js';
import { profileAllTime } from './showprogress.js';
import { dataTable, plainTable } from '../tables.js';
import { loadUpcoming, agenda } from '../upcoming.js';
import { hasWatchlist } from '../watchlist.js';
import { button } from '../../finui/components/button/button.js';
import { t, tParts } from '../i18n.js';

// Loaders are shared with the prefetcher, so a prefetched view has exactly the address the page asks for.
const loadUsers = (days, signal) => api.get('/users', { days }, { signal });
async function loadUser(id, days, signal) {
  const o = { signal };
  const [detail, recent, groups] = await Promise.all([
    api.get(`/users/${id}`, { days }, o),
    api.get('/activity', { days, user_id: id, page: 1, per_page: 10 }, o),
    soft(api.get('/stats/groups', { days, user_id: id }, o)),
  ]);
  return { detail, recent, groups };
}
export const prefetchUsers = ({ query, signal }) => [() => loadUsers(readDays(query), signal)];
// What is coming for the shows this person watches. Only asked for when a Sonarr is connected.
const hasComing = () => !!(state.user && state.user.features && state.user.features.upcoming);
const loadComing = (id, signal) => soft(loadUpcoming({ days: 30, userId: id, mine: true }, signal));
export const prefetchUser = ({ params, query, signal }) => [() => loadUser(params.id, readDays(query), signal), ...(hasComing() ? [() => loadComing(params.id, signal)] : [])];

// ---------------------------------------------------------------- /users
export function usersPage(ctx) {
  ctx.title(t('nav.users'));
  let days = readDays(ctx.query);
  const view = h('div');
  const dv = dataView({
    container: view, signal: ctx.signal,
    skeleton: () => sk.rows(5),
    fetch: () => loadUsers(days, ctx.signal),
    render: (data) => {
      const users = (data.users || []).slice().sort((a, b) => (b.watch_s || 0) - (a.watch_s || 0));
      if (!users.length) return emptyState(t('pages.users.empty_title'), t('pages.users.empty_sub'));
      return dataTable(h('table', { class: 'fui-data-table fui-data-table--hover' },
        h('thead', null, h('tr', null, h('th', null, t('plays_table.user')), h('th', { class: 'r' }, t('pages.users.plays')), h('th', { class: 'r' }, t('terms.watch_time')),
          h('th', null, t('pages.users.last_played')), h('th', null, t('pages.users.last_title')), h('th', null, t('pages.users.last_client')), h('th', null, t('pages.users.last_seen_jellyfin')))),
        h('tbody', null, users.map((u) => h('tr', { class: u.removed || u.is_disabled ? 'is-dim' : '' },
          h('td', null, h('a', { class: 'user-cell user-cell-lg', href: `/users/${u.id}` }, avatar(u.id, u.name, { size: 30, hasImage: u.has_image }),
            h('span', { class: 'user-ident' }, h('span', { class: 'user-name' }, u.name),
              h('span', { class: 'user-tags' }, u.is_admin ? chip(t('pages.users.admin')) : null, u.is_disabled ? chip(t('pages.users.disabled')) : null, u.removed ? chip(t('pages.users.removed_chip')) : null)))),
          h('td', { class: 'mono r' }, num(u.plays)),
          h('td', { class: 'r' }, durEl(u.watch_s)),
          h('td', null, u.last_played_at ? relEl(u.last_played_at) : h('span', { class: 'muted' }, t('pages.users.never'))),
          h('td', { class: 'trunc-cell' }, u.last_item_name || h('span', { class: 'muted' }, '–')),
          h('td', null, u.last_client || h('span', { class: 'muted' }, '–')),
          h('td', null, u.last_activity_at ? relEl(u.last_activity_at) : h('span', { class: 'muted' }, '–')))))));
    },
  });
  ctx.root.append(pageHeader(t('nav.users'), t('pages.users.sub')),
    filterBar({ days, signal: ctx.signal, onDays: (v) => { days = v; saveDays(v); replaceQuery({ days }); dv.load(); } }),
    card({ cls: 'fui-card--flush', body: view }));
  dv.load();
}

/** Overview | Timeline | Watchlist, under a person's header. Links, so each view has its own address. The watchlist is
 *  only ever one's own, so only one's own profile has the tab. */
export function userTabs(id, current) {
  const tab = (key, label, href) => h('a', { class: 'fui-segmented__option', href, 'aria-current': key === current ? 'page' : null }, label);
  const own = hasWatchlist() && state.user && id === state.user.id;
  return h('nav', { class: 'fui-segmented entity-tabs', 'aria-label': t('pages.users.tabs_label') },
    tab('overview', t('pages.users.tab_overview'), `/users/${id}`), tab('timeline', t('pages.users.tab_timeline'), `/users/${id}/timeline`),
    own ? tab('watchlist', t('terms.watchlist'), `/users/${id}/watchlist`) : null);
}

// ---------------------------------------------------------------- /users/:id
export function userPage(ctx) {
  const id = ctx.params.id;
  ctx.title(t('pages.users.page_title'));
  let days = readDays(ctx.query);
  const headerSlot = h('div');
  const view = h('div', { class: 'stack' });
  const allTime = profileAllTime({ userId: id, signal: ctx.signal });

  // Loads on its own and is slotted into every render below, like the all-time cards. Hidden when there is nothing to wait for.
  const comingBody = h('div');
  const comingCard = card({ title: t('pages.users.coming_up'), sub: t('pages.users.coming_up_sub'), cls: 'card-agenda', body: comingBody,
    actions: button({ variant: 'ghost', size: 'sm', href: `/pipeline?tab=upcoming&mine=1${can('see_everyone') ? `&user_id=${encodeURIComponent(id)}` : ''}` }, t('pages.users.calendar')) });
  comingCard.hidden = true;
  if (hasComing()) {
    dataView({
      container: comingBody, signal: ctx.signal, skeleton: () => null,
      fetch: () => loadComing(id, ctx.signal),
      render: (d) => {
        const list = ((d && d.entries) || []).slice(0, 8);
        comingCard.hidden = !list.length;
        return list.length ? agenda(list) : null;
      },
    }).load();
  }

  const dv = dataView({
    container: view, signal: ctx.signal,
    skeleton: () => [sk.tiles(4), sk.cardBlock(260), h('div', { class: 'grid-2' }, sk.cardRows(4), sk.cardRows(4))],
    fetch: () => loadUser(id, days, ctx.signal),
    render: ({ detail: d, recent, groups }) => {
      const u = d.user;
      ctx.title(u.name);
      headerSlot.replaceChildren(h('header', { class: 'fui-page-header entity-header' },
        avatar(u.id, u.name, { size: 56, hasImage: u.has_image }),
        h('div', null, h('h1', { class: 'fui-page-header__title' }, u.name),
          isAdmin() && state.user && u.id !== state.user.id ? button({ variant: 'ghost', size: 'sm', class: 'entity-action', href: `/recap?user=${encodeURIComponent(u.id)}` }, icon('recap', 14), t('pages.users.open_recap')) : null,
          h('p', { class: 'fui-page-header__sub' }, [u.is_admin ? t('nav.role_admin') : t('pages.users.role_user'), u.is_disabled ? t('pages.users.disabled_lower') : null, u.removed ? t('pages.users.removed') : null].filter(Boolean).join(' · '),
            u.last_activity_at ? [' · ', ...tParts('pages.users.last_seen', { when: relEl(u.last_activity_at, '') })] : null))));
      const tot = d.totals || {};
      return [
        h('div', { class: 'fui-stat-tile__grid' },
          statTile({ label: t('terms.watch_time'), value: duration(tot.watch_s), title: durationExact(tot.watch_s), hint: ' ' }),
          statTile({ label: t('pages.users.plays'), value: compact(tot.plays), title: num(tot.plays), hint: t('pages.users.different_titles', { count: tot.distinct_items }) }),
          statTile({ label: t('pages.users.movies'), value: compact(tot.movies), hint: t('pages.users.plays_hint') }),
          statTile({ label: t('pages.users.episodes'), value: compact(tot.episodes), hint: tot.tracks ? t('pages.users.and_music_plays', { count: tot.tracks }) : t('pages.users.plays_hint') })),
        jellyfinStrip(d.jellyfin),
        activityCard({ daily: d.daily, bucket: d.bucket }),
        h('div', { class: 'grid-2' },
          card({ title: t('pages.users.top_series'), sub: t('pages.users.by_watch_time'), body: topList(d.top_series) }),
          card({ title: t('pages.users.top_movies'), sub: t('pages.users.by_watch_time'), body: topList(d.top_movies) })),
        heatmapCard({ data: d.heatmap, title: t('pages.users.when_watches', { name: u.name }) }),
        h('div', { class: 'grid-2' },
          card({ title: t('pages.users.play_methods'), sub: t('pages.users.share_of_plays'), body: methodsBar(d.methods) }),
          card({ title: t('pages.users.clients'), sub: t('pages.users.by_plays'), body: bucketList(d.clients) })),
        allTime.shows,
        comingCard,
        Array.isArray(d.genres) ? genresCard(d.genres) : null,
        groupsCard(groups, { forUser: id }),
        card({ title: t('pages.users.devices'), cls: 'fui-card--flush', body: devicesTable(d.devices) }),
        can('see_network') ? card({ title: t('pages.users.ip_addresses'), sub: t('pages.users.ip_addresses_sub'), cls: 'fui-card--flush', body: ipsTable(d.ips) }) : null,
        card({ title: t('pages.users.recent_plays'), cls: 'fui-card--flush',
          actions: button({ variant: 'ghost', size: 'sm', href: `/activity?user_id=${encodeURIComponent(id)}&days=${days}` }, t('pages.users.view_all')),
          body: playsTable(recent.rows, { showUser: false, onOpen: (p) => openPlayModal(p, { onDeleted: () => dv.load() }), empty: t('plays_table.empty_range') }) }),
      ];
    },
  });

  // Streaks and show progress cover all time and load on their own. The streaks sit under the header;
  // the shows card is slotted into the page further down (the same node on every re-render).
  ctx.root.append(headerSlot, userTabs(id, 'overview'), allTime.tiles, filterBar({ days, signal: ctx.signal, onDays: (v) => { days = v; saveDays(v); replaceQuery({ days }); dv.load(); } }), view);
  headerSlot.append(h('header', { class: 'fui-page-header entity-header' }, h('span', { class: 'fui-skeleton', style: { width: '56px', height: '56px', borderRadius: '50%' } }), h('div', null, sk.line('180px', 26))));
  dv.load();
}

/** Jellyfin's own flags: they cover history from before FinStats was installed. */
function jellyfinStrip(j) {
  if (!j) return null;
  const cell = (k, v) => h('div', null, h('dt', null, k), h('dd', { class: 'mono' }, num(v)));
  return h('section', { class: 'strip', 'aria-label': t('pages.users.in_jellyfin') },
    h('h2', { class: 'strip-title' }, t('pages.users.in_jellyfin')),
    h('dl', { class: 'strip-facts' }, cell(t('pages.users.movies_played'), j.played_movies), cell(t('pages.users.episodes_played'), j.played_episodes), cell(t('pages.users.favourites'), j.favorites)),
    h('p', { class: 'strip-note' }, t('pages.users.jellyfin_note')));
}

function devicesTable(devices) {
  if (!devices || !devices.length) return emptyState(t('pages.users.no_devices'));
  return plainTable(h('table', { class: 'fui-data-table' },
    h('thead', null, h('tr', null, h('th', null, t('pages.users.device')), h('th', null, t('plays_table.client')), h('th', null, t('pages.users.version')), h('th', { class: 'r' }, t('pages.users.plays')), h('th', null, t('pages.users.last_used')))),
    h('tbody', null, devices.map((d) => h('tr', null,
      h('td', null, d.device_name || '–'), h('td', null, d.client || '–'), h('td', { class: 'mono' }, d.app_version || '–'),
      h('td', { class: 'mono r' }, num(d.plays)), h('td', null, relEl(d.last_seen)))))));
}

function ipsTable(ips) {
  if (!ips || !ips.length) return emptyState(t('pages.users.no_ips'));
  return plainTable(h('table', { class: 'fui-data-table' },
    h('thead', null, h('tr', null, h('th', null, t('plays_table.ip_address')), h('th', null, t('pages.users.network')), h('th', { class: 'r' }, t('pages.users.plays')), h('th', null, t('pages.users.first_seen')), h('th', null, t('pages.users.last_seen_head')))),
    h('tbody', null, ips.map((ip) => h('tr', null,
      h('td', { class: 'mono' }, ip.ip), h('td', null, chip(ip.is_local ? t('pages.users.local') : t('pages.users.remote'))),
      h('td', { class: 'mono r' }, num(ip.plays)), h('td', null, relEl(ip.first_seen)), h('td', null, relEl(ip.last_seen)))))));
}
