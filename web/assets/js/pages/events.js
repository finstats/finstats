import { h, icon, debounce, dateTime, relTime } from '../dom.js';
import { api } from '../api.js';
import { replaceQuery, navigate } from '../router.js';
import { card, dataView, sk, pagination, emptyState } from '../components.js';
import { dataTable } from '../tables.js';
import { t } from '../i18n.js';

const PER_PAGE = 50;
// Status colors always ship with an icon + label.
const SEVERITY = {
  error: { cls: 'fui-badge--critical', icon: 'alert', label: t('pages.events.level.error') }, critical: { cls: 'fui-badge--critical', icon: 'alert', label: t('pages.events.level.critical') },
  fatal: { cls: 'fui-badge--critical', icon: 'alert', label: t('pages.events.level.fatal') },
  warn: { cls: 'fui-badge--warning', icon: 'alert', label: t('pages.events.level.warning') }, warning: { cls: 'fui-badge--warning', icon: 'alert', label: t('pages.events.level.warning') },
  information: { cls: 'fui-badge--info', icon: 'info', label: t('pages.events.level.info') }, info: { cls: 'fui-badge--info', icon: 'info', label: t('pages.events.level.info') },
  debug: { cls: 'fui-badge--info', icon: 'info', label: t('pages.events.level.debug') }, trace: { cls: 'fui-badge--info', icon: 'info', label: t('pages.events.level.trace') },
};

// Shared with the prefetcher, so a prefetched view has exactly the address the page asks for.
const loadEvents = (f, signal) => api.get('/events', { ...f, per_page: PER_PAGE }, { signal });
const filtersOf = (query) => ({ q: query.get('q') || '', type: query.get('type') || '', sort: query.get('sort') || '', dir: query.get('dir') || '', page: Math.max(1, Number(query.get('page')) || 1) });
export const prefetchEvents = ({ query, signal }) => [() => loadEvents(filtersOf(query), signal)];

/** `/events` was the log's own page; it lives under Server now, and the old address forwards with its filters. */
export default function events() {
  navigate('/server/log' + location.search, { replace: true, scroll: false });
}

/** The log itself: filters, count and the table. Mounted by the Server page's Log section. */
export function logView(ctx) {
  const f = filtersOf(ctx.query);
  const view = h('div');
  const summary = h('p', { class: 'result-count', 'aria-live': 'polite' });
  const typeSlot = h('span');

  function paintType() {
    typeSlot.replaceChildren(f.type ? h('span', { class: 'fui-chip fui-chip--removable' }, t('pages.events.type_chip', { type: f.type }),
      h('button', { type: 'button', class: 'fui-chip__x', 'aria-label': t('pages.events.remove_type'), onClick: () => { f.type = ''; apply(); } }, icon('x', 12))) : '');
  }
  function apply(reset = true) {
    if (reset) f.page = 1;
    paintType();
    replaceQuery({ q: f.q, type: f.type, sort: f.sort, dir: f.dir, page: f.page > 1 ? f.page : '' });
    dv.load();
  }

  const dv = dataView({
    container: view, signal: ctx.signal,
    skeleton: () => sk.tableRows(5),
    fetch: () => loadEvents(f, ctx.signal),
    render: (data) => {
      summary.textContent = t('pages.events.entries', { count: data.total });
      if (!data.rows.length) return emptyState(f.q || f.type ? t('pages.events.empty_filtered') : t('pages.events.empty'), f.q || f.type ? null : t('pages.events.empty_sub'));
      return [dataTable(h('table', { class: 'fui-data-table events' },
        h('thead', null, h('tr', null, h('th', { 'data-key': 'when', 'data-first': 'desc' }, t('pages.events.col.when')), h('th', { 'data-nosort': '' }, t('pages.events.col.level')), h('th', { 'data-key': 'event' }, t('pages.events.col.event')), h('th', { 'data-key': 'user' }, t('pages.events.col.user')), h('th', { 'data-key': 'type' }, t('pages.events.col.type')))),
        h('tbody', null, data.rows.map((e) => {
          const sev = SEVERITY[String(e.severity || '').toLowerCase()] || SEVERITY.info;
          return h('tr', null,
            h('td', { class: 'mono nowrap', title: dateTime(e.date) }, relTime(e.date)),
            h('td', null, h('span', { class: 'fui-badge--status ' + sev.cls }, icon(sev.icon, 13), sev.label)),
            h('td', null, h('div', { class: 'event-name' }, e.name), e.overview ? h('div', { class: 'event-overview' }, e.overview) : null),
            h('td', null, e.user_id ? h('a', { href: `/users/${e.user_id}` }, e.user_name || t('pages.events.col.user')) : h('span', { class: 'muted' }, '–')),
            h('td', null, e.type ? h('button', { type: 'button', class: 'fui-chip fui-chip--button mono', title: t('pages.events.only_this_type'), onClick: () => { f.type = e.type; apply(); } }, e.type) : null));
        }))), { server: { key: f.sort, dir: f.dir, onSort: (key, dir) => { f.sort = key; f.dir = dir; apply(); } } }),
        data.total > PER_PAGE ? pagination({ page: data.page || f.page, perPage: data.per_page || PER_PAGE, total: data.total, onPage: (p) => { f.page = p; apply(false); window.scrollTo({ top: 0 }); } }) : null];
    },
  });

  const search = h('input', { class: 'fui-field__input fui-field__input--search', type: 'search', placeholder: t('pages.events.search_placeholder'), value: f.q, 'aria-label': t('pages.events.search_aria'), autocomplete: 'off' });
  const onSearch = debounce(() => { f.q = search.value.trim(); apply(); }, 250);
  search.addEventListener('input', onSearch);
  ctx.onCleanup(() => onSearch.cancel());

  paintType();
  dv.load();
  return [h('div', { class: 'filters' }, h('div', { class: 'fui-field__search' }, icon('search', 14), search), typeSlot),
    summary, card({ cls: 'fui-card--flush', id: 'log', body: view })];
}
