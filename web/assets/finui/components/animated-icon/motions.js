// FinUI: what each icon does when it moves. An icon is drawn in stroke by stroke, does what it is about — refresh turns,
// download drops into its tray, a heart beats — and is drawn out again; the drawing is the same for all, the act is the
// icon's own. A motion names the act, the shapes that do it (by their place in the icon, all of them when left out), the
// way they go (dx, dy: -1, 0 or 1; dir: 1 or -1 for a turn; sx: how far a stretch goes) and the point they turn about (in the icon's 24×24 box;
// left out, each shape turns about its own centre). The shapes of a motion move as one; `stagger` has them take turns, a
// bar after a bar. Pure, so a test reads it without a page.

/** Every act there is; animated-icon.css has the keyframes of each. */
export const ACTS = ['spin', 'exit', 'nudge', 'beat', 'flicker', 'twinkle', 'swing', 'shake', 'bob', 'pop', 'redraw', 'grow', 'blink', 'slide', 'squeeze', 'turn', 'flip', 'orbit', 'sweep', 'lift', 'stretch'];

const C = [12, 12];
export const MOTIONS = {
  home: [{ act: 'bob', parts: [0], dy: -1 }],
  activity: [{ act: 'redraw' }],
  users: [{ act: 'bob', parts: [0, 2], dy: -1, stagger: true }],
  user: [{ act: 'bob', parts: [0], dy: -1 }],
  library: [{ act: 'swing', parts: [2], origin: [18, 20] }],
  play: [{ act: 'nudge', dx: 1 }],
  pause: [{ act: 'bob', dy: 1, stagger: true }],
  // Each knob slides 3 units, and the line either side of it stretches from its far end so the gap goes with the knob.
  sliders: [
    { act: 'stretch', parts: [0], sx: 0.7, origin: [4, 6] }, { act: 'stretch', parts: [1], sx: 2.5, origin: [20, 6] }, { act: 'slide', parts: [2], dx: -1 },
    { act: 'stretch', parts: [3], sx: 2.5, origin: [4, 12] }, { act: 'stretch', parts: [4], sx: 0.7, origin: [20, 12] }, { act: 'slide', parts: [5], dx: 1 },
    { act: 'stretch', parts: [6], sx: 0.7, origin: [4, 18] }, { act: 'stretch', parts: [7], sx: 2.5, origin: [20, 18] }, { act: 'slide', parts: [8], dx: -1 },
  ],
  log: [{ act: 'redraw', parts: [2, 3], stagger: true }],
  settings: [{ act: 'spin', parts: [0], origin: C }],
  search: [{ act: 'orbit' }],
  plus: [{ act: 'turn', parts: [0, 1], origin: C }],
  x: [{ act: 'turn', origin: C }],
  check: [{ act: 'redraw' }],
  copy: [{ act: 'nudge', parts: [0], dx: -1, dy: -1 }],
  menu: [{ act: 'squeeze', stagger: true }],
  arrowUp: [{ act: 'exit', dy: -1 }],
  trendUp: [{ act: 'redraw', parts: [0] }, { act: 'nudge', parts: [1], dx: 1, dy: -1 }],
  trendDown: [{ act: 'redraw', parts: [0] }, { act: 'nudge', parts: [1], dx: 1, dy: 1 }],
  minus: [{ act: 'squeeze' }],
  chevronDown: [{ act: 'nudge', dy: 1 }],
  chevronRight: [{ act: 'nudge', dx: 1 }],
  chevronLeft: [{ act: 'nudge', dx: -1 }],
  table: [{ act: 'redraw', parts: [1, 2, 3], stagger: true }],
  chart: [{ act: 'grow', parts: [0, 1, 2], stagger: true }],
  upload: [{ act: 'exit', parts: [0, 1], dy: -1 }],
  together: [{ act: 'bob', parts: [0, 4], dy: -1 }],
  share: [{ act: 'pop', parts: [0, 1, 2], stagger: true }],
  download: [{ act: 'exit', parts: [0, 1], dy: 1 }],
  inbox: [{ act: 'bob', parts: [0], dy: 1 }],
  plug: [{ act: 'nudge', parts: [0, 1, 2], dy: -1 }],
  alert: [{ act: 'shake', parts: [0] }, { act: 'blink', parts: [1, 2] }],
  info: [{ act: 'redraw', parts: [1] }, { act: 'pop', parts: [2] }],
  logout: [{ act: 'exit', parts: [1, 2], dx: 1 }],
  trash: [{ act: 'lift', parts: [0, 1], origin: [4, 7] }],
  refresh: [{ act: 'spin', origin: C }],
  film: [{ act: 'blink', parts: [3, 5, 4, 6], stagger: true }],
  database: [{ act: 'bob', dy: -1, stagger: true }],
  link: [{ act: 'nudge', parts: [0], dx: 1, dy: -1 }, { act: 'nudge', parts: [1], dx: -1, dy: 1 }],
  shield: [{ act: 'beat' }],
  server: [{ act: 'blink', parts: [2, 3], stagger: true }],
  bookmark: [{ act: 'nudge', dy: 1 }],
  heart: [{ act: 'beat' }],
  globe: [{ act: 'flip', parts: [2, 3], origin: C }],
  lan: [{ act: 'redraw', parts: [3, 4], stagger: true }],
  external: [{ act: 'exit', parts: [0, 1], dx: 1, dy: -1 }],
  skip: [{ act: 'nudge', dx: 1 }],
  stop: [{ act: 'beat' }],
  volume: [{ act: 'blink', parts: [1, 2], stagger: true }],
  captions: [{ act: 'redraw', parts: [1, 2, 3, 4], stagger: true }],
  tv: [{ act: 'swing', parts: [1], origin: [12, 7] }],
  tag: [{ act: 'swing', origin: [7.5, 7.5] }],
  recap: [{ act: 'spin', dir: -1, origin: C }],
  cpu: [{ act: 'pop', parts: [1] }, { act: 'blink', parts: [2, 3, 4, 5, 6, 7, 8, 9], stagger: true }],
  clock: [{ act: 'spin', parts: [1], origin: C }],
  github: [{ act: 'swing', parts: [1], origin: [9, 18] }],
  calendar: [{ act: 'bob', parts: [2, 3], dy: -1, stagger: true }],
  flame: [{ act: 'flicker' }],
  repeat: [{ act: 'nudge', parts: [0], dx: 1 }, { act: 'nudge', parts: [2], dx: -1 }],
  sparkle: [{ act: 'twinkle' }],
  trophy: [{ act: 'swing', origin: [12, 21] }],
  compass: [{ act: 'spin', parts: [1], origin: C }],
  monitor: [{ act: 'redraw', parts: [0] }],
  layers: [{ act: 'bob', parts: [0], dy: -1 }],
  sun: [{ act: 'spin', parts: [1, 2, 3, 4, 5, 6, 7, 8], origin: C }, { act: 'beat', parts: [0] }],
  moon: [{ act: 'swing', origin: C }],
  gauge: [{ act: 'sweep', parts: [1], origin: [12, 14] }],
  lock: [{ act: 'shake', parts: [1] }],
  unlock: [{ act: 'nudge', parts: [1], dy: -1 }],
  shuffle: [{ act: 'nudge', parts: [1, 4], dx: 1 }],
  anchor: [{ act: 'swing', origin: [12, 5] }],
};

/** Each of the `count` shapes of icon `name`, in order: { act, k, dx, dy, dir, sx, origin } — act null for a shape that is
 *  only drawn; k is its place among the shapes of its act, for a group that takes turns; origin a CSS transform-origin,
 *  or null for the shape's own centre. */
export function plan(name, count) {
  const motions = MOTIONS[name];
  if (!motions) throw new Error(`no motion for the icon ${name}`);
  const parts = Array.from({ length: count }, () => ({ act: null, k: 0, dx: 0, dy: 0, dir: 1, sx: 1, origin: null }));
  for (const m of motions) {
    const which = m.parts || parts.map((_, i) => i);
    // Parts move as one unless the motion says they take turns: a staggered refresh comes apart as it spins.
    which.forEach((i, k) => {
      parts[i] = { act: m.act, k: m.stagger ? k : 0, dx: m.dx || 0, dy: m.dy || 0, dir: m.dir || 1, sx: m.sx || 1, origin: m.origin ? `${m.origin[0]}px ${m.origin[1]}px` : null };
    });
  }
  return parts;
}
