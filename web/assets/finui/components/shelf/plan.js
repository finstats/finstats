// FinUI: shelf, the rules that are not drawing — where one glide of a shelf goes (a page of whole cards on or back,
// never past its ends) and whether it is at an end. Pure; tested in FinUI's repository.

/** Where a glide from `scrollLeft` goes `dir` (1 on, −1 back) in `box` { width, scrollWidth, card } (a card and its gap). */
export function target(scrollLeft, dir, box) {
  const page = Math.max(1, Math.floor(box.width / box.card)) * box.card;
  const max = box.scrollWidth - box.width;
  return Math.max(0, Math.min(max, scrollLeft + dir * page));
}
/** Is the shelf at its start, at its end? A pixel short counts as there. */
export const ends = (scrollLeft, box) => ({ start: scrollLeft <= 1, end: scrollLeft >= box.scrollWidth - box.width - 1 });
