// Settings → Backups: the history, settings and permissions in one file. Jellyfin administrators only.

import { h, icon, num, bytes, relTime, dateTime, mount, untilText } from '../dom.js';
import { api, isAbort, uploadRaw } from '../api.js';
import { isAdmin } from '../state.js';
import { card, sk, setBusy, inlineError, errorState } from '../components.js';
import { plainTable } from '../tables.js';
import { progressOf, settingRow } from './common.js';
import { button } from '../../finui/components/button/button.js';

export default {
  key: 'backups', label: 'Backups', sub: 'Keep your history safe, or move it', group: 'Data', icon: 'database',
  visible: () => isAdmin(),
  entries: [
    { id: 'backups', label: 'Back up now', hint: 'backups download delete list' },
    { id: 'restore', label: 'Restore from a file', hint: 'restore upload move new install merge' },
    { id: 'backup-schedule', label: 'When backups are written', hint: 'schedule automatic every days weekly daily task' },
    { id: 'backup_keep', label: 'Keep the newest', hint: 'schedule keep count prune' },
  ],
  async render(slot, store) {
    const body = h('div', { class: 'net-stack' }, sk.rows(2));
    mount(slot, card({ title: 'Backups', sub: 'Your history, settings and permissions in one file, to keep safe or to move to another finstats', body, id: 'backups' }));
    await store.loadSettings();

    let backupsData = null;
    let pending = null;            // {name, action: 'restore' | 'delete'}: waiting for the second click
    let err = null;
    let restoreSettings = true;
    const restoreUpload = { active: false, progress: 0, name: '' };
    const wasRunning = { backup: false, restore: false };
    let sig = '';

    async function loadBackups() {
      try { backupsData = await api.get('/backups', null, { signal: store.signal }); sig = ''; paint(); }
      catch (e) { if (isAbort(e) || e.status === 401) return; mount(body, errorState(e, loadBackups)); }
    }
    const act = async (fn) => { err = null; try { await fn(); } catch (e) { err = e.message; } pending = null; await store.poke(1000); await loadBackups(); };

    function scheduleForm() {
      const s = store.settings;
      const keep = h('input', { class: 'input input-num mono', type: 'text', inputMode: 'numeric', id: 'f-backup-keep', value: String(s.backup_keep), autocomplete: 'off', 'aria-describedby': 'backup_keep-help' });
      const formErr = h('div'), note = h('span', { class: 'saved-note', 'aria-live': 'polite' });
      const save = button({ variant: 'primary', type: 'submit' }, 'Save');
      const form = h('form', { class: 'setting-rows', noValidate: true },
        settingRow({ id: 'backup-schedule', label: 'When backups are written', help: 'The Backup task’s schedule: daily, weekly, on an interval, or only by hand.',
          control: button({ size: 'sm', href: '/settings/tasks/backup' }, icon('clock', 13), 'Schedule') }),
        settingRow({ id: 'backup_keep', label: 'Keep the newest', labelFor: 'f-backup-keep', help: 'Older ones are removed when a new one is written. 1–100.',
          control: h('div', { class: 'field-input' }, keep, h('span', { class: 'unit' }, 'backups')) }),
        h('div', { class: 'form-actions setting-actions' }, save, note), formErr);
      form.addEventListener('submit', async (e) => {
        e.preventDefault(); mount(formErr, '');
        const k = Number(keep.value.trim());
        if (!/^\d+$/.test(keep.value.trim()) || k < 1 || k > 100) { mount(formErr, inlineError('bk-e', 'Keep must be a whole number from 1 to 100.')); keep.focus(); return; }
        setBusy(save, true, 'Saving…');
        try { await store.put({ backup_keep: k }); await loadBackups(); }
        catch (e2) { mount(formErr, inlineError('bk-e', `Couldn’t save: ${e2.message}`)); }
        finally { setBusy(save, false); }
      });
      return form;
    }

    function paint() {
      if (!backupsData) return;
      const s = store.settings;
      const bk = store.task('backup'), rs = store.task('restore');
      const busy = (bk && bk.state === 'running') || (rs && rs.state === 'running') || restoreUpload.active;
      const now = JSON.stringify([backupsData, bk, rs, pending, err, restoreSettings, restoreUpload, s.backup_keep]);
      if (now === sig) return;
      sig = now;

      const rows = backupsData.backups || [];
      const table = rows.length ? plainTable(h('table', { class: 'table backups' },
        h('thead', null, h('tr', null, h('th', null, 'Made'), h('th', { class: 'r' }, 'Size'), h('th', { 'data-nosort': '' }, h('span', { class: 'sr-only' }, 'Actions')))),
        h('tbody', null, rows.map((b) => {
          const mine = pending && pending.name === b.name ? pending.action : null;
          const ask = (action) => () => { pending = { name: b.name, action }; sig = ''; paint(); };
          const cancel = button({ size: 'sm', variant: 'ghost', type: 'button', onClick: () => { pending = null; sig = ''; paint(); } }, 'Cancel');
          const actions = mine === 'delete'
            ? [h('span', { class: 'muted' }, 'Delete this backup?'), button({ size: 'sm', variant: 'danger', type: 'button', onClick: () => act(() => api.del(`/backups/${b.name}`)) }, 'Delete'), cancel]
            : mine === 'restore'
              ? [h('span', { class: 'muted' }, restoreSettings ? 'Merge its history in and replace settings and permissions?' : 'Merge its history in?'),
                button({ size: 'sm', variant: 'primary', type: 'button', onClick: () => act(() => api.post(`/backups/${b.name}/restore?settings=${restoreSettings}`)) }, 'Restore'), cancel]
              : [button({ size: 'sm', href: `/api/backups/${b.name}`, download: b.name }, icon('upload', 12, 'flip-v'), 'Download'),
                button({ size: 'sm', variant: 'ghost', type: 'button', disabled: busy, onClick: ask('restore') }, 'Restore'),
                button({ variant: 'icon', type: 'button', 'aria-label': `Delete the backup from ${dateTime(b.created_at)}`, title: 'Delete', disabled: busy, onClick: ask('delete') }, icon('trash', 14))];
          return h('tr', null,
            h('td', null, h('span', { class: 'when-cell' }, h('time', { dateTime: new Date(b.created_at * 1000).toISOString(), title: b.name }, dateTime(b.created_at)), h('span', { class: 'cell-sub mono' }, relTime(b.created_at)))),
            h('td', { class: 'mono r' }, bytes(b.size_bytes)),
            h('td', null, h('div', { class: 'backup-actions' }, actions)));
        }))))
        : h('p', { class: 'help' }, backupsData.scheduled ? 'No backups yet. The first one is written by itself once there is something to back up, or make one now.' : 'No backups yet, and none are scheduled.');

      const restored = rs && rs.state === 'ok' && rs.result ? h('p', { class: 'sev sev-good sev-line' }, icon('check', 13),
        `Restored ${num(rs.result.plays_imported)} plays, ${num(rs.result.plays_skipped)} were already here${rs.result.settings_restored ? '; settings and permissions restored' : ''}.`) : null;
      const failed = rs && rs.state === 'error' && rs.error ? inlineError('restore-err', `Restore failed: ${rs.error} Nothing was changed.`) : null;

      const makeNow = button({ type: 'button', disabled: busy, onClick: () => act(() => api.post('/backups')) }, icon('plus', 13), 'Back up now');
      const file = h('input', { type: 'file', class: 'sr-only', id: 'restore-file', accept: '.gz,.jsonl,application/gzip', tabindex: -1 });
      file.addEventListener('change', () => {
        const f = file.files && file.files[0];
        if (!f) return;
        Object.assign(restoreUpload, { active: true, progress: 0, name: f.name }); err = null; sig = ''; paint();
        uploadRaw(`/backups/restore?settings=${restoreSettings}`, f, (p) => { restoreUpload.progress = p; sig = ''; paint(); }).promise
          .catch((e) => { err = e.status === 413 ? 'The server rejected the file as too large. Behind a reverse proxy, raise its upload limit and try again.' : e.message; })
          .finally(() => { restoreUpload.active = false; sig = ''; store.poke(1000); paint(); });
      });
      const keepSettings = h('label', { class: 'check' }, h('input', { type: 'checkbox', checked: restoreSettings, onChange: (e) => { restoreSettings = e.target.checked; sig = ''; paint(); } }),
        h('span', null, 'Also restore settings and permissions'));

      mount(body,
        h('div', { class: 'backup-head' },
          h('p', { class: 'help' }, backupsData.scheduled
            ? [`Written on the `, h('a', { href: '/settings/tasks/backup' }, 'Backup task’s schedule'), `, the newest ${num(s.backup_keep)} kept`,
              backupsData.next_at ? [', next ', h('span', { title: dateTime(backupsData.next_at) }, untilText(backupsData.next_at)), '.'] : '.']
            : ['No backups are scheduled (', h('a', { href: '/settings/tasks/backup' }, 'schedule them'), ').'], ' They live in the ', h('span', { class: 'mono' }, 'backups'), ' folder of your data directory.'),
          makeNow),
        progressOf(bk, 'Backup progress'), bk && bk.state === 'error' && bk.error ? inlineError('backup-err', `Backup failed: ${bk.error}`) : null,
        table,
        err ? inlineError('backups-err', err) : null,
        h('div', { class: 'field', id: 'restore' },
          h('div', { class: 'setting-label' }, 'Restore from a file'),
          h('p', { class: 'help' }, 'Restoring merges: plays already here are skipped, so it is safe to do twice. A backup holds everyone’s history and addresses, never your Jellyfin API key.'),
          restoreUpload.active
            ? h('div', { class: 'task-progress' }, h('div', { class: 'meter meter-wide', role: 'progressbar', 'aria-label': 'Upload progress', 'aria-valuemin': 0, 'aria-valuemax': 100, 'aria-valuenow': Math.round(restoreUpload.progress * 100) },
              h('span', { class: 'meter-fill', style: { width: restoreUpload.progress * 100 + '%' } })), h('span', { class: 'mono task-msg' }, `Uploading ${restoreUpload.name} · ${Math.round(restoreUpload.progress * 100)}%`))
            : h('div', { class: 'backup-restore' }, file, button({ tag: 'label', disabled: busy, htmlFor: busy ? null : 'restore-file' }, icon('upload', 13), 'Choose a backup file…'), keepSettings),
          progressOf(rs, 'Restore progress'), restored, failed),
        scheduleForm());
    }

    // The list changes when a backup finishes, and nearly everything changes when a restore does.
    store.onTasks(() => {
      for (const id of ['backup', 'restore']) {
        const t = store.task(id);
        const running = !!t && t.state === 'running';
        if (wasRunning[id] && !running) { loadBackups(); if (id === 'restore') store.reloadSettings().then(() => { sig = ''; paint(); }).catch(() => {}); }
        wasRunning[id] = running;
      }
      paint();
    });
    await Promise.all([loadBackups(), store.loadTasks().catch(() => {})]);
  },
};
