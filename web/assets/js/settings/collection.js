// Settings → Collection: how closely plays are followed, and what counts as one.

import { h, icon, mount } from '../dom.js';
import { can, isAdmin } from '../state.js';
import { card, sk } from '../components.js';
import { numberForm, settingRow } from './common.js';
import { button } from '../../finui/components/button/button.js';
import { t } from '../i18n.js';

const FIELDS = [
  { key: 'active_interval_s', label: t('settings.collection.active.label'), unit: t('settings.collection.seconds'), min: 1, max: 60, help: t('settings.collection.active.help') },
  { key: 'idle_interval_s', label: t('settings.collection.idle.label'), unit: t('settings.collection.seconds'), min: 1, max: 60, help: t('settings.collection.idle.help') },
  { key: 'merge_window_s', label: t('settings.collection.merge.label'), unit: t('settings.collection.seconds'), min: 0, max: 3600, help: t('settings.collection.merge.help') },
  { key: 'group_window_s', label: t('settings.collection.group.label'), unit: t('settings.collection.seconds'), min: 5, max: 600, help: t('settings.collection.group.help') },
  { key: 'min_play_s', label: t('settings.collection.min_play.label'), unit: t('settings.collection.seconds'), min: 0, max: 3600, help: t('settings.collection.min_play.help') },
];

export default {
  key: 'collection', label: t('settings.sections.collection.label'), sub: t('settings.sections.collection.sub'), group: t('settings.groups.tracking'), icon: 'activity',
  visible: () => can('manage'),
  entries: [
    ...FIELDS.map((f) => ({ id: f.key, label: f.label, hint: `${f.key.replace(/_/g, ' ')} ${t('settings.collection.field_hint')}` })),
  ],
  async render(slot, store) {
    const body = h('div', { class: 'fui-setting-row__rows' }, sk.rows(3));
    mount(slot, card({ title: t('settings.sections.collection.label'), sub: t('settings.collection.card_sub'), body, id: 'collection' }));
    await store.loadSettings();
    mount(body,
      settingRow({ id: 'library-read', label: t('settings.collection.library_read'),
        help: t('settings.collection.library_read_help'),
        control: button({ size: 'sm', href: '/settings/tasks/sync_libraries' }, icon('clock', 13), t('settings.collection.schedule')) }),
      // The merge window rewrites how history is kept, for good: an administrator's to change.
      numberForm(store, FIELDS.filter((f) => f.key !== 'merge_window_s' || isAdmin()), 'collect-err'));
  },
};
