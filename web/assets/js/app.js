// The FinStats layout: the menu, and every page. main.js loads this unless this browser chose Simple mode.

import { h, icon } from './dom.js';
import { route, setLayout } from './router.js';
import { layout } from './shell.js';
import { pageHeader, emptyState } from './components.js';
import { boot } from './boot.js';

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

// Every icon in something pressable (a link, a button, a tab) does its act once when pointed at or focused, wherever it is drawn.
animateWithin(document.body);
// FinMotion: every FinUI component moves, at the pace of the person's Motion choice and not at all with reduced motion.
motion(document.body);
installMenus();

boot({ top: (me) => ['/', `/users/${me.id}`, '/activity', '/together', '/libraries', '/users', '/playback', `/users/${me.id}/timeline`, '/changelog', '/server', '/server/log', '/recap', '/pipeline'] });
