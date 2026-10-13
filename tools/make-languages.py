#!/usr/bin/env python3
"""Draw the README's Languages picture: how far each language FinStats speaks has come.

Input:  the languages in `src/i18n.rs` (`LOCALES`, the source first, and `CHECKED`), each one's catalogue in
        `web/assets/i18n/<tag>.json`, and the Patch notes: `CHANGELOG.md` (the source) and `changelog/<tag>.md`.
Output: `.github/languages.svg`, a card drawn like a translation platform's status widget (a bar per language, the
        ones the maintainer checks first, in GitHub's light or dark as the reader's page is), and the README's
        Languages section between its two markers: the picture, said in words for whoever cannot see it.

    python3 tools/make-languages.py            # redraw both
    python3 tools/make-languages.py --check    # exit 1 when either is out of date (the QA static stage runs this)

Re-run it whenever a catalogue, a changelog translation or the list of languages changes.
"""
import json
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
README = os.path.join(ROOT, "README.md")
PICTURE = ".github/languages.svg"
START = "<!-- languages: tools/make-languages.py writes this; run it again when a language changes -->"
END = "<!-- /languages -->"
PLURAL_FORMS = {"zero", "one", "two", "few", "many", "other"}


def read(*parts):
    with open(os.path.join(ROOT, *parts), encoding="utf-8") as f:
        return f.read()


def const_list(source, name):
    """The tags of `pub const NAME: [&str; n] = [...]` in src/i18n.rs."""
    m = re.search(r"pub const " + name + r": \[&str; \d+\] = \[(.*?)\];", source, re.S)
    if not m:
        sys.exit(f"src/i18n.rs has no {name}")
    return re.findall(r'"([^"]+)"', m.group(1))


def texts(value, prefix="", out=None):
    """Every text of a catalogue by its dotted key; a plural is one text, an empty string none."""
    out = [] if out is None else out
    if isinstance(value, dict):
        if isinstance(value.get("other"), str) and set(value) <= PLURAL_FORMS:
            out.append(prefix)
            return out
        for k, v in value.items():
            if not prefix and k == "$meta":
                continue
            texts(v, f"{prefix}.{k}" if prefix else k, out)
    elif isinstance(value, str) and value.strip():
        out.append(prefix)
    return out


def releases(text):
    return {line[4:].split("]")[0] for line in text.splitlines() if line.startswith("## [")}


def status():
    """Each language's share of the source: its texts of the source's, and the Patch notes releases told in it."""
    source = read("src", "i18n.rs")
    locales, checked = const_list(source, "LOCALES"), set(const_list(source, "CHECKED"))
    catalogue = lambda tag: json.loads(read("web", "assets", "i18n", f"{tag}.json"))
    words = set(texts(catalogue(locales[0])))
    notes = releases(read("CHANGELOG.md"))
    langs = []
    for tag in locales:
        cat = catalogue(tag)
        have = set(texts(cat))
        if tag == locales[0]:
            told = len(notes)
        else:
            path = os.path.join(ROOT, "changelog", f"{tag}.md")
            told = len(releases(read(path)) & notes) if os.path.exists(path) else 0
        langs.append({"tag": tag, "name": cat["$meta"]["name"], "checked": tag in checked,
                      "share": share(len(words & have) + told, len(words) + len(notes))})
    return len(words), len(notes), langs


def share(part, whole):
    """A share, rounded down: a language is at 100 only when nothing is missing."""
    return part * 100 // whole if whole else 0


def picture(total, notes, langs):
    w, row, bar_x, bar_w = 600, 30, 268, 250
    groups = [
        ("Checked by the maintainer", [l for l in langs if l["checked"]]),
        ("Not checked by a native speaker: may hold mistakes, corrections welcome", [l for l in langs if not l["checked"]]),
    ]
    h = 70 + sum(34 + row * len(ls) for _, ls in groups) + 4
    font = "-apple-system, BlinkMacSystemFont, 'Segoe UI', 'Noto Sans', Helvetica, Arial, sans-serif"
    out = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}" role="img" aria-labelledby="title">',
        '<title id="title">Translation status of FinStats</title>',
        "<style>",
        ".card { fill: #ffffff; stroke: #d0d7de; }",
        f".title {{ fill: #1f2328; font: 600 16px {font}; }}",
        f".group {{ fill: #1f2328; font: 600 12.5px {font}; }}",
        f".sub, .tag {{ fill: #59636e; font: 12px {font}; }}",
        f".name {{ fill: #1f2328; font: 500 14px {font}; }}",
        f".share {{ fill: #1f2328; font: 600 14px {font}; }}",
        ".track { fill: #ece6fd; }",
        ".bar { fill: #8a5cf5; }",
        ".bar.unchecked { fill: #b39af8; }",
        "@media (prefers-color-scheme: dark) {",
        ".card { fill: #0d1117; stroke: #3d444d; }",
        ".title, .group, .name, .share { fill: #f0f6fc; }",
        ".sub, .tag { fill: #9198a1; }",
        ".track { fill: #2b2440; }",
        ".bar.unchecked { fill: #6f52c9; }",
        "}",
        "</style>",
        f'<rect class="card" x="0.5" y="0.5" width="{w - 1}" height="{h - 1}" rx="10"/>',
        '<text class="title" x="24" y="34">Translation status</text>',
        f'<text class="sub" x="24" y="54">{total:,} texts in the interface and {notes} Patch notes releases, counted from the catalogues</text>',
    ]
    y = 70
    for i, (heading, ls) in enumerate(groups):
        out.append(f'<text class="group" x="24" y="{y + 20}">{heading}</text>')
        y += 34
        for l in ls:
            pct = l["share"]
            cls = "bar" if i == 0 else "bar unchecked"
            out += [
                f'<text class="name" x="24" y="{y + 12}">{l["name"]}</text>',
                f'<text class="tag" x="196" y="{y + 12}">{l["tag"]}</text>',
                f'<rect class="track" x="{bar_x}" y="{y + 2}" width="{bar_w}" height="10" rx="5"/>',
                f'<rect class="{cls}" x="{bar_x}" y="{y + 2}" width="{bar_w * pct // 100}" height="10" rx="5"/>',
                f'<text class="share" x="{w - 24}" y="{y + 12}" text-anchor="end">{pct}%</text>',
            ]
            y += row
    return "\n".join(out) + "\n</svg>\n"


def section(langs):
    said = ", ".join(f'{l["name"]} {l["share"]}%' for l in langs)
    return f'{START}\n<p align="center"><img src="{PICTURE}" width="600" alt="Translation status: {said}"></p>\n{END}'


def main():
    check = "--check" in sys.argv[1:]
    total, notes, langs = status()
    drawn, want = picture(total, notes, langs), section(langs)
    readme = read("README.md")
    m = re.search(r"<!-- languages:.*?-->.*?" + re.escape(END), readme, re.S)
    if not m:
        sys.exit(f"README.md has no Languages section: put {START} and {END} where it goes")
    path = os.path.join(ROOT, PICTURE)
    on_disk = read(PICTURE) if os.path.exists(path) else ""
    stale = [name for name, now, new in (("README.md", m.group(0), want), (PICTURE, on_disk, drawn)) if now != new]
    if check:
        if stale:
            sys.exit(f"out of date: {', '.join(stale)}. Run python3 tools/make-languages.py")
        return
    with open(README, "w", encoding="utf-8") as f:
        f.write(readme[:m.start()] + want + readme[m.end():])
    with open(path, "w", encoding="utf-8") as f:
        f.write(drawn)
    print(f"{len(langs)} languages drawn{': ' + ', '.join(stale) + ' rewritten' if stale else ', nothing changed'}")


if __name__ == "__main__":
    main()
