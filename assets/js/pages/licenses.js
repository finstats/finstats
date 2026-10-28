// Licences: FinStats' own, and every third-party licence the binary is built on, in full.
//
// One window, one licence: the one picked from the lists beside it. All 181 texts laid out at
// once made the page eighty thousand pixels tall, which is a wall rather than a document, and
// nobody reads a licence by scrolling past a hundred others. Nothing folds all the same: the
// window is always open, showing something, and a crate that carries several files switches
// between them in place.

import { h, icon, mount } from '../dom.js';
import { api } from '../api.js';
import { pageHeader, card, dataView, sk, emptyState } from '../components.js';
import { dataTable, plainTable } from '../tables.js';
import { button } from '../../finui/components/button/button.js';
import { t } from '../i18n.js';

const calm = () => typeof matchMedia === 'function' && matchMedia('(prefers-reduced-motion: reduce)').matches;

/** The host of a repository address, which is all of it worth reading in a table. */
function host(url) {
  try { return new URL(url).host.replace(/^www\./, ''); } catch { return url; }
}

function source(url) {
  if (!url) return h('span', { class: 'muted' }, '–');
  return h('a', { class: 'lic-src', href: url, target: '_blank', rel: 'noopener noreferrer' }, host(url), icon('external', 11),
    h('span', { class: 'sr-only' }, ` ${t('pages.licenses.new_tab')}`));
}

/** The window. `show(component, text)` is the only way anything gets into it. */
function viewer(notices, components) {
  const name = h('span', { class: 'lic-view-name' });
  const version = h('span', { class: 'mono lic-view-ver' });
  const sub = h('span');
  const tabs = h('div', { class: 'lic-tabs' });
  const shared = h('p', { class: 'fui-field__help lic-shared' });
  const body = h('pre', { class: 'lic-body', tabIndex: 0, 'aria-label': t('pages.licenses.text_aria') });
  const el = card({ title: [name, version], sub, actions: tabs, body: [shared, body], id: 'lic-view', cls: 'lic-view' });

  function show(c, at = null) {
    if (!c) return;
    const carried = c.notices || [];
    const pick = at != null ? at : carried[0];
    name.textContent = c.name;
    version.textContent = c.version || '';
    mount(sub, c.license || t('pages.licenses.none_stated'), c.repository ? [' · ', source(c.repository)] : null);
    // A crate under "MIT OR Apache-2.0" ships both files; the window switches between them.
    mount(tabs, carried.length > 1 ? carried.map((i) => h('button', { type: 'button', class: ['lic-tab', i === pick && 'is-on'], 'aria-pressed': String(i === pick), onClick: () => show(c, i) }, notices[i].file)) : null);
    const notice = pick != null ? notices[pick] : null;
    if (!notice) {
      mount(shared, '');
      body.textContent = t('pages.licenses.no_file', { name: c.name });
      return;
    }
    // Hundreds of crates ship the same MIT wording, so say whose text this also is.
    const others = components.reduce((n, x) => n + (x !== c && (x.notices || []).includes(pick) ? 1 : 0), 0);
    mount(shared, notice.file, others ? ` · ${t('pages.licenses.shared', { count: others })}` : '');
    body.textContent = notice.text;
    body.scrollTop = 0;
  }
  return { el, show };
}

/** The licence cell: the expression, and the way into the window when there is a text to show. */
function licenceCell(c, go) {
  if (!c.notices || !c.notices.length) return h('span', null, c.license || '–');
  return h('button', { type: 'button', class: 'lic-jump', onClick: () => go(c) }, c.license || t('pages.licenses.see_text'), icon('chevronRight', 12));
}

/** The index beside the window. The long crate list leaves out the source column: 253 rows of
 *  "github.com" cost the width the licence expressions need, and the window gives the link for
 *  whichever one is being read. */
function componentTable(rows, go, label) {
  const withSource = !label;
  const table = h('table', { class: 'fui-data-table fui-data-table--dense lic-table' },
    h('thead', null, h('tr', null,
      h('th', { scope: 'col' }, t('pages.licenses.col.component')),
      h('th', { scope: 'col' }, t('pages.licenses.col.version')),
      h('th', { scope: 'col' }, t('pages.licenses.col.licence')),
      withSource ? h('th', { scope: 'col' }, t('pages.licenses.col.source')) : null)),
    h('tbody', null, rows.map((c) => h('tr', null,
      h('th', { scope: 'row', class: 'lic-name' }, c.name),
      h('td', { class: 'mono lic-ver' }, c.version || '–'),
      h('td', null, licenceCell(c, go)),
      withSource ? h('td', null, source(c.repository)) : null))));
  return label ? dataTable(table, { label }) : plainTable(table);
}

// Shared with the prefetcher, so a prefetched view has exactly the address the page asks for.
const loadLicenses = (signal) => api.get('/licenses', null, { signal });
export const prefetchLicenses = ({ signal }) => [() => loadLicenses(signal)];

export default function licensesPage(ctx) {
  ctx.title(t('pages.licenses.title'));
  const view = h('div');
  ctx.root.append(pageHeader(t('pages.licenses.title'), t('pages.licenses.sub')), view);
  dataView({
    container: view, signal: ctx.signal,
    skeleton: () => [sk.cardBlock(140), sk.cardRows(5)],
    fetch: () => loadLicenses(ctx.signal),
    render: (d) => {
      const components = d.components || [];
      const notices = d.notices || [];
      if (!components.length) return emptyState(t('pages.licenses.empty'), t('pages.licenses.empty_sub'));

      const app = components.find((c) => c.kind === 'app');
      const bundled = components.filter((c) => c.kind === 'bundled');
      const crates = components.filter((c) => c.kind === 'crate');

      const window_ = viewer(notices, components);
      // Reading it is the point, so bring the window to the reader when it is out of sight. Beside
      // the lists on a wide screen it never is, and nothing moves.
      const go = (c) => {
        window_.show(c);
        const r = window_.el.getBoundingClientRect();
        if (r.top > innerHeight - 120 || r.bottom < 120) window_.el.scrollIntoView({ behavior: calm() ? 'auto' : 'smooth', block: 'start' });
      };
      window_.show(app || bundled[0] || crates[0]);

      return h('div', { class: 'lic-wrap' },
        h('div', { class: 'stack lic-main' },
          app ? card({ title: t('pages.licenses.app.title'), sub: `v${app.version} · ${app.license}`, id: 'lic-app', actions: source(app.repository),
            body: [h('p', { class: 'fui-field__help' }, t('pages.licenses.app.body')),
              app.notices.length ? button({ size: 'sm', type: 'button', onClick: () => go(app) }, icon('log', 14), t('pages.licenses.app.read')) : null] }) : null,
          bundled.length ? card({ title: t('pages.licenses.bundled.title'), sub: t('pages.licenses.bundled.sub'), id: 'lic-bundled',
            body: componentTable(bundled, go) }) : null,
          card({ title: t('pages.licenses.crates.title'), sub: t('pages.licenses.crates.sub', { count: crates.length }), id: 'lic-crates',
            body: componentTable(crates, go, t('pages.licenses.crates.find')) })),
        window_.el);
    },
  }).load();
}
