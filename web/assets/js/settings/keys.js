// Settings → API keys: use FinStats from outside. A key carries exactly its maker's permissions, is
// shown once when it is made, and dies with their access. A calendar key opens the calendar feed and
// nothing else, which is why it may travel in a subscription address. Administrators see everyone's.

import { h, icon, num, mount, relEl, dateTime, untilText } from '../dom.js';
import { api, isAbort } from '../api.js';
import { isAdmin, state } from '../state.js';
import { card, sk, setBusy, inlineError, errorState, formField, segmented, copyButton, avatar } from '../components.js';
import { plainTable } from '../tables.js';
import { button } from '../../finui/components/button/button.js';
import { t, tParts } from '../i18n.js';

const SCOPE_LABEL = { full: t('settings.keys.scope.full'), calendar: t('settings.keys.scope.calendar') };
const EXPIRY = [['', t('settings.keys.expiry.never')], ['30', t('settings.keys.expiry.d30')], ['90', t('settings.keys.expiry.d90')], ['365', t('settings.keys.expiry.year')]];

export default {
  key: 'keys', label: t('settings.sections.keys.label'), sub: t('settings.sections.keys.sub'), group: t('settings.groups.account'), icon: 'link',
  visible: () => true,
  entries: [
    { id: 'keys', label: t('settings.sections.keys.label'), hint: t('settings.keys.keys_hint') },
    { id: 'key-new', label: t('settings.keys.make'), hint: t('settings.keys.make_hint') },
  ],
  async render(slot, store) {
    const body = h('div', { class: 'net-stack' }, sk.rows(2));
    mount(slot, card({ title: t('settings.sections.keys.label'), sub: t('settings.keys.card_sub'), body, id: 'keys' }));
    let data = null, revealed = null, pending = null, err = null, scope = 'full';
    // FinStats' own address, which /auth/me tells everybody: the settings that hold it are a manager's to read, and
    // reading them made a 403 for anybody else and a feed link that only worked at home.
    const base = (state.user && state.user.public_url) || '';

    async function load() {
      try { data = await api.get('/keys', null, { signal: store.signal }); }
      catch (e) { if (isAbort(e) || e.status === 401) return; mount(body, errorState(e, load)); return; }
      paint();
    }

    function feedUrl(key) { return `${base || location.origin}/api/calendar.ics?key=${encodeURIComponent(key)}`; }

    function revealBox(k) {
      const isCal = k.scope === 'calendar';
      return h('div', { class: 'key-reveal', role: 'status' },
        h('div', { class: 'fui-setting-row__label' }, t('settings.keys.new_key', { name: k.name })),
        h('p', { class: 'fui-field__help' }, t('settings.keys.shown_once')),
        h('div', { class: 'key-line' }, h('code', { class: 'mono key-token' }, k.key), copyButton(k.key, t('settings.keys.copy_key'))),
        isCal ? [h('p', { class: 'fui-field__help' }, t('settings.keys.subscribe')),
          h('div', { class: 'key-line' }, h('code', { class: 'mono key-token' }, feedUrl(k.key)), copyButton(feedUrl(k.key), t('settings.keys.copy_address'))),
          // Only an administrator can set the address (it is one of the access settings), so only they are told to.
          !base && isAdmin() ? h('p', { class: 'fui-field__help' }, t('settings.keys.no_public_url')) : null]
        : h('p', { class: 'fui-field__help' }, tParts('settings.keys.send_as', { header: h('code', { class: 'mono' }, 'Authorization: Bearer ' + k.key.slice(0, 7) + '…') })),
        h('div', { class: 'form-actions' }, button({ size: 'sm', type: 'button', onClick: () => { revealed = null; paint(); } }, t('settings.keys.done'))));
    }

    function makeForm() {
      const name = formField({ id: 'key-name', label: t('settings.keys.name'), autocomplete: 'off', placeholder: t('settings.keys.name_placeholder') });
      const expiry = h('select', { class: 'fui-field__input', id: 'key-expiry', 'aria-label': t('settings.keys.expires') }, EXPIRY.map(([v, l]) => h('option', { value: v }, l)));
      // One sentence per scope, from one place: the form is rebuilt after a key is made and the choice survives it.
      const scopeHelp = (v) => t(v === 'calendar' ? 'settings.keys.scope_help.calendar' : 'settings.keys.scope_help.full', { scope: SCOPE_LABEL[v] });
      const scopePick = segmented({ label: t('settings.keys.what_it_opens'), value: scope, options: [{ value: 'full', label: t('settings.keys.everything') }, { value: 'calendar', label: t('settings.keys.calendar_only') }], onChange: (v) => { scope = v; help.textContent = scopeHelp(v); } });
      const help = h('p', { class: 'fui-field__help' }, scopeHelp(scope));
      const save = button({ variant: 'primary', type: 'submit' }, icon('plus', 13), t('settings.keys.make'));
      const formErr = h('div');
      const form = h('form', { class: 'key-form', id: 'key-new', noValidate: true },
        h('div', { class: 'key-form-row' }, name.el, h('div', { class: 'fui-field' }, h('label', { class: 'fui-field__label', htmlFor: 'key-expiry' }, t('settings.keys.expires')), expiry)),
        h('div', { class: 'fui-field' }, h('div', { class: 'fui-field__label' }, t('settings.keys.what_it_opens')), scopePick, help),
        h('div', { class: 'form-actions' }, save), formErr);
      form.addEventListener('submit', async (e) => {
        e.preventDefault(); mount(formErr, ''); name.setError('');
        const n = name.input.value.trim();
        if (!n) { name.setError(t('settings.keys.name_missing')); name.input.focus(); return; }
        setBusy(save, true, t('settings.keys.making'));
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
        h('thead', null, h('tr', null, h('th', null, t('settings.keys.name')), h('th', null, t('settings.keys.col_opens')), admin ? h('th', null, t('settings.keys.col_owner')) : null, h('th', null, t('settings.keys.col_made')), h('th', null, t('settings.keys.col_last_used')), h('th', null, t('settings.keys.expires')), h('th', { 'data-nosort': '' }, h('span', { class: 'sr-only' }, t('settings.keys.col_actions'))))),
        h('tbody', null, rows.map((k) => {
          const ask = pending === k.id;
          const actions = ask
            ? [h('span', { class: 'muted' }, t('settings.keys.revoke_confirm')),
              button({ size: 'sm', variant: 'danger', type: 'button', onClick: async (ev) => { setBusy(ev.currentTarget, true, t('settings.keys.revoking')); try { await api.del(`/keys/${k.id}`); } catch (e) { err = e.message; } pending = null; await load(); } }, t('settings.keys.revoke')),
              button({ size: 'sm', variant: 'ghost', type: 'button', onClick: () => { pending = null; paint(); } }, t('common.cancel'))]
            : button({ size: 'sm', variant: 'ghost', tone: 'danger', type: 'button', onClick: () => { pending = k.id; paint(); } }, t('settings.keys.revoke_ask'));
          return h('tr', null,
            h('td', null, k.name),
            h('td', null, h('span', { class: 'fui-chip' }, k.scope === 'calendar' ? t('settings.keys.calendar') : t('settings.keys.everything'))),
            admin ? h('td', null, h('span', { class: 'user-cell' }, avatar(k.user_id, k.user_name, { size: 20, hasImage: k.has_image }), k.user_name, k.mine ? h('span', { class: 'muted' }, ' ', t('settings.keys.you')) : null)) : null,
            h('td', { 'data-sort': k.created_at }, relEl(k.created_at)),
            h('td', { 'data-sort': k.last_used_at || 0 }, k.last_used_at ? h('span', { title: k.last_used_ip ? t('settings.keys.from_ip', { ip: k.last_used_ip }) : null }, relEl(k.last_used_at)) : h('span', { class: 'muted' }, t('time.never'))),
            // When it runs out is ahead, which relTime cannot say ("just now" for every key): untilText looks forward.
            h('td', { 'data-sort': k.expires_at || 0 }, k.expires_at ? h('span', { title: dateTime(k.expires_at) }, k.expires_at > Date.now() / 1000 ? untilText(k.expires_at) : t('settings.keys.expired')) : h('span', { class: 'muted' }, t('time.never'))),
            h('td', null, h('div', { class: 'backup-actions' }, actions)));
        }))));
    }

    function paint() {
      if (!data) return;
      const rows = data.keys || [];
      const mine = rows.filter((k) => k.mine).length;
      mount(body,
        revealed ? revealBox(revealed) : null,
        rows.length ? table(rows) : h('p', { class: 'fui-field__help' }, t('settings.keys.none_yet')),
        err ? inlineError('keys-err', err) : null,
        h('div', null, h('h3', { class: 'section-label' }, t('settings.keys.make')), makeForm()),
        h('p', { class: 'fui-field__help fui-setting-row__foot' }, t('settings.keys.foot', { mine: num(mine), max: num(20) })));
      err = null;
    }
    await load();
  },
};
