// Server → Library health: what is wrong with a file only shows beside its neighbours — a hole in a season, a season in
// another resolution, the same film twice, a file far too thin for what it claims, a dub that stops, a title Jellyfin
// never identified. The server works the findings out after a library read; this only draws them, and finstats changes
// nothing in Jellyfin. A tile per kind picks the list below it; a finding can be set aside, and comes back by itself
// when the file behind it changes.

import { h, icon, num, bytes, relTime, dateTime, episodeCode, languageName } from '../dom.js';
import { api } from '../api.js';
import { can } from '../state.js';
import { replaceQuery } from '../router.js';
import { card, dataView, sk, pagination, emptyState, poster, facts, openModal, setBusy, inlineError, openInJellyfin } from '../components.js';
import { dataTable } from '../tables.js';
import { button } from '../../finui/components/button/button.js';

/** Each kind as the page names it, with one line saying what it is. In the order the server lists them. */
export const KINDS = {
  gap: ['Gaps', 'Episodes missing between two files of a season'],
  season_drift: ['Seasons that differ', 'A season in another resolution, range or codec'],
  episode_drift: ['Odd episodes', 'A file unlike the rest of its season'],
  copies: ['Copies', 'The same thing twice, in one resolution'],
  versions: ['Versions', 'One title kept in more than one resolution'],
  thin: ['Thin files', 'Far fewer bits than the resolution usually takes'],
  dub: ['Dubs that stop', 'A language some seasons have and others lack'],
  unidentified: ['Never identified', 'Matched with no TMDB, TVDB or IMDb id'],
};

// Shared with the prefetcher, so a prefetched view has exactly the address the page asks for.
const filtersOf = (query) => ({
  kind: KINDS[query.get('kind')] ? query.get('kind') : '', library_id: query.get('library_id') || '', dismissed: query.get('dismissed') === '1',
  sort: query.get('sort') || '', dir: query.get('dir') || '', page: Math.max(1, Number(query.get('page')) || 1),
});
const loadSummary = (f, signal) => api.get('/library/health', { library_id: f.library_id }, { signal });
const loadFindings = (f, signal) => api.get('/library/health/findings', { kind: f.kind, library_id: f.library_id, dismissed: f.dismissed ? 1 : '', sort: f.sort, dir: f.dir, page: f.page > 1 ? f.page : '' }, { signal });
export const prefetchHealth = ({ query, signal }) => { const f = filtersOf(query); return [() => loadSummary(f, signal), () => loadFindings(f, signal)]; };

// ---------------------------------------------------------------- words

const mbps = (bps) => `${(bps / 1e6).toFixed(bps >= 1e6 ? 1 : 2)} Mbps`;
const and = (list) => (list.length < 2 ? list.join('') : `${list.slice(0, -1).join(', ')} and ${list[list.length - 1]}`);
const seasons = (list) => `${list.length === 1 ? 'season' : 'seasons'} ${and(list.map(String))}`;
const Seasons = (list) => { const s = seasons(list); return s[0].toUpperCase() + s.slice(1); };
const run = ([a, b]) => (a === b ? `${a}` : b === a + 1 ? `${a} and ${b}` : `${a}–${b}`);
const plural = (n, one, many = one + 's') => `${num(n)} ${n === 1 ? one : many}`;

/** One line, in numbers, saying what was found. Never a verdict: finstats cannot know why a file is the way it is. */
export function describe(f) {
  const e = f.evidence || {};
  switch (f.kind) {
    case 'gap': {
      const missing = e.missing || [];
      const count = missing.reduce((n, [a, b]) => n + b - a + 1, 0);
      const between = missing.length === 1 ? [missing[0][0] - 1, missing[0][1] + 1] : [e.from, e.to];
      return `Season ${e.season}: ${count === 1 ? 'episode' : 'episodes'} ${and(missing.map(run))} ${count === 1 ? 'is' : 'are'} missing between ${between[0]} and ${between[1]}`;
    }
    case 'season_drift': {
      const rows = e.seasons || [];
      return (e.differs || []).map((k) => {
        const tally = new Map();
        for (const r of rows) if (r[k]) tally.set(r[k], (tally.get(r[k]) || 0) + 1);
        const usual = [...tally.entries()].sort((a, b) => b[1] - a[1])[0];
        const odd = new Map();
        for (const r of rows) if (r[k] && usual && r[k] !== usual[0]) odd.set(r[k], [...(odd.get(r[k]) || []), r.season]);
        return [...odd.entries()].map(([v, list]) => `${Seasons(list)} ${list.length === 1 ? 'is' : 'are'} ${v}`).join(', ') + (usual ? `, the rest ${usual[0]}` : '');
      }).join('. ');
    }
    case 'episode_drift': {
      const k = e.differs || [];
      return `${episodeCode(e.season, e.episode)} is ${k.map((x) => e[x]).join(' ')}, the rest of season ${e.season} ${k.map((x) => e['season_' + x]).join(' ')}`;
    }
    case 'copies':
      return Array.isArray(e.files)
        ? `${plural(e.files.length, 'copy', 'copies')} in ${and(e.resolutions || [])}, ${bytes(f.wasted_bytes)} kept twice`
        : `${plural(e.episodes, 'episode')} here more than once, ${bytes(f.wasted_bytes)} kept twice`;
    case 'versions':
      return Array.isArray(e.files) ? `Kept in ${and(e.resolutions || [])}` : `${plural(e.episodes, 'episode')} kept in ${and(e.resolutions || [])}`;
    case 'thin': {
      if (e.bitrate_bps != null) return `${e.resolution} ${e.codec} at ${mbps(e.bitrate_bps)}; files like it are usually over ${mbps(e.threshold_bps)}`;
      const first = (e.episodes || [])[0] || {};
      const range = e.lowest_bps === e.highest_bps ? mbps(e.lowest_bps) : `${mbps(e.lowest_bps).replace(' Mbps', '')}–${mbps(e.highest_bps)}`;
      return `Season ${e.season}: ${num(e.files)} of ${plural(e.of, 'episode')} at ${range}, ${first.resolution} ${first.codec}; files like them are usually over ${mbps(first.threshold_bps)}`;
    }
    case 'dub': {
      const name = languageName(e.language) || e.language;
      const partial = (e.partial || []).map((p) => `, season ${p.season} has it on ${num(p.episodes)} of ${num(p.of)}`).join('');
      return `${name} in ${seasons(e.full || [])}, not in ${seasons(e.none || [])}${partial}`;
    }
    case 'unidentified':
      return `Jellyfin matched this ${e.type === 'Series' ? 'show' : 'film'} with no catalogue, so nothing that goes by TMDB, TVDB or IMDb ids can recognise it`;
    default:
      return '';
  }
}

/** The files behind a finding, where they say more than the line does: paths, sizes, libraries. */
function details(f, libraryName) {
  const e = f.evidence || {};
  const file = (x) => facts([
    ['Resolution', [x.resolution, x.codec].filter(Boolean).join(' ') || null],
    ['Size', x.size_bytes != null ? bytes(x.size_bytes) : null],
    ['Library', libraryName(x.library_id)],
    x.path ? ['Path', h('span', { class: 'mono path' }, x.path), { wide: true }] : null,
  ]);
  // A film's copies list their files; a gap or a thin season counts them, and that count is not a list.
  if (Array.isArray(e.files)) return h('div', { class: 'health-files' }, e.files.map(file));
  if (f.kind === 'copies' && e.examples) {
    const shown = e.examples.slice(0, 3);
    return h('div', { class: 'health-files' }, shown.map((x) => h('div', null, h('p', { class: 'health-sub' }, episodeCode(x.season, x.episode)), x.files.map(file))),
      e.episodes > shown.length ? h('p', { class: 'fui-field__help' }, `and ${plural(e.episodes - shown.length, 'more episode')}`) : null);
  }
  if (f.kind === 'season_drift') {
    return h('ul', { class: 'health-chips' }, (e.seasons || []).map((r) => h('li', { class: 'fui-chip' }, [`S${r.season}`, r.resolution, r.codec, r.range].filter(Boolean).join(' · '))));
  }
  if (f.kind === 'unidentified' && e.path) return facts([['Path', h('span', { class: 'mono path' }, e.path), { wide: true }]]);
  return null;
}

// ---------------------------------------------------------------- the section

/** The section: tiles, filters, and the list of the kind picked. Mounted by the Server page. */
export function healthView(ctx) {
  const f = filtersOf(ctx.query);
  const manage = can('manage') && can('see_server');
  const view = h('div', { class: 'stack' });

  function apply(reset = true) {
    if (reset) f.page = 1;
    replaceQuery({ kind: f.kind, library_id: f.library_id, dismissed: f.dismissed ? 1 : '', sort: f.sort, dir: f.dir, page: f.page > 1 ? f.page : '' });
    dv.load();
  }

  function dismissDialog(item) {
    const note = h('input', { class: 'fui-field__input', type: 'text', maxLength: 500, id: 'health-note', autocomplete: 'off', placeholder: 'Why it is fine, for whoever looks next' });
    const error = inlineError('health-error', '');
    error.hidden = true;
    const save = button({ variant: 'primary', type: 'submit' }, icon('check', 14), 'Dismiss');
    const form = h('form', { class: 'stack-sm' },
      h('p', { class: 'fui-field__help' }, `${item.title}: ${describe(item)}.`),
      h('div', { class: 'fui-field' }, h('label', { class: 'fui-field__label', for: 'health-note' }, 'Note (optional)'), note),
      h('p', { class: 'fui-field__help' }, 'It comes back by itself if the files behind it change.'),
      error,
      h('div', { class: 'form-actions' }, save, button({ variant: 'ghost', type: 'button', onClick: () => modal.close() }, 'Cancel')));
    const modal = openModal({ title: 'Dismiss finding', body: form, initialFocus: note });
    form.addEventListener('submit', async (ev) => {
      ev.preventDefault();
      setBusy(save, true, 'Dismissing…');
      try {
        await api.post('/library/health/dismiss', { key: item.key, note: note.value });
        modal.close();
        dv.load();
      } catch (err) {
        setBusy(save, false);
        error.hidden = false;
        error.lastChild.textContent = err.message;
      }
    });
  }

  async function bringBack(btn, item) {
    setBusy(btn, true, 'Bringing back…');
    try { await api.post('/library/health/undismiss', { key: item.key }); } finally { dv.load(); }
  }

  function tiles(s) {
    return h('div', { class: 'fui-stat-tile__grid health-tiles' }, s.kinds.map((k) => {
      const [label, what] = KINDS[k.kind] || [k.kind, ''];
      const on = f.kind === k.kind;
      return h('button', { type: 'button', class: ['fui-stat-tile fui-stat-tile--pressable', !k.count && 'is-quiet'], 'aria-pressed': String(on), title: on ? 'Show every kind' : `Show only ${label.toLowerCase()}`,
        onClick: () => { f.kind = on ? '' : k.kind; f.sort = ''; f.dir = ''; apply(); } },
        h('span', { class: 'fui-stat-tile__label' }, label),
        h('span', { class: 'fui-stat-tile__value' }, num(k.count)),
        h('span', { class: 'fui-stat-tile__foot' }, h('span', { class: 'fui-stat-tile__vs' }, k.kind === 'copies' && k.wasted_bytes ? `${bytes(k.wasted_bytes)} kept twice` : what)));
    }));
  }

  function controls(s) {
    const libs = s.libraries || [];
    const select = h('select', { class: 'fui-field__input', 'aria-label': 'Library' },
      h('option', { value: '' }, 'All libraries'),
      libs.map((l) => h('option', { value: l.id, selected: l.id === f.library_id }, `${l.name || 'A library Jellyfin no longer lists'} (${num(l.count)})`)),
      f.library_id && !libs.some((l) => l.id === f.library_id) ? h('option', { value: f.library_id, selected: true }, 'This library (nothing to look at)') : null);
    select.addEventListener('change', () => { f.library_id = select.value; apply(); });
    const aside = s.kinds.reduce((n, k) => n + k.dismissed, 0);
    const toggle = aside || f.dismissed
      ? button({ size: 'sm', variant: 'ghost', type: 'button', 'aria-pressed': String(f.dismissed), onClick: () => { f.dismissed = !f.dismissed; apply(); } },
        icon(f.dismissed ? 'chevronLeft' : 'inbox', 13), f.dismissed ? 'Back to what to look at' : `Set aside (${num(aside)})`)
      : null;
    return h('div', { class: 'filters' }, libs.length > 1 || f.library_id ? select : null, toggle);
  }

  function table(list, s) {
    const names = new Map((s.libraries || []).map((l) => [l.id, l.name]));
    for (const it of list.items) for (const l of it.libraries || []) if (l.name) names.set(l.id, l.name);
    const libraryName = (id) => (id ? names.get(id) || null : null);
    const withWasted = f.kind === 'copies';
    const server = { key: f.sort, dir: f.dir, onSort: (key, dir) => { f.sort = key; f.dir = dir; apply(); } };
    return dataTable(h('table', { class: 'table health' },
      h('thead', null, h('tr', null,
        h('th', { 'data-key': 'title', 'data-first': 'asc' }, 'Title'),
        h('th', { 'data-nosort': '' }, 'What'),
        withWasted ? h('th', { 'data-key': 'wasted', class: 'r' }, 'Kept twice') : null,
        h('th', { 'data-key': 'found' }, 'Found'),
        h('th', { 'data-nosort': '' }, ''))),
      h('tbody', null, list.items.map((it) => h('tr', null,
        h('td', null, h('div', { class: 'title-cell' }, poster(it.image_item_id, it.title, { w: 80, cls: 'fui-poster--sm' }),
          h('div', { class: 'play-title' },
            h('a', { class: 'play-title-main', href: `/items/${it.item_id}` }, it.title),
            h('div', { class: 'play-title-sub' }, [f.kind ? null : (KINDS[it.kind] || [it.kind])[0], it.year, ...(it.libraries || []).map((l) => l.name || 'a removed library'),
              it.removed ? 'no longer in Jellyfin' : null].filter(Boolean).join(' · '))))),
        h('td', { class: 'wrap-cell health-what' }, h('p', null, describe(it)), details(it, libraryName),
          it.dismissed ? h('p', { class: 'health-aside' }, icon('check', 12), `Set aside ${relTime(it.dismissed_at)}${it.dismissed_by ? ' by ' + it.dismissed_by : ''}`, it.note ? `: ${it.note}` : '') : null),
        withWasted ? h('td', { class: 'mono r' }, it.wasted_bytes ? bytes(it.wasted_bytes) : h('span', { class: 'muted' }, '–')) : null,
        h('td', { class: 'mono nowrap', title: dateTime(it.found_at) }, relTime(it.found_at)),
        h('td', { class: 'health-actions' },
          manage ? (it.dismissed
            ? button({ size: 'sm', type: 'button', onClick: (e) => bringBack(e.currentTarget, it) }, icon('refresh', 13), 'Bring back')
            : button({ size: 'sm', type: 'button', onClick: () => dismissDialog(it) }, icon('check', 13), 'Dismiss…')) : null,
          openInJellyfin(it.jellyfin_link, { compact: true })))))), { server });
  }

  const dv = dataView({
    container: view, signal: ctx.signal,
    skeleton: () => [sk.tiles(4), sk.tableRows(6)],
    fetch: () => Promise.all([loadSummary(f, ctx.signal), loadFindings(f, ctx.signal)]).then(([summary, list]) => ({ summary, list })),
    render: ({ summary: s, list }) => {
      if (!s.computed_at) {
        return emptyState('Not worked out yet', 'finstats looks the library over after its next library read, and again after each look for metadata changes. Nothing is worked out when this page opens.');
      }
      const filtered = f.kind || f.library_id;
      const count = h('p', { class: 'result-count', 'aria-live': 'polite' }, `${plural(list.total, 'finding')} ${f.dismissed ? 'set aside' : 'to look at'}`);
      const body = list.items.length
        ? [table(list, s), list.total > list.page_size ? pagination({ page: list.page, perPage: list.page_size, total: list.total, onPage: (p) => { f.page = p; apply(false); window.scrollTo({ top: 0 }); } }) : null]
        : f.dismissed ? emptyState('Nothing set aside', filtered ? 'Nothing of this kind was dismissed here.' : 'A dismissed finding waits here until somebody brings it back, or its files change.')
          : emptyState(filtered ? 'Nothing here' : 'Nothing to look at', filtered ? 'Nothing of this kind was found in this part of the library.' : 'No holes, no copies, nothing too thin, and every season alike.');
      const thinNote = (!f.kind || f.kind === 'thin') && list.items.some((it) => it.kind === 'thin')
        ? h('p', { class: 'fui-field__help' }, 'Jellyfin keeps the first version of an item for finstats to read, so an item with several versions is judged by its first.') : null;
      return [tiles(s), controls(s), count, card({ cls: 'fui-card--flush', id: 'health', body }), thinNote];
    },
  });
  dv.load();
  return [view];
}
