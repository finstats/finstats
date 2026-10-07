// Settings → Appearance: how finstats looks to you, and to nobody else. A FinUI preset (the code FinUI create makes,
// finstats.github.io/finui/create), whose tokens the server serves after FinUI's own in /assets/finui.css, to you
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
import { mobileNavChoice, setMobileNav, desktopNavChoice, setDesktopNav, searchMode, setSearchMode } from '../state.js';

const CREATE = 'https://finstats.github.io/finui/create/';
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

export default {
  key: 'appearance', label: 'Appearance', sub: 'How finstats looks to you', group: 'Account', icon: 'sliders',
  visible: () => true,
  entries: [{ id: 'desktop_menu', label: 'Menu', hint: 'menu sidebar navigation grouped rail search dock command bar pinned tiles', visible: () => !matchMedia(PHONE).matches },
    { id: 'desktop_side', label: 'Menu on the right', hint: 'menu sidebar right left side', visible: () => !matchMedia(PHONE).matches },
    { id: 'search_mode', label: 'Search opens', hint: 'search find menu page integrated where ctrl space' },
    { id: 'mobile_menu', label: 'Menu on this phone', hint: 'mobile phone menu navigation tab bar peek full screen thumb arc address', visible: () => matchMedia(PHONE).matches },
    { id: 'finui_preset', label: 'FinUI preset', hint: 'appearance look theme colours accent radius density borders cards buttons fields tables menu motion icons contrast finui preset code customize create' }],
  async render(slot, store) {
    const body = h('div');
    mount(slot, card({ title: 'Appearance', sub: 'How finstats looks to you: your choice, nobody else’s', body, id: 'appearance' }));
    const [presets, mine] = await Promise.all([fetch('/assets/finui/create/presets.json', { signal: store.signal }).then((r) => r.json()), api.get('/me/appearance', null, { signal: store.signal })]);
    let current = mine.finui_preset || '';
    const input = h('input', { class: 'fui-field__input mono', id: 'f-finui_preset', type: 'text', autocomplete: 'off', spellcheck: false,
      value: current, placeholder: 'None: FinUI as it ships', 'aria-describedby': 'finui_preset-help' });
    const note = h('span', { class: 'saved-note', 'aria-live': 'polite' });
    const err = h('div');
    const meaning = h('div', { class: 'fui-setting-row__notes' });
    const save = button({ size: 'sm' }, 'Save');
    const reset = button({ size: 'sm', variant: 'ghost' }, 'Reset');

    function paint() {
      const code = current;
      reset.hidden = !code;
      const chosen = presets.axes.map((a, i) => { const o = parseInt(code[i] || '0', 36); return o ? [a.label, a.options[o] && a.options[o].label] : null; }).filter(Boolean);
      mount(meaning,
        chosen.length ? facts(chosen) : h('p', { class: 'fui-field__help' }, 'Every choice at its default: FinUI as it ships.'),
        h('p', { class: 'fui-field__help' }, button({ href: code ? `${CREATE}?preset=${code}` : CREATE, target: '_blank', rel: 'noopener noreferrer', size: 'sm' },
          icon('sliders', 14), code ? 'Change it in FinUI create' : 'Make one in FinUI create', icon('external', 14))));
    }
    async function store_(code) {
      mount(err, '');
      setBusy(save, true, 'Saving…');
      try {
        current = (await api.put('/me/appearance', { finui_preset: code })).finui_preset;
        input.value = current;
        input.removeAttribute('aria-invalid');
        restyle();
        paint();
        note.replaceChildren(icon('check', 13), 'Saved');
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
    const [menuRow] = settingRow({ id: 'mobile_menu', label: 'Menu on this phone', labelFor: 'f-mobile_menu',
      help: 'How the menu opens here. Kept by this browser alone: every phone and tablet keeps its own.', control: h('div', { class: 'fui-field__row' }, menuNote, menu) });
    menu.addEventListener('change', () => {
      setMobileNav(menu.value);
      menuNote.replaceChildren(icon('check', 13), 'Saved');
      setTimeout(() => menuNote.replaceChildren(), 2000);
    });
    // A wide screen's menu, and the side it sits on where it has one (not a dock, not a command bar).
    const saved = (note) => { note.replaceChildren(icon('check', 13), 'Saved'); setTimeout(() => note.replaceChildren(), 2000); };
    const deskMenu = h('select', { class: 'fui-field__input', id: 'f-desktop_menu', 'aria-describedby': 'desktop_menu-help' }, DESKTOP_STYLES.map((st) => h('option', { value: st.key }, st.label)));
    deskMenu.value = desktopStyleOf(desktopNavChoice().style);
    const deskNote = h('span', { class: 'saved-note', 'aria-live': 'polite' });
    const sideNote = h('span', { class: 'saved-note', 'aria-live': 'polite' });
    const sideSwitch = toggle({ checked: desktopNavChoice().side === 'right', labelledby: 'desktop_side-label', describedby: 'desktop_side-help',
      onChange: (next) => { setDesktopNav({ side: next ? 'right' : 'left' }); saved(sideNote); } });
    const [deskRow] = settingRow({ id: 'desktop_menu', label: 'Menu', labelFor: 'f-desktop_menu',
      help: 'How the pages are laid out on this computer. Kept by this browser alone: your other devices keep their own.', control: h('div', { class: 'fui-field__row' }, deskNote, deskMenu) });
    const [sideRow] = settingRow({ id: 'desktop_side', label: 'Menu on the right', help: 'The sidebar on the right-hand edge of the window instead of the left.', control: [sideNote, sideSwitch] });
    deskMenu.addEventListener('change', () => { setDesktopNav({ style: deskMenu.value }); saved(deskNote); fit(); });

    // Where search opens, on every width: grown into the menu, or in the page's place.
    const modeSel = h('select', { class: 'fui-field__input', id: 'f-search_mode', 'aria-describedby': 'search_mode-help' },
      h('option', { value: 'menu' }, 'In the menu'), h('option', { value: 'page' }, 'In the page'));
    modeSel.value = searchMode();
    const modeNote = h('span', { class: 'saved-note', 'aria-live': 'polite' });
    modeSel.addEventListener('change', () => { setSearchMode(modeSel.value); saved(modeNote); });
    const [modeRow] = settingRow({ id: 'search_mode', label: 'Search opens', labelFor: 'f-search_mode',
      help: 'Inside your menu, which turns into the search, or in place of the page beside it. Kept by this browser alone.', control: h('div', { class: 'fui-field__row' }, modeNote, modeSel) });

    const phone = matchMedia(PHONE);
    const fit = () => {
      menuRow.hidden = !phone.matches;
      deskRow.hidden = phone.matches;
      sideRow.hidden = phone.matches || !sideOf(deskMenu.value, 'left');
    };
    phone.addEventListener('change', fit, { signal: store.signal });
    fit();

    mount(body, h('div', { class: 'fui-setting-row__rows' },
      menuRow, deskRow, sideRow, modeRow,
      settingRow({ id: 'finui_preset', label: 'FinUI preset', labelFor: 'f-finui_preset',
        help: 'A code made at FinUI create: colours, corners, density and more. Every page wears it for you; others choose their own.',
        control: h('div', { class: 'fui-field__row' }, note, input, save, reset), error: err })),
      meaning);
    paint();
  },
};
