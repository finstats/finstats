// Settings → Access: who can use finstats and what they can see. Jellyfin administrators only.
// Rows: "Everyone" (the defaults) and one per user. A switch takes effect at once. What everyone
// has is shown as on, and locked, on each person's row, because personal grants only ever add.

import { h, icon, mount } from '../dom.js';
import { api, isAbort } from '../api.js';
import { isAdmin } from '../state.js';
import { card, sk, toggle, spinner, inlineError, errorState, avatar } from '../components.js';
import { dataTable } from '../tables.js';

export default {
  key: 'access', label: 'Access', sub: 'Who can use finstats and what they can see', icon: 'users',
  visible: () => isAdmin(),
  entries: [{ id: 'access', label: 'Permissions', hint: 'who can see everyone network server downloads notify manage sign in users' }],
  async render(slot, store) {
    const body = h('div', null, sk.rows(3));
    mount(slot, card({ title: 'Access', sub: 'Who can use finstats and what they can see', body, id: 'access' }));
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
              note.replaceChildren(icon('check', 13), 'Saved');
              clearTimeout(timer); timer = setTimeout(() => note.replaceChildren(), 1800);
              if (id === null) { defaults = want; paint(); } // everyone's rows inherit from this one
            } catch (e) {
              revert(!next); note.replaceChildren();
              mount(problem, inlineError('perm-err', `Couldn’t save: ${e.message}`));
            }
          }); } });
        if (inherited) { sw.disabled = true; sw.classList.add('is-inherited'); sw.title = 'Everyone has this, so it can’t be taken away from one person'; }
        return h('td', { class: 'perm-cell' }, sw);
      });
      return h('tr', { 'data-pin': id === null ? '' : null }, // "Everyone" stays on top however the people are sorted
        h('th', { scope: 'row', class: 'perm-who', id: `perm-r-${id || 'all'}` }, label, sub ? h('span', { class: 'perm-sub' }, sub) : null),
        cells, h('td', { class: 'perm-saved' }, note));
    }

    function paint() {
      mount(rowsEl,
        row({ id: null, label: h('span', { class: 'perm-name' }, 'Everyone'), sub: 'Applies to every Jellyfin user', granted: new Set(defaults),
          save: (list) => api.put('/permissions/defaults', { permissions: list }) }),
        people.map((u) => row({ id: u.id,
          label: h('span', { class: 'user-cell' }, avatar(u.id, u.name, { size: 24, hasImage: u.has_image }), h('span', { class: 'perm-name' }, u.name)),
          sub: u.is_disabled ? 'Disabled in Jellyfin' : null, granted: new Set(u.permissions || []),
          save: async (list) => { const r = await api.put(`/permissions/users/${u.id}`, { permissions: list }); u.permissions = r.permissions; } })));
    }
    paint();

    mount(body,
      h('p', { class: 'help perm-intro' }, 'Jellyfin administrators', admins.length ? [' (', admins.map((u) => u.name).join(', '), ')'] : null,
        ' always have full access. Everyone else gets what you switch on here; recaps stay each person’s own.'),
      dataTable(
        h('table', { class: 'perm-table' },
          h('thead', null, h('tr', null, h('th', { scope: 'col' }, 'Who'),
            perms.map((pm) => h('th', { scope: 'col', id: `perm-h-${pm.key}`, title: pm.description }, pm.label)), h('th', { 'data-nosort': '' }, h('span', { class: 'sr-only' }, 'Status')))),
          rowsEl)),
      problem,
      h('dl', { class: 'perm-legend' }, perms.map((pm) => [h('dt', null, pm.label), h('dd', null, pm.description)])));
  },
};
