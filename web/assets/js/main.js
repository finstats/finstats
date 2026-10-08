// FinStats SPA entry point.

import { h, icon } from './dom.js';
import { api, setUnauthorizedHandler } from './api.js';
import { state, resetCaches, applyTheme } from './state.js';
import { route, setLayout, start, navigate } from './router.js';
import { layout } from './shell.js';
import { startPrefetching } from './prefetch.js';
import { pageHeader, emptyState, setBusy } from './components.js';

import dashboard, { prefetchDashboard } from './pages/dashboard.js';
import activity, { prefetchActivity } from './pages/activity.js';
import { usersPage, userPage, prefetchUsers, prefetchUser } from './pages/users.js';
import { timelinePage, prefetchTimeline } from './pages/timeline.js';
import { watchlistPage, prefetchWatchlist } from './pages/watchlist.js';
import { librariesPage, libraryPage, libraryTitlesPage, prefetchLibraries, prefetchLibrary, prefetchLibraryTitles } from './pages/libraries.js';
import itemPage, { prefetchItem } from './pages/item.js';
import personPage, { prefetchPerson } from './pages/person.js';
import playback, { prefetchPlayback } from './pages/playback.js';
import recapPage, { prefetchRecap } from './pages/recap.js';
import events from './pages/events.js';
import serverPage, { prefetchServer } from './pages/server.js';
import securityPage, { prefetchSecurity } from './pages/security.js';
import pipelinePage, { prefetchPipeline } from './pages/pipeline.js';
import settings from './pages/settings.js';
import togetherPage, { prefetchTogether } from './pages/together.js';
import changelogPage, { prefetchChangelog } from './pages/changelog.js';
import licensesPage, { prefetchLicenses } from './pages/licenses.js';
import { setupPage, loginPage } from './pages/auth.js';
import { button } from '../finui/components/button/button.js';
import { animateWithin } from '../finui/components/animated-icon/animated-icon.js';
import { motion } from '../finmotion/core/finmotion.js';
import { installMenus } from './menus.js';

route('/setup', setupPage, { bare: true });
route('/login', loginPage, { bare: true });
// `prefetch` hands the prefetcher the page's own loader, so what it fetches is found again by the page.
route('/', dashboard, { prefetch: prefetchDashboard });
route('/recap', recapPage, { prefetch: prefetchRecap });
route('/activity', activity, { prefetch: prefetchActivity });
route('/together', togetherPage, { prefetch: prefetchTogether });
route('/users', usersPage, { perm: 'see_everyone', prefetch: prefetchUsers });
route('/users/:id', userPage, { prefetch: prefetchUser });
route('/users/:id/timeline', timelinePage, { prefetch: prefetchTimeline });
route('/users/:id/watchlist', watchlistPage, { prefetch: prefetchWatchlist });
route('/libraries', librariesPage, { prefetch: prefetchLibraries });
route('/libraries/:id', libraryPage, { prefetch: prefetchLibrary });
route('/libraries/:id/titles', libraryTitlesPage, { prefetch: prefetchLibraryTitles });
route('/items/:id', itemPage, { prefetch: prefetchItem });
route('/people/:id', personPage, { prefetch: prefetchPerson });
route('/playback', playback, { prefetch: prefetchPlayback });
route('/pipeline', pipelinePage, { prefetch: prefetchPipeline });
route('/server', serverPage, { perm: 'see_server', prefetch: prefetchServer });
route('/server/:section', serverPage, { perm: 'see_server', prefetch: prefetchServer });
route('/events', events, { perm: 'see_server' });   // forwards to /server/log
route('/security', securityPage, { perm: 'see_network', prefetch: prefetchSecurity });
route('/settings', settings);   // every signed-in user has at least their own API keys here
route('/settings/:section', settings);
route('/settings/:section/:task', settings);   // a task's own schedule: /settings/tasks/backup
route('/changelog', changelogPage, { prefetch: prefetchChangelog });
// Anyone signed in may read the licences; the shortcut to them sits in Settings.
route('/licenses', licensesPage, { prefetch: prefetchLicenses });
route('*', (ctx) => {
  ctx.title('Not found');
  ctx.root.append(pageHeader('Page not found'), emptyState('There’s nothing at this address.', 'It may have been a link to something that was removed.',
    button({ href: '/' }, icon('home', 14), 'Go to the dashboard')));
});

setLayout(layout);

// Session expired (or never existed): drop the user and let the router's guard
// send us to /login?next=<where we were>.
setUnauthorizedHandler(() => {
  if (!state.user) return;
  state.user = null;
  resetCaches();
  navigate(location.pathname + location.search, { replace: true });
});

applyTheme();   // theme.js already set the page's colours; this points the browser's own bar at the same choice
// Every icon in something pressable (a link, a button, a tab) does its act once when pointed at or focused, wherever it is drawn.
animateWithin(document.body);
// FinMotion: every FinUI component moves, at the pace of the person's Motion choice and not at all with reduced motion.
motion(document.body);
installMenus();

async function boot() {
  const app = document.getElementById('app');
  try {
    state.status = await api.get('/status', null, { quiet401: true });
    if (state.status.configured) {
      try { state.user = (await api.get('/auth/me', null, { quiet401: true })).user; } catch (e) { if (e.status !== 401) throw e; }
    }
  } catch (e) {
    app.replaceChildren(h('main', { class: 'bare' }, h('div', { class: 'auth-card' },
      h('h1', { class: 'auth-title' }, 'FinStats isn’t responding'),
      h('p', { class: 'auth-sub' }, e.message),
      button({ variant: 'primary', type: 'button', onClick: (e) => { setBusy(e.currentTarget, true, 'Trying again…'); location.reload(); } }, icon('refresh', 14), 'Try again'))));
    return;
  }
  startPrefetching();
  start();
}

boot();
