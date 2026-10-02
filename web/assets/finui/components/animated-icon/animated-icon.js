// FinUI: animated icon. The same icon as icon() draws, moving: drawn in stroke by stroke, doing what it is about (motions.js
// says what that is for each one), drawn out, and again. At rest it is the static icon, so it can stand where one stands.

import { h, icon, iconNames } from '../../core.js';
import { plan } from './motions.js';
import { button } from '../button/button.js';

/**
 * animatedIcon(name, { size, play, label, cls })
 * play: 'loop' (for ever), 'hover' (while it, or what holds it, is pointed at or focused) or 'once' (drawn in, its act, and
 * it stays). label: what it means, for an icon that stands alone; without one it is hidden from assistive technology.
 */
export function animatedIcon(name, { size = 16, play = 'loop', label = null, cls = '' } = {}) {
  const el = icon(name, size, ['fui-animated-icon', play !== 'loop' && `fui-animated-icon--${play}`, cls].filter(Boolean).join(' '));
  const shapes = [...el.children];
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
  if (label) { el.removeAttribute('aria-hidden'); el.setAttribute('role', 'img'); el.setAttribute('aria-label', label); }
  return el;
}

export const meta = {
  name: 'animated-icon',
  purpose: 'An icon that moves: drawn in, doing what it is about — refresh turns, download drops into its tray, a heart beats — and drawn out again.',
  use: 'Where something is happening and the icon says what (a refresh running, a download on its way), or on hover, to say a control is alive. Every icon has one.',
  avoid: 'Beside text people read for long: something moving for ever pulls the eye. Several on one screen at once. Anything a spinner says better.',
  variants: ['loop', 'on hover', 'once', 'any size'],
  states: ['at rest it is the static icon', 'still with reduced motion', 'still with Motion: Off (--icon-cycle 0s)'],
  a11y: 'aria-hidden unless given a label, as a static icon is. Reduced motion and Motion: Off leave the static icon.',
  props: {
    'animatedIcon(name, { size, play, label, cls })': "name: any icon of core.js; play: 'loop' | 'hover' | 'once'; label: role=img with that name",
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
