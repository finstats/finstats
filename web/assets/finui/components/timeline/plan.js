// FinUI: timeline, the rules that are not drawing — events fall into the local days they happened on, newest day first,
// each day's events newest first. Pure; tested in FinUI's repository.

const dayKey = (t) => { const d = new Date(t); return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`; };
/** [{ day: 'YYYY-MM-DD', events }], from events with `at` (ms). */
export function byDay(events) {
  const days = new Map();
  for (const e of [...events].sort((a, b) => b.at - a.at)) {
    const k = dayKey(e.at);
    if (!days.has(k)) days.set(k, []);
    days.get(k).push(e);
  }
  return [...days].map(([day, list]) => ({ day, events: list }));
}
