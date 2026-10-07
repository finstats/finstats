// FinUI: slider. A number picked along a track (or a stretch of it, between two thumbs) by dragging, pressing the
// track, or the keys. Its value is said beside its name and to screen readers in words (format).

import { h, mount } from '../../core.js';
import { snap, fraction, fromPointer, keyed, nearest } from './plan.js';

/** slider({ label, min, max, step, value, onChange, format, marks, disabled, hideLabel }): `value` a number, or [from, to]
 *  for two thumbs. `format(v)` says a value ("30 min"); `marks`: values to name under the track; `hideLabel` shows only
 *  the value, for a row that names the slider already. */
export function slider({ label, min = 0, max = 100, step = 1, value = min, onChange = () => {}, format = (v) => String(v), marks = [], disabled = false, hideLabel = false }) {
  const o = { min, max, step };
  const two = Array.isArray(value);
  let values = (two ? value : [value]).map((v) => snap(v, o));
  const out = h('output', { class: 'fui-slider__value' });
  const fill = h('span', { class: 'fui-slider__fill' });
  const thumbs = values.map((_, i) => h('span', { class: 'fui-slider__thumb', role: 'slider', tabindex: disabled ? '-1' : '0', 'aria-label': two ? `${label}, ${i ? 'highest' : 'lowest'}` : label,
    'aria-valuemin': String(min), 'aria-valuemax': String(max), 'aria-disabled': disabled ? 'true' : null }));
  const track = h('div', { class: 'fui-slider__track' }, fill, thumbs);
  const el = h('div', { class: ['fui-slider', disabled && 'is-disabled'] },
    // In a row that names it already (a setting row), only its value shows; its thumbs keep the name for screen readers.
    h('div', { class: 'fui-slider__head' }, hideLabel ? null : h('span', { class: 'fui-slider__label' }, label), out), track,
    marks.length ? h('div', { class: 'fui-slider__marks', 'aria-hidden': 'true' }, marks.map((m) => { const s = h('span', null, format(m)); s.style.left = `${fraction(m, o) * 100}%`; return s; })) : null);

  function paint() {
    const f = values.map((v) => fraction(v, o));
    thumbs.forEach((t, i) => { t.style.left = `${f[i] * 100}%`; t.setAttribute('aria-valuenow', String(values[i])); t.setAttribute('aria-valuetext', format(values[i])); });
    fill.style.left = `${(two ? f[0] : 0) * 100}%`;
    fill.style.width = `${((two ? f[1] - f[0] : f[0])) * 100}%`;
    mount(out, two ? `${format(values[0])} – ${format(values[1])}` : format(values[0]));
  }
  /** Thumb `i` to `v`, never past the other one. */
  function set(i, v) {
    let next = snap(v, o);
    if (two) next = i === 0 ? Math.min(next, values[1]) : Math.max(next, values[0]);
    if (next === values[i]) return;
    values = values.map((x, k) => (k === i ? next : x));
    paint();
    onChange(two ? [...values] : values[0]);
  }
  if (!disabled) {
    track.addEventListener('pointerdown', (e) => {
      e.preventDefault();
      const v = fromPointer(e.clientX, track.getBoundingClientRect(), o);
      const i = two ? nearest(values, v) : 0;
      set(i, v); thumbs[i].focus();
      track.setPointerCapture(e.pointerId);
      const move = (ev) => set(i, fromPointer(ev.clientX, track.getBoundingClientRect(), o));
      const up = () => { track.removeEventListener('pointermove', move); track.removeEventListener('pointerup', up); };
      track.addEventListener('pointermove', move); track.addEventListener('pointerup', up);
    });
    thumbs.forEach((t, i) => t.addEventListener('keydown', (e) => { const v = keyed(values[i], e.key, o); if (v !== null) { e.preventDefault(); set(i, v); } }));
  }
  paint();
  return el;
}

export const meta = {
  name: 'slider',
  purpose: 'Picks a number along a track, or a stretch of it between two thumbs, by dragging, pressing or the keys.',
  use: 'A value whose place in a range matters more than its digits: a minimum play length, a quality, a span of years. format says the value as words; marks name points under the track.',
  avoid: 'An exact number somebody types (number-stepper). A choice of three named options (segmented).',
  variants: ['one thumb', 'two thumbs (a range)', 'with marks', 'disabled'],
  states: ['dragging', 'focus-visible', 'disabled'],
  a11y: 'Each thumb is role="slider" with aria-valuemin, -max, -now and aria-valuetext from format; arrows move a step, Page Up and Down a tenth, Home and End the ends.',
  props: { 'slider({ label, min, max, step, value, onChange, format, marks, disabled, hideLabel })': 'value: a number, or [from, to]; hideLabel in a row that names it' },
  playground: {
    controls: [
      { key: 'range', label: 'Two thumbs' },
      { key: 'marks', label: 'Marks under it' },
      { key: 'disabled', label: 'Unavailable' },
    ],
    render: (o) => h('div', { class: 'fui-slider__demo' }, slider({ label: o.range ? 'Release years' : 'Shortest play counted', min: o.range ? 1990 : 0, max: o.range ? 2030 : 30, step: o.range ? 1 : 1,
      value: o.range ? [2004, 2020] : 2, format: o.range ? String : (v) => (v ? `${v} min` : 'Every play'), marks: o.marks ? (o.range ? [1990, 2010, 2030] : [0, 10, 20, 30]) : [], disabled: o.disabled })),
  },
};
