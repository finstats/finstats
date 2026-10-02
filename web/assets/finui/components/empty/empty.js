// FinUI: empty. What a view says when it has nothing to show — why, and the way out when there is one — instead of a
// blank space somebody has to interpret.

import { h, icon } from '../../core.js';
import { button } from '../button/button.js';

export function emptyState(title, text, action) {
  return h('div', { class: 'fui-empty' }, h('p', { class: 'fui-empty__title' }, title), text ? h('p', { class: 'fui-empty__text' }, text) : null, action || null);
}

export const meta = {
  name: 'empty',
  purpose: 'Says that there is nothing here, why, and what to do about it if anything.',
  use: 'Whenever a list, a chart or a page has nothing to show. In a card it loses its frame. fui-empty--chart (and --chart-sm) for a chart with no data.',
  avoid: 'An empty state for an error (error), or for something still loading (skeleton). Blaming the reader: say what is true ("No plays in this range"), not what they did wrong.',
  variants: ['with text', 'with an action', 'in a card', 'chart', 'chart, small'],
  states: [],
  a11y: 'Plain text. An action is a real button or link.',
  props: { 'emptyState(title, text, action)': 'title and text are words; action is a node (a button, or several in fui-empty__buttons)' },
  playground: {
    controls: [
      { key: 'where', label: 'Where', choices: [['page', 'A page'], ['chart', 'A chart']] },
      { key: 'action', label: 'A way out' },
    ],
    render: (o) => o.where === 'chart'
      ? h('div', { class: 'fui-empty--chart fui-empty--chart-sm' }, o.action ? 'No plays in this range. Try all time.' : 'No plays in this range.')
      : emptyState(o.action ? 'No backups yet' : 'No libraries yet', o.action ? 'A backup holds your history and settings, never your Jellyfin key.' : 'Libraries appear after the first sync with Jellyfin.',
        o.action ? button({ variant: 'primary' }, icon('plus', 14), 'Make a backup now') : null),
  },
};
