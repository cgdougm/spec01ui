//! 02 Switch: a recessed track that fills with the key color, and a white
//! thumb that slides across. Click to toggle; the thumb eases over.

use gpui::{
    Context, Div, FontWeight, InteractiveElement, MouseButton, ParentElement, Styled, div,
    linear_color_stop, linear_gradient,
};

use crate::{
    rack::{Host, Rack},
    theme::{self, Theme},
    ui::{fx, z, label, module, sh},
};

pub fn switch_module<H: Host>(r: &Rack, ri: usize, t: &Theme, cx: &mut Context<H>) -> Div {
    let at = r.switch_at; // 0 off .. 1 on, animated
    let off = theme::rgb(188, 188, 184);
    // The key color comes in as the thumb crosses over.
    let fill = if at > 0.5 { t.key.hsla() } else { off };
    let side = |s: &str| div().text_size(z(8.5)).font_weight(FontWeight::MEDIUM).text_color(theme::dim()).child(s.to_string());
    let track = div()
        .relative()
        .w(z(52.))
        .h(z(28.))
        .rounded(z(28.))
        .bg(fill)
        .border_1()
        .border_color(theme::rgba(0, 0, 0, 0.06))
        .cursor_pointer()
        .on_mouse_down(MouseButton::Left, cx.listener(move |this: &mut H, _, _, cx| {
            let r = this.rack_mut(ri);
            r.power = !r.power;
            cx.notify();
        }))
        .child(
            div()
                .absolute()
                .top(z(2.))
                .left(z(2. + 24. * at))
                .size(z(22.))
                .rounded_full()
                .bg(linear_gradient(180., linear_color_stop(theme::rgb(255, 255, 255), 0.), linear_color_stop(theme::rgb(242, 242, 241), 1.)))
                .shadow(fx(vec![
                    sh(0., 0., 0., 0.5, theme::rgba(0, 0, 0, 0.07)),
                    sh(0., 1., 1.5, 0., theme::rgba(0, 0, 0, 0.16)),
                    sh(0., 2., 4., -1., theme::rgba(0, 0, 0, 0.11)),
                    sh(0., 4., 8., -4., theme::rgba(0, 0, 0, 0.125)),
                ])),
        );
    let body = div()
        .flex()
        .flex_col()
        .items_center()
        .gap(z(10.))
        .child(label("Power"))
        .child(div().flex().items_center().gap(z(10.)).child(side("OFF")).child(track).child(side("ON")));
    module(t, "02", "Switch", 238., body).h(z(190.))
}
