// FinUI: timeline. What happened, newest first, along a line: each event its time, what it was and a line more, marked
// by its kind; by day, under each day's name, when there are many.

import { h, icon } from '../../core.js';
import { byDay } from './plan.js';

const dayName = new Intl.DateTimeFormat('en-GB', { weekday: 'long', day: 'numeric', month: 'long' });
const clock = new Intl.DateTimeFormat('en-GB', { hour: '2-digit', minute: '2-digit' });
/** timeline({ events: [{ at, title, detail, icon, tone }], days }): `at` in ms; `tone` 'good', 'warning', 'critical'. */
export function timeline({ events, days = true }) {
  const item = (e) => h('li', { class: ['fui-timeline__event', e.tone && `fui-timeline__event--${e.tone}`] },
    h('span', { class: 'fui-timeline__mark', 'aria-hidden': 'true' }, e.icon ? icon(e.icon, 13) : null),
    h('div', { class: 'fui-timeline__words' },
      h('div', { class: 'fui-timeline__line' }, h('span', { class: 'fui-timeline__title' }, e.title), h('time', { class: 'fui-timeline__time', dateTime: new Date(e.at).toISOString() }, clock.format(e.at))),
      e.detail ? h('p', { class: 'fui-timeline__detail' }, e.detail) : null));
  if (!days) return h('ol', { class: 'fui-timeline' }, [...events].sort((a, b) => b.at - a.at).map(item));
  return h('div', { class: 'fui-timeline__days' }, byDay(events).map((d) => h('section', { class: 'fui-timeline__day' },
    h('h3', { class: 'fui-timeline__day-name' }, dayName.format(new Date(`${d.day}T12:00:00`))), h('ol', { class: 'fui-timeline' }, d.events.map(item)))));
}

const T = (d, hh, mm) => new Date(2026, 9, d, hh, mm).getTime();
const EVENTS = [
  { at: T(5, 21, 4), title: 'alice started Sintel', detail: 'Living room TV · Direct play', icon: 'play' },
  { at: T(5, 20, 31), title: 'Low Orbit S2E5 arrived', detail: '1080p HEVC · 1.2 GB', icon: 'download', tone: 'good' },
  { at: T(4, 23, 12), title: 'Sonarr stopped answering', detail: 'Coming up shows what it knew at 22:58', icon: 'alert', tone: 'warning' },
  { at: T(4, 19, 2), title: 'bob finished Big Buck Bunny', detail: 'Phone · Transcode', icon: 'check' },
];
export const meta = {
  name: 'timeline',
  purpose: 'Shows what happened, newest first, along a line: when, what, and a line more.',
  use: 'An activity feed, a play’s pauses and seeks, a server’s log in brief. days folds the events under each day’s name; tone marks what went well or wrong.',
  avoid: 'A table of many columns to sort (data-table). Steps still to do (steps).',
  variants: ['by day', 'one list', 'with icons', 'good, warning, critical'],
  states: [],
  a11y: 'Ordered lists, each under its day’s heading; every time is a <time> with its moment; a tone is said in the words, the mark only repeats it.',
  props: { 'timeline({ events, days })': 'events: [{ at, title, detail, icon, tone }]' },
  playground: {
    controls: [
      { key: 'days', label: 'By day', on: true },
      { key: 'icons', label: 'Icons', on: true },
      { key: 'detail', label: 'A line more', on: true },
    ],
    render: (o) => h('div', { class: 'fui-timeline__demo' }, timeline({ days: o.days, events: EVENTS.map((e) => ({ ...e, icon: o.icons ? e.icon : null, detail: o.detail ? e.detail : null })) })),
  },
};
