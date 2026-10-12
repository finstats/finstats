// A published profile, for a reader without an account (2.0). Deliberately outside the app: no
// state, no api.js, no router, no prefetch. Nothing here knows about signing in, so nothing here can
// send a stranger to /login or ask for more than the one answer. The server has already filled in the
// link preview; this draws the sections the owner published, with the same charts the app uses.

import { loadLanguage, t, tParts, lang } from '../i18n.js';

// The words first: dom.js and the charts write in the language in use from the moment they are evaluated.
await loadLanguage();
const { h, mount, logo, num, duration, initials, parseDay } = await import('../dom.js');
const { heatmap } = await import('../charts.js');
const { button } = await import('../../finui/components/button/button.js');

const token = location.pathname.split('/')[2] || '';
const root = document.getElementById('public');
const base = `/api/public/${encodeURIComponent(token)}`;
const dayF = new Intl.DateTimeFormat(lang(), { day: 'numeric', month: 'short', year: 'numeric' });

function poster(x, w = 160) {
  const box = h('span', { class: 'fui-poster', 'aria-hidden': 'true' });
  const fallback = () => mount(box, h('span', { class: 'fui-poster__fallback' }, initials(x.name)));
  if (!x.image) fallback();
  else box.append(h('img', { src: `${base}/img/${x.image}?w=${w}`, alt: '', loading: 'lazy', decoding: 'async', onError: fallback }));
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
    h('ol', { class: 'pub-shelf' }, list.map((x) => h('li', { class: 'pub-title' },
      poster(x),
      h('span', { class: 'pub-title-name' }, x.name),
      h('span', { class: 'pub-title-sub muted' }, [x.sub, duration(x.watch_s)].filter(Boolean).join(' · '))))));
}

function totals(x) {
  return section(t('public_profile.what_they_watch'), 'pub-totals',
    h('div', { class: 'fui-stat-tile__grid' },
      tile(t('public_profile.hours_watched'), hours(x.watch_s)), tile(t('public_profile.plays'), num(x.plays)),
      tile(t('public_profile.films'), num(x.movies)), tile(t('public_profile.episodes'), num(x.episodes)), x.tracks ? tile(t('public_profile.songs'), num(x.tracks)) : null),
    shelf(t('public_profile.chapter.shows'), x.top_series), shelf(t('public_profile.chapter.films'), x.top_movies), shelf(t('public_profile.chapter.music'), x.top_tracks));
}

function tile(label, value) {
  return h('div', { class: 'fui-stat-tile' }, h('span', { class: 'fui-stat-tile__label' }, label), h('span', { class: 'fui-stat-tile__value' }, value));
}

function habits(x) {
  const days = (n) => t('public_profile.days', { count: Number(n) || 0 });
  return section(t('public_profile.when_they_watch'), 'pub-habits',
    h('div', { class: 'fui-stat-tile__grid' }, tile(t('public_profile.longest_streak'), days(x.longest_streak_days)), tile(t('public_profile.days_with_something_on'), num(x.active_days))),
    heatmap({ data: x.heatmap }),
    x.genres.length ? h('div', { class: 'pub-genres' }, h('h3', { class: 'pub-sub' }, t('public_profile.chapter.genres')),
      h('ul', { class: 'fui-chip__set' }, x.genres.map((g) => h('li', { class: 'fui-chip' }, g.name, h('span', { class: 'muted' }, duration(g.watch_s)))))) : null);
}

// What each card of the story is about: its alternative text, and the caption under it.
const CHAPTER = Object.fromEntries(['year', 'numbers', 'shows', 'films', 'music', 'genres', 'persona', 'rhythm', 'days', 'records', 'together',
  'finished', 'asked', 'versus'].map((k) => [k, t(`public_profile.chapter.${k}`)]));

function recap(r, story) {
  const base = `/u/${encodeURIComponent(token)}`;
  const slug = r.label.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '');
  return section(t('public_profile.recap_title', { label: r.label }), 'pub-recap',
    h('p', { class: 'pub-year-line' }, ...tParts(r.records.longest_streak_days ? 'public_profile.year_line_streak' : 'public_profile.year_line', {
      hours: h('strong', null, hours(r.totals.watch_s)), plays: t('terms.play', { count: r.totals.plays }), days: t('public_profile.days', { count: r.totals.active_days }),
      streak: t('public_profile.streak', { count: r.records.longest_streak_days || 0 }) })),
    r.persona ? h('p', { class: 'pub-persona' }, h('strong', null, r.persona.title), ' ', h('span', { class: 'muted' }, r.persona.line)) : null,
    (story || []).length ? h('ol', { class: 'pub-story' }, story.map((key) => h('li', null,
      h('a', { href: `${base}/recap/${key}`, target: '_blank', rel: 'noopener', 'aria-label': t('public_profile.open_card', { chapter: CHAPTER[key] || key }) },
        h('img', { src: `${base}/recap/${key}`, alt: t('public_profile.card_alt', { chapter: CHAPTER[key] || key }), loading: 'lazy', decoding: 'async', width: 1080, height: 1920 })),
      h('span', { class: 'pub-title-sub muted' }, CHAPTER[key] || key, ' · ', h('a', { href: `${base}/recap/${key}`, download: `finstats-${slug}-${key}.png` }, t('public_profile.download')))))) : null,
    h('div', { class: 'pub-actions' },
      (story || []).length ? button({ href: `${base}/recap.zip`, download: `finstats-${slug}.zip` }, t('public_profile.download_all')) : null,
      button({ class: 'pub-card-link', href: `${base}/card.png?kind=recap`, download: `finstats-${slug}.png` }, t('public_profile.download_wide'))));
}

function recent(list) {
  if (!list.length) return null;
  return section(t('public_profile.lately'), 'pub-recent',
    h('ol', { class: 'pub-recent-list' }, list.map((p) => h('li', null,
      poster(p, 64),
      h('span', { class: 'pub-recent-text' }, h('span', { class: 'pub-title-name' }, p.name), p.sub ? h('span', { class: 'muted' }, p.sub) : null),
      h('time', { class: 'muted mono', datetime: p.day }, dayF.format(parseDay(p.day)))))));
}

function page(a) {
  const name = a.name || t('public_profile.default_name');
  document.title = a.name ? t('public_profile.doc_title', { name: a.name }) : t('public_profile.default_name');
  const profileCard = a.totals || a.habits;
  return [
    h('header', { class: 'pub-hero' },
      a.avatar ? h('span', { class: 'fui-avatar pub-avatar', 'aria-hidden': 'true' }, h('img', { src: `${base}/avatar`, alt: '', onError: (e) => e.target.replaceWith(initials(name)) })) : null,
      h('div', { class: 'pub-hero-text' }, h('h1', { class: 'fui-page-header__title' }, name),
        h('p', { class: 'muted' }, t('public_profile.hero'))),
      profileCard ? button({ class: 'pub-card-link', href: `/u/${encodeURIComponent(token)}/card.png`, download: 'finstats-profile.png' }, t('public_profile.download_card')) : null),
    a.totals ? totals(a.totals) : null,
    a.habits ? habits(a.habits) : null,
    a.recap ? recap(a.recap, a.story) : null,
    a.recent ? recent(a.recent) : null,
    h('footer', { class: 'pub-foot muted' }, logo(18), h('span', null, t('public_profile.made_with'))),
  ].filter(Boolean);
}

async function load() {
  try {
    const r = await fetch(base, { headers: { Accept: 'application/json' }, credentials: 'omit' });
    if (!r.ok) throw new Error(String(r.status));
    mount(root, ...page(await r.json()));
  } catch {
    document.title = t('not_found.page_title');
    mount(root, h('div', { class: 'pub-gone' }, h('h1', { class: 'fui-page-header__title' }, t('public_profile.gone_title')),
      h('p', { class: 'muted' }, t('public_profile.gone_body'))));
  }
}

load();
