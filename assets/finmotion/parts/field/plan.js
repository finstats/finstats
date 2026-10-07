// FinMotion: field, the rules that are not drawing. Pure; tested in its QA.

/** A gentle no: a head shaken the way a person does (away and back, each swing smaller than the last, at rest at the
 *  end), as keyframes, `wide` px at the first swing. */
export function shake(wide) {
  return [0, -1, 0.85, -0.65, 0.45, -0.25, 0.1, 0].map((k) => ({ transform: `translateX(${+(k * wide).toFixed(2)}px)` }));
}
