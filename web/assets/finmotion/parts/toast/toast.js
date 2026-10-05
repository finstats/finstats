// FinMotion: toast. FinUI's notices are held like a hand of cards: piled, the newest in front and the older ones peeking
// out behind it, smaller and fainter; pointed at or focused, the hand fans open so each can be read. A notice that arrives
// rises into the pile; one dismissed drops away while the pile closes up.

import { play, ended, gap } from '../../core/motion.js';
import { pose } from './plan.js';

export const part = {
  name: 'toast',
  selector: '.fui-toast__region',
  enhance(region) {
    let open = false, shutting = 0;
    const known = new WeakSet();
    const cards = () => [...region.children].filter((c) => c.matches('.fui-toast') && !c.classList.contains('fm-toast--leaving'));
    // Each card moves from wherever it stands now — FinUI re-adds its cards on every paint, which a CSS transition does
    // not survive — to where the hand puts it.
    const lay = () => {
      const list = cards();
      pose(list.map((c) => c.offsetHeight), open).forEach((p, i) => {
        const c = list[i], cs = getComputedStyle(c);
        const from = known.has(c) ? { transform: cs.transform, opacity: cs.opacity } : { transform: 'translateY(46px)', opacity: '0' };
        const to = { transform: `translateY(${p.y}px) rotate(${p.angle}deg) scale(${p.scale})`, opacity: String(p.opacity) };
        c.getAnimations().forEach((a) => a.cancel());
        c.style.transform = to.transform; c.style.opacity = to.opacity; c.style.zIndex = String(p.z);
        c.inert = !open && i > 0;
        if (from.transform !== to.transform || from.opacity !== to.opacity) play(c, [from, to], 'settle', { fill: 'none' });
        if (!known.has(c)) { known.add(c); c.addEventListener('pointerenter', () => fan(true)); c.addEventListener('pointerleave', unfan); }
      });
    };
    const fan = (on) => { clearTimeout(shutting); if (open !== on) { open = on; lay(); } };
    // Between two fanned cards is the page: a moment's grace, or crossing the gap would close the hand under the pointer.
    const unfan = () => { clearTimeout(shutting); shutting = setTimeout(() => { if (!region.matches(':focus-within')) fan(false); }, gap('settle', region) * 5); };
    region.addEventListener('focusin', () => fan(true)); region.addEventListener('focusout', unfan);
    // A card FinUI took away leaves as a copy of itself, dropping out of the pile.
    const watch = new MutationObserver((records) => {
      for (const r of records) for (const n of r.removedNodes) {
        if (n.nodeType !== 1 || !n.matches('.fui-toast') || n.isConnected || !known.has(n)) continue;
        const ghost = n.cloneNode(true);
        ghost.classList.add('fm-toast--leaving'); ghost.inert = true; ghost.setAttribute('aria-hidden', 'true');
        region.append(ghost);
        ended(play(ghost, [{ opacity: Number(n.style.opacity) || 1 }, { opacity: 0, translate: '0 12px' }], 'snap', { leaving: true })).then(() => ghost.remove());
      }
      if (!cards().length) open = false;
      lay();
    });
    watch.observe(region, { childList: true });
    lay();
    return () => watch.disconnect();
  },
};
