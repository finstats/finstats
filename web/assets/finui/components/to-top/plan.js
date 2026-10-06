// FinUI: to-top's rules, apart from any page. When the button shows, and where it rests: in the bottom-right corner, clear
// of whatever else keeps a part of it (a menu's cornerOf, a status bar). Pure, so a test reads it without a page.

/** The room it leaves between itself and the edge, or what is there. */
export const GAP = 16;

/** Whether it shows: once the page has scrolled further than `after` px, so it is there only when the top is far. */
export const shown = (scrollY, after = 400) => scrollY > after;

/** Where it rests, in px from the window's bottom and right: `gap` beyond the most that anything in `kept` holds of the
 *  corner. kept: [{ bottom, right }], each how far up and in it reaches (a missing side is none). */
export function rest(kept = [], gap = GAP) {
  const most = (side) => Math.max(0, ...kept.filter(Boolean).map((k) => Number(k[side]) || 0));
  return { bottom: most('bottom') + gap, right: most('right') + gap };
}
