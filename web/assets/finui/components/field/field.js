// FinUI: field. A labelled input with its help and its error, the input itself, a search box with its icon, and a
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
  playground: {
    controls: [
      { key: 'kind', label: 'Kind', choices: [['text', 'Text'], ['search', 'Search'], ['number', 'A number'], ['check', 'A checkbox']] },
      { key: 'help', label: 'Help', on: true },
      { key: 'error', label: 'An error' },
    ],
    render: (o) => {
      if (o.kind === 'search') return h('div', { class: 'fui-field__search' }, icon('search', 14), h('input', { class: 'fui-field__input fui-field__input--search', type: 'search', placeholder: o.help ? 'Find a title, a person…' : 'Find…', 'aria-label': 'Find a title', 'aria-invalid': o.error ? 'true' : null }));
      if (o.kind === 'number') return h('div', { class: 'fui-field' }, h('label', { class: 'fui-field__label', htmlFor: 'demo-min' }, 'Minutes'), h('input', { class: 'fui-field__input fui-field__input--num', id: 'demo-min', type: 'number', value: 30, 'aria-invalid': o.error ? 'true' : null }),
        o.help ? h('p', { class: 'fui-field__help' }, 'Shorter plays are left out.') : null, o.error ? inlineError('demo-min-err', 'At most 600.') : null);
      if (o.kind === 'check') return h('div', { class: 'fui-field' }, h('label', { class: 'fui-field__check' }, h('input', { type: 'checkbox', checked: true }), 'Keep my settings'),
        o.help ? h('p', { class: 'fui-field__help' }, 'They stay in this browser.') : null, o.error ? inlineError('demo-check-err', 'This browser keeps nothing.') : null);
      const f = formField({ id: 'demo-url', label: 'Address', placeholder: 'http://192.168.1.10:8096', help: o.help ? 'Where Jellyfin answers.' : null });
      if (o.error) { f.input.value = 'jellyfin.local'; f.setError('That is not an address: it needs http:// or https://'); }
      return f.el;
    },
  },
};
