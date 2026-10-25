// Larger blocks shared by several pages.

import { h, icon, num, compact, bytes, duration, durationExact, clock, bitrate, humanize, episodeCode, store, mount, relTime, dateTime, dayLabelLong, languageName, pct } from './dom.js';
import { columnsChart, columnsTable, heatmap, heatmapTable, sparkline, bucketList, libBucketList, simpleColumns, simpleColumnsTable, radarChart } from './charts.js';
import { card, chartCard, segmented, statTile, poster, avatar, methodBadge, facts, sk } from './components.js';
import { nowPlayingView, followNowPlaying } from './nowplaying.js';
import { rangeLong, can } from './state.js';
import { button } from '../finui/components/button/button.js';
import { typed } from './menus.js';
import { t, lang } from './i18n.js';

const METRICS = [{ value: 'watch_s', label: t('terms.watch_time') }, { value: 'plays', label: t('widgets.plays') }];

// Jellyfin's TranscodeReason values, each said in the language in use; one Jellyfin adds later reads as its id spaced out.
const TRANSCODE_REASONS = new Set(['ContainerNotSupported', 'VideoCodecNotSupported', 'AudioCodecNotSupported', 'SubtitleCodecNotSupported', 'AudioIsExternal',
  'SecondaryAudioNotSupported', 'VideoProfileNotSupported', 'VideoLevelNotSupported', 'VideoResolutionNotSupported', 'VideoBitDepthNotSupported',
  'VideoFramerateNotSupported', 'RefFramesNotSupported', 'AnamorphicVideoNotSupported', 'InterlacedVideoNotSupported', 'AudioChannelsNotSupported',
  'AudioProfileNotSupported', 'AudioSampleRateNotSupported', 'AudioBitDepthNotSupported', 'ContainerBitrateExceedsLimit', 'VideoBitrateNotSupported',
  'AudioBitrateNotSupported', 'UnknownVideoStreamInfo', 'UnknownAudioStreamInfo', 'DirectPlayError', 'VideoRangeTypeNotSupported',
  'VideoCodecTagNotSupported', 'StreamCountExceedsLimit']);
/** "ContainerNotSupported" → "Container not supported", in the language in use. */
export const transcodeReason = (id) => (TRANSCODE_REASONS.has(String(id)) ? t(`pages.playback.reason.${id}`) : humanize(id));

function metricPref() { const m = store.get('finstats.metric', 'watch_s'); return m === 'plays' ? 'plays' : 'watch_s'; }

/** Stacked columns by media type, with a Watch time | Plays view switch. */
export function activityCard({ daily, bucket, title = t('widgets.activity.title') }) {
  let metric = metricPref();
  const el = chartCard({
    title,
    sub: bucket === 'week' ? t('widgets.activity.per_week') : t('widgets.activity.per_day'),
    controls: segmented({ label: t('widgets.activity.measure'), size: 'sm', value: metric, options: METRICS,
      onChange: (v) => { metric = v; store.set('finstats.metric', v); el.rerender(); } }),
    chart: () => columnsChart({ daily, bucket, metric }),
    table: () => columnsTable({ daily, bucket }),
  });
  return el;
}

export function heatmapCard({ data, title = t('widgets.heatmap.title') }) {
  return chartCard({
    title, sub: t('widgets.heatmap.sub'),
    chart: () => heatmap({ data, metric: 'plays' }),
    table: () => heatmapTable({ data }),
  });
}

export function overviewTiles({ totals, previous, daily, days, scoped }) {
  const vs = previous ? (days === 365 ? t('widgets.tiles.vs_year') : t('widgets.tiles.vs_days', { count: days })) : null;
  const tot = totals || {}, p = previous || null;
  return h('div', { class: 'fui-stat-tile__grid' },
    statTile({ label: t('terms.watch_time'), value: duration(tot.watch_s), title: durationExact(tot.watch_s), current: tot.watch_s, previous: p && p.watch_s, vsLabel: vs,
      hint: rangeLong(days), spark: sparkline((daily || []).map((d) => d.watch_s)) }),
    statTile({ label: t('widgets.plays'), value: compact(tot.plays), title: num(tot.plays), current: tot.plays, previous: p && p.plays, vsLabel: vs,
      hint: rangeLong(days), spark: sparkline((daily || []).map((d) => d.plays)) }),
    scoped ? null : statTile({ label: t('widgets.tiles.active_users'), value: compact(tot.active_users), title: num(tot.active_users), current: tot.active_users, previous: p && p.active_users, vsLabel: vs, hint: rangeLong(days) }),
    statTile({ label: t('widgets.tiles.distinct_titles'), value: compact(tot.distinct_items), title: num(tot.distinct_items), current: tot.distinct_items, previous: p && p.distinct_items, vsLabel: vs, hint: rangeLong(days) }));
}

// ---------------------------------------------------------------- now playing
const openTranscode = new Set(); // keeps <details> open across polls

export function nowPlayingCard(sn) {
  const code = episodeCode(sn.season_number, sn.episode_number);
  const prog = sn.runtime_s ? Math.max(0, Math.min(1, (sn.position_s || 0) / sn.runtime_s)) : null;
  const title = sn.series_name || sn.item_name;
  const tc = sn.transcode;

  let details = null;
  if (tc) {
    details = h('details', { class: 'np-details' },
      h('summary', null, icon('chevronRight', 13), t('widgets.np.transcode_details')),
      facts([
        [t('widgets.media.video'), tc.is_video_direct ? t('widgets.media.copied_direct') : (tc.video_codec || '–').toUpperCase(), { mono: true }],
        [t('widgets.media.audio'), tc.is_audio_direct ? t('widgets.media.copied_direct') : (tc.audio_codec || '–').toUpperCase(), { mono: true }],
        [t('widgets.media.container'), tc.container, { mono: true }],
        [t('widgets.media.hardware'), tc.hw_accel && tc.hw_accel !== 'none' ? tc.hw_accel.toUpperCase() : t('widgets.media.software'), { mono: true }],
        tc.progress != null ? [t('widgets.media.transcoded'), pct(Math.min(1, tc.progress)), { mono: true }] : null,
        [t('widgets.media.reasons'), tc.reasons && tc.reasons.length ? tc.reasons.map(transcodeReason).join(', ') : '–'],
      ]));
    details.open = openTranscode.has(sn.key);
    details.addEventListener('toggle', () => { if (details.open) openTranscode.add(sn.key); else openTranscode.delete(sn.key); });
  }

  return h('article', { class: ['np', sn.is_paused && 'is-paused'], 'data-np-key': sn.key },
    poster(sn.image_item_id, title, { w: 300, cls: 'fui-poster--np' }),
    h('div', { class: 'np-main' },
      h('div', { class: 'np-title' }, h('a', { href: `/items/${sn.series_id || sn.item_id}`, dataset: typed(sn.series_id ? 'Series' : sn.item_type) }, title)),
      sn.series_name ? h('div', { class: 'np-sub' }, code ? h('span', { class: 'mono' }, code) : null, code ? ' · ' : null, sn.item_name) : null,
      h('div', { class: 'np-user' }, avatar(sn.user_id, sn.user_name, { size: 20 }), h('a', { href: `/users/${sn.user_id}` }, sn.user_name),
        h('span', { class: 'muted' }, ' · ', sn.client && sn.device_name ? t('widgets.np.client_on_device', { client: sn.client, device: sn.device_name }) : sn.client || sn.device_name || '')),
      h('div', { class: 'np-badges' },
        sn.is_paused ? h('span', { class: 'fui-badge fui-badge--paused' }, icon('pause', 11), t('widgets.np.paused')) : h('span', { class: 'fui-badge fui-badge--live' }, h('span', { class: 'fui-badge__dot' }), t('widgets.np.playing')),
        methodBadge(sn.play_method),
        [sn.video, sn.audio].filter(Boolean).map((x) => h('span', { class: 'fui-chip mono' }, x)),
        sn.bitrate ? h('span', { class: 'fui-chip mono' }, bitrate(sn.bitrate)) : null,
        sn.group && sn.group.with && sn.group.with.length ? h('span', { class: 'fui-chip fui-chip--group', title: t('widgets.np.together_title', { count: sn.group.size }) }, icon('together', 12),
          // The server names a few companions; the size says how many there are in all.
          sn.group.size - 1 > sn.group.with.length ? t('widgets.np.with_more', { names: sn.group.with.map((w) => w.user_name).join(', '), count: sn.group.size - 1 - sn.group.with.length })
            : t('widgets.np.with', { names: sn.group.with.map((w) => w.user_name).join(', ') })) : null,
        can('see_network') && sn.remote_ip ? h('span', { class: 'fui-chip mono', title: t('plays_table.ip_address') }, sn.remote_ip) : null),
      h('div', { class: 'np-progress' },
        h('div', { class: 'fui-meter fui-meter--wide', role: 'progressbar', 'aria-label': t('widgets.np.position_aria'), 'aria-valuemin': 0, 'aria-valuemax': 100, 'aria-valuenow': prog == null ? null : Math.round(prog * 100) },
          h('span', { class: 'fui-meter__fill', style: { width: (prog || 0) * 100 + '%' } })),
        h('span', { class: 'mono np-time' }, sn.runtime_s ? `${clock(sn.position_s)} / ${clock(sn.runtime_s)}` : clock(sn.position_s))),
      details));
}

/** Now playing as a home opens with it: the heading with a count, the live view, and the one way it is fed (pushed over the
 *  live channel when it changes, asked for every 5 s only while that is not there). The dashboard and Simple view both
 *  draw this, so both narrow, tick and fall back the same way. `el` goes on the page; `start()` once it is there. */
export function nowPlayingSection(ctx, { title = t('terms.now_playing'), id } = {}) {
  const count = h('span', { class: 'fui-badge--count mono', hidden: true });
  const body = h('div', { class: 'np-wrap' }, h('div', { class: 'np-grid' }, h('div', { class: 'np fui-skeleton--wide' }, h('span', { class: 'fui-skeleton fui-skeleton--poster' }),
    h('div', { class: 'fui-skeleton__lines' }, sk.line('50%', 14), sk.line('35%'), sk.line('80%', 8)))));
  let first = true;
  const live = nowPlayingView({ list: nowPlayingList, parts: (card) => ({ clock: card.querySelector('.np-time'), bar: card.querySelector('.np-progress .fui-meter'), fill: card.querySelector('.np-progress .fui-meter__fill') }) }); // ticks every second between the polls
  ctx.onCleanup(() => live.destroy());
  function paint(data) {
    const sessions = data.sessions || [];
    count.hidden = !sessions.length;
    count.textContent = String(sessions.length);
    if (first) mount(body, live.el);
    live.update(sessions);
    first = false;
  }
  const failed = (e) => { if (first) mount(body, h('p', { class: 'np-empty' }, icon('alert', 14), ' ' + t('widgets.np.unavailable', { reason: e.message }))); };
  const el = h('section', { class: 'np-section', id, 'aria-label': title }, h('h2', { class: 'section-title' }, title, count), body);
  return { el, start: () => followNowPlaying(ctx, { paint, failed }) };
}

export function nowPlayingList(sessions) {
  if (!sessions.length) return h('p', { class: 'np-empty' }, t('widgets.np.empty'));
  // forget <details> state for sessions that ended
  const keys = new Set(sessions.map((x) => x.key));
  for (const k of openTranscode) if (!keys.has(k)) openTranscode.delete(k);
  return h('div', { class: 'np-grid' }, sessions.map(nowPlayingCard));
}

// ---------------------------------------------------------------- insights (GET /api/stats/insights)
const upper = (x) => (x && String(x).length <= 6 ? String(x).toUpperCase() : x);

/** The quieter second row of dashboard tiles. Returns null when there is nothing to say. */
export function insightTiles(ins) {
  if (!ins) return null;
  const c = ins.concurrency || {};
  const net = Array.isArray(ins.network) ? ins.network : [];
  const netTotal = net.reduce((a, b) => a + (b.plays || 0), 0);
  const remote = (net.find((b) => b.name === 'Remote') || {}).plays || 0;
  const tiles = [
    c.peak != null ? h('div', { class: 'fui-stat-tile tile-quiet' },
      h('div', { class: 'fui-stat-tile__label' }, t('widgets.insights.peak')),
      h('div', { class: 'fui-stat-tile__value' }, num(c.peak)),
      h('div', { class: 'fui-stat-tile__foot' }, h('span', { class: 'fui-stat-tile__vs' },
        c.peak_transcodes > 0 ? t('widgets.insights.peak_transcodes', { count: c.peak_transcodes }) : c.peak_at ? h('span', { title: dateTime(c.peak_at) }, relTime(c.peak_at)) : ' '))) : null,
    ins.data_bytes != null ? h('div', { class: 'fui-stat-tile tile-quiet' },
      h('div', { class: 'fui-stat-tile__label' }, t('widgets.insights.data_streamed')),
      h('div', { class: 'fui-stat-tile__value', title: t('widgets.insights.bytes', { count: ins.data_bytes }) }, bytes(ins.data_bytes)),
      h('div', { class: 'fui-stat-tile__foot' }, h('span', { class: 'fui-stat-tile__vs' }, t('widgets.insights.estimated')))) : null,
    can('see_network') && netTotal > 0 ? h('div', { class: 'fui-stat-tile tile-quiet' },
      h('div', { class: 'fui-stat-tile__label' }, t('widgets.insights.remote_plays')),
      h('div', { class: 'fui-stat-tile__value' }, pct(remote / netTotal)),
      h('div', { class: 'fui-stat-tile__foot' }, h('span', { class: 'fui-stat-tile__vs' }, t('widgets.insights.remote_of', { remote, count: netTotal })))) : null,
  ].filter(Boolean);
  return tiles.length ? h('div', { class: 'fui-stat-tile__grid fui-stat-tile__grid--quiet' }, tiles) : null;
}

/** Genres as the ranked list (the actual numbers, the default) or as a radar (the shape of someone's taste). The choice is remembered. */
export function genresCard(genres, { sub = t('widgets.by_watch_time') } = {}) {
  const body = h('div', { class: 'genres-body' });
  let view = store.get('finstats.genresView', 'list') === 'radar' ? 'radar' : 'list';
  const paint = () => mount(body, view === 'radar'
    ? radarChart(genres, { ariaLabel: t('widgets.genres.aria'), empty: t('widgets.genres.empty') })
    : bucketList(genres, { empty: t('widgets.genres.empty') }));
  paint();
  return card({ title: t('widgets.genres.title'), sub, body,
    actions: segmented({ label: t('widgets.genres.view'), value: view, options: [{ value: 'list', label: t('widgets.genres.list') }, { value: 'radar', label: t('widgets.genres.radar') }],
      onChange: (v) => { view = v; store.set('finstats.genresView', v); paint(); } }) });
}

/** The titles watched together most, as a ranked list with posters. */
export function titlesList(titles, n = 5) {
  return h('ol', { class: 'fui-rank-list' }, (titles || []).slice(0, n).map((x, i) => h('li', { class: 'fui-rank-list__row' },
    h('span', { class: 'fui-rank-list__rank mono' }, String(i + 1)), poster(x.image_item_id, x.name, { w: 120, cls: 'fui-poster--sm' }),
    h('div', { class: 'fui-rank-list__main' }, h('a', { class: 'fui-rank-list__name', href: `/items/${x.id}` }, x.name), h('div', { class: 'fui-rank-list__sub' }, t('widgets.groups.evenings', { count: x.sessions }))),
    h('div', { class: 'fui-rank-list__nums' }, h('span', { class: 'mono fui-rank-list__watch', title: durationExact(x.together_s) }, duration(x.together_s))))));
}

/** Who watches together. `g` is /api/stats/groups; hidden entirely when nobody has. */
export function groupsCard(g, { title = t('terms.watched_together'), forUser = null } = {}) {
  if (!g || !g.totals || !g.totals.sessions) return null;
  const tot = g.totals;
  // The person's own share of watch time spent in company, when the answer carries them.
  const me = forUser && Array.isArray(g.people) ? g.people.find((p) => p.user_id === forUser) : null;
  const link = button({ variant: 'ghost', size: 'sm', href: forUser ? `/together?user_id=${encodeURIComponent(forUser)}` : '/together' }, t('nav.together'), icon('chevronRight', 14));
  const faces = (members) => h('span', { class: 'faces' }, members.filter((m) => m.user_id !== forUser).map((m) => avatar(m.user_id, m.user_name, { size: 22, hasImage: m.has_image })));
  const names = (members) => members.filter((m) => m.user_id !== forUser).map((m) => m.user_name).join(forUser ? ', ' : ' + ');
  return card({ title, sub: t('widgets.groups.sub'), actions: link,
    body: [
      h('div', { class: 'group-facts' },
        h('div', null, h('span', { class: 'group-num' }, duration(tot.together_s)), h('span', { class: 'group-label' }, t('widgets.groups.time_together'))),
        h('div', null, h('span', { class: 'group-num' }, num(tot.sessions)), h('span', { class: 'group-label' }, t('widgets.groups.evenings_word', { count: tot.sessions }))),
        forUser ? null : h('div', null, h('span', { class: 'group-num' }, num(tot.people)), h('span', { class: 'group-label' }, t('widgets.groups.people_word'))),
        me && me.share != null ? h('div', null, h('span', { class: 'group-num' }, pct(me.share)), h('span', { class: 'group-label' }, t('widgets.groups.share'))) : null),
      h('div', { class: 'grid-2 group-lists' },
        h('div', null, h('h3', { class: 'group-head' }, forUser ? t('widgets.groups.most_often_with') : t('widgets.groups.groups')),
          h('ol', { class: 'fui-rank-list' }, (g.companions || []).slice(0, 5).map((c, i) => h('li', { class: 'fui-rank-list__row' },
            h('span', { class: 'fui-rank-list__rank mono' }, String(i + 1)), faces(c.members),
            h('div', { class: 'fui-rank-list__main' }, h('span', { class: 'fui-rank-list__name' }, names(c.members)), h('div', { class: 'fui-rank-list__sub' }, t('widgets.groups.sessions', { count: c.sessions }))),
            h('div', { class: 'fui-rank-list__nums' }, h('span', { class: 'mono fui-rank-list__watch', title: durationExact(c.together_s) }, duration(c.together_s))))))),
        h('div', null, h('h3', { class: 'group-head' }, t('widgets.groups.titles')), titlesList(g.titles)))] });
}

/** Admin-only; hidden entirely when there is nothing to show. */
export function failedLoginsCard(rows) {
  if (!can('see_server') || !Array.isArray(rows) || !rows.length) return null;
  return card({ title: t('widgets.failed_logins.title'), sub: t('widgets.failed_logins.sub'),
    actions: button({ variant: 'ghost', size: 'sm', href: '/server/log' }, t('widgets.failed_logins.server_log'), icon('chevronRight', 14)),
    body: h('ul', { class: 'mini-list' }, rows.map((r) => h('li', { class: 'mini-row' },
      h('span', { class: 'fui-badge--status fui-badge--warning' }, icon('alert', 13)),
      h('span', { class: 'mini-main' }, r.overview || t('widgets.failed_logins.one'), r.user_name ? h('span', { class: 'muted' }, ' · ' + r.user_name) : null),
      h('time', { class: 'mono muted mini-when', title: dateTime(r.date) }, relTime(r.date))))) });
}

// ---------------------------------------------------------------- library make-up (GET /api/library/insights)
const monthf = new Intl.DateTimeFormat(lang(), { month: 'short', year: '2-digit' });
const monthLong = new Intl.DateTimeFormat(lang(), { month: 'long', year: 'numeric' });
function monthDate(str) { const [y, m] = String(str).split('-').map(Number); return new Date(y || 1970, (m || 1) - 1, 1); }

function libItemRows(items, { empty, showAdded = false }) {
  if (!items || !items.length) return h('div', { class: 'fui-empty--chart fui-empty--chart-sm' }, empty);
  return h('ol', { class: 'fui-rank-list' }, items.map((it, i) => h('li', { class: 'fui-rank-list__row' },
    h('span', { class: 'fui-rank-list__rank mono' }, String(i + 1)),
    poster(it.image_item_id || it.id, it.name, { w: 120, cls: 'fui-poster--sm' }),
    h('div', { class: 'fui-rank-list__main' }, it.id ? h('a', { href: `/items/${it.id}`, class: 'fui-rank-list__name', dataset: typed(it.type) }, it.name) : h('span', { class: 'fui-rank-list__name' }, it.name),
      h('div', { class: 'fui-rank-list__sub' }, [it.year, it.type === 'Series' ? t('item_type.Series') : null, showAdded && it.date_created ? t('widgets.library.added', { when: relTime(it.date_created) }) : null].filter(Boolean).join(' · ') || ' ')),
    h('div', { class: 'fui-rank-list__nums' }, h('span', { class: 'mono fui-rank-list__watch' }, it.size_bytes ? bytes(it.size_bytes) : '–')))));
}

/**
 * "What your library is made of". Not scoped by the time range: it describes the files,
 * so it renders under its own heading, away from the range-filtered cards.
 */
export function libraryInsights(d, { scoped = false } = {}) {
  if (!d) return null;
  const tot = d.totals || {};
  if (!(tot.files > 0) && !(d.resolutions || []).length && !(d.largest || []).length) {
    return h('section', { class: 'subsection' }, h('h2', { class: 'subsection-title' }, scoped ? t('widgets.library.title_this') : t('widgets.library.title_yours')),
      h('p', { class: 'np-empty' }, t('widgets.library.after_sync')));
  }
  const b = (title, sub, rows, labelFn, unit) => card({ title, sub, body: libBucketList(rows, { labelFn, unit }) });
  const decades = (d.decades || []).map((x) => ({ label: x.name, title: x.name, value: x.count }));
  const added = (d.added || []).map((x) => ({ label: monthf.format(monthDate(x.month)), title: monthLong.format(monthDate(x.month)), value: x.count }));
  const un = d.unwatched || null;
  const counts = [['movies', tot.movies], ['series', tot.series], ['episodes', tot.episodes], ['tracks', tot.tracks]].filter(([, v]) => v > 0);
  return h('section', { class: 'subsection stack' },
    h('div', null, h('h2', { class: 'subsection-title' }, scoped ? t('widgets.library.title_this') : t('widgets.library.title_yours')),
      h('p', { class: 'subsection-sub' }, t('widgets.library.about_files'))),
    h('div', { class: 'fui-stat-tile__grid fui-stat-tile__grid--three' },
      h('div', { class: 'fui-stat-tile' }, h('div', { class: 'fui-stat-tile__label' }, t('widgets.library.files')), h('div', { class: 'fui-stat-tile__value' }, compact(tot.files)),
        h('div', { class: 'fui-stat-tile__foot' }, h('span', { class: 'fui-stat-tile__vs' }, counts.map(([k, v]) => t(`widgets.library.count.${k}`, { count: v })).join(' · ') || ' '))),
      h('div', { class: 'fui-stat-tile' }, h('div', { class: 'fui-stat-tile__label' }, t('widgets.library.total_size')), h('div', { class: 'fui-stat-tile__value' }, bytes(tot.size_bytes)),
        h('div', { class: 'fui-stat-tile__foot' }, h('span', { class: 'fui-stat-tile__vs' }, tot.files > 0 && tot.size_bytes > 0 ? t('widgets.library.per_file', { size: bytes(tot.size_bytes / tot.files) }) : ' '))),
      h('div', { class: 'fui-stat-tile' }, h('div', { class: 'fui-stat-tile__label' }, t('widgets.library.total_runtime')), h('div', { class: 'fui-stat-tile__value', title: durationExact(tot.runtime_s) }, duration(tot.runtime_s)),
        h('div', { class: 'fui-stat-tile__foot' }, h('span', { class: 'fui-stat-tile__vs' }, t('widgets.library.play_all_once'))))),
    h('div', { class: 'grid-3' },
      b(t('widgets.library.resolutions'), t('widgets.library.video_files'), d.resolutions),
      b(t('widgets.library.video_codecs'), t('widgets.library.video_files'), d.video_codecs, upper),
      b(t('widgets.library.dynamic_range'), t('widgets.library.video_files'), d.video_ranges)),
    h('div', { class: 'grid-3' },
      b(t('widgets.library.containers'), t('widgets.library.all_files'), d.containers, upper),
      b(t('widgets.library.audio_codecs'), t('widgets.library.first_audio'), d.audio_codecs, upper),
      b(t('widgets.genres.title'), t('widgets.library.movies_and_series'), d.genres, undefined, t('widgets.library.titles'))),
    (d.audio_languages && d.audio_languages.length) || (d.subtitle_languages && d.subtitle_languages.length) ? h('div', { class: 'grid-2' },
      b(t('widgets.library.audio_languages'), t('widgets.library.audio_languages_sub'), d.audio_languages, languageName),
      b(t('widgets.library.subtitle_languages'), t('widgets.library.subtitle_languages_sub'), d.subtitle_languages, languageName)) : null,
    h('div', { class: 'grid-2' },
      chartCard({ title: t('widgets.library.by_decade'), sub: t('widgets.library.by_decade_sub'),
        chart: () => simpleColumns({ rows: decades, unit: (n) => t('terms.title', { count: n }), ariaLabel: t('widgets.library.by_decade_aria') }),
        table: () => simpleColumnsTable({ rows: decades, head: [t('widgets.library.decade'), t('widgets.library.titles')] }) }),
      chartCard({ title: t('widgets.library.added_per_month'), sub: t('widgets.library.last_24_months'),
        chart: () => simpleColumns({ rows: added, unit: (n) => t('terms.title', { count: n }), ariaLabel: t('widgets.library.added_aria') }),
        table: () => simpleColumnsTable({ rows: added, head: [t('widgets.library.month'), t('widgets.library.added_head')] }) })),
    h('div', { class: 'grid-2' },
      card({ title: t('widgets.library.largest'), sub: t('widgets.library.largest_sub'), body: libItemRows(d.largest, { empty: t('widgets.library.no_sizes') }) }),
      card({ title: t('widgets.library.never_watched'),
        sub: un && un.count > 0 ? t('widgets.library.never_watched_sub', { count: un.count, size: bytes(un.size_bytes) }) : t('widgets.library.all_played'),
        body: [libItemRows(un && un.items, { empty: t('widgets.library.nothing_unwatched'), showAdded: true }),
          h('p', { class: 'fui-field__help fui-card__note' }, t('widgets.library.played_note'))] })));
}

// ---------------------------------------------------------------- a shelf: posters in a row
/** A row of posters that scrolls sideways inside itself; the arrows (put into `arrows`) move it by most of a screen. */
export function shelfRow(cards, arrows, label) {
  const list = h('ul', { class: 'shelf', tabindex: 0, 'aria-label': label }, cards.map((c) => h('li', null, c)));
  // Scrolling is done by hand. CSS scroll-snap swallowed a wheel notch that moved less than half a card (the row sprang back),
  // native arrow keys moved 40 px at a time and Home/End not at all. Everything goes through one glide towards a target, so
  // fast repeated input adds up instead of restarting an animation.
  const calm = window.matchMedia && matchMedia('(prefers-reduced-motion: reduce)').matches;
  let target = 0, frame = 0;
  const max = () => Math.max(0, list.scrollWidth - list.clientWidth);
  const step = () => { const li = list.firstElementChild; return li ? li.getBoundingClientRect().width + (parseFloat(getComputedStyle(list).columnGap) || 0) : 148; };
  function glide(to) {
    target = Math.max(0, Math.min(max(), to));
    if (calm) { list.scrollLeft = target; return; }
    if (frame) return;
    const tick = () => {
      const d = target - list.scrollLeft, before = list.scrollLeft;
      if (Math.abs(d) < 1.5) { list.scrollLeft = target; frame = 0; return; }
      list.scrollLeft = before + Math.sign(d) * Math.max(1, Math.abs(d) * 0.24);
      frame = list.scrollLeft === before ? 0 : requestAnimationFrame(tick); // an edge we cannot pass: stop
    };
    frame = requestAnimationFrame(tick);
  }
  const from = () => (frame ? target : list.scrollLeft); // mid-glide, the next input continues from where the glide is heading
  const page = (dir) => glide(from() + dir * Math.max(step(), list.clientWidth - step()));

  const prev = button({ variant: 'icon', type: 'button', 'aria-label': t('widgets.shelf.back'), onClick: () => page(-1) }, icon('chevronLeft', 16));
  const next = button({ variant: 'icon', type: 'button', 'aria-label': t('widgets.shelf.on'), onClick: () => page(1) }, icon('chevronRight', 16));
  arrows.replaceChildren(prev, next);

  // Shift + wheel (a mouse has no sideways wheel). A trackpad's own sideways swipe and the
  // plain wheel are left to the browser, so the page still scrolls normally with the pointer over the row.
  list.addEventListener('wheel', (e) => {
    if (!e.shiftKey || e.ctrlKey || e.metaKey) return;
    const raw = e.deltaX || e.deltaY; // Chromium turns shift+wheel into deltaX, Firefox leaves it in deltaY
    if (!raw || max() <= 0) return;
    e.preventDefault();
    const px = Math.abs(e.deltaMode === 1 ? raw * 16 : e.deltaMode === 2 ? raw * list.clientWidth : raw);
    // One notch is one poster, like one press of an arrow key, however many pixels the browser calls a notch (48 to 100). People
    // flick several notches at a time, so anything more per notch overshoots. The small deltas of a trackpad or a free-spinning
    // wheel scale instead, a notch's worth of them adding up to the same poster.
    glide(from() + Math.sign(raw) * (px >= 30 ? step() : px * step() / 50));
  }, { passive: false });

  // Arrow keys move one poster, Page Up/Down a screenful, Home/End to the ends, on the row itself or on a card in it.
  list.addEventListener('keydown', (e) => {
    if (e.altKey || e.ctrlKey || e.metaKey) return;
    const to = { ArrowRight: () => from() + step(), ArrowLeft: () => from() - step(), PageDown: () => from() + list.clientWidth - step(),
      PageUp: () => from() - list.clientWidth + step(), Home: () => 0, End: () => max() }[e.key];
    if (!to) return;
    e.preventDefault();
    glide(to());
  });

  // The arrows stay clickable at either end (aria-disabled says why nothing happens) and vanish when everything fits.
  const paint = () => {
    arrows.hidden = max() <= 4;
    prev.setAttribute('aria-disabled', String(list.scrollLeft <= 2));
    next.setAttribute('aria-disabled', String(list.scrollLeft >= max() - 2));
  };
  list.addEventListener('scroll', paint, { passive: true });
  if ('ResizeObserver' in window) new ResizeObserver(paint).observe(list);
  requestAnimationFrame(paint);
  return list;
}
