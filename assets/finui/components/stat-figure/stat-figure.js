// FinUI: stat-figure. A figure as a classic stats page writes it: a label, then numbers large with their units small beside
// them, a slash between two measures of one thing ("21 plays / 9 hrs 14 mins").

import { h } from '../../core.js';

/** statFigure({ label, measures }): measures are [[[number, unit], …], …], one array per measure, divided by a slash. */
export function statFigure({ label, measures }) {
  const parts = [];
  measures.forEach((m, i) => {
    if (i) parts.push(h('span', { class: 'fui-stat-figure__slash', 'aria-hidden': 'true' }, '/'));
    for (const [n, unit] of m) parts.push(h('span', { class: 'fui-stat-figure__number' }, n), unit ? h('span', { class: 'fui-stat-figure__unit' }, unit) : null);
  });
  return h('div', { class: 'fui-stat-figure' }, h('div', { class: 'fui-stat-figure__label' }, label), h('div', { class: 'fui-stat-figure__value' }, parts));
}

/** statFigures(figures): a row of them, as many to a row as the width allows. */
export const statFigures = (figures) => h('div', { class: 'fui-stat-figure__row' }, figures);

/** A duration in seconds as the measure a figure takes: days, hrs and mins, the larger ones only when they are there. */
export function durationMeasure(sec) {
  const m = Math.floor(Math.max(0, Number(sec) || 0) / 60);
  const d = Math.floor(m / 1440), hr = Math.floor((m % 1440) / 60), min = m % 60;
  return [d ? [String(d), 'days'] : null, d || hr ? [String(hr), 'hrs'] : null, [String(min), 'mins']].filter(Boolean);
}

export const meta = {
  name: 'stat-figure',
  purpose: 'Writes a figure the way a classic stats page does: a label over large numbers, each with its small unit.',
  use: 'How much was watched over a span: today, the last week, all time; a row of them.',
  avoid: 'A figure with a trend or a chart beside it (stat-tile). A list of many figures (facts).',
  variants: ['one measure', 'two measures divided by a slash', 'in a row'],
  states: [],
  a11y: 'Text throughout; the slash between two measures is hidden from a screen reader, which reads the measures in turn.',
  props: { 'statFigure({ label, measures })': 'measures: [[[number, unit], …], …]', 'statFigures(figures)': 'a row', 'durationMeasure(sec)': '[[d, "days"], [h, "hrs"], [m, "mins"]]' },
  playground: {
    controls: [{ key: 'two', label: 'Two measures', on: true }],
    render: (o) => statFigures([['Today', 21, 33240], ['Last 7 days', 103, 166020], ['All time', 10443, 15398340]]
      .map(([label, plays, s]) => statFigure({ label, measures: o.two ? [[[String(plays), 'plays']], durationMeasure(s)] : [[[String(plays), 'plays']]] }))),
  },
};
