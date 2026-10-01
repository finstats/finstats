// finui: combobox. One choice (or several) out of a list too long to show: a button that names the choice, and a list
// that opens under it, filtered as you type. multiSelect is the short kind, ticked rather than searched.

import { h, icon, mount } from '../../core.js';

export function combobox({ value = '', onChange, placeholder = 'All users', allLabel = 'All users', load, label = 'User',
                           multiple = false, searchable = true, iconName = 'user' }) {
  let options = [];
  let open = false, activeIdx = 0, filtered = [];
  // Inside, always a list of chosen values; outside, always the comma-separated string a URL holds.
  const split = (v) => String(v || '').split(',').map((x) => x.trim()).filter(Boolean);
  let chosen = split(value);
  const uid = 'cb' + Math.random().toString(36).slice(2, 8);
  const btnLabel = h('span', { class: 'fui-combobox__label' }, placeholder);
  const btn = h('button', { type: 'button', class: 'fui-combobox__button', 'aria-haspopup': 'listbox', 'aria-expanded': 'false', 'aria-label': label },
    iconName ? icon(iconName, 14) : null, btnLabel, icon('chevronDown', 14, 'fui-combobox__caret'));
  const input = h('input', { class: 'fui-combobox__input', type: 'text', placeholder: 'Type to filter…', autocomplete: 'off', spellcheck: false,
    role: 'combobox', 'aria-controls': uid, 'aria-expanded': 'true', 'aria-autocomplete': 'list', 'aria-label': 'Filter ' + label.toLowerCase() + 's' });
  const list = h('ul', { class: 'fui-combobox__list', role: 'listbox', id: uid, tabindex: -1, 'aria-multiselectable': multiple ? 'true' : null });
  const search = searchable ? h('div', { class: 'fui-combobox__search' }, icon('search', 14), input) : null;
  const pop = h('div', { class: 'fui-combobox__pop', hidden: true }, search, list);
  const root = h('div', { class: ['fui-combobox', multiple && 'is-multi'] }, btn, pop);

  const isOn = (v) => (v ? chosen.includes(v) : chosen.length === 0);

  function paintLabel() {
    const named = chosen.map((v) => (options.find((x) => x.value === v) || {}).label).filter(Boolean);
    // One is named; several are the first and a count, so the button never grows with the choice.
    btnLabel.textContent = !chosen.length ? allLabel
      : named.length === 0 ? placeholder
      : named.length === 1 ? named[0]
      : `${named[0]} +${named.length - 1}`;
    root.classList.toggle('has-value', chosen.length > 0);
  }
  function renderList() {
    const q = searchable ? input.value.trim().toLowerCase() : '';
    const all = [{ value: '', label: allLabel }, ...options];
    filtered = q ? options.filter((o) => o.label.toLowerCase().includes(q)) : all;
    activeIdx = Math.min(activeIdx, Math.max(0, filtered.length - 1));
    if (!filtered.length) { mount(list, h('li', { class: 'fui-combobox__empty' }, 'No results')); input.removeAttribute('aria-activedescendant'); return; }
    mount(list, filtered.map((o, i) => h('li', { id: `${uid}-${i}`, role: 'option', class: ['fui-combobox__option', i === activeIdx && 'is-active'],
      'aria-selected': String(isOn(o.value)),
      onPointerdown: (e) => { e.preventDefault(); choose(o); },
      onPointermove: () => { if (activeIdx !== i) { activeIdx = i; renderList(); } } },
      h('span', null, o.label), isOn(o.value) ? icon('check', 14) : null)));
    (searchable ? input : list).setAttribute('aria-activedescendant', `${uid}-${activeIdx}`);
    list.querySelector('.is-active')?.scrollIntoView({ block: 'nearest' });
  }
  function choose(o) {
    const before = chosen.join(',');
    if (!multiple) {
      chosen = o.value ? [o.value] : [];
    } else if (!o.value) {
      chosen = []; // "All" is not one more thing to tick: it is nothing ticked.
    } else {
      chosen = chosen.includes(o.value) ? chosen.filter((v) => v !== o.value) : [...chosen, o.value];
    }
    paintLabel();
    // Ticking several means staying open; choosing one means you are done.
    if (multiple) renderList(); else close(true);
    if (chosen.join(',') !== before) onChange(chosen.join(','));
  }
  function openPop() {
    if (open) return;
    open = true; pop.hidden = false; btn.setAttribute('aria-expanded', 'true');
    input.value = ''; activeIdx = 0; renderList();
    (searchable ? input : list).focus();
    document.addEventListener('pointerdown', outside, true);
  }
  function close(refocus) {
    if (!open) return;
    open = false; pop.hidden = true; btn.setAttribute('aria-expanded', 'false');
    input.value = ''; // search is cleared on close; the button keeps the full selected label
    document.removeEventListener('pointerdown', outside, true);
    if (refocus) btn.focus();
  }
  const outside = (e) => { if (!root.contains(e.target)) close(false); };
  btn.addEventListener('click', () => (open ? close(true) : openPop()));
  input.addEventListener('input', () => { activeIdx = 0; renderList(); });
  const keys = (e) => {
    if (e.key === 'ArrowDown') { e.preventDefault(); activeIdx = Math.min(filtered.length - 1, activeIdx + 1); renderList(); }
    else if (e.key === 'ArrowUp') { e.preventDefault(); activeIdx = Math.max(0, activeIdx - 1); renderList(); }
    else if (e.key === 'Enter' || (!searchable && e.key === ' ')) { e.preventDefault(); if (filtered[activeIdx]) choose(filtered[activeIdx]); }
    // The overlay's own Escape: `shell.js` handles it globally and would step back a page.
    else if (e.key === 'Escape') { e.preventDefault(); e.stopPropagation(); close(true); }
    else if (e.key === 'Tab') close(false);
  };
  input.addEventListener('keydown', keys);
  list.addEventListener('keydown', keys);

  paintLabel();
  Promise.resolve(load()).then((opts) => { options = opts || []; paintLabel(); if (open) renderList(); }).catch(() => {});
  return root;
}

/// A dropdown of a known handful of options, ticked rather than chosen: media types, play methods,
/// which tracker recorded a play. No search — for four options it is only noise.
export function multiSelect({ options, value, onChange, label, allLabel, iconName = null }) {
  return combobox({ value, onChange, label, allLabel, placeholder: allLabel, iconName, multiple: true, searchable: false, load: () => options });
}

const PEOPLE = [{ value: 'u1', label: 'alice' }, { value: 'u2', label: 'bob' }, { value: 'u3', label: 'carol' }, { value: 'u4', label: 'dave' }];
export const meta = {
  name: 'combobox',
  purpose: 'Chooses one or several things from a list too long to show all at once.',
  use: 'Someone out of the household; anything with a search box’s worth of options. multiSelect for a handful you tick (media types, trackers).',
  avoid: 'Two to six options that fit in a row (segmented). A free text field.',
  variants: ['single, searchable', 'several (multiple)', 'multiSelect: ticked, no search'],
  states: ['closed', 'open', 'has a value', 'nothing found'],
  a11y: 'The button says aria-haspopup="listbox" and aria-expanded; the search is role="combobox" with aria-activedescendant; options are role="option" with aria-selected. Arrows, Enter, and Esc to close (which stops there and goes no further back).',
  props: { 'combobox({ value, onChange, placeholder, allLabel, load, label, multiple, searchable, iconName })': 'load() gives [{ value, label }], or a promise of them; the value is a comma-separated string', 'multiSelect({ options, value, onChange, label, allLabel, iconName })': 'a handful, ticked' },
  examples: [
    { name: 'Someone', render: () => combobox({ label: 'User', load: () => PEOPLE, onChange: () => {} }) },
    { name: 'Several, ticked', render: () => multiSelect({ label: 'Type', allLabel: 'All types', iconName: 'film', options: [{ value: 'Movie', label: 'Movies' }, { value: 'Episode', label: 'Episodes' }, { value: 'Audio', label: 'Music' }], value: 'Movie', onChange: () => {} }) },
  ],
};
