// Simple mode's home, top to bottom: what is playing (a card per stream, on the clock both layouts share), the watch
// statistics as ranked cards over the last N days by play count or duration, what each library holds, and what arrived
// lately. Every card reads an endpoint another page reads, through the same scoping, and hides itself when it is empty.

import { h, num, clock, relTime, store, episodeCode, mount } from '../../dom.js';
import { api, soft, imgItem } from '../../api.js';
import { can } from '../../state.js';
import { replaceQuery } from '../../router.js';
import { dataView, poster, avatar, segmented, sk } from '../../components.js';
import { nowPlayingView, followNowPlaying } from '../../nowplaying.js';
import { streamCard, streamCards } from '../../../finui/components/stream-card/stream-card.js';
import { statCard, statCards } from '../../../finui/components/stat-card/stat-card.js';
import { tile } from '../../../finui/components/tile-stat/tile-stat.js';
import { shelf } from '../../../finui/components/shelf/shelf.js';
import { mediaCard } from '../../../finui/components/media-card/media-card.js';
import { hhmm, rate, platformTile, libraryIcon } from '../platform.js';

const ROWS = 5;
const DAYS_KEY = 'finstats.simpleDays', BY_KEY = 'finstats.simpleBy';
const MEASURES = [{ value: 'plays', label: 'Play Count' }, { value: 'duration', label: 'Play Duration' }];
const KINDS = [{ value: '', label: 'All' }, { value: 'movies', label: 'Movies' }, { value: 'shows', label: 'TV Shows' }, { value: 'music', label: 'Music' }];

const validDays = (v) => Number.isInteger(v) && v >= 1 && v <= 3650;
/** The last N days and the measure: the address first, then this browser's last choice, then 30 days by play count. */
export function scopeOf(query) {
  const days = [Number(query.get('days')), Number(store.get(DAYS_KEY))].find(validDays) || 30;
  const by = [query.get('by'), store.get(BY_KEY)].find((v) => MEASURES.some((m) => m.value === v)) || 'plays';
  return { days, by };
}

/** Every watch statistics card, in one go. Shared with the prefetcher, so both ask for exactly the same addresses. */
export function loadStats({ days, by }, signal) {
  const o = { signal };
  const everyone = can('see_everyone');
  const sort = by === 'plays' ? 'plays' : undefined;
  const top = (kind, s = sort) => api.get('/stats/top', { days, kind, limit: ROWS, sort: s }, o);
  // Without see_everyone a person is every title's only viewer, so Most popular would say nothing; and the people are theirs alone.
  return Promise.all([
    top('series'), everyone ? top('series', 'users') : null, top('movies'), everyone ? top('movies', 'users') : null, top('music'), everyone ? top('music', 'users') : null,
    api.get('/activity', { days, page: 1, per_page: ROWS, watched: 1 }, o),
    top('libraries'), everyone ? top('users') : null, top('clients'),
    soft(api.get('/stats/insights', { days, parts: 'concurrency' }, o)),
  ]).then(([series, popSeries, movies, popMovies, music, popMusic, watched, libraries, users, clients, insights]) =>
    ({ series, popSeries, movies, popMovies, music, popMusic, watched, libraries, users, clients, insights }));
}
const loadLibraries = (signal) => api.get('/libraries', { days: 0 }, { signal });
const loadRecent = (kind, signal) => soft(api.get('/library/recent', { limit: 30, kind: kind || undefined }, { signal }));

export const prefetchHome = ({ query, signal }) => [() => loadStats(scopeOf(query), signal), () => loadLibraries(signal), () => loadRecent('', signal)];

/** A section's header row: its name, a line beside it, its controls on the right. */
const head = (title, note, tools) => h('div', { class: 'simple-head' }, h('h2', { class: 'simple-head__title' }, title, note), tools ? h('div', { class: 'simple-head__tools' }, tools) : null);

export default function home(ctx) {
  ctx.title('Home');
  let { days, by } = scopeOf(ctx.query);

  // ---- activity
  const note = h('span', { class: 'simple-head__note' });
  const live = nowPlayingView({ list: streams, cls: 'simple-activity',
    parts: (card) => ({ clock: card.querySelector('.fui-stream-card__time'), fill: card.querySelector('.fui-stream-card__fill'), bar: card.querySelector('.fui-stream-card__bar') }) });
  ctx.onCleanup(() => live.destroy());
  const activityBody = h('div', null, h('p', { class: 'simple-empty' }, 'Looking for streams…'));
  let first = true;
  const paint = (d) => {
    const sessions = d.sessions || [];
    if (first) { mount(activityBody, live.el); first = false; }
    live.update(sessions);
    note.textContent = summary(sessions);
  };
  const failed = (e) => { if (first) mount(activityBody, h('p', { class: 'simple-empty' }, 'Live sessions are unavailable: ' + e.message)); };

  // ---- watch statistics
  const grid = h('div');
  const stats = dataView({
    container: grid, signal: ctx.signal,
    skeleton: () => statCards(Array.from({ length: 6 }, () => sk.cardRows(ROWS))),
    fetch: () => loadStats({ days, by }, ctx.signal),
    render: (d) => statCards(watchCards(d, by)),
  });
  const sync = () => replaceQuery({ days: days === 30 ? null : days, by: by === 'plays' ? null : by });
  const daysField = h('input', { class: 'simple-days__field', id: 'simple-days', type: 'number', min: '1', max: '3650', value: String(days), 'aria-label': 'Days' });
  const setDays = () => {
    const v = Number(daysField.value);
    if (!validDays(v) || v === days) { daysField.value = String(days); return; }
    days = v; store.set(DAYS_KEY, String(v)); sync(); stats.load();
  };
  daysField.addEventListener('change', setDays);
  daysField.addEventListener('keydown', (e) => { if (e.key === 'Enter') setDays(); });
  const tools = [
    segmented({ label: 'Rank by', value: by, options: MEASURES, onChange: (v) => { by = v; store.set(BY_KEY, v); sync(); stats.load(); } }),
    h('label', { class: 'simple-days', for: 'simple-days' }, h('span', null, 'Last'), daysField, h('span', null, 'days')),
  ];

  // ---- library statistics: about the library, so no filter applies
  const libBody = h('div');
  const libSection = h('section', { class: 'simple-section', id: 'simple-libraries', 'aria-label': 'Library statistics' }, head('Library statistics'), libBody);
  dataView({
    container: libBody, signal: ctx.signal, skeleton: () => null,
    fetch: () => loadLibraries(ctx.signal),
    render: (d) => {
      const libs = ((d && d.libraries) || []).filter((l) => !l.removed);
      libSection.hidden = !libs.length;
      return libs.length ? statCards(libs.map(libraryCard)) : null;
    },
  }).load();

  // ---- recently added
  let kind = '';
  const recentBody = h('div');
  const recent = dataView({
    container: recentBody, signal: ctx.signal, skeleton: () => null,
    fetch: () => loadRecent(kind, ctx.signal),
    render: (d) => {
      const items = (d && d.items) || [];
      return items.length ? shelf({ label: 'Recently added', items: items.map(arrival) }) : h('p', { class: 'simple-empty' }, 'Nothing has been added lately.');
    },
  });

  ctx.root.append(
    h('section', { class: 'simple-section', id: 'simple-activity', 'aria-label': 'Activity' }, head('Activity', note), activityBody),
    h('section', { class: 'simple-section', id: 'simple-stats', 'aria-label': 'Watch statistics' }, head('Watch statistics', null, tools), grid),
    libSection,
    h('section', { class: 'simple-section simple-recent', id: 'simple-recent', 'aria-label': 'Recently added' },
      head('Recently added', null, segmented({ label: 'Recently added', value: kind, options: KINDS, onChange: (v) => { kind = v; recent.load(); } })), recentBody));
  followNowPlaying(ctx, { paint, failed });
  stats.load();
  recent.load();
}

// ---------------------------------------------------------------- activity

const METHOD = { DirectPlay: 'Direct play', DirectStream: 'Direct stream', Transcode: 'Transcode' };
const up = (x) => (x ? String(x).toUpperCase() : null);

/** "Streams: 3 streams (1 direct play, 2 transcodes) | Bandwidth: 37.3 Mbps (LAN: 36.7 Mbps, WAN: 680 kbps)" */
function summary(sessions) {
  if (!sessions.length) return '';
  const count = (m) => sessions.filter((s) => s.play_method === m).length;
  const kinds = [['direct play', 'direct plays', count('DirectPlay')], ['direct stream', 'direct streams', count('DirectStream')], ['transcode', 'transcodes', count('Transcode')]]
    .filter(([, , n]) => n).map(([one, many, n]) => `${n} ${n === 1 ? one : many}`);
  const sum = (list) => list.reduce((n, s) => n + (s.bitrate || 0), 0);
  const known = sessions.some((s) => s.is_local != null);
  const where = known ? ` (LAN: ${rate(sum(sessions.filter((s) => s.is_local)))}, WAN: ${rate(sum(sessions.filter((s) => s.is_local === false)))})` : '';
  return `Streams: ${sessions.length} ${sessions.length === 1 ? 'stream' : 'streams'}${kinds.length ? ` (${kinds.join(', ')})` : ''} | Bandwidth: ${rate(sum(sessions))}${where}`;
}

function streams(sessions) {
  if (!sessions.length) return h('p', { class: 'simple-empty' }, 'Nothing is playing right now.');
  return streamCards(sessions.map(stream));
}

function stream(s) {
  const tc = s.transcode;
  const changed = (direct, from, to) => (tc && !direct ? `Transcode (${from || '?'} → ${up(to) || '?'})` : `Direct play${from ? ` (${from})` : ''}`);
  const left = s.runtime_s ? Math.max(0, s.runtime_s - (s.position_s || 0)) : null;
  const eta = left != null && !s.is_paused ? new Date(Date.now() + left * 1000).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }) : null;
  const episode = s.item_type === 'Episode';
  const where = s.is_local == null ? 'Unknown' : s.is_local ? 'LAN' : 'WAN';
  return streamCard({
    key: s.key,
    poster: poster(s.image_item_id, s.series_name || s.item_name, { w: 320 }),
    art: imgItem(s.series_id || s.item_id, 960, 'backdrop'),
    tile: platformTile(s.client, 46),
    facts: [
      [['Product', s.client || 'Unknown'], ['Player', s.device_name || 'Unknown'], ['Quality', `${tc ? 'Converted' : 'Original'} (${rate(s.bitrate)})`]],
      [['Stream', METHOD[s.play_method] || s.play_method || 'Unknown'], ['Container', tc ? `Transcode (${up(s.container) || '?'} → ${up(tc.container) || '?'})` : `Direct play (${up(s.container) || '?'})`],
        ['Video', s.video ? changed(tc && tc.is_video_direct, s.video, tc && tc.video_codec) : null], ['Audio', changed(tc && tc.is_audio_direct, s.audio, tc && tc.audio_codec)],
        ['Subtitle', s.subtitle || 'None']].filter(([, v]) => v != null),
      [['Location', s.remote_ip ? `${where}: ${s.remote_ip}` : where], ['Bandwidth', rate(s.bitrate)]],
    ],
    eta: eta ? `ETA: ${eta}` : null,
    time: s.runtime_s ? `${clock(s.position_s || 0)} / ${clock(s.runtime_s)}` : clock(s.position_s || 0),
    progress: s.runtime_s ? (s.position_s || 0) / s.runtime_s : 0,
    buffer: tc && tc.progress != null ? tc.progress : null,
    paused: s.is_paused,
    title: { text: episode && s.series_name ? `${s.series_name} - ${s.item_name}` : s.item_name, href: `/items/${s.item_id}` },
    sub: episode ? { icon: 'tv', text: episodeCode(s.season_number, s.episode_number) || 'Episode' } : s.item_type === 'Audio' ? { icon: 'music', text: 'Track' } : { icon: 'film', text: 'Movie' },
    who: h('span', { class: 'simple-who' }, h('span', { class: 'simple-who__name' }, s.user_name), avatar(s.user_id, s.user_name, { size: 34 })),
  });
}

// ---------------------------------------------------------------- watch statistics

const art = (id) => (id ? imgItem(id, 960, 'backdrop') : null);
const rowsOf = (d) => (d && d.rows) || [];
const measure = (by, r) => (by === 'users' ? num(r.users) : by === 'plays' ? num(r.plays) : hhmm(r.watch_s));
const columnOf = (by) => (by === 'users' ? 'Users' : by === 'plays' ? 'Plays' : 'Duration');

/** A ranked card, or nothing when there is nothing to rank. */
function ranked(id, title, rows, { by, href, picture, backdrop = true }) {
  if (!rows.length) return null;
  const el = statCard({ title, column: columnOf(by), art: backdrop ? art(rows[0].image_item_id) : null, picture: picture(rows[0]),
    rows: rows.slice(0, ROWS).map((r) => ({ href: href(r), name: r.name, value: measure(by, r) })) });
  el.dataset.card = id;
  return el;
}

function watchCards(d, by) {
  const titles = (id, title, rows, measureBy) => ranked(id, title, rowsOf(rows), { by: measureBy, href: (r) => (r.id ? `/items/${r.id}` : null),
    picture: (r) => poster(r.image_item_id, r.name, { w: 200 }) });
  const watched = (d.watched && d.watched.rows) || [];
  const recent = watched.length ? statCard({ title: 'Recently watched', art: art(watched[0].image_item_id), picture: poster(watched[0].image_item_id, watched[0].series_name || watched[0].item_name, { w: 200 }),
    rows: watched.slice(0, ROWS).map((p) => ({ href: p.item_exists !== false ? `/items/${p.item_id}` : null, name: p.series_name ? `${p.series_name} ${episodeCode(p.season_number, p.episode_number) || ''}`.trim() : p.item_name,
      sub: can('see_everyone') ? p.user_name : null, value: relTime(p.ended_at) })) }) : null;
  if (recent) recent.dataset.card = 'recently-watched';
  const c = d.insights && d.insights.concurrency;
  const peak = c && c.peak ? statCard({ title: 'Most concurrent streams', ranked: false, picture: tile({ icon: 'layers', tone: 'neutral', size: 64 }),
    rows: [['Concurrent streams', c.peak], ['Transcodes', c.peak_transcodes], ['Direct streams', c.peak_direct_streams], ['Direct plays', c.peak_direct_plays]].map(([name, n]) => ({ name, value: num(n || 0) })) }) : null;
  if (peak) peak.dataset.card = 'concurrent';
  return [
    titles('watched-series', 'Most watched TV shows', d.series, by),
    titles('popular-series', 'Most popular TV shows', d.popSeries, 'users'),
    titles('watched-movies', 'Most watched movies', d.movies, by),
    titles('popular-movies', 'Most popular movies', d.popMovies, 'users'),
    titles('watched-music', 'Most played music', d.music, by),
    titles('popular-music', 'Most popular music', d.popMusic, 'users'),
    recent,
    ranked('libraries', 'Most active libraries', rowsOf(d.libraries), { by, backdrop: false, href: (r) => `/libraries/${r.id}`, picture: (r) => tile({ icon: libraryIcon(r.sub), tone: 'neutral', size: 64 }) }),
    d.users ? ranked('users', 'Most active users', rowsOf(d.users), { by, backdrop: false, href: (r) => `/users/${r.id}`, picture: (r) => avatar(r.id, r.name, { size: 64 }) }) : null,
    ranked('platforms', 'Most active platforms', rowsOf(d.clients), { by, backdrop: false, href: () => null, picture: (r) => platformTile(r.name, 64) }),
    peak,
  ].filter(Boolean);
}

// ---------------------------------------------------------------- libraries and arrivals

const COUNTS = [['movie_count', 'Movies'], ['series_count', 'Shows'], ['season_count', 'Seasons'], ['episode_count', 'Episodes'], ['artist_count', 'Artists'], ['album_count', 'Albums'], ['track_count', 'Tracks']];

function libraryCard(l) {
  const rows = COUNTS.filter(([k]) => l[k] > 0).map(([k, name]) => ({ name, value: num(l[k]) }));
  if (!rows.length) rows.push({ name: 'Items', value: num(l.item_count || 0) });
  rows.push({ name: 'Last played', value: l.last_played_at ? relTime(l.last_played_at) : 'Never' });
  const el = statCard({ title: l.name, ranked: false, picture: h('a', { href: `/libraries/${l.id}`, 'aria-label': l.name }, tile({ icon: libraryIcon(l.collection_type), tone: 'neutral', size: 64 })), rows });
  el.dataset.library = l.id;
  return el;
}

function arrival(it) {
  const what = it.kind === 'episodes' ? (it.episodes === 1 ? (it.episode_number != null ? `Episode ${it.episode_number}` : '1 episode') : `${it.episodes} episodes`) : it.sub;
  return mediaCard({ href: `/items/${it.id}`, poster: poster(it.image_item_id || it.id, it.name, { w: 240, cls: 'fui-poster--grid' }), name: it.name, sub: [what, relTime(it.added_at)].filter(Boolean).join(' · ') });
}
