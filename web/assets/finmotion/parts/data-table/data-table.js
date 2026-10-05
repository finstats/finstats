// FinMotion: data-table. When a FinUI table is sorted, its rows make room: each moves from where it was to where the sort
// put it, on its own spring. A table marked fm-arrive deals its rows in, one after another, the first time it is seen.

import { noted, deal, whenSeen, still } from '../../core/motion.js';

/** The most rows a sort moves rather than jumps: a few hundred moving at once is a blur nobody can follow. */
const FLIP_MAX = 150;
/** The most rows dealt in when a table arrives: the ones a screen holds. */
const DEAL_MAX = 24;

export const part = {
  name: 'data-table',
  selector: '.fui-data-table__scroll',
  enhance(box) {
    const table = box.querySelector('table'), body = table && table.tBodies[0];
    if (!body) return;
    // FinUI sorts in its sort button's own click: FinMotion notes where every row stood before it (capturing, so first)
    // and moves them once the sort is done (bubbling, so after).
    let moved = null;
    const before = (e) => { moved = e.target.closest('.fui-data-table__sort') && !still(box) && body.rows.length <= FLIP_MAX ? noted(body.rows) : null; };
    const after = () => { if (moved) { moved(); moved = null; } };
    table.addEventListener('click', before, true); table.addEventListener('click', after);
    if (box.classList.contains('fm-arrive')) whenSeen(box, () => deal([...body.rows].filter((r) => !r.hidden).slice(0, DEAL_MAX)));
    return () => { table.removeEventListener('click', before, true); table.removeEventListener('click', after); };
  },
};
