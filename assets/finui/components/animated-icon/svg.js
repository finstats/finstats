// FinUI: an animated icon as a file of its own. The same icon animate() makes on a page, written out as markup with its
// motion inside it: the rules of animated-icon.css that it needs, and nothing more, in a <style> of its own. Saved as a
// .svg it moves wherever an image does (an <img>, a CSS background, a README), with no FinUI beside it. Pure: it is
// given the stylesheet's text, so a build reads it from disk and a page fetches it.

import { iconSvg } from '../../core.js';
import { plan } from './motions.js';

/** The top-level blocks of a stylesheet, comments gone: [prelude, body] each, the body without its outer braces. */
function blocks(css) {
  const text = css.replace(/\/\*[\s\S]*?\*\//g, '');
  const out = [];
  let depth = 0, start = 0, open = 0;
  for (let i = 0; i < text.length; i++) {
    if (text[i] === '{' && depth++ === 0) open = i;
    else if (text[i] === '}' && --depth === 0) { out.push([text.slice(start, open).trim(), text.slice(open + 1, i).trim()]); start = i + 1; }
  }
  return out;
}

const SHARED = new Set(['.fui-animated-icon', '.fui-animated-icon > *', '.fui-animated-icon > [data-act]']);

/** The rules an icon whose shapes do `acts` needs from animated-icon.css: the drawing, and each of its acts. */
function rulesFor(css, acts) {
  const wanted = (prelude) => SHARED.has(prelude)
    || acts.some((a) => prelude === `.fui-animated-icon > [data-act="${a}"]` || prelude === `@keyframes fui-animated-icon-${a}`)
    || prelude === '@keyframes fui-animated-icon-draw';
  return blocks(css).filter(([prelude]) => wanted(prelude)).map(([prelude, body]) =>
    prelude.startsWith('@') ? `${prelude} {\n  ${body.split('\n').map((l) => l.trim()).join('\n  ')}\n}` : `${prelude} { ${body.replace(/\s+/g, ' ')} }`);
}

/**
 * animatedSvg(name, css, { size, cycle }): icon `name`, moving, as markup to save as name.svg; null for an icon there is
 * not. css: the text of animated-icon.css. It loops, one cycle every `cycle` (a CSS time), and is still with reduced
 * motion. It draws in currentColor, as iconSvg does.
 */
export function animatedSvg(name, css, { size = 24, cycle = '3s' } = {}) {
  const still = iconSvg(name, size);
  if (!still) return null;
  const shapes = [...still.matchAll(/<(?:path|circle|rect|ellipse)\b[^>]*\/>/g)].map((m) => m[0]);
  const parts = plan(name, shapes.length);
  // What animate() sets on each shape where it stands, written as attributes.
  const moving = shapes.map((shape, i) => {
    const p = parts[i];
    const style = [`--i: ${i}`];
    if (p.act) {
      style.push(`--k: ${p.k}`, `--dx: ${p.dx}`, `--dy: ${p.dy}`, `--dir: ${p.dir}`, `--sx: ${p.sx}`);
      if (p.origin) style.push('transform-box: view-box', `transform-origin: ${p.origin}`);
    }
    return shape.replace(/\/>$/, ` pathLength="1"${p.act ? ` data-act="${p.act}"` : ''} style="${style.join('; ')}"/>`);
  });
  const acts = [...new Set(parts.map((p) => p.act).filter(Boolean))];
  const style = [...rulesFor(css, acts), '@media (prefers-reduced-motion: reduce) { .fui-animated-icon > * { animation: none; } }'].join('\n');
  const open = still.slice(0, still.indexOf('>'));
  return `${open} class="fui-animated-icon" style="--icon-cycle: ${cycle}"><style>\n${style}\n</style>${moving.join('')}</svg>\n`;
}
