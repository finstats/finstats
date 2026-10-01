// FinUI: pagination. Where a long list stands — "51–100 of 3,300", page 2 of 66 — and the way to the page before and after.

import { h, icon } from '../../core.js';
import { num } from '../../format.js';
import { button } from '../button/button.js';

export function pagination({ page, perPage, total, onPage }) {
  const pages = Math.max(1, Math.ceil(total / perPage));
  const from = total ? (page - 1) * perPage + 1 : 0, to = Math.min(total, page * perPage);
  // Buttons stay enabled; out-of-range clicks are simply ignored (aria-disabled communicates the edge).
  const mk = (lbl, ic, target, off) => button({ size: 'sm', variant: 'ghost', type: 'button', 'aria-label': lbl, 'aria-disabled': off ? 'true' : null,
    onClick: () => { if (!off) onPage(target); } }, icon(ic, 14));
  return h('nav', { class: 'fui-pagination', 'aria-label': 'Pagination' },
    h('span', { class: 'fui-pagination__info mono' }, `${num(from)}–${num(to)} of ${num(total)}`),
    h('div', { class: 'fui-pagination__buttons' }, mk('Previous page', 'chevronLeft', page - 1, page <= 1),
      h('span', { class: 'fui-pagination__page mono' }, `${page} / ${pages}`),
      mk('Next page', 'chevronRight', page + 1, page >= pages)));
}

// ---------------------------------------------------------------- media

export const meta = {
  name: 'pagination',
  purpose: 'Says which part of a long list is shown, and moves to the part before or after.',
  use: 'Under a list the server pages (Activity, the audit log, Library health). The list itself is sorted and filtered on the server.',
  avoid: 'A list that fits on one page. Infinite scrolling where somebody needs to come back to a place (a page number is a place).',
  variants: ['first page', 'middle', 'last page'],
  states: ['an edge button is aria-disabled: shown, not pressable'],
  a11y: 'A <nav aria-label="Pagination">; the buttons are named Previous page and Next page.',
  props: { 'pagination({ page, perPage, total, onPage })': 'onPage(n) is asked for a page that exists' },
  examples: [
    { name: 'The first page', render: () => pagination({ page: 1, perPage: 50, total: 3300, onPage: () => {} }) },
    { name: 'Somewhere in the middle', render: () => pagination({ page: 2, perPage: 50, total: 3300, onPage: () => {} }) },
  ],
};
