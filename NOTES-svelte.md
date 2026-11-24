# Notes: a SvelteKit version of SPEC-01

The plain-HTML gallery proves the skin works without React. A Svelte version
would add the real ui-sdk behaviours (focus management, keyboard navigation,
popups, typeahead) instead of hand-written ones.

## What's verified (19 Nov 2026)

- `@ui-sdk/svelte` exists on npm: `0.1.0-beta.13`, peer deps `svelte >=5.33` and
  `@ui-sdk/core ^0.1.0-beta.13`.
- SPEC-01 itself pins a **snapshot** build: `@ui-sdk/core` / `@ui-sdk/react`
  `0.0.0-20261111125312.ga16c1d57`. Part names and attributes may differ slightly
  from `0.1.0-beta.13`. Compare the rendered attributes against `index.html` here.
- The skin is plain CSS with no Tailwind. `lib/*.ts` has no framework imports.
- The registry `.tsx` files only import `@ui-sdk/react/<part>` and local helpers.

## Corrections to the Gemini notes

- The key colour variables are **`--theme-l`, `--theme-c`, `--theme-h`**, not
  `--key-l/c/h`. `tokens.css` derives `--key-color`, `--key-ink` and the rest from them.
- The `<Hardware>` root must also call `installHardware()` (from `lib/install.ts`)
  once on mount. Without it there are no haptics, sounds, focus rings or block caret.
- The stylesheet to import is `ui/hardware/skin.css`, which `@import`s the other
  28 files in order. Copy the whole `ui/hardware/` folder, not three files.
- The font is "Spec Mono", a subset of IBM Plex Mono shipped as woff2 in
  `public/fonts/` with `@font-face` rules in the site's `index.html` (copied here
  as `vendor/spec-01/css/fonts.css`). Google Fonts' IBM Plex Mono is the fallback in `--sans`.

## Plan

1. **Scaffold** `npx sv create` (Svelte 5, TypeScript). Then
   `npm i @ui-sdk/core @ui-sdk/svelte`. Optionally `npx skills add vercel-labs/ui --skill ui-sdk`
   for agent docs (not checked).
2. **Reuse this repo's vendor folder**: `vendor/spec-01/css` → `src/lib/hardware/css`,
   fonts → `static/fonts`, and `lib-src/*.ts` → `src/lib/hardware/lib/` (fix the
   `@/registry/hardware/lib/` import paths).
3. **Hardware.svelte**:
   ```svelte
   <script lang="ts">
     import { onMount } from "svelte"
     import "$lib/hardware/css/skin.css"
     import { installHardware } from "$lib/hardware/lib/install"
     let { keyColor, children, ...rest } = $props()
     onMount(installHardware)
   </script>

   <div
     data-skin="hardware"
     style:--theme-l={keyColor?.l}
     style:--theme-c={keyColor?.c}
     style:--theme-h={keyColor?.h}
     {...rest}
   >
     {@render children?.()}
   </div>
   ```
4. **Port components one at a time** from `.local/spec-01-src/registry/hardware/ui/*.tsx`:
   - Swap `@ui-sdk/react/<part>` for the Svelte adapter's equivalent. Check the
     adapter's actual API first; I haven't confirmed it mirrors the React part names.
   - Keep every `data-kit` and `data-tone` / `data-shape` / `data-color` attribute
     exactly. The skin depends on them.
   - Move each component's React state (drag refs, `--turn`, `--at`, `--i`, wheel
     paint) to `$state` / `$effect`. `js/gallery.js` already has vanilla versions
     of knob, slider, mode dial, slide switch and select wheel to crib from.
   - Check each port against `index.html`: the rendered DOM should match.
5. **Start with** Button, Switch, Slider, Mode dial and Select, then popups
   (Menu, Dialog, Popover), which gain the most from ui-sdk.
6. **Then the mini apps** in `registry/hardware/blocks/` (timer is the smallest).

## Open questions

- Does `@ui-sdk/svelte` run SSR-safe in SvelteKit? The `lib/` helpers guard
  `document`, but check the adapter.
- Should this live in this repo (e.g. `svelte/`) or in its own repo?
