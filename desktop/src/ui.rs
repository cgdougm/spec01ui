//! Shared pieces: zoom, the frame clock, shadows, the module panel and its
//! caption, labels, keys, screens, the key picker and batched paths.
//!
//! Every size in the components goes through [`z`], which scales by the
//! current zoom, so zooming re-lays everything out rather than stretching a
//! picture. App chrome (navbar, HUD, scrollbar) stays at 1x with `px`.

use std::{cell::Cell, sync::OnceLock};

use gpui::{
    Bounds, BoxShadow, Context, Div, FontWeight, Hsla, InteractiveElement, IntoElement,
    MouseButton, ParentElement, PathBuilder, Pixels, Point, ScrollDelta, ScrollWheelEvent, Styled,
    Window, div, linear_color_stop, linear_gradient, point, px,
};

use crate::{
    rack::Host,
    theme::{self, KEYS, Theme},
};

pub const FONT: &str = "IBM Plex Mono";

thread_local! {
    static ZOOM: Cell<f32> = const { Cell::new(1.) };
    static TIME: Cell<f32> = const { Cell::new(0.) };
}

/// The current zoom factor (1 = the skin's own sizes).
pub fn zoom() -> f32 {
    ZOOM.with(|z| z.get())
}

pub fn set_zoom(z: f32) {
    ZOOM.with(|c| c.set(z));
}

/// A length in the skin's units, scaled by the zoom.
pub fn z(v: f32) -> Pixels {
    px(v * zoom())
}

/// Seconds since the app started, as of this frame: for blinking carets,
/// pulsing lamps and running clocks.
pub fn now() -> f32 {
    TIME.with(|t| t.get())
}

pub fn set_now(secs: f32) {
    TIME.with(|t| t.set(secs));
}

/// Diagnostics: `SPEC01_NO_SHADOWS=1` / `SPEC01_NO_PATHS=1` leave those out,
/// to see what they cost.
pub fn no_shadows() -> bool {
    static V: OnceLock<bool> = OnceLock::new();
    *V.get_or_init(|| std::env::var_os("SPEC01_NO_SHADOWS").is_some())
}

pub fn no_paths() -> bool {
    static V: OnceLock<bool> = OnceLock::new();
    *V.get_or_init(|| std::env::var_os("SPEC01_NO_PATHS").is_some())
}

/// Level of detail for the current zoom: 2 full, 1 reduced, 0 minimal.
/// Below about 60% the skin's stacked shadows and hairline knurling can't
/// be told apart, so fewer are drawn.
pub fn lod() -> u8 {
    let k = zoom();
    if k >= 0.6 { 2 } else if k >= 0.3 { 1 } else { 0 }
}

/// A shadow stack trimmed to the level of detail. Stacks list their most
/// telling shadows first (rings and lips, then soft drops), so trimming
/// keeps the shape and drops the subtlety.
pub fn fx(mut v: Vec<BoxShadow>) -> Vec<BoxShadow> {
    let keep = match lod() {
        2 => usize::MAX,
        1 => 2,
        _ => 1,
    };
    v.truncate(keep);
    v
}

/// A box shadow, written in CSS order: x, y, blur, spread, color; zoomed.
///
/// gpui draws nothing for a shadow with no blur at all, so the skin's crisp
/// lips and rings (blur 0 in CSS) get the smallest blur that still draws.
pub fn sh(x: f32, y: f32, blur: f32, spread: f32, color: Hsla) -> BoxShadow {
    let k = zoom();
    if no_shadows() {
        return BoxShadow { color: theme::rgba(0, 0, 0, 0.), offset: point(px(0.), px(0.)), blur_radius: px(0.), spread_radius: px(-10_000.) };
    }
    BoxShadow { color, offset: point(px(x * k), px(y * k)), blur_radius: px((blur * k).max(0.5)), spread_radius: px(spread * k) }
}

/// The same at 1x, for app chrome.
pub fn sh_px(x: f32, y: f32, blur: f32, spread: f32, color: Hsla) -> BoxShadow {
    BoxShadow { color, offset: point(px(x), px(y)), blur_radius: px(blur.max(0.5)), spread_radius: px(spread) }
}

/// The panel every component sits on: a light molded plate with a stacked
/// drop shadow, a caption with the component's number, name and a lit LED.
pub fn module(t: &Theme, number: &str, name: &str, width: f32, body: impl IntoElement) -> Div {
    div()
        .w(z(width))
        .flex()
        .flex_col()
        .pt(z(16.))
        .px(z(18.))
        .pb(z(20.))
        .rounded(z(26.))
        .bg(theme::panel())
        .shadow(if lod() == 2 {
            vec![
                sh(0., 0., 0., 1., theme::shade(0.055)),
                sh(0.5, 1., 1., 0., theme::shade(0.07)),
                sh(1., 2., 2., 0., theme::shade(0.07)),
                sh(2., 4., 4., 0., theme::shade(0.07)),
                sh(4., 8., 8., 0., theme::shade(0.07)),
                sh(8., 16., 16., 0., theme::shade(0.063)),
            ]
        } else {
            fx(vec![sh(2., 6., 10., 0., theme::shade(0.16)), sh(0., 0., 0., 1., theme::shade(0.055))])
        })
        .child(caption(t, number, name))
        .child(div().flex_1().flex().items_center().justify_center().child(body))
}

/// Text too small to read is "greeked": drawn as a grey bar the length
/// of the text, which costs no shaping or glyphs. True below ~4 px.
pub fn greeked(size: f32) -> bool {
    size * zoom() < 4.
}

/// A greeked stand-in for `chars` characters of text at `size`.
pub fn greek(chars: usize, size: f32, color: Hsla) -> Div {
    div().w(z(chars as f32 * size * 0.6)).h(z(size * 0.55)).my(z(size * 0.2)).rounded(z(size * 0.2)).bg(color.opacity(0.45))
}

fn caption(t: &Theme, number: &str, name: &str) -> Div {
    if greeked(10.) {
        return div()
            .flex()
            .items_center()
            .gap(z(8.))
            .pb(z(10.))
            .h(z(22.))
            .child(greek(number.len(), 10., t.ink.hsla()))
            .child(greek(name.len(), 10., theme::dim()))
            .child(div().flex_1().h(z(1.)).bg(theme::rgba(0, 0, 0, 0.07)));
    }
    div()
        .flex()
        .items_center()
        .gap(z(8.))
        .pb(z(10.))
        .text_size(z(10.))
        .line_height(z(12.))
        .font_weight(FontWeight::MEDIUM)
        .child(div().text_color(t.ink.hsla()).child(number.to_string()))
        .child(div().text_color(theme::dim()).child(name.to_uppercase()))
        .child(led(t, 6.))
        .child(div().flex_1().h(z(1.)).bg(theme::rgba(0, 0, 0, 0.07)))
}

/// A small lit lamp in the key color.
pub fn led(t: &Theme, size: f32) -> Div {
    div().size(z(size)).rounded_full().bg(t.lamp.hsla()).shadow(fx(vec![sh(0., 0., 6., 0., t.lamp.alpha(0.7))]))
}

/// A lamp that's lit or not.
pub fn lamp(t: &Theme, size: f32, on: bool) -> Div {
    if on { led(t, size) } else { div().size(z(size)).rounded_full().bg(theme::rgb(200, 200, 196)) }
}

/// The small uppercase printed label used on every control.
pub fn label(text: &str) -> Div {
    if greeked(8.) {
        return div().h(z(10.)).flex().items_center().child(greek(text.len(), 8., theme::text()));
    }
    div().text_size(z(8.)).line_height(z(10.)).font_weight(FontWeight::MEDIUM).text_color(theme::text()).child(text.to_uppercase())
}

/// A white key cap: raised on a 3px lip, or pressed down onto it.
pub fn key_cap(w: f32, h: f32, radius: f32, down: bool) -> Div {
    let travel = if down { 2. } else { 0. };
    let lip = 3. - travel;
    div()
        .w(z(w))
        .h(z(h))
        .mt(z(travel))
        .rounded(z(radius))
        .bg(linear_gradient(0., linear_color_stop(theme::rgb(253, 253, 252), 0.), linear_color_stop(theme::rgb(if down { 232 } else { 239 }, if down { 232 } else { 239 }, if down { 229 } else { 236 }), 0.7)))
        .shadow(fx(vec![
            sh(0., 0., 0., 1., theme::shade(0.07)),
            sh(0., lip, 0., 0., theme::rgb(220, 220, 216)),
            sh(0., lip + 0.5, 1., 0., theme::shade(0.118)),
            sh(0.5, 6. - travel * 2., 9., -3., theme::shade(0.082)),
        ]))
}

/// A recessed LCD: grey-green glass behind a dark rim.
pub fn lcd_glass(w: f32, h: f32) -> Div {
    div()
        .w(z(w))
        .h(z(h))
        .rounded(z(9.))
        .bg(linear_gradient(180., linear_color_stop(theme::rgb(190, 191, 185), 0.), linear_color_stop(theme::rgb(206, 206, 200), 0.5)))
        .border_1()
        .border_color(theme::rgba(0, 0, 0, 0.28))
        .shadow(fx(vec![sh(0., 1., 0., 0., theme::rgb(255, 255, 255)), sh(0., 0., 0., 2., theme::rgba(0, 0, 0, 0.04))]))
}

/// A black screen.
pub fn dark_glass(w: f32, h: f32) -> Div {
    div()
        .w(z(w))
        .h(z(h))
        .rounded(z(12.))
        .bg(linear_gradient(180., linear_color_stop(theme::rgb(30, 30, 29), 0.), linear_color_stop(theme::rgb(18, 18, 17), 1.)))
        .shadow(fx(vec![sh(0., 1., 0., 0., theme::rgb(255, 255, 255)), sh(0., 2., 6., 0., theme::shade(0.2))]))
}

/// One step per wheel notch, up positive. Shift+wheel may arrive as a
/// horizontal delta, so either axis counts.
pub fn scroll_steps(e: &ScrollWheelEvent) -> f32 {
    let (x, y) = match e.delta {
        ScrollDelta::Lines(p) => (p.x, p.y),
        ScrollDelta::Pixels(p) => (f32::from(p.x) / 20., f32::from(p.y) / 20.),
    };
    let v = if y != 0. { y } else { x };
    if v == 0. { 0. } else { v.signum() }
}

/// The six key-color swatches; the chosen one ringed.
pub fn key_picker<H: Host>(key: usize, cx: &mut Context<H>) -> Div {
    let swatches = KEYS.iter().enumerate().map(|(i, (_, k))| {
        let on = i == key;
        // Nested circles, as gpui keeps a shadow's corner radius as it spreads.
        let ring = |size: f32, color| div().size(px(size)).rounded_full().flex().items_center().justify_center().bg(color);
        let swatch = div().size(px(18.)).rounded_full().bg(k.hsla()).border_1().border_color(theme::rgba(0, 0, 0, 0.12));
        let clear = theme::rgba(0, 0, 0, 0.);
        ring(25., if on { theme::text() } else { clear })
            .child(ring(23., if on { theme::rgb(246, 246, 244) } else { clear }).child(swatch))
            .cursor_pointer()
            .on_mouse_down(MouseButton::Left, cx.listener(move |this: &mut H, _, _, cx| {
                this.set_key(i);
                cx.notify();
            }))
    });
    div().flex().items_center().gap(px(2.)).children(swatches)
}

/// The floating bar's look: a frosted pill.
pub fn bar_shell() -> Div {
    div()
        .rounded(px(26.))
        .bg(linear_gradient(180., linear_color_stop(theme::rgb(248, 248, 246), 0.), linear_color_stop(theme::rgb(242, 242, 240), 1.)))
        .border_1()
        .border_color(theme::rgba(255, 255, 255, 0.6))
        .shadow(fx(vec![
            sh_px(0., 0., 0., 1., theme::rgba(0, 0, 0, 0.07)),
            sh_px(0., 1., 2., 0., theme::shade(0.11)),
            sh_px(0., 12., 28., -12., theme::shade(0.27)),
        ]))
}

/// The floating navbar: the key picker on the left, `right` on the right.
pub fn navbar<H: Host>(key: usize, right: impl IntoElement, cx: &mut Context<H>) -> Div {
    bar_shell().h(px(52.)).px(px(22.)).flex().items_center().justify_between().child(key_picker(key, cx)).child(right)
}

/// Whether `b` shows at all: canvases skip their paths when scrolled away.
pub fn visible(b: Bounds<Pixels>, window: &Window) -> bool {
    b.intersects(&window.content_mask().bounds)
}

/// Many short radial strokes of one width and color, built as one path so
/// they tessellate and draw once rather than one by one. Widths and radii
/// are in skin units; they're zoomed here.
pub struct Rays {
    path: PathBuilder,
    empty: bool,
    k: f32,
}

impl Rays {
    pub fn new(width: f32) -> Self {
        let k = zoom();
        Rays { path: PathBuilder::stroke(px(width * k)), empty: true, k }
    }

    /// A stroke from radius r0 to r1 at `deg` degrees, clockwise from up.
    pub fn add(&mut self, c: Point<Pixels>, deg: f32, r0: f32, r1: f32) {
        let (s, co) = (deg.to_radians().sin(), -deg.to_radians().cos());
        let (r0, r1) = (r0 * self.k, r1 * self.k);
        self.path.move_to(point(c.x + px(s * r0), c.y + px(co * r0)));
        self.path.line_to(point(c.x + px(s * r1), c.y + px(co * r1)));
        self.empty = false;
    }

    pub fn paint(self, window: &mut Window, color: Hsla) {
        if self.empty || no_paths() {
            return;
        }
        if let Ok(p) = self.path.build() {
            window.paint_path(p, color);
        }
    }
}

/// A grid of small cells (lamps, ticks, LEDs) painted straight to the GPU as
/// quads: no element per cell, so no layout per cell. `color(col, row)`
/// returns the cell's color and whether it glows. Sizes in skin units.
pub fn cells(
    cols: usize,
    rows: usize,
    cell: (f32, f32),
    pitch: (f32, f32),
    radius: f32,
    color: impl Fn(usize, usize) -> (Hsla, bool) + 'static,
    glow: Hsla,
) -> gpui::Canvas<()> {
    gpui::canvas(
        |_, _, _| {},
        move |b: Bounds<Pixels>, _, window, _| {
            if !visible(b, window) {
                return;
            }
            let k = zoom();
            let w = (cols as f32 - 1.) * pitch.0 + cell.0;
            let h = (rows as f32 - 1.) * pitch.1 + cell.1;
            // Centered in the canvas.
            let x0 = b.origin.x + (b.size.width - px(w * k)) / 2.;
            let y0 = b.origin.y + (b.size.height - px(h * k)) / 2.;
            for r in 0..rows {
                for c in 0..cols {
                    let (color, lit) = color(c, r);
                    let at = point(x0 + px(c as f32 * pitch.0 * k), y0 + px(r as f32 * pitch.1 * k));
                    let bounds = Bounds::new(at, gpui::size(px(cell.0 * k), px(cell.1 * k)));
                    if lit && lod() > 0 {
                        let halo = bounds.dilate(px(1.5 * k));
                        window.paint_quad(gpui::quad(halo, px((radius + 1.5) * k), glow, px(0.), gpui::transparent_black(), Default::default()));
                    }
                    window.paint_quad(gpui::quad(bounds, px(radius * k), color, px(0.), gpui::transparent_black(), Default::default()));
                }
            }
        },
    )
}
