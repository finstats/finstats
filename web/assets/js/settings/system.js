// Settings → System: what finstats is doing (its jobs), everywhere it can reach, what it holds, and
// what it is built on. Nothing here is a setting; it is the machine showing its work.

import { h, icon, num, bytes, relTime, dateTime, mount, humanize } from '../dom.js';
import { api, isAbort } from '../api.js';
import { isAdmin, state } from '../state.js';
import { card, sk, spinner, inlineError, errorState, facts } from '../components.js';
import { progressOf } from './common.js';

const SERVICE_TASKS = { sync_upcoming: 'upcoming' };   // task → the feature it belongs to
const TASK_LABEL = {
  sync_users: ['Sync users', 'Names, roles and last-seen times from Jellyfin'],
  sync_libraries: ['Read libraries and items', 'Titles and file details from Jellyfin. Read-only: it never starts a scan there'],
  sync_events: ['Sync server log', 'Jellyfin’s activity log: sign-ins, failed logins, tasks'],
  sync_server: ['Server details', 'Version, storage, plugins, scheduled tasks and devices'],
  sync_userdata: ['Watched & favourites', 'Per-user played flags and favourites from Jellyfin'],
  import: ['Jellystat import', 'Runs when you upload a Jellystat backup under Import'],
  import_streamystats: ['Streamystats import', 'Runs when you upload a Streamystats backup under Import'],
  backup: ['Backup', 'Writes a finstats backup on the schedule under Backups'],
  restore: ['Restore', 'Runs when you restore a finstats backup under Backups'],
  sync_upcoming: ['Read Sonarr and Radarr calendars', 'What is about to air or be released. Read-only, every 15 minutes'],
  geoip: ['Geolocation database', 'Downloads the city database the Security page places addresses with. Started under Security'],
};
const OWN_CARD = new Set(['import', 'import_streamystats', 'backup', 'restore']); // started from their own section, not with “Run now”
const STATE_LABEL = { always: ['is-always', 'Always on'], on: ['is-on', 'On'], off: ['', 'Off'] };

export default {
  key: 'system', label: 'System', sub: 'Jobs, outbound connections, the database', group: 'Data', icon: 'cpu',
  visible: () => true,
  entries: [
    { id: 'tasks', label: 'Tasks', hint: 'jobs run now sync users libraries server log failed' },
    { id: 'outbound', label: 'Outbound connections', hint: 'privacy what finstats reaches hosts telemetry' },
    { id: 'database', label: 'Database', hint: 'size on disk plays items oldest' },
    { id: 'licences', label: 'Licences', hint: 'gpl third-party open source fonts map' },
  ],
  async render(slot, store) {
    const tasksSlot = h('div', null, sk.rows(3));
    const outboundSlot = h('div', { class: 'net-stack' }, sk.rows(3));
    const dbSlot = h('div', null, sk.rows(1));
    mount(slot,
      card({ title: 'Tasks', sub: 'What finstats does on its own, and when it last did', body: tasksSlot, id: 'tasks' }),
      isAdmin() ? card({ title: 'Outbound connections', sub: 'Everywhere finstats can reach, and whether it is switched on', body: outboundSlot, id: 'outbound' }) : null,
      card({ title: 'Database', body: dbSlot, id: 'database' }),
      // finstats' own licence and every third-party one, on their own page: it is half a megabyte
      // of licence text, which belongs where somebody goes looking for it, not in a settings card.
      card({ title: 'Licences', sub: 'GNU GPL v3, on open-source Rust crates, two fonts and public-domain map data', id: 'licences',
        body: h('a', { class: 'btn', href: '/licenses' }, icon('log', 14), 'Third-party licences', icon('chevronRight', 14)) }));

    // ---- tasks
    const runBusy = new Set();
    const taskErr = {};
    let tasksSig = '';
    function paintTasks(force = false) {
      if (!store.tasks) return;
      // A task that reads from a service nobody has connected would only ever say so.
      const feat = (state.user && state.user.features) || {};
      const tasks = (store.tasks.tasks || []).filter((t) => !SERVICE_TASKS[t.id] || feat[SERVICE_TASKS[t.id]]);
      const sig = JSON.stringify([tasks, [...runBusy], taskErr]);
      if (!force && sig === tasksSig) return; // don't rebuild (and drop focus) when nothing changed
      tasksSig = sig;
      mount(tasksSlot, h('ul', { class: 'tasks' }, tasks.map((t) => {
        const [name, desc] = TASK_LABEL[t.id] || [humanize(String(t.id || 'task').replace(/^sync_/, 'Sync ')), ''];
        const running = t.state === 'running' || runBusy.has(t.id);
        const st = t.state === 'running' ? h('span', { class: 'sev sev-run' }, spinner(12), 'Running')
          : t.state === 'ok' ? h('span', { class: 'sev sev-good' }, icon('check', 13), 'Finished', t.finished_at ? h('span', { class: 'muted mono', title: dateTime(t.finished_at) }, ' ' + relTime(t.finished_at)) : null)
          : t.state === 'error' ? h('span', { class: 'sev sev-critical' }, icon('alert', 13), 'Failed', t.finished_at ? h('span', { class: 'muted mono' }, ' ' + relTime(t.finished_at)) : null)
          : h('span', { class: 'muted' }, 'Hasn’t run yet');
        let btn = null;
        if (!OWN_CARD.has(t.id)) {
          btn = h('button', { type: 'button', class: 'btn btn-sm' }, icon('play', 12), 'Run now');
          if (running) { btn.disabled = true; btn.replaceChildren(spinner(12), h('span', null, 'Running…')); }
          btn.addEventListener('click', async () => {
            runBusy.add(t.id); delete taskErr[t.id]; paintTasks();
            try { await api.post(`/tasks/${t.id}/run`); }
            catch (e) { if (e.status !== 409) taskErr[t.id] = e.message; }
            runBusy.delete(t.id);
            store.poke(2000);
          });
        }
        return h('li', { class: 'task' },
          h('div', { class: 'task-main' }, h('div', { class: 'task-name' }, name), h('div', { class: 'help' }, desc),
            progressOf(t, name + ' progress') || (t.message && t.state !== 'idle' ? h('div', { class: 'mono task-msg' }, t.message) : null),
            t.state === 'error' && t.error ? inlineError('task-err-' + t.id, t.error) : null,
            taskErr[t.id] ? inlineError('task-run-err-' + t.id, `Couldn’t start: ${taskErr[t.id]}`) : null),
          h('div', { class: 'task-side' }, st, btn));
      })));
    }
    function paintDb() {
      const d = store.tasks && store.tasks.db;
      if (!d) return;
      mount(dbSlot, facts([
        ['Size on disk', bytes(d.size_bytes), { mono: true }],
        ['Plays', num(d.plays), { mono: true }],
        ['Library items', num(d.items), { mono: true }],
        ['Oldest play', d.oldest_play_at ? dateTime(d.oldest_play_at) : '–', { mono: true }],
      ]));
    }

    // ---- outbound: the privacy promise in the README, assembled from what the running program
    // knows. Nothing is recorded for this list: every line is read from something finstats already kept.
    async function loadOutbound() {
      if (!isAdmin()) return;
      try {
        const data = await api.get('/outbound', null, { signal: store.signal });
        const rows = (data.destinations || []).map((d) => {
          const [cls, label] = STATE_LABEL[d.state] || STATE_LABEL.off;
          return h('div', { class: 'out-row' },
            h('div', { class: 'out-head' }, h('span', { class: 'out-what' }, d.what), h('span', { class: 'out-state ' + cls }, label)),
            d.hosts && d.hosts.length ? h('div', { class: 'out-hosts mono' }, d.hosts.join(', ')) : null,
            h('p', { class: 'help' }, d.why),
            d.last_at ? h('p', { class: 'help' }, ['Last answered ', h('span', { title: dateTime(d.last_at) }, relTime(d.last_at))]) : null,
            d.error ? h('p', { class: 'help' }, `Last attempt failed: ${d.error}`) : null);
        });
        const off = (data.total || 0) - (data.reachable || 0);
        mount(outboundSlot, rows,
          h('p', { class: 'help' }, `${num(data.reachable || 0)} of ${num(data.total || 0)} switched on${off ? `, ${num(off)} off` : ''}. finstats never sends anything about you or your server to any of these, and there is nothing else: no telemetry, no update check, no fonts or scripts from the internet.`));
      } catch (e) {
        if (isAbort(e) || e.status === 401 || e.status === 403) return;
        mount(outboundSlot, errorState(e, loadOutbound));
      }
    }

    store.onTasks(() => { paintTasks(); paintDb(); });
    loadOutbound();
    try { await store.loadTasks(); } catch (e) { mount(tasksSlot, errorState(e, () => store.loadTasks())); }
  },
};
