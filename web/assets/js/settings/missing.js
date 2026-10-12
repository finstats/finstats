// Settings → Unlinked media (and a pointer to it from Import): titles plays point at that the library does not have under that name (a Plex
// history calls a film what Plex called it, a guide moves an episode into the specials), each with where it most likely
// is, and Locate to say where it is. The choice moves the plays (`POST /library/locate`) and is kept, so the same history
// imported again attaches by itself. The card is there only while something is missing.
import { h, icon, mount, debounce, episodeCode, TRACKERS } from '../dom.js';
import { api } from '../api.js';
import { openModal, poster } from '../components.js';
import { button } from '../../finui/components/button/button.js';
import { t, tParts, lang } from '../i18n.js';

/** A title as one line: "The Empire Strikes Back (1980)", "The Grand Tour · S00E05 · A Massive Hunt". */
const label = (x) => (x.item_type === 'Episode'
  ? [x.series_name, episodeCode(x.season, x.episode), x.name].filter(Boolean).join(' · ')
  : x.year ? `${x.name} (${x.year})` : x.name);

/** Where a title's plays came from, in words: "from Tautulli", "from Jellystat and Tautulli". */
const fromWhere = (m) => {
  const names = (m.sources || []).map((s) => TRACKERS[s] || s);
  return names.length ? ` · ${t('settings.missing.from', { sources: new Intl.ListFormat(lang(), { type: 'conjunction' }).format(names) })}` : '';
};

/** Fill `slot` with the missing titles. With nothing missing, `card` hides or, given `empty`, says that instead.
 *  Answers `{ refresh }`. */
export function missingCard(slot, card, { empty = null } = {}) {
  async function refresh() {
    let list;
    try { list = (await api.get('/library/missing')).missing || []; } catch { list = []; }
    card.hidden = !list.length && !empty;
    if (!list.length) { mount(slot, empty ? h('p', { class: 'locate-done' }, icon('check', 14), empty) : null); return; }
    const films = list.filter((m) => m.item_type !== 'Episode');
    const shows = new Map();
    for (const m of list.filter((x) => x.item_type === 'Episode')) {
      const k = m.series_name || t('settings.missing.unknown_show');
      if (!shows.has(k)) shows.set(k, []);
      shows.get(k).push(m);
    }
    const row = (m, text) => {
      const likely = m.suggestions && m.suggestions[0];
      return h('li', { class: 'locate-row', 'data-missing': m.id },
        h('span', { class: 'locate-what' }, h('span', { class: 'locate-name' }, text),
          h('span', { class: 'locate-meta' }, `${t('terms.play', { count: m.plays })}${fromWhere(m)}`,
            likely ? [' · ', ...tParts('settings.missing.probably', { title: h('strong', null, label(likely)) })] : ` · ${t('settings.missing.nothing_like')}`)),
        button({ size: 'sm', class: 'locate-btn', type: 'button', onClick: () => pick(m, refresh) }, icon('search', 13), t('settings.missing.locate')));
    };
    mount(slot,
      h('p', { class: 'fui-field__help' }, t('settings.missing.intro', { count: list.length })),
      films.length ? h('ul', { class: 'locate-list' }, films.map((m) => row(m, m.name))) : null,
      [...shows].map(([show, eps]) => h('div', { class: 'locate-show' }, h('h3', { class: 'section-label' }, show),
        h('ul', { class: 'locate-list' }, eps.map((m) => row(m, [episodeCode(m.season, m.episode), m.name].filter(Boolean).join(' · ')))))));
  }
  refresh();
  return { refresh };
}

/** The picker for one missing title: its likeliest places at once, and a search for anything else. */
function pick(m, done) {
  const input = h('input', { type: 'search', class: 'fui-field__input', placeholder: t('settings.missing.search_placeholder'), 'aria-label': t('settings.missing.search_label'), autocomplete: 'off' });
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
        poster(c.id, c.name, { w: 120, cls: 'fui-poster--xs' }), h('span', { class: 'locate-opt-name' }, label(c)),
        h('span', { class: 'locate-opt-kind' }, c.item_type === 'Episode' ? t('item_type.Episode') : c.item_type === 'Video' ? t('settings.missing.video') : t('item_type.Movie'))))
      : h('p', { class: 'locate-none' }, q ? t('settings.missing.no_match', { query: q }) : t('settings.missing.no_likely')));
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
    title: t('settings.missing.where_is', { name: m.name }),
    body: h('div', { class: 'locate-pick' },
      h('p', { class: 'fui-field__help' }, m.item_type === 'Episode'
        ? t('settings.missing.will_move', { count: m.plays, what: [m.series_name, episodeCode(m.season, m.episode)].filter(Boolean).join(' ') })
        : t('settings.missing.will_move_film', { count: m.plays })),
      input, list, said),
    initialFocus: input,
  });
  show('');
}
