// Settings → Backups: the history, settings and permissions in one file. Jellyfin administrators only.

import { h, icon, bytes, relTime, dateTime, mount, untilText } from '../dom.js';
import { api, isAbort, uploadRaw } from '../api.js';
import { isAdmin } from '../state.js';
import { card, sk, setBusy, inlineError, errorState } from '../components.js';
import { plainTable } from '../tables.js';
import { progressOf, settingRow } from './common.js';
import { button } from '../../finui/components/button/button.js';
import { toast } from '../../finui/components/toast/toast.js';
import { t, tParts } from '../i18n.js';

export default {
  key: 'backups', label: t('settings.sections.backups.label'), sub: t('settings.sections.backups.sub'), group: t('settings.groups.data'), icon: 'database',
  visible: () => isAdmin(),
  entries: [
    { id: 'backups', label: t('settings.backups.back_up_now'), hint: t('settings.backups.hint.backups') },
    { id: 'restore', label: t('settings.backups.restore_title'), hint: t('settings.backups.hint.restore') },
    { id: 'backup-schedule', label: t('settings.backups.when_written'), hint: t('settings.backups.hint.schedule') },
    { id: 'backup_keep', label: t('settings.backups.keep_newest'), hint: t('settings.backups.hint.keep') },
    { id: 'backups-deleted', label: t('settings.backups.recently_deleted'), hint: t('settings.backups.hint.deleted') },
  ],
  async render(slot, store) {
    const body = h('div', { class: 'net-stack' }, sk.rows(2));
    mount(slot, card({ title: t('settings.sections.backups.label'), sub: t('settings.backups.card_sub'), body, id: 'backups' }));
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
    // Clears only the question it answered: a question asked meanwhile (Delete pressed while "Back up now" was still
    // finishing) used to vanish under the pointer when the earlier action ended.
    const act = async (fn) => {
      const answering = pending;
      err = null;
      try { await fn(); } catch (e) { err = e.message; }
      if (pending === answering) pending = null;
      await store.poke(1000); await loadBackups();
    };
    // Into the trash, and the toast that takes it back out; and back out from the Recently deleted list.
    const trash = (name, deleted) => api.put(`/backups/${name}`, { deleted });
    const moveToTrash = (b) => act(async () => {
      await trash(b.name, true);
      toast({ text: t('settings.backups.deleted_toast', { date: dateTime(b.created_at) }), action: t('settings.backups.undo'),
        onAction: () => act(() => trash(b.name, false)) });
    });

    function scheduleForm() {
      const s = store.settings;
      const keep = h('input', { class: 'fui-field__input fui-field__input--num mono', type: 'text', inputMode: 'numeric', id: 'f-backup-keep', value: String(s.backup_keep), autocomplete: 'off', 'aria-describedby': 'backup_keep-help' });
      const formErr = h('div'), note = h('span', { class: 'saved-note', 'aria-live': 'polite' });
      const save = button({ variant: 'primary', type: 'submit' }, t('common.save'));
      const form = h('form', { class: 'fui-setting-row__rows', noValidate: true },
        settingRow({ id: 'backup-schedule', label: t('settings.backups.when_written'), help: t('settings.backups.when_written_help'),
          control: button({ size: 'sm', href: '/settings/tasks/backup' }, icon('clock', 13), t('settings.backups.schedule')) }),
        settingRow({ id: 'backup_keep', label: t('settings.backups.keep_newest'), labelFor: 'f-backup-keep', help: t('settings.backups.keep_help'),
          control: h('div', { class: 'fui-field__row' }, keep, h('span', { class: 'unit' }, t('settings.backups.keep_unit'))) }),
        h('div', { class: 'form-actions fui-setting-row__actions' }, save, note), formErr);
      form.addEventListener('submit', async (e) => {
        e.preventDefault(); mount(formErr, '');
        const k = Number(keep.value.trim());
        if (!/^\d+$/.test(keep.value.trim()) || k < 1 || k > 100) { mount(formErr, inlineError('bk-e', t('settings.backups.keep_invalid'))); keep.focus(); return; }
        setBusy(save, true, t('settings.backups.saving'));
        try { await store.put({ backup_keep: k }); await loadBackups(); }
        catch (e2) { mount(formErr, inlineError('bk-e', t('settings.backups.couldnt_save', { error: e2.message }))); }
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
      const table = rows.length ? plainTable(h('table', { class: 'fui-data-table backups' },
        h('thead', null, h('tr', null, h('th', null, t('settings.backups.made')), h('th', { class: 'r' }, t('settings.backups.size')), h('th', { 'data-nosort': '' }, h('span', { class: 'sr-only' }, t('settings.backups.actions'))))),
        h('tbody', null, rows.map((b) => {
          const mine = pending && pending.name === b.name ? pending.action : null;
          const ask = (action) => () => { pending = { name: b.name, action }; sig = ''; paint(); };
          const cancel = button({ size: 'sm', variant: 'ghost', type: 'button', onClick: () => { pending = null; sig = ''; paint(); } }, t('common.cancel'));
          const actions = mine === 'delete'
            ? [h('span', { class: 'muted' }, t('settings.backups.delete_ask')), button({ size: 'sm', variant: 'danger', type: 'button', onClick: () => moveToTrash(b) }, t('settings.backups.delete')), cancel]
            : mine === 'restore'
              ? [h('span', { class: 'muted' }, restoreSettings ? t('settings.backups.restore_ask_settings') : t('settings.backups.restore_ask')),
                button({ size: 'sm', variant: 'primary', type: 'button', onClick: () => act(() => api.post(`/backups/${b.name}/restore?settings=${restoreSettings}`)) }, t('settings.backups.restore')), cancel]
              : [button({ size: 'sm', href: `/api/backups/${b.name}`, download: b.name }, icon('upload', 12, 'flip-v'), t('settings.backups.download')),
                button({ size: 'sm', variant: 'ghost', type: 'button', disabled: busy, onClick: ask('restore') }, t('settings.backups.restore')),
                button({ variant: 'icon', type: 'button', 'aria-label': t('settings.backups.delete_aria', { date: dateTime(b.created_at) }), title: t('settings.backups.delete'), disabled: busy, onClick: ask('delete') }, icon('trash', 14))];
          return h('tr', null,
            h('td', null, h('span', { class: 'when-cell' }, h('time', { dateTime: new Date(b.created_at * 1000).toISOString(), title: b.name }, dateTime(b.created_at)), h('span', { class: 'cell-sub mono' }, relTime(b.created_at)))),
            h('td', { class: 'mono r' }, bytes(b.size_bytes)),
            h('td', null, h('div', { class: 'backup-actions' }, actions)));
        }))))
        : h('p', { class: 'fui-field__help' }, backupsData.scheduled ? t('settings.backups.empty_scheduled') : t('settings.backups.empty_unscheduled'));

      // Recently deleted: shown only while it holds something.
      const gone = backupsData.deleted || [];
      const deletedList = gone.length ? h('div', { class: 'fui-field', id: 'backups-deleted' },
        h('div', { class: 'fui-setting-row__label' }, t('settings.backups.recently_deleted')),
        h('p', { class: 'fui-field__help' }, t('settings.backups.deleted_help')),
        plainTable(h('table', { class: 'fui-data-table backups backups-deleted' },
          h('thead', null, h('tr', null, h('th', null, t('settings.backups.made')), h('th', null, t('settings.backups.removed_for_good')), h('th', { class: 'r' }, t('settings.backups.size')), h('th', { 'data-nosort': '' }, h('span', { class: 'sr-only' }, t('settings.backups.actions'))))),
          h('tbody', null, gone.map((b) => h('tr', null,
            h('td', null, h('time', { dateTime: new Date(b.created_at * 1000).toISOString(), title: b.name }, dateTime(b.created_at))),
            h('td', null, h('time', { dateTime: new Date(b.purge_at * 1000).toISOString() }, dateTime(b.purge_at))),
            h('td', { class: 'mono r' }, bytes(b.size_bytes)),
            h('td', null, h('div', { class: 'backup-actions' }, button({ size: 'sm', type: 'button', disabled: busy, onClick: () => act(() => trash(b.name, false)) }, icon('refresh', 13), t('settings.backups.undelete')))))))))) : null;

      const restored = rs && rs.state === 'ok' && rs.result ? h('p', { class: 'fui-badge--status fui-badge--good fui-badge--line' }, icon('check', 13),
        t(rs.result.settings_restored ? 'settings.backups.restored_settings' : 'settings.backups.restored', { count: rs.result.plays_imported, skipped: rs.result.plays_skipped })) : null;
      const failed = rs && rs.state === 'error' && rs.error ? inlineError('restore-err', t('settings.backups.restore_failed', { error: rs.error })) : null;

      const makeNow = button({ type: 'button', disabled: busy, onClick: () => act(() => api.post('/backups')) }, icon('plus', 13), t('settings.backups.back_up_now'));
      const file = h('input', { type: 'file', class: 'sr-only', id: 'restore-file', accept: '.gz,.jsonl,application/gzip', tabindex: -1 });
      file.addEventListener('change', () => {
        const f = file.files && file.files[0];
        if (!f) return;
        Object.assign(restoreUpload, { active: true, progress: 0, name: f.name }); err = null; sig = ''; paint();
        uploadRaw(`/backups/restore?settings=${restoreSettings}`, f, (p) => { restoreUpload.progress = p; sig = ''; paint(); }).promise
          .catch((e) => { err = e.status === 413 ? t('settings.backups.too_large') : e.message; })
          .finally(() => { restoreUpload.active = false; sig = ''; store.poke(1000); paint(); });
      });
      const keepSettings = h('label', { class: 'fui-field__check' }, h('input', { type: 'checkbox', checked: restoreSettings, onChange: (e) => { restoreSettings = e.target.checked; sig = ''; paint(); } }),
        h('span', null, t('settings.backups.also_settings')));

      mount(body,
        h('div', { class: 'backup-head' },
          h('p', { class: 'fui-field__help' }, backupsData.scheduled
            ? tParts(`settings.backups.${s.backup_keep === 1 ? 'head_single' : 'head'}${backupsData.next_at ? '_next' : ''}`, { link: h('a', { href: '/settings/tasks/backup' }, t('settings.backups.head_link')), count: s.backup_keep,
              next: backupsData.next_at ? h('span', { title: dateTime(backupsData.next_at) }, untilText(backupsData.next_at)) : '' })
            : tParts('settings.backups.head_unscheduled', { link: h('a', { href: '/settings/tasks/backup' }, t('settings.backups.head_unscheduled_link')) }), ' ', ...tParts('settings.backups.folder', { folder: h('span', { class: 'mono' }, 'backups') })),
          makeNow),
        progressOf(bk, t('settings.backups.backup_progress')), bk && bk.state === 'error' && bk.error ? inlineError('backup-err', t('settings.backups.backup_failed', { error: bk.error })) : null,
        table,
        err ? inlineError('backups-err', err) : null,
        deletedList,
        h('div', { class: 'fui-field', id: 'restore' },
          h('div', { class: 'fui-setting-row__label' }, t('settings.backups.restore_title')),
          h('p', { class: 'fui-field__help' }, t('settings.backups.restore_help')),
          restoreUpload.active
            ? h('div', { class: 'task-progress' }, h('div', { class: 'fui-meter fui-meter--wide', role: 'progressbar', 'aria-label': t('settings.backups.upload_progress'), 'aria-valuemin': 0, 'aria-valuemax': 100, 'aria-valuenow': Math.round(restoreUpload.progress * 100) },
              h('span', { class: 'fui-meter__fill', style: { width: restoreUpload.progress * 100 + '%' } })), h('span', { class: 'mono task-msg' }, t('settings.backups.uploading', { file: restoreUpload.name, percent: Math.round(restoreUpload.progress * 100) })))
            : h('div', { class: 'backup-restore' }, file, button({ tag: 'label', disabled: busy, htmlFor: busy ? null : 'restore-file' }, icon('upload', 13), t('settings.backups.choose_file')), keepSettings),
          progressOf(rs, t('settings.backups.restore_progress')), restored, failed),
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
