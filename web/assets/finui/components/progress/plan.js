// FinUI: progress, the rules that are not drawing: the time left worked out from the rate so far (never guessed before
// anything has moved), and said in the words a person uses. Pure; tested in FinUI's repository.

import { say } from '../../i18n.js';

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
  if (seconds <= 0) return say('progress.done', 'done');
  if (seconds < 15) return say('progress.seconds_left', 'a few seconds left');
  if (seconds < 60) return say('progress.under_a_minute', 'under a minute left');
  if (seconds < 3600) { const m = Math.round(seconds / 60); return m === 1 ? say('progress.a_minute_left', 'about a minute left') : say('progress.minutes_left', `about ${m} minutes left`, { count: m }); }
  const hours = Math.floor(seconds / 3600), minutes = Math.round((seconds % 3600) / 60);
  const h = say('progress.hours', plural(hours, 'hour', 'hours'), { count: hours });
  const time = minutes ? say('progress.hours_minutes', `${h} ${plural(minutes, 'minute', 'minutes')}`, { hours: h, minutes: say('progress.minutes', plural(minutes, 'minute', 'minutes'), { count: minutes }) }) : h;
  return say('progress.about_left', `about ${time} left`, { time });
}
