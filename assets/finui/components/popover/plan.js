// FinUI: popover, the rules that are not drawing: where a box opens beside what it belongs to: on the side asked for,
// lined up with its start, its end or its middle, on the other side when there is no room, and held inside the window.
// Pure; tested in FinUI's repository.

const OPPOSITE = { bottom: 'top', top: 'bottom', right: 'left', left: 'right' };

/** Where a box of `size` opens beside `anchor` (a DOMRect) in a window of `view`. Answers { left, top, side }. */
export function beside(anchor, size, view, { side = 'bottom', align = 'start', gap = 6, margin = 8 } = {}) {
  const fits = (s) => (s === 'bottom' ? anchor.bottom + gap + size.h <= view.h - margin
    : s === 'top' ? anchor.top - gap - size.h >= margin
    : s === 'right' ? anchor.right + gap + size.w <= view.w - margin
    : anchor.left - gap - size.w >= margin);
  const at = fits(side) || !fits(OPPOSITE[side]) ? side : OPPOSITE[side];
  const across = (start, end, length, room) => {
    const v = align === 'end' ? end - length : align === 'center' ? start + (end - start - length) / 2 : start;
    return Math.round(Math.max(margin, Math.min(room - length - margin, v)));
  };
  if (at === 'bottom' || at === 'top') {
    return { left: across(anchor.left, anchor.right, size.w, view.w), top: at === 'bottom' ? anchor.bottom + gap : anchor.top - gap - size.h, side: at };
  }
  return { left: at === 'right' ? anchor.right + gap : anchor.left - gap - size.w, top: across(anchor.top, anchor.bottom, size.h, view.h), side: at };
}
