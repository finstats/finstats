// The wiring board for a Tautulli import: Plex users on the left, Jellyfin users on the right, and a wire from each Plex
// user to whoever they are now, like Among Us's Fix Wiring, for deciding whose history is whose. Drag a plug onto a socket,
// or pick a plug (a click, Enter) and then a socket; the arrow keys walk the sockets. A wire pulled out into nothing
// comes unplugged, and so does Delete on a plug. Several plugs may go into one socket, one plug into one socket only,
// and a Plex user left without a wire is not imported.
import { h, s, num, mount, icon } from '../dom.js';
import { avatar } from '../components.js';
import { button } from '../../finui/components/button/button.js';

const COLOURS = 8; // --wire-0 … --wire-7, the panel's wire colours

// The wires live outside the board, so a board repainted by a poll, or opened again after a look at another page,
// keeps what was drawn on it. `key` says which upload they belong to.
const kept = { key: null, wires: new Map() };

/** Year range of a Plex user's plays, "2024 – 2025", or nothing. */
function span(u) {
  if (!u.first_at) return null;
  const [a, b] = [new Date(u.first_at * 1000).getFullYear(), new Date((u.last_at || u.first_at) * 1000).getFullYear()];
  return a === b ? String(a) : `${a} – ${b}`;
}

/**
 * The board for `board` (`{plex_users, jellyfin_users}` from `/import/tautulli`). `onImport(wires)` is called with
 * `[{plex_user_id, jellyfin_user_id}]` and answers a promise; `onReset()` puts the board away.
 */
export function wiringBoard(board, { onImport, onReset }) {
  const key = JSON.stringify(board.plex_users.map((u) => u.id));
  if (kept.key !== key) Object.assign(kept, { key, wires: new Map() });
  const wires = kept.wires;
  const jfName = new Map(board.jellyfin_users.map((u) => [u.id, u.name]));
  for (const [plex, jf] of wires) if (!jfName.has(jf)) wires.delete(plex);

  const plugs = new Map(), sockets = new Map(), colour = new Map();
  let picking = null, drag = null, frame = 0, swallowClick = false;
  const said = h('p', { class: 'sr-only', 'aria-live': 'polite' });
  const layer = s('svg', { class: 'wire-layer', 'aria-hidden': 'true' });
  const plexById = new Map(board.plex_users.map((u) => [u.id, u]));

  const plexCol = h('ul', { class: 'wire-col', 'aria-label': 'Plex users, from Tautulli' }, board.plex_users.map((u, i) => {
    colour.set(u.id, `var(--wire-${i % COLOURS})`);
    const plug = h('button', { type: 'button', class: 'wire-plug', 'data-plex': String(u.id), 'aria-pressed': 'false' }, h('span', { class: 'wire-nub' }));
    plug.style.setProperty('--wire', colour.get(u.id));
    plugs.set(u.id, plug);
    plug.addEventListener('pointerdown', (e) => grab(e, u.id));
    plug.addEventListener('click', () => { if (swallowClick) { swallowClick = false; return; } pick(u.id); });
    plug.addEventListener('keydown', (e) => {
      if ((e.key === 'Delete' || e.key === 'Backspace') && wires.has(u.id)) { e.preventDefault(); unplug(u.id); }
      else if (e.key === 'Escape' && picking != null) { e.preventDefault(); e.stopPropagation(); stopPicking(true); }
    });
    return h('li', { class: ['wire-row', !u.plays && 'is-empty'] },
      h('span', { class: 'wire-who' }, h('span', { class: 'wire-name' }, u.name),
        h('span', { class: 'wire-meta' }, u.plays ? [`${num(u.plays)} play${u.plays === 1 ? '' : 's'}`, span(u) ? ` · ${span(u)}` : ''] : 'No films or episodes')),
      plug);
  }));

  const jfCol = h('ul', { class: 'wire-col', 'aria-label': 'Jellyfin users' }, board.jellyfin_users.map((u) => {
    const socket = h('button', { type: 'button', class: 'wire-socket', 'data-jf': u.id }, h('span', { class: 'wire-hole' }));
    sockets.set(u.id, socket);
    socket.addEventListener('click', () => { if (picking != null) connect(picking, u.id, true); });
    socket.addEventListener('keydown', (e) => {
      if (picking == null) return;
      if (e.key === 'ArrowDown' || e.key === 'ArrowUp') { e.preventDefault(); step(u.id, e.key === 'ArrowDown' ? 1 : -1); }
      else if (e.key === 'Escape') { e.preventDefault(); e.stopPropagation(); stopPicking(true); }
    });
    return h('li', { class: 'wire-row' }, socket, avatar(u.id, u.name, { size: 24, hasImage: u.has_image }), h('span', { class: 'wire-name' }, u.name));
  }));

  const el = h('div', { class: 'wire-board', role: 'group', 'aria-label': 'Connect Plex users to Jellyfin users' },
    h('div', { class: 'wire-head' }, h('span', null, icon('play', 13), 'Plex'), h('span', null, 'Jellyfin')),
    h('div', { class: 'wire-panel' }, plexCol, layer, jfCol));
  const summary = h('p', { class: 'wire-summary' });
  const failed = h('p', { class: 'wire-error', role: 'alert' });
  const importBtn = button({ variant: 'primary', class: 'wire-import', type: 'button', onClick: start });
  const resetBtn = button({ class: 'wire-reset', type: 'button', onClick: () => onReset() }, 'Start over');
  const root = h('div', { class: 'wire-wrap' },
    h('p', { class: 'fui-field__help' }, 'Drag a wire from each Plex user to who they are on Jellyfin, or click one, then the other. Several Plex users may go to one person. A Plex user without a wire is not imported.'),
    el, said, summary, failed, h('div', { class: 'wire-actions' }, resetBtn, importBtn));

  // ---- wiring
  function connect(plex, jf, fromKeys) {
    wires.set(plex, jf);
    const socket = sockets.get(jf);
    socket.classList.remove('is-snapped'); void socket.offsetWidth; socket.classList.add('is-snapped');
    said.textContent = `${plexById.get(plex).name} connected to ${jfName.get(jf)}.`;
    stopPicking(false);
    if (fromKeys) plugs.get(plex).focus();
    update();
  }
  function unplug(plex) {
    if (!wires.delete(plex)) return;
    said.textContent = `${plexById.get(plex).name} unplugged.`;
    update();
  }
  function pick(plex) {
    if (picking === plex) { stopPicking(true); return; }
    picking = plex;
    for (const [id, p] of plugs) p.classList.toggle('is-picking', id === plex);
    el.classList.add('is-picking');
    said.textContent = `Choose who ${plexById.get(plex).name} is on Jellyfin. Escape cancels.`;
    update();
    (sockets.get(wires.get(plex)) || sockets.values().next().value)?.focus();
  }
  function stopPicking(refocus) {
    const was = picking;
    picking = null;
    for (const p of plugs.values()) p.classList.remove('is-picking');
    el.classList.remove('is-picking');
    if (refocus && was != null) plugs.get(was).focus();
    update();
  }
  function step(from, by) {
    const ids = [...sockets.keys()];
    sockets.get(ids[(ids.indexOf(from) + by + ids.length) % ids.length]).focus();
  }

  // ---- dragging: the wire follows the pointer, and lands in the socket under it, or in nothing, and comes out
  function grab(e, plex) {
    if (e.button !== 0) return;
    plugs.get(plex).setPointerCapture(e.pointerId);
    drag = { plex, x0: e.clientX, y0: e.clientY, x: e.clientX, y: e.clientY, moved: false };
    const move = (ev) => {
      drag.x = ev.clientX; drag.y = ev.clientY;
      if (!drag.moved && Math.hypot(drag.x - drag.x0, drag.y - drag.y0) > 4) { drag.moved = true; if (picking != null) stopPicking(false); }
      if (drag.moved && !frame) frame = requestAnimationFrame(() => { frame = 0; draw(); });
    };
    const up = (ev) => {
      const plug = plugs.get(plex);
      plug.removeEventListener('pointermove', move); plug.removeEventListener('pointerup', up); plug.removeEventListener('pointercancel', up);
      const moved = drag && drag.moved;
      drag = null;
      if (!moved) { draw(); return; }
      swallowClick = true; // the click that follows a drag is not a pick
      setTimeout(() => { swallowClick = false; }, 0);
      const target = ev.type === 'pointerup' ? document.elementFromPoint(ev.clientX, ev.clientY)?.closest('.wire-socket') : null;
      if (target && el.contains(target)) connect(plex, target.dataset.jf, false);
      else unplug(plex);
      draw();
    };
    plugs.get(plex).addEventListener('pointermove', move);
    plugs.get(plex).addEventListener('pointerup', up);
    plugs.get(plex).addEventListener('pointercancel', up);
  }

  // ---- drawing: a curve from each plug to its socket, over a darker edge so it reads as a cable
  const centre = (node, at) => {
    const r = node.getBoundingClientRect();
    return [r.left + r.width / 2 - at.left, r.top + r.height / 2 - at.top];
  };
  function cable(plex, [x1, y1], [x2, y2], live) {
    const dx = Math.max(30, Math.abs(x2 - x1) * 0.45);
    const d = `M ${x1} ${y1} C ${x1 + dx} ${y1}, ${x2 - dx} ${y2}, ${x2} ${y2}`;
    const edge = s('path', { class: 'wire-edge', d });
    const wire = s('path', { class: ['wire', live && 'is-live'], d });
    wire.style.stroke = colour.get(plex);
    return [edge, wire];
  }
  function draw() {
    const at = layer.getBoundingClientRect();
    layer.setAttribute('width', at.width); layer.setAttribute('height', at.height);
    const paths = [];
    for (const [plex, jf] of wires) {
      if (drag && drag.moved && drag.plex === plex) continue;
      paths.push(...cable(plex, centre(plugs.get(plex).firstChild, at), centre(sockets.get(jf).firstChild, at), false));
    }
    if (drag && drag.moved) paths.push(...cable(drag.plex, centre(plugs.get(drag.plex).firstChild, at), [drag.x - at.left, drag.y - at.top], true));
    mount(layer, paths);
  }

  // ---- what the board says
  function update() {
    const into = new Map();
    for (const [plex, jf] of wires) into.set(jf, (into.get(jf) || 0) + 1);
    for (const [id, plug] of plugs) {
      const u = plexById.get(id), to = wires.get(id);
      plug.setAttribute('aria-pressed', String(!!to));
      plug.classList.toggle('is-wired', !!to);
      plug.setAttribute('aria-label', `${u.name} on Plex, ${u.plays} play${u.plays === 1 ? '' : 's'}: ${to ? `connected to ${jfName.get(to)}` : 'not connected'}`);
    }
    for (const [id, socket] of sockets) {
      const n = into.get(id) || 0;
      socket.classList.toggle('is-wired', n > 0);
      socket.setAttribute('aria-label', picking != null ? `Connect ${plexById.get(picking).name} to ${jfName.get(id)}`
        : `${jfName.get(id)} on Jellyfin${n ? `, ${n} Plex user${n === 1 ? '' : 's'} connected` : ''}`);
      // The colour of the wire in it, or the first of them: a socket shows what is plugged in.
      const first = [...wires].find(([, jf]) => jf === id);
      if (first) socket.style.setProperty('--wire', colour.get(first[0])); else socket.style.removeProperty('--wire');
    }
    const plays = [...wires.keys()].reduce((sum, id) => sum + (plexById.get(id).plays || 0), 0);
    const total = board.plex_users.length;
    summary.textContent = `${wires.size} of ${total} Plex users connected · ${num(plays)} play${plays === 1 ? '' : 's'} will come in`
      + (wires.size && wires.size < total ? ' · the rest stay behind' : '');
    importBtn.disabled = !wires.size;
    importBtn.textContent = wires.size ? `Import ${num(plays)} play${plays === 1 ? '' : 's'}` : 'Import';
    draw();
  }

  async function start() {
    failed.textContent = '';
    importBtn.disabled = true;
    try {
      await onImport([...wires].map(([plex, jf]) => ({ plex_user_id: plex, jellyfin_user_id: jf })));
      kept.key = null; // imported: a new upload starts a new board
    } catch (e) {
      failed.textContent = e.message;
      importBtn.disabled = !wires.size;
    }
  }

  // Picking ends with a click anywhere off the board, and the wires follow the board when it changes size.
  const away = (e) => { if (picking != null && !el.contains(e.target)) stopPicking(false); };
  document.addEventListener('pointerdown', away);
  const watch = new ResizeObserver(() => draw());
  watch.observe(el);
  update();
  return { el: root, destroy: () => { document.removeEventListener('pointerdown', away); watch.disconnect(); cancelAnimationFrame(frame); } };
}
