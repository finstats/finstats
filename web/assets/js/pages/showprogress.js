// The all-time part of a profile: day streaks and how far someone is through each show.
// One segment per episode that exists as a file on the server; on your own profile a segment
// (or a whole season / show) can be marked as seen by hand.

import { h, icon, mount, num, pct, parseDay } from '../dom.js';
import { api, isAbort } from '../api.js';
import { card, segmented, poster, statTile, emptyState, errorState, inlineError, sk, setBusy } from '../components.js';
import { button } from '../../finui/components/button/button.js';
import { t, lang } from '../i18n.js';

const PAGE = 6;
const dayLabel = (iso, withYear) => parseDay(iso).toLocaleDateString(lang(), { day: 'numeric', month: 'short', ...(withYear ? { year: 'numeric' } : {}) });

function streakTiles(s) {
  if (!s) return null;
  const l = s.longest;
  const c = s.current || {};
  return h('div', { class: 'fui-stat-tile__grid fui-stat-tile__grid--three' },
    statTile({ label: t('pages.showprogress.longest_streak'), value: l ? t('pages.showprogress.days', { count: l.days }) : '–',
      hint: l ? (l.days > 1 ? `${dayLabel(l.from)} – ${dayLabel(l.to, true)}` : dayLabel(l.from, true)) : t('pages.showprogress.no_plays_yet') }),
    statTile({ label: t('pages.showprogress.current_streak'), value: c.days ? t('pages.showprogress.days', { count: c.days }) : '–',
      hint: !c.days ? t('pages.showprogress.nothing_today') : c.includes_today ? t('pages.showprogress.since', { day: dayLabel(c.since) }) : t('pages.showprogress.since_keep', { day: dayLabel(c.since) }) }),
    statTile({ label: t('pages.showprogress.days_with_play'), value: num(s.active_days || 0), hint: t('pages.showprogress.all_time') }));
}

const SOURCES = ['played', 'jellyfin', 'manual'];
const epLabel = (seasonNo, e) => `S${String(seasonNo).padStart(2, '0')}E${String(e.episode_number ?? '?').padStart(2, '0')} · ${e.name}`;

function bar(episodes, { seasonNo, editable, onToggle, dense }) {
  return h('div', { class: ['ep-bar', (dense || episodes.length > 120) && 'ep-bar-dense'], role: editable ? 'group' : 'img',
    'aria-label': t('pages.showprogress.episodes_seen', { seen: episodes.filter((e) => e.state === 'seen').length, count: episodes.length }) },
    episodes.map((e) => {
      const what = e.state === 'seen' ? t('pages.showprogress.seen_how', { how: SOURCES.includes(e.source) ? t(`pages.showprogress.source.${e.source}`) : t('pages.showprogress.seen') })
        : e.state === 'started' ? t('pages.showprogress.started') : t('pages.showprogress.not_seen');
      const title = `${epLabel(e.season ?? seasonNo, e)} · ${what}`;
      if (!editable) return h('span', { class: `ep ep-${e.state}`, title });
      const locked = e.state === 'seen' && e.source !== 'manual'; // a real play can't be un-watched here
      return h('button', { type: 'button', class: `ep ep-${e.state}`, title: locked ? title : t(e.state === 'seen' ? 'pages.showprogress.click_unmark' : 'pages.showprogress.click_mark', { title }),
        'aria-label': title, 'aria-pressed': String(e.state === 'seen'), 'aria-disabled': locked ? 'true' : null,
        onClick: () => { if (!locked) onToggle([e.id], e.state !== 'seen'); } });
    }));
}

/**
 * Returns two elements fed by one request: `tiles` (the streaks, for the top of the profile) and
 * `shows` (the progress card, placed further down). Both cover all time, whatever range the rest
 * of the page shows; the same nodes are handed back on every re-render, so an opened show or a
 * chosen tab survives a change of range.
 */
export function profileAllTime({ userId, signal }) {
  const tiles = h('div', { class: 'profile-alltime' });
  const showsEl = h('div', { class: 'profile-shows' });
  let data = null;
  let filter = 'progress';
  let shown = PAGE;
  const open = new Set();
  const problem = h('div');

  async function load(first) {
    if (first) { mount(tiles, sk.tiles(3)); mount(showsEl, sk.cardRows(4)); }
    try { data = await api.get(`/users/${encodeURIComponent(userId)}/shows`, null, { signal }); paint(); }
    catch (e) { if (isAbort(e) || e.status === 401) return; if (first) { mount(tiles, ''); mount(showsEl, errorState(e, () => load(true))); } }
  }

  async function toggle(ids, seen, btn) {
    mount(problem, '');
    if (btn) setBusy(btn, true);
    try { await api.post('/me/seen', { item_ids: ids, seen }); await load(false); }
    catch (e) { if (btn) setBusy(btn, false); mount(problem, inlineError('seen-err', t('pages.showprogress.couldnt_save', { error: e.message }))); }
  }

  function row(s) {
    const all = s.seasons.flatMap((se) => se.episodes.map((e) => ({ ...e, season: se.season_number })));
    const isOpen = open.has(s.id);
    const done = s.seen >= s.total;
    const head = h('button', { type: 'button', class: 'show-head', 'aria-expanded': String(isOpen),
      onClick: () => { if (isOpen) open.delete(s.id); else open.add(s.id); paint(); } },
      h('span', { class: 'show-name' }, s.name, s.removed ? h('span', { class: 'fui-chip' }, t('pages.showprogress.not_in_library')) : null),
      h('span', { class: 'show-count mono' }, `${num(s.seen)}/${num(s.total)}`), h('span', { class: 'show-pct mono' }, pct(s.total ? s.seen / s.total : 0)),
      icon('chevronRight', 14, 'show-chev'));
    const unseen = (eps) => eps.filter((e) => e.state !== 'seen').map((e) => e.id);
    const manual = (eps) => eps.filter((e) => e.source === 'manual').map((e) => e.id);
    const markBtn = (eps, label) => {
      if (!data.editable) return null;
      const todo = unseen(eps);
      const undo = manual(eps);
      if (todo.length) { const b = button({ variant: 'ghost', size: 'sm', type: 'button' }, icon('check', 13), label); b.addEventListener('click', () => toggle(todo, true, b)); return b; }
      if (undo.length) { const b = button({ variant: 'ghost', size: 'sm', type: 'button' }, t('pages.showprogress.undo_marks')); b.addEventListener('click', () => toggle(undo, false, b)); return b; }
      return null;
    };
    return h('li', { class: ['show-row', isOpen && 'is-open', done && 'is-done'] },
      s.removed ? h('span', { class: 'fui-poster fui-poster--sm fui-poster--placeholder', 'aria-hidden': 'true' }, icon('tv', 16)) : h('a', { href: `/items/${s.id}`, tabindex: -1, 'aria-hidden': 'true' }, poster(s.image_item_id, s.name, { w: 160, cls: 'fui-poster--sm' })),
      h('div', { class: 'show-main' }, head,
        isOpen ? null : bar(all, { editable: false }),
        isOpen ? h('div', { class: 'show-seasons' },
          s.seasons.map((se) => h('div', { class: 'season-row' },
            h('div', { class: 'season-head' }, h('span', { class: 'season-name' }, t('pages.showprogress.season_n', { number: String(se.season_number) })),
              h('span', { class: 'mono season-count' }, `${num(se.seen)}/${num(se.total)} (${pct(se.total ? se.seen / se.total : 0)})`), markBtn(se.episodes, t('pages.showprogress.mark_season'))),
            bar(se.episodes, { seasonNo: se.season_number, editable: data.editable, onToggle: (ids, seen) => toggle(ids, seen) }))),
          data.editable ? h('div', { class: 'season-foot' }, markBtn(all, t('pages.showprogress.mark_show')),
            h('span', { class: 'fui-field__help' }, t('pages.showprogress.marks_help'))) : null) : null));
  }

  function paint() {
    const shows = (data.shows || []).map((s) => ({ ...s, ratio: s.total ? s.seen / s.total : 0 }));
    const lists = {
      progress: shows.filter((s) => s.seen < s.total).sort((a, b) => b.ratio - a.ratio || (b.last_played_at || 0) - (a.last_played_at || 0)),
      finished: shows.filter((s) => s.total > 0 && s.seen >= s.total).sort((a, b) => (b.last_played_at || 0) - (a.last_played_at || 0)),
    };
    lists.all = [...lists.progress, ...lists.finished];
    const list = lists[filter];
    const body = !shows.length ? emptyState(t('pages.showprogress.no_shows'), t('pages.showprogress.no_shows_sub'))
      : !list.length ? emptyState(filter === 'finished' ? t('pages.showprogress.no_finished') : t('pages.showprogress.nothing_in_progress'))
      : [h('ul', { class: 'show-list' }, list.slice(0, shown).map(row)),
        list.length > shown ? button({ variant: 'ghost', class: 'show-more', type: 'button', onClick: () => { shown = list.length; paint(); } }, t('pages.showprogress.show_all', { count: list.length })) : null];
    mount(tiles, streakTiles(data.streaks));
    mount(showsEl,
      card({ title: t('pages.showprogress.shows'), sub: t('pages.showprogress.shows_sub'),
        actions: shows.length ? segmented({ label: t('pages.showprogress.which_shows'), value: filter, onChange: (v) => { filter = v; shown = PAGE; paint(); },
          options: [{ value: 'progress', label: t('pages.showprogress.in_progress_n', { count: lists.progress.length }) }, { value: 'finished', label: t('pages.showprogress.finished_n', { count: lists.finished.length }) }, { value: 'all', label: t('pages.showprogress.all') }] }) : null,
        body: [body, problem] }));
  }

  load(true);
  return { tiles, shows: showsEl };
}
