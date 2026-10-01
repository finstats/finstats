// /users/:id/watchlist — the films and shows somebody means to watch, grouped by where each one stands now. Only ever
// one's own: the server answers the caller's list whatever the address says, so somebody else's address opens yours.
import { h, icon } from '../dom.js';
import { api } from '../api.js';
import { navigate } from '../router.js';
import { dataView, sk, avatar, poster, emptyState, errorState } from '../components.js';
import { state } from '../state.js';
import { userTabs } from './users.js';
import { dayName, upcomingPoster } from '../upcoming.js';
import { button } from '../../finui/components/button/button.js';

const loadWatchlist = (signal) => api.get('/me/watchlist', null, { signal });
export const prefetchWatchlist = ({ signal }) => [() => loadWatchlist(signal)];

// Where each title stands, in the order a person would look: what they are part-way through first, what is already
// watched last. The state names are the server's (docs/api.md).
const GROUPS = [
  ['started', 'Started'],
  ['on_server', 'On the server'],
  ['coming_up', 'Coming up'],
  ['requested', 'Requested'],
  ['not_on_server', 'Not on the server'],
  ['left_library', 'Left the library'],
  ['watched', 'Watched'],
];

/** "5 of 26 episodes", "Coming up Friday", "Requested by bob": where one title stands, in words. */
export function stateWords(e) {
  const p = e.progress;
  switch (e.state) {
    case 'watched': return p && p.total > 1 ? `All ${p.total} episodes watched` : 'Watched';
    case 'started': return p ? `${p.seen} of ${p.total} episodes` : 'Started';
    case 'on_server': return 'On the server';
    case 'requested': {
      const r = e.request || {};
      return r.by_you ? 'Requested by you' : r.user_name ? `Requested by ${r.user_name}` : 'Requested';
    }
    case 'coming_up': {
      if (!e.next || !e.next.day) return 'Coming up';
      const day = dayName(e.next.day);
      return `Coming up ${day === 'Today' || day === 'Tomorrow' ? day.toLowerCase() : day}`;
    }
    case 'left_library': return 'Left the library';
    default: return 'Not on the server';
  }
}

export function watchlistPage(ctx) {
  const me = state.user && state.user.id;
  // A watchlist is its owner's alone: there is no address for anybody else's.
  if (ctx.params.id !== me) { navigate(`/users/${me}/watchlist`, { replace: true }); return; }
  ctx.title('Watchlist');
  const headerSlot = h('div');
  const view = h('div');

  const dv = dataView({
    container: view, signal: ctx.signal,
    skeleton: () => h('div', { class: 'wl-grid' }, [0, 1, 2, 3, 4, 5].map(() => h('span', { class: 'fui-skeleton wl-sk' }))),
    fetch: () => loadWatchlist(ctx.signal),
    render: (d) => {
      paintHeader(d.user);
      const entries = d.entries || [];
      if (!entries.length) return emptyState('Nothing on your watchlist yet', 'Add a film or a show from its page, from Upcoming, from Recently added or from search.');
      return GROUPS.map(([key, label]) => {
        const list = entries.filter((e) => e.state === key);
        if (!list.length) return null;
        return h('section', { class: 'wl-group', 'aria-label': label },
          h('h2', { class: 'wl-group-head' }, label, h('span', { class: 'wl-count' }, String(list.length))),
          h('ul', { class: 'wl-grid' }, list.map((e) => h('li', { class: 'wl-card' }, cardFor(e)))));
      });
    },
  });

  function paintHeader(u) {
    ctx.title(`${u.name} · Watchlist`);
    headerSlot.replaceChildren(h('header', { class: 'page-header entity-header' },
      avatar(u.id, u.name, { size: 56, hasImage: u.has_image }),
      h('div', null, h('h1', { class: 'page-title' }, u.name), h('p', { class: 'page-sub' }, 'Films and shows to watch, newest first. Only you can see this list.'))));
  }

  function cardFor(e) {
    const href = e.item_id ? `/items/${e.item_id}` : null;
    const art = e.poster && e.poster.item_id ? poster(e.poster.item_id, e.title, { w: 300, cls: 'fui-poster--grid' }) : upcomingPoster(e, { w: 300, cls: 'fui-poster--grid' });
    const remove = button({ variant: 'icon', class: 'wl-remove', type: 'button', 'aria-label': `Remove ${e.title} from your watchlist`, title: 'Remove from your watchlist',
      onClick: async () => {
        remove.disabled = true;
        try { await api.del(`/me/watchlist/${e.id}`); dv.load(); } catch (err) { remove.disabled = false; mountError(err); }
      } }, icon('x', 15));
    return [
      href ? h('a', { class: 'wl-poster', href, tabindex: -1, 'aria-hidden': 'true' }, art) : h('span', { class: 'wl-poster' }, art),
      href ? h('a', { class: 'wl-name', href }, e.title) : h('span', { class: 'wl-name' }, e.title),
      h('span', { class: 'wl-sub' }, [e.year ? String(e.year) : null, e.kind === 'Series' ? 'Show' : 'Film'].filter(Boolean).join(' · ')),
      h('span', { class: ['wl-state', (e.state === 'coming_up' || e.state === 'started') && 'is-live'] }, stateWords(e)),
      remove,
    ];
  }

  function mountError(err) { view.prepend(errorState(err, () => dv.load())); }

  headerSlot.append(h('header', { class: 'page-header entity-header' }, h('span', { class: 'fui-skeleton wl-sk-avatar' }), h('div', null, sk.line('180px', 26))));
  ctx.root.append(headerSlot, userTabs(me, 'watchlist'), view);
  dv.load();
}
