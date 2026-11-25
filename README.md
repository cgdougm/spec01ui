# spec01ui

A plain HTML gallery of [SPEC-01/HARDWARE](https://spec-01.vercel.app), Shu Ding's
hardware skin for [Vercel UI SDK](https://ui-sdk.dev), running with **no React and
no build step**.

## Run it

ES modules need a web server (opening `index.html` from disk won't load them):

```sh
python -m http.server 5610
# then open http://localhost:5610
```

Any static server works (`npx serve`, VS Code Live Server, …).

## How it works

SPEC-01's look is **plain CSS** scoped to `[data-skin="hardware"]`. It styles
`data-kit="…"` hooks and the state attributes ui-sdk writes (`data-checked`,
`data-pressed`, `data-selected`, `data-open`, `aria-*`). React only produces
that markup. So the gallery:

1. **Copies the stylesheets unchanged** into `vendor/spec-01/css/hardware/`.
   `skin.css` pulls in the rest.
2. **Uses the markup SPEC-01 actually renders.** `index.html` is generated from a
   DOM snapshot of the live site (`tools/extract-gallery.py`), so all 52
   components look exactly like the original.
3. **Adds a small vanilla layer** (`js/gallery.js`) that flips the same attributes
   and CSS variables ui-sdk would. Controls with a lit dot by their name are wired up.
4. **Runs SPEC-01's page-wide behaviours as-is.** Haptics, synthesized Web Audio
   sounds, keyboard-only focus rings, the block caret and the Safari Tab fix are
   transpiled from `lib/*.ts` to `vendor/spec-01/lib/*.js` and started with
   `installHardware()`. They listen on the document, so they work on any markup.

### Key colour

The key colour is an OKLCH triple on the root:

```html
<div data-skin="hardware" style="--theme-l: 0.64; --theme-c: 0.31; --theme-h: 48">
```

Everything else (`--key-ink`, `--key-lamp`, `--key-glow`, …) is derived in
`tokens.css`. Leave the variables off and the keys are black.

### Wired up so far

Button, Switch, Checkbox, Segmented control, Slider, Flat knob, Select (scroll
wheel), Tabs, Radio group, Number stepper, Accordion, Toolbar, Toggle group, Mode
dial and Slide switch. Text inputs (text field, textarea, combobox, date picker
input) also work. The key picker and sound toggle in the navbar work, and the
key choice is remembered.

Static for now: popups (menu, dialog, popover, tooltip, select lists, combobox
list, command, drawer, toast, hover card, context menu), calendar, pagination
wheel, carousel, resizable, D-pad, OTP and similar. Their markup and styling are
complete, so each one only needs its behaviour added in `gallery.js`.

## Layout

```
index.html                 generated gallery page (edit by hand freely, or regenerate)
css/gallery.css            gallery-only additions (the "live" LED)
js/gallery.js              vanilla behaviours
vendor/spec-01/            MIT-licensed files from shuding/spec-01 (see LICENSE there)
  css/hardware/*.css       the skin, unchanged
  css/base.css, site.css   the site's page styles, unchanged
  css/fonts.css            Spec Mono @font-face (IBM Plex Mono subset, OFL)
  fonts/                   Spec Mono woff2 files
  lib/*.js                 page-wide behaviours, transpiled from lib-src/*.ts
  lib-src/*.ts             original sources (with comments) for reference
  photos/                  carousel photos (Unsplash License, see PHOTOS.md)
tools/extract-gallery.py   regenerates index.html from a DOM snapshot
NOTES-svelte.md            plan for a SvelteKit version
desktop/                   native Rust + gpui port of five components (see desktop/README.md)
.local/                    untracked: spec-01 clone and DOM snapshot
```

## Regenerating from the live site

```sh
msedge --headless=new --disable-gpu --virtual-time-budget=15000 \
  --window-size=1400,1000 --dump-dom https://spec-01.vercel.app > .local/spec-01-dom.html
python tools/extract-gallery.py .local/spec-01-dom.html index.html
```

(`chrome` works the same as `msedge`.) Vendored files came from `shuding/spec-01`
at commit `b8f91b6` (14 Nov 2026).

## Credits

SPEC-01/HARDWARE © Shu Ding, MIT License (`vendor/spec-01/LICENSE`).
