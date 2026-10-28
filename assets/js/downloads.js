// What is arriving right now, as Sonarr and Radarr report it. Shared by the Pipeline tab and the dashboard.
//
// The list is polled while a page shows it; the server keeps its snapshot fresh only for as long as somebody is
// actually looking (`?live=1`), so a forgotten tab or the prefetcher never keeps those services busy.

import { h, icon, bytes, duration, relTime, pct } from './dom.js';
import { api } from './api.js';
import { upcomingPoster } from './upcoming.js';
import { t } from './i18n.js';

export const loadDownloads = (signal, live = true) => api.get('/downloads', live ? { live: 1 } : null, { signal });

const STATE = {
  downloading: { cls: 'fui-badge--info', icon: 'download', label: t('downloads.state.downloading') },
  importing: { cls: 'fui-badge--info', icon: 'inbox', label: t('downloads.state.importing') },
  stalled: { cls: 'fui-badge--warning', icon: 'alert', label: t('downloads.state.stalled') },
  queued: { cls: 'fui-badge--info', icon: 'clock', label: t('downloads.state.queued') },
  paused: { cls: 'fui-badge--info', icon: 'pause', label: t('downloads.state.paused') },
  checking: { cls: 'fui-badge--info', icon: 'refresh', label: t('downloads.state.checking') },
  failed: { cls: 'fui-badge--critical', icon: 'alert', label: t('downloads.state.failed') },
  unknown: { cls: 'fui-badge--info', icon: 'info', label: t('common.unknown') },
};

export const speed = (bps) => (bps > 0 ? `${bytes(bps)}/s` : '–');
export const stateLabel = (key) => (STATE[key] || STATE.unknown).label;

/** The bar every progress is shown with. Its width is a style property, which the CSP allows through JS. */
export function progressBar(fraction, { label } = {}) {
  const pctText = pct(fraction || 0);
  const fill = h('span', { class: 'dl-fill' });
  fill.style.width = `${Math.max(0, Math.min(1, fraction || 0)) * 100}%`;
  return h('span', { class: 'dl-bar', role: 'progressbar', 'aria-valuenow': Math.round((fraction || 0) * 100), 'aria-valuemin': '0', 'aria-valuemax': '100', 'aria-label': label || t('downloads.done', { pct: pctText }) }, fill);
}

function row(d) {
  const st = STATE[d.state] || STATE.unknown;
  const who = d.requested_by;
  return h('li', { class: 'dl-row' },
    upcomingPoster({ poster: d.poster, title: d.title, series_title: null }, { w: 96, cls: 'fui-poster--sm' }),
    h('div', { class: 'dl-main' },
      h('div', { class: 'dl-line' },
        d.item_id ? h('a', { class: 'dl-title', href: `/items/${d.item_id}` }, d.title) : h('span', { class: 'dl-title' }, d.title),
        d.sub ? h('span', { class: 'muted' }, ' · ' + d.sub) : null),
      h('div', { class: 'dl-meta' },
        h('span', { class: 'fui-badge--status ' + st.cls }, icon(st.icon, 13), st.label),
        d.size ? h('span', { class: 'mono' }, t('downloads.of', { done: bytes(d.size * (d.progress || 0)), size: bytes(d.size) })) : null,
        d.down_bps > 0 ? h('span', { class: 'mono dl-down' }, icon('download', 12), speed(d.down_bps)) : null,
        d.eta_s ? h('span', { class: 'mono' }, t('downloads.left', { time: duration(d.eta_s) })) : null,
        d.client ? h('span', { class: 'muted', title: t('downloads.handed', { service: d.service_name || 'Sonarr', client: d.client }) }, d.client) : null,
        who ? h('span', { class: 'dl-who' }, icon('inbox', 12), who.user_id ? h('a', { href: `/users/${who.user_id}` }, who.user_name) : who.user_name) : null),
      d.error ? h('p', { class: 'dl-error' }, icon('alert', 13), d.error) : null,
      progressBar(d.progress),
      d.release && d.release !== d.title ? h('div', { class: 'dl-release mono', title: d.release }, d.release) : null));
}

/** The whole list, with its totals. `compact` is the dashboard's version: no totals row, a few rows only. */
export function downloadsList(d, { compact = false, limit = 0 } = {}) {
  const rows = (d.rows || []).slice(0, limit || undefined);
  const tot = d.totals || {};
  const totals = compact ? null : h('div', { class: 'dl-totals' },
    h('span', { class: 'mono dl-down', title: t('downloads.speed_title') }, icon('download', 13), speed(tot.down_bps)),
    h('span', null, t('downloads.totals.downloading', { count: tot.downloading || 0 })),
    tot.queued ? h('span', null, t('downloads.totals.queued', { count: tot.queued })) : null,
    tot.importing ? h('span', null, t('downloads.totals.importing', { count: tot.importing })) : null,
    tot.failed ? h('span', { class: 'fui-badge--status fui-badge--critical' }, icon('alert', 13), t('downloads.totals.failed', { count: tot.failed })) : null,
    d.at ? h('span', { class: 'muted dl-when' }, t('downloads.updated', { when: relTime(d.at) })) : null);
  const problems = (d.problems || []).map((p) => h('p', { class: 'dl-error' }, icon('alert', 13), `${p.service}: ${p.error}`));
  return [totals, ...problems, rows.length ? h('ul', { class: 'dl-list' }, rows.map(row)) : null];
}

export const nothingDownloading = (d) => !(d.rows || []).length;
