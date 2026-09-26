// The theme this browser chose, applied before the first paint: a classic script in <head>, so it blocks,
// where a module would run after the page had already been painted in the device's colours. Nothing stored
// means "follow the device", which app.css does by itself. state.js reads and writes the same key.
try { const t = localStorage.getItem('finstats.theme'); if (t === 'light' || t === 'dark') document.documentElement.dataset.theme = t; } catch { /* storage blocked: the device decides */ }
