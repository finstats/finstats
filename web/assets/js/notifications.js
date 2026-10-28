// Settings → Notifications: where what FinStats finds is sent. A destination belongs either to the
// server (Jellyfin administrators) or to one person, and a person's own destination is only ever sent
// what they may already see in the app. The server decides that; this only draws it.
//
// The address is write-only, like a key: a Discord webhook URL carries its own token, so the API
// answers the host and never the URL. Editing without re-typing it keeps the stored one.

import { h, icon, mount, relTime, dateTime } from './dom.js';
import { api, isAbort } from './api.js';
import { isAdmin, can, state } from './state.js';
import { setBusy, inlineError, formField, errorState, sk } from './components.js';
import { dataTable } from './tables.js';
import { button } from '../finui/components/button/button.js';
import { t, tParts } from './i18n.js';

const SECRET_HELP = new Set(['webhook', 'ntfy', 'gotify', 'telegram', 'pushover', 'pushbullet', 'email']);

// The address field, where a channel needs more said about it than “post here”.
const ADDRESS_HELP = new Set(['discord', 'slack', 'email']);

const STATE_LABEL = new Set(['sent', 'queued', 'failed']);

const CHANNELS = new Set(['discord', 'telegram', 'slack', 'email', 'ntfy', 'gotify', 'pushover', 'pushbullet', 'webhook']);
const EVENTS = new Set(['travel', 'new_country', 'failed_sign_ins', 'task_failed', 'backup_failed', 'service_down', 'service_back',
  'new_items', 'request_available', 'play_started', 'play_stopped', 'recap_ready', 'watchlist_available', 'test']);
const GROUPS = new Set(['security', 'housekeeping', 'library', 'playback']);
const EXTRAS = new Set(['from', 'username']);
const said = (c, field) => (c[field] && CHANNELS.has(c.key) ? t(`notifications.channel.${c.key}.${field}`) : c[field]);
const channelName = (key, label) => (CHANNELS.has(key) ? t(`notifications.channel.${key}.label`) : label);
const eventSaid = (e, field) => (EVENTS.has(e.key) ? t(`notifications.event.${e.key}.${field}`) : e[field]);
const extraSaid = (e, field) => (EXTRAS.has(e.key) ? t(`notifications.extra.${e.key}.${field}`) : e[field]);

export function notificationsPanel(ctx) {
  const root = h('div', { class: 'net-stack' }, sk.rows(2));
  let data = null;         // { targets, catalogue, public_url, can_add_server, max_own }
  let history = null;      // { events }
  let editing = null;      // null | 'new' | id
  let removing = null;     // id awaiting the second click
  let rowMsg = null;       // { id, ok, text }: the result of a test or a failed removal
  let lastSig = '';

  async function load({ quiet = false } = {}) {
    try {
      const [next, sent] = await Promise.all([
        api.get('/notifications', null, { signal: ctx.signal }),
        api.get('/notifications/history', { limit: 25 }, { signal: ctx.signal }),
      ]);
      data = next;
      history = sent;
    } catch (e) {
      if (isAbort(e) || e.status === 401 || e.status === 403) return;
      if (!quiet) mount(root, errorState(e, load));
      return;
    }
    const sig = JSON.stringify([data, history]);
    if (editing !== null && quiet) return;   // a background refresh must not wipe what is being typed
    if (sig === lastSig && quiet) return;
    lastSig = sig;
    render();
  }

  const channelOf = (key) => data.catalogue.channels.find((c) => c.key === key) || data.catalogue.channels[0];

  function statusOf(d) {
    if (!d.enabled) return h('span', { class: 'fui-badge--status fui-badge--info' }, icon('minus', 13), t('notifications.switched_off'));
    if (d.last_error) return h('span', { class: 'fui-badge--status fui-badge--critical' }, icon('alert', 13), h('span', null, t('notifications.last_failed', { error: d.last_error })));
    if (d.last_ok_at) return h('span', { class: 'fui-badge--status fui-badge--good' }, icon('check', 13),
      h('span', null, ...tParts('notifications.last_ok', { when: h('span', { title: dateTime(d.last_ok_at) }, relTime(d.last_ok_at)) })));
    return h('span', { class: 'fui-badge--status fui-badge--info' }, icon('clock', 13), t('notifications.nothing_sent'));
  }

  function row(dest) {
    const act = async (fn) => {
      try { await fn(); rowMsg = null; } catch (e) { rowMsg = { id: dest.id, ok: false, text: e.message }; }
      removing = null; lastSig = '';
      await load();
    };
    const test = button({ size: 'sm', type: 'button' }, t('notifications.test'));
    test.addEventListener('click', async () => {
      setBusy(test, true, t('notifications.sending'));
      try {
        const r = await api.post(`/notifications/targets/${dest.id}/test`);
        rowMsg = { id: dest.id, ok: r.ok, text: r.ok ? t('notifications.test_sent') : r.error };
      } catch (e) { rowMsg = { id: dest.id, ok: false, text: e.message }; }
      setBusy(test, false);
      lastSig = '';
      await load();
    });
    const actions = removing === dest.id
      ? [h('span', { class: 'muted' }, t('notifications.remove_confirm')),
        button({ size: 'sm', variant: 'danger', type: 'button', onClick: (e) => { setBusy(e.currentTarget, true, t('notifications.removing')); act(() => api.del(`/notifications/targets/${dest.id}`)); } }, t('notifications.remove')),
        button({ size: 'sm', variant: 'ghost', type: 'button', onClick: () => { removing = null; render(); } }, t('common.cancel'))]
      : [test,
        button({ size: 'sm', type: 'button', onClick: () => { editing = dest.id; removing = null; rowMsg = null; render(); } }, t('notifications.edit')),
        button({ size: 'sm', variant: 'ghost', tone: 'danger', type: 'button', onClick: () => { removing = dest.id; render(); } }, t('notifications.remove_ask'))];
    const ticked = dest.events.length;
    const total = data.catalogue.events.length;
    const about = [t('notifications.events_ticked', { ticked, total })];
    if (dest.with_addresses) about.push(t('notifications.addresses_included'));
    if (dest.min_severity !== 'info') about.push(t(`notifications.severity_above.${dest.min_severity}`));
    return h('li', { class: 'conn-row' },
      h('div', { class: 'conn-main' },
        h('div', { class: 'conn-name' }, h('strong', null, dest.name), h('span', { class: 'fui-chip' }, channelName(dest.kind, dest.label)),
          // "me" is any personal destination; an administrator sees other people's too.
          dest.scope !== 'me' ? null : dest.owner_id === state.user?.id ? h('span', { class: 'fui-chip' }, t('notifications.yours')) : h('span', { class: 'fui-chip' }, dest.owner_name ? t('notifications.owners', { name: dest.owner_name }) : t('notifications.somebodys'))),
        h('div', { class: 'conn-url mono' }, dest.shown),
        h('div', { class: 'conn-status' }, statusOf(dest)),
        h('p', { class: 'fui-field__help' }, about.join(' · ')),
        rowMsg && rowMsg.id === dest.id
          ? (rowMsg.ok ? h('p', { class: 'fui-badge--status fui-badge--good' }, icon('check', 13), rowMsg.text) : inlineError(`notify-err-${dest.id}`, rowMsg.text))
          : null),
      h('div', { class: 'conn-actions' }, actions));
  }

  function form(existing) {
    const channels = data.catalogue.channels;
    let channel = existing ? channelOf(existing.kind) : channels[0];
    const kindSel = h('select', { class: 'fui-field__input', id: 'notify-kind', 'aria-describedby': 'notify-kind-help' }, channels.map((c) => h('option', { value: c.key }, channelName(c.key, c.label))));
    const kindHelp = h('p', { class: 'fui-field__help', id: 'notify-kind-help' });
    const name = formField({ id: 'notify-name', label: t('notifications.name_label'), autocomplete: 'off', help: t('notifications.name_help') });
    const url = formField({ id: 'notify-url', label: t('notifications.address'), autocomplete: 'off', inputMode: 'url', help: ' ' });
    const topic = formField({ id: 'notify-topic', label: t('notifications.topic'), autocomplete: 'off', help: ' ' });
    // One field per extra any channel asks for (only mail has any), shown for the one being added.
    const extraKeys = [...new Set(channels.flatMap((c) => (c.extras || []).map((e) => e.key)))];
    const extras = new Map(extraKeys.map((key) => [key, formField({ id: `notify-x-${key}`, label: key, autocomplete: 'off', help: ' ' })]));
    const specOf = (key) => (channel.extras || []).find((e) => e.key === key);
    const secret = formField({ id: 'notify-secret', label: t('notifications.token'), type: 'password', autocomplete: 'new-password', help: ' ' });
    const severity = h('select', { class: 'fui-field__input', id: 'notify-sev' }, data.catalogue.severities.map((s) => h('option', { value: s }, ['info', 'warn', 'alert'].includes(s) ? t(`notifications.severity.${s}`) : s)));
    const addresses = h('input', { type: 'checkbox', id: 'notify-addresses', 'aria-describedby': 'notify-addresses-help' });
    const certs = h('input', { type: 'checkbox', id: 'notify-certs', 'aria-describedby': 'notify-certs-help' });
    const enabled = h('input', { type: 'checkbox', id: 'notify-enabled' });
    const scopeServer = h('input', { type: 'radio', name: 'notify-scope', id: 'notify-scope-server', checked: true });
    const scopeMine = h('input', { type: 'radio', name: 'notify-scope', id: 'notify-scope-me' });
    const formErr = h('div');
    const saveBtn = button({ variant: 'primary', type: 'submit' }, existing ? t('common.save') : t('notifications.add_destination'));

    // One tick per kind of event, grouped the way the catalogue groups them.
    const ticks = new Map();
    // What is only ever somebody's own (their watchlist) starts unticked, as the chatty playback kinds do.
    const chosen = existing ? existing.events : data.catalogue.events.filter((e) => e.group !== 'playback' && !e.own_only).map((e) => e.key);
    const ownOnly = new Map();   // kind → its row, offered only to a destination of one's own
    const groups = data.catalogue.groups.map((g) => {
      const events = data.catalogue.events.filter((e) => e.group === g.key);
      if (!events.length) return null;
      return h('div', { class: 'notify-group' },
        h('h4', { class: 'section-label' }, GROUPS.has(g.key) ? t(`notifications.group.${g.key}`) : g.label),
        events.map((e) => {
          const box = h('input', { type: 'checkbox', id: `notify-ev-${e.key}`, checked: chosen.includes(e.key) });
          ticks.set(e.key, box);
          const row = h('label', { class: 'fui-field__check notify-event' }, box, h('span', null, h('span', { class: 'notify-event-label' }, eventSaid(e, 'label')), h('span', { class: 'fui-field__help' }, eventSaid(e, 'what'))));
          if (e.own_only) ownOnly.set(e.key, row);
          return row;
        }));
    });

    function paintKind() {
      kindHelp.textContent = said(channel, 'what');
      // A channel FinStats already knows the address of asks for no address at all.
      url.el.hidden = !!channel.fixed_url;
      url.input.placeholder = existing && existing.kind === channel.key ? t('notifications.unchanged') : channel.example;
      url.el.querySelector('.fui-field__help').textContent = ADDRESS_HELP.has(channel.key) ? t(`notifications.address_help.${channel.key}`) : t('notifications.address_help_any', { example: channel.example });
      topic.el.hidden = !channel.needs_topic;
      if (channel.needs_topic) {
        topic.el.querySelector('.fui-field__label').textContent = said(channel, 'topic_label');
        topic.el.querySelector('.fui-field__help').textContent = said(channel, 'topic_help');
        // Somebody else's topic is not shown, not even to an administrator: it stays as it is unless typed again.
        topic.input.placeholder = existing && existing.topic_hidden ? t('notifications.unchanged') : channel.topic_example;
      }
      for (const [key, field] of extras) {
        const spec = specOf(key);
        field.el.hidden = !spec;
        if (!spec) continue;
        field.el.querySelector('.fui-field__label').textContent = extraSaid(spec, 'label');
        field.el.querySelector('.fui-field__help').textContent = extraSaid(spec, 'help');
        field.input.placeholder = spec.example;
      }
      secret.el.hidden = !channel.secret_label;
      if (channel.secret_label) {
        secret.el.querySelector('.fui-field__label').textContent = said(channel, 'secret_label');
        secret.el.querySelector('.fui-field__help').textContent = [existing && existing.has_secret ? t('notifications.keep_secret') : '', SECRET_HELP.has(channel.key) ? t(`notifications.secret_help.${channel.key}`) : ''].filter(Boolean).join(' ');
        secret.input.placeholder = existing && existing.has_secret ? t('notifications.unchanged') : '';
      }
    }
    if (existing) {
      kindSel.value = existing.kind; kindSel.disabled = true;
      name.input.value = existing.name;
      topic.input.value = existing.topic || '';
      for (const [key, field] of extras) field.input.value = (existing.options && existing.options[key]) || '';
      severity.value = existing.min_severity;
      addresses.checked = !!existing.with_addresses;
      certs.checked = !!existing.accept_invalid_certs;
      enabled.checked = !!existing.enabled;
      scopeMine.checked = existing.scope === 'me';
      scopeServer.checked = existing.scope !== 'me';
    } else {
      enabled.checked = true;
      scopeServer.checked = !!data.can_add_server;
      scopeMine.checked = !data.can_add_server;
    }
    kindSel.addEventListener('change', () => { channel = channelOf(kindSel.value); paintKind(); });
    // A destination of the server's is never sent what is somebody's own, so it is not offered one.
    function paintScope() {
      const mine = existing ? existing.scope === 'me' : scopeMine.checked;
      for (const [key, row] of ownOnly) { row.hidden = !mine; if (!mine) ticks.get(key).checked = false; }
    }
    for (const r of [scopeServer, scopeMine]) r.addEventListener('change', paintScope);
    paintScope();
    for (const f of [url, secret, topic, ...extras.values()]) f.input.addEventListener('input', () => f.setError(''));
    paintKind();

    const body = () => {
      const b = {
        kind: channel.key, name: name.input.value.trim(), events: [...ticks].filter(([, box]) => box.checked).map(([key]) => key),
        with_addresses: addresses.checked, min_severity: severity.value, accept_invalid_certs: certs.checked, enabled: enabled.checked,
      };
      if (!existing) b.scope = scopeMine.checked ? 'me' : 'server';
      if (url.input.value.trim() && !channel.fixed_url) b.url = url.input.value.trim();
      if (secret.input.value) b.secret = secret.input.value;
      if (channel.needs_topic && topic.input.value.trim()) b.topic = topic.input.value.trim();
      if (channel.extras && channel.extras.length) {
        b.options = {};
        for (const e of channel.extras) b.options[e.key] = extras.get(e.key).input.value.trim();
      }
      return b;
    };
    function valid() {
      let ok = true;
      if (!existing && !url.input.value.trim() && !channel.fixed_url) { url.setError(t('notifications.enter_address', { example: channel.example })); ok = false; }
      if (channel.needs_topic && !topic.input.value.trim() && !(existing && existing.topic_hidden)) { topic.setError(CHANNELS.has(channel.key) ? t(`notifications.channel.${channel.key}.topic_missing`) : t('notifications.enter_value', { what: channel.topic_label.toLowerCase() })); ok = false; }
      for (const e of channel.extras || []) {
        if (e.required && !extras.get(e.key).input.value.trim()) { extras.get(e.key).setError(EXTRAS.has(e.key) ? t(`notifications.extra.${e.key}.missing`) : t('notifications.enter_value', { what: e.label.toLowerCase() })); ok = false; }
      }
      if (channel.secret_required && !secret.input.value && !(existing && existing.has_secret)) { secret.setError(CHANNELS.has(channel.key) ? t(`notifications.channel.${channel.key}.secret_missing`) : t('notifications.enter_value', { what: channel.secret_label })); ok = false; }
      if (!ok) root.querySelector('[aria-invalid="true"]').focus();
      return ok;
    }
    function place(err) {
      const text = err.message || t('notifications.went_wrong');
      // The server says which field an error is about; its words are in the reader's language and are not matched.
      const fields = { url, topic, secret };
      const named = err.field && (fields[err.field] || extras.get(err.field));
      if (named) { named.setError(text); named.input.focus(); return; }
      const extra = (channel.extras || []).find((e) => text.toLowerCase().includes(e.label.toLowerCase()));
      if (extra) { const f = extras.get(extra.key); f.setError(text); f.input.focus(); }
      else if (channel.needs_topic && text.toLowerCase().includes(channel.topic_label.toLowerCase())) { topic.setError(text); topic.input.focus(); }
      else if (/topic|chat|mailbox/i.test(text)) { topic.setError(text); topic.input.focus(); }
      else if (/token|key/i.test(text)) { secret.setError(text); secret.input.focus(); }
      else if (err.status === 400) { url.setError(text); url.input.focus(); }
      else mount(formErr, inlineError('notify-form-err', text));
    }

    const el = h('form', { class: 'conn-form', noValidate: true },
      h('h3', { class: 'conn-form-title' }, existing ? t('notifications.edit_title', { name: existing.name }) : t('notifications.add_title')),
      h('div', { class: 'form-grid' },
        h('div', { class: 'fui-field' }, h('label', { class: 'fui-field__label', htmlFor: 'notify-kind' }, t('notifications.kind')), kindSel, kindHelp),
        name.el, url.el, topic.el, ...[...extras.values()].map((f) => f.el), secret.el,
        !existing && data.can_add_server
          ? h('div', { class: 'fui-field' }, h('span', { class: 'fui-field__label' }, t('notifications.who_for')),
            h('label', { class: 'fui-field__check' }, scopeServer, t('notifications.scope_server')),
            h('label', { class: 'fui-field__check' }, scopeMine, t('notifications.scope_me')))
          : null,
        h('div', { class: 'fui-field' }, h('label', { class: 'fui-field__label', htmlFor: 'notify-sev' }, t('notifications.how_much')), severity)),
      h('div', { class: 'fui-field' },
        h('span', { class: 'fui-field__label' }, t('notifications.what_to_send')),
        h('div', { class: 'notify-events' }, groups)),
      h('div', { class: 'fui-field' },
        // Addresses reach a personal destination only with see_network, as they do on every page.
        can('see_network') ? h('label', { class: 'fui-field__check' }, addresses, t('notifications.with_addresses')) : null,
        can('see_network') ? h('p', { class: 'fui-field__help', id: 'notify-addresses-help' }, t('notifications.with_addresses_help')) : null,
        isAdmin() ? h('label', { class: 'fui-field__check' }, certs, t('notifications.self_signed')) : null,
        isAdmin() ? h('p', { class: 'fui-field__help', id: 'notify-certs-help' }, t('notifications.self_signed_help')) : null,
        existing ? h('label', { class: 'fui-field__check' }, enabled, t('notifications.switched_on')) : null),
      formErr,
      h('div', { class: 'form-actions' }, saveBtn, button({ variant: 'ghost', type: 'button', onClick: () => { editing = null; render(); } }, t('common.cancel'))));
    el.addEventListener('submit', async (e) => {
      e.preventDefault();
      mount(formErr, '');
      if (!valid()) return;
      setBusy(saveBtn, true, t('notifications.saving'));
      try {
        if (existing) await api.put(`/notifications/targets/${existing.id}`, body());
        else await api.post('/notifications/targets', body());
        editing = null; lastSig = '';
        await load();
      } catch (err) { setBusy(saveBtn, false); place(err); }
    });
    return el;
  }

  function sentTable() {
    const events = (history && history.events) || [];
    if (!events.length) return h('p', { class: 'fui-field__help' }, t('notifications.history_empty'));
    const delivery = (d) => h('div', { class: 'notify-delivery' },
      h('span', { class: ['fui-badge--status', d.state === 'sent' ? 'fui-badge--good' : d.state === 'failed' ? 'fui-badge--critical' : 'fui-badge--info'] },
        icon(d.state === 'sent' ? 'check' : d.state === 'failed' ? 'alert' : 'clock', 12),
        `${STATE_LABEL.has(d.state) ? t(`notifications.state.${d.state}`) : d.state}${d.target ? ' · ' + d.target : ''}`),
      d.error ? h('span', { class: 'cell-sub' }, d.error) : null);
    const went = (e) => {
      if (e.historic) return h('span', { class: 'muted' }, t('notifications.historic'));
      if (!e.deliveries.length) return h('span', { class: 'muted' }, t('notifications.nobody_listening'));
      return e.deliveries.map(delivery);
    };
    const line = (e) => h('tr', null,
      h('td', null, h('span', { class: 'when-cell' },
        h('time', { dateTime: new Date(e.at * 1000).toISOString(), title: dateTime(e.at) }, relTime(e.at)),
        h('span', { class: 'cell-sub' }, EVENTS.has(e.kind) ? t(`notifications.event.${e.kind}.label`) : e.label || e.kind))),
      h('td', null, h('div', { class: 'notify-what' }, h('span', null, e.title), e.body ? h('span', { class: 'cell-sub' }, e.body) : null)),
      h('td', { class: 'notify-sent' }, went(e)));
    const table = h('table', { class: 'fui-data-table' },
      h('thead', null, h('tr', null, h('th', null, t('notifications.col_when')), h('th', null, t('notifications.col_what')), h('th', { class: 'notify-sent' }, t('notifications.col_sent_to')))),
      h('tbody', null, events.map(line)));
    // Its own scroll box: a message and where it went are wide, and the card must not push the page sideways.
    return dataTable(table, { filter: false });
  }

  function publicUrlField() {
    if (!isAdmin()) return null;
    const field = formField({ id: 'notify-public-url', label: t('notifications.public_url'), autocomplete: 'off', inputMode: 'url',
      help: t('notifications.public_url_help') });
    field.input.value = data.public_url || '';
    field.input.placeholder = 'https://finstats.example';
    const save = button({ size: 'sm', type: 'button' }, t('common.save'));
    save.addEventListener('click', async () => {
      setBusy(save, true, t('notifications.saving'));
      try {
        await api.put('/settings', { public_url: field.input.value.trim() });
        // What /auth/me told this page at its start: API keys makes a calendar's address with it, in this tab too.
        if (state.user) state.user.public_url = field.input.value.trim().replace(/\/+$/, '') || null;
        field.setError('');
        lastSig = '';
        await load();
      } catch (e) { field.setError(e.message); } finally { setBusy(save, false); }
    });
    return h('div', { class: 'notify-public' }, field.el, h('div', { class: 'form-actions' }, save));
  }

  function render() {
    if (!data) return;
    const targets = data.targets || [];
    const existing = typeof editing === 'number' ? targets.find((t) => t.id === editing) : null;
    const intro = isAdmin()
      ? t('notifications.intro_admin')
      : t('notifications.intro_own');
    mount(root,
      h('p', { class: 'fui-field__help' }, intro),
      publicUrlField(),
      targets.length ? h('ul', { class: 'conn-list' }, targets.map(row)) : h('p', { class: 'fui-field__help' }, t('notifications.none_yet')),
      editing === null
        ? h('div', { class: 'form-actions' }, button({ type: 'button', onClick: () => { editing = 'new'; removing = null; rowMsg = null; render(); } }, icon('plus', 14), t('notifications.add_title')))
        : form(existing),
      h('div', null, h('h3', { class: 'section-label' }, t('notifications.recently_sent')), sentTable()));
    if (editing !== null) { const first = root.querySelector(existing ? '#notify-name' : '#notify-kind'); if (first) first.focus(); }
  }

  load();
  ctx.every(() => load({ quiet: true }), 30000, { visibleOnly: true });
  return root;
}

/** Who sees the card at all. The server enforces the same rule on every endpoint. */
export const mayNotify = () => isAdmin() || can('notify');
