// Simple mode's Patch notes: every release since the first, newest first, as plainly as a changelog reads. Each release
// is its version and day (the running one says so), its line of what it is about when it has one, and each kind of
// change named once over a list of its notes. A list of versions in the head jumps to any of them; nothing is folded.

import { h, icon } from '../../dom.js';
import { api } from '../../api.js';
import { markVersionSeen } from '../../state.js';
import { dataView, emptyState, sk } from '../../components.js';
import { inline, days } from '../../changelog-text.js';

const loadChangelog = (signal) => api.get('/changelog', null, { signal });
export const prefetchChangelog = ({ signal }) => [() => loadChangelog(signal)];

function release(r, current) {
  return h('section', { class: 'simple-release', id: `v${r.version}` },
    h('h2', { class: 'simple-release__head', tabindex: '-1' },
      h('span', { class: 'simple-release__version' }, r.version),
      r.date ? h('time', { class: 'simple-release__day', dateTime: r.date }, days(r.started || r.date, r.date)) : null,
      r.version === current ? h('span', { class: 'simple-release__now' }, icon('check', 12), 'Running now') : null),
    r.summary ? h('p', { class: 'simple-release__title' }, inline(r.summary)) : null,
    r.groups.filter((g) => g.items.length).map((g) => [
      h('h3', { class: 'simple-release__kind' }, g.kind),
      h('ul', { class: 'simple-release__notes' }, g.items.map((t) => h('li', null, inline(t)))),
    ]));
}

export default function changelogPage(ctx) {
  ctx.title('Patch notes');
  const jump = h('select', { class: 'simple-select', id: 'simple-changelog-jump', 'aria-label': 'Jump to a version' }, h('option', { value: '' }, 'Jump to a version'));
  jump.addEventListener('change', () => {
    const at = jump.value && document.getElementById(`v${jump.value}`);
    if (at) { at.scrollIntoView({ block: 'start' }); at.querySelector('h2').focus({ preventScroll: true }); }
    jump.value = '';
  });
  const body = h('div', { class: 'simple-panel simple-changelog__body' });
  ctx.root.append(h('div', { id: 'simple-changelog' },
    h('div', { class: 'simple-panel-head' }, h('h1', { class: 'simple-page-title' }, icon('tag', 22), 'Patch notes'), h('div', { class: 'simple-head__tools' }, jump)),
    body));
  dataView({
    container: body, signal: ctx.signal, skeleton: () => sk.cardRows(8),
    fetch: () => loadChangelog(ctx.signal),
    render: (d) => {
      const list = (d && d.releases) || [];
      if (!list.length) return emptyState('No patch notes', 'This build carries none.');
      if (d.current) markVersionSeen(d.current);
      jump.replaceChildren(jump.firstChild, ...list.map((r) => h('option', { value: r.version }, r.version === d.current ? `${r.version} (running now)` : r.version)));
      return list.map((r) => release(r, d.current));
    },
  }).load();
}
