// FinMotion: the little it draws itself — a line under a row of tabs, a ring — is built here, since FinMotion imports
// nothing from FinUI. Text and attributes only: nothing reaches the page as markup.

/** h(tag, attrs, ...children): an element with its attributes set (a class as a string) and its children appended. */
export function h(tag, attrs = {}, ...children) {
  const el = document.createElement(tag);
  for (const [k, v] of Object.entries(attrs || {})) if (v != null && v !== false) el.setAttribute(k === 'class' ? 'class' : k, String(v));
  for (const c of children.flat()) if (c != null && c !== false) el.append(c.nodeType ? c : document.createTextNode(String(c)));
  return el;
}
