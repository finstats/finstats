// The words of the interface, in the language this browser reads FinStats in. Every text a page shows goes through t()
// (or tParts(), when a placeholder is an element): the catalogues are web/assets/i18n/<BCP 47 tag>.json, British
// English the source, and a test holds the others to its keys, placeholders and plural forms.
//
// The language is this browser's (Settings → Appearance → Language, kept in localStorage like the layout): nothing
// stored follows the browser's own languages. main.js loads the catalogue before any page module runs, so a module may
// say t('…') at the top level; choosing another language reloads the page. The server is told it with every request
// (api.js sends Accept-Language), so what it answers in words is in the same language. FinUI's components speak
// through the catalogue's `finui` section.

import { words as finuiWords } from '../finui/i18n.js';

/** Every language FinStats speaks, by its own name; the first is the source. */
export const LANGUAGES = [
  { tag: 'en-GB', name: 'English (UK)' },
  { tag: 'en-US', name: 'English (US)' },
  { tag: 'nb-NO', name: 'Norsk bokmål' },
  { tag: 'nn-NO', name: 'Norsk nynorsk' },
];
const SOURCE = LANGUAGES[0].tag;
const KEY = 'finstats.language';

/** The first of a browser's languages that FinStats speaks: a tag as written, else its language (nb, nn, and no for
 *  Bokmål; en is British unless it says America). Nothing usable is the source. The server's `i18n::pick` is the same rule. */
export function pickLanguage(list) {
  for (const raw of list || []) {
    const tag = String(raw || '').trim().toLowerCase();
    const exact = LANGUAGES.find((l) => l.tag.toLowerCase() === tag);
    if (exact) return exact.tag;
    const lang = tag.split('-')[0];
    if (lang === 'nb' || lang === 'no') return 'nb-NO';
    if (lang === 'nn') return 'nn-NO';
    if (lang === 'en') return SOURCE;
  }
  return SOURCE;
}

/** What this browser chose ('' = follow the browser). */
export function storedLanguage() {
  try { const v = localStorage.getItem(KEY); return LANGUAGES.some((l) => l.tag === v) ? v : ''; } catch { return ''; }
}
/** The language to read FinStats in: the one chosen here, else the browser's. */
export function chosenLanguage() {
  return storedLanguage() || pickLanguage(typeof navigator !== 'undefined' ? navigator.languages || [navigator.language] : []);
}
/** Keep a choice ('' follows the browser) and read the page again in it. */
export function setLanguage(tag) {
  try { if (tag) localStorage.setItem(KEY, tag); else localStorage.removeItem(KEY); } catch { /* storage blocked: this visit only */ }
  location.reload();
}

let current = SOURCE;
let words = {};
let plurals = new Intl.PluralRules(SOURCE);
let numbers = new Intl.NumberFormat(SOURCE);

/** The language in use, for Intl and <html lang>. */
export const lang = () => current;

/** Speak `tag` from `catalogue` (its JSON). */
export function useCatalogue(tag, catalogue) {
  current = tag;
  words = catalogue || {};
  plurals = new Intl.PluralRules(tag);
  numbers = new Intl.NumberFormat(tag);
  finuiWords(words.finui, tag);
}

/** Fetch the chosen language's catalogue and put it in use; the source's words if it cannot be had. */
export async function loadLanguage(tag = chosenLanguage()) {
  try {
    const res = await fetch(`/assets/i18n/${tag}.json`);
    if (!res.ok) throw new Error(String(res.status));
    useCatalogue(tag, await res.json());
  } catch {
    if (tag !== SOURCE) return loadLanguage(SOURCE);
  }
  if (typeof document !== 'undefined') {
    document.documentElement.lang = current;
    // The server reads it too (i18n::asked): a picture or a download the browser fetches by itself sends the browser's
    // own languages, and a card drawn in them would not be the language this page reads in.
    try { document.cookie = `finstats_lang=${current}; path=/; max-age=31536000; SameSite=Lax`; } catch { /* cookies blocked: Accept-Language decides */ }
  }
  return current;
}

const missing = new Set();
function lookup(key) {
  let v = words;
  for (const part of String(key).split('.')) { v = v && typeof v === 'object' ? v[part] : undefined; }
  return v;
}
/** The text of `key` (a plural picks its form by `vars.count`), its placeholders not yet filled; null if there is none. */
function text(key, vars) {
  const v = lookup(key);
  if (typeof v === 'string') return v;
  if (v && typeof v === 'object' && typeof v.other === 'string') {
    const n = vars && vars.count != null ? Number(vars.count) : NaN;
    return v[Number.isFinite(n) ? plurals.select(n) : 'other'] ?? v.other;
  }
  if (!missing.has(key)) { missing.add(key); if (typeof console !== 'undefined') console.warn(`i18n: no text for ${key} in ${current}`); }
  return null;
}
const filled = (value) => (typeof value === 'number' ? numbers.format(value) : String(value ?? ''));

/** `key` in the language in use, `{name}` filled from `vars` (numbers written the language's way). A key with no text
 *  is the key itself, so a gap shows as one rather than as nothing. */
export function t(key, vars) {
  const s = text(key, vars);
  if (s == null) return String(key);
  return vars ? s.replace(/\{([a-z_]+)\}/g, (m, name) => (name in vars ? filled(vars[name]) : m)) : s;
}

/** As t(), but `fallback` (said as it is) where the catalogue has no text for `key`: for ids from the server that the
 *  catalogue may not know yet (a new kind, a font's own name). */
export function tOr(key, fallback, vars) {
  const v = lookup(key);
  return typeof v === 'string' || (v && typeof v === 'object' && typeof v.other === 'string') ? t(key, vars) : fallback;
}

/** As t(), for a text with an element in it: an array for h() of the text around each placeholder and what fills it, a
 *  node left a node ("Use your Jellyfin account on <strong>Jellyfin</strong>."). */
export function tParts(key, vars = {}) {
  const s = text(key, vars);
  if (s == null) return [String(key)];
  const out = [];
  let at = 0;
  for (const m of s.matchAll(/\{([a-z_]+)\}/g)) {
    if (!(m[1] in vars)) continue;
    const value = vars[m[1]];
    const before = s.slice(at, m.index);
    at = m.index + m[0].length;
    if (value && typeof value === 'object') { if (before) out.push(before); out.push(value); }
    else { const glued = before + filled(value); if (typeof out[out.length - 1] === 'string') out[out.length - 1] += glued; else if (glued) out.push(glued); }
  }
  const rest = s.slice(at);
  if (rest) { if (typeof out[out.length - 1] === 'string') out[out.length - 1] += rest; else out.push(rest); }
  return out;
}
