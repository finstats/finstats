// FinMotion: progress. A progress bar marked fm-film runs on film: its track becomes a strip whose frames slide under the
// gate as the share grows — for something played, not something copied.

import { h } from '../../core/dom.js';

export const part = {
  name: 'progress',
  selector: '.fui-progress.fm-film',
  enhance(bar) {
    const track = bar.querySelector('.fui-progress__track');
    if (!track) return;
    const frames = h('span', { class: 'fm-progress__frames', 'aria-hidden': 'true' });
    track.prepend(frames);
    // The share FinUI says (aria-valuenow, 0 to 100) places the strip; it slides on settle as the share grows.
    const at = () => bar.style.setProperty('--fm-progress-at', String((Number(track.getAttribute('aria-valuenow')) || 0) / 100));
    at();
    const watch = new MutationObserver(at);
    watch.observe(track, { attributes: true, attributeFilter: ['aria-valuenow'] });
    return () => { watch.disconnect(); frames.remove(); };
  },
};
