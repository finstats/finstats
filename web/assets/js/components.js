// Reusable UI pieces. All of them return DOM nodes.

import { h, icon, num, compact, duration, durationExact, durEl, relEl, initials, episodeCode, methodLabel, pct, mount, clock, shortStamp } from './dom.js';
import { api, imgItem, imgUser, isAbort, recordRequests, viewCacheGet, viewCacheSet } from './api.js';
import { RANGES, userList, can } from './state.js';
import { dataTable } from './tables.js';
import { button } from '../finui/components/button/button.js';

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
export { segmented } from '../finui/components/segmented/segmented.js';
export { combobox, multiSelect } from '../finui/components/combobox/combobox.js';
export { copyButton } from '../finui/components/copy/copy.js';
export { toggle } from '../finui/components/toggle/toggle.js';

// ---------------------------------------------------------------- layout bits
export function pageHeader(title, sub, right) {
  return h('header', { class: 'page-header' },
    h('div', null, h('h1', { class: 'page-title' }, title), sub ? h('p', { class: 'page-sub' }, sub) : null),
    right ? h('div', { class: 'page-header-right' }, right) : null);
}

export { card, chartCard } from '../finui/components/card/card.js';

/** Put a button into / out of its busy state (disabled only while the request runs). */
export function setBusy(btn, busy, busyLabel) {
  if (busy) {
    btn._label = btn._label || Array.from(btn.childNodes);
    btn.disabled = true;
    btn.setAttribute('aria-busy', 'true');
    btn.replaceChildren(spinner(), h('span', null, busyLabel || 'Working…'));
  } else if (btn._label) {
    btn.disabled = false;
    btn.removeAttribute('aria-busy');
    btn.replaceChildren(...btn._label);
    btn._label = null;
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

/** One row of filters above everything they scope. */
export function filterBar({ days, onDays, userId, onUser, signal, extra = [] }) {
  return h('div', { class: 'filters', role: 'group', 'aria-label': 'Filters' },
    rangeControl(days, onDays),
    onUser && can('see_everyone') ? userCombobox({ value: userId || '', onChange: onUser, signal, multiple: true }) : null,
    extra);
}

export function pagination({ page, perPage, total, onPage }) {
  const pages = Math.max(1, Math.ceil(total / perPage));
  const from = total ? (page - 1) * perPage + 1 : 0, to = Math.min(total, page * perPage);
  // Buttons stay enabled; out-of-range clicks are simply ignored (aria-disabled communicates the edge).
  const mk = (lbl, ic, target, off) => button({ size: 'sm', variant: 'ghost', type: 'button', 'aria-label': lbl, 'aria-disabled': off ? 'true' : null,
    onClick: () => { if (!off) onPage(target); } }, icon(ic, 14));
  return h('nav', { class: 'pager', 'aria-label': 'Pagination' },
    h('span', { class: 'pager-info mono' }, `${num(from)}–${num(to)} of ${num(total)}`),
    h('div', { class: 'pager-btns' }, mk('Previous page', 'chevronLeft', page - 1, page <= 1),
      h('span', { class: 'pager-page mono' }, `${page} / ${pages}`),
      mk('Next page', 'chevronRight', page + 1, page >= pages)));
}

// ---------------------------------------------------------------- media
export function poster(id, name, { w = 120, cls = '', kind = 'primary' } = {}) {
  const box = h('span', { class: 'poster ' + cls, 'aria-hidden': 'true' });
  const fallback = () => mount(box, h('span', { class: 'poster-fallback' }, initials(name)));
  if (!id) { fallback(); return box; }
  box.append(h('img', { src: imgItem(id, w, kind), alt: '', loading: 'lazy', decoding: 'async', onError: fallback }));
  return box;
}

/** "Open in Jellyfin": the title's own page there, in a new tab. Null without a link (a title Jellyfin no longer has).
 *  `compact` is the icon alone, for a row of a list, named for screen readers and on hover. */
export function openInJellyfin(link, { compact = false } = {}) {
  if (!link) return null;
  if (compact) return button({ variant: 'icon', class: 'open-in-jellyfin', href: link, target: '_blank', rel: 'noopener noreferrer', 'aria-label': 'Open in Jellyfin', title: 'Open in Jellyfin' }, icon('external', 15));
  return button({ variant: 'primary', class: 'open-in-jellyfin', href: link, target: '_blank', rel: 'noopener noreferrer' }, icon('play', 14), 'Open in Jellyfin');
}

export function avatar(id, name, { size = 28, hasImage = true } = {}) {
  const box = h('span', { class: 'avatar', style: { width: size + 'px', height: size + 'px', fontSize: Math.max(10, size * 0.38) + 'px' }, 'aria-hidden': 'true' });
  const fallback = () => mount(box, initials(name));
  if (!id || hasImage === false) { fallback(); return box; }
  box.append(h('img', { src: imgUser(id, size * 2 > 96 ? 192 : 96), alt: '', loading: 'lazy', decoding: 'async', onError: fallback }));
  return box;
}

export function methodBadge(method) {
  const cls = { DirectPlay: 'm-direct', DirectStream: 'm-stream', Transcode: 'm-transcode' }[method] || '';
  return h('span', { class: 'fui-badge method ' + cls }, h('span', { class: 'fui-badge__dot' }), methodLabel(method));
}

export { chip } from '../finui/components/chip/chip.js';

// ---------------------------------------------------------------- stats
function deltaEl(cur, prev) {
  if (prev == null) return null;
  if (!prev && !cur) return h('span', { class: 'delta flat' }, icon('minus', 13), 'No change');
  if (!prev) return h('span', { class: 'delta up' }, icon('trendUp', 13), 'New');
  const d = (cur - prev) / prev;
  if (Math.abs(d) < 0.005) return h('span', { class: 'delta flat' }, icon('minus', 13), 'No change');
  const up = d > 0;
  return h('span', { class: 'delta ' + (up ? 'up' : 'down') }, icon(up ? 'trendUp' : 'trendDown', 13),
    `${up ? '+' : '−'}${Math.abs(d) >= 10 ? Math.round(Math.abs(d) * 100) : (Math.abs(d) * 100).toFixed(Math.abs(d) < 0.1 ? 1 : 0)}%`);
}

export function statTile({ label, value, title, current, previous, vsLabel, spark, hint }) {
  return h('div', { class: 'tile' },
    h('div', { class: 'tile-label' }, label),
    h('div', { class: 'tile-row' }, h('div', { class: 'tile-value', title }, value), spark || null),
    h('div', { class: 'tile-foot' }, previous != null ? [deltaEl(current, previous), h('span', { class: 'tile-vs' }, vsLabel)] : hint ? h('span', { class: 'tile-vs' }, hint) : h('span', { class: 'tile-vs' }, ' ')));
}

/** Ranked poster rows for movies/series/music, avatar rows for users. */
export function topList(rows, { kind = 'items', empty = 'No plays in this range.' } = {}) {
  if (!rows || !rows.length) return h('div', { class: 'fui-empty--chart fui-empty--chart-sm' }, empty);
  return h('ol', { class: 'toplist' }, rows.map((r, i) => {
    const href = !r.id ? null : kind === 'users' ? `/users/${r.id}` : kind === 'libraries' ? `/libraries/${r.id}` : kind === 'items' ? `/items/${r.id}` : null;
    const thumb = kind === 'users' ? avatar(r.id, r.name, { size: 36 }) : kind === 'items' ? poster(r.image_item_id, r.name, { w: 120, cls: 'poster-sm' }) : null;
    const name = href ? h('a', { href, class: 'toplist-name' }, r.name) : h('span', { class: 'toplist-name' }, r.name);
    return h('li', { class: 'toplist-row' },
      h('span', { class: 'toplist-rank mono' }, String(i + 1)),
      thumb,
      h('div', { class: 'toplist-main' }, name,
        h('div', { class: 'toplist-sub' }, [r.sub, r.users != null ? `${num(r.users)} ${r.users === 1 ? 'user' : 'users'}` : null].filter(Boolean).join(' · ') || ' ')),
      h('div', { class: 'toplist-nums' }, durEl(r.watch_s, 'mono toplist-watch'), h('span', { class: 'mono toplist-plays' }, `${num(r.plays)} ${r.plays === 1 ? 'play' : 'plays'}`)));
  }));
}

// ---------------------------------------------------------------- plays table
export function playTitle(p, { link = true } = {}) {
  const code = episodeCode(p.season_number, p.episode_number);
  const canLink = link && p.item_exists !== false;
  const main = canLink && p.item_id ? h('a', { href: `/items/${p.item_id}`, onClick: (e) => e.stopPropagation() }, p.item_name || 'Unknown item') : h('span', null, p.item_name || 'Unknown item');
  if (!p.series_name) return h('div', { class: 'play-title' }, h('div', { class: 'play-title-main' }, main));
  const series = canLink && p.series_id ? h('a', { href: `/items/${p.series_id}`, onClick: (e) => e.stopPropagation() }, p.series_name) : h('span', null, p.series_name);
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
      h('span', { class: 'meter', role: 'img', 'aria-label': `${pct(c)} watched` }, h('span', { class: 'meter-fill', style: { width: c * 100 + '%' } })),
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
  return dataTable(h('table', { class: 'table table-hover plays' },
    h('thead', null, h('tr', null,
      showUser ? th('user', 'User', 'asc') : null, th('title', 'Title', 'asc'), th('when', 'When', 'desc'), th('watched', 'Watched', 'desc', 'r'),
      th('progress', 'Progress', 'desc'), th('client', 'Client', 'asc'), th('method', 'Method', 'asc'), admin ? th('ip', 'IP address', 'asc') : null)),
    h('tbody', null, rows.map((p) => {
      const tr = h('tr', { tabindex: 0, class: p.active ? 'is-live' : '', 'aria-label': `Open details for ${p.item_name || 'play'}` },
        showUser ? h('td', null, h('a', { class: 'user-cell', href: `/users/${p.user_id}`, onClick: (e) => e.stopPropagation() }, avatar(p.user_id, p.user_name, { size: 22 }), h('span', null, p.user_name))) : null,
        h('td', { class: 'td-title' }, h('div', { class: 'title-cell' }, poster(p.image_item_id, p.series_name || p.item_name, { w: 120, cls: 'poster-xs' }), playTitle(p),
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
const modalStack = [];
/** X button, Esc and click-outside all close it; focus returns to the trigger. */
export function openModal({ title, body, wide = false, onClose, initialFocus, labelId = 'modal-title', bare = false, cls = '' }) {
  const trigger = document.activeElement;
  const closeBtn = button({ variant: 'icon', class: 'modal-x', type: 'button', 'aria-label': 'Close' }, icon('x', 16));
  const bodyEl = h('div', { class: bare ? '' : 'modal-body' }, body);
  const dialog = h('div', { class: ['modal', wide && 'modal-wide', cls], role: 'dialog', 'aria-modal': 'true', 'aria-labelledby': bare ? null : labelId, 'aria-label': bare ? title : null },
    bare ? null : h('div', { class: 'modal-head' }, h('h2', { class: 'modal-title', id: labelId }, title), closeBtn), bodyEl);
  const overlay = h('div', { class: 'overlay' }, dialog);
  let closed = false;

  function close() {
    if (closed) return;
    closed = true; modalStack.splice(modalStack.indexOf(onKey), 1);
    document.removeEventListener('keydown', onKey, true);
    overlay.classList.add('is-closing');
    setTimeout(() => overlay.remove(), 140);
    if (!modalStack.length) document.documentElement.classList.remove('no-scroll');
    if (trigger && trigger.isConnected && trigger.focus) trigger.focus();
    if (onClose) onClose();
  }
  function onKey(e) {
    if (modalStack[modalStack.length - 1] !== onKey) return;
    if (e.key === 'Escape') { e.preventDefault(); e.stopPropagation(); close(); return; }
    if (e.key !== 'Tab') return;
    const f = dialog.querySelectorAll('a[href],button:not([disabled]),input:not([disabled]),select,textarea,[tabindex]:not([tabindex="-1"])');
    if (!f.length) return;
    const first = f[0], last = f[f.length - 1];
    if (e.shiftKey && document.activeElement === first) { e.preventDefault(); last.focus(); }
    else if (!e.shiftKey && document.activeElement === last) { e.preventDefault(); first.focus(); }
  }
  closeBtn.addEventListener('click', close);
  overlay.addEventListener('pointerdown', (e) => { if (e.target === overlay) close(); });
  document.addEventListener('keydown', onKey, true);
  document.body.append(overlay);
  document.documentElement.classList.add('no-scroll');
  modalStack.push(onKey);
  requestAnimationFrame(() => (initialFocus && initialFocus.isConnected ? initialFocus : closeBtn.isConnected ? closeBtn : dialog).focus());
  return { close, body: bodyEl, dialog };
}

// ---------------------------------------------------------------- definition grid
/** A grid of label/value pairs. `{ wide: true }` gives a pair the whole row: a long value (a file path) squeezed into
 *  one cell broke onto a line every few characters beside a row left empty. */
export function facts(pairs) {
  return h('dl', { class: 'facts' }, pairs.filter(Boolean).map(([k, v, opts]) =>
    h('div', { class: ['fact', opts && opts.wide && 'fact-wide'] }, h('dt', null, k), h('dd', { class: opts && opts.mono ? 'mono' : '' }, v == null || v === '' ? '–' : v))));
}

export { num, compact, duration, durationExact, api };
