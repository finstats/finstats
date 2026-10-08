// Simple mode's History: every play this person may see, newest first, in one table.

import { h, icon } from '../../dom.js';
import { historyList } from '../history.js';

export default function history(ctx) {
  ctx.title('History');
  const list = historyList(ctx, { id: 'simple-history' });
  ctx.root.append(h('div', { id: 'simple-history' },
    h('div', { class: 'simple-panel-head' }, h('h1', { class: 'simple-page-title' }, icon('repeat', 20), 'History'), h('div', { class: 'simple-head__tools' }, list.tools)), list.el));
}
