// Settings → Security: where addresses are placed, and what counts as impossible travel.

import { h, icon, mount, dateOnly } from '../dom.js';
import { can } from '../state.js';
import { api } from '../api.js';
import { card, sk, setBusy, inlineError } from '../components.js';
import { toggleRow, numberForm, settingRow } from './common.js';

const TRAVEL_FIELDS = [
  { key: 'travel_speed_kmh', label: 'Impossible travel is faster than', unit: 'km/h', min: 100, max: 5000, help: 'Two sightings of one person that would need more than this raise an alert. 900 is a little above an airliner. 100–5,000.' },
  { key: 'travel_min_km', label: 'Only between places at least', unit: 'km apart', min: 50, max: 5000, help: 'City databases are often a few hundred kilometres off; closer places than this never raise an alert. 50–5,000.' },
];

export default {
  key: 'security', label: 'Security', sub: 'Places, and impossible travel', group: 'Tracking', icon: 'shield',
  visible: () => can('manage'),
  entries: [
    { id: 'geoip-database', label: 'Geolocation database', hint: 'geoip mmdb city db-ip maxmind download places map' },
    { id: 'geoip_download', label: 'Keep the database up to date', hint: 'geoip download monthly db-ip' },
    ...TRAVEL_FIELDS.map((f) => ({ id: f.key, label: f.label, hint: 'impossible travel alert speed distance km' })),
  ],
  async render(slot, store) {
    const body = h('div', { class: 'setting-rows' }, sk.rows(3));
    mount(slot, card({ title: 'Security', sub: 'Where addresses are placed, and what counts as impossible travel', body, id: 'security' }));
    await store.loadSettings();
    let geoWasRunning = false, geoSig = null;
    const paint = () => {
      const s = store.settings, g = s.geoip || {}, dbInfo = g.database;
      const t = store.task('geoip');
      const running = !!t && t.state === 'running';
      const status = dbInfo
        ? h('p', { class: 'help' }, h('strong', null, dbInfo.kind), `, built ${dateOnly(dbInfo.built_at)}`, dbInfo.file ? [' · ', h('span', { class: 'mono' }, dbInfo.file)] : null)
        : h('p', { class: 'help' }, 'None yet. The Security page stays empty until there is one.');
      const err = h('div');
      const get = h('button', { type: 'button', class: 'btn btn-sm', disabled: running || g.from_env }, icon('upload', 13, 'flip-v'), running ? (t.message || 'Downloading…') : dbInfo ? 'Download the newest' : 'Download (about 60 MB)');
      get.addEventListener('click', async () => {
        mount(err, '');
        setBusy(get, true, 'Starting…');
        try { await api.post('/security/database'); await store.poke(1000); }
        catch (e) { setBusy(get, false); mount(err, inlineError('geoip-err', e.message)); }
      });
      if (t && t.state === 'error') mount(err, inlineError('geoip-err', t.error || 'The download failed.'));
      mount(body,
        settingRow({ id: 'geoip-database', label: 'Geolocation database',
          help: g.from_env ? 'Set with FINSTATS_GEOIP_DB; replace that file to update it.'
            : `Addresses are placed from a file on this machine, never over the network: the newest .mmdb in ${g.folder || 'the geoip folder'}.`,
          control: [h('div', { class: 'home-known' }, status, h('div', { class: 'form-actions' }, get), err)] }),
        g.from_env ? null : toggleRow(store, { key: 'geoip_download', label: 'Keep the database up to date', onSaved: paint,
          help: 'Downloads DB-IP’s free city file (about 60 MB, CC BY 4.0) now and once a month. A plain download, with nothing about you in it.' }),
        numberForm(store, TRAVEL_FIELDS, 'travel-err'));
    };
    paint();
    store.onTasks(() => {
      const geo = store.task('geoip'), geoRunning = !!geo && geo.state === 'running';
      const now = JSON.stringify(geo ? [geo.state, geo.message, geo.error] : null);
      // Only when something changed: a re-render would wipe a number somebody is typing.
      if (geoWasRunning && !geoRunning) store.reloadSettings().then(paint).catch(() => {});
      else if (now !== geoSig) paint();
      geoWasRunning = geoRunning; geoSig = now;
    });
    store.loadTasks().catch(() => {});
  },
};
