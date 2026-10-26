// /setup (first run) and /login. Submit buttons stay enabled; we validate on
// submit, show errors next to the field, and only disable while a request runs.

import { h, icon, mount, logo } from '../dom.js';
import { api } from '../api.js';
import { state, resetCaches } from '../state.js';
import { navigate } from '../router.js';
import { setBusy, inlineError, formField as field } from '../components.js';
import { button } from '../../finui/components/button/button.js';
import { t, tParts } from '../i18n.js';
import { welcomeCard } from '../welcome.js';

function brand() {
  return h('div', { class: 'auth-brand' }, logo(34), h('span', { class: 'brand-name' }, 'FinStats'));
}

function normalizeUrl(raw) {
  let u = raw.trim();
  if (!u) return { error: t('auth.setup.address_empty') };
  if (!/^https?:\/\//i.test(u)) u = 'http://' + u;
  // Chrome escapes a space in a host rather than refusing it, so a host it had to escape is not one.
  try { const parsed = new URL(u); if (!parsed.hostname || /[\s%]/.test(parsed.hostname)) throw new Error(); } catch { return { error: t('auth.setup.address_invalid') }; }
  return { url: u.replace(/\/+$/, '') };
}

// ---------------------------------------------------------------- /setup
export function setupPage(ctx) {
  ctx.title(t('auth.setup.page_title'));
  let tested = null; // {url, server_name, version}

  // The one-time code FinStats wrote to its log: until setup is done, whoever reaches the port could otherwise run it.
  const code = field({ id: 'setup-code', label: t('auth.setup.code'), placeholder: 'XXXX-XXXX', autocomplete: 'off', help: t('auth.setup.code_help') });
  code.input.spellcheck = false;
  const url = field({ id: 'jf-url', label: t('auth.setup.address'), placeholder: 'http://jellyfin:8096', inputMode: 'url', autocomplete: 'url',
    help: t('auth.setup.address_help') });
  const testBtn = button({ variant: 'primary', type: 'submit' }, t('auth.setup.test'));
  const testResult = h('div', { 'aria-live': 'polite' });
  const step1 = h('form', { class: 'auth-form', noValidate: true }, code.el, url.el, h('div', { class: 'form-actions' }, testBtn), testResult);

  const user = field({ id: 'username', label: t('auth.setup.admin_username'), autocomplete: 'username' });
  const pass = field({ id: 'password', label: t('auth.login.password'), type: 'password', autocomplete: 'current-password' });
  const finishBtn = button({ variant: 'primary', type: 'submit' }, t('auth.setup.finish'));
  const finishErr = h('div');
  const step2 = h('form', { class: 'auth-form', noValidate: true, hidden: true },
    user.el, pass.el,
    h('p', { class: 'fui-field__help' }, icon('shield', 13), ' ' + t('auth.setup.password_note')),
    h('div', { class: 'form-actions' }, finishBtn), finishErr);

  const s1 = h('li', { class: 'wizard-step is-current' }, h('h2', { class: 'wizard-title' }, t('auth.setup.step_connect')), step1);
  const s2 = h('li', { class: 'wizard-step is-locked' }, h('h2', { class: 'wizard-title' }, t('auth.setup.step_sign_in')),
    h('p', { class: 'fui-field__help wizard-locked-note' }, t('auth.setup.test_first')), step2);

  url.input.addEventListener('input', () => {
    if (tested) { tested = null; mount(testResult, ''); step2.hidden = true; s2.classList.add('is-locked'); s1.classList.add('is-current'); }
  });

  step1.addEventListener('submit', async (e) => {
    e.preventDefault();
    const said = code.input.value.trim();
    code.setError(said ? null : t('auth.setup.code_empty'));
    if (!said) { code.input.focus(); return; }
    const n = normalizeUrl(url.input.value);
    url.setError(n.error || null);
    if (n.error) { url.input.focus(); return; }
    url.input.value = n.url;
    setBusy(testBtn, true, t('auth.setup.testing'));
    try {
      const info = await api.post('/setup/test', { url: n.url, code: said }, { quiet401: true });
      tested = { url: n.url, ...info };
      mount(testResult, h('p', { class: 'fui-badge--status fui-badge--good test-ok' }, icon('check', 14), h('span', null, ...tParts('auth.setup.connected', { server: h('strong', null, info.server_name || 'Jellyfin') }), info.version ? h('span', { class: 'mono' }, ` · Jellyfin ${info.version}`) : null)));
      step2.hidden = false; s2.classList.remove('is-locked'); s1.classList.remove('is-current');
      user.input.focus();
    } catch (err) {
      tested = null;
      if (err.status === 409) { state.status = await api.get('/status'); navigate('/login', { replace: true }); return; }
      if (err.status === 428) { code.setError(t('auth.setup.code_wrong')); code.input.select(); code.input.focus(); return; }
      // A refusal of the address itself is not a failed connection; and the server's own words are one sentence, finished.
      const said = /[.!?]$/.test(err.message) ? err.message : `${err.message}.`;
      url.setError(err.status === 400 ? said : t('auth.setup.couldnt_connect', { reason: said }));
      url.input.focus();
    } finally { setBusy(testBtn, false); }
  });

  step2.addEventListener('submit', async (e) => {
    e.preventDefault();
    mount(finishErr, '');
    const u = user.input.value.trim(), p = pass.input.value;
    user.setError(u ? null : t('auth.login.username_empty'));
    if (!u) { user.input.focus(); return; }
    if (!tested) { url.setError(t('auth.setup.test_first')); url.input.focus(); return; }
    setBusy(finishBtn, true, t('auth.setup.finishing'));
    try {
      const res = await api.post('/setup', { url: tested.url, username: u, password: p, code: code.input.value.trim() }, { quiet401: true });
      state.user = res.user;
      try { state.status = await api.get('/status'); } catch { state.status = { ...state.status, configured: true, server_name: tested.server_name }; }
      // The same card goes on with the welcome's questions, setup's two steps shown done above them.
      history.replaceState(history.state, '', '/welcome');
      ctx.root.replaceChildren(welcomeCard({ setup: true }));
      ctx.root.querySelector('#w-language')?.focus();
    } catch (err) {
      setBusy(finishBtn, false);
      if (err.status === 428) { code.setError(t('auth.setup.code_wrong')); code.input.select(); code.input.focus(); }
      else if (err.status === 401) { pass.setError(t('auth.login.wrong')); pass.input.select(); pass.input.focus(); }
      else if (err.status === 403) { user.setError(t('auth.setup.not_admin')); user.input.focus(); }
      else mount(finishErr, inlineError('setup-err', err.message));
    }
  });

  ctx.root.append(h('div', { class: 'auth-card auth-card-wide' }, brand(),
    h('h1', { class: 'auth-title' }, t('auth.setup.title')),
    h('p', { class: 'auth-sub' }, t('auth.setup.sub')),
    h('ol', { class: 'wizard' }, s1, s2)));
  requestAnimationFrame(() => code.input.focus());
}

// ---------------------------------------------------------------- /login
export function loginPage(ctx) {
  ctx.title(t('auth.login.title'));
  const server = state.status && state.status.server_name;
  const user = field({ id: 'username', label: t('auth.login.username'), autocomplete: 'username' });
  const pass = field({ id: 'password', label: t('auth.login.password'), type: 'password', autocomplete: 'current-password' });
  const btn = button({ variant: 'primary', block: true, type: 'submit' }, t('auth.login.sign_in'));
  const formErr = h('div');
  const form = h('form', { class: 'auth-form', noValidate: true }, user.el, pass.el, formErr, btn);

  form.addEventListener('submit', async (e) => {
    e.preventDefault();
    mount(formErr, '');
    const u = user.input.value.trim(), p = pass.input.value;
    user.setError(u ? null : t('auth.login.username_empty'));
    pass.setError(null);
    if (!u) { user.input.focus(); return; }
    setBusy(btn, true, t('auth.login.signing_in'));
    try {
      const res = await api.post('/auth/login', { username: u, password: p }, { quiet401: true });
      state.user = res.user;
      resetCaches();
      const next = ctx.query.get('next');
      navigate(next && next.startsWith('/') && !next.startsWith('//') ? next : '/', { replace: true });
    } catch (err) {
      setBusy(btn, false);
      if (err.status === 428) { code.setError(t('auth.setup.code_wrong')); code.input.select(); code.input.focus(); }
      else if (err.status === 401) { pass.setError(t('auth.login.wrong')); pass.input.select(); pass.input.focus(); }
      else if (err.status === 403) mount(formErr, inlineError('login-err', t('auth.login.admins_only')));
      else if (err.status === 429) mount(formErr, inlineError('login-err', t('auth.login.too_many')));
      else mount(formErr, inlineError('login-err', err.message));
    }
  });

  ctx.root.append(h('div', { class: 'auth-card' }, brand(),
    h('h1', { class: 'auth-title' }, t('auth.login.title')),
    h('p', { class: 'auth-sub' }, server ? tParts('auth.login.sub_on', { server: h('strong', null, server) }) : t('auth.login.sub')),
    form));
  requestAnimationFrame(() => user.input.focus());
}
