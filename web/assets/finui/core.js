// FinUI core: the element builder and the icons. Everything a FinUI component draws is made here; data reaches the
// DOM as text nodes, never as markup. The only markup strings are the static icon paths below.

const SVG_NS = 'http://www.w3.org/2000/svg';

function append(el, children) {
  for (const c of children) {
    if (c == null || c === false || c === true) continue;
    if (Array.isArray(c)) append(el, c);
    else el.append(c instanceof Node ? c : document.createTextNode(String(c)));
  }
}

function apply(el, props, isSvg) {
  for (const [k, v] of Object.entries(props || {})) {
    if (k === 'spellcheck') { el.setAttribute('spellcheck', String(!!v)); continue; }
    if (v == null || v === false) continue;
    if (k === 'class') el.setAttribute('class', Array.isArray(v) ? v.filter(Boolean).join(' ') : v);
    else if (k === 'style' && typeof v === 'object') Object.assign(el.style, v);
    else if (k === 'dataset') Object.assign(el.dataset, v);
    else if (k.startsWith('on') && typeof v === 'function') el.addEventListener(k.slice(2).toLowerCase(), v);
    else if (!isSvg && k in el && !k.startsWith('aria') && k !== 'list' && k !== 'form' && k !== 'role') {
      try { el[k] = v; } catch { el.setAttribute(k, v); }
    } else el.setAttribute(k, v === true ? '' : v);
  }
}

/** h('div', {class: 'x', onClick}, child, [children], 'text') */
export function h(tag, props, ...children) {
  const el = document.createElement(tag);
  apply(el, props, false);
  append(el, children);
  return el;
}

/** Same as h() for SVG elements. */
export function s(tag, props, ...children) {
  const el = document.createElementNS(SVG_NS, tag);
  apply(el, props, true);
  append(el, children);
  return el;
}

export function clear(el) { el.replaceChildren(); return el; }
export function mount(el, ...children) { el.replaceChildren(); append(el, children); return el; }

// ---------------------------------------------------------------- icons
// Static, trusted path data (24×24, stroke). Never mixed with API data.
const ICONS = {
  home: '<path d="M3 10.5 12 3l9 7.5"/><path d="M5 9.5V21h14V9.5"/><path d="M10 21v-6h4v6"/>',
  activity: '<path d="M3 12h4l3-8 4 16 3-8h4"/>',
  users: '<circle cx="9" cy="8" r="3.5"/><path d="M2.5 20c.6-3.6 3.2-5.5 6.5-5.5s5.9 1.9 6.5 5.5"/><path d="M16 4.8a3.5 3.5 0 0 1 0 6.4"/><path d="M18 14.8c2 .8 3.2 2.6 3.6 5.2"/>',
  user: '<circle cx="12" cy="8" r="4"/><path d="M4 21c.7-4 3.8-6 8-6s7.3 2 8 6"/>',
  library: '<path d="M4 4v16"/><path d="M8 4v16"/><path d="m12 6 4.5-1.2 3.6 14.4-4.5 1.2z"/>',
  play: '<path d="M7 4.5v15l12-7.5z"/>',
  pause: '<path d="M8 5v14"/><path d="M16 5v14"/>',
  sliders: '<path d="M4 6h10"/><path d="M18 6h2"/><circle cx="16" cy="6" r="2"/><path d="M4 12h2"/><path d="M10 12h10"/><circle cx="8" cy="12" r="2"/><path d="M4 18h10"/><path d="M18 18h2"/><circle cx="16" cy="18" r="2"/>',
  log: '<path d="M6 3h9l4 4v14H6z"/><path d="M14 3v5h5"/><path d="M9 13h7"/><path d="M9 17h5"/>',
  settings: '<path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/><circle cx="12" cy="12" r="3"/>',
  search: '<circle cx="11" cy="11" r="7"/><path d="m20 20-3.8-3.8"/>',
  plus: '<path d="M12 5v14"/><path d="M5 12h14"/>',
  x: '<path d="M6 6l12 12"/><path d="M18 6 6 18"/>',
  check: '<path d="m5 12.5 4.5 4.500L19 7.5"/>',
  copy: '<rect x="9" y="9" width="11" height="11" rx="2"/><path d="M5 15V6a2 2 0 0 1 2-2h9"/>',
  menu: '<path d="M4 7h16"/><path d="M4 12h16"/><path d="M4 17h16"/>',
  arrowUp: '<path d="M12 19V5"/><path d="m6 11 6-6 6 6"/>',
  trendUp: '<path d="m4 16 6-6 4 4 6-7"/><path d="M15 7h5v5"/>',
  trendDown: '<path d="m4 8 6 6 4-4 6 7"/><path d="M15 17h5v-5"/>',
  minus: '<path d="M6 12h12"/>',
  chevronDown: '<path d="m6 9 6 6 6-6"/>',
  chevronRight: '<path d="m9 6 6 6-6 6"/>',
  chevronLeft: '<path d="m15 6-6 6 6 6"/>',
  table: '<rect x="3" y="4" width="18" height="16" rx="2"/><path d="M3 10h18"/><path d="M3 15h18"/><path d="M9 4v16"/>',
  chart: '<path d="M4 20V10"/><path d="M10 20V4"/><path d="M16 20v-7"/><path d="M22 20H2"/>',
  upload: '<path d="M12 16V4"/><path d="m7 9 5-5 5 5"/><path d="M4 16v3a1 1 0 0 0 1 1h14a1 1 0 0 0 1-1v-3"/>',
  together: '<path d="M5 11V8a3 3 0 0 1 3-3h8a3 3 0 0 1 3 3v3"/><path d="M3 13a2 2 0 0 1 4 0v2h10v-2a2 2 0 0 1 4 0v4a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1z"/><path d="M6 18v2"/><path d="M18 18v2"/><path d="M12 5v10"/>',
  share: '<circle cx="18" cy="5" r="2.5"/><circle cx="6" cy="12" r="2.5"/><circle cx="18" cy="19" r="2.5"/><path d="m8.2 10.8 7.6-4.4"/><path d="m8.2 13.2 7.6 4.4"/>',
  download: '<path d="M12 4v12"/><path d="m7 11 5 5 5-5"/><path d="M4 16v3a1 1 0 0 0 1 1h14a1 1 0 0 0 1-1v-3"/>',
  inbox: '<path d="M4 13h4l2 3h4l2-3h4"/><path d="M5.5 5h13l2.5 8v5a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1v-5z"/>',
  plug: '<path d="M9 3v6"/><path d="M15 3v6"/><path d="M6 9h12v3a6 6 0 0 1-12 0z"/><path d="M12 18v3"/>',
  alert: '<path d="M12 3 2.5 20h19z"/><path d="M12 10v4.5"/><path d="M12 17.500v.01"/>',
  info: '<circle cx="12" cy="12" r="9"/><path d="M12 11v6"/><path d="M12 7.500v.01"/>',
  logout: '<path d="M10 4H5v16h5"/><path d="M14 8l4 4-4 4"/><path d="M18 12H9"/>',
  trash: '<path d="M4 7h16"/><path d="M9 7V4h6v3"/><path d="M6 7l1 13h10l1-13"/>',
  refresh: '<path d="M20 11a8 8 0 0 0-14.5-4"/><path d="M5 3v4h4"/><path d="M4 13a8 8 0 0 0 14.5 4"/><path d="M19 21v-4h-4"/>',
  film: '<rect x="3" y="4" width="18" height="16" rx="2"/><path d="M7 4v16"/><path d="M17 4v16"/><path d="M3 9h4"/><path d="M3 15h4"/><path d="M17 9h4"/><path d="M17 15h4"/>',
  database: '<ellipse cx="12" cy="6" rx="8" ry="3"/><path d="M4 6v6c0 1.7 3.6 3 8 3s8-1.3 8-3V6"/><path d="M4 12v6c0 1.7 3.6 3 8 3s8-1.3 8-3v-6"/>',
  link: '<path d="M10 14a4 4 0 0 0 5.7 0l3-3a4 4 0 0 0-5.7-5.700l-1 1"/><path d="M14 10a4 4 0 0 0-5.7 0l-3 3a4 4 0 0 0 5.7 5.700l1-1"/>',
  shield: '<path d="M12 3 4.5 6v6c0 4.5 3 7.7 7.5 9 4.5-1.3 7.5-4.5 7.5-9V6z"/>',
  server: '<rect x="3" y="4" width="18" height="7" rx="2"/><rect x="3" y="13" width="18" height="7" rx="2"/><path d="M7 7.5h.01"/><path d="M7 16.500h.01"/>',
  bookmark: '<path d="M6.5 3.5h11v17L12 16.5l-5.5 4z"/>',
  heart: '<path d="M12 20s-7.5-4.6-7.5-10.2A4.3 4.3 0 0 1 12 7.300a4.300 4.300 0 0 1 7.500 2.500C19.500 15.400 12 20 12 20z"/>',
  globe: '<circle cx="12" cy="12" r="9"/><path d="M3 12h18"/><path d="M12 3c2.800 3 2.800 15 0 18"/><path d="M12 3c-2.800 3-2.800 15 0 18"/>',
  lan: '<rect x="9" y="3" width="6" height="5" rx="1"/><rect x="3" y="16" width="6" height="5" rx="1"/><rect x="15" y="16" width="6" height="5" rx="1"/><path d="M12 8v4"/><path d="M6 16v-4h12v4"/>',
  external: '<path d="M14 4h6v6"/><path d="M20 4 11 13"/><path d="M18 14v5a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V7a1 1 0 0 1 1-1h5"/>',
  skip: '<path d="m4 6 8 6-8 6z"/><path d="m12 6 8 6-8 6z"/>',
  stop: '<rect x="6" y="6" width="12" height="12" rx="1.500"/>',
  volume: '<path d="M4 10v4h3.500L12 18V6l-4.500 4z"/><path d="M15.500 9a4 4 0 0 1 0 6"/><path d="M18 6.500a8 8 0 0 1 0 11"/>',
  captions: '<rect x="3" y="5" width="18" height="14" rx="2"/><path d="M7 14h4"/><path d="M13 14h4"/><path d="M7 10.500h2"/><path d="M11 10.500h6"/>',
  tv: '<rect x="2" y="7" width="20" height="15" rx="2"/><path d="m17 2-5 5-5-5"/>',
  tag: '<path d="M12.586 2.586A2 2 0 0 0 11.172 2H4a2 2 0 0 0-2 2v7.172a2 2 0 0 0 .586 1.414l8.704 8.704a2.426 2.426 0 0 0 3.42 0l6.58-6.58a2.426 2.426 0 0 0 0-3.42z"/><circle cx="7.5" cy="7.5" r="1"/>',
  recap: '<path d="M4 12a8 8 0 1 0 2.600-5.900"/><path d="M4 4v4h4"/><path d="M12 8v4.500l3 1.800"/>',
  cpu: '<rect x="6" y="6" width="12" height="12" rx="2"/><rect x="10" y="10" width="4" height="4"/><path d="M9 3v3"/><path d="M15 3v3"/><path d="M9 18v3"/><path d="M15 18v3"/><path d="M3 9h3"/><path d="M3 15h3"/><path d="M18 9h3"/><path d="M18 15h3"/>',
  clock: '<circle cx="12" cy="12" r="9"/><path d="M12 7v5l3 2"/>',
  github: '<path d="M15 22v-4a4.8 4.8 0 0 0-1-3.5c3 0 6-2 6-5.5.08-1.25-.27-2.48-1-3.5.28-1.15.28-2.35 0-3.5 0 0-1 0-3 1.5-2.64-.5-5.36-.5-8 0C6 2 5 2 5 2c-.3 1.15-.3 2.35 0 3.5A5.4 5.4 0 0 0 4 9c0 3.5 3 5.5 6 5.5-.39.49-.68 1.05-.85 1.65-.17.6-.22 1.23-.15 1.85v4"/><path d="M9 18c-4.51 2-5-2-7-2"/>',
  calendar: '<rect x="3" y="5" width="18" height="16" rx="2"/><path d="M3 10h18"/><path d="M8 3v4"/><path d="M16 3v4"/>',
  flame: '<path d="M12 3c.6 3.2 5 5.4 5 10.2A5 5 0 0 1 12 21a5 5 0 0 1-5-7.8c.4 1.400 1.300 2.300 2.400 2.600C9 12 9.800 6.500 12 3z"/>',
  repeat: '<path d="m17 2 4 4-4 4"/><path d="M3 11v-1a4 4 0 0 1 4-4h14"/><path d="m7 22-4-4 4-4"/><path d="M21 13v1a4 4 0 0 1-4 4H3"/>',
  sparkle: '<path d="M12 3l1.900 5.600a2 2 0 0 0 1.300 1.300L21 12l-5.800 2.100a2 2 0 0 0-1.300 1.300L12 21l-1.900-5.600a2 2 0 0 0-1.300-1.300L3 12l5.800-2.100a2 2 0 0 0 1.300-1.300z"/>',
  users: '<circle cx="9" cy="8" r="3.500"/><path d="M2.500 20c.6-3.600 3-5.500 6.500-5.500s5.900 1.900 6.500 5.500"/><path d="M16 4.700a3.500 3.500 0 0 1 0 6.600"/><path d="M18.500 14.900c1.700.8 2.700 2.500 3 5.100"/>',
  trophy: '<path d="M8 21h8"/><path d="M12 17v4"/><path d="M7 4h10v5a5 5 0 0 1-10 0z"/><path d="M17 5h3v2a3 3 0 0 1-3 3"/><path d="M7 5H4v2a3 3 0 0 0 3 3"/>',
  compass: '<circle cx="12" cy="12" r="9"/><path d="m15.500 8.500-2 5-5 2 2-5z"/>',
  monitor: '<rect x="3" y="4" width="18" height="13" rx="2"/><path d="M8 21h8"/><path d="M12 17v4"/>',
  layers: '<path d="m12 3 9 5-9 5-9-5z"/><path d="m3 13 9 5 9-5"/>',
  sun: '<circle cx="12" cy="12" r="4"/><path d="M12 2v2"/><path d="M12 20v2"/><path d="m4.9 4.9 1.4 1.4"/><path d="m17.7 17.7 1.4 1.4"/><path d="M2 12h2"/><path d="M20 12h2"/><path d="m6.3 17.7-1.4 1.4"/><path d="m19.1 4.9-1.4 1.4"/>',
  moon: '<path d="M20.5 14.5A8.5 8.5 0 1 1 9.5 3.5a7 7 0 0 0 11 11z"/>',
  gauge: '<path d="M4.2 17.5a8.5 8.5 0 1 1 15.6 0"/><path d="m12 14 4-4.5"/><circle cx="12" cy="14" r="1.3"/>',
  anchor: '<circle cx="12" cy="5" r="2"/><path d="M12 7v14"/><path d="M8.5 10.5h7"/><path d="M4.5 13.5a7.5 7.5 0 0 0 15 0"/>',
};

const iconTpl = document.createElement('template');
export function icon(name, size = 16, cls = '') {
  // It says which icon it is (data-icon), so an animated icon can be made of it where it stands.
  const known = Object.hasOwn(ICONS, name);
  iconTpl.innerHTML =
    `<svg xmlns="${SVG_NS}" viewBox="0 0 24 24" width="${Number(size)}" height="${Number(size)}" fill="none" ` +
    `stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" ` +
    `class="icon"${known ? ` data-icon="${name}"` : ''}>${known ? ICONS[name] : ''}</svg>`;
  const el = iconTpl.content.firstChild;
  if (cls) el.classList.add(...cls.split(' '));
  return el;
}

/** Every icon's name, for the gallery. */
export const iconNames = () => Object.keys(ICONS);

export const meta = {
  name: 'core',
  purpose: 'Builds elements (h for HTML, s for SVG), replaces an element’s children (mount, clear), and draws an icon.',
  use: 'Everything in FinUI is built with h(). Data goes in as children, which become text nodes; never as markup.',
  avoid: 'innerHTML with anything but the static icon paths. el.append(null) prints "null": pass children through h() or mount(), which skip null, false and true.',
  props: {
    'h(tag, props, ...children)': 'props: class (a string, or an array whose falsy entries are dropped), style (an object), dataset, on<Event> handlers, attributes. Children nest in arrays.',
    'icon(name, size = 16, cls)': 'An inline SVG with aria-hidden="true": an icon is never the only name a control has.',
  },
  a11y: 'Icons are hidden from assistive technology; the control or text beside them carries the name.',
};
