// Settings → Collection: how closely plays are followed, and what counts as one.

import { h, icon, mount } from '../dom.js';
import { can } from '../state.js';
import { card, sk } from '../components.js';
import { numberForm, settingRow } from './common.js';
import { button } from '../../finui/components/button/button.js';

const FIELDS = [
  { key: 'active_interval_s', label: 'While someone is watching, check every', unit: 'seconds', min: 1, max: 60, help: 'Pauses, skips and track changes are recorded to this precision. 1–60.' },
  { key: 'idle_interval_s', label: 'While nothing is playing, check every', unit: 'seconds', min: 1, max: 60, help: 'Only while Jellyfin is not pushing; the time before a play is noticed is not counted. 1–60.' },
  { key: 'merge_window_s', label: 'Treat a restart as the same play within', unit: 'seconds', min: 0, max: 86400, help: 'Same person, same title, same device. 0 turns merging off.' },
  { key: 'group_window_s', label: 'Count it as watching together within', unit: 'seconds', min: 5, max: 600, help: 'Different people starting the same title this close together, for a couple of minutes at least. 5–600.' },
  { key: 'min_play_s', label: 'Ignore plays shorter than', unit: 'seconds', min: 0, max: 3600, help: 'Kept in the database, left out of the stats. 0 counts everything.' },
];

export default {
  key: 'collection', label: 'Collection', sub: 'How closely plays are followed', group: 'Tracking', icon: 'activity',
  visible: () => can('manage'),
  entries: [
    ...FIELDS.map((f) => ({ id: f.key, label: f.label, hint: `${f.key.replace(/_/g, ' ')} interval poll ${f.unit}` })),
  ],
  async render(slot, store) {
    const body = h('div', { class: 'fui-setting-row__rows' }, sk.rows(3));
    mount(slot, card({ title: 'Collection', sub: 'How closely plays are followed, and what counts as one', body, id: 'collection' }));
    await store.loadSettings();
    mount(body,
      settingRow({ id: 'library-read', label: 'When the library is read',
        help: 'After Jellyfin’s own library scan, unless you change it. FinStats never starts a scan.',
        control: button({ size: 'sm', href: '/settings/tasks/sync_libraries' }, icon('clock', 13), 'Schedule') }),
      numberForm(store, FIELDS, 'collect-err'));
  },
};
