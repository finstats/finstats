// FinMotion: field. A gentle no: when FinUI gives a field a new error, the input shakes its head the way a person does
// and the error arrives under it a moment later.

import { play, gap } from '../../core/motion.js';
import { shake } from './plan.js';

/** A gentle no for any element: it shakes its head, and `error` (if any) arrives a moment later. */
export function refuse(el, error = null) {
  play(el, shake(8), 'settle', { easing: 'ease-out', fill: 'none' });
  if (error) play(error, [{ opacity: 0, transform: 'translateY(-4px)' }, { opacity: 1, transform: 'none' }], 'settle', { delay: gap('settle', error) * 3, fill: 'backwards' });
}

export const part = {
  name: 'field',
  selector: '.fui-field',
  enhance(field) {
    // FinUI's setError() puts a new .fui-field__error in place: that arriving is the no.
    const watch = new MutationObserver((records) => {
      for (const r of records) for (const n of r.addedNodes) {
        const error = n.nodeType === 1 && (n.matches('.fui-field__error') ? n : n.querySelector('.fui-field__error'));
        const input = error && field.querySelector('.fui-field__input');
        if (input) refuse(input, error);
      }
    });
    watch.observe(field, { childList: true, subtree: true });
    return () => watch.disconnect();
  },
};
