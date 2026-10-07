// FinMotion: line-chart. The first time a line chart is seen, each line is drawn as a pen would, left to right; the wash
// under it follows, and the last value lands as its dot.

import { play, spring, whenSeen, pen } from '../../core/motion.js';

export const part = {
  name: 'line-chart',
  selector: '.fui-line-chart__box',
  enhance(box) {
    whenSeen(box, () => {
      for (const line of box.querySelectorAll('.fui-chart__line')) pen(line, 'glide');
      const d = spring('glide', box).duration;
      for (const wash of box.querySelectorAll('.fui-chart__area')) play(wash, [{ opacity: 0 }, { opacity: 1 }], 'glide', { delay: d * 0.3, fill: 'backwards' });
      // The end of each line; the crosshair's dots (hidden until pointed at) are not drawn in.
      for (const dot of box.querySelectorAll('.fui-chart__dot:not([visibility])')) play(dot, [{ opacity: 0, transform: 'scale(0)' }, { opacity: 1, transform: 'none' }], 'drift', { delay: d * 0.55, fill: 'backwards' });
    });
  },
};
