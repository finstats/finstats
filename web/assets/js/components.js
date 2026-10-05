// Reusable UI pieces. All of them return DOM nodes.

import { h, icon, num, compact, duration, durationExact, durEl, relEl, initials, episodeCode, methodLabel, pct, mount, clock, shortStamp } from './dom.js';
import { api, imgItem, imgUser, isAbort, recordRequests, viewCacheGet, viewCacheSet } from './api.js';
import { RANGES, userList, can } from './state.js';
import { dataTable } from './tables.js';
import { button } from '../finui/components/button/button.js';
import { animate, setBusy as setTurning } from '../finui/components/animated-icon/animated-icon.js';

export { emptyState } from '../finui/components/empty/empty.js';
export { errorState } from '../finui/components/error/error.js';
export { spinner } from '../finui/components/spinner/spinner.js';
export { sk } from '../finui/components/skeleton/skeleton.js';
import { spinner } from '../finui/components/spinner/spinner.js';
import { emptyState } from '../finui/components/empty/empty.js';
import { errorState } from '../finui/components/error/error.js';
import { segmented } from '../finui/components/segmented/segmented.js';
import { combobox } from '../finui/components/combobox/combobox.js';
export { inlineError, formField } from '../finui/components/field/field.js';
export { pagination } from '../finui/components/pagination/pagination.js';
export { pageHeader } from '../finui/components/page-header/page-header.js';
export { openModal } from '../finui/components/modal/modal.js';
import { rankList } from '../finui/components/rank-list/rank-list.js';
export { statTile } from '../finui/components/stat-tile/stat-tile.js';
export { facts } from '../finui/components/facts/facts.js';
export { meter } from '../finui/components/meter/meter.js';
import { poster as posterOf } from '../finui/components/poster/poster.js';
import { avatar as avatarOf } from '../finui/components/avatar/avatar.js';
import { statTile } from '../finui/components/stat-tile/stat-tile.js';
import { facts } from '../finui/components/facts/facts.js';
import { typed } from './menus.js';

/** A title's poster, by its id: FinUI's poster, given the address finstats serves it at. */
export function poster(id, name, { w = 120, cls = '', kind = 'primary' } = {}) {
  return posterOf(id ? imgItem(id, w, kind) : null, name, { cls });
}

/** A person's picture, by their id. */
export function avatar(id, name, { size = 28, hasImage = true } = {}) {
  return avatarOf(id && hasImage !== false ? imgUser(id, size * 2 > 96 ? 192 : 96) : null, name, { size });
}
export { segmented } from '../finui/components/segmented/segmented.js';
export { combobox, multiSelect } from '../finui/components/combobox/combobox.js';
export { copyButton } from '../finui/components/copy/copy.js';
export { toggle } from '../finui/components/toggle/toggle.js';

// ---------------------------------------------------------------- layout bits
export { card, chartCard } from '../finui/components/card/card.js';

/** Put a button into / out of its busy state (disabled only while the request runs). */
export function setBusy(btn, busy, busyLabel) {
  // A refresh is its own spinner: it keeps turning, and finishes its turn once it is done, rather than giving way to one.
  const turn = btn.querySelector('svg.icon[data-icon="refresh"]');
  if (busy) {
    btn._label = btn._label || Array.from(btn.childNodes);
    btn.disabled = true;
    btn.setAttribute('aria-busy', 'true');
    if (turn) setTurning(animate(turn, { play: 'hover' }), true);
    btn.replaceChildren(turn || spinner(), h('span', null, busyLabel || 'Working…'));
  } else if (btn._label) {
    btn.disabled = false;
    btn.removeAttribute('aria-busy');
    btn.replaceChildren(...btn._label);
    btn._label = null;
    if (turn) setTurning(turn, false);
  }
}

// ---------------------------------------------------------------- skeletons

/**
 * First load → skeleton (kept ≥300ms so it never flashes).
 * Later loads → keep the previous render, dimmed, until the new data lands.
 */
export function dataView({ container, skeleton, fetch, render, signal }) {
  let loaded = false;
  let seq = 0;
  let shownJson = null; // what is on screen, to skip re-rendering identical data
  async function load() {
    const my = ++seq;
    // Calling fetch() fires the page's GET requests synchronously; their URLs identify this view.
    const { result, key } = recordRequests(fetch);
    let skeletonAt = 0;
    let skeletonTimer = null;
    if (!loaded) {
      const remembered = viewCacheGet(key);
      if (remembered !== undefined) {
        // Been here before: show what we had at once, refresh quietly behind it.
        loaded = true;
        shownJson = JSON.stringify(remembered);
        mount(container, render(remembered));
      } else {
        // A skeleton that flashes for a few milliseconds is worse than none: only show it when
        // loading is actually slow, and once shown keep it long enough to read as deliberate.
        skeletonTimer = setTimeout(() => { if (my === seq && !loaded) { skeletonAt = performance.now(); mount(container, skeleton()); } }, 150);
      }
    } else {
      // A filter changed. If this exact view was seen before, switch to it at once.
      const remembered = viewCacheGet(key);
      const json = remembered === undefined ? null : JSON.stringify(remembered);
      if (json !== null && json !== shownJson) {
        shownJson = json;
        mount(container, render(remembered));
      } else if (json === null) {
        container.classList.add('is-stale');
      }
    }
    container.setAttribute('aria-busy', 'true');
    try {
      const data = await result;
      clearTimeout(skeletonTimer);
      if (skeletonAt) {
        const wait = 300 - (performance.now() - skeletonAt);
        if (wait > 0) await new Promise((r) => setTimeout(r, wait));
      }
      if (my !== seq || (signal && signal.aborted)) return;
      viewCacheSet(key, data);
      const json = JSON.stringify(data);
      container.classList.remove('is-stale');
      if (loaded && json === shownJson) return; // nothing changed; leave the page (and its scroll, hover, focus) alone
      if (!loaded) container.classList.add('fade-in');
      loaded = true;
      shownJson = json;
      mount(container, render(data));
    } catch (e) {
      clearTimeout(skeletonTimer);
      if (isAbort(e) || my !== seq) return;
      if (e.status === 401) return;
      container.classList.remove('is-stale');
      loaded = false;
      shownJson = null;
      mount(container, errorState(e, load));
    } finally {
      if (my === seq) container.removeAttribute('aria-busy');
    }
  }
  return { load, get loaded() { return loaded; } };
}

// ---------------------------------------------------------------- controls
/** Visible options instead of a dropdown (2–5 choices). */
export function rangeControl(days, onChange) {
  return segmented({ label: 'Time range', value: days, onChange,
    options: RANGES.map((r) => ({ value: r.value, label: r.label, title: r.long })) });
}

/** Searchable select. options: [{value, label}] or an async loader. */
/// A dropdown of options, one of them or several.
///
/// `multiple` makes it a list you tick: `value` and what `onChange` is handed are then a
/// comma-separated string rather than one value, which is what a URL carries either way — so a
/// caller that already passes its filter straight into the query string needs no change at all.
/// `searchable` is worth having for a list of people and only noise for a list of four, and a list
/// you tick stays open while you tick it.
export function userCombobox({ value, onChange, signal, multiple = false }) {
  return combobox({ value, onChange, multiple, load: () => userList(signal).then((us) => us.map((u) => ({ value: u.id, label: u.name }))) });
}

/** One row of filters above everything they scope. It stays on screen while the page scrolls (`.filters-bar`), and
 *  says when something has scrolled under it (`is-stuck`). Its parent must be the page, not a wrapper of its own height. */
export function filterBar({ days, onDays, userId, onUser, signal, extra = [] }) {
  const bar = h('div', { class: 'filters filters-bar', role: 'group', 'aria-label': 'Filters' },
    rangeControl(days, onDays),
    onUser && can('see_everyone') ? userCombobox({ value: userId || '', onChange: onUser, signal, multiple: true }) : null,
    extra);
  // One look per frame while scrolling; the listener goes with the page (or, for a caller without a signal, with the bar).
  let queued = false, seen = false;
  const look = () => {
    queued = false;
    if (!bar.isConnected) { if (seen) removeEventListener('scroll', onScroll); return; }
    seen = true;
    bar.classList.toggle('is-stuck', bar.getBoundingClientRect().top <= parseFloat(getComputedStyle(bar).top) + 0.5 && scrollY > 0);
  };
  const onScroll = () => { if (!queued) { queued = true; requestAnimationFrame(look); } };
  addEventListener('scroll', onScroll, { passive: true, signal });
  return bar;
}

/** "Open in Jellyfin": the title's own page there, in a new tab. Null without a link (a title Jellyfin no longer has).
 *  `compact` is the icon alone, for a row of a list, named for screen readers and on hover. */
export function openInJellyfin(link, { compact = false } = {}) {
  if (!link) return null;
  if (compact) return button({ variant: 'icon', class: 'open-in-jellyfin', href: link, target: '_blank', rel: 'noopener noreferrer', 'aria-label': 'Open in Jellyfin', title: 'Open in Jellyfin' }, icon('external', 15));
  return button({ variant: 'primary', class: 'open-in-jellyfin', href: link, target: '_blank', rel: 'noopener noreferrer' }, icon('play', 14), 'Open in Jellyfin');
}

export function methodBadge(method) {
  const cls = { DirectPlay: 'm-direct', DirectStream: 'm-stream', Transcode: 'm-transcode' }[method] || '';
  return h('span', { class: 'fui-badge method ' + cls }, h('span', { class: 'fui-badge__dot' }), methodLabel(method));
}

export { chip } from '../finui/components/chip/chip.js';

/** Ranked poster rows for movies/series/music, avatar rows for users. */
export function topList(rows, { kind = 'items', empty = 'No plays in this range.' } = {}) {
  if (!rows || !rows.length) return h('div', { class: 'fui-empty--chart fui-empty--chart-sm' }, empty);
  return rankList(rows.map((r) => ({
    href: !r.id ? null : kind === 'users' ? `/users/${r.id}` : kind === 'libraries' ? `/libraries/${r.id}` : kind === 'items' ? `/items/${r.id}` : null,
    thumb: kind === 'users' ? avatar(r.id, r.name, { size: 36 }) : kind === 'items' ? poster(r.image_item_id, r.name, { w: 120, cls: 'fui-poster--sm' }) : null,
    name: r.name,
    sub: [r.sub, r.users != null ? `${num(r.users)} ${r.users === 1 ? 'user' : 'users'}` : null].filter(Boolean).join(' · ') || null,
    value: durEl(r.watch_s, 'mono fui-rank-list__watch'),
    note: `${num(r.plays)} ${r.plays === 1 ? 'play' : 'plays'}`,
  })));
}

// ---------------------------------------------------------------- plays table
export function playTitle(p, { link = true } = {}) {
  const code = episodeCode(p.season_number, p.episode_number);
  const canLink = link && p.item_exists !== false;
  const main = canLink && p.item_id ? h('a', { href: `/items/${p.item_id}`, dataset: typed(p.item_type), onClick: (e) => e.stopPropagation() }, p.item_name || 'Unknown item') : h('span', null, p.item_name || 'Unknown item');
  if (!p.series_name) return h('div', { class: 'play-title' }, h('div', { class: 'play-title-main' }, main));
  const series = canLink && p.series_id ? h('a', { href: `/items/${p.series_id}`, dataset: typed('Series'), onClick: (e) => e.stopPropagation() }, p.series_name) : h('span', null, p.series_name);
  return h('div', { class: 'play-title' }, h('div', { class: 'play-title-main' }, series),
    h('div', { class: 'play-title-sub' }, code ? h('span', { class: 'mono' }, code) : null, code ? ' · ' : null, main));
}

export function completionEl(p) {
  if (p.completion == null) return h('span', { class: 'muted mono' }, '–');
  const c = Math.max(0, Math.min(1, p.completion));
  // Where playback stopped, as a position in the title. Imported plays never recorded one.
  const stoppedAt = p.position_s != null && p.runtime_s ? `${clock(p.position_s)} / ${clock(p.runtime_s)}` : null;
  return h('span', { class: 'completion-cell' },
    h('span', { class: 'completion', title: stoppedAt ? `Stopped at ${stoppedAt}` : `Stopped at ${pct(c)}` },
      h('span', { class: 'fui-meter', role: 'img', 'aria-label': `${pct(c)} watched` }, h('span', { class: 'fui-meter__fill', style: { width: c * 100 + '%' } })),
      h('span', { class: 'mono' }, pct(c))),
    stoppedAt ? h('span', { class: 'cell-sub mono' }, stoppedAt) : null);
}

/**
 * rows: Play[]; onOpen(play, rowEl) opens the detail modal. `sort: {key, dir, onSort}` when the list is
 * paginated and the server does the sorting; without it the rows on screen are sorted in the browser.
 */
export function playsTable(rows, { showUser = true, onOpen, empty = 'No plays match these filters.', sort = null } = {}) {
  if (!rows || !rows.length) return emptyState(empty);
  const admin = can('see_network'); // the IP column
  // data-first: the direction a first click gives, so it matches what the browser-side tables do.
  const th = (key, label, first, cls) => h('th', { class: cls || null, 'data-key': key, 'data-first': first }, label);
  return dataTable(h('table', { class: 'fui-data-table fui-data-table--hover plays' },
    h('thead', null, h('tr', null,
      showUser ? th('user', 'User', 'asc') : null, th('title', 'Title', 'asc'), th('when', 'When', 'desc'), th('watched', 'Watched', 'desc', 'r'),
      th('progress', 'Progress', 'desc'), th('client', 'Client', 'asc'), th('method', 'Method', 'asc'), admin ? th('ip', 'IP address', 'asc') : null)),
    h('tbody', null, rows.map((p) => {
      const tr = h('tr', { tabindex: 0, class: p.active ? 'is-live' : '', 'aria-label': `Open details for ${p.item_name || 'play'}` },
        showUser ? h('td', null, h('a', { class: 'user-cell', href: `/users/${p.user_id}`, onClick: (e) => e.stopPropagation() }, avatar(p.user_id, p.user_name, { size: 22 }), h('span', null, p.user_name))) : null,
        h('td', { class: 'td-title' }, h('div', { class: 'title-cell' }, poster(p.image_item_id, p.series_name || p.item_name, { w: 120, cls: 'fui-poster--xs' }), playTitle(p),
          p.group_size > 1 ? h('span', { class: 'group-mark', role: 'img', title: `Watched together · ${p.group_size} people`, 'aria-label': `Watched together by ${p.group_size} people` }, icon('together', 13)) : null)),
        h('td', { 'data-sort': p.active ? String(Date.now()) : null }, p.active ? h('span', { class: 'fui-badge fui-badge--live' }, h('span', { class: 'fui-badge__dot' }), 'Playing now')
          : h('span', { class: 'when-cell' }, relEl(p.ended_at || p.started_at), h('span', { class: 'cell-sub mono' }, shortStamp(p.ended_at || p.started_at)))),
        h('td', { class: 'r' }, durEl(p.duration_s)),
        h('td', null, completionEl(p)),
        h('td', null, h('div', { class: 'client-cell' }, h('span', null, p.client || '–'), h('span', { class: 'muted' }, p.device_name || ''))),
        h('td', null, methodBadge(p.play_method)),
        admin ? h('td', { class: 'mono' }, p.remote_ip ? h('span', { class: 'ip-cell' }, p.remote_ip,
          p.is_local == null ? null : h('span', { class: 'ip-net', role: 'img', title: p.is_local ? 'Local network' : 'Remote', 'aria-label': p.is_local ? 'Local network' : 'Remote' }, icon(p.is_local ? 'lan' : 'globe', 12))) : '–') : null);
      if (onOpen) {
        tr.addEventListener('click', (e) => { if (!e.target.closest('a,button')) onOpen(p, tr); });
        tr.addEventListener('keydown', (e) => { if ((e.key === 'Enter' || e.key === ' ') && e.target === tr) { e.preventDefault(); onOpen(p, tr); } });
      }
      return tr;
    }))), { server: sort, filter: false });
}

// ---------------------------------------------------------------- modal
// The open dialogs, newest last. Every one listens for keys on the document, and only the one on top may
// answer: Esc in the search palette over a play's details used to close both.
// ---------------------------------------------------------------- definition grid
export { num, compact, duration, durationExact, api };
