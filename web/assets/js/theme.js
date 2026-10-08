// The theme this browser chose, applied before the first paint: a classic script in <head>, so it blocks,
// where a module would run after the page had already been painted in the device's colours. Nothing stored
// means "follow the device", which app.css does by itself. state.js reads and writes the same key.
try { const t = localStorage.getItem('finstats.theme'); if (t === 'light' || t === 'dark') document.documentElement.dataset.theme = t; } catch { /* storage blocked: the device decides */ }
// The layout this browser wears (state.js), set here for the same reason: Simple mode's look is in the stylesheet, under
// [data-layout="simple"], and main.js loads that layout's modules from it.
// Its stylesheet is written here, parser-inserted after app.css, so it holds the first paint as the others do and a
// FinStats browser never fetches it.
try {
  if (localStorage.getItem('finstats.layout') === 'simple') {
    document.documentElement.dataset.layout = 'simple';
    document.write('<link rel="stylesheet" href="/assets/simple-tokens.css"><link rel="stylesheet" href="/assets/simple.css">');
  }
} catch { /* storage blocked: FinStats */ }
