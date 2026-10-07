// FinMotion: page-header. A FinUI page header marked fm-gathers stays at the top of what scrolls and, tied to the scroll
// and never on a timer, gathers itself in: the title shrinks into a bar, the line under it folds away.

import { gathered } from './plan.js';

/** How far the page scrolls while a header gathers itself, in px. */
const OVER = 90;

export const part = {
  name: 'page-header',
  selector: '.fui-page-header.fm-gathers',
  enhance(head) {
    // Whatever scrolls it (the page or a box it sits in) is heard in the capture phase.
    const scrolled = (e) => {
      const by = e.target === document ? document.scrollingElement : e.target;
      if (by !== document.scrollingElement && !by.contains(head)) return;
      head.style.setProperty('--fm-gathered', String(gathered(by.scrollTop, OVER)));
    };
    document.addEventListener('scroll', scrolled, { capture: true, passive: true });
    return () => document.removeEventListener('scroll', scrolled, true);
  },
};
