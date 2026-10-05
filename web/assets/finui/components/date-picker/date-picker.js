// FinUI: date-picker. A day typed or picked: a field that reads the ways people write a day, and beside it a button
// that opens the calendar under it. The day chosen is said in full under the field.

import { h, icon, mount } from '../../core.js';
import { calendar } from '../calendar/calendar.js';
import { inlineError } from '../field/field.js';
import { parseDay } from './plan.js';

const full = new Intl.DateTimeFormat('en-GB', { weekday: 'long', day: 'numeric', month: 'long', year: 'numeric', timeZone: 'UTC' });
const short = new Intl.DateTimeFormat('en-GB', { day: 'numeric', month: 'short', year: 'numeric', timeZone: 'UTC' });
const at = (key) => new Date(`${key}T12:00:00Z`);

/** datePicker({ label, value, onChange(day), min, max, open, id }): days as YYYY-MM-DD; `open` starts with the calendar shown. */
export function datePicker({ label, value = null, onChange = () => {}, min = null, max = null, open = false, id = null }) {
  // Named after its label unless given an id, as a field is; two pickers of one label on a page take an id each.
  id = id || `fui-date-${String(label).toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '')}`;
  let current = value;
  const input = h('input', { class: 'fui-field__input fui-date-picker__input', id, type: 'text', placeholder: 'e.g. 5 Oct 2026', autocomplete: 'off', 'aria-describedby': `${id}-said` });
  const said = h('p', { class: 'fui-field__help', id: `${id}-said`, 'aria-live': 'polite' });
  const error = h('div');
  const button = h('button', { type: 'button', class: 'fui-date-picker__button', 'aria-label': `Choose ${label.toLowerCase()} on a calendar`, 'aria-haspopup': 'dialog', 'aria-expanded': 'false', 'aria-controls': `${id}-pop` }, icon('calendar', 15));
  const pop = h('div', { class: 'fui-date-picker__pop', id: `${id}-pop`, role: 'dialog', 'aria-label': label, hidden: true });
  const el = h('div', { class: 'fui-field fui-date-picker' }, h('label', { class: 'fui-field__label', htmlFor: id }, label),
    h('div', { class: 'fui-date-picker__row' }, input, button, pop), said, error);
  const show = () => {
    input.value = current ? short.format(at(current)) : '';
    said.textContent = [current ? full.format(at(current)) : 'No day chosen yet.', min && max ? `Between ${short.format(at(min))} and ${short.format(at(max))}.` : min ? `From ${short.format(at(min))}.` : max ? `Until ${short.format(at(max))}.` : null].filter(Boolean).join(' ');
    input.removeAttribute('aria-invalid'); mount(error);
  };
  const set = (day) => { current = day; show(); onChange(day); };
  const outside = (e) => { if (!el.contains(e.target)) toggle(false); };
  function toggle(on) {
    pop.hidden = !on;
    button.setAttribute('aria-expanded', String(on));
    if (on) {
      mount(pop, calendar({ mode: 'single', value: current, min, max, label, onChange: (day) => { set(day); toggle(false); button.focus(); } }));
      document.addEventListener('pointerdown', outside, true);
    } else document.removeEventListener('pointerdown', outside, true);
  }
  button.addEventListener('click', () => toggle(pop.hidden));
  pop.addEventListener('keydown', (e) => { if (e.key === 'Escape') { e.preventDefault(); e.stopPropagation(); toggle(false); button.focus(); } });
  input.addEventListener('change', () => {
    const day = parseDay(input.value);
    if (!input.value.trim()) { set(null); return; }
    if (!day || (min && day < min) || (max && day > max)) {
      input.setAttribute('aria-invalid', 'true');
      mount(error, inlineError(`${id}-err`, day ? 'That day is outside what can be chosen.' : 'That is not a day. Try 5 Oct 2026 or 2026-10-05.'));
      return;
    }
    set(day);
  });
  show();
  if (open) toggle(true);
  return el;
}

export const meta = {
  name: 'date-picker',
  purpose: 'Takes a day, typed in the ways people write one or picked on a calendar that opens under the field.',
  use: 'A day in a form: when a backup should start, a release date, the first day of a range to keep. min and max bound it; the day chosen is said in full under the field.',
  avoid: 'Several days or a range (calendar, in its multiple or range mode, on the page). A time of day alone.',
  variants: ['empty', 'a day chosen', 'with limits', 'the calendar open'],
  states: ['open', 'a day typed that is not one (an error in words)'],
  a11y: 'A labelled text input; the button that opens the calendar says what it does and whether it is open (aria-expanded); the calendar is a dialog Esc closes; the day chosen is announced in full.',
  props: { 'datePicker({ label, value, onChange, min, max, open, id })': 'days as YYYY-MM-DD; id when two share a label' },
  playground: {
    controls: [
      { key: 'value', label: 'A day chosen', on: true },
      { key: 'limits', label: 'Only this month' },
      { key: 'open', label: 'The calendar open' },
    ],
    render: (o) => h('div', { class: 'fui-date-picker__demo' }, datePicker({ label: 'First backup', value: o.value ? '2026-10-09' : null, min: o.limits ? '2026-10-01' : null, max: o.limits ? '2026-10-31' : null, open: o.open })),
  },
};
