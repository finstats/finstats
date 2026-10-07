// FinMotion: tabs. FinUI marks the chosen tab with an edge of its own; FinMotion lays one line under the row instead and
// moves it like an inchworm when another tab is chosen: the edge it goes towards leads, the other follows.

import { h } from '../../core/dom.js';
import { play, gap } from '../../core/motion.js';
import { inchworm } from './plan.js';

export const part = {
  name: 'tabs',
  selector: '.fui-tabs__list',
  enhance(row) {
    const tabs = () => [...row.querySelectorAll('[role="tab"]')];
    const chosen = () => tabs().findIndex((t) => t.getAttribute('aria-selected') === 'true');
    const line = h('span', { class: 'fm-tabs__line', 'aria-hidden': 'true' });
    row.append(line);
    let at = -1;
    const place = (i, animate) => {
      const b = tabs()[i];
      if (!b || !b.offsetWidth) return;
      const left = b.offsetLeft, right = row.scrollWidth - left - b.offsetWidth, plan = animate && at >= 0 ? inchworm(at, i) : null;
      if (plan) {
        const was = { left: line.offsetLeft, right: row.scrollWidth - line.offsetLeft - line.offsetWidth }, now = { left, right };
        for (const edge of ['left', 'right']) play(line, [{ [edge]: `${was[edge]}px` }, { [edge]: `${now[edge]}px` }], plan[edge].spring, { delay: plan[edge].lag ? gap('snap', line) * 2 : 0, fill: 'backwards' });
      }
      line.style.left = `${left}px`; line.style.right = `${right}px`;
      row.classList.add('fm-tabs--lined'); at = i;
    };
    // FinUI moves aria-selected; the line follows it. Laid out or resized, it goes under the chosen tab without moving.
    const watch = new MutationObserver(() => { const i = chosen(); if (i !== at) place(i, true); });
    watch.observe(row, { attributes: true, subtree: true, attributeFilter: ['aria-selected'] });
    const sized = typeof ResizeObserver === 'function' ? new ResizeObserver(() => place(chosen(), false)) : null;
    if (sized) sized.observe(row);
    return () => { watch.disconnect(); if (sized) sized.disconnect(); line.remove(); row.classList.remove('fm-tabs--lined'); };
  },
};
