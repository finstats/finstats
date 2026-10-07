// FinUI: toast. A short message that comes and goes at the edge of the window (something was saved, something
// happened) with at most one thing to do (Undo). It stays as long as it takes to read, waits while pointed at or
// focused, and a few are shown at once, newest first.

import { h, icon } from '../../core.js';
import { button } from '../button/button.js';
import { lifetime, shown } from './plan.js';

const TONE_ICON = { info: 'info', good: 'check', warning: 'alert', critical: 'alert' };
let region = null, queue = [], seq = 0;

/** One toast as it looks, held in a page or in the stack. */
export function toastCard({ text, tone = 'info', action = null, onAction = null, onClose = null }) {
  return h('div', { class: ['fui-toast', `fui-toast--${tone}`], role: tone === 'critical' ? 'alert' : 'status' },
    icon(TONE_ICON[tone] || 'info', 16, 'fui-toast__icon'), h('span', { class: 'fui-toast__text' }, text),
    action ? button({ size: 'sm', variant: 'ghost', class: 'fui-toast__action', onClick: onAction }, action) : null,
    button({ variant: 'icon', class: 'fui-toast__close', 'aria-label': 'Dismiss', onClick: onClose }, icon('x', 14)));
}

function paint() {
  if (!region) { region = h('div', { class: 'fui-toast__region', 'aria-live': 'polite' }); document.body.append(region); }
  region.replaceChildren(...shown(queue, 3).map((t) => t.el));
}
/** toast({ text, tone, action, onAction, sticky }) → dismiss(). `tone`: info, good, warning, critical. */
export function toast({ text, tone = 'info', action = null, onAction = () => {}, sticky = false }) {
  const t = { id: ++seq };
  let timer = null, left = lifetime({ text, action, sticky }), since = 0;
  const dismiss = () => { clearTimeout(timer); queue = queue.filter((x) => x !== t); paint(); };
  t.el = toastCard({ text, tone, action, onClose: dismiss, onAction: () => { onAction(); dismiss(); } });
  // A toast being read or reached for waits.
  const run = () => { if (Number.isFinite(left)) { since = Date.now(); timer = setTimeout(dismiss, left); } };
  const hold = () => { clearTimeout(timer); left -= Date.now() - since; };
  t.el.addEventListener('pointerenter', hold); t.el.addEventListener('pointerleave', run);
  t.el.addEventListener('focusin', hold); t.el.addEventListener('focusout', run);
  queue.push(t); paint(); run();
  return dismiss;
}

export const meta = {
  name: 'toast',
  purpose: 'Says briefly at the edge of the window that something happened, with at most one thing to do about it.',
  use: 'The outcome of something done: saved, copied, removed (with Undo), a backup written. It stays as long as it takes to read, longer with an action, and waits while it is pointed at or focused; sticky ones wait to be closed.',
  avoid: 'Anything that must be read or answered (a callout on the page, a modal). An error about a field (under the field). More than one sentence.',
  variants: ['info', 'good', 'warning', 'critical', 'with an action (Undo)', 'sticky'],
  states: ['shown', 'paused (pointed at or focused)', 'dismissed'],
  a11y: 'The stack is an aria-live="polite" region; a critical toast is role="alert". Each has a Dismiss button; the action is a real button.',
  props: {
    'toast({ text, tone, action, onAction, sticky })': '→ dismiss()',
    'toastCard({ text, tone, action, onAction, onClose })': 'one toast, held in a page',
  },
  playground: {
    controls: [
      { key: 'tone', label: 'Tone', choices: [['good', 'Good'], ['info', 'Info'], ['warning', 'Warning'], ['critical', 'Critical']] },
      { key: 'action', label: 'An action (Undo)', on: true },
    ],
    render: (o) => {
      const text = { good: 'Big Buck Bunny is off your watchlist.', info: 'The library is being read.', warning: 'Jellyfin is slow to answer.', critical: 'The backup could not be written.' }[o.tone];
      return h('div', { class: 'fui-toast__demo' },
        toastCard({ text, tone: o.tone, action: o.action ? (o.tone === 'critical' ? 'Retry' : 'Undo') : null }),
        button({ size: 'sm', onClick: () => toast({ text, tone: o.tone, action: o.action ? 'Undo' : null }) }, 'Show it for real'));
    },
  },
};
