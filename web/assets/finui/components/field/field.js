// finui: field. A labelled input with its help and its error, the input itself, a search box with its icon, and a
// checkbox with its words. The label is always there; the help says what to type; the error says what was wrong.

import { h, icon, mount } from '../../core.js';

export function inlineError(id, text) {
  return h('p', { class: 'fui-field__error', id, role: 'alert' }, icon('alert', 14), h('span', null, text));
}

/** A labelled input with help text and a place for its error. `setError('')` clears it. */
export function formField({ id, label, type = 'text', autocomplete, placeholder, inputMode, help }) {
  const input = h('input', { class: 'fui-field__input', id, name: id, type, autocomplete, placeholder, inputMode, autocapitalize: 'none', autocorrect: 'off', spellcheck: false,
    'aria-describedby': help ? id + '-help' : null });
  const err = h('div');
  const el = h('div', { class: 'fui-field' }, h('label', { htmlFor: id, class: 'fui-field__label' }, label), input, help ? h('p', { class: 'fui-field__help', id: id + '-help' }, help) : null, err);
  return {
    el, input,
    setError(msg) {
      input.setAttribute('aria-invalid', msg ? 'true' : 'false');
      input.setAttribute('aria-describedby', [msg ? id + '-err' : null, help ? id + '-help' : null].filter(Boolean).join(' '));
      mount(err, msg ? inlineError(id + '-err', msg) : '');
    },
  };
}

export const meta = {
  name: 'field',
  purpose: 'A labelled place to type, with help beneath it and an error that says what was wrong.',
  use: 'formField for a labelled text input in a form; fui-field__input on any input, select or textarea; fui-field__search with an icon for a filter; fui-field__check for a checkbox and its words; fui-field__help for one line of help.',
  avoid: 'A placeholder instead of a label. An error that only turns the border red: inlineError says it in words, and is announced.',
  variants: ['text', 'number (fui-field__input--num)', 'search', 'invalid', 'checkbox'],
  states: ['hover', 'focus (an accent ring)', 'aria-invalid'],
  a11y: 'The label is a <label for>; help and error are tied to the input with aria-describedby; the error is role="alert".',
  props: { 'formField({ id, label, type, autocomplete, placeholder, inputMode, help })': '→ { el, input, setError(msg) }', 'inlineError(id, text)': 'an error line, announced' },
  examples: [
    { name: 'A labelled input with help', render: () => formField({ id: 'demo-name', label: 'Name', placeholder: 'Living room laptop', help: 'So you can tell your keys apart.' }).el },
    { name: 'With an error', render: () => { const f = formField({ id: 'demo-url', label: 'Address', help: 'Where Jellyfin answers.' }); f.input.value = 'jellyfin.local'; f.setError('That is not an address: it needs http:// or https://'); return f.el; } },
    { name: 'Search, a number and a checkbox', render: () => h('div', { class: 'fui-field' },
      h('div', { class: 'fui-field__search' }, icon('search', 14), h('input', { class: 'fui-field__input fui-field__input--search', type: 'search', placeholder: 'Find a title…', 'aria-label': 'Find a title' })),
      h('input', { class: 'fui-field__input fui-field__input--num', type: 'number', value: 30, 'aria-label': 'Minutes' }),
      h('label', { class: 'fui-field__check' }, h('input', { type: 'checkbox', checked: true }), 'Keep my settings')) },
  ],
};
