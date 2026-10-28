// Settings → Home network: which plays count as local and which as remote. Private addresses are
// local by nature. The household's own public address is local too (a phone on the Wi-Fi reaching
// Jellyfin through its public name arrives with it), and FinStats has to learn that one.

import { h, mount, relTime, dateTime } from '../dom.js';
import { can } from '../state.js';
import { api } from '../api.js';
import { card, sk, setBusy, inlineError } from '../components.js';
import { toggleRow, settingRow } from './common.js';
import { button } from '../../finui/components/button/button.js';
import { t, tParts, lang } from '../i18n.js';

export default {
  key: 'network', label: t('settings.sections.network.label'), sub: t('settings.sections.network.sub'), group: t('settings.groups.tracking'), icon: 'lan',
  visible: () => can('manage'),
  entries: [
    { id: 'public_ip_lookup', label: t('settings.network.lookup'), hint: t('settings.network.lookup_hint') },
    { id: 'known-home', label: t('settings.network.known'), hint: t('settings.network.known_hint') },
    { id: 'home_addresses', label: t('settings.network.other'), hint: t('settings.network.other_hint') },
  ],
  async render(slot, store) {
    const body = h('div', { class: 'fui-setting-row__rows' }, sk.rows(3));
    mount(slot, card({ title: t('settings.sections.network.label'), sub: t('settings.network.card_sub'), body, id: 'network' }));
    await store.loadSettings();
    const paint = () => {
      const s = store.settings;
      const services = (s.public_ip_services || []).map((u) => String(u).replace(/^https?:\/\//, ''));
      const known = s.known_home_addresses || [];
      const list = known.length
        ? h('ul', { class: 'home-ips' }, known.map((a) => h('li', null, h('span', { class: 'mono' }, a.ip),
          h('span', { class: 'muted' }, a.source === 'manual' ? t('settings.network.added_by_you') : tParts('settings.network.found', { when: h('span', { title: dateTime(a.last_seen) }, relTime(a.last_seen)) })))))
        : h('p', { class: 'fui-field__help' }, s.public_ip_lookup ? t('settings.network.none_yet') : t('settings.network.none'));

      // The only thing that ever asks again, because a person pressed it.
      const lookupErr = h('div');
      const lookup = button({ size: 'sm', type: 'button' }, t('settings.network.look_up'));
      lookup.addEventListener('click', async () => {
        mount(lookupErr, '');
        setBusy(lookup, true, t('settings.network.asking'));
        try { store.settings = await api.post('/settings/public-ip'); paint(); }
        catch (e) { mount(lookupErr, inlineError('lookup-err', t('settings.network.lookup_failed', { error: e.message }))); }
        finally { setBusy(lookup, false); }
      });

      const input = h('textarea', { class: 'fui-field__input home-input mono', id: 'f-home', rows: 2, spellcheck: false, autocomplete: 'off', 'aria-describedby': 'home_addresses-help',
        placeholder: '203.0.113.7' }, (s.home_addresses || []).join('\n'));
      const note = h('span', { class: 'saved-note', 'aria-live': 'polite' });
      const err = h('div');
      const save = button({ variant: 'primary', type: 'submit' }, t('settings.network.save'));
      const form = h('form', { class: 'fui-setting-row__rows', noValidate: true },
        settingRow({ id: 'home_addresses', label: t('settings.network.other'), labelFor: 'f-home',
          help: t('settings.network.other_help'),
          control: input, error: err }),
        h('div', { class: 'form-actions fui-setting-row__actions' }, save, note));
      form.addEventListener('submit', async (e) => {
        e.preventDefault();
        mount(err, '');
        const home_addresses = input.value.split(/[\s,;]+/).map((x) => x.trim()).filter(Boolean);
        setBusy(save, true, t('settings.network.saving'));
        try { await store.put({ home_addresses }); paint(); }
        catch (e2) { input.setAttribute('aria-invalid', 'true'); mount(err, inlineError('home-err', t('settings.network.couldnt_save', { error: e2.message }))); }
        finally { setBusy(save, false); }
      });

      mount(body,
        toggleRow(store, { key: 'public_ip_lookup', label: t('settings.network.lookup'), onSaved: paint,
          help: t('settings.network.lookup_help') }),
        // The household's addresses are network details, and calling one home decides which alerts are raised.
        can('see_network') ? settingRow({ id: 'known-home', label: t('settings.network.known'), help: t('settings.network.known_help'),
          control: [h('div', { class: 'home-known' }, list, s.public_ip_lookup ? h('div', { class: 'form-actions' }, lookup) : null, lookupErr)] }) : null,
        can('see_network') && can('see_everyone') ? form : null,
        h('p', { class: 'fui-field__help fui-setting-row__foot' }, services.length ? t('settings.network.foot', { services: new Intl.ListFormat(lang(), { type: 'disjunction' }).format(services) }) : t('settings.network.foot_none')));
    };
    paint();
  },
};
