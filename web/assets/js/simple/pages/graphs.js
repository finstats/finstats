// Simple mode's Graphs: three tabs of charts, as a classic stats server has them. Plays by period: day by day, by day of
// the week and by hour of the day, stacked by TV, movies and music. Stream info: how things play and on what. Play totals:
// week by week over the last year, and the busiest apps and people. By play count or duration, over the last N days,
// for everybody or one person (with see_everyone).

import { h, icon, compact, store, mount } from '../../dom.js';
import { api, soft } from '../../api.js';
import { can } from '../../state.js';
import { replaceQuery } from '../../router.js';
import { dataView, segmented, userCombobox, sk } from '../../components.js';
import { lineChart } from '../../../finui/components/line-chart/line-chart.js';
import { barChart } from '../../../finui/components/bar-chart/bar-chart.js';

const TABS = [{ key: 'period', label: 'Plays by period' }, { key: 'streams', label: 'Stream Info' }, { key: 'totals', label: 'Play Totals' }];
const MEASURES = [{ value: 'plays', label: 'Play Count' }, { value: 'duration', label: 'Play Duration' }];
const DAYS_KEY = 'finstats.simpleGraphDays', BY_KEY = 'finstats.simpleBy';
// The kinds of media, in the order a series takes its colour: TV, movies, music, then the rest.
const KINDS = [['Episode', 'TV'], ['Movie', 'Movies'], ['Audio', 'Music'], ['Other', 'Other']];
const WEEKDAYS = ['Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday', 'Sunday'];
const METHODS = [['direct_play', 'Direct play'], ['direct_stream', 'Direct stream'], ['transcode', 'Transcode']];

const validDays = (v) => Number.isInteger(v) && v >= 1 && v <= 3650;
const hm = (sec) => { const m = Math.round((Number(sec) || 0) / 60); return `${Math.floor(m / 60)} h ${m % 60} m`; };
const dayLabel = (iso) => new Date(`${iso}T12:00:00`).toLocaleDateString([], { month: 'short', day: 'numeric' });

export function scopeOf(query) {
  const days = [Number(query.get('days')), Number(store.get(DAYS_KEY))].find(validDays) || 30;
  const by = [query.get('by'), store.get(BY_KEY)].find((v) => MEASURES.some((m) => m.value === v)) || 'plays';
  const tab = TABS.some((t) => t.key === query.get('tab')) ? query.get('tab') : 'period';
  return { days, by, tab, user: can('see_everyone') ? query.get('user_id') || '' : '' };
}

/** One tab's data. Shared with the prefetcher, so both ask for exactly the same addresses. */
export function loadTab({ tab, days, user }, signal) {
  const f = { days, user_id: user || undefined }, o = { signal };
  if (tab === 'streams') return Promise.all([api.get('/stats/playback', f, o), soft(api.get('/stats/insights', { ...f, parts: 'client_methods' }, o))]).then(([playback, insights]) => ({ playback, insights }));
  if (tab === 'totals') {
    return Promise.all([api.get('/stats/overview', { user_id: user || undefined, days: 365 }, o), api.get('/stats/top', { ...f, kind: 'clients', limit: 10, sort: 'plays' }, o),
      can('see_everyone') && !user ? api.get('/stats/top', { ...f, kind: 'users', limit: 10, sort: 'plays' }, o) : null]).then(([year, clients, users]) => ({ year, clients, users }));
  }
  return Promise.all([api.get('/stats/overview', f, o), api.get('/stats/heatmap', f, o)]).then(([overview, heat]) => ({ overview, heat }));
}
export const prefetchGraphs = ({ query, signal }) => [() => loadTab(scopeOf(query), signal)];

export default function graphs(ctx) {
  ctx.title('Graphs');
  let { days, by, tab, user } = scopeOf(ctx.query);
  const body = h('div', { class: 'simple-graphs__body' });
  // Count and duration are the same answers drawn twice: switching between them draws what is here again, since the view
  // draws an answer only when it differs from the last.
  let last = null;
  const draw = (d) => (tab === 'streams' ? streams(d, by) : tab === 'totals' ? totals(d, by) : period(d, by, days));
  const view = dataView({
    container: body, signal: ctx.signal, skeleton: () => [sk.cardBlock(260), sk.cardBlock(220)],
    fetch: () => loadTab({ tab, days, user }, ctx.signal),
    render: (d) => { last = d; return draw(d); },
  });
  const sync = () => replaceQuery({ tab: tab === 'period' ? null : tab, days: days === 30 ? null : days, by: by === 'plays' ? null : by, user_id: user || null });
  const daysField = h('input', { class: 'simple-days__field', id: 'simple-graphs-days', type: 'number', min: '1', max: '3650', value: String(days), 'aria-label': 'Days' });
  const setDays = () => { const v = Number(daysField.value); if (!validDays(v) || v === days) { daysField.value = String(days); return; } days = v; store.set(DAYS_KEY, String(v)); sync(); view.load(); };
  daysField.addEventListener('change', setDays);
  daysField.addEventListener('keydown', (e) => { if (e.key === 'Enter') setDays(); });
  const tabs = h('nav', { class: 'simple-tabs simple-tabs--boxed', 'aria-label': 'Graphs' }, TABS.map((t) => h('a', { href: `/graphs${t.key === 'period' ? '' : `?tab=${t.key}`}`, 'aria-current': t.key === tab ? 'page' : null }, t.label)));
  ctx.root.append(h('div', { id: 'simple-graphs' },
    h('div', { class: 'simple-panel-head' }, h('h1', { class: 'simple-page-title' }, icon('chart', 22), 'Graphs'),
      h('div', { class: 'simple-head__tools' },
        can('see_everyone') ? h('div', { class: 'simple-history__who' }, userCombobox({ value: user, signal: ctx.signal, onChange: (v) => { user = v; sync(); view.load(); } })) : null,
        h('span', { class: 'simple-graphs__measure' }, segmented({ label: 'Measure', value: by, options: MEASURES, onChange: (v) => { by = v; store.set(BY_KEY, v); sync(); if (last) mount(body, draw(last)); else view.load(); } })),
        h('label', { class: 'simple-days', for: 'simple-graphs-days' }, h('span', null, 'Last'), daysField, h('span', null, 'days')))),
    h('div', { class: 'simple-panel' }, tabs, body)));
  view.load();
}

const word = (by) => (by === 'plays' ? 'count' : 'duration');
const pick = (by, pair) => (by === 'plays' ? pair[0] : pair[1]);
const fmt = (by) => (by === 'plays' ? compact : hm);
const note = (text) => h('p', { class: 'simple-graphs__note' }, text);

function period(d, by, days) {
  const daily = (d.overview && d.overview.daily) || [];
  const t = (d.heat && d.heat.by_type) || {};
  const present = KINDS.filter(([k]) => daily.some((x) => x.by_type && x.by_type[k] && x.by_type[k][0]));
  const kinds = present.length ? present : KINDS.slice(0, 3);
  const span = days === 1 ? 'Today' : `Last ${days} days`;
  return [
    note(`The total play ${word(by)} of TV, movies and music played per ${d.overview && d.overview.bucket === 'week' ? 'week' : 'day'}.`),
    lineChart({ title: `Daily play ${word(by)}`, sub: span, labels: daily.map((x) => dayLabel(x.date)), format: fmt(by), height: 260,
      series: kinds.map(([k, name]) => ({ name, values: daily.map((x) => pick(by, (x.by_type && x.by_type[k]) || [0, 0])) })) }),
    h('div', { class: 'simple-graphs__pair' },
      barChart({ title: `Play ${word(by)} by day of week`, sub: span, categories: WEEKDAYS, format: fmt(by),
        series: kinds.map(([k, name]) => ({ name, values: ((t[k] && t[k].weekday) || []).map((x) => pick(by, x)) })) }),
      barChart({ title: `Play ${word(by)} by hour of day`, sub: span, categories: Array.from({ length: 24 }, (_, i) => String(i).padStart(2, '0')), format: fmt(by),
        series: kinds.map(([k, name]) => ({ name, values: ((t[k] && t[k].hour) || []).map((x) => pick(by, x)) })) })),
  ];
}

function streams(d, by) {
  const p = d.playback || {};
  const bucket = (title, rows) => (rows && rows.length ? barChart({ title, categories: rows.map((r) => r.name), format: fmt(by), series: [{ name: by === 'plays' ? 'Plays' : 'Duration', values: rows.map((r) => (by === 'plays' ? r.plays : r.watch_s)) }] }) : null);
  const cm = (d.insights && d.insights.client_methods) || [];
  return [
    note('How things were played: as they are, or converted on the way, and on what.'),
    h('div', { class: 'simple-graphs__pair' }, bucket(`Play ${word(by)} by stream type`, (p.methods || []).map((m) => ({ ...m, name: { DirectPlay: 'Direct play', DirectStream: 'Direct stream', Transcode: 'Transcode' }[m.name] || m.name }))),
      bucket(`Play ${word(by)} by source resolution`, p.resolutions)),
    cm.length ? barChart({ title: 'Play count by platform stream type', categories: cm.map((c) => c.client), format: compact,
      series: METHODS.map(([k, name]) => ({ name, values: cm.map((c) => c[k] || 0) })) }) : null,
    h('div', { class: 'simple-graphs__pair' }, bucket(`Play ${word(by)} by video codec`, p.video_codecs), bucket(`Play ${word(by)} by audio codec`, p.audio_codecs)),
  ].filter(Boolean);
}

function totals(d, by) {
  const year = (d.year && d.year.daily) || [];
  const kinds = KINDS.filter(([k]) => year.some((x) => x.by_type && x.by_type[k] && x.by_type[k][0]));
  const rows = (x) => (x && x.rows) || [];
  const top = (title, list) => (list.length ? barChart({ title, categories: list.map((r) => r.name), format: fmt(by), series: [{ name: by === 'plays' ? 'Plays' : 'Duration', values: list.map((r) => (by === 'plays' ? r.plays : r.watch_s)) }] }) : null);
  return [
    note('Totals over the last year week by week, and the apps and people that played the most.'),
    barChart({ title: `Play ${word(by)} by week`, sub: 'Last year', categories: year.map((x) => dayLabel(x.date)), format: fmt(by), height: 260,
      series: (kinds.length ? kinds : KINDS.slice(0, 3)).map(([k, name]) => ({ name, values: year.map((x) => pick(by, (x.by_type && x.by_type[k]) || [0, 0])) })) }),
    h('div', { class: 'simple-graphs__pair' }, top(`Play ${word(by)} by top 10 platforms`, rows(d.clients)), d.users ? top(`Play ${word(by)} by top 10 users`, rows(d.users)) : null),
  ].filter(Boolean);
}
