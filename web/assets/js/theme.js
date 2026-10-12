// The theme this browser chose, applied before the first paint: a classic script in <head>, so it blocks,
// where a module would run after the page had already been painted in the device's colours. Nothing stored
// means "follow the device", which app.css does by itself. state.js reads and writes the same key.
try { const t = localStorage.getItem('finstats.theme'); if (t === 'light' || t === 'dark') document.documentElement.dataset.theme = t; } catch { /* storage blocked: the device decides */ }
// The layout this browser wears (state.js), set here for the same reason: Simple mode's look is in the stylesheet, under
// [data-layout="simple"], and main.js loads that layout's modules from it.
// Its stylesheet is written here, parser-inserted after app.css, so it holds the first paint as the others do and a
// FinStats browser never fetches it.
// A public profile (public.html, data-page="public") stands outside the app, and is seen as anyone sees it.
try {
  if (localStorage.getItem('finstats.layout') === 'simple' && document.documentElement.dataset.page !== 'public') {
    document.documentElement.dataset.layout = 'simple';
    document.write('<link rel="stylesheet" href="/assets/simple-tokens.css"><link rel="stylesheet" href="/assets/simple.css">');
  }
} catch { /* storage blocked: FinStats */ }
// The words of the language this browser reads in (i18n.js decides the same way): fetched while the rest of the page
// loads, so main.js finds them already here. <html lang> is set now, for the first paint's hyphenation and screen readers.
try {
  const tags = ['en-GB', 'en-US', 'nb-NO', 'nn-NO'];
  let tag = localStorage.getItem('finstats.language');
  if (!tags.includes(tag)) {
    tag = 'en-GB';
    for (const raw of navigator.languages || [navigator.language]) {
      const t = String(raw || '').toLowerCase(), l = t.split('-')[0];
      const hit = tags.find((x) => x.toLowerCase() === t) || (l === 'nb' || l === 'no' ? 'nb-NO' : l === 'nn' ? 'nn-NO' : l === 'en' ? 'en-GB' : null);
      if (hit) { tag = hit; break; }
    }
  }
  document.documentElement.lang = tag;
  document.write('<link rel="preload" href="/assets/i18n/' + tag + '.json" as="fetch" crossorigin="anonymous">');
} catch { /* storage blocked: i18n.js decides by itself */ }
