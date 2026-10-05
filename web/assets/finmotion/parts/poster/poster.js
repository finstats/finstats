// FinMotion: poster. A FinUI poster marked fm-develop develops like a photograph as its picture arrives, from soft and
// colourless (not one already in the cache: it was there before anybody could watch). One marked fm-light tips towards the
// pointer, and a sheen crosses it where the light would fall — for one large poster, not a grid of them.

import { h } from '../../core/dom.js';
import { play, follower } from '../../core/motion.js';
import { lean } from './plan.js';

function develops(box) {
  const img = box.querySelector('img');
  if (!img || img.complete) return;
  img.addEventListener('load', () => play(img, [{ filter: 'blur(8px) grayscale(1) sepia(.6) brightness(1.15)', opacity: 0.6 }, { filter: 'none', opacity: 1 }], 'glide', { fill: 'none' }), { once: true });
}

function catchesLight(box) {
  const sheen = h('span', { class: 'fm-poster__sheen' });
  const tilt = follower({ ry: 0, rx: 0 }, ({ ry, rx }) => { box.style.transform = ry || rx ? `perspective(600px) rotateY(${ry}deg) rotateX(${rx}deg)` : ''; });
  const moved = (e) => {
    const r = box.getBoundingClientRect(), l = lean((e.clientX - r.left) / r.width - 0.5, (e.clientY - r.top) / r.height - 0.5);
    if (!sheen.isConnected) box.append(sheen);
    tilt.to({ ry: l.ry, rx: l.rx }, 'snap');
    sheen.style.setProperty('--fm-poster-x', `${l.x}%`); sheen.style.setProperty('--fm-poster-y', `${l.y}%`);
  };
  const left = () => { tilt.to({ ry: 0, rx: 0 }, 'drift'); sheen.remove(); };
  box.addEventListener('pointermove', moved); box.addEventListener('pointerleave', left);
  return () => { box.removeEventListener('pointermove', moved); box.removeEventListener('pointerleave', left); tilt.stop(); };
}

export const part = {
  name: 'poster',
  selector: '.fui-poster.fm-develop, .fui-poster.fm-light',
  enhance(box) {
    if (box.classList.contains('fm-develop')) develops(box);
    if (box.classList.contains('fm-light')) return catchesLight(box);
  },
};
