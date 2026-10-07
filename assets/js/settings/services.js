// Settings → Connections and Notifications: the services around Jellyfin, and where what finstats
// finds is sent. Both panels manage themselves; these sections only give them a place.

import { h, mount } from '../dom.js';
import { isAdmin } from '../state.js';
import { card } from '../components.js';
import { connectionsPanel } from '../connections.js';
import { notificationsPanel, mayNotify } from '../notifications.js';

export const connections = {
  key: 'connections', label: 'Connections', sub: 'Sonarr, Radarr and Seerr', group: 'Services', icon: 'plug',
  visible: () => isAdmin(),
  entries: [{ id: 'connections', label: 'Sonarr, Radarr and Seerr', hint: 'connections services arr requests downloads calendar api key' }],
  async render(slot, store) {
    mount(slot, card({ title: 'Connections', sub: 'Sonarr, Radarr and Seerr: what is requested, coming and downloading', body: connectionsPanel(store.ctx), id: 'connections' }));
  },
};

export const notifications = {
  key: 'notifications', label: 'Notifications', sub: 'Where what finstats finds is sent', group: 'Services', icon: 'inbox',
  visible: () => mayNotify(),
  entries: [
    { id: 'notifications', label: 'Notification destinations', hint: 'discord slack telegram email ntfy gotify pushover pushbullet webhook send alerts' },
    { id: 'notify-public-url', label: 'The address of finstats', hint: 'public url link messages' },
  ],
  async render(slot, store) {
    mount(slot, card({ title: 'Notifications', sub: 'Discord, Slack, Telegram, e-mail, ntfy, Gotify, Pushover, Pushbullet or a webhook of your own', body: notificationsPanel(store.ctx), id: 'notifications' }));
  },
};

export default [connections, notifications];
