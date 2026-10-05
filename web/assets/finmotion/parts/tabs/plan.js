// FinMotion: tabs, the rules that are not drawing. Pure; tested in its QA.

/** An inchworm underline: going from tab `from` to tab `to`, the edge on the side it goes leads on snap and the other
 *  follows on settle, a moment later, so the line stretches and gathers rather than slides. null when it stays. */
export function inchworm(from, to) {
  if (from === to) return null;
  const lead = { spring: 'snap', lag: false }, trail = { spring: 'settle', lag: true };
  return to > from ? { left: trail, right: lead } : { left: lead, right: trail };
}
