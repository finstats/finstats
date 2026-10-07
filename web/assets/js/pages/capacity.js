// Server → Capacity: will this server cope? The busiest moments on record, how fast each disk fills, and what that says
// about the server's own settings. The server works it all out after its reads; this only draws it. FinStats cannot know
// what the machine could do, so nothing here says how many streams it can take: it shows what it has seen.

import { h, icon, num, bytes, duration, relTime, dateTime } from '../dom.js';
import { api } from '../api.js';
import { can } from '../state.js';
import { card, dataView, sk, emptyState, setBusy, facts, statTile } from '../components.js';
import { plainTable } from '../tables.js';
import { askDismiss } from './health.js';
import { button } from '../../finui/components/button/button.js';

// Shared with the prefetcher, so a prefetched view has exactly the address the page asks for.
const loadCapacity = (signal) => api.get('/server/capacity', null, { signal });
export const prefetchCapacity = ({ signal }) => [() => loadCapacity(signal)];

const plural = (n, one, many = one + 's') => `${num(n)} ${n === 1 ? one : many}`;
const and = (list) => (list.length < 2 ? list.join('') : `${list.slice(0, -1).join(', ')} and ${list[list.length - 1]}`);
/** "about 9 months"; under one, "under a month"; past two years, in years, since a pace that far out is a rough guide. */
const months = (m) => (m < 1 ? 'under a month' : m < 1.5 ? 'about a month' : m < 24 ? `about ${num(Math.round(m))} months` : `about ${num(Math.round(m / 12))} years`);

/** One finding in a sentence, from its evidence. */
export function describe(f) {
  const e = f.evidence || {};
  switch (f.kind) {
    case 'software_transcodes':
      return `In the last ${num(e.days)} days ${plural(e.transcodes, 'play was', 'plays were')} transcoded out of ${num(e.plays)}. `
        + `Of those Jellyfin reported on, ${num(e.software)} ran without hardware acceleration and none with it`
        + `${e.unsaid ? ` (${num(e.unsaid)} did not say)` : ''}, at most ${num(e.most_at_once)} at once. Whether Jellyfin transcodes on a graphics card is set in its Playback settings`;
    case 'scan_busy':
      return `Jellyfin's library scan is set to run every ${duration(e.interval_s)} and last took ${duration(e.last_duration_s)}: `
        + `${num(e.duty_pct)}% of the time, over ${plural(e.items, 'file')}. How often it runs is set in Jellyfin's scheduled tasks`;
    case 'runway':
      return `${bytes(e.free_bytes)} free, and ${bytes(e.added_per_month)} added a month over the last ${num(e.window_months)} months `
        + `(${and(e.libraries || [])}): at that pace, ${months(e.months_left)} of space are left`;
    default:
      return '';
  }
}

export function capacityView(ctx) {
  const manage = can('manage') && can('see_server');
  const view = h('div', { class: 'stack' });

  async function bringBack(btn, f) {
    setBusy(btn, true, 'Bringing back…');
    try { await api.post('/library/health/undismiss', { key: f.key }); } finally { dv.load(); }
  }

  function findings(list) {
    if (!list.length) return null;
    return card({ title: 'Worth a look', sub: 'What the busiest moments, the disks and the server’s settings say', id: 'capacity-findings',
      body: h('ul', { class: 'capacity-findings' }, list.map((f) => h('li', { class: ['capacity-finding', f.dismissed && 'is-aside'] },
        h('p', { class: 'capacity-finding__title' }, f.title),
        h('p', { class: 'fui-field__help' }, `${describe(f)}.`),
        f.dismissed ? h('p', { class: 'health-aside' }, icon('check', 12), `Set aside ${relTime(f.dismissed_at)}${f.dismissed_by ? ' by ' + f.dismissed_by : ''}`, f.note ? `: ${f.note}` : '') : null,
        h('p', { class: 'muted capacity-finding__found', title: dateTime(f.found_at) }, `Found ${relTime(f.found_at)}`),
        manage ? h('div', { class: 'health-actions' }, f.dismissed
          ? button({ size: 'sm', type: 'button', onClick: (e) => bringBack(e.currentTarget, f) }, icon('refresh', 13), 'Bring back')
          : button({ size: 'sm', type: 'button', onClick: () => askDismiss(f, describe(f), () => dv.load()) }, icon('check', 13), 'Dismiss…')) : null))) });
  }

  function busiest(d) {
    const tile = (label, m) => statTile({ label, value: m ? plural(m.streams, 'stream') : '–',
      hint: m ? `${m.transcodes ? `${plural(m.transcodes, 'transcode')}, ` : ''}${relTime(m.at)}` : `Nothing played in the last ${num(d.recent_days)} days` });
    const rows = (d.busiest || []).map((m) => h('tr', null,
      h('td', { class: 'nowrap' }, h('time', { datetime: new Date(m.at * 1000).toISOString(), title: relTime(m.at) }, dateTime(m.at))),
      h('td', { class: 'mono r' }, num(m.streams)),
      h('td', { class: 'mono r' }, num(m.transcodes)),
      h('td', { class: 'mono r' }, num(m.hardware)),
      h('td', { class: 'mono r' }, num(m.software)),
      h('td', { class: 'mono r' }, num(m.unsaid))));
    return card({ title: 'Busiest moments', sub: 'The most streams at once FinStats has seen, and how many were transcoded. It cannot know what your server could do; this is what it has done.', id: 'capacity-busiest',
      body: h('div', { class: 'stack' },
        h('div', { class: 'fui-stat-tile__grid' }, tile('Busiest on record', (d.busiest || [])[0]), tile(`Busiest in ${num(d.recent_days)} days`, d.recent)),
        rows.length ? plainTable(h('table', { class: 'fui-data-table' },
          h('thead', null, h('tr', null, h('th', null, 'When'), h('th', { class: 'r' }, 'Streams'), h('th', { class: 'r' }, 'Transcoded'),
            h('th', { class: 'r', title: 'Jellyfin said it used hardware acceleration' }, 'In hardware'), h('th', { class: 'r', title: 'Jellyfin said it used none' }, 'In software'),
            h('th', { class: 'r', title: 'An older Jellyfin, or an imported play, says neither' }, 'Not said'))),
          h('tbody', null, rows))) : emptyState('Nothing played yet', 'The busiest moments appear once something has been played.')) });
  }

  function disks(d) {
    const window = `the last ${num(d.window_months)} months`;
    if (!d.free_space_reported) {
      const rows = (d.growth || []).map((g) => h('tr', null, h('td', null, g.library), h('td', { class: 'mono r' }, bytes(g.added_bytes)), h('td', { class: 'mono r' }, bytes(g.added_bytes / d.window_months))));
      return card({ title: 'Disk space', sub: `What was added to each library over ${window}`, id: 'capacity-disks',
        body: [h('p', { class: 'fui-field__help' }, 'Jellyfin 10.11 and newer report how much space is free; this server does not, so only how fast each library grows is shown, not how long the space will last.'),
          rows.length ? plainTable(h('table', { class: 'fui-data-table' }, h('thead', null, h('tr', null, h('th', null, 'Library'), h('th', { class: 'r' }, `Added in ${num(d.window_months)} months`), h('th', { class: 'r' }, 'A month'))), h('tbody', null, rows))) : null] });
    }
    const said = (k) => (k.added_per_month > 0
      ? `${and(k.libraries)} grew ${bytes(k.added_per_month)} a month over ${window}; at that pace ${months(k.months_left)} of space are left.`
      : `Nothing was added to ${and(k.libraries)} over ${window}.`);
    return card({ title: 'Disk space', sub: `Each disk the libraries are on, and how fast what Jellyfin added over ${window} fills it`, id: 'capacity-disks',
      body: h('div', { class: 'stack' },
        h('div', { class: 'stack-sm' }, d.disks.map((k) => h('div', { class: 'capacity-disk' },
          h('p', null, said(k)),
          facts([['Folders', k.folders.join(', '), { wide: true }], ['Free', bytes(k.free_bytes), { mono: true }], ['Added a month', bytes(k.added_per_month), { mono: true }]])))),
        h('p', { class: 'fui-field__help' }, 'Growth is counted by the date Jellyfin added each file, so a replaced file counts as new and one deleted since is not counted.')) });
  }

  const dv = dataView({
    container: view, signal: ctx.signal,
    skeleton: () => [sk.cardBlock(200), sk.cardBlock(160)],
    fetch: () => loadCapacity(ctx.signal),
    render: (d) => {
      if (!d) return emptyState('Not worked out yet', 'FinStats works this out after its next read of the server’s details, within a quarter of an hour, and after each library read. Nothing is worked out when this page opens.');
      return [findings(d.findings || []), busiest(d), disks(d)];
    },
  });
  dv.load();
  return [view];
}
