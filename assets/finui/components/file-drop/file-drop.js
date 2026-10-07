// FinUI: file-drop. A place to drop files, or press to choose them: what it takes is said, what it cannot take is
// refused with the reason, and each file it holds shows its size, how far along it is, and a way to take it out.

import { h, icon } from '../../core.js';
import { accepts, fileSize } from './plan.js';

let seq = 0;
/** fileDrop({ label, help, accept, multiple, maxSize, onFiles(files) }) → the drop, with .show(rows) to list files:
 *  rows [{ name, size, progress (0–1, or null), error, onRemove }]. */
export function fileDrop({ label = 'Drop a file here', help = null, accept = '', multiple = false, maxSize = Infinity, onFiles = () => {} } = {}) {
  const id = `fui-file-drop-${++seq}`;
  const input = h('input', { class: 'fui-file-drop__input', type: 'file', id, accept: accept || null, multiple, 'aria-describedby': help ? `${id}-help` : null });
  const list = h('ul', { class: 'fui-file-drop__list' });
  // The input is inside its label, so the label needs no for: the same drop drawn again is the same markup.
  const zone = h('label', { class: 'fui-file-drop__zone' }, input, icon('upload', 22),
    h('span', { class: 'fui-file-drop__label' }, label), h('span', { class: 'fui-file-drop__or' }, multiple ? 'or press to choose files' : 'or press to choose one'),
    help ? h('span', { class: 'fui-file-drop__help', id: `${id}-help` }, help) : null);
  const el = h('div', { class: 'fui-file-drop' }, zone, list);
  const take = (files) => {
    const all = [...files].slice(0, multiple ? Infinity : 1);
    const why = (f) => (!accepts(f, accept) ? `Not a kind it takes (${accept})` : f.size > maxSize ? `Larger than ${fileSize(maxSize)}` : null);
    onFiles(all.filter((f) => !why(f)), all.filter(why).map((f) => ({ file: f, reason: why(f) })));
  };
  input.addEventListener('change', () => { take(input.files || []); input.value = ''; });
  for (const t of ['dragenter', 'dragover']) zone.addEventListener(t, (e) => { e.preventDefault(); zone.classList.add('is-over'); });
  for (const t of ['dragleave', 'drop']) zone.addEventListener(t, () => zone.classList.remove('is-over'));
  zone.addEventListener('drop', (e) => { e.preventDefault(); take((e.dataTransfer && e.dataTransfer.files) || []); });
  el.show = (rows) => list.replaceChildren(...rows.map((r) => h('li', { class: ['fui-file-drop__file', r.error && 'is-error'] },
    icon(r.error ? 'alert' : r.progress === 1 ? 'check' : 'database', 15),
    h('span', { class: 'fui-file-drop__name trunc' }, r.name),
    h('span', { class: 'fui-file-drop__size' }, r.error || (r.progress != null && r.progress < 1 ? `${Math.round(r.progress * 100)}% of ${fileSize(r.size)}` : fileSize(r.size))),
    r.progress != null && r.progress < 1 && !r.error ? h('span', { class: 'fui-file-drop__bar', role: 'progressbar', 'aria-valuemin': '0', 'aria-valuemax': '100', 'aria-valuenow': String(Math.round(r.progress * 100)), 'aria-label': `${r.name} uploading` }, (() => { const f = h('span'); f.style.width = `${r.progress * 100}%`; return f; })()) : null,
    r.onRemove ? h('button', { type: 'button', class: 'fui-file-drop__remove', 'aria-label': `Remove ${r.name}`, onClick: r.onRemove }, icon('x', 13)) : null)));
  return el;
}

export const meta = {
  name: 'file-drop',
  purpose: 'Takes files dropped on it or chosen by pressing it, says what it cannot take and why, and shows each file as it goes.',
  use: 'An import (a tracker’s backup), a restore, a poster. accept and maxSize decide what it takes; onFiles hands over the files it took and those it refused with a reason; show() lists them with progress.',
  avoid: 'A form field that only ever takes one small file nobody drags (a plain file input in a field is enough).',
  variants: ['one file', 'several', 'with files on their way', 'a file refused'],
  states: ['a file dragged over it', 'uploading (a bar)', 'done', 'refused'],
  a11y: 'A real file input inside the <label>: the keyboard reaches it and Space opens the chooser. Each file says its state in words; a bar is role="progressbar"; remove buttons name their file.',
  props: { 'fileDrop({ label, help, accept, multiple, maxSize, onFiles })': 'onFiles(taken, refused[{ file, reason }]) → the drop, with .show(rows)' },
  playground: {
    controls: [
      { key: 'multiple', label: 'Several files' },
      { key: 'files', label: 'Files on their way', on: true },
      { key: 'refused', label: 'One refused' },
    ],
    render: (o) => {
      const drop = fileDrop({ label: o.multiple ? 'Drop backups here' : 'Drop a backup here', help: 'A Jellystat .jsonl, or a .jsonl.gz, up to 2 GB.', accept: '.jsonl,.jsonl.gz', multiple: o.multiple, maxSize: 2 * 1024 ** 3 });
      const rows = [];
      if (o.files) rows.push({ name: 'jellystat-backup-2026-09.jsonl.gz', size: 412 * 1024 ** 2, progress: 0.64, onRemove: () => {} }, { name: 'jellystat-backup-2026-08.jsonl', size: 96 * 1024 ** 2, progress: 1, onRemove: () => {} });
      if (o.refused) rows.push({ name: 'holiday.mkv', size: 4.1 * 1024 ** 3, error: 'Not a kind it takes (.jsonl,.jsonl.gz)', onRemove: () => {} });
      drop.show(rows);
      return h('div', { class: 'fui-file-drop__demo' }, drop);
    },
  },
};
