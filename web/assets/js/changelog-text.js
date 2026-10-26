// How a patch note's words are drawn, for both layouts' Patch notes: its emphasis and code as elements, and its dates
// as the browser writes them.

import { h } from './dom.js';
import { t, lang } from './i18n.js';

/** Notes may use **bold**, *italic* and `code`. Built as DOM nodes; nothing is ever parsed as HTML. */
export function inline(text) {
  const out = [];
  // Bold before italic, so ** is never read as two empty emphases; an italic opens on a letter,
  // so a lone asterisk in running text stays an asterisk.
  const re = /\*\*([^*]+)\*\*|\*([^*\s][^*]*?)\*|`([^`]+)`/g;
  let last = 0;
  let m;
  while ((m = re.exec(text))) {
    if (m.index > last) out.push(text.slice(last, m.index));
    // Emphasis may hold code (**a broken `%`-escape**), so what it holds is drawn the same way.
    out.push(m[1] != null ? h('strong', null, inline(m[1])) : m[2] != null ? h('em', null, inline(m[2])) : h('code', { class: 'mono cl-code' }, m[3]));
    last = re.lastIndex;
  }
  if (last < text.length) out.push(text.slice(last));
  return out;
}

const KINDS = ['Added', 'Changed', 'Security', 'Performance', 'Stability', 'Fixed', 'Removed'];
/** A group's kind as the changelog names it (Added, Fixed…), in the language in use. */
export const kindName = (kind) => (KINDS.includes(kind) ? t(`changelog_text.kind.${kind}`) : kind);

const DAY = { year: 'numeric', month: 'long', day: 'numeric' };
export const day = (iso) => {
  const d = /^\d{4}-\d{2}-\d{2}$/.test(iso || '') ? new Date(iso + 'T12:00:00') : null;
  return d && !isNaN(d) ? d : null;
};

export function longDate(iso) {
  const d = day(iso);
  return d ? d.toLocaleDateString(lang(), DAY) : iso || '';
}

/** "September 25 – 28, 2026": the locale decides what the two days share; one day reads as one. */
export function days(first, last) {
  const a = day(first), b = day(last);
  if (!a || !b || first === last) return longDate(last || first);
  const f = new Intl.DateTimeFormat(lang(), DAY);
  return typeof f.formatRange === 'function' ? f.formatRange(a, b) : `${f.format(a)} – ${f.format(b)}`;
}
