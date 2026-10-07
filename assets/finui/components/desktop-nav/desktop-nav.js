// FinUI: desktop-nav. The menu of a wide screen, in eight styles the person chooses between: a sidebar (grouped or not),
// a rail of icons that opens into the sidebar under the pointer, a sidebar that leads with search, one with the pages you pinned on top, one of coloured tiles (any of
// those on the left or the right), a dock at the bottom that tucks itself away while the page scrolls down, and a command
// bar at the top. Which style, which side, which page is open and what is pinned are the app's; the menu says what it
// needs of the window (`STYLES[].edge` and `size`). Search lives in it: the app hands it a FinUI search (openSearch), and
// it grows out of the menu's own search control into the place the style has for it.

import { h, icon } from '../../core.js';
import { STYLES, styleOf, sideOf, groups, pinsOf, toggled, tone, dockShown } from './plan.js';
import { grow, shrink } from '../search/search.js';

export { STYLES, styleOf, sideOf } from './plan.js';

const dotOn = (p) => (typeof p.dot === 'function' ? !!p.dot() : !!p.dot);

/**
 * The menu. `pages`: [{ key, href, label, icon, group, primary, dot }]. `brand`: { href, mark, name } (mark a node).
 * `me`: { href, avatar, name, role } (avatar a node), `theme`: the app's theme control, both drawn at the foot, or behind
 * the avatar, where a style has no room. `onSearch()`: the menu's search control was pressed (the app opens its search,
 * here with openSearch or wherever it keeps it). `keys`: the shortcut to say
 * beside the search. `pins`/`onPins(keys)`: what is pinned (Pinned keeps them; the app stores them). `insets`: { top,
 * bottom } px of the window the app keeps (its status bar). `contained` keeps it inside its positioned parent.
 * Answers { el, setCurrent, repaint, destroy, openSearch(panel), closeSearch(), searchControl }.
 */
export function desktopNav({ style, side = 'left', pages, current = null, brand = null, me = null, theme = null, onSearch = null, keys = '',
  pins = null, onPins = () => {}, insets = {}, label = 'Pages', contained = false }) {
  const kind = styleOf(style);
  const spec = STYLES.find((s) => s.key === kind);
  const at = sideOf(kind, side);
  const el = h('div', { class: ['fui-desktop-nav', `fui-desktop-nav--${kind}`, at && `fui-desktop-nav--${at}`, contained && 'fui-desktop-nav--contained'] });
  el.style.setProperty('--fui-desktop-nav-top', `${insets.top || 0}px`);
  el.style.setProperty('--fui-desktop-nav-bottom', `${insets.bottom || 0}px`);
  el.style.setProperty('--fui-desktop-nav-size', `${spec.size}px`);
  el.style.setProperty('--fui-desktop-nav-open', `${spec.opens || spec.size}px`);
  const links = [];
  const closers = [];        // the popovers open now, each closed by the next press elsewhere or Esc
  const hooks = [];          // what else repaints when the open page changes (the command bar's name)
  const controls = [];       // what opens search: the search button, or the command bar
  let here = current;
  const compact = ['dock', 'command'].includes(kind);
  const tips = kind === 'dock';

  // ---- the pieces
  function link(p, cls = null, { size = 15, tip = tips } = {}) {
    const dot = p.dot ? h('span', { class: 'fui-desktop-nav__dot' }, h('span', { class: 'sr-only' }, 'New')) : null;
    const a = h('a', { class: ['fui-desktop-nav__link', cls], href: p.href, 'aria-label': tip ? p.label : null }, icon(p.icon, size),
      h('span', { class: 'fui-desktop-nav__label' }, p.label), dot, tip ? h('span', { class: 'fui-desktop-nav__tip', 'aria-hidden': 'true' }, p.label) : null);
    a.addEventListener('click', () => closeAll());
    links.push({ a, p, dot });
    return a;
  }
  const brandEl = () => (brand ? h('a', { class: 'fui-desktop-nav__brand', href: brand.href || '/' }, brand.mark || null, h('span', { class: 'fui-desktop-nav__name' }, brand.name || '')) : null);
  function searchButton(cls = null, big = false) {
    if (!onSearch) return null;
    const b = h('button', { type: 'button', class: ['fui-desktop-nav__search-btn', cls], 'aria-keyshortcuts': keys ? keys.replace(/\s+/g, '+').replace('Ctrl', 'Control') : null, 'aria-label': tips ? 'Search' : null },
      icon('search', big ? 18 : kind === 'dock' ? 20 : kind === 'rail' ? 18 : 14), h('span', { class: 'fui-desktop-nav__label' }, big ? 'Search' : 'Search…'), keys && !tips ? h('kbd', { class: 'fui-desktop-nav__keys' }, keys) : null,
      tips ? h('span', { class: 'fui-desktop-nav__tip', 'aria-hidden': 'true' }, keys ? `Search · ${keys}` : 'Search') : null);
    b.addEventListener('click', () => onSearch());
    controls.push(b);
    return b;
  }
  const meLink = () => (me ? h('a', { class: 'fui-desktop-nav__me', href: me.href }, me.avatar || null,
    h('span', { class: 'fui-desktop-nav__who' }, h('span', { class: 'fui-desktop-nav__me-name' }, me.name), me.role ? h('span', { class: 'fui-desktop-nav__me-role' }, me.role) : null)) : null);
  /** A popover beside its button; one open at a time, closed by a press elsewhere, Esc, or a page chosen. */
  function popover(button, content, cls) {
    const pop = h('div', { class: ['fui-desktop-nav__pop', cls], hidden: true }, content);
    const set = (open) => {
      pop.hidden = !open;
      button.setAttribute('aria-expanded', String(open));
      if (open) { closeAll(set); closers.push(set); } else { const i = closers.indexOf(set); if (i >= 0) closers.splice(i, 1); }
    };
    button.setAttribute('aria-expanded', 'false');
    button.addEventListener('click', () => set(pop.hidden));
    return { pop, set };
  }
  function foot() {
    if (!me && !theme) return null;
    const themed = theme ? h('span', { class: 'fui-desktop-nav__theme' }, theme) : null;
    if (!compact) return h('div', { class: 'fui-desktop-nav__foot' }, meLink(), themed);
    const btn = h('button', { type: 'button', class: 'fui-desktop-nav__me-btn', 'aria-label': me ? `${me.name}: you, and the theme` : 'The theme', 'aria-haspopup': 'true' }, me && me.avatar ? me.avatar.cloneNode(true) : icon('user', 18));
    const { pop } = popover(btn, h('div', { class: 'fui-desktop-nav__foot' }, meLink(), themed), 'fui-desktop-nav__pop--me');
    return h('div', { class: 'fui-desktop-nav__foot-wrap' }, btn, pop);
  }
  const groupBlock = (g, i, inner, { label: named = true } = {}) => h('div', { class: 'fui-desktop-nav__group', role: g.name ? 'group' : null, 'aria-label': g.name || null, 'data-tone': String(tone(i)) },
    named && g.name ? h('div', { class: 'fui-desktop-nav__group-name' }, g.name) : null, inner);

  // ---- the eight
  const frame = h(kind === 'command' ? 'header' : kind === 'dock' ? 'div' : 'aside', { class: 'fui-desktop-nav__frame' });
  const nav = (children, cls = null) => h('nav', { class: ['fui-desktop-nav__nav', cls], 'aria-label': label }, children);
  const all = groups(pages);
  if (kind === 'sidebar') {
    frame.append(...[brandEl(), searchButton(), nav(pages.map((p) => link(p))), foot()].filter(Boolean));
  } else if (kind === 'grouped') {
    frame.append(...[brandEl(), searchButton(), nav(all.map((g, i) => groupBlock(g, i, g.pages.map((p) => link(p))))), foot()].filter(Boolean));
  } else if (kind === 'rail') {
    // The grouped sidebar, drawn whole: closed, the CSS shows its icons alone; open, everything, over the page.
    frame.append(...[brandEl(), searchButton(), nav(all.map((g, i) => groupBlock(g, i, g.pages.map((p) => link(p, null, { size: 17 }))))), foot()].filter(Boolean));
  } else if (kind === 'search') {
    frame.append(...[brandEl(), searchButton('fui-desktop-nav__search-btn--big', true), nav(pages.map((p) => link(p, 'fui-desktop-nav__link--big', { size: 17 })), 'fui-desktop-nav__nav--well'), foot()].filter(Boolean));
  } else if (kind === 'pinned') {
    let pinned = pinsOf(pages, pins);
    const top = h('div', { class: 'fui-desktop-nav__pins', role: 'group', 'aria-label': 'Pinned' });
    const pinButtons = [];
    const drawPins = () => {
      for (const x of links.filter((l) => l.pinned)) x.gone = true;
      top.replaceChildren(h('div', { class: 'fui-desktop-nav__group-name' }, 'Pinned'), ...(pinned.length
        ? pinned.map((k) => { const l = link(pages.find((p) => p.key === k)); links[links.length - 1].pinned = true; return l; })
        : [h('p', { class: 'fui-desktop-nav__hint' }, 'Point at a page and press its pin to keep it here.')]));
      links.splice(0, links.length, ...links.filter((x) => !x.gone));
      for (const b of pinButtons) { const on = pinned.includes(b.dataset.key); b.classList.toggle('is-on', on); b.setAttribute('aria-pressed', String(on)); }
      setCurrent(here);
      repaint();
    };
    const line = (p) => {
      const b = h('button', { type: 'button', class: 'fui-desktop-nav__pin', 'data-key': p.key, 'aria-label': `Pin ${p.label}`, 'aria-pressed': 'false' }, icon('bookmark', 13));
      b.addEventListener('click', () => { pinned = toggled(pinned, p.key); onPins(pinned); drawPins(); });
      pinButtons.push(b);
      return h('div', { class: 'fui-desktop-nav__line' }, link(p), b);
    };
    frame.append(...[brandEl(), searchButton(), nav([top, ...all.map((g, i) => groupBlock(g, i, g.pages.map(line)))]), foot()].filter(Boolean));
    drawPins();
  } else if (kind === 'tiles') {
    frame.append(...[brandEl(), searchButton(), nav(all.map((g, i) => groupBlock(g, i, g.pages.map((p) => {
      const a = link(p, 'fui-desktop-nav__link--tile', { size: 14 });
      a.prepend(h('span', { class: ['fui-desktop-nav__tile', `fui-desktop-nav__tile--${tone(i)}`] }, a.firstChild));
      return a;
    }), { label: false }))), foot()].filter(Boolean));
  } else if (kind === 'dock') {
    frame.append(nav(all.map((g, i) => [i ? h('span', { class: 'fui-desktop-nav__sep' }) : null, g.pages.map((p) => link(p, null, { size: 20 }))])),
      ...[onSearch ? h('span', { class: 'fui-desktop-nav__sep' }) : null, searchButton(), foot()].filter(Boolean));
  } else {
    // The command bar: the open page's name, which turns into the search where it stands.
    const where = h('span', { class: 'fui-desktop-nav__label' });
    const whereIcon = h('span', { class: 'fui-desktop-nav__where' });
    const trigger = h('button', { type: 'button', class: 'fui-desktop-nav__trigger', 'aria-label': `Search, from ${label}`, 'aria-keyshortcuts': keys ? keys.replace(/\s+/g, '+').replace('Ctrl', 'Control') : null }, whereIcon,
      brand ? h('span', { class: 'fui-desktop-nav__crumb' }, brand.name, h('span', { class: 'fui-desktop-nav__crumb-sep' }, '/')) : null, where, icon('search', 14));
    trigger.addEventListener('click', () => onSearch && onSearch());
    controls.push(trigger);
    hooks.push(() => {
      const p = pages.find((x) => x.key === here);
      where.textContent = p ? p.label : label;
      whereIcon.replaceChildren(icon(p ? p.icon : 'menu', 15));
    });
    frame.append(...[brandEl(), h('div', { class: 'fui-desktop-nav__bar' }, trigger), foot()].filter(Boolean));
  }
  el.append(frame);

  // Where search lives in this style: the frame itself (a sidebar widens, the dock rises into a card) or, in the command
  // bar, the bar it stands in.
  const found = h('div', { class: 'fui-desktop-nav__found' });
  const host = kind === 'command' ? frame.querySelector('.fui-desktop-nav__bar') : frame;
  host.append(found);
  let hosted = null, opener = null;
  const searchControl = () => controls.find((c) => c.isConnected) || null;
  /** The menu as it stands, laid over itself for the length of a morph: the root's own frame and classes, nothing live. */
  function ghost() {
    const wrap = el.cloneNode(false);
    wrap.classList.remove('is-searching');
    wrap.setAttribute('aria-hidden', 'true');
    wrap.inert = true;
    const copy = frame.cloneNode(true);
    // A rail held open by the pointer or the keyboard's focus is drawn open: the copy has neither, and would show it closed.
    copy.classList.toggle('is-open', frame.matches(':hover, :has(:focus-visible)'));
    copy.querySelector('.fui-desktop-nav__found')?.remove();
    for (const n of copy.querySelectorAll('[id]')) n.removeAttribute('id');
    wrap.append(copy);
    el.after(wrap);
    return wrap;
  }
  // The command bar's search is a card of its own that drops out of its name; everywhere else the menu itself becomes it.
  const shapeOf = (panel) => (kind === 'command' ? { host: panel.el, fresh: true } : { host, fresh: false });
  function openSearch(panel) {
    if (hosted) return;
    hosted = panel;
    // What had the keyboard's focus (the menu's search, or a place in the page Ctrl+Space was pressed in) is where Esc
    // gives it back. A click's focus is let go instead: handed back after a key, it is the keyboard's, and opens a rail.
    opener = document.activeElement !== document.body && document.activeElement.matches(':focus-visible') ? document.activeElement : null;
    closeAll();
    // The dock rises into the card at its own width, so the shape only grows upwards out of it.
    const dockWidth = kind === 'dock' ? frame.getBoundingClientRect().width : 0;
    const { host: shape, fresh } = shapeOf(panel);
    grow({ from: searchControl(), host: shape, bar: panel.bar, list: panel.list, fresh, ghost, show: () => {
      found.replaceChildren(panel.el);
      if (dockWidth) frame.style.width = `${dockWidth}px`;
      el.classList.add('is-searching');
    } });
    panel.focus();
  }
  /** Put the search back into what it grew out of, the menu fading back in over it as it arrives. `restore` gives the
   *  focus back to what had it when search opened (Esc), quietly, so a tooltip does not pop up as the search goes. The
   *  focus leaves the search before anything is measured: a rail holds itself open while the focus is in it, and the
   *  shape closing into an open rail under a closed one faded in over it was both at once, then a rail left open. */
  function closeSearch(restore = false) {
    const p = hosted;
    if (!p) return Promise.resolve();
    hosted = null;
    const control = searchControl();
    const back = opener && opener.isConnected && !p.el.contains(opener) ? opener : null;
    opener = null;
    // Once the menu is laid out again (each hide: the first, before shrink measures anything, and the last, because the
    // search laid back over it for the animation hid the control and the browser let its focus go).
    const giveBack = () => {
      if (restore && back) {
        if (document.activeElement === back) return;
        back.dataset.quiet = '';
        const loud = () => { delete back.dataset.quiet; back.removeEventListener('blur', loud); back.removeEventListener('pointerenter', loud); };
        back.addEventListener('blur', loud);
        back.addEventListener('pointerenter', loud);
        back.focus({ preventScroll: true });
      } else if (p.el.contains(document.activeElement)) document.activeElement.blur();
    };
    const { host: shape, fresh } = shapeOf(p);
    const width = frame.style.width;
    return shrink({ to: control, host: shape, bar: p.bar, list: p.list, gone: fresh, ghost,
      show: () => { frame.style.width = width; el.classList.add('is-searching'); },
      hide: () => { frame.style.width = ''; el.classList.remove('is-searching'); giveBack(); } })
      .then(() => { if (!hosted) found.replaceChildren(); });
  }

  // The dock tucks itself away while the page scrolls down, like a dock that hides: back on scrolling up, at the top, with
  // the pointer near the bottom edge, or with the focus in it (dockShown).
  let undock = () => {};
  if (kind === 'dock' && !contained) {
    let lastY = window.scrollY, near = false, shown = true, frameReq = 0;
    const NEAR_PX = 90;
    const paintDock = () => {
      frameReq = 0;
      const y = window.scrollY;
      // Focus counts only when it was reached from the keyboard: a link pressed in the dock keeps the focus after the click,
      // and that is nobody using the dock.
      const keyed = frame.contains(document.activeElement) && document.activeElement.matches(':focus-visible');
      shown = dockShown({ was: shown, scrollY: y, dy: y - lastY, near, focused: keyed || closers.length > 0 || !!hosted });
      lastY = y;
      frame.classList.toggle('is-tucked', !shown);
    };
    const soon = () => { if (!frameReq) frameReq = requestAnimationFrame(paintDock); };
    const onMove = (e) => { const was = near; near = e.clientY >= window.innerHeight - NEAR_PX; if (near !== was) soon(); };
    window.addEventListener('scroll', soon, { passive: true });
    window.addEventListener('pointermove', onMove, { passive: true });
    frame.addEventListener('focusin', soon);
    frame.addEventListener('focusout', () => setTimeout(soon));
    undock = () => { window.removeEventListener('scroll', soon); window.removeEventListener('pointermove', onMove); cancelAnimationFrame(frameReq); };
  }

  // ---- what it does
  function closeAll(except = null) { for (const set of [...closers]) if (set !== except) set(false); }
  const onPress = (e) => { if (closers.length && !e.target.closest('.fui-desktop-nav__pop, .fui-desktop-nav__trigger, .fui-desktop-nav__me-btn')) closeAll(); };
  const onKey = (e) => {
    if (e.key !== 'Escape' || !closers.length) return;
    e.preventDefault(); e.stopPropagation();
    closeAll();
    const back = el.querySelector('[aria-expanded="false"].fui-desktop-nav__trigger, [aria-expanded="false"].fui-desktop-nav__me-btn');
    if (back) back.focus({ preventScroll: true });
  };
  const root = contained ? el : document;
  root.addEventListener('pointerdown', onPress, true);
  root.addEventListener('keydown', onKey, true);

  function setCurrent(key) {
    here = key;
    for (const { a, p } of links) {
      const on = p.key === key;
      a.classList.toggle('is-current', on);
      if (on) a.setAttribute('aria-current', 'page'); else a.removeAttribute('aria-current');
    }
    for (const fn of hooks) fn();
  }
  function repaint() { for (const { p, dot } of links) if (dot) dot.hidden = !dotOn(p); }
  function destroy() { undock(); root.removeEventListener('pointerdown', onPress, true); root.removeEventListener('keydown', onKey, true); el.remove(); }

  setCurrent(current);
  repaint();
  return { el, setCurrent, repaint, destroy, openSearch, closeSearch, get searchControl() { return searchControl(); }, get searching() { return !!hosted; } };
}

const DEMO_PAGES = [
  { key: 'home', href: '#home', label: 'Dashboard', icon: 'home', group: 'You', primary: true }, { key: 'recap', href: '#recap', label: 'Recap', icon: 'recap', group: 'You' },
  { key: 'activity', href: '#activity', label: 'Activity', icon: 'activity', group: 'Watching', primary: true }, { key: 'together', href: '#together', label: 'Together', icon: 'together', group: 'Watching' },
  { key: 'libraries', href: '#libraries', label: 'Libraries', icon: 'library', group: 'Library', primary: true }, { key: 'users', href: '#users', label: 'Users', icon: 'users', group: 'Library' },
  { key: 'settings', href: '#settings', label: 'Settings', icon: 'settings', group: 'Server' }, { key: 'notes', href: '#notes', label: 'Patch notes', icon: 'tag', group: 'Server', dot: true },
];

export const meta = {
  name: 'desktop-nav',
  purpose: 'The menu of a wide screen, in eight styles the person chooses between, the sidebars on either side.',
  use: 'Once, above the width where mobile-nav takes over. The app keeps the person’s choice (style, side, pins), marks the open page (setCurrent) and lays its page beside what STYLES says the style keeps.',
  avoid: 'Choosing the style for the person. A side for a dock or a command bar: sideOf answers none. Pages that are not pages (a sign-out).',
  variants: STYLES.map((s) => `${s.label}: ${s.line}`),
  states: ['is-current: the open page (aria-current="page")', 'on the right (fui-desktop-nav--right)', 'a pinned page (Pinned)', 'a page with something new (dot)', 'a popover open: the command panel, or you and the theme behind the avatar'],
  a11y: 'Links are links, the open page aria-current; a page named only by its icon (rail, dock) carries its name as aria-label and shows it beside the pointer. The command panel and the avatar’s popover say aria-expanded on their buttons, close on Esc or a press elsewhere and give the focus back. A pin says aria-pressed.',
  props: {
    'desktopNav({ style, side, pages, current, brand, me, theme, onSearch, keys, pins, onPins, insets, label, contained })': "style: 'sidebar' | 'grouped' | 'rail' | 'search' | 'dock' | 'command' | 'pinned' | 'tiles'; side: 'left' | 'right'; pages: [{ key, href, label, icon, group, primary, dot }]",
    'STYLES': '[{ key, label, line, edge, size }]: edge is side, bottom or top, size its px',
    '→ { el, setCurrent(key), repaint(), destroy() }': '',
  },
  playground: {
    controls: [
      { key: 'style', label: 'Style', choices: STYLES.map((s) => [s.key, s.label]) },
      { key: 'right', label: 'On the right', on: false },
    ],
    render: (o) => {
      const nav = desktopNav({ style: o.style, side: o.right ? 'right' : 'left', pages: DEMO_PAGES, current: 'home', contained: true, keys: 'Ctrl Space', onSearch: () => {},
        brand: { href: '#', mark: icon('play', 18), name: 'Example' }, me: { href: '#me', avatar: icon('user', 18), name: 'alice', role: 'Viewer' } });
      return h('div', { class: 'fui-desktop-nav__demo' }, h('div', { class: 'fui-desktop-nav__demo-page' }, h('span'), h('span'), h('span')), nav.el);
    },
  },
};
