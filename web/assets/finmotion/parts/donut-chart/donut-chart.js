// FinMotion: donut-chart. The first time a donut is seen it is poured round from the top, one share after another, the
// largest first and longest, through a conic mask (donut-chart.css), whole at rest. The shares are read from how FinUI
// drew the slices.

import { play, spring, whenSeen } from '../../core/motion.js';
import { pour, shares } from './plan.js';

export const part = {
  name: 'donut-chart',
  selector: '.fui-donut-chart__svg',
  enhance(svg) {
    whenSeen(svg, () => {
      const c = (svg.viewBox.baseVal && svg.viewBox.baseVal.width) / 2 || svg.clientWidth / 2;
      const parts = shares([...svg.querySelectorAll('.fui-chart__bar')].map((p) => p.getAttribute('d') || ''), c);
      let reached = 0, upto = 0;
      const frames = [{ '--fm-poured': 0, offset: 0 }, ...pour(parts).map((x, i) => {
        upto = Math.min(1, upto + parts[i]); reached = Math.max(reached, x.start + x.length);
        return { '--fm-poured': i === parts.length - 1 ? 1 : upto, offset: reached };
      })];
      play(svg, frames, 'glide', { easing: 'cubic-bezier(.3, .75, .35, 1)', duration: spring('glide', svg).duration * 1.4, fill: 'none' });
    });
  },
};
