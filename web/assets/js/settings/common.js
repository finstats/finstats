// What every settings section shares: one store for the two answers most of them read (`/settings`,
// `/tasks`), and the two shapes a setting takes on the page — a switch that saves itself, and a row
// of numbers saved together. A section is a module: `{ key, label, sub, group, icon, visible,
// entries, render }`. `entries` is what the finder in the page header offers; `render(slot, store)`
// paints into the slot and may keep painting while the page is open (`store.onTasks`).

import { h, icon, num, mount } from '../dom.js';
import { api, isAbort } from '../api.js';
import { toggle, spinner, inlineError, setBusy } from '../components.js';

/** One place for the answers several sections read, fetched once per visit and polled only if asked. */
export function createStore(ctx) {
  let settingsP = null;
  const tasksSubs = new Set();
  const hints = new Set();   // () => ms | null: a section that wants the tasks read more often says so here
  let setPoll = null;
  const store = {
    ctx,
    signal: ctx.signal,
    settings: null,
    tasks: null,
    loadSettings() {
      if (!settingsP) settingsP = api.get('/settings', null, { signal: ctx.signal }).then((d) => (store.settings = d));
      return settingsP;
    },
    reloadSettings() { settingsP = null; return store.loadSettings(); },
    async put(patch) {
      store.settings = await api.put('/settings', patch);
      settingsP = Promise.resolve(store.settings);
      return store.settings;
    },
    async loadTasks() {
      try {
        store.tasks = await api.get('/tasks', null, { signal: ctx.signal });
      } catch (e) {
        if (isAbort(e) || e.status === 401) return null;
        throw e;
      }
      const running = (store.tasks.tasks || []).some((t) => t.state === 'running');
      let ms = running ? 2000 : 10000;
      for (const hint of hints) { const want = hint(); if (want && want < ms) ms = want; }
      if (setPoll) setPoll(ms);
      for (const fn of tasksSubs) { try { fn(store.tasks); } catch (e) { console.error(e); } }
      return store.tasks;
    },
    /** Be told after every read of the tasks; the first subscriber starts the poll. */
    onTasks(fn) {
      tasksSubs.add(fn);
      if (!setPoll) setPoll = ctx.every(() => store.loadTasks().catch(() => {}), 10000);
    },
    /** `fn()` answers how soon this section wants the next read, or null for no opinion. */
    hurry(fn) { hints.add(fn); },
    /** Read the tasks again now, and at least this often for a while. */
    poke(ms = 1000) { if (setPoll) setPoll(ms); return store.loadTasks().catch(() => {}); },
    task(id) { return store.tasks && (store.tasks.tasks || []).find((t) => t.id === id); },
  };
  return store;
}

/** The two-sided row every setting sits in: what it is on the left, the control on the right. */
export function settingRow({ id, label, help, control, labelFor, error }) {
  const labelEl = labelFor
    ? h('label', { class: 'setting-label', id: `${id}-label`, htmlFor: labelFor }, label)
    : h('div', { class: 'setting-label', id: `${id}-label` }, label);
  return [h('div', { class: 'setting-row', id },
    h('div', null, labelEl, help ? h('p', { class: 'help', id: `${id}-help` }, help) : null),
    h('div', { class: 'setting-control' }, control)), error || null];
}

/** An immediate-effect setting: a switch that saves on change and confirms next to itself. */
export function toggleRow(store, { key, label, help, onSaved }) {
  const note = h('span', { class: 'saved-note', 'aria-live': 'polite' });
  const err = h('div');
  let noteTimer;
  const sw = toggle({ checked: !!store.settings[key], labelledby: `${key}-label`, describedby: help ? `${key}-help` : null,
    onChange: async (next, revert) => {
      mount(err, ''); note.replaceChildren(spinner(12));
      try {
        await store.put({ [key]: next });
        note.replaceChildren(icon('check', 13), 'Saved');
        if (onSaved) onSaved();
        clearTimeout(noteTimer); noteTimer = setTimeout(() => note.replaceChildren(), 2000);
      } catch (e) {
        revert(!next); note.replaceChildren();
        mount(err, inlineError(`${key}-err`, `Couldn’t save: ${e.message}`));
      }
    } });
  return settingRow({ id: key, label, help, control: [note, sw], error: err });
}

/** Whole-number settings, one row each, checked here and saved in one request. */
export function numberForm(store, FIELDS, errId) {
  const inputs = {};
  const errs = {};
  const rows = FIELDS.flatMap((f) => {
    const input = h('input', { class: 'input input-num mono', type: 'text', inputMode: 'numeric', id: 'f-' + f.key, name: f.key, value: String(store.settings[f.key] ?? ''),
      autocomplete: 'off', 'aria-describedby': `${f.key}-help` });
    inputs[f.key] = input;
    errs[f.key] = h('div');
    return settingRow({ id: f.key, label: f.label, help: f.help, labelFor: 'f-' + f.key,
      control: h('div', { class: 'field-input' }, input, h('span', { class: 'unit' }, f.unit)), error: errs[f.key] });
  });
  const note = h('span', { class: 'saved-note', 'aria-live': 'polite' });
  const formErr = h('div');
  const save = h('button', { type: 'submit', class: 'btn btn-primary' }, 'Save changes');
  let noteTimer;
  const form = h('form', { class: 'setting-rows', noValidate: true }, rows, h('div', { class: 'form-actions setting-actions' }, save, note), formErr);
  form.addEventListener('submit', async (e) => {
    e.preventDefault();
    mount(formErr, '');
    const body = {};
    let firstBad = null;
    for (const f of FIELDS) {
      const raw = inputs[f.key].value.trim();
      const n = Number(raw);
      const bad = raw === '' || !/^\d+$/.test(raw) ? `Enter a whole number between ${f.min} and ${num(f.max)}.`
        : n < f.min || n > f.max ? `Must be between ${f.min} and ${num(f.max)}.` : null;
      inputs[f.key].setAttribute('aria-invalid', bad ? 'true' : 'false');
      inputs[f.key].setAttribute('aria-describedby', bad ? `e-${f.key} ${f.key}-help` : `${f.key}-help`);
      mount(errs[f.key], bad ? inlineError(`e-${f.key}`, bad) : '');
      if (bad && !firstBad) firstBad = inputs[f.key];
      body[f.key] = n;
    }
    if (firstBad) { firstBad.focus(); return; }
    setBusy(save, true, 'Saving…');
    try {
      await store.put(body);
      for (const f of FIELDS) inputs[f.key].value = String(store.settings[f.key]);
      note.replaceChildren(icon('check', 13), 'Saved');
      clearTimeout(noteTimer); noteTimer = setTimeout(() => note.replaceChildren(), 2000);
    } catch (err) {
      mount(formErr, inlineError(errId, `Couldn’t save: ${err.message}`));
    } finally { setBusy(save, false); }
  });
  return form;
}

/** A progress meter with its message, for a task that is running. */
export function progressOf(t, label) {
  if (!t || t.state !== 'running') return null;
  return h('div', { class: 'task-progress' },
    h('div', { class: ['meter meter-wide', t.progress == null && 'is-indeterminate'], role: 'progressbar', 'aria-label': label, 'aria-valuemin': 0, 'aria-valuemax': 100, 'aria-valuenow': t.progress == null ? null : Math.round(t.progress * 100) },
      h('span', { class: 'meter-fill', style: { width: (t.progress == null ? 30 : t.progress * 100) + '%' } })),
    h('span', { class: 'mono task-msg' }, t.message || 'Working…'));
}
