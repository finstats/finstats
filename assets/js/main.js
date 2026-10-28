// FinStats SPA entry point. A browser wears one of two layouts (Settings → Appearance; theme.js has marked which before
// the first paint), and only that layout's modules are loaded: Simple mode never fetches the half of FinStats it leaves out.
// The words come first: every page module may say t('…') as it is evaluated, so the catalogue of the language this
// browser reads in is in use before either layout is imported.

import { loadLanguage } from './i18n.js';

await loadLanguage();
import(document.documentElement.dataset.layout === 'simple' ? './simple/app.js' : './app.js');
