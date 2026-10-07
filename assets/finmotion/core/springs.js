// FinMotion: the springs. Pure: no page, no style. Each spring is physics (a stiffness and a damping) simulated once and
// compiled to what CSS understands, a linear() curve and how long it takes to rest. core/springs.css carries the result (a
// test holds the two together), so a transition, a keyframe and a script all move on the same four feelings.

/** FinMotion's four springs: each a feeling, named, the same everywhere. */
export const SPRINGS = {
  settle: { stiffness: 170, damping: 26, line: 'The default: arrives quickly and rests without a wobble you could name.' },
  snap: { stiffness: 380, damping: 30, line: 'For what the hand does: toggles, presses, a knob meeting its end.' },
  drift: { stiffness: 110, damping: 13, line: 'For what lands: a card dealt, a stamp, a bookmark falling into place.' },
  glide: { stiffness: 90, damping: 22, line: 'For the large and the slow: sheets, pages, a morph across the screen.' },
};

/** A damped spring from 0 to 1, simulated at 120 Hz until it rests: its samples and how long it took. */
export function simulate({ stiffness, damping, mass = 1 }) {
  const dt = 1 / 120, samples = [];
  let x = 0, v = 0, t = 0;
  for (;;) {
    const a = (-stiffness * (x - 1) - damping * v) / mass;
    v += a * dt; x += v * dt; t += dt;
    samples.push(x);
    if ((Math.abs(x - 1) < 0.002 && Math.abs(v) < 0.02) || t > 3) break;   // the rest is a tail nobody sees
  }
  return { samples, ms: Math.round(t * 1000) };
}

/** A spring as CSS: linear() with enough points to keep its overshoot, and its duration in ms. */
export function springOf(spec) {
  const { samples, ms } = simulate(spec);
  const n = 48, pts = [0];
  for (let i = 1; i < n; i++) {
    const at = (i / n) * (samples.length - 1), lo = Math.floor(at);   // evenly in time: between two samples, not the one below
    pts.push(+(samples[lo] + (samples[lo + 1] - samples[lo]) * (at - lo)).toFixed(4));
  }
  pts.push(1);
  return { curve: `linear(${pts.join(', ')})`, ms, samples };
}

/** One step of a spring chasing a target: the next position and velocity after `dt` seconds. A follower steps this every
 *  frame, so a new target bends a movement in flight instead of waiting for it to end. */
export function step({ stiffness, damping, mass = 1 }, x, v, goal, dt) {
  for (; dt > 0; dt -= 1 / 240) {
    const h = Math.min(dt, 1 / 240);
    v += ((-stiffness * (x - goal) - damping * v) / mass) * h; x += v * h;
  }
  return [x, v];
}
