// Settings → Connections and Notifications: the services around Jellyfin, and where what FinStats
// finds is sent. Both panels manage themselves; these sections only give them a place.

import { h, mount } from '../dom.js';
import { isAdmin } from '../state.js';
import { card } from '../components.js';
import { connectionsPanel } from '../connections.js';
import { notificationsPanel, mayNotify } from '../notifications.js';
import { t } from '../i18n.js';

export const connections = {
  key: 'connections', label: t('settings.sections.connections.label'), sub: t('settings.sections.connections.sub'), group: t('settings.groups.services'), icon: 'plug',
  visible: () => isAdmin(),
  entries: [{ id: 'connections', label: t('settings.sections.connections.sub'), hint: t('settings.services.connections_hint') }],
  async render(slot, store) {
    mount(slot, card({ title: t('settings.sections.connections.label'), sub: t('settings.services.connections_sub'), body: connectionsPanel(store.ctx), id: 'connections' }));
  },
};

export const notifications = {
  key: 'notifications', label: t('settings.sections.notifications.label'), sub: t('settings.sections.notifications.sub'), group: t('settings.groups.services'), icon: 'inbox',
  visible: () => mayNotify(),
  entries: [
    { id: 'notifications', label: t('settings.services.destinations'), hint: t('settings.services.destinations_hint') },
    { id: 'notify-public-url', label: t('settings.services.public_url'), hint: t('settings.services.public_url_hint') },
  ],
  async render(slot, store) {
    mount(slot, card({ title: t('settings.sections.notifications.label'), sub: t('settings.services.notifications_sub'), body: notificationsPanel(store.ctx), id: 'notifications' }));
  },
};

export default [connections, notifications];
