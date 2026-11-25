//! Key-based components: Checkbox, Segmented control, Radio group, Number
//! stepper, Toggle group, Badge.

use gpui::{
    Context, Div, FontWeight, InteractiveElement, MouseButton, ParentElement, Styled, div,
    linear_color_stop, linear_gradient,
};

use crate::{
    rack::{Host, Rack},
    theme::{self, Theme, oklch},
    ui::{fx, key_cap, label, lamp, module, sh, z},
};

/// 03 Checkbox: black latching keys. Up, a key stands proud of its well;
/// latched, it sits down inside it.
pub fn checkbox_module<H: Host>(r: &Rack, ri: usize, t: &Theme, cx: &mut Context<H>) -> Div {
    let keys = ["SYNC", "LOOP", "METRO"].iter().enumerate().map(|(i, name)| {
        let latched = r.checks[i];
        let lift = if latched { 0. } else { 9. };
        let cap = div()
            .absolute()
            .left(z(6.))
            .top(z(6. - lift))
            .w(z(50.))
            .h(z(42.))
            .rounded(z(11.))
            .flex()
            .items_center()
            .justify_center()
            .bg(linear_gradient(180., linear_color_stop(theme::rgb(58, 58, 56), 0.), linear_color_stop(theme::rgb(30, 30, 29), 1.)))
            .border_t_1()
            .border_color(theme::rgba(255, 255, 255, 0.12))
            .shadow(if latched {
                vec![sh(0., 1., 2., 0., theme::rgba(0, 0, 0, 0.6))]
            } else {
                vec![sh(0., 4., 0., 0., theme::rgb(14, 14, 13)), sh(0., 8., 10., -2., theme::rgba(0, 0, 0, 0.45))]
            })
            .text_size(z(8.5))
            .font_weight(FontWeight::MEDIUM)
            .text_color(if latched { theme::rgb(178, 178, 172) } else { theme::rgb(236, 236, 230) })
            .child(name.to_string());
        div()
            .relative()
            .w(z(62.))
            .h(z(52.))
            .rounded(z(14.))
            .bg(theme::rgb(12, 12, 11))
            .cursor_pointer()
            .child(cap)
            .on_mouse_down(MouseButton::Left, cx.listener(move |this: &mut H, _, _, cx| {
                let r = this.rack_mut(ri);
                r.checks[i] = !r.checks[i];
                cx.notify();
            }))
    });
    module(t, "03", "Checkbox", 238., div().pt(z(10.)).flex().gap(z(8.)).children(keys))
}

/// 04 Segmented control: a recessed bar with a white key that slides to the
/// chosen segment, and a lamp under each.
pub fn segmented_module<H: Host>(r: &Rack, ri: usize, t: &Theme, cx: &mut Context<H>) -> Div {
    const W: f32 = 200.;
    let seg = W / 4.;
    let key = div()
        .absolute()
        .top(z(4.))
        .left(z(4. + seg * r.segment_at))
        .w(z(seg - 8.))
        .h(z(38.))
        .rounded(z(10.))
        .bg(linear_gradient(180., linear_color_stop(theme::rgb(255, 255, 255), 0.), linear_color_stop(theme::rgb(240, 240, 237), 1.)))
        .shadow(fx(vec![sh(0., 0., 0., 1., theme::shade(0.08)), sh(0., 2., 3., 0., theme::shade(0.15)), sh(0., 4., 8., -2., theme::shade(0.12))]));
    let slots = (0..4).map(|i| {
        div()
            .relative()
            .w(z(seg))
            .h_full()
            .cursor_pointer()
            .when_divider(i > 0 && i != r.segment && i != r.segment + 1)
            .on_mouse_down(MouseButton::Left, cx.listener(move |this: &mut H, _, _, cx| {
                this.rack_mut(ri).segment = i;
                cx.notify();
            }))
    });
    let bar = div()
        .relative()
        .w(z(W))
        .h(z(46.))
        .rounded(z(14.))
        .bg(theme::rgb(234, 234, 231))
        .border_1()
        .border_color(theme::rgba(0, 0, 0, 0.06))
        .flex()
        .children(slots)
        .child(key);
    let lamps = div().w(z(W)).flex().children((0..4).map(|i| {
        div()
            .w(z(seg))
            .flex()
            .flex_col()
            .items_center()
            .gap(z(5.))
            .child(lamp(t, 5., i == r.segment))
            .child(div().text_size(z(9.)).font_weight(FontWeight::MEDIUM).text_color(if i == r.segment { theme::text() } else { theme::dim() }).child(format!("{}", i + 1)))
    }));
    module(t, "04", "Segmented control", 238., div().flex().flex_col().gap(z(10.)).child(label("Pattern")).child(bar).child(lamps))
}

trait Divider {
    fn when_divider(self, on: bool) -> Self;
}

impl Divider for Div {
    /// A thin printed line at a segment's left edge.
    fn when_divider(self, on: bool) -> Self {
        if !on {
            return self;
        }
        self.child(div().absolute().left_0().top(z(14.)).w(z(1.5)).h(z(18.)).bg(theme::rgb(196, 196, 191)))
    }
}

/// 14 Radio group: three white keys, the chosen one pressed with its lamp lit.
pub fn radio_module<H: Host>(r: &Rack, ri: usize, t: &Theme, cx: &mut Context<H>) -> Div {
    let keys = ["Internal", "MIDI", "USB"].iter().enumerate().map(|(i, name)| {
        let on = r.radio == i;
        key_cap(62., 46., 12., on)
            .relative()
            .cursor_pointer()
            .flex()
            .items_end()
            .justify_center()
            .pb(z(9.))
            .child(div().absolute().left(z(9.)).top(z(8.)).child(lamp(t, 6., on)))
            .child(div().text_size(z(8.)).font_weight(FontWeight::MEDIUM).text_color(theme::text()).child(name.to_uppercase()))
            .on_mouse_down(MouseButton::Left, cx.listener(move |this: &mut H, _, _, cx| {
                this.rack_mut(ri).radio = i;
                cx.notify();
            }))
    });
    module(t, "14", "Radio group", 238., div().flex().flex_col().gap(z(10.)).child(label("Sync source")).child(div().flex().gap(z(8.)).children(keys)))
}

/// 15 Number stepper: minus and plus keys round a recessed readout.
pub fn stepper_module<H: Host>(r: &Rack, ri: usize, t: &Theme, cx: &mut Context<H>) -> Div {
    let key = |sign: i32, cx: &mut Context<H>| {
        key_cap(40., 40., 12., false)
            .flex()
            .items_center()
            .justify_center()
            .cursor_pointer()
            .text_size(z(14.))
            .text_color(theme::text())
            .child(if sign < 0 { "−" } else { "+" })
            .on_mouse_down(MouseButton::Left, cx.listener(move |this: &mut H, _, _, cx| {
                let r = this.rack_mut(ri);
                r.tempo = (r.tempo + sign).clamp(20, 300);
                cx.notify();
            }))
    };
    let readout = div()
        .w(z(92.))
        .h(z(40.))
        .rounded(z(10.))
        .bg(theme::rgb(236, 236, 233))
        .border_1()
        .border_color(theme::rgba(0, 0, 0, 0.07))
        .flex()
        .items_center()
        .justify_center()
        .text_size(z(24.))
        .font_weight(FontWeight::LIGHT)
        .text_color(t.ink.hsla())
        .child(format!("{}", r.tempo));
    let row = div().flex().items_center().gap(z(8.)).child(key(-1, cx)).child(readout).child(key(1, cx));
    module(t, "15", "Number stepper", 238., div().flex().flex_col().gap(z(10.)).child(label("Tempo")).child(row))
}

/// 26 Toggle group: three alignment keys, one pressed, its lamp lit.
pub fn toggles_module<H: Host>(r: &Rack, ri: usize, t: &Theme, cx: &mut Context<H>) -> Div {
    // Line lengths of the four text lines in each alignment icon.
    let icon = |a: usize| {
        let lines = [16., 11., 16., 11.];
        div().w(z(16.)).flex().flex_col().gap(z(2.5)).children(lines.iter().map(move |&l| {
            let bar = div().w(z(l)).h(z(1.6)).bg(theme::text());
            let row = div().w(z(16.)).flex();
            match a {
                0 => row.justify_start().child(bar),
                1 => row.justify_center().child(bar),
                _ => row.justify_end().child(bar),
            }
        }))
    };
    let keys = (0..3).map(|i| {
        let on = r.align == i;
        key_cap(44., 44., 12., on)
            .relative()
            .flex()
            .items_center()
            .justify_center()
            .cursor_pointer()
            .child(icon(i))
            .child(div().absolute().right(z(6.)).top(z(6.)).child(lamp(t, 4.5, on)))
            .on_mouse_down(MouseButton::Left, cx.listener(move |this: &mut H, _, _, cx| {
                this.rack_mut(ri).align = i;
                cx.notify();
            }))
    });
    module(t, "26", "Toggle group", 238., div().flex().gap(z(8.)).children(keys))
}

/// 34 Badge: status keys with a colored pill lamp. BUILDING breathes.
pub fn badge_module<H: Host>(_r: &Rack, ri: usize, t: &Theme, _cx: &mut Context<H>) -> Div {
    let pulse = 0.55 + 0.45 * (crate::ui::now() * 3. + ri as f32 * 0.7).sin();
    let badge = |name: &str, color: theme::Oklch, glow: f32| {
        div()
            .h(z(26.))
            .px(z(10.))
            .flex()
            .items_center()
            .gap(z(8.))
            .rounded(z(8.))
            .bg(linear_gradient(180., linear_color_stop(theme::rgb(253, 253, 252), 0.), linear_color_stop(theme::rgb(240, 240, 237), 1.)))
            .shadow(fx(vec![sh(0., 0., 0., 1., theme::shade(0.09)), sh(0., 1.5, 2., 0., theme::shade(0.1))]))
            .child(div().w(z(3.5)).h(z(12.)).rounded(z(2.)).bg(color.hsla()).shadow(fx(vec![sh(0., 0., 5., 0., color.alpha(glow))])))
            .child(div().text_size(z(8.5)).font_weight(FontWeight::MEDIUM).text_color(theme::text()).child(name.to_string()))
    };
    let body = div()
        .flex()
        .flex_col()
        .items_center()
        .gap(z(8.))
        .child(div().flex().gap(z(8.)).child(badge("READY", oklch(0.72, 0.19, 150.), 0.)).child(badge("BUILDING", oklch(0.72, 0.12, 255.).with_l(0.62 + 0.15 * pulse), pulse * 0.8)))
        .child(div().flex().gap(z(8.)).child(badge("DEGRADED", oklch(0.8, 0.16, 80.), 0.)).child(badge("ERROR", oklch(0.62, 0.24, 25.), 0.7)))
        .child(badge("QUEUED", oklch(0.75, 0., 0.), 0.));
    module(t, "34", "Badge", 238., body)
}
