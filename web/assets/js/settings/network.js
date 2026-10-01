// Settings → Home network: which plays count as local and which as remote. Private addresses are
// local by nature. The household's own public address is local too (a phone on the Wi-Fi reaching
// Jellyfin through its public name arrives with it), and finstats has to learn that one.

import { h, mount, relTime, dateTime } from '../dom.js';
import { can } from '../state.js';
import { api } from '../api.js';
import { card, sk, setBusy, inlineError } from '../components.js';
import { toggleRow, settingRow } from './common.js';
import { button } from '../../finui/components/button/button.js';

export default {
  key: 'network', label: 'Home network', sub: 'Which plays count as local', group: 'Tracking', icon: 'lan',
  visible: () => can('manage'),
  entries: [
    { id: 'public_ip_lookup', label: 'Recognise my own public address', hint: 'public ip lookup what is my ip local remote' },
    { id: 'known-home', label: 'Known home addresses', hint: 'public ip found look up now' },
    { id: 'home_addresses', label: 'Other addresses that count as home', hint: 'vpn second home ip manual local' },
  ],
  async render(slot, store) {
    const body = h('div', { class: 'fui-setting-row__rows' }, sk.rows(3));
    mount(slot, card({ title: 'Home network', sub: 'Which plays count as local and which as remote', body, id: 'network' }));
    await store.loadSettings();
    const paint = () => {
      const s = store.settings;
      const services = (s.public_ip_services || []).map((u) => String(u).replace(/^https?:\/\//, ''));
      const known = s.known_home_addresses || [];
      const list = known.length
        ? h('ul', { class: 'home-ips' }, known.map((a) => h('li', null, h('span', { class: 'mono' }, a.ip),
          h('span', { class: 'muted' }, a.source === 'manual' ? 'added by you' : ['found automatically · last seen ', h('span', { title: dateTime(a.last_seen) }, relTime(a.last_seen))]))))
        : h('p', { class: 'fui-field__help' }, s.public_ip_lookup ? 'None learned yet; finstats looks one up within a few minutes of starting.' : 'None. Private addresses (192.168.x.x, 10.x.x.x and the like) always count as local.');

      // The only thing that ever asks again, because a person pressed it.
      const lookupErr = h('div');
      const lookup = button({ size: 'sm', type: 'button' }, 'Look up now');
      lookup.addEventListener('click', async () => {
        mount(lookupErr, '');
        setBusy(lookup, true, 'Asking…');
        try { store.settings = await api.post('/settings/public-ip'); paint(); }
        catch (e) { mount(lookupErr, inlineError('lookup-err', `Couldn’t look it up: ${e.message}`)); }
        finally { setBusy(lookup, false); }
      });

      const input = h('textarea', { class: 'fui-field__input home-input mono', id: 'f-home', rows: 2, spellcheck: false, autocomplete: 'off', 'aria-describedby': 'home_addresses-help',
        placeholder: '203.0.113.7' }, (s.home_addresses || []).join('\n'));
      const note = h('span', { class: 'saved-note', 'aria-live': 'polite' });
      const err = h('div');
      const save = button({ variant: 'primary', type: 'submit' }, 'Save addresses');
      const form = h('form', { class: 'fui-setting-row__rows', noValidate: true },
        settingRow({ id: 'home_addresses', label: 'Other addresses that count as home', labelFor: 'f-home',
          help: 'One per line: an earlier address of yours, a second home, a VPN exit. Your history is sorted again when you save.',
          control: input, error: err }),
        h('div', { class: 'form-actions fui-setting-row__actions' }, save, note));
      form.addEventListener('submit', async (e) => {
        e.preventDefault();
        mount(err, '');
        const home_addresses = input.value.split(/[\s,;]+/).map((x) => x.trim()).filter(Boolean);
        setBusy(save, true, 'Saving…');
        try { await store.put({ home_addresses }); paint(); }
        catch (e2) { input.setAttribute('aria-invalid', 'true'); mount(err, inlineError('home-err', `Couldn’t save: ${e2.message}`)); }
        finally { setBusy(save, false); }
      });

      mount(body,
        toggleRow(store, { key: 'public_ip_lookup', label: 'Recognise my own public address', onSaved: paint,
          help: 'Asks a public “what is my IP” service once, when first needed, so a device at home reaching Jellyfin by its public name counts as local.' }),
        settingRow({ id: 'known-home', label: 'Known home addresses', help: 'Public addresses of this household, past and present.',
          control: [h('div', { class: 'home-known' }, list, s.public_ip_lookup ? h('div', { class: 'form-actions' }, lookup) : null, lookupErr)] }),
        form,
        h('p', { class: 'fui-field__help fui-setting-row__foot' }, `That request goes to ${services.join(' or ') || 'nowhere: no service is configured'} and says nothing about you or your server. With it and the geolocation download under Security both off, finstats makes no outside requests at all.`));
    };
    paint();
  },
};
