// FinUI: popover. A small box beside what opened it (more about a thing, a short form, a few choices) that opens on
// the side with room, lines itself up with its button, and closes on Esc, a press elsewhere or its button again.

import { h, icon } from '../../core.js';
import { button } from '../button/button.js';
import { beside } from './plan.js';

let seq = 0;
/** popover({ trigger, content, label, side, align }) → { el, open(), close() }. `trigger` is the button it hangs from;
 *  `content` is a node or a function that draws it each time it opens. */
export function popover({ trigger, content, label, side = 'bottom', align = 'start' }) {
  const id = `fui-popover-${++seq}`;
  const pop = h('div', { class: 'fui-popover', id, role: 'dialog', 'aria-label': label, tabindex: '-1', hidden: true });
  trigger.setAttribute('aria-haspopup', 'dialog');
  trigger.setAttribute('aria-expanded', 'false');
  trigger.setAttribute('aria-controls', id);
  const place = () => {
    const at = beside(trigger.getBoundingClientRect(), { w: pop.offsetWidth, h: pop.offsetHeight }, { w: document.documentElement.clientWidth, h: document.documentElement.clientHeight }, { side, align });
    Object.assign(pop.style, { left: `${at.left}px`, top: `${at.top}px` });
    pop.dataset.side = at.side;
  };
  const outside = (e) => { if (!pop.contains(e.target) && !trigger.contains(e.target)) close(); };
  function open() {
    pop.replaceChildren(typeof content === 'function' ? content() : content);
    document.body.append(pop);
    pop.hidden = false; trigger.setAttribute('aria-expanded', 'true');
    place();
    (pop.querySelector('input, button, a[href], [tabindex="0"]') || pop).focus();
    document.addEventListener('pointerdown', outside, true);
    window.addEventListener('resize', place); window.addEventListener('scroll', place, true);
  }
  function close(refocus = false) {
    if (pop.hidden) return;
    pop.hidden = true; trigger.setAttribute('aria-expanded', 'false');
    document.removeEventListener('pointerdown', outside, true);
    window.removeEventListener('resize', place); window.removeEventListener('scroll', place, true);
    pop.remove();
    if (refocus) trigger.focus();
  }
  trigger.addEventListener('click', () => (pop.hidden ? open() : close(true)));
  pop.addEventListener('keydown', (e) => { if (e.key === 'Escape') { e.preventDefault(); e.stopPropagation(); close(true); } });
  return { el: trigger, open, close };
}

/** The box as it looks open, held in place in a page (the gallery shows it this way). */
export const popoverCard = ({ side = 'bottom', title = null, arrow = true }, ...children) =>
  h('div', { class: ['fui-popover', 'fui-popover--held', arrow && 'fui-popover--arrow'], dataset: { side }, role: 'group', 'aria-label': title || 'A popover, held open' },
    title ? h('p', { class: 'fui-popover__title' }, title) : null, children);

export const meta = {
  name: 'popover',
  purpose: 'Opens a small box beside what was pressed: more about it, a short form, a few choices.',
  use: 'Something to read or set that belongs to one control and would crowd the page: who is in a group, a quick filter, a note. side and align say where; it moves to the side with room.',
  avoid: 'A menu of actions (dropdown-menu). Anything that needs the whole attention (modal). A label for an icon (a tooltip, title).',
  variants: ['below, above, beside', 'lined up with the start, the middle or the end', 'with a title', 'with an arrow'],
  states: ['open', 'closed'],
  a11y: 'The button says aria-haspopup="dialog", aria-expanded and aria-controls; the box is role="dialog" named by its label; the focus goes into it, Esc closes it and gives the focus back.',
  props: {
    'popover({ trigger, content, label, side, align })': '→ { el, open(), close() }; content: a node or a function',
    'popoverCard({ side, title, arrow }, ...children)': 'the box, held open in a page',
  },
  playground: {
    controls: [
      { key: 'side', label: 'Side', choices: [['bottom', 'Below'], ['top', 'Above'], ['right', 'Beside']] },
      { key: 'title', label: 'A title', on: true },
      { key: 'arrow', label: 'An arrow', on: true },
    ],
    render: (o) => {
      const btn = button({ size: 'sm' }, icon('users', 13), 'Who watched');
      popover({ trigger: btn, label: 'Who watched', side: o.side, content: () => h('p', null, 'alice, bob and carol, together on Friday.') });
      return h('div', { class: ['fui-popover__demo', `is-${o.side}`] }, btn,
        popoverCard({ side: o.side, title: o.title ? 'Who watched' : null, arrow: o.arrow }, h('p', null, 'alice, bob and carol, together on Friday.')));
    },
  },
};
