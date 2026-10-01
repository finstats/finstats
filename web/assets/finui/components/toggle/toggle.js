// FinUI: toggle. A switch for a setting that takes effect at once: no Save button follows it.

import { h } from '../../core.js';

/** Immediate-effect setting → toggle switch. */
export function toggle({ checked, onChange, labelledby, describedby }) {
  const btn = h('button', { type: 'button', class: 'fui-toggle', role: 'switch', 'aria-checked': String(!!checked),
    'aria-labelledby': labelledby, 'aria-describedby': describedby }, h('span', { class: 'fui-toggle__knob' }));
  btn.addEventListener('click', () => {
    const next = btn.getAttribute('aria-checked') !== 'true';
    btn.setAttribute('aria-checked', String(next));
    onChange(next, (v) => btn.setAttribute('aria-checked', String(v)));
  });
  return btn;
}

export const meta = {
  name: 'toggle',
  purpose: 'Switches something on or off, at once.',
  use: 'A setting that applies the moment it changes (toggleRow in Settings). Labelled by the words beside it.',
  avoid: 'A choice that waits for Save (a checkbox in a form). Two choices that are not on and off (segmented).',
  variants: ['off', 'on', 'disabled', 'inherited (a default someone else set)'],
  states: ['on', 'off', 'disabled', 'focus-visible'],
  a11y: 'A <button role="switch" aria-checked>, named by aria-labelledby and described by aria-describedby. Space and Enter switch it.',
  props: { 'toggle({ checked, onChange, labelledby, describedby })': 'onChange(next, revert): call revert(previous) if saving failed' },
  examples: [
    { name: 'Off and on', render: () => h('div', { class: 'fui-toggle__demo' }, h('span', { id: 'demo-t1' }, 'Public profiles'), toggle({ checked: false, onChange: () => {}, labelledby: 'demo-t1' }), h('span', { id: 'demo-t2' }, 'Look up the public address'), toggle({ checked: true, onChange: () => {}, labelledby: 'demo-t2' })) },
  ],
};
