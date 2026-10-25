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

/** Every language FinStats speaks, by its own name; the first is the source. `checked`: the owner reads it and has
 *  checked it; the others are translations nobody here can check, and may lag behind (what one lacks is said in British
 *  English). The server's `i18n::LOCALES` is the same list. */
export const LANGUAGES = [
  { tag: 'en-GB', name: 'English (UK)', checked: true },
  { tag: 'en-US', name: 'English (US)', checked: true },
  { tag: 'nb-NO', name: 'Norsk Bokmål', checked: true },
  { tag: 'nn-NO', name: 'Norsk Nynorsk', checked: true },
  { tag: 'nn-NO-synnmore', name: 'Synnmørsk', checked: true },
  { tag: 'da-DK', name: 'Dansk' },
  { tag: 'de-DE', name: 'Deutsch' },
  { tag: 'es-ES', name: 'Español' },
  { tag: 'fi-FI', name: 'Suomi' },
  { tag: 'fr-FR', name: 'Français' },
  { tag: 'hi-IN', name: 'हिन्दी' },
  { tag: 'is-IS', name: 'Íslenska' },
  { tag: 'it-IT', name: 'Italiano' },
  { tag: 'ja-JP', name: '日本語' },
  { tag: 'ko-KR', name: '한국어' },
  { tag: 'nl-NL', name: 'Nederlands' },
  { tag: 'pl-PL', name: 'Polski' },
  { tag: 'pt-BR', name: 'Português (Brasil)' },
  { tag: 'ru-RU', name: 'Русский' },
  { tag: 'sv-SE', name: 'Svenska' },
  { tag: 'tr-TR', name: 'Türkçe' },
  { tag: 'uk-UA', name: 'Українська' },
  { tag: 'zh-CN', name: '简体中文' },
  { tag: 'zh-TW', name: '繁體中文' },
];
const SOURCE = LANGUAGES[0].tag;
/** What a reader is given when nothing their browser asks for is a language FinStats speaks (the server's `i18n::DEFAULT`). */
const DEFAULT = 'en-US';
const KEY = 'finstats.language';

/** The catalogue a browser language asks for: the tag as written, else its language (no is Bokmål; en is the default, and
 *  British where it names another country; pt is Brazilian; Chinese by its script or region). The server's
 *  `i18n::language_of` is the same rule. */
function languageOf(raw) {
  const tag = String(raw || '').trim().toLowerCase();
  const exact = LANGUAGES.find((l) => l.tag.toLowerCase() === tag);
  if (exact) return exact.tag;
  const [lang, ...rest] = tag.split('-');
  if (lang === 'nb' || lang === 'no') return 'nb-NO';
  if (lang === 'nn') return 'nn-NO';
  if (lang === 'en') return rest.length ? 'en-GB' : DEFAULT;
  if (lang === 'zh') return rest.some((p) => ['hant', 'tw', 'hk', 'mo'].includes(p)) ? 'zh-TW' : 'zh-CN';
  const same = LANGUAGES.find((l) => l.tag.split('-')[0].toLowerCase() === lang);
  return same ? same.tag : null;
}

/** The first of a browser's languages that FinStats speaks; nothing usable is the default. */
export function pickLanguage(list) {
  for (const raw of list || []) {
    const found = languageOf(raw);
    if (found) return found;
  }
  return DEFAULT;
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
let fallback = null;   // the source's words, for a language that does not have every text yet
let plurals = new Intl.PluralRules(SOURCE);
let numbers = new Intl.NumberFormat(SOURCE);

/** The language in use, for Intl and <html lang>. */
export const lang = () => current;

/** Speak `tag` from `catalogue` (its JSON), with `source` (British English's) for what it lacks. */
export function useCatalogue(tag, catalogue, source = null) {
  current = tag;
  words = catalogue || {};
  fallback = source && source !== catalogue ? source : null;
  plurals = new Intl.PluralRules(tag);
  numbers = new Intl.NumberFormat(tag);
  // The table reader reads numbers back as this language writes them: its units, and "5 of 26".
  const units = { ...((fallback && fallback.units) || {}), ...(words.units || {}) };
  const of = words.pages?.person?.of_total || fallback?.pages?.person?.of_total;
  finuiWords({ ...words.finui, units: of ? { ...units, of } : units }, tag);
}

/** Fetch the chosen language's catalogue and put it in use; the source's words if it cannot be had. */
export async function loadLanguage(tag = chosenLanguage()) {
  const get = async (t) => { const res = await fetch(`/assets/i18n/${t}.json`); if (!res.ok) throw new Error(String(res.status)); return res.json(); };
  try {
    // A language nobody here checks may lack a text: the source's words come along to say it.
    const checked = (LANGUAGES.find((l) => l.tag === tag) || {}).checked;
    const [own, source] = await Promise.all([get(tag), checked || tag === SOURCE ? null : get(SOURCE)]);
    useCatalogue(tag, own, source);
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
function find(from, key) {
  let v = from;
  for (const part of String(key).split('.')) { v = v && typeof v === 'object' ? v[part] : undefined; }
  return typeof v === 'string' || (v && typeof v === 'object' && typeof v.other === 'string') ? v : undefined;
}
let sourcePlurals = new Intl.PluralRules(SOURCE);
/** The text of `key` and the plural rules that fit it: the language's own, else the source's. */
function lookup(key) {
  const own = find(words, key);
  if (own !== undefined || !fallback) return own;
  return find(fallback, key);
}
const rulesFor = (key) => (find(words, key) === undefined && fallback ? sourcePlurals : plurals);
/** The text of `key` (a plural picks its form by `vars.count`), its placeholders not yet filled; null if there is none. */
function text(key, vars) {
  const v = lookup(key);
  if (typeof v === 'string') return v;
  if (v && typeof v === 'object' && typeof v.other === 'string') {
    const n = vars && vars.count != null ? Number(vars.count) : NaN;
    return v[Number.isFinite(n) ? rulesFor(key).select(n) : 'other'] ?? v.other;
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
