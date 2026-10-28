import { h, humanize, store, methodLabel } from '../dom.js';
import { api, soft } from '../api.js';
import { readDays, saveDays, can } from '../state.js';
import { replaceQuery } from '../router.js';
import { pageHeader, card, chartCard, filterBar, dataView, sk, segmented } from '../components.js';
import { bucketList, methodsBar, simpleColumns, simpleColumnsTable, clientMethods, methodLegend } from '../charts.js';
import { num, duration, pct, dayLabel, dayLabelLong, dateTime, clock, relEl } from '../dom.js';
import { chartTable, plainTable } from '../tables.js';
import { t, lang } from '../i18n.js';

const upper = (x) => (x && x.length <= 5 ? x.toUpperCase() : x);
const CHANNELS = { 1: t('pages.playback.mono'), 2: t('pages.playback.stereo'), Mono: t('pages.playback.mono'), Stereo: t('pages.playback.stereo'), 6: '5.1', 8: '7.1' };
const chLabel = (x) => CHANNELS[x] || (/^\d+$/.test(String(x)) ? t('pages.playback.channels', { count: Number(x) })
  : /^\d+ ch$/.test(String(x)) ? t('pages.playback.channels_short', { count: parseInt(x, 10) }) : x);
const REASONS = ['ContainerNotSupported', 'VideoCodecNotSupported', 'AudioCodecNotSupported', 'SubtitleCodecNotSupported', 'AudioIsExternal',
  'SecondaryAudioNotSupported', 'VideoProfileNotSupported', 'VideoLevelNotSupported', 'VideoResolutionNotSupported', 'VideoBitDepthNotSupported',
  'VideoFramerateNotSupported', 'RefFramesNotSupported', 'AnamorphicVideoNotSupported', 'InterlacedVideoNotSupported', 'AudioChannelsNotSupported',
  'AudioProfileNotSupported', 'AudioSampleRateNotSupported', 'AudioBitDepthNotSupported', 'ContainerBitrateExceedsLimit', 'VideoBitrateNotSupported',
  'AudioBitrateNotSupported', 'UnknownVideoStreamInfo', 'UnknownAudioStreamInfo', 'DirectPlayError', 'VideoRangeTypeNotSupported',
  'VideoCodecTagNotSupported', 'StreamCountExceedsLimit'];
const reasonLabel = (x) => (REASONS.includes(x) ? t(`pages.playback.reason.${x}`) : humanize(x));
const COMPLETION = { 'Under 10%': 'under_10', '10–50%': 'from_10', '50–90%': 'from_50', 'Finished (90%+)': 'finished' };
const completionLabel = (x) => (COMPLETION[x] ? t(`pages.playback.completion.${COMPLETION[x]}`) : x);
const NETWORK = { Local: t('pages.playback.local'), Remote: t('plays_table.remote'), Unknown: t('common.unknown') };
const networkLabel = (x) => NETWORK[x] || x;
const oneDigit = new Intl.NumberFormat(lang(), { minimumFractionDigits: 1, maximumFractionDigits: 1 });

// Shared with the prefetcher, so a prefetched view has exactly the address the page asks for.
async function loadPlayback({ days, userId }, signal) {
  const f = { days, user_id: userId }, o = { signal };
  const [d, ins, files] = await Promise.all([api.get('/stats/playback', f, o), soft(api.get('/stats/insights', f, o)), soft(api.get('/stats/files', f, o))]);
  return { d, ins, files };
}
const scopeOf = (query) => ({ days: readDays(query), userId: can('see_everyone') ? query.get('user_id') || '' : '' });
export const prefetchPlayback = ({ query, signal }) => [() => loadPlayback(scopeOf(query), signal)];

export default function playback(ctx) {
  ctx.title(t('nav.playback'));
  let { days, userId } = scopeOf(ctx.query);
  let metric = store.get('finstats.methodMetric', 'plays') === 'watch_s' ? 'watch_s' : 'plays';
  const view = h('div', { class: 'stack' });

  const dv = dataView({
    container: view, signal: ctx.signal,
    skeleton: () => [sk.cardBlock(80), h('div', { class: 'grid-3' }, sk.cardRows(5), sk.cardRows(5), sk.cardRows(5))],
    fetch: () => loadPlayback({ days, userId }, ctx.signal),
    render: ({ d, ins, files }) => {
      const methodsCard = chartCard({
        title: t('pages.playback.methods_title'), sub: t('pages.playback.methods_sub'),
        controls: segmented({ label: t('pages.playback.measure'), size: 'sm', value: metric, options: [{ value: 'plays', label: t('pages.playback.plays') }, { value: 'watch_s', label: t('terms.watch_time') }],
          onChange: (v) => { metric = v; store.set('finstats.methodMetric', v); methodsCard.rerender(); } }),
        chart: () => methodsBar(d.methods, metric),
        table: () => methodsTable(d.methods),
      });
      const b = (title, sub, rows, labelFn, empty) => card({ title, sub, body: bucketList(rows, { labelFn, empty }) });
      return [
        methodsCard,
        insightCards(ins),
        fileCards(files),
        h('div', { class: 'grid-3' },
          b(t('pages.playback.reasons_title'), t('pages.playback.reasons_sub'), d.transcode_reasons, reasonLabel, t('pages.playback.nothing_transcoded')),
          b(t('pages.playback.hw_title'), t('pages.playback.hw_sub'), d.hw_accel, (x) => (x && x !== 'None' && x !== 'Software' ? upper(x) : t('pages.playback.software')), t('pages.playback.nothing_transcoded')),
          b(t('pages.playback.clients_title'), t('pages.playback.clients_sub'), d.clients)),
        h('div', { class: 'grid-3' },
          b(t('pages.playback.video_codecs'), t('pages.playback.source_files'), d.video_codecs, upper),
          b(t('pages.playback.resolutions'), t('pages.playback.source_files'), d.resolutions),
          b(t('pages.playback.dynamic_range'), t('pages.playback.source_files'), d.video_ranges)),
        h('div', { class: 'grid-3' },
          b(t('pages.playback.audio_codecs'), t('pages.playback.selected_audio'), d.audio_codecs, upper),
          b(t('pages.playback.audio_channels'), t('pages.playback.selected_audio'), d.audio_channels, chLabel),
          b(t('pages.playback.containers'), t('pages.playback.source_files'), d.containers, upper)),
        h('div', { class: 'grid-3' },
          b(t('pages.playback.subtitles'), t('pages.playback.selected_subtitle'), d.subtitles, (x) => (x === 'None' ? t('pages.playback.off') : x))),
      ];
    },
  });

  const sync = () => replaceQuery({ days, user_id: userId });
  ctx.root.append(pageHeader(t('nav.playback'), t('pages.playback.sub')),
    filterBar({ days, userId, signal: ctx.signal, onDays: (v) => { days = v; saveDays(v); sync(); dv.load(); }, onUser: (v) => { userId = v; sync(); dv.load(); } }),
    view);
  dv.load();
}

/** Cards fed by /api/stats/insights. Every one of them is optional. */
function insightCards(ins) {
  if (!ins) return null;
  const c = ins.concurrency || {};
  const week = c.bucket === 'week';
  const series = (c.series || []).map((x) => ({ label: dayLabel(x.date), title: week ? t('pages.playback.week_of', { date: dayLabelLong(x.date) }) : dayLabelLong(x.date), value: x.peak }));
  const beh = ins.behaviour || {};
  const net = Array.isArray(ins.network) ? ins.network : [];
  const one = (x) => (x == null ? '–' : oneDigit.format(Number(x)));

  const concurrency = chartCard({
    title: t('pages.playback.concurrency_title'),
    sub: c.peak ? [c.peak_at ? t(`pages.playback.peak_at_${week ? 'week' : 'day'}`, { peak: c.peak, when: dateTime(c.peak_at) }) : t(`pages.playback.peak_${week ? 'week' : 'day'}`, { peak: c.peak }),
      c.peak_transcodes ? ` · ${t('pages.playback.peak_transcodes', { count: c.peak_transcodes })}` : ''].join('') : t(`pages.playback.per_${week ? 'week' : 'day'}`),
    chart: () => simpleColumns({ rows: series, unit: (v) => `${num(v)} ${t('pages.playback.unit_streams', { count: v })}`, ariaLabel: t('pages.playback.peak_aria'), empty: t('plays_table.empty_range') }),
    table: () => simpleColumnsTable({ rows: series, head: [week ? t('pages.playback.week_of_head') : t('pages.playback.date'), t('pages.playback.peak_streams')] }),
  });
  const clients = card({ title: t('pages.playback.which_clients'), sub: t('pages.playback.which_clients_sub'), actions: methodLegend(), cls: 'card-legend', body: clientMethods(ins.client_methods) });
  const completion = card({ title: t('pages.playback.how_far'), sub: t('pages.playback.how_far_sub'),
    body: bucketList(ins.completion, { labelFn: completionLabel, watch: false, empty: t('pages.playback.no_video_plays') }) });
  const network = can('see_network') && net.length ? card({ title: t('pages.playback.network'), sub: t('pages.playback.network_sub'), body: bucketList(net, { labelFn: networkLabel }) }) : null;
  const behaviour = beh.plays_measured > 0 ? card({ title: t('pages.playback.behaviour_title'), sub: t('pages.playback.based_on', { count: beh.plays_measured }),
    body: h('dl', { class: 'kpis' },
      h('div', null, h('dt', null, t('pages.playback.pauses')), h('dd', { class: 'mono' }, one(beh.avg_pauses))),
      h('div', null, h('dt', null, t('pages.playback.skips')), h('dd', { class: 'mono' }, one(beh.avg_seeks))),
      h('div', null, h('dt', null, t('pages.playback.picked_up')), h('dd', { class: 'mono' }, beh.resumed_share == null ? '–' : pct(beh.resumed_share)))) }) : null;
  return [concurrency, h('div', { class: 'grid-2' }, clients, completion), network || behaviour ? h('div', { class: 'grid-2' }, network, behaviour) : null];
}

/** Files worth a look, from everyone's plays. Empty lists, and a caller who may only see themselves, show nothing. */
function fileCards(files) {
  if (!files || !can('see_everyone')) return null;
  const title = (r) => h('a', { href: `/items/${r.id}` }, r.series_name ? `${r.series_name} · ${r.name}` : r.name);
  const table = (head, rows, cells) => plainTable(h('table', { class: 'fui-data-table fui-data-table--dense' },
    h('thead', null, h('tr', null, head.map(([label, right]) => h('th', { class: right ? 'r' : null }, label)))),
    h('tbody', null, rows.map((r) => h('tr', null, h('td', { class: 'bucket-name', title: r.series_name ? `${r.series_name} · ${r.name}` : r.name }, title(r)), cells(r))))));
  const r = (x) => h('td', { class: 'mono r' }, x);
  const broken = (files.broken || []).length ? card({ title: t('pages.playback.broken_title'), sub: t('pages.playback.broken_sub'), cls: 'fui-card--flush',
    body: table([[t('plays_table.title')], [t('pages.playback.tries'), 1], [t('pages.playback.people'), 1], [t('pages.playback.apps')], [t('pages.playback.last_tried')]], files.broken,
      (x) => [r(num(x.plays)), r(num(x.users)), h('td', null, (x.clients || []).join(', ') || '–'), h('td', null, relEl(x.last_tried_at))]) }) : null;
  const rewound = (files.rewound || []).length ? card({ title: t('pages.playback.rewound_title'), sub: t('pages.playback.rewound_sub'), cls: 'fui-card--flush',
    body: table([[t('plays_table.title')], [t('pages.playback.plays'), 1], [t('pages.playback.rewinds'), 1], [t('pages.playback.per_play'), 1], [t('pages.playback.hot_spot'), 1]], files.rewound,
      (x) => [r(num(x.plays)), r(num(x.rewinds)), r(oneDigit.format(Number(x.per_play))), r(x.hot_s == null ? '–' : clock(x.hot_s))]) }) : null;
  const subtitled = (files.subtitled || []).length ? card({ title: t('pages.playback.subtitled_title'), sub: t('pages.playback.subtitled_sub'), cls: 'fui-card--flush',
    body: table([[t('plays_table.title')], [t('pages.playback.plays'), 1], [t('pages.playback.switched_on'), 1], [t('pages.playback.share'), 1], [t('pages.playback.typically_at'), 1]], files.subtitled,
      (x) => [r(num(x.plays)), r(num(x.switched_on)), r(pct(x.share)), r(x.typical_s == null ? '–' : clock(x.typical_s))]) }) : null;
  const cards = [broken, rewound, subtitled].filter(Boolean);
  return cards.length ? cards : null;   // each its own full-width card: five columns do not share a row
}

function methodsTable(methods) {
  const rows = methods || [];
  const total = rows.reduce((a, m) => a + (m.plays || 0), 0);
  return chartTable(h('table', { class: 'fui-data-table' },
    h('thead', null, h('tr', null, h('th', null, t('plays_table.method')), h('th', { class: 'r' }, t('pages.playback.plays')), h('th', { class: 'r' }, t('pages.playback.share')), h('th', { class: 'r' }, t('terms.watch_time')))),
    h('tbody', null, rows.map((m) => h('tr', null, h('td', null, methodLabel(m.name)), h('td', { class: 'mono r' }, num(m.plays)),
      h('td', { class: 'mono r' }, total ? pct(m.plays / total, 1) : '–'), h('td', { class: 'mono r' }, duration(m.watch_s)))))));
}
