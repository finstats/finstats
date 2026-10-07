// FinMotion: heatmap. A cell pressed answers like water: the cells around it light and settle in rings, the nearest first.

import { play, gap } from '../../core/motion.js';
import { ripple } from './plan.js';

export const part = {
  name: 'heatmap',
  selector: '.fui-heatmap__box',
  enhance(box) {
    const pressed = (e) => {
      const cells = [...box.querySelectorAll('.fui-heatmap__cell')];
      if (!cells.length) return;
      // Which cell was pressed, and how many make a row, from where FinUI drew them.
      const k = cells.findIndex((c) => { const r = c.getBoundingClientRect(); return e.clientX >= r.left - 1 && e.clientX <= r.right + 1 && e.clientY >= r.top - 1 && e.clientY <= r.bottom + 1; });
      if (k < 0) return;
      const cols = cells.filter((c) => c.getAttribute('y') === cells[0].getAttribute('y')).length;
      const d = ripple(k, cols, cells.length), step = gap('settle', box) * 1.3;
      cells.forEach((c, i) => play(c, [{ transform: 'none', filter: 'none' }, { transform: 'scale(1.3)', filter: 'brightness(1.5)', offset: 0.35 }, { transform: 'none', filter: 'none' }], 'settle', { easing: 'ease-out', delay: d[i] * step, fill: 'none' }));
    };
    box.addEventListener('pointerdown', pressed);
    return () => box.removeEventListener('pointerdown', pressed);
  },
};
