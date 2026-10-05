// FinUI: slider, the rules that are not drawing — where a value lands, where a pointer puts it, what the keys do, and
// which of two thumbs a press moves. Pure; tested in FinUI's repository.

/** A number without the float dust of adding tenths: 0.30000000000000004 is 0.3. */
const tidy = (n, step) => { const d = (String(step).split('.')[1] || '').length; return Number(n.toFixed(Math.min(10, d))); };

/** The value nearest `v` on the steps from `min`, inside `min`…`max`. */
export function snap(v, { min, max, step }) {
  const s = step > 0 ? step : 1;
  // A hair over the half rounds up: 0.35 / 0.1 is 3.4999999999999996 in floating point.
  const on = min + Math.round((Math.min(max, Math.max(min, v)) - min) / s + 1e-9) * s;
  return tidy(Math.min(max, Math.max(min, on)), s);
}
/** Where a value is along the track, 0 to 1. */
export const fraction = (v, { min, max }) => (max > min ? (Math.min(max, Math.max(min, v)) - min) / (max - min) : 0);
/** The value under a pointer at `x` on a track at `rect`. */
export const fromPointer = (x, rect, o) => snap(o.min + ((x - rect.left) / Math.max(1, rect.width)) * (o.max - o.min), o);
/** What a key makes of `v`: a step, a page (a tenth of the track, at least a step), an end — or null, not the slider's. */
export function keyed(v, key, o) {
  const page = Math.max(o.step, (o.max - o.min) / 10);
  const to = { ArrowRight: v + o.step, ArrowUp: v + o.step, ArrowLeft: v - o.step, ArrowDown: v - o.step, PageUp: v + page, PageDown: v - page, Home: o.min, End: o.max }[key];
  return to === undefined ? null : snap(to, o);
}
/** Of two thumbs at `values`, the one a press at `v` moves: the nearer, and when they meet, the one that can go that way. */
export function nearest(values, v) {
  const [a, b] = values;
  if (a === b) return v > b ? 1 : 0;
  return Math.abs(v - a) <= Math.abs(v - b) ? 0 : 1;
}
