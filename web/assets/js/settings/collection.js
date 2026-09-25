// Settings → Collection: how closely plays are followed, and what counts as one.

import { h, mount } from '../dom.js';
import { can } from '../state.js';
import { card, sk } from '../components.js';
import { toggleRow, numberForm } from './common.js';

const FIELDS = [
  { key: 'active_interval_s', label: 'While someone is watching, check every', unit: 'seconds', min: 1, max: 60, help: 'Pauses, skips and track changes are recorded to this precision. 1–60.' },
  { key: 'idle_interval_s', label: 'While nothing is playing, check every', unit: 'seconds', min: 1, max: 60, help: 'Only while Jellyfin is not pushing; the time before a play is noticed is not counted. 1–60.' },
  { key: 'sync_interval_h', label: 'Otherwise, re-read the library every', unit: 'hours', min: 1, max: 168, help: 'The fallback when finstats is not following Jellyfin’s scan. 1–168.' },
  { key: 'merge_window_s', label: 'Treat a restart as the same play within', unit: 'seconds', min: 0, max: 86400, help: 'Same person, same title, same device. 0 turns merging off.' },
  { key: 'group_window_s', label: 'Count it as watching together within', unit: 'seconds', min: 5, max: 600, help: 'Different people starting the same title this close together, for a couple of minutes at least. 5–600.' },
  { key: 'min_play_s', label: 'Ignore plays shorter than', unit: 'seconds', min: 0, max: 3600, help: 'Kept in the database, left out of the stats. 0 counts everything.' },
];

export default {
  key: 'collection', label: 'Collection', sub: 'How closely plays are followed', group: 'Tracking', icon: 'activity',
  visible: () => can('manage'),
  entries: [
    { id: 'follow_jellyfin_scan', label: 'Follow Jellyfin’s library scan', hint: 'sync library read scan schedule' },
    ...FIELDS.map((f) => ({ id: f.key, label: f.label, hint: `${f.key.replace(/_/g, ' ')} interval poll ${f.unit}` })),
  ],
  async render(slot, store) {
    const body = h('div', { class: 'setting-rows' }, sk.rows(3));
    mount(slot, card({ title: 'Collection', sub: 'How closely plays are followed, and what counts as one', body, id: 'collection' }));
    await store.loadSettings();
    mount(body,
      toggleRow(store, { key: 'follow_jellyfin_scan', label: 'Follow Jellyfin’s library scan',
        help: 'Re-read the library only after Jellyfin’s own scan has finished. finstats never starts one.' }),
      numberForm(store, FIELDS, 'collect-err'));
  },
};
