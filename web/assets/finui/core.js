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
  refresh: '<path d="M4 11a8 8 0 0 1 14.5-4"/><path d="M19 3v4h-4"/><path d="M20 13a8 8 0 0 1-14.5 4"/><path d="M5 21v-4h4"/>',
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
  lock: '<rect x="5" y="11" width="14" height="10" rx="2"/><path d="M8 11V7a4 4 0 0 1 8 0v4"/>',
  unlock: '<rect x="5" y="11" width="14" height="10" rx="2"/><path d="M8 11V7a4 4 0 0 1 7.6-1.7"/>',
  shuffle: '<path d="M3 7h3.5c2 0 3.2 1 4.3 2.7l2.4 4.6c1.1 1.7 2.3 2.7 4.3 2.7H21"/><path d="m18 14 3 3-3 3"/><path d="M3 17h3.5c1.3 0 2.2-.4 3-1.2"/><path d="M14.5 8.2c.8-.8 1.7-1.2 3-1.2H21"/><path d="m18 4 3 3-3 3"/>',
  anchor: '<circle cx="12" cy="5" r="2"/><path d="M12 7v14"/><path d="M8.5 10.5h7"/><path d="M4.5 13.5a7.5 7.5 0 0 0 15 0"/>',
  volumeUp: '<path d="M4 10v4h3.500L12 18V6l-4.500 4z"/><path d="M18 9v6"/><path d="M15 12h6"/>',
  volumeDown: '<path d="M4 10v4h3.500L12 18V6l-4.500 4z"/><path d="M15 12h6"/>',
  volumeLow: '<path d="M4 10v4h3.500L12 18V6l-4.500 4z"/><path d="M15.500 9a4 4 0 0 1 0 6"/>',
  volumeMute: '<path d="M4 10v4h3.500L12 18V6l-4.500 4z"/><path d="m16 9.500 5 5"/><path d="m21 9.500-5 5"/>',
  volumeOff: '<path d="M4 10v4h3.500L12 18V6l-4.500 4z"/><path d="M15.500 9a4 4 0 0 1 0 6"/><path d="M18 6.500a8 8 0 0 1 0 11"/><path d="M3 3l18 18"/>',
  headset: '<path d="M5 11v-1a7 7 0 0 1 14 0v1"/><rect x="3" y="11" width="4.500" height="6.500" rx="1.750"/><rect x="16.500" y="11" width="4.500" height="6.500" rx="1.750"/><path d="M5.250 17.500v.5a2.500 2.500 0 0 0 2.500 2.500H10"/><rect x="10" y="19.250" width="3.500" height="2.500" rx="1.250"/>',
  headsetOff: '<path d="M5 11v-1a7 7 0 0 1 14 0v1"/><rect x="3" y="11" width="4.500" height="6.500" rx="1.750"/><rect x="16.500" y="11" width="4.500" height="6.500" rx="1.750"/><path d="M5.250 17.500v.5a2.500 2.500 0 0 0 2.500 2.500H10"/><rect x="10" y="19.250" width="3.500" height="2.500" rx="1.250"/><path d="M3 3l18 18"/>',
  headphones: '<path d="M4.500 13.500V12a7.500 7.500 0 0 1 15 0v1.500"/><rect x="3" y="13.500" width="4.500" height="6.500" rx="1.750"/><rect x="16.500" y="13.500" width="4.500" height="6.500" rx="1.750"/>',
  mic: '<rect x="9" y="2.500" width="6" height="11.500" rx="3"/><path d="M5.500 10.500v.5a6.500 6.500 0 0 0 13 0v-.5"/><path d="M12 17.500V21"/><path d="M8.500 21h7"/>',
  micOff: '<rect x="9" y="2.500" width="6" height="11.500" rx="3"/><path d="M5.500 10.500v.5a6.500 6.500 0 0 0 13 0v-.5"/><path d="M12 17.500V21"/><path d="M8.500 21h7"/><path d="M3 3l18 18"/>',
  speaker: '<rect x="5" y="2.500" width="14" height="19" rx="2.500"/><path d="M12 7v.01"/><circle cx="12" cy="14.500" r="3.500"/>',
  music: '<path d="M9 17.500V5.500L20 3v12.500"/><path d="M9 9.500 20 7"/><circle cx="6.500" cy="17.500" r="2.500"/><circle cx="17.500" cy="15.500" r="2.500"/>',
  listMusic: '<path d="M3 6h11"/><path d="M3 11h11"/><path d="M3 16h7"/><circle cx="15" cy="17.500" r="2.500"/><path d="M17.500 17.500V5l3.500 1.500"/>',
  radio: '<rect x="2.500" y="8" width="19" height="12.500" rx="2"/><path d="m6.500 8 10-4.500"/><circle cx="8.500" cy="14.250" r="3"/><path d="M14.500 12h3.500"/><path d="M14.500 16.500h3.500"/>',
  podcast: '<circle cx="12" cy="10.500" r="2.250"/><path d="M12 15.500V21"/><path d="M8 13.500a5 5 0 1 1 8 0"/><path d="M5.500 16a8.500 8.500 0 1 1 13 0"/>',
  rewind: '<path d="M11.500 6.500 3.500 12l8 5.500z"/><path d="m20.500 6.500-8 5.500 8 5.500z"/>',
  fastForward: '<path d="m12.500 6.500 8 5.500-8 5.500z"/><path d="m3.500 6.500 8 5.500-8 5.500z"/>',
  skipBack: '<path d="M5.500 5v14"/><path d="M18.500 5.500 9 12l9.500 6.500z"/>',
  playCircle: '<circle cx="12" cy="12" r="9"/><path d="M10 8.500v7l5.500-3.500z"/>',
  pauseCircle: '<circle cx="12" cy="12" r="9"/><path d="M10 9v6"/><path d="M14 9v6"/>',
  record: '<circle cx="12" cy="12" r="9"/><circle cx="12" cy="12" r="3.500"/>',
  disc: '<circle cx="12" cy="12" r="9"/><circle cx="12" cy="12" r="2"/><path d="M12 6.500a5.500 5.500 0 0 1 5.500 5.500"/>',
  equalizer: '<path d="M4 20v-7"/><path d="M8 20V8"/><path d="M12 20V11"/><path d="M16 20V4"/><path d="M20 20v-9"/>',
  waveform: '<path d="M3 10.500v3"/><path d="M6.500 7v10"/><path d="M10 4v16"/><path d="M13.500 8.500v7"/><path d="M17 6v12"/><path d="M20.500 10v4"/>',
  cast: '<path d="M3 8V6a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2h-5"/><path d="M3 20h.01"/><path d="M3 16a4 4 0 0 1 4 4"/><path d="M3 12a8 8 0 0 1 8 8"/>',
  airplay: '<path d="M7 18H5a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2h-2"/><path d="m12 15 4.500 5.500h-9z"/>',
  video: '<rect x="2.500" y="6" width="13" height="12" rx="2.500"/><path d="M15.500 10 21 7v10l-5.500-3"/><path d="M6 9.500h.01"/>',
  videoOff: '<rect x="2.500" y="6" width="13" height="12" rx="2.500"/><path d="M15.500 10 21 7v10l-5.500-3"/><path d="M3 3l18 18"/>',
  camera: '<path d="M3 8.500a2 2 0 0 1 2-2h2.500L9 4h6l1.500 2.500H19a2 2 0 0 1 2 2V18a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/><circle cx="12" cy="13" r="3.500"/><path d="M17.500 9.500h.01"/>',
  cameraOff: '<path d="M3 8.500a2 2 0 0 1 2-2h2.500L9 4h6l1.500 2.500H19a2 2 0 0 1 2 2V18a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/><circle cx="12" cy="13" r="3.500"/><path d="M3 3l18 18"/>',
  image: '<rect x="3" y="4" width="18" height="16" rx="2"/><circle cx="8.500" cy="9.500" r="1.750"/><path d="m3 17.500 5-4.500 3.500 3 4-4.500 5.500 5"/>',
  repeatOne: '<path d="m17 2 4 4-4 4"/><path d="M3 11v-1a4 4 0 0 1 4-4h14"/><path d="m7 22-4-4 4-4"/><path d="M21 13v1a4 4 0 0 1-4 4H3"/><path d="M10.750 10.500 12.500 9.250V15"/>',
  pencil: '<path d="M16.4 4.4a2.26 2.26 0 0 1 3.2 3.2L8.6 18.6 4 20l1.4-4.6z"/><path d="m13.4 7.4 3.2 3.2"/>',
  penSquare: '<path d="M11 4H6a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-5"/><path d="M16.7 4.7a1.84 1.84 0 0 1 2.6 2.6l-6.5 6.5L9 15l1.2-3.8z"/>',
  eraser: '<path d="M20.5 20.5H8.5"/><path d="m8.5 20.5-5-5 9.5-9.5 5 5z"/><path d="m7.8 11.2 5 5"/>',
  highlighter: '<path d="M3.5 20.5h7"/><path d="m9.5 10.5 6.3-6.3a1.6 1.6 0 0 1 2.3 0l1.7 1.7a1.6 1.6 0 0 1 0 2.3l-6.3 6.3z"/><path d="m9.5 10.5-3.5 3.5v3h3l4.5-2.5"/>',
  brush: '<path d="M10.5 11.5 18 4a2 2 0 0 1 2.8 2.8l-7.5 7.5"/><path d="M8.5 13.5c-2 0-3.5 1.6-3.5 3.5 0 1.5-1 2.5-2 3 1 .8 2.5 1 4 1 2.6 0 4.5-2 4.5-4.5 0-1.7-1.3-3-3-3z"/>',
  palette: '<path d="M12 3a9 9 0 0 0 0 18c1.2 0 2-.8 2-1.9 0-.5-.2-.9-.5-1.3s-.5-.8-.5-1.3c0-1 .8-1.8 1.8-1.8H17a4 4 0 0 0 4-4C21 6.4 17 3 12 3z"/><path d="M7.5 12.5v.01"/><path d="M8.5 8v.01"/><path d="M12.5 6.5v.01"/><path d="M16.5 8.5v.01"/>',
  scissors: '<circle cx="6" cy="6.5" r="2.5"/><path d="M7.9 8.1 19.5 19.5"/><circle cx="6" cy="17.5" r="2.5"/><path d="M7.9 15.9 19.5 4.5"/>',
  clipboard: '<path d="M8 4.5H7a2 2 0 0 0-2 2V19a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V6.5a2 2 0 0 0-2-2h-1"/><rect x="8" y="3" width="8" height="3.5" rx="1"/>',
  clipboardCheck: '<path d="M8 4.5H7a2 2 0 0 0-2 2V19a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V6.5a2 2 0 0 0-2-2h-1"/><rect x="8" y="3" width="8" height="3.5" rx="1"/><path d="m9 13.5 2 2 4-4"/>',
  file: '<path d="M13.5 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8.5z"/><path d="M13.5 3v4.5a1 1 0 0 0 1 1H19"/>',
  fileText: '<path d="M13.5 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8.5z"/><path d="M13.5 3v4.5a1 1 0 0 0 1 1H19"/><path d="M9 12.5h6"/><path d="M9 16h6"/><path d="M9 9h2"/>',
  filePlus: '<path d="M13.5 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8.5z"/><path d="M13.5 3v4.5a1 1 0 0 0 1 1H19"/><path d="M12 11.5v6"/><path d="M9 14.5h6"/>',
  folder: '<path d="M3 6.5a2 2 0 0 1 2-2h4l2 2.5h8a2 2 0 0 1 2 2V18a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>',
  folderOpen: '<path d="M5 20a2 2 0 0 1-2-2V5.5a2 2 0 0 1 2-2h3.5l2 2.5H17a2 2 0 0 1 2 2V10"/><path d="m5 20 2.6-8.6A2 2 0 0 1 9.5 10H20a1.5 1.5 0 0 1 1.4 2l-2.4 6.6a2 2 0 0 1-1.9 1.4z"/>',
  folderPlus: '<path d="M3 6.5a2 2 0 0 1 2-2h4l2 2.5h8a2 2 0 0 1 2 2V18a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/><path d="M12 10.5v6"/><path d="M9 13.5h6"/>',
  save: '<path d="M6 3.5h9.6l4.4 4.4v10.6a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2v-13a2 2 0 0 1 2-2z"/><path d="M7.5 20.5v-6h9v6"/><path d="M8 3.5v4h6.5v-4"/>',
  paperclip: '<path d="M16.5 9v7.5a4.5 4.5 0 0 1-9 0V6a3 3 0 0 1 6 0v10a1.5 1.5 0 0 1-3 0V9"/>',
  type: '<path d="M5 7V5h14v2"/><path d="M12 5v14"/><path d="M9.5 19h5"/>',
  alignLeft: '<path d="M4 6h16"/><path d="M4 10h10"/><path d="M4 14h16"/><path d="M4 18h12"/>',
  alignCenter: '<path d="M4 6h16"/><path d="M7 10h10"/><path d="M4 14h16"/><path d="M6 18h12"/>',
  alignRight: '<path d="M4 6h16"/><path d="M10 10h10"/><path d="M4 14h16"/><path d="M8 18h12"/>',
  listOrdered: '<path d="M10.5 6h10"/><path d="M10.5 12h10"/><path d="M10.5 18h10"/><path d="M4 5.2 5.6 4v5"/><path d="M4 14.4a1.6 1.6 0 0 1 3.1.5c0 1-3.1 2.2-3.1 3.1h3.2"/>',
  listChecks: '<path d="M12.5 6.5h8"/><path d="M12.5 12h8"/><path d="M12.5 17.5h8"/><path d="m3.5 6.5 2 2 3.5-3.5"/><path d="m3.5 16 2 2 3.5-3.5"/>',
  quote: '<path d="M10 11.5H5.5a1 1 0 0 1-1-1V8a1 1 0 0 1 1-1H9a1 1 0 0 1 1 1v3.5c0 2.8-1.4 4.5-4 5"/><path d="M19.5 11.5H15a1 1 0 0 1-1-1V8a1 1 0 0 1 1-1h3.5a1 1 0 0 1 1 1v3.5c0 2.8-1.4 4.5-4 5"/>',
  code: '<path d="m8.5 7-5 5 5 5"/><path d="m15.5 7 5 5-5 5"/><path d="m13.5 5-3 14"/>',
  terminal: '<rect x="3" y="4" width="18" height="16" rx="2"/><path d="m7 9.5 3 2.5-3 2.5"/><path d="M12.5 15h4.5"/>',
  undo: '<path d="M4 9h10a5 5 0 0 1 0 10h-3.5"/><path d="M8.5 4.5 4 9l4.5 4.5"/>',
  redo: '<path d="M20 9H10a5 5 0 0 0 0 10h3.5"/><path d="m15.5 4.5 4.5 4.5-4.5 4.5"/>',
  crop: '<path d="M7 3v12.5A1.5 1.5 0 0 0 8.5 17H21"/><path d="M17 21V8.5A1.5 1.5 0 0 0 15.5 7H3"/>',
  wand: '<path d="M4 20 15 9"/><path d="M18 2.5v5M15.5 5h5"/><path d="M10.5 3v3M9 4.5h3"/><path d="M19.5 11.5v3M18 13h3"/>',
  pipette: '<path d="M13.4 8.9 5.4 16.9 4 20l3.1-1.4 8-8"/><path d="m11.5 7 5.5 5.5"/><path d="m13 8.5 3.4-3.4a1.77 1.77 0 0 1 2.5 2.5L15.5 11"/>',
  book: '<path d="M12 6.5C10 5 7 4.5 3.5 5v13c3.5-.5 6.5 0 8.5 1.5"/><path d="M12 6.5c2-1.5 5-2 8.5-1.5v13c-3.5-.5-6.5 0-8.5 1.5"/><path d="M12 6.5v13"/><path d="M12 6.5c2-1.5 5-2 8.5-1.5v13c-3.5-.5-6.5 0-8.5 1.5"/>',
  mail: '<rect x="3" y="5" width="18" height="14" rx="2"/><path d="m3.5 7.5 8.5 5.5 8.5-5.5"/>',
  mailOpen: '<path d="M3 10.5V19a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-8.5"/><path d="M3 10.5 12 4l9 6.5"/><path d="m3.5 11.5 8.5 5.5 8.5-5.5"/><path d="M7.5 13.5V8h9v5.5"/>',
  send: '<path d="M21 3 3 10.2l7.5 3.3 3.3 7.5z"/><path d="m21 3-10.5 10.5"/>',
  message: '<path d="M3 6a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2H9l-6 4z"/>',
  messages: '<path d="M3 5a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v6a2 2 0 0 1-2 2H8l-5 3.5z"/><path d="M18.5 8h.5a2 2 0 0 1 2 2v11l-3.5-3H11a2 2 0 0 1-2-2v-1"/>',
  messageDots: '<path d="M3 6a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2H9l-6 4z"/><path d="M8 10.5v.01"/><path d="M12 10.5v.01"/><path d="M16 10.5v.01"/>',
  phone: '<path d="M5 3.5h3l1.5 4.5-2.2 1.5a9 9 0 0 0 7.2 7.2l1.5-2.2 4.5 1.5v3a2 2 0 0 1-2 2A16 16 0 0 1 3 5.5a2 2 0 0 1 2-2z"/>',
  phoneCall: '<path d="M5 3.5h3l1.5 4.5-2.2 1.5a9 9 0 0 0 7.2 7.2l1.5-2.2 4.5 1.5v3a2 2 0 0 1-2 2A16 16 0 0 1 3 5.5a2 2 0 0 1 2-2z"/><path d="M14 6.5a3.5 3.5 0 0 1 3.5 3.5"/><path d="M14 3a7 7 0 0 1 7 7"/>',
  phoneOff: '<path d="M5 3.5h3l1.5 4.5-2.2 1.5a9 9 0 0 0 7.2 7.2l1.5-2.2 4.5 1.5v3a2 2 0 0 1-2 2A16 16 0 0 1 3 5.5a2 2 0 0 1 2-2z"/><path d="M21 3 3 21"/>',
  bell: '<path d="M18 16.5V11a6 6 0 0 0-12 0v5.5L4 19h16z"/><path d="M10 21a2 2 0 0 0 4 0"/><path d="M12 3v2"/>',
  bellOff: '<path d="M18 16.5V11a6 6 0 0 0-12 0v5.5L4 19h16z"/><path d="M10 21a2 2 0 0 0 4 0"/><path d="M12 3v2"/><path d="M3 3l18 18"/>',
  bellRing: '<path d="M18 16.5V11a6 6 0 0 0-12 0v5.5L4 19h16z"/><path d="M10 21a2 2 0 0 0 4 0"/><path d="M12 3v2"/><path d="M2.5 10a10 10 0 0 1 2.5-5.5"/><path d="M21.5 10A10 10 0 0 0 19 4.5"/>',
  atSign: '<circle cx="12" cy="12" r="3.5"/><path d="M15.5 8.5v4.5a2.5 2.5 0 0 0 5 0v-1a8.5 8.5 0 1 0-3.4 6.8"/>',
  hash: '<path d="M4 9h16"/><path d="M4 15h16"/><path d="M10 3 8 21"/><path d="m16 3-2 18"/>',
  megaphone: '<path d="M3 10.5a1 1 0 0 1 1-1h3l9-4.5v14l-9-4.5H4a1 1 0 0 1-1-1z"/><path d="m7 14.5 1.5 5h2.5l-1.1-3.6"/><path d="M18.5 9.5a3.5 3.5 0 0 1 0 5"/><path d="M20.5 7.5a6.5 6.5 0 0 1 0 9"/>',
  thumbsUp: '<rect x="3" y="10" width="4" height="10" rx="1"/><path d="M7 11l3.5-7a2 2 0 0 1 2.5 2.5L12 10h6a2 2 0 0 1 2 2.4l-1.4 6A2 2 0 0 1 16.6 20H7"/>',
  thumbsDown: '<rect x="3" y="4" width="4" height="10" rx="1"/><path d="M7 13l3.5 7a2 2 0 0 0 2.5-2.5L12 14h6a2 2 0 0 0 2-2.4l-1.4-6A2 2 0 0 0 16.6 4H7"/>',
  smile: '<circle cx="12" cy="12" r="9"/><path d="M8.5 14.5a4 4 0 0 0 7 0"/><path d="M9 9.5v.01"/><path d="M15 9.5v.01"/>',
  star: '<path d="M12 3l2.5 6.2 6.6.4-5.1 4.3 1.6 6.5-5.6-3.6-5.6 3.6 1.6-6.5-5.1-4.3 6.6-.4z"/>',
  flag: '<path d="M5 21V4"/><path d="M5 4.5c2-1.3 4-1.3 6.5 0s4.5 1.3 7 0v9c-2.5 1.3-4.5 1.3-7 0s-4.5-1.3-6.5 0"/>',
  gift: '<path d="M5 11v9a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1v-9"/><path d="M12 11v10"/><rect x="3" y="7.5" width="18" height="3.5" rx="1"/><path d="M12 7.5V11"/><path d="M12 7.5C10.5 3.5 6.5 3.5 7 6c.3 1.3 2.5 1.5 5 1.5"/><path d="M12 7.5c1.5-4 5.5-4 5-1.5-.3 1.3-2.5 1.5-5 1.5"/>',
  userPlus: '<circle cx="9" cy="8" r="3.5"/><path d="M2.5 20c.6-3.6 3.2-5.5 6.5-5.5s5.9 1.9 6.5 5.5"/><path d="M19 8v6"/><path d="M16 11h6"/>',
  userMinus: '<circle cx="9" cy="8" r="3.5"/><path d="M2.5 20c.6-3.6 3.2-5.5 6.5-5.5s5.9 1.9 6.5 5.5"/><path d="M16 11h6"/>',
  userCheck: '<circle cx="9" cy="8" r="3.5"/><path d="M2.5 20c.6-3.6 3.2-5.5 6.5-5.5s5.9 1.9 6.5 5.5"/><path d="m16 11 2 2 4-4"/>',
  userX: '<circle cx="9" cy="8" r="3.5"/><path d="M2.5 20c.6-3.6 3.2-5.5 6.5-5.5s5.9 1.9 6.5 5.5"/><path d="m16.5 8.5 5 5"/><path d="m21.5 8.5-5 5"/>',
  crown: '<path d="M3 7l4 5 5-7 5 7 4-5-2 11H5z"/><path d="M5 21h14"/>',
  eye: '<path d="M2.5 12C5 7 8.5 5 12 5s7 2 9.5 7c-2.5 5-6 7-9.5 7s-7-2-9.5-7z"/><circle cx="12" cy="12" r="3"/>',
  eyeOff: '<path d="M2.5 12C5 7 8.5 5 12 5s7 2 9.5 7c-2.5 5-6 7-9.5 7s-7-2-9.5-7z"/><circle cx="12" cy="12" r="3"/><path d="M3 3l18 18"/>',
  reply: '<path d="M9 7l-5 5 5 5"/><path d="M4 12h10a6 6 0 0 1 6 6v1"/>',
  forward: '<path d="m15 7 5 5-5 5"/><path d="M20 12H10a6 6 0 0 0-6 6v1"/>',
  award: '<circle cx="12" cy="9" r="6"/><path d="M8.5 13.9 7 21.5l5-2.8 5 2.8-1.5-7.6"/>',
  contact: '<rect x="3" y="5" width="18" height="14" rx="2"/><circle cx="9" cy="10.5" r="2"/><path d="M5.5 16c.5-1.8 1.8-2.8 3.5-2.8s3 1 3.5 2.8"/><path d="M15 10h3"/><path d="M15 14h3"/>',
  arrowDown: '<path d="M12 5v14"/><path d="m6 13 6 6 6-6"/>',
  arrowLeft: '<path d="M19 12H5"/><path d="m11 6-6 6 6 6"/>',
  arrowRight: '<path d="M5 12h14"/><path d="m13 6 6 6-6 6"/>',
  arrowUpRight: '<path d="M7 17 17 7"/><path d="M8.5 7H17v8.5"/>',
  chevronUp: '<path d="m6 15 6-6 6 6"/>',
  chevronsUpDown: '<path d="m7 9 5-5 5 5"/><path d="m7 15 5 5 5-5"/>',
  moreHorizontal: '<circle cx="5.5" cy="12" r="1"/><circle cx="12" cy="12" r="1"/><circle cx="18.5" cy="12" r="1"/>',
  moreVertical: '<circle cx="12" cy="5.5" r="1"/><circle cx="12" cy="12" r="1"/><circle cx="12" cy="18.5" r="1"/>',
  grid: '<rect x="3.5" y="3.5" width="7" height="7" rx="1.5"/><rect x="13.5" y="3.5" width="7" height="7" rx="1.5"/><rect x="3.5" y="13.5" width="7" height="7" rx="1.5"/><rect x="13.5" y="13.5" width="7" height="7" rx="1.5"/>',
  filter: '<path d="M3.5 4h17l-6.5 7.5V17h-4v-5.5z"/><path d="M12 20.5v.01"/>',
  sortAsc: '<path d="M7 20V4"/><path d="m3.5 7.5 3.5-3.5 3.5 3.5"/><path d="M14 6h3"/><path d="M14 12h5"/><path d="M14 18h7"/>',
  sortDesc: '<path d="M7 4v16"/><path d="m3.5 16.5 3.5 3.5 3.5-3.5"/><path d="M14 6h7"/><path d="M14 12h5"/><path d="M14 18h3"/>',
  maximize: '<path d="M4 9V4h5"/><path d="M15 4h5v5"/><path d="M20 15v5h-5"/><path d="M9 20H4v-5"/>',
  minimize: '<path d="M4 9h5V4"/><path d="M15 4v5h5"/><path d="M20 15h-5v5"/><path d="M9 20v-5H4"/>',
  sidebar: '<rect x="3" y="4" width="18" height="16" rx="2"/><path d="M9 4v16"/>',
  login: '<path d="M14 4h5v16h-5"/><path d="m11 8 4 4-4 4"/><path d="M4 12h11"/>',
  power: '<path d="M17.1 6.4a8 8 0 1 1-10.2 0"/><path d="M12 3v8"/>',
  toggleOn: '<rect x="2" y="7" width="20" height="10" rx="5"/><circle cx="17" cy="12" r="2.5"/>',
  toggleOff: '<rect x="2" y="7" width="20" height="10" rx="5"/><circle cx="7" cy="12" r="2.5"/>',
  loader: '<path d="M20.5 12a8.5 8.5 0 1 1-8.5-8.5"/>',
  circleCheck: '<circle cx="12" cy="12" r="9"/><path d="m8 12.5 2.8 2.8L16 9.5"/>',
  circleX: '<circle cx="12" cy="12" r="9"/><path d="m9 9 6 6"/><path d="m15 9-6 6"/>',
  circleHelp: '<circle cx="12" cy="12" r="9"/><path d="M9.4 9.4a2.7 2.7 0 0 1 5.2 1c0 1.8-2.6 2.2-2.6 3.8"/><path d="M12 17v.01"/>',
  ban: '<circle cx="12" cy="12" r="9"/><path d="m5.6 5.6 12.8 12.8"/>',
  zoomIn: '<circle cx="11" cy="11" r="7"/><path d="m20 20-3.8-3.8"/><path d="M11 8v6"/><path d="M8 11h6"/>',
  zoomOut: '<circle cx="11" cy="11" r="7"/><path d="m20 20-3.8-3.8"/><path d="M8 11h6"/>',
  mapPin: '<path d="M12 21s-6.5-5.6-6.5-11a6.5 6.5 0 0 1 13 0c0 5.4-6.5 11-6.5 11z"/><circle cx="12" cy="10" r="2.5"/>',
  map: '<path d="M3 6.5 9 4l6 2.5L21 4v13.5L15 20l-6-2.5L3 20z"/><path d="M9 4v13.5"/><path d="M15 6.5V20"/>',
  navigation: '<path d="M4 11.5 20 4l-7.5 16-2-6.5z"/>',
  move: '<path d="M12 3v18"/><path d="M3 12h18"/><path d="m9 6 3-3 3 3"/><path d="m18 9 3 3-3 3"/><path d="m15 18-3 3-3-3"/><path d="m6 15-3-3 3-3"/>',
  grip: '<circle cx="9" cy="6" r="1"/><circle cx="15" cy="6" r="1"/><circle cx="9" cy="12" r="1"/><circle cx="15" cy="12" r="1"/><circle cx="9" cy="18" r="1"/><circle cx="15" cy="18" r="1"/>',
  pin: '<path d="M9.5 4h5v5l3 3.5V16h-11v-3.5l3-3.5z"/><path d="M8 4h8"/><path d="M12 16v5"/>',
  wifi: '<path d="M12 19.5h.01"/><path d="M9.2 16.7a4 4 0 0 1 5.6 0"/><path d="M6.3 13.8a8 8 0 0 1 11.4 0"/><path d="M3.5 11a12 12 0 0 1 17 0"/>',
  wifiOff: '<path d="M12 19.5h.01"/><path d="M9.2 16.7a4 4 0 0 1 5.6 0"/><path d="M6.3 13.8a8 8 0 0 1 11.4 0"/><path d="M3.5 11a12 12 0 0 1 17 0"/><path d="M3 3l18 18"/>',
  bluetooth: '<path d="M6.5 7.5 17 16.5l-5 4.5V3l5 4.5-10.5 9"/>',
  battery: '<rect x="2" y="7" width="17" height="10" rx="2"/><path d="M22 10.5v3"/><path d="M6 10v4"/><path d="M10.5 10v4"/><path d="M15 10v4"/>',
  batteryCharging: '<rect x="2" y="7" width="17" height="10" rx="2"/><path d="M22 10.5v3"/><path d="M12.4 8.9 9.2 12.3h4.4l-3 3"/>',
  smartphone: '<rect x="6" y="2.5" width="12" height="19" rx="2.5"/><path d="M12 18h.01"/>',
  laptop: '<rect x="4" y="4" width="16" height="11" rx="2"/><path d="M4.5 15 2 19.5h20L19.5 15"/>',
  printer: '<path d="M6.5 17.5h-2a2 2 0 0 1-2-2v-5a2 2 0 0 1 2-2h15a2 2 0 0 1 2 2v5a2 2 0 0 1-2 2h-2"/><path d="M6.5 8.5v-5h11v5"/><path d="M17.5 11.5h.01"/><rect x="6.5" y="14" width="11" height="7.5" rx="1"/>',
  keyboard: '<rect x="2" y="5" width="20" height="14" rx="2"/><path d="M6 9h.01"/><path d="M10 9h.01"/><path d="M14 9h.01"/><path d="M18 9h.01"/><path d="M8 12.2h.01"/><path d="M12 12.2h.01"/><path d="M16 12.2h.01"/><path d="M8 15.5h8"/>',
  mouse: '<rect x="6" y="3" width="12" height="18" rx="6"/><path d="M12 7v2.5"/>',
  hardDrive: '<path d="M3 15 5.5 6.3a2 2 0 0 1 1.9-1.4h9.2a2 2 0 0 1 1.9 1.4L21 15"/><rect x="3" y="13" width="18" height="7" rx="2"/><path d="M7 16.5h.01"/><path d="M10.5 16.5h.01"/>',
  cloud: '<path d="M7 19a4.5 4.5 0 0 1-.6-9 6 6 0 0 1 11.3 1.1A4 4 0 0 1 17.5 19z"/>',
  cloudUpload: '<path d="M8 16H7a4.5 4.5 0 0 1-.6-9 6 6 0 0 1 11.3 1.1A4 4 0 0 1 17.5 16H16"/><path d="M12 21v-9"/><path d="m8.5 15.5 3.5-3.5 3.5 3.5"/>',
  cloudDownload: '<path d="M8 16H7a4.5 4.5 0 0 1-.6-9 6 6 0 0 1 11.3 1.1A4 4 0 0 1 17.5 16H16"/><path d="M12 12v9"/><path d="m8.5 17.5 3.5 3.5 3.5-3.5"/>',
  cloudRain: '<path d="M7 15a4.5 4.5 0 0 1-.6-9 6 6 0 0 1 11.3 1.1A4 4 0 0 1 17.5 15z"/><path d="M8.5 18 7.5 20.5"/><path d="M12.5 18l-1 2.5"/><path d="M16.5 18l-1 2.5"/>',
  snowflake: '<path d="M12 3v18"/><path d="M19.79 7.5 4.21 16.5"/><path d="M19.79 16.5 4.21 7.5"/><path d="M13.91 4.09 12 6l-1.91-1.91"/><path d="M19.8 9.7 17.2 9l.69-2.61"/><path d="M17.89 17.61 17.2 15l2.6-.7"/><path d="M10.09 19.91 12 18l1.91 1.91"/><path d="M4.2 14.3 6.8 15l-.69 2.61"/><path d="M6.11 6.39 6.8 9l-2.6.7"/>',
  zap: '<path d="M13.5 2.5 4.5 13.5h7l-1 8 9-11h-7z"/>',
  umbrella: '<path d="M3 12a9 9 0 0 1 18 0 3.6 3.6 0 0 0-6 0 3.6 3.6 0 0 0-6 0 3.6 3.6 0 0 0-6 0z"/><path d="M12 12v7.5a2 2 0 0 1-4 0"/><path d="M12 2v1"/>',
  wind: '<path d="M3 8h8.5a2.5 2.5 0 1 0-2.5-2.5"/><path d="M3 12h14.5a2.5 2.5 0 1 0-2.5-2.5"/><path d="M3 16h10a2.5 2.5 0 1 1-2.5 2.5"/>',
  thermometer: '<path d="M10 13.5V5a2 2 0 0 1 4 0v8.5a3.8 3.8 0 1 1-4 0z"/><path d="M12 16.7V9"/>',
  droplet: '<path d="M12 2.500c3.300 4 6.500 7.600 6.500 11.500a6.500 6.500 0 0 1-13 0c0-3.900 3.200-7.500 6.500-11.500z"/><path d="M9 14.500a3 3 0 0 0 2.500 3"/>',
  key: '<circle cx="12" cy="7" r="4.500"/><path d="M12 7v.01"/><path d="M12 11.500v10"/><path d="M12 16.500h3"/><path d="M12 19.500h2"/>',
  fingerprint: '<path d="M4.500 16a8 8 0 0 1-.500-3.500 8 8 0 0 1 15.200-3.500"/><path d="M19.900 12.500c.100 2-.100 4-.700 6"/><path d="M8 20c-.800-2-1.200-4.300-1.200-7a5.200 5.200 0 0 1 10.400 0c0 2.200-.200 4.300-.700 6.300"/><path d="M12 13c0 3.300.400 6 1.200 8.500"/><path d="M9.800 21c-.500-1.300-.800-3.600-.800-7.500a3 3 0 0 1 6 0"/>',
  cart: '<path d="M2.500 3.500h2.500l2.300 11.500h11l2-8H6"/><circle cx="9" cy="19.500" r="1.500"/><circle cx="17" cy="19.500" r="1.500"/>',
  creditCard: '<rect x="2.500" y="5" width="19" height="14" rx="2"/><path d="M2.500 9.500h19"/><path d="M6 15h4"/>',
  wallet: '<rect x="3" y="6" width="18" height="14" rx="2"/><path d="M5.500 6 15 3.200l1 2.800"/><path d="M21 10.500h-4a2.500 2.500 0 0 0 0 5h4"/><path d="M17 13h.01"/>',
  coins: '<circle cx="9" cy="14.500" r="6"/><path d="M9 12v5"/><path d="M9.100 8.500A6 6 0 1 1 14.900 15.500"/>',
  receipt: '<path d="M6 3h12v18l-2-1.500-2 1.500-2-1.500-2 1.500-2-1.500L6 21z"/><path d="M9 8h6"/><path d="M9 11.500h6"/><path d="M9 15h3.500"/>',
  package: '<path d="M12 2.500 20.500 7v10L12 21.500 3.500 17V7z"/><path d="M3.500 7 12 11.500 20.500 7"/><path d="M12 11.500v10"/><path d="m7.800 4.800 8.500 4.500"/>',
  truck: '<path d="M14 17V6a1 1 0 0 0-1-1H3a1 1 0 0 0-1 1v10a1 1 0 0 0 1 1h1.500"/><path d="M8.500 17h6"/><path d="M14 9h3.500l3.500 4v3a1 1 0 0 1-1 1h-1.500"/><circle cx="6.500" cy="17.500" r="2"/><circle cx="16.500" cy="17.500" r="2"/>',
  rocket: '<path d="M14 5c2-1.600 4.200-2.200 6-2 .200 1.800-.400 4-2 6l-6.500 6.500-4-4z"/><path d="M9.500 9.500 6 9.200 3.500 11.500 7.500 13"/><path d="M14.500 14.500l.300 3.500-2.300 2.500L11 16.500"/><circle cx="16" cy="8" r="1.300"/><path d="M6.500 17.500 4 20"/>',
  lightbulb: '<path d="M9.500 17c0-1.500-.700-2.300-1.700-3.300a5.500 5.500 0 1 1 8.400 0c-1 1-1.700 1.800-1.700 3.300z"/><path d="M9.500 20h5"/><path d="M3.500 10H5"/><path d="m5.600 4.100 1 1"/><path d="M12 1.500V3"/><path d="m18.400 4.100-1 1"/><path d="M20.500 10H19"/>',
  coffee: '<path d="M4 10h13v5a5 5 0 0 1-5 5H9a5 5 0 0 1-5-5z"/><path d="M17 11.500h1.500a2.500 2.500 0 0 1 0 5H17"/><path d="M7.500 3c-.800.900-.800 1.800 0 2.700s.800 1.800 0 2.700"/><path d="M10.500 3c-.800.900-.800 1.800 0 2.700s.800 1.800 0 2.700"/><path d="M13.500 3c-.800.900-.800 1.800 0 2.700s.800 1.800 0 2.700"/>',
  bug: '<rect x="7.500" y="8.500" width="9" height="12" rx="4.500"/><path d="M9.500 8.500a2.500 2.500 0 0 1 5 0"/><path d="M12 13v7.500"/><path d="M10.500 6.200 9 3.500"/><path d="m13.500 6.200 1.500-2.700"/><path d="M7.500 12 4 10.500"/><path d="M7.500 15H3.500"/><path d="M7.700 18 4 19.500"/><path d="m16.500 12 3.500-1.500"/><path d="M16.500 15h4"/><path d="m16.300 18 3.700 1.500"/>',
  gamepad: '<path d="M7 6h10a5 5 0 0 1 5 5v4.500a3 3 0 0 1-5.400 1.800L15 15.500H9l-1.600 1.800A3 3 0 0 1 2 15.500V11a5 5 0 0 1 5-5z"/><path d="M5.500 11h4"/><path d="M7.500 9v4"/><path d="M15 12h.01"/><path d="M17.500 9.500h.01"/>',
  puzzle: '<path d="M3.750 7.750h4.500a2.300 2.300 0 1 1 4 0h4.500v4.500a2.300 2.300 0 1 1 0 4v4.500h-13z"/>',
  hourglass: '<path d="M6 2.500h12"/><path d="M6 21.500h12"/><path d="M7.500 2.500v2c0 3 4.500 4.500 4.500 7.500s-4.500 4.500-4.500 7.500v2"/><path d="M16.500 2.500v2c0 3-4.500 4.500-4.500 7.500s4.500 4.500 4.500 7.500v2"/><path d="M8.500 21.500c0-2 1.500-3.500 3.500-4 2 .500 3.500 2 3.500 4"/>',
  timer: '<circle cx="12" cy="13.500" r="7.500"/><path d="M12 13.500v-4"/><path d="M10 2.500h4"/><path d="M12 2.500V6"/><path d="m18.500 6.500 1.500-1.500"/>',
  alarm: '<circle cx="12" cy="13" r="7.500"/><path d="M12 9.500V13l2.500 1.500"/><path d="M3 7.500A4 4 0 0 1 7.500 3"/><path d="M21 7.500A4 4 0 0 0 16.500 3"/><path d="m6.800 18.500-1.800 2.500"/><path d="m17.200 18.500 1.800 2.500"/>',
  target: '<circle cx="11" cy="13" r="8.500"/><circle cx="11" cy="13" r="4.500"/><path d="M11 13 20.500 3.500"/><path d="M18 3.500V6h2.500"/>',
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
