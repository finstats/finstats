// FinUI: card. A surface that holds one thing — a chart, a list, a form — with a heading, a line under it and actions
// on the right. Flush, it gives its body's edges to a table or a list that draws its own.

import { h, icon, mount } from '../../core.js';
import { button } from '../button/button.js';

export function card({ title, sub, actions, body, cls = '', id, href } = {}) {
  // With an href the whole card is the link: an entry of an index, a way into what it names.
  return h(href ? 'a' : 'section', { class: ['fui-card', href && 'fui-card--link', cls], id, href },
    title || actions ? h('div', { class: 'fui-card__head' },
      h('div', null, title ? h('h2', { class: 'fui-card__title' }, title) : null, sub ? h('p', { class: 'fui-card__sub' }, sub) : null),
      actions ? h('div', { class: 'fui-card__actions' }, actions) : null) : null,
    h('div', { class: 'fui-card__body' }, body));
}

/** A card whose body flips between a chart and a table of the same data between the chart and a table of the same data. */
export function chartCard({ title, sub, controls, chart, table, cls = '' }) {
  let showTable = false;
  const body = h('div');
  const label = h('span', null, 'Table');
  const toggle = button({ variant: 'ghost', size: 'sm', type: 'button', 'aria-pressed': 'false',
    onClick: () => { showTable = !showTable; render(); } }, icon('table', 14), label);
  function render() {
    toggle.setAttribute('aria-pressed', String(showTable));
    toggle.replaceChildren(icon(showTable ? 'chart' : 'table', 14), h('span', null, showTable ? 'Chart' : 'Table'));
    mount(body, showTable ? table() : chart());
  }
  render();
  const el = card({ title, sub, actions: [controls, table ? toggle : null], body, cls: 'chart-card ' + cls });
  el.rerender = render;
  return el;
}

const said = (n) => h('p', null, `${n} plays this week.`);
export const meta = {
  name: 'card',
  purpose: 'A surface for one thing: a heading, an optional line under it, actions on the right, and a body.',
  use: 'One card per question a page answers. Flush (cls: "fui-card--flush") when the body is a table or a list with its own edges. chartCard when the same data reads as a chart and as a table.',
  avoid: 'Cards inside cards. A card for a single number (statTile). Folding a card away: content stays visible, and long pages get a list to jump with.',
  variants: ['plain', 'with sub and actions', 'flush', 'a link (the whole card)', 'chartCard (chart ⇄ table)'],
  states: ['is-hit (a link landed on it: reveal() marks it for a moment)'],
  a11y: 'A <section> with an <h2> heading. chartCard’s switch is a toggle button (aria-pressed) whose label says what it shows next.',
  props: {
    'card({ title, sub, actions, body, cls, id, href })': 'title and sub are text; actions and body are nodes; cls adds classes (fui-card--flush); id makes it a place a link can land; href makes the whole card a link.',
    'chartCard({ title, sub, controls, chart, table, cls })': 'chart and table are functions that draw; the card calls one of them. el.rerender() draws again.',
  },
  playground: {
    controls: [
      { key: 'sub', label: 'A line under the title', on: true },
      { key: 'action', label: 'An action', on: true },
      { key: 'flush', label: 'Flush, for a list' },
      { key: 'link', label: 'The whole card a link' },
      { key: 'chart', label: 'A chart that is also a table' },
    ],
    // A link card has no actions of its own (the whole card is the one thing to press), so it leaves that switch be.
    render: (o) => o.chart && !o.link
      ? chartCard({ title: 'Plays', sub: o.sub ? 'This week' : null, chart: () => said(42), table: () => h('table', null, h('tbody', null, h('tr', null, h('th', null, 'Plays'), h('td', null, '42')))) })
      : o.link ? card({ href: '#', title: 'Recently added', sub: o.sub ? 'What arrived this month' : null, body: h('p', null, 'Big Buck Bunny, Sintel and Tears of Steel.') })
      : card({ title: o.flush ? 'Devices' : 'Recently added', sub: o.sub ? 'What arrived this month' : null, cls: o.flush ? 'fui-card--flush' : '',
        actions: o.action ? button({ size: 'sm', variant: 'ghost' }, 'Everything in it', icon('chevronRight', 13)) : null,
        body: o.flush ? h('ul', { class: 'fui-card__demo-list' }, ['Living room TV', 'Phone', 'Laptop'].map((d) => h('li', null, d))) : h('p', null, 'Big Buck Bunny, Sintel and Tears of Steel.') }),
  },
};
