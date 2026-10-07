// FinUI: copy. A small button beside a value that puts it on the clipboard and says, for two seconds, that it did.

import { h, icon, mount } from '../../core.js';

export function copyButton(text, label = 'Copy') {
  let timer = null;
  const btn = h('button', { type: 'button', class: 'fui-copy__button', 'aria-label': label, title: label }, icon('copy', 13));
  const note = h('span', { class: 'fui-copy__note', 'aria-live': 'polite' });
  btn.addEventListener('click', async (e) => {
    e.stopPropagation();
    let ok = true;
    try { await navigator.clipboard.writeText(text); } catch { ok = false; }
    btn.replaceChildren(icon(ok ? 'check' : 'x', 13));
    btn.classList.toggle('is-ok', ok);
    note.textContent = ok ? 'Copied' : 'Copy failed. Select the text instead';
    clearTimeout(timer);
    timer = setTimeout(() => { btn.replaceChildren(icon('copy', 13)); btn.classList.remove('is-ok'); note.textContent = ''; }, 2000);
  });
  return h('span', { class: 'fui-copy' }, btn, note);
}

export const meta = {
  name: 'copy',
  purpose: 'Copies a value (an address, a key, a path) and says that it did.',
  use: 'Beside a value somebody will paste elsewhere. fui-copy__row keeps the value and the button on one line where they fit.',
  avoid: 'Copying something the reader cannot see. A copy button as the only way to the value: it is also selectable text.',
  variants: ['button', 'in a row with its value'],
  states: ['copied (a tick, "Copied")', 'failed ("select the text instead")'],
  a11y: 'The button has an aria-label; what happened is said in an aria-live note.',
  props: { 'copyButton(text, label)': 'text is what is copied; label names the button' },
  playground: {
    controls: [{ key: 'value', label: 'Beside its value', on: true }],
    render: (o) => o.value
      ? h('span', { class: 'fui-copy__row' }, h('code', { class: 'mono' }, 'http://192.168.1.10:8096'), copyButton('http://192.168.1.10:8096', 'Copy address'))
      : copyButton('http://192.168.1.10:8096', 'Copy address'),
  },
};
