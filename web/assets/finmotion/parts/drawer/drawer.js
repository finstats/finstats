// FinMotion: drawer. A bottom sheet is held by its head: down it follows the finger and, let go a third of the way or
// flicked, it goes; up it gives like a band. Let go, a spring takes it home from exactly where it was left.

import { play, ended } from '../../core/motion.js';
import { band, unband, letGo } from './plan.js';

/** How far a sheet pulled up past its rest gives, at most. */
const GIVE = 120;

export const part = {
  name: 'drawer',
  selector: '.fui-drawer--bottom:not(.fui-drawer--held)',
  enhance(sheet) {
    const head = sheet.querySelector('.fui-modal__head');
    if (!head) return;
    head.classList.add('fm-drawer__grip');
    let y0 = null, dy = 0, last = null, v = 0;
    const where = (d) => (d > 0 ? d : band(d, GIVE));
    const down = (e) => {
      if (e.button || e.target.closest('button, a, input')) return;
      const at = parseFloat(getComputedStyle(sheet).translate.split(' ')[1]) || 0;   // caught mid-spring: on from there
      sheet.getAnimations().forEach((a) => a.cancel());   // its entrance or a spring home: the finger has it now
      y0 = e.clientY - (at > 0 ? at : unband(at, GIVE)); last = { y: e.clientY, t: e.timeStamp }; v = 0;
      try { head.setPointerCapture(e.pointerId); } catch {}
    };
    const move = (e) => {
      if (y0 == null) return;
      dy = e.clientY - y0;
      if (e.timeStamp > last.t) v = (e.clientY - last.y) / (e.timeStamp - last.t);
      last = { y: e.clientY, t: e.timeStamp };
      sheet.style.translate = `0 ${where(dy)}px`;
    };
    const up = () => {
      if (y0 == null) return;
      y0 = null;
      const from = where(dy); sheet.style.translate = '';
      // Away is FinUI's own close (its ×), once the sheet has gone; home is a spring from where it was let go.
      if (letGo(dy, sheet.offsetHeight, v) === 'close') ended(play(sheet, [{ translate: `0 ${from}px` }, { translate: '0 100%' }], 'glide', { leaving: true })).then(() => { const x = sheet.querySelector('.fui-modal__close'); if (x) x.click(); });
      else play(sheet, [{ translate: `0 ${from}px` }, { translate: '0 0' }], 'drift', { fill: 'none' });
      dy = 0;
    };
    head.addEventListener('pointerdown', down); head.addEventListener('pointermove', move);
    head.addEventListener('pointerup', up); head.addEventListener('pointercancel', up);
    return () => { head.removeEventListener('pointerdown', down); head.removeEventListener('pointermove', move); head.removeEventListener('pointerup', up); head.removeEventListener('pointercancel', up); };
  },
};
