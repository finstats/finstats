// The world, hand-drawn like every other chart here: country outlines from /assets/geo/world.json
// (Natural Earth, projected ahead of time by tools/make-world-map.py) and dots placed with the same
// Mercator projection: north is straight up wherever you zoom. No tiles, no map service: nothing about a place ever leaves the browser.

import { h, s, icon, num } from './dom.js';
import { showTip, hideTip, tipShown } from './charts.js';
import { button } from '../finui/components/button/button.js';

const MIN_W = 70;            // the closest zoom, in map units (the whole world is 2000 wide)
// How it stays smooth. The coastline is 445 kB of path, and drawing it again is what costs: every change of the viewBox
// rasterises all of it, and anything animated inside the SVG makes the browser record all of it again every frame.
// So during a drag or a zoom the picture already drawn is only moved and scaled (a CSS transform the GPU does), and the
// map is drawn again once, when the gesture settles. It is drawn with a margin of half a frame all round (the stage is
// twice the frame each way), so a drag uncovers land rather than empty sea. The pulse of somebody playing now is HTML
// above the SVG, animated by transform and opacity, so it never touches the map at all.
const SETTLE_MS = 160;       // a gesture has settled when nothing has moved for this long
const KINDS = {
  home: { label: 'Home network', cls: 'wm-home' },
  remote: { label: 'Away from home', cls: 'wm-remote' },
  failed: { label: 'Failed sign-ins', cls: 'wm-failed' },
  live: { label: 'Playing now', cls: 'wm-live' },
};

let worldPromise = null;
function loadWorld() {
  if (!worldPromise) {
    worldPromise = fetch('/assets/geo/world.json').then((r) => { if (!r.ok) throw new Error('The map could not be loaded'); return r.json(); })
      .catch((e) => { worldPromise = null; throw e; });
  }
  return worldPromise;
}

function projector(world) {
  return (lon, lat) => {
    const phi = Math.max(-85, Math.min(85, lat)) * Math.PI / 180;
    return [(lon * Math.PI / 180) * world.scale + world.width / 2, -Math.log(Math.tan(Math.PI / 4 + phi / 2)) * world.scale - world.top];
  };
}

/**
 * points: [{ kind: 'home'|'remote'|'failed'|'live', latitude, longitude, weight, label, tip: () => Node, data }]
 * Returns { el, show(coords, { line }), destroy }.
 */
export function worldMap({ points, countries = [], onPick, signal }) {
  const svg = s('svg', { class: 'wm-svg', role: 'group', 'aria-label': 'World map. Drag to move, use the buttons or Ctrl and the wheel to zoom.', tabindex: '0' });
  const land = s('g', { class: 'wm-lands' }), routeLayer = s('g', { class: 'wm-routes' }), dotLayer = s('g', { class: 'wm-dots' });
  svg.append(land, routeLayer, dotLayer);
  const hint = h('div', { class: 'wm-hint', hidden: true }, 'Hold Ctrl and scroll to zoom');
  const status = h('div', { class: 'wm-status' }, 'Loading the map…');
  const zoomIn = button({ size: 'sm', class: 'wm-btn', type: 'button', 'aria-label': 'Zoom in', title: 'Zoom in (+)' }, icon('plus', 14));
  const zoomOut = button({ size: 'sm', class: 'wm-btn', type: 'button', 'aria-label': 'Zoom out', title: 'Zoom out (−)' }, icon('minus', 14));
  const fitBtn = button({ size: 'sm', class: 'wm-btn', type: 'button', 'aria-label': 'Show every dot', title: 'Show every dot (0)' }, icon('compass', 14));
  const pulses = h('div', { class: 'wm-pulses', 'aria-hidden': 'true' });
  const stage = h('div', { class: 'wm-stage' }, svg, pulses);
  const frame = h('div', { class: 'wm-frame' }, stage, status, hint, h('div', { class: 'wm-controls' }, zoomIn, zoomOut, fitBtn));
  const used = [...new Set(points.map((p) => p.kind))];
  const legend = h('ul', { class: 'legend wm-legend' }, Object.keys(KINDS).filter((k) => used.includes(k)).map((k) =>
    h('li', null, h('span', { class: 'wm-key ' + KINDS[k].cls }), KINDS[k].label)));
  const el = h('div', { class: 'wm' }, frame, legend);

  // `view` is what the frame shows; `drawn` is what the SVG was last drawn for. Between the two is a transform.
  let world = null, project = null, view = null, drawn = null, route = null, hintTimer = 0, settleTimer = 0;
  let placed = [];
  // The dots as drawn, bottom to top: what the pointer is over is worked out from these (`hover`).
  let shown = [], hovering = null;
  const hot = new Set(countries);
  let size = { w: 0, h: 0 };
  const measure = () => { const r = frame.getBoundingClientRect(); size = { w: r.width, h: r.height }; };

  const aspect = () => (size.w > 0 && size.h > 0 ? size.w / size.h : 2);
  const unitsPerPx = () => view.w / Math.max(1, size.w);

  function clampView(v) {
    const a = aspect();
    let w = Math.min(Math.max(v.w, MIN_W), Math.max(world.width, world.height * a));
    let hgt = w / a;
    const cx = v.x + v.w / 2, cy = v.y + v.h / 2;
    let x = cx - w / 2, y = cy - hgt / 2;
    // Keep the world in sight: centre it when it is smaller than the frame, otherwise let at most a third of the
    // frame be empty sea, so that dots in the far north can still sit in the middle of a tall phone screen.
    const mx = w * 0.35, my = hgt * 0.35;
    x = w >= world.width ? (world.width - w) / 2 : Math.min(Math.max(x, -mx), world.width - w + mx);
    y = hgt >= world.height ? (world.height - hgt) / 2 : Math.min(Math.max(y, -my), world.height - hgt + my);
    return { x, y, w, h: hgt };
  }

  // Move to `v`. Now (`settle`), or by moving what is drawn and drawing it again once nothing has moved for a moment.
  function setView(v, { settle = false } = {}) {
    view = clampView(v);
    if (settle || !drawn) return draw();
    const scale = drawn.w / view.w;
    const tx = ((drawn.x - view.x) / view.w) * size.w, ty = ((drawn.y - view.y) / view.h) * size.h;
    // Past the margin drawn around the frame there is nothing to move into sight: draw it now instead.
    if (scale < 0.55 || scale > 1.9 || Math.abs(tx) > size.w * 0.45 || Math.abs(ty) > size.h * 0.45) return draw();
    stage.style.transform = `translate(${tx.toFixed(1)}px, ${ty.toFixed(1)}px) scale(${scale.toFixed(4)})`;
    clearTimeout(settleTimer);
    if (!drag) settleTimer = setTimeout(draw, SETTLE_MS);
  }

  // Draw the map for `view`, with half a frame of margin on every side.
  function draw() {
    clearTimeout(settleTimer);
    drawn = { ...view };
    svg.setAttribute('viewBox', `${(view.x - view.w / 2).toFixed(2)} ${(view.y - view.h / 2).toFixed(2)} ${(view.w * 2).toFixed(2)} ${(view.h * 2).toFixed(2)}`);
    frame.dataset.view = [view.x, view.y, view.w, view.h].map((n) => n.toFixed(2)).join(' ');
    stage.style.transform = '';
    layout();
  }

  function fit(list, { pad = 1.35, minW = 420 } = {}) {
    const pts = list.filter((p) => p.xy);
    if (!pts.length) return setView({ x: 0, y: 0, w: world.width, h: world.height }, { settle: true });
    const xs = pts.map((p) => p.xy[0]), ys = pts.map((p) => p.xy[1]);
    const x0 = Math.min(...xs), x1 = Math.max(...xs), y0 = Math.min(...ys), y1 = Math.max(...ys);
    const a = aspect();
    const w = Math.max((x1 - x0) * pad, (y1 - y0) * pad * a, minW);
    setView({ x: (x0 + x1) / 2 - w / 2, y: (y0 + y1) / 2 - w / a / 2, w, h: w / a }, { settle: true });
  }

  // The opening view is about the people: failed sign-ins come from everywhere and would always make it the whole world.
  function fitAll() {
    const people = placed.filter((p) => p.kind !== 'failed');
    fit(people.length ? people : placed);
  }

  function zoomBy(factor, at) {
    const c = at || [view.x + view.w / 2, view.y + view.h / 2];
    const w = view.w * factor, hgt = view.h * factor;
    setView({ x: c[0] - (c[0] - view.x) * factor, y: c[1] - (c[1] - view.y) * factor, w, h: hgt });
  }

  const radiusPx = (weight) => Math.min(17, 4.5 + Math.sqrt(Math.max(1, weight)) * 0.9);

  // Dots that would sit on top of each other become one, until the zoom pulls them apart.
  function cluster() {
    const k = unitsPerPx();
    const groups = [];
    for (const p of [...placed].sort((a, b) => b.weight - a.weight)) {
      const near = groups.find((g) => g.kind === p.kind && Math.hypot(g.xy[0] - p.xy[0], g.xy[1] - p.xy[1]) / k < (radiusPx(g.weight) + radiusPx(p.weight)) * 0.75);
      if (near) { near.members.push(p); near.weight += p.weight; } else groups.push({ kind: p.kind, xy: p.xy, weight: p.weight, members: [p] });
    }
    return groups;
  }

  function layout() {
    if (!world) return;
    hideTip();
    const k = unitsPerPx();
    const order = { home: 0, remote: 1, failed: 2, live: 3 };
    const groups = cluster().sort((a, b) => order[a.kind] - order[b.kind] || b.weight - a.weight);
    // Where on the stage a point is, as a share of it: the stage is the drawn area, twice the frame each way.
    const at = (xy) => ({ left: `${(((xy[0] - drawn.x + drawn.w / 2) / (drawn.w * 2)) * 100).toFixed(3)}%`, top: `${(((xy[1] - drawn.y + drawn.h / 2) / (drawn.h * 2)) * 100).toFixed(3)}%` });
    pulses.replaceChildren(...groups.filter((g) => g.kind === 'live').map((g) => {
      const d = radiusPx(g.weight) * 1.9 * 2;
      return h('span', { class: 'wm-pulse', style: { ...at(g.xy), width: `${d}px`, height: `${d}px` } });
    }));
    shown = []; hovering = null;
    dotLayer.replaceChildren(...groups.map((g) => {
      const r = radiusPx(g.weight) * k, one = g.members.length === 1 ? g.members[0] : null;
      const label = one ? one.label : `${g.members.length} places near ${g.members[0].label}`;
      const node = s('g', { class: 'wm-dot ' + KINDS[g.kind].cls, tabindex: '0', role: 'button', 'aria-label': `${KINDS[g.kind].label}: ${label}` },
        s('circle', { class: 'wm-mark', cx: g.xy[0], cy: g.xy[1], r }),
        one ? null : s('text', { class: 'wm-count', x: g.xy[0], y: g.xy[1], 'font-size': 10.5 * k, dy: '0.35em' }, String(g.members.length)));
      const tip = () => (one ? one.tip() : h('div', null, h('div', { class: 'fui-tooltip__title' }, `${g.members.length} places`),
        g.members.slice(0, 6).map((m) => h('div', { class: 'fui-tooltip__row' }, h('span', null, m.label))), h('div', { class: 'fui-tooltip__title' }, 'Click to zoom in')));
      const pick = () => { if (one) { if (onPick) onPick(one); } else fit(g.members, { pad: 1.8, minW: MIN_W }); };
      // Anchored on the circle itself: the group's box is the circle and whatever else a group ever holds.
      const mark = node.querySelector('.wm-mark');
      shown.push({ xy: g.xy, r, tip, mark });
      node.addEventListener('focus', () => showTip(mark.getBoundingClientRect(), tip()));
      node.addEventListener('blur', hideTip);
      node.addEventListener('click', (e) => { e.stopPropagation(); pick(); });
      node.addEventListener('keydown', (e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); e.stopPropagation(); pick(); } });
      return node;
    }));
    if (route) {
      const [a, b] = route, mx = (a[0] + b[0]) / 2, my = (a[1] + b[1]) / 2 - Math.hypot(b[0] - a[0], b[1] - a[1]) * 0.18;
      routeLayer.replaceChildren(s('path', { class: 'wm-route', d: `M${a[0]},${a[1]}Q${mx},${my} ${b[0]},${b[1]}` }),
        s('circle', { class: 'wm-route-end', cx: a[0], cy: a[1], r: 4 * k }), s('circle', { class: 'wm-route-end', cx: b[0], cy: b[1], r: 4 * k }));
    } else routeLayer.replaceChildren();
  }

  // ---- input
  const toMap = (e) => { const r = frame.getBoundingClientRect(); return [view.x + (e.clientX - r.left) / r.width * view.w, view.y + (e.clientY - r.top) / r.height * view.h]; };
  let drag = null;
  svg.addEventListener('pointerdown', (e) => {
    if (!world || e.button !== 0 || e.target.closest('.wm-dot')) return;
    drag = { x: e.clientX, y: e.clientY, view: { ...view }, moved: false };
    svg.setPointerCapture(e.pointerId);
    hideTip();
  });
  // One tooltip for everything under the pointer, worked out from where the dots are rather than from which of two
  // overlapping circles the browser says was entered: the same spot always says the same thing, and a dot under
  // another one (home, with somebody playing there) is never out of reach. Top first, anchored on the top one.
  function hover(e) {
    if (!world || drag || !shown.length) return;
    const m = toMap(e), slack = 2 * unitsPerPx();
    const under = shown.filter((d) => Math.hypot(d.xy[0] - m[0], d.xy[1] - m[1]) <= d.r + slack).reverse();
    const key = under.map((d) => shown.indexOf(d)).join(',');
    // Nothing new under the pointer — unless something else (a scroll hides every tooltip) took ours away meanwhile.
    if (key === hovering && (!under.length || tipShown())) return;
    hovering = key;
    if (!under.length) return hideTip();
    showTip(under[0].mark.getBoundingClientRect(), under.length === 1 ? under[0].tip() : h('div', { class: 'wm-tips' }, under.map((d) => d.tip())));
  }
  svg.addEventListener('pointerleave', () => { if (hovering !== null) { hovering = null; hideTip(); } });
  svg.addEventListener('pointermove', (e) => {
    if (!drag) return hover(e);
    const k = unitsPerPx(), dx = e.clientX - drag.x, dy = e.clientY - drag.y;
    if (Math.abs(dx) + Math.abs(dy) > 3) { drag.moved = true; svg.classList.add('is-dragging'); }
    if (drag.moved) setView({ ...drag.view, x: drag.view.x - dx * k, y: drag.view.y - dy * k });
  });
  // Let go: draw the map where it was left.
  const endDrag = () => { const moved = drag && drag.moved; drag = null; svg.classList.remove('is-dragging'); if (moved) draw(); };
  svg.addEventListener('pointerup', endDrag);
  svg.addEventListener('pointercancel', endDrag);
  svg.addEventListener('dblclick', (e) => { if (world) zoomBy(0.5, toMap(e)); });
  // The wheel belongs to the page unless Ctrl says otherwise: a map that eats scrolling traps people on it.
  frame.addEventListener('wheel', (e) => {
    if (!world) return;
    if (!e.ctrlKey && !e.metaKey) {
      hint.hidden = false;
      clearTimeout(hintTimer);
      hintTimer = setTimeout(() => { hint.hidden = true; }, 1400);
      return;
    }
    e.preventDefault();
    zoomBy(e.deltaY > 0 ? 1.25 : 0.8, toMap(e));
  }, { passive: false });
  svg.addEventListener('keydown', (e) => {
    if (!world || e.target !== svg) return;
    const step = view.w * 0.15;
    const moves = { ArrowLeft: [-step, 0], ArrowRight: [step, 0], ArrowUp: [0, -step], ArrowDown: [0, step] };
    if (moves[e.key]) { e.preventDefault(); setView({ ...view, x: view.x + moves[e.key][0], y: view.y + moves[e.key][1] }); }
    else if (e.key === '+' || e.key === '=') { e.preventDefault(); zoomBy(0.7); }
    else if (e.key === '-' || e.key === '_') { e.preventDefault(); zoomBy(1.4); }
    else if (e.key === '0') { e.preventDefault(); fitAll(); }
  });
  zoomIn.addEventListener('click', () => world && zoomBy(0.6));
  zoomOut.addEventListener('click', () => world && zoomBy(1.6));
  fitBtn.addEventListener('click', () => { if (world) { route = null; fitAll(); } });

  const ro = new ResizeObserver(() => { measure(); if (world && view) setView(view, { settle: true }); });
  ro.observe(frame);
  const destroy = () => { ro.disconnect(); clearTimeout(settleTimer); clearTimeout(hintTimer); hideTip(); };
  if (signal) signal.addEventListener('abort', destroy, { once: true });

  loadWorld().then((w) => {
    if (signal && signal.aborted) return;
    world = w;
    project = projector(w);
    land.replaceChildren(...w.countries.map((c) => s('path', { class: 'wm-land' + (c.c && hot.has(c.c) ? ' is-hot' : ''), d: c.d }, s('title', null, c.n))));
    placed = points.filter((p) => Number.isFinite(p.latitude) && Number.isFinite(p.longitude)).map((p) => ({ ...p, xy: project(p.longitude, p.latitude) }));
    status.hidden = true;
    measure();
    fitAll();
  }).catch((e) => { status.textContent = e.message || 'The map could not be loaded'; });

  return {
    el,
    /** Bring these coordinates into view: [{latitude, longitude}]. With two, draw the line between them. */
    show(coords, { line = false } = {}) {
      if (!world) return;
      const pts = coords.filter((c) => Number.isFinite(c.latitude) && Number.isFinite(c.longitude)).map((c) => ({ xy: project(c.longitude, c.latitude) }));
      route = line && pts.length === 2 ? [pts[0].xy, pts[1].xy] : null;
      fit(pts, { pad: 1.7, minW: pts.length > 1 ? 160 : 260 });
      frame.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
    },
    destroy,
  };
}

export const placeTip = (title, rows) => h('div', null, h('div', { class: 'fui-tooltip__title' }, title),
  rows.filter(Boolean).map(([value, label]) => h('div', { class: 'fui-tooltip__row' }, h('strong', null, typeof value === 'number' ? num(value) : value), h('span', null, label))));
