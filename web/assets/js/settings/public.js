// Settings → Public profile (2.0): a part of your own profile, at a link anyone can open without an
// account. Off for everyone until a Jellyfin administrator allows it; each section off until you turn
// it on; devices, addresses and file paths are never published, whatever is ticked here. The link is
// random and can be reset, which is the one way to take back a link already sent.

import { h, icon, mount, relEl } from '../dom.js';
import { api, isAbort } from '../api.js';
import { state, isAdmin } from '../state.js';
import { card, sk, toggle, spinner, setBusy, inlineError, errorState, copyButton, avatar } from '../components.js';
import { plainTable } from '../tables.js';
import { settingRow, toggleRow } from './common.js';
import { button } from '../../finui/components/button/button.js';

const SECTIONS = [
  ['totals', 'Totals and top titles', 'Hours, plays, and your top shows, films and music with their posters.'],
  ['habits', 'Streaks and when you watch', 'Your longest streak, the weekday and hour grid, your genres.'],
  ['recap', 'Your year', 'The headline of this year’s recap, and a card of it to post.'],
  ['recent', 'Recently watched', 'Your last ten plays, by day only, and never until a day after.'],
];
const SECTION_NAMES = Object.fromEntries(SECTIONS.map(([k, l]) => [k, l]));

const absolute = (url) => (/^https?:\/\//.test(url) ? url : location.origin + url);
const pathOf = (url) => url.replace(/^https?:\/\/[^/]+/, '');

export default {
  key: 'public', label: 'Public profile', sub: 'Share a part of your profile at a link, with a card to post', group: 'Account', icon: 'link',
  visible: () => isAdmin() || !!(state.user && state.user.features && state.user.features.public_profiles),
  entries: [
    { id: 'public-mine', label: 'Your public profile', hint: 'share publish link card image profile public' },
    { id: 'public_profiles', label: 'Allow public profiles', hint: 'public profiles server switch share' },
  ],
  async render(slot, store) {
    const own = h('div', { class: 'net-stack' }, sk.rows(3));
    const server = isAdmin() ? h('div', { class: 'net-stack' }, sk.rows(2)) : null;
    mount(slot,
      server ? card({ title: 'Public profiles on this server', sub: 'Nothing is readable without an account until you allow it here', body: server, id: 'public-server' }) : null,
      card({ title: 'Your public profile', sub: 'Devices, addresses and file paths are never published, whatever is ticked here', body: own, id: 'public-mine' }));

    let mine = null, list = null, confirming = null, err = null;

    async function load() {
      try {
        const [m, l] = await Promise.all([
          api.get('/me/public-profile', null, { signal: store.signal }),
          server ? api.get('/public-profiles', null, { signal: store.signal }) : null,
          server ? store.loadSettings() : null,
        ]);
        mine = m; list = l;
      } catch (e) { if (isAbort(e) || e.status === 401) return; mount(own, errorState(e, load)); return; }
      paint();
    }

    function paintServer() {
      if (!server) return;
      const rows = (list && list.profiles) || [];
      mount(server,
        toggleRow(store, { key: 'public_profiles', label: 'Allow public profiles',
          help: 'People may publish parts of their own profile at a link. Off: every link stops working at once.', onSaved: load }),
        rows.length ? plainTable(h('table', { class: 'table table-dense pub-list' },
          h('thead', null, h('tr', null, h('th', null, 'Who'), h('th', null, 'Shown as'), h('th', null, 'Sections'), h('th', null, 'Changed'), h('th', { 'data-nosort': '' }, h('span', { class: 'sr-only' }, 'Actions')))),
          h('tbody', null, rows.map((p) => h('tr', null,
            h('td', null, h('span', { class: 'user-cell' }, avatar(p.user_id, p.user_name, { size: 20 }), p.user_name)),
            h('td', null, p.display_name || h('span', { class: 'muted' }, 'no name')),
            h('td', null, p.published ? Object.keys(SECTION_NAMES).filter((k) => p.sections[k]).map((k) => h('span', { class: 'chip' }, SECTION_NAMES[k])) : h('span', { class: 'muted' }, 'Not published')),
            h('td', { 'data-sort': p.updated_at }, relEl(p.updated_at)),
            h('td', null, p.published ? h('div', { class: 'backup-actions' }, confirming === p.user_id
              ? [h('span', { class: 'muted' }, 'Take it down? The link stops working; they can publish again.'),
                button({ size: 'sm', variant: 'danger', type: 'button', onClick: async (ev) => { setBusy(ev.currentTarget, true, 'Taking down…'); try { await api.del(`/public-profiles/${p.user_id}`); } catch (e) { err = e.message; } confirming = null; await load(); } }, 'Take down'),
                button({ size: 'sm', variant: 'ghost', type: 'button', onClick: () => { confirming = null; paint(); } }, 'Cancel')]
              : button({ size: 'sm', variant: 'ghost', tone: 'danger', type: 'button', onClick: () => { confirming = p.user_id; paint(); } }, 'Take down…')) : null)))))) : null);
    }

    // Like every switch in Settings, each of these saves itself and says so beside itself; the name saves
    // when you leave the field. One request each, carrying the whole choice as it now stands.
    let draft = null, drawnAt = Date.now();
    // One at a time, each from what the last one saved: two switches flipped before the first answer both
    // started from the same draft, and the second put back a section the first had just switched off.
    let saving = Promise.resolve();
    function save(patch) {
      const run = saving.then(async () => {
        const next = { ...draft, ...patch, sections: { ...draft.sections, ...(patch.sections || {}) } };
        mine = await api.put('/me/public-profile', next);
        draft = pick(mine);
        drawnAt = Date.now();
      });
      saving = run.catch(() => {});
      return run;
    }
    const pick = (m) => ({ published: m.published, display_name: m.display_name || '', show_avatar: m.show_avatar, sections: { ...m.sections } });

    function savedNote(note, errBox, run) {
      return async (...args) => {
        mount(errBox, ''); note.replaceChildren(spinner(12));
        try {
          await run(...args);
          note.replaceChildren(icon('check', 13), 'Saved');
          setTimeout(() => note.replaceChildren(), 2000);
          paintTop();
        } catch (e) { note.replaceChildren(); mount(errBox, inlineError('pub-err', `Couldn’t save: ${e.message}`)); throw e; }
      };
    }

    function switchRow(id, label, help, checked, patchFor) {
      const note = h('span', { class: 'saved-note', 'aria-live': 'polite' });
      const errBox = h('div');
      const saveIt = savedNote(note, errBox, (next) => save(patchFor(next)));
      const sw = toggle({ checked, labelledby: `${id}-row-label`, describedby: `${id}-row-help`,
        onChange: (next, revert) => saveIt(next).catch(() => revert(!next)) });
      sw.id = id;
      return settingRow({ id: `${id}-row`, label, help, control: [note, sw], error: errBox });
    }

    /** What a chat app shows under the link: below the switches, so saving one never moves the row you clicked. */
    function previewBox(m) {
      const path = pathOf(m.url);
      const cards = [];
      // The browser keeps a card for an hour; a preview drawn after a change asks under a new address.
      if (m.published && (m.sections.totals || m.sections.habits)) cards.push(['Profile card', `${path}/card.png?v=${drawnAt}`]);
      if (m.published && m.sections.recap) cards.push(['Year card', `${path}/card.png?kind=recap&v=${drawnAt}`]);
      if (!cards.length) return null;
      return h('div', { class: 'pub-preview' }, cards.map(([label, src]) => h('figure', null,
        h('img', { src, alt: `${label}: what a chat app shows under your link`, loading: 'lazy', width: 600, height: 315 }),
        h('figcaption', { class: 'help' }, label, ' · ', h('a', { href: src, download: '' }, 'Download')))));
    }

    function linkBox(m) {
      const url = absolute(m.url);
      const path = pathOf(m.url);
      const resetting = confirming === 'reset';
      return h('div', { class: 'pub-link-box' },
        h('div', { class: 'setting-label' }, m.published ? 'Your link' : 'Your link (not published: it opens nothing)'),
        h('div', { class: 'key-line pub-link' }, h('code', { class: 'mono key-token' }, url), copyButton(url, 'Copy link'),
          m.published ? button({ size: 'sm', variant: 'ghost', href: path, target: '_blank', rel: 'noopener' }, icon('external', 13), 'Open') : null),
        h('div', { class: 'backup-actions' }, resetting
          ? [h('span', { class: 'muted' }, 'Reset it? Every copy of the old link stops working.'),
            button({ size: 'sm', variant: 'danger', type: 'button', onClick: async (ev) => { setBusy(ev.currentTarget, true, 'Resetting…'); try { mine = await api.post('/me/public-profile/reset', {}); drawnAt = Date.now(); } catch (e) { err = e.message; } confirming = null; paintTop(); } }, 'Reset'),
            button({ size: 'sm', variant: 'ghost', type: 'button', onClick: () => { confirming = null; paintTop(); } }, 'Cancel')]
          : button({ size: 'sm', variant: 'ghost', type: 'button', onClick: () => { confirming = 'reset'; paintTop(); } }, 'Reset link…')));
    }

    function form(m) {
      draft = pick(m);
      const name = h('input', { class: 'input', id: 'pub-name', name: 'pub-name', type: 'text', maxLength: 60, autocomplete: 'off',
        placeholder: 'Leave empty to show no name', value: draft.display_name, 'aria-describedby': 'pub-name-row-help' });
      const nameNote = h('span', { class: 'saved-note', 'aria-live': 'polite' });
      const nameErr = h('div');
      const saveName = savedNote(nameNote, nameErr, () => save({ display_name: name.value.trim() }));
      name.addEventListener('change', () => { if (name.value.trim() !== draft.display_name) saveName().catch(() => {}); });
      name.addEventListener('keydown', (e) => { if (e.key === 'Enter') { e.preventDefault(); name.blur(); } });
      return h('div', { class: 'setting-rows', id: 'pub-form' },
        switchRow('pub-published', 'Publish my profile', 'Anyone with the link can read what is switched on below, without signing in.', draft.published, (v) => ({ published: v })),
        settingRow({ id: 'pub-name-row', label: 'Name shown', help: 'Your Jellyfin user name is never shown unless you type it here.', labelFor: 'pub-name', control: [nameNote, name], error: nameErr }),
        switchRow('pub-avatar', 'Show my picture', 'Your Jellyfin picture next to the name.', draft.show_avatar, (v) => ({ show_avatar: v })),
        SECTIONS.map(([k, label, help]) => switchRow(`pub-${k}`, label, help, !!draft.sections[k], (v) => ({ sections: { [k]: v } }))));
    }

    // The link and the preview sit above the rows and are redrawn after every save; the rows are drawn
    // once, so a switch keeps its focus and its "Saved" while the preview catches up.
    const top = h('div');
    const bottom = h('div');
    function paintTop() {
      mount(top, mine && mine.url ? linkBox(mine) : null, err ? inlineError('pub-list-err', err) : null);
      mount(bottom, mine && mine.url ? previewBox(mine) : null);
      err = null;
    }

    function paintOwn() {
      if (!mine) return;
      if (!mine.server_enabled) {
        mount(own, h('p', { class: 'help' }, isAdmin() ? 'Allow public profiles above to publish your own.' : 'An administrator has not allowed public profiles.'));
        return;
      }
      paintTop();
      mount(own, top, form(mine), bottom);
    }

    function paint() { paintServer(); paintOwn(); err = null; }
    await load();
  },
};
