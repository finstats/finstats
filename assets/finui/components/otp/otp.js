// FinUI: otp. A one-time code in a row of boxes, a character each: typing moves on, Backspace moves back, and a pasted
// code fills the boxes from the one it went into. When the last box fills, the code is handed over.

import { h } from '../../core.js';
import { spread } from './plan.js';

/** otp({ label, length, onComplete(code), letters, grouped }): `letters` takes letters too; `grouped` splits the row in two. */
export function otp({ label = 'Code', length = 6, onComplete = () => {}, letters = false, grouped = false } = {}) {
  const allowed = letters ? /[0-9a-z]/i : /[0-9]/;
  let boxes = Array.from({ length }, () => '');
  const inputs = boxes.map((_, i) => h('input', { class: 'fui-otp__box', type: 'text', inputMode: letters ? 'text' : 'numeric', maxLength: 1, autocomplete: i ? 'off' : 'one-time-code',
    'aria-label': `${label}, ${i + 1} of ${length}`, spellcheck: false }));
  const el = h('div', { class: 'fui-otp', role: 'group', 'aria-label': label },
    inputs.map((inp, i) => (grouped && i === Math.floor(length / 2) ? [h('span', { class: 'fui-otp__gap', 'aria-hidden': 'true' }, '–'), inp] : inp)));
  const show = () => { inputs.forEach((inp, i) => { inp.value = boxes[i]; }); el.classList.toggle('is-complete', boxes.every(Boolean)); };
  const put = (i, text) => {
    boxes = spread(boxes, i, text, length, allowed);
    show();
    const next = boxes.findIndex((b, k) => k >= i && !b);
    (inputs[next >= 0 ? next : length - 1]).focus();
    if (boxes.every(Boolean)) onComplete(boxes.join(''));
  };
  inputs.forEach((inp, i) => {
    inp.addEventListener('input', () => { const t = inp.value; boxes[i] = ''; put(i, t); });
    inp.addEventListener('paste', (e) => { e.preventDefault(); put(i, (e.clipboardData && e.clipboardData.getData('text')) || ''); });
    inp.addEventListener('keydown', (e) => {
      if (e.key === 'Backspace' && !inp.value && i) { e.preventDefault(); boxes[i - 1] = ''; show(); inputs[i - 1].focus(); }
      else if (e.key === 'ArrowLeft' && i) { e.preventDefault(); inputs[i - 1].focus(); }
      else if (e.key === 'ArrowRight' && i < length - 1) { e.preventDefault(); inputs[i + 1].focus(); }
    });
    inp.addEventListener('focus', () => inp.select());
  });
  show();
  return el;
}

export const meta = {
  name: 'otp',
  purpose: 'Takes a one-time code in a row of boxes, a character each, and hands it over when the last fills.',
  use: 'A second step of signing in, a code from an e-mail, a pairing code shown on a television. Pasting the whole code works from any box.',
  avoid: 'A password or anything longer than a dozen characters (a field). A code somebody must not see (a password field).',
  variants: ['four or six', 'digits, or letters too', 'grouped in two halves'],
  states: ['typing', 'complete'],
  a11y: 'A group named by its label; each box is an input named "Code, 2 of 6"; the first says autocomplete="one-time-code", so a phone offers the code it was sent.',
  props: { 'otp({ label, length, onComplete, letters, grouped })': 'onComplete(code)' },
  playground: {
    controls: [
      { key: 'length', label: 'Length', choices: [['6', 'Six'], ['4', 'Four']] },
      { key: 'grouped', label: 'In two halves' },
      { key: 'letters', label: 'Letters too' },
    ],
    render: (o) => otp({ label: o.letters ? 'Pairing code' : 'Code', length: Number(o.length), grouped: o.grouped, letters: o.letters }),
  },
};
