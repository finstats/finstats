// Server → Library health: what is wrong with a file only shows beside its neighbours: a hole in a season, a season in
// another resolution, the same film twice, a file far too thin for what it claims, a dub that stops, a title Jellyfin
// never identified. The server works the findings out after a library read; this only draws them, and FinStats changes
// nothing in Jellyfin. A tile per kind picks the list below it; a finding can be set aside, and comes back by itself
// when the file behind it changes.

import { h, icon, num, bytes, relTime, dateTime, episodeCode, languageName } from '../dom.js';
import { api } from '../api.js';
import { can } from '../state.js';
import { replaceQuery } from '../router.js';
import { card, dataView, sk, pagination, emptyState, poster, facts, openModal, setBusy, inlineError, openInJellyfin } from '../components.js';
import { dataTable } from '../tables.js';
import { button } from '../../finui/components/button/button.js';
import { toast } from '../../finui/components/toast/toast.js';
import { t, lang } from '../i18n.js';

/** Each kind as the page names it, with one line saying what it is. In the order the server lists them. */
export const KINDS = Object.fromEntries(['gap', 'season_drift', 'episode_drift', 'copies', 'versions', 'thin', 'dub', 'unidentified']
  .map((k) => [k, [t(`pages.health.kind.${k}.label`), t(`pages.health.kind.${k}.what`)]]));

// Shared with the prefetcher, so a prefetched view has exactly the address the page asks for.
const filtersOf = (query) => ({
  kind: KINDS[query.get('kind')] ? query.get('kind') : '', library_id: query.get('library_id') || '', dismissed: query.get('dismissed') === '1',
  sort: query.get('sort') || '', dir: query.get('dir') || '', page: Math.max(1, Number(query.get('page')) || 1),
});
const loadSummary = (f, signal) => api.get('/library/health', { library_id: f.library_id }, { signal });
const loadFindings = (f, signal) => api.get('/library/health/findings', { kind: f.kind, library_id: f.library_id, dismissed: f.dismissed ? 1 : '', sort: f.sort, dir: f.dir, page: f.page > 1 ? f.page : '' }, { signal });
/** Both at once, as the page's one view asks for them: as two jobs each was cached under its own address, and the page,
 *  whose view is keyed by the two together, found neither. */
const loadPage = (f, signal) => Promise.all([loadSummary(f, signal), loadFindings(f, signal)]).then(([summary, list]) => ({ summary, list }));
export const prefetchHealth = ({ query, signal }) => [() => loadPage(filtersOf(query), signal)];

// ---------------------------------------------------------------- words

const mbpsf = [1, 2].map((d) => new Intl.NumberFormat(lang(), { minimumFractionDigits: d, maximumFractionDigits: d }));
const rate = (bps) => mbpsf[bps >= 1e6 ? 0 : 1].format(bps / 1e6);
const mbps = (bps) => `${rate(bps)} Mbps`;
const listf = new Intl.ListFormat(lang(), { style: 'long', type: 'conjunction' });
const and = (list) => listf.format(list.map(String));
const seasons = (list) => t(list.length === 1 ? 'pages.health.words.season' : 'pages.health.words.seasons', { list: and(list) });
const run = ([a, b]) => (a === b ? `${a}` : b === a + 1 ? and([a, b]) : `${a}–${b}`);

/** One line, in numbers, saying what was found. Never a verdict: FinStats cannot know why a file is the way it is. */
export function describe(f) {
  const e = f.evidence || {};
  switch (f.kind) {
    case 'gap': {
      const missing = e.missing || [];
      const count = missing.reduce((n, [a, b]) => n + b - a + 1, 0);
      const between = missing.length === 1 ? [missing[0][0] - 1, missing[0][1] + 1] : [e.from, e.to];
      return t(count === 1 ? 'pages.health.words.gap_one' : 'pages.health.words.gap_many', { season: e.season, episodes: and(missing.map(run)), from: between[0], to: between[1] });
    }
    case 'season_drift': {
      const rows = e.seasons || [];
      return (e.differs || []).map((k) => {
        const tally = new Map();
        for (const r of rows) if (r[k]) tally.set(r[k], (tally.get(r[k]) || 0) + 1);
        const usual = [...tally.entries()].sort((a, b) => b[1] - a[1])[0];
        const odd = new Map();
        for (const r of rows) if (r[k] && usual && r[k] !== usual[0]) odd.set(r[k], [...(odd.get(r[k]) || []), r.season]);
        const said = [...odd.entries()].map(([v, list]) => t(list.length === 1 ? 'pages.health.words.season_is' : 'pages.health.words.seasons_are', { list: and(list), value: v })).join(', ');
        return usual ? t('pages.health.words.the_rest', { odd: said, usual: usual[0] }) : said;
      }).join('. ');
    }
    case 'episode_drift': {
      const k = e.differs || [];
      return t('pages.health.words.episode_drift', { code: episodeCode(e.season, e.episode), value: k.map((x) => e[x]).join(' '), season: e.season, usual: k.map((x) => e['season_' + x]).join(' ') });
    }
    case 'copies':
      return Array.isArray(e.files)
        ? t('pages.health.words.copies_files', { count: e.files.length, resolutions: and(e.resolutions || []), size: bytes(f.wasted_bytes) })
        : t('pages.health.words.copies_episodes', { count: e.episodes, size: bytes(f.wasted_bytes) });
    case 'versions':
      return Array.isArray(e.files) ? t('pages.health.words.versions_files', { resolutions: and(e.resolutions || []) }) : t('pages.health.words.versions_episodes', { count: e.episodes, resolutions: and(e.resolutions || []) });
    case 'thin': {
      if (e.bitrate_bps != null) return t('pages.health.words.thin_file', { look: `${e.resolution} ${e.codec}`, rate: mbps(e.bitrate_bps), usual: mbps(e.threshold_bps) });
      const first = (e.episodes || [])[0] || {};
      const range = e.lowest_bps === e.highest_bps ? mbps(e.lowest_bps) : `${rate(e.lowest_bps)}–${mbps(e.highest_bps)}`;
      return t('pages.health.words.thin_season', { season: e.season, files: e.files, count: e.of, rate: range, look: `${first.resolution} ${first.codec}`, usual: mbps(first.threshold_bps) });
    }
    case 'dub': {
      const name = String(languageName(e.language) || e.language).replace(/^./, (c) => c.toLocaleUpperCase(lang()));
      const partial = (e.partial || []).map((p) => t('pages.health.words.dub_partial', { season: p.season, episodes: p.episodes, of: p.of }));
      return [t('pages.health.words.dub', { language: name, full: seasons(e.full || []), none: seasons(e.none || []) }), ...partial].join(', ');
    }
    case 'unidentified':
      return t(e.type === 'Series' ? 'pages.health.words.unidentified_show' : 'pages.health.words.unidentified_film');
    default:
      return '';
  }
}

/** The files behind a finding, where they say more than the line does: paths, sizes, libraries. */
function details(f, libraryName) {
  const e = f.evidence || {};
  const file = (x) => facts([
    [t('pages.health.facts.resolution'), [x.resolution, x.codec].filter(Boolean).join(' ') || null],
    [t('pages.health.facts.size'), x.size_bytes != null ? bytes(x.size_bytes) : null],
    [t('pages.health.facts.library'), libraryName(x.library_id)],
    x.path ? [t('pages.health.facts.path'), h('span', { class: 'mono path' }, x.path), { wide: true }] : null,
  ]);
  // A film's copies list their files; a gap or a thin season counts them, and that count is not a list.
  if (Array.isArray(e.files)) return h('div', { class: 'health-files' }, e.files.map(file));
  if (f.kind === 'copies' && e.examples) {
    const shown = e.examples.slice(0, 3);
    return h('div', { class: 'health-files' }, shown.map((x) => h('div', null, h('p', { class: 'health-sub' }, episodeCode(x.season, x.episode)), x.files.map(file))),
      e.episodes > shown.length ? h('p', { class: 'fui-field__help' }, t('pages.health.more_episodes', { count: e.episodes - shown.length })) : null);
  }
  if (f.kind === 'season_drift') {
    return h('ul', { class: 'health-chips' }, (e.seasons || []).map((r) => h('li', { class: 'fui-chip' }, [`S${r.season}`, r.resolution, r.codec, r.range].filter(Boolean).join(' · '))));
  }
  if (f.kind === 'unidentified' && e.path) return facts([[t('pages.health.facts.path'), h('span', { class: 'mono path' }, e.path), { wide: true }]]);
  return null;
}

// ---------------------------------------------------------------- the section

/** The section: tiles, filters, and the list of the kind picked. Mounted by the Server page. */
/** Set a finding aside, with a note, after asking: Library health's and Capacity's alike (one dismissal for both).
 *  `line` says what the finding is; `onDone` redraws the page once it is set aside. */
export function askDismiss(item, line, onDone) {
  const note = h('input', { class: 'fui-field__input', type: 'text', maxLength: 500, id: 'health-note', autocomplete: 'off', placeholder: t('pages.health.dismiss.note_placeholder') });
  const error = inlineError('health-error', '');
  error.hidden = true;
  const save = button({ variant: 'primary', type: 'submit' }, icon('check', 14), t('pages.health.dismiss.dismiss'));
  const form = h('form', { class: 'stack-sm' },
    h('p', { class: 'fui-field__help' }, `${item.title}: ${line}.`),
    h('div', { class: 'fui-field' }, h('label', { class: 'fui-field__label', for: 'health-note' }, t('pages.health.dismiss.note')), note),
    h('p', { class: 'fui-field__help' }, item.key.startsWith('capacity:') ? t('pages.health.dismiss.comes_back_capacity') : t('pages.health.dismiss.comes_back')),
    error,
    h('div', { class: 'form-actions' }, save, button({ variant: 'ghost', type: 'button', onClick: () => modal.close() }, t('common.cancel'))));
  const modal = openModal({ title: t('pages.health.dismiss.title'), body: form, initialFocus: note });
  form.addEventListener('submit', async (ev) => {
    ev.preventDefault();
    setBusy(save, true, t('pages.health.dismiss.dismissing'));
    try {
      await api.post('/library/health/dismiss', { key: item.key, note: note.value });
      modal.close();
      onDone();
    } catch (err) {
      setBusy(save, false);
      error.hidden = false;
      error.lastChild.textContent = err.message;
    }
  });
}

export function healthView(ctx) {
  const f = filtersOf(ctx.query);
  const manage = can('manage') && can('see_server');
  const view = h('div', { class: 'stack' });

  function apply(reset = true) {
    if (reset) f.page = 1;
    replaceQuery({ kind: f.kind, library_id: f.library_id, dismissed: f.dismissed ? 1 : '', sort: f.sort, dir: f.dir, page: f.page > 1 ? f.page : '' });
    dv.load();
  }

  const dismissDialog = (item) => askDismiss(item, describe(item), () => dv.load());

  async function bringBack(btn, item) {
    setBusy(btn, true, t('pages.health.bringing_back'));
    // Refused, the page has nothing new to draw, so the button would stay busy and nobody would be told.
    try { await api.post('/library/health/undismiss', { key: item.key }); } catch (e) { setBusy(btn, false); toast({ text: t('pages.health.couldnt_bring_back', { error: e.message }), tone: 'critical' }); return; }
    dv.load();
  }

  function tiles(s) {
    return h('div', { class: 'fui-stat-tile__grid health-tiles' }, s.kinds.map((k) => {
      const [label, what] = KINDS[k.kind] || [k.kind, ''];
      const on = f.kind === k.kind;
      return h('button', { type: 'button', class: ['fui-stat-tile fui-stat-tile--pressable', !k.count && 'is-quiet'], 'aria-pressed': String(on), title: on ? t('pages.health.show_every_kind') : t('pages.health.show_only', { kind: label.toLocaleLowerCase(lang()) }),
        onClick: () => { f.kind = on ? '' : k.kind; f.sort = ''; f.dir = ''; apply(); } },
        h('span', { class: 'fui-stat-tile__label' }, label),
        h('span', { class: 'fui-stat-tile__value' }, num(k.count)),
        h('span', { class: 'fui-stat-tile__foot' }, h('span', { class: 'fui-stat-tile__vs' }, k.kind === 'copies' && k.wasted_bytes ? t('pages.health.kept_twice_size', { size: bytes(k.wasted_bytes) }) : what)));
    }));
  }

  function controls(s) {
    const libs = s.libraries || [];
    const select = h('select', { class: 'fui-field__input', 'aria-label': t('pages.health.facts.library') },
      h('option', { value: '' }, t('pages.health.all_libraries')),
      libs.map((l) => h('option', { value: l.id, selected: l.id === f.library_id }, `${l.name || t('pages.health.unlisted_library')} (${num(l.count)})`)),
      f.library_id && !libs.some((l) => l.id === f.library_id) ? h('option', { value: f.library_id, selected: true }, t('pages.health.this_library_empty')) : null);
    select.addEventListener('change', () => { f.library_id = select.value; apply(); });
    const aside = s.kinds.reduce((n, k) => n + k.dismissed, 0);
    const toggle = aside || f.dismissed
      ? button({ size: 'sm', variant: 'ghost', type: 'button', 'aria-pressed': String(f.dismissed), onClick: () => { f.dismissed = !f.dismissed; apply(); } },
        icon(f.dismissed ? 'chevronLeft' : 'inbox', 13), f.dismissed ? t('pages.health.back_to_open') : t('pages.health.set_aside_count', { count: aside }))
      : null;
    return h('div', { class: 'filters' }, libs.length > 1 || f.library_id ? select : null, toggle);
  }

  function table(list, s) {
    const names = new Map((s.libraries || []).map((l) => [l.id, l.name]));
    for (const it of list.items) for (const l of it.libraries || []) if (l.name) names.set(l.id, l.name);
    const libraryName = (id) => (id ? names.get(id) || null : null);
    const withWasted = f.kind === 'copies';
    const server = { key: f.sort, dir: f.dir, onSort: (key, dir) => { f.sort = key; f.dir = dir; apply(); } };
    return dataTable(h('table', { class: 'fui-data-table health' },
      h('thead', null, h('tr', null,
        h('th', { 'data-key': 'title', 'data-first': 'asc' }, t('plays_table.title')),
        h('th', { 'data-nosort': '' }, t('pages.health.col.what')),
        withWasted ? h('th', { 'data-key': 'wasted', class: 'r' }, t('pages.health.col.kept_twice')) : null,
        h('th', { 'data-key': 'found' }, t('pages.health.col.found')),
        h('th', { 'data-nosort': '' }, ''))),
      h('tbody', null, list.items.map((it) => h('tr', null,
        h('td', null, h('div', { class: 'title-cell' }, poster(it.image_item_id, it.title, { w: 80, cls: 'fui-poster--sm' }),
          h('div', { class: 'play-title' },
            h('a', { class: 'play-title-main', href: `/items/${it.item_id}` }, it.title),
            h('div', { class: 'play-title-sub' }, [f.kind ? null : (KINDS[it.kind] || [it.kind])[0], it.year, ...(it.libraries || []).map((l) => l.name || t('pages.health.removed_library')),
              it.removed ? t('pages.health.not_in_jellyfin') : null].filter(Boolean).join(' · '))))),
        h('td', { class: 'wrap-cell health-what' }, h('p', null, describe(it)), details(it, libraryName),
          it.dismissed ? h('p', { class: 'health-aside' }, icon('check', 12), it.dismissed_by ? t('pages.health.set_aside_by', { when: relTime(it.dismissed_at), name: it.dismissed_by }) : t('pages.health.set_aside_when', { when: relTime(it.dismissed_at) }), it.note ? `: ${it.note}` : '') : null),
        withWasted ? h('td', { class: 'mono r' }, it.wasted_bytes ? bytes(it.wasted_bytes) : h('span', { class: 'muted' }, '–')) : null,
        h('td', { class: 'mono nowrap', title: dateTime(it.found_at) }, relTime(it.found_at)),
        h('td', { class: 'health-actions' },
          manage ? (it.dismissed
            ? button({ size: 'sm', type: 'button', onClick: (e) => bringBack(e.currentTarget, it) }, icon('refresh', 13), t('pages.health.bring_back'))
            : button({ size: 'sm', type: 'button', onClick: () => dismissDialog(it) }, icon('check', 13), t('pages.health.dismiss_button'))) : null,
          openInJellyfin(it.jellyfin_link, { compact: true })))))), { server });
  }

  const dv = dataView({
    container: view, signal: ctx.signal,
    skeleton: () => [sk.tiles(4), sk.tableRows(6)],
    fetch: () => loadPage(f, ctx.signal),
    render: ({ summary: s, list }) => {
      if (!s.computed_at) {
        return emptyState(t('pages.health.empty.not_yet_title'), t('pages.health.empty.not_yet'));
      }
      const filtered = f.kind || f.library_id;
      const count = h('p', { class: 'result-count', 'aria-live': 'polite' }, t(f.dismissed ? 'pages.health.count_aside' : 'pages.health.count_open', { count: list.total }));
      const body = list.items.length
        ? [table(list, s), list.total > list.page_size ? pagination({ page: list.page, perPage: list.page_size, total: list.total, onPage: (p) => { f.page = p; apply(false); window.scrollTo({ top: 0 }); } }) : null]
        : f.dismissed ? emptyState(t('pages.health.empty.aside_title'), filtered ? t('pages.health.empty.aside_filtered') : t('pages.health.empty.aside'))
          : emptyState(filtered ? t('pages.health.empty.filtered_title') : t('pages.health.empty.clean_title'), filtered ? t('pages.health.empty.filtered') : t('pages.health.empty.clean'));
      const thinNote = (!f.kind || f.kind === 'thin') && list.items.some((it) => it.kind === 'thin')
        ? h('p', { class: 'fui-field__help' }, t('pages.health.thin_note')) : null;
      return [tiles(s), controls(s), count, card({ cls: 'fui-card--flush', id: 'health', body }), thinNote];
    },
  });
  dv.load();
  return [view];
}
