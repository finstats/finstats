// One actor or director: what they are in on this server, and how much of it has been watched.

import { h, num, duration, durationExact, durEl, relEl, compact } from '../dom.js';
import { api } from '../api.js';
import { readDays, saveDays, can } from '../state.js';
import { replaceQuery } from '../router.js';
import { card, filterBar, dataView, sk, poster, avatar, chip, statTile, emptyState } from '../components.js';
import { plainTable } from '../tables.js';
import { t, tParts } from '../i18n.js';

const typeLabel = (type) => (type === 'Movie' || type === 'Series' ? t(`item_type.${type}`) : type);

// Shared with the prefetcher, so a prefetched view has exactly the address the page asks for.
const loadPerson = (id, days, signal) => api.get(`/people/${id}`, { days }, { signal });
export const prefetchPerson = ({ params, query, signal }) => [() => loadPerson(params.id, readDays(query), signal)];

export default function personPage(ctx) {
  const id = ctx.params.id;
  ctx.title(t('pages.person.page_title'));
  let days = readDays(ctx.query);
  const view = h('div', { class: 'stack' });

  const dv = dataView({
    container: view, signal: ctx.signal,
    skeleton: () => [h('div', { class: 'item-hero' }, h('span', { class: 'fui-skeleton fui-skeleton--poster-lg' }), h('div', { class: 'fui-skeleton__lines' }, sk.line('40%', 28), sk.line('30%'))), sk.tiles(3), sk.cardBlock(320)],
    fetch: () => loadPerson(id, days, ctx.signal),
    render: (d) => {
      const p = d.person, tot = d.totals || {};
      ctx.title(p.name);
      const titles = d.titles || [];
      const watched = titles.filter((x) => x.plays > 0), unwatched = titles.filter((x) => !x.plays);
      return [
        h('div', { class: 'item-hero' },
          poster(p.has_image ? p.id : null, p.name, { w: 300, cls: 'fui-poster--lg' }),
          h('div', { class: 'item-hero-text' },
            h('h1', { class: 'fui-page-header__title' }, p.name),
            h('div', { class: 'fui-chip__set' }, p.is_actor ? chip(t('credit.actor')) : null, p.is_director ? chip(t('credit.director')) : null,
              chip(t('pages.person.titles_in_library', { count: p.titles }))))),
        h('div', { class: 'fui-stat-tile__grid fui-stat-tile__grid--three' },
          statTile({ label: t('terms.watch_time'), value: duration(tot.watch_s), title: durationExact(tot.watch_s), hint: t('pages.person.watch_time_hint') }),
          statTile({ label: t('pages.person.plays'), value: compact(tot.plays), title: num(tot.plays), hint: tot.last_played_at ? tParts('pages.person.last_when', { when: relEl(tot.last_played_at, '') }) : t('pages.person.never_played') }),
          statTile({ label: t('pages.person.titles_watched'), value: t('pages.person.of_total', { seen: tot.titles_watched, total: p.titles }), hint: ' ' })),
        card({ title: t('pages.person.watched'), sub: t('pages.person.watched_sub'), body: watched.length ? titleGrid(watched, true) : emptyState(t('pages.person.nothing_played')) }),
        unwatched.length ? card({ title: t('pages.person.also_in_library'), sub: t('pages.person.not_played'), body: titleGrid(unwatched, false) }) : null,
        can('see_everyone') && d.watchers && d.watchers.length ? card({ title: t('pages.person.watched_by'), cls: 'fui-card--flush', body: watchers(d.watchers) }) : null,
      ];
    },
  });

  ctx.root.append(filterBar({ days, signal: ctx.signal, onDays: (v) => { days = v; saveDays(v); replaceQuery({ days }); dv.load(); } }), view);
  dv.load();
}

function titleGrid(rows, withTime) {
  return h('ul', { class: 'fui-media-card__grid' }, rows.map((x) => {
    const credit = String(x.kinds || '').includes('Director') ? (String(x.kinds).includes('Actor') ? (x.role ? t('pages.person.director_as', { role: x.role }) : t('pages.person.director_actor')) : t('credit.director')) : x.role ? t('pages.person.as_role', { role: x.role }) : null;
    const inner = [
      poster(x.id, x.name, { w: 300, cls: 'fui-poster--grid' }),
      h('span', { class: 'fui-media-card__name' }, x.name),
      h('span', { class: 'fui-media-card__sub' }, [typeLabel(x.type), x.year, x.removed ? t('pages.person.not_in_library') : null].filter(Boolean).join(' · ')),
      credit ? h('span', { class: 'fui-media-card__sub' }, credit) : null,
      withTime ? h('span', { class: 'fui-media-card__sub mono', title: durationExact(x.watch_s) }, `${duration(x.watch_s)} · ${t('terms.play', { count: x.plays })}`) : null,
    ];
    return h('li', null, h('a', { class: 'fui-media-card', href: `/items/${x.id}` }, inner));
  }));
}

function watchers(rows) {
  return plainTable(h('table', { class: 'fui-data-table' },
    h('thead', null, h('tr', null, h('th', null, t('plays_table.user')), h('th', { class: 'r' }, t('pages.person.plays')), h('th', { class: 'r' }, t('terms.watch_time')), h('th', null, t('pages.person.last_played')))),
    h('tbody', null, rows.map((w) => h('tr', null,
      h('td', null, h('a', { class: 'user-cell', href: `/users/${w.user_id}` }, avatar(w.user_id, w.user_name, { size: 22 }), h('span', null, w.user_name))),
      h('td', { class: 'mono r' }, num(w.plays)), h('td', { class: 'r' }, durEl(w.watch_s)), h('td', null, relEl(w.last_played_at)))))));
}
