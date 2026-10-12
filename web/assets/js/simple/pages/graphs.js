// Simple mode's Graphs, the set a classic stats server draws, in three tabs. Media Type: plays day by day, by day of the
// week and by hour, and the top ten apps and people, each split into TV, movies and music. Stream Type: the same days and
// the most streams at once each day by how they played, then resolutions, apps and people by how they played. Play
// Totals: month by month. By play count or duration, over the last N days (N months for the totals), for everybody or
// the people chosen (with see_everyone).

import { h, icon, compact, store, mount } from '../../dom.js';
import { api } from '../../api.js';
import { can } from '../../state.js';
import { t, tParts, lang } from '../../i18n.js';
import { replaceQuery } from '../../router.js';
import { dataView, segmented, userCombobox, sk } from '../../components.js';
import { lineChart } from '../../../finui/components/line-chart/line-chart.js';
import { barChart } from '../../../finui/components/bar-chart/bar-chart.js';

const TABS = [{ key: 'media', label: t('simple.graphs.tab.media') }, { key: 'streams', label: t('simple.graphs.tab.streams') }, { key: 'totals', label: t('simple.graphs.tab.totals') }];
const MEASURES = [{ value: 'plays', label: t('simple.home.measure.plays') }, { value: 'duration', label: t('simple.home.measure.duration') }];
const DAYS_KEY = 'finstats.simpleGraphDays', MONTHS_KEY = 'finstats.simpleGraphMonths', BY_KEY = 'finstats.simpleBy';
// The parts a breakdown is split into, as the answer names them, in the order a series takes its colour. The kinds of media
// are in FinStats' order, so each wears the colour it has on every other chart (films, episodes, music, the rest).
const KINDS = { Movie: t('simple.graphs.kind.movies'), Episode: t('simple.graphs.kind.tv'), Audio: t('simple.graphs.kind.music'), Other: t('common.other') };
const METHODS = { DirectPlay: t('simple.graphs.method.DirectPlay'), DirectStream: t('simple.graphs.method.DirectStream'), Transcode: t('simple.graphs.method.Transcode') };
const WEEKDAYS = Array.from({ length: 7 }, (_, i) => new Date(Date.UTC(2024, 0, 1 + i)).toLocaleDateString(lang(), { weekday: 'long', timeZone: 'UTC' }));
const HOURS = Array.from({ length: 24 }, (_, i) => String(i).padStart(2, '0'));

const valid = (v) => Number.isInteger(v) && v >= 1 && v <= 3650;
const hm = (sec) => { const m = Math.round((Number(sec) || 0) / 60); return t('simple.graphs.hm', { hours: Math.floor(m / 60), minutes: m % 60 }); };
const dayLabel = (iso) => new Date(`${iso}T12:00:00`).toLocaleDateString(lang(), { month: 'short', day: 'numeric' });
const monthLabel = (ym) => new Date(`${ym}-15T12:00:00`).toLocaleDateString(lang(), { month: 'short', year: 'numeric' });

/** The days from the first of the month `months - 1` months back to today, both counted: the window of "last N months". */
export function monthsAsDays(months, now = new Date()) {
  const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
  const first = new Date(now.getFullYear(), now.getMonth() - (months - 1), 1);
  return Math.round((today - first) / 86_400_000) + 1;
}

export function scopeOf(query) {
  const days = [Number(query.get('days')), Number(store.get(DAYS_KEY))].find(valid) || 30;
  const months = [Number(query.get('months')), Number(store.get(MONTHS_KEY))].find((v) => valid(v) && v <= 120) || 12;
  const by = [query.get('by'), store.get(BY_KEY)].find((v) => MEASURES.some((m) => m.value === v)) || 'plays';
  const tab = TABS.some((x) => x.key === query.get('tab')) ? query.get('tab') : 'media';
  return { days, months, by, tab, users: can('see_everyone') ? query.get('user_id') || '' : '' };
}

/** One tab's data. Shared with the prefetcher, so both ask for exactly the same addresses. */
export function loadTab({ tab, days, months, by, users }, signal) {
  const f = { days, user_id: users || undefined }, o = { signal };
  const split = (group, of, extra = {}) => api.get('/stats/split', { ...f, group, by: of, ...extra }, o);
  // The top tens are ranked by what is drawn.
  const ranked = (group, of) => split(group, of, { limit: 10, sort: by === 'duration' ? 'watch' : undefined });
  const people = (of) => (can('see_everyone') ? ranked('user', of) : null);
  const all = (parts) => Promise.all(Object.values(parts)).then((v) => Object.fromEntries(Object.keys(parts).map((k, i) => [k, v[i]])));
  if (tab === 'totals') return all({ months: api.get('/stats/split', { days: monthsAsDays(months), user_id: users || undefined, group: 'month', by: 'type' }, o) });
  if (tab === 'streams') {
    return all({ daily: split('day', 'method'), peaks: api.get('/stats/insights', { ...f, parts: 'concurrency' }, o), source: ranked('source_resolution', 'method'),
      stream: ranked('stream_resolution', 'method'), clients: ranked('client', 'method'), users: people('method') });
  }
  return all({ daily: split('day', 'type'), heat: api.get('/stats/heatmap', f, o), clients: ranked('client', 'type'), users: people('type') });
}
export const prefetchGraphs = ({ query, signal }) => [() => loadTab(scopeOf(query), signal)];

export default function graphs(ctx) {
  ctx.title(t('simple.nav.graphs'));
  let { days, months, by, tab, users } = scopeOf(ctx.query);
  const body = h('div', { class: 'simple-graphs__body' });
  // Count and duration are the same answers drawn twice: switching between them draws what is here again at once, since
  // the view draws an answer only when it differs from the last, then asks again for the top tens ranked the other way.
  let last = null;
  const draw = (d) => (tab === 'streams' ? streams(d, by, days) : tab === 'totals' ? totals(d, by, months) : media(d, by, days));
  const view = dataView({
    container: body, signal: ctx.signal, skeleton: () => [sk.cardBlock(260), sk.cardBlock(220)],
    fetch: () => loadTab({ tab, days, months, by, users }, ctx.signal),
    render: (d) => { last = d; return draw(d); },
  });
  const sync = () => replaceQuery({ tab: tab === 'media' ? null : tab, days: days === 30 ? null : days, months: months === 12 ? null : months, by: by === 'plays' ? null : by, user_id: users || null });
  // One field, in days or in months as the tab counts.
  const inMonths = tab === 'totals';
  const field = h('input', { class: 'simple-days__field', id: 'simple-graphs-days', type: 'number', min: '1', max: inMonths ? '120' : '3650', value: String(inMonths ? months : days), 'aria-label': inMonths ? t('simple.graphs.months_aria') : t('simple.home.days_aria') });
  const setSpan = () => {
    const v = Number(field.value), now = inMonths ? months : days;
    if (!valid(v) || (inMonths && v > 120) || v === now) { field.value = String(now); return; }
    if (inMonths) { months = v; store.set(MONTHS_KEY, String(v)); } else { days = v; store.set(DAYS_KEY, String(v)); }
    sync(); view.load();
  };
  field.addEventListener('change', setSpan);
  field.addEventListener('keydown', (e) => { if (e.key === 'Enter') setSpan(); });
  const keep = (extra) => { const q = new URLSearchParams(extra); if (users) q.set('user_id', users); const s = q.toString(); return s ? `?${s}` : ''; };
  const tabs = h('nav', { class: 'simple-tabs simple-tabs--boxed', 'aria-label': t('simple.nav.graphs') },
    TABS.map((x) => h('a', { href: `/graphs${keep(x.key === 'media' ? {} : { tab: x.key })}`, 'aria-current': x.key === tab ? 'page' : null }, x.label)));
  ctx.root.append(h('div', { id: 'simple-graphs' },
    h('div', { class: 'simple-panel-head' }, h('h1', { class: 'simple-page-title' }, icon('chart', 22), t('simple.nav.graphs')),
      h('div', { class: 'simple-head__tools' },
        can('see_everyone') ? h('div', { class: 'simple-history__who' }, userCombobox({ value: users, multiple: true, signal: ctx.signal, onChange: (v) => { users = v; sync(); view.load(); } })) : null,
        h('span', { class: 'simple-graphs__measure' }, segmented({ label: t('simple.graphs.measure'), value: by, options: MEASURES, onChange: (v) => {
          by = v; store.set(BY_KEY, v); sync();
          if (last) mount(body, draw(last));
          view.load();
        } })),
        h('label', { class: 'simple-days', for: 'simple-graphs-days' }, tParts(inMonths ? 'simple.graphs.last_months' : 'simple.home.last_days', { field })
          .map((x) => (typeof x === 'string' ? x.trim() : x)).filter((x) => x !== '').map((x) => (typeof x === 'string' ? h('span', null, x) : x))))),
    h('div', { class: 'simple-panel' }, tabs, body)));
  view.load();
}

// ---- drawing

/** A chart's title or line, worded for the measure (and the bucket, where a series has one): every combination is a sentence of its own. */
const graphWords = (key, by, bucket = null) => t(`simple.graphs.${key}.${bucket ? `${bucket === 'week' ? 'week' : 'day'}.` : ''}${by === 'plays' ? 'plays' : 'duration'}`);
const fmt = (by) => (by === 'plays' ? compact : hm);
const pick = (by, pair) => (pair ? (by === 'plays' ? pair[0] : pair[1]) : 0);
const spanOf = (n, unit) => (n === 1 ? (unit === 'days' ? t('time.today') : t('simple.graphs.this_month')) : t(unit === 'days' ? 'simple.graphs.last_n_days' : 'simple.graphs.last_n_months', { count: n }));
// Past 120 days a series counts weeks, not days (overview.daily's rule), and its title says so: `graphWords` takes the bucket.

/** A chart under its heading, with what it shows in a line: the way a classic stats page lays one out. */
const graph = (iconName, title, span, help, chart, wide = false) => h('section', { class: ['simple-graph', wide && 'simple-graph--wide'] },
  h('h2', { class: 'simple-graph__title' }, icon(iconName, 16), h('span', { class: 'simple-graph__name' }, title), h('small', null, span)),
  h('p', { class: 'simple-graph__help' }, help), chart);

/** The parts of a breakdown that hold anything, named, in the answer's order; none at all keeps the first three. */
function partsOf(answer, names, rows = answer?.rows || []) {
  const keys = (answer?.keys || Object.keys(names)).filter((k) => names[k]);
  const held = keys.filter((k) => rows.some((r) => r.by && r.by[k] && r.by[k][0]));
  return (held.length ? held : keys.slice(0, 3)).map((k) => [k, names[k]]);
}
/** Films, TV and music always, each keeping its colour even when it has nothing; the rest only when it holds something. */
const kindsOf = (answer, rows = []) => Object.entries(KINDS).filter(([k], i) => i < 3 || rows.some((r) => r.by && r.by[k] && r.by[k][0]));

/** A series of days or months as lines, one per part. */
function overTime(answer, parts, by, label) {
  const rows = answer?.rows || [];
  return lineChart({ labels: rows.map((r) => label(r.date)), format: fmt(by), height: 260,
    series: parts.map(([k, name]) => ({ name, values: rows.map((r) => pick(by, r.by && r.by[k])) })) });
}

/** Ranked rows (apps, people, resolutions) as stacked columns, one stack per part. */
function columns(answer, parts, by) {
  const rows = answer?.rows || [];
  return rows.length ? barChart({ categories: rows.map((r) => r.name), format: fmt(by), series: parts.map(([k, name]) => ({ name, values: rows.map((r) => pick(by, r.by && r.by[k])) })) }) : null;
}

const pair = (...charts) => (charts.some(Boolean) ? h('div', { class: 'simple-graphs__pair' }, charts.filter(Boolean)) : null);

function media(d, by, days) {
  const span = spanOf(days, 'days'), byType = (d.heat && d.heat.by_type) || {};
  const kinds = kindsOf(d.daily, d.daily?.rows);
  const grid = (axis, categories) => barChart({ categories, format: fmt(by), series: kinds.map(([k, name]) => ({ name, values: ((byType[k] && byType[k][axis]) || []).map((x) => pick(by, x)) })) });
  const ranked = (iconName, answer, title, help) => (answer && answer.rows.length ? graph(iconName, title, span, help, columns(answer, kindsOf(answer, answer.rows), by)) : null);
  return [
    graph('activity', graphWords('media_daily', by, d.daily?.bucket), span, graphWords('media_daily_help', by, d.daily?.bucket), overTime(d.daily, kinds, by, dayLabel), true),
    pair(graph('calendar', graphWords('weekday', by), span, t('simple.graphs.weekday_help'), grid('weekday', WEEKDAYS)),
      graph('clock', graphWords('hour', by), span, t('simple.graphs.hour_help'), grid('hour', HOURS))),
    pair(ranked('monitor', d.clients, graphWords('top_platforms', by), t('simple.graphs.top_platforms_help')),
      ranked('user', d.users, graphWords('top_users', by), t('simple.graphs.top_users_help'))),
  ].filter(Boolean);
}

function streams(d, by, days) {
  const span = spanOf(days, 'days');
  const methods = (answer) => partsOf(answer, METHODS);
  const ranked = (iconName, answer, title, help) => (answer && answer.rows.length ? graph(iconName, title, span, help, columns(answer, methods(answer), by)) : null);
  const c = d.peaks && d.peaks.concurrency;
  const peaks = c && c.series && c.series.length ? lineChart({ labels: c.series.map((b) => dayLabel(b.date)), format: compact, height: 260, series: [
    ...Object.entries(METHODS).map(([k, name]) => ({ name, values: c.series.map((b) => (b.by && b.by[k]) || 0) })),
    { name: t('simple.graphs.most_at_once'), values: c.series.map((b) => b.peak) }] }) : null;
  return [
    graph('video', graphWords('streams_daily', by, d.daily?.bucket), span, graphWords('streams_daily_help', by, d.daily?.bucket),
      overTime(d.daily, methods(d.daily), by, dayLabel), true),
    peaks ? graph('video', t(c.bucket === 'week' ? 'simple.graphs.concurrent.week' : 'simple.graphs.concurrent.day'), span, t(c.bucket === 'week' ? 'simple.graphs.concurrent_help.week' : 'simple.graphs.concurrent_help.day'), peaks, true) : null,
    pair(ranked('maximize', d.source, graphWords('source_resolution', by), t('simple.graphs.source_resolution_help')),
      ranked('maximize', d.stream, graphWords('stream_resolution', by), t('simple.graphs.stream_resolution_help'))),
    pair(ranked('monitor', d.clients, graphWords('platform_method', by), t('simple.graphs.platform_method_help')),
      ranked('user', d.users, graphWords('user_method', by), t('simple.graphs.user_method_help'))),
  ].filter(Boolean);
}

function totals(d, by, months) {
  const rows = d.months?.rows || [];
  const kinds = kindsOf(d.months, rows);
  return [graph('calendar', graphWords('monthly', by), spanOf(months, 'months'), t('simple.graphs.monthly_help'),
    barChart({ categories: rows.map((r) => monthLabel(r.date)), format: fmt(by), height: 260, series: kinds.map(([k, name]) => ({ name, values: rows.map((r) => pick(by, r.by && r.by[k])) })) }), true)];
}
