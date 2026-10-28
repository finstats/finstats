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
// The words of the language this browser reads in (i18n.js decides the same way, and LANGUAGES there is this list): fetched
// while the rest of the page loads, so main.js finds them already here, with British English beside a language nobody here
// checks (what it lacks is said in that). <html lang> is set now, for the first paint's hyphenation and screen readers.
try {
  const tags = ['en-GB', 'en-US', 'nb-NO', 'nn-NO', 'nn-NO-synnmore', 'da-DK', 'de-DE', 'es-ES', 'fi-FI', 'fr-FR', 'hi-IN', 'is-IS', 'it-IT', 'ja-JP', 'ko-KR',
    'nl-NL', 'pl-PL', 'pt-BR', 'ru-RU', 'sv-SE', 'tr-TR', 'uk-UA', 'zh-CN', 'zh-TW'];
  const checked = tags.slice(0, 5);
  const of = (raw) => {
    const t = String(raw || '').toLowerCase(), [l, ...rest] = t.split('-');
    const exact = tags.find((x) => x.toLowerCase() === t);
    if (exact) return exact;
    if (l === 'nb' || l === 'no') return 'nb-NO';
    if (l === 'nn') return 'nn-NO';
    if (l === 'en') return rest.length ? 'en-GB' : 'en-US';
    if (l === 'zh') return rest.some((p) => ['hant', 'tw', 'hk', 'mo'].includes(p)) ? 'zh-TW' : 'zh-CN';
    return tags.find((x) => x.split('-')[0].toLowerCase() === l) || null;
  };
  let tag = localStorage.getItem('finstats.language');
  if (!tags.includes(tag)) {
    tag = 'en-US';
    for (const raw of navigator.languages || [navigator.language]) { const hit = of(raw); if (hit) { tag = hit; break; } }
  }
  document.documentElement.lang = tag;
  for (const t of checked.includes(tag) ? [tag] : [tag, 'en-GB']) document.write('<link rel="preload" href="/assets/i18n/' + t + '.json" as="fetch" crossorigin="anonymous">');
} catch { /* storage blocked: i18n.js decides by itself */ }
