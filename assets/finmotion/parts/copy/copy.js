// FinMotion: copy. When FinUI's copy button has copied, its new mark is drawn in one stroke rather than swapped in, and a
// ring spreads from the point pressed (the middle, from the keyboard).

import { h } from '../../core/dom.js';
import { play, ended } from '../../core/motion.js';

export const part = {
  name: 'copy',
  selector: '.fui-copy__button',
  enhance(btn) {
    let at = null;
    const pressed = (e) => { const r = btn.getBoundingClientRect(); at = e.detail ? { x: e.clientX - r.left, y: e.clientY - r.top } : { x: r.width / 2, y: r.height / 2 }; };
    btn.addEventListener('click', pressed, true);
    // FinUI swaps its mark when the copy is done (a tick, or a cross); drawn, it is one brush stroke.
    const watch = new MutationObserver(() => {
      const mark = btn.querySelector('svg[data-icon="check"], svg[data-icon="x"]');
      if (!mark || mark.dataset.fmDrawn != null || !at) return;
      mark.dataset.fmDrawn = '';
      for (const p of mark.querySelectorAll('path, polyline, line')) {
        p.setAttribute('pathLength', '1');
        play(p, [{ strokeDasharray: '1', strokeDashoffset: '1' }, { strokeDasharray: '1', strokeDashoffset: '0' }], 'settle', { fill: 'none' });
      }
      const ring = h('span', { class: 'fm-copy__ring' });
      ring.style.left = `${at.x}px`; ring.style.top = `${at.y}px`;
      btn.append(ring); at = null;
      ended(play(ring, [{ transform: 'translate(-50%, -50%) scale(0)', opacity: 0.45 }, { transform: 'translate(-50%, -50%) scale(5)', opacity: 0 }], 'glide', { easing: 'ease-out' })).then(() => ring.remove());
    });
    watch.observe(btn, { childList: true });
    return () => { watch.disconnect(); btn.removeEventListener('click', pressed, true); };
  },
};
