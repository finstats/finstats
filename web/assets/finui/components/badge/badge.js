// FinUI: badge. A small label beside something: a pill (with a dot for what is live or paused), a count, or a status
// line — an icon and a few words in one of four tones. It says; it is never pressed.

import { h, icon } from '../../core.js';

/** badge({ dot, live, paused, count, outlined, class }, ...children): a pill. */
export function badge({ dot = false, live = false, paused = false, count = false, class: extra = null } = {}, ...children) {
  return h('span', { class: [count ? 'fui-badge--count' : 'fui-badge', live && 'fui-badge--live', paused && 'fui-badge--paused', extra] }, dot || live || paused ? h('span', { class: 'fui-badge__dot' }) : null, children);
}

const TONE_ICON = { good: 'check', warning: 'alert', critical: 'alert', info: 'info' };
/** status({ tone, icon, line, outlined, class }, ...children): an icon and words, coloured by tone. `line` lets a sentence wrap. */
export function status({ tone = 'info', icon: name = null, line = false, outlined = false, class: extra = null } = {}, ...children) {
  return h('span', { class: ['fui-badge--status', `fui-badge--${tone}`, line && 'fui-badge--line', outlined && 'fui-badge--outlined', extra] }, icon(name || TONE_ICON[tone] || 'info', 13), children);
}

export const meta = {
  name: 'badge',
  purpose: 'A small label that says something about what it sits beside: a state, a count, how something went.',
  use: 'A pill for a state (Direct play, Live, Paused). A status for how something went (Connected, Failed), in the tone that says so. A count beside a heading.',
  avoid: 'A badge that does something when pressed (a chip, or a button). Colour as the only signal: a status always has its icon and words.',
  variants: ['pill', 'pill with a dot', 'live (the dot breathes)', 'paused', 'count', 'status: good, warning, critical, info', 'status line (wraps)', 'outlined status'],
  states: ['live respects prefers-reduced-motion'],
  a11y: 'Plain text with an icon hidden from screen readers; the words carry the meaning in every tone.',
  props: { 'badge({ dot, live, paused, count })': 'a pill; children are its words', 'status({ tone, icon, line, outlined })': "tone: 'good' | 'warning' | 'critical' | 'info'; icon overrides the tone's own" },
  playground: {
    controls: [
      { key: 'pill', label: 'Pill', choices: [['plain', 'Plain'], ['dot', 'Dot'], ['live', 'Live'], ['paused', 'Paused'], ['count', 'Count']] },
      { key: 'tone', label: 'Status', choices: [['good', 'Good'], ['warning', 'Warning'], ['critical', 'Critical'], ['info', 'Info']] },
      { key: 'outlined', label: 'Outlined status' },
      { key: 'line', label: 'A status that is a sentence' },
    ],
    render: (o) => h('div', { class: 'fui-badge__demo' },
      badge({ [o.pill]: o.pill !== 'plain' }, o.pill === 'count' ? '12' : o.pill === 'live' ? 'Live' : o.pill === 'paused' ? 'Paused' : 'Direct play'),
      status({ tone: o.tone, outlined: o.outlined, line: o.line }, o.line ? 'Restored 1,204 plays from the backup of 3 March, and 18 that were already here were left alone.' : { good: 'Connected', warning: 'Slow to answer', critical: 'Failed', info: 'Update available' }[o.tone])),
  },
};
