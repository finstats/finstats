// FinUI: setting-row. One setting on one line: what it is on the left (a label and at most a line of help), the
// control on the right. On a phone the two stack.

import { h } from '../../core.js';

/** The two-sided row every setting sits in. Returns [row, error]: the error goes under the row. */
export function settingRow({ id, label, help, control, labelFor, error }) {
  const labelEl = labelFor
    ? h('label', { class: 'fui-setting-row__label', id: `${id}-label`, htmlFor: labelFor }, label)
    : h('div', { class: 'fui-setting-row__label', id: `${id}-label` }, label);
  return [h('div', { class: 'fui-setting-row', id },
    h('div', null, labelEl, help ? h('p', { class: 'fui-field__help', id: `${id}-help` }, help) : null),
    h('div', { class: 'fui-setting-row__control' }, control)), error || null];
}

export const meta = {
  name: 'setting-row',
  purpose: 'Lays out one setting: its name and a line of help on the left, its control on the right.',
  use: 'Every setting. A switch saves itself (toggle); a number or a text waits for its section’s one Save (fui-setting-row__rows in a form, fui-setting-row__actions at the bottom right).',
  avoid: 'Help longer than a line (about 150 characters): what does not fit is a link to the docs. Two controls in one row.',
  variants: ['with a switch', 'with an input', 'without help'],
  states: ['is-hit (a link landed on it)'],
  a11y: 'The label names the control (label for, or aria-labelledby on a switch); the help describes it (aria-describedby).',
  props: { 'settingRow({ id, label, help, control, labelFor, error })': '→ [row, error]' },
  playground: {
    controls: [
      { key: 'control', label: 'Control', choices: [['number', 'A number'], ['switch', 'A switch'], ['text', 'Words']] },
      { key: 'help', label: 'Help', on: true },
      { key: 'error', label: 'An error' },
    ],
    render: (o) => {
      const control = o.control === 'switch' ? h('button', { type: 'button', class: 'fui-toggle', role: 'switch', 'aria-checked': 'true', 'aria-labelledby': 'demo-min-label' }, h('span', { class: 'fui-toggle__knob' }))
        : h('input', { class: ['fui-field__input', o.control === 'number' && 'fui-field__input--num'], id: 'demo-min-in', type: o.control === 'number' ? 'number' : 'text', value: o.control === 'number' ? 120 : 'Living room' });
      return h('div', { class: 'fui-setting-row__rows' }, settingRow({ id: 'demo-min', label: o.control === 'text' ? 'Server name' : 'Shortest play counted',
        help: o.help ? (o.control === 'text' ? 'What this server is called here.' : 'Plays shorter than this are left out of every statistic.') : null,
        labelFor: o.control === 'switch' ? null : 'demo-min-in', control, error: o.error ? h('p', { class: 'fui-field__error', role: 'alert' }, 'That cannot be saved: it is longer than ten minutes.') : null }));
    },
  },
};
