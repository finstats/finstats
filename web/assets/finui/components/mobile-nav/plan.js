// FinUI: the mobile menu's rules, apart from any page. Which styles there are, which pages sit in a bar and which wait
// behind More, and where the thumb arc puts each page. Pure, so a test reads it without a page.

/** The styles, the tab bar first: it is what a menu is before anybody chooses. `space` is how much of the bottom of the
 *  screen the style keeps for itself while closed, so the app can leave the end of its page clear of it; `corner` how far up
 *  from the bottom it reaches in the right-hand corner (the safe area aside), so what rests there (to-top) stays clear:
 *  the tab bar's top, the arc's button's, the address pill's. */
export const STYLES = [
  { key: 'tabs', label: 'Tab bar', line: 'Four pages always at the bottom; More raises the rest.', space: 64, corner: 64 },
  { key: 'peek', label: 'Peek', line: 'A sheet that opens half way on the pages used most; drag it up for all of them.', space: 0, corner: 0 },
  { key: 'full', label: 'Full screen', line: 'The whole screen becomes the menu, in large type.', space: 0, corner: 0 },
  { key: 'arc', label: 'Thumb arc', line: 'A corner button fans every page out around the thumb.', space: 72, corner: 72 },
  { key: 'address', label: 'Address bar', line: 'A pill at the bottom names the page you are on and grows into the menu.', space: 74, corner: 62 },
];

/** How much of the screen's bottom-right corner a style keeps while closed, as desktop-nav's cornerOf says it. */
export const cornerOf = (style) => ({ bottom: STYLES.find((s) => s.key === styleOf(style)).corner, right: 0 });

/** A stored choice as a style: one nobody knows (from an older or newer version) is the tab bar. */
export const styleOf = (value) => (STYLES.some((s) => s.key === value) ? value : STYLES[0].key);

/** The pages a bar shows (`n` at most: the ones marked `primary`, then the first of the others) and the rest, both in page order. */
export function split(pages, n = 4) {
  const marked = pages.filter((p) => p.primary).slice(0, n);
  const fill = pages.filter((p) => !marked.includes(p)).slice(0, n - marked.length);
  const bar = pages.filter((p) => marked.includes(p) || fill.includes(p));
  return { bar, rest: pages.filter((p) => !bar.includes(p)) };
}

/**
 * Where the thumb arc puts `count` pages: rings round the corner, `first` px out and `step` further each, the nearest
 * filled first. A ring spreads its pages over `span` degrees starting `from` the floor and holds as many as keep `gap` px
 * between neighbours; a ring of one sits on the diagonal. x is leftwards and y upwards from the corner, in px.
 */
export function arc(count, { first = 96, step = 72, from = 8, span = 74, gap = 56 } = {}) {
  const rad = (d) => (d * Math.PI) / 180;
  const holds = (r) => { let n = 1; while (2 * r * Math.sin(rad(span / n / 2)) >= gap) n += 1; return n; };
  const out = [];
  for (let ring = 0, left = count; left > 0; ring += 1) {
    const r = first + step * ring;
    const n = Math.min(left, holds(r));
    for (let k = 0; k < n; k++) {
      const a = rad(n === 1 ? from + span / 2 : from + (span * k) / (n - 1));
      out.push({ x: +(r * Math.cos(a)).toFixed(1), y: +(r * Math.sin(a)).toFixed(1), ring });
    }
    left -= n;
  }
  return out;
}
