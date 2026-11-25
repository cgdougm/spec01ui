//! 06 Flat knob: a flat white cap ringed in black, a lit dot that turns with
//! the value through 270deg, and a ring of 28 ticks, every ninth heavier,
//! those up to the value printed dark. Drag up/down or scroll to turn.

use gpui::{
    Bounds, Context, Div, FontWeight, InteractiveElement, MouseButton, ParentElement, PathBuilder,
    Pixels, ScrollDelta, Styled, canvas, div, linear_color_stop, linear_gradient, point, px,
};

use crate::{
    gallery::{Drag, Gallery},
    theme::{self, Theme},
    ui::{label, module, sh},
};

pub const NAMES: [&str; 4] = ["Tone", "Drive", "Space", "Mix"];
const TICKS: usize = 27; // intervals; 28 ticks
const DIAL: f32 = 72.;

/// How many tick intervals a value lights, as flat-knob.tsx counts them.
pub fn lit_ticks(v: f32) -> usize {
    ((v * TICKS as f32) / 100.).floor() as usize
}

impl Gallery {
    pub fn knob_module(&self, t: &Theme, cx: &mut Context<Self>) -> Div {
        let body = div().flex().gap(px(12.)).children((0..4).map(|i| self.knob(t, i, cx)));
        module(t, "06", "Flat knob", 492., body).h(px(190.))
    }

    fn knob(&self, t: &Theme, i: usize, cx: &mut Context<Self>) -> Div {
        let v = self.knobs[i];
        let share = v / 100.;
        let turn = (share * 270. - 135.).to_radians();
        let lit = lit_ticks(v);

        // The tick ring, drawn as stroked paths: 28 short radial lines.
        let ticks = canvas(
            |_, _, _| {},
            move |b: Bounds<Pixels>, _, window, _| {
                let c = b.center();
                for n in 0..=TICKS {
                    let a = (-135. + 270. * n as f32 / TICKS as f32).to_radians();
                    let (s, co) = (a.sin(), -a.cos());
                    let major = n % 9 == 0;
                    let mut p = PathBuilder::stroke(px(if major { 2.4 } else { 1.4 }));
                    p.move_to(point(c.x + px(s * 28.8), c.y + px(co * 28.8)));
                    p.line_to(point(c.x + px(s * 36.), c.y + px(co * 36.)));
                    if let Ok(path) = p.build() {
                        let color = if n <= lit { theme::rgb(42, 42, 41) } else { theme::rgb(154, 154, 150) };
                        window.paint_path(path, color);
                    }
                }
            },
        )
        .absolute()
        .size_full();

        let (dx, dy) = (turn.sin() * 15., -turn.cos() * 15.);
        let cap = div()
            .absolute()
            .left(px(12.))
            .top(px(12.))
            .size(px(48.))
            .rounded_full()
            .bg(linear_gradient(180., linear_color_stop(theme::rgb(255, 255, 255), 0.2), linear_color_stop(theme::rgb(243, 243, 242), 0.85)))
            .shadow(vec![
                sh(0., 0., 0., 2., theme::rgb(44, 44, 43)),
                sh(0.5, 1.5, 1.5, 2., theme::shade(0.18)),
                sh(1., 3., 5., 2., theme::shade(0.11)),
                sh(2., 5., 9., -1., theme::shade(0.11)),
            ])
            .child(
                div()
                    .absolute()
                    .left(px(24. + dx - 2.1))
                    .top(px(24. + dy - 2.1))
                    .size(px(4.2))
                    .rounded_full()
                    .bg(t.key.hsla())
                    .shadow(vec![sh(0., 0., 2., 0., t.key.alpha(0.55))]),
            );

        let dial = div()
            .relative()
            .size(px(DIAL))
            .cursor_ns_resize()
            .child(ticks)
            .child(cap)
            .on_mouse_down(MouseButton::Left, cx.listener(move |this, e: &gpui::MouseDownEvent, _, cx| {
                this.drag = Drag::Knob { index: i, start_y: f32::from(e.position.y), start: this.knobs[i] };
                cx.notify();
            }))
            .on_scroll_wheel(cx.listener(move |this, e: &gpui::ScrollWheelEvent, _, cx| {
                let dy = match e.delta {
                    ScrollDelta::Lines(p) => p.y,
                    ScrollDelta::Pixels(p) => f32::from(p.y) / 20.,
                };
                this.set_knob(i, this.knobs[i] + dy.signum());
                cx.notify();
            }));

        div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(6.))
            .w(px(DIAL + 12.))
            .child(dial)
            .child(
                div()
                    .pt(px(2.))
                    .text_size(px(15.))
                    .font_weight(FontWeight::LIGHT)
                    .text_color(theme::text())
                    .child(format!("{}", v.round() as i32)),
            )
            .child(label(NAMES[i]))
    }

    pub fn set_knob(&mut self, i: usize, v: f32) {
        self.knobs[i] = v.clamp(0., 100.).round();
    }
}
