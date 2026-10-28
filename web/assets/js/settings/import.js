// Settings → Import: bring the history along from another tracker. One card per tracker, built from
// its entry in IMPORTERS. Only one import can run at a time, so a card whose own import is not the one
// going says so rather than offering a drop zone. A tracker whose people are not Jellyfin's (`board`:
// Tautulli, which is Plex) does not import on upload: the upload answers with a wiring board, and the
// import starts once its wires are drawn.

import { h, icon, num, bytes, relTime, dateTime, mount } from '../dom.js';
import { can } from '../state.js';
import { api, uploadRaw } from '../api.js';
import { card, spinner, inlineError } from '../components.js';
import { wiringBoard } from './wiring.js';
import { button } from '../../finui/components/button/button.js';
import { t, tParts } from '../i18n.js';

// The trackers FinStats can take history from.
export const IMPORTERS = [
  {
    key: 'jellystat', task: 'import', name: 'Jellystat', endpoint: '/import/jellystat', accept: '.jsonl,.json,application/json', pattern: /\.(jsonl|json)$/i,
    steps: [
      ['open'],
      ['settings_backup', 'Settings', 'Backup'],
      ['only_activity', 'Activity'],
      ['under_settings', 'Settings'],
      ['start_backup'],
      ['back_to_backups', 'Backups'],
      ['download', 'Actions', 'Download'],
      ['upload'],
    ],
    rows: ['plays_imported', 'plays_skipped', 'users', 'libraries', 'items', 'seasons', 'episodes', 'item_info'],
  },
  {
    key: 'streamystats', task: 'import_streamystats', name: 'Streamystats', endpoint: '/import/streamystats', accept: '.json,application/json', pattern: /\.json$/i,
    steps: [
      ['open'],
      ['settings_backup_import', 'Settings', 'Backup & Import'],
      ['scroll_to', 'Backup & Restore'],
      ['click_download', 'Download Backup'],
      ['upload'],
    ],
    rows: ['plays_imported', 'plays_skipped', 'marked_watched', 'users', 'sessions_read'],
  },
  {
    key: 'tautulli', task: 'import_tautulli', name: 'Tautulli', endpoint: '/import/tautulli', accept: '.db,.zip,application/zip', pattern: /\.(db|zip)$/i,
    board: true,
    steps: [
      ['open'],
      ['settings_import_backups', 'Settings', 'Import & Backups'],
      ['backup_database', 'Backup Database'],
      ['find_it', 'backups', '/config'],
      ['newest', 'tautulli.backup-…', 'config.backup-…'],
      ['copy'],
      ['upload_then_wire'],
    ],
    rows: ['plays_imported', 'plays_skipped', 'users_wired', 'not_wired', 'other_media'],
  },
].map((imp) => ({ ...imp,
  file: t(`settings.import.${imp.key}.file`), title: t('settings.import.title', { name: imp.name }), sub: t(`settings.import.${imp.key}.sub`), note: t(`settings.import.${imp.key}.note`),
  steps: imp.steps.map(([step, ...names]) => ({ key: `settings.import.${imp.key}.step.${step}`, names })),
  rows: imp.rows.map((k) => [k, t(`settings.import.result.${k}`)]) }));

// The upload lives outside the page so it keeps going if you navigate away. One at a time, so one
// object: `source` says which card owns it.
const upload = { active: false, progress: 0, loaded: 0, total: 0, fileName: '', error: null, doneAt: 0, handle: null, source: null };
const uploadSubs = new Set();
const notifyUpload = () => uploadSubs.forEach((fn) => fn());

// A board waiting for its wires, per importer that draws one: what the upload answered, or what the server still holds.
const boards = {};

function startUpload(file, imp) {
  Object.assign(upload, { active: true, progress: 0, loaded: 0, total: file.size, fileName: file.name, error: null, doneAt: 0, source: imp.key });
  notifyUpload();
  const handle = uploadRaw(imp.endpoint, file, (p, loaded, total) => { Object.assign(upload, { progress: p, loaded, total }); notifyUpload(); });
  upload.handle = handle;
  handle.promise.then((data) => {
    if (imp.board) boards[imp.key] = (data && data.board) || null;
    Object.assign(upload, { active: false, doneAt: imp.board ? 0 : Date.now(), handle: null }); notifyUpload();
  })
    .catch((e) => {
      const msg = e.status === -1 ? null
        : e.status === 409 ? t('settings.import.already_running')
        : e.status === 413 ? t('settings.import.too_large')
        : e.message;
      Object.assign(upload, { active: false, error: msg, handle: null }); notifyUpload();
    });
}

/** The import task that is going right now, if any: the tasks poll hurries while one is. */
export const importing = (store) => IMPORTERS.some((i) => { const t = store.task(i.task); return t && t.state === 'running'; });

export default {
  key: 'import', label: t('settings.sections.import.label'), sub: t('settings.sections.import.sub'), group: t('settings.groups.data'), icon: 'upload',
  visible: () => can('manage') && can('see_everyone'), // an import writes other people's history
  entries: IMPORTERS.map((i) => ({ id: `import-${i.key}`, label: i.title, hint: `${i.name} ${t('settings.import.find_hint')}` })),
  async render(slot, store) {
    const slots = Object.fromEntries(IMPORTERS.map((i) => [i.key, h('div')]));
    // What an import could not match is linked under Settings → Unlinked media: said here while there is any.
    const pointer = h('div');
    const pointerBox = card({ title: t('settings.sections.unlinked.label'), body: pointer, id: 'import-unlinked' });
    pointerBox.hidden = true;
    mount(slot, IMPORTERS.map((i) => card({ title: i.title, sub: i.sub, body: slots[i.key], id: `import-${i.key}` })), pointerBox);
    async function unlinked() {
      let n = 0;
      try { n = ((await api.get('/library/missing')).missing || []).length; } catch { /* nothing to point at */ }
      pointerBox.hidden = !n;
      mount(pointer, n ? h('p', { class: 'fui-field__help' }, ...tParts('settings.import.unlinked', { count: n, link: h('a', { href: '/settings/unlinked' }, t('settings.import.unlinked_link')) })) : null);
    }
    unlinked();
    // An import that finishes may have left titles that did not match, or matched some that had not.
    let finished = '';
    store.onTasks(() => {
      const now = JSON.stringify(IMPORTERS.map((i) => { const t = store.task(i.task); return t && t.state === 'ok' ? t.finished_at : null; }));
      if (finished && now !== finished) unlinked();
      finished = now;
    });

    const localErr = {}, sawRunning = {}, sig = {};
    const fileInputs = Object.fromEntries(IMPORTERS.map((imp) => {
      const input = h('input', { type: 'file', accept: imp.accept, class: 'sr-only', id: `import-file-${imp.key}`, tabIndex: -1 });
      input.addEventListener('change', () => { if (input.files[0]) pick(input.files[0], imp); input.value = ''; });
      return [imp.key, input];
    }));

    function pick(file, imp) {
      localErr[imp.key] = null;
      if (!imp.pattern.test(file.name)) localErr[imp.key] = t('settings.import.wrong_file', { file: file.name, name: imp.name, what: imp.file });
      else if (!file.size) localErr[imp.key] = t('settings.import.empty_file', { name: imp.name });
      if (localErr[imp.key]) { paintAll(); return; }
      sawRunning[imp.key] = false;
      startUpload(file, imp);
      store.poke(1000);
    }

    // The board is kept across repaints (a poll must not throw away the wires being drawn) and torn down when it goes.
    const views = {};
    function boardView(imp) {
      const v = views[imp.key];
      if (v && v.data === boards[imp.key]) return v.board.el;
      if (v) v.board.destroy();
      const board = wiringBoard(boards[imp.key], {
        onImport: async (wires) => {
          await api.post(`${imp.endpoint}/run`, { wires });
          boards[imp.key] = null; sawRunning[imp.key] = false;
          store.poke(500); paintAll();
        },
        onReset: async () => {
          await api.del(imp.endpoint).catch(() => {});
          boards[imp.key] = null; paintAll();
        },
      });
      views[imp.key] = { data: boards[imp.key], board };
      return board.el;
    }
    store.ctx.onCleanup(() => { for (const v of Object.values(views)) v.board.destroy(); });

    function paint(imp) {
      const task = store.task(imp.task);
      const running = !!task && task.state === 'running';
      if (running) sawRunning[imp.key] = true;
      const mine = upload.source === imp.key;
      const justUploaded = mine && upload.doneAt && Date.now() - upload.doneAt < 15000 && !sawRunning[imp.key] && !(task && task.state === 'error');
      // Somebody else's import: this card must not offer to start a second one.
      const elsewhere = IMPORTERS.some((other) => other.key !== imp.key && store.task(other.task) && store.task(other.task).state === 'running');
      const uploading = upload.active && mine;
      const waiting = !!(imp.board && boards[imp.key]) && !running;
      const now = JSON.stringify([task, running, !!justUploaded, uploading, upload.loaded, upload.error && mine, localErr[imp.key], elsewhere, waiting && views[imp.key] ? views[imp.key].data === boards[imp.key] : waiting]);
      if (now === sig[imp.key]) return; // keep the drop zone (and its focus) stable between polls
      sig[imp.key] = now;

      let stage;
      if (uploading) {
        stage = h('div', { class: 'import-stage', role: 'status' },
          h('div', { class: 'import-stage-title' }, t('settings.import.uploading', { file: upload.fileName })),
          h('div', { class: 'fui-meter fui-meter--wide', role: 'progressbar', 'aria-label': t('settings.import.upload_progress'), 'aria-valuemin': 0, 'aria-valuemax': 100, 'aria-valuenow': Math.round(upload.progress * 100) },
            h('span', { class: 'fui-meter__fill', style: { width: upload.progress * 100 + '%' } })),
          h('div', { class: 'import-stage-row' }, h('span', { class: 'mono' }, t('settings.import.upload_line', { percent: Math.round(upload.progress * 100), loaded: bytes(upload.loaded), total: bytes(upload.total) })),
            button({ size: 'sm', type: 'button', onClick: () => upload.handle && upload.handle.abort() }, t('settings.import.cancel_upload'))),
          h('p', { class: 'fui-field__help' }, t('settings.import.keep_open')));
      } else if (waiting) {
        stage = boardView(imp);
      } else if (running || justUploaded) {
        stage = h('div', { class: 'import-stage', role: 'status' },
          h('div', { class: 'import-stage-title' }, spinner(14), running ? t('settings.import.importing') : t('settings.import.upload_complete')),
          h('div', { class: ['fui-meter fui-meter--wide', !(running && task.progress != null) && 'is-indeterminate'], role: 'progressbar', 'aria-label': t('settings.import.import_progress'), 'aria-valuemin': 0, 'aria-valuemax': 100, 'aria-valuenow': running && task.progress != null ? Math.round(task.progress * 100) : null },
            h('span', { class: 'fui-meter__fill', style: { width: (running && task.progress != null ? task.progress * 100 : 30) + '%' } })),
          h('div', { class: 'mono import-msg', 'aria-live': 'polite' }, running ? task.message || t('settings.import.reading') : t('settings.import.starting')),
          h('p', { class: 'fui-field__help' }, t('settings.import.safe_to_leave')));
      } else {
        const input = fileInputs[imp.key];
        const drop = h('label', { class: ['dropzone', elsewhere && 'is-disabled'], htmlFor: elsewhere ? null : input.id, tabindex: elsewhere ? -1 : 0, role: 'button', 'aria-disabled': elsewhere ? 'true' : null, 'aria-label': t('settings.import.choose_aria', { name: imp.name }) },
          icon('upload', 20), h('span', { class: 'dropzone-title' }, elsewhere ? t('settings.import.another_running') : t('settings.import.drop_here', { name: imp.name })),
          h('span', { class: 'dropzone-sub' }, elsewhere ? t('settings.import.one_at_a_time') : t('settings.import.or_click', { what: imp.file })));
        if (!elsewhere) {
          drop.addEventListener('keydown', (e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); input.click(); } });
          drop.addEventListener('dragover', (e) => { e.preventDefault(); drop.classList.add('is-over'); });
          drop.addEventListener('dragleave', () => drop.classList.remove('is-over'));
          drop.addEventListener('drop', (e) => { e.preventDefault(); drop.classList.remove('is-over'); const f = e.dataTransfer.files[0]; if (f) pick(f, imp); });
        }

        let outcome = null;
        if (task && task.state === 'ok' && task.result) {
          outcome = h('div', { class: 'import-result' },
            h('div', { class: 'fui-badge--status fui-badge--good' }, icon('check', 14), t('settings.import.finished'), task.finished_at ? h('span', { class: 'muted mono', title: dateTime(task.finished_at) }, ' ' + relTime(task.finished_at)) : null),
            task.message ? h('p', { class: 'fui-field__help' }, task.message) : null,
            h('table', { class: 'fui-data-table fui-data-table--dense result-table' }, h('tbody', null, imp.rows.filter(([k]) => task.result[k] != null).map(([k, label]) =>
              h('tr', null, h('th', { scope: 'row' }, label), h('td', { class: 'mono r' }, num(task.result[k])))))),
            button({ size: 'sm', href: '/' }, t('settings.import.see_stats'), icon('chevronRight', 14)));
        } else if (task && task.state === 'error') {
          outcome = h('div', { class: 'import-result' }, inlineError(`import-task-err-${imp.key}`, t('settings.import.failed', { error: task.error || task.message || t('settings.import.unknown_error') })),
            h('p', { class: 'fui-field__help' }, t('settings.import.nothing_half', { name: imp.name })));
        }
        stage = [drop,
          localErr[imp.key] ? inlineError(`import-local-err-${imp.key}`, localErr[imp.key]) : null,
          upload.error && mine ? inlineError(`import-upload-err-${imp.key}`, t('settings.import.upload_failed', { error: upload.error })) : null,
          outcome];
      }

      // The board takes the whole card: the wires need the width, and the export steps are done by now.
      if (waiting) { mount(slots[imp.key], h('h3', { class: 'section-label' }, t('settings.import.connect_wires')), stage); return; }
      mount(slots[imp.key],
        h('div', { class: 'import-cols' },
          h('div', null, h('h3', { class: 'section-label' }, t('settings.import.export_from', { name: imp.name })),
            h('ol', { class: 'steps' }, imp.steps.map(({ key, names }) => h('li', null, ...tParts(key, Object.fromEntries(names.map((n, i) => ['abc'[i], h('strong', null, n)]))))))),
          h('div', null, h('h3', { class: 'section-label' }, t('settings.import.upload_backup')), fileInputs[imp.key], stage,
            h('p', { class: 'fui-field__help import-note' }, icon('info', 13), ' ', imp.note))));
    }
    const paintAll = () => { for (const imp of IMPORTERS) paint(imp); };

    const onUpload = () => { paintAll(); if (!upload.active && upload.doneAt) store.poke(1000); };
    uploadSubs.add(onUpload);
    store.ctx.onCleanup(() => uploadSubs.delete(onUpload));
    store.hurry(() => (importing(store) || (upload.doneAt && Date.now() - upload.doneAt < 15000) ? 1000 : null));
    store.onTasks(paintAll);
    paintAll();
    store.loadTasks().catch(() => {});
    // A board left waiting (the page closed half-way through the wiring) is picked up where it was.
    for (const imp of IMPORTERS.filter((i) => i.board)) {
      api.get(imp.endpoint).then((d) => { boards[imp.key] = d.board || null; paintAll(); }, () => {});
    }
  },
};
