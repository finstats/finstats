// Formatting helpers, the logo, and FinUI's builder re-exported. Everything that comes from the API goes
// through text nodes, never innerHTML.

import { h, s, clear, mount, icon } from '../finui/core.js';
import { num, initials } from '../finui/format.js';

const nf = new Intl.NumberFormat('en-US');   // the other formatters' numbers, the way num() prints them

// FinUI's builder and icons, for the modules that still import them from here.
export { h, s, clear, mount, icon, num, initials };

/** The FinStats logo. One source for favicon, sidebar and sign-in: /assets/logo.svg. */
export function logo(size = 24) {
  return h('img', { class: 'brand-logo', src: '/assets/logo.svg', width: size, height: size, alt: '', decoding: 'async' });
}

// ---------------------------------------------------------------- formatting

export function compact(n) {
  n = Number(n) || 0;
  const abs = Math.abs(n);
  if (abs < 10000) return nf.format(Math.round(n));
  if (abs < 1e6) return trim1(n / 1e3) + 'K';
  if (abs < 1e9) return trim1(n / 1e6) + 'M';
  return trim1(n / 1e9) + 'B';
}
const trim1 = (x) => (Math.abs(x) >= 100 ? Math.round(x).toString() : x.toFixed(1).replace(/\.0$/, ''));

/** 3h 12m · 48m · 2d 4h · 35s */
export function duration(sec) {
  sec = Math.max(0, Math.round(Number(sec) || 0));
  if (sec < 60) return sec + 's';
  const m = Math.floor(sec / 60);
  if (m < 60) return m + 'm';
  const hh = Math.floor(m / 60);
  if (hh < 48) return hh + 'h' + (m % 60 ? ' ' + (m % 60) + 'm' : '');
  const d = Math.floor(hh / 24);
  return d + 'd' + (hh % 24 ? ' ' + (hh % 24) + 'h' : '');
}

export function durationExact(sec) {
  sec = Math.max(0, Math.round(Number(sec) || 0));
  const hh = Math.floor(sec / 3600), m = Math.floor((sec % 3600) / 60), ss = sec % 60;
  return `${nf.format(hh)}h ${m}m ${ss}s`;
}

/** h:mm:ss clock for playback positions */
export function clock(sec) {
  sec = Math.max(0, Math.round(Number(sec) || 0));
  const hh = Math.floor(sec / 3600), m = Math.floor((sec % 3600) / 60), ss = sec % 60;
  const p = (x) => String(x).padStart(2, '0');
  return hh ? `${hh}:${p(m)}:${p(ss)}` : `${m}:${p(ss)}`;
}

export function durEl(sec, cls = 'mono') {
  return h('span', { class: cls, title: durationExact(sec) }, duration(sec));
}

export function bytes(n) {
  n = Number(n) || 0;
  if (n < 1024) return n + ' B';
  const u = ['KB', 'MB', 'GB', 'TB', 'PB'];
  let i = -1;
  do { n /= 1024; i++; } while (n >= 1024 && i < u.length - 1);
  return (n >= 100 ? Math.round(n) : n.toFixed(1)) + ' ' + u[i];
}

export function bitrate(bps) {
  bps = Number(bps) || 0;
  if (!bps) return '–';
  return bps >= 1e6 ? (bps / 1e6).toFixed(1) + ' Mbps' : Math.round(bps / 1e3) + ' kbps';
}

const dtf = new Intl.DateTimeFormat(undefined, { dateStyle: 'medium', timeStyle: 'short' });
const dayf = new Intl.DateTimeFormat(undefined, { month: 'short', day: 'numeric' });
const dayfy = new Intl.DateTimeFormat(undefined, { month: 'short', day: 'numeric', year: 'numeric' });
const weekdayf = new Intl.DateTimeFormat(undefined, { weekday: 'short', month: 'short', day: 'numeric', year: 'numeric' });

export const dateTime = (ts) => (ts ? dtf.format(new Date(ts * 1000)) : '–');
/** The day of a moment, with its year: "Sep 1, 2026". */
export const dateOnly = (ts) => (ts ? dayfy.format(new Date(ts * 1000)) : '–');

export function relTime(ts) {
  if (!ts) return 'never';
  const d = Date.now() / 1000 - ts;
  if (d < 45) return 'just now';
  if (d < 3600) return Math.round(d / 60) + 'm ago';
  if (d < 86400) return Math.round(d / 3600) + 'h ago';
  if (d < 86400 * 30) return Math.round(d / 86400) + 'd ago';
  if (d < 86400 * 365) return Math.round(d / (86400 * 30)) + 'mo ago';
  return Math.round(d / (86400 * 365)) + 'y ago';
}

/** "in 6 days": `relTime` only looks backwards. */
export function untilText(ts) {
  if (!ts) return '';
  const s = ts - Date.now() / 1000;
  if (s <= 0) return 'due now';
  if (s <= 90) return 'within a minute or two';
  if (s < 3600) return `in ${Math.round(s / 60)} minutes`;
  if (s < 86400 * 1.5) return `in ${Math.round(s / 3600)} hours`;
  return `in ${Math.round(s / 86400)} days`;
}

/** A compact absolute timestamp: "19 Sep, 14:04", with the year once it is not this year. */
export function shortStamp(ts) {
  if (!ts) return '';
  const d = new Date(ts * 1000);
  const sameYear = d.getFullYear() === new Date().getFullYear();
  return d.toLocaleString(undefined, { day: 'numeric', month: 'short', ...(sameYear ? {} : { year: 'numeric' }), hour: '2-digit', minute: '2-digit', hour12: false });
}

export function relEl(ts, cls = 'mono') {
  return h('time', { class: cls, title: dateTime(ts), dateTime: ts ? new Date(ts * 1000).toISOString() : '' }, relTime(ts));
}

/** 'YYYY-MM-DD' → local Date (no TZ shifting) */
export function parseDay(str) {
  const [y, m, d] = String(str).split('-').map(Number);
  return new Date(y, (m || 1) - 1, d || 1);
}
export const dayLabel = (str) => dayf.format(parseDay(str));
export const dayLabelLong = (str) => weekdayf.format(parseDay(str));
export const dayLabelYear = (str) => dayfy.format(parseDay(str));

export const pct = (x, digits = 0) => (x == null ? '–' : (x * 100).toFixed(digits) + '%');

/** Where a play came from, by the `source` the server gives it: FinStats itself, or a tracker it imports from. One list
 *  for every page, so a tracker added to FinStats is not missing from one of them. */
export const TRACKERS = { live: 'FinStats', jellystat: 'Jellystat', streamystats: 'Streamystats', tautulli: 'Tautulli' };


export function episodeCode(season, episode) {
  if (season == null && episode == null) return '';
  const p = (x) => String(x).padStart(2, '0');
  return (season != null ? 'S' + p(season) : '') + (episode != null ? 'E' + p(episode) : '');
}

// Jellyfin reports ISO 639-2 codes, and for some languages the "bibliographic" one (ger, fre) that browsers do not know.
const LANG_B = { ger: 'deu', fre: 'fra', chi: 'zho', dut: 'nld', cze: 'ces', gre: 'ell', rum: 'ron', per: 'fas', slo: 'slk', ice: 'isl', mac: 'mkd',
  may: 'msa', alb: 'sqi', arm: 'hye', baq: 'eus', bur: 'mya', geo: 'kat', tib: 'bod', wel: 'cym' };
let langNames;
/** "jpn" → "Japanese". An unknown code is shown as it is; a track without a language is "Unknown". */
export function languageName(code) {
  const c = String(code || '').trim().toLowerCase();
  if (!c || c === 'und' || c === 'unk' || c === 'zxx' || c === 'mis') return 'Unknown';
  if (c === 'other') return 'Other';
  try {
    if (langNames === undefined) langNames = typeof Intl.DisplayNames === 'function' ? new Intl.DisplayNames(['en'], { type: 'language', fallback: 'none' }) : null;
    const name = langNames && langNames.of(LANG_B[c] || c);
    if (name) return name;
  } catch { /* not a well-formed code */ }
  return c.toUpperCase();
}

export const METHOD_LABEL = { DirectPlay: 'Direct play', DirectStream: 'Direct stream', Transcode: 'Transcode' };
export const methodLabel = (m) => METHOD_LABEL[m] || m || 'Unknown';

/** "ContainerNotSupported" → "Container not supported" */
export function humanize(str) {
  const sp = String(str || '').replace(/([a-z0-9])([A-Z])/g, '$1 $2').replace(/_/g, ' ');
  return sp.charAt(0).toUpperCase() + sp.slice(1).toLowerCase();
}

/** External links come from the API: only https: is ever rendered as a link. */
export function safeHttps(url) {
  try { const u = new URL(String(url)); return u.protocol === 'https:' ? u.href : null; } catch { return null; }
}

/** Time of day, for timelines: 21:04:12 */
const timef = new Intl.DateTimeFormat(undefined, { hour: '2-digit', minute: '2-digit', second: '2-digit', hour12: false });
export const timeOfDay = (ts) => (ts ? timef.format(new Date(ts * 1000)) : '–');

export function debounce(fn, ms) {
  let t;
  const d = (...a) => { clearTimeout(t); t = setTimeout(() => fn(...a), ms); };
  d.cancel = () => clearTimeout(t);
  return d;
}

export const store = {
  get(k, fallback = null) { try { const v = localStorage.getItem(k); return v == null ? fallback : v; } catch { return fallback; } },
  set(k, v) { try { localStorage.setItem(k, v); } catch { /* private mode etc. */ } },
  remove(k) { try { localStorage.removeItem(k); } catch { /* private mode etc. */ } },
};
