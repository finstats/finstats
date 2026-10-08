// Settings → API keys: use FinStats from outside. A key carries exactly its maker's permissions, is
// shown once when it is made, and dies with their access. A calendar key opens the calendar feed and
// nothing else, which is why it may travel in a subscription address. Administrators see everyone's.

import { h, icon, num, mount, relEl, dateTime } from '../dom.js';
import { api, isAbort } from '../api.js';
import { isAdmin } from '../state.js';
import { card, sk, setBusy, inlineError, errorState, formField, segmented, copyButton, avatar } from '../components.js';
import { plainTable } from '../tables.js';
import { button } from '../../finui/components/button/button.js';

const SCOPE_LABEL = { full: 'Everything you may see', calendar: 'Calendar feed only' };
const EXPIRY = [['', 'Never'], ['30', '30 days'], ['90', '90 days'], ['365', 'A year']];

export default {
  key: 'keys', label: 'API keys', sub: 'Use FinStats from outside: scripts, and a calendar to subscribe to', group: 'Account', icon: 'link',
  visible: () => true,
  entries: [
    { id: 'keys', label: 'API keys', hint: 'token bearer script curl calendar ics subscribe revoke' },
    { id: 'key-new', label: 'Make a key', hint: 'create new token calendar feed' },
  ],
  async render(slot, store) {
    const body = h('div', { class: 'net-stack' }, sk.rows(2));
    mount(slot, card({ title: 'API keys', sub: 'A key does what you may do, no more, and stops the moment your access does', body, id: 'keys' }));
    let data = null, revealed = null, pending = null, err = null, scope = 'full';
    const settings = await store.loadSettings().catch(() => null);
    const base = (settings && settings.public_url ? settings.public_url.replace(/\/+$/, '') : '');

    async function load() {
      try { data = await api.get('/keys', null, { signal: store.signal }); }
      catch (e) { if (isAbort(e) || e.status === 401) return; mount(body, errorState(e, load)); return; }
      paint();
    }

    function feedUrl(key) { return `${base || location.origin}/api/calendar.ics?key=${encodeURIComponent(key)}`; }

    function revealBox(k) {
      const isCal = k.scope === 'calendar';
      return h('div', { class: 'key-reveal', role: 'status' },
        h('div', { class: 'fui-setting-row__label' }, `Your new key “${k.name}”`),
        h('p', { class: 'fui-field__help' }, 'Shown once. Copy it now; FinStats keeps only a hash of it.'),
        h('div', { class: 'key-line' }, h('code', { class: 'mono key-token' }, k.key), copyButton(k.key, 'Copy key')),
        isCal ? [h('p', { class: 'fui-field__help' }, 'Subscribe your calendar to this address. It opens the feed and nothing else.'),
          h('div', { class: 'key-line' }, h('code', { class: 'mono key-token' }, feedUrl(k.key)), copyButton(feedUrl(k.key), 'Copy address')),
          settings && !base ? h('p', { class: 'fui-field__help' }, 'FinStats does not know its own address yet, so this one only works from inside. Set it under Notifications for a link that works from your phone.') : null]
        : h('p', { class: 'fui-field__help' }, ['Send it as ', h('code', { class: 'mono' }, 'Authorization: Bearer ' + k.key.slice(0, 7) + '…'), ' with every request.']),
        h('div', { class: 'form-actions' }, button({ size: 'sm', type: 'button', onClick: () => { revealed = null; paint(); } }, 'Done')));
    }

    function makeForm() {
      const name = formField({ id: 'key-name', label: 'Name', autocomplete: 'off', placeholder: 'Laptop script, Phone calendar…' });
      const expiry = h('select', { class: 'fui-field__input', id: 'key-expiry', 'aria-label': 'Expires' }, EXPIRY.map(([v, l]) => h('option', { value: v }, l)));
      // One sentence per scope, from one place: the form is rebuilt after a key is made and the choice survives it.
      const scopeHelp = (v) => SCOPE_LABEL[v] + (v === 'calendar' ? ': a key you can put in a subscription address safely.' : ': the same as you signed in, for scripts and other tools.');
      const scopePick = segmented({ label: 'What it opens', value: scope, options: [{ value: 'full', label: 'Everything' }, { value: 'calendar', label: 'Calendar only' }], onChange: (v) => { scope = v; help.textContent = scopeHelp(v); } });
      const help = h('p', { class: 'fui-field__help' }, scopeHelp(scope));
      const save = button({ variant: 'primary', type: 'submit' }, icon('plus', 13), 'Make a key');
      const formErr = h('div');
      const form = h('form', { class: 'key-form', id: 'key-new', noValidate: true },
        h('div', { class: 'key-form-row' }, name.el, h('div', { class: 'fui-field' }, h('label', { class: 'fui-field__label', htmlFor: 'key-expiry' }, 'Expires'), expiry)),
        h('div', { class: 'fui-field' }, h('div', { class: 'fui-field__label' }, 'What it opens'), scopePick, help),
        h('div', { class: 'form-actions' }, save), formErr);
      form.addEventListener('submit', async (e) => {
        e.preventDefault(); mount(formErr, ''); name.setError('');
        const n = name.input.value.trim();
        if (!n) { name.setError('Give the key a name.'); name.input.focus(); return; }
        setBusy(save, true, 'Making…');
        try {
          const body = { name: n, scope };
          if (expiry.value) body.expires_in_d = Number(expiry.value);
          revealed = await api.post('/keys', body);
          await load();
        } catch (e2) { mount(formErr, inlineError('key-err', e2.message)); }
        finally { setBusy(save, false); }
      });
      return form;
    }

    function table(rows) {
      const admin = isAdmin();
      return plainTable(h('table', { class: 'fui-data-table fui-data-table--dense keys' },
        h('thead', null, h('tr', null, h('th', null, 'Name'), h('th', null, 'Opens'), admin ? h('th', null, 'Owner') : null, h('th', null, 'Made'), h('th', null, 'Last used'), h('th', null, 'Expires'), h('th', { 'data-nosort': '' }, h('span', { class: 'sr-only' }, 'Actions')))),
        h('tbody', null, rows.map((k) => {
          const ask = pending === k.id;
          const actions = ask
            ? [h('span', { class: 'muted' }, 'Revoke it? Anything using it stops at once.'),
              button({ size: 'sm', variant: 'danger', type: 'button', onClick: async (ev) => { setBusy(ev.currentTarget, true, 'Revoking…'); try { await api.del(`/keys/${k.id}`); } catch (e) { err = e.message; } pending = null; await load(); } }, 'Revoke'),
              button({ size: 'sm', variant: 'ghost', type: 'button', onClick: () => { pending = null; paint(); } }, 'Cancel')]
            : button({ size: 'sm', variant: 'ghost', tone: 'danger', type: 'button', onClick: () => { pending = k.id; paint(); } }, 'Revoke…');
          return h('tr', null,
            h('td', null, k.name),
            h('td', null, h('span', { class: 'fui-chip' }, k.scope === 'calendar' ? 'Calendar' : 'Everything')),
            admin ? h('td', null, h('span', { class: 'user-cell' }, avatar(k.user_id, k.user_name, { size: 20, hasImage: k.has_image }), k.user_name, k.mine ? h('span', { class: 'muted' }, ' (you)') : null)) : null,
            h('td', { 'data-sort': k.created_at }, relEl(k.created_at)),
            h('td', { 'data-sort': k.last_used_at || 0 }, k.last_used_at ? h('span', { title: k.last_used_ip ? `from ${k.last_used_ip}` : null }, relEl(k.last_used_at)) : h('span', { class: 'muted' }, 'never')),
            h('td', { 'data-sort': k.expires_at || 0 }, k.expires_at ? h('span', { title: dateTime(k.expires_at) }, relEl(k.expires_at)) : h('span', { class: 'muted' }, 'never')),
            h('td', null, h('div', { class: 'backup-actions' }, actions)));
        }))));
    }

    function paint() {
      if (!data) return;
      const rows = data.keys || [];
      const mine = rows.filter((k) => k.mine).length;
      mount(body,
        revealed ? revealBox(revealed) : null,
        rows.length ? table(rows) : h('p', { class: 'fui-field__help' }, 'No keys yet.'),
        err ? inlineError('keys-err', err) : null,
        h('div', null, h('h3', { class: 'section-label' }, 'Make a key'), makeForm()),
        h('p', { class: 'fui-field__help fui-setting-row__foot' }, `${num(mine)} of yours, ${num(20)} at most. A key is stored only as a hash, is never part of a backup, and cannot make or revoke keys itself.`));
      err = null;
    }
    await load();
  },
};
