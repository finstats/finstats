// FinMotion: how things move, from script, on the same four springs CSS moves on (--spring-settle, -snap, -drift,
// -glide in core/springs.css), read from the page, so FinUI's Motion choice and the device's reduced motion reach a script
// exactly as they reach a transition. Nothing here moves by itself; a part asks.

import { SPRINGS, step } from './springs.js';

export { SPRINGS };

const seconds = (t) => { const s = String(t).trim(); const n = parseFloat(s); return Number.isFinite(n) ? (s.endsWith('ms') ? n : n * 1000) : 0; };
const reducedHere = () => typeof matchMedia === 'function' && matchMedia('(prefers-reduced-motion: reduce)').matches;

/** What FinUI's Motion choice says, as a pace for every spring: its --ease against the 150 ms FinUI ships with (Quick is
 *  quicker, Slow slower, Off 0). Under no FinUI at all, FinMotion keeps its own pace, 1. */
export function paceOf(ease) {
  const t = String(ease).trim().split(/\s+/)[0];
  if (!t) return 1;
  return seconds(t) / 150;
}

/** Sets the page's pace from FinUI's --ease: every spring's time is multiplied by it (core/springs.css). */
export function pace(root = document.documentElement) {
  root.style.setProperty('--fm-pace', String(paceOf(getComputedStyle(root).getPropertyValue('--ease'))));
}

/** What leaves gets out of the way: a third of its spring's time, on a curve that starts slowly and ends fast. */
const LEAVE = { part: 1 / 3, easing: 'cubic-bezier(.4, 0, 1, 1)' };

/** A spring's timing as `style` (a computed style) gives it: `{ duration, easing }` in ms, for Element.animate. Still
 *  (duration 0) when the preset's Motion is Off or the device asks for reduced motion. `leaving`: the same spring for
 *  something on its way out. */
export function timing(style, name, { reduced = false, leaving = false } = {}) {
  if (!SPRINGS[name]) throw new Error(`FinUI has no spring called ${name}`);
  const full = reduced ? 0 : seconds(style.getPropertyValue(`--spring-${name}-dur`));
  if (leaving) return { duration: Math.round(full * LEAVE.part), easing: LEAVE.easing };
  return { duration: Math.round(full), easing: style.getPropertyValue(`--spring-${name}-curve`).trim() || 'ease' };
}

/** The spring `name` as it stands for `el` (a preset may be scoped to part of a page). An element not yet in the page has
 *  no style to read (it would read as still), so the page's own springs stand in for it. */
export const spring = (name = 'settle', el = document.documentElement, { leaving = false } = {}) =>
  timing(getComputedStyle(el && el.isConnected ? el : document.documentElement), name, { reduced: reducedHere(), leaving });

/** The time between one thing and the next of a row arriving one after another, on `name`. */
export const gap = (name = 'snap', el = document.documentElement) => spring(name, el).duration / 24;

/** Nothing should move: reduced motion, or Motion: Off. */
export const still = (el = document.documentElement) => spring('settle', el).duration === 0;

/** Plays keyframes on `el` on a spring and answers the Animation. Still, it is a jump to the last frame. Play an element
 *  once it is in the page: one cloned from a template and not yet placed belongs to an inert document, where nothing runs. */
export function play(el, frames, name = 'settle', extra = {}) {
  const { leaving, ...rest } = extra;
  return el.animate(frames, { ...spring(name, el, { leaving }), fill: 'both', ...rest });
}

/** Resolves when `a` has finished or been cancelled, or after `ms` and a little more: an animation that never runs (a
 *  hidden tab) still lets what waits on it go on. */
export const ended = (a, ms = a.effect ? a.effect.getComputedTiming().endTime : 0) => Promise.race([a.finished.catch(() => {}), new Promise((r) => setTimeout(r, Number(ms) + 80))]);

/** Every animation under `el` cancelled, so a replay starts from rest rather than on top of one in flight. */
export function settle(el) { for (const a of el.getAnimations({ subtree: true })) a.cancel(); }

/** Values that chase a target on a spring, stepped every frame: a new target bends the movement in flight rather than
 *  waiting for an animation to end, which is what a pointer needs. `apply(values)` paints; it is called on every frame. */
export function follower(start, apply, name = 'settle') {
  const x = { ...start }, v = Object.fromEntries(Object.keys(start).map((k) => [k, 0]));
  const goal = { ...start };
  let spec = SPRINGS[name], raf = 0, last = 0;
  const frame = (now) => {
    const dt = Math.min((now - last) / 1000, 1 / 30); last = now;
    let moving = false;
    for (const k in x) { [x[k], v[k]] = step(spec, x[k], v[k], goal[k], dt); if (Math.abs(x[k] - goal[k]) > 0.01 || Math.abs(v[k]) > 0.01) moving = true; }
    if (!moving) { Object.assign(x, goal); for (const k in v) v[k] = 0; }
    apply(x);
    raf = moving ? requestAnimationFrame(frame) : 0;
  };
  return {
    to(target, next) {
      Object.assign(goal, target); if (next) spec = SPRINGS[next];
      if (still()) { Object.assign(x, goal); apply(x); return; }
      if (!raf) { last = performance.now(); raf = requestAnimationFrame(frame); }
    },
    jump(values) { cancelAnimationFrame(raf); raf = 0; Object.assign(x, goal, values); Object.assign(goal, x); for (const k in v) v[k] = 0; apply(x); },
    stop() { cancelAnimationFrame(raf); raf = 0; },
    now: () => ({ ...x }),
  };
}

/** The first half of a FLIP: notes where `els` stand now, and answers a function that, called once they have moved (a
 *  sort FinUI did, a row taken out), plays each from where it was to where it is, on `name`. */
export function noted(els, name = 'settle') {
  const before = new Map([...els].map((el) => [el, el.getBoundingClientRect()]));
  return () => {
    const out = [];
    for (const [el, a] of before) {
      if (!el.isConnected) continue;
      const b = el.getBoundingClientRect(), dx = a.left - b.left, dy = a.top - b.top;
      if (dx || dy) out.push(play(el, [{ transform: `translate(${dx}px, ${dy}px)` }, { transform: 'none' }], name, { fill: 'none' }));
    }
    return out;
  };
}

/** Moves `els` from where they are to where `change()` puts them (First, Last, Invert, Play): a list re-sorted, a row
 *  making room. Elements `change()` adds are left to appear; elements it removes are not followed. */
export function flip(els, change, name = 'settle') {
  const go = noted(els, name);
  change();
  return go();
}

/** Deals `els` in one after another, like cards onto a table: each rises a little and arrives on `name`, `every` ms apart
 *  (a little of the spring, unless said). */
export function deal(els, { name = 'drift', every = null, rise = 10 } = {}) {
  const list = [...els], step = every ?? gap(name, list[0]) * 2;
  return list.map((el, i) => play(el, [{ opacity: 0, transform: `translateY(${rise}px)` }, { opacity: 1, transform: 'none' }], name, { delay: i * step, fill: 'backwards' }));
}

/** Runs `enter()` once, the first time `el` comes into view (a chart drawing itself, a number rolling up), and never
 *  when nothing should move. A page that draws something far below is not spent on an entrance nobody saw. */
export function whenSeen(el, enter) {
  if (typeof IntersectionObserver !== 'function') return;
  const io = new IntersectionObserver((es) => {
    if (!es.some((e) => e.isIntersecting)) return;
    io.disconnect();
    if (!still(el)) enter();
  });
  io.observe(el);
}

/** Draws `path` (in the page) in from its start, as a pen would, on `name`. */
export function pen(path, name = 'glide', extra = {}) {
  path.setAttribute('pathLength', '1');
  return play(path, [{ strokeDasharray: '1 1', strokeDashoffset: '1' }, { strokeDasharray: '1 1', strokeDashoffset: '0' }], name, { fill: 'none', ...extra });
}
