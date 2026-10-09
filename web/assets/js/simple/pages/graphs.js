// Simple mode's Graphs, the set a classic stats server draws, in three tabs. Media Type: plays day by day, by day of the
// week and by hour, and the top ten apps and people, each split into TV, movies and music. Stream Type: the same days and
// the most streams at once each day by how they played, then resolutions, apps and people by how they played. Play
// Totals: month by month. By play count or duration, over the last N days (N months for the totals), for everybody or
// the people chosen (with see_everyone).

import { h, icon, compact, store, mount } from '../../dom.js';
import { api } from '../../api.js';
import { can } from '../../state.js';
import { replaceQuery } from '../../router.js';
import { dataView, segmented, userCombobox, sk } from '../../components.js';
import { lineChart } from '../../../finui/components/line-chart/line-chart.js';
import { barChart } from '../../../finui/components/bar-chart/bar-chart.js';

const TABS = [{ key: 'media', label: 'Media Type' }, { key: 'streams', label: 'Stream Type' }, { key: 'totals', label: 'Play Totals' }];
const MEASURES = [{ value: 'plays', label: 'Play Count' }, { value: 'duration', label: 'Play Duration' }];
const DAYS_KEY = 'finstats.simpleGraphDays', MONTHS_KEY = 'finstats.simpleGraphMonths', BY_KEY = 'finstats.simpleBy';
// The parts a breakdown is split into, as the answer names them, in the order a series takes its colour.
const KINDS = { Episode: 'TV', Movie: 'Movies', Audio: 'Music', Other: 'Other' };
const METHODS = { DirectPlay: 'Direct Play', DirectStream: 'Direct Stream', Transcode: 'Transcode' };
const WEEKDAYS = ['Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday', 'Sunday'];
const HOURS = Array.from({ length: 24 }, (_, i) => String(i).padStart(2, '0'));

const valid = (v) => Number.isInteger(v) && v >= 1 && v <= 3650;
const hm = (sec) => { const m = Math.round((Number(sec) || 0) / 60); return `${Math.floor(m / 60)} h ${m % 60} m`; };
const dayLabel = (iso) => new Date(`${iso}T12:00:00`).toLocaleDateString([], { month: 'short', day: 'numeric' });
const monthLabel = (ym) => new Date(`${ym}-15T12:00:00`).toLocaleDateString([], { month: 'short', year: 'numeric' });

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
  const tab = TABS.some((t) => t.key === query.get('tab')) ? query.get('tab') : 'media';
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
  ctx.title('Graphs');
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
  const field = h('input', { class: 'simple-days__field', id: 'simple-graphs-days', type: 'number', min: '1', max: inMonths ? '120' : '3650', value: String(inMonths ? months : days), 'aria-label': inMonths ? 'Months' : 'Days' });
  const setSpan = () => {
    const v = Number(field.value), now = inMonths ? months : days;
    if (!valid(v) || (inMonths && v > 120) || v === now) { field.value = String(now); return; }
    if (inMonths) { months = v; store.set(MONTHS_KEY, String(v)); } else { days = v; store.set(DAYS_KEY, String(v)); }
    sync(); view.load();
  };
  field.addEventListener('change', setSpan);
  field.addEventListener('keydown', (e) => { if (e.key === 'Enter') setSpan(); });
  const keep = (extra) => { const q = new URLSearchParams(extra); if (users) q.set('user_id', users); const s = q.toString(); return s ? `?${s}` : ''; };
  const tabs = h('nav', { class: 'simple-tabs simple-tabs--boxed', 'aria-label': 'Graphs' },
    TABS.map((t) => h('a', { href: `/graphs${keep(t.key === 'media' ? {} : { tab: t.key })}`, 'aria-current': t.key === tab ? 'page' : null }, t.label)));
  ctx.root.append(h('div', { id: 'simple-graphs' },
    h('div', { class: 'simple-panel-head' }, h('h1', { class: 'simple-page-title' }, icon('chart', 22), 'Graphs'),
      h('div', { class: 'simple-head__tools' },
        can('see_everyone') ? h('div', { class: 'simple-history__who' }, userCombobox({ value: users, multiple: true, signal: ctx.signal, onChange: (v) => { users = v; sync(); view.load(); } })) : null,
        h('span', { class: 'simple-graphs__measure' }, segmented({ label: 'Measure', value: by, options: MEASURES, onChange: (v) => {
          by = v; store.set(BY_KEY, v); sync();
          if (last) mount(body, draw(last));
          view.load();
        } })),
        h('label', { class: 'simple-days', for: 'simple-graphs-days' }, h('span', null, 'Last'), field, h('span', null, inMonths ? 'months' : 'days')))),
    h('div', { class: 'simple-panel' }, tabs, body)));
  view.load();
}

// ---- drawing

const word = (by) => (by === 'plays' ? 'count' : 'duration');
const Word = (by) => (by === 'plays' ? 'Play count' : 'Play duration');
const fmt = (by) => (by === 'plays' ? compact : hm);
const pick = (by, pair) => (pair ? (by === 'plays' ? pair[0] : pair[1]) : 0);
const spanOf = (n, unit) => (n === 1 && unit === 'days' ? 'Today' : `Last ${n} ${unit}`);

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
// TV leads, as on a classic stats page: the answer's order is the database's.
const kindsOf = (answer, rows) => partsOf({ ...answer, keys: Object.keys(KINDS) }, KINDS, rows);

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
  const span = spanOf(days, 'days'), t = (d.heat && d.heat.by_type) || {};
  const kinds = kindsOf(d.daily, d.daily?.rows);
  const grid = (axis, categories) => barChart({ categories, format: fmt(by), series: kinds.map(([k, name]) => ({ name, values: ((t[k] && t[k][axis]) || []).map((x) => pick(by, x)) })) });
  const ranked = (iconName, answer, title, help) => (answer && answer.rows.length ? graph(iconName, title, span, help, columns(answer, kindsOf(answer, answer.rows), by)) : null);
  return [
    graph('activity', `Daily play ${word(by)} by media type`, span, `The total play ${word(by)} of TV, movies and music played each ${d.daily?.bucket === 'week' ? 'week' : 'day'}.`, overTime(d.daily, kinds, by, dayLabel), true),
    pair(graph('calendar', `${Word(by)} by day of week`, span, 'TV, movies and music together, by the day of the week they were played.', grid('weekday', WEEKDAYS)),
      graph('clock', `${Word(by)} by hour of day`, span, 'TV, movies and music together, by the hour of the day they were played.', grid('hour', HOURS))),
    pair(ranked('monitor', d.clients, `${Word(by)} by top 10 platforms`, 'TV, movies and music together, on the ten apps that played the most.'),
      ranked('user', d.users, `${Word(by)} by top 10 users`, 'TV, movies and music together, by the ten people who played the most.')),
  ].filter(Boolean);
}

function streams(d, by, days) {
  const span = spanOf(days, 'days');
  const methods = (answer) => partsOf(answer, METHODS);
  const ranked = (iconName, answer, title, help) => (answer && answer.rows.length ? graph(iconName, title, span, help, columns(answer, methods(answer), by)) : null);
  const c = d.peaks && d.peaks.concurrency;
  const peaks = c && c.series && c.series.length ? lineChart({ labels: c.series.map((b) => dayLabel(b.date)), format: compact, height: 260, series: [
    ...Object.entries(METHODS).map(([k, name]) => ({ name, values: c.series.map((b) => (b.by && b.by[k]) || 0) })),
    { name: 'Most at once', values: c.series.map((b) => b.peak) }] }) : null;
  return [
    graph('video', `Daily play ${word(by)} by stream type`, span, `The total play ${word(by)} of TV, movies and music each ${d.daily?.bucket === 'week' ? 'week' : 'day'}, by whether it played as it is or was converted.`,
      overTime(d.daily, methods(d.daily), by, dayLabel), true),
    peaks ? graph('video', 'Daily concurrent stream count by stream type', span, 'The most streams at once each day, of each kind and in all.', peaks, true) : null,
    pair(ranked('maximize', d.source, `${Word(by)} by source resolution`, 'TV and movies by the resolution of the file, before any conversion.'),
      ranked('maximize', d.stream, `${Word(by)} by stream resolution`, 'TV and movies by the resolution that was sent, after any conversion.')),
    pair(ranked('monitor', d.clients, `${Word(by)} by platform and stream type`, 'TV, movies and music by app and by how they played.'),
      ranked('user', d.users, `${Word(by)} by user and stream type`, 'TV, movies and music by person and by how they played.')),
  ].filter(Boolean);
}

function totals(d, by, months) {
  const rows = d.months?.rows || [];
  const kinds = kindsOf(d.months, rows);
  return [graph('calendar', `Total play ${word(by)} by month`, spanOf(months, 'months'), 'TV, movies and music together, month by month.',
    barChart({ categories: rows.map((r) => monthLabel(r.date)), format: fmt(by), height: 260, series: kinds.map(([k, name]) => ({ name, values: rows.map((r) => pick(by, r.by && r.by[k])) })) }), true)];
}
