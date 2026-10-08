// How Simple mode draws numbers and apps: a duration as hours and minutes (41:18), a bitrate in Mbps or kbps, and an app as
// a coloured tile by the kind of client it is, an icon of FinUI's own rather than anybody's logo.

import { tile } from '../../finui/components/tile-stat/tile-stat.js';

/** Hours and minutes, as a classic stats home writes a duration: 41:18. */
export const hhmm = (sec) => {
  const m = Math.floor(Math.max(0, Number(sec) || 0) / 60);
  return `${Math.floor(m / 60)}:${String(m % 60).padStart(2, '0')}`;
};

/** A bitrate in bits per second, in the larger unit that keeps it readable. */
export const rate = (bps) => (!bps ? '–' : bps >= 1e6 ? `${(bps / 1e6).toFixed(1)} Mbps` : `${Math.round(bps / 1e3)} kbps`);

/** The kind of client an app is, by its name: a television, a phone of either kind, a browser, a music player. */
export function platformOf(client = '') {
  const c = String(client).toLowerCase();
  if (/android ?tv|google ?tv|roku|fire ?tv|tizen|webos|kodi|xbox|playstation|apple ?tv|tvos|chromecast|living room/.test(c)) return { icon: 'tv', tone: 'accent' };
  if (/android/.test(c)) return { icon: 'smartphone', tone: 'good' };
  if (/ios|iphone|ipad|swiftfin|infuse/.test(c)) return { icon: 'smartphone', tone: 'series-4' };
  if (/web|chrome|firefox|safari|edge|browser/.test(c)) return { icon: 'globe', tone: 'critical' };
  return { icon: /finamp|music|audio|symfonium|feishin|amperfy/.test(c) ? 'music' : 'monitor', tone: 'neutral' };
}

/** An app's tile, `size` pixels square. */
export const platformTile = (client, size = 48) => tile({ ...platformOf(client), size, label: client || 'Unknown app' });
