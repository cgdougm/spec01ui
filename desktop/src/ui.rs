//! Shared pieces: shadows, the module panel and its caption, labels.

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

/// A box shadow, written in CSS order: x, y, blur, spread, color.
///
/// gpui draws nothing for a shadow with no blur at all, so the skin's crisp
/// lips and rings (blur 0 in CSS) get the smallest blur that still draws.
pub fn sh(x: f32, y: f32, blur: f32, spread: f32, color: Hsla) -> BoxShadow {
    BoxShadow { color, offset: point(px(x), px(y)), blur_radius: px(blur.max(0.5)), spread_radius: px(spread) }
}

/// The panel every component sits on: a light molded plate with a stacked
/// drop shadow, a caption with the component's number, name and a lit LED.
pub fn module(t: &Theme, number: &str, name: &str, width: f32, body: impl IntoElement) -> Div {
    div()
        .w(px(width))
        .flex()
        .flex_col()
        .pt(px(16.))
        .px(px(18.))
        .pb(px(20.))
        .rounded(px(26.))
        .bg(theme::panel())
        .shadow(vec![
            sh(0., 0., 0., 1., theme::shade(0.055)),
            sh(0.5, 1., 1., 0., theme::shade(0.07)),
            sh(1., 2., 2., 0., theme::shade(0.07)),
            sh(2., 4., 4., 0., theme::shade(0.07)),
            sh(4., 8., 8., 0., theme::shade(0.07)),
            sh(8., 16., 16., 0., theme::shade(0.063)),
        ])
        .child(caption(t, number, name))
        .child(div().flex_1().flex().items_center().justify_center().child(body))
}

fn caption(t: &Theme, number: &str, name: &str) -> Div {
    div()
        .flex()
        .items_center()
        .gap(px(8.))
        .pb(px(10.))
        .text_size(px(10.))
        .line_height(px(12.))
        .font_weight(FontWeight::MEDIUM)
        .child(div().text_color(t.ink.hsla()).child(number.to_string()))
        .child(div().text_color(theme::dim()).child(name.to_uppercase()))
        .child(led(t, 6.))
        .child(div().flex_1().h(px(1.)).bg(theme::rgba(0, 0, 0, 0.07)))
}

/// A small lit lamp in the key color.
pub fn led(t: &Theme, size: f32) -> Div {
    div()
        .size(px(size))
        .rounded_full()
        .bg(t.lamp.hsla())
        .shadow(vec![sh(0., 0., 6., 0., t.lamp.alpha(0.7))])
}

/// The small uppercase printed label used on every control.
pub fn label(text: &str) -> Div {
    div()
        .text_size(px(8.))
        .line_height(px(10.))
        .font_weight(FontWeight::MEDIUM)
        .text_color(theme::text())
        .child(text.to_uppercase())
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

/// The floating navbar: the key picker on the left, `right` on the right.
pub fn navbar<H: Host>(key: usize, right: impl IntoElement, cx: &mut Context<H>) -> Div {
    let swatches = KEYS.iter().enumerate().map(|(i, (_, k))| {
        let on = i == key;
        // The chosen key is ringed: a dark circle, a light gap, the key.
        // (Nested circles, as gpui keeps a shadow's corner radius as it spreads.)
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
    div()
        .h(px(52.))
        .px(px(22.))
        .flex()
        .items_center()
        .justify_between()
        .rounded(px(32.))
        .bg(linear_gradient(180., linear_color_stop(theme::rgb(248, 248, 246), 0.), linear_color_stop(theme::rgb(242, 242, 240), 1.)))
        .border_1()
        .border_color(theme::rgba(255, 255, 255, 0.6))
        .shadow(vec![
            sh(0., 0., 0., 1., theme::rgba(0, 0, 0, 0.07)),
            sh(0., 1., 2., 0., theme::shade(0.11)),
            sh(0., 12., 28., -12., theme::shade(0.27)),
        ])
        .child(div().flex().items_center().gap(px(2.)).children(swatches))
        .child(right)
}

/// Whether `b` shows at all: canvases skip their paths when scrolled away.
pub fn visible(b: Bounds<Pixels>, window: &Window) -> bool {
    b.intersects(&window.content_mask().bounds)
}

/// Many short radial strokes of one width and color, built as one path so
/// they tessellate and draw once rather than one by one.
pub struct Rays {
    path: PathBuilder,
    empty: bool,
}

impl Rays {
    pub fn new(width: f32) -> Self {
        Rays { path: PathBuilder::stroke(px(width)), empty: true }
    }

    /// A stroke from radius r0 to r1 at `deg` degrees, clockwise from up.
    pub fn add(&mut self, c: Point<Pixels>, deg: f32, r0: f32, r1: f32) {
        let (s, co) = (deg.to_radians().sin(), -deg.to_radians().cos());
        self.path.move_to(point(c.x + px(s * r0), c.y + px(co * r0)));
        self.path.line_to(point(c.x + px(s * r1), c.y + px(co * r1)));
        self.empty = false;
    }

    pub fn paint(self, window: &mut Window, color: Hsla) {
        if self.empty {
            return;
        }
        if let Ok(p) = self.path.build() {
            window.paint_path(p, color);
        }
    }
}
