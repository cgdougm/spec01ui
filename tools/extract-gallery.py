"""Build index.html from a DOM snapshot of https://spec-01.vercel.app.

The snapshot is the page after React has rendered, saved with headless Edge or Chrome:

    msedge --headless=new --disable-gpu --virtual-time-budget=15000 \
        --window-size=1400,1000 --dump-dom https://spec-01.vercel.app > dom.html

Then run:

    python tools/extract-gallery.py dom.html index.html

The markup stays as SPEC-01 renders it (data-kit / data-slot hooks and data-checked
state attributes), so the vendored skin.css styles it with no changes. The script
only removes React and ui-sdk leftovers and swaps in this project's header.
"""

import re
import sys

src, out = sys.argv[1], sys.argv[2]
h = open(src, encoding="utf8").read()

# ui-sdk injects a few base rules (hidden parts, toast region) at runtime. Keep them.
sdk_styles = "\n".join(re.findall(r'<style id="ui-sdk-[^"]*">.*?</style>', h, re.S))

# Key picker and sound toggle from the navbar.
picker_start = h.find('<div data-kit="key-picker"')
nav_start = h.find('<nav aria-label="Pages"')
picker = h[picker_start:nav_start]

# The module grid.
grid_start = h.find('<div data-kit="grid"')
grid_end = h.find("</main>", grid_start)
grid = h[grid_start:grid_end]


def clean(s: str) -> str:
    # Hidden form inputs that ui-sdk mirrors state into. Nothing here submits a form.
    s = re.sub(r'<input data-slot="(?:radio|switch|checkbox)-input"[^>]*>', "", s)
    # Captions open a hover card about the SDK chain. Drop the trigger wiring but
    # keep the <a>: the skin colors `[data-kit="caption"] span` (the number).
    s = re.sub(
        r'<a data-kit="chain-trigger"[^>]*>([^<]*)</a>',
        r'<a data-kit="chain-trigger">\1</a>',
        s,
    )
    # The carousel's photos (Unsplash License, via Lorem Picsum) are vendored.
    s = s.replace('src="/photos/', 'src="vendor/spec-01/photos/')
    return s


picker = clean(picker)
grid = clean(grid)

# Give each module an anchor id built from its name.
def add_id(m):
    attrs, label = m.group(1), m.group(2)
    slug = re.sub(r"[^a-z0-9]+", "-", label.lower()).strip("-")
    return f'<section id="{slug}" data-kit="module"{attrs} aria-label="{label}"'


grid = re.sub(r'<section data-kit="module"((?: [a-z-]+="[^"]*")*) aria-label="([^"]*)"', add_id, grid)

count = len(re.findall(r'data-kit="module"', grid))

page = f"""<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover">
<title>SPEC-01 Gallery</title>
<meta name="description" content="A plain HTML gallery of SPEC-01/HARDWARE: Shu Ding's hardware skin for Vercel UI SDK, running with no React and no build step.">
<meta name="theme-color" content="#eeeeec">
<link rel="icon" href="data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 32 32'%3E%3Ccircle cx='16' cy='16' r='16' fill='%23ff3b1f'/%3E%3C/svg%3E">
{sdk_styles}
<link rel="stylesheet" href="vendor/spec-01/css/fonts.css">
<link rel="stylesheet" href="vendor/spec-01/css/base.css">
<link rel="stylesheet" href="vendor/spec-01/css/hardware/skin.css">
<link rel="stylesheet" href="vendor/spec-01/css/site.css">
<link rel="stylesheet" href="css/gallery.css">
</head>
<body>
<div data-skin="hardware" data-key="orange" data-site="true" style="--theme-l: 0.64; --theme-c: 0.31; --theme-h: 48; --nav-h: 52px;">
<header data-kit="navbar">{picker}<nav aria-label="Pages" data-kit="navbar-links"><a href="./" data-kit="navbar-link" draggable="false" aria-current="page">Components</a><a href="https://spec-01.vercel.app" data-kit="navbar-link" draggable="false" target="_blank" rel="noopener">Original</a><a href="https://github.com/shuding/spec-01" data-kit="navbar-link" draggable="false" target="_blank" rel="noopener">Source</a></nav></header>
<main data-site-page="components" data-kit="sheet">
<header data-kit="masthead">
<h1>SPEC-01/Hardware</h1>
<p><a href="https://github.com/shuding/spec-01" target="_blank" rel="noopener">SPEC-01</a>, Shu Ding's hardware skin for <a href="https://ui-sdk.dev" target="_blank" rel="noopener">Vercel UI SDK</a>, running as plain HTML: the original stylesheets, the markup the components render, and a little vanilla JavaScript. No React, no build step.</p>
<p data-gallery-legend>Controls with a lit dot <span data-gallery-dot aria-hidden="true"></span> by their name respond to input. The rest are static markup for now.</p>
</header>
{grid}
</main>
</div>
<script type="module" src="js/gallery.js"></script>
</body>
</html>
"""

open(out, "w", encoding="utf8", newline="\n").write(page)
print(f"wrote {out}: {count} modules, {len(page)//1024} KB")
