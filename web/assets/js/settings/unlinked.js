// Settings → Unlinked media: plays whose title the library does not have under that name — almost always from an import
// (Tautulli, Jellystat, Streamystats), whose server named things differently — each linked to the right title by hand,
// with the likeliest one already found. The list itself is `missing.js`; this is its page.
import { h } from '../dom.js';
import { can } from '../state.js';
import { card } from '../components.js';
import { missingCard } from './missing.js';

export default {
  key: 'unlinked', label: 'Unlinked media', sub: 'Imported plays that match nothing in your library', group: 'Data', icon: 'link',
  visible: () => can('manage'),
  entries: [{ id: 'unlinked', label: 'Link a title by hand', hint: 'unlinked missing unmatched locate match title tautulli jellystat streamystats import plex' }],
  async render(slot) {
    const body = h('div');
    const box = card({ title: 'Unlinked media', sub: 'Plays whose title your library calls something else, or no longer has', body, id: 'unlinked' });
    slot.append(box);
    missingCard(body, box, { empty: 'Everything in your history is linked to a title in your library.' });
  },
};
