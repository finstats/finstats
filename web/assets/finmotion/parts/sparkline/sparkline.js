// FinMotion: sparkline. The first time it is seen it is drawn in, and the last value lands as its dot.

import { play, spring, whenSeen, pen } from '../../core/motion.js';

export const part = {
  name: 'sparkline',
  selector: 'svg.fui-sparkline',
  enhance(svg) {
    whenSeen(svg, () => {
      const d = spring('settle', svg).duration, line = svg.querySelector('.fui-sparkline__line'), wash = svg.querySelector('.fui-sparkline__area'), now = svg.querySelector('.fui-sparkline__now');
      if (line) pen(line, 'settle');
      if (wash) play(wash, [{ opacity: 0 }, { opacity: 1 }], 'settle', { delay: d * 0.3, fill: 'backwards' });
      if (now) play(now, [{ opacity: 0, transform: 'scale(0)' }, { opacity: 1, transform: 'none' }], 'drift', { delay: d * 0.5, fill: 'backwards' });
    });
  },
};
