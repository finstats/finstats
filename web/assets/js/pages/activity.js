import { h, icon, debounce, num, mount, TRACKERS } from '../dom.js';
import { api } from '../api.js';
import { readDays, saveDays, rangeLong, can } from '../state.js';
import { replaceQuery } from '../router.js';
import { pageHeader, card, filterBar, dataView, sk, playsTable, pagination, multiSelect } from '../components.js';
import { openPlayModal } from '../playmodal.js';

// Ticked, not chosen: films *and* episodes without music is a question worth being able to ask.
// "All" is nothing ticked rather than one more thing to tick, so it is left out of these.
const METHODS = [
  { value: 'DirectPlay', label: 'Direct play' }, { value: 'DirectStream', label: 'Direct stream' }, { value: 'Transcode', label: 'Transcode' },
];
const TYPES = [
  { value: 'Movie', label: 'Movies' }, { value: 'Episode', label: 'Episodes' }, { value: 'Audio', label: 'Music' }, { value: 'Other', label: 'Other' },
];
// Which tracker a play came from. Only offered for the ones this history actually holds, which the
// answer lists: an install that has never imported anything has nothing to choose between.
const SOURCES = TRACKERS;
const PER_PAGE = 50;

export const loadActivity = (f, signal) => api.get('/activity', { ...f, per_page: PER_PAGE }, { signal });
export const prefetchActivity = ({ query, signal }) => [() => loadActivity(filtersOf(query), signal)];

function filtersOf(q0) {
  return {
    days: readDays(q0),
    user_id: can('see_everyone') ? q0.get('user_id') || '' : '',
    method: q0.get('method') || '',
    type: q0.get('type') || '',
    q: q0.get('q') || '',
    item_id: q0.get('item_id') || '',
    series_id: q0.get('series_id') || '',
    source: q0.get('source') || '',
    sort: q0.get('sort') || '',
    dir: q0.get('dir') || '',
    page: Math.max(1, Number(q0.get('page')) || 1),
  };
}

export default function activity(ctx) {
  ctx.title('Activity');
  const f = filtersOf(ctx.query);

  const view = h('div');
  const summary = h('p', { class: 'result-count', 'aria-live': 'polite' });
  const dv = dataView({
    container: view, signal: ctx.signal,
    skeleton: () => sk.tableRows(5),
    fetch: () => loadActivity(f, ctx.signal),
    render: (data) => {
      summary.textContent = `${num(data.total)} ${data.total === 1 ? 'play' : 'plays'} · ${rangeLong(f.days).toLowerCase()}`;
      renderSources(data.sources);
      return [
        playsTable(data.rows, { showUser: can('see_everyone'), sort: { key: f.sort, dir: f.dir, onSort: (key, dir) => { f.sort = key; f.dir = dir; apply(); } }, onOpen: (p) => openPlayModal(p, { onDeleted: () => dv.load() }),
          empty: 'No plays match these filters. Try a longer range or clear the search.' }),
        data.total > PER_PAGE ? pagination({ page: data.page || f.page, perPage: data.per_page || PER_PAGE, total: data.total,
          onPage: (p) => { f.page = p; apply(false); window.scrollTo({ top: 0 }); } }) : null,
      ];
    },
  });

  function apply(resetPage = true) {
    if (resetPage) f.page = 1;
    replaceQuery({ ...f, page: f.page > 1 ? f.page : '' });
    dv.load();
  }

  const search = h('input', { class: 'input input-search', type: 'search', placeholder: 'Search titles…', value: f.q, 'aria-label': 'Search titles', autocomplete: 'off' });
  const onSearch = debounce(() => { f.q = search.value.trim(); apply(); }, 250);
  search.addEventListener('input', onSearch);
  ctx.onCleanup(() => onSearch.cancel());

  // Built from the answer rather than up front, because only the server knows what this history is
  // made of — and rebuilt only when that changes, so tabbing to the control does not lose the focus.
  const sourceSlot = h('div', { class: 'seg-slot' });
  let sourceSig = null;
  function renderSources(present) {
    const held = (Array.isArray(present) ? present : []).filter((s) => SOURCES[s]);
    // Whatever is being filtered on stays offered even where there is none of it, so that a filter
    // somebody arrived with in the address can be seen and cleared rather than silently emptying
    // the page.
    const asked = f.source.split(',').map((x) => x.trim()).filter((x) => SOURCES[x]);
    const list = [...held, ...asked.filter((x) => !held.includes(x))];
    const sig = list.join();
    if (sig === sourceSig) return;
    sourceSig = sig;
    // One tracker, or none: nothing to choose between, so no filter at all.
    mount(sourceSlot, list.length < 2 ? null
      : multiSelect({ label: 'Recorded by', allLabel: 'All trackers', options: list.map((v) => ({ value: v, label: SOURCES[v] })),
        value: f.source, onChange: (v) => { f.source = v; apply(); } }));
  }

  const scopeChip = f.item_id || f.series_id ? h('span', { class: 'fui-chip fui-chip--removable' }, f.series_id ? 'One series' : 'One title',
    h('button', { type: 'button', class: 'fui-chip__x', 'aria-label': 'Remove title filter', onClick: (e) => { f.item_id = ''; f.series_id = ''; e.target.closest('.fui-chip').remove(); apply(); } }, icon('x', 12))) : null;

  const filters = filterBar({ days: f.days, userId: f.user_id, signal: ctx.signal,
    onDays: (v) => { f.days = v; saveDays(v); apply(); },
    onUser: (v) => { f.user_id = v; apply(); },
    extra: [
      multiSelect({ label: 'Play method', allLabel: 'All methods', options: METHODS, value: f.method, onChange: (v) => { f.method = v; apply(); } }),
      multiSelect({ label: 'Media type', allLabel: 'All types', options: TYPES, value: f.type, onChange: (v) => { f.type = v; apply(); } }),
      sourceSlot,
      h('div', { class: 'search-field' }, icon('search', 14), search),
      scopeChip,
    ] });

  ctx.root.append(pageHeader('Activity', 'Every play finstats knows about. Newest first, or click a column to sort by it'), filters, summary,
    card({ cls: 'fui-card--flush', body: view }));
  dv.load();
}
