// Settings → Jellyfin: the server FinStats reads from, and how the collector is being told about it.

import { h, icon, num, relTime, dateTime, mount } from '../dom.js';
import { can, isAdmin } from '../state.js';
import { card, sk, facts, setBusy, inlineError } from '../components.js';
import { settingRow } from './common.js';
import { button } from '../../finui/components/button/button.js';

export default {
  key: 'jellyfin', label: 'Jellyfin', sub: 'The server FinStats reads from', icon: 'server',
  visible: () => can('manage'),
  entries: [
    { id: 'jellyfin', label: 'Jellyfin connection', hint: 'server address version collector live socket asked told' },
    { id: 'jellyfin_public_url', label: 'Jellyfin’s address for people', hint: 'open in jellyfin play button external url link domain reverse proxy' },
  ],
  async render(slot, store) {
    const body = h('div', null, sk.rows(2));
    const address = h('div');
    mount(slot, card({ title: 'Jellyfin connection', sub: 'Read-only: FinStats never changes anything on it', body: [body, address], id: 'jellyfin' }));
    await store.loadSettings();
    mount(address, peopleAddress(store));
    const paint = () => {
      const s = store.settings, c = store.tasks && store.tasks.collector;
      const status = !c ? h('span', { class: 'muted' }, 'Checking…')
        : c.connected ? h('span', { class: 'fui-badge--status fui-badge--good' }, icon('check', 13), `Connected · ${num(c.active_sessions)} active ${c.active_sessions === 1 ? 'session' : 'sessions'}`)
        : h('span', { class: 'fui-badge--status fui-badge--critical' }, icon('alert', 13), 'Not connected' + (c.error ? `: ${c.error}` : ''));
      // With the live connection carrying, each transport does the half it is good at: Jellyfin
      // says when something starts, and FinStats asks for the detail while it plays.
      const live = c && c.socket_live;
      const how = !c || !c.connected ? null
        : live ? h('span', { class: 'fui-badge--status fui-badge--good' }, icon('activity', 13), 'Told by Jellyfin, live')
        : h('span', { class: 'fui-badge--status fui-badge--warning' }, icon('clock', 13), `Asked every ${num(s.active_interval_s)} s while watching, every ${num(s.idle_interval_s)} s otherwise`);
      mount(body, facts([
        ['Server', s.server_name],
        ['Address', s.jellyfin_url, { mono: true }],
        ['Jellyfin version', s.server_version, { mono: true }],
        ['Collector', status],
        how ? ['How', how] : null,
        c && c.last_poll_at ? ['Last checked', h('span', { title: dateTime(c.last_poll_at) }, relTime(c.last_poll_at)), { mono: true }] : null,
        live ? ['Asking right now', c.transport === 'poll' ? 'Yes, something is playing' : 'No, nothing is playing or everything is paused'] : null,
      ]),
      h('div', { class: 'fui-setting-row__notes' },
      live ? h('p', { class: 'fui-field__help' }, `Nothing is asked for while nothing plays or everything is paused; while something runs FinStats asks every ${num(s.active_interval_s)} s, which is what keeps pauses and skips exact.`)
        : c && c.socket_error ? h('p', { class: 'fui-field__help' }, `The live connection is not carrying: ${c.socket_error}. FinStats keeps trying; nothing is missed meanwhile.`) : null,
      h('p', { class: 'fui-field__help' }, 'Connected with its own API key, made during setup. To point FinStats at another server, start it with a fresh data directory.')));
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
  const save = button({ size: 'sm', type: 'button' }, 'Save');
  save.addEventListener('click', async () => {
    mount(err, ''); setBusy(save, true, 'Saving…');
    try {
      await store.put({ jellyfin_public_url: input.value.trim() });
      note.replaceChildren(icon('check', 13), 'Saved');
      setTimeout(() => note.replaceChildren(), 2000);
    } catch (e) { mount(err, inlineError('jellyfin_public_url-err', e.message)); }
    finally { setBusy(save, false); }
  });
  input.addEventListener('keydown', (e) => { if (e.key === 'Enter') { e.preventDefault(); save.click(); } });
  return settingRow({ id: 'jellyfin_public_url', label: 'Jellyfin’s address for people', labelFor: 'f-jellyfin_public_url',
    help: 'Where the “Open in Jellyfin” buttons go. Leave empty to use the address FinStats connects to.',
    control: h('div', { class: 'fui-field__row' }, note, input, isAdmin() ? save : null), error: err });
}

