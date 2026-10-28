// Settings → System: everywhere FinStats can reach, what it holds, and what it is built on. Its jobs
// have a section of their own (Tasks). Nothing here is a setting; it is the machine showing its work.

import { h, icon, num, bytes, relTime, dateTime, mount } from '../dom.js';
import { api, isAbort } from '../api.js';
import { isAdmin, can } from '../state.js';
import { card, sk, errorState, facts } from '../components.js';
import { button } from '../../finui/components/button/button.js';
import { t, tParts } from '../i18n.js';

const STATE_LABEL = { always: ['is-always', t('settings.system.state.always')], on: ['is-on', t('settings.system.state.on')], off: ['', t('settings.system.state.off')] };

export default {
  key: 'system', label: t('settings.sections.system.label'), sub: t('settings.sections.system.sub'), group: t('settings.groups.data'), icon: 'cpu',
  visible: () => can('manage'),
  entries: [
    { id: 'outbound', label: t('settings.system.outbound'), hint: t('settings.system.outbound_hint') },
    { id: 'database', label: t('settings.system.database'), hint: t('settings.system.database_hint') },
    { id: 'licences', label: t('settings.system.licences'), hint: t('settings.system.licences_hint') },
    { id: 'components', label: t('settings.system.gallery'), hint: t('settings.system.gallery_hint') },
  ],
  async render(slot, store) {
    const outboundSlot = h('div', { class: 'net-stack' }, sk.rows(3));
    const dbSlot = h('div', null, sk.rows(1));
    mount(slot,
      isAdmin() ? card({ title: t('settings.system.outbound'), sub: t('settings.system.outbound_sub'), body: outboundSlot, id: 'outbound' }) : null,
      card({ title: t('settings.system.database'), body: dbSlot, id: 'database' }),
      // FinStats' own licence and every third-party one, on their own page: it is half a megabyte
      // of licence text, which belongs where somebody goes looking for it, not in a settings card.
      card({ title: t('settings.system.licences'), sub: t('settings.system.licences_sub'), id: 'licences',
        body: button({ href: '/licenses' }, icon('log', 14), t('settings.system.third_party'), icon('chevronRight', 14)) }),
      // FinUI is its own project: its gallery and FinUI create are a static site of its own, not a page of FinStats.
      card({ title: t('settings.system.components'), sub: t('settings.system.components_sub'), id: 'components',
        body: button({ href: 'https://finui.finstats.no/', target: '_blank', rel: 'noopener noreferrer' }, icon('layers', 14), 'FinUI', icon('external', 14)) }));

    function paintDb() {
      const d = store.tasks && store.tasks.db;
      if (!d) return;
      mount(dbSlot, facts([
        [t('settings.system.size_on_disk'), bytes(d.size_bytes), { mono: true }],
        [t('settings.system.plays'), num(d.plays), { mono: true }],
        [t('settings.system.library_items'), num(d.items), { mono: true }],
        [t('settings.system.oldest_play'), d.oldest_play_at ? dateTime(d.oldest_play_at) : '–', { mono: true }],
      ]));
    }

    // ---- outbound: the privacy promise in the README, assembled from what the running program
    // knows. Nothing is recorded for this list: every line is read from something FinStats already kept.
    async function loadOutbound() {
      if (!isAdmin()) return;
      try {
        const data = await api.get('/outbound', null, { signal: store.signal });
        const rows = (data.destinations || []).map((d) => {
          const [cls, label] = STATE_LABEL[d.state] || STATE_LABEL.off;
          return h('div', { class: 'out-row' },
            h('div', { class: 'out-head' }, h('span', { class: 'out-what' }, d.what), h('span', { class: 'out-state ' + cls }, label)),
            d.hosts && d.hosts.length ? h('div', { class: 'out-hosts mono' }, d.hosts.join(', ')) : null,
            h('p', { class: 'fui-field__help' }, d.why),
            d.last_at ? h('p', { class: 'fui-field__help' }, tParts('settings.system.last_answered', { when: h('span', { title: dateTime(d.last_at) }, relTime(d.last_at)) })) : null,
            d.error ? h('p', { class: 'fui-field__help' }, t('settings.system.last_failed', { error: d.error })) : null);
        });
        const off = (data.total || 0) - (data.reachable || 0);
        mount(outboundSlot, rows,
          h('p', { class: 'fui-field__help' }, t(off ? 'settings.system.reach_some_off' : 'settings.system.reach_all', { on: data.reachable || 0, total: data.total || 0, off }), ' ', t('settings.system.promise')));
      } catch (e) {
        if (isAbort(e) || e.status === 401 || e.status === 403) return;
        mount(outboundSlot, errorState(e, loadOutbound));
      }
    }

    store.onTasks(() => paintDb());
    loadOutbound();
    try { await store.loadTasks(); } catch (e) { mount(dbSlot, errorState(e, () => store.loadTasks())); }
  },
};
