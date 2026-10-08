// FinUI: code-block. Code to read and take: one line of it (a command) or panes of it (JavaScript, CSS, HTML), with
// a way to copy what is shown. Code reaches the page as text, never as markup.

import { h, mount } from '../../core.js';
import { copyButton } from '../copy/copy.js';
import { segmented } from '../segmented/segmented.js';

/** One line (a command to paste) and its copy button. `.set(text)` puts another line in its place. */
export function codeLine(text, { label = 'Copy the command', class: extra = null } = {}) {
  const code = h('code', { class: 'fui-code-block__text' });
  const copySlot = h('span', { class: 'fui-code-block__copy' });
  const el = h('div', { class: ['fui-code-block--line', extra] }, code, copySlot);
  el.set = (t) => { code.textContent = t; mount(copySlot, copyButton(t, label)); };
  el.set(text);
  return el;
}

/**
 * Panes of code, one shown at a time: `panes` [{ key, label, text, wrap }] (`wrap`: long lines wrap, as HTML reads
 * best), a choice between them when there are several, a way to copy the one shown, and a `note` on how to use it.
 * `fill`: as tall as its container, the pane scrolling inside it.
 */
export function codeBlock({ panes, note = null, label = 'Code', value = null, fill = false, class: extra = null }) {
  let shown = value || panes[0].key;
  const copySlot = h('span');
  const el = panes.map((p) => h('pre', { class: ['fui-code-block__pane', p.wrap && 'fui-code-block__pane--wrap'], dataset: { lang: p.key }, tabindex: 0, hidden: p.key !== shown }, p.text));
  const show = () => {
    el.forEach((pre) => { pre.hidden = pre.dataset.lang !== shown; });
    const p = panes.find((x) => x.key === shown);
    mount(copySlot, copyButton(p.text, `Copy the ${p.label}`));
  };
  show();
  return h('section', { class: ['fui-code-block', fill && 'fui-code-block--fill', extra], 'aria-label': label },
    h('div', { class: 'fui-code-block__bar' },
      panes.length > 1 ? segmented({ label, size: 'sm', value: shown, options: panes.map((p) => ({ value: p.key, label: p.label })), onChange: (v) => { shown = v; show(); } }) : h('span', { class: 'fui-code-block__name' }, panes[0].label),
      copySlot),
    note ? h('p', { class: 'fui-code-block__note' }, note) : null,
    el);
}

const JS = "const read = button({ variant: 'primary' }, 'Read again');\ndocument.body.append(read);\n";
const CSS = '.reader { display: grid; gap: 12px; }\n';
export const meta = {
  name: 'code-block',
  purpose: 'Shows code to read and take: a command on one line, or panes of code to choose between, each with a way to copy it.',
  use: 'An install line beside the words that explain it; the JavaScript, CSS and HTML of an example. note says how to use what is shown.',
  avoid: 'Code to run on the page (it is only ever text). Words that are not code: a sentence in a pane reads as something to paste.',
  variants: ['one line', 'panes with a choice between them', 'with a note', 'filling its container'],
  states: ['copied (the copy button says so)'],
  a11y: 'Each pane is focusable (tabindex 0) so a keyboard can scroll it; the choice is a segmented radio group; the copy button is named for what it copies.',
  props: {
    'codeLine(text, { label, class })': 'one line; .set(text) changes it',
    'codeBlock({ panes, note, label, value, fill, class })': 'panes: [{ key, label, text, wrap }]',
  },
  playground: {
    controls: [
      { key: 'kind', label: 'Kind', choices: [['line', 'One line'], ['panes', 'Panes']] },
      { key: 'note', label: 'A note on how to use it' },
    ],
    render: (o) => (o.kind === 'line'
      ? h('div', { class: 'fui-code-block__demo' }, o.note ? h('p', null, 'Copies FinUI into ./finui:') : null, codeLine('curl -fsSL https://finui.finstats.no/install.sh | sh'))
      : codeBlock({ panes: [{ key: 'js', label: 'JavaScript', text: JS }, { key: 'css', label: 'CSS', text: CSS }], note: o.note ? 'Import the JavaScript and add the CSS.' : null })),
  },
};
