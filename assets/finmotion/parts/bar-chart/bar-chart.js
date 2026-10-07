// FinMotion: bar-chart. The first time a bar chart is seen, its columns rise from the axis one after another and overshoot
// a hair before resting.

import { play, spring, whenSeen } from '../../core/motion.js';

export const part = {
  name: 'bar-chart',
  selector: '.fui-bar-chart__box',
  enhance(box) {
    whenSeen(box, () => {
      const cols = [...box.querySelectorAll('.fui-chart__bar-group')];
      cols.forEach((g, i) => play(g, [{ transform: 'scaleY(0)' }, { transform: 'none' }], 'drift', { delay: (i / cols.length) * spring('drift', g).duration * 0.5, fill: 'backwards' }));
    });
  },
};
