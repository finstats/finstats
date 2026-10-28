// Settings → Jellyfin: the server FinStats reads from, and how the collector is being told about it.

import { h, icon, relTime, dateTime, mount } from '../dom.js';
import { can, isAdmin } from '../state.js';
import { card, sk, facts, setBusy, inlineError } from '../components.js';
import { settingRow } from './common.js';
import { button } from '../../finui/components/button/button.js';
import { t } from '../i18n.js';

export default {
  key: 'jellyfin', label: t('settings.sections.jellyfin.label'), sub: t('settings.sections.jellyfin.sub'), icon: 'server',
  visible: () => can('manage'),
  entries: [
    { id: 'jellyfin', label: t('settings.jellyfin.connection'), hint: t('settings.jellyfin.connection_hint') },
    { id: 'jellyfin_public_url', label: t('settings.jellyfin.public_url'), hint: t('settings.jellyfin.public_url_hint') },
  ],
  async render(slot, store) {
    const body = h('div', null, sk.rows(2));
    const address = h('div');
    mount(slot, card({ title: t('settings.jellyfin.connection'), sub: t('settings.jellyfin.connection_sub'), body: [body, address], id: 'jellyfin' }));
    await store.loadSettings();
    mount(address, peopleAddress(store));
    const paint = () => {
      const s = store.settings, c = store.tasks && store.tasks.collector;
      const status = !c ? h('span', { class: 'muted' }, t('settings.jellyfin.checking'))
        : c.connected ? h('span', { class: 'fui-badge--status fui-badge--good' }, icon('check', 13), t('settings.jellyfin.connected', { count: c.active_sessions }))
        : h('span', { class: 'fui-badge--status fui-badge--critical' }, icon('alert', 13), c.error ? t('settings.jellyfin.not_connected_because', { error: c.error }) : t('settings.jellyfin.not_connected'));
      // With the live connection carrying, each transport does the half it is good at: Jellyfin
      // says when something starts, and FinStats asks for the detail while it plays.
      const live = c && c.socket_live;
      const how = !c || !c.connected ? null
        : live ? h('span', { class: 'fui-badge--status fui-badge--good' }, icon('activity', 13), t('settings.jellyfin.told_live'))
        : h('span', { class: 'fui-badge--status fui-badge--warning' }, icon('clock', 13), t('settings.jellyfin.asked_every', { active: s.active_interval_s, idle: s.idle_interval_s }));
      mount(body, facts([
        [t('settings.jellyfin.server'), s.server_name],
        s.jellyfin_url ? [t('settings.jellyfin.address'), s.jellyfin_url, { mono: true }] : null,
        [t('settings.jellyfin.version'), s.server_version, { mono: true }],
        [t('settings.jellyfin.collector'), status],
        how ? [t('settings.jellyfin.how'), how] : null,
        c && c.last_poll_at ? [t('settings.jellyfin.last_checked'), h('span', { title: dateTime(c.last_poll_at) }, relTime(c.last_poll_at)), { mono: true }] : null,
        live ? [t('settings.jellyfin.asking_now'), c.transport === 'poll' ? t('settings.jellyfin.asking_yes') : t('settings.jellyfin.asking_no')] : null,
      ]),
      h('div', { class: 'fui-setting-row__notes' },
      live ? h('p', { class: 'fui-field__help' }, t('settings.jellyfin.live_note', { active: s.active_interval_s }))
        : c && c.socket_error ? h('p', { class: 'fui-field__help' }, t('settings.jellyfin.socket_error', { error: c.socket_error })) : null,
      h('p', { class: 'fui-field__help' }, t('settings.jellyfin.own_key'))));
    };
    paint();
    store.onTasks(paint);
    store.loadTasks().catch(() => {});
  },
};

/** Where "Open in Jellyfin" points. Only a Jellyfin administrator may change it: it is a link everyone follows. */
function peopleAddress(store) {
  const current = store.settings.jellyfin_public_url || '';
  const input = h('input', { class: 'fui-field__input', id: 'f-jellyfin_public_url', type: 'url', inputMode: 'url', autocomplete: 'off', spellcheck: false,
    value: current, placeholder: store.settings.jellyfin_url || 'https://jellyfin.example.com', disabled: !isAdmin(), 'aria-describedby': 'jellyfin_public_url-help' });
  const note = h('span', { class: 'saved-note', 'aria-live': 'polite' });
  const err = h('div');
  const save = button({ size: 'sm', type: 'button' }, t('common.save'));
  save.addEventListener('click', async () => {
    mount(err, ''); setBusy(save, true, t('settings.common.saving'));
    try {
      await store.put({ jellyfin_public_url: input.value.trim() });
      note.replaceChildren(icon('check', 13), t('settings.common.saved'));
      setTimeout(() => note.replaceChildren(), 2000);
    } catch (e) { mount(err, inlineError('jellyfin_public_url-err', e.message)); }
    finally { setBusy(save, false); }
  });
  input.addEventListener('keydown', (e) => { if (e.key === 'Enter') { e.preventDefault(); save.click(); } });
  return settingRow({ id: 'jellyfin_public_url', label: t('settings.jellyfin.public_url'), labelFor: 'f-jellyfin_public_url',
    help: t('settings.jellyfin.public_url_help'),
    control: h('div', { class: 'fui-field__row' }, note, input, isAdmin() ? save : null), error: err });
}

