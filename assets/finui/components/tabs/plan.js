// FinUI: tabs, the rules that are not drawing: where the keys move between tabs: the arrows to the next or the one
// before (round at the ends), Home and End to the ends, a tab that is off stepped over. Pure; tested in FinUI's repository.

/** The tab the key moves to from `at`, among tabs whose `off` says which are unavailable; null for a key that does not move. */
export function nextTab(at, key, off) {
  const n = off.length, on = (i) => !off[i];
  const walk = (from, dir) => { for (let k = 1; k <= n; k++) { const i = (from + dir * k + n * k) % n; if (on(i)) return i; } return at; };
  if (key === 'ArrowRight' || key === 'ArrowDown') return walk(at, 1);
  if (key === 'ArrowLeft' || key === 'ArrowUp') return walk(at, -1);
  if (key === 'Home') return on(0) ? 0 : walk(0, 1);
  if (key === 'End') return on(n - 1) ? n - 1 : walk(n - 1, -1);
  return null;
}
