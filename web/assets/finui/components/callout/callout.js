// FinUI: callout. A note on the page that stays until it is dealt with: something to know, something that went well,
// something to watch, something wrong; in words, with what to do about it.

import { h, icon } from '../../core.js';
import { button } from '../button/button.js';

const TONE_ICON = { info: 'info', good: 'check', warning: 'alert', critical: 'alert' };
/** callout({ tone, title, body, action, onDismiss }): `action` is a button (or a link) beside the words. */
export function callout({ tone = 'info', title = null, body = null, action = null, onDismiss = null }) {
  return h('div', { class: ['fui-callout', `fui-callout--${tone}`], role: tone === 'critical' || tone === 'warning' ? 'alert' : 'note' },
    icon(TONE_ICON[tone] || 'info', 16, 'fui-callout__icon'),
    h('div', { class: 'fui-callout__words' }, title ? h('p', { class: 'fui-callout__title' }, title) : null, body ? h('div', { class: 'fui-callout__body' }, body) : null),
    action ? h('div', { class: 'fui-callout__action' }, action) : null,
    onDismiss ? button({ variant: 'icon', class: 'fui-callout__close', 'aria-label': 'Dismiss', onClick: onDismiss }, icon('x', 14)) : null);
}

export const meta = {
  name: 'callout',
  purpose: 'Says something on the page that stays until it is dealt with, in words, with what to do about it.',
  use: 'An update is available, a connection stopped answering, an import finished with warnings, a feature needs a setting. One per page section, at its top.',
  avoid: 'A passing outcome (toast). An error about one field (under the field). Decoration: a callout that says nothing to do or know.',
  variants: ['info', 'good', 'warning', 'critical', 'with a title', 'with an action', 'dismissible'],
  states: [],
  a11y: 'A warning or a critical callout is role="alert", read out when it appears; the rest are role="note". Its tone is said in words, never by colour alone.',
  props: { 'callout({ tone, title, body, action, onDismiss })': '' },
  playground: {
    controls: [
      { key: 'tone', label: 'Tone', choices: [['warning', 'Warning'], ['info', 'Info'], ['good', 'Good'], ['critical', 'Critical']] },
      { key: 'title', label: 'A title', on: true },
      { key: 'action', label: 'An action', on: true },
      { key: 'dismiss', label: 'Dismissible' },
    ],
    render: (o) => callout({ tone: o.tone, title: o.title ? { warning: 'Sonarr is not answering', info: 'finstats 2.2.0 is out', good: 'The import finished', critical: 'The library could not be read' }[o.tone] : null,
      body: { warning: 'Coming up shows what it knew an hour ago.', info: 'Library health and FinUI.', good: '1,284 plays from Jellystat, none twice.', critical: 'Nothing was removed; finstats stopped to keep your data.' }[o.tone],
      action: o.action ? button({ size: 'sm' }, o.tone === 'info' ? 'What changed' : 'Look at it') : null, onDismiss: o.dismiss ? () => {} : null }),
  },
};
