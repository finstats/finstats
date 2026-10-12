// Server → Audit: what changed in FinStats, and who did it. Jellyfin administrators only; the
// server refuses everyone else, this only draws. The shape is the server log's: When · Who · What ·
// Detail · From, a search, a kind you can pin, and paging.

import { h, icon, debounce, dateTime, relTime } from '../dom.js';
import { api } from '../api.js';
import { replaceQuery } from '../router.js';
import { card, dataView, sk, pagination, emptyState, avatar } from '../components.js';
import { dataTable } from '../tables.js';
import { t } from '../i18n.js';

const PER_PAGE = 50;
const KINDS = [
  'sign_in', 'sign_in_failed', 'sign_in_refused', 'sign_out', 'setup_completed',
  'key_created', 'key_revoked', 'key_used',
  'setting_changed', 'permissions_changed',
  'service_added', 'service_changed', 'service_removed',
  'target_added', 'target_changed', 'target_removed',
  'backup_made', 'backup_restored', 'backup_deleted', 'backup_undeleted', 'backup_downloaded',
  'task_run', 'task_schedule_changed', 'import_started', 'import_finished',
  'play_deleted', 'play_undeleted', 'trash_purged', 'title_located',
  'alert_resolved', 'alert_reopened',
  'finding_dismissed', 'finding_undismissed',
  'profile_published', 'profile_changed', 'profile_unpublished',
  'profile_link_reset', 'appearance_changed',
];
const KIND_LABEL = Object.fromEntries(KINDS.map((k) => [k, t(`pages.audit.kind.${k}`)]));
const OUTCOME = { failed: ['sev-critical', 'alert', t('pages.audit.outcome.failed')], refused: ['sev-warning', 'alert', t('pages.audit.outcome.refused')] };

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
    case 'permissions_changed': {
      const perms = (d.permissions || []).join(', ') || t('pages.audit.detail.nothing');
      return e.target === 'defaults' ? t('pages.audit.detail.permissions_everyone', { permissions: perms }) : t('pages.audit.detail.permissions_user', { user: e.target || '', permissions: perms });
    }
    case 'key_created': parts.push(`${d.name} (${d.scope})`); if (d.expires_at) parts.push(t('pages.audit.detail.expires', { when: dateTime(d.expires_at) })); break;
    case 'key_revoked': parts.push(d.name || t('pages.audit.detail.key', { id: e.target })); break;
    case 'key_used': parts.push(t('pages.audit.detail.key', { id: e.target })); break;
    case 'service_added': case 'service_changed': case 'service_removed': parts.push(`${d.name} (${d.kind})`); if (d.moved) parts.push(t('pages.audit.detail.moved')); break;
    case 'target_added': case 'target_changed': case 'target_removed': parts.push(t(d.owner === 'own' ? 'pages.audit.detail.target_personal' : 'pages.audit.detail.target_server', { name: d.name, channel: d.channel })); break;
    case 'backup_made': parts.push(d.trigger === 'schedule' ? t('pages.audit.detail.by_schedule') : t('pages.audit.detail.on_request')); if (e.target) parts.push(e.target); if (d.plays != null) parts.push(t('terms.play', { count: d.plays })); if (d.error) parts.push(d.error); break;
    case 'backup_restored': parts.push(e.target || ''); if (d.plays_imported != null) parts.push(t('pages.audit.detail.plays_restored', { count: d.plays_imported, skipped: d.plays_skipped })); if (d.settings_restored) parts.push(t('pages.audit.detail.settings_too')); if (d.error) parts.push(d.error); break;
    case 'backup_deleted': case 'backup_undeleted': case 'backup_downloaded': parts.push(e.target || ''); break;
    case 'task_run': case 'task_schedule_changed': parts.push(e.target || ''); break;
    case 'import_started': parts.push(e.target || ''); break;
    case 'import_finished': parts.push(e.target || ''); if (d.plays_imported != null) parts.push(t('pages.audit.detail.plays_imported', { count: d.plays_imported, skipped: d.plays_skipped })); if (d.error) parts.push(d.error); break;
    case 'play_deleted': case 'play_undeleted': parts.push(t('pages.audit.detail.title_by', { title: d.title || '', user: d.user || '' })); if (d.started_at) parts.push(dateTime(d.started_at)); break;
    case 'trash_purged': parts.push(t('pages.audit.detail.purged', { plays: t('terms.play', { count: d.plays || 0 }), backups: t('pages.audit.detail.backups', { count: d.backups || 0 }) })); break;
    case 'profile_published': case 'profile_changed': parts.push(Object.entries(d.sections || {}).filter(([, on]) => on).map(([k]) => k).join(', ') || t('pages.audit.detail.no_sections')); break;
    case 'profile_unpublished': case 'profile_link_reset': parts.push(e.target || ''); break;
    case 'appearance_changed': parts.push(d.finui_preset ? t('pages.audit.detail.preset', { code: d.finui_preset }) : t('pages.audit.detail.as_it_ships')); break;
    case 'title_located': parts.push(t('pages.audit.detail.plays_moved', { count: d.plays || 0 })); break;
    case 'alert_resolved': parts.push(e.target === 'all' ? t('pages.audit.detail.alerts', { count: d.resolved || 0 }) : t('pages.audit.detail.alert', { id: e.target })); if (d.muted) parts.push(t('pages.audit.detail.muted')); break;
    case 'alert_reopened': parts.push(t('pages.audit.detail.alert', { id: e.target })); break;
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
    kindSlot.replaceChildren(f.kind ? h('span', { class: 'fui-chip fui-chip--removable' }, t('pages.audit.kind_chip', { kind: KIND_LABEL[f.kind] || f.kind }),
      h('button', { type: 'button', class: 'fui-chip__x', 'aria-label': t('pages.audit.remove_kind'), onClick: () => { f.kind = ''; apply(); } }, icon('x', 12))) : '');
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
      summary.textContent = t('pages.audit.entries', { count: data.total });
      if (!data.rows.length) return emptyState(f.q || f.kind ? t('pages.audit.empty_filtered') : t('pages.audit.empty'), f.q || f.kind ? null : t('pages.audit.empty_sub'));
      return [dataTable(h('table', { class: 'fui-data-table audit' },
        h('thead', null, h('tr', null, h('th', { 'data-key': 'when', 'data-first': 'desc' }, t('pages.audit.col.when')), h('th', { 'data-key': 'user' }, t('pages.audit.col.who')), h('th', { 'data-key': 'kind' }, t('pages.audit.col.what')), h('th', { 'data-nosort': '' }, t('pages.audit.col.detail')), h('th', { 'data-nosort': '' }, t('pages.audit.col.from')))),
        h('tbody', null, data.rows.map((e) => {
          const out = OUTCOME[e.outcome];
          return h('tr', null,
            h('td', { class: 'mono nowrap', title: dateTime(e.at) }, relTime(e.at)),
            h('td', null, e.user_id ? h('span', { class: 'user-cell' }, avatar(e.user_id, e.user_name || '?', { size: 20, hasImage: e.has_image }), h('a', { href: `/users/${e.user_id}` }, e.user_name || t('pages.audit.user')))
              : e.user_name ? h('span', { class: 'muted', title: t('pages.audit.as_typed') }, e.user_name) : h('span', { class: 'muted' }, 'FinStats')),
            h('td', null, h('div', { class: 'event-name' }, h('button', { type: 'button', class: 'fui-chip fui-chip--button', title: t('pages.audit.only_this_kind'), onClick: () => { f.kind = e.kind; apply(); } }, KIND_LABEL[e.kind] || e.kind),
              out ? h('span', { class: 'fui-badge--status ' + out[0] }, icon(out[1], 12), out[2]) : null, e.key_name ? h('span', { class: 'muted' }, ` ${t('pages.audit.via_key', { name: e.key_name })}`) : null)),
            h('td', { class: 'wrap-cell' }, h('div', { class: 'event-overview' }, detailText(e))),
            h('td', { class: 'mono' }, e.ip || h('span', { class: 'muted' }, '–')));
        }))), { server: { key: f.sort, dir: f.dir, onSort: (key, dir) => { f.sort = key; f.dir = dir; apply(); } } }),
        data.total > PER_PAGE ? pagination({ page: data.page || f.page, perPage: data.per_page || PER_PAGE, total: data.total, onPage: (p) => { f.page = p; apply(false); window.scrollTo({ top: 0 }); } }) : null];
    },
  });

  const search = h('input', { class: 'fui-field__input fui-field__input--search', type: 'search', placeholder: t('pages.audit.search_placeholder'), value: f.q, 'aria-label': t('pages.audit.search_aria'), autocomplete: 'off' });
  const onSearch = debounce(() => { f.q = search.value.trim(); apply(); }, 250);
  search.addEventListener('input', onSearch);
  ctx.onCleanup(() => onSearch.cancel());

  paintKind();
  dv.load();
  return [h('div', { class: 'filters' }, h('div', { class: 'fui-field__search' }, icon('search', 14), search), kindSlot),
    summary, card({ cls: 'fui-card--flush', id: 'audit', body: view })];
}
