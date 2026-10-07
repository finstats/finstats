// FinMotion: stat-tile. A tile's number rolls like an odometer when it changes: a page redrawn with a new figure under the
// same label rolls from the one it showed before, so you see by how much. A tile marked fm-roll rolls up from zero the
// first time it is seen.

import { odometer } from '../../components/odometer/odometer.js';

/** The last figure each label showed, on this page and the ones after it. */
const shown = new Map();

export const part = {
  name: 'stat-tile',
  selector: '.fui-stat-tile',
  enhance(tile) {
    const value = tile.querySelector('.fui-stat-tile__value'), label = tile.querySelector('.fui-stat-tile__label');
    // Only a figure written as text rolls: a value FinUI was given as something else is left as it is.
    if (!value || value.children.length || !/\d/.test(value.textContent)) return;
    const now = value.textContent, key = label ? label.textContent : null, before = key != null ? shown.get(key) : undefined;
    if (key != null) shown.set(key, now);
    const from = tile.classList.contains('fm-roll') ? now.replace(/\d/g, '0') : before != null && before !== now ? before : null;
    if (from == null) return;
    value.replaceChildren(odometer({ value: now, from }));
  },
};
