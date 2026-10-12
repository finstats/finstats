// Settings → Access: who can use FinStats and what they can see. Jellyfin administrators only.
// Rows: "Everyone" (the defaults) and one per user. A switch takes effect at once. What everyone
// has is shown as on, and locked, on each person's row, because personal grants only ever add.

import { h, icon, mount } from '../dom.js';
import { api, isAbort } from '../api.js';
import { isAdmin } from '../state.js';
import { card, sk, toggle, spinner, inlineError, errorState, avatar } from '../components.js';
import { dataTable } from '../tables.js';
import { t } from '../i18n.js';

const PERMS = new Set(['sign_in', 'see_everyone', 'see_network', 'see_server', 'see_downloads', 'notify', 'manage']);
const permSaid = (pm, field) => (PERMS.has(pm.key) ? t(`settings.access.perm.${pm.key}.${field}`) : pm[field]);

export default {
  key: 'access', label: t('settings.sections.access.label'), sub: t('settings.sections.access.sub'), icon: 'users',
  visible: () => isAdmin(),
  entries: [{ id: 'access', label: t('settings.access.permissions'), hint: t('settings.access.permissions_hint') }],
  async render(slot, store) {
    const body = h('div', null, sk.rows(3));
    mount(slot, card({ title: t('settings.sections.access.label'), sub: t('settings.sections.access.sub'), body, id: 'access' }));
    let data;
    try { data = await api.get('/permissions', null, { signal: store.signal }); }
    catch (e) { if (isAbort(e) || e.status === 401) return; mount(body, errorState(e, () => this.render(slot, store))); return; }
    const perms = data.available || [];
    const admins = (data.users || []).filter((u) => u.is_admin);
    const people = (data.users || []).filter((u) => !u.is_admin);
    let defaults = new Set(data.defaults || []);
    const rowsEl = h('tbody');
    const problem = h('div');

    function row({ id, label, sub, granted, save }) {
      const note = h('span', { class: 'saved-note perm-note', 'aria-live': 'polite' });
      let timer;
      // One save at a time per row, each built from what the one before it saved: two switches flipped
      // before the first answer used to send the same starting list twice, and the second undid the first.
      let queue = Promise.resolve();
      const cells = perms.map((pm) => {
        const inherited = id !== null && defaults.has(pm.key);
        const sw = toggle({ checked: inherited || granted.has(pm.key), labelledby: `perm-h-${pm.key} perm-r-${id || 'all'}`,
          onChange: (next, revert) => { queue = queue.then(async () => {
            mount(problem, '');
            const want = new Set(granted); if (next) want.add(pm.key); else want.delete(pm.key);
            note.replaceChildren(spinner(12));
            try {
              await save([...want]);
              granted = want;
              note.replaceChildren(icon('check', 13), t('settings.access.saved'));
              clearTimeout(timer); timer = setTimeout(() => note.replaceChildren(), 1800);
              if (id === null) { defaults = want; paint(); } // everyone's rows inherit from this one
            } catch (e) {
              revert(!next); note.replaceChildren();
              mount(problem, inlineError('perm-err', t('settings.access.couldnt_save', { error: e.message })));
            }
          }); } });
        if (inherited) { sw.disabled = true; sw.classList.add('is-inherited'); sw.title = t('settings.access.inherited'); }
        return h('td', { class: 'perm-cell' }, sw);
      });
      return h('tr', { 'data-pin': id === null ? '' : null }, // "Everyone" stays on top however the people are sorted
        h('th', { scope: 'row', class: 'perm-who', id: `perm-r-${id || 'all'}` }, label, sub ? h('span', { class: 'perm-sub' }, sub) : null),
        cells, h('td', { class: 'perm-saved' }, note));
    }

    function paint() {
      mount(rowsEl,
        row({ id: null, label: h('span', { class: 'perm-name' }, t('common.everyone')), sub: t('settings.access.everyone_sub'), granted: new Set(defaults),
          save: (list) => api.put('/permissions/defaults', { permissions: list }) }),
        people.map((u) => row({ id: u.id,
          label: h('span', { class: 'user-cell' }, avatar(u.id, u.name, { size: 24, hasImage: u.has_image }), h('span', { class: 'perm-name' }, u.name)),
          sub: u.is_disabled ? t('settings.access.disabled') : null, granted: new Set(u.permissions || []),
          save: async (list) => { const r = await api.put(`/permissions/users/${u.id}`, { permissions: list }); u.permissions = r.permissions; } })));
    }
    paint();

    mount(body,
      h('p', { class: 'fui-field__help perm-intro' }, admins.length ? t('settings.access.intro_named', { names: admins.map((u) => u.name).join(', ') }) : t('settings.access.intro')),
      dataTable(
        h('table', { class: 'perm-table' },
          h('thead', null, h('tr', null, h('th', { scope: 'col' }, t('settings.access.who')),
            perms.map((pm) => h('th', { scope: 'col', id: `perm-h-${pm.key}`, title: permSaid(pm, 'description') }, permSaid(pm, 'label'))), h('th', { 'data-nosort': '' }, h('span', { class: 'sr-only' }, t('settings.access.status'))))),
          rowsEl)),
      problem,
      h('dl', { class: 'perm-legend' }, perms.map((pm) => [h('dt', null, permSaid(pm, 'label')), h('dd', null, permSaid(pm, 'description'))])));
  },
};
