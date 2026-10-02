// FinUI: error. What a view says when it could not load: that it failed, the reason in the server's words, and a way to
// try again. It is an alert, so a screen reader says it as it appears.

import { h, icon } from '../../core.js';
import { button } from '../button/button.js';

export function errorState(err, retry) {
  return h('div', { class: 'fui-error', role: 'alert' }, icon('alert', 18),
    h('div', null, h('p', { class: 'fui-error__title' }, 'Couldn’t load this'), h('p', { class: 'fui-error__text' }, err.message || String(err))),
    retry ? button({ size: 'sm', type: 'button', onClick: retry }, icon('refresh', 14), 'Try again') : null);
}

export const meta = {
  name: 'error',
  purpose: 'Says a view could not load, gives the reason, and offers to try again.',
  use: 'In place of what failed to load, the size of a card or smaller. In a flush card it keeps a margin.',
  avoid: 'An error for a field (fieldError in a form). An error for an empty result (empty). Hiding the reason: the words the server gave are shown.',
  variants: ['with Try again', 'without'],
  states: [],
  a11y: 'role="alert": announced when it appears. The icon is hidden; the title says it.',
  props: { 'errorState(err, retry)': 'err: an Error (its message is shown); retry: a function, which adds Try again' },
  playground: {
    controls: [{ key: 'retry', label: 'A way to try again', on: true }],
    render: (o) => errorState(new Error(o.retry ? 'Jellyfin did not answer within 30 seconds.' : 'The backup is damaged at line 1,204.'), o.retry ? () => {} : null),
  },
};
