// Simple mode's Users and a person's page. The list (with see_everyone) is everybody in a table: when they were last
// seen, from where and on what, what they last played and how much. A person's page has their global figures, the apps
// they play on, what they watched lately, their history and (with see_network) the addresses they played from.

import { h, icon, num, relTime, dateTime } from '../../dom.js';
import { api } from '../../api.js';
import { can, state } from '../../state.js';
import { dataView, poster, avatar, sk, emptyState } from '../../components.js';
import { dataTable } from '../../tables.js';
import { tileStat, tileStats } from '../../../finui/components/tile-stat/tile-stat.js';
import { shelf } from '../../../finui/components/shelf/shelf.js';
import { mediaCard } from '../../../finui/components/media-card/media-card.js';
import { platformTile, longDuration } from '../platform.js';
import { profileHead, box, globalStats } from '../profile.js';
import { historyList } from '../history.js';

export const loadUsers = (signal) => api.get('/users', { days: 0 }, { signal });
export const prefetchUsers = ({ signal }) => [() => loadUsers(signal)];

function table(users) {
  const network = can('see_network');
  const th = (label, opts = {}) => h('th', opts, label);
  const sorted = (n, text) => h('td', { dataset: { sort: String(n || 0) } }, text);
  return dataTable(h('table', { class: 'fui-data-table fui-data-table--hover simple-table' },
    h('thead', null, h('tr', null, th('', { 'data-nosort': '' }), th('User'), th('Last Seen'), network ? th('Last Known IP') : null, th('Last Platform'), th('Last Player'),
      th('Last Played'), th('Total Plays'), th('Total Played Duration'))),
    h('tbody', null, users.map((u) => {
      const seen = Math.max(u.last_activity_at || 0, u.last_played_at || 0);
      return h('tr', null,
        h('td', { class: 'simple-table__icon' }, avatar(u.id, u.name, { size: 36, hasImage: u.has_image })),
        h('td', null, h('a', { href: `/users/${u.id}` }, u.name)),
        sorted(seen, seen ? relTime(seen) : 'never'),
        network ? h('td', { class: 'mono' }, u.last_ip || 'n/a') : null,
        h('td', null, u.last_client || ''),
        h('td', null, u.last_device || ''),
        h('td', null, u.last_item_id && u.last_item_exists !== false ? h('a', { href: `/items/${u.last_item_id}` }, u.last_item_name) : u.last_item_name || 'n/a'),
        sorted(u.plays, num(u.plays)),
        sorted(u.watch_s, longDuration(u.watch_s)));
    }))));
}

export function usersPage(ctx) {
  ctx.title('Users');
  const body = h('div', { id: 'simple-users-table' });
  dataView({
    container: body, signal: ctx.signal, skeleton: () => sk.cardRows(8),
    fetch: () => loadUsers(ctx.signal),
    render: (d) => {
      const users = (d.users || []).filter((u) => !u.removed);
      return [table(users), h('div', { class: 'simple-table__foot' }, h('span', { class: 'simple-table__count' }, `Showing ${num(users.length)} of ${num(users.length)} users`))];
    },
  }).load();
  ctx.root.append(h('div', { class: 'simple-panel-head' }, h('h1', { class: 'simple-page-title' }, icon('users', 22), 'All Users')), h('div', { class: 'simple-panel' }, body));
}

/** A person, their four figures and who they are in one answer (it was the whole list once per span). */
const loadSpans = (id, signal) => api.get('/stats/spans', { user_id: id }, { signal });

/** A person's page, in one go: who they are and their figures, the apps they play on, and what they watched lately. */
export function loadUser(id, signal) {
  const o = { signal };
  return Promise.all([
    loadSpans(id, signal),
    api.get('/stats/top', { days: 0, kind: 'clients', user_id: id, limit: 24, sort: 'plays' }, o),
    api.get('/activity', { days: 0, user_id: id, page: 1, per_page: 30 }, o),
  ]).then(([p, clients, recent]) => ({ user: p.user, spans: p.spans, clients, recent }));
}
export const prefetchUser = ({ params, signal }) => [() => loadUser(params.id, signal)];

const loadAddresses = (id, signal) => api.get(`/users/${encodeURIComponent(id)}`, { days: 0 }, { signal });

function addresses(ips) {
  if (!ips.length) return h('p', { class: 'simple-empty' }, 'No addresses recorded.');
  return dataTable(h('table', { class: 'fui-data-table fui-data-table--hover simple-table' },
    h('thead', null, h('tr', null, h('th', null, 'IP Address'), h('th', null, 'Network'), h('th', null, 'First Seen'), h('th', null, 'Last Seen'), h('th', null, 'Plays'))),
    h('tbody', null, ips.map((x) => h('tr', null, h('td', { class: 'mono' }, x.ip), h('td', null, x.is_local ? 'LAN' : x.is_local === false ? 'WAN' : 'Unknown'),
      h('td', { dataset: { sort: String(x.first_seen || 0) } }, x.first_seen ? dateTime(x.first_seen) : ''), h('td', { dataset: { sort: String(x.last_seen || 0) } }, x.last_seen ? relTime(x.last_seen) : ''),
      h('td', null, num(x.plays)))))));
}

export function userPage(ctx) {
  const id = ctx.params.id;
  // Somebody else's page, for a person who may see only their own: every list on it would be narrowed to their own
  // plays by the server, and the History tab showed exactly that under the other person's address.
  if (!can('see_everyone') && id !== state.user.id) {
    ctx.title('Not found');
    ctx.root.append(emptyState('No such person', 'They may have been removed from Jellyfin, or their page is not yours to see.'));
    return;
  }
  const network = can('see_network');
  const tab = ['history', 'ips'].includes(ctx.query.get('tab')) && (ctx.query.get('tab') !== 'ips' || network) ? ctx.query.get('tab') : 'profile';
  const head = h('div');
  const body = h('div');
  const named = (u) => {
    ctx.title(u.name);
    head.replaceChildren(profileHead({ picture: avatar(u.id, u.name, { size: 80, hasImage: u.has_image }), name: u.name, tabs: [
      { label: 'Profile', href: `/users/${id}`, current: tab === 'profile' }, { label: 'History', href: `/users/${id}?tab=history`, current: tab === 'history' },
      network ? { label: 'IP Addresses', href: `/users/${id}?tab=ips`, current: tab === 'ips' } : null].filter(Boolean) }));
  };
  ctx.root.append(head, body);
  if (tab !== 'profile') {
    // The header needs only the name; each tab asks for its own list.
    loadSpans(id, ctx.signal).then((d) => { if (d.user) named(d.user); }).catch(() => {});
    if (tab === 'history') {
      const list = historyList(ctx, { id: 'simple-user-history', fixed: { user_id: id } });
      body.append(h('div', { class: 'simple-panel-head' }, h('h2', { class: 'simple-page-title' }, 'History'), h('div', { class: 'simple-head__tools' }, list.tools)), list.el);
    } else {
      const ips = h('div', { id: 'simple-ips' });
      dataView({ container: ips, signal: ctx.signal, skeleton: () => sk.cardRows(4), fetch: () => loadAddresses(id, ctx.signal), render: (d) => addresses(d.ips || []) }).load();
      body.append(box({ id: 'simple-ip-box', title: 'IP Addresses', icon: 'globe', body: ips }));
    }
    return;
  }
  dataView({
    container: body, signal: ctx.signal, skeleton: () => sk.cardRows(6),
    fetch: () => loadUser(id, ctx.signal),
    render: (d) => {
      if (!d.user) return emptyState('No such person', 'They may have been removed from Jellyfin, or their page is not yours to see.');
      named(d.user);
      const apps = d.clients && d.clients.rows.length ? box({ id: 'simple-player-stats', title: 'Player Stats', icon: 'monitor',
        body: tileStats(d.clients.rows.map((c) => tileStat({ picture: platformTile(c.name, 80), name: c.name, figure: num(c.plays), unit: c.plays === 1 ? 'play' : 'plays' }))) }) : null;
      const seen = new Set();
      const watched = ((d.recent && d.recent.rows) || []).filter((p) => { const k = p.series_id || p.item_id; if (seen.has(k)) return false; seen.add(k); return true; }).slice(0, 18);
      const recent = watched.length ? box({ id: 'simple-user-recent', title: 'Recently Watched', icon: 'repeat',
        body: shelf({ label: 'Recently watched', items: watched.map((p) => mediaCard({ href: p.item_exists !== false ? `/items/${p.series_id || p.item_id}` : null, poster: poster(p.image_item_id, p.series_name || p.item_name, { w: 240, cls: 'fui-poster--grid' }),
          name: p.series_name || p.item_name, sub: relTime(p.ended_at) })) }) }) : null;
      return [box({ id: 'simple-global', title: 'Global Stats', icon: 'trendUp', body: globalStats(d.spans) }), apps, recent].filter(Boolean);
    },
  }).load();
}
