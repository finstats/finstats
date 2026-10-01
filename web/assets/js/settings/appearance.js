// Settings → Appearance: how finstats looks to you, and to nobody else. A FinUI preset — the code FinUI create makes
// (finstats.github.io/finui/create) — whose tokens the server serves after FinUI's own in /assets/finui.css, to you
// alone. Saving swaps that stylesheet on this page at once; your other pages get it on their next load.

import { h, icon, mount } from '../dom.js';
import { api } from '../api.js';
import { card, setBusy, inlineError, facts } from '../components.js';
import { settingRow } from './common.js';
import { button } from '../../finui/components/button/button.js';

const CREATE = 'https://finstats.github.io/finui/create/';
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
  entries: [{ id: 'finui_preset', label: 'FinUI preset', hint: 'appearance look theme colours accent radius density borders cards motion icons finui preset code customize create' }],
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

    mount(body, h('div', { class: 'fui-setting-row__rows' },
      settingRow({ id: 'finui_preset', label: 'FinUI preset', labelFor: 'f-finui_preset',
        help: 'A code made at FinUI create: colours, corners, density and more. Every page wears it for you; others choose their own.',
        control: h('div', { class: 'fui-field__row' }, note, input, save, reset), error: err })),
      meaning);
    paint();
  },
};
