// FinUI: progress, the rules that are not drawing — the time left worked out from the rate so far (never guessed before
// anything has moved), and said in the words a person uses. Pure; tested in FinUI's repository.

/** Seconds left at `fraction` done after `elapsed` seconds; null with nothing done yet. */
export function remaining({ fraction, elapsed }) {
  if (!(fraction > 0)) return null;
  if (fraction >= 1) return 0;
  return Math.round((elapsed * (1 - fraction)) / fraction);
}
const plural = (n, one, many) => `${n} ${n === 1 ? one : many}`;
/** "about 3 minutes left", "a few seconds left", "done"; null for no estimate. */
export function timeLeft(seconds) {
  if (seconds == null) return null;
  if (seconds <= 0) return 'done';
  if (seconds < 15) return 'a few seconds left';
  if (seconds < 60) return 'under a minute left';
  if (seconds < 3600) { const m = Math.round(seconds / 60); return m === 1 ? 'about a minute left' : `about ${m} minutes left`; }
  const hours = Math.floor(seconds / 3600), minutes = Math.round((seconds % 3600) / 60);
  return `about ${plural(hours, 'hour', 'hours')}${minutes ? ` ${plural(minutes, 'minute', 'minutes')}` : ''} left`;
}
