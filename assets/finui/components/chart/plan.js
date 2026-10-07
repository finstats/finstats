// FinUI: chart, the arithmetic every chart shares: round steps on an axis, a value's place along it, stacks, the path
// of a line and of a ring's slice, how busy a heat cell is, and numbers said compactly. Pure; tested in FinUI's repository.

const dust = (n) => Number(n.toPrecision(12));
/** Round ticks from 0 (or below) covering `min`…`max`, about `count` of them: steps of 1, 2, 2.5 or 5 times a power of ten. */
export function niceTicks(min, max, count = 4) {
  if (!(max > min)) return [min, min + 1];
  const raw = (max - min) / Math.max(1, count);
  const pow = 10 ** Math.floor(Math.log10(raw));
  const step = [1, 2, 2.5, 5, 10].map((m) => m * pow).find((s) => s >= raw);
  const out = [];
  for (let v = Math.floor(min / step) * step; v < max + step * 0.999; v += step) out.push(dust(v));
  return out;
}
/** A scale from `domain` to `range`. */
export const linear = ([d0, d1], [r0, r1]) => (v) => (d1 === d0 ? r0 : r0 + ((v - d0) / (d1 - d0)) * (r1 - r0));
/** For series of values (series × index), where each part of each stack starts and ends. */
export function stack(series) {
  const n = Math.max(0, ...series.map((s) => s.length));
  const base = Array(n).fill(0);
  return series.map((s) => s.map((v, i) => { const from = base[i]; base[i] += Math.max(0, v || 0); return { from, to: base[i] }; }));
}
const r2 = (n) => Math.round(n * 100) / 100;
/** The path through `points` [[x, y], …]. */
export const linePath = (points) => points.map(([x, y], i) => `${i ? 'L' : 'M'}${r2(x)} ${r2(y)}`).join('');
/** A slice of a ring centred at cx, cy between radii `inner` and `outer`, from angle a0 to a1 (radians, 0 at the top). */
export function slice(cx, cy, outer, inner, a0, a1) {
  const at = (r, a) => [r2(cx + r * Math.sin(a)), r2(cy - r * Math.cos(a))];
  const large = a1 - a0 > Math.PI ? 1 : 0;
  const [x0, y0] = at(outer, a0), [x1, y1] = at(outer, a1), [x2, y2] = at(inner, a1), [x3, y3] = at(inner, a0);
  return `M${x0} ${y0}A${outer} ${outer} 0 ${large} 1 ${x1} ${y1}L${x2} ${y2}A${inner} ${inner} 0 ${large} 0 ${x3} ${y3}Z`;
}
/** How busy a cell is, 0 (nothing) to `steps`: anything at all is at least the first step. */
export const level = (v, max, steps = 6) => (!(v > 0) || !(max > 0) ? 0 : Math.max(1, Math.min(steps, Math.round((v / max) * steps))));
const whole = new Intl.NumberFormat('en-US');
/** 1,284 · 12.9K · 4.2M */
export function compact(n) {
  const a = Math.abs(n);
  if (a < 10000) return whole.format(Math.round(n));
  const [d, u] = a >= 1e9 ? [1e9, 'B'] : a >= 1e6 ? [1e6, 'M'] : [1e3, 'K'];
  const v = n / d;
  return `${Math.abs(v) >= 100 ? Math.round(v) : Number(v.toFixed(1))}${u}`;
}
