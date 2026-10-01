// /server — what the Jellyfin server itself looks like (GET /api/server, see_server). One section on
// screen at a time, like Settings: Overview (system and storage), Jobs (live), Devices, Plugins.

import { h, icon, num, bytes, duration, relEl, dateTime, debounce, mount, untilText } from '../dom.js';
import { api, isAbort } from '../api.js';
import { state } from '../state.js';
import { pageHeader, card, dataView, sk, emptyState, facts, setBusy, inlineError } from '../components.js';
import { plainTable } from '../tables.js';
import { reveal, pickSection, sectionNav, sectionLayout } from '../sections.js';
import { logView, prefetchEvents } from './events.js';
import { auditView, prefetchAudit } from './audit.js';
import { healthView, prefetchHealth } from './health.js';
import { isAdmin } from '../state.js';
import { button } from '../../finui/components/button/button.js';

const RESULT = {
  Completed: ['sev-good', 'check', 'Completed'],
  Failed: ['sev-critical', 'alert', 'Failed'],
  Aborted: ['sev-warning', 'alert', 'Aborted'],
  Cancelled: ['sev-warning', 'alert', 'Cancelled'],
};

// Shared with the prefetcher, so a prefetched view has exactly the address the page asks for.
const loadServer = (signal) => api.get('/server', null, { signal });
export const prefetchServer = (c) => {
  const section = c.params && c.params.section;
  if (section === 'log') return prefetchEvents(c);
  if (section === 'audit') return prefetchAudit(c);
  if (section === 'health') return prefetchHealth(c);
  return [() => loadServer(c.signal)];
};

const SECTIONS = [
  { key: 'overview', label: 'Overview', sub: 'Version, system and storage', icon: 'server' },
  { key: 'jobs', label: 'Jobs', sub: 'What Jellyfin is doing, live', icon: 'clock' },
  { key: 'devices', label: 'Devices', sub: 'Every device that has signed in', icon: 'monitor' },
  { key: 'plugins', label: 'Plugins', sub: 'What is installed on Jellyfin', icon: 'layers' },
  // Not the image's HEALTHCHECK: what is wrong with the files, found by comparing them with their neighbours.
  { key: 'health', label: 'Library health', sub: 'Holes in seasons, copies, thin files and seasons that differ, found by comparing each file with its neighbours', icon: 'gauge' },
  { key: 'log', label: 'Log', sub: 'Jellyfin’s own activity log: sign-ins, failed logins, playback, tasks', icon: 'log' },
  // finstats' own doings, for administrators: it names who changed what.
  { key: 'audit', label: 'Audit', sub: 'What changed in finstats, and who did it', icon: 'shield', visible: () => isAdmin() },
];
// The one-page card anchors, so a link from before still lands.
const LEGACY = { jobs: 'jobs' };

export default function serverPage(ctx) {
  const visible = SECTIONS.filter((s) => !s.visible || s.visible());
  const section = pickSection(ctx, '/server', visible, LEGACY);
  if (!section) return;
  ctx.title(`${section.label} · Server`);
  const headerSlot = h('div', null, pageHeader('Server', section.sub));
  const view = h('div', { class: 'section-body stack' });
  let deviceFilter = '';

  const dv = dataView({
    container: view, signal: ctx.signal,
    skeleton: () => [sk.cardBlock(120), h('div', { class: 'grid-2' }, sk.cardRows(4), sk.cardRows(4)), sk.cardBlock(200)],
    fetch: () => loadServer(ctx.signal),
    render,
  });

  function paintHeader(d) {
    d = d || {};
    const info = d.info || null;
    const name = (info && info.server_name) || (state.status && state.status.server_name) || 'Jellyfin';
    const chips = [
      info && info.has_update_available ? h('span', { class: 'fui-badge--status fui-badge--info fui-badge--outlined' }, icon('arrowUp', 13), 'Update available') : null,
      info && info.has_pending_restart ? h('span', { class: 'fui-badge--status fui-badge--warning fui-badge--outlined' }, icon('refresh', 13), 'Restart pending') : null,
    ].filter(Boolean);
    headerSlot.replaceChildren(pageHeader(name,
      [info && info.version ? `Jellyfin ${info.version}` : 'Your Jellyfin server', d.fetched_at ? ['· details from ', relEl(d.fetched_at, '')] : null],
      chips.length ? h('div', { class: 'fui-chip__set' }, chips) : null));
  }

  function render(d) {
    d = d || {};
    const info = d.info || null;
    paintHeader(d);
    // Jobs are live and their own request: they are worth nothing a quarter of an hour old, so the
    // section stands even before the rest has ever been fetched.
    if (section.key === 'jobs') return jobsCard();
    if (!d.fetched_at) return fetchPrompt();
    if (section.key === 'devices') return devicesCard(d.devices);
    if (section.key === 'plugins') return pluginsCard(d.plugins);
    return [
      info ? card({ title: 'System', id: 'system', body: facts([
        ['Version', info.version, { mono: true }],
        ['Operating system', [info.operating_system, info.architecture].filter(Boolean).join(' · ') || null],
        ['Transcoder', info.encoder_location],
        info.transcoding_temp_path ? ['Transcode folder', h('span', { class: 'mono path' }, info.transcoding_temp_path), { wide: true }] : null,
        info.cache_path ? ['Cache folder', h('span', { class: 'mono path' }, info.cache_path), { wide: true }] : null,
        info.program_data_path ? ['Data folder', h('span', { class: 'mono path' }, info.program_data_path), { wide: true }] : null,
        info.log_path ? ['Log folder', h('span', { class: 'mono path' }, info.log_path), { wide: true }] : null,
      ]) }) : null,
      storageCard(d.storage),
    ];
  }

  // ---- nothing fetched yet
  function fetchPrompt() {
    const err = h('div');
    const btn = button({ variant: 'primary', type: 'button' }, icon('refresh', 14), 'Fetch server details');
    btn.addEventListener('click', async () => {
      mount(err);
      setBusy(btn, true, 'Fetching…');
      try {
        await api.post('/tasks/sync_server/run', {}, { signal: ctx.signal });
      } catch (e) {
        if (isAbort(e) || e.status === 401) return;
        if (e.status !== 409) { setBusy(btn, false); mount(err, inlineError('server-fetch-err', e.message)); return; } // 409 = already running: just wait for it
      }
      const deadline = Date.now() + 30000;
      const poll = async () => {
        if (ctx.signal.aborted) return;
        try {
          const d = await api.get('/server', null, { signal: ctx.signal });
          if (d && d.fetched_at) { dv.load(); return; }
        } catch (e) { if (isAbort(e) || e.status === 401) return; }
        if (Date.now() > deadline) {
          setBusy(btn, false);
          mount(err, inlineError('server-fetch-err', 'Jellyfin hasn’t answered yet. Check Settings → Tasks for the reason, then try again.'));
          return;
        }
        timer = setTimeout(poll, 2000);
      };
      timer = setTimeout(poll, 2000);
    });
    return emptyState('No server details yet', 'finstats fetches version, storage, plugins, scheduled tasks and devices from Jellyfin on its regular sync. You can also fetch them now.',
      h('div', { class: 'fui-empty__action' }, btn, err));
  }
  let timer = null;
  ctx.onCleanup(() => clearTimeout(timer));

  // ---- storage
  function storageCard(storage) {
    const rows = (Array.isArray(storage) ? storage : []).filter((x) => x && x.used_bytes >= 0 && x.free_bytes >= 0 && x.used_bytes + x.free_bytes > 0);
    if (!rows.length) {
      return card({ title: 'Storage', id: 'storage', body: h('p', { class: 'fui-field__help' }, 'This Jellyfin version doesn’t report disk usage (it arrived in 10.11).') });
    }
    const order = { library: 0, system: 1 };
    rows.sort((a, b) => (order[a.kind] ?? 2) - (order[b.kind] ?? 2));
    return card({ title: 'Storage', id: 'storage', sub: 'Disk usage of the volumes Jellyfin lives on. Entries with identical numbers share a disk.',
      body: h('ul', { class: 'storage' }, rows.map((x) => {
        const total = x.used_bytes + x.free_bytes, share = x.used_bytes / total;
        // Severity lives in the fill; the track is a lighter step of the same hue.
        const level = share >= 0.95 ? 'is-critical' : share >= 0.85 ? 'is-warning' : '';
        return h('li', { class: 'storage-row' },
          h('div', { class: 'storage-head' },
            h('span', { class: 'storage-label' }, x.label || 'Volume', x.kind === 'library' ? h('span', { class: 'fui-chip' }, 'Library') : null),
            h('span', { class: 'mono storage-nums' }, `${bytes(x.free_bytes)} free of ${bytes(total)}`)),
          h('div', { class: ['fui-meter fui-meter--block storage-meter', level], role: 'progressbar', 'aria-label': `${x.label || 'Volume'} disk usage`, 'aria-valuemin': 0, 'aria-valuemax': 100, 'aria-valuenow': Math.round(share * 100) },
            h('span', { class: 'fui-meter__fill', style: { width: Math.max(1, share * 100) + '%' } })),
          h('div', { class: 'storage-foot' }, x.path ? h('span', { class: 'mono path' }, x.path) : h('span'),
            level ? h('span', { class: ['fui-badge--status', level === 'is-critical' ? 'fui-badge--critical' : 'fui-badge--warning'] }, icon('alert', 13), level === 'is-critical' ? 'Almost full' : 'Getting full')
                  : h('span', { class: 'mono muted' }, Math.round(share * 100) + '% used')));
      })) });
  }

  // ---- devices (client-side filter; the list is small)
  function devicesCard(devices) {
    const all = Array.isArray(devices) ? devices : [];
    const body = h('div');
    const count = h('span', { class: 'mono muted' });
    const paint = () => {
      const q = deviceFilter.trim().toLowerCase();
      const rows = q ? all.filter((d) => [d.name, d.app, d.app_version, d.last_user_name].some((v) => v && String(v).toLowerCase().includes(q))) : all;
      count.textContent = q ? `${num(rows.length)} of ${num(all.length)}` : num(all.length);
      if (!all.length) return mount(body, emptyState('No devices known yet.'));
      if (!rows.length) return mount(body, emptyState('No devices match', 'Try a device, app or user name.'));
      mount(body, plainTable(h('table', { class: 'table' },
        h('thead', null, h('tr', null, h('th', null, 'Device'), h('th', null, 'App'), h('th', null, 'Last user'), h('th', null, 'Last seen'))),
        h('tbody', null, rows.map((d) => h('tr', null,
          h('td', { class: 'wrap-cell' }, d.name || 'Unknown device'),
          h('td', null, h('div', { class: 'client-cell' }, h('span', null, d.app || '–'), h('span', { class: 'muted mono' }, d.app_version || ''))),
          h('td', null, d.last_user_id ? h('a', { href: `/users/${d.last_user_id}` }, d.last_user_name || 'Unknown user') : d.last_user_name || h('span', { class: 'muted' }, '–')),
          h('td', null, d.last_seen ? relEl(d.last_seen) : h('span', { class: 'muted' }, '–'))))))));
    };
    const onInput = debounce((v) => { deviceFilter = v; paint(); }, 150);
    ctx.onCleanup(() => onInput.cancel());
    const input = h('input', { class: 'fui-field__input fui-field__input--search', type: 'search', placeholder: 'Filter devices…', 'aria-label': 'Filter devices', value: deviceFilter, spellcheck: false,
      onInput: (e) => onInput(e.target.value) });
    paint();
    return card({ title: ['Devices ', count], id: 'devices', sub: 'Every device that has signed in to Jellyfin',
      actions: all.length > 5 ? h('div', { class: 'fui-field__search fui-field__search--sm' }, icon('search', 14), input) : null, cls: 'fui-card--flush', body });
  }

  function pluginsCard(plugins) {
    const rows = Array.isArray(plugins) ? plugins : [];
    return card({ title: 'Plugins', id: 'plugins', cls: 'fui-card--flush', body: !rows.length ? emptyState('No plugins reported.') :
      plainTable(h('table', { class: 'table' },
        h('thead', null, h('tr', null, h('th', null, 'Plugin'), h('th', null, 'Version'), h('th', null, 'Status'))),
        h('tbody', null, rows.map((p) => {
          const ok = !p.status || p.status === 'Active';
          return h('tr', null, h('td', { title: p.description || null }, p.name || '–'), h('td', { class: 'mono' }, p.version || '–'),
            h('td', null, h('span', { class: ['fui-badge--status', ok ? 'fui-badge--good' : 'fui-badge--warning'] }, icon(ok ? 'check' : 'alert', 13), p.status || 'Active')));
        })))) });
  }

  // ---- Jellyfin's own jobs, live. Its scheduled tasks say what the code is called ("Detect and Analyze
  // Media Segments"); this says what they are doing to your server, how far along they are and how much
  // is left. The list is its own request, because it is worth nothing if it is a quarter of an hour old.
  const jobsSlot = h('div', { class: 'net-stack' }, sk.rows(3));
  // Built once and kept: the card is redrawn by the page's own refresh, the badge by the jobs poll.
  const jobsCount = h('span', null, '');
  const jobsBadge = h('span', { class: 'fui-badge fui-badge--live', hidden: true }, h('span', { class: 'fui-badge__dot' }), jobsCount);
  let jobsData = null, jobsAt = 0, jobsErr = null;

  async function loadJobs() {
    try {
      jobsData = await api.get('/jellyfin/jobs', null, { signal: ctx.signal });
      jobsErr = null;
    } catch (e) {
      if (isAbort(e) || e.status === 401) return;
      jobsErr = e.message;
    }
    jobsAt = Date.now();
    renderJobs();
  }

  function jobsCard() {
    return card({
      title: 'Jellyfin’s jobs',
      sub: 'What your server does in the background, what each one is for, and how far along it is',
      id: 'jobs',
      actions: jobsBadge,
      body: jobsSlot,
    });
  }

  /** "about 4 minutes left" when finstats has watched the percentage move — and, when it has not, the same
      sentence with three cycling dots where the time will go: "about . left", "about .. left", "about ... left".
      The words around them are what make the dots mean something: on their own they read as a page still
      loading. A guess put where an estimate goes would read exactly like an estimate, so there is no number. */
  function leftText(job) {
    if (job.eta_s == null) {
      return h('span', { class: 'job-working' },
        h('span', { 'aria-hidden': 'true' }, 'about'),
        h('span', { class: 'dots', 'aria-hidden': 'true' }, h('span', null, '.'), h('span', null, '.'), h('span', null, '.')),
        h('span', { 'aria-hidden': 'true' }, 'left'),
        h('span', { class: 'sr-only' }, 'Working; no estimate yet'));
    }
    if (job.eta_s <= 5) return h('span', null, 'finishing');
    return h('span', null, 'about ', h('strong', null, duration(job.eta_s)), ' left');
  }

  /** A percentage that has not moved for a while: a slow job, said out loud, so it does not read as a
      stuck page. Jellyfin only reports progress when the job bothers to, and some barely do. */
  function stillText(job) {
    if (!job.unchanged_for_s || job.unchanged_for_s < 30) return null;
    return h('span', { class: 'muted' }, `· ${Math.round(job.progress || 0)}% for ${duration(job.unchanged_for_s)}`);
  }

  /** How long finstats has been watching — which is how young the estimate is. Left out when the job has
      stood still for that whole time, because then the line above has already said it. */
  function watchedText(job) {
    if (!job.watching_since) return null;
    const watched = Math.max(1, Math.floor(Date.now() / 1000) - job.watching_since);
    if (job.unchanged_for_s >= 30 && watched - job.unchanged_for_s <= 5) return null;
    return h('span', { class: 'muted' }, ['· watched for ', duration(watched)]);
  }

  function runningJob(job) {
    const pct = Math.round(job.progress || 0);
    return h('div', { class: 'job' },
      h('div', { class: 'job-head' },
        h('strong', null, job.name),
        h('span', { class: 'fui-badge fui-badge--live' }, h('span', { class: 'fui-badge__dot' }), job.state === 'Cancelling' ? 'Stopping' : 'Running'),
        h('span', { class: 'job-pct mono' }, `${pct}%`)),
      h('div', { class: 'fui-meter fui-meter--wide', role: 'progressbar', 'aria-label': `${job.name} progress`, 'aria-valuemin': 0, 'aria-valuemax': 100, 'aria-valuenow': pct },
        h('span', { class: 'fui-meter__fill', style: { width: `${Math.max(pct, 1)}%` } })),
      h('div', { class: 'job-meta' }, leftText(job), stillText(job), watchedText(job),
        job.last_duration_s ? h('span', { class: 'muted' }, ['· last time it took ', duration(job.last_duration_s)]) : null),
      h('p', { class: 'fui-field__help job-what' }, job.what));
  }

  function jobRow(job) {
    const [cls, ic, label] = RESULT[job.last_result] || ['sev-info', 'info', job.last_result || null];
    const when = job.schedule && job.schedule.length ? job.schedule.join(', ') : 'only when something asks for it';
    return h('tr', null,
      h('td', null, h('div', { class: 'job-cell' },
        h('span', null, job.name, job.hidden ? h('span', { class: 'fui-chip' }, 'hidden') : null),
        h('span', { class: 'cell-sub job-what' }, job.what))),
      h('td', null, h('div', { class: 'job-cell' },
        h('span', null, when),
        job.next_at ? h('span', { class: 'cell-sub' }, `next ${untilText(job.next_at)}`) : null)),
      h('td', null, h('div', { class: 'job-cell' },
        job.last_run_at ? relEl(job.last_run_at) : h('span', { class: 'muted' }, 'never run'),
        label ? h('span', { class: 'cell-sub' }, h('span', { class: 'fui-badge--status ' + cls }, icon(ic, 12), label, job.last_duration_s != null ? ` · ${duration(job.last_duration_s)}` : '')) : null)));
  }

  function renderJobs() {
    if (jobsErr && !jobsData) return mount(jobsSlot, inlineError('jobs-err', jobsErr));
    const jobs = (jobsData && jobsData.jobs) || [];
    const running = jobs.filter((j) => j.running);
    const idle = jobs.filter((j) => !j.running);
    mount(jobsSlot,
      jobsData && jobsData.error ? h('p', { class: 'fui-field__help' }, `Jellyfin did not answer just now (${jobsData.error}); this is the last thing it said.`) : null,
      running.length ? h('div', { class: 'job-list' }, running.map(runningJob)) : h('p', { class: 'fui-field__help' }, 'Nothing is running on Jellyfin right now.'),
      idle.length
        ? plainTable(h('table', { class: 'table jobs-table' },
          h('thead', null, h('tr', null, h('th', null, 'Job'), h('th', null, 'Runs'), h('th', null, 'Last run'))),
          h('tbody', null, idle.map(jobRow))))
        : null);
    const want = jobsData ? jobsData.running : 0;
    jobsCount.textContent = `${num(want)} running`;
    jobsBadge.hidden = !want;
  }

  ctx.root.append(headerSlot, sectionLayout(sectionNav('/server', visible, section.key, 'Server sections'), view));
  if (section.key === 'log' || section.key === 'audit' || section.key === 'health') {
    // The logs and Library health are their own requests with their own filters; the server details only name the page.
    view.append(...(section.key === 'log' ? logView(ctx) : section.key === 'audit' ? auditView(ctx) : healthView(ctx)));
    loadServer(ctx.signal).then((d) => { if (!ctx.signal.aborted) paintHeader(d); }).catch(() => {});
  } else dv.load();
  if (section.key === 'jobs') {
    loadJobs();
    // One timer: every three seconds while Jellyfin is busy (a percentage that only moves every quarter of
    // an hour is not a progress bar), every twenty when it is not.
    ctx.every(() => {
      const busy = jobsData && jobsData.running > 0;
      if (busy || Date.now() - jobsAt >= 20000) loadJobs();
      else renderJobs();               // the "watched for" clock still ticks
    }, 3000, { visibleOnly: true });
  }
  const id = location.hash.slice(1);
  if (id) requestAnimationFrame(() => reveal(id));
}
