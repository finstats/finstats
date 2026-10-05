// FinUI: choice. Checkboxes and radio buttons with their words: one thing on or off, several of a list, or one of a
// list. The native inputs do the work — keys, forms, screen readers — and FinUI draws the box beside each.

import { h, icon } from '../../core.js';

let seq = 0;
function item({ kind, label, help, checked, mixed, disabled, name, value, onInput }) {
  const id = `fui-choice-${++seq}`;
  const input = h('input', { class: 'fui-choice__input', type: kind, id, name, value, checked: !!checked, disabled: !!disabled, 'aria-describedby': help ? `${id}-help` : null });
  if (mixed) input.indeterminate = true;
  const el = h('label', { class: ['fui-choice', `fui-choice--${kind}`, checked && 'is-checked', disabled && 'is-disabled'] }, input,
    h('span', { class: 'fui-choice__box', 'aria-hidden': 'true' }, kind === 'checkbox' ? [icon('check', 12, 'fui-choice__tick'), icon('minus', 12, 'fui-choice__dash')] : null),
    h('span', { class: 'fui-choice__text' }, h('span', { class: 'fui-choice__label' }, label), help ? h('span', { class: 'fui-choice__help', id: `${id}-help` }, help) : null));
  // The label says what the input holds (is-checked), so a radio that loses its tick to another says so too.
  input.addEventListener('change', () => {
    for (const other of input.name ? document.querySelectorAll(`input[name="${input.name}"]`) : [input]) other.closest('.fui-choice')?.classList.toggle('is-checked', other.checked);
    el.classList.toggle('is-checked', input.checked);
    onInput(input);
  });
  return el;
}

/** checkbox({ label, help, checked, mixed, disabled, onChange(checked) }): one thing on or off. `mixed`: some of what it
 *  stands for are on (a "select all"). */
export function checkbox({ label, help = null, checked = false, mixed = false, disabled = false, onChange = () => {} } = {}) {
  return item({ kind: 'checkbox', label, help, checked, mixed, disabled, onInput: (i) => onChange(i.checked) });
}

/** choiceGroup({ label, kind, options: [{ value, label, help, disabled }], value, onChange, inline, name }): several of a list
 *  (kind 'checkbox', value an array) or one of it (kind 'radio', value a string), under the words that name them. */
export function choiceGroup({ label, kind = 'radio', options, value = null, onChange = () => {}, inline = false, name = null }) {
  // Named after its label, so the same group drawn again is the same markup; give two groups of one label a name each.
  name = name || `fui-choice-${String(label).toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '')}`;
  let current = kind === 'radio' ? value : [...(value || [])];
  const items = options.map((o) => item({ kind, label: o.label, help: o.help, disabled: o.disabled, name, value: o.value,
    checked: kind === 'radio' ? current === o.value : current.includes(o.value),
    onInput: (i) => {
      if (kind === 'radio') current = o.value;
      else current = i.checked ? [...current, o.value] : current.filter((v) => v !== o.value);
      onChange(current);
    } }));
  return h('fieldset', { class: 'fui-choice__group' }, h('legend', { class: 'fui-choice__legend' }, label),
    h('div', { class: ['fui-choice__options', inline && 'fui-choice__options--inline'] }, items));
}

const KINDS = [{ value: 'films', label: 'Films', help: 'Movies in any library' }, { value: 'shows', label: 'Shows', help: 'Every episode' }, { value: 'music', label: 'Music', help: 'Albums and tracks', disabled: true }];
export const meta = {
  name: 'choice',
  purpose: 'Asks for one thing on or off, several of a list, or one of a list, each with its words beside it.',
  use: 'checkbox for one thing that is on or off and is saved with a form (a switch, toggle, acts at once). choiceGroup for several of a short list, or one of it, every option in sight.',
  avoid: 'One of many (combobox or picker). One of two or three that change a view at once (segmented). A radio group of one.',
  variants: ['a checkbox', 'with a line of help', 'mixed (some of it on)', 'several of a list', 'one of a list', 'side by side', 'one unavailable'],
  states: ['checked', 'mixed', 'disabled', 'focus-visible (a ring round the box)'],
  a11y: 'Native checkboxes and radio buttons inside their <label>: arrows move through a radio group, Space ticks, and the help is aria-describedby. A group is a <fieldset> named by its <legend>.',
  props: {
    'checkbox({ label, help, checked, mixed, disabled, onChange })': 'onChange(checked)',
    'choiceGroup({ label, kind, options, value, onChange, inline, name })': "kind: 'radio' | 'checkbox'; options: [{ value, label, help, disabled }]; name: when two groups share a label",
  },
  playground: {
    controls: [
      { key: 'kind', label: 'Kind', choices: [['several', 'Several'], ['one', 'On or off'], ['radio', 'One of a list']] },
      { key: 'help', label: 'A line of help', on: true },
      { key: 'inline', label: 'Side by side' },
      { key: 'mixed', label: 'Mixed (some of it)' },
    ],
    render: (o) => {
      const opts = KINDS.map((k) => ({ ...k, help: o.help ? k.help : null }));
      if (o.kind === 'one') return checkbox({ label: 'Count plays shorter than two minutes', help: o.help ? 'They are left out of every total otherwise.' : null, checked: !o.mixed, mixed: o.mixed });
      return choiceGroup({ label: 'What to count', kind: o.kind === 'radio' ? 'radio' : 'checkbox', inline: o.inline, options: opts,
        value: o.kind === 'radio' ? 'films' : o.mixed ? ['films'] : ['films', 'shows'] });
    },
  },
};
