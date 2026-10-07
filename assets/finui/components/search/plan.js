// FinUI: the search's rules, apart from any page: the shape it grows out of (the control or the menu that opened it)
// and moving through the results. Pure, so a test reads it.

const px = (n) => `${+n.toFixed(2)}px`;

/** The place `h` clipped down to `t` (the control or the menu it grows out of) as far as `t` lies inside it, with the
 *  corners `r` that shape has: the first frame of a search opening, the last of one closing. */
export function cut(t, h, r = 24) {
  const c = (v, max) => Math.max(0, Math.min(max, v));
  return `inset(${px(c(t.y - h.y, h.h))} ${px(c(h.x + h.w - t.x - t.w, h.w))} ${px(c(h.y + h.h - t.y - t.h, h.h))} ${px(c(t.x - h.x, h.w))} round ${px(r)})`;
}

/** The result after `at` by `delta`, wrapping at both ends; -1 when there is nothing. */
export function step(at, delta, count) {
  if (!count) return -1;
  const from = Math.min(Math.max(at, 0), count - 1);
  return (((from + delta) % count) + count) % count;
}

/** Every row of every group, in order. */
export const flat = (groups) => groups.flatMap((g) => g.rows || []);
