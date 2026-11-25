//! 05 Slider: a dark slot lit in the key color up to the value, a round
//! white cap with a hairline needle, and a printed scale. Click or drag
//! anywhere on it; scroll to nudge.

use gpui::{
    Bounds, Context, Div, FontWeight, InteractiveElement, MouseButton, ParentElement, Pixels,
    Styled, canvas, div, linear_color_stop, linear_gradient, px,
};

use crate::{
    rack::{Drag, Host, Rack},
    theme::{self, Theme},
    ui::{label, module, scroll_steps, sh},
};

pub const WIDTH: f32 = 456.;

pub fn slider_module<H: Host>(r: &Rack, ri: usize, t: &Theme, cx: &mut Context<H>) -> Div {
    let v = r.volume;
    let x = WIDTH * v / 100.;
    let bounds = r.slider_bounds.clone();

    let scale = div().absolute().top(px(36.)).left_0().w(px(WIDTH)).h(px(8.)).children((0..=20).map(|i| {
        let major = i % 5 == 0;
        div()
            .absolute()
            .left(px(WIDTH * i as f32 / 20. - if major { 0.75 } else { 0.5 }))
            .w(px(if major { 1.5 } else { 1. }))
            .h(px(if major { 8. } else { 4. }))
            .bg(theme::rgb(27, 27, 25))
    }));

    let track = div()
        .absolute()
        .top(px(14.))
        .left_0()
        .w(px(WIDTH))
        .h(px(8.))
        .rounded(px(8.))
        .bg(theme::rgb(28, 28, 27))
        .shadow(vec![sh(0., 1., 0., 0., theme::rgb(255, 255, 255))])
        .child(
            div()
                .h_full()
                .w(px(x.max(8.)))
                .rounded(px(8.))
                .bg(linear_gradient(180., linear_color_stop(t.key.with_l(t.key.l * 0.85).hsla(), 0.), linear_color_stop(t.key.hsla(), 1.)))
                .shadow(vec![sh(0., 0., 8., -1., t.key.alpha(0.55))]),
        );

    let thumb = div()
        .absolute()
        .left(px(x - 15.))
        .top(px(-5.))
        .w(px(30.))
        .h(px(46.))
        // The needle, standing above and below the cap.
        .child(div().absolute().left(px(14.5)).top_0().w(px(1.)).h(px(46.)).bg(theme::rgb(141, 140, 136)))
        .child(
            div()
                .absolute()
                .top(px(6.5))
                .size(px(30.))
                .rounded_full()
                .bg(linear_gradient(180., linear_color_stop(theme::rgb(255, 255, 255), 0.3), linear_color_stop(theme::rgb(238, 238, 235), 1.)))
                .shadow(vec![
                    sh(0., 0., 0., 0.5, theme::rgba(0, 0, 0, 0.145)),
                    sh(0., 3., 0., 0., theme::rgb(210, 210, 206)),
                    sh(0., 3., 0., 0.5, theme::rgba(0, 0, 0, 0.09)),
                    sh(0., 4., 2., 0., theme::rgba(0, 0, 0, 0.145)),
                    sh(0., 8., 10., -4., theme::rgba(0, 0, 0, 0.145)),
                ]),
        );

    let control = div()
        .relative()
        .w(px(WIDTH))
        .h(px(44.))
        .cursor_pointer()
        // Remember where the control is, to turn pointer x into a value.
        .child(canvas(move |b: Bounds<Pixels>, _, _| bounds.set(b), |_, _, _, _| {}).absolute().size_full())
        .child(scale)
        .child(track)
        .child(thumb)
        .on_mouse_down(MouseButton::Left, cx.listener(move |this: &mut H, e: &gpui::MouseDownEvent, _, cx| {
            *this.drag_mut() = Drag::Slider { rack: ri };
            this.rack_mut(ri).slide_to(e.position.x);
            cx.notify();
        }))
        .on_scroll_wheel(cx.listener(move |this: &mut H, e: &gpui::ScrollWheelEvent, _, cx| {
            // Shift+wheel nudges; a plain wheel keeps scrolling the page.
            if !e.modifiers.shift {
                return;
            }
            let r = this.rack_mut(ri);
            r.volume = (r.volume + scroll_steps(e)).clamp(0., 100.);
            cx.stop_propagation();
            cx.notify();
        }));

    let body = div()
        .flex()
        .flex_col()
        .gap(px(10.))
        .child(
            div()
                .flex()
                .justify_between()
                .items_end()
                .child(label("Volume"))
                .child(div().text_size(px(10.)).font_weight(FontWeight::MEDIUM).text_color(theme::dim()).child(format!("{}", v.round() as i32))),
        )
        .child(control);
    module(t, "05", "Slider", 492., body).h(px(190.))
}
