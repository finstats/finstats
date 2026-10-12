// Settings → Connections: Sonarr, Radarr, Seerr and the torrent clients. Jellyfin administrators only.
// A connection is tested before it is saved (the server does that too), and a key or password is
// write-only: the API says whether one is stored, never what it is.

import { h, icon, mount, relTime, dateTime } from './dom.js';
import { api, isAbort } from './api.js';
import { state } from './state.js';
import { setBusy, inlineError, formField, errorState, sk } from './components.js';
import { button } from '../finui/components/button/button.js';
import { t, tParts } from './i18n.js';

const KINDS = new Set(['sonarr', 'radarr', 'seerr']);

/** A self-managing card body. `ctx` is the page context (signal, every). */
export function connectionsPanel(ctx) {
  const root = h('div', { class: 'net-stack' }, sk.rows(2));
  let data = null;          // { services, kinds }
  let editing = null;       // null | 'new' | service id
  let removing = null;      // service id waiting for the second click
  let lastSig = '';
  let rowErr = null;        // {id, text}: a removal that failed

  async function load({ quiet = false } = {}) {
    try {
      data = await api.get('/services', null, { signal: ctx.signal });
    } catch (e) {
      if (isAbort(e) || e.status === 401) return;
      if (!quiet) mount(root, errorState(e, load));
      return;
    }
    // While a form is open, a background refresh must not wipe what is being typed.
    const sig = JSON.stringify(data);
    if (editing !== null && quiet) return;
    if (sig === lastSig && quiet) return;
    lastSig = sig;
    render();
  }

  function statusOf(s) {
    if (!s.enabled) return h('span', { class: 'fui-badge--status fui-badge--info' }, icon('minus', 13), t('connections.switched_off'));
    if (s.last_error) return h('span', { class: 'fui-badge--status fui-badge--critical' }, icon('alert', 13), h('span', null, t('connections.not_answering', { error: s.last_error })));
    if (s.last_ok_at) return h('span', { class: 'fui-badge--status fui-badge--good' }, icon('check', 13),
      h('span', null, t('connections.connected'), s.version ? h('span', { class: 'mono' }, ` · ${s.version}`) : null, ' · ', ...tParts('connections.checked', { when: h('span', { title: dateTime(s.last_ok_at) }, relTime(s.last_ok_at)) })));
    return h('span', { class: 'fui-badge--status fui-badge--info' }, icon('clock', 13), t('connections.not_checked'));
  }

  function row(s) {
    const act = async (fn) => { try { data = await fn(); } catch (e) { rowErr = { id: s.id, text: e.message }; } removing = null; lastSig = ''; render(); };
    const actions = removing === s.id
      ? [h('span', { class: 'muted' }, t('connections.remove_confirm')),
        button({ size: 'sm', variant: 'danger', type: 'button', onClick: (e) => {
          setBusy(e.currentTarget, true, t('connections.removing'));
          act(async () => {
            const left = await api.del(`/services/${s.id}`);
            // Which pages exist depends on what is connected, as after a save.
            try { state.user = (await api.get('/auth/me')).user; } catch { /* the next navigation will */ }
            return left;
          });
        } }, t('connections.remove')),
        button({ size: 'sm', variant: 'ghost', type: 'button', onClick: () => { removing = null; render(); } }, t('common.cancel'))]
      : [button({ size: 'sm', type: 'button', onClick: () => { editing = s.id; removing = null; render(); } }, t('connections.edit')),
        button({ size: 'sm', variant: 'ghost', tone: 'danger', type: 'button', onClick: () => { removing = s.id; render(); } }, t('connections.remove_ask'))];
    return h('li', { class: 'conn-row' },
      h('div', { class: 'conn-main' },
        h('div', { class: 'conn-name' }, h('strong', null, s.name), s.name !== s.label ? h('span', { class: 'fui-chip' }, s.label) : null),
        h('div', { class: 'conn-url mono' }, s.url),
        h('div', { class: 'conn-status' }, statusOf(s)),
        rowErr && rowErr.id === s.id ? inlineError(`conn-err-${s.id}`, rowErr.text) : null),
      h('div', { class: 'conn-actions' }, actions));
  }
  function form(existing) {
    const kinds = data.kinds;
    let kind = existing ? kinds.find((k) => k.key === existing.kind) : kinds[0];
    const kindSel = h('select', { class: 'fui-field__input', id: 'conn-kind', 'aria-describedby': 'conn-kind-help' }, kinds.map((k) => h('option', { value: k.key }, k.label)));
    const kindHelp = h('p', { class: 'fui-field__help', id: 'conn-kind-help' });
    const name = formField({ id: 'conn-name', label: t('connections.name_label'), autocomplete: 'off', help: t('connections.name_help') });
    const url = formField({ id: 'conn-url', label: t('connections.address'), autocomplete: 'off', inputMode: 'url' });
    const secret = formField({ id: 'conn-secret', label: t('connections.api_key'), type: 'password', autocomplete: 'new-password', help: ' ' });
    const certs = h('input', { type: 'checkbox', id: 'conn-certs', 'aria-describedby': 'conn-certs-help' });
    const enabled = h('input', { type: 'checkbox', id: 'conn-enabled' });
    const result = h('div', { 'aria-live': 'polite' });
    const formErr = h('div');
    const testBtn = button({ type: 'button' }, t('connections.test'));
    const saveBtn = button({ variant: 'primary', type: 'submit' }, existing ? t('common.save') : t('connections.test_and_save'));

    function paintKind() {
      kindHelp.textContent = KINDS.has(kind.key) ? t(`connections.kind.${kind.key}`) : kind.what;
      url.input.placeholder = kind.example;
      secret.el.querySelector('.fui-field__help').textContent = [existing && existing.has_secret ? t('connections.keep_secret') : '', KINDS.has(kind.key) ? t(`connections.secret_help.${kind.key}`) : ''].filter(Boolean).join(' ');
      secret.input.placeholder = existing && existing.has_secret ? t('connections.unchanged') : '';
    }
    if (existing) {
      kindSel.value = existing.kind; kindSel.disabled = true;
      name.input.value = existing.name; url.input.value = existing.url;
      certs.checked = !!existing.accept_invalid_certs; enabled.checked = !!existing.enabled;
    } else enabled.checked = true;
    kindSel.addEventListener('change', () => { kind = kinds.find((k) => k.key === kindSel.value); mount(result, ''); paintKind(); });
    for (const f of [url, secret]) f.input.addEventListener('input', () => { f.setError(''); mount(result, ''); });
    paintKind();

    const body = () => {
      const b = { kind: kind.key, name: name.input.value.trim(), url: url.input.value.trim(), accept_invalid_certs: certs.checked, enabled: enabled.checked };
      if (secret.input.value) b.secret = secret.input.value;
      if (existing) b.id = existing.id;
      return b;
    };
    function valid() {
      let ok = true;
      if (!url.input.value.trim()) { url.setError(t('connections.enter_address', { service: kind.label, example: kind.example })); ok = false; }
      if (!secret.input.value && !(existing && existing.has_secret)) { secret.setError(t('connections.enter_key')); ok = false; }
      if (!ok) (url.input.getAttribute('aria-invalid') === 'true' ? url.input : secret.input).focus();
      return ok;
    }
    // The server's sentence names what is wrong; put it next to the field it is about.
    function place(err) {
      mount(result, '');
      const text = err.message || t('connections.went_wrong');
      // The server says which field an error is about; its words are in the reader's language and are not matched.
      const named = err.field && ({ url, secret })[err.field];
      if (named) { named.setError(text); named.input.focus(); return; }
      if (/API key/i.test(text)) { secret.setError(text); secret.input.focus(); }
      else if (err.status === 400 || err.status === 502) { url.setError(text); url.input.focus(); }
      else mount(formErr, inlineError('conn-form-err', text));
    }
    testBtn.addEventListener('click', async () => {
      mount(formErr, '');
      if (!valid()) return;
      setBusy(testBtn, true, t('connections.testing'));
      try {
        const r = await api.post('/services/test', body());
        mount(result, h('p', { class: 'fui-badge--status fui-badge--good test-ok' }, icon('check', 14), h('span', null, ...tParts('connections.connected_to', { app: h('strong', null, r.app || kind.label) }), r.version ? h('span', { class: 'mono' }, ` · ${r.version}`) : null)));
      } catch (e) { place(e); } finally { setBusy(testBtn, false); }
    });
    const el = h('form', { class: 'conn-form', noValidate: true },
      h('h3', { class: 'conn-form-title' }, existing ? t('connections.edit_title', { name: existing.name }) : t('connections.add')),
      h('div', { class: 'form-grid' },
        h('div', { class: 'fui-field' }, h('label', { class: 'fui-field__label', htmlFor: 'conn-kind' }, t('connections.service')), kindSel, kindHelp),
        name.el, url.el, secret.el,
        h('div', { class: 'fui-field' },
          h('label', { class: 'fui-field__check' }, certs, t('connections.self_signed')),
          h('p', { class: 'fui-field__help', id: 'conn-certs-help' }, t('connections.self_signed_help')),
          existing ? h('label', { class: 'fui-field__check' }, enabled, t('connections.switched_on')) : null)),
      result, formErr,
      h('div', { class: 'form-actions' }, saveBtn, testBtn, button({ variant: 'ghost', type: 'button', onClick: () => { editing = null; render(); } }, t('common.cancel'))));
    el.addEventListener('submit', async (e) => {
      e.preventDefault();
      mount(formErr, '');
      if (!valid()) return;
      setBusy(saveBtn, true, t('connections.testing'));
      try {
        data = existing ? await api.put(`/services/${existing.id}`, body()) : await api.post('/services', body());
        editing = null; lastSig = '';
        // Which pages exist depends on what is connected: ask again who we are and what there is.
        try { state.user = (await api.get('/auth/me')).user; } catch { /* the next navigation will */ }
        render();
      } catch (err) { setBusy(saveBtn, false); place(err); }
    });
    return el;
  }

  function render() {
    if (!data) return;
    const list = data.services.length
      ? h('ul', { class: 'conn-list' }, data.services.map(row))
      : h('p', { class: 'fui-field__help' }, t('connections.none_yet'));
    const existing = typeof editing === 'number' ? data.services.find((s) => s.id === editing) : null;
    mount(root,
      h('p', { class: 'fui-field__help' }, t('connections.intro')),
      list,
      editing === null
        ? h('div', { class: 'form-actions' }, button({ type: 'button', onClick: () => { editing = 'new'; removing = null; render(); } }, icon('plus', 14), t('connections.add')))
        : form(existing));
    if (editing !== null) { const first = root.querySelector(existing ? '#conn-name' : '#conn-kind'); if (first) first.focus(); }
  }

  load();
  ctx.every(() => load({ quiet: true }), 30000, { visibleOnly: true });
  return root;
}
