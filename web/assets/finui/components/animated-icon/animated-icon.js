// FinUI: animated icon. The same icon as icon() draws, moving: drawn in stroke by stroke, doing what it is about (motions.js
// says what that is for each one), drawn out, and again. At rest it is the static icon, so it can stand where one stands.

import { h, icon, iconNames } from '../../core.js';
import { plan, MOTIONS } from './motions.js';
import { button } from '../button/button.js';

/** What can be pressed: an icon inside one of these moves when it is pointed at or focused. */
export const PRESSABLE = 'a[href], button, summary, label, [role="button"], [role="link"], [role="tab"], [role="menuitem"], [role="option"], [role="switch"], [role="checkbox"], [role="radio"]';

/**
 * animate(svg, { play, label }): makes an icon() where it stands an animated one; an icon it does not know, or one already
 * animated, is left as it is. play: 'loop' (for ever), 'hover' (once each time it, or what holds it, is pointed at or
 * focused) or 'once' (drawn in, its act, and it stays). label: what it means, for an icon that stands alone; without one
 * it stays hidden from assistive technology.
 */
export function animate(svg, { play = 'loop', label = null } = {}) {
  const name = svg.dataset.icon;
  if (!name || !Object.hasOwn(MOTIONS, name) || svg.classList.contains('fui-animated-icon')) return svg;
  svg.classList.add('fui-animated-icon');
  if (play !== 'loop') svg.classList.add(`fui-animated-icon--${play}`);
  const shapes = [...svg.children];
  plan(name, shapes.length).forEach((p, i) => {
    const s = shapes[i];
    s.setAttribute('pathLength', '1');
    s.style.setProperty('--i', String(i));
    if (!p.act) return;
    s.dataset.act = p.act;
    s.style.setProperty('--k', String(p.k));
    s.style.setProperty('--dx', String(p.dx));
    s.style.setProperty('--dy', String(p.dy));
    s.style.setProperty('--dir', String(p.dir));
    s.style.setProperty('--sx', String(p.sx));
    if (p.origin) { s.style.transformBox = 'view-box'; s.style.transformOrigin = p.origin; }
  });
  if (label) { svg.removeAttribute('aria-hidden'); svg.setAttribute('role', 'img'); svg.setAttribute('aria-label', label); }
  return svg;
}

/** animatedIcon(name, { size, play, label, cls }): icon(name) animated; play and label as animate() takes them. */
export function animatedIcon(name, { size = 16, play = 'loop', label = null, cls = '' } = {}) {
  return animate(icon(name, size, cls), { play, label });
}

/**
 * animateWithin(root, { play }): every icon inside something pressable under root moves (once on hover, unless play says
 * otherwise), the ones there now and the ones that arrive later; an icon among words stays still. Returns a function that
 * stops looking for new ones.
 */
export function animateWithin(root, { play = 'hover' } = {}) {
  const sweep = (node) => {
    if (node.nodeType !== 1) return;
    const found = node.matches('svg.icon[data-icon]') ? [node] : node.querySelectorAll('svg.icon[data-icon]:not(.fui-animated-icon)');
    for (const svg of found) if (svg.parentElement && svg.parentElement.closest(PRESSABLE)) animate(svg, { play });
  };
  sweep(root);
  const watch = new MutationObserver((records) => { for (const r of records) for (const n of r.addedNodes) sweep(n); });
  watch.observe(root, { childList: true, subtree: true });
  return () => watch.disconnect();
}

/**
 * setBusy(svg, on): an icon that turns (refresh, recap, settings…) keeps turning while something is under way, pointed at
 * or not, and when it is over finishes the turn it is in, so it stops upright rather than snapping back.
 */
export function setBusy(svg, on) {
  if (!svg) return;
  if (on) { svg.dataset.busy = '1'; svg.classList.add('is-busy'); return; }
  delete svg.dataset.busy;
  const stop = () => {
    if (svg.dataset.busy || !svg.classList.contains('is-busy')) return;
    // Still pointed at (it was pressed): that is not a new hover, so it rests until the pointer or the focus leaves.
    const holder = (svg.parentElement && svg.parentElement.closest(PRESSABLE)) || svg;
    if (holder.matches(':hover, :focus-visible')) {
      svg.classList.add('fui-animated-icon--rested');
      const wake = () => { svg.classList.remove('fui-animated-icon--rested'); holder.removeEventListener('pointerleave', wake); holder.removeEventListener('focusout', wake); };
      holder.addEventListener('pointerleave', wake);
      holder.addEventListener('focusout', wake);
    }
    svg.classList.remove('is-busy');
  };
  const turn = [...svg.querySelectorAll('[data-act="spin"]')].flatMap((p) => p.getAnimations()).find((a) => a.playState === 'running');
  if (!turn) { stop(); return; }
  turn.effect.target.addEventListener('animationiteration', stop, { once: true });
  setTimeout(stop, (Number(turn.effect.getTiming().duration) || 0) + 100);   // should the turn's end never be told
}

export const meta = {
  name: 'animated-icon',
  purpose: 'An icon that moves: drawn in, doing what it is about — refresh turns, download drops into its tray, a heart beats — and drawn out again.',
  use: 'On hover, once, to say a control is alive (animateWithin does it for every icon in anything pressable); busy, a refresh turning until its answer is in; looped, where something is happening and the icon says what. Every icon has one.',
  avoid: 'Beside text people read for long: something moving for ever pulls the eye. Several on one screen at once. Anything a spinner says better.',
  variants: ['loop', 'once on hover', 'once', 'busy (setBusy)', 'any size'],
  states: ['at rest it is the static icon', 'still with reduced motion', 'still with Motion: Off (--icon-cycle 0s)'],
  a11y: 'aria-hidden unless given a label, as a static icon is. Reduced motion and Motion: Off leave the static icon.',
  props: {
    'animatedIcon(name, { size, play, label, cls })': "name: any icon of core.js; play: 'loop' | 'hover' (once per hover) | 'once'; label: role=img with that name",
    'animate(svg, { play, label })': 'an icon() made animated where it stands',
    'animateWithin(root, { play })': "every icon in something pressable under root, now and later; play 'hover' unless told; returns a stop function",
    'setBusy(svg, on)': 'a turning icon turns on until told to stop, then ends its turn',
    '--icon-cycle': 'how long one cycle takes (the Motion choice of a preset sets it)',
  },
  playground: {
    controls: [
      { key: 'name', label: 'Icon', choices: iconNames().map((n) => [n, n]) },
      { key: 'play', label: 'Play', choices: [['loop', 'Loop'], ['hover', 'On hover'], ['once', 'Once']] },
      { key: 'size', label: 'Size', choices: [[48, 'Large'], [24, 'Medium'], [16, 'Small']] },
    ],
    render: (o) => h('div', { class: 'fui-animated-icon__demo' },
      animatedIcon(o.name, { size: o.size, play: o.play, label: o.name }),
      button({}, animatedIcon(o.name, { play: o.play }), o.name)),
  },
};
