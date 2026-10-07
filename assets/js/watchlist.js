// The watchlist, wherever a film or a show is offered: one toggle, "Add to watchlist" / "On your watchlist".
// What is on the list is asked for once (GET /me/watchlist/keys) and shared by every toggle on the page, so a shelf of
// thirty posters asks nothing more; a change asks once again and repaints every toggle on screen, because the same
// show can be offered twice on one page (an agenda lists each of its episodes).
import { h, icon } from './dom.js';
import { api, isAbort, onViewCacheCleared } from './api.js';
import { state } from './state.js';
import { button } from '../finui/components/button/button.js';

/** An older server says nothing about watchlists, and then no page offers one. */
export const hasWatchlist = () => !!(state.user && state.user.features && state.user.features.watchlist);

/** What can go on a list: films and shows, as Jellyfin types them. */
export const watchable = (type) => type === 'Movie' || type === 'Series';

let asked = null;   // the list, on its way or here; null = ask again
// Any write may have changed it (and a sign-out certainly has): ask again next time.
onViewCacheCleared(() => { asked = null; });

function entries() {
  if (!asked) asked = api.get('/me/watchlist/keys').then((d) => d.entries || [], (e) => { asked = null; throw e; });
  return asked;
}

/** The entry that stands for this title: any copy of it in the library, or the same kind and one of its ids. */
function entryFor(list, t) {
  return list.find((e) => (t.item_id && e.item_ids.includes(t.item_id))
    || (e.kind === t.kind && ((t.tmdb_id != null && e.tmdb_id === String(t.tmdb_id)) || (t.tvdb_id != null && e.tvdb_id === String(t.tvdb_id))))) || null;
}

async function repaintAll() {
  const list = await entries();
  for (const btn of document.querySelectorAll('.wl-toggle')) if (btn._wl) btn._wl.paint(list);
}

/** Put a title on the list, or take its entry off, and repaint every toggle on screen. */
async function flip(t, entry) {
  if (entry) await api.del(`/me/watchlist/${entry.id}`);
  else await api.post('/me/watchlist', t.item_id ? { item_id: t.item_id } : t);
  asked = null;
  await repaintAll();
}

/** The watchlist as a context menu's item, for a film or a show, given `t` as the toggle takes it (the library's item or the
 *  title's ids). It says which way it goes once the list is known (at once, when a toggle on the page has asked already),
 *  and goes. Null on a server without watchlists, or for a title that cannot be named. */
export function watchMenuItem(t) {
  if (!hasWatchlist() || !(t.item_id || ((t.tmdb_id != null || t.tvdb_id != null) && watchable(t.kind)))) return null;
  return { label: 'Watchlist', icon: 'bookmark', later: entries().then((list) => {
    const entry = entryFor(list, t);
    return { label: entry ? 'Remove from watchlist' : 'Add to watchlist', icon: 'bookmark', onSelect: () => flip(t, entry).catch(() => {}) };
  }) };
}

/**
 * The toggle for one film or show. `t`: `{item_id}` for a title in the library, else `{kind, tmdb_id, tvdb_id, title, year}`.
 * `compact`: an icon alone, for a row, a card or a search result, named for screen readers by `name`. `keepFocus`: a
 * click leaves the focus where it was (the search field). `ofShow`: offered on an episode or a season, so it says it is
 * the show that goes on the list. Hidden until the list is known; nothing at all on a server without watchlists, or for
 * a title that cannot be named.
 */
export function watchToggle(t, { compact = false, name = '', keepFocus = false, ofShow = false } = {}) {
  if (!hasWatchlist() || !(t.item_id || ((t.tmdb_id != null || t.tvdb_id != null) && watchable(t.kind)))) return null;
  const [addWords, onWords] = ofShow ? ['Add show to watchlist', 'Show is on your watchlist'] : ['Add to watchlist', 'On your watchlist'];
  const label = compact ? null : h('span', null, addWords);
  const note = h('span', { class: 'wl-error', role: 'status' });
  const btn = button({ variant: compact ? 'icon' : 'default', class: ['wl-toggle', compact && 'wl-compact'], hidden: true, 'aria-pressed': 'false',
    onMousedown: keepFocus ? (e) => e.preventDefault() : null }, icon('bookmark', compact ? 16 : 14), label);
  let entry = null;
  btn._wl = {
    paint(list) {
      entry = entryFor(list, t);
      const words = entry ? onWords : addWords;
      btn.setAttribute('aria-pressed', String(!!entry));
      btn.classList.toggle('is-on', !!entry);
      if (label) label.textContent = words;
      else { btn.setAttribute('aria-label', name ? `${words}: ${name}` : words); btn.title = entry ? `${onWords}. Click to take it off.` : addWords; }
      btn.hidden = false;
    },
  };
  btn.addEventListener('click', async (e) => {
    // On a card or a search result the toggle sits beside a link: the click is the toggle's alone.
    e.preventDefault(); e.stopPropagation();
    if (btn.disabled) return;
    btn.disabled = true; note.textContent = '';
    try {
      await flip(t, entry);
    } catch (err) {
      if (!isAbort(err)) note.textContent = err.message;
    } finally { btn.disabled = false; }
  });
  entries().then((list) => btn._wl.paint(list), () => {});   // an answer that never comes leaves it hidden
  return h('span', { class: ['wl', compact && 'wl-is-compact'] }, btn, note);
}

