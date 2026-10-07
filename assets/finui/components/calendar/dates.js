// FinUI: calendar's dates. A day is its key, 'YYYY-MM-DD', and worked out in UTC: a day is a day, whatever the clock does
// that night. Pure, so a test reads it without a page.

const MS = 86400000;
const toDate = (key) => { const [y, m, d] = key.split('-').map(Number); return new Date(Date.UTC(y, m - 1, d)); };
const toKey = (date) => date.toISOString().slice(0, 10);

/** Today where the person is, as a key. */
export function today() {
  const n = new Date();
  return toKey(new Date(Date.UTC(n.getFullYear(), n.getMonth(), n.getDate())));
}

export const addDays = (key, n) => toKey(new Date(toDate(key).getTime() + n * MS));

/** The same day `n` months on, or the month's last when it has no such day (31 January → 28 February). */
export function addMonths(key, n) {
  const d = toDate(key);
  const y = d.getUTCFullYear(), m = d.getUTCMonth() + n;
  const last = new Date(Date.UTC(y, m + 1, 0)).getUTCDate();
  return toKey(new Date(Date.UTC(y, m, Math.min(d.getUTCDate(), last))));
}

/** The first day of the month a day is in. */
export const monthOf = (key) => `${key.slice(0, 7)}-01`;
const weekday = (key) => toDate(key).getUTCDay();

/** The month a day is in, as whole weeks of seven, each day { key, inMonth }; `weekStart` 1 is Monday, 0 Sunday. */
export function weeks(key, weekStart = 1) {
  const first = monthOf(key);
  let day = addDays(first, -((weekday(first) - weekStart + 7) % 7));
  const out = [];
  do {
    const week = [];
    for (let i = 0; i < 7; i++) { week.push({ key: day, inMonth: day.slice(0, 7) === first.slice(0, 7) }); day = addDays(day, 1); }
    out.push(week);
  } while (day.slice(0, 7) === first.slice(0, 7));
  return out;
}

/** Where a key takes the focus in a grid of days; null for a key that moves nothing. Shift with Page keys moves a year. */
export function move(key, name, weekStart = 1, shift = false) {
  switch (name) {
    case 'ArrowRight': return addDays(key, 1);
    case 'ArrowLeft': return addDays(key, -1);
    case 'ArrowDown': return addDays(key, 7);
    case 'ArrowUp': return addDays(key, -7);
    case 'Home': return addDays(key, -((weekday(key) - weekStart + 7) % 7));
    case 'End': return addDays(key, 6 - ((weekday(key) - weekStart + 7) % 7));
    case 'PageUp': return addMonths(key, shift ? -12 : -1);
    case 'PageDown': return addMonths(key, shift ? 12 : 1);
    default: return null;
  }
}

/** A day between `min` and `max`, either of which may be left out. */
export const allowed = (key, { min = null, max = null } = {}) => (!min || key >= min) && (!max || key <= max);

/** What a click on `key` makes of `value`: single, one day; multiple, days in order, toggled, at most `limit`; range, the
 *  first click starts it, the second ends it either way round, a third starts again. */
export function pick(mode, value, key, { min = null, max = null, limit = Infinity } = {}) {
  if (!allowed(key, { min, max })) return value;
  if (mode === 'multiple') {
    const days = value || [];
    if (days.includes(key)) return days.filter((d) => d !== key);
    return days.length >= limit ? days : [...days, key].sort();
  }
  if (mode === 'range') {
    const r = value || { from: null, to: null };
    if (!r.from || r.to) return { from: key, to: null };
    return key < r.from ? { from: key, to: r.from } : { from: r.from, to: key };
  }
  return key;
}

/** Inside a range, its ends included. */
export const inRange = (key, r) => !!(r && r.from && r.to && key >= r.from && key <= r.to);

/** Is `key` one of what is picked: the day, one of the days, one of the range's ends. */
export function picked(mode, value, key) {
  if (mode === 'multiple') return (value || []).includes(key);
  if (mode === 'range') return !!value && (value.from === key || value.to === key);
  return value === key;
}
