// A published profile, for a reader without an account (2.0). Deliberately outside the app: no
// state, no api.js, no router, no prefetch. Nothing here knows about signing in, so nothing here can
// send a stranger to /login or ask for more than the one answer. The server has already filled in the
// link preview; this draws the sections the owner published, with the same charts the app uses.

import { h, mount, logo, num, duration, initials, parseDay } from '../dom.js';
import { heatmap } from '../charts.js';
import { button } from '../../finui/components/button/button.js';

const token = location.pathname.split('/')[2] || '';
const root = document.getElementById('public');
const base = `/api/public/${encodeURIComponent(token)}`;
const dayF = new Intl.DateTimeFormat(undefined, { day: 'numeric', month: 'short', year: 'numeric' });

function poster(t, w = 160) {
  const box = h('span', { class: 'fui-poster', 'aria-hidden': 'true' });
  const fallback = () => mount(box, h('span', { class: 'fui-poster__fallback' }, initials(t.name)));
  if (!t.image) fallback();
  else box.append(h('img', { src: `${base}/img/${t.image}?w=${w}`, alt: '', loading: 'lazy', decoding: 'async', onError: fallback }));
  return box;
}

const hours = (sec) => num((Number(sec) || 0) / 3600);

function section(title, cls, ...body) {
  return h('section', { class: `fui-card pub-section ${cls}` },
    h('div', { class: 'fui-card__head' }, h('h2', { class: 'fui-card__title' }, title)),
    h('div', { class: 'fui-card__body' }, ...body));
}

function shelf(title, list) {
  if (!list || !list.length) return null;
  return h('div', { class: 'pub-shelf-group' },
    h('h3', { class: 'pub-sub' }, title),
    h('ol', { class: 'pub-shelf' }, list.map((t) => h('li', { class: 'pub-title' },
      poster(t),
      h('span', { class: 'pub-title-name' }, t.name),
      h('span', { class: 'pub-title-sub muted' }, [t.sub, duration(t.watch_s)].filter(Boolean).join(' · '))))));
}

function totals(t) {
  return section('What they watch', 'pub-totals',
    h('div', { class: 'fui-stat-tile__grid' },
      tile('Hours watched', hours(t.watch_s)), tile('Plays', num(t.plays)),
      tile('Films', num(t.movies)), tile('Episodes', num(t.episodes)), t.tracks ? tile('Songs', num(t.tracks)) : null),
    shelf('Top shows', t.top_series), shelf('Top films', t.top_movies), shelf('Top music', t.top_tracks));
}

function tile(label, value) {
  return h('div', { class: 'fui-stat-tile' }, h('span', { class: 'fui-stat-tile__label' }, label), h('span', { class: 'fui-stat-tile__value' }, value));
}

function habits(x) {
  const days = (n) => `${num(n)} ${Number(n) === 1 ? 'day' : 'days'}`;
  return section('When they watch', 'pub-habits',
    h('div', { class: 'fui-stat-tile__grid' }, tile('Longest streak', days(x.longest_streak_days)), tile('Days with something on', num(x.active_days))),
    heatmap({ data: x.heatmap }),
    x.genres.length ? h('div', { class: 'pub-genres' }, h('h3', { class: 'pub-sub' }, 'Genres'),
      h('ul', { class: 'fui-chip__set' }, x.genres.map((g) => h('li', { class: 'fui-chip' }, g.name, h('span', { class: 'muted' }, duration(g.watch_s)))))) : null);
}

// What each card of the story is about: its alternative text, and the caption under it.
const CHAPTER = {
  year: 'The year', numbers: 'In numbers', shows: 'Top shows', films: 'Top films', music: 'Top music', genres: 'Genres',
  persona: 'The persona', rhythm: 'Hours and days', days: 'Every day', records: 'Records', together: 'In company',
  finished: 'Shows finished', asked: 'Requests', versus: 'Against the year before',
};

function recap(r, story) {
  const base = `/u/${encodeURIComponent(token)}`;
  const slug = r.label.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '');
  return section(`Their ${r.label}`, 'pub-recap',
    h('p', { class: 'pub-year-line' }, h('strong', null, hours(r.totals.watch_s)), ' hours, ', num(r.totals.plays), ' plays on ', num(r.totals.active_days), ' days',
      r.records.longest_streak_days ? `, a ${r.records.longest_streak_days}-day streak` : '', '.'),
    r.persona ? h('p', { class: 'pub-persona' }, h('strong', null, r.persona.title), ' ', h('span', { class: 'muted' }, r.persona.line)) : null,
    (story || []).length ? h('ol', { class: 'pub-story' }, story.map((key) => h('li', null,
      h('a', { href: `${base}/recap/${key}`, target: '_blank', rel: 'noopener', 'aria-label': `Open the card: ${CHAPTER[key] || key}` },
        h('img', { src: `${base}/recap/${key}`, alt: `${CHAPTER[key] || key}, as a card`, loading: 'lazy', decoding: 'async', width: 1080, height: 1920 })),
      h('span', { class: 'pub-title-sub muted' }, CHAPTER[key] || key, ' · ', h('a', { href: `${base}/recap/${key}`, download: `finstats-${slug}-${key}.png` }, 'Download'))))) : null,
    h('div', { class: 'pub-actions' },
      (story || []).length ? button({ href: `${base}/recap.zip`, download: `finstats-${slug}.zip` }, 'Download every card') : null,
      button({ class: 'pub-card-link', href: `${base}/card.png?kind=recap`, download: `finstats-${slug}.png` }, 'Download the wide card')));
}

function recent(list) {
  if (!list.length) return null;
  return section('Lately', 'pub-recent',
    h('ol', { class: 'pub-recent-list' }, list.map((p) => h('li', null,
      poster(p, 64),
      h('span', { class: 'pub-recent-text' }, h('span', { class: 'pub-title-name' }, p.name), p.sub ? h('span', { class: 'muted' }, p.sub) : null),
      h('time', { class: 'muted mono', datetime: p.day }, dayF.format(parseDay(p.day)))))));
}

function page(a) {
  const name = a.name || 'A finstats profile';
  document.title = a.name ? `${a.name} on finstats` : 'A finstats profile';
  const profileCard = a.totals || a.habits;
  return [
    h('header', { class: 'pub-hero' },
      a.avatar ? h('span', { class: 'fui-avatar pub-avatar', 'aria-hidden': 'true' }, h('img', { src: `${base}/avatar`, alt: '', onError: (e) => e.target.replaceWith(initials(name)) })) : null,
      h('div', { class: 'pub-hero-text' }, h('h1', { class: 'fui-page-header__title' }, name),
        h('p', { class: 'muted' }, 'What they watch, published by them. Everything here is at least a day old.')),
      profileCard ? button({ class: 'pub-card-link', href: `/u/${encodeURIComponent(token)}/card.png`, download: 'finstats-profile.png' }, 'Download as a card') : null),
    a.totals ? totals(a.totals) : null,
    a.habits ? habits(a.habits) : null,
    a.recap ? recap(a.recap, a.story) : null,
    a.recent ? recent(a.recent) : null,
    h('footer', { class: 'pub-foot muted' }, logo(18), h('span', null, 'Made with finstats, a statistics server for Jellyfin.')),
  ].filter(Boolean);
}

async function load() {
  try {
    const r = await fetch(base, { headers: { Accept: 'application/json' }, credentials: 'omit' });
    if (!r.ok) throw new Error(String(r.status));
    mount(root, ...page(await r.json()));
  } catch {
    document.title = 'Not found';
    mount(root, h('div', { class: 'pub-gone' }, h('h1', { class: 'fui-page-header__title' }, 'There is no profile here'),
      h('p', { class: 'muted' }, 'The link may have been reset, or its owner stopped publishing.')));
  }
}

load();
