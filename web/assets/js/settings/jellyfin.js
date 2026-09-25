// Settings → Jellyfin: the server finstats reads from, and how the collector is being told about it.

import { h, icon, num, relTime, dateTime, mount } from '../dom.js';
import { can } from '../state.js';
import { card, sk, facts } from '../components.js';

export default {
  key: 'jellyfin', label: 'Jellyfin', sub: 'The server finstats reads from', icon: 'server',
  visible: () => can('manage'),
  entries: [{ id: 'jellyfin', label: 'Jellyfin connection', hint: 'server address version collector live socket asked told' }],
  async render(slot, store) {
    const body = h('div', null, sk.rows(2));
    mount(slot, card({ title: 'Jellyfin connection', sub: 'Read-only: finstats never changes anything on it', body, id: 'jellyfin' }));
    await store.loadSettings();
    const paint = () => {
      const s = store.settings, c = store.tasks && store.tasks.collector;
      const status = !c ? h('span', { class: 'muted' }, 'Checking…')
        : c.connected ? h('span', { class: 'sev sev-good' }, icon('check', 13), `Connected · ${num(c.active_sessions)} active ${c.active_sessions === 1 ? 'session' : 'sessions'}`)
        : h('span', { class: 'sev sev-critical' }, icon('alert', 13), 'Not connected' + (c.error ? ` — ${c.error}` : ''));
      // With the live connection carrying, each transport does the half it is good at: Jellyfin
      // says when something starts, and finstats asks for the detail while it plays.
      const live = c && c.socket_live;
      const how = !c || !c.connected ? null
        : live ? h('span', { class: 'sev sev-good' }, icon('activity', 13), 'Told by Jellyfin, live')
        : h('span', { class: 'sev sev-warning' }, icon('clock', 13), `Asked every ${num(s.active_interval_s)} s while watching, every ${num(s.idle_interval_s)} s otherwise`);
      mount(body, facts([
        ['Server', s.server_name],
        ['Address', s.jellyfin_url, { mono: true }],
        ['Jellyfin version', s.server_version, { mono: true }],
        ['Collector', status],
        how ? ['How', how] : null,
        c && c.last_poll_at ? ['Last checked', h('span', { title: dateTime(c.last_poll_at) }, relTime(c.last_poll_at)), { mono: true }] : null,
        live ? ['Asking right now', c.transport === 'poll' ? 'Yes — something is playing' : 'No — nothing is playing, or everything is paused'] : null,
      ]),
      h('div', { class: 'setting-notes' },
      live ? h('p', { class: 'help' }, `Nothing is asked for while nothing plays or everything is paused; while something runs finstats asks every ${num(s.active_interval_s)} s, which is what keeps pauses and skips exact.`)
        : c && c.socket_error ? h('p', { class: 'help' }, `The live connection is not carrying: ${c.socket_error}. finstats keeps trying; nothing is missed meanwhile.`) : null,
      h('p', { class: 'help' }, 'Connected with its own API key, made during setup. To point finstats at another server, start it with a fresh data directory.')));
    };
    paint();
    store.onTasks(paint);
    store.loadTasks().catch(() => {});
  },
};
