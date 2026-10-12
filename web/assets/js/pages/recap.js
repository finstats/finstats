// Recap: the year in review. The one loud page in an otherwise quiet app. It opens on the year
// "replayed" as a waveform, a bar per week, beside the posters that filled it; then one idea per
// chapter (a headline with a single accented word, a sentence that carries the figures, the
// chapter's word standing faint behind it), and it ends the way films do: with the credits.

import { h, icon, mount, num, duration, durationExact, pct, parseDay } from '../dom.js';
import { api, imgItem } from '../api.js';
import { state, isAdmin, userList } from '../state.js';
import { replaceQuery } from '../router.js';
import { dataView, segmented, poster, emptyState, sk, combobox, openModal, avatar } from '../components.js';
import { showTip, hideTip } from '../charts.js';
import { button } from '../../finui/components/button/button.js';
import { t, tParts, lang } from '../i18n.js';

const BAR = 'var(--single)'; // the single series hue used everywhere else in FinStats
const BAR_PEAK = 'var(--peak)'; // emphasis: the extreme, nothing else

// ---------------------------------------------------------------- formatting
const wdF = new Intl.DateTimeFormat(lang(), { weekday: 'long' });
const wdShortF = new Intl.DateTimeFormat(lang(), { weekday: 'short' });
const mF = new Intl.DateTimeFormat(lang(), { month: 'long' });
const myF = new Intl.DateTimeFormat(lang(), { month: 'long', year: 'numeric' });
const hmF = new Intl.DateTimeFormat(lang(), { hour: '2-digit', minute: '2-digit' });
const mShortF = new Intl.DateTimeFormat(lang(), { month: 'short' });
const longF = new Intl.DateTimeFormat(lang(), { weekday: 'long', day: 'numeric', month: 'long' });
const dmF = new Intl.DateTimeFormat(lang(), { day: 'numeric', month: 'long' });
const monday = (i) => new Date(2024, 0, 1 + i); // 1 January 2024 was a Monday
const WEEKDAYS = [0, 1, 2, 3, 4, 5, 6].map((i) => wdF.format(monday(i)));
const WEEKDAYS_SHORT = [0, 1, 2, 3, 4, 5, 6].map((i) => wdShortF.format(monday(i)));

/** "Saturday 14 March" */
const longDate = (d) => longF.format(d);
const longDay = (str) => longDate(parseDay(str));
const dayMonth = (d) => dmF.format(d);
/** "1–9 February" or "28 January – 3 February" */
function dayRange(from, to) {
  const a = parseDay(from), b = parseDay(to);
  if (from === to) return dayMonth(a);
  return typeof dmF.formatRange === 'function' ? dmF.formatRange(a, b) : `${dayMonth(a)} – ${dayMonth(b)}`;
}
const monthDate = (ym) => { const [y, m] = String(ym).split('-').map(Number); return new Date(y, (m || 1) - 1, 1); };
const hoursOf = (sec) => (Number(sec) || 0) / 3600;
const upTo1 = new Intl.NumberFormat(lang(), { maximumFractionDigits: 1 });
const figure = (x, whole) => (whole ? num(x) : upTo1.format(Math.round(x * 10) / 10));
const hoursText = (sec) => { const x = hoursOf(sec); return figure(x, x >= 10); };
const rules = new Intl.PluralRules(lang());
/** A plural whose number is drawn as `node` (counted up, or formatted its own way); its form is picked by `n`. */
const countParts = (key, n, node, vars) => tParts(`${key}.${rules.select(Number(n) || 0)}`, { ...vars, count: node });
const hoursB = (sec) => b(countParts('pages.recap.hours', hoursOf(sec), hoursText(sec)));
const hour2 = (i) => String(i).padStart(2, '0') + ':00';

// ---------------------------------------------------------------- voice
/** Your own recap speaks to you; an administrator looking at someone else's reads about them by name. */
function voiceFor(scope, me) {
  if (scope && scope.kind === 'server') return { you: false, server: true, who: scope.server_name || t('pages.recap.this_server') };
  if (!scope || !scope.user_id || (me && scope.user_id === me.id)) return { you: true, who: '' };
  return { you: false, who: scope.user_name || t('pages.recap.this_user') };
}
/** A sentence in the voice: `key` has a text for you, for a user (by {name}) and for the server. */
const form = (v) => (v.you ? 'you' : v.server ? 'server' : 'user');
const say = (v, key, vars) => tParts(`${key}.${form(v)}`, { name: v.who, ...vars });
const sayText = (v, key, vars) => t(`${key}.${form(v)}`, { name: v.who, ...vars });

// ---------------------------------------------------------------- motion
const reducedMotion = () => typeof matchMedia === 'function' && matchMedia('(prefers-reduced-motion: reduce)').matches;

/** A number that counts up once, the first time its chapter scrolls into view. */
function countUp(value, format = num) {
  const el = h('span', { class: 'rc-count' }, format(value));
  el._count = { value: Number(value) || 0, format };
  return el;
}
function runCount(el) {
  const c = el._count;
  if (!c || c.done) return;
  c.done = true;
  if (reducedMotion() || c.value <= 0) return;
  const t0 = performance.now(), ms = 900;
  const step = (t) => {
    const k = Math.max(0, Math.min(1, (t - t0) / ms));
    el.textContent = c.format(c.value * (1 - Math.pow(1 - k, 3)));
    if (k < 1 && el.isConnected) requestAnimationFrame(step); else el.textContent = c.format(c.value);
  };
  el.textContent = c.format(0);
  requestAnimationFrame(step);
}

/** Chapters fade up as they arrive; their direct children follow in a short stagger. */
function revealer() {
  const show = (el) => { el.classList.add('is-in'); el.querySelectorAll('.rc-count').forEach(runCount); };
  if (typeof IntersectionObserver !== 'function' || reducedMotion()) return { animated: false, watch: show, stop() {} };
  const io = new IntersectionObserver((entries) => {
    for (const e of entries) if (e.isIntersecting) { show(e.target); io.unobserve(e.target); }
  }, { threshold: 0.12, rootMargin: '0px 0px -6% 0px' });
  return { animated: true, watch: (el) => io.observe(el), stop: () => io.disconnect() };
}

// ---------------------------------------------------------------- small pieces
const em = (...t) => h('em', { class: 'rc-accent' }, ...t);
const b = (...t) => h('strong', null, ...t);
const tail = (...t) => h('i', { class: 'rc-tail' }, ...t);

/** [{value: Node|string, label}] → big figures over small capitals. */
function statRow(stats) {
  const rows = (stats || []).filter(Boolean);
  if (!rows.length) return null;
  return h('dl', { class: 'rc-stats' }, rows.map((x) => h('div', null, h('dd', null, x.value), h('dt', null, x.label))));
}

/**
 * One chapter: icon + eyebrow, headline, a ruled lead sentence, optional figures, then the body.
 * `mark` is the chapter's word, set huge and faint behind everything. Decoration only.
 */
function chapter({ id, icon: ico, eyebrow, title, lead, stats, body, mark, cls = '', label }) {
  const kids = [
    eyebrow ? h('p', { class: 'rc-eyebrow' }, ico ? icon(ico, 15) : null, h('span', null, eyebrow)) : null,
    title ? h('h2', { class: 'rc-title' }, title) : null,
    lead ? h('p', { class: 'rc-lead' }, lead) : null,
    statRow(stats),
    ...(Array.isArray(body) ? body : [body]),
  ].filter(Boolean);
  kids.forEach((k, i) => k.style.setProperty('--i', String(i)));
  return h('section', { class: 'rc-chapter rc-reveal ' + cls, id: id ? 'rc-' + id : null, 'aria-label': label || eyebrow || null },
    mark ? h('span', { class: 'rc-mark', 'aria-hidden': 'true' }, String(mark)) : null, kids);
}

const titleLink = (t, cls = '') => (t.id && t.item_exists !== false
  ? h('a', { class: 'rc-link ' + cls, href: `/items/${t.id}` }, t.name || unknownTitle())
  : h('span', { class: cls }, t.name || unknownTitle()));
const unknownTitle = () => t('pages.recap.unknown_title');

/** The #1 of a list: poster bleeding large beside the type. */
function feature(t, facts) {
  const bleed = t.image_item_id ? h('img', { class: 'rc-bleed', src: imgItem(t.image_item_id, 300), alt: '', 'aria-hidden': 'true', decoding: 'async', onError: (e) => e.target.remove() }) : null;
  return h('div', { class: 'rc-feature' }, bleed,
    poster(t.image_item_id, t.name, { w: 480, cls: 'rc-feature-poster' }),
    h('div', { class: 'rc-feature-text' },
      h('span', { class: 'rc-rank-big mono' }, '#1'),
      h('h3', { class: 'rc-feature-name' }, titleLink(t)),
      t.sub ? h('p', { class: 'rc-feature-sub' }, t.sub) : null,
      h('dl', { class: 'rc-facts' }, facts.filter(Boolean).map(([k, v]) => h('div', null, h('dt', null, k), h('dd', { class: 'mono' }, v))))));
}

function rankedList(rows, start = 2, extra = () => null) {
  if (!rows.length) return null;
  return h('ol', { class: 'rc-ranked', start }, rows.map((t, i) => h('li', { class: 'rc-ranked-row' },
    h('span', { class: 'rc-ranked-n mono' }, String(start + i)),
    poster(t.image_item_id, t.name, { w: 120, cls: 'fui-poster--sm' }),
    h('div', { class: 'rc-ranked-main' }, titleLink(t, 'rc-ranked-name'), h('span', { class: 'rc-ranked-sub' }, [t.sub, extra(t)].filter(Boolean).join(' · ') || ' ')),
    h('span', { class: 'rc-ranked-val mono', title: durationExact(t.watch_s) }, duration(t.watch_s)))));
}

/** Ranked horizontal bars, one hue, value at the end. */
function rankBars(rows, valueOf, format) {
  const max = Math.max(1, ...rows.map(valueOf));
  return h('div', { class: 'rc-rankbars', role: 'list' }, rows.map((r) => h('div', { class: 'rc-rankbar', role: 'listitem' },
    h('span', { class: 'rc-rankbar-name' }, r.name || t('common.unknown')),
    h('span', { class: 'rc-rankbar-track' }, h('span', { class: 'rc-rankbar-fill', style: { width: Math.max(1, (valueOf(r) / max) * 100) + '%' } })),
    h('span', { class: 'rc-rankbar-val mono' }, format(r)))));
}

/**
 * Thin columns from one baseline. One tab stop; ←/→ walk the columns and show the same
 * tooltip as hover. Only the peak carries a direct label; a visually hidden table holds every value.
 * rows: [{label, title, value, foot?: Node}]
 */
function barStrip({ rows, format, ariaLabel, head, cls = '', labelEvery = 1 }) {
  const max = Math.max(0, ...rows.map((r) => Number(r.value) || 0));
  const peak = rows.findIndex((r) => (Number(r.value) || 0) === max);
  let active = -1;
  const cols = rows.map((r, i) => {
    const v = Number(r.value) || 0;
    const bar = h('span', { class: 'rc-bar', style: { height: (max > 0 ? Math.max(v > 0 ? 1.5 : 0, (v / max) * 100) : 0) + '%', background: i === peak && max > 0 ? BAR_PEAK : BAR } });
    return h('div', { class: 'rc-col', onPointerenter: () => setActive(i) },
      h('div', { class: 'rc-col-track' }, i === peak && max > 0 ? h('span', { class: 'rc-col-peak mono' }, format(v)) : null, bar),
      h('span', { class: 'rc-col-label mono' }, i % labelEvery === 0 ? r.label : ' '),
      r.foot || null);
  });
  const wrap = h('div', { class: 'rc-bars ' + cls, tabindex: max > 0 ? 0 : null, role: 'group', 'aria-label': t('pages.recap.bars_aria', { label: ariaLabel }) },
    h('div', { class: 'rc-cols', 'aria-hidden': 'true' }, cols),
    h('table', { class: 'sr-only' }, h('caption', null, ariaLabel),
      h('thead', null, h('tr', null, h('th', null, head[0]), h('th', null, head[1]))),
      h('tbody', null, rows.map((r) => h('tr', null, h('td', null, r.title || r.label), h('td', null, format(Number(r.value) || 0)))))));
  function setActive(i) {
    if (active >= 0 && cols[active]) cols[active].classList.remove('is-active');
    active = i;
    if (i < 0) { hideTip(); return; }
    cols[i].classList.add('is-active');
    const r = rows[i];
    showTip(cols[i].querySelector('.rc-col-track').getBoundingClientRect(), h('div', null,
      h('div', { class: 'fui-tooltip__title' }, r.title || r.label),
      h('div', { class: 'fui-tooltip__row' }, h('span', { class: 'fui-tooltip__key', style: { background: BAR } }), h('strong', null, format(Number(r.value) || 0)), r.note ? h('span', null, r.note) : null)));
  }
  wrap.addEventListener('pointerleave', () => { if (document.activeElement !== wrap) setActive(-1); });
  wrap.addEventListener('keydown', (e) => {
    if (e.key === 'ArrowLeft' || e.key === 'ArrowRight') {
      e.preventDefault();
      setActive(active < 0 ? Math.max(0, peak) : Math.max(0, Math.min(rows.length - 1, active + (e.key === 'ArrowRight' ? 1 : -1))));
    } else if (e.key === 'Escape') setActive(-1);
  });
  wrap.addEventListener('focus', () => { if (active < 0) setActive(Math.max(0, peak)); });
  wrap.addEventListener('blur', () => setActive(-1));
  return wrap;
}

function recordCard({ label, value, context, image, name }) {
  return h('article', { class: 'rc-record' },
    image !== undefined ? poster(image, name, { w: 300, cls: 'rc-record-poster' }) : null,
    h('div', { class: 'rc-record-text' },
      h('p', { class: 'rc-record-label' }, label),
      h('p', { class: 'rc-record-value' }, value),
      context ? h('p', { class: 'rc-record-context' }, context) : null));
}

// ---------------------------------------------------------------- chapters
/** The period picker: the same segmented control as every other range switch in the app. */
function yearTabs(d, current, onPick) {
  let years = (d.years || []).map(String);
  if (/^\d{4}$/.test(current) && !years.includes(current)) years.push(current);
  years = [...new Set(years)].sort();
  return h('div', { class: 'rc-years' }, segmented({ label: t('pages.recap.period'), value: current,
    options: [...years.map((y) => ({ value: y, label: y })), { value: 'last12', label: t('pages.recap.last_12_months') }], onChange: onPick }));
}

/** The period cut into Monday-to-Sunday weeks: [{from, to, sec}]. */
function weeksOf(d) {
  const months = (d.months || []).filter((m) => m && m.month);
  if (!months.length) return [];
  const first = monthDate(months[0].month), lastM = monthDate(months[months.length - 1].month);
  const last = new Date(lastM.getFullYear(), lastM.getMonth() + 1, 0);
  const byDay = new Map((d.days || []).map((x) => [x.date, x.watch_s || 0]));
  const out = [];
  let cur = null;
  for (let dt = new Date(first); dt <= last; dt.setDate(dt.getDate() + 1)) {
    if (!cur || (dt.getDay() + 6) % 7 === 0) { cur = { from: new Date(dt), to: new Date(dt), sec: 0, month: null }; out.push(cur); }
    if (dt.getDate() === 1) cur.month = mShortF.format(dt);
    cur.to = new Date(dt);
    cur.sec += byDay.get(isoDay(dt)) || 0;
  }
  return out;
}

/** The year as sound: one bar a week, mirrored around the middle like a waveform. The loudest week is lit. */
function waveform(d) {
  const weeks = weeksOf(d);
  const max = Math.max(0, ...weeks.map((w) => w.sec));
  if (!weeks.length || max <= 0) return null;
  const peak = weeks.findIndex((w) => w.sec === max);
  const range = (w) => dayRange(isoDay(w.from), isoDay(w.to));
  const bars = weeks.map((w, i) => {
    const bar = h('span', { class: 'rc-wave-bar' + (i === peak ? ' is-peak' : '') + (w.sec > 0 ? '' : ' is-silent'), style: { height: (w.sec > 0 ? Math.max(4, (w.sec / max) * 100) : 0) + '%' } });
    bar.style.setProperty('--i', String(i));
    const slot = h('span', { class: 'rc-wave-slot', onPointerenter: () => showTip(slot.getBoundingClientRect(), h('div', null,
      h('div', { class: 'fui-tooltip__title' }, range(w)),
      h('div', { class: 'fui-tooltip__row' }, h('span', { class: 'fui-tooltip__key', style: { background: i === peak ? BAR_PEAK : BAR } }), w.sec > 0 ? h('strong', null, duration(w.sec)) : h('span', null, t('pages.recap.nothing_played'))))) }, bar);
    return slot;
  });
  const cols = { gridTemplateColumns: `repeat(${weeks.length}, minmax(0, 1fr))` };
  let lastAt = -9;
  const labels = weeks.map((w, i) => { if (!w.month || i - lastAt < 3) return null; lastAt = i; return h('span', { class: 'mono', style: { gridColumn: String(i + 1) } }, w.month); });
  // Decoration with a caption: the calendar further down is the version that can be read day by day.
  return h('figure', { class: 'rc-wave-wrap' },
    h('div', { class: 'rc-wave', style: cols, 'aria-hidden': 'true', onPointerleave: hideTip }, bars),
    h('div', { class: 'rc-wave-months', style: cols, 'aria-hidden': 'true' }, labels),
    h('figcaption', { class: 'rc-note' }, tParts('pages.recap.wave_caption', { week: h('strong', null, range(weeks[peak])), time: duration(max) })));
}

function heroChapter(d, v, periodLabel, picker, share) {
  const tot = d.totals || {};
  const isYear = /^\d{4}$/.test(periodLabel);
  const hrs = hoursOf(tot.watch_s);
  const r = d.rank;
  const fan = [...(d.top_series || []), ...(d.top_movies || [])].filter((x) => x && x.image_item_id).sort((a, c) => (c.watch_s || 0) - (a.watch_s || 0)).slice(0, 5);
  return h('header', { class: 'rc-hero rc-reveal' },
    picker,
    h('div', { class: 'rc-hero-main' },
      h('div', { class: 'rc-hero-text' },
        h('p', { class: 'rc-eyebrow' }, icon('recap', 15), h('span', null, t('pages.recap.eyebrow_year'))),
        h('h2', { class: 'rc-hero-title' }, isYear ? say(v, 'pages.recap.hero_title_year', { year: em(periodLabel) }) : say(v, 'pages.recap.hero_title_last12', { months: em(t('pages.recap.twelve_months')) })),
        h('p', { class: 'rc-hero-line' }, tParts('pages.recap.hero_line', {
          hours: b(countParts('pages.recap.hours', hrs, countUp(hrs, (x) => figure(x, hrs >= 10)))),
          plays: b(t('terms.play', { count: tot.plays || 0 })), titles: b(t('terms.title', { count: tot.distinct_items || 0 })) })),
        // The share button and the rank sit in one row with room between them, and wrap on a narrow screen.
        h('div', { class: 'rc-hero-actions' }, share,
          r && r.position && r.of > 1 ? h('p', { class: 'rc-hero-rank mono' }, t(r.share != null ? 'pages.recap.rank_share' : 'pages.recap.rank', { position: r.position, count: r.of, share: pct(r.share) })) : null)),
      fan.length >= 3 ? h('div', { class: 'rc-fan', 'aria-hidden': 'true' }, fan.map((x) => h('img', { src: imgItem(x.image_item_id, 300), alt: '', decoding: 'async', onError: (e) => e.target.remove() }))) : null),
    waveform(d));
}

/** A square tile per figure. The icon is only a signpost; the number does the talking. */
function tile(ico, value, label, title) {
  return h('div', { class: 'rc-tile', title: title || null }, h('span', { class: 'rc-tile-icon', 'aria-hidden': 'true' }, icon(ico, 18)),
    h('span', { class: 'rc-tile-num' }, value), h('span', { class: 'rc-tile-label' }, label));
}

function numbersChapter(d, v, periodLabel) {
  const tot = d.totals || {};
  const hrs = hoursOf(tot.watch_s), days = hrs / 24;
  const daysRounded = days >= 10 ? Math.round(days) : Math.round(days * 10) / 10;
  const streak = d.records && d.records.longest_streak;
  const rw = d.rewatch;
  const split = [
    { name: t('pages.recap.episodes_label'), watch_s: tot.episode_watch_s || 0, plays: tot.episodes || 0 },
    { name: t('pages.recap.movies_label'), watch_s: tot.movie_watch_s || 0, plays: tot.movies || 0 },
    { name: t('pages.recap.music_label'), watch_s: tot.track_watch_s || 0, plays: tot.tracks || 0 },
  ].filter((x) => x.plays > 0).sort((a, c) => c.watch_s - a.watch_s);
  return chapter({
    id: 'numbers', icon: 'chart', eyebrow: t('pages.recap.numbers_eyebrow'), mark: /^\d{4}$/.test(periodLabel) ? periodLabel : '365',
    title: say(v, 'pages.recap.numbers_title', { numbers: em(t('pages.recap.numbers_word')) }),
    lead: [say(v, 'pages.recap.numbers_lead', { times: b(t('pages.recap.times', { count: tot.plays || 0 })), titles: b(t('pages.recap.different_titles', { count: tot.distinct_items || 0 })) }),
      hrs >= 24 ? tail(t('pages.recap.numbers_days', { count: daysRounded })) : '.'],
    body: [
      h('div', { class: 'rc-tiles' },
        tile('play', countUp(tot.plays), t('pages.recap.plays_label')),
        tile('clock', countUp(hrs, (x) => t('units.hours', { count: figure(x, hrs >= 10) })), t('terms.watch_time'), durationExact(tot.watch_s)),
        tile('layers', countUp(tot.distinct_items), t('pages.recap.tile_titles')),
        tile('calendar', countUp(tot.active_days), t('pages.recap.tile_days')),
        tile('flame', streak && streak.days > 1 ? countUp(streak.days) : t('common.none'), t('pages.recap.tile_streak'), streak && streak.days > 1 ? dayRange(streak.from, streak.to) : null),
        tile('repeat', rw ? countUp((rw.share || 0) * 100, (x) => pct(Math.round(x) / 100)) : t('common.none'), t('pages.recap.tile_rewatches'), rw ? t(v.you ? 'pages.recap.rewatch_you' : 'pages.recap.rewatch_other', { count: rw.rewatches || 0 }) : null)),
      split.length > 1 ? h('div', { class: 'rc-split' },
        h('h3', { class: 'rc-subhead' }, t('pages.recap.made_of')),
        rankBars(split, (x) => x.watch_s, (x) => `${duration(x.watch_s)} · ${t('terms.play', { count: x.plays })}`)) : null,
    ],
  });
}

function topChapter(id, rows, { icon: ico, eyebrow, title, lead, facts, extra, mark }) {
  if (!rows || !rows.length) return null;
  const [first, ...rest] = rows;
  return chapter({ id, icon: ico, eyebrow, mark, title: title(first), lead: lead(first), body: h('div', { class: 'rc-top' }, feature(first, facts(first)), rankedList(rest, 2, extra)) });
}

function genresChapter(d, v) {
  const rows = d.top_genres || [];
  if (!rows.length) return null;
  const top = rows[0], g = d.genres || {};
  const share = g.watch_s > 0 ? top.watch_s / g.watch_s : null;
  const an = /^[aeiou]/i.test(top.name);
  return chapter({
    id: 'genres', icon: 'tag', eyebrow: t('pages.recap.genres'), mark: top.name,
    title: say(v, an ? 'pages.recap.genres_title_an' : 'pages.recap.genres_title', { genre: em(top.name) }),
    lead: [tParts('pages.recap.genres_lead', { genre: b(top.name), time: b(duration(top.watch_s)), plays: b(t('terms.play', { count: top.plays || 0 })) }),
      rows[1] ? tail(t('pages.recap.genres_close', { genre: rows[1].name })) : '.'],
    stats: [
      g.count ? { value: countUp(g.count), label: g.count === 1 ? t('pages.recap.genre') : t('pages.recap.genres') } : null,
      share != null ? { value: countUp(share * 100, (x) => pct(Math.round(x) / 100)), label: t('pages.recap.stat_was', { genre: top.name }) } : null,
      { value: countUp(top.plays), label: t('pages.recap.plays_label') },
    ],
    body: [rankBars(rows, (x) => x.watch_s || 0, (x) => duration(x.watch_s)),
      h('p', { class: 'rc-note' }, t('pages.recap.genres_note'))],
  });
}

function peopleChapter(d, v) {
  const p = d.people || {};
  const sets = [['actors', t('pages.recap.tab_actors'), p.actors || []], ['directors', t('pages.recap.tab_directors'), p.directors || []]].filter((x) => x[2].length);
  if (!sets.length) return null;
  let tab = sets[0][0];
  const title = h('h2', { class: 'rc-title' }), lead = h('p', { class: 'rc-lead' }), grid = h('ol', { class: 'rc-people' });
  function paint() {
    const rows = sets.find((x) => x[0] === tab)[2], top = rows[0];
    mount(title, tParts(tab === 'actors' ? 'pages.recap.actor_title' : 'pages.recap.director_title', { person: em(top.name) }));
    mount(lead, [tParts(tab === 'actors' ? 'pages.recap.actor_lead' : 'pages.recap.director_lead', { time: b(duration(top.watch_s)), titles: b(t('terms.title', { count: top.titles || 0 })) }),
      top.top_title ? tail(t('pages.recap.people_most', { title: top.top_title })) : '.']);
    mount(grid, rows.map((x, i) => h('li', null, h('a', { class: 'rc-person', href: `/people/${x.id}` },
      h('span', { class: 'rc-person-n mono', 'aria-hidden': 'true' }, '.' + String(i + 1).padStart(2, '0')),
      poster(x.has_image ? x.id : null, x.name, { w: 300, cls: 'rc-person-photo' }),
      h('span', { class: 'rc-person-name' }, x.name),
      h('span', { class: 'rc-person-sub mono', title: durationExact(x.watch_s) }, `${duration(x.watch_s)} · ${t('terms.title', { count: x.titles || 0 })}`)))));
  }
  paint();
  const el = chapter({
    id: 'people', icon: 'users', eyebrow: t('pages.recap.people'), mark: t('pages.recap.people_mark'),
    body: [title, lead,
      sets.length > 1 ? segmented({ label: t('pages.recap.who'), value: tab, options: sets.map(([value, label]) => ({ value, label })), onChange: (val) => { tab = val; paint(); } }) : null,
      grid],
  });
  return el;
}

const PERSONAS = ['night_owl', 'early_bird', 'music_lover', 'weekend_warrior', 'binge_watcher', 'movie_buff', 'creature_of_habit'];

function personaChapter(d, v) {
  const p = d.persona;
  if (!p) return null;
  const known = PERSONAS.includes(p.key);
  const name = known ? t(`pages.recap.persona.name.${p.key}`) : p.title || t('pages.recap.persona.name.creature_of_habit');
  return chapter({
    id: 'persona', icon: 'sparkle', eyebrow: t('pages.recap.persona_eyebrow'), mark: name, cls: 'rc-persona',
    title: say(v, 'pages.recap.persona_title', { persona: tParts(`pages.recap.persona.article.${known ? p.key : /^[aeiou]/i.test(name) ? 'other_an' : 'other'}`, { persona: em(name.toLocaleLowerCase(lang())) }) }),
    lead: p.line ? [p.line, '.'] : null,
  });
}

/** Hours, weekdays and months share one chapter; the headline follows whichever is showing. */
function rhythmChapter(d, v) {
  const hours = Array.isArray(d.hours) && d.hours.length === 24 && d.hours.some((x) => x > 0) ? d.hours : null;
  const weekdays = Array.isArray(d.weekdays) && d.weekdays.length === 7 && d.weekdays.some((x) => x > 0) ? d.weekdays : null;
  const months = (d.months || []).filter((m) => m && m.month);
  const hasMonths = months.some((m) => m.watch_s > 0);
  const tabs = [hours && ['hours', t('pages.recap.tab_hours')], weekdays && ['days', t('pages.recap.tab_days')], hasMonths && ['months', t('pages.recap.tab_months')]].filter(Boolean);
  if (!tabs.length) return null;
  const total = (d.totals && d.totals.watch_s) || 0;
  const peakOf = (arr) => arr.reduce((best, x, i) => (x > arr[best] ? i : best), 0);
  let tab = weekdays ? 'days' : tabs[0][0];
  const title = h('h2', { class: 'rc-title' }), lead = h('p', { class: 'rc-lead' }), plot = h('div', { class: 'rc-plot' });

  function paint() {
    hideTip();
    if (tab === 'hours') {
      const i = peakOf(hours);
      mount(title, say(v, 'pages.recap.hours_title', { hour: em(hour2(i)) }));
      mount(lead, [tParts('pages.recap.hours_lead', { time: b(duration(hours[i])), from: hour2(i), to: hour2((i + 1) % 24) }), total ? tail(t('pages.recap.hours_share', { pct: pct(hours[i] / total) })) : '.']);
      mount(plot, barStrip({ rows: hours.map((x, k) => ({ label: String(k).padStart(2, '0'), title: `${hour2(k)} – ${hour2((k + 1) % 24)}`, value: x })), format: duration, ariaLabel: t('pages.recap.hours_aria'), head: [t('pages.recap.head_hour'), t('terms.watch_time')], labelEvery: 3, cls: 'rc-bars-24' }));
    } else if (tab === 'days') {
      const i = peakOf(weekdays), avg = weekdays.reduce((a, x) => a + x, 0) / 7;
      const over = avg > 0 ? weekdays[i] / avg - 1 : 0;
      mount(title, tParts('pages.recap.days_title_crown', { weekday: em(WEEKDAYS[i]) }));
      mount(lead, [tParts('pages.recap.days_lead', { time: b(duration(weekdays[i])), weekday: WEEKDAYS[i] }), over >= 0.05 ? tail(t('pages.recap.days_over', { pct: pct(over) })) : '.']);
      mount(plot, barStrip({ rows: weekdays.map((x, k) => ({ label: WEEKDAYS_SHORT[k], title: WEEKDAYS[k], value: x })), format: duration, ariaLabel: t('pages.recap.weekday_aria'), head: [t('pages.recap.head_weekday'), t('terms.watch_time')], cls: 'rc-bars-7' }));
    } else {
      const spansYears = new Set(months.map((m) => String(m.month).slice(0, 4))).size > 1;
      const best = months.reduce((a, c) => (c.watch_s > a.watch_s ? c : a));
      const rows = months.map((m) => {
        const dt = monthDate(m.month), name = m.top && m.top.name;
        return { label: mShortF.format(dt), title: (spansYears ? myF : mF).format(dt), value: m.watch_s, note: name ? t('pages.recap.month_mostly', { title: name }) : null,
          foot: h('span', { class: 'rc-col-foot', title: name || null }, m.top ? [poster(m.top.image_item_id, name, { w: 120, cls: 'fui-poster--xs' }), h('span', { class: 'rc-col-foot-name' }, name)] : null) };
      });
      mount(title, tParts('pages.recap.month_title', { month: em(mF.format(monthDate(best.month))) }));
      mount(lead, [tParts('pages.recap.month_lead', { time: b(duration(best.watch_s)) }), best.top && best.top.name ? tail(t('pages.recap.month_most', { title: best.top.name })) : '.']);
      mount(plot, [barStrip({ rows, format: duration, ariaLabel: t('pages.recap.month_aria'), head: [t('pages.recap.head_month'), t('terms.watch_time')], cls: 'rc-bars-months' }),
        h('table', { class: 'sr-only' }, h('caption', null, t('pages.recap.month_caption')),
          h('tbody', null, rows.map((r, k) => h('tr', null, h('td', null, r.title), h('td', null, (months[k].top && months[k].top.name) || t('pages.recap.nothing'))))))]);
    }
  }
  paint();
  return chapter({
    id: 'rhythm', icon: 'activity', eyebrow: t('pages.recap.rhythm'), mark: t('pages.recap.rhythm_mark'),
    body: [title, lead,
      tabs.length > 1 ? segmented({ label: t('pages.recap.rhythm_by'), value: tab, options: tabs.map(([value, label]) => ({ value, label })), onChange: (val) => { tab = val; paint(); } }) : null,
      plot],
  });
}

const HEAT = ['var(--rc-heat-0)', 'var(--rc-heat-1)', 'var(--rc-heat-2)', 'var(--rc-heat-3)', 'var(--rc-heat-4)']; // one hue, quiet → busy (app.css)
const isoDay = (dt) => `${dt.getFullYear()}-${String(dt.getMonth() + 1).padStart(2, '0')}-${String(dt.getDate()).padStart(2, '0')}`;

/** Every day of the period as one cell, a week per column. ←/→ move a week, ↑/↓ a day. */
function heatmap(d) {
  const months = (d.months || []).filter((m) => m && m.month);
  if (!months.length) return null;
  const first = monthDate(months[0].month);
  const lastM = monthDate(months[months.length - 1].month);
  const last = new Date(lastM.getFullYear(), lastM.getMonth() + 1, 0);
  const byDay = new Map((d.days || []).map((x) => [x.date, x]));
  // Steps follow the 95th percentile, so a single marathon day doesn't wash the rest of the year out.
  const sorted = [...byDay.values()].map((x) => x.watch_s).filter((x) => x > 0).sort((a, c) => a - c);
  const cap = sorted.length ? sorted[Math.min(sorted.length - 1, Math.floor(sorted.length * 0.95))] : 0;
  const level = (sec) => (sec > 0 && cap > 0 ? 1 + Math.min(3, Math.floor(Math.min(1, sec / cap) * 4)) : 0);
  const today = isoDay(new Date());

  const cells = [], labels = [];
  const dow = (dt) => (dt.getDay() + 6) % 7; // Monday first
  let col = 1, lastLabelCol = -9;
  for (let dt = new Date(first); dt <= last; dt.setDate(dt.getDate() + 1)) {
    const row = dow(dt) + 1;
    if (row === 1 && cells.length) col++;
    if (dt.getDate() === 1) {
      const at = row === 1 ? col : col + 1; // the first full week of the month
      if (at - lastLabelCol >= 3) { labels.push(h('span', { class: 'rc-heat-month mono', style: { gridColumn: String(at) } }, mShortF.format(dt))); lastLabelCol = at; }
    }
    const key = isoDay(dt), x = byDay.get(key), sec = x ? x.watch_s : 0;
    const cell = h('span', { class: 'rc-heat-cell' + (key > today ? ' is-future' : ''), style: { gridColumn: String(col), gridRow: String(row), background: HEAT[level(sec)] } });
    cell._d = { key, sec, plays: x ? x.plays : 0, col, row };
    cells.push(cell);
  }
  const cols = col;
  const at = (c, r) => cells.find((x) => x._d.col === c && x._d.row === r);
  let active = null;
  function setActive(cell) {
    if (active) active.classList.remove('is-active');
    active = cell || null;
    if (!active) { hideTip(); return; }
    active.classList.add('is-active');
    const x = active._d;
    showTip(active.getBoundingClientRect(), h('div', null,
      h('div', { class: 'fui-tooltip__title' }, longDay(x.key)),
      h('div', { class: 'fui-tooltip__row' }, h('span', { class: 'fui-tooltip__key', style: { background: HEAT[Math.max(1, level(x.sec))] } }),
        x.sec > 0 ? [h('strong', null, duration(x.sec)), h('span', null, t('terms.play', { count: x.plays || 0 }))] : h('span', null, t('pages.recap.nothing_played')))));
  }
  const grid = h('div', { class: 'rc-heat-grid', style: { gridTemplateColumns: `repeat(${cols}, minmax(0, 1fr))` }, 'aria-hidden': 'true' }, cells);
  grid.addEventListener('pointerover', (e) => { if (e.target._d) setActive(e.target); });
  const activeDays = sorted.length;
  const wrap = h('div', { class: 'rc-heat', tabindex: 0, role: 'group',
    'aria-label': t('pages.recap.heat_aria', { count: activeDays }) },
    h('div', { class: 'rc-heat-scroll' },
      h('div', { class: 'rc-heat-body', style: { minWidth: cols * 13 + 34 + 'px' } },
        h('div', { class: 'rc-heat-months', style: { gridTemplateColumns: `repeat(${cols}, minmax(0, 1fr))` }, 'aria-hidden': 'true' }, labels),
        h('div', { class: 'rc-heat-rows' },
          h('div', { class: 'rc-heat-dow mono', 'aria-hidden': 'true' }, WEEKDAYS_SHORT.map((x, k) => h('span', null, k % 2 || k > 4 ? '' : x))),
          grid))),
    h('div', { class: 'rc-heat-legend mono', 'aria-hidden': 'true' }, h('span', null, t('pages.recap.less')), HEAT.map((c) => h('i', { style: { background: c } })), h('span', null, t('pages.recap.more'))),
    h('p', { class: 'sr-only', 'aria-live': 'polite' }));
  const live = wrap.lastChild;
  wrap.addEventListener('pointerleave', () => { if (document.activeElement !== wrap) setActive(null); });
  wrap.addEventListener('blur', () => setActive(null));
  wrap.addEventListener('focus', () => { if (!active) setActive(cells.find((x) => x._d.sec > 0) || cells[0]); });
  wrap.addEventListener('keydown', (e) => {
    const move = { ArrowLeft: [-1, 0], ArrowRight: [1, 0], ArrowUp: [0, -1], ArrowDown: [0, 1] }[e.key];
    if (e.key === 'Escape') { setActive(null); return; }
    if (!move) return;
    e.preventDefault();
    const from = active ? active._d : cells[0]._d;
    const next = at(from.col + move[0], from.row + move[1]);
    if (!next) return;
    setActive(next);
    next.scrollIntoView({ block: 'nearest', inline: 'nearest' });
    live.textContent = next._d.sec > 0 ? t('pages.recap.heat_live', { day: longDay(next._d.key), time: duration(next._d.sec), plays: t('terms.play', { count: next._d.plays || 0 }) }) : t('pages.recap.heat_live_none', { day: longDay(next._d.key) });
  });
  return wrap;
}

function daysChapter(d, v) {
  const map = heatmap(d);
  const tot = d.totals || {};
  if (!map || !tot.active_days) return null;
  const r = d.records || {};
  const streak = r.longest_streak && r.longest_streak.days > 1 ? r.longest_streak : null;
  return chapter({
    id: 'days', icon: 'calendar', eyebrow: sayText(v, 'pages.recap.days_eyebrow'), mark: t('pages.recap.days_mark'),
    title: countParts('pages.recap.days_on', tot.active_days, em(num(tot.active_days))),
    lead: streak
      ? [say(v, 'pages.recap.streak_lead', { days: b(t('pages.recap.streak_days', { count: streak.days })), range: dayRange(streak.from, streak.to) }),
        r.biggest_day ? tail(t('pages.recap.biggest_tail', { day: longDay(r.biggest_day.date), time: duration(r.biggest_day.watch_s) })) : '.']
      : r.biggest_day ? [tParts('pages.recap.biggest_lead', { day: b(longDay(r.biggest_day.date)) }), tail(t('pages.recap.biggest_lead_tail', { time: duration(r.biggest_day.watch_s) }))] : null,
    body: map,
  });
}

function recordsChapter(d, v, isYear) {
  const r = d.records || {};
  const cards = [];
  if (r.biggest_day) cards.push(recordCard({ label: t('pages.recap.biggest_day'), value: duration(r.biggest_day.watch_s), context: `${longDay(r.biggest_day.date)} · ${t('terms.play', { count: r.biggest_day.plays || 0 })}` }));
  if (r.biggest_binge) cards.push(recordCard({ label: t('pages.recap.biggest_binge'), value: t('terms.episode', { count: r.biggest_binge.episodes || 0 }),
    context: `${t('pages.recap.binge_context', { show: r.biggest_binge.series_name || t('pages.recap.one_show'), day: longDay(r.biggest_binge.date) })}${r.biggest_binge.watch_s ? ` · ${duration(r.biggest_binge.watch_s)}` : ''}`,
    image: r.biggest_binge.image_item_id || null, name: r.biggest_binge.series_name }));
  if (r.longest_streak && r.longest_streak.days > 1) cards.push(recordCard({ label: t('pages.recap.longest_streak'), value: t('pages.recap.in_a_row', { count: r.longest_streak.days }), context: dayRange(r.longest_streak.from, r.longest_streak.to) }));
  if (r.longest_play) cards.push(recordCard({ label: t('pages.recap.longest_play'), value: duration(r.longest_play.duration_s),
    context: [r.longest_play.name, r.longest_play.date ? longDay(r.longest_play.date) : null].filter(Boolean).join(' · '), image: r.longest_play.image_item_id || null, name: r.longest_play.name }));
  if (r.most_rewatched) cards.push(recordCard({ label: t('pages.recap.most_rewatched'), value: t('pages.recap.times', { count: r.most_rewatched.plays || 0 }), context: r.most_rewatched.name, image: r.most_rewatched.image_item_id || null, name: r.most_rewatched.name }));
  if (r.first_play) cards.push(recordCard({ label: isYear ? t('pages.recap.first_play_year') : t('pages.recap.first_play_period'), value: r.first_play.name || unknownTitle(),
    context: r.first_play.at ? t('pages.recap.first_play_at', { day: longDate(new Date(r.first_play.at * 1000)), time: hmF.format(new Date(r.first_play.at * 1000)) }) : null,
    image: r.first_play.image_item_id || null, name: r.first_play.name }));
  if (r.oldest_title) cards.push(recordCard({ label: t('pages.recap.oldest_title'), value: r.oldest_title.year ? t('pages.recap.from_year', { year: String(r.oldest_title.year) }) : r.oldest_title.name, context: r.oldest_title.name, image: r.oldest_title.image_item_id || null, name: r.oldest_title.name }));
  if (!cards.length) return null;
  return chapter({ id: 'records', icon: 'trophy', eyebrow: t('pages.recap.records'), mark: t('pages.recap.records'),
    title: v.you ? tParts('pages.recap.records_title_you', { word: em(t('pages.recap.remember')) }) : tParts('pages.recap.records_title_other', { word: em(t('pages.recap.stood_out')) }), body: h('div', { class: 'rc-records' }, cards) });
}

function discoveryChapter(d, v) {
  const x = d.discovery;
  if (!x) return null;
  const once = (x.one_and_done || []).filter(Boolean);
  if (!x.new_series && !x.finished_movies && !x.finished_episodes && !once.length) return null;
  return chapter({
    id: 'discovery', icon: 'compass', eyebrow: t('pages.recap.discovery'), mark: t('pages.recap.discovery_mark'),
    title: x.new_series ? say(v, 'pages.recap.discovery_title', { shows: em(t('terms.show', { count: x.new_series })) }) : tParts('pages.recap.seen_end', { end: em(t('pages.recap.end')) }),
    stats: [
      { value: countUp(x.new_series || 0), label: t('pages.recap.shows_started') },
      { value: countUp(x.finished_movies || 0), label: t('pages.recap.movies_finished') },
      { value: countUp(x.finished_episodes || 0), label: t('pages.recap.episodes_finished') },
    ],
    body: once.length ? h('div', { class: 'rc-once' },
      h('h3', { class: 'rc-subhead' }, t('pages.recap.one_and_done')),
      h('p', { class: 'rc-note' }, t('pages.recap.one_and_done_note')),
      h('ul', { class: 'rc-posterrow' }, once.map((s) => h('li', null, poster(s.image_item_id, s.name, { w: 300, cls: 'rc-posterrow-poster' }),
        s.id ? h('a', { class: 'rc-link rc-posterrow-name', href: `/items/${s.id}` }, s.name || t('pages.recap.unknown_show')) : h('span', { class: 'rc-posterrow-name' }, s.name || t('pages.recap.unknown_show')))))) : null,
  });
}

// ---------------------------------------------------------------- 2.0: what the rest of FinStats left lying around

/** Evenings in company. Companions are named here, in the app, and never on a card. */
function togetherChapter(d, v) {
  const g = d.together;
  if (!g || !g.evenings) return null;
  const top = g.top_title;
  const people = g.companions || [];
  return chapter({
    id: 'together', icon: 'together', eyebrow: t('nav.together'), mark: t('pages.recap.together_mark'),
    title: tParts('pages.recap.together_title', { evenings: em(t('pages.recap.evenings', { count: g.evenings })) }),
    lead: [tParts('pages.recap.together_lead', { hours: hoursB(g.together_s) }), g.share ? tail(sayText(v, 'pages.recap.together_share', { pct: pct(g.share) })) : '.'],
    body: [
      people.length ? h('div', null, h('h3', { class: 'rc-subhead' }, sayText(v, 'pages.recap.watched_with')),
        h('ul', { class: 'rc-chips' }, people.map((p) => h('li', { class: 'rc-chip' },
          h('a', { class: 'rc-link rc-person', href: `/users/${p.user_id}` }, avatar(p.user_id, p.user_name, { size: 22, hasImage: p.has_image }), h('span', { class: 'rc-chip-name' }, p.user_name)),
          h('span', { class: 'rc-chip-val mono' }, t('pages.recap.evenings', { count: p.evenings || 0 })))))) : null,
      people_in(g),
      top ? h('div', { class: 'rc-once' }, h('h3', { class: 'rc-subhead' }, t('pages.recap.together_top')),
        h('ul', { class: 'rc-posterrow' }, h('li', null, poster(top.image_item_id, top.name, { w: 300, cls: 'rc-posterrow-poster' }),
          h('a', { class: 'rc-link rc-posterrow-name', href: `/items/${top.id}` }, top.name), h('span', { class: 'rc-note' }, t('pages.recap.evenings', { count: top.evenings || 0 }))))) : null,
    ],
  });
}
const people_in = (g) => (g.people_in_company ? h('p', { class: 'rc-note' }, t('pages.recap.people_in_company', { count: g.people_in_company })) : null);

/** Seen to the end, and left for later, by the same "seen" as the profile's progress bars. */
function finishedChapter(d, v) {
  const f = d.finished;
  if (!f || (!f.count && !f.dropped_count)) return null;
  const shelf = (rows, sub) => h('ul', { class: 'rc-posterrow' }, rows.map((s) => h('li', null, poster(s.image_item_id, s.name, { w: 300, cls: 'rc-posterrow-poster' }),
    h('a', { class: 'rc-link rc-posterrow-name', href: `/items/${s.id}` }, s.name), h('span', { class: 'rc-note' }, sub(s)))));
  return chapter({
    id: 'finished', icon: 'check', eyebrow: t('pages.recap.finished'), mark: t('pages.recap.finished_mark'),
    title: f.count ? tParts('pages.recap.finished_title', { shows: em(t('terms.show', { count: f.count })) }) : tParts('pages.recap.left_title', { later: em(t('pages.recap.later')) }),
    body: [
      f.series && f.series.length ? shelf(f.series, (s) => `${t('terms.episode', { count: s.episodes || 0 })} · ${dayMonth(parseDay(s.finished_on))}`) : null,
      f.dropped && f.dropped.length ? h('div', { class: 'rc-once' }, h('h3', { class: 'rc-subhead' }, t('pages.recap.left_for_later')),
        h('p', { class: 'rc-note' }, t('pages.recap.dropped_note')),
        shelf(f.dropped, (s) => t('pages.recap.seen_of', { seen: s.seen || 0, count: s.total || 0 }))) : null,
    ],
  });
}

/** What was asked for through Seerr, what arrived, and what was then watched. */
function askedChapter(d, v) {
  const q = d.requests;
  if (!q || !q.made) return null;
  return chapter({
    id: 'asked', icon: 'download', eyebrow: t('pages.recap.asked'), mark: t('pages.recap.asked_mark'),
    title: say(v, 'pages.recap.asked_title', { requests: em(t('pages.recap.requests', { count: q.made })) }),
    stats: [
      { value: countUp(q.made), label: t('pages.recap.asked') },
      { value: countUp(q.available), label: t('pages.recap.arrived') },
      { value: countUp(q.watched), label: t('pages.recap.watched_after') },
    ],
    body: q.top && q.top.length ? h('ul', { class: 'rc-posterrow' }, q.top.map((r) => h('li', null, poster(r.image_item_id, r.title, { w: 300, cls: 'rc-posterrow-poster' }),
      r.item_id ? h('a', { class: 'rc-link rc-posterrow-name', href: `/items/${r.item_id}` }, r.title) : h('span', { class: 'rc-posterrow-name' }, r.title)))) : null,
  });
}

/** The year against the one before it. */
function versusChapter(d, v) {
  const p = d.versus;
  const tot = d.totals || {};
  if (!p || !p.watch_s) return null;
  const change = Math.round(((tot.watch_s || 0) - p.watch_s) / p.watch_s * 100);
  const year = String(p.year);
  return chapter({
    id: 'versus', icon: 'chart', eyebrow: t('pages.recap.versus_eyebrow', { year }), mark: t('pages.recap.versus_mark'),
    title: tParts('pages.recap.versus_title', { change: em(t(change >= 0 ? 'pages.recap.versus_more' : 'pages.recap.versus_less', { pct: pct(Math.abs(change) / 100) })), year }),
    stats: [
      { value: countUp(hoursOf(tot.watch_s)), label: t('pages.recap.hours_in', { year: String(d.year) }) },
      { value: countUp(hoursOf(p.watch_s)), label: t('pages.recap.hours_in', { year }) },
      { value: countUp(p.plays), label: t('pages.recap.plays_in', { year }) },
    ],
  });
}

// ---------------------------------------------------------------- 2.0: the year as a story of cards

const CARD_NAMES = Object.fromEntries(['year', 'numbers', 'shows', 'films', 'music', 'genres', 'persona', 'rhythm', 'days', 'records', 'together', 'finished', 'asked', 'versus']
  .map((k) => [k, t(`pages.recap.card.${k}`)]));

/** Every card of the year, to look at and to save. Cards name nobody but the server; a login name never goes on one. */
function openStory(d, params) {
  const q = new URLSearchParams(Object.entries(params).filter(([, x]) => x));
  const slug = String(d.year || 'recap').toLowerCase().replace(/[^a-z0-9]+/g, '-');
  const body = h('div', { class: 'story-body' },
    h('p', { class: 'fui-field__help' }, t('pages.recap.cards_help')),
    h('ol', { class: 'story-grid' }, (d.story || []).map((key) => {
      const src = `/api/recap/cards/${key}?${q}`;
      return h('li', null,
        h('img', { src, alt: t('pages.recap.card_alt', { card: CARD_NAMES[key] || key }), loading: 'lazy', decoding: 'async', width: 1080, height: 1920 }),
        h('span', { class: 'fui-field__help' }, CARD_NAMES[key] || key, ' · ', h('a', { href: src, download: `finstats-${slug}-${key}.png` }, t('pages.recap.download'))));
    })),
    h('div', { class: 'form-actions' }, button({ variant: 'primary', href: `/api/recap/cards.zip?${q}`, download: `finstats-${slug}.zip` }, icon('download', 13), t('pages.recap.download_all'))));
  openModal({ title: t('pages.recap.cards_title'), body, wide: true, cls: 'story-modal' });
}

function clientsChapter(d, v) {
  const rows = (d.clients || []).filter((c) => c && c.name);
  if (!rows.length) return null;
  const total = (d.totals && d.totals.plays) || rows.reduce((a, c) => a + (c.plays || 0), 0) || 1;
  return chapter({
    id: 'clients', icon: 'monitor', eyebrow: sayText(v, 'pages.recap.clients_eyebrow'), mark: t('pages.recap.clients_mark'),
    title: tParts('pages.recap.clients_title', { app: em(rows[0].name) }),
    body: h('ul', { class: 'rc-chips' }, rows.map((c) => h('li', { class: 'rc-chip' }, h('span', { class: 'rc-chip-name' }, c.name), h('span', { class: 'rc-chip-val mono' }, t('pages.recap.of_plays', { pct: pct((c.plays || 0) / total) }))))),
  });
}

/** The end, the way films end: the year's credits. */
function outroChapter(d, v, periodLabel) {
  const isYear = /^\d{4}$/.test(periodLabel);
  const names = (rows, n) => (rows || []).slice(0, n).map((x) => x && x.name).filter(Boolean);
  const p = d.people || {};
  const credits = [
    [t('pages.recap.credit.starring'), names(d.top_series, 3)],
    [t('pages.recap.credit.feature'), names(d.top_movies, 1)],
    [t('pages.recap.credit.with'), names(p.actors, 2)],
    [t('pages.recap.credit.directed'), names(p.directors, 1)],
    [t('pages.recap.credit.soundtrack'), names(d.top_tracks, 1)],
    [t('pages.recap.credit.genre'), names(d.top_genres, 1)],
    [t('pages.recap.credit.screened'), names(d.clients, 1)],
    [t('pages.recap.credit.running'), d.totals && d.totals.watch_s ? [duration(d.totals.watch_s)] : []],
  ].filter(([, list]) => list.length);
  const rows = credits.map(([role, list]) => h('div', { class: 'rc-credit' }, h('dt', null, role), h('dd', null, list.map((n) => h('span', null, n)))));
  rows.forEach((el, i) => el.style.setProperty('--i', String(i)));
  return h('section', { class: 'rc-credits rc-reveal', id: 'rc-outro', 'aria-label': t('pages.recap.credits') },
    h('p', { class: 'rc-eyebrow' }, icon('film', 15), h('span', null, t('pages.recap.roll_credits'))),
    h('h2', { class: 'rc-title' }, isYear ? tParts('pages.recap.that_was', { year: em(periodLabel) }) : tParts('pages.recap.that_was_last12', { months: em(t('pages.recap.twelve_months')) })),
    h('dl', { class: 'rc-credits-list' }, rows),
    h('p', { class: 'rc-credits-line' }, sayText(v, 'pages.recap.outro_line')),
    h('div', { class: 'rc-outro-links' }, button({ href: '/activity' }, icon('activity', 14), t('pages.recap.see_activity')),
      d.scope && d.scope.user_id ? button({ variant: 'ghost', href: `/users/${d.scope.user_id}` }, icon('user', 14), sayText(v, 'pages.recap.profile')) : null));
}

// ---------------------------------------------------------------- page
function buildStory(d, me, periodLabel, picker, share) {
  const v = voiceFor(d.scope, me);
  const parts = [
    heroChapter(d, v, periodLabel, picker, share),
    numbersChapter(d, v, periodLabel),
    topChapter('shows', d.top_series, {
      icon: 'tv', eyebrow: t('pages.recap.top_shows'), mark: t('pages.recap.shows_mark'),
      title: (x) => say(v, 'pages.recap.shows_title', { title: em(x.name) }),
      lead: (x) => [x.episodes ? tParts('pages.recap.shows_lead_episodes', { hours: hoursB(x.watch_s), episodes: b(t('terms.episode', { count: x.episodes })) }) : tParts('pages.recap.shows_lead', { hours: hoursB(x.watch_s) }), tail(t('pages.recap.shows_more'))],
      facts: (x) => [[t('terms.watch_time'), duration(x.watch_s)], x.episodes ? [t('pages.recap.episodes_label'), num(x.episodes)] : null, [t('pages.recap.plays_label'), num(x.plays)]],
      extra: (x) => (x.episodes ? t('terms.episode', { count: x.episodes }) : null),
    }),
    topChapter('movies', d.top_movies, {
      icon: 'film', eyebrow: t('pages.recap.top_movies'), mark: t('pages.recap.movies_mark'),
      title: (x) => say(v, 'pages.recap.movies_title', { title: em(x.name) }),
      lead: (x) => [tParts('pages.recap.movies_lead', { time: b(duration(x.watch_s)) }), x.plays > 1 ? tail(sayText(v, 'pages.recap.movies_back', { count: x.plays })) : '.'],
      facts: (x) => [[t('terms.watch_time'), duration(x.watch_s)], [t('pages.recap.plays_label'), num(x.plays)]],
      extra: (x) => t('terms.play', { count: x.plays || 0 }),
    }),
    d.top_tracks && d.top_tracks.length ? chapter({ id: 'tracks', icon: 'volume', eyebrow: t('pages.recap.top_tracks'), mark: t('pages.recap.tracks_mark'), title: tParts('pages.recap.tracks_title', { repeat: em(t('pages.recap.repeat')) }), body: rankedList(d.top_tracks, 1, (x) => t('terms.play', { count: x.plays || 0 })) }) : null,
    genresChapter(d, v),
    peopleChapter(d, v),
    personaChapter(d, v),
    rhythmChapter(d, v),
    daysChapter(d, v),
    recordsChapter(d, v, /^\d{4}$/.test(periodLabel)),
    discoveryChapter(d, v),
    togetherChapter(d, v),
    finishedChapter(d, v),
    askedChapter(d, v),
    versusChapter(d, v),
    clientsChapter(d, v),
    outroChapter(d, v, periodLabel),
  ];
  return parts.filter(Boolean);
}

// Shared with the prefetcher, so a prefetched view has exactly the address the page asks for.
const loadRecap = (year, userId, scope, signal) => api.get('/recap', { year, user_id: userId, scope }, { signal });
const yearOf = (query) => { const y = query.get('year'); return /^\d{4}$/.test(y || '') || y === 'last12' ? y : ''; };
const scopeOf = (query) => (isAdmin() && query.get('scope') === 'server' ? 'server' : '');
export const prefetchRecap = ({ query, signal }) => [() => loadRecap(yearOf(query), isAdmin() ? query.get('user') || '' : '', scopeOf(query), signal)];

export default function recapPage(ctx) {
  ctx.title(t('nav.recap'));
  const me = state.user;
  let year = yearOf(ctx.query);
  // Administrators can open one other person's recap, or the whole server's year (2.0).
  let userId = isAdmin() ? ctx.query.get('user') || '' : '';
  let scope = scopeOf(ctx.query);
  let reveal = revealer();
  ctx.onCleanup(() => { reveal.stop(); hideTip(); });

  const people = isAdmin() ? combobox({ value: userId, allLabel: t('pages.recap.my_recap'), placeholder: t('pages.recap.my_recap'), label: t('pages.recap.whose_recap'),
    load: () => userList(ctx.signal).then((us) => us.filter((u) => !me || u.id !== me.id).map((u) => ({ value: u.id, label: u.name }))),
    onChange: (val) => { userId = val; year = ''; sync(); dv.load(); } }) : null;
  const scopeSwitch = isAdmin() ? h('div', { class: 'rc-scope' }, segmented({ label: t('pages.recap.whose_year'), value: scope || 'user',
    options: [{ value: 'user', label: t('pages.recap.a_person') }, { value: 'server', label: t('nav.server') }],
    onChange: (val) => { scope = val === 'server' ? 'server' : ''; people.hidden = !!scope; sync(); dv.load(); } })) : null;
  if (people) people.hidden = !!scope;
  const controls = isAdmin() ? h('div', { class: 'filters rc-controls' }, scopeSwitch, people) : null;
  const view = h('div', { class: 'rc-story' });

  const periodLabel = (y) => (y === 'last12' ? t('pages.recap.last_12_months') : String(y));
  const sync = () => replaceQuery({ year, user: scope ? '' : userId, scope });

  const dv = dataView({
    container: view, signal: ctx.signal,
    skeleton: () => [h('div', { class: 'rc-hero rc-sk' }, h('div', { class: 'rc-hero-text' }, sk.line('120px', 12), sk.line('min(80%, 520px)', 64), sk.line('min(90%, 360px)', 16), sk.block(120))),
      h('div', { class: 'rc-chapter is-in' }, sk.line('120px', 12), sk.line('min(80%, 380px)', 30), sk.block(220)),
      h('div', { class: 'rc-chapter is-in' }, sk.line('120px', 12), sk.line('min(80%, 380px)', 30), sk.block(220))],
    fetch: () => loadRecap(year, scope ? '' : userId, scope, ctx.signal),
    render: (d) => {
      if (d.year != null) year = String(d.year);
      const picker = yearTabs(d, String(year || ''), (val) => { year = val; sync(); dv.load(); });
      const label = year ? periodLabel(year) : t('pages.recap.this_period');
      ctx.title(t('pages.recap.title_period', { period: label }));
      reveal.stop();
      reveal = revealer();
      view.classList.toggle('rc-anim', reveal.animated); // content is only ever hidden when something will reveal it
      if (d.empty || !d.totals || !d.totals.plays) {
        const v = voiceFor(d.scope, me);
        return [h('div', { class: 'rc-empty-picker' }, picker),
          emptyState(year === 'last12' ? t('pages.recap.empty_last12') : t('pages.recap.empty_period', { period: label }), sayText(v, 'pages.recap.empty_sub'))];
      }
      const share = d.story && d.story.length ? button({ class: 'rc-share', type: 'button', onClick: () => openStory(d, { year: String(d.year), user_id: scope ? '' : userId, scope }) },
        icon('share', 14), t('pages.recap.share')) : null;
      const story = buildStory(d, me, label, picker, share);
      // Observe after mount so the first screen reveals immediately.
      requestAnimationFrame(() => story.forEach((el) => { if (el.classList.contains('rc-reveal')) reveal.watch(el); }));
      return story;
    },
  });

  ctx.root.append(...[h('h1', { class: 'sr-only' }, t('nav.recap')), controls ? h('div', { class: 'rc-top-row' }, controls) : null, view].filter(Boolean));
  ctx.root.classList.add('rc-page');
  ctx.onCleanup(() => ctx.root.classList.remove('rc-page'));
  dv.load();
}

/** Dashboard banner, December and January only. */
export function recapBanner() {
  const now = new Date();
  const m = now.getMonth();
  if (m !== 11 && m !== 0) return null;
  const y = m === 11 ? now.getFullYear() : now.getFullYear() - 1;
  return h('a', { class: 'recap-banner', href: `/recap?year=${y}` }, icon('recap', 15),
    h('span', null, t('pages.recap.banner', { year: String(y) })), icon('chevronRight', 14, 'recap-banner-go'));
}
