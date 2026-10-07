# finstats demo

This branch is the demo site for [finstats](https://github.com/finstats/finstats), served by GitHub Pages from the
branch itself (Settings → Pages → *Deploy from a branch*, `demo`, `/ (root)`). No workflow builds it and there is no
server: it is plain HTML, CSS and JavaScript.

Everything in it is invented: the people (alex, maria, jonas, priya, sam, leo), the titles, the devices and the
addresses (`192.0.2.0/24`, `198.51.100.0/24` and `203.0.113.0/24` are reserved for documentation). It is the same
generated library finstats' own test suite runs against.

## How it works

- `index.html` and `assets/` are finstats' web app as it ships (2.2.0), with a few small changes for living under
  a path and without a server: in `router.js`, `shell.js`, `prefetch.js`, `menus.js`, `main.js` and `api.js` (the
  router and the code that reads links work in the app's own paths), and the word "Demo" in the status bar and a
  phone's top bar. `assets/finui.css` and `assets/finmotion.css` are the stylesheets finstats builds when asked.
- `data/api/<path>/` holds the answers a running finstats gave for that API path, one file per query string, and
  `_.json` lists them. `assets/js/demo.js` answers the app's requests from there: the very same query, or else the
  nearest one asked for the same ids. Anything that would change something is refused with a short message.
- The Activity list is answered in the browser from every play (`data/activity.json`), so its filters, sorting
  and pages all work. Search looks through `data/search.json`.
- The clock starts at the moment the answers were taken (`data/meta.js`) and runs on from there, so "last 30
  days", "2 minutes ago" and what is playing now read as they did.
- `data/img/` holds the posters and portraits, and `data/img.js` says which address each one answers.
- `404.html` is a copy of `index.html`: GitHub Pages serves it for every address that is not a file, which is how
  `/finstats/items/…` opens straight onto a title.

The site expects to live at `/finstats/`. Served anywhere else (a fork, a custom domain), change `<base href>` in
both `index.html` and `404.html`; nothing else names the path.

This branch shares no history with `main` and is never merged into it.

## Licence

finstats is GPL-3.0-only; see `LICENSE`. The fonts in `assets/fonts` are OFL-1.1, with their licences beside them.
