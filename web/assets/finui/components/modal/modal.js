// FinUI: modal. A dialog over the page for one task — confirm, edit, look closer — closed by its ×, by Esc or by a
// click outside, with the focus held inside it and given back to what opened it. Dialogs stack: only the one on top
// answers a key, and its Esc goes no further, so a page behind never steps back.

import { h, icon } from '../../core.js';
import { button } from '../button/button.js';

const modalStack = [];
/** X button, Esc and click-outside all close it; focus returns to the trigger. */
export function openModal({ title, body, wide = false, onClose, initialFocus, labelId = 'modal-title', bare = false, cls = '' }) {
  const trigger = document.activeElement;
  const closeBtn = button({ variant: 'icon', class: 'fui-modal__close', type: 'button', 'aria-label': 'Close' }, icon('x', 16));
  const bodyEl = h('div', { class: bare ? '' : 'fui-modal__body' }, body);
  const dialog = h('div', { class: ['fui-modal', wide && 'fui-modal--wide', cls], role: 'dialog', 'aria-modal': 'true', 'aria-labelledby': bare ? null : labelId, 'aria-label': bare ? title : null },
    bare ? null : h('div', { class: 'fui-modal__head' }, h('h2', { class: 'fui-modal__title', id: labelId }, title), closeBtn), bodyEl);
  const overlay = h('div', { class: 'fui-modal__overlay' }, dialog);
  let closed = false;

  function close() {
    if (closed) return;
    closed = true; modalStack.splice(modalStack.indexOf(onKey), 1);
    document.removeEventListener('keydown', onKey, true);
    overlay.classList.add('is-closing');
    setTimeout(() => overlay.remove(), 140);
    if (!modalStack.length) document.documentElement.classList.remove('no-scroll');
    if (trigger && trigger.isConnected && trigger.focus) trigger.focus();
    if (onClose) onClose();
  }
  function onKey(e) {
    if (modalStack[modalStack.length - 1] !== onKey) return;
    if (e.key === 'Escape') { e.preventDefault(); e.stopPropagation(); close(); return; }
    if (e.key !== 'Tab') return;
    const f = dialog.querySelectorAll('a[href],button:not([disabled]),input:not([disabled]),select,textarea,[tabindex]:not([tabindex="-1"])');
    if (!f.length) return;
    const first = f[0], last = f[f.length - 1];
    if (e.shiftKey && document.activeElement === first) { e.preventDefault(); last.focus(); }
    else if (!e.shiftKey && document.activeElement === last) { e.preventDefault(); first.focus(); }
  }
  closeBtn.addEventListener('click', close);
  overlay.addEventListener('pointerdown', (e) => { if (e.target === overlay) close(); });
  document.addEventListener('keydown', onKey, true);
  document.body.append(overlay);
  document.documentElement.classList.add('no-scroll');
  modalStack.push(onKey);
  requestAnimationFrame(() => (initialFocus && initialFocus.isConnected ? initialFocus : closeBtn.isConnected ? closeBtn : dialog).focus());
  return { close, body: bodyEl, dialog };
}

export const meta = {
  name: 'modal',
  purpose: 'Asks for one thing, or shows one thing closer, above the page.',
  use: 'A short form (a note, a choice) or a closer look (a play’s details). wide for a closer look. bare when the content draws its own heading (the search palette).',
  avoid: 'A modal for something a page could hold. A modal that opens another one without need. Hiding an error in one.',
  variants: ['plain', 'wide', 'bare'],
  states: ['open', 'closing (fades, already gone for the pointer)', 'stacked'],
  a11y: 'role="dialog" aria-modal="true", named by its title; Tab stays inside; Esc closes only the top one and stops there; focus returns to what opened it.',
  props: { 'openModal({ title, body, wide, onClose, initialFocus, labelId, bare, cls })': '→ { close, body, dialog }' },
  examples: [
    { name: 'A button that opens one', render: () => button({ onClick: () => openModal({ title: 'Dismiss finding', body: h('p', null, 'Low Orbit: episodes 4 and 5 are missing between 3 and 6.') }) }, 'Open a dialog') },
  ],
};
