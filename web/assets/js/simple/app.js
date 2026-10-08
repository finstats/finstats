// Simple mode: FinStats laid out the way a classic Plex stats server is, for people who come from one. main.js loads this
// for a browser that chose it (Settings → Appearance), and nothing of the FinStats layout: no menu, no FinMotion, and
// none of the pages this layout leaves out (Recap, Together, Pipeline, Security, Server and the rest are no page here).
// Pages it shares with the other layout (signing in, Settings, licences, patch notes) are fetched when they are opened.

import { icon } from '../dom.js';
import { route, setLayout } from '../router.js';
import { pageHeader, emptyState } from '../components.js';
import { boot } from '../boot.js';
import { button } from '../../finui/components/button/button.js';
import { layout } from './shell.js';
import home, { prefetchHome } from './pages/home.js';
import history from './pages/history.js';

/** A page fetched when it is first opened: `load()` answers its function, drawn only if the reader is still there. */
const later = (load) => (ctx) => load().then((page) => { if (!ctx.signal.aborted) page(ctx); });
const settings = later(() => import('../pages/settings.js').then((m) => m.default));

route('/setup', later(() => import('../pages/auth.js').then((m) => m.setupPage)), { bare: true });
route('/login', later(() => import('../pages/auth.js').then((m) => m.loginPage)), { bare: true });
route('/', home, { prefetch: prefetchHome });
route('/history', history);
route('/settings', settings);
route('/settings/:section', settings);
route('/settings/:section/:task', settings);
route('/changelog', later(() => import('../pages/changelog.js').then((m) => m.default)));
route('/licenses', later(() => import('../pages/licenses.js').then((m) => m.default)));
route('*', (ctx) => {
  ctx.title('Not found');
  ctx.root.append(pageHeader('Page not found'), emptyState('There’s nothing at this address.', 'Simple mode has the home, libraries, users, history and graphs; the rest of FinStats is in its own layout (Settings → Appearance).',
    button({ href: '/' }, icon('home', 14), 'Go home')));
});

setLayout(layout);

boot({ top: () => ['/'] });
