// FinMotion: toggle, the rules that are not drawing. Pure; tested in its QA.

/** A switch with weight: the knob's keyframes from one end to the other, stretched on the way and at rest at both ends.
 *  `travel` is how far the knob goes, in px. */
export function stretch(on, travel) {
  const from = on ? 0 : travel, to = on ? travel : 0;
  return [
    { transform: `translateX(${from}px) scaleX(1)` },
    { transform: `translateX(${from + (to - from) * 0.45}px) scaleX(1.45)`, offset: 0.4 },
    { transform: `translateX(${to}px) scaleX(1)` },
  ];
}
