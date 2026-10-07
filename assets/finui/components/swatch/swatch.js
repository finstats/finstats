// FinUI: swatch. A small picture of a look: colours as a stack, as a day and a night half, as dots or as bars; a corner
// at a radius; a typeface saying Aa; an icon at a stroke. Shown beside a name, it says what choosing it would do.

import { h, icon } from '../../core.js';

const SHAPES = ['stack', 'halves', 'dots', 'bars'];
/** swatch(colours, { shape, label }): colours as `stack` (overlapping), `halves` (day and night), `dots` or `bars`.
 *  Hidden from screen readers unless it has a label: beside a name it repeats it. */
export function swatch(colours, { shape = 'stack', label = null } = {}) {
  const kind = SHAPES.includes(shape) ? shape : 'stack';
  return h('span', { class: ['fui-swatch', `fui-swatch--${kind}`], role: label ? 'img' : null, 'aria-label': label, 'aria-hidden': label ? null : 'true' },
    colours.map((c) => { const s = h('span'); s.style.background = c; return s; }));
}
/** A corner rounded as `radius` rounds it. */
export function cornerSwatch(radius) {
  const el = h('span', { class: 'fui-swatch fui-swatch--corner', 'aria-hidden': 'true' });
  el.style.borderTopLeftRadius = radius;
  return el;
}
/** Aa in a typeface. */
export function typeSwatch(family) {
  const el = h('span', { class: 'fui-swatch fui-swatch--type', 'aria-hidden': 'true' }, 'Aa');
  el.style.fontFamily = family;
  return el;
}
/** An icon drawn at a stroke width. */
export function strokeSwatch(width, name = 'sparkle') {
  const el = icon(name, 16, 'fui-swatch fui-swatch--stroke');
  el.style.strokeWidth = width;
  return el;
}

const COLOURS = ['var(--series-1)', 'var(--series-2)', 'var(--series-3)', 'var(--series-4)'];
export const meta = {
  name: 'swatch',
  purpose: 'Shows a look small: its colours, a corner, a typeface or an icon’s stroke, beside the name of what would give it.',
  use: 'In a picker of looks, palettes, radii, fonts: beside each option, so it is seen before it is chosen.',
  avoid: 'Saying anything on its own: a swatch is beside a name, or has a label. A chart’s legend (that is the chart’s own key).',
  variants: ['stack', 'halves', 'dots', 'bars', 'corner', 'type', 'stroke'],
  states: [],
  a11y: 'Hidden from screen readers (aria-hidden) beside a name it repeats; role="img" with an aria-label when it stands alone.',
  props: {
    'swatch(colours, { shape, label })': "shape: 'stack' | 'halves' | 'dots' | 'bars'",
    'cornerSwatch(radius)': '', 'typeSwatch(family)': '', 'strokeSwatch(width, name)': '',
  },
  playground: {
    controls: [{ key: 'shape', label: 'Shape', choices: [['stack', 'Stack'], ['halves', 'Halves'], ['dots', 'Dots'], ['bars', 'Bars'], ['corner', 'A corner'], ['type', 'A typeface']] }],
    render: (o) => h('span', { class: 'fui-swatch__demo' },
      o.shape === 'corner' ? cornerSwatch('var(--radius-control)') : o.shape === 'type' ? typeSwatch('var(--font-heading)')
        : swatch(o.shape === 'halves' ? COLOURS.slice(0, 2) : o.shape === 'stack' ? COLOURS.slice(0, 3) : COLOURS, { shape: o.shape }),
      h('span', null, 'Washi')),
  },
};
