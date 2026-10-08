// The demo: FinStats with no server behind it. A classic script in <head>, so all of it is in place before the
// first module runs.
//
// Every answer the API gave while somebody walked an instance built from invented data is a file under
// data/api/<path>/, and data/api/<path>/_.json lists the query strings it was asked with. A GET is answered from
// there (the very same query, else the nearest one that names the same ids); anything else is refused, since
// there is nothing to change. The clock stands where it stood when the answers were taken, and moves on from there,
// so "last 30 days" and "2 minutes ago" read as they did. The site lives under a path (`<base href>`), so every
// address the app writes as `/x` is moved under it on its way into the page and out of it again for the router.
(() => {
  const DEMO = window.FINSTATS_DEMO;
  const BASE = new URL(document.baseURI).pathname.replace(/\/$/, '');   // "" or "/finstats"
  const inBase = (p) => BASE !== '' && (p === BASE || p.startsWith(BASE + '/'));
  /** The app's own address for a path of the site: "/finstats/items/x" → "/items/x". */
  const app = (p) => (inBase(p) ? p.slice(BASE.length) || '/' : p);
  /** The site's address for one of the app's: "/items/x" → "/finstats/items/x". */
  const site = (p) => (typeof p === 'string' && p.startsWith('/') && !p.startsWith('//') && !inBase(p) ? BASE + p : p);

  // ---- the clock: the moment the answers were taken, moving at the speed of the real one
  const RealDate = Date;
  const offset = DEMO.at - RealDate.now();
  class DemoDate extends RealDate {
    constructor(...a) { if (a.length) super(...a); else super(RealDate.now() + offset); }
    static now() { return RealDate.now() + offset; }
  }
  window.Date = DemoDate;
  const since = () => (RealDate.now() + offset - DEMO.at) / 1000;   // seconds the demo has been open

  // ---- pictures: synchronous, because an <img> is given its address in one go
  const IMG = window.FINSTATS_DEMO_IMG || {};
  function picture(path, search) {
    const got = IMG[path];
    const q = new URLSearchParams(search);
    q.delete('v');
    if (!got) return `${BASE}/data/img/none`;
    const exact = got[q.toString()];
    if (exact) return `${BASE}/data/img/${exact}`;
    if (!path.startsWith('/api/img/')) return `${BASE}/data/img/none`;   // a recap card is somebody's, not any
    // Another width of the same picture: the smallest one at least as wide, else the widest.
    const want = Number(q.get('w') || 0), kind = q.get('kind');
    const options = Object.entries(got).map(([k, f]) => { const p = new URLSearchParams(k); return { w: Number(p.get('w') || 0), kind: p.get('kind'), f }; })
      .filter((o) => o.kind === kind).sort((a, b) => a.w - b.w);
    const pick = options.find((o) => o.w >= want) || options[options.length - 1];
    return pick ? `${BASE}/data/img/${pick.f}` : `${BASE}/data/img/none`;
  }
  function rewrite(value) {
    if (typeof value !== 'string' || !value.startsWith('/') || value.startsWith('//') || inBase(value)) return value;
    if (value.startsWith('/api/')) { const u = new URL(value, location.origin); return picture(u.pathname, u.search); }
    return BASE + value;
  }
  for (const [Cls, prop] of [[HTMLAnchorElement, 'href'], [HTMLImageElement, 'src'], [HTMLLinkElement, 'href'], [HTMLSourceElement, 'src']]) {
    const d = Object.getOwnPropertyDescriptor(Cls.prototype, prop);
    Object.defineProperty(Cls.prototype, prop, { ...d, set(v) { d.set.call(this, rewrite(v)); } });
  }
  const setAttribute = Element.prototype.setAttribute;
  Element.prototype.setAttribute = function (name, value) {
    const n = String(name).toLowerCase();
    return setAttribute.call(this, name, n === 'href' || n === 'src' || n === 'xlink:href' ? rewrite(value) : value);
  };
  for (const m of ['pushState', 'replaceState']) {
    const real = history[m].bind(history);
    history[m] = (state, title, url) => real(state, title, url == null ? url : site(String(url)));
  }

  // ---- the API
  const realFetch = window.fetch.bind(window);
  const json = (status, body) => new Response(JSON.stringify(body), { status, headers: { 'Content-Type': 'application/json' } });
  const REFUSED = 'This is a demo with invented data, so nothing here can be changed. Run FinStats on your own server to try it.';
  const lists = new Map();
  const listOf = (path) => {
    if (!lists.has(path)) lists.set(path, realFetch(`${BASE}/data${path}/_.json`).then((r) => (r.ok ? r.json() : {})).catch(() => ({})));
    return lists.get(path);
  };

  // The nearest query string the answer was taken for. Ids must be the same ones (another title's plays are not
  // these); everything else counts one point per parameter that agrees.
  function nearest(queries, search) {
    const want = new URLSearchParams(search);
    want.delete('v');
    const key = want.toString();
    if (key in queries) return queries[key];
    let best = null, score = -Infinity;
    for (const [k, file] of Object.entries(queries)) {
      const have = new URLSearchParams(k);
      let s = 0, ok = true;
      const keys = new Set([...want.keys(), ...have.keys()]);
      for (const p of keys) {
        const a = want.get(p), b = have.get(p);
        if (a === b) s += 2;
        else if (/(^|_)ids?$|^q$|^year$|^scope$/.test(p)) { ok = false; break; }
        else if (a == null || b == null) s -= 1;
        else s -= 0.5;
      }
      if (ok && s > score) { score = s; best = file; }
    }
    return best;
  }

  const norm = (s) => String(s || '').normalize('NFD').replace(/[\u0300-\u036f]/g, '').toLowerCase().replace(/[^\p{L}\p{N}.:]+/gu, ' ').trim();
  const wordsOf = (q) => norm(q).split(' ').filter(Boolean);
  /** Every word somewhere in one of the fields, as the server's word-by-word search has it. */
  const matches = (words, fields) => { const text = norm(fields.filter((x) => x != null).join(' ')); return words.every((w) => text.includes(w)); };

  const TYPES = ['Movie', 'Series', 'MusicAlbum', 'Audio'];
  let corpus = null;
  async function search(params) {
    corpus = corpus || realFetch(`${BASE}/data/search.json`).then((r) => r.json());
    const all = await corpus;
    const words = wordsOf(params.get('q'));
    const limit = Number(params.get('limit') || 12);
    const hits = (list, name) => (words.length ? list.filter((x) => { const t = norm(name(x)).split(' '); return words.every((w) => t.some((x2) => x2.startsWith(w))); }) : []).slice(0, limit);
    return { items: hits(all.items.filter((i) => TYPES.includes(i.type)), (i) => i.name), people: hits(all.people, (p) => p.name), users: hits(all.users, (u) => u.name) };
  }

  // Plays that are running keep running: their position moves on with the clock, round to the start at the end.
  function playing(body) {
    const s = since();
    for (const x of body.sessions || []) {
      if (x.is_paused || !x.runtime_s) continue;
      x.position_s = Math.floor((x.position_s + s) % x.runtime_s);
      if (x.watched_s != null) x.watched_s = Math.floor(x.watched_s + s);
    }
    return body;
  }

  // ---- Activity: every play is in data/activity.json, so its filters, sorts and pages all work here, as the
  // server's (stats.rs, `activity`) do. `days` holds which plays each range took in, as the server counted them.
  let history_ = null;
  const CHARTED = ['Movie', 'Episode', 'Audio'];
  const SORTS = {
    when: (p) => p.ended_at, user: (p) => (p.user_name || '').toLowerCase(), title: (p) => (p.series_name || p.item_name || '').toLowerCase(),
    watched: (p) => p.duration_s, progress: (p) => p.completion, client: (p) => (p.client || '').toLowerCase(), method: (p) => p.play_method, ip: (p) => p.remote_ip,
  };
  async function activity(params) {
    history_ = history_ || realFetch(`${BASE}/data/activity.json`).then((r) => r.json()).then((h) => ({ ...h, sets: Object.fromEntries(Object.entries(h.days).map(([d, ids]) => [d, new Set(ids)])) }));
    const h = await history_;
    const many = (k) => (params.get(k) || '').split(',').map((x) => x.trim()).filter(Boolean);
    const days = params.get('days') || '30', inRange = h.sets[days];
    const methods = many('method'), types = many('type'), sources = many('source'), words = wordsOf(params.get('q'));
    const [item, series, user] = ['item_id', 'series_id', 'user_id'].map((k) => params.get(k) || '');
    const deleted = params.get('deleted') === '1';
    let rows = deleted ? [] : h.rows.filter((p) => (!inRange || inRange.has(p.id))
      && (!user || p.user_id === user) && (!item || p.item_id === item) && (!series || p.series_id === series)
      && (!methods.length || methods.includes(p.play_method)) && (!sources.length || sources.includes(p.source))
      && (!types.length || types.includes(p.item_type) || (types.includes('Other') && !CHARTED.includes(p.item_type)))
      && (!words.length || matches(words, [p.item_name, p.series_name, p.user_name, p.client, p.device_name, p.remote_ip])));
    const key = SORTS[params.get('sort')], asc = params.get('dir') === 'asc';
    const cmp = (a, b) => (a < b ? -1 : a > b ? 1 : 0);
    rows = rows.slice().sort((a, b) => {
      if (key) {
        const x = key(a), y = key(b);
        if ((x == null) !== (y == null)) return x == null ? 1 : -1;
        const c = cmp(x, y) * (asc ? 1 : -1);
        if (c) return c;
      }
      return cmp(b.ended_at, a.ended_at) || b.id - a.id;
    });
    const page = Math.max(1, Number(params.get('page')) || 1), per = Math.min(200, Math.max(1, Number(params.get('per_page')) || 50));
    return { total: rows.length, page, per_page: per, rows: rows.slice((page - 1) * per, page * per), sources: h.sources, in_trash: 0 };
  }

  // A list searched with words nobody asked for while the answers were taken: the same list without them, narrowed here.
  const LISTS = ['items', 'rows', 'events', 'titles', 'entries'];
  function narrowed(body, words) {
    for (const k of LISTS) {
      if (!Array.isArray(body[k])) continue;
      const before = body[k].length;
      body[k] = body[k].filter((x) => matches(words, Object.values(x).filter((v) => typeof v === 'string')));
      if (typeof body.total === 'number') body.total -= before - body[k].length;
    }
    return body;
  }

  async function answer(method, path, search_) {
    if (method !== 'GET' && method !== 'HEAD') return json(403, { error: REFUSED });
    const params = new URLSearchParams(search_);
    if (path === '/api/search') return json(200, await search(params));
    if (path === '/api/activity') return json(200, await activity(params));
    const queries = await listOf(path);
    let file = nearest(queries, search_), words = [];
    if (!file && params.get('q')) {
      words = wordsOf(params.get('q'));
      params.delete('q');
      file = nearest(queries, params.toString());
    }
    if (!file) return json(404, { error: 'The demo has no answer for this.' });
    const r = await realFetch(`${BASE}/data${path}/${file}`);
    if (!r.ok) return json(404, { error: 'The demo has no answer for this.' });
    let body = await r.json();
    if (words.length) body = narrowed(body, words);
    if (path === '/api/now-playing') body = playing(body);
    return json(200, body);
  }

  window.fetch = (input, init = {}) => {
    const url = new URL(typeof input === 'string' || input instanceof URL ? String(input) : input.url, location.href);
    if (url.origin !== location.origin || inBase(url.pathname)) return realFetch(input, init);
    const method = String(init.method || (input instanceof Request ? input.method : 'GET')).toUpperCase();
    if (url.pathname.startsWith('/api/')) {
      if (init.signal && init.signal.aborted) return Promise.reject(new DOMException('Aborted', 'AbortError'));
      return answer(method, url.pathname, url.search);
    }
    return realFetch(BASE + url.pathname + url.search, init);
  };
  window.finstatsDemo = { app, site, base: BASE };
})();
