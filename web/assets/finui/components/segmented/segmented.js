// FinUI: segmented. A few mutually exclusive choices side by side — a measure, a range, a sort — of which exactly one
// is on. The arrow keys move between them, as in a radio group.

import { h } from '../../core.js';

export function segmented({ options, value, onChange, label, size = '' }) {
  let currentValue = value;
  const small = size === 'sm' || size === 'seg-sm';   // 'seg-sm' is how callers said it before FinUI
  const group = h('div', { class: ['fui-segmented', small && 'fui-segmented--sm'], role: 'radiogroup', 'aria-label': label });
  const btns = options.map((o) => h('button', { type: 'button', class: 'fui-segmented__option', role: 'radio', title: o.title || null,
    onClick: () => select(o.value, true) }, o.label));
  function paint() {
    btns.forEach((b, i) => {
      const on = options[i].value === currentValue;
      b.setAttribute('aria-checked', String(on));
      b.tabIndex = on ? 0 : -1;
    });
  }
  function select(v, fire) {
    if (v === currentValue) return;
    currentValue = v; paint();
    if (fire) onChange(v);
  }
  group.addEventListener('keydown', (e) => {
    const dir = { ArrowRight: 1, ArrowDown: 1, ArrowLeft: -1, ArrowUp: -1 }[e.key];
    if (!dir) return;
    e.preventDefault();
    const i = options.findIndex((o) => o.value === currentValue);
    const n = (i + dir + options.length) % options.length;
    select(options[n].value, true);
    btns[n].focus();
  });
  group.append(...btns);
  paint();
  group.setValue = (v) => select(v, false);
  return group;
}

export const meta = {
  name: 'segmented',
  purpose: 'Picks one of a few choices that exclude each other, with all of them in view.',
  use: 'Two to six short choices: a measure (Watch time · Plays), a range, a sort, a view. size: \'sm\' beside a card title.',
  avoid: 'Choices that can be on together (chips that toggle). More than six, or long labels (a select or a combobox). Navigation between pages (links).',
  variants: ['default', 'small'],
  states: ['checked', 'hover', 'focus-visible'],
  a11y: 'role="radiogroup" with role="radio" buttons and aria-checked; only the checked one is in the tab order; arrow keys move and choose.',
  props: { 'segmented({ options, value, onChange, label, size })': "options: [{ value, label, title }]; label names the group; size: 'sm'. el.setValue(v) changes it quietly." },
  playground: {
    controls: [
      { key: 'choices', label: 'Choices', choices: [[2, 'Two'], [4, 'Four']] },
      { key: 'small', label: 'Small' },
    ],
    render: (o) => segmented({ label: 'Sort by', size: o.small ? 'sm' : '', value: 'name',
      options: [{ value: 'name', label: 'Name' }, { value: 'year', label: 'Year' }, { value: 'added', label: 'Added' }, { value: 'size', label: 'Size' }].slice(0, o.choices), onChange: () => {} }),
  },
};
