// FinMotion: toggle. A switch with weight: pressed, FinUI's knob is thrown across on snap and stretches on the way.

import { play } from '../../core/motion.js';
import { stretch } from './plan.js';

export const part = {
  name: 'toggle',
  selector: '.fui-toggle',
  enhance(btn) {
    // FinUI's own click has already said where the knob goes (aria-checked); this one throws it there.
    const thrown = () => {
      const knob = btn.querySelector('.fui-toggle__knob');
      if (!knob) return;
      const travel = btn.clientWidth - knob.offsetWidth - 2 * knob.offsetLeft;
      play(knob, stretch(btn.getAttribute('aria-checked') === 'true', travel), 'snap', { fill: 'none' });
    };
    btn.addEventListener('click', thrown);
    return () => btn.removeEventListener('click', thrown);
  },
};
