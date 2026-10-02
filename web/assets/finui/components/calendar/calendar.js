// FinUI: calendar. A month to pick in: one day, several, or a range from one day to another. The buttons and Page Up /
// Page Down change the month, the arrows move a day or a week, Home and End go to the week's ends, Enter or Space picks.
// Marked days carry a dot and say what is on them. Days outside min and max cannot be picked. The dates are dates.js.

import { h, icon } from '../../core.js';
import { weeks, addMonths, monthOf, move, pick, picked, inRange, allowed, today } from './dates.js';

const date = (key) => new Date(`${key}T12:00:00Z`);

/**
 * calendar({ mode, value, onChange, month, marks, min, max, limit, weekStart, locale, label })
 * mode: 'single' (value a key, 'YYYY-MM-DD'), 'multiple' (an array of keys, at most `limit`) or 'range' ({ from, to }).
 * marks: { key: 'what is on it' }. el.setValue(v) changes it quietly; el.value is what is picked.
 */
export function calendar({ mode = 'single', value = null, onChange = () => {}, month = null, marks = {}, min = null, max = null, limit = Infinity,
  weekStart = 1, locale = undefined, label = 'Calendar' } = {}) {
  let current = mode === 'multiple' ? [...(value || [])] : mode === 'range' ? { from: null, to: null, ...(value || {}) } : value;
  const first = mode === 'multiple' ? current[0] : mode === 'range' ? current.from : current;
  let shown = monthOf(month || first || today());
  let focus = first || shown;
  const now = today();
  const caption = new Intl.DateTimeFormat(locale, { month: 'long', year: 'numeric', timeZone: 'UTC' });
  const long = new Intl.DateTimeFormat(locale, { weekday: 'long', day: 'numeric', month: 'long', year: 'numeric', timeZone: 'UTC' });
  const short = new Intl.DateTimeFormat(locale, { weekday: 'short', timeZone: 'UTC' });
  const longDay = new Intl.DateTimeFormat(locale, { weekday: 'long', timeZone: 'UTC' });

  const title = h('span', { class: 'fui-calendar__caption', 'aria-live': 'polite' });
  const prev = h('button', { type: 'button', class: 'fui-calendar__nav', 'aria-label': 'Previous month', onClick: () => turn(-1) }, icon('chevronLeft', 16));
  const next = h('button', { type: 'button', class: 'fui-calendar__nav', 'aria-label': 'Next month', onClick: () => turn(1) }, icon('chevronRight', 16));
  const grid = h('table', { class: 'fui-calendar__grid', role: 'grid' });
  const el = h('div', { class: ['fui-calendar', `fui-calendar--${mode}`], role: 'group', 'aria-label': label },
    h('div', { class: 'fui-calendar__head' }, prev, title, next), grid);

  function turn(by, focusTo = null) {
    shown = addMonths(shown, by);
    focus = focusTo || (focus.slice(0, 7) === shown.slice(0, 7) ? focus : addMonths(focus, by));
    if (monthOf(focus) !== shown) focus = shown;
    paint();
  }
  function choose(key) {
    if (!allowed(key, { min, max })) return;
    current = pick(mode, current, key, { min, max, limit });
    focus = key;
    if (monthOf(key) !== shown) shown = monthOf(key);
    paint(true);
    onChange(mode === 'multiple' ? [...current] : mode === 'range' ? { ...current } : current);
  }
  function paint(keepFocus = false) {
    const hadFocus = keepFocus || el.contains(document.activeElement) && document.activeElement.classList.contains('fui-calendar__day');
    title.textContent = caption.format(date(shown));
    grid.setAttribute('aria-label', title.textContent);
    prev.disabled = !!min && addMonths(shown, -1).slice(0, 7) < min.slice(0, 7);
    next.disabled = !!max && addMonths(shown, 1).slice(0, 7) > max.slice(0, 7);
    const rows = weeks(shown, weekStart);
    const head = h('thead', null, h('tr', null, rows[0].map((d) => h('th', { scope: 'col', abbr: longDay.format(date(d.key)) }, short.format(date(d.key)).slice(0, 2)))));
    const body = h('tbody', null, rows.map((week) => h('tr', null, week.map((d) => {
      const on = picked(mode, current, d.key);
      const between = mode === 'range' && inRange(d.key, current) && !on;
      const mark = marks[d.key];
      const b = h('button', { type: 'button', class: 'fui-calendar__day', dataset: { day: d.key }, tabindex: d.key === focus ? 0 : -1,
        disabled: !allowed(d.key, { min, max }), 'aria-current': d.key === now ? 'date' : null,
        'aria-label': `${long.format(date(d.key))}${mark ? `, ${mark}` : ''}` },
        String(Number(d.key.slice(8))), mark ? h('span', { class: 'fui-calendar__mark', 'aria-hidden': 'true' }) : null);
      return h('td', { role: 'gridcell', 'aria-selected': String(on),
        class: [!d.inMonth && 'is-outside', between && 'is-in-range', mode === 'range' && current.from === d.key && current.to && 'is-range-start', mode === 'range' && current.to === d.key && 'is-range-end'] }, b);
    }))));
    grid.replaceChildren(head, body);
    if (hadFocus) grid.querySelector(`[data-day="${focus}"]`)?.focus();
  }
  grid.addEventListener('click', (e) => { const b = e.target.closest('.fui-calendar__day'); if (b && !b.disabled) choose(b.dataset.day); });
  grid.addEventListener('keydown', (e) => {
    const b = e.target.closest('.fui-calendar__day');
    if (!b) return;
    if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); choose(b.dataset.day); return; }
    const to = move(b.dataset.day, e.key, weekStart, e.shiftKey);
    if (!to) return;
    e.preventDefault();
    if (!allowed(to, { min, max })) return;
    focus = to;
    if (monthOf(to) !== shown) shown = monthOf(to);
    paint(true);
  });
  paint();
  el.setValue = (v) => { current = mode === 'multiple' ? [...(v || [])] : mode === 'range' ? { from: null, to: null, ...(v || {}) } : v; paint(); };
  Object.defineProperty(el, 'value', { get: () => (mode === 'multiple' ? [...current] : mode === 'range' ? { ...current } : current) });
  return el;
}

const RELEASES = { '2026-10-02': 'Sintel, season 2, episode 3', '2026-10-09': 'Sintel, season 2, episode 4', '2026-10-14': 'Tears of Steel', '2026-10-22': 'Cosmos Laundromat', '2026-10-30': 'Spring' };

export const meta = {
  name: 'calendar',
  purpose: 'A month to pick in: one day, several days, or a range from one day to another.',
  use: 'Choosing when: a day to look at, the days of a plan, the week a report covers. Marks show what is on a day.',
  avoid: 'A date somebody types better than they find (a birthday years back: a field). A time of day (time slots).',
  variants: ['one day', 'several days', 'a range', 'within limits', 'week starting Sunday'],
  states: ['today', 'picked', 'in a range', 'marked', 'outside the month', 'not allowed', 'focus-visible'],
  a11y: 'A grid of day buttons, one in the tab order; arrows move a day or a week, Home/End to the week\'s ends, Page Up/Down a month (with Shift a year), Enter or Space picks. Each day is named in full, with its mark; the month is announced as it changes.',
  props: {
    'calendar({ mode, value, onChange, month, marks, min, max, limit, weekStart, locale, label })': "mode: 'single' | 'multiple' | 'range'; value: a key ('YYYY-MM-DD'), an array of keys, or { from, to }; marks: { key: text }; limit: most days in multiple; weekStart: 1 Monday, 0 Sunday.",
    'el.setValue(v), el.value': 'change it quietly; what is picked',
  },
  playground: {
    controls: [
      { key: 'several', label: 'Several days', excludes: ['range'] },
      { key: 'range', label: 'A range', excludes: ['several'] },
      { key: 'marks', label: 'Marked days', on: true },
      { key: 'limits', label: 'Within limits' },
      { key: 'sunday', label: 'Week starts on Sunday' },
    ],
    render: (o) => calendar({
      mode: o.range ? 'range' : o.several ? 'multiple' : 'single', month: '2026-10-01', limit: 5,
      marks: o.marks ? RELEASES : {}, min: o.limits ? '2026-10-02' : null, max: o.limits ? '2026-11-20' : null, weekStart: o.sunday ? 0 : 1, locale: 'en-GB',
      value: o.range || o.several ? null : '2026-10-09',
    }),
  },
};
