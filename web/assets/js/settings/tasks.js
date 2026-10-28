// Settings → Tasks: every job FinStats does by itself, when it last ran, and (a click on a row) when it runs,
// laid out the way Jellyfin lays out its scheduled tasks: a list, then a job's own page with its triggers and an
// "Add trigger" dialog. `/settings/tasks` is the list, `/settings/tasks/:task` one job. The server holds every
// rule (`schedule.rs`); this page only says it in words and sends the whole list back when it changes.

import { h, icon, mount, dateTime, clockTime, timeFormatChoice } from '../dom.js';
import { api } from '../api.js';
import { can, state, isAdmin } from '../state.js';
import { card, sk, spinner, inlineError, errorState, openModal, setBusy } from '../components.js';
import { progressOf } from './common.js';
import { button } from '../../finui/components/button/button.js';
import { t as tr, tParts, lang } from '../i18n.js';

/** What each job is called and does, and the group it is listed under. */
const TASK_GROUP = {
  sync_users: 'jellyfin', sync_libraries: 'jellyfin', sync_changes: 'jellyfin', sync_userdata: 'jellyfin', sync_events: 'jellyfin', sync_server: 'jellyfin',
  sync_upcoming: 'arr', sync_grabs: 'arr', sync_requests: 'arr',
  backup: 'finstats', geoip: 'finstats', import: 'finstats', import_streamystats: 'finstats', import_tautulli: 'finstats', restore: 'finstats',
};
export const TASK_LABEL = Object.fromEntries(Object.entries(TASK_GROUP).map(([id, g]) => [id, [tr(`settings.tasks.task.${id}.name`), tr(`settings.tasks.task.${id}.what`), g]]));
const GROUPS = ['jellyfin', 'arr', 'finstats'];
// Where a job that runs on an upload is started instead.
const STARTED_FROM = { import: ['/settings/import', tr('settings.sections.import.label')], import_streamystats: ['/settings/import', tr('settings.sections.import.label')], import_tautulli: ['/settings/import', tr('settings.sections.import.label')], restore: ['/settings/backups#restore', tr('settings.sections.backups.label')] };
const SERVICE_TASKS = { sync_upcoming: 'upcoming' };   // task → the feature it belongs to
const dayf = new Intl.DateTimeFormat(lang(), { weekday: 'long', timeZone: 'UTC' });
const DAYS = Array.from({ length: 7 }, (_, i) => dayf.format(new Date(Date.UTC(2000, 0, 2 + i))));
const INTERVALS = [300, 900, 1800, 2700, 3600, 7200, 10800, 14400, 21600, 28800, 43200, 86400, 2 * 86400, 3 * 86400, 7 * 86400, 14 * 86400, 30 * 86400];

const nameOf = (id) => (TASK_LABEL[id] || [id])[0];

// ---------------------------------------------------------------- words
/** A length of time as a sentence says it: "6 minutes", "2 hours", "under a minute". */
export function spoken(sec) {
  sec = Math.max(0, Math.round(sec));
  if (sec < 60) return tr('settings.tasks.spoken.under_minute');
  if (sec < 3600) return tr('settings.tasks.spoken.minutes', { count: Math.round(sec / 60) });
  if (sec < 86400 * 2) return tr('settings.tasks.spoken.hours', { count: Math.round(sec / 3600) });
  return tr('settings.tasks.spoken.days', { count: Math.round(sec / 86400) });
}
const ago = (ts) => (Date.now() / 1000 - ts < 60 ? tr('settings.tasks.ago_under') : tr('settings.tasks.ago', { time: spoken(Date.now() / 1000 - ts) }));
/** An exact length in its largest whole unit: "15 minutes", "1 hour", "7 days". */
const amount = (sec) => (sec % 86400 === 0 ? tr('settings.tasks.spoken.days', { count: sec / 86400 })
  : sec % 3600 === 0 ? tr('settings.tasks.spoken.hours', { count: sec / 3600 }) : tr('settings.tasks.spoken.minutes', { count: Math.round(sec / 60) }));
/** "Every 15 minutes", "Every hour", "Every 7 days". */
export function every(sec) {
  const [unit, n] = sec % 86400 === 0 ? ['day', sec / 86400] : sec % 3600 === 0 ? ['hour', sec / 3600] : ['minute', Math.round(sec / 60)];
  return n === 1 ? tr(`settings.tasks.every.${unit}`) : tr(`settings.tasks.every.${unit}s`, { count: n });
}
const timef = new Intl.DateTimeFormat(lang(), { hour: 'numeric', minute: '2-digit', timeZone: 'UTC' });
/** A minute of the day as the reader's clock writes it: "3:00 AM", "03:00". */
export const timeWords = (min) => (timeFormatChoice() ? clockTime(new Date(2000, 0, 1, 0, min)) : timef.format(new Date(Date.UTC(2000, 0, 1, 0, min))));
/** One trigger in words, the way Jellyfin lists them. */
export function triggerWords(t) {
  switch (t.type) {
    case 'daily': return tr('settings.tasks.trigger.daily_at', { time: timeWords(t.at_min) });
    case 'weekly': return tr('settings.tasks.trigger.weekly_at', { day: DAYS[t.day], time: timeWords(t.at_min) });
    case 'interval': return every(t.every_s);
    case 'startup': return tr('settings.tasks.trigger.startup');
    case 'after_scan': return tr('settings.tasks.trigger.after_scan');
    default: return t.type;
  }
}
const limitWords = (s) => (s % 3600 === 0 ? tr('settings.tasks.spoken.hours', { count: s / 3600 }) : tr('settings.tasks.spoken.minutes', { count: Math.round(s / 60) }));

/** How a job last went, in one sentence: "Last ran 38 minutes ago, taking 6 minutes." */
function lastRun(t) {
  if (t.state === 'running') return tr('settings.tasks.running');
  if (!t.finished_at) return tr('settings.tasks.not_run');
  const when = ago(t.finished_at);
  const verb = t.state === 'error' ? 'failed' : 'last_ran';
  return t.started_at ? tr(`settings.tasks.${verb}_taking`, { when, took: spoken(t.finished_at - t.started_at) }) : tr(`settings.tasks.${verb}`, { when });
}
function nextRun(t) {
  if (!t.schedulable) return null;
  if (!t.triggers.length) return tr('settings.tasks.only_by_hand');
  if (t.next_at) return t.next_at - Date.now() / 1000 < 60 ? tr('settings.tasks.next_within_minute') : tr('settings.tasks.next_in', { time: spoken(t.next_at - Date.now() / 1000) });
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
  const btn = button({ variant: 'icon', class: 'task-run', type: 'button', 'aria-label': tr('settings.tasks.run_aria', { name }), title: running ? tr('settings.tasks.running') : tr('settings.tasks.run_now'), disabled: running },
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
  mount(slot, card({ title: tr('settings.sections.tasks.label'), sub: tr('settings.tasks.list_sub'), body, id: 'tasks' }));
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
      return h('div', { class: 'task-group' }, h('h3', { class: 'task-group-title' }, tr(`settings.tasks.group.${g}`)),
        h('ul', { class: 'task-list' }, mine.map((t) => {
          const [name, what] = TASK_LABEL[t.id];
          const next = nextRun(t);
          return h('li', { class: ['task-row', t.state === 'running' && 'is-running'], id: `task-${t.id}` },
            h('span', { class: ['task-icon', t.state === 'error' && 'is-failed'], 'aria-hidden': 'true' }, icon(t.state === 'error' ? 'alert' : 'clock', 18)),
            h('div', { class: 'task-main' },
              h('a', { class: 'task-link', href: `/settings/tasks/${t.id}` }, h('span', { class: 'task-name' }, name)),
              h('p', { class: 'task-when' }, lastRun(t), next ? h('span', { class: 'task-next' }, ` ${next}.`) : null),
              progressOf(t, tr('settings.tasks.progress_aria', { name })),
              runErr[t.id] ? inlineError('task-run-err-' + t.id, tr('settings.tasks.couldnt_start', { error: runErr[t.id] })) : null,
              h('p', { class: 'sr-only' }, what)),
            runButton(t, store, paint));
        })));
    }), h('p', { class: 'fui-field__help task-zone' }, zoneWords(store)));
  }
  store.onTasks(() => paint());
  return store.loadTasks().then(() => paint(true));
}

const zoneWords = (store) => (store.tasks.time_zone ? tr('settings.tasks.zone', { zone: store.tasks.time_zone }) : tr('settings.tasks.zone_unknown'));

// Jobs whose schedule only an administrator changes (the server's `ADMIN_SCHEDULES`): the users read carries Jellyfin's
// disables to FinStats, the activity log its sign-ins, and the backups are the administrator's own.
const ADMIN_SCHEDULES = ['sync_users', 'sync_events', 'backup'];

// ---------------------------------------------------------------- one job
function taskView(slot, store, id) {
  const body = h('div', { class: 'stack' }, sk.rows(3));
  const [name, what] = TASK_LABEL[id] || [id, ''];
  mount(slot,
    h('a', { class: 'back-link', href: '/settings/tasks' }, icon('chevronLeft', 14), tr('settings.tasks.all_tasks')),
    card({ title: name, sub: what, body, id: `task-${id}` }));
  let err = null, sig = '';

  async function save(triggers) {
    err = null;
    try { await api.put(`/tasks/${encodeURIComponent(id)}/triggers`, { triggers }); }
    catch (e) { err = e.message; }
    await store.poke(2000);
    paint(true);
  }

  function paint(force = false) {
    const t = store.task(id);
    if (!t) { mount(body, h('p', { class: 'fui-field__help' }, tr('settings.tasks.no_such_task')), button({ href: '/settings/tasks' }, tr('settings.tasks.all_tasks'))); return; }
    const now = JSON.stringify([said(t), t.triggers, t.custom, t.next_at && dateTime(t.next_at), err, [...runBusy], runErr[id]]);
    if (!force && now === sig) return;
    sig = now;
    const status = h('div', { class: 'task-status' },
      h('p', { class: 'task-when' }, lastRun(t), t.finished_at ? h('span', { class: 'muted' }, ` (${dateTime(t.finished_at)})`) : null),
      progressOf(t, tr('settings.tasks.progress_aria', { name })),
      t.state === 'error' && t.error ? inlineError('task-err', t.error) : null,
      runErr[id] ? inlineError('task-run-err', tr('settings.tasks.couldnt_start', { error: runErr[id] })) : null);
    const run = t.runnable ? button({ type: 'button', disabled: t.state === 'running' || runBusy.has(id) }, icon('play', 12), t.state === 'running' ? tr('settings.tasks.running') : tr('settings.tasks.run_now')) : null;
    if (run) run.addEventListener('click', () => runButton(t, store, paint).click());

    if (!t.schedulable) {
      const [href, where] = STARTED_FROM[id] || ['/settings', tr('nav.settings')];
      mount(body, status, h('p', { class: 'fui-field__help' }, ...tParts('settings.tasks.needs_file', { where: h('a', { href }, where) })));
      return;
    }
    const mine = isAdmin() || !ADMIN_SCHEDULES.includes(id);
    const add = mine ? button({ variant: 'primary', type: 'button' }, icon('plus', 13), tr('settings.tasks.add_trigger')) : null;
    if (add) add.addEventListener('click', () => addDialog(t, (trigger) => save([...t.triggers, trigger])));
    const reset = mine && t.custom ? button({ variant: 'ghost', type: 'button' }, tr('settings.tasks.reset')) : null;
    if (reset) reset.addEventListener('click', async () => {
      setBusy(reset, true, tr('settings.tasks.resetting')); err = null;
      try { await api.del(`/tasks/${encodeURIComponent(id)}/triggers`); } catch (e) { err = e.message; }
      await store.poke(2000); paint(true);
    });
    const list = t.triggers.length
      ? h('ul', { class: 'trigger-list', id: 'task-triggers' }, t.triggers.map((g, i) => {
        const words = triggerWords(g);
        const remove = mine ? button({ variant: 'icon', class: 'trigger-remove', type: 'button', 'aria-label': tr('settings.tasks.remove_aria', { trigger: words }), title: tr('settings.tasks.remove') }, icon('minus', 14)) : null;
        if (remove) remove.addEventListener('click', () => { remove.disabled = true; save(t.triggers.filter((_, j) => j !== i)); });
        return h('li', { class: 'trigger-row' }, h('span', { class: 'trigger-what' }, words),
          g.limit_s ? h('span', { class: 'trigger-limit muted' }, tr('settings.tasks.stops_after', { time: limitWords(g.limit_s) })) : null, remove);
      }))
      : h('div', { id: 'task-triggers' }, h('p', { class: 'fui-field__help' }, tr('settings.tasks.no_triggers')));
    mount(body, status,
      h('div', { class: 'form-actions task-actions' }, add, run, reset),
      h('div', { class: 'fui-field' }, h('div', { class: 'fui-field__label' }, tr('settings.tasks.runs')), list, err ? inlineError('trigger-err', err) : null),
      h('p', { class: 'fui-field__help' }, zoneWords(store), ' ',
        t.next_at ? tr('settings.tasks.next_run', { when: dateTime(t.next_at) }) : ''));
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
  const type = select('trigger-type', tr('settings.tasks.dialog.type'), [opt('daily', tr('settings.tasks.dialog.daily')), opt('weekly', tr('settings.tasks.dialog.weekly')), opt('interval', tr('settings.tasks.dialog.interval')), opt('startup', tr('settings.tasks.trigger.startup')),
    t.can.after_scan ? opt('after_scan', tr('settings.tasks.trigger.after_scan')) : null]);
  const day = select('trigger-day', tr('settings.tasks.dialog.day'), DAYS.map((d, i) => opt(String(i), d[0].toLocaleUpperCase(lang()) + d.slice(1))));
  const time = select('trigger-time', tr('settings.tasks.dialog.time'), Array.from({ length: 96 }, (_, i) => opt(String(i * 15), timeWords(i * 15), i * 15 === 180)));
  const every_ = select('trigger-every', tr('settings.tasks.dialog.every'), INTERVALS.map((s) => opt(String(s), amount(s), s === 3600)));
  const limit = t.can.limit ? h('input', { class: 'fui-field__input fui-field__input--num mono', id: 'trigger-limit', type: 'number', min: '0.25', max: '168', step: '0.25', inputMode: 'decimal', 'aria-describedby': 'trigger-limit-help' }) : null;
  const limitField = limit ? h('div', { class: 'fui-field' }, h('label', { class: 'fui-field__label', htmlFor: 'trigger-limit' }, tr('settings.tasks.dialog.limit')), limit,
    h('p', { class: 'fui-field__help', id: 'trigger-limit-help' }, tr('settings.tasks.dialog.limit_help'))) : null;
  const formErr = h('div');
  const addBtn = button({ variant: 'primary', type: 'submit' }, tr('settings.tasks.dialog.add'));
  const cancel = button({ variant: 'ghost', type: 'button' }, tr('common.cancel'));
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
  const modal = openModal({ title: tr('settings.tasks.add_trigger'), body: form, initialFocus: type.el });
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
      if (!(hours > 0 && hours <= 168)) { mount(formErr, inlineError('trigger-limit-err', tr('settings.tasks.dialog.limit_range'))); limit.focus(); return; }
      trigger.limit_s = Math.round(hours * 3600);
    }
    if (t.triggers.some((x) => triggerWords(x) === triggerWords(trigger) && (x.limit_s || null) === (trigger.limit_s || null))) {
      mount(formErr, inlineError('trigger-dup-err', tr('settings.tasks.dialog.duplicate'))); return;
    }
    modal.close();
    onAdd(trigger);
  });
}

export default {
  key: 'tasks', label: tr('settings.sections.tasks.label'), sub: tr('settings.sections.tasks.sub'), group: tr('settings.groups.data'), icon: 'clock',
  visible: () => can('manage'),
  entries: Object.entries(TASK_LABEL).map(([id, [label, what]]) => ({ id: `task-${id}`, label, hint: `${what} ${tr('settings.tasks.find_hint')}` })),
  async render(slot, store) {
    const id = store.ctx.params.task;
    try { await (id ? taskView(slot, store, id) : listView(slot, store)); }
    catch (e) { mount(slot, errorState(e, () => this.render(slot, store))); }
  },
};
