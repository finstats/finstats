// FinUI: stream-card. One stream as it plays: its poster, and beside it, over its art dimmed, what plays it and how, where
// it plays from, and when it ends; a bar for how far it has got (and how far ahead a conversion is); under it the title,
// a second line, and who is watching.

import { h, icon } from '../../core.js';
import { say } from '../../i18n.js';

const share = (x) => `${Math.max(0, Math.min(1, x || 0)) * 100}%`;

/** streamCard({ key, poster, art, tile, facts, eta, time, progress, buffer, title, sub, who, paused }) → the card.
 *  poster: a node. art: an image address drawn dimmed behind the details, or null. tile: a node in the corner (an app's
 *  tile). facts: groups of [label, value] pairs, [[['Product', 'Web'], …], …], a gap between groups. eta: a line ("ETA:
 *  21:40"). time: the position as text, in `.fui-stream-card__time` for a clock to move. progress and buffer: 0 to 1, the
 *  bar's fill in `.fui-stream-card__fill`. title: { text, href }; sub: { text, icon }; who: a node. key: `data-np-key`. */
export function streamCard({ key = null, poster = null, art = null, tile = null, facts = [], eta = null, time = '', progress = 0, buffer = null, title = {}, sub = null, who = null, paused = false }) {
  const fill = h('span', { class: 'fui-stream-card__fill' });
  fill.style.width = share(progress);
  const bar = h('div', { class: 'fui-stream-card__bar', role: 'progressbar', 'aria-label': say('stream_card.progress', 'How far it has got'), 'aria-valuemin': '0', 'aria-valuemax': '100', 'aria-valuenow': String(Math.round((progress || 0) * 100)) },
    buffer != null ? h('span', { class: 'fui-stream-card__buffer' }) : null, fill);
  if (buffer != null) bar.firstChild.style.width = share(buffer);
  return h('article', { class: 'fui-stream-card', dataset: key ? { npKey: key } : {} },
    h('div', { class: 'fui-stream-card__panel' },
      art ? h('img', { class: 'fui-stream-card__art', src: art, alt: '', loading: 'lazy', decoding: 'async', onError: (e) => e.target.remove() }) : null,
      poster ? h('div', { class: 'fui-stream-card__poster' }, poster) : null,
      h('div', { class: 'fui-stream-card__details' },
        facts.map((group) => h('dl', { class: 'fui-stream-card__facts' }, group.filter(Boolean).map(([k, v]) => [h('dt', null, k), h('dd', { title: typeof v === 'string' ? v : null }, v)])))),
      tile ? h('div', { class: 'fui-stream-card__tile' }, tile) : null,
      h('div', { class: 'fui-stream-card__when' }, eta ? h('span', null, eta) : null, h('span', { class: 'fui-stream-card__time' }, time))),
    bar,
    h('div', { class: 'fui-stream-card__foot' },
      h('div', { class: 'fui-stream-card__title' }, icon(paused ? 'pause' : 'play', 13), title.href ? h('a', { href: title.href }, title.text) : h('span', null, title.text)),
      sub ? h('div', { class: 'fui-stream-card__sub' }, sub.icon ? icon(sub.icon, 13) : null, h('span', null, sub.text)) : null,
      who ? h('div', { class: 'fui-stream-card__who' }, who) : null));
}

/** streamCards(cards): a grid of them. */
export const streamCards = (cards) => h('div', { class: 'fui-stream-card__grid' }, cards);

export const meta = {
  name: 'stream-card',
  purpose: 'Shows one stream as it plays: what and how it plays, where from, how far it has got and who is watching.',
  use: 'What is playing now on a server, a card per stream, each one’s clock moved by its page.',
  avoid: 'A play that has ended (a row of history). A title that is not playing (media-card).',
  variants: ['with art behind it', 'a conversion ahead of the position', 'paused'],
  states: ['playing (a play sign)', 'paused (a pause sign)'],
  a11y: 'An article; the details are a description list, the bar a progressbar with its value.',
  props: { 'streamCard({ key, poster, art, tile, facts, eta, time, progress, buffer, title, sub, who, paused })': 'facts: [[[label, value], …], …]', 'streamCards(cards)': 'a grid' },
  playground: {
    controls: [{ key: 'paused', label: 'Paused' }, { key: 'buffer', label: 'Converting ahead', on: true }],
    render: (o) => streamCards([streamCard({ poster: h('span', { class: 'fui-stream-card__demo-poster', 'aria-hidden': 'true' }), paused: o.paused,
      facts: [[['Product', 'Web browser'], ['Player', 'Firefox'], ['Quality', 'Original (8.8 Mbps)']], [['Stream', 'Transcode'], ['Video', 'HEVC → H264 1080p'], ['Audio', 'EAC3 5.1 → AAC Stereo']], [['Location', 'LAN'], ['Bandwidth', '8.8 Mbps']]],
      eta: 'ETA: 21:40', time: '45:58 / 1:45:00', progress: 0.44, buffer: o.buffer ? 0.6 : null, title: { text: 'Big Buck Bunny', href: '#' }, sub: { text: '2008', icon: 'film' }, who: 'alice' })]),
  },
};
