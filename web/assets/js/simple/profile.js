// What the page of one library and of one person share in Simple mode: a header with its picture, its name and its tabs;
// boxed sections with a title bar; and the four global figures (today, the last 7 and 30 days, all time).

import { h, icon, num } from '../dom.js';
import { t } from '../i18n.js';
import { statFigure, statFigures, durationMeasure } from '../../finui/components/stat-figure/stat-figure.js';

/** The spans the global figures cover, as /api/stats/spans answers them: whole local days, or all time. */
export const SPANS = [[1, t('time.today')], [7, t('range.7.long')], [30, t('range.30.long')], [0, t('range.0.long')]];

/** The header: picture, name, and tabs as links ([{ label, href, current }]); the open one marked. */
export const profileHead = ({ picture, name, tabs }) => h('header', { class: 'simple-profile' }, h('div', { class: 'simple-profile__picture' }, picture),
  h('div', null, h('h1', { class: 'simple-profile__name' }, name),
    h('nav', { class: 'simple-tabs', 'aria-label': t('simple.profile.views') }, tabs.map((tab) => h('a', { href: tab.href, 'aria-current': tab.current ? 'page' : null }, tab.label)))));

/** A boxed section with its title bar. */
export const box = ({ id, title, icon: name, body }) => h('section', { class: 'simple-box', id, 'aria-label': title },
  h('h2', { class: 'simple-box__title' }, name ? icon(name, 20) : null, title), h('div', { class: 'simple-box__body' }, body));

const UNITS = { days: 'simple.profile.unit.days', hrs: 'simple.profile.unit.hrs', mins: 'simple.profile.unit.mins' };
/** durationMeasure's days, hrs and mins, in the language in use. */
export const duration = (sec) => durationMeasure(sec).map(([n, unit]) => [n, UNITS[unit] ? t(UNITS[unit]) : unit]);
/** "play" or "plays", the small word after a number. */
export const playsUnit = (n) => t(n === 1 ? 'simple.profile.unit.play' : 'simple.profile.unit.plays');

/** The four figures, from a row per span ({ plays, watch_s }), in SPANS' order. */
export const globalStats = (rows) => statFigures(SPANS.map(([, label], i) => {
  const r = rows[i] || {};
  return statFigure({ label, measures: [[[num(r.plays || 0), playsUnit(r.plays)]], duration(r.watch_s)] });
}));
