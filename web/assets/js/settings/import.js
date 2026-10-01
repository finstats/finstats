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

// The trackers finstats can take history from.
export const IMPORTERS = [
  {
    key: 'jellystat', task: 'import', name: 'Jellystat', endpoint: '/import/jellystat', file: 'the .jsonl file', accept: '.jsonl,.json,application/json', pattern: /\.(jsonl|json)$/i,
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
    key: 'streamystats', task: 'import_streamystats', name: 'Streamystats', endpoint: '/import/streamystats', file: 'the .json file', accept: '.json,application/json', pattern: /\.json$/i,
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
  {
    key: 'tautulli', task: 'import_tautulli', name: 'Tautulli', endpoint: '/import/tautulli', file: 'the .db or .zip file', accept: '.db,.zip,application/zip', pattern: /\.(db|zip)$/i,
    board: true,
    title: 'Import from Tautulli', sub: 'Bring your Plex history over, person by person',
    steps: [
      ['Open your Tautulli instance.'],
      ['Go to ', 'Settings', ' and select ', 'Import & Backups', '.'],
      ['Under the database backups, click ', 'Backup Database', '.'],
      ['Download the newest backup — a ', '.db', ' file, or a ', '.zip', ' holding one.'],
      ['Upload it here, then connect each Plex user to who they are on Jellyfin.'],
    ],
    rows: [['plays_imported', 'Plays imported'], ['plays_skipped', 'Already here'], ['users_wired', 'Plex users connected'],
      ['not_wired', 'Left behind, without a wire'], ['other_media', 'Music and other media, not imported']],
    note: 'Plex names rarely match Jellyfin’s, so you connect each Plex user yourself. Films and episodes are matched by name; one not on your server yet is kept, and attached when it arrives. The backup holds Plex’s access tokens: finstats never reads them, and removes the file once you import or start over.',
  },
];

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
    // What an import could not match is linked under Settings → Unlinked media: said here while there is any.
    const pointer = h('div');
    const pointerBox = card({ title: 'Unlinked media', body: pointer, id: 'import-unlinked' });
    pointerBox.hidden = true;
    mount(slot, IMPORTERS.map((i) => card({ title: i.title, sub: i.sub, body: slots[i.key], id: `import-${i.key}` })), pointerBox);
    async function unlinked() {
      let n = 0;
      try { n = ((await api.get('/library/missing')).missing || []).length; } catch { /* nothing to point at */ }
      pointerBox.hidden = !n;
      mount(pointer, n ? h('p', { class: 'help' }, `${num(n)} title${n === 1 ? '' : 's'} in your history ${n === 1 ? 'doesn’t' : 'don’t'} match anything in your library — usually a name the other server used. `,
        h('a', { href: '/settings/unlinked' }, 'Link them under Unlinked media'), '.') : null);
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
      if (!imp.pattern.test(file.name)) localErr[imp.key] = `“${file.name}” isn’t a ${imp.name} backup. Choose ${imp.file} you downloaded from ${imp.name}.`;
      else if (!file.size) localErr[imp.key] = `That file is empty. Download the backup from ${imp.name} again.`;
      if (localErr[imp.key]) { paintAll(); return; }
      sawRunning[imp.key] = false;
      startUpload(file, imp);
      store.poke(1000);
    }

    // The board is kept across repaints — a poll must not throw away the wires being drawn — and torn down when it goes.
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
          h('div', { class: 'import-stage-title' }, `Uploading ${upload.fileName}`),
          h('div', { class: 'meter meter-wide', role: 'progressbar', 'aria-label': 'Upload progress', 'aria-valuemin': 0, 'aria-valuemax': 100, 'aria-valuenow': Math.round(upload.progress * 100) },
            h('span', { class: 'meter-fill', style: { width: upload.progress * 100 + '%' } })),
          h('div', { class: 'import-stage-row' }, h('span', { class: 'mono' }, `${Math.round(upload.progress * 100)}% · ${bytes(upload.loaded)} of ${bytes(upload.total)}`),
            button({ size: 'sm', type: 'button', onClick: () => upload.handle && upload.handle.abort() }, 'Cancel upload')),
          h('p', { class: 'help' }, 'Keep this tab open until the upload finishes. You can browse other finstats pages meanwhile.'));
      } else if (waiting) {
        stage = boardView(imp);
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
            button({ size: 'sm', href: '/' }, 'See your stats', icon('chevronRight', 14)));
        } else if (task && task.state === 'error') {
          outcome = h('div', { class: 'import-result' }, inlineError(`import-task-err-${imp.key}`, `The import failed: ${task.error || task.message || 'unknown error'}`),
            h('p', { class: 'help' }, `Nothing was half-imported. Check that the file is an unmodified ${imp.name} backup, then upload it again.`));
        }
        stage = [drop,
          localErr[imp.key] ? inlineError(`import-local-err-${imp.key}`, localErr[imp.key]) : null,
          upload.error && mine ? inlineError(`import-upload-err-${imp.key}`, `Upload failed: ${upload.error}`) : null,
          outcome];
      }

      // The board takes the whole card: the wires need the width, and the export steps are done by now.
      if (waiting) { mount(slots[imp.key], h('h3', { class: 'section-label' }, 'Connect the wires'), stage); return; }
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
    // A board left waiting — the page closed half-way through the wiring — is picked up where it was.
    for (const imp of IMPORTERS.filter((i) => i.board)) {
      api.get(imp.endpoint).then((d) => { boards[imp.key] = d.board || null; paintAll(); }, () => {});
    }
  },
};
