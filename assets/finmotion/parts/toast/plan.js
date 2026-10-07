// FinMotion: toast, the rules that are not drawing. Pure; tested in its QA.

/** Where each card of the hand stands (newest first; `heights` in px): its foot's offset `y` (upwards is negative), its
 *  turn, size, opacity and stacking. Piled, the older ones peek out above the newest, smaller and fainter; fanned
 *  (`open`), each stands clear of the one in front by `gap`, turned a little, as cards held in a hand. */
export function pose(heights, open, gap = 8) {
  const n = heights.length, mid = (n - 1) / 2;
  let foot = 0;
  return heights.map((hgt, i) => {
    const at = open ? foot : i * -9 || 0;
    foot -= hgt + gap;
    return {
      y: at, z: n - i,
      angle: open && n > 1 ? +((i - mid) * -2.5).toFixed(2) : 0,
      scale: open ? 1 : +(1 - i * 0.05).toFixed(3),
      opacity: open ? 1 : +Math.max(0, 1 - i * 0.25).toFixed(3),
    };
  });
}
