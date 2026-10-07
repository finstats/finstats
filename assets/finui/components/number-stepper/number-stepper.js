// FinUI: number-stepper. An exact number with a step down and a step up beside it: typed, arrowed or pressed, and
// always kept inside its ends; a button at an end says so by being unavailable.

import { h, icon } from '../../core.js';
import { clamp, parseNumber, stepBy } from './plan.js';

/** numberStepper({ label, value, min, max, step, unit, onChange }): `unit` is said after the number ("GB"). */
export function numberStepper({ label, value = 0, min = -Infinity, max = Infinity, step = 1, unit = null, onChange = () => {} }) {
  const o = { min, max, step };
  let current = clamp(value, o);
  const input = h('input', { class: 'fui-number-stepper__input', type: 'text', inputMode: step % 1 ? 'decimal' : 'numeric', 'aria-label': unit ? `${label}, in ${unit}` : label, autocomplete: 'off' });
  const less = h('button', { type: 'button', class: 'fui-number-stepper__button', 'aria-label': `Less: ${label}` }, icon('minus', 14));
  const more = h('button', { type: 'button', class: 'fui-number-stepper__button', 'aria-label': `More: ${label}` }, icon('plus', 14));
  const show = () => { input.value = String(current); less.disabled = current <= min; more.disabled = current >= max; };
  const set = (v) => { const next = clamp(v, o); const changed = next !== current; current = next; show(); if (changed) onChange(current); };
  less.addEventListener('click', () => set(stepBy(current, -1, o)));
  more.addEventListener('click', () => set(stepBy(current, 1, o)));
  input.addEventListener('keydown', (e) => {
    if (e.key === 'ArrowUp' || e.key === 'ArrowDown') { e.preventDefault(); set(stepBy(current, e.key === 'ArrowUp' ? 1 : -1, o)); }
    if (e.key === 'Enter') { e.preventDefault(); input.blur(); }
  });
  // What was typed is kept when it is a number, put inside the ends; anything else puts the last good value back.
  input.addEventListener('change', () => { const v = parseNumber(input.value); if (v === null) show(); else set(v); });
  show();
  return h('div', { class: 'fui-number-stepper' }, less, input, unit ? h('span', { class: 'fui-number-stepper__unit' }, unit) : null, more);
}

export const meta = {
  name: 'number-stepper',
  purpose: 'Takes an exact number, typed or stepped down and up, kept inside its ends.',
  use: 'A count or a size where the digits matter: copies kept, minutes, gigabytes. unit says what it counts.',
  avoid: 'A position in a range, where roughly is enough (slider). A number nobody steps through (a plain field).',
  variants: ['plain', 'with a unit', 'in halves (decimal steps)', 'at an end'],
  states: ['a button unavailable at an end', 'focus-visible'],
  a11y: 'A text input with inputmode numeric or decimal, named by its label (with the unit); arrows step it; the two buttons are named Less and More. A typed value that is not a number is put back.',
  props: { 'numberStepper({ label, value, min, max, step, unit, onChange })': '' },
  playground: {
    controls: [
      { key: 'unit', label: 'A unit', on: true },
      { key: 'half', label: 'Steps of a half' },
      { key: 'end', label: 'At its end' },
    ],
    render: (o) => h('div', { class: 'fui-number-stepper__demo' }, h('span', null, 'Backups kept'),
      numberStepper({ label: 'Backups kept', value: o.end ? 10 : o.half ? 2.5 : 5, min: 1, max: 10, step: o.half ? 0.5 : 1, unit: o.unit ? 'copies' : null })),
  },
};
