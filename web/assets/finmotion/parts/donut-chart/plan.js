// FinMotion: donut-chart, the rules that are not drawing. Pure; tested in its QA.

/** A donut poured: each share's start and length as parts of one unit of time, one after another, the next starting
 *  before the last has quite settled (at 72% of it), the larger the longer — so a spring's duration stretches the whole. */
export function pour(values) {
  const sum = values.reduce((s, v) => s + v, 0) || 1;
  const lengths = values.map((v) => 0.35 + (v / sum) * 1.5);
  let at = 0;
  const raw = lengths.map((l) => { const x = { start: at, length: l }; at += l * 0.72; return x; });
  const end = Math.max(...raw.map((x) => x.start + x.length));
  return raw.map((x) => ({ start: x.start / end, length: x.length / end }));
}

/** The shares of a donut as FinUI drew it: each slice path's outer arc (its M point to its first A's end), clockwise from
 *  the top of a ring centred on (c, c), as a part of the whole ring. */
export function shares(paths, c) {
  const angle = (x, y) => { const a = Math.atan2(x - c, -(y - c)); return a < 0 ? a + Math.PI * 2 : a; };
  return paths.map((d) => {
    const n = (d.match(/-?[\d.]+/g) || []).map(Number);   // M x0 y0 A rx ry rot large sweep x1 y1 …
    if (n.length < 9) return 0;
    let a0 = angle(n[0], n[1]), a1 = angle(n[7], n[8]);
    if (a1 <= a0 + 1e-6) a1 += Math.PI * 2;
    return Math.min(1, (a1 - a0) / (Math.PI * 2));
  }).map((x) => (x > 0.999 ? 1 : x));
}
