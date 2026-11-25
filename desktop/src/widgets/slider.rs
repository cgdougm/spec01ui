//! 05 Slider: a dark slot lit in the key color up to the value, a round
//! white cap with a hairline needle, and a printed scale. Click or drag
//! anywhere on it; scroll to nudge.

use gpui::{
    Bounds, Context, Div, FontWeight, InteractiveElement, MouseButton, ParentElement, Pixels,
    Styled, canvas, div, linear_color_stop, linear_gradient,
};

use crate::{
    rack::{Drag, Host, Rack},
    theme::{self, Theme},
    ui::{fx, z, label, module, scroll_steps, sh},
};

pub const WIDTH: f32 = 456.;

pub fn slider_module<H: Host>(r: &Rack, ri: usize, t: &Theme, cx: &mut Context<H>) -> Div {
    let v = r.volume;
    let x = WIDTH * v / 100.;
    let bounds = r.slider_bounds.clone();

    // The printed scale: 21 ticks, every fifth long, painted as quads.
    let ink = theme::rgb(27, 27, 25);
    let scale = canvas(
        |_, _, _| {},
        move |b: Bounds<Pixels>, _, window, _| {
            if !crate::ui::visible(b, window) {
                return;
            }
            let k = crate::ui::zoom();
            for i in 0..=20 {
                let major = i % 5 == 0;
                let (w, h) = (if major { 1.5 } else { 1. }, if major { 8. } else { 4. });
                let x = b.origin.x + gpui::px((WIDTH * i as f32 / 20. - w / 2.) * k);
                window.paint_quad(gpui::fill(Bounds::new(gpui::point(x, b.origin.y), gpui::size(gpui::px(w * k), gpui::px(h * k))), ink));
            }
        },
    )
    .absolute()
    .top(z(36.))
    .left_0()
    .w(z(WIDTH))
    .h(z(8.));

    let track = div()
        .absolute()
        .top(z(14.))
        .left_0()
        .w(z(WIDTH))
        .h(z(8.))
        .rounded(z(8.))
        .bg(theme::rgb(28, 28, 27))
        .shadow(fx(vec![sh(0., 1., 0., 0., theme::rgb(255, 255, 255))]))
        .child(
            div()
                .h_full()
                .w(z(x.max(8.)))
                .rounded(z(8.))
                .bg(linear_gradient(180., linear_color_stop(t.key.with_l(t.key.l * 0.85).hsla(), 0.), linear_color_stop(t.key.hsla(), 1.)))
                .shadow(fx(vec![sh(0., 0., 8., -1., t.key.alpha(0.55))])),
        );

    let thumb = div()
        .absolute()
        .left(z(x - 15.))
        .top(z(-5.))
        .w(z(30.))
        .h(z(46.))
        // The needle, standing above and below the cap.
        .child(div().absolute().left(z(14.5)).top_0().w(z(1.)).h(z(46.)).bg(theme::rgb(141, 140, 136)))
        .child(
            div()
                .absolute()
                .top(z(6.5))
                .size(z(30.))
                .rounded_full()
                .bg(linear_gradient(180., linear_color_stop(theme::rgb(255, 255, 255), 0.3), linear_color_stop(theme::rgb(238, 238, 235), 1.)))
                .shadow(fx(vec![
                    sh(0., 0., 0., 0.5, theme::rgba(0, 0, 0, 0.145)),
                    sh(0., 3., 0., 0., theme::rgb(210, 210, 206)),
                    sh(0., 3., 0., 0.5, theme::rgba(0, 0, 0, 0.09)),
                    sh(0., 4., 2., 0., theme::rgba(0, 0, 0, 0.145)),
                    sh(0., 8., 10., -4., theme::rgba(0, 0, 0, 0.145)),
                ])),
        );

    let control = div()
        .relative()
        .w(z(WIDTH))
        .h(z(44.))
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
        .gap(z(10.))
        .child(
            div()
                .flex()
                .justify_between()
                .items_end()
                .child(label("Volume"))
                .child(div().text_size(z(10.)).font_weight(FontWeight::MEDIUM).text_color(theme::dim()).child(format!("{}", v.round() as i32))),
        )
        .child(control);
    module(t, "05", "Slider", 492., body).h(z(190.))
}
