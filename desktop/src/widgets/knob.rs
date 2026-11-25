//! 06 Flat knob: a flat white cap ringed in black, a lit dot that turns with
//! the value through 270deg, and a ring of 28 ticks, every ninth heavier,
//! those up to the value printed dark. Drag up/down or Shift+scroll to turn.

use gpui::{
    Bounds, Context, Div, FontWeight, InteractiveElement, MouseButton, ParentElement, Pixels,
    Styled, canvas, div, linear_color_stop, linear_gradient, px,
};

use crate::{
    rack::{Drag, Host, Rack},
    theme::{self, Theme},
    ui::{Rays, label, module, scroll_steps, sh, visible},
};

pub const NAMES: [&str; 4] = ["Tone", "Drive", "Space", "Mix"];
const TICKS: usize = 27; // intervals; 28 ticks
const DIAL: f32 = 72.;

/// How many tick intervals a value lights, as flat-knob.tsx counts them.
pub fn lit_ticks(v: f32) -> usize {
    ((v * TICKS as f32) / 100.).floor() as usize
}

pub fn knob_module<H: Host>(r: &Rack, ri: usize, t: &Theme, cx: &mut Context<H>) -> Div {
    let body = div().flex().gap(px(12.)).children((0..4).map(|i| knob(r, ri, t, i, cx)));
    module(t, "06", "Flat knob", 492., body).h(px(190.))
}

fn knob<H: Host>(r: &Rack, ri: usize, t: &Theme, i: usize, cx: &mut Context<H>) -> Div {
    let v = r.knobs[i];
    let turn = (v / 100. * 270. - 135.).to_radians();
    let lit = lit_ticks(v);

    // The tick ring, drawn as stroked paths: 28 short radial lines.
    let ticks = canvas(
        |_, _, _| {},
        move |b: Bounds<Pixels>, _, window, _| {
            if !visible(b, window) {
                return;
            }
            // Four batches: lit and unlit, minor and every ninth (major).
            let c = b.center();
            let mut sets = [Rays::new(1.4), Rays::new(2.4), Rays::new(1.4), Rays::new(2.4)];
            for n in 0..=TICKS {
                let set = (n > lit) as usize * 2 + (n % 9 == 0) as usize;
                sets[set].add(c, -135. + 270. * n as f32 / TICKS as f32, 28.8, 36.);
            }
            let [lit_minor, lit_major, dim_minor, dim_major] = sets;
            lit_minor.paint(window, theme::rgb(42, 42, 41));
            lit_major.paint(window, theme::rgb(42, 42, 41));
            dim_minor.paint(window, theme::rgb(154, 154, 150));
            dim_major.paint(window, theme::rgb(154, 154, 150));
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
        .on_mouse_down(MouseButton::Left, cx.listener(move |this: &mut H, e: &gpui::MouseDownEvent, _, cx| {
            let start = this.rack_mut(ri).knobs[i];
            *this.drag_mut() = Drag::Knob { rack: ri, index: i, start_y: f32::from(e.position.y), start };
            cx.notify();
        }))
        .on_scroll_wheel(cx.listener(move |this: &mut H, e: &gpui::ScrollWheelEvent, _, cx| {
            if !e.modifiers.shift {
                return;
            }
            let r = this.rack_mut(ri);
            r.set_knob(i, r.knobs[i] + scroll_steps(e));
            cx.stop_propagation();
            cx.notify();
        }));

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(6.))
        .w(px(DIAL + 12.))
        .child(dial)
        .child(div().pt(px(2.)).text_size(px(15.)).font_weight(FontWeight::LIGHT).text_color(theme::text()).child(format!("{}", v.round() as i32)))
        .child(label(NAMES[i]))
}
