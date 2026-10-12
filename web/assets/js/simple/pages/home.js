// Simple mode's home, top to bottom: what is playing (a card per stream, on the clock both layouts share), the watch
// statistics as ranked cards over the last N days by play count or duration, what each library holds, and what arrived
// lately. Every card reads an endpoint another page reads, through the same scoping, and hides itself when it is empty.

import { h, num, clock, relTime, store, episodeCode, mount, METHOD_LABEL } from '../../dom.js';
import { api, soft, imgItem } from '../../api.js';
import { can } from '../../state.js';
import { t, tParts, lang } from '../../i18n.js';
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
const MEASURES = [{ value: 'plays', label: t('simple.home.measure.plays') }, { value: 'duration', label: t('simple.home.measure.duration') }];
const KINDS = [{ value: '', label: t('simple.history.kind.all') }, { value: 'movies', label: t('simple.history.kind.movies') }, { value: 'shows', label: t('simple.history.kind.shows') }, { value: 'music', label: t('simple.history.kind.music') }];

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
/** A text with an element in it, each run of words a span of its own. */
const spans = (parts) => parts.map((x) => (typeof x === 'string' ? x.trim() : x)).filter((x) => x !== '').map((x) => (typeof x === 'string' ? h('span', null, x) : x));

const head = (title, note, tools) => h('div', { class: 'simple-head' }, h('h2', { class: 'simple-head__title' }, title, note), tools ? h('div', { class: 'simple-head__tools' }, tools) : null);

export default function home(ctx) {
  ctx.title(t('simple.nav.home'));
  let { days, by } = scopeOf(ctx.query);

  // ---- activity
  const note = h('span', { class: 'simple-head__note' });
  const live = nowPlayingView({ list: streams, cls: 'simple-activity',
    parts: (card) => {
      // The ETA follows a skip and the buffer a conversion running ahead: neither redraws the card.
      const eta = card.querySelector('.fui-stream-card__when span:not(.fui-stream-card__time)'), buffer = card.querySelector('.fui-stream-card__buffer');
      return { clock: card.querySelector('.fui-stream-card__time'), fill: card.querySelector('.fui-stream-card__fill'), bar: card.querySelector('.fui-stream-card__bar'),
        also: (s, pos) => {
          const at = etaOf(s, pos);
          if (eta && at) eta.textContent = at;
          if (buffer && s.transcode && s.transcode.progress != null) buffer.style.width = `${Math.max(0, Math.min(1, s.transcode.progress)) * 100}%`;
        } };
    } });
  ctx.onCleanup(() => live.destroy());
  const activityBody = h('div', null, h('p', { class: 'simple-empty' }, t('simple.home.looking')));
  let first = true;
  const paint = (d) => {
    const sessions = d.sessions || [];
    if (first) { mount(activityBody, live.el); first = false; }
    live.update(sessions);
    note.textContent = summary(sessions);
  };
  const failed = (e) => { if (first) mount(activityBody, h('p', { class: 'simple-empty' }, t('simple.home.live_unavailable', { reason: e.message }))); };

  // ---- watch statistics
  const grid = h('div');
  // The measure is drawn from outside the answer, and the view draws an answer only when it differs from the last: when
  // both rankings come back the same, switching would change nothing on screen. So a switch draws what is here again.
  let lastStats = null;
  const stats = dataView({
    container: grid, signal: ctx.signal,
    skeleton: () => statCards(Array.from({ length: 6 }, () => sk.cardRows(ROWS))),
    fetch: () => loadStats({ days, by }, ctx.signal),
    render: (d) => { lastStats = d; return statCards(watchCards(d, by)); },
  });
  const sync = () => replaceQuery({ days: days === 30 ? null : days, by: by === 'plays' ? null : by });
  const daysField = h('input', { class: 'simple-days__field', id: 'simple-days', type: 'number', min: '1', max: '3650', value: String(days), 'aria-label': t('simple.home.days_aria') });
  const setDays = () => {
    const v = Number(daysField.value);
    if (!validDays(v) || v === days) { daysField.value = String(days); return; }
    days = v; store.set(DAYS_KEY, String(v)); sync(); stats.load();
  };
  daysField.addEventListener('change', setDays);
  daysField.addEventListener('keydown', (e) => { if (e.key === 'Enter') setDays(); });
  const tools = [
    segmented({ label: t('simple.home.rank_by'), value: by, options: MEASURES, onChange: (v) => {
      by = v; store.set(BY_KEY, v); sync();
      if (lastStats) mount(grid, statCards(watchCards(lastStats, by)));
      stats.load();
    } }),
    h('label', { class: 'simple-days', for: 'simple-days' }, spans(tParts('simple.home.last_days', { field: daysField }))),
  ];

  // ---- library statistics: about the library, so no filter applies
  const libBody = h('div');
  const libSection = h('section', { class: 'simple-section', id: 'simple-libraries', 'aria-label': t('simple.home.library_statistics') }, head(t('simple.home.library_statistics')), libBody);
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
      return items.length ? shelf({ label: t('simple.home.recently_added'), items: items.map(arrival) }) : h('p', { class: 'simple-empty' }, t('simple.home.nothing_added'));
    },
  });

  ctx.root.append(
    h('section', { class: 'simple-section', id: 'simple-activity', 'aria-label': t('nav.activity') }, head(t('nav.activity'), note), activityBody),
    h('section', { class: 'simple-section', id: 'simple-stats', 'aria-label': t('simple.home.watch_statistics') }, head(t('simple.home.watch_statistics'), null, tools), grid),
    libSection,
    h('section', { class: 'simple-section simple-recent', id: 'simple-recent', 'aria-label': t('simple.home.recently_added') },
      head(t('simple.home.recently_added'), null, segmented({ label: t('simple.home.recently_added'), value: kind, options: KINDS, onChange: (v) => { kind = v; recent.load(); } })), recentBody));
  followNowPlaying(ctx, { paint, failed });
  stats.load();
  recent.load();
}

// ---------------------------------------------------------------- activity

const LAN = t('simple.home.lan'), WAN = t('simple.home.wan');
const up = (x) => (x ? String(x).toUpperCase() : null);

/** "Streams: 3 streams (1 direct play, 2 transcodes) | Bandwidth: 37.3 Mbps (LAN: 36.7 Mbps, WAN: 680 kbps)" */
function summary(sessions) {
  if (!sessions.length) return '';
  const count = (m) => sessions.filter((s) => s.play_method === m).length;
  const kinds = [['simple.home.summary.direct_plays', count('DirectPlay')], ['simple.home.summary.direct_streams', count('DirectStream')], ['simple.home.summary.transcodes', count('Transcode')]]
    .filter(([, n]) => n).map(([key, n]) => t(key, { count: n }));
  const sum = (list) => list.reduce((n, s) => n + (s.bitrate || 0), 0);
  const known = sessions.some((s) => s.is_local != null);
  const bandwidth = known ? t('simple.home.summary.bandwidth_split', { total: rate(sum(sessions)), lan: rate(sum(sessions.filter((s) => s.is_local))), wan: rate(sum(sessions.filter((s) => s.is_local === false))) }) : rate(sum(sessions));
  const total = t('simple.home.summary.streams', { count: sessions.length });
  return kinds.length ? t('simple.home.summary.line_kinds', { streams: total, kinds: kinds.join(', '), bandwidth }) : t('simple.home.summary.line', { streams: total, bandwidth });
}

function streams(sessions) {
  if (!sessions.length) return h('p', { class: 'simple-empty' }, t('simple.home.nothing_playing'));
  return streamCards(sessions.map(stream));
}

/** "ETA: 21:40" from where a playing stream is, or nothing for one paused or without a length. */
function etaOf(s, pos) {
  if (!s.runtime_s || s.is_paused) return null;
  const left = Math.max(0, s.runtime_s - pos);
  return t('simple.home.eta', { time: new Date(Date.now() + left * 1000).toLocaleTimeString(lang(), { hour: '2-digit', minute: '2-digit' }) });
}

/** What a stream is, under its title: an episode's code, and a name for every other kind Jellyfin plays. */
const KIND = { Audio: ['music', t('item_type.Audio')], TvChannel: ['tv', t('item_type.TvChannel')], MusicVideo: ['music', t('simple.home.kind.music_video')], AudioBook: ['music', t('simple.home.kind.audiobook')], Video: ['film', t('simple.home.kind.video')], Movie: ['film', t('simple.home.kind.movie')] };

function stream(s) {
  const tc = s.transcode;
  const changed = (direct, from, to) => (tc && !direct ? t('simple.home.transcode_from_to', { from: from || '?', to: up(to) || '?' }) : from ? t('simple.home.direct_play_of', { from }) : t('play_method.DirectPlay'));
  const eta = etaOf(s, s.position_s || 0);
  const episode = s.item_type === 'Episode';
  const [kindIcon, kindName] = KIND[s.item_type] || ['film', s.item_type || t('simple.home.kind.video')];
  const where = s.is_local == null ? t('common.unknown') : s.is_local ? LAN : WAN;
  return streamCard({
    key: s.key,
    poster: poster(s.image_item_id, s.series_name || s.item_name, { w: 320 }),
    art: imgItem(s.series_id || s.item_id, 960, 'backdrop'),
    tile: platformTile(s.client, 46),
    facts: [
      [[t('simple.home.fact.product'), s.client || t('common.unknown')], [t('simple.home.fact.player'), s.device_name || t('common.unknown')], [t('simple.home.fact.quality'), t(tc ? 'simple.home.quality_converted' : 'simple.home.quality_original', { rate: rate(s.bitrate) })]],
      [[t('simple.home.fact.stream'), METHOD_LABEL[s.play_method] || s.play_method || t('common.unknown')], [t('simple.home.fact.container'), tc ? t('simple.home.transcode_from_to', { from: up(s.container) || '?', to: up(tc.container) || '?' }) : t('simple.home.direct_play_of', { from: up(s.container) || '?' })],
        [t('simple.home.fact.video'), s.video ? changed(tc && tc.is_video_direct, s.video, tc && tc.video_codec) : null], [t('simple.home.fact.audio'), changed(tc && tc.is_audio_direct, s.audio, tc && tc.audio_codec)],
        [t('simple.home.fact.subtitle'), s.subtitle || t('common.none')]].filter(([, v]) => v != null),
      [[t('simple.home.fact.location'), s.remote_ip ? `${where}: ${s.remote_ip}` : where], [t('simple.home.fact.bandwidth'), rate(s.bitrate)]],
    ],
    eta,
    time: s.runtime_s ? `${clock(s.position_s || 0)} / ${clock(s.runtime_s)}` : clock(s.position_s || 0),
    progress: s.runtime_s ? (s.position_s || 0) / s.runtime_s : 0,
    buffer: tc && tc.progress != null ? tc.progress : null,
    paused: s.is_paused,
    title: { text: episode && s.series_name ? `${s.series_name} - ${s.item_name}` : s.item_name, href: `/items/${s.item_id}` },
    sub: episode ? { icon: 'tv', text: episodeCode(s.season_number, s.episode_number) || t('item_type.Episode') } : { icon: kindIcon, text: kindName },
    who: h('span', { class: 'simple-who' }, h('span', { class: 'simple-who__name' }, s.user_name), avatar(s.user_id, s.user_name, { size: 34, hasImage: s.user_has_image })),
  });
}

// ---------------------------------------------------------------- watch statistics

const art = (id) => (id ? imgItem(id, 960, 'backdrop') : null);
const rowsOf = (d) => (d && d.rows) || [];
const measure = (by, r) => (by === 'users' ? num(r.users) : by === 'plays' ? num(r.plays) : hhmm(r.watch_s));
const columnOf = (by) => t(by === 'users' ? 'simple.home.col.users' : by === 'plays' ? 'simple.home.col.plays' : 'simple.home.col.duration');

/** A ranked card, or nothing when there is nothing to rank. */
function ranked(id, title, rows, { by, href, picture, backdrop = true }) {
  if (!rows.length) return null;
  const el = statCard({ title, column: columnOf(by), art: backdrop && rows[0].has_backdrop !== false ? art(rows[0].image_item_id) : null, picture: picture(rows[0]),
    rows: rows.slice(0, ROWS).map((r) => ({ href: href(r), name: r.name, value: measure(by, r) })) });
  el.dataset.card = id;
  return el;
}

function watchCards(d, by) {
  const titles = (id, title, rows, measureBy) => ranked(id, title, rowsOf(rows), { by: measureBy, href: (r) => (r.id && r.item_exists !== false ? `/items/${r.id}` : null),
    picture: (r) => poster(r.image_item_id, r.name, { w: 200 }) });
  const watched = (d.watched && d.watched.rows) || [];
  const recent = watched.length ? statCard({ title: t('simple.home.recently_watched'), art: art(watched[0].image_item_id), picture: poster(watched[0].image_item_id, watched[0].series_name || watched[0].item_name, { w: 200 }),
    rows: watched.slice(0, ROWS).map((p) => ({ href: p.item_exists !== false ? `/items/${p.item_id}` : null, name: p.series_name ? `${p.series_name} ${episodeCode(p.season_number, p.episode_number) || ''}`.trim() : p.item_name,
      sub: can('see_everyone') ? p.user_name : null, value: relTime(p.ended_at) })) }) : null;
  if (recent) recent.dataset.card = 'recently-watched';
  const c = d.insights && d.insights.concurrency;
  const peak = c && c.peak ? statCard({ title: t('simple.home.peak.title'), ranked: false, picture: tile({ icon: 'layers', tone: 'neutral', size: 64 }),
    rows: [[t('simple.home.peak.streams'), c.peak], [t('simple.home.peak.transcodes'), c.peak_transcodes], [t('simple.home.peak.direct_streams'), c.peak_direct_streams], [t('simple.home.peak.direct_plays'), c.peak_direct_plays]].map(([name, n]) => ({ name, value: num(n || 0) })) }) : null;
  if (peak) peak.dataset.card = 'concurrent';
  return [
    titles('watched-series', t('simple.home.card.watched_series'), d.series, by),
    titles('popular-series', t('simple.home.card.popular_series'), d.popSeries, 'users'),
    titles('watched-movies', t('simple.home.card.watched_movies'), d.movies, by),
    titles('popular-movies', t('simple.home.card.popular_movies'), d.popMovies, 'users'),
    titles('watched-music', t('simple.home.card.watched_music'), d.music, by),
    titles('popular-music', t('simple.home.card.popular_music'), d.popMusic, 'users'),
    recent,
    ranked('libraries', t('simple.home.card.libraries'), rowsOf(d.libraries), { by, backdrop: false, href: (r) => `/libraries/${r.id}`, picture: (r) => tile({ icon: libraryIcon(r.sub), tone: 'neutral', size: 64 }) }),
    d.users ? ranked('users', t('simple.home.card.users'), rowsOf(d.users), { by, backdrop: false, href: (r) => `/users/${r.id}`, picture: (r) => avatar(r.id, r.name, { size: 64, hasImage: r.has_image }) }) : null,
    ranked('platforms', t('simple.home.card.platforms'), rowsOf(d.clients), { by, backdrop: false, href: () => null, picture: (r) => platformTile(r.name, 64) }),
    peak,
  ].filter(Boolean);
}

// ---------------------------------------------------------------- libraries and arrivals

const COUNTS = [['movie_count', t('simple.home.count.movies')], ['series_count', t('simple.home.count.shows')], ['season_count', t('simple.home.count.seasons')], ['episode_count', t('simple.home.count.episodes')], ['artist_count', t('simple.home.count.artists')], ['album_count', t('simple.home.count.albums')], ['track_count', t('simple.home.count.tracks')]];

function libraryCard(l) {
  const rows = COUNTS.filter(([k]) => l[k] > 0).map(([k, name]) => ({ name, value: num(l[k]) }));
  if (!rows.length) rows.push({ name: t('simple.home.count.items'), value: num(l.item_count || 0) });
  rows.push({ name: t('simple.home.count.last_played'), value: l.last_played_at ? relTime(l.last_played_at) : t('simple.home.count.never') });
  const el = statCard({ title: l.name, ranked: false, picture: h('a', { href: `/libraries/${l.id}`, 'aria-label': l.name }, tile({ icon: libraryIcon(l.collection_type), tone: 'neutral', size: 64 })), rows });
  el.dataset.library = l.id;
  return el;
}

function arrival(it) {
  const what = it.kind === 'episodes' ? (it.episodes === 1 ? (it.episode_number != null ? t('simple.home.episode_n', { number: String(it.episode_number) }) : t('terms.episode', { count: 1 })) : t('terms.episode', { count: it.episodes })) : it.sub;
  return mediaCard({ href: `/items/${it.id}`, poster: poster(it.image_item_id || it.id, it.name, { w: 240, cls: 'fui-poster--grid' }), name: it.name, sub: [what, relTime(it.added_at)].filter(Boolean).join(' · ') });
}
