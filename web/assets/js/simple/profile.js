// What the page of one library and of one person share in Simple mode: a header with its picture, its name and its tabs;
// boxed sections with a title bar; and the four global figures (today, the last 7 and 30 days, all time).

import { h, icon, num } from '../dom.js';
import { statFigure, statFigures, durationMeasure } from '../../finui/components/stat-figure/stat-figure.js';

/** The spans the global figures cover, as /api/libraries and /api/users count them: whole local days, or all time. */
export const SPANS = [[1, 'Today'], [7, 'Last 7 days'], [30, 'Last 30 days'], [0, 'All time']];

/** The header: picture, name, and tabs as links ([{ label, href, current }]); the open one marked. */
export const profileHead = ({ picture, name, tabs }) => h('header', { class: 'simple-profile' }, h('div', { class: 'simple-profile__picture' }, picture),
  h('div', null, h('h1', { class: 'simple-profile__name' }, name),
    h('nav', { class: 'simple-tabs', 'aria-label': 'Views' }, tabs.map((t) => h('a', { href: t.href, 'aria-current': t.current ? 'page' : null }, t.label)))));

/** A boxed section with its title bar. */
export const box = ({ id, title, icon: name, body }) => h('section', { class: 'simple-box', id, 'aria-label': title },
  h('h2', { class: 'simple-box__title' }, name ? icon(name, 20) : null, title), h('div', { class: 'simple-box__body' }, body));

/** The four figures, from a row per span ({ plays, watch_s }), in SPANS' order. */
export const globalStats = (rows) => statFigures(SPANS.map(([, label], i) => {
  const r = rows[i] || {};
  return statFigure({ label, measures: [[[num(r.plays || 0), r.plays === 1 ? 'play' : 'plays']], durationMeasure(r.watch_s)] });
}));
