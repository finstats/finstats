// FinUI: meter. How far along something is, as a thin bar: a play, a job, a disk. Indeterminate when nobody knows yet.

import { h } from '../../core.js';

/** meter({ value, wide, block, indeterminate, label, class }): value from 0 to 1. A progressbar unless `label` is null. */
export function meter({ value = 0, wide = false, block = false, indeterminate = false, label = null, class: extra = null } = {}) {
  const pct = Math.max(0, Math.min(1, Number(value) || 0)) * 100;
  return h('span', { class: ['fui-meter', wide && 'fui-meter--wide', block && 'fui-meter--block', indeterminate && 'is-indeterminate', extra],
    role: label ? 'progressbar' : null, 'aria-label': label, 'aria-valuemin': label && !indeterminate ? 0 : null, 'aria-valuemax': label && !indeterminate ? 100 : null,
    'aria-valuenow': label && !indeterminate ? Math.round(pct) : null }, h('span', { class: 'fui-meter__fill', style: { width: (indeterminate ? 30 : pct) + '%' } }));
}

export const meta = {
  name: 'meter',
  purpose: 'Shows how far along something is, or how full, as a bar.',
  use: 'Beside a number it illustrates (how much of a play was watched), --wide under a running job, --block for a disk. Indeterminate while a job has not said how far it is.',
  avoid: 'The only statement of a value: the number is written beside it. A bar for several quantities at once (a chart).',
  variants: ['inline', 'wide', 'block', 'indeterminate'],
  states: ['indeterminate holds still with reduced motion'],
  a11y: 'role="progressbar" with aria-valuenow when it is given a label; without one it is decoration beside a number.',
  props: { 'meter({ value, wide, block, indeterminate, label })': 'value 0–1; label names it for screen readers' },
  playground: {
    controls: [
      { key: 'shape', label: 'Shape', choices: [['inline', 'Inline'], ['wide', 'Wide'], ['block', 'Block']] },
      { key: 'value', label: 'How far', choices: [[0.62, '62%'], [0.15, '15%'], [1, 'All']] },
      { key: 'unknown', label: 'Not known yet' },
    ],
    render: (o) => {
      const m = meter({ value: o.value, wide: o.shape === 'wide', block: o.shape === 'block', indeterminate: o.unknown, label: 'Library scan' });
      return o.shape === 'inline' ? h('span', { class: 'fui-meter__demo' }, m, o.unknown ? '…' : `${Math.round(o.value * 100)}%`) : m;
    },
  },
};
