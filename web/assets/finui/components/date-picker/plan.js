// FinUI: date-picker, the rules that are not drawing: a day typed into its field, read in the ways people write one:
// 2026-10-05, 5.10.2026, 05/10/2026 (day first, as most of the world writes it), 5 October 2026, Oct 5, 2026. Pure;
// tested in FinUI's repository.

const MONTHS = ['jan', 'feb', 'mar', 'apr', 'may', 'jun', 'jul', 'aug', 'sep', 'oct', 'nov', 'dec'];
const key = (y, m, d) => {
  const t = new Date(Date.UTC(y, m - 1, d));
  if (t.getUTCFullYear() !== y || t.getUTCMonth() !== m - 1 || t.getUTCDate() !== d) return null;   // 31 February
  return `${String(y).padStart(4, '0')}-${String(m).padStart(2, '0')}-${String(d).padStart(2, '0')}`;
};
const bare = (w) => String(w).toLowerCase().replace(/\.$/, '');
const month = (word, names = []) => {
  const w = bare(word);
  const own = names.findIndex((forms) => forms.some((f) => bare(f) === w || (w.length >= 3 && bare(f).startsWith(w))));
  if (own >= 0) return own + 1;
  const i = MONTHS.indexOf(w.slice(0, 3));
  return i < 0 ? null : i + 1;
};

/** The day `text` names, as YYYY-MM-DD; null when it names none. `names`: the months as a language writes them, twelve
 *  lists of forms ('oktober', 'okt.'), read beside the English. */
export function parseDay(text, names = []) {
  const t = String(text || '').trim();
  let m;
  if ((m = /^(\d{4})-(\d{1,2})-(\d{1,2})$/.exec(t))) return key(+m[1], +m[2], +m[3]);
  if ((m = /^(\d{1,2})[./-](\d{1,2})[./-](\d{4})$/.exec(t))) return key(+m[3], +m[2], +m[1]);
  if ((m = /^(\d{1,2})\.?\s+(\p{L}+)\.?,?\s+(\d{4})$/iu.exec(t)) && month(m[2], names)) return key(+m[3], month(m[2], names), +m[1]);
  if ((m = /^(\p{L}+)\.?\s+(\d{1,2}),?\s+(\d{4})$/iu.exec(t)) && month(m[1], names)) return key(+m[3], month(m[1], names), +m[2]);
  return null;
}
