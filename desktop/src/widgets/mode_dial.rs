//! 53 Mode dial: a knurled rotary knob that turns between printed stops on
//! an arc scale; the chosen stop is printed on a chip in the key color.
//! Click a stop or the knob, or Shift+scroll over the dial. The knob eases round.

use gpui::{
    Bounds, Context, Div, FontWeight, InteractiveElement, MouseButton, ParentElement, Pixels,
    Styled, canvas, div, linear_color_stop, linear_gradient, point, px,
};

use crate::{
    rack::{Host, Rack},
    theme::{self, Theme},
    ui::{Rays, label, module, scroll_steps, sh, visible},
};

const W: f32 = 214.;
const H: f32 = 184.;
const R: f32 = 106.; // scale radius
const STOP_R: f32 = 74.; // radius of the stop labels
const KNOB: f32 = 72.;

pub struct Dial {
    pub name: &'static str,
    pub stops: &'static [&'static str],
    pub sweep: f32,
    pub chosen: usize,
    /// The pointer's drawn angle, easing toward the chosen stop's.
    pub at: f32,
}

impl Dial {
    pub fn new(name: &'static str, stops: &'static [&'static str], sweep: f32, chosen: usize) -> Self {
        let mut d = Dial { name, stops, sweep, chosen, at: 0. };
        d.at = d.angle(chosen);
        d
    }

    /// A stop's angle in degrees from straight up, clockwise.
    pub fn angle(&self, i: usize) -> f32 {
        let n = self.stops.len().max(2) - 1;
        -self.sweep / 2. + self.sweep * i as f32 / n as f32
    }

    pub fn step(&mut self, by: i32) {
        let n = self.stops.len() as i32;
        self.chosen = (self.chosen as i32 + by).clamp(0, n - 1) as usize;
    }
}

pub fn mode_dial_module<H: Host>(r: &Rack, ri: usize, t: &Theme, cx: &mut Context<H>) -> Div {
    let body = div().flex().gap(px(40.)).children((0..r.dials.len()).map(|di| dial(r, ri, di, t, cx)));
    module(t, "53", "Mode dial", 492., body)
}

fn dial<H: Host>(r: &Rack, ri: usize, di: usize, t: &Theme, cx: &mut Context<H>) -> Div {
    let d = &r.dials[di];
    let sweep = d.sweep;
    let at = d.at;
    let pointer = t.ink.hsla();

    // The printed scale: a tick every 6deg across the sweep.
    let scale = canvas(
        |_, _, _| {},
        move |b: Bounds<Pixels>, _, window, _| {
            if !visible(b, window) {
                return;
            }
            let c = point(b.origin.x + px(W / 2.), b.origin.y + px(R + 1.));
            let mut ticks = Rays::new(1.);
            for k in 0..=(sweep / 6.).round() as i32 {
                ticks.add(c, -sweep / 2. + 6. * k as f32, R - 7., R - 1.);
            }
            ticks.paint(window, theme::print());
        },
    )
    .absolute()
    .size_full();

    // The knurled ring round the knob.
    let knurl = canvas(
        |_, _, _| {},
        move |b: Bounds<Pixels>, _, window, _| {
            if !visible(b, window) {
                return;
            }
            let mut knurl = Rays::new(1.);
            for k in 0..90 {
                knurl.add(b.center(), k as f32 * 4., 36.5, 40.);
            }
            knurl.paint(window, theme::rgb(200, 199, 194));
        },
    )
    .absolute()
    .size_full();

    let pointer_line = canvas(
        |_, _, _| {},
        move |b: Bounds<Pixels>, _, window, _| {
            if !visible(b, window) {
                return;
            }
            let mut line = Rays::new(3.);
            line.add(b.center(), at, 13., 29.);
            line.paint(window, pointer);
        },
    )
    .absolute()
    .size_full();

    // Ring (80px with the knurl), face, faint inner ring, pointer.
    let knob_c = (W / 2., R + 1. - 3.);
    let knob = div()
        .absolute()
        .left(px(knob_c.0 - 40.))
        .top(px(knob_c.1 - 40.))
        .size(px(80.))
        .rounded_full()
        .bg(theme::rgb(217, 216, 211))
        .cursor_pointer()
        .shadow(vec![
            sh(0., 0., 0., 0.5, theme::rgba(0, 0, 0, 0.08)),
            sh(0., 3., 0., 0., theme::rgb(196, 195, 190)),
            sh(0., 3., 0., 0.5, theme::rgba(0, 0, 0, 0.1)),
            sh(0.5, 4., 2., 0., theme::shade(0.18)),
            sh(1., 8., 12., -2., theme::shade(0.16)),
        ])
        .child(knurl)
        .child(
            div()
                .absolute()
                .left(px(4.))
                .top(px(4.))
                .size(px(KNOB))
                .rounded_full()
                .bg(linear_gradient(155., linear_color_stop(theme::rgb(255, 255, 255), 0.15), linear_color_stop(theme::rgb(224, 223, 218), 1.)))
                .border_t_1()
                .border_color(theme::rgb(255, 255, 255))
                .child(
                    div()
                        .absolute()
                        .left(px(KNOB / 2. - 31.))
                        .top(px(KNOB / 2. - 31.))
                        .size(px(62.))
                        .rounded_full()
                        .border_1()
                        .border_color(theme::rgba(0, 0, 0, 0.05)),
                ),
        )
        .child(pointer_line)
        .on_mouse_down(MouseButton::Left, cx.listener(move |this: &mut H, _, _, cx| {
            let d = &mut this.rack_mut(ri).dials[di];
            d.chosen = (d.chosen + 1) % d.stops.len();
            cx.notify();
        }));

    let stops = d.stops.iter().enumerate().map(|(i, s)| {
        let a = d.angle(i).to_radians();
        let (sx, sy) = (W / 2. + a.sin() * STOP_R, R + 1. - a.cos() * STOP_R);
        let chip = div()
            .h(px(20.))
            .px(px(5.))
            .min_w(px(20.))
            .flex()
            .items_center()
            .justify_center()
            .rounded(px(10.))
            .text_size(px(8.))
            .font_weight(FontWeight::MEDIUM)
            .cursor_pointer()
            .child(s.to_uppercase());
        let chip = if i == d.chosen {
            chip.bg(t.cap.hsla()).text_color(t.on_accent).shadow(vec![sh(0., 1., 0., 0., theme::rgba(255, 255, 255, 0.6))])
        } else {
            chip.text_color(theme::print())
        };
        // Centered on its point: a 60px box, the chip centered inside.
        div()
            .absolute()
            .left(px(sx - 30.))
            .top(px(sy - 10.))
            .w(px(60.))
            .flex()
            .justify_center()
            .child(chip.on_mouse_down(MouseButton::Left, cx.listener(move |this: &mut H, _, _, cx| {
                this.rack_mut(ri).dials[di].chosen = i;
                cx.notify();
            })))
    });

    let face = div()
        .relative()
        .w(px(W))
        .h(px(H))
        .child(scale)
        .children(stops)
        .child(knob)
        .on_scroll_wheel(cx.listener(move |this: &mut H, e: &gpui::ScrollWheelEvent, _, cx| {
            if !e.modifiers.shift {
                return;
            }
            let step = scroll_steps(e);
            if step != 0. {
                this.rack_mut(ri).dials[di].step(step as i32);
                cx.stop_propagation();
                cx.notify();
            }
        }));

    div().flex().flex_col().gap(px(8.)).child(label(d.name)).child(face)
}
