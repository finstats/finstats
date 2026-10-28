// Simple mode's History: every play this person may see, newest first, in one table.

import { h, icon } from '../../dom.js';
import { t } from '../../i18n.js';
import { historyList } from '../history.js';

export default function history(ctx) {
  ctx.title(t('simple.nav.history'));
  const list = historyList(ctx, { id: 'simple-history' });
  ctx.root.append(h('div', { id: 'simple-history' },
    h('div', { class: 'simple-panel-head' }, h('h1', { class: 'simple-page-title' }, icon('repeat', 20), t('simple.nav.history')), h('div', { class: 'simple-head__tools' }, list.tools)), list.el));
}
