// Settings → Tasks: every job FinStats does by itself, when it last ran, and (a click on a row) when it runs,
// laid out the way Jellyfin lays out its scheduled tasks: a list, then a job's own page with its triggers and an
// "Add trigger" dialog. `/settings/tasks` is the list, `/settings/tasks/:task` one job. The server holds every
// rule (`schedule.rs`); this page only says it in words and sends the whole list back when it changes.

import { h, icon, mount, dateTime } from '../dom.js';
import { api } from '../api.js';
import { can, state } from '../state.js';
import { card, sk, spinner, inlineError, errorState, openModal, setBusy } from '../components.js';
import { progressOf } from './common.js';
import { button } from '../../finui/components/button/button.js';

/** What each job is called and does, and the group it is listed under. */
export const TASK_LABEL = {
  sync_users: ['Sync users', 'Names, roles and last-seen times from Jellyfin', 'Jellyfin'],
  sync_libraries: ['Read libraries and items', 'Titles and file details from Jellyfin. Read-only: it never starts a scan there', 'Jellyfin'],
  sync_changes: ['Metadata changes', 'Titles, cast and crew, posters, portraits and file details changed in Jellyfin since the last look', 'Jellyfin'],
  sync_userdata: ['Watched & favourites', 'Per-user played flags and favourites from Jellyfin', 'Jellyfin'],
  sync_events: ['Sync server log', 'Jellyfin’s activity log: sign-ins, failed logins, tasks', 'Jellyfin'],
  sync_server: ['Server details', 'Version, storage, plugins, scheduled tasks and devices', 'Jellyfin'],
  sync_upcoming: ['Read Sonarr and Radarr calendars', 'What is about to air or be released. Read-only', 'Sonarr, Radarr and Seerr'],
  sync_grabs: ['Read the download history', 'What Sonarr and Radarr grabbed, imported or failed. Read-only', 'Sonarr, Radarr and Seerr'],
  sync_requests: ['Read requests from Seerr', 'Who asked for what, and how far it has got. Read-only', 'Sonarr, Radarr and Seerr'],
  backup: ['Backup', 'Writes a FinStats backup; the newest ones are kept, as set under Backups', 'FinStats'],
  geoip: ['Geolocation database', 'Downloads DB-IP’s free city file when a newer month is out (about 60 MB). Places addresses on the Security page', 'FinStats'],
  import: ['Jellystat import', 'Runs when you upload a Jellystat backup under Import', 'FinStats'],
  import_streamystats: ['Streamystats import', 'Runs when you upload a Streamystats backup under Import', 'FinStats'],
  import_tautulli: ['Tautulli import', 'Runs when the wires of an uploaded Tautulli backup are connected under Import', 'FinStats'],
  restore: ['Restore', 'Runs when you restore a FinStats backup under Backups', 'FinStats'],
};
const GROUPS = ['Jellyfin', 'Sonarr, Radarr and Seerr', 'FinStats'];
// Where a job that runs on an upload is started instead.
const STARTED_FROM = { import: ['/settings/import', 'Import'], import_streamystats: ['/settings/import', 'Import'], import_tautulli: ['/settings/import', 'Import'], restore: ['/settings/backups#restore', 'Backups'] };
const SERVICE_TASKS = { sync_upcoming: 'upcoming' };   // task → the feature it belongs to
const DAYS = ['Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday'];
const INTERVALS = [300, 900, 1800, 2700, 3600, 7200, 10800, 14400, 21600, 28800, 43200, 86400, 2 * 86400, 3 * 86400, 7 * 86400, 14 * 86400, 30 * 86400];

const nameOf = (id) => (TASK_LABEL[id] || [id])[0];

// ---------------------------------------------------------------- words
const plural = (n, one, many = one + 's') => `${n} ${n === 1 ? one : many}`;
/** A length of time as a sentence says it: "6 minutes", "2 hours", "under a minute". */
export function spoken(sec) {
  sec = Math.max(0, Math.round(sec));
  if (sec < 60) return 'under a minute';
  if (sec < 3600) return plural(Math.round(sec / 60), 'minute');
  if (sec < 86400 * 2) return plural(Math.round(sec / 3600), 'hour');
  return plural(Math.round(sec / 86400), 'day');
}
const ago = (ts) => (Date.now() / 1000 - ts < 60 ? 'less than a minute ago' : `${spoken(Date.now() / 1000 - ts)} ago`);
/** "Every 15 minutes", "Every hour", "Every 7 days". */
export function every(sec) {
  if (sec % 86400 === 0) return sec === 86400 ? 'day' : `${sec / 86400} days`;
  if (sec % 3600 === 0) return sec === 3600 ? 'hour' : `${sec / 3600} hours`;
  return plural(Math.round(sec / 60), 'minute');
}
const timef = new Intl.DateTimeFormat(undefined, { hour: 'numeric', minute: '2-digit', timeZone: 'UTC' });
/** A minute of the day as the reader's clock writes it: "3:00 AM", "03:00". */
export const timeWords = (min) => timef.format(new Date(Date.UTC(2000, 0, 1, 0, min)));
/** One trigger in words, the way Jellyfin lists them. */
export function triggerWords(t) {
  switch (t.type) {
    case 'daily': return `Daily at ${timeWords(t.at_min)}`;
    case 'weekly': return `Every ${DAYS[t.day]} at ${timeWords(t.at_min)}`;
    case 'interval': return `Every ${every(t.every_s)}`;
    case 'startup': return 'On application startup';
    case 'after_scan': return 'After Jellyfin’s library scan';
    default: return t.type;
  }
}
const limitWords = (s) => (s % 3600 === 0 ? plural(s / 3600, 'hour') : plural(Math.round(s / 60), 'minute'));

/** How a job last went, in one sentence: "Last ran 38 minutes ago, taking 6 minutes." */
function lastRun(t) {
  if (t.state === 'running') return 'Running…';
  if (!t.finished_at) return 'Hasn’t run yet.';
  const took = t.started_at ? `, taking ${spoken(t.finished_at - t.started_at)}` : '';
  return t.state === 'error' ? `Failed ${ago(t.finished_at)}${took}.` : `Last ran ${ago(t.finished_at)}${took}.`;
}
function nextRun(t) {
  if (!t.schedulable) return null;
  if (!t.triggers.length) return 'Only when you run it';
  if (t.next_at) return `Next ${t.next_at - Date.now() / 1000 < 60 ? 'within a minute' : 'in ' + spoken(t.next_at - Date.now() / 1000)}`;
  return t.triggers.map(triggerWords).join(' · ');
}

/** Everything a row says about a job, to tell whether it needs painting again. */
const said = (t) => [t.id, t.state, lastRun(t), nextRun(t), t.message, t.progress, t.error, t.runnable];

// ---------------------------------------------------------------- running a job
const runBusy = new Set();
const runErr = {};
function runButton(t, store, paint) {
  if (!t.runnable) return null;
  const name = nameOf(t.id);
  const running = t.state === 'running' || runBusy.has(t.id);
  const btn = button({ variant: 'icon', class: 'task-run', type: 'button', 'aria-label': `Run ${name}`, title: running ? 'Running…' : 'Run now', disabled: running },
    running ? spinner(14) : icon('play', 14));
  btn.addEventListener('click', async (e) => {
    e.preventDefault(); e.stopPropagation();
    runBusy.add(t.id); delete runErr[t.id]; paint(true);
    try { await api.post(`/tasks/${t.id}/run`); }
    catch (err) { if (err.status !== 409) runErr[t.id] = err.message; }
    runBusy.delete(t.id);
    store.poke(1000);
  });
  return btn;
}

// ---------------------------------------------------------------- the list
function listView(slot, store) {
  const body = h('div', null, sk.rows(6));
  mount(slot, card({ title: 'Tasks', sub: 'What FinStats does by itself. Open one to change when it runs.', body, id: 'tasks' }));
  let sig = '';
  function paint(force = false) {
    if (!store.tasks) return;
    const feat = (state.user && state.user.features) || {};
    const tasks = (store.tasks.tasks || []).filter((t) => TASK_LABEL[t.id] && (!SERVICE_TASKS[t.id] || feat[SERVICE_TASKS[t.id]]));
    // What the rows say, not the raw answer: a job that is due has a `next_at` of "now", which moves every second.
    const now = JSON.stringify([tasks.map(said), [...runBusy], runErr]);
    if (!force && now === sig) return; // don't rebuild (and drop focus) when nothing changed
    sig = now;
    mount(body, GROUPS.map((g) => {
      const mine = tasks.filter((t) => TASK_LABEL[t.id][2] === g).sort((a, b) => Object.keys(TASK_LABEL).indexOf(a.id) - Object.keys(TASK_LABEL).indexOf(b.id));
      if (!mine.length) return null;
      return h('div', { class: 'task-group' }, h('h3', { class: 'task-group-title' }, g),
        h('ul', { class: 'task-list' }, mine.map((t) => {
          const [name, what] = TASK_LABEL[t.id];
          const next = nextRun(t);
          return h('li', { class: ['task-row', t.state === 'running' && 'is-running'], id: `task-${t.id}` },
            h('span', { class: ['task-icon', t.state === 'error' && 'is-failed'], 'aria-hidden': 'true' }, icon(t.state === 'error' ? 'alert' : 'clock', 18)),
            h('div', { class: 'task-main' },
              h('a', { class: 'task-link', href: `/settings/tasks/${t.id}` }, h('span', { class: 'task-name' }, name)),
              h('p', { class: 'task-when' }, lastRun(t), next ? h('span', { class: 'task-next' }, ` ${next}.`) : null),
              progressOf(t, name + ' progress'),
              runErr[t.id] ? inlineError('task-run-err-' + t.id, `Couldn’t start: ${runErr[t.id]}`) : null,
              h('p', { class: 'sr-only' }, what)),
            runButton(t, store, paint));
        })));
    }), h('p', { class: 'fui-field__help task-zone' }, `Times of day are in ${store.tasks.time_zone || 'FinStats’ time zone'}.`));
  }
  store.onTasks(() => paint());
  return store.loadTasks().then(() => paint(true));
}

// ---------------------------------------------------------------- one job
function taskView(slot, store, id) {
  const body = h('div', { class: 'stack' }, sk.rows(3));
  const [name, what] = TASK_LABEL[id] || [id, ''];
  mount(slot,
    h('a', { class: 'back-link', href: '/settings/tasks' }, icon('chevronLeft', 14), 'All tasks'),
    card({ title: name, sub: what, body, id: `task-${id}` }));
  let err = null, sig = '';

  async function save(triggers) {
    err = null;
    try { await api.put(`/tasks/${id}/triggers`, { triggers }); }
    catch (e) { err = e.message; }
    await store.poke(2000);
    paint(true);
  }

  function paint(force = false) {
    const t = store.task(id);
    if (!t) { mount(body, h('p', { class: 'fui-field__help' }, 'There is no such task.'), button({ href: '/settings/tasks' }, 'All tasks')); return; }
    const now = JSON.stringify([said(t), t.triggers, t.custom, t.next_at && dateTime(t.next_at), err, [...runBusy], runErr[id]]);
    if (!force && now === sig) return;
    sig = now;
    const status = h('div', { class: 'task-status' },
      h('p', { class: 'task-when' }, lastRun(t), t.finished_at ? h('span', { class: 'muted' }, ` (${dateTime(t.finished_at)})`) : null),
      progressOf(t, name + ' progress'),
      t.state === 'error' && t.error ? inlineError('task-err', t.error) : null,
      runErr[id] ? inlineError('task-run-err', `Couldn’t start: ${runErr[id]}`) : null);
    const run = t.runnable ? button({ type: 'button', disabled: t.state === 'running' || runBusy.has(id) }, icon('play', 12), t.state === 'running' ? 'Running…' : 'Run now') : null;
    if (run) run.addEventListener('click', () => runButton(t, store, paint).click());

    if (!t.schedulable) {
      const [href, where] = STARTED_FROM[id] || ['/settings', 'Settings'];
      mount(body, status, h('p', { class: 'fui-field__help' }, 'This one has no schedule: it needs a file. It runs when you upload one under ', h('a', { href }, where), '.'));
      return;
    }
    const add = button({ variant: 'primary', type: 'button' }, icon('plus', 13), 'Add trigger');
    add.addEventListener('click', () => addDialog(t, (trigger) => save([...t.triggers, trigger])));
    const reset = t.custom ? button({ variant: 'ghost', type: 'button' }, 'Back to the defaults') : null;
    if (reset) reset.addEventListener('click', async () => {
      setBusy(reset, true, 'Resetting…'); err = null;
      try { await api.del(`/tasks/${id}/triggers`); } catch (e) { err = e.message; }
      await store.poke(2000); paint(true);
    });
    const list = t.triggers.length
      ? h('ul', { class: 'trigger-list', id: 'task-triggers' }, t.triggers.map((g, i) => {
        const words = triggerWords(g);
        const remove = button({ variant: 'icon', class: 'trigger-remove', type: 'button', 'aria-label': `Remove “${words}”`, title: 'Remove' }, icon('minus', 14));
        remove.addEventListener('click', () => { remove.disabled = true; save(t.triggers.filter((_, j) => j !== i)); });
        return h('li', { class: 'trigger-row' }, h('span', { class: 'trigger-what' }, words),
          g.limit_s ? h('span', { class: 'trigger-limit muted' }, `stops after ${limitWords(g.limit_s)}`) : null, remove);
      }))
      : h('div', { id: 'task-triggers' }, h('p', { class: 'fui-field__help' }, 'No triggers: it runs only when you press Run now.'));
    mount(body, status,
      h('div', { class: 'form-actions task-actions' }, add, run, reset),
      h('div', { class: 'fui-field' }, h('div', { class: 'fui-field__label' }, 'Runs'), list, err ? inlineError('trigger-err', err) : null),
      h('p', { class: 'fui-field__help' }, `Times of day are in ${store.tasks.time_zone || 'FinStats’ time zone'}. `,
        t.next_at ? `Next run: ${dateTime(t.next_at)}.` : ''));
  }
  store.onTasks(() => paint());
  return store.loadTasks().then(() => paint(true));
}

/** Jellyfin's "Add Trigger" dialog: a type, what that type needs, and a time limit where a run can be stopped. */
function addDialog(t, onAdd) {
  const opt = (value, label, selected = false) => h('option', { value, selected }, label);
  const select = (id, label, options) => {
    const el = h('select', { class: 'fui-field__input', id }, options);
    return { el, field: h('div', { class: 'fui-field' }, h('label', { class: 'fui-field__label', htmlFor: id }, label), el) };
  };
  const type = select('trigger-type', 'Trigger type', [opt('daily', 'Daily'), opt('weekly', 'Weekly'), opt('interval', 'On an interval'), opt('startup', 'On application startup'),
    t.can.after_scan ? opt('after_scan', 'After Jellyfin’s library scan') : null]);
  const day = select('trigger-day', 'Day of week', DAYS.map((d, i) => opt(String(i), d)));
  const time = select('trigger-time', 'Time', Array.from({ length: 96 }, (_, i) => opt(String(i * 15), timeWords(i * 15), i * 15 === 180)));
  const every_ = select('trigger-every', 'Every', INTERVALS.map((s) => opt(String(s), every(s).replace(/^(day|hour)$/, '1 $1'), s === 3600)));
  const limit = t.can.limit ? h('input', { class: 'fui-field__input fui-field__input--num mono', id: 'trigger-limit', type: 'number', min: '0.25', max: '168', step: '0.25', inputMode: 'decimal', 'aria-describedby': 'trigger-limit-help' }) : null;
  const limitField = limit ? h('div', { class: 'fui-field' }, h('label', { class: 'fui-field__label', htmlFor: 'trigger-limit' }, 'Time limit (hours)'), limit,
    h('p', { class: 'fui-field__help', id: 'trigger-limit-help' }, 'A run that takes longer is stopped. Empty: no limit.')) : null;
  const formErr = h('div');
  const addBtn = button({ variant: 'primary', type: 'submit' }, 'Add');
  const cancel = button({ variant: 'ghost', type: 'button' }, 'Cancel');
  const form = h('form', { class: 'stack trigger-form', noValidate: true }, type.field, day.field, time.field, every_.field, limitField, formErr,
    h('div', { class: 'form-actions' }, cancel, addBtn));
  const show = () => {
    const k = type.el.value;
    day.field.hidden = k !== 'weekly';
    time.field.hidden = k !== 'daily' && k !== 'weekly';
    every_.field.hidden = k !== 'interval';
  };
  type.el.addEventListener('change', show);
  show();
  const modal = openModal({ title: 'Add trigger', body: form, initialFocus: type.el });
  cancel.addEventListener('click', () => modal.close());
  form.addEventListener('submit', (e) => {
    e.preventDefault(); mount(formErr, '');
    const k = type.el.value;
    const trigger = { type: k };
    if (k === 'daily' || k === 'weekly') trigger.at_min = Number(time.el.value);
    if (k === 'weekly') trigger.day = Number(day.el.value);
    if (k === 'interval') trigger.every_s = Number(every_.el.value);
    if (limit && limit.value.trim()) {
      const hours = Number(limit.value);
      if (!(hours > 0 && hours <= 168)) { mount(formErr, inlineError('trigger-limit-err', 'A time limit is between a quarter of an hour and 168 hours.')); limit.focus(); return; }
      trigger.limit_s = Math.round(hours * 3600);
    }
    if (t.triggers.some((x) => triggerWords(x) === triggerWords(trigger) && (x.limit_s || null) === (trigger.limit_s || null))) {
      mount(formErr, inlineError('trigger-dup-err', 'That trigger is already there.')); return;
    }
    modal.close();
    onAdd(trigger);
  });
}

export default {
  key: 'tasks', label: 'Tasks', sub: 'What FinStats does by itself, and when', group: 'Data', icon: 'clock',
  visible: () => can('manage'),
  entries: Object.entries(TASK_LABEL).map(([id, [label, what]]) => ({ id: `task-${id}`, label, hint: `${what} task job schedule trigger run` })),
  async render(slot, store) {
    const id = store.ctx.params.task;
    try { await (id ? taskView(slot, store, id) : listView(slot, store)); }
    catch (e) { mount(slot, errorState(e, () => this.render(slot, store))); }
  },
};
