// FinUI: shelf. A row of cards that goes on past the edge: it scrolls by hand — the wheel with Shift, a drag, a swipe —
// and its arrows and keys glide a page of whole cards at a time. No snapping: a short wheel notch moves it a little.

import { h, icon } from '../../core.js';
import { target, ends } from './plan.js';

/** shelf({ label, items, arrows }) → the row: `items` the cards (nodes); `arrows` adds the two buttons beside its name. */
export function shelf({ label, items, arrows = true }) {
  const row = h('ul', { class: 'fui-shelf__row', tabindex: '0', 'aria-label': label }, items.map((it) => h('li', { class: 'fui-shelf__item' }, it)));
  const back = h('button', { type: 'button', class: 'fui-shelf__arrow', 'aria-label': `Back along ${label.toLowerCase()}` }, icon('chevronLeft', 15));
  const on = h('button', { type: 'button', class: 'fui-shelf__arrow', 'aria-label': `On along ${label.toLowerCase()}` }, icon('chevronRight', 15));
  const box = () => { const first = row.firstElementChild, second = first && first.nextElementSibling; return { width: row.clientWidth, scrollWidth: row.scrollWidth, card: second ? second.offsetLeft - first.offsetLeft : (first ? first.offsetWidth : 200) }; };
  const smooth = () => (matchMedia('(prefers-reduced-motion: reduce)').matches ? 'auto' : 'smooth');
  const glide = (dir) => row.scrollTo({ left: target(row.scrollLeft, dir, box()), behavior: smooth() });
  const paint = () => { const e = ends(row.scrollLeft, box()); back.setAttribute('aria-disabled', String(e.start)); on.setAttribute('aria-disabled', String(e.end)); };
  back.addEventListener('click', () => glide(-1));
  on.addEventListener('click', () => glide(1));
  row.addEventListener('scroll', paint, { passive: true });
  row.addEventListener('keydown', (e) => {
    const b = box();
    const to = { ArrowRight: row.scrollLeft + b.card, ArrowLeft: row.scrollLeft - b.card, Home: 0, End: b.scrollWidth }[e.key];
    if (to === undefined) return;
    e.preventDefault();
    row.scrollTo({ left: to, behavior: smooth() });
  });
  requestAnimationFrame(paint);
  return h('section', { class: 'fui-shelf', 'aria-label': label },
    h('div', { class: 'fui-shelf__head' }, h('h3', { class: 'fui-shelf__title' }, label), arrows ? h('div', { class: 'fui-shelf__arrows' }, back, on) : null), row);
}

const FILMS = ['Big Buck Bunny', 'Sintel', 'Tears of Steel', 'Cosmos Laundromat', 'Elephants Dream', 'Spring', 'Agent 327', 'Caminandes'];
export const meta = {
  name: 'shelf',
  purpose: 'Lays out a row of cards that goes on past the edge, moved by hand or a page at a time by its arrows and keys.',
  use: 'Recently added, coming up, a person’s films: many of one kind, browsed rather than read. Each card is what the page gives it (a media card, a poster).',
  avoid: 'Everything there is (a grid with a way to page or filter). Scroll snapping: a short wheel notch springs back and the row barely moves.',
  variants: ['with arrows', 'without'],
  states: ['at the start (back unavailable)', 'at the end (on unavailable)'],
  a11y: 'A section named by its label; the row is focusable and named too: the arrows move it a card, Home and End to its ends; the buttons say aria-disabled at an end.',
  props: { 'shelf({ label, items, arrows })': 'items: nodes' },
  playground: {
    controls: [
      { key: 'arrows', label: 'Arrows', on: true },
      { key: 'many', label: 'Many cards', on: true },
    ],
    render: (o) => h('div', { class: 'fui-shelf__demo' }, shelf({ label: 'Recently added', arrows: o.arrows,
      items: (o.many ? FILMS : FILMS.slice(0, 3)).map((f, i) => h('a', { class: 'fui-shelf__demo-card', href: '#' }, h('span', { class: 'fui-shelf__demo-art' }, f.split(' ').map((w) => w[0]).join('')), h('span', null, f), h('span', { class: 'muted' }, `${2006 + i}`))) })),
  },
};
