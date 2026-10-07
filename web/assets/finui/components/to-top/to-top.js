// FinUI: to-top. A round button in the bottom-right corner that takes a long page back to its top: it shows once the page
// has scrolled away from the top and goes when it is back, and rests clear of what else keeps that corner (a dock, a tab
// bar, a status bar), told by place(). Fixed to the window, so an app puts one in its frame and keeps it.

import { h, icon } from '../../core.js';
import { shown, rest } from './plan.js';

/**
 * toTop({ label, after, kept, onTop }): the button; answers { el, place(kept), destroy() }. after: how far down (px) the
 * page must be before it shows. kept: what keeps a part of the corner, [{ bottom, right }] in px (desktop-nav's and
 * mobile-nav's cornerOf, a status bar's height); place() says it again when the layout changes. onTop: called once the
 * page is sent up (where the focus should go, say).
 */
/** Nothing should move: reduced motion, or a preset's Motion: Off (an --ease of 0s). */
const still = (el) => matchMedia('(prefers-reduced-motion: reduce)').matches || parseFloat(getComputedStyle(el).getPropertyValue('--ease')) === 0;

export function toTop({ label = 'Scroll to top', after = 400, kept = [], onTop = null } = {}) {
  const el = h('button', { type: 'button', class: 'fui-to-top', 'aria-label': label, title: label }, icon('arrowUp', 16));
  el.addEventListener('click', () => {
    window.scrollTo({ top: 0, behavior: still(el) ? 'auto' : 'smooth' });
    if (onTop) onTop();
  });
  let queued = false;
  const look = () => { queued = false; el.classList.toggle('is-shown', shown(window.scrollY, after)); };
  const onScroll = () => { if (!queued) { queued = true; requestAnimationFrame(look); } };
  window.addEventListener('scroll', onScroll, { passive: true });
  const place = (corner = []) => {
    const at = rest(corner);
    el.style.setProperty('--fui-to-top-bottom', `${at.bottom}px`);
    el.style.setProperty('--fui-to-top-right', `${at.right}px`);
  };
  place(kept);
  look();
  return { el, place, destroy() { window.removeEventListener('scroll', onScroll); el.remove(); } };
}

export const meta = {
  name: 'to-top',
  purpose: 'Takes a long page back to its top from wherever it has been scrolled to.',
  use: 'Once, in the frame of an app whose pages run long. Tell it what keeps the bottom-right corner (the menus’ cornerOf, a status bar), and again when that changes, and it rests clear of them.',
  avoid: 'Short pages: it never shows. More than one on a screen. Inside something that scrolls on its own: it follows the window.',
  variants: ['clear of a dock, a tab bar, a thumb arc’s button, a status bar or a menu on the right'],
  states: ['hidden near the top', 'shown once scrolled past after', 'a jump rather than a glide with reduced motion or Motion: Off'],
  a11y: 'A button named by its label (aria-label and a tooltip). Hidden, it is out of the tab order and out of the accessibility tree.',
  props: {
    'toTop({ label, after, kept, onTop })': "label: 'Scroll to top'; after: 400 (px); kept: [{ bottom, right }]; onTop: after the page is sent up",
    'place(kept)': 'where it rests, again: the layout changed',
    'rest(kept, gap) in plan.js': 'that place in px from the bottom and the right, gap (16) beyond the most anything keeps',
  },
  playground: {
    controls: [
      { key: 'kept', label: 'Corner', choices: [['none', 'Nothing there'], ['bar', 'A status bar'], ['tabs', 'A tab bar'], ['arc', 'A thumb arc’s button']] },
    ],
    render: (o) => {
      const corner = { none: [], bar: [{ bottom: 26 }], tabs: [{ bottom: 64 }], arc: [{ bottom: 72 }] }[o.kept];
      const t = toTop({ kept: corner });
      t.el.classList.add('is-shown', 'fui-to-top--demo');
      return h('div', { class: 'fui-to-top__stage' }, t.el);
    },
  },
};
