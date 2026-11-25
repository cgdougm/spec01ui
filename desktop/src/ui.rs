//! Shared pieces: shadows, the module panel and its caption, labels.

use gpui::{
    BoxShadow, Div, FontWeight, Hsla, IntoElement, ParentElement, Styled, div, point,
    px,
};

use crate::theme::{self, Theme};

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
