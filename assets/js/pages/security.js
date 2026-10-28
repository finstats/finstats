import { h, icon, num, duration, dateTime, dateOnly, relEl, relTime } from '../dom.js';
import { api } from '../api.js';
import { readDays, saveDays, can, rangeLong } from '../state.js';
import { replaceQuery } from '../router.js';
import { pageHeader, card, filterBar, dataView, sk, emptyState, statTile, segmented, pagination, avatar, openModal, setBusy, inlineError } from '../components.js';
import { bucketList, counted } from '../charts.js';
import { dataTable } from '../tables.js';
import { worldMap, placeTip, countryName } from '../worldmap.js';
import { button } from '../../finui/components/button/button.js';
import { toast } from '../../finui/components/toast/toast.js';
import { t } from '../i18n.js';

const PER_PAGE = 5;
// Status colours always ship with an icon and a word.
const SEVERITY = { high: { cls: 'fui-badge--critical', icon: 'alert', label: t('pages.security.severity.high') }, medium: { cls: 'fui-badge--warning', icon: 'alert', label: t('pages.security.severity.medium') } };
const KIND = { impossible_travel: t('pages.security.kind.impossible_travel'), new_country: t('pages.security.kind.new_country') };

// Shared with the prefetcher, so a prefetched view has exactly the addresses the page asks for.
function loadSecurity({ days, userId, status, page }, signal) {
  const o = { signal };
  return Promise.all([
    api.get('/security', { days, user_id: userId }, o),
    api.get('/security/alerts', { status, user_id: userId, page, per_page: PER_PAGE }, o),
  ]).then(([overview, alerts]) => ({ overview, alerts }));
}
const scopeOf = (query) => ({ days: readDays(query), userId: query.get('user_id') || '', status: ['resolved', 'all'].includes(query.get('alerts')) ? query.get('alerts') : 'open',
  page: Math.max(1, Number(query.get('page')) || 1) });
export const prefetchSecurity = ({ query, signal }) => [() => loadSecurity(scopeOf(query), signal)];

const km = (n) => t('pages.security.km', { distance: num(n) });
function gapText(sec) {
  if (sec < 90) return t('pages.security.gap_seconds', { count: Math.max(1, Math.round(sec)) });
  if (sec < 5400) return t('pages.security.gap_minutes', { count: Math.round(sec / 60) });
  return duration(sec);
}

function travelSentence(d) {
  return d.overlap
    ? t('pages.security.travel_overlap', { distance: km(d.distance_km) })
    : t('pages.security.travel_speed', { distance: km(d.distance_km), gap: gapText(d.gap_s), speed: num(d.speed_kmh) });
}
const country = (d) => countryName(d.country_code, d.country);
const homeLabel = (p) => (p.home ? t('pages.security.home_place', { place: p.label }) : p.label);
const usersText = (users) => users.slice(0, 3).map((u) => u.name).join(', ') + (users.length > 3 ? ` +${users.length - 3}` : '');

function sightingFacts(label, side) {
  return h('div', { class: 'alert-side' },
    h('div', { class: 'alert-side-label' }, label),
    h('div', { class: 'alert-side-place' }, side.home ? icon('home', 13) : icon('globe', 13), side.place),
    h('div', { class: 'alert-side-what' }, side.what),
    h('div', { class: 'alert-side-meta mono' }, h('span', { title: dateTime(side.at) }, dateTime(side.at)), side.ip ? h('span', null, side.ip) : null));
}

export default function securityPage(ctx) {
  ctx.title(t('nav.security'));
  const f = scopeOf(ctx.query);
  const view = h('div', { class: 'stack' });
  let map = null;

  const sync = () => replaceQuery({ days: f.days, user_id: f.userId, alerts: f.status === 'open' ? '' : f.status, page: f.page > 1 ? f.page : '' });
  const reload = () => { sync(); dv.load(); };

  function resolveDialog(a) {
    const travel = a.kind === 'impossible_travel';
    const note = h('input', { class: 'fui-field__input', type: 'text', maxLength: 500, placeholder: t('pages.security.note_placeholder'), id: 'alert-note', autocomplete: 'off' });
    const mute = h('input', { type: 'checkbox' });
    const error = inlineError('alert-error', '');
    error.hidden = true;
    const save = button({ variant: 'primary', type: 'submit' }, icon('check', 14), t('pages.security.resolve'));
    const form = h('form', { class: 'stack-sm' },
      h('p', { class: 'fui-field__help' }, travel ? t('pages.security.resolve_help_travel', { user: a.user_name, travel: travelSentence(a.details) })
        : t('pages.security.resolve_help_country', { user: a.user_name, country: country(a.details) })),
      h('div', { class: 'fui-field' }, h('label', { class: 'fui-field__label', for: 'alert-note' }, t('pages.security.note_label')), note),
      travel ? h('label', { class: 'fui-field__check' }, mute, t('pages.security.mute_pair', { from: a.details.from.place, to: a.details.to.place, user: a.user_name })) : null,
      error,
      h('div', { class: 'form-actions' }, save, button({ variant: 'ghost', type: 'button', onClick: () => modal.close() }, t('common.cancel'))));
    const modal = openModal({ title: t('pages.security.resolve_title'), body: form, initialFocus: note });
    form.addEventListener('submit', async (e) => {
      e.preventDefault();
      setBusy(save, true, t('pages.security.resolving'));
      try {
        await api.post(`/security/alerts/${a.id}/resolve`, { note: note.value, mute: mute.checked });
        modal.close();
        dv.load();
      } catch (err) {
        setBusy(save, false);
        error.hidden = false;
        error.lastChild.textContent = err.message;
      }
    });
  }

  /** A button's write, then the page again; refused, the page has nothing new to draw, so the button is given back and the
   *  refusal said, or it would stay busy and nobody would be told. */
  async function pressed(btn, busy, write) {
    setBusy(btn, true, busy);
    try { await write(); } catch (e) { setBusy(btn, false); toast({ text: e.message, tone: 'critical' }); return; }
    dv.load();
  }

  function alertRow(a, manage) {
    const sev = SEVERITY[a.severity] || SEVERITY.medium, d = a.details || {};
    const travel = a.kind === 'impossible_travel' && d.from && d.to;
    const showBtn = d.to ? button({ size: 'sm', variant: 'ghost', type: 'button', onClick: () => map && map.show(travel ? [d.from, d.to] : [d.to], { line: !!travel }) }, icon('compass', 13), t('pages.security.show_on_map')) : null;
    const act = !manage ? null : a.resolved_at
      ? button({ size: 'sm', variant: 'ghost', type: 'button', onClick: (e) => pressed(e.currentTarget, t('pages.security.reopening'), () => api.post(`/security/alerts/${a.id}/reopen`)) }, icon('refresh', 13), t('pages.security.reopen'))
      : button({ size: 'sm', type: 'button', onClick: () => resolveDialog(a) }, icon('check', 13), t('pages.security.resolve_more'));
    return h('li', { class: 'alert-row' + (a.resolved_at ? ' is-resolved' : '') },
      h('div', { class: 'alert-head' },
        h('span', { class: 'fui-badge--status ' + sev.cls }, icon(sev.icon, 13), sev.label),
        h('strong', { class: 'alert-kind' }, KIND[a.kind] || a.kind),
        h('a', { class: 'user-cell', href: `/users/${a.user_id}` }, avatar(a.user_id, a.user_name, { size: 20, hasImage: a.has_image }), a.user_name),
        h('span', { class: 'alert-when' }, relEl(a.at))),
      h('p', { class: 'alert-text' }, travel ? t('pages.security.travel_text', { from: d.from.place, to: d.to.place, travel: travelSentence(d) })
        : (d.to && d.to.place ? t('pages.security.first_time_place', { country: country(d), place: d.to.place }) : t('pages.security.first_time', { country: country(d) }))
          + ' ' + t('pages.security.known_before', { count: d.known })),
      h('div', { class: 'alert-sides' }, travel ? [sightingFacts(t('pages.security.side_before'), d.from), sightingFacts(t('pages.security.side_then'), d.to)] : d.to ? sightingFacts(t('pages.security.side_seen'), d.to) : null),
      a.resolved_at ? h('p', { class: 'alert-resolved' }, icon('check', 13), t('pages.security.resolved_by', { when: relTime(a.resolved_at), name: a.resolved_by || t('pages.security.someone') }), a.note ? `: ${a.note}` : '', a.muted ? ' · ' + t('pages.security.muted') : '') : null,
      h('div', { class: 'alert-actions' }, showBtn, act));
  }

  function alertsCard(alerts, manage) {
    // The last alert of a later page resolved: that page is empty now and has no pager to leave it by. The last page there is.
    if (!alerts.rows.length && alerts.total > 0 && f.page > 1) {
      const last = Math.max(1, Math.ceil(alerts.total / alerts.per_page));
      queueMicrotask(() => { f.page = last; reload(); });
    }
    const tabs = segmented({ label: t('pages.security.which_alerts'), size: 'sm', value: f.status, options: [{ value: 'open', label: alerts.open ? t('pages.security.tab_open_count', { count: alerts.open }) : t('pages.security.tab_open') }, { value: 'resolved', label: t('pages.security.tab_resolved') }, { value: 'all', label: t('pages.security.tab_all') }],
      onChange: (v) => { f.status = v; f.page = 1; reload(); } });
    const resolveAll = manage && f.status === 'open' && alerts.open > 1 && !f.userId
      ? button({ size: 'sm', variant: 'ghost', type: 'button', onClick: (e) => pressed(e.currentTarget, t('pages.security.resolving'), () => api.post('/security/alerts/resolve-all')) }, icon('check', 13), t('pages.security.resolve_all'))
      : null;
    const body = alerts.rows.length
      ? [h('ul', { class: 'alert-list' }, alerts.rows.map((a) => alertRow(a, manage))),
        alerts.total > PER_PAGE ? pagination({ page: alerts.page, perPage: alerts.per_page, total: alerts.total, onPage: (p) => { f.page = p; reload(); } }) : null]
      : h('div', { class: 'fui-empty--chart fui-empty--chart-sm' }, f.status === 'open' ? t('pages.security.alerts_empty_open') : t('pages.security.alerts_empty'));
    return card({ title: t('pages.security.alerts_title'), sub: t('pages.security.alerts_sub'), actions: [resolveAll, tabs], body });
  }

  function mapPoints(o) {
    const pts = [];
    for (const p of o.places) {
      pts.push({ kind: p.home ? 'home' : 'remote', latitude: p.latitude, longitude: p.longitude, weight: p.plays + p.sign_ins, label: homeLabel(p), data: p,
        tip: () => placeTip(p.home ? t('pages.security.home_tip', { place: p.label }) : p.label, [counted('terms.play', p.plays), p.sign_ins ? counted('pages.security.sign_ins', p.sign_ins) : null,
          [usersText(p.users), ''], [relTime(p.last_seen), t('pages.security.last_seen_tip')]]) });
    }
    for (const x of o.failed) {
      pts.push({ kind: 'failed', latitude: x.latitude, longitude: x.longitude, weight: x.attempts, label: x.label, data: { ...x, failed: true },
        tip: () => placeTip(x.label, [counted('pages.security.failed_sign_ins', x.attempts), [relTime(x.last_at), t('pages.security.last_attempt')]]) });
    }
    for (const n of o.now_playing) {
      pts.push({ kind: 'live', latitude: n.latitude, longitude: n.longitude, weight: 1, label: t('pages.security.user_in_place', { user: n.user_name, place: n.place }), data: { ...n, live: true },
        tip: () => placeTip(t('pages.security.user_playing_now', { user: n.user_name }), [[n.series_name ? `${n.series_name} · ${n.item_name}` : n.item_name, ''], [n.home ? t('pages.security.home') : n.place, n.device_name || '']]) });
    }
    return pts;
  }

  function pickedPanel(slot, point) {
    const d = point.data;
    const head = h('div', { class: 'picked-head' }, h('strong', null, point.label),
      button({ variant: 'icon', type: 'button', 'aria-label': t('pages.security.close_place'), onClick: () => slot.replaceChildren() }, icon('x', 14)));
    let body;
    if (d.failed) {
      body = [h('p', { class: 'fui-field__help' }, t('pages.security.failed_summary', { count: d.attempts, when: relTime(d.last_at) })), h('ul', { class: 'picked-list' }, d.events.map((e) => h('li', null, e))),
        can('see_server') ? h('a', { href: '/server/log?type=AuthenticationFailed' }, t('pages.security.open_server_log')) : null];
    } else if (d.live) {
      const title = d.series_name ? `${d.series_name} · ${d.item_name}` : d.item_name;
      body = h('p', { class: 'fui-field__help' }, d.device_name ? t('pages.security.live_playing_on', { user: d.user_name, title, device: d.device_name }) : t('pages.security.live_playing', { user: d.user_name, title }));
    } else {
      const parts = { plays: t('terms.play', { count: d.plays }), time: duration(d.watch_s), when: relTime(d.last_seen),
        sign_ins: d.sign_ins ? t('pages.security.sign_ins', { count: d.sign_ins }) : '', addresses: d.addresses ? t('pages.security.addresses', { count: d.addresses }) : '' };
      const summary = `pages.security.place_summary${d.sign_ins ? '_sign_ins' : ''}${d.addresses ? '_addresses' : ''}`;
      body = [h('p', { class: 'fui-field__help' }, t(summary, parts)),
        h('ul', { class: 'picked-list' }, d.users.map((u) => h('li', null, h('a', { class: 'user-cell', href: `/users/${u.id}` }, avatar(u.id, u.name, { size: 20 }), u.name),
          h('span', { class: 'mono muted' }, t('terms.play', { count: u.plays }) + (u.sign_ins ? ' · ' + t('pages.security.sign_ins', { count: u.sign_ins }) : '')))))];
    }
    slot.replaceChildren(h('div', { class: 'picked', role: 'status' }, head, body));
  }

  function placesTable(places) {
    return dataTable(h('table', { class: 'fui-data-table places-table' },
      h('thead', null, h('tr', null, h('th', null, t('pages.security.col.place')), h('th', null, t('pages.security.col.people')), h('th', { 'data-first': 'desc' }, t('pages.security.col.plays')), h('th', { 'data-first': 'desc' }, t('terms.watch_time')), h('th', { 'data-first': 'desc' }, t('pages.security.col.sign_ins')), h('th', { 'data-first': 'desc' }, t('pages.security.col.last_seen')), h('th', { 'data-nosort': '' }, h('span', { class: 'sr-only' }, t('pages.security.col.map'))))),
      h('tbody', null, places.map((p) => h('tr', null,
        h('td', null, h('span', { class: 'place-cell' }, icon(p.home ? 'home' : 'globe', 13), homeLabel(p))),
        h('td', { 'data-sort': p.users.length }, usersText(p.users)),
        h('td', { class: 'mono' }, num(p.plays)),
        h('td', { class: 'mono', 'data-sort': p.watch_s }, duration(p.watch_s)),
        h('td', { class: 'mono' }, num(p.sign_ins)),
        h('td', { class: 'mono nowrap', 'data-sort': p.last_seen, title: dateTime(p.last_seen) }, relTime(p.last_seen)),
        h('td', null, button({ size: 'sm', variant: 'ghost', type: 'button', 'aria-label': t('pages.security.show_place_on_map', { place: p.label }), onClick: () => map && map.show([p]) }, icon('compass', 13))))))));
  }

  function noDatabase(o) {
    const btn = o.can_manage ? button({ variant: 'primary', type: 'button', onClick: async (e) => {
      const b = e.currentTarget;
      setBusy(b, true, t('pages.security.downloading'));
      try {
        await api.post('/security/database');
        const wait = setInterval(async () => {
          const task = (await api.get('/tasks', null, { signal: ctx.signal })).tasks.find((x) => x.id === 'geoip');
          if (task && task.state === 'running') { b.lastChild.textContent = task.message || t('pages.security.downloading'); return; }
          clearInterval(wait);
          if (task && task.state === 'error') { setBusy(b, false); note.textContent = task.error || t('pages.security.download_failed'); } else dv.load();
        }, 1500);
        ctx.onCleanup(() => clearInterval(wait));
      } catch (err) { setBusy(b, false); note.textContent = err.message; }
    } }, icon('upload', 14), t('pages.security.download_database')) : null;
    const note = h('p', { class: 'fui-field__help', 'aria-live': 'polite' });
    return emptyState(t('pages.security.no_database_title'),
      t('pages.security.no_database_text'),
      h('div', { class: 'fui-empty__action' }, btn, note));
  }

  const dv = dataView({
    container: view, signal: ctx.signal,
    skeleton: () => [sk.tiles(4), sk.cardBlock(120), sk.cardBlock(360)],
    fetch: () => loadSecurity(f, ctx.signal),
    render: ({ overview: o, alerts }) => {
      if (map) { map.destroy(); map = null; }
      if (!o.database) return noDatabase(o);
      const picked = h('div', { class: 'picked-slot' });
      const points = mapPoints(o);
      map = worldMap({ points, countries: o.countries.map((c) => c.code), signal: ctx.signal, onPick: (p) => pickedPanel(picked, p) });
      const away = o.places.filter((p) => !p.home);
      const failedTotal = o.failed.reduce((n, x) => n + x.attempts, 0);
      return [
        h('div', { class: 'fui-stat-tile__grid' },
          statTile({ label: t('pages.security.tile.open_alerts'), value: num(o.open_alerts), hint: o.open_alerts ? t('pages.security.tile.waiting') : t('pages.security.tile.all_clear') }),
          statTile({ label: t('pages.security.tile.countries'), value: num(o.countries.length), hint: rangeLong(f.days) }),
          statTile({ label: t('pages.security.tile.places_away'), value: num(away.length), hint: rangeLong(f.days) }),
          can('see_server') && !f.userId ? statTile({ label: t('pages.security.tile.failed_outside'), value: num(failedTotal), hint: rangeLong(f.days) }) : null),
        card({ title: t('pages.security.map_title'), sub: o.home_known ? t('pages.security.map_sub') : t('pages.security.map_sub_no_home'),
          body: points.length ? [map.el, picked] : h('div', { class: 'fui-empty--chart' }, t('pages.security.map_empty')) }),
        alertsCard(alerts, o.can_manage),
        h('div', { class: 'grid-heat' },
          card({ title: t('pages.security.places_title'), cls: 'fui-card--flush', body: o.places.length ? placesTable(o.places) : h('div', { class: 'fui-empty--chart fui-empty--chart-sm' }, t('pages.security.nothing_in_range')) }),
          card({ title: t('pages.security.countries_title'), sub: t('pages.security.countries_sub'), body: bucketList(o.countries.map((c) => ({ name: countryName(c.code, c.name), plays: c.plays, watch_s: 0 })), { watch: false, empty: t('pages.security.nothing_in_range') }) })),
        h('p', { class: 'attribution' },
          o.database.dbip ? [h('a', { href: 'https://db-ip.com', target: '_blank', rel: 'noopener noreferrer' }, 'IP Geolocation by DB-IP'), ' · '] : null,
          t('pages.security.database_built', { kind: o.database.kind, date: dateOnly(o.database.built_at) }),
          o.addresses_without_place ? ' · ' + t('pages.security.addresses_without_place', { count: o.addresses_without_place }) : '',
          ' · ' + t('pages.security.map_outlines')),
      ];
    },
  });

  ctx.onCleanup(() => { if (map) map.destroy(); });
  ctx.root.append(pageHeader(t('nav.security'), t('pages.security.page_sub')),

    filterBar({ days: f.days, userId: f.userId, signal: ctx.signal,
      onDays: (d) => { f.days = d; saveDays(d); reload(); }, onUser: (u) => { f.userId = u; f.page = 1; reload(); } }),
    view);
  dv.load();
}
