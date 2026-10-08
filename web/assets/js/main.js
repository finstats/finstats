// FinStats SPA entry point. A browser wears one of two layouts (Settings → Appearance; theme.js has marked which before
// the first paint), and only that layout's modules are loaded: Simple mode never fetches the half of FinStats it leaves out.

import(document.documentElement.dataset.layout === 'simple' ? './simple/app.js' : './app.js');
