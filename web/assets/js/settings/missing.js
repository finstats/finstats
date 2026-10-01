// Settings → Import, "Not in your library": titles plays point at that the library does not have under that name — a Plex
// history calls a film what Plex called it, a guide moves an episode into the specials — each with where it most likely
// is, and Locate to say where it is. The choice moves the plays (`POST /library/locate`) and is kept, so the same history
// imported again attaches by itself. The card is there only while something is missing.
import { h, icon, num, mount, debounce, episodeCode } from '../dom.js';
import { api } from '../api.js';
import { openModal, poster } from '../components.js';

/** A title as one line: "The Empire Strikes Back (1980)", "The Grand Tour · S00E05 · A Massive Hunt". */
const label = (t) => (t.item_type === 'Episode'
  ? [t.series_name, episodeCode(t.season, t.episode), t.name].filter(Boolean).join(' · ')
  : t.year ? `${t.name} (${t.year})` : t.name);

/** Fill `slot` with the missing titles; `card` is hidden while there are none. Answers `{ refresh }`. */
export function missingCard(slot, card) {
  async function refresh() {
    let list;
    try { list = (await api.get('/library/missing')).missing || []; } catch { list = []; }
    card.hidden = !list.length;
    if (!list.length) { mount(slot); return; }
    const films = list.filter((m) => m.item_type !== 'Episode');
    const shows = new Map();
    for (const m of list.filter((x) => x.item_type === 'Episode')) {
      const k = m.series_name || 'Unknown show';
      if (!shows.has(k)) shows.set(k, []);
      shows.get(k).push(m);
    }
    const row = (m, text) => {
      const likely = m.suggestions && m.suggestions[0];
      return h('li', { class: 'locate-row', 'data-missing': m.id },
        h('span', { class: 'locate-what' }, h('span', { class: 'locate-name' }, text),
          h('span', { class: 'locate-meta' }, `${num(m.plays)} play${m.plays === 1 ? '' : 's'}`,
            likely ? [' · probably ', h('strong', null, label(likely))] : ' · nothing in the library looks like it')),
        h('button', { type: 'button', class: 'btn btn-sm locate-btn', onClick: () => pick(m, refresh) }, icon('search', 13), 'Locate'));
    };
    mount(slot,
      h('p', { class: 'help' }, `${num(list.length)} title${list.length === 1 ? '' : 's'} in your history ${list.length === 1 ? 'isn’t' : 'aren’t'} in your library under that name — usually because the server it was imported from called ${list.length === 1 ? 'it' : 'them'} something else. Locate one to attach its plays; anything you leave stays in your history as it is.`),
      films.length ? h('ul', { class: 'locate-list' }, films.map((m) => row(m, m.name))) : null,
      [...shows].map(([show, eps]) => h('div', { class: 'locate-show' }, h('h3', { class: 'section-label' }, show),
        h('ul', { class: 'locate-list' }, eps.map((m) => row(m, [episodeCode(m.season, m.episode), m.name].filter(Boolean).join(' · ')))))));
  }
  refresh();
  return { refresh };
}

/** The picker for one missing title: its likeliest places at once, and a search for anything else. */
function pick(m, done) {
  const input = h('input', { type: 'search', class: 'input', placeholder: 'Search films and episodes', 'aria-label': 'Search the library', autocomplete: 'off' });
  const list = h('div', { class: 'locate-opts', role: 'list' });
  const said = h('p', { class: 'wire-error', role: 'alert' });
  let modal = null, asked = 0;
  async function show(q) {
    const mine = ++asked;
    let found = [];
    try { found = (await api.get(`/library/missing/${encodeURIComponent(m.id)}/candidates`, q ? { q } : null)).candidates || []; } catch (e) { said.textContent = e.message; }
    if (mine !== asked) return; // an answer to something typed earlier
    mount(list, found.length
      ? found.map((c) => h('button', { type: 'button', class: 'locate-opt', role: 'listitem', 'data-id': c.id, onClick: () => choose(c) },
        poster(c.id, c.name, { w: 120, cls: 'poster-xs' }), h('span', { class: 'locate-opt-name' }, label(c)),
        h('span', { class: 'locate-opt-kind' }, c.item_type === 'Episode' ? 'Episode' : c.item_type === 'Video' ? 'Video' : 'Film')))
      : h('p', { class: 'locate-none' }, q ? `Nothing in the library matches “${q}”.` : 'Nothing in the library looks like it. Search for it by the name your library uses.'));
  }
  async function choose(c) {
    said.textContent = '';
    for (const b of list.querySelectorAll('button')) b.disabled = true;
    try {
      await api.post('/library/locate', { from: m.id, to: c.id });
      modal.close();
      done();
    } catch (e) {
      said.textContent = e.message;
      for (const b of list.querySelectorAll('button')) b.disabled = false;
    }
  }
  input.addEventListener('input', debounce(() => show(input.value.trim()), 200));
  modal = openModal({
    title: `Where is “${m.name}”?`,
    body: h('div', { class: 'locate-pick' },
      h('p', { class: 'help' }, `${num(m.plays)} play${m.plays === 1 ? '' : 's'} of ${m.item_type === 'Episode' ? [m.series_name, episodeCode(m.season, m.episode)].filter(Boolean).join(' ') : 'this film'} will move to the title you choose, and a re-import of the same history will find it by itself.`),
      input, list, said),
    initialFocus: input,
  });
  show('');
}
