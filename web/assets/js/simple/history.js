// Simple mode's history table: every play, a row each, the way a classic stats server lists them. Shared by History and by
// the History tab of a library, a person and a title, each narrowing the same list (`fixed`). Sorted, searched and paged
// on the server: it is a paginated list, so the browser never sorts it.

import { h, icon, num, debounce, episodeCode, store } from '../dom.js';
import { api } from '../api.js';
import { can } from '../state.js';
import { dataView, segmented, userCombobox, sk } from '../components.js';
import { dataTable } from '../tables.js';
import { pagination } from '../../finui/components/pagination/pagination.js';

const SIZES = [25, 50, 100];
const SIZE_KEY = 'finstats.simplePageSize';
const KINDS = [{ value: '', label: 'All' }, { value: 'Movie', label: 'Movies' }, { value: 'Episode', label: 'TV Shows' }, { value: 'Audio', label: 'Music' }];
const KIND_ICON = { Movie: 'film', Episode: 'tv', Audio: 'music' };

const two = (n) => String(n).padStart(2, '0');
const day = (t) => { const d = new Date(t * 1000); return `${d.getFullYear()}-${two(d.getMonth() + 1)}-${two(d.getDate())}`; };
const time = (t) => { const d = new Date(t * 1000); return `${two(d.getHours())}:${two(d.getMinutes())}`; };
const mins = (s) => `${Math.round((s || 0) / 60)} mins`;

/** How far a play got, drawn as a circle filled that far: empty, a quarter, half, three quarters, whole. */
function done(c) {
  const el = h('span', { class: 'simple-done', role: 'img', 'aria-label': c == null ? 'Unknown' : `${Math.round(c * 100)}% watched`, title: c == null ? 'Unknown' : `${Math.round(c * 100)}%` });
  el.style.setProperty('--done', `${Math.round(Math.min(1, Math.max(0, c || 0)) * 4) * 25}%`);
  return el;
}

function title(p) {
  const kind = h('span', { class: 'simple-kind', dataset: { type: p.item_type || '' }, 'aria-hidden': 'true' }, icon(KIND_ICON[p.item_type] || 'play', 14));
  const code = episodeCode(p.season_number, p.episode_number);
  const words = p.series_name ? `${p.series_name} - ${p.item_name}${code ? ` (${code})` : ''}` : p.item_name || 'Unknown item';
  return h('span', { class: 'simple-title' }, kind, p.item_exists !== false && p.item_id ? h('a', { href: `/items/${p.item_id}` }, words) : h('span', null, words));
}

/** The table, for one page of rows; `sort: { key, dir, onSort }`. Columns a person may not see are left out. */
function table(rows, sort) {
  const everyone = can('see_everyone'), network = can('see_network');
  const th = (label, key, first = null) => h('th', { 'data-key': key || null, 'data-first': first, 'data-nosort': key ? null : '' }, label);
  return dataTable(h('table', { class: 'fui-data-table fui-data-table--dense fui-data-table--hover simple-table' },
    h('thead', null, h('tr', null, th('Date', 'when'), everyone ? th('User', 'user', 'asc') : null, network ? th('IP Address', 'ip', 'asc') : null,
      th('Platform', 'client', 'asc'), th('Player'), th('Title', 'title', 'asc'), th('Started'), th('Paused'), th('Stopped'), th('Duration', 'watched'), th('', 'progress'))),
    h('tbody', null, rows.map((p) => h('tr', null,
      h('td', { class: 'mono' }, day(p.started_at)),
      everyone ? h('td', null, p.user_id ? h('a', { href: `/users/${p.user_id}` }, p.user_name) : p.user_name) : null,
      network ? h('td', { class: 'mono' }, p.remote_ip || 'n/a') : null,
      h('td', null, p.client || ''), h('td', null, p.device_name || ''),
      h('td', { class: 'simple-table__title' }, title(p)),
      h('td', { class: 'mono' }, time(p.started_at)), h('td', null, mins(p.paused_s)), h('td', { class: 'mono' }, p.active ? 'n/a' : time(p.ended_at)),
      h('td', null, mins(p.duration_s)), h('td', null, done(p.completion)))))), { server: sort, filter: false });
}

/** A history list: `fixed` narrows it for good (a library, a person, a title), `id` names its search field; the filters a reader may
 *  change are the media kind, the search, the person (with see_everyone, when no person is fixed), the order and the page. */
export function historyList(ctx, { id, fixed = {} } = {}) {
  const f = { type: '', q: '', user_id: '', sort: null, dir: null, page: 1, per_page: SIZES.includes(Number(store.get(SIZE_KEY))) ? Number(store.get(SIZE_KEY)) : 25 };
  const body = h('div');
  const view = dataView({
    container: body, signal: ctx.signal,
    skeleton: () => sk.cardRows(10),
    fetch: () => api.get('/activity', { days: 0, ...fixed, type: f.type || undefined, q: f.q || undefined, user_id: fixed.user_id || f.user_id || undefined,
      sort: f.sort || undefined, dir: f.dir || undefined, page: f.page, per_page: f.per_page }, { signal: ctx.signal }),
    render: (d) => {
      const from = d.total ? (f.page - 1) * f.per_page + 1 : 0, to = Math.min(d.total, f.page * f.per_page);
      return [
        d.rows.length ? table(d.rows, { key: f.sort, dir: f.dir, onSort: (key, dir) => { f.sort = key; f.dir = dir; f.page = 1; view.load(); } })
          : h('p', { class: 'simple-empty' }, 'No plays match.'),
        h('div', { class: 'simple-table__foot' }, h('span', { class: 'simple-table__count' }, `Showing ${num(from)} to ${num(to)} of ${num(d.total)} entries`),
          d.total > f.per_page ? pagination({ page: f.page, perPage: f.per_page, total: d.total, onPage: (p) => { f.page = p; view.load(); } }) : null),
      ];
    },
  });
  const size = h('select', { class: 'fui-field__input simple-table__size', 'aria-label': 'Entries per page' }, SIZES.map((n) => h('option', { value: String(n) }, String(n))));
  size.value = String(f.per_page);
  size.addEventListener('change', () => { f.per_page = Number(size.value); store.set(SIZE_KEY, size.value); f.page = 1; view.load(); });
  const search = h('input', { class: 'fui-field__input simple-table__search', id: `${id}-q`, type: 'search', placeholder: 'Search', 'aria-label': 'Search the history', autocomplete: 'off' });
  search.addEventListener('input', debounce(() => { f.q = search.value.trim(); f.page = 1; view.load(); }, 250));
  const who = can('see_everyone') && !fixed.user_id
    ? h('div', { class: 'simple-history__who' }, userCombobox({ value: '', signal: ctx.signal, onChange: (v) => { f.user_id = v; f.page = 1; view.load(); } })) : null;
  const tools = [who, segmented({ label: 'Media', value: f.type, options: KINDS, onChange: (v) => { f.type = v; f.page = 1; view.load(); } })];
  const bar = h('div', { class: 'simple-table__bar' }, h('label', { class: 'simple-table__show' }, 'Show', size, 'entries per page'), h('label', { class: 'simple-table__find' }, 'Search:', search));
  view.load();
  return { tools, el: h('div', { class: 'simple-panel' }, bar, body) };
}
