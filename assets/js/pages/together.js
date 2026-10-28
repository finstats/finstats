// /together: who watches with whom, how much, and how that has changed. Everything on the page is
// read from one call, /api/stats/groups, which the dashboard and profile cards read too: the server
// scopes it (a viewer who may only see themselves is answered their own pairs and nobody else's
// time alone), the page only draws it.

import { h, num, duration, durationExact, relEl, dateTime, episodeCode, compact, pct } from '../dom.js';
import { api } from '../api.js';
import { readDays, saveDays, can } from '../state.js';
import { replaceQuery } from '../router.js';
import { pageHeader, card, chartCard, filterBar, dataView, sk, statTile, avatar, poster, emptyState } from '../components.js';
import { columnsChart, columnsTable, TOGETHER } from '../charts.js';
import { plainTable } from '../tables.js';
import { titlesList } from '../widgets.js';
import { t } from '../i18n.js';

// Shared with the prefetcher, so a prefetched view has exactly the address the page asks for.
const loadTogether = ({ days, userId }, signal) => api.get('/stats/groups', { days, user_id: userId }, { signal });
const scopeOf = (query) => ({ days: readDays(query), userId: can('see_everyone') ? query.get('user_id') || '' : '' });
export const prefetchTogether = ({ query, signal }) => [() => loadTogether(scopeOf(query), signal)];

const faces = (members) => h('span', { class: 'faces' }, members.map((m) => avatar(m.user_id, m.user_name, { size: 22, hasImage: m.has_image })));
const names = (members, sep) => members.map((m, i) => [i ? sep : '', h('a', { href: `/users/${m.user_id}` }, m.user_name)]);

export default function togetherPage(ctx) {
  ctx.title(t('nav.together'));
  let { days, userId } = scopeOf(ctx.query);
  const view = h('div', { class: 'stack' });
  const everyone = can('see_everyone');

  const dv = dataView({
    container: view, signal: ctx.signal,
    skeleton: () => [sk.tiles(4), sk.cardBlock(232), sk.cardRows(5)],
    fetch: () => loadTogether({ days, userId }, ctx.signal),
    render: (g) => {
      const tot = g.totals || {};
      if (!tot.sessions) return emptyState(t('pages.together.empty_title'), t('pages.together.empty_body'));
      const p = g.previous;
      const vs = days > 0 ? t('pages.together.vs_previous', { count: days }) : '';
      // Without "see everyone" the answer is the caller's own company: the people count is their companions.
      const peopleLabel = everyone ? t('pages.together.people') : t('pages.together.watched_with');
      const peopleValue = everyone ? tot.people : Math.max(0, (tot.people || 1) - 1);
      const tiles = h('div', { class: 'fui-stat-tile__grid' },
        statTile({ label: t('pages.together.time_together'), value: duration(tot.together_s), title: durationExact(tot.together_s), current: tot.together_s, previous: p && p.together_s, vsLabel: vs, hint: ' ' }),
        statTile({ label: t('pages.together.evenings'), value: compact(tot.sessions), title: num(tot.sessions), current: tot.sessions, previous: p && p.sessions, vsLabel: vs, hint: ' ' }),
        statTile({ label: peopleLabel, value: compact(peopleValue), title: num(peopleValue), current: peopleValue, previous: p && (everyone ? p.people : Math.max(0, (p.people || 1) - 1)), vsLabel: vs, hint: ' ' }),
        statTile({ label: t('pages.together.share_of_watch_time'), value: pct(tot.share), current: tot.share, previous: p && p.share, vsLabel: vs, hint: t('pages.together.spent_in_company') }));

      const series = (g.series || []).map((d) => ({ date: d.date, plays: 0, watch_s: (d.together_s || 0) + (d.alone_s || 0), by_type: { together: [0, d.together_s || 0], alone: [0, d.alone_s || 0] } }));
      const overTime = chartCard({ title: t('pages.together.over_time_title'), sub: t(`pages.together.over_time_sub_${g.bucket === 'week' ? 'week' : 'day'}`),
        chart: () => columnsChart({ daily: series, bucket: g.bucket, metric: 'watch_s', series: TOGETHER }),
        table: () => columnsTable({ daily: series, bucket: g.bucket, series: TOGETHER }) });

      const pairs = (g.pairs || []).length ? card({ title: t('pages.together.pairs_title'), sub: t('pages.together.pairs_sub'), cls: 'fui-card--flush',
        body: plainTable(h('table', { class: 'fui-data-table pairs' },
          h('thead', null, h('tr', null, h('th', null, t('pages.together.pair')), h('th', { class: 'r' }, t('pages.together.evenings')), h('th', { class: 'r' }, t('pages.together.time_together')), h('th', null, t('pages.together.last')), h('th', null, t('pages.together.watches_most')))),
          h('tbody', null, g.pairs.map((x) => h('tr', null,
            h('td', null, h('span', { class: 'pair-cell' }, faces(x.members), h('span', null, names(x.members, ' + ')))),
            h('td', { class: 'mono r' }, num(x.sessions)),
            h('td', { class: 'mono r', 'data-sort': x.together_s, title: durationExact(x.together_s) }, duration(x.together_s)),
            h('td', null, relEl(x.last_at)),
            h('td', null, x.top_title ? h('a', { href: `/items/${x.top_title.id}` }, x.top_title.name) : h('span', { class: 'muted' }, '–'))))))) }) : null;

      const titles = (g.titles || []).length ? card({ title: t('pages.together.most_title'), body: titlesList(g.titles, 8) }) : null;

      const recent = (g.recent || []).length ? card({ title: t('pages.together.recent_title'), cls: 'fui-card--flush',
        body: plainTable(h('table', { class: 'fui-data-table evenings' },
          h('thead', null, h('tr', null, h('th', null, t('plays_table.title')), h('th', null, t('pages.together.people')), h('th', null, t('plays_table.when')), h('th', { class: 'r' }, t('pages.together.time_together')))),
          h('tbody', null, g.recent.map((r) => {
            const code = episodeCode(r.season_number, r.episode_number);
            return h('tr', null,
              h('td', null, h('span', { class: 'title-cell' }, poster(r.image_item_id, r.series_name || r.item_name, { w: 120, cls: 'fui-poster--sm' }),
                h('span', null, h('a', { href: `/items/${r.item_id}` }, r.series_name || r.item_name), code ? h('span', { class: 'cell-sub mono' }, `${code} · ${r.item_name}`) : null))),
              h('td', null, h('span', { class: 'pair-cell' }, faces(r.members), h('span', null, names(r.members, ', ')))),
              h('td', { 'data-sort': r.started_at, title: dateTime(r.started_at) }, relEl(r.started_at)),
              h('td', { class: 'mono r', 'data-sort': r.together_s }, duration(r.together_s)));
          })))) }) : null;

      const people = everyone && (g.people || []).length > 1 ? card({ title: t('pages.together.people_title'), sub: t('pages.together.people_sub'), cls: 'fui-card--flush',
        body: plainTable(h('table', { class: 'fui-data-table shares' },
          h('thead', null, h('tr', null, h('th', null, t('pages.together.person')), h('th', { 'data-nosort': '' }, h('span', { class: 'sr-only' }, t('pages.together.share'))), h('th', { class: 'r' }, t('nav.together')), h('th', { class: 'r' }, t('pages.together.alone')), h('th', { class: 'r' }, t('pages.together.share')))),
          h('tbody', null, g.people.map((x) => h('tr', null,
            h('td', null, h('span', { class: 'user-cell' }, avatar(x.user_id, x.user_name, { size: 24, hasImage: x.has_image }), h('a', { href: `/users/${x.user_id}` }, x.user_name))),
            h('td', { class: 'bucket-bar' }, h('span', { class: 'bucket-track' }, x.share ? h('span', { class: 'bucket-fill', style: { width: Math.max(2, x.share * 100) + '%' } }) : null)),
            h('td', { class: 'mono r', 'data-sort': x.together_s }, duration(x.together_s)),
            h('td', { class: 'mono r', 'data-sort': x.alone_s }, duration(x.alone_s)),
            h('td', { class: 'mono r', 'data-sort': x.share }, pct(x.share))))))) }) : null;

      return [tiles, overTime, pairs, h('div', { class: 'grid-2' }, titles, people), recent];
    },
  });

  const sync = () => replaceQuery({ days, user_id: userId });
  ctx.root.append(pageHeader(t('nav.together'), t('pages.together.sub')),
    filterBar({ days, userId, signal: ctx.signal, onDays: (v) => { days = v; saveDays(v); sync(); dv.load(); }, onUser: (v) => { userId = v; sync(); dv.load(); } }),
    view);
  dv.load();
}
