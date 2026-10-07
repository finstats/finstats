// FinUI: context-menu, the rules that are not drawing. Where the menu opens beside the point that was pressed (and so
// the point it grows out of), moving through it with the keys, finding an item by typing, and when a touch is a
// long-press. Pure; tested in FinUI's repository.

/** How long a touch must be held still to ask for the menu, and how far a finger may drift and still be still. */
export const LONG_PRESS_MS = 500;
export const SLOP_PX = 10;

/** Where a menu of `size` opens for a press at `at` in a window of `view`: below and to the right of the point, else
 *  on the side that has room, else held `gap` inside the window. `origin` is the point within the menu that it grows
 *  out of: the pressed point, or the nearest place to it the menu covers. */
export function place(at, size, view, gap = 8) {
  const axis = (p, len, room) => {
    if (p + len <= room - gap) return p;
    if (p - len >= gap) return p - len;
    return Math.max(gap, Math.min(p, room - len - gap));
  };
  const left = axis(at.x, size.w, view.w), top = axis(at.y, size.h, view.h);
  const ox = Math.max(0, Math.min(size.w, at.x - left)), oy = Math.max(0, Math.min(size.h, at.y - top));
  return { left, top, origin: `${ox}px ${oy}px` };
}

const choosable = (it) => !!it && !it.separator && !it.disabled;

/** The next item the arrows land on from `from` (−1: none yet) going `dir` (1 down, −1 up): separators and what
 *  cannot be chosen are stepped over, and it wraps. −1 when there is nothing to choose. */
export function step(items, from, dir) {
  const n = items.length;
  if (!items.some(choosable)) return -1;
  let i = from < 0 ? (dir > 0 ? -1 : n) : from;
  for (let k = 0; k < n; k++) {
    i = (i + dir + n) % n;
    if (choosable(items[i])) return i;
  }
  return -1;
}

/** Home and End: the first or the last item that can be chosen. */
export function edge(items, which) {
  return which === 'last' ? step(items, -1, -1) : step(items, -1, 1);
}

const fold = (s) => String(s || '').normalize('NFD').replace(/[̀-ͯ]/g, '').toLowerCase();

/** The next item after `from` whose label starts with what was typed (letters typed quickly are one word), wrapping
 *  round to `from` itself last; −1 when none can be chosen. Case and accents do not matter. */
export function typeahead(items, from, text) {
  const want = fold(text), n = items.length;
  if (!want) return -1;
  for (let k = 1; k <= n; k++) {
    const i = (Math.max(from, -1) + k + n) % n;
    if (choosable(items[i]) && fold(items[i].label).startsWith(want)) return i;
  }
  return -1;
}

/** Is a touch held this long, this far from where it began, a long-press? */
export function held({ ms, dx, dy }) {
  return ms >= LONG_PRESS_MS && Math.hypot(dx, dy) <= SLOP_PX;
}
