// FinUI: data-table. Sorting and filtering for every table in the app.
//
// sortable(table) turns the column headers into sort buttons. It reads what a cell *means*, not what
// it says: "3d 2h", "1.4 GB", "42%", "2 hours ago" and "1,204" all sort by their value. A cell can
// state its own value with data-sort; a header opts out with data-nosort. Empty cells go last in
// either direction, and a third click gives the original order back.
//
// Paginated tables cannot be sorted in the browser — that would only shuffle the page on screen — so
// they pass `server: {key, dir, onSort}` and mark their headers with data-key; the click is handed
// to the page, which asks the server.

import { h, icon } from '../../core.js';

// The separators the app itself prints with (`num()` in FinUI's format.js is en-US), not the browser's: read with a
// German or Norwegian locale's, "1,204" was 1.204 and "12,345,678" was not a number at all.
const GROUP_RE = /[,\s\u00a0\u202f]/g;
const collator = new Intl.Collator(undefined, { numeric: true, sensitivity: 'base' });

const BYTES = { B: 1, KB: 1024, MB: 1024 ** 2, GB: 1024 ** 3, TB: 1024 ** 4, PB: 1024 ** 5 };
const SCALE = { K: 1e3, M: 1e6, B: 1e9 };

function plainNumber(text) {
  const t = text.replace(GROUP_RE, '');
  return /^[-+−]?\d+(\.\d+)?$/.test(t) ? Number(t.replace('−', '-')) : null;
}

/** What a piece of cell text is worth: a number when it is one of the app's formats, else null. */
export function textValue(raw) {
  const text = String(raw == null ? '' : raw).trim();
  if (!text || text === '–' || text === '—' || text === '-') return null;
  let m;
  if ((m = /^(?:(\d+)d)?\s*(?:(\d+)h)?\s*(?:(\d+)m)?\s*(?:(\d+)s)?$/.exec(text)) && m[0]) return (+m[1] || 0) * 86400 + (+m[2] || 0) * 3600 + (+m[3] || 0) * 60 + (+m[4] || 0);
  if ((m = /^([\d.,\s]+?)\s*%/.exec(text))) return plainNumber(m[1]);
  if ((m = /^([\d.]+)\s*(B|KB|MB|GB|TB|PB)$/.exec(text))) return Number(m[1]) * BYTES[m[2]];
  if ((m = /^([\d.]+)\s*(kbps|Mbps)$/.exec(text))) return Number(m[1]) * (m[2] === 'Mbps' ? 1e6 : 1e3);
  if ((m = /^([\d.]+)(K|M|B)$/.exec(text))) return Number(m[1]) * SCALE[m[2]];
  if ((m = /^★\s*([\d.]+)$/.exec(text))) return Number(m[1]);
  if ((m = /^([\d.,\s]+?)\s+of\s+[\d.,\s]+$/.exec(text))) return plainNumber(m[1]);
  return plainNumber(text);
}

/** [number | null, text] for one cell. */
function cellValue(cell) {
  if (!cell) return [null, ''];
  const own = cell.dataset.sort != null ? cell : cell.querySelector('[data-sort]');
  if (own) { const v = own.dataset.sort; const n = v === '' ? null : Number(v); return Number.isFinite(n) ? [n, v] : [null, v]; }
  const sw = cell.querySelector('[role="switch"], input[type="checkbox"]');
  if (sw) return [sw.getAttribute('aria-checked') === 'true' || sw.checked === true ? 1 : 0, ''];
  const time = cell.querySelector('time[datetime]');
  if (time) { const t = Date.parse(time.getAttribute('datetime')); return [Number.isFinite(t) ? t : null, '']; }
  // Only what is read as the value counts: not a screen-reader suffix, a second line, or an avatar's initials.
  let text = '';
  const walk = (n) => { for (const c of n.childNodes) { if (c.nodeType === 3) text += c.nodeValue; else if (c.nodeType === 1 && !c.matches('.sr-only, .cell-sub, .perm-sub, [aria-hidden="true"]')) { walk(c); text += ' '; } } };
  walk(cell);
  text = text.replace(/\s+/g, ' ').trim();
  return [textValue(text), text === '–' ? '' : text];
}

const arrow = () => icon('arrowUp', 11, 'fui-data-table__sort-icon');

export function sortable(table, { server = null } = {}) {
  const head = table.tHead && table.tHead.rows[0];
  const body = table.tBodies[0];
  if (!head || !body) return table;
  let original = [...body.rows];
  let active = -1, dir = null;
  // A page may redraw the rows under us (a setting that repaints the list): start over from what is there.
  function resync() {
    const current = [...body.rows];
    if (current.length === original.length && current.every((r, i) => original.includes(r))) return;
    original = current; active = -1; dir = null;
  }
  const headers = [...head.cells];

  function paint() {
    headers.forEach((th, i) => {
      const on = server ? th.dataset.key && th.dataset.key === server.key : i === active;
      const d = server ? server.dir : dir;
      if (th.querySelector('.fui-data-table__sort')) th.setAttribute('aria-sort', on && d ? (d === 'asc' ? 'ascending' : 'descending') : 'none');
      th.classList.toggle('is-sorted', !!(on && d));
    });
  }

  function sortBy(i, direction) {
    const pinned = original.filter((r) => r.dataset.pin != null);
    const rows = original.filter((r) => r.dataset.pin == null);
    if (direction) {
      const vals = new Map(rows.map((r) => [r, cellValue(r.cells[i])]));
      const numeric = rows.some((r) => vals.get(r)[0] != null) && rows.every((r) => vals.get(r)[0] != null || !vals.get(r)[1]);
      const sign = direction === 'asc' ? 1 : -1;
      rows.sort((a, b) => {
        const [an, at] = vals.get(a), [bn, bt] = vals.get(b);
        const ae = numeric ? an == null : !at, be = numeric ? bn == null : !bt;
        if (ae || be) return ae === be ? 0 : ae ? 1 : -1; // empty last, whichever way
        return sign * (numeric ? an - bn : collator.compare(at, bt));
      });
    }
    body.append(...pinned, ...rows);
  }

  /** Numbers open largest first, words open A to Z. */
  function firstDirection(i) {
    const rows = original.filter((r) => r.dataset.pin == null);
    const vals = rows.map((r) => cellValue(r.cells[i]));
    return vals.some((v) => v[0] != null) && vals.every((v) => v[0] != null || !v[1]) ? 'desc' : 'asc';
  }

  headers.forEach((th, i) => {
    if (th.dataset.nosort != null || !th.textContent.trim() || (server && !th.dataset.key)) return;
    const label = th.textContent.trim();
    const btn = h('button', { type: 'button', class: 'fui-data-table__sort', title: `Sort by ${label.toLowerCase()}` }, [...th.childNodes], arrow());
    th.append(btn);
    btn.addEventListener('click', () => {
      if (server) {
        const same = server.key === th.dataset.key;
        const first = th.dataset.first || 'asc';
        const next = !same ? first : server.dir === first ? (first === 'asc' ? 'desc' : 'asc') : null;
        server.onSort(next ? th.dataset.key : '', next || '');
        return;
      }
      resync();
      const first = firstDirection(i);
      if (active !== i) { active = i; dir = first; } else if (dir === first) dir = first === 'asc' ? 'desc' : 'asc'; else { active = -1; dir = null; }
      sortBy(i, dir);
      paint();
    });
  });
  paint();
  return table;
}

/**
 * A table in its scroll box, sortable, with a quick row filter above it once it is long enough to
 * need one. `filter`: 'auto' (10 rows or more), true or false.
 */
export function dataTable(table, { server = null, filter = 'auto', cls = '', label = 'Filter rows' } = {}) {
  sortable(table, { server });
  const body = table.tBodies[0];
  const rows = body ? [...body.rows] : [];
  const wantFilter = !server && (filter === true || (filter === 'auto' && rows.length >= 10));
  const scroll = h('div', { class: 'fui-data-table__scroll ' + cls }, table);
  if (!wantFilter) return scroll;

  const none = h('p', { class: 'fui-data-table__none', hidden: true }, 'No rows match.');
  const input = h('input', { class: 'fui-field__input fui-field__input--search', type: 'search', placeholder: label + '…', 'aria-label': label, autocomplete: 'off' });
  const texts = new Map(rows.map((r) => [r, r.textContent.toLowerCase()]));
  input.addEventListener('input', () => {
    const words = input.value.toLowerCase().split(/\s+/).filter(Boolean);
    let shown = 0;
    for (const r of rows) { const ok = words.every((w) => texts.get(r).includes(w)); r.hidden = !ok; if (ok) shown++; }
    none.hidden = shown > 0;
  });
  // Esc clears the field first; only an empty field lets the page's own Esc through.
  input.addEventListener('keydown', (e) => { if (e.key === 'Escape' && input.value) { e.stopPropagation(); input.value = ''; input.dispatchEvent(new Event('input')); } });
  return h('div', { class: 'dt' }, h('div', { class: 'fui-data-table__tools' }, h('div', { class: 'fui-field__search' }, icon('search', 14), input)), scroll, none);
}

/** Sortable, without the row filter: short lists, and tables that already have a search of their own. */
export const plainTable = (table) => dataTable(table, { filter: false });
/** The "Table" view of a chart. */
export const chartTable = (table) => dataTable(table, { filter: false, cls: 'chart-table' });

export const meta = {
  name: 'data-table',
  purpose: 'Makes a table readable and sortable: a scroll box, sort buttons in every header, a filter as you type.',
  use: 'Every table. dataTable for a long one (filter box), plainTable without one, chartTable as a chart’s twin. A paginated list passes server: { key, dir, onSort } and data-key headers: it is sorted on the server, never in the browser.',
  avoid: 'A table for a layout. Sorting a paginated list in the browser (it would only shuffle the page on screen).',
  variants: ['with a filter', 'plain', 'chart twin', 'server-sorted', 'hover', 'dense'],
  states: ['sorted up, sorted down, unsorted (a third click)', 'nothing matches the filter'],
  a11y: 'Headers are buttons with aria-sort on the th; values sort by meaning (3d 2h, 1.4 GB, 42%, dates), and numbers are read as num() prints them, whatever the browser’s language.',
  props: { 'dataTable(table, { server, filter, cls, label })': 'table: a built <table class="fui-data-table">', 'plainTable(table)': 'no filter', 'chartTable(table)': 'a chart’s table', 'sortable(table, { server })': 'headers only' },
  playground: {
    controls: [
      { key: 'filter', label: 'Filter as you type', on: true },
      { key: 'dense', label: 'Dense' },
      { key: 'hover', label: 'Rows light up' },
      { key: 'more', label: 'More rows' },
    ],
    render: (o) => {
      const rows = [['Big Buck Bunny', '3h 12m', '4.1 GB', '1,204'], ['Sintel', '52m', '850 MB', '96'], ['Tears of Steel', '1d 2h', '12 GB', '15']];
      const all = o.more ? [...rows, ['Cosmos Laundromat', '12m', '310 MB', '4'], ['Elephants Dream', '2h 1m', '2.2 GB', '41'], ['Spring', '8m', '160 MB', '7']] : rows;
      return dataTable(h('table', { class: ['fui-data-table', o.dense && 'fui-data-table--dense', o.hover && 'fui-data-table--hover'] },
        h('thead', null, h('tr', null, h('th', null, 'Title'), h('th', { class: 'r' }, 'Watch time'), h('th', { class: 'r' }, 'Size'), h('th', { class: 'r' }, 'Plays'))),
        h('tbody', null, all.map((r) => h('tr', null, h('td', null, r[0]), h('td', { class: 'mono r' }, r[1]), h('td', { class: 'mono r' }, r[2]), h('td', { class: 'mono r' }, r[3]))))), { filter: o.filter });
    },
  },
};
