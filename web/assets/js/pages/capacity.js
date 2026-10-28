// Server → Capacity: will this server cope? The busiest moments on record, how fast each disk fills, and what that says
// about the server's own settings. The server works it all out after its reads; this only draws it. FinStats cannot know
// what the machine could do, so nothing here says how many streams it can take: it shows what it has seen.

import { h, icon, num, bytes, duration, relTime, dateTime, dateOnly } from '../dom.js';
import { api } from '../api.js';
import { can } from '../state.js';
import { card, dataView, sk, emptyState, setBusy, facts, statTile } from '../components.js';
import { plainTable } from '../tables.js';
import { askDismiss } from './health.js';
import { button } from '../../finui/components/button/button.js';
import { toast } from '../../finui/components/toast/toast.js';
import { t, lang } from '../i18n.js';

// Shared with the prefetcher, so a prefetched view has exactly the address the page asks for.
const loadCapacity = (signal) => api.get('/server/capacity', null, { signal });
export const prefetchCapacity = ({ signal }) => [() => loadCapacity(signal)];

const and = (list) => new Intl.ListFormat(lang(), { type: 'conjunction' }).format(list);
/** "about 9 months"; under one, "under a month"; past two years, in years, since a pace that far out is a rough guide. */
const months = (m) => (m < 1 ? t('pages.capacity.left.under_a_month') : m < 1.5 ? t('pages.capacity.left.about_a_month') : m < 24 ? t('pages.capacity.left.months', { count: Math.round(m) }) : t('pages.capacity.left.years', { count: Math.round(m / 12) }));

/** One finding in a sentence, from its evidence. */
export function describe(f) {
  const e = f.evidence || {};
  switch (f.kind) {
    case 'software_transcodes':
      return [t('pages.capacity.describe.software_transcodes.transcoded', { count: e.transcodes, days: e.days, plays: e.plays }),
        e.unsaid ? t('pages.capacity.describe.software_transcodes.software_unsaid', { software: e.software, unsaid: e.unsaid, most: e.most_at_once })
          : t('pages.capacity.describe.software_transcodes.software', { software: e.software, most: e.most_at_once }),
        t('pages.capacity.describe.software_transcodes.where')].join(' ');
    case 'scan_busy':
      return [t(e.duty_pct >= 100 ? 'pages.capacity.describe.scan_busy.always' : 'pages.capacity.describe.scan_busy.share',
        { interval: duration(e.interval_s), last: duration(e.last_duration_s), pct: e.duty_pct, count: e.items }),
        t('pages.capacity.describe.scan_busy.where')].join(' ');
    case 'runway':
      return t('pages.capacity.describe.runway', { free: bytes(e.free_bytes), added: bytes(e.added_per_month), count: e.window_months,
        libraries: and(e.libraries || []), left: months(e.months_left) });
    default:
      return '';
  }
}

export function capacityView(ctx) {
  const manage = can('manage') && can('see_server');
  const view = h('div', { class: 'stack' });

  async function bringBack(btn, f) {
    setBusy(btn, true, t('pages.capacity.bringing_back'));
    // Refused, the page has nothing new to draw, so the button would stay busy and nobody would be told.
    try { await api.post('/library/health/undismiss', { key: f.key }); } catch (e) { setBusy(btn, false); toast({ text: t('pages.capacity.bring_back_failed', { error: e.message }), tone: 'critical' }); return; }
    dv.load();
  }

  function findings(list) {
    if (!list.length) return null;
    return card({ title: t('pages.capacity.findings.title'), sub: t('pages.capacity.findings.sub'), id: 'capacity-findings',
      body: h('ul', { class: 'capacity-findings' }, list.map((f) => h('li', { class: ['capacity-finding', f.dismissed && 'is-aside'] },
        h('p', { class: 'capacity-finding__title' }, f.title),
        h('p', { class: 'fui-field__help' }, `${describe(f)}.`),
        f.dismissed ? h('p', { class: 'health-aside' }, icon('check', 12), f.dismissed_by ? t('pages.capacity.findings.set_aside_by', { when: relTime(f.dismissed_at), name: f.dismissed_by }) : t('pages.capacity.findings.set_aside', { when: relTime(f.dismissed_at) }), f.note ? `: ${f.note}` : '') : null,
        h('p', { class: 'muted capacity-finding__found', title: dateTime(f.found_at) }, t('pages.capacity.findings.found', { when: relTime(f.found_at) })),
        manage ? h('div', { class: 'health-actions' }, f.dismissed
          ? button({ size: 'sm', type: 'button', onClick: (e) => bringBack(e.currentTarget, f) }, icon('refresh', 13), t('pages.capacity.findings.bring_back'))
          : button({ size: 'sm', type: 'button', onClick: () => askDismiss(f, describe(f), () => dv.load()) }, icon('check', 13), t('pages.capacity.findings.dismiss'))) : null))) });
  }

  function busiest(d) {
    // Without everyone's plays the server answers the day of a busy moment, not its minute.
    const when = (at) => (d.by_day ? dateOnly(at) : relTime(at));
    const tile = (label, m, none) => statTile({ label, value: m ? t('pages.capacity.busiest.streams', { count: m.streams }) : '–',
      hint: m ? (m.transcodes ? t('pages.capacity.busiest.hint_transcodes', { transcodes: t('pages.capacity.busiest.transcodes', { count: m.transcodes }), when: when(m.at) }) : when(m.at)) : none });
    const rows = (d.busiest || []).map((m) => h('tr', null,
      h('td', { class: 'nowrap' }, h('time', { datetime: new Date(m.at * 1000).toISOString(), title: when(m.at) }, d.by_day ? dateOnly(m.at) : dateTime(m.at))),
      h('td', { class: 'mono r' }, num(m.streams)),
      h('td', { class: 'mono r' }, num(m.transcodes)),
      h('td', { class: 'mono r' }, num(m.hardware)),
      h('td', { class: 'mono r' }, num(m.software)),
      h('td', { class: 'mono r' }, num(m.unsaid))));
    return card({ title: t('pages.capacity.busiest.title'), sub: t('pages.capacity.busiest.sub'), id: 'capacity-busiest',
      body: h('div', { class: 'stack' },
        h('div', { class: 'fui-stat-tile__grid' }, tile(t('pages.capacity.busiest.on_record'), (d.busiest || [])[0], t('pages.capacity.busiest.nothing_played')), tile(t('pages.capacity.busiest.in_days', { count: d.recent_days }), d.recent, t('pages.capacity.busiest.nothing_in_days', { count: d.recent_days }))),
        rows.length ? plainTable(h('table', { class: 'fui-data-table' },
          h('thead', null, h('tr', null, h('th', null, t('pages.capacity.busiest.when')), h('th', { class: 'r' }, t('pages.capacity.busiest.col_streams')), h('th', { class: 'r' }, t('pages.capacity.busiest.col_transcoded')),
            h('th', { class: 'r', title: t('pages.capacity.busiest.hardware_title') }, t('pages.capacity.busiest.hardware')), h('th', { class: 'r', title: t('pages.capacity.busiest.software_title') }, t('pages.capacity.busiest.software')),
            h('th', { class: 'r', title: t('pages.capacity.busiest.unsaid_title') }, t('pages.capacity.busiest.unsaid')))),
          h('tbody', null, rows))) : emptyState(t('pages.capacity.busiest.nothing_played'), t('pages.capacity.busiest.empty_sub'))) });
  }

  function disks(d) {
    if (!d.free_space_reported) {
      const rows = (d.growth || []).map((g) => h('tr', null, h('td', null, g.library), h('td', { class: 'mono r' }, bytes(g.added_bytes)), h('td', { class: 'mono r' }, g.young ? '–' : bytes(g.added_per_month))));
      return card({ title: t('pages.capacity.disks.title'), sub: t('pages.capacity.disks.growth_sub', { count: d.window_months }), id: 'capacity-disks',
        body: [h('p', { class: 'fui-field__help' }, t('pages.capacity.disks.unreported')),
          rows.length ? plainTable(h('table', { class: 'fui-data-table' }, h('thead', null, h('tr', null, h('th', null, t('pages.capacity.disks.library')), h('th', { class: 'r' }, t('pages.capacity.disks.added_in', { count: d.window_months })), h('th', { class: 'r' }, t('pages.capacity.disks.a_month')))), h('tbody', null, rows))) : null] });
    }
    // Each disk's own window: a disk filled more recently is measured from a month after its first file, and not before.
    const over = (k) => (k.window_months != null ? k.window_months : d.window_months);
    const said = (k) => (k.young
      ? t('pages.capacity.disks.young', { libraries: and(k.libraries) })
      : k.added_per_month > 0
        ? t('pages.capacity.disks.grew', { libraries: and(k.libraries), added: bytes(k.added_per_month), count: over(k), left: months(k.months_left) })
        : t('pages.capacity.disks.nothing_added', { libraries: and(k.libraries), count: over(k) }));
    return card({ title: t('pages.capacity.disks.title'), sub: t('pages.capacity.disks.sub', { count: d.window_months }), id: 'capacity-disks',
      body: h('div', { class: 'stack' },
        h('div', { class: 'stack-sm' }, d.disks.map((k) => h('div', { class: 'capacity-disk' },
          h('p', null, said(k)),
          // A disk too new to have a pace has none to show, not one of nothing.
          facts([[t('pages.capacity.disks.folders'), k.folders.join(', '), { wide: true }], [t('pages.capacity.disks.free'), bytes(k.free_bytes), { mono: true }], k.young ? null : [t('pages.capacity.disks.added_a_month'), bytes(k.added_per_month), { mono: true }]].filter(Boolean))))),
        h('p', { class: 'fui-field__help' }, t('pages.capacity.disks.counted_by'))) });
  }

  const dv = dataView({
    container: view, signal: ctx.signal,
    skeleton: () => [sk.cardBlock(200), sk.cardBlock(160)],
    fetch: () => loadCapacity(ctx.signal),
    render: (d) => {
      if (!d) return emptyState(t('pages.capacity.empty'), t('pages.capacity.empty_sub'));
      return [findings(d.findings || []), busiest(d), disks(d)];
    },
  });
  dv.load();
  return [view];
}
