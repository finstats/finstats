// FinUI: the desktop menu's rules, apart from any page. Which styles there are and the edge of the window each keeps, which
// may sit on the right, how pages fall into groups, what is pinned, and a group's colour. Pure, so a test reads it.

/** The styles, today's sidebar first: it is what a menu is before anybody chooses. `edge` and `size` say what each keeps
 *  of the window — a side (px wide), the bottom or the top (px tall) — so the app can lay its page beside it. A rail
 *  `opens` that wide under the pointer, over the page rather than pushing it. */
export const STYLES = [
  { key: 'sidebar', label: 'Sidebar', line: 'Every page in one quiet list down the side.', edge: 'side', size: 220 },
  { key: 'grouped', label: 'Grouped', line: 'The same list in named groups, so a page is found by where it lives.', edge: 'side', size: 220 },
  { key: 'rail', label: 'Rail that opens', line: 'Icons until the pointer comes, then the whole sidebar over the page.', edge: 'side', size: 60, opens: 232 },
  { key: 'search', label: 'Search first', line: 'A large search above the pages, and rows a little larger.', edge: 'side', size: 240 },
  { key: 'dock', label: 'Dock', line: 'The pages in a dock at the bottom; the status bar moves to the top.', edge: 'bottom', size: 78 },
  { key: 'command', label: 'Command bar', line: 'No list on screen: the page’s name at the top opens every page.', edge: 'top', size: 52 },
  { key: 'pinned', label: 'Pinned', line: 'The pages you pin on top, every page below in its group.', edge: 'side', size: 220 },
  { key: 'tiles', label: 'Coloured tiles', line: 'Each icon in a tile coloured by its group.', edge: 'side', size: 232 },
];

/** A stored choice as a style: one nobody knows is today's sidebar. */
export const styleOf = (value) => (STYLES.some((s) => s.key === value) ? value : STYLES[0].key);

/** Which side a style sits on: the right only when asked for and it is a side at all; a dock or a command bar has none. */
export function sideOf(style, wanted) {
  return STYLES.find((s) => s.key === styleOf(style)).edge !== 'side' ? null : wanted === 'right' ? 'right' : 'left';
}

/** How much of the window's bottom-right corner a style keeps, in px up from the bottom and in from the right, so what
 *  rests in that corner (to-top) stays clear of it: a side on the right keeps its width; the dock floats in the middle of
 *  the bottom and leaves the corner free, as every other style does. */
export function cornerOf(style, side) {
  const s = STYLES.find((x) => x.key === styleOf(style));
  return { bottom: 0, right: sideOf(s.key, side) === 'right' ? s.size : 0 };
}

/** Pages in their groups, the groups in the order they first appear; pages without one are a group with no name. */
export function groups(pages) {
  const out = [];
  for (const p of pages) {
    const name = p.group || '';
    let g = out.find((x) => x.name === name);
    if (!g) out.push(g = { name, pages: [] });
    g.pages.push(p);
  }
  return out;
}

/** What is pinned: the primary pages until somebody pins, then their pins in the order pinned, of pages that exist. */
export function pinsOf(pages, stored) {
  if (!Array.isArray(stored)) return pages.filter((p) => p.primary).map((p) => p.key);
  const keys = new Set(pages.map((p) => p.key));
  return [...new Set(stored)].filter((k) => keys.has(k));
}

/** A pin pressed: off when on, on at the end when off. */
export const toggled = (pins, key) => (pins.includes(key) ? pins.filter((k) => k !== key) : [...pins, key]);

/** A group's colour, by its place: the four chart colours in turn (1 to 4). */
export const tone = (i) => 1 + (i % 4);

/** How far the page must move before the dock answers it, so a trackpad's tremor does not flicker it. */
const SCROLL_PX = 4;
/** Whether the dock is out: tucked away while the page scrolls down, back on scrolling up, at the top of the page, with the
 *  pointer near the bottom edge (`near`) or the focus in it — like a dock that hides itself. */
export function dockShown({ was, scrollY, dy, near, focused }) {
  if (near || focused || scrollY <= 40) return true;
  if (dy > SCROLL_PX) return false;
  if (dy < -SCROLL_PX) return true;
  return was;
}
