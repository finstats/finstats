// FinUI: date-picker, the rules that are not drawing — a day typed into its field, read in the ways people write one:
// 2026-10-05, 5.10.2026, 05/10/2026 (day first, as most of the world writes it), 5 October 2026, Oct 5, 2026. Pure;
// tested in FinUI's repository.

const MONTHS = ['jan', 'feb', 'mar', 'apr', 'may', 'jun', 'jul', 'aug', 'sep', 'oct', 'nov', 'dec'];
const key = (y, m, d) => {
  const t = new Date(Date.UTC(y, m - 1, d));
  if (t.getUTCFullYear() !== y || t.getUTCMonth() !== m - 1 || t.getUTCDate() !== d) return null;   // 31 February
  return `${String(y).padStart(4, '0')}-${String(m).padStart(2, '0')}-${String(d).padStart(2, '0')}`;
};
const month = (word) => { const i = MONTHS.indexOf(String(word).slice(0, 3).toLowerCase()); return i < 0 ? null : i + 1; };

/** The day `text` names, as YYYY-MM-DD; null when it names none. */
export function parseDay(text) {
  const t = String(text || '').trim();
  let m;
  if ((m = /^(\d{4})-(\d{1,2})-(\d{1,2})$/.exec(t))) return key(+m[1], +m[2], +m[3]);
  if ((m = /^(\d{1,2})[./-](\d{1,2})[./-](\d{4})$/.exec(t))) return key(+m[3], +m[2], +m[1]);
  if ((m = /^(\d{1,2})\.?\s+([a-z]+)\.?,?\s+(\d{4})$/i.exec(t)) && month(m[2])) return key(+m[3], month(m[2]), +m[1]);
  if ((m = /^([a-z]+)\.?\s+(\d{1,2}),?\s+(\d{4})$/i.exec(t)) && month(m[1])) return key(+m[3], month(m[1]), +m[2]);
  return null;
}
