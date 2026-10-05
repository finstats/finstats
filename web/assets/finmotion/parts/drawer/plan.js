// FinMotion: drawer, the rules that are not drawing. Pure; tested in its QA.

/** A pull of `d` px past where a sheet can go, as the distance it gives: harder and harder, never as far as `limit`. */
export const band = (d, limit) => Math.sign(d) * (1 - 1 / ((Math.abs(d) * 0.55) / limit + 1)) * limit;

/** The pull that gives `b` (band's inverse): a sheet caught mid-spring is dragged on from where it is, not from rest. */
export const unband = (b, limit) => Math.sign(b) * (1 / (1 - Math.min(Math.abs(b), limit * 0.999) / limit) - 1) * limit / 0.55;

/** A bottom sheet let go `dy` px below where it rests (moving at `v` px/ms, down positive): 'close' when it was pulled a
 *  third of its `height` down or flicked down, 'stay' otherwise — up is never away. */
export function letGo(dy, height, v) {
  if (v < -0.3) return 'stay';
  return dy > height / 3 || (dy > 0 && v > 0.6) ? 'close' : 'stay';
}
