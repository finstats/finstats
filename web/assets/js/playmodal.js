// Detail modal for one play (GET /api/activity/{id}).

import { h, icon, num, duration, durationExact, dateTime, timeOfDay, clock, bitrate, pct, humanize, episodeCode, mount, TRACKERS, methodLabel } from './dom.js';
import { api, isAbort } from './api.js';
import { can } from './state.js';
import { openModal, copyButton, methodBadge, facts, errorState, sk, poster, setBusy, inlineError } from './components.js';
import { button } from '../finui/components/button/button.js';
import { toast } from '../finui/components/toast/toast.js';
import { transcodeReason } from './widgets.js';
import { t } from './i18n.js';

const res = (w, hgt) => (w && hgt ? `${w}×${hgt}` : null);
const EVENT = {
  start: [t('playmodal.event.start'), 'play'], pause: [t('playmodal.event.pause'), 'pause'], resume: [t('playmodal.event.resume'), 'play'], seek: [t('playmodal.event.seek'), 'skip'],
  audio: [t('playmodal.event.audio'), 'volume'], subtitle: [t('playmodal.event.subtitle'), 'captions'], transcode: [t('playmodal.event.transcode'), 'cpu'], stop: [t('playmodal.event.stop'), 'stop'],
};

/** Local / Remote as icon + word (never colour alone). `null` = unknown → nothing. */
export function networkChip(isLocal) {
  if (isLocal == null) return null;
  return h('span', { class: 'fui-chip', title: isLocal ? t('playmodal.network.local_title') : t('playmodal.network.remote_title') },
    icon(isLocal ? 'lan' : 'globe', 12), isLocal ? t('playmodal.network.local') : t('playmodal.network.remote'));
}

// Where a play came from. Anything but 'live' is history somebody brought with them: one row per
// play, so there is no timeline behind it and no pause or skip count.
const imported = (p) => p.source && p.source !== 'live';
const sourceName = (p) => (!imported(p) ? t('playmodal.source.live') : TRACKERS[p.source] ? t('playmodal.source.imported_from', { tracker: TRACKERS[p.source] }) : t('playmodal.source.imported'));

// "Transcode: ContainerNotSupported, AudioCodecNotSupported" as the collector writes it, said in the language in use.
function transcodeDetail(detail) {
  const [method, reasons] = String(detail).split(': ');
  if (reasons == null) return methodLabel(method);
  return t('playmodal.event.transcode_detail', { method: methodLabel(method), reasons: reasons.split(', ').map(transcodeReason).join(', ') });
}

function timeline(p) {
  const events = Array.isArray(p.events) ? p.events : [];
  if (!events.length) {
    const why = imported(p) ? (TRACKERS[p.source] ? t('playmodal.no_events.imported_from', { tracker: TRACKERS[p.source] }) : t('playmodal.no_events.imported'))
      : t('playmodal.no_events.live');
    return h('p', { class: 'fui-field__help' }, why);
  }
  return h('ol', { class: 'timeline' }, events.map((e) => {
    const [label, ic] = EVENT[e.kind] || [e.kind ? humanize(e.kind) : t('playmodal.event.other'), 'info'];
    return h('li', { class: 'tl-row' },
      h('time', { class: 'mono tl-time', title: dateTime(e.at) }, timeOfDay(e.at)),
      h('span', { class: 'tl-mark', 'aria-hidden': 'true' }, icon(ic, 12)),
      h('div', { class: 'tl-body' },
        h('span', { class: 'tl-kind' }, label),
        e.position_s != null ? h('span', { class: 'mono tl-pos', title: t('playmodal.position_title') }, t('playmodal.at', { time: clock(e.position_s) })) : null,
        // "Transcode: ContainerNotSupported" reads better as "Transcode: Container not supported"
        e.detail ? h('span', { class: 'tl-detail' }, e.kind === 'transcode' ? transcodeDetail(e.detail) : e.kind === 'subtitle' && e.detail === 'Off' ? t('playmodal.off') : String(e.detail)) : null));
  }));
}

const channels = (n) => (n == null ? null : { 1: t('playmodal.channels.mono'), 2: t('playmodal.channels.stereo'), 6: '5.1', 8: '7.1' }[n] || t('playmodal.channels.count', { count: n }));

export function openPlayModal(play, { onDeleted, onRestored } = {}) {
  const abort = new AbortController();
  const body = h('div', { class: 'play-modal' }, sk.rows(3));
  const modal = openModal({ title: t('playmodal.title'), body, wide: true, onClose: () => abort.abort() });

  async function load() {
    try {
      const p = await api.get(`/activity/${play.id}`, null, { signal: abort.signal });
      mount(body, render(p));
    } catch (e) {
      if (isAbort(e) || e.status === 401) return;
      mount(body, errorState(e, () => { mount(body, sk.rows(3)); load(); }));
    }
  }

  function render(p) {
    const code = episodeCode(p.season_number, p.episode_number);
    const canLink = p.item_exists !== false;
    const close = () => modal.close();
    const titleMain = p.series_name || p.item_name || t('common.unknown_item');
    const head = h('div', { class: 'play-head' },
      poster(p.image_item_id, titleMain, { w: 300, cls: 'fui-poster--md' }),
      h('div', { class: 'play-head-text' },
        h('div', { class: 'play-head-title' }, canLink && (p.series_id || p.item_id) ? h('a', { href: `/items/${p.series_id || p.item_id}`, onClick: close }, titleMain) : titleMain),
        p.series_name ? h('div', { class: 'play-head-sub' }, code ? h('span', { class: 'mono' }, code) : null, code ? ' · ' : null,
          canLink ? h('a', { href: `/items/${p.item_id}`, onClick: close }, p.item_name) : p.item_name) : null,
        h('div', { class: 'play-head-meta' }, methodBadge(p.play_method),
          p.active ? h('span', { class: 'fui-badge fui-badge--live' }, h('span', { class: 'fui-badge__dot' }), t('plays_table.playing_now')) : null,
          h('span', { class: 'fui-chip' }, sourceName(p)),
          canLink ? null : h('span', { class: 'fui-chip' }, t('playmodal.not_in_library')))));

    const session = facts([
      [t('plays_table.user'), h('a', { href: `/users/${p.user_id}`, onClick: close }, p.user_name)],
      [t('playmodal.started'), dateTime(p.started_at), { mono: true }],
      [t('playmodal.ended'), p.active ? t('playmodal.still_playing') : dateTime(p.ended_at), { mono: true }],
      [t('plays_table.watched'), h('span', { title: durationExact(p.duration_s) }, duration(p.duration_s)), { mono: true }],
      [t('playmodal.paused_for'), p.paused_s ? duration(p.paused_s) : '–', { mono: true }],
      p.start_position_s > 30 ? [t('playmodal.resumed_from'), clock(p.start_position_s), { mono: true }] : null,
      !imported(p) && p.pause_count != null ? [t('playmodal.pauses'), num(p.pause_count), { mono: true }] : null,
      !imported(p) && p.seek_count != null ? [t('playmodal.skips'), num(p.seek_count), { mono: true }] : null,
      [t('playmodal.stopped_at'), p.position_s != null ? `${clock(p.position_s)}${p.runtime_s ? ' / ' + clock(p.runtime_s) : ''}${p.completion != null ? ` (${pct(Math.min(1, p.completion))})` : ''}` : '–', { mono: true }],
    ]);

    const device = facts([
      [t('plays_table.client'), [p.client, p.app_version].filter(Boolean).join(' ') || '–'],
      [t('playmodal.device'), p.device_name],
      p.device_id ? [t('playmodal.device_id'), h('span', { class: 'fui-copy__row' }, h('span', { class: 'mono trunc' }, p.device_id), copyButton(p.device_id, t('playmodal.copy_device_id')))] : null,
      p.watched_with && p.watched_with.length ? [t('playmodal.watched_with'), h('span', null, p.watched_with.map((w, i) => [i ? ', ' : '', h('a', { href: `/users/${w.user_id}` }, w.user_name)]))] : null,
      can('see_network') ? [t('plays_table.ip_address'), p.remote_ip ? h('span', { class: 'fui-copy__row' }, h('span', { class: 'mono' }, p.remote_ip), copyButton(p.remote_ip, t('playmodal.copy_ip')), networkChip(p.is_local)) : '–'] : null,
    ]);

    const media = facts([
      [t('widgets.media.container'), p.container, { mono: true }],
      [t('widgets.media.bitrate'), bitrate(p.bitrate), { mono: true }],
      [t('widgets.media.video'), [p.video_codec && p.video_codec.toUpperCase(), res(p.width, p.height), p.video_range, p.bit_depth ? t('playmodal.bit_depth', { bits: p.bit_depth }) : null].filter(Boolean).join(' · ') || p.video, { mono: true }],
      [t('widgets.media.audio'), [p.audio_codec && p.audio_codec.toUpperCase(), channels(p.audio_channels), p.audio_language].filter(Boolean).join(' · ') || p.audio, { mono: true }],
      [t('widgets.media.subtitles'), [p.subtitle_language, p.subtitle_codec].filter(Boolean).join(' · ') || p.subtitle || t('playmodal.off'), { mono: true }],
    ]);

    const tc = p.transcode;
    const transcode = tc ? [h('h3', { class: 'section-label' }, t('playmodal.event.transcode')), facts([
      [t('widgets.media.video'), tc.is_video_direct ? t('widgets.media.copied_direct') : [tc.video_codec && tc.video_codec.toUpperCase(), res(tc.width, tc.height)].filter(Boolean).join(' · '), { mono: true }],
      [t('widgets.media.audio'), tc.is_audio_direct ? t('widgets.media.copied_direct') : [tc.audio_codec && tc.audio_codec.toUpperCase(), channels(tc.audio_channels)].filter(Boolean).join(' · '), { mono: true }],
      [t('widgets.media.container'), tc.container, { mono: true }],
      [t('widgets.media.bitrate'), bitrate(tc.bitrate), { mono: true }],
      [t('widgets.media.hardware'), tc.hw_accel && tc.hw_accel !== 'none' ? tc.hw_accel.toUpperCase() : t('widgets.media.software'), { mono: true }],
      [t('widgets.media.reasons'), tc.reasons && tc.reasons.length ? h('span', { class: 'fui-chip__set' }, tc.reasons.map((r) => h('span', { class: 'fui-chip' }, transcodeReason(r)))) : '–'],
    ])] : null;

    return [head,
      h('h3', { class: 'section-label' }, t('playmodal.section.session')), session,
      h('h3', { class: 'section-label' }, t('playmodal.device')), device,
      h('h3', { class: 'section-label' }, t('playmodal.section.media')), media,
      transcode,
      h('h3', { class: 'section-label' }, t('playmodal.section.timeline')), timeline(p),
      can('manage') && !p.active ? deleteRow(p) : null];
  }

  // Destructive action: inline two-step inside the modal, explicit Cancel/Delete.
  function deleteRow(p) {
    const row = h('div', { class: 'danger-row' });
    const idle = () => mount(row, button({ variant: 'ghost', size: 'sm', tone: 'danger', type: 'button', onClick: confirm }, icon('trash', 14), t('playmodal.delete.start')));
    function confirm() {
      const err = h('div');
      const del = button({ size: 'sm', variant: 'danger', type: 'button' }, t('playmodal.delete.confirm'));
      const cancel = button({ size: 'sm', type: 'button', onClick: () => { idle(); row.querySelector('button').focus(); } }, t('common.cancel'));
      del.addEventListener('click', async () => {
        setBusy(del, true, t('playmodal.delete.deleting')); cancel.disabled = true;
        try {
          await api.put(`/activity/${p.id}`, { deleted: true });
          modal.close();
          if (onDeleted) onDeleted(p);
          undoToast(p, onRestored || onDeleted);
        } catch (e) {
          setBusy(del, false); cancel.disabled = false;
          mount(err, inlineError('del-err', e.message));
        }
      });
      mount(row, h('div', { class: 'confirm', role: 'group', 'aria-label': t('playmodal.delete.aria') },
        h('p', null, t('playmodal.delete.question')), h('div', { class: 'confirm-btns' }, cancel, del)), err);
      del.focus();
    }
    idle();
    return row;
  }

  load();
  return modal;
}

/** Said the moment a play goes to the trash, with the one button that takes it back out. */
export function undoToast(p, onRestored) {
  toast({ text: p.item_name ? t('playmodal.undo.deleted_named', { title: p.item_name }) : t('playmodal.undo.deleted'), tone: 'info', action: t('playmodal.undo.undo'),
    onAction: async () => {
      try { await api.put(`/activity/${p.id}`, { deleted: false }); if (onRestored) onRestored(p); }
      catch (e) { toast({ text: t('playmodal.undo.failed', { reason: e.message }), tone: 'critical' }); }
    } });
}
