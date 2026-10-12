// /server: what the Jellyfin server itself looks like (GET /api/server, see_server). One section on
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
import { capacityView, prefetchCapacity } from './capacity.js';
import { isAdmin } from '../state.js';
import { button } from '../../finui/components/button/button.js';
import { t, tOr, tParts } from '../i18n.js';

const RESULT = {
  Completed: ['sev-good', 'check', t('pages.server.result.Completed')],
  Failed: ['sev-critical', 'alert', t('pages.server.result.Failed')],
  Aborted: ['sev-warning', 'alert', t('pages.server.result.Aborted')],
  Cancelled: ['sev-warning', 'alert', t('pages.server.result.Cancelled')],
};

// Jellyfin's PluginStatus values.
const PLUGIN_STATUS = ['Active', 'Restart', 'Deleted', 'Superceded', 'Malfunctioned', 'NotSupported', 'Disabled'];

// Shared with the prefetcher, so a prefetched view has exactly the address the page asks for.
const loadServer = (signal) => api.get('/server', null, { signal });
export const prefetchServer = (c) => {
  const section = c.params && c.params.section;
  if (section === 'log') return prefetchEvents(c);
  if (section === 'audit') return prefetchAudit(c);
  if (section === 'health') return prefetchHealth(c);
  if (section === 'capacity') return prefetchCapacity(c);
  return [() => loadServer(c.signal)];
};

const SECTIONS = [
  { key: 'overview', label: t('pages.server.sections.overview.label'), sub: t('pages.server.sections.overview.sub'), icon: 'server' },
  { key: 'jobs', label: t('pages.server.sections.jobs.label'), sub: t('pages.server.sections.jobs.sub'), icon: 'clock' },
  { key: 'devices', label: t('pages.server.sections.devices.label'), sub: t('pages.server.sections.devices.sub'), icon: 'monitor' },
  { key: 'plugins', label: t('pages.server.sections.plugins.label'), sub: t('pages.server.sections.plugins.sub'), icon: 'layers' },
  // Not the image's HEALTHCHECK: what is wrong with the files, found by comparing them with their neighbours.
  // Will it cope: the busiest moments on record, how fast the disks fill, and what that says about the server's settings.
  { key: 'capacity', label: t('pages.server.sections.capacity.label'), sub: t('pages.server.sections.capacity.sub'), icon: 'hardDrive' },
  { key: 'health', label: t('pages.server.sections.health.label'), sub: t('pages.server.sections.health.sub'), icon: 'gauge' },
  { key: 'log', label: t('pages.server.sections.log.label'), sub: t('pages.server.sections.log.sub'), icon: 'log' },
  // FinStats' own doings, for administrators: it names who changed what.
  { key: 'audit', label: t('pages.server.sections.audit.label'), sub: t('pages.server.sections.audit.sub'), icon: 'shield', visible: () => isAdmin() },
];
// The one-page card anchors, so a link from before still lands.
const LEGACY = { jobs: 'jobs' };

export default function serverPage(ctx) {
  const visible = SECTIONS.filter((s) => !s.visible || s.visible());
  const section = pickSection(ctx, '/server', visible, LEGACY);
  if (!section) return;
  ctx.title(`${section.label} · ${t('nav.server')}`);
  const headerSlot = h('div', null, pageHeader(t('nav.server'), section.sub));
  const view = h('div', { class: 'fui-sections__body stack' });
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
      info && info.has_update_available ? h('span', { class: 'fui-badge--status fui-badge--info fui-badge--outlined' }, icon('arrowUp', 13), t('pages.server.update_available')) : null,
      info && info.has_pending_restart ? h('span', { class: 'fui-badge--status fui-badge--warning fui-badge--outlined' }, icon('refresh', 13), t('pages.server.restart_pending')) : null,
    ].filter(Boolean);
    headerSlot.replaceChildren(pageHeader(name,
      [info && info.version ? `Jellyfin ${info.version}` : t('pages.server.your_server'), d.fetched_at ? [...tParts('pages.server.details_from', { when: relEl(d.fetched_at, '') })] : null],
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
      info ? card({ title: t('pages.server.system.title'), id: 'system', body: facts([
        [t('pages.server.system.version'), info.version, { mono: true }],
        [t('pages.server.system.os'), [info.operating_system, info.architecture].filter(Boolean).join(' · ') || null],
        [t('pages.server.system.transcoder'), info.encoder_location],
        info.transcoding_temp_path ? [t('pages.server.system.transcode_folder'), h('span', { class: 'mono path' }, info.transcoding_temp_path), { wide: true }] : null,
        info.cache_path ? [t('pages.server.system.cache_folder'), h('span', { class: 'mono path' }, info.cache_path), { wide: true }] : null,
        info.program_data_path ? [t('pages.server.system.data_folder'), h('span', { class: 'mono path' }, info.program_data_path), { wide: true }] : null,
        info.log_path ? [t('pages.server.system.log_folder'), h('span', { class: 'mono path' }, info.log_path), { wide: true }] : null,
      ]) }) : null,
      storageCard(d.storage),
    ];
  }

  // ---- nothing fetched yet
  function fetchPrompt() {
    const err = h('div');
    const btn = button({ variant: 'primary', type: 'button' }, icon('refresh', 14), t('pages.server.fetch.button'));
    btn.addEventListener('click', async () => {
      mount(err);
      setBusy(btn, true, t('pages.server.fetch.busy'));
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
          mount(err, inlineError('server-fetch-err', t('pages.server.fetch.timeout')));
          return;
        }
        timer = setTimeout(poll, 2000);
      };
      timer = setTimeout(poll, 2000);
    });
    return emptyState(t('pages.server.fetch.empty'), t('pages.server.fetch.empty_sub'),
      h('div', { class: 'fui-empty__action' }, btn, err));
  }
  let timer = null;
  ctx.onCleanup(() => clearTimeout(timer));

  // ---- storage
  // Jellyfin's own folders are named by FinStats (in English, as stored); a library's folder by the library's own name.
  const folderName = (x) => !x.label ? t('pages.server.storage.volume')
    : x.kind === 'system' ? tOr(`pages.server.storage.folder.${x.label.toLowerCase().replace(/\W+/g, '_')}`, x.label) : x.label;
  function storageCard(storage) {
    const rows = (Array.isArray(storage) ? storage : []).filter((x) => x && x.used_bytes >= 0 && x.free_bytes >= 0 && x.used_bytes + x.free_bytes > 0);
    if (!rows.length) {
      return card({ title: t('pages.server.storage.title'), id: 'storage', body: h('p', { class: 'fui-field__help' }, t('pages.server.storage.unreported')) });
    }
    const order = { library: 0, system: 1 };
    rows.sort((a, b) => (order[a.kind] ?? 2) - (order[b.kind] ?? 2));
    return card({ title: t('pages.server.storage.title'), id: 'storage', sub: t('pages.server.storage.sub'),
      body: h('ul', { class: 'storage' }, rows.map((x) => {
        const total = x.used_bytes + x.free_bytes, share = x.used_bytes / total;
        // Severity lives in the fill; the track is a lighter step of the same hue.
        const level = share >= 0.95 ? 'is-critical' : share >= 0.85 ? 'is-warning' : '';
        return h('li', { class: 'storage-row' },
          h('div', { class: 'storage-head' },
            h('span', { class: 'storage-label' }, folderName(x), x.kind === 'library' ? h('span', { class: 'fui-chip' }, t('pages.server.storage.library')) : null),
            h('span', { class: 'mono storage-nums' }, t('pages.server.storage.free_of', { free: bytes(x.free_bytes), total: bytes(total) }))),
          h('div', { class: ['fui-meter fui-meter--block storage-meter', level], role: 'progressbar', 'aria-label': t('pages.server.storage.usage_aria', { volume: folderName(x) }), 'aria-valuemin': 0, 'aria-valuemax': 100, 'aria-valuenow': Math.round(share * 100) },
            h('span', { class: 'fui-meter__fill', style: { width: Math.max(1, share * 100) + '%' } })),
          h('div', { class: 'storage-foot' }, x.path ? h('span', { class: 'mono path' }, x.path) : h('span'),
            level ? h('span', { class: ['fui-badge--status', level === 'is-critical' ? 'fui-badge--critical' : 'fui-badge--warning'] }, icon('alert', 13), level === 'is-critical' ? t('pages.server.storage.almost_full') : t('pages.server.storage.getting_full'))
                  : h('span', { class: 'mono muted' }, t('pages.server.storage.used', { pct: Math.round(share * 100) }))));
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
      count.textContent = q ? t('pages.server.devices.shown_of', { shown: rows.length, total: all.length }) : num(all.length);
      if (!all.length) return mount(body, emptyState(t('pages.server.devices.none')));
      if (!rows.length) return mount(body, emptyState(t('pages.server.devices.no_match'), t('pages.server.devices.no_match_sub')));
      mount(body, plainTable(h('table', { class: 'fui-data-table' },
        h('thead', null, h('tr', null, h('th', null, t('pages.server.devices.device')), h('th', null, t('pages.server.devices.app')), h('th', null, t('pages.server.devices.last_user')), h('th', null, t('pages.server.devices.last_seen')))),
        h('tbody', null, rows.map((d) => h('tr', null,
          h('td', { class: 'wrap-cell' }, d.name || t('pages.server.devices.unknown_device')),
          h('td', null, h('div', { class: 'client-cell' }, h('span', null, d.app || '–'), h('span', { class: 'muted mono' }, d.app_version || ''))),
          h('td', null, d.last_user_id ? h('a', { href: `/users/${d.last_user_id}` }, d.last_user_name || t('pages.server.devices.unknown_user')) : d.last_user_name || h('span', { class: 'muted' }, '–')),
          h('td', null, d.last_seen ? relEl(d.last_seen) : h('span', { class: 'muted' }, '–'))))))));
    };
    const onInput = debounce((v) => { deviceFilter = v; paint(); }, 150);
    ctx.onCleanup(() => onInput.cancel());
    const input = h('input', { class: 'fui-field__input fui-field__input--search', type: 'search', placeholder: t('pages.server.devices.filter_placeholder'), 'aria-label': t('pages.server.devices.filter_aria'), value: deviceFilter, spellcheck: false,
      onInput: (e) => onInput(e.target.value) });
    paint();
    return card({ title: [t('pages.server.devices.title'), ' ', count], id: 'devices', sub: t('pages.server.devices.sub'),
      actions: all.length > 5 ? h('div', { class: 'fui-field__search fui-field__search--sm' }, icon('search', 14), input) : null, cls: 'fui-card--flush', body });
  }

  function pluginsCard(plugins) {
    const rows = Array.isArray(plugins) ? plugins : [];
    return card({ title: t('pages.server.plugins.title'), id: 'plugins', cls: 'fui-card--flush', body: !rows.length ? emptyState(t('pages.server.plugins.none')) :
      plainTable(h('table', { class: 'fui-data-table' },
        h('thead', null, h('tr', null, h('th', null, t('pages.server.plugins.plugin')), h('th', null, t('pages.server.plugins.version')), h('th', null, t('pages.server.plugins.status')))),
        h('tbody', null, rows.map((p) => {
          const ok = !p.status || p.status === 'Active';
          return h('tr', null, h('td', { title: p.description || null }, p.name || '–'), h('td', { class: 'mono' }, p.version || '–'),
            h('td', null, h('span', { class: ['fui-badge--status', ok ? 'fui-badge--good' : 'fui-badge--warning'] }, icon(ok ? 'check' : 'alert', 13), PLUGIN_STATUS.includes(p.status || 'Active') ? t(`pages.server.plugins.state.${p.status || 'Active'}`) : p.status)));
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
      title: t('pages.server.jobs.title'),
      sub: t('pages.server.jobs.sub'),
      id: 'jobs',
      actions: jobsBadge,
      body: jobsSlot,
    });
  }

  /** "about 4 minutes left" when FinStats has watched the percentage move and, when it has not, the same
      sentence with three cycling dots where the time will go: "about . left", "about .. left", "about ... left".
      The words around them are what make the dots mean something: on their own they read as a page still
      loading. A guess put where an estimate goes would read exactly like an estimate, so there is no number. */
  function leftText(job) {
    if (job.eta_s == null) {
      const dots = h('span', { class: 'dots', 'aria-hidden': 'true' }, h('span', null, '.'), h('span', null, '.'), h('span', null, '.'));
      return h('span', { class: 'job-working' },
        ...tParts('pages.server.jobs.eta_dots', { dots }).map((p) => (typeof p === 'string' ? (p.trim() ? h('span', { 'aria-hidden': 'true' }, p.trim()) : null) : p)).filter(Boolean),
        h('span', { class: 'sr-only' }, t('pages.server.jobs.eta_none')));
    }
    if (job.eta_s <= 5) return h('span', null, t('pages.server.jobs.finishing'));
    return h('span', null, ...tParts('pages.server.jobs.eta', { time: h('strong', null, duration(job.eta_s)) }));
  }

  /** A percentage that has not moved for a while: a slow job, said out loud, so it does not read as a
      stuck page. Jellyfin only reports progress when the job bothers to, and some barely do. */
  function stillText(job) {
    if (!job.unchanged_for_s || job.unchanged_for_s < 30) return null;
    return h('span', { class: 'muted' }, t('pages.server.jobs.still', { pct: Math.round(job.progress || 0), time: duration(job.unchanged_for_s) }));
  }

  /** How long FinStats has been watching, which is how young the estimate is. Left out when the job has
      stood still for that whole time, because then the line above has already said it. */
  function watchedText(job) {
    if (!job.watching_since) return null;
    const watched = Math.max(1, Math.floor(Date.now() / 1000) - job.watching_since);
    if (job.unchanged_for_s >= 30 && watched - job.unchanged_for_s <= 5) return null;
    return h('span', { class: 'muted' }, t('pages.server.jobs.watched_for', { time: duration(watched) }));
  }

  function runningJob(job) {
    const pct = Math.round(job.progress || 0);
    return h('div', { class: 'job' },
      h('div', { class: 'job-head' },
        h('strong', null, job.name),
        h('span', { class: 'fui-badge fui-badge--live' }, h('span', { class: 'fui-badge__dot' }), job.state === 'Cancelling' ? t('pages.server.jobs.stopping') : t('pages.server.jobs.running')),
        h('span', { class: 'job-pct mono' }, t('pages.server.jobs.percent', { pct }))),
      h('div', { class: 'fui-meter fui-meter--wide', role: 'progressbar', 'aria-label': t('pages.server.jobs.progress_aria', { name: job.name }), 'aria-valuemin': 0, 'aria-valuemax': 100, 'aria-valuenow': pct },
        h('span', { class: 'fui-meter__fill', style: { width: `${Math.max(pct, 1)}%` } })),
      h('div', { class: 'job-meta' }, leftText(job), stillText(job), watchedText(job),
        job.last_duration_s ? h('span', { class: 'muted' }, t('pages.server.jobs.last_took', { time: duration(job.last_duration_s) })) : null),
      h('p', { class: 'fui-field__help job-what' }, job.what));
  }

  function jobRow(job) {
    const [cls, ic, label] = RESULT[job.last_result] || ['sev-info', 'info', job.last_result || null];
    const when = job.schedule && job.schedule.length ? job.schedule.join(', ') : t('pages.server.jobs.on_demand');
    return h('tr', null,
      h('td', null, h('div', { class: 'job-cell' },
        h('span', null, job.name, job.hidden ? h('span', { class: 'fui-chip' }, t('pages.server.jobs.hidden')) : null),
        h('span', { class: 'cell-sub job-what' }, job.what))),
      h('td', null, h('div', { class: 'job-cell' },
        h('span', null, when),
        job.next_at ? h('span', { class: 'cell-sub' }, t('pages.server.jobs.next', { when: untilText(job.next_at) })) : null)),
      h('td', null, h('div', { class: 'job-cell' },
        job.last_run_at ? relEl(job.last_run_at) : h('span', { class: 'muted' }, t('pages.server.jobs.never_run')),
        label ? h('span', { class: 'cell-sub' }, h('span', { class: 'fui-badge--status ' + cls }, icon(ic, 12), label, job.last_duration_s != null ? ` · ${duration(job.last_duration_s)}` : '')) : null)));
  }

  function renderJobs() {
    if (jobsErr && !jobsData) return mount(jobsSlot, inlineError('jobs-err', jobsErr));
    const jobs = (jobsData && jobsData.jobs) || [];
    const running = jobs.filter((j) => j.running);
    const idle = jobs.filter((j) => !j.running);
    mount(jobsSlot,
      jobsData && jobsData.error ? h('p', { class: 'fui-field__help' }, t('pages.server.jobs.stale', { error: jobsData.error })) : null,
      running.length ? h('div', { class: 'job-list' }, running.map(runningJob)) : h('p', { class: 'fui-field__help' }, t('pages.server.jobs.nothing_running')),
      idle.length
        ? plainTable(h('table', { class: 'fui-data-table jobs-table' },
          h('thead', null, h('tr', null, h('th', null, t('pages.server.jobs.job')), h('th', null, t('pages.server.jobs.runs')), h('th', null, t('pages.server.jobs.last_run')))),
          h('tbody', null, idle.map(jobRow))))
        : null);
    const want = jobsData ? jobsData.running : 0;
    jobsCount.textContent = t('pages.server.jobs.running_count', { count: want });
    jobsBadge.hidden = !want;
  }

  ctx.root.append(headerSlot, sectionLayout(sectionNav('/server', visible, section.key, t('pages.server.sections_aria')), view));
  const own = { log: logView, audit: auditView, health: healthView, capacity: capacityView }[section.key];
  if (own) {
    // The logs, Library health and Capacity are their own requests; the server details only name the page.
    view.append(...own(ctx));
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
