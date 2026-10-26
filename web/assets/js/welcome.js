// The welcome: the first time somebody signs in on a browser, and at the end of setup, the same card asks what this
// browser reads FinStats in: the language, how dates and times are written, and which layout. It is asked once a person
// (the server keeps that), and what it asks is the browser's own choice (Settings → Appearance). Skip keeps British English, 31/01/2026,
// 13:05, the device's colours and FinStats. Choosing a language speaks it at once, in place, so setup's steps above are not lost to a reload.

import { h, icon, mount, logo, dateExample, timeExample, setFormats } from './dom.js';
import { setLayoutChoice, markWelcomed, themeChoice, setTheme } from './state.js';
import { t, lang, LANGUAGES, keepLanguage, loadLanguage } from './i18n.js';
import { DATE_FORMATS, TIME_FORMATS } from './when.js';
import { button } from '../finui/components/button/button.js';
import { choiceGroup } from '../finui/components/choice/choice.js';

/** What Skip keeps, and where the choices start: the United Kingdom's way, and FinStats. */
export const DEFAULTS = { language: 'en-GB', date: 'dd/mm/yyyy', time: 'HH:mm', theme: 'device', layout: 'finstats' };
const STEPS = ['language', 'formats', 'theme', 'layout'];

/** Where to go once done: `next` if it is one of FinStats' own pages, else home. Read the way the browser will read it,
 *  never by its first characters: a browser takes `/\host` and `/<tab>/host` for `//host`, another site. */
const safeNext = (next) => {
  let to;
  try { to = new URL(next || '/', location.origin); } catch { return '/'; }
  return to.origin === location.origin && !to.pathname.startsWith('/welcome') ? to.pathname + to.search + to.hash : '/';
};

/** The card. `setup` draws setup's two steps above, done. */
export function welcomeCard({ next = '/', setup = false } = {}) {
  const pick = { ...DEFAULTS, language: LANGUAGES.some((l) => l.tag === lang()) ? lang() : DEFAULTS.language, theme: themeChoice() };
  let step = 0;
  const card = h('div', { class: 'auth-card auth-card-wide welcome' });

  const finish = async (chosen) => {
    keepLanguage(chosen.language);
    setFormats({ date: chosen.date, time: chosen.time }, false);
    setTheme(chosen.theme);
    // Before the page loads again, or the request goes with it. Should it fail, the next visit asks again.
    try { await markWelcomed(); } catch (e) { console.error(e); }
    setLayoutChoice(chosen.layout, safeNext(next)); // loads the page again, in the layout chosen
  };
  const go = (to) => { step = to; draw(); card.querySelector('.wizard-step.is-current select, .wizard-step.is-current input')?.focus(); };

  const select = (id, label, options, value, onPick) => {
    const sel = h('select', { class: 'fui-field__input', id: `w-${id}` }, options.map(([v, text, attrs]) => h('option', { value: v, ...(attrs || {}) }, text)));
    sel.value = value;
    sel.addEventListener('change', () => onPick(sel.value));
    return h('div', { class: 'fui-field' }, h('label', { class: 'fui-field__label', htmlFor: `w-${id}` }, label), sel);
  };
  const body = {
    language: () => select('language', t('settings.appearance.language'),
      LANGUAGES.map((l) => [l.tag, l.checked ? l.name : `${l.name} · ${t('welcome.unchecked')}`, { lang: l.tag }]), pick.language,
      async (tag) => { pick.language = tag; await loadLanguage(tag); draw(); card.querySelector('#w-language')?.focus(); }),
    formats: () => h('div', { class: 'auth-form' },
      select('date', t('settings.appearance.date_format'), DATE_FORMATS.map((f) => [f, dateExample(f)]), pick.date, (v) => { pick.date = v; }),
      select('time', t('settings.appearance.time_format'), TIME_FORMATS.map((f) => [f, timeExample(f)]), pick.time, (v) => { pick.time = v; })),
    // Shown at once: the choice is the page's colours, so the card itself is the preview.
    theme: () => choiceGroup({ label: t('welcome.theme_label'), kind: 'radio', name: 'w-theme', value: pick.theme,
      options: [{ value: 'light', label: t('theme.light'), help: t('welcome.theme_light') },
        { value: 'device', label: t('theme.device'), help: t('welcome.theme_device') },
        { value: 'dark', label: t('theme.dark'), help: t('welcome.theme_dark') }],
      onChange: (v) => { pick.theme = v; setTheme(v); } }),
    layout: () => h('div', { class: 'welcome-layouts', role: 'radiogroup', 'aria-label': t('settings.appearance.layout') },
      [['finstats', 'FinStats', t('welcome.layout_finstats')], ['simple', t('settings.appearance.layout_simple'), t('welcome.layout_simple')]].map(([value, name, line]) =>
        h('label', { class: ['welcome-layout', pick.layout === value && 'is-chosen'] },
          h('input', { type: 'radio', name: 'w-layout', value, checked: pick.layout === value, onChange: () => { pick.layout = value; draw(); card.querySelector(`input[value="${value}"]`)?.focus(); } }),
          h('img', { src: `/assets/welcome/${value}.png`, alt: '', width: 480, height: 300, loading: 'lazy', decoding: 'async' }),
          h('span', { class: 'welcome-layout__name' }, name), h('span', { class: 'welcome-layout__line' }, line)))),
  };

  function draw() {
    const done = (title) => h('li', { class: 'wizard-step is-done' }, h('h2', { class: 'wizard-title' }, title, icon('check', 14)));
    const title = { language: t('welcome.step_language'), formats: t('welcome.step_formats'), theme: t('welcome.step_theme'), layout: t('welcome.step_layout') };
    const steps = STEPS.map((key, i) => {
      const cls = i === step ? 'is-current' : i < step ? 'is-done' : 'is-locked';
      const last = i === STEPS.length - 1;
      return h('li', { class: ['wizard-step', cls] }, h('h2', { class: 'wizard-title' }, title[key], i < step ? icon('check', 14) : null),
        i === step ? [body[key](), h('div', { class: 'form-actions welcome-actions' },
          i > 0 ? button({ variant: 'ghost', type: 'button', onClick: () => go(i - 1) }, t('welcome.back')) : null,
          button({ variant: 'primary', type: 'button', onClick: () => (last ? finish(pick) : go(i + 1)) }, last ? t('welcome.done') : t('welcome.next')))] : null);
    });
    mount(card, [
      h('div', { class: 'auth-brand' }, logo(34), h('span', { class: 'brand-name' }, 'FinStats')),
      h('h1', { class: 'auth-title' }, setup ? t('auth.setup.title') : t('welcome.title')),
      h('p', { class: 'auth-sub' }, t('welcome.sub')),
      h('ol', { class: 'wizard' }, setup ? [done(t('auth.setup.step_connect')), done(t('auth.setup.step_sign_in'))] : null, steps),
      h('div', { class: 'welcome-skip' }, button({ variant: 'ghost', size: 'sm', type: 'button', onClick: () => finish(DEFAULTS) }, t('welcome.skip')),
        h('span', { class: 'fui-field__help' }, t('welcome.skip_note'))),
    ]);
  }
  draw();
  return card;
}

/** `/welcome?next=…`. */
export function welcomePage(ctx) {
  ctx.title(t('welcome.title'));
  ctx.root.append(welcomeCard({ next: ctx.query.get('next') || '/' }));
  requestAnimationFrame(() => ctx.root.querySelector('#w-language')?.focus());
}
