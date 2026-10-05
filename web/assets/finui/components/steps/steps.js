// FinUI: steps. The way through a wizard: each step numbered, the ones done ticked, the one now marked, the rest to
// come; a step that can be gone to is a button — back to any done, forward only to the next once this one is done.

import { h, icon } from '../../core.js';
import { reachable, stateOf } from './plan.js';

/** steps({ steps: [{ label, detail }], current, done, onGo(i), vertical }) */
export function steps({ steps: list, current = 0, done = [], onGo = null, vertical = false }) {
  return h('ol', { class: ['fui-steps', vertical && 'fui-steps--vertical'], 'aria-label': 'Steps' }, list.map((st, i) => {
    const state = stateOf(i, current, done);
    const mark = h('span', { class: 'fui-steps__mark', 'aria-hidden': 'true' }, state === 'done' ? icon('check', 13) : String(i + 1));
    const words = h('span', { class: 'fui-steps__words' }, h('span', { class: 'fui-steps__label' }, st.label), st.detail ? h('span', { class: 'fui-steps__detail' }, st.detail) : null);
    const go = onGo && i !== current && reachable(i, current, done);
    return h('li', { class: ['fui-steps__step', `is-${state}`], 'aria-current': state === 'current' ? 'step' : null },
      go ? h('button', { type: 'button', class: 'fui-steps__go', onClick: () => onGo(i) }, mark, words) : h('span', { class: 'fui-steps__go' }, mark, words));
  }));
}

const STEPS = [{ label: 'Jellyfin', detail: 'Where it answers' }, { label: 'Sign in', detail: 'An administrator' }, { label: 'History', detail: 'Import what you have' }, { label: 'Done', detail: 'Start counting' }];
export const meta = {
  name: 'steps',
  purpose: 'Shows the way through a wizard: which steps are done, which is now, which are to come.',
  use: 'A setup, an import, anything done in a fixed order of a few steps. onGo lets a step done be gone back to, and the next once this one is done.',
  avoid: 'Views of one thing in any order (tabs). A long form that could be one page.',
  variants: ['across', 'down', 'with a line under each name'],
  states: ['done (ticked)', 'current (aria-current="step")', 'to come'],
  a11y: 'An ordered list named Steps; the step now says aria-current="step"; a step that can be gone to is a button; done and to come are said in words by the mark and the order, never by colour alone.',
  props: { 'steps({ steps, current, done, onGo, vertical })': 'steps: [{ label, detail }]; done: indexes' },
  playground: {
    controls: [
      { key: 'at', label: 'Now at', choices: [['1', 'Sign in'], ['0', 'Jellyfin'], ['3', 'Done']] },
      { key: 'vertical', label: 'Down the side' },
      { key: 'detail', label: 'A line under each', on: true },
    ],
    render: (o) => { const at = Number(o.at); return steps({ steps: STEPS.map((st) => ({ ...st, detail: o.detail ? st.detail : null })), current: at, done: [...Array(at).keys()], onGo: () => {}, vertical: o.vertical }); },
  },
};
