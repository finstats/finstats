// Settings → Appearance: how FinStats looks to you, and to nobody else. A FinUI preset (the code FinUI create makes,
// finui.finstats.no/create), whose tokens the server serves after FinUI's own in /assets/finui.css, to you
// alone. Saving swaps that stylesheet on this page at once; your other pages get it on their next load.
// Which menu this browser has: on a phone one of FinUI mobile-nav's styles, on a wider screen one of desktop-nav's and the side
// it sits on. Kept by this browser (state.js) and nowhere else; each is offered only at the width where it is in use.

import { h, icon, mount } from '../dom.js';
import { api } from '../api.js';
import { card, setBusy, inlineError, facts } from '../components.js';
import { settingRow } from './common.js';
import { button } from '../../finui/components/button/button.js';
import { STYLES, styleOf } from '../../finui/components/mobile-nav/mobile-nav.js';
import { STYLES as DESKTOP_STYLES, styleOf as desktopStyleOf, sideOf } from '../../finui/components/desktop-nav/desktop-nav.js';
import { toggle } from '../../finui/components/toggle/toggle.js';
import { t, tOr, lang, LANGUAGES, storedLanguage, setLanguage } from '../i18n.js';
import { mobileNavChoice, setMobileNav, desktopNavChoice, setDesktopNav, searchMode, setSearchMode, layoutChoice, setLayoutChoice } from '../state.js';

// FinUI create's names for a preset's axes and options, in the language in use where the catalogue has them; a palette's or a
// font's own name (Nord, Inter) is the same in every language.
const presetWord = (kind, key, label) => tOr(`settings.appearance.preset_${kind}.${key}`, label);

const CREATE = 'https://finui.finstats.no/create/';
// Where the mobile menu is in use: app.css' own width for it. The choice is offered only there.
const PHONE = '(max-width: 820px)';
/** The code in anything that holds one: the code itself, the install line, a FinUI create address. */
const codeIn = (text) => (/sh -s --\s+([0-9a-z]+)/.exec(text) || /[?&]preset=([0-9a-z]+)/.exec(text) || /^\s*([0-9a-z]*)\s*$/.exec(text) || [])[1] ?? text.trim();

/** Load /assets/finui.css again and drop the old one once the new one is in, so the page never shows unstyled. */
function restyle() {
  const old = document.querySelector('link[rel="stylesheet"][href^="/assets/finui.css"]');
  if (!old) return;
  const next = h('link', { rel: 'stylesheet', href: `/assets/finui.css?at=${Date.now()}` });
  next.addEventListener('load', () => old.remove(), { once: true });
  old.after(next);
}

const simpleLayout = () => layoutChoice() === 'simple';

export default {
  key: 'appearance', label: t('settings.sections.appearance.label'), sub: t('settings.sections.appearance.sub'), group: t('settings.groups.account'), icon: 'sliders',
  visible: () => true,
  // The finder offers what render() draws: in Simple mode, the layout alone.
  entries: [{ id: 'language', label: t('settings.appearance.language'), hint: t('settings.appearance.language_hint') },
    { id: 'layout', label: t('settings.appearance.layout'), hint: t('settings.appearance.layout_hint') },
    { id: 'desktop_menu', label: t('settings.appearance.menu'), hint: t('settings.appearance.menu_hint'), visible: () => !simpleLayout() && !matchMedia(PHONE).matches },
    { id: 'desktop_side', label: t('settings.appearance.menu_right'), hint: t('settings.appearance.menu_right_hint'), visible: () => !simpleLayout() && !matchMedia(PHONE).matches },
    { id: 'search_mode', label: t('settings.appearance.search_opens'), hint: t('settings.appearance.search_opens_hint'), visible: () => !simpleLayout() },
    { id: 'mobile_menu', label: t('settings.appearance.phone_menu'), hint: t('settings.appearance.phone_menu_hint'), visible: () => !simpleLayout() && matchMedia(PHONE).matches },
    { id: 'finui_preset', label: t('settings.appearance.preset'), hint: t('settings.appearance.preset_hint'), visible: () => !simpleLayout() }],
  async render(slot, store) {
    const body = h('div');
    mount(slot, card({ title: t('settings.sections.appearance.label'), sub: t('settings.appearance.card_sub'), body, id: 'appearance' }));
    const [presets, mine] = await Promise.all([fetch('/assets/finui/create/presets.json', { signal: store.signal }).then((r) => r.json()), api.get('/me/appearance', null, { signal: store.signal })]);
    let current = mine.finui_preset || '';
    const input = h('input', { class: 'fui-field__input mono', id: 'f-finui_preset', type: 'text', autocomplete: 'off', spellcheck: false,
      value: current, placeholder: t('settings.appearance.preset_placeholder'), 'aria-describedby': 'finui_preset-help' });
    const note = h('span', { class: 'saved-note', 'aria-live': 'polite' });
    const err = h('div');
    const meaning = h('div', { class: 'fui-setting-row__notes' });
    const save = button({ size: 'sm' }, t('common.save'));
    const reset = button({ size: 'sm', variant: 'ghost' }, t('settings.appearance.reset'));

    function paint() {
      const code = current;
      reset.hidden = !code;
      const chosen = presets.axes.map((a, i) => { const o = parseInt(code[i] || '0', 36); return o ? [presetWord('axis', a.key, a.label), a.options[o] && presetWord('option', a.options[o].key, a.options[o].label)] : null; }).filter(Boolean);
      mount(meaning,
        chosen.length ? facts(chosen) : h('p', { class: 'fui-field__help' }, t('settings.appearance.preset_default')),
        h('p', { class: 'fui-field__help' }, button({ href: code ? `${CREATE}?preset=${code}` : CREATE, target: '_blank', rel: 'noopener noreferrer', size: 'sm' },
          icon('sliders', 14), code ? t('settings.appearance.preset_change') : t('settings.appearance.preset_make'), icon('external', 14))));
    }
    async function store_(code) {
      mount(err, '');
      setBusy(save, true, t('settings.common.saving'));
      try {
        current = (await api.put('/me/appearance', { finui_preset: code })).finui_preset;
        input.value = current;
        input.removeAttribute('aria-invalid');
        restyle();
        paint();
        note.replaceChildren(icon('check', 13), t('settings.common.saved'));
        setTimeout(() => note.replaceChildren(), 2000);
      } catch (e) {
        input.setAttribute('aria-invalid', 'true');
        mount(err, inlineError('finui_preset-err', e.message));
      } finally { setBusy(save, false); }
    }
    save.addEventListener('click', () => store_(codeIn(input.value)));
    reset.addEventListener('click', () => store_(''));
    input.addEventListener('keydown', (e) => { if (e.key === 'Enter') { e.preventDefault(); save.click(); } });

    // The phone's menu: saved the moment it is picked, and the menu changes behind the page at once.
    const menu = h('select', { class: 'fui-field__input', id: 'f-mobile_menu', 'aria-describedby': 'mobile_menu-help' }, STYLES.map((st) => h('option', { value: st.key }, st.label)));
    menu.value = styleOf(mobileNavChoice());
    const menuNote = h('span', { class: 'saved-note', 'aria-live': 'polite' });
    const [menuRow] = settingRow({ id: 'mobile_menu', label: t('settings.appearance.phone_menu'), labelFor: 'f-mobile_menu',
      help: t('settings.appearance.phone_menu_help'), control: h('div', { class: 'fui-field__row' }, menuNote, menu) });
    menu.addEventListener('change', () => {
      setMobileNav(menu.value);
      menuNote.replaceChildren(icon('check', 13), t('settings.common.saved'));
      setTimeout(() => menuNote.replaceChildren(), 2000);
    });
    // A wide screen's menu, and the side it sits on where it has one (not a dock, not a command bar).
    const saved = (note) => { note.replaceChildren(icon('check', 13), t('settings.common.saved')); setTimeout(() => note.replaceChildren(), 2000); };
    const deskMenu = h('select', { class: 'fui-field__input', id: 'f-desktop_menu', 'aria-describedby': 'desktop_menu-help' }, DESKTOP_STYLES.map((st) => h('option', { value: st.key }, st.label)));
    deskMenu.value = desktopStyleOf(desktopNavChoice().style);
    const deskNote = h('span', { class: 'saved-note', 'aria-live': 'polite' });
    const sideNote = h('span', { class: 'saved-note', 'aria-live': 'polite' });
    const sideSwitch = toggle({ checked: desktopNavChoice().side === 'right', labelledby: 'desktop_side-label', describedby: 'desktop_side-help',
      onChange: (next) => { setDesktopNav({ side: next ? 'right' : 'left' }); saved(sideNote); } });
    const [deskRow] = settingRow({ id: 'desktop_menu', label: t('settings.appearance.menu'), labelFor: 'f-desktop_menu',
      help: t('settings.appearance.menu_help'), control: h('div', { class: 'fui-field__row' }, deskNote, deskMenu) });
    const [sideRow] = settingRow({ id: 'desktop_side', label: t('settings.appearance.menu_right'), help: t('settings.appearance.menu_right_help'), control: [sideNote, sideSwitch] });
    deskMenu.addEventListener('change', () => { setDesktopNav({ style: deskMenu.value }); saved(deskNote); fit(); });

    // Where search opens, on every width: grown into the menu, or in the page's place.
    const modeSel = h('select', { class: 'fui-field__input', id: 'f-search_mode', 'aria-describedby': 'search_mode-help' },
      h('option', { value: 'menu' }, t('settings.appearance.search_in_menu')), h('option', { value: 'page' }, t('settings.appearance.search_in_page')));
    modeSel.value = searchMode();
    const modeNote = h('span', { class: 'saved-note', 'aria-live': 'polite' });
    modeSel.addEventListener('change', () => { setSearchMode(modeSel.value); saved(modeNote); });
    const [modeRow] = settingRow({ id: 'search_mode', label: t('settings.appearance.search_opens'), labelFor: 'f-search_mode',
      help: t('settings.appearance.search_opens_help'), control: h('div', { class: 'fui-field__row' }, modeNote, modeSel) });

    // Which layout this browser wears, on every width. Switching loads the other layout's own modules, so it reloads here.
    const layoutSel = h('select', { class: 'fui-field__input', id: 'f-layout', 'aria-describedby': 'layout-help' },
      h('option', { value: 'finstats' }, 'FinStats'), h('option', { value: 'simple' }, t('settings.appearance.layout_simple')));
    layoutSel.value = layoutChoice();
    layoutSel.addEventListener('change', () => setLayoutChoice(layoutSel.value));
    const [layoutRow] = settingRow({ id: 'layout', label: t('settings.appearance.layout'), labelFor: 'f-layout',
      help: t('settings.appearance.layout_help'), control: layoutSel });

    // The language this browser reads FinStats in, on every width and in both layouts. Choosing one reloads the page in it.
    // The owner reads English and Norwegian and has checked those; the rest are translations nobody here can check, and the
    // list and the help line say so.
    const option = (l) => h('option', { value: l.tag, lang: l.tag }, l.name);
    const langSel = h('select', { class: 'fui-field__input', id: 'f-language', 'aria-describedby': 'language-help' },
      h('option', { value: '' }, t('settings.appearance.language_browser')),
      h('optgroup', { label: t('settings.appearance.language_checked') }, LANGUAGES.filter((l) => l.checked).map(option)),
      h('optgroup', { label: t('settings.appearance.language_unchecked') }, LANGUAGES.filter((l) => !l.checked).map(option)));
    langSel.value = storedLanguage();
    langSel.addEventListener('change', () => setLanguage(langSel.value));
    const unchecked = !(LANGUAGES.find((l) => l.tag === lang()) || {}).checked;
    const [languageRow] = settingRow({ id: 'language', label: t('settings.appearance.language'), labelFor: 'f-language',
      help: unchecked ? `${t('settings.appearance.language_help')} ${t('settings.appearance.language_unchecked_note')}` : t('settings.appearance.language_help'), control: langSel });

    const phone = matchMedia(PHONE);
    // Simple mode has no menu, no search of FinStats' kind and a look of its own: only the layout is chosen there.
    const simple = simpleLayout();
    const fit = () => {
      menuRow.hidden = simple || !phone.matches;
      deskRow.hidden = simple || phone.matches;
      sideRow.hidden = simple || phone.matches || !sideOf(deskMenu.value, 'left');
      modeRow.hidden = simple;
    };
    phone.addEventListener('change', fit, { signal: store.signal });
    fit();

    mount(body, h('div', { class: 'fui-setting-row__rows' },
      languageRow, layoutRow, menuRow, deskRow, sideRow, modeRow,
      simple ? null : settingRow({ id: 'finui_preset', label: t('settings.appearance.preset'), labelFor: 'f-finui_preset',
        help: t('settings.appearance.preset_help'),
        control: h('div', { class: 'fui-field__row' }, note, input, save, reset), error: err })),
      simple ? null : meaning);
    paint();
  },
};
