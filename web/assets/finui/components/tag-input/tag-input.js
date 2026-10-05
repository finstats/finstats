// FinUI: tag-input. A short list of words typed into one field: Enter or a comma makes the word a tag, a pasted list
// becomes several, Backspace in an empty field takes the last back, and each tag has its own ×.

import { h } from '../../core.js';
import { removableChip } from '../chip/chip.js';
import { addTags } from './plan.js';

/** tagInput({ label, value, onChange, placeholder, max }): `max` stops it at that many, and it says how many. */
export function tagInput({ label, value = [], onChange = () => {}, placeholder = 'Add…', max = Infinity }) {
  let tags = addTags([], value.join(','), { max });
  const field = h('input', { class: 'fui-tag-input__field', type: 'text', placeholder, 'aria-label': label, autocomplete: 'off', enterKeyHint: 'enter' });
  const list = h('span', { class: 'fui-tag-input__tags' });
  const count = Number.isFinite(max) ? h('span', { class: 'fui-tag-input__count', 'aria-live': 'polite' }) : null;
  const el = h('div', { class: 'fui-tag-input', onPointerdown: (e) => { if (e.target === el) { e.preventDefault(); field.focus(); } } }, list, field, count);
  const set = (next) => { const changed = next.join('\n') !== tags.join('\n'); tags = next; draw(); if (changed) onChange([...tags]); };
  function draw() {
    list.replaceChildren(...tags.map((t, i) => removableChip({ label: t, removeLabel: `Remove ${t}`, onRemove: () => { set(tags.filter((_, k) => k !== i)); field.focus(); } })));
    const full = tags.length >= max;
    field.hidden = full;
    if (count) count.textContent = `${tags.length} / ${max}`;
  }
  const take = () => { if (field.value.trim()) { set(addTags(tags, field.value, { max })); field.value = ''; } };
  field.addEventListener('keydown', (e) => {
    if (e.key === 'Enter' || e.key === ',') { e.preventDefault(); take(); }
    else if (e.key === 'Backspace' && !field.value && tags.length) set(tags.slice(0, -1));
  });
  field.addEventListener('paste', (e) => { const text = e.clipboardData && e.clipboardData.getData('text'); if (text && /[,\n]/.test(text)) { e.preventDefault(); set(addTags(tags, text, { max })); } });
  field.addEventListener('blur', take);
  draw();
  return el;
}

export const meta = {
  name: 'tag-input',
  purpose: 'Takes a short list of words in one field, each a tag with its own way out.',
  use: 'Genres to filter by, addresses that count as home, words to look for. Enter or a comma makes a tag; a pasted list makes several; max stops it.',
  avoid: 'Choosing from a fixed list (combobox, several). Long text (a textarea).',
  variants: ['empty', 'with tags', 'at most a number of them'],
  states: ['typing', 'full (the field steps aside)'],
  a11y: 'One text input named by its label; each tag is a removable chip whose × says which tag it removes; the count is announced.',
  props: { 'tagInput({ label, value, onChange, placeholder, max })': 'value: the tags' },
  playground: {
    controls: [
      { key: 'some', label: 'Some already', on: true },
      { key: 'max', label: 'At most four' },
    ],
    render: (o) => h('div', { class: 'fui-tag-input__demo' }, tagInput({ label: 'Genres', value: o.some ? ['Drama', 'Animation'] : [], max: o.max ? 4 : Infinity, placeholder: 'Add a genre…' })),
  },
};
