// FinMotion: heatmap, the rules that are not drawing. Pure; tested in its QA.

/** A ripple from the cell pressed: every cell's distance from cell `at`, in cells, `cols` to a row: its ring. */
export function ripple(at, cols, n) {
  const x0 = at % cols, y0 = Math.floor(at / cols);
  return Array.from({ length: n }, (_, i) => Math.hypot((i % cols) - x0, Math.floor(i / cols) - y0));
}
