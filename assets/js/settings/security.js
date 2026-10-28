// Settings → Security: where addresses are placed, and what counts as impossible travel.

import { h, icon, mount, dateOnly } from '../dom.js';
import { can } from '../state.js';
import { api } from '../api.js';
import { card, sk, setBusy, inlineError } from '../components.js';
import { numberForm, settingRow } from './common.js';
import { triggerWords } from './tasks.js';
import { button } from '../../finui/components/button/button.js';
import { t, tParts } from '../i18n.js';

const TRAVEL_FIELDS = [
  { key: 'travel_speed_kmh', label: t('settings.security.speed'), unit: 'km/h', min: 100, max: 5000, help: t('settings.security.speed_help') },
  { key: 'travel_min_km', label: t('settings.security.distance'), unit: t('settings.security.distance_unit'), min: 50, max: 5000, help: t('settings.security.distance_help') },
];

export default {
  key: 'security', label: t('settings.sections.security.label'), sub: t('settings.sections.security.sub'), group: t('settings.groups.tracking'), icon: 'shield',
  visible: () => can('manage'),
  entries: [
    { id: 'geoip-database', label: t('settings.security.database'), hint: t('settings.security.database_hint') },
    { id: 'geoip-schedule', label: t('settings.security.up_to_date'), hint: t('settings.security.up_to_date_hint') },
    ...TRAVEL_FIELDS.map((f) => ({ id: f.key, label: f.label, hint: t('settings.security.travel_hint') })),
  ],
  async render(slot, store) {
    const body = h('div', { class: 'fui-setting-row__rows' }, sk.rows(3));
    mount(slot, card({ title: t('settings.sections.security.label'), sub: t('settings.security.card_sub'), body, id: 'security' }));
    await store.loadSettings();
    let geoWasRunning = false, geoSig = null;
    const paint = () => {
      const s = store.settings, g = s.geoip || {}, dbInfo = g.database;
      const task = store.task('geoip');
      const running = !!task && task.state === 'running';
      const status = dbInfo
        ? h('p', { class: 'fui-field__help' }, ...tParts('settings.security.built', { kind: h('strong', null, dbInfo.kind), date: dateOnly(dbInfo.built_at) }), dbInfo.file ? [' · ', h('span', { class: 'mono' }, dbInfo.file)] : null)
        : h('p', { class: 'fui-field__help' }, t('settings.security.none_yet'));
      const err = h('div');
      const get = button({ size: 'sm', type: 'button', disabled: running || g.from_env }, icon('upload', 13, 'flip-v'), running ? (task.message || t('settings.security.downloading')) : dbInfo ? t('settings.security.download_newest') : t('settings.security.download'));
      get.addEventListener('click', async () => {
        mount(err, '');
        setBusy(get, true, t('settings.security.starting'));
        try { await api.post('/security/database'); await store.poke(1000); }
        catch (e) { setBusy(get, false); mount(err, inlineError('geoip-err', e.message)); }
      });
      if (task && task.state === 'error') mount(err, inlineError('geoip-err', task.error || t('settings.security.download_failed')));
      mount(body,
        settingRow({ id: 'geoip-database', label: t('settings.security.database'),
          help: g.from_env ? t('settings.security.from_env')
            : g.folder ? t('settings.security.from_folder', { folder: g.folder }) : t('settings.security.from_default_folder'),
          control: [h('div', { class: 'home-known' }, status, h('div', { class: 'form-actions' }, get), err)] }),
        g.from_env ? null : settingRow({ id: 'geoip-schedule', label: t('settings.security.up_to_date'),
          help: task && task.triggers && task.triggers.length
            ? t('settings.security.scheduled', { triggers: task.triggers.map(triggerWords).join(', ') })
            : t('settings.security.not_scheduled'),
          control: button({ size: 'sm', href: '/settings/tasks/geoip' }, icon('clock', 13), t('settings.security.schedule')) }),
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
