// FinMotion: poster, the rules that are not drawing. Pure; tested in its QA.

const clamp = (v, lo, hi) => Math.max(lo, Math.min(hi, v));

/** A poster catching the light: with the pointer at (px, py) from its middle (−0.5 to 0.5 across and down), how far it
 *  turns (ry, about its upright) and tips (rx), at most `max` degrees, and where on it the sheen falls (x, y in %). */
export function lean(px, py, max = 9) {
  const x = clamp(px, -0.5, 0.5), y = clamp(py, -0.5, 0.5);
  return { ry: clamp(x * 2 * max, -max, max) + 0, rx: clamp(-y * 2 * max, -max, max) + 0, x: (x + 0.5) * 100, y: (y + 0.5) * 100 };
}
