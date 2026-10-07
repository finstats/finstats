// FinUI: progress. How far along something is that is happening now (a download, an import, a scan): a bar or a ring,
// the share in words, and the time left once there is a rate to work it out from. With no share known, it says it is
// working rather than inventing one.

import { h, s } from '../../core.js';
import { remaining, timeLeft } from './plan.js';

/** progressBar({ label, value, elapsed, detail, tone }): `value` 0–1, or null for "working, no share known"; `elapsed`
 *  seconds so far, for the time left; `detail` says what (8 of 12 files). */
export function progressBar({ label, value = null, elapsed = null, detail = null, tone = 'accent' }) {
  const known = value != null;
  const left = known && elapsed != null ? timeLeft(remaining({ fraction: value, elapsed })) : null;
  const fill = h('span', { class: 'fui-progress__fill' });
  if (known) fill.style.width = `${Math.round(Math.min(1, Math.max(0, value)) * 100)}%`;
  return h('div', { class: ['fui-progress', !known && 'is-working', `fui-progress--${tone}`] },
    h('div', { class: 'fui-progress__head' }, h('span', { class: 'fui-progress__label' }, label), h('span', { class: 'fui-progress__share' }, known ? `${Math.round(value * 100)}%` : 'Working…')),
    h('div', { class: 'fui-progress__track', role: 'progressbar', 'aria-label': label, 'aria-valuemin': known ? '0' : null, 'aria-valuemax': known ? '100' : null,
      'aria-valuenow': known ? String(Math.round(value * 100)) : null, 'aria-valuetext': known ? [`${Math.round(value * 100)}%`, left].filter(Boolean).join(', ') : 'working' }, fill),
    detail || left ? h('div', { class: 'fui-progress__foot' }, detail ? h('span', null, detail) : h('span'), left ? h('span', null, left) : null) : null);
}
/** progressRing({ label, value, size }): the same share as a ring, for a small place (a card's corner, a button). */
export function progressRing({ label, value = null, size = 40 }) {
  const known = value != null, r = 16, c = 2 * Math.PI * r;
  const arc = s('circle', { class: 'fui-progress__arc', cx: '20', cy: '20', r: String(r), 'stroke-dasharray': `${c.toFixed(2)}`, 'stroke-dashoffset': known ? `${(c * (1 - value)).toFixed(2)}` : `${(c * 0.7).toFixed(2)}` });
  const svg = s('svg', { class: 'fui-progress__ring-svg', viewBox: '0 0 40 40', width: String(size), height: String(size), 'aria-hidden': 'true' }, s('circle', { class: 'fui-progress__ring-track', cx: '20', cy: '20', r: String(r) }), arc);
  return h('span', { class: ['fui-progress__ring', !known && 'is-working'], role: 'progressbar', 'aria-label': label, 'aria-valuemin': known ? '0' : null, 'aria-valuemax': known ? '100' : null, 'aria-valuenow': known ? String(Math.round(value * 100)) : null },
    svg, known ? h('span', { class: 'fui-progress__ring-text' }, `${Math.round(value * 100)}`) : null);
}

export const meta = {
  name: 'progress',
  purpose: 'Shows how far along something happening now is, and the time left once there is a rate to tell it from.',
  use: 'A download, an import, a library read, a backup. elapsed lets it say the time left from the rate so far; detail says what is counted. A ring in a small place.',
  avoid: 'A measure that is not moving (meter: storage used, a share watched). A guess: with no share known it says Working, never a number.',
  variants: ['a bar', 'a ring', 'working (no share known)', 'with the time left', 'good or critical'],
  states: ['under way', 'working', 'done'],
  a11y: 'role="progressbar" with aria-valuenow and an aria-valuetext that says the time left; working leaves the value out, as the role asks.',
  props: { 'progressBar({ label, value, elapsed, detail, tone })': 'value 0–1 or null', 'progressRing({ label, value, size })': '' },
  playground: {
    controls: [
      { key: 'kind', label: 'Kind', choices: [['bar', 'A bar'], ['ring', 'A ring']] },
      { key: 'working', label: 'No share known yet' },
      { key: 'eta', label: 'The time left', on: true },
    ],
    render: (o) => (o.kind === 'ring'
      ? h('div', { class: 'fui-progress__demo-ring' }, progressRing({ label: 'Importing', value: o.working ? null : 0.62 }), h('span', null, o.eta ? 'Importing · about 2 minutes left' : 'Importing'))
      : h('div', { class: 'fui-progress__demo' }, progressBar({ label: 'Importing from Jellystat', value: o.working ? null : 0.62, elapsed: o.eta ? 190 : null, detail: '7,940 of 12,804 plays' }))),
  },
};
