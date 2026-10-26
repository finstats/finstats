// A date or a time of day in the format a person chose (Settings → Appearance), or null for "the language's own way",
// which the caller then writes with Intl as before. Pure, so the rules are tested in node like i18n.js's.

/** The date formats on offer, as they read for 31 January 2026: 31.01.2026, 31/01/2026, … 2026/01/31. */
export const DATE_FORMATS = ['dd.mm.yyyy', 'dd/mm/yyyy', 'dd-mm-yyyy', 'mm/dd/yyyy', 'yyyy-mm-dd', 'yyyy.mm.dd', 'yyyy/mm/dd'];
/** The time formats on offer: 01:01–23:59, 1:01–23:59, 01:01 am–11:59 pm, 1:01 am–11:59 pm. */
export const TIME_FORMATS = ['HH:mm', 'H:mm', 'hh:mm a', 'h:mm a'];

const two = (n) => String(n).padStart(2, '0');

/** `d` in `format`, day and month always two digits; null for a format not on offer. */
export function writeDate(d, format) {
  if (!DATE_FORMATS.includes(format)) return null;
  const parts = { dd: two(d.getDate()), mm: two(d.getMonth() + 1), yyyy: String(d.getFullYear()) };
  const sep = format.replace(/[a-z]/g, '')[0];
  return format.split(sep).map((p) => parts[p]).join(sep);
}

/** `d`'s time of day in `format`, with the seconds when asked; `words` are the language's {am, pm}. Null for a format
 *  not on offer. In twelve-hour time midnight is 12 am and noon 12 pm. */
export function writeTime(d, format, words, { seconds = false } = {}) {
  if (!TIME_FORMATS.includes(format)) return null;
  const twelve = format.endsWith(' a');
  const h24 = d.getHours();
  const h = twelve ? (h24 % 12 || 12) : h24;
  const hour = format.startsWith('HH') || format.startsWith('hh') ? two(h) : String(h);
  const clock = `${hour}:${two(d.getMinutes())}${seconds ? ':' + two(d.getSeconds()) : ''}`;
  return twelve ? `${clock} ${h24 < 12 ? words.am : words.pm}` : clock;
}
