// Server → Audit: what changed in finstats, and who did it. Jellyfin administrators only; the
// server refuses everyone else, this only draws. The shape is the server log's: When · Who · What ·
// Detail · From, a search, a kind you can pin, and paging.

import { h, icon, debounce, num, dateTime, relTime } from '../dom.js';
import { api } from '../api.js';
import { replaceQuery } from '../router.js';
import { card, dataView, sk, pagination, emptyState, avatar } from '../components.js';
import { dataTable } from '../tables.js';

const PER_PAGE = 50;
const KIND_LABEL = {
  sign_in: 'Signed in', sign_in_failed: 'Sign-in failed', sign_in_refused: 'Sign-in refused', sign_out: 'Signed out', setup_completed: 'Set up finstats',
  key_created: 'Made a key', key_revoked: 'Revoked a key', key_used: 'Used a key for the first time',
  setting_changed: 'Changed settings', permissions_changed: 'Changed permissions',
  service_added: 'Added a connection', service_changed: 'Changed a connection', service_removed: 'Removed a connection',
  target_added: 'Added a notification destination', target_changed: 'Changed a notification destination', target_removed: 'Removed a notification destination',
  backup_made: 'Backup written', backup_restored: 'Backup restored', backup_deleted: 'Backup deleted', backup_downloaded: 'Backup downloaded',
  task_run: 'Ran a task', task_schedule_changed: 'Changed a task’s schedule', import_started: 'Import started', import_finished: 'Import finished', play_deleted: 'Deleted a play', title_located: 'Located a missing title',
  alert_resolved: 'Resolved an alert', alert_reopened: 'Reopened an alert',
  finding_dismissed: 'Dismissed a library finding', finding_undismissed: 'Brought back a library finding',
};
const OUTCOME = { failed: ['sev-critical', 'alert', 'Failed'], refused: ['sev-warning', 'alert', 'Refused'] };

// Shared with the prefetcher, so a prefetched view has exactly the address the page asks for.
const loadAudit = (f, signal) => api.get('/audit', { ...f, per_page: PER_PAGE }, { signal });
const filtersOf = (query) => ({ q: query.get('q') || '', kind: query.get('kind') || '', sort: query.get('sort') || '', dir: query.get('dir') || '', page: Math.max(1, Number(query.get('page')) || 1) });
export const prefetchAudit = ({ query, signal }) => [() => loadAudit(filtersOf(query), signal)];

/** What an entry was about, in words: one line per kind, from its detail. */
function detailText(e) {
  const d = e.detail || {};
  const parts = [];
  switch (e.kind) {
    case 'setting_changed': return (d.changed || []).map((c) => `${c.key}: ${JSON.stringify(c.from)} → ${JSON.stringify(c.to)}`).join(' · ');
    case 'permissions_changed': return `${e.target === 'defaults' ? 'everyone' : 'user ' + (e.target || '')}: ${(d.permissions || []).join(', ') || 'nothing'}`;
    case 'key_created': parts.push(`${d.name} (${d.scope})`); if (d.expires_at) parts.push('expires ' + dateTime(d.expires_at)); break;
    case 'key_revoked': parts.push(d.name || `key ${e.target}`); break;
    case 'key_used': parts.push(`key ${e.target}`); break;
    case 'service_added': case 'service_changed': case 'service_removed': parts.push(`${d.name} (${d.kind})`); if (d.moved) parts.push('moved to another instance'); break;
    case 'target_added': case 'target_changed': case 'target_removed': parts.push(`${d.name} (${d.channel}, ${d.owner === 'own' ? 'personal' : 'server'})`); break;
    case 'backup_made': parts.push(d.trigger === 'schedule' ? 'by the schedule' : 'on request'); if (e.target) parts.push(e.target); if (d.plays != null) parts.push(`${num(d.plays)} plays`); if (d.error) parts.push(d.error); break;
    case 'backup_restored': parts.push(e.target || ''); if (d.plays_imported != null) parts.push(`${num(d.plays_imported)} plays restored, ${num(d.plays_skipped)} already here`); if (d.settings_restored) parts.push('settings too'); if (d.error) parts.push(d.error); break;
    case 'backup_deleted': case 'backup_downloaded': parts.push(e.target || ''); break;
    case 'task_run': case 'task_schedule_changed': parts.push(e.target || ''); break;
    case 'import_started': parts.push(e.target || ''); break;
    case 'import_finished': parts.push(e.target || ''); if (d.plays_imported != null) parts.push(`${num(d.plays_imported)} plays, ${num(d.plays_skipped)} already here`); if (d.error) parts.push(d.error); break;
    case 'play_deleted': parts.push(`${d.title || ''} by ${d.user || ''}`); if (d.started_at) parts.push(dateTime(d.started_at)); break;
    case 'title_located': parts.push(`${num(d.plays || 0)} play${d.plays === 1 ? '' : 's'} moved`); break;
    case 'alert_resolved': parts.push(e.target === 'all' ? `${num(d.resolved || 0)} alerts` : `alert ${e.target}`); if (d.muted) parts.push('muted'); break;
    case 'alert_reopened': parts.push(`alert ${e.target}`); break;
    case 'finding_dismissed': parts.push(e.target || ''); if (d.note) parts.push(d.note); break;
    case 'finding_undismissed': parts.push(e.target || ''); break;
    default: break;
  }
  return parts.filter(Boolean).join(' · ');
}

/** The log itself: filters, count and the table. Mounted by the Server page's Audit section. */
export function auditView(ctx) {
  const f = filtersOf(ctx.query);
  const view = h('div');
  const summary = h('p', { class: 'result-count', 'aria-live': 'polite' });
  const kindSlot = h('span');

  function paintKind() {
    kindSlot.replaceChildren(f.kind ? h('span', { class: 'fui-chip fui-chip--removable' }, 'Kind: ' + (KIND_LABEL[f.kind] || f.kind),
      h('button', { type: 'button', class: 'fui-chip__x', 'aria-label': 'Remove kind filter', onClick: () => { f.kind = ''; apply(); } }, icon('x', 12))) : '');
  }
  function apply(reset = true) {
    if (reset) f.page = 1;
    paintKind();
    replaceQuery({ q: f.q, kind: f.kind, sort: f.sort, dir: f.dir, page: f.page > 1 ? f.page : '' });
    dv.load();
  }

  const dv = dataView({
    container: view, signal: ctx.signal,
    skeleton: () => sk.tableRows(5),
    fetch: () => loadAudit(f, ctx.signal),
    render: (data) => {
      summary.textContent = `${num(data.total)} ${data.total === 1 ? 'entry' : 'entries'}`;
      if (!data.rows.length) return emptyState(f.q || f.kind ? 'No entries match these filters.' : 'Nothing recorded yet', f.q || f.kind ? null : 'Sign-ins, settings changes, keys and backups are written down here from now on.');
      return [dataTable(h('table', { class: 'fui-data-table audit' },
        h('thead', null, h('tr', null, h('th', { 'data-key': 'when', 'data-first': 'desc' }, 'When'), h('th', { 'data-key': 'user' }, 'Who'), h('th', { 'data-key': 'kind' }, 'What'), h('th', { 'data-nosort': '' }, 'Detail'), h('th', { 'data-nosort': '' }, 'From'))),
        h('tbody', null, data.rows.map((e) => {
          const out = OUTCOME[e.outcome];
          return h('tr', null,
            h('td', { class: 'mono nowrap', title: dateTime(e.at) }, relTime(e.at)),
            h('td', null, e.user_id ? h('span', { class: 'user-cell' }, avatar(e.user_id, e.user_name || '?', { size: 20, hasImage: e.has_image }), h('a', { href: `/users/${e.user_id}` }, e.user_name || 'User'))
              : e.user_name ? h('span', { class: 'muted', title: 'as typed' }, e.user_name) : h('span', { class: 'muted' }, 'finstats')),
            h('td', null, h('div', { class: 'event-name' }, h('button', { type: 'button', class: 'fui-chip fui-chip--button', title: 'Show only this kind', onClick: () => { f.kind = e.kind; apply(); } }, KIND_LABEL[e.kind] || e.kind),
              out ? h('span', { class: 'fui-badge--status ' + out[0] }, icon(out[1], 12), out[2]) : null, e.key_name ? h('span', { class: 'muted' }, ` via key “${e.key_name}”`) : null)),
            h('td', { class: 'wrap-cell' }, h('div', { class: 'event-overview' }, detailText(e))),
            h('td', { class: 'mono' }, e.ip || h('span', { class: 'muted' }, '–')));
        }))), { server: { key: f.sort, dir: f.dir, onSort: (key, dir) => { f.sort = key; f.dir = dir; apply(); } } }),
        data.total > PER_PAGE ? pagination({ page: data.page || f.page, perPage: data.per_page || PER_PAGE, total: data.total, onPage: (p) => { f.page = p; apply(false); window.scrollTo({ top: 0 }); } }) : null];
    },
  });

  const search = h('input', { class: 'fui-field__input fui-field__input--search', type: 'search', placeholder: 'Search the audit log…', value: f.q, 'aria-label': 'Search the audit log', autocomplete: 'off' });
  const onSearch = debounce(() => { f.q = search.value.trim(); apply(); }, 250);
  search.addEventListener('input', onSearch);
  ctx.onCleanup(() => onSearch.cancel());

  paintKind();
  dv.load();
  return [h('div', { class: 'filters' }, h('div', { class: 'fui-field__search' }, icon('search', 14), search), kindSlot),
    summary, card({ cls: 'fui-card--flush', id: 'audit', body: view })];
}
