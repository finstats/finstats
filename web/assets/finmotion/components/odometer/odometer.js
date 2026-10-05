// FinMotion: odometer. A number that changes rolls its digits like a film counter, the rightmost quickest and each on its
// own spring, so you see by how much it moved and not only to what. What it says is plain text for everyone; the reels are
// drawing, hidden from assistive technology.

import { h } from '../../core/dom.js';
import { play, gap, whenSeen } from '../../core/motion.js';
import { reels } from './plan.js';

/** A reel's place for a digit value v (−10 to 19: a roll may go one round past either end). */
const at = (v) => `translateY(calc(${-(v + 10)} * var(--fm-odometer-line)))`;
const REEL = Array.from({ length: 30 }, (_, k) => String(((k % 10) + 10) % 10));

/** odometer({ value, from }): `value` as text ("1,284", "3h 20m"); `from`, if given, is rolled up from once the odometer is
 *  first seen. `.set(value)` rolls to a new one. */
export function odometer({ value, from = null }) {
  const said = h('span', { class: 'fm-odometer__said' });
  const drawn = h('span', { class: 'fm-odometer__reels', 'aria-hidden': 'true' });
  const el = h('span', { class: 'fm-odometer' }, said, drawn);
  let now = String(value);
  const show = (was, next, roll) => {
    said.textContent = next;
    const plan = reels(was, next), n = plan.length;
    drawn.replaceChildren(...plan.map((r, i) => {
      if (r.kind === 'char') return h('span', { class: 'fm-odometer__char' }, r.char);
      const reel = h('span', { class: 'fm-odometer__reel' }, REEL.map((d) => h('span', null, d)));
      reel.style.transform = at(((r.to % 10) + 10) % 10);
      if (roll && r.from !== r.to) play(reel, [{ transform: at(r.from) }, { transform: at(r.to) }], i === n - 1 ? 'snap' : 'drift', { delay: (n - 1 - i) * gap('drift', el) * 1.5, fill: 'backwards' });
      return h('span', { class: 'fm-odometer__col' }, reel);
    }));
  };
  show(now, now, false);
  el.set = (next) => { const was = now; now = String(next); show(was, now, true); };
  if (from != null) whenSeen(el, () => show(String(from), now, true));
  return el;
}
