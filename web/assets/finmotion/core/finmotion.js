// FinMotion: how FinUI moves, added on top of it. FinUI stays still and plain; FinMotion is one stylesheet after FinUI's
// own (finmotion.css: every stylesheet in registry.json, in its order) and one call:
//
//   import { motion } from './finmotion/core/finmotion.js';
//   motion();
//
// From then on every FinUI component on the page moves, the ones there now and the ones drawn later, at the pace FinUI's
// Motion choice sets, and none of it under reduced motion. FinUI needs no change for it: a part finds its component by
// FinUI's own classes and moves what FinUI draws.

import { pace } from './motion.js';
import { PARTS } from '../parts/index.js';

export * from './motion.js';

/** Moves every FinUI component under `root` (the page by default), now and as they are drawn. Answers stop(). */
export function motion(root = document.body, { parts = PARTS } = {}) {
  pace();
  // A preset may arrive as a stylesheet after this: every stylesheet that loads may change FinUI's --ease.
  const repace = (e) => { if (e.target instanceof HTMLLinkElement) pace(); };
  document.addEventListener('load', repace, true);
  // What each moved element answers to stop it, kept only while the element is in the page: a stop holds its element,
  // and a list that only grew held every page that was ever left, with the listeners a part put outside it.
  const done = new WeakMap(), stops = new Map();
  const sweep = (node) => {
    if (node.nodeType !== 1) return;
    for (const part of parts) {
      const found = node.matches(part.selector) ? [node, ...node.querySelectorAll(part.selector)] : node.querySelectorAll(part.selector);
      for (const el of found) {
        const had = done.get(el) || new Set();
        if (had.has(part.name)) continue;
        had.add(part.name); done.set(el, had);
        const stop = part.enhance(el);
        if (typeof stop === 'function') stops.set(el, [...(stops.get(el) || []), stop]);
      }
    }
  };
  // An element that left is stopped and forgotten once the changes are done (one that only moved is still here), and
  // moved again should it come back.
  const letGo = () => {
    for (const [el, fns] of stops) {
      if (root.contains(el)) continue;
      stops.delete(el); done.delete(el);
      fns.forEach((s) => s());
    }
  };
  sweep(root);
  const watch = new MutationObserver((records) => {
    let left = false;
    for (const r of records) {
      for (const n of r.addedNodes) sweep(n);
      if (!left) for (const n of r.removedNodes) if (n.nodeType === 1) { left = true; break; }
    }
    if (left) letGo();
  });
  watch.observe(root, { childList: true, subtree: true });
  return () => { watch.disconnect(); document.removeEventListener('load', repace, true); for (const fns of stops.values()) fns.forEach((s) => s()); stops.clear(); };
}
