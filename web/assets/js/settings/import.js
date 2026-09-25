// Settings → Import: bring the history along from another tracker. One card per tracker, built from
// its entry in IMPORTERS. Only one import can run at a time, so a card whose own import is not the one
// going says so rather than offering a drop zone.

import { h, icon, num, bytes, relTime, dateTime, mount } from '../dom.js';
import { can } from '../state.js';
import { uploadRaw } from '../api.js';
import { card, spinner, inlineError } from '../components.js';

// The trackers finstats can take history from.
export const IMPORTERS = [
  {
    key: 'jellystat', task: 'import', name: 'Jellystat', endpoint: '/import/jellystat', file: 'the .jsonl file',
    title: 'Import from Jellystat', sub: 'Bring your playback history with you',
    steps: [
      ['Open your Jellystat instance.'],
      ['Go to ', 'Settings', ' and select the ', 'Backup', ' tab.'],
      ['Select only ', 'Activity', ' — it turns purple when selected.'],
      ['Under settings, click ', 'Settings', '.'],
      ['Scroll all the way to the end and start a backup.'],
      ['Go back to ', 'Backups', '.'],
      ['Once the new backup shows up, open its ', 'Actions', ' menu and click ', 'Download', '.'],
      ['Upload the file here.'],
    ],
    rows: [['plays_imported', 'Plays imported'], ['plays_skipped', 'Already here'], ['users', 'Users'], ['libraries', 'Libraries'],
      ['items', 'Movies, series and tracks'], ['seasons', 'Seasons'], ['episodes', 'Episodes'], ['item_info', 'File details']],
    note: 'Importing the same backup again is safe — plays that are already here are skipped. Backups that also contain libraries and users work too.',
  },
  {
    key: 'streamystats', task: 'import_streamystats', name: 'Streamystats', endpoint: '/import/streamystats', file: 'the .json file',
    title: 'Import from Streamystats', sub: 'Bring your playback history with you',
    steps: [
      ['Open your Streamystats instance.'],
      ['Go to ', 'Settings', ' and select ', 'Backup & Import', '.'],
      ['Scroll down to ', 'Backup & Restore', '.'],
      ['Click ', 'Download Backup', '.'],
      ['Upload the file here.'],
    ],
    rows: [['plays_imported', 'Plays imported'], ['plays_skipped', 'Already here'], ['marked_watched', 'Marked watched, never played'],
      ['users', 'Users'], ['sessions_read', 'Sessions read']],
    note: 'Ran both trackers? Import both files — an evening either one already brought in is not counted twice. Streamystats keeps no library data, so titles come from your own Jellyfin.',
  },
];

// The upload lives outside the page so it keeps going if you navigate away. One at a time, so one
// object: `source` says which card owns it.
const upload = { active: false, progress: 0, loaded: 0, total: 0, fileName: '', error: null, doneAt: 0, handle: null, source: null };
const uploadSubs = new Set();
const notifyUpload = () => uploadSubs.forEach((fn) => fn());

function startUpload(file, imp) {
  Object.assign(upload, { active: true, progress: 0, loaded: 0, total: file.size, fileName: file.name, error: null, doneAt: 0, source: imp.key });
  notifyUpload();
  const handle = uploadRaw(imp.endpoint, file, (p, loaded, total) => { Object.assign(upload, { progress: p, loaded, total }); notifyUpload(); });
  upload.handle = handle;
  handle.promise.then(() => { Object.assign(upload, { active: false, doneAt: Date.now(), handle: null }); notifyUpload(); })
    .catch((e) => {
      const msg = e.status === -1 ? null
        : e.status === 409 ? 'An import is already running. Wait for it to finish, then try again.'
        : e.status === 413 ? 'The server rejected the file as too large. If finstats sits behind a reverse proxy, raise its upload limit (for nginx: client_max_body_size) and try again.'
        : e.message;
      Object.assign(upload, { active: false, error: msg, handle: null }); notifyUpload();
    });
}

/** The import task that is going right now, if any: the tasks poll hurries while one is. */
export const importing = (store) => IMPORTERS.some((i) => { const t = store.task(i.task); return t && t.state === 'running'; });

export default {
  key: 'import', label: 'Import', sub: 'Bring your history from another tracker', group: 'Data', icon: 'upload',
  visible: () => can('manage'),
  entries: IMPORTERS.map((i) => ({ id: `import-${i.key}`, label: i.title, hint: `${i.name} backup upload history tracker` })),
  async render(slot, store) {
    const slots = Object.fromEntries(IMPORTERS.map((i) => [i.key, h('div')]));
    mount(slot, IMPORTERS.map((i) => card({ title: i.title, sub: i.sub, body: slots[i.key], id: `import-${i.key}` })));

    const localErr = {}, sawRunning = {}, sig = {};
    const fileInputs = Object.fromEntries(IMPORTERS.map((imp) => {
      const input = h('input', { type: 'file', accept: '.jsonl,.json,application/json', class: 'sr-only', id: `import-file-${imp.key}`, tabIndex: -1 });
      input.addEventListener('change', () => { if (input.files[0]) pick(input.files[0], imp); input.value = ''; });
      return [imp.key, input];
    }));

    function pick(file, imp) {
      localErr[imp.key] = null;
      if (!/\.(jsonl|json)$/i.test(file.name)) localErr[imp.key] = `“${file.name}” isn’t a ${imp.name} backup. Choose ${imp.file} you downloaded from ${imp.name}.`;
      else if (!file.size) localErr[imp.key] = `That file is empty. Download the backup from ${imp.name} again.`;
      if (localErr[imp.key]) { paintAll(); return; }
      sawRunning[imp.key] = false;
      startUpload(file, imp);
      store.poke(1000);
    }

    function paint(imp) {
      const task = store.task(imp.task);
      const running = !!task && task.state === 'running';
      if (running) sawRunning[imp.key] = true;
      const mine = upload.source === imp.key;
      const justUploaded = mine && upload.doneAt && Date.now() - upload.doneAt < 15000 && !sawRunning[imp.key] && !(task && task.state === 'error');
      // Somebody else's import: this card must not offer to start a second one.
      const elsewhere = IMPORTERS.some((other) => other.key !== imp.key && store.task(other.task) && store.task(other.task).state === 'running');
      const uploading = upload.active && mine;
      const now = JSON.stringify([task, running, !!justUploaded, uploading, upload.loaded, upload.error && mine, localErr[imp.key], elsewhere]);
      if (now === sig[imp.key]) return; // keep the drop zone (and its focus) stable between polls
      sig[imp.key] = now;

      let stage;
      if (uploading) {
        stage = h('div', { class: 'import-stage', role: 'status' },
          h('div', { class: 'import-stage-title' }, `Uploading ${upload.fileName}`),
          h('div', { class: 'meter meter-wide', role: 'progressbar', 'aria-label': 'Upload progress', 'aria-valuemin': 0, 'aria-valuemax': 100, 'aria-valuenow': Math.round(upload.progress * 100) },
            h('span', { class: 'meter-fill', style: { width: upload.progress * 100 + '%' } })),
          h('div', { class: 'import-stage-row' }, h('span', { class: 'mono' }, `${Math.round(upload.progress * 100)}% · ${bytes(upload.loaded)} of ${bytes(upload.total)}`),
            h('button', { type: 'button', class: 'btn btn-sm', onClick: () => upload.handle && upload.handle.abort() }, 'Cancel upload')),
          h('p', { class: 'help' }, 'Keep this tab open until the upload finishes. You can browse other finstats pages meanwhile.'));
      } else if (running || justUploaded) {
        stage = h('div', { class: 'import-stage', role: 'status' },
          h('div', { class: 'import-stage-title' }, spinner(14), running ? 'Importing your history' : 'Upload complete'),
          h('div', { class: ['meter meter-wide', !(running && task.progress != null) && 'is-indeterminate'], role: 'progressbar', 'aria-label': 'Import progress', 'aria-valuemin': 0, 'aria-valuemax': 100, 'aria-valuenow': running && task.progress != null ? Math.round(task.progress * 100) : null },
            h('span', { class: 'meter-fill', style: { width: (running && task.progress != null ? task.progress * 100 : 30) + '%' } })),
          h('div', { class: 'mono import-msg', 'aria-live': 'polite' }, running ? task.message || 'Reading the backup…' : 'Starting the import…'),
          h('p', { class: 'help' }, 'This runs on the server. It’s safe to leave this page.'));
      } else {
        const input = fileInputs[imp.key];
        const drop = h('label', { class: ['dropzone', elsewhere && 'is-disabled'], htmlFor: elsewhere ? null : input.id, tabindex: elsewhere ? -1 : 0, role: 'button', 'aria-disabled': elsewhere ? 'true' : null, 'aria-label': `Choose a ${imp.name} backup file` },
          icon('upload', 20), h('span', { class: 'dropzone-title' }, elsewhere ? 'Another import is running' : `Drop your ${imp.name} backup here`),
          h('span', { class: 'dropzone-sub' }, elsewhere ? 'One at a time — this one can start when that one finishes' : `or click to choose ${imp.file} — large backups are fine`));
        if (!elsewhere) {
          drop.addEventListener('keydown', (e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); input.click(); } });
          drop.addEventListener('dragover', (e) => { e.preventDefault(); drop.classList.add('is-over'); });
          drop.addEventListener('dragleave', () => drop.classList.remove('is-over'));
          drop.addEventListener('drop', (e) => { e.preventDefault(); drop.classList.remove('is-over'); const f = e.dataTransfer.files[0]; if (f) pick(f, imp); });
        }

        let outcome = null;
        if (task && task.state === 'ok' && task.result) {
          outcome = h('div', { class: 'import-result' },
            h('div', { class: 'sev sev-good' }, icon('check', 14), 'Import finished', task.finished_at ? h('span', { class: 'muted mono', title: dateTime(task.finished_at) }, ' ' + relTime(task.finished_at)) : null),
            task.message ? h('p', { class: 'help' }, task.message) : null,
            h('table', { class: 'table table-dense result-table' }, h('tbody', null, imp.rows.filter(([k]) => task.result[k] != null).map(([k, label]) =>
              h('tr', null, h('th', { scope: 'row' }, label), h('td', { class: 'mono r' }, num(task.result[k])))))),
            h('a', { class: 'btn btn-sm', href: '/' }, 'See your stats', icon('chevronRight', 14)));
        } else if (task && task.state === 'error') {
          outcome = h('div', { class: 'import-result' }, inlineError(`import-task-err-${imp.key}`, `The import failed: ${task.error || task.message || 'unknown error'}`),
            h('p', { class: 'help' }, `Nothing was half-imported. Check that the file is an unmodified ${imp.name} backup, then upload it again.`));
        }
        stage = [drop,
          localErr[imp.key] ? inlineError(`import-local-err-${imp.key}`, localErr[imp.key]) : null,
          upload.error && mine ? inlineError(`import-upload-err-${imp.key}`, `Upload failed: ${upload.error}`) : null,
          outcome];
      }

      mount(slots[imp.key],
        h('div', { class: 'import-cols' },
          h('div', null, h('h3', { class: 'section-label' }, `Export from ${imp.name}`),
            h('ol', { class: 'steps' }, imp.steps.map((parts) => h('li', null, parts.map((part, i) => (i % 2 ? h('strong', null, part) : part)))))),
          h('div', null, h('h3', { class: 'section-label' }, 'Upload the backup'), fileInputs[imp.key], stage,
            h('p', { class: 'help import-note' }, icon('info', 13), ' ', imp.note))));
    }
    const paintAll = () => { for (const imp of IMPORTERS) paint(imp); };

    const onUpload = () => { paintAll(); if (!upload.active && upload.doneAt) store.poke(1000); };
    uploadSubs.add(onUpload);
    store.ctx.onCleanup(() => uploadSubs.delete(onUpload));
    store.hurry(() => (importing(store) || (upload.doneAt && Date.now() - upload.doneAt < 15000) ? 1000 : null));
    store.onTasks(paintAll);
    paintAll();
    store.loadTasks().catch(() => {});
  },
};
