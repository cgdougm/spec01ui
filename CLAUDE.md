# CLAUDE.md

Guide for coding agents working in this repo.

## What this is

A study of **SPEC-01/HARDWARE** (https://spec-01.vercel.app, by Shu Ding, MIT), a
React theme for Vercel UI SDK that draws controls as synthesizer hardware. The
owner wants this look **without React**, and ultimately as a **fast native desktop
app for Windows and Linux**.

There are two independent halves:

| Path | What | Stack |
|---|---|---|
| `index.html`, `js/`, `css/`, `vendor/spec-01/` | Web gallery of all 52 SPEC-01 components, published on GitHub Pages (https://cgdougm.github.io/spec01ui/) | Plain HTML, CSS and vanilla JS with no build step; the original CSS vendored unchanged |
| `desktop/` | Native port of 23 components, plus a stress test | Rust + [gpui](https://gpui.rs) 0.2.2 (Zed's GPU UI framework) |

Owner preferences:
- **Avoid React.** Use vanilla web code or native Rust.
- **No build step for the web half.**
- Commit and push only when asked.

## Desktop app (`desktop/`): build and run

```sh
# Ubuntu/Debian system deps for gpui (X11 + Wayland, Vulkan renderer)
sudo apt install gcc g++ pkg-config libxkbcommon-x11-dev libxkbcommon-dev libxcb1-dev \
  libx11-xcb-dev libwayland-dev libfontconfig-dev libfreetype6-dev libzstd-dev \
  libvulkan1 mesa-vulkan-drivers
# Rust stable (if missing)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

cd desktop
cargo build --release            # first build compiles gpui: several minutes
./target/release/spec01          # 5-component gallery
./target/release/stress          # the stress test wall (see below)
```

Fonts (IBM Plex Mono) and the two photos are embedded, so the binaries are
self-contained.

### Linux status

**Developed and measured on Windows only.** The first Linux build hasn't happened
yet. If it fails, likely causes:
- missing `-dev` packages (read the linker or pkg-config error)
- no Vulkan driver (check with `vulkaninfo | head`, from the `vulkan-tools` package)
- Wayland vs X11 quirks (try `WAYLAND_DISPLAY= ./target/release/stress` to force X11)

`build.rs` only adds Windows linker flags. On Linux it does nothing.

### CI

`.github/workflows/desktop.yml` builds both binaries on `windows-latest` and
`ubuntu-24.04` for every push or PR touching `desktop/`. It uploads them as
artifacts (`spec01-windows-x86_64.zip`, `spec01-linux-x86_64.tar.gz`) and
attaches them to a GitHub Release when a `v*` tag is pushed. Its apt list is
the reference for Linux build dependencies.

## Stress test: benchmarking across machines

```sh
./target/release/stress --suite bench/<label>.jsonl --label <label>
python3 tools/compare-bench.py bench/*.jsonl
```

- `--suite` runs 8 fixed scenarios for about 7 s each, about a minute in all. It
  appends one JSON line per scenario: FPS, p50/p95/p99 frame ms, render-build ms,
  widgets on screen, OS, CPU count, display scale and viewport.
- **Keep the window in front and don't touch it while it runs.** gpui only draws
  when the window is visible.
- Commit the `.jsonl` to `desktop/bench/`. `bench/windows-rtx2060.jsonl` is the
  Windows baseline (RTX 2060 laptop, 60 Hz 4K panel at 150%).
- FPS is capped at the display's refresh rate. Compare p95 frame ms, and FPS
  below the cap.

Other flags: `--widgets N`, `--zoom Z`, `--layout mixed|grouped`,
`--motion off|noise|wave`, `--hue`, `--scroll`, `--no-cull`, `--kind NAME` (a wall
of one kind), `--bench SECS` (one scenario; prints one line and quits).
Diagnostics: the env vars `SPEC01_NO_SHADOWS=1` and `SPEC01_NO_PATHS=1`.

Keys in the app:
- Ctrl+= / Ctrl+- / Ctrl+0 zoom (Ctrl+wheel too)
- PgUp/PgDn/Home/End/arrows scroll
- M cycles motion, H toggles hue, S toggles auto-scroll, L toggles layout

## Code map (`desktop/src/`)

- `lib.rs`: `load_fonts`, `rack_view` (the gallery's 5-module layout), and GPU
  hint statics.
- `theme.rs`: OKLCH key colour to sRGB. `Theme` holds the same derived tokens as
  `vendor/spec-01/css/hardware/tokens.css` (ink, lamp, cap, on_accent).
- `ui.rs`: the shared toolkit:
  - **`z(v)`:** every component size goes through this (zoom). App chrome uses
    plain `px`.
  - `sh` builds shadows, and `fx` trims shadow stacks by level of detail (`lod()`
    from zoom).
  - `module` draws the panel and caption. `label`.
  - Shared pieces: `key_cap`, `lcd_glass`, `dark_glass`.
  - `cells` draws grids as GPU quads, not elements. `Rays` draws batched radial
    strokes.
  - `greeked`/`greek`: text too small to read becomes a bar.
  - `now()` is the frame clock.
- `rack.rs`: `Rack` holds the state of one instance of *every* component.
  `drive(v, t)` sets every control from one value (motion modes), and `ease`
  animates. `Host` is the trait a view implements so component event handlers
  can reach a rack by index.
- `widgets/mod.rs`: `Kind`, the catalogue of 23 kinds (name, width 238/492,
  height), and `Kind::render`.
- `widgets/{button,switch,slider,knob,mode_dial}.rs`: the original five.
- `widgets/{keys,screens,controls}.rs`: the other 18.
- `main.rs`: the gallery binary. `bin/stress.rs`: the stress wall, which has its
  own packing, virtualization (only on-screen rows are built), scrollbar, HUD,
  bench and suite.

## gpui 0.2 gotchas (learned the hard way)

- `Window::request_animation_frame()` may only be called during render or paint.
  **Calling it in an event handler panics**, and the panic aborts a release
  build. Call `cx.notify()` instead and let `render` request frames.
- A `BoxShadow` with blur 0 draws nothing, so `ui::sh` enforces at least 0.5.
  A shadow's spread doesn't round its corners further.
- There are no inset shadows, no radial or conic gradients, linear gradients
  take 2 stops only, and there's no letter-spacing, no element rotation and no
  `corner-shape`. The README in `desktop/` lists the workarounds used.
- `img()` draws at the image's natural size unless given an explicit `w`/`h`.
  Clip with `overflow_hidden()` on the parent.
- On Windows gpui uses the *first* DXGI adapter, which on hybrid laptops is the
  iGPU. `build.rs` exports `NvOptimusEnablement` / `AmdPowerXpressRequestHighPerformance`
  so drivers pick the discrete GPU. That alone made frames about 40% faster here.
- gpui is immediate-mode: every visible element is rebuilt, laid out and painted
  every frame. Cost is roughly linear in elements on screen (about 0.1–0.6 ms per
  widget on the Windows machine). Prefer canvases or quads for repetitive marks,
  and keep huge views virtualized.

## Web gallery notes

- `index.html` is **generated** from a DOM snapshot of the live site by
  `tools/extract-gallery.py`. Regenerate instead of hand-editing the module
  markup.
- `js/gallery.js` flips the same attributes ui-sdk would. Haptics and sounds come
  from SPEC-01's own `lib/*.ts`, transpiled to `vendor/spec-01/lib/*.js`.
- `.local/` (git-ignored) holds the spec-01 clone and snapshots.
- `NOTES-svelte.md` is a plan for a SvelteKit port. Not started.
