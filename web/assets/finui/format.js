// FinUI format: the two ways a component writes a value itself. Numbers are printed in the app's language (i18n.js),
// the way tables read them back when they sort; a name with no picture becomes its initials.

import { locale } from './i18n.js';

let nf = null, nfFor = '';
/** 1234.5 → "1,235" (en), "1 235" (nb). */
export function num(n) {
  if (nfFor !== locale()) { nfFor = locale(); nf = new Intl.NumberFormat(nfFor); }
  return nf.format(Math.round(Number(n) || 0));
}

/** "Big Buck Bunny" → "BB". */
export function initials(name) {
  const parts = String(name || '?').trim().split(/\s+/).filter(Boolean);
  const a = parts[0]?.[0] || '?';
  const b = parts.length > 1 ? parts[parts.length - 1][0] : '';
  return (a + b).toUpperCase();
}
