// What both layouts do once their routes are in place: ask who is signed in, then start the router and the prefetcher.

import { h, icon } from './dom.js';
import { api, setUnauthorizedHandler } from './api.js';
import { state, resetCaches, applyTheme } from './state.js';
import { start, navigate, onRouteChange } from './router.js';
import { startPrefetching } from './prefetch.js';
import { setBusy, closeDialogs } from './components.js';
import { button } from '../finui/components/button/button.js';
import { t } from './i18n.js';

/** `top(me)`: the pages this layout's prefetcher warms, in order. */
export async function boot({ top }) {
  // Session expired (or never existed): drop the user and let the router's guard
  // send us to /login?next=<where we were>.
  setUnauthorizedHandler(() => {
    if (!state.user) return;
    state.user = null;
    resetCaches();
    navigate(location.pathname + location.search, { replace: true });
  });
  applyTheme();   // theme.js already set the page's colours; this points the browser's own bar at the same choice

  const app = document.getElementById('app');
  try {
    state.status = await api.get('/status', null, { quiet401: true });
    if (state.status.configured) {
      try { state.user = (await api.get('/auth/me', null, { quiet401: true })).user; } catch (e) { if (e.status !== 401) throw e; }
    }
  } catch (e) {
    app.replaceChildren(h('main', { class: 'bare' }, h('div', { class: 'auth-card' },
      h('h1', { class: 'auth-title' }, t('boot.not_responding')),
      h('p', { class: 'auth-sub' }, e.message),
      button({ variant: 'primary', type: 'button', onClick: (e) => { setBusy(e.currentTarget, true, t('common.trying_again')); location.reload(); } }, icon('refresh', 14), t('common.try_again')))));
    return;
  }
  // finstatsWelcome() in the browser's console opens the welcome again, to choose afresh or to try it by hand.
  window.finstatsWelcome = () => navigate('/welcome?next=' + encodeURIComponent(location.pathname + location.search));
  startPrefetching({ top });
  onRouteChange(closeDialogs);
  start();
}
