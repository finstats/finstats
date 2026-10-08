// Settings → System: everywhere FinStats can reach, what it holds, and what it is built on. Its jobs
// have a section of their own (Tasks). Nothing here is a setting; it is the machine showing its work.

import { h, icon, num, bytes, relTime, dateTime, mount } from '../dom.js';
import { api, isAbort } from '../api.js';
import { isAdmin, can } from '../state.js';
import { card, sk, errorState, facts } from '../components.js';
import { button } from '../../finui/components/button/button.js';

const STATE_LABEL = { always: ['is-always', 'Always on'], on: ['is-on', 'On'], off: ['', 'Off'] };

export default {
  key: 'system', label: 'System', sub: 'Outbound connections, the database, licences', group: 'Data', icon: 'cpu',
  visible: () => can('manage'),
  entries: [
    { id: 'outbound', label: 'Outbound connections', hint: 'privacy what FinStats reaches hosts telemetry' },
    { id: 'database', label: 'Database', hint: 'size on disk plays items oldest' },
    { id: 'licences', label: 'Licences', hint: 'gpl third-party open source fonts map' },
    { id: 'components', label: 'Component gallery', hint: 'FinUI finui components gallery design system create customize preset theme' },
  ],
  async render(slot, store) {
    const outboundSlot = h('div', { class: 'net-stack' }, sk.rows(3));
    const dbSlot = h('div', null, sk.rows(1));
    mount(slot,
      isAdmin() ? card({ title: 'Outbound connections', sub: 'Everywhere FinStats can reach, and whether it is switched on', body: outboundSlot, id: 'outbound' }) : null,
      card({ title: 'Database', body: dbSlot, id: 'database' }),
      // FinStats' own licence and every third-party one, on their own page: it is half a megabyte
      // of licence text, which belongs where somebody goes looking for it, not in a settings card.
      card({ title: 'Licences', sub: 'GNU GPL v3, on open-source Rust crates, open-licensed fonts and public-domain map data', id: 'licences',
        body: button({ href: '/licenses' }, icon('log', 14), 'Third-party licences', icon('chevronRight', 14)) }),
      // FinUI is its own project: its gallery and FinUI create are a static site of its own, not a page of FinStats.
      card({ title: 'Components', sub: 'FinUI, the components this interface is built from: every one in both themes, and FinUI create to make them yours', id: 'components',
        body: button({ href: 'https://finui.finstats.no/', target: '_blank', rel: 'noopener noreferrer' }, icon('layers', 14), 'FinUI', icon('external', 14)) }));

    function paintDb() {
      const d = store.tasks && store.tasks.db;
      if (!d) return;
      mount(dbSlot, facts([
        ['Size on disk', bytes(d.size_bytes), { mono: true }],
        ['Plays', num(d.plays), { mono: true }],
        ['Library items', num(d.items), { mono: true }],
        ['Oldest play', d.oldest_play_at ? dateTime(d.oldest_play_at) : '–', { mono: true }],
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
            d.last_at ? h('p', { class: 'fui-field__help' }, ['Last answered ', h('span', { title: dateTime(d.last_at) }, relTime(d.last_at))]) : null,
            d.error ? h('p', { class: 'fui-field__help' }, `Last attempt failed: ${d.error}`) : null);
        });
        const off = (data.total || 0) - (data.reachable || 0);
        mount(outboundSlot, rows,
          h('p', { class: 'fui-field__help' }, `${num(data.reachable || 0)} of ${num(data.total || 0)} switched on${off ? `, ${num(off)} off` : ''}. FinStats never sends anything about you or your server to any of these, and there is nothing else: no telemetry, no update check, no fonts or scripts from the internet.`));
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
