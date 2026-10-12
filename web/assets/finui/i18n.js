// FinUI i18n: the words a component writes itself (a button's "Try again", a dialog's "Close", an empty table's line)
// are English unless the app hands FinUI its own, and numbers are written in the app's language. FinUI ships no
// catalogue: `words(dict, locale)` is how an app speaks through it, and a key it has no word for is the English
// the component passed beside it.

let given = {};
let tag = 'en-US';
let rules = new Intl.PluralRules(tag);

/** Speak `locale` (a BCP 47 tag) with `dict`: { component: { key: text } }, a text with `{name}` placeholders or an
 *  object of plural forms ({ one, other }) picked by `vars.count`. */
export function words(dict, locale) {
  given = dict || {};
  if (locale) { tag = locale; rules = new Intl.PluralRules(tag); }
}

/** The language numbers and dates are written in. */
export const locale = () => tag;

/** The app's word for `key` ('error.title'), else `english`; `{name}` filled from `vars`. */
export function say(key, english, vars) {
  let v = given;
  for (const part of key.split('.')) v = v && typeof v === 'object' ? v[part] : undefined;
  if (v && typeof v === 'object' && typeof v.other === 'string') {
    const n = vars && vars.count != null ? Number(vars.count) : NaN;
    v = v[Number.isFinite(n) ? rules.select(n) : 'other'] ?? v.other;
  }
  const s = typeof v === 'string' ? v : english;
  return vars ? s.replace(/\{([a-z_]+)\}/g, (m, name) => (name in vars ? String(vars[name]) : m)) : s;
}
