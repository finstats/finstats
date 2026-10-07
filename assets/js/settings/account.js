// Settings → Account: who is signed in here, and signing out. Signing out ends this browser's session only; keys and
// other browsers are left as they are.

import { h, icon, mount } from '../dom.js';
import { api } from '../api.js';
import { state, resetCaches } from '../state.js';
import { navigate } from '../router.js';
import { card, avatar, setBusy } from '../components.js';
import { button } from '../../finui/components/button/button.js';

async function signOut(btn) {
  setBusy(btn, true, 'Signing out…');
  try { await api.post('/auth/logout', {}, { quiet401: true }); } catch { /* signing out anyway */ }
  state.user = null;
  resetCaches();
  navigate('/login');
}

export default {
  key: 'account', label: 'Account', sub: 'Who is signed in here, and signing out', group: 'Account', icon: 'user',
  visible: () => !!state.user,
  entries: [
    { id: 'sign-out', label: 'Sign out', hint: 'log out logout leave session' },
  ],
  async render(slot) {
    const me = state.user;
    const out = button({ type: 'button', id: 'sign-out' }, icon('logout', 14), 'Sign out');
    out.addEventListener('click', () => signOut(out));
    mount(slot, card({
      title: 'Account', id: 'account', sub: 'Signing out ends the session in this browser; API keys and other browsers keep working',
      body: h('div', { class: 'fui-setting-row account-row' },
        h('div', { class: 'account-who' }, avatar(me.id, me.name, { size: 36, hasImage: me.has_image }),
          h('div', { class: 'me-text' }, h('span', { class: 'account-name' }, me.name), h('span', { class: 'me-role' }, me.is_admin ? 'Jellyfin administrator' : 'Signed in with Jellyfin'))),
        out),
    }));
  },
};
