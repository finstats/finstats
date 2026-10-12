// /pipeline: what is coming in. One page, a tab per question: what did people ask for, what airs when,
// what is arriving right now. A tab exists only when a connected service can answer it and the viewer may see it.

import { h, icon, num, store, debounce, duration, relTime, dateTime, pct, bytes, dayLabel, dayLabelLong } from '../dom.js';
import { state, can } from '../state.js';
import { replaceQuery } from '../router.js';
import { pageHeader, card, chartCard, dataView, sk, emptyState, segmented, userCombobox, stickyFilters, statTile, pagination, avatar } from '../components.js';
import { dataTable } from '../tables.js';
import { simpleColumns, simpleColumnsTable, libBucketList } from '../charts.js';
import { loadUpcoming, agenda, upcomingPoster } from '../upcoming.js';
import { loadDownloads, downloadsList, nothingDownloading } from '../downloads.js';
import { follow } from '../live.js';
import { api, qs } from '../api.js';
import { button } from '../../finui/components/button/button.js';
import { t, tParts, lang } from '../i18n.js';

const TABS = [
  { key: 'requests', label: t('pages.pipeline.tabs.requests.label'), feature: 'requests', sub: t('pages.pipeline.tabs.requests.sub') },
  { key: 'upcoming', label: t('pages.pipeline.tabs.upcoming.label'), feature: 'upcoming', sub: t('pages.pipeline.tabs.upcoming.sub') },
  { key: 'downloads', label: t('pages.pipeline.tabs.downloads.label'), feature: 'downloads', perm: 'see_downloads', sub: t('pages.pipeline.tabs.downloads.sub') },
];
const features = () => (state.user && state.user.features) || {};
export const pipelineTabs = () => TABS.filter((t) => features()[t.feature] && (!t.perm || can(t.perm)));
// In the menu as soon as a service can answer something, and always for an administrator, who is the only
// person who can connect one. A page that can only be reached by typing its address is a page nobody finds.
export const hasPipeline = () => pipelineTabs().length > 0 || !!(state.user && state.user.is_admin);

const SPANS = [7, 14, 30, 90].map((value) => ({ value, label: t('pages.pipeline.days', { count: value }) }));
const upcomingScope = (query) => ({
  days: SPANS.some((s) => s.value === Number(query.get('days'))) ? Number(query.get('days')) : Number(store.get('finstats.upcomingDays', 14)) || 14,
  userId: can('see_everyone') ? query.get('user_id') || '' : '',
  mine: query.get('mine') === '1',
});
const tabOf = (query) => (pipelineTabs().find((t) => t.key === query.get('tab')) || pipelineTabs()[0] || {}).key;
// The prefetcher asks for exactly what the page would. (Never anything live: see the Downloads tab.)
export const prefetchPipeline = ({ query, signal }) => {
  const tab = tabOf(query);
  if (tab === 'upcoming') return [() => loadUpcoming(upcomingScope(query), signal)];
  if (tab === 'requests') return [() => loadRequests(requestScope(query), signal)];
  return [];   // the downloads tab is live: asking for it in advance would keep three services busy for a page nobody opened
};

function tabBar(current) {
  const tabs = pipelineTabs();
  if (tabs.length < 2) return null;
  return h('nav', { class: 'fui-segmented entity-tabs', 'aria-label': t('pages.pipeline.sections_aria') }, tabs.map((t) =>
    h('a', { class: 'fui-segmented__option', href: `/pipeline?tab=${t.key}`, 'aria-current': t.key === current ? 'page' : null }, t.label)));
}

// ---------------------------------------------------------------- requests

const PER_PAGE = 25;
const monthShort = new Intl.DateTimeFormat(lang(), { month: 'short' });
const monthLong = new Intl.DateTimeFormat(lang(), { month: 'long', year: 'numeric' });
// A colour always comes with an icon and a word.
const STATE = {
  pending: { cls: 'fui-badge--warning', icon: 'clock', label: t('pages.pipeline.state.pending') },
  approved: { cls: 'fui-badge--info', icon: 'check', label: t('pages.pipeline.state.approved') },
  processing: { cls: 'fui-badge--info', icon: 'download', label: t('pages.pipeline.state.processing') },
  partial: { cls: 'fui-badge--info', icon: 'download', label: t('pages.pipeline.state.partial') },
  available: { cls: 'fui-badge--good', icon: 'check', label: t('pages.pipeline.state.available') },
  declined: { cls: 'fui-badge--critical', icon: 'x', label: t('pages.pipeline.state.declined') },
  failed: { cls: 'fui-badge--critical', icon: 'alert', label: t('pages.pipeline.state.failed') },
  removed: { cls: 'fui-badge--info', icon: 'minus', label: t('pages.pipeline.state.removed') },
};
const STATUSES = [{ value: '', label: t('pages.pipeline.status.all') }, { value: 'open', label: t('pages.pipeline.status.open') }, { value: 'arrived', label: t('pages.pipeline.status.arrived') }, { value: 'declined', label: t('pages.pipeline.status.declined') }];

const requestScope = (query) => ({
  status: STATUSES.some((s) => s.value === query.get('status')) ? query.get('status') : '',
  userId: can('see_everyone') ? query.get('user_id') || '' : '',
  q: query.get('q') || '',
  sort: query.get('sort') || '', dir: query.get('dir') || '',
  page: Math.max(1, Number(query.get('page')) || 1),
});
const loadRequests = (f, signal) => {
  const o = { signal };
  return Promise.all([
    api.get('/requests', { status: f.status, user_id: f.userId, q: f.q, sort: f.sort, dir: f.dir, page: f.page, per_page: PER_PAGE }, o),
    api.get('/requests/summary', { user_id: f.userId }, o),
  ]).then(([list, summary]) => ({ list, summary }));
};

/** What a season request is about: "Season 2", "Seasons 1–3", "Every season". */
function seasonsText(r) {
  const s = Array.isArray(r.seasons) ? r.seasons : [];
  if (r.media_type !== 'tv') return null;
  if (!s.length) return t('pages.pipeline.seasons.every');
  if (s.length === 1) return t('pages.pipeline.seasons.one', { season: s[0] });
  const run = s.every((n, i) => i === 0 || n === s[i - 1] + 1);
  return run ? t('pages.pipeline.seasons.run', { first: s[0], last: s[s.length - 1] }) : t('terms.season', { count: s.length });
}

function titleCell(r) {
  const name = r.title || t('pages.pipeline.unknown_title');
  const inner = [upcomingPoster({ poster: r.poster, title: name, series_title: null }, { w: 96, cls: 'fui-poster--sm' }),
    h('span', { class: 'req-title-text' },
      h('span', { class: 'req-title-name' }, name, r.year ? h('span', { class: 'muted' }, ` (${r.year})`) : null),
      h('span', { class: 'req-title-sub' }, [seasonsText(r), r.is_4k ? '4K' : null].filter(Boolean).join(' · ') || (r.media_type === 'movie' ? t('item_type.Movie') : t('item_type.Series'))))];
  return r.item_id ? h('a', { class: 'req-title', href: `/items/${r.item_id}` }, inner) : h('span', { class: 'req-title' }, inner);
}

function requestsTab(ctx, root) {
  const f = requestScope(ctx.query);
  const view = h('div', { class: 'stack' });
  const apply = (resetPage = true) => {
    if (resetPage) f.page = 1;
    replaceQuery({ tab: 'requests', status: f.status, user_id: f.userId, q: f.q, sort: f.sort, dir: f.dir, page: f.page > 1 ? f.page : '' });
    dv.load();
  };

  const dv = dataView({
    container: view, signal: ctx.signal,
    skeleton: () => [sk.tiles(4), sk.cardBlock(320)],
    fetch: () => loadRequests(f, ctx.signal),
    render: ({ list, summary }) => {
      const tot = summary.totals || {};
      const everyone = can('see_everyone');
      const watched = everyone && tot.watched_by_anyone != null ? tot.watched_by_anyone : tot.watched;
      const table = list.rows.length ? dataTable(h('table', { class: 'fui-data-table requests' },
        h('thead', null, h('tr', null,
          h('th', { 'data-key': 'title' }, t('pages.pipeline.col.title')),
          everyone ? h('th', { 'data-key': 'user' }, t('pages.pipeline.col.asked_by')) : null,
          h('th', { 'data-key': 'when', 'data-first': 'desc' }, t('pages.pipeline.col.asked')),
          h('th', { 'data-key': 'state' }, t('pages.pipeline.col.state')),
          h('th', { 'data-key': 'arrived', 'data-first': 'asc' }, t('pages.pipeline.col.took')),
          h('th', { 'data-key': 'watched' }, t('pages.pipeline.col.watched')))),
        h('tbody', null, list.rows.map((r) => {
          const st = STATE[r.state] || STATE.approved;
          return h('tr', null,
            h('td', null, titleCell(r)),
            everyone ? h('td', null, r.user_id ? h('a', { class: 'user-cell', href: `/users/${r.user_id}` }, avatar(r.user_id, r.user_name, { size: 22, hasImage: r.has_image }), r.user_name || t('common.unknown'))
              : h('span', { class: 'user-cell muted', title: t('pages.pipeline.unlinked_user') }, r.user_name || t('common.unknown'))) : null,
            h('td', { class: 'mono nowrap', title: dateTime(r.requested_at) }, relTime(r.requested_at)),
            h('td', null, h('span', { class: 'fui-badge--status ' + st.cls }, icon(st.icon, 13), st.label)),
            h('td', { class: 'mono nowrap', 'data-sort': r.arrived_after_s == null ? '' : r.arrived_after_s }, r.arrived_after_s == null ? h('span', { class: 'muted' }, '–') : duration(r.arrived_after_s)),
            h('td', null, r.watched ? h('span', { class: 'fui-badge--status fui-badge--good' }, icon('check', 13), everyone ? t('pages.pipeline.watched.by_them') : t('pages.pipeline.watched.yes'))
              : r.watched_by_anyone ? h('span', { class: 'fui-badge--status fui-badge--info' }, icon('users', 13), t('pages.pipeline.watched.by_someone_else'))
              : r.state === 'available' ? h('span', { class: 'muted' }, t('pages.pipeline.watched.not_yet')) : h('span', { class: 'muted' }, '–')));
        }))), { server: { key: f.sort, dir: f.dir, onSort: (key, dir) => { f.sort = key; f.dir = dir; apply(); } } })
        : emptyState(f.q || f.status || f.userId ? t('pages.pipeline.requests_empty_filtered') : t('pages.pipeline.requests_empty'), t('pages.pipeline.requests_empty_sub'));

      const trend = (summary.trend || []).filter((m) => m.median_s != null).map((m) => {
        const d = new Date(`${m.month}-02T00:00:00`);
        return { label: monthShort.format(d), title: `${monthLong.format(d)} · ${t('terms.title', { count: m.arrived })}`, value: Math.round((m.median_s / 3600) * 10) / 10 };
      });
      const never = summary.never_played || [];
      return [
        h('div', { class: 'fui-stat-tile__grid' },
          statTile({ label: t('pages.pipeline.tile.titles'), value: num(tot.titles), hint: tot.requests !== tot.titles ? t('pages.pipeline.tile.requests_in_all', { count: tot.requests }) : ' ' }),
          statTile({ label: t('pages.pipeline.tile.waiting'), value: num(tot.open), hint: tot.open ? t('pages.pipeline.tile.not_here_yet') : t('pages.pipeline.tile.nothing_outstanding') }),
          statTile({ label: t('pages.pipeline.tile.typical_wait'), value: summary.median_arrive_s == null ? '–' : duration(summary.median_arrive_s), hint: t('pages.pipeline.tile.typical_wait_hint') }),
          statTile({ label: t('pages.pipeline.tile.watched_after'), value: tot.arrived ? pct(watched / tot.arrived) : '–', hint: t('pages.pipeline.tile.watched_after_hint', { watched, arrived: tot.arrived }) })),
        trend.length > 1 ? chartCard({ title: t('pages.pipeline.trend.title'), sub: t('pages.pipeline.trend.sub'),
          chart: () => simpleColumns({ rows: trend, unit: (n) => t('pages.pipeline.trend.hours', { count: n }), ariaLabel: t('pages.pipeline.trend.aria') }),
          table: () => simpleColumnsTable({ rows: trend, head: [t('pages.pipeline.trend.month'), t('pages.pipeline.trend.hours_to_arrive')] }) }) : null,
        card({ cls: 'fui-card--flush', body: [table, list.total > PER_PAGE ? pagination({ page: list.page, perPage: list.per_page, total: list.total, onPage: (p) => { f.page = p; apply(false); window.scrollTo({ top: 0 }); } }) : null] }),
        never.length ? card({ title: t('pages.pipeline.never.title'), sub: t('pages.pipeline.never.sub'), cls: 'fui-card--flush',
          body: dataTable(h('table', { class: 'fui-data-table' },
            h('thead', null, h('tr', null, h('th', null, t('pages.pipeline.col.title')), everyone ? h('th', null, t('pages.pipeline.col.asked_by')) : null, h('th', { 'data-first': 'asc' }, t('pages.pipeline.never.here_since')))),
            h('tbody', null, never.map((r) => h('tr', null,
              h('td', null, titleCell(r)),
              everyone ? h('td', null, r.user_name || t('common.unknown')) : null,
              h('td', { class: 'mono nowrap', 'data-sort': r.available_at, title: dateTime(r.available_at) }, relTime(r.available_at))))))) }) : null,
        summary.people && summary.people.length > 1 ? card({ title: t('pages.pipeline.people.title'), sub: t('pages.pipeline.people.sub'), cls: 'fui-card--flush',
          body: dataTable(h('table', { class: 'fui-data-table' },
            h('thead', null, h('tr', null, h('th', null, t('pages.pipeline.people.person')), h('th', { class: 'r', 'data-first': 'desc' }, t('pages.pipeline.people.asked_for')), h('th', { class: 'r' }, t('pages.pipeline.people.arrived')), h('th', { class: 'r' }, t('pages.pipeline.col.watched')))),
            h('tbody', null, summary.people.map((p) => h('tr', null,
              h('td', null, p.user_id ? h('a', { class: 'user-cell', href: `/users/${p.user_id}` }, avatar(p.user_id, p.user_name, { size: 22 }), p.user_name) : h('span', { class: 'muted' }, p.user_name)),
              h('td', { class: 'mono r' }, num(p.requests)), h('td', { class: 'mono r' }, num(p.arrived)),
              h('td', { class: 'mono r' }, p.arrived ? `${num(p.watched)} · ${pct(p.watched / p.arrived)}` : '–')))))) }) : null,
      ];
    },
  });

  const search = h('input', { class: 'fui-field__input fui-field__input--search', type: 'search', placeholder: t('pages.pipeline.search_placeholder'), value: f.q, 'aria-label': t('pages.pipeline.search_aria'), autocomplete: 'off' });
  const onSearch = debounce(() => { f.q = search.value.trim(); apply(); }, 250);
  search.addEventListener('input', onSearch);
  ctx.onCleanup(() => onSearch.cancel());
  root.append(
    stickyFilters([
      segmented({ label: t('pages.pipeline.which_requests'), value: f.status, options: STATUSES, onChange: (v) => { f.status = v; apply(); } }),
      can('see_everyone') ? userCombobox({ value: f.userId, signal: ctx.signal, onChange: (u) => { f.userId = u; apply(); } }) : null,
      h('div', { class: 'fui-field__search' }, icon('search', 14), search)], ctx.signal),
    view);
  dv.load();
}

function upcomingTab(ctx, root) {
  const f = upcomingScope(ctx.query);
  const view = h('div', { class: 'stack' });
  const summary = h('p', { class: 'result-count', 'aria-live': 'polite' });
  // What this list is, as a file to import once: the page's own filters, read by its own route (behind the session).
  const exportHref = () => '/api/upcoming.ics' + qs({ days: f.days, user_id: f.userId, mine: f.mine ? 'true' : '' });
  const exportLink = button({ variant: 'ghost', size: 'sm', href: exportHref(), download: 'finstats-upcoming.ics' }, icon('download', 14), t('pages.pipeline.export_ics'));
  const exportHelp = h('p', { class: 'fui-field__help upcoming-export-help' },
    ...tParts('pages.pipeline.export_help', { link: h('a', { href: '/settings/keys' }, `${t('nav.settings')} → ${t('settings.sections.keys.label')}`) }));
  const apply = () => { replaceQuery({ tab: 'upcoming', days: f.days, user_id: f.userId, mine: f.mine ? '1' : '' }); exportLink.setAttribute('href', exportHref()); dv.load(); };
  const dv = dataView({
    container: view, signal: ctx.signal,
    skeleton: () => [sk.cardRows(4), sk.cardRows(3)],
    fetch: () => loadUpcoming(f, ctx.signal),
    render: (d) => {
      const list = d.entries || [];
      const episodes = list.filter((e) => e.kind === 'episode').length;
      summary.textContent = list.length ? `${t('terms.episode', { count: episodes })} · ${t('pages.pipeline.film_releases', { count: list.length - episodes })}` : '';
      if (!list.length) {
        return emptyState(f.mine ? t('pages.pipeline.upcoming_empty_mine') : t('pages.pipeline.upcoming_empty'),
          f.mine ? t('pages.pipeline.upcoming_empty_mine_sub') : t('pages.pipeline.upcoming_empty_sub'));
      }
      return card({ cls: 'card-agenda', body: agenda(list, { people: can('see_everyone'), watch: true }) });
    },
  });
  const mineBtn = segmented({ label: t('pages.pipeline.which_titles'), size: 'sm', value: f.mine ? 'mine' : 'all',
    options: [{ value: 'all', label: t('pages.pipeline.everything') }, { value: 'mine', label: f.userId ? t('pages.pipeline.only_theirs') : t('pages.pipeline.only_mine') }],
    onChange: (v) => { f.mine = v === 'mine'; apply(); } });
  root.append(
    stickyFilters([
      segmented({ label: t('pages.pipeline.how_far_ahead'), value: f.days, options: SPANS, onChange: (v) => { f.days = v; store.set('finstats.upcomingDays', String(v)); apply(); } }),
      can('see_everyone') ? userCombobox({ value: f.userId, signal: ctx.signal, onChange: (u) => { f.userId = u; apply(); } }) : null,
      mineBtn, exportLink], ctx.signal),
    exportHelp, summary, view);
  dv.load();
}

const HISTORY_SPANS = [...[7, 30, 90].map((value) => ({ value, label: t('pages.pipeline.days', { count: value }) })), { value: 365, label: t('pages.pipeline.a_year') }];

/** What came in over time: the same data as the live list, only after the fact. */
function historySection(ctx) {
  let days = Number(store.get('finstats.grabDays', 30)) || 30;
  const view = h('div', { class: 'stack' });
  const dv = dataView({
    container: view, signal: ctx.signal,
    skeleton: () => sk.cardBlock(220),
    fetch: () => api.get('/downloads/history', { days }, { signal: ctx.signal }),
    render: (d) => {
      const tot = d.totals || {};
      if (!tot.imported && !tot.failed && !tot.grabbed) return null;
      const daily = (d.daily || []).map((x) => ({ label: dayLabel(x.day), title: dayLabelLong(x.day), value: Math.round((x.size_bytes / 1e9) * 10) / 10 }));
      const bucket = (title, sub, rows, unit) => card({ title, sub, body: libBucketList(rows, { unit }) });
      return [
        h('div', { class: 'fui-stat-tile__grid' },
          statTile({ label: t('pages.pipeline.history.arrived'), value: num(tot.imported), hint: days === 365 ? t('pages.pipeline.history.in_the_last_year') : t('pages.pipeline.history.in_the_last_days', { count: days }) }),
          statTile({ label: t('pages.pipeline.history.downloaded'), value: bytes(tot.size_bytes), hint: tot.imported ? t('pages.pipeline.history.on_average', { size: bytes(tot.size_bytes / tot.imported) }) : ' ' }),
          statTile({ label: t('pages.pipeline.history.failed'), value: num(tot.failed), hint: tot.grabbed ? t('pages.pipeline.history.of_grabbed', { grabbed: tot.grabbed }) : ' ' })),
        chartCard({ title: t('pages.pipeline.history.per_day_title'), sub: t('pages.pipeline.history.per_day_sub'),
          chart: () => simpleColumns({ rows: daily, unit: (n) => t('pages.pipeline.history.gb', { count: n }), ariaLabel: t('pages.pipeline.history.per_day_aria') }),
          table: () => simpleColumnsTable({ rows: daily, head: [t('pages.pipeline.history.day'), 'GB'] }) }),
        h('div', { class: 'grid-3' },
          bucket(t('pages.pipeline.history.indexers'), t('pages.pipeline.history.indexers_sub'), d.indexers, t('charts.files')),
          bucket(t('pages.pipeline.history.quality'), t('pages.pipeline.history.quality_sub'), d.quality, t('charts.files')),
          bucket(t('pages.pipeline.history.clients'), t('pages.pipeline.history.clients_sub'), d.clients, t('charts.files'))),
        (d.failures || []).length ? card({ title: t('pages.pipeline.history.failures_title'), sub: t('pages.pipeline.history.failures_sub'), cls: 'fui-card--flush',
          body: dataTable(h('table', { class: 'fui-data-table' },
            h('thead', null, h('tr', null, h('th', { 'data-first': 'desc' }, t('pages.pipeline.history.when')), h('th', null, t('pages.pipeline.col.title')), h('th', null, t('pages.pipeline.history.indexer')))),
            h('tbody', null, d.failures.map((f) => h('tr', null,
              h('td', { class: 'mono nowrap', 'data-sort': f.at, title: dateTime(f.at) }, relTime(f.at)),
              h('td', null, h('span', { class: 'dl-release mono' }, f.source || f.title || t('common.unknown'))),
              h('td', null, f.indexer || h('span', { class: 'muted' }, '–'))))))) }) : null,
      ];
    },
  });
  const el = h('section', { class: 'dl-history' },
    h('div', { class: 'filters' }, h('h2', { class: 'section-title' }, t('pages.pipeline.history.title')),
      segmented({ label: t('pages.pipeline.history.how_far_back'), size: 'sm', value: days, options: HISTORY_SPANS, onChange: (v) => { days = v; store.set('finstats.grabDays', String(v)); dv.load(); } })),
    view);
  dv.load();
  return el;
}

function downloadsTab(ctx, root) {
  const view = h('div');
  let failed = null;
  const dv = dataView({
    container: view, signal: ctx.signal,
    skeleton: () => sk.cardBlock(240),
    fetch: () => loadDownloads(ctx.signal),
    render: (d) => {
      failed = null;
      if (nothingDownloading(d) && !(d.problems || []).length) {
        return emptyState(t('pages.pipeline.downloads_empty'), d.sources ? t('pages.pipeline.downloads_empty_sub') : t('pages.pipeline.downloads_empty_connect'));
      }
      return card({ cls: 'fui-card--flush dl-card', body: h('div', { class: 'dl-wrap' }, downloadsList(d)) });
    },
  });
  root.append(view, historySection(ctx));
  dv.load();
  // A live list while this page is open, and only then: pushed when the queue is read again, or asked for every 5 s
  // while the live channel is not there.
  follow('downloads', {
    poll: async () => { try { await dv.load(); failed = null; } catch (e) { if (!failed) failed = e; } },
    every: 5000, onData: (d) => { dv.show(d); failed = null; }, signal: ctx.signal,
  });
}

export default function pipelinePage(ctx) {
  ctx.title(t('nav.pipeline'));
  const tabs = pipelineTabs();
  if (!tabs.length) {
    ctx.root.append(pageHeader(t('nav.pipeline'), t('pages.pipeline.sub')),
      emptyState(t('pages.pipeline.nothing_connected'), t('pages.pipeline.nothing_connected_sub'),
        state.user && state.user.is_admin ? button({ variant: 'primary', href: '/settings/connections' }, icon('plus', 14), t('pages.pipeline.add_connection')) : null));
    return;
  }
  const current = tabOf(ctx.query);
  const tab = tabs.find((t) => t.key === current);
  ctx.title(`${tab.label} · ${t('nav.pipeline')}`);
  // Native append() prints a missing node as the text "null": with one tab there is no tab bar.
  ctx.root.append(...[pageHeader(t('nav.pipeline'), tab.sub), tabBar(current)].filter(Boolean));
  if (current === 'upcoming') upcomingTab(ctx, ctx.root);
  else if (current === 'requests') requestsTab(ctx, ctx.root);
  else if (current === 'downloads') downloadsTab(ctx, ctx.root);
}
