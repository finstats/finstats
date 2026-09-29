import { h, icon, num, bytes, bitrate, duration, durationExact, durEl, relEl, dateTime, episodeCode, compact, safeHttps, languageName, pct } from '../dom.js';
import { api, imgItem } from '../api.js';
import { readDays, saveDays, can } from '../state.js';
import { replaceQuery } from '../router.js';
import { card, chartCard, filterBar, dataView, sk, poster, avatar, chip, statTile, playsTable, emptyState, facts } from '../components.js';
import { retentionChart, retentionTable, simpleColumns, simpleColumnsTable } from '../charts.js';
import { activityCard } from '../widgets.js';
import { openPlayModal } from '../playmodal.js';
import { plainTable } from '../tables.js';
import { agenda } from '../upcoming.js';
import { watchToggle, watchable } from '../watchlist.js';

const TYPE_LABEL = { Movie: 'Movie', Series: 'Series', Episode: 'Episode', Season: 'Season', Audio: 'Track', MusicAlbum: 'Album' };

// Shared with the prefetcher, so a prefetched view has exactly the address the page asks for.
async function loadItem(id, days, signal) {
  const o = { signal };
  const d = await api.get(`/items/${id}`, { days }, o);
  const key = d.item.type === 'Series' ? 'series_id' : 'item_id';
  const recent = await api.get('/activity', { days, [key]: id, page: 1, per_page: 10 }, o);
  return { d, recent, key };
}
export const prefetchItem = ({ params, query, signal }) => [() => loadItem(params.id, readDays(query), signal)];

export default function itemPage(ctx) {
  const id = ctx.params.id;
  ctx.title('Title');
  let days = readDays(ctx.query);
  const view = h('div', { class: 'stack' });
  const filtersSlot = h('div');

  const dv = dataView({
    container: view, signal: ctx.signal,
    skeleton: () => [h('div', { class: 'item-hero' }, h('span', { class: 'sk sk-poster-lg' }), h('div', { class: 'sk-row-lines' }, sk.line('40%', 28), sk.line('60%'), sk.line('30%'))), sk.tiles(3), sk.cardBlock(240)],
    fetch: () => loadItem(id, days, ctx.signal),
    render: ({ d, recent, key }) => {
      const it = d.item, t = d.totals || {}, langs = languages(it);
      ctx.title(it.name);
      const code = episodeCode(it.season_number, it.episode_number);
      const meta = [
        chip(TYPE_LABEL[it.type] || it.type),
        it.year ? chip(String(it.year)) : null,
        it.official_rating ? chip(it.official_rating) : null,
        it.community_rating ? chip('★ ' + Number(it.community_rating).toFixed(1), 'Community rating') : null,
        it.runtime_s ? chip(duration(it.runtime_s), 'Runtime') : null,
        it.video ? h('span', { class: 'chip mono' }, it.video) : null,
        it.audio ? h('span', { class: 'chip mono' }, it.audio) : null,
        it.removed ? chip('No longer in library') : null,
      ];
      // An episode shows its own still (16:9); without one it borrows the show's poster.
      const still = it.type === 'Episode' && it.has_image;
      const hero = h('div', { class: ['item-hero', it.has_backdrop && 'has-backdrop'] },
        it.has_backdrop ? h('div', { class: 'item-backdrop', 'aria-hidden': 'true' }, h('img', { src: imgItem(it.id, 1280, 'backdrop'), alt: '', decoding: 'async', onError: (e) => e.target.parentNode.remove() })) : null,
        still ? poster(it.id, it.name, { w: 480, cls: 'poster-lg poster-still' })
          : poster(it.type === 'Episode' && it.series_id ? it.series_id : it.id, it.name, { w: 300, cls: 'poster-lg' }),
        h('div', { class: 'item-hero-text' },
          it.series_name && it.series_id ? h('a', { class: 'item-series', href: `/items/${it.series_id}` }, it.series_name, code ? h('span', { class: 'mono' }, ' · ' + code) : null) : null,
          h('h1', { class: 'page-title' }, it.name),
          h('div', { class: 'chips' }, meta),
          it.genres && it.genres.length ? h('p', { class: 'item-genres' }, it.genres.join(' · ')) : null,
          Array.isArray(it.studios) && it.studios.length ? h('p', { class: 'item-studios' }, it.studios.slice(0, 4).join(' · ')) : null,
          languageLines(langs),
          externalLinks(it.external),
          itemActions(it),
          it.overview ? h('p', { class: 'item-overview' }, it.overview) : null,
          it.library_id ? h('p', { class: 'item-lib' }, 'In ', h('a', { href: `/libraries/${it.library_id}` }, it.library_name || 'library'),
            it.date_created ? [' · added ', h('span', { title: dateTime(it.date_created) }, relEl(it.date_created, ''))] : null) : null));

      const file = facts([
        it.container ? ['Container', it.container, { mono: true }] : null,
        it.size_bytes ? ['Size', bytes(it.size_bytes), { mono: true }] : null,
        it.bitrate ? ['Bitrate', bitrate(it.bitrate), { mono: true }] : null,
        it.bit_depth ? ['Bit depth', it.bit_depth + '-bit', { mono: true }] : null,
        it.framerate ? ['Frame rate', (Math.round(Number(it.framerate) * 1000) / 1000) + ' fps', { mono: true }] : null,
        can('see_server') && it.path ? ['Path', h('span', { class: 'mono path' }, it.path), { wide: true }] : null,
      ]);

      return [
        hero,
        h('div', { class: 'tiles tiles-3' },
          statTile({ label: 'Watch time', value: duration(t.watch_s), title: durationExact(t.watch_s), hint: ' ' }),
          statTile({ label: 'Plays', value: compact(t.plays), title: num(t.plays), hint: t.last_played_at ? ['last ', relEl(t.last_played_at, '')] : 'Never played' }),
          statTile({ label: 'Watched by', value: `${num(t.users)} ${t.users === 1 ? 'user' : 'users'}`, hint: ' ' })),
        activityCard({ daily: d.daily, bucket: d.bucket, title: 'Plays over time' }),
        whereTheyStop(d.insights),
        whoKeepsWatching(it, d.seasons),
        castCard(d.people),
        comingCard(it),
        languagesCard(langs),
        d.seasons && d.seasons.length ? card({ title: 'Seasons', sub: 'Plays per episode in this range', body: seasons(d.seasons) }) : null,
        card({ title: 'Watched by', cls: 'card-flush', body: watchers(d.watchers) }),
        playedBy(d.played_by),
        card({ title: 'Recent plays', cls: 'card-flush',
          actions: h('a', { class: 'btn btn-ghost btn-sm', href: `/activity?${key}=${encodeURIComponent(id)}&days=${days}` }, 'View all'),
          body: playsTable(recent.rows, { showUser: can('see_everyone'), onOpen: (p) => openPlayModal(p, { onDeleted: () => dv.load() }), empty: 'No plays in this range.' }) }),
        file.children.length ? card({ title: 'File', body: file }) : null,
      ];
    },
  });

  filtersSlot.append(filterBar({ days, onDays: (v) => { days = v; saveDays(v); replaceQuery({ days }); dv.load(); } }));
  ctx.root.append(filtersSlot, view);
  dv.load();
}

/** Where plays of this film or episode stopped, with rewinds and subtitle switch-ons on the same axis.
 *  Absent under three plays, or on anything without a runtime: the server answers null for those. */
function whereTheyStop(ins) {
  if (!ins || !Array.isArray(ins.curve) || ins.curve.length < 2) return null;
  const parts = [`${num(ins.plays)} ${ins.plays === 1 ? 'play' : 'plays'}`,
    `${num(ins.measured)} measured exactly, ${num(ins.estimated)} estimated from time watched`, 'skips under 20 s are not recorded'];
  return chartCard({ title: 'Where people stop', sub: parts.join(' · '),
    chart: () => retentionChart(ins),
    table: () => retentionTable(ins) });
}

/** A show: how many people started each episode, in order. "Everyone quits episode three" is people
 *  who never press play on episode four. Needs three played episodes to be a shape. */
function whoKeepsWatching(it, seasons) {
  if (!it || it.type !== 'Series' || !Array.isArray(seasons)) return null;
  const eps = seasons.flatMap((sn) => sn.episodes || []).filter((e) => e.plays > 0 && e.users != null);
  if (eps.length < 3) return null;
  const rows = eps.map((e) => {
    const code = episodeCode(e.season_number ?? seasons.find((sn) => (sn.episodes || []).includes(e))?.season_number, e.episode_number);
    return { label: code, title: `${code} · ${e.name}`, value: e.users };
  });
  return chartCard({ title: 'Who keeps watching', sub: 'People who started each episode, in order',
    chart: () => simpleColumns({ rows, unit: ['viewer', 'viewers'], ariaLabel: 'Viewers per episode' }),
    table: () => simpleColumnsTable({ rows, head: ['Episode', 'Viewers'] }) });
}

/** What Sonarr or Radarr expect next for this title. Absent without a connection or without anything due. */
function comingCard(it) {
  const list = Array.isArray(it.upcoming) ? it.upcoming : [];
  if (!list.length) return null;
  return card({ title: it.type === 'Series' ? 'Next episodes' : 'Coming releases', sub: 'From Sonarr and Radarr', cls: 'card-agenda', body: agenda(list) });
}

/**
 * Which languages it can be played in. For a file: its tracks. For a show or a season: each language with how many
 * episodes have it, because a dub that stops after season one is exactly what people want to know before starting.
 * `total` is null for a file, whose every language covers all of it.
 */
function languages(it) {
  const cov = it.language_coverage;
  if (cov && cov.episodes) return { total: cov.episodes, audio: cov.audio || [], subtitles: cov.subtitles || [] };
  const files = (codes) => (Array.isArray(codes) ? codes : []).map((code) => ({ code, episodes: null }));
  return { total: null, audio: files(it.audio_languages), subtitles: files(it.subtitle_languages) };
}
const partial = (L, r) => L.total != null && r.episodes < L.total;
const ofEpisodes = (L, r) => `${num(r.episodes)} of ${num(L.total)} episodes`;

// A long show can carry two dozen subtitle languages. The hero names the first few, most complete first, one line
// each, and the Languages card further down has all of them: listing them all here pushed the artwork down a screen.
const HERO_NAMES = 3;
const heroNames = (rows) => (rows.length <= HERO_NAMES + 1 ? rows : rows.slice(0, HERO_NAMES));
const needsCard = (L) => [L.audio, L.subtitles].some((rows) => heroNames(rows).length < rows.length || rows.some((r) => partial(L, r)));

function jumpTo(id, text) {
  return h('a', { class: 'item-langs-more', href: '#' + id, onClick: (e) => {
    const target = document.getElementById(id);
    if (!target) return;
    e.preventDefault();   // a bare #fragment is a history entry, and the router re-mounts the page on popstate
    target.scrollIntoView({ behavior: matchMedia('(prefers-reduced-motion: reduce)').matches ? 'auto' : 'smooth', block: 'start' });
    target.focus({ preventScroll: true });
  } }, text);
}

function languageLines(L) {
  const card = needsCard(L);
  const line = (label, rows) => {
    if (!rows.length) return null;
    const shown = heroNames(rows), rest = rows.length - shown.length;
    return h('p', { class: 'item-langs' }, h('span', { class: 'item-langs-label' }, label),
      h('span', { class: 'item-langs-names' }, shown.flatMap((r, i) => [i ? ' · ' : null,
        partial(L, r) ? h('span', { class: 'lang-partial', title: ofEpisodes(L, r) }, languageName(r.code)) : languageName(r.code)]).filter((n) => n != null)),
      rest ? jumpTo('languages', `+${num(rest)} more`) : card && rows.some((r) => partial(L, r)) ? jumpTo('languages', 'by episode') : null);
  };
  const lines = [line('Audio', L.audio), line('Subtitles', L.subtitles)].filter(Boolean);
  return lines.length ? h('div', { class: 'item-langs-box' }, lines) : null;
}

/** Every language, when the hero could not say it all: the complete ones as a list, the partial ones with how far they go. */
function languagesCard(L) {
  if (!needsCard(L)) return null;
  const mark = (kind, r) => ({ 'data-kind': kind, 'data-lang': r.code, 'data-episodes': r.episodes == null ? null : String(r.episodes) });
  const group = (kind, label, rows) => {
    if (!rows.length) return null;
    const full = rows.filter((r) => !partial(L, r)).sort((a, b) => languageName(a.code).localeCompare(languageName(b.code)));
    const part = rows.filter((r) => partial(L, r));
    return h('div', { class: 'lang-group' },
      h('h3', { class: 'lang-group-title' }, label, h('span', { class: 'lang-group-count mono' }, num(rows.length))),
      full.length ? [L.total != null && part.length ? h('p', { class: 'lang-sub' }, `In all ${num(L.total)} episodes`) : null,
        h('ul', { class: 'chips lang-chips' }, full.map((r) => h('li', { class: 'chip', ...mark(kind, r) }, languageName(r.code))))] : null,
      part.length ? [h('p', { class: 'lang-sub' }, full.length ? 'In some of them' : `In some of the ${num(L.total)} episodes`),
        h('ul', { class: 'lang-bars' }, part.map((r) => h('li', mark(kind, r),
          h('span', { class: 'lang-name' }, languageName(r.code)),
          h('span', { class: 'lang-track', 'aria-hidden': 'true' }, h('span', { class: 'lang-fill', style: { width: Math.max(2, (r.episodes / L.total) * 100) + '%' } })),
          h('span', { class: 'lang-count mono' }, `${num(r.episodes)} of ${num(L.total)}`))))] : null);
  };
  const el = card({ title: 'Languages', id: 'languages', cls: 'card-langs',
    sub: L.total != null ? `Across the ${num(L.total)} episodes on disk` : 'Tracks in the file',
    body: h('div', { class: 'grid-2' }, group('audio', 'Audio', L.audio), group('subtitles', 'Subtitles', L.subtitles)) });
  el.tabIndex = -1;   // the hero's "+n more" moves focus here, so a keyboard continues from the list it asked for
  return el;
}

/** Directors first, then the billed cast. Each one opens that person's page. */
function castCard(people) {
  const rows = (Array.isArray(people) ? people : []).filter((p) => p && p.id && p.name);
  if (!rows.length) return null;
  return card({ title: 'Cast & crew', body: h('ul', { class: 'cast-row' }, rows.map((p) => h('li', null, h('a', { class: 'cast-person', href: `/people/${p.id}` },
    poster(p.has_image ? p.id : null, p.name, { w: 160, cls: 'cast-photo' }),
    h('span', { class: 'cast-name' }, p.name),
    h('span', { class: 'cast-role' }, p.kind === 'Director' ? 'Director' : p.role || 'Actor'))))) });
}

function externalLinks(list) {
  const links = (Array.isArray(list) ? list : []).map((x) => ({ label: x && x.label, url: safeHttps(x && x.url) })).filter((x) => x.url && x.label);
  if (!links.length) return null;
  return h('div', { class: 'chips' }, links.map((x) =>
    h('a', { class: 'chip chip-link', href: x.url, target: '_blank', rel: 'noopener noreferrer' }, x.label, icon('external', 11), h('span', { class: 'sr-only' }, ' (opens in a new tab)'))));
}

/** Jellyfin's played flags: covers people who watched before finstats existed. */
function playedBy(rows) {
  if (!Array.isArray(rows) || !rows.length) return null;
  return card({ title: 'Marked played in Jellyfin', sub: 'Jellyfin’s own flags, including history from before finstats', cls: 'card-flush',
    body: plainTable(h('table', { class: 'table' },
      h('thead', null, h('tr', null, h('th', null, 'User'), h('th', null, 'Last played'), h('th', null, h('span', { class: 'sr-only' }, 'Favourite')))),
      h('tbody', null, rows.map((r) => h('tr', null,
        h('td', null, h('a', { class: 'user-cell', href: `/users/${r.user_id}` }, avatar(r.user_id, r.user_name, { size: 22 }), h('span', null, r.user_name || 'Unknown user'))),
        h('td', null, r.last_played_at ? relEl(r.last_played_at) : h('span', { class: 'muted' }, '–')),
        h('td', null, r.is_favorite ? h('span', { class: 'sev fav' }, icon('heart', 13), 'Favourite') : null)))))) });
}

function watchers(rows) {
  if (!rows || !rows.length) return emptyState('Nobody has played this in the selected range.');
  return plainTable(h('table', { class: 'table' },
    h('thead', null, h('tr', null, h('th', null, 'User'), h('th', { class: 'r' }, 'Plays'), h('th', { class: 'r' }, 'Watch time'), h('th', null, 'Last played'))),
    h('tbody', null, rows.map((w) => h('tr', null,
      h('td', null, h('a', { class: 'user-cell', href: `/users/${w.user_id}` }, avatar(w.user_id, w.user_name, { size: 22 }), h('span', null, w.user_name))),
      h('td', { class: 'mono r' }, num(w.plays)), h('td', { class: 'r' }, durEl(w.watch_s)), h('td', null, relEl(w.last_played_at)))))));
}

function seasons(list) {
  const withAudio = list.some((sn) => sn.episodes.some((e) => Array.isArray(e.audio_languages) && e.audio_languages.length));
  const withFinished = list.some((sn) => sn.episodes.some((e) => e.finished != null));
  return h('div', { class: 'seasons' }, list.map((sn, i) => {
    const plays = sn.episodes.reduce((a, e) => a + (e.plays || 0), 0);
    const max = Math.max(1, ...sn.episodes.map((e) => e.plays || 0));
    const det = h('details', { class: 'season' },
      h('summary', null, icon('chevronRight', 14), h('span', { class: 'season-name' }, sn.name || `Season ${sn.season_number}`),
        h('span', { class: 'season-meta mono' }, `${num(sn.episodes.length)} ep · ${num(plays)} ${plays === 1 ? 'play' : 'plays'}`)),
      plainTable(h('table', { class: 'table table-dense episodes' },
        h('thead', null, h('tr', null, h('th', null, '#'), h('th', null, 'Episode'), withAudio ? h('th', null, 'Audio') : null, h('th', { 'data-nosort': '' }, h('span', { class: 'sr-only' }, 'Share')), h('th', { class: 'r' }, 'Plays'), withFinished ? h('th', { class: 'r' }, 'Finished') : null, h('th', { class: 'r' }, 'Watch time'))),
        h('tbody', null, sn.episodes.map((e) => h('tr', null,
          h('td', { class: 'mono muted ep-num' }, e.episode_number != null ? String(e.episode_number) : '–'),
          h('td', null, h('a', { href: `/items/${e.id}` }, e.name)),
          withAudio ? h('td', { class: 'ep-langs', title: (e.audio_languages || []).map(languageName).join(', ') }, (e.audio_languages || []).map(languageName).join(' · ') || h('span', { class: 'muted' }, '–')) : null,
          h('td', { class: 'bucket-bar' }, h('span', { class: 'bucket-track' }, e.plays ? h('span', { class: 'bucket-fill', style: { width: Math.max(2, (e.plays / max) * 100) + '%' } }) : null)),
          h('td', { class: 'mono r' }, e.plays ? num(e.plays) : h('span', { class: 'muted' }, '0')),
          withFinished ? h('td', { class: 'mono r', title: e.plays ? `${num(e.finished)} of ${num(e.plays)} plays reached 90% of the episode` : null }, e.plays ? pct(e.finished / e.plays) : h('span', { class: 'muted' }, '–')) : null,
          h('td', { class: 'mono r' }, e.watch_s ? duration(e.watch_s) : h('span', { class: 'muted' }, '–'))))))));
    if (i === 0 && list.length === 1) det.open = true;
    return det;
  }));
}

/** Open in Jellyfin, and the watchlist toggle for a film or a show; the row is left out when neither applies. */
function itemActions(it) {
  const open = it.jellyfin_link ? h('a', { class: 'btn btn-primary open-in-jellyfin', href: it.jellyfin_link, target: '_blank', rel: 'noopener noreferrer' },
    icon('play', 14), 'Open in Jellyfin') : null;
  const keep = watchable(it.type) ? watchToggle({ item_id: it.id }) : null;
  return open || keep ? h('p', { class: 'item-open' }, open, keep) : null;
}
