//! Controls: D-pad, Slide switch, Tabs.

use gpui::{
    Bounds, Context, Div, FontWeight, InteractiveElement, MouseButton, ParentElement, PathBuilder,
    Pixels, Styled, canvas, div, linear_color_stop, linear_gradient, point, px,
};

use crate::{
    rack::{Host, Rack},
    theme::{self, Theme},
    ui::{cells, fx, dark_glass, label, module, sh, visible, z, zoom},
};

/// 52 D-pad: a black cross on a dished plate moves a lit cell round a grid.
pub fn dpad_module<H: Host>(r: &Rack, ri: usize, t: &Theme, cx: &mut Context<H>) -> Div {
    let (cell, on, off) = (r.cell, t.lamp.hsla(), theme::rgb(52, 52, 50));
    let grid = dark_glass(110., 74.).child(
        cells(5, 3, (12., 12.), (18., 18.), 1.5, move |x, y| if (x as i32, y as i32) == cell { (on, true) } else { (off, false) }, t.lamp.alpha(0.5)).size_full(),
    );

    // Arms of the cross: (dx, dy) and where each sits.
    let arm = |dx: i32, dy: i32, cx: &mut Context<H>| {
        div()
            .absolute()
            .left(z(52. + dx as f32 * 34.))
            .top(z(52. + dy as f32 * 34.))
            .size(z(34.))
            .cursor_pointer()
            .on_mouse_down(MouseButton::Left, cx.listener(move |this: &mut H, _, _, cx| {
                let r = this.rack_mut(ri);
                r.cell = ((r.cell.0 + dx).clamp(0, 4), (r.cell.1 + dy).clamp(0, 2));
                cx.notify();
            }))
    };
    let bar = |w: f32, h: f32| {
        div()
            .absolute()
            .left(z((138. - w) / 2.))
            .top(z((138. - h) / 2. - 3.))
            .w(z(w))
            .h(z(h))
            .rounded(z(8.))
            .bg(linear_gradient(180., linear_color_stop(theme::rgb(54, 54, 52), 0.), linear_color_stop(theme::rgb(32, 32, 31), 1.)))
            .shadow(fx(vec![sh(0., 3., 0., 0., theme::rgb(14, 14, 13)), sh(0., 8., 12., -3., theme::rgba(0, 0, 0, 0.35))]))
    };
    // Little printed arrows on each arm, filled triangles in one path.
    let arrows = canvas(
        |_, _, _| {},
        |b: Bounds<Pixels>, _, window, _| {
            if !visible(b, window) {
                return;
            }
            let k = zoom();
            let c = point(b.center().x, b.center().y - px(3. * k));
            let mut p = PathBuilder::fill();
            for (dx, dy) in [(0., -1.), (0., 1.), (-1., 0.), (1., 0.)] {
                let tip = point(c.x + px(dx * 51. * k), c.y + px(dy * 51. * k));
                let base = point(c.x + px(dx * 45. * k), c.y + px(dy * 45. * k));
                let (nx, ny) = (-dy * 4. * k, dx * 4. * k);
                p.move_to(tip);
                p.line_to(point(base.x + px(nx), base.y + px(ny)));
                p.line_to(point(base.x - px(nx), base.y - px(ny)));
                p.close();
            }
            if let Ok(path) = p.build() {
                window.paint_path(path, theme::rgb(150, 150, 145));
            }
        },
    )
    .absolute()
    .size_full();
    let plate = div()
        .relative()
        .size(z(138.))
        .rounded_full()
        .bg(linear_gradient(180., linear_color_stop(theme::rgb(226, 226, 222), 0.), linear_color_stop(theme::rgb(244, 244, 241), 1.)))
        .border_1()
        .border_color(theme::rgba(0, 0, 0, 0.08))
        .child(bar(102., 34.))
        .child(bar(34., 102.))
        .child(div().absolute().left(z(56.)).top(z(53.)).size(z(26.)).rounded_full().bg(theme::rgb(40, 40, 39)).border_1().border_color(theme::rgba(0, 0, 0, 0.5)))
        .child(arrows)
        .child(arm(0, -1, cx))
        .child(arm(0, 1, cx))
        .child(arm(-1, 0, cx))
        .child(arm(1, 0, cx));
    module(t, "52", "D-pad", 492., div().flex().items_center().gap(z(44.)).child(grid).child(plate))
}

/// 57 Slide switch: three slide switches, two upright and one across, each
/// with a ridged white lever on a black slot. Click a position to move it.
pub fn slide_switch_module<H: Host>(r: &Rack, ri: usize, t: &Theme, cx: &mut Context<H>) -> Div {
    let names: [(&str, [&str; 3]); 3] = [("Output", ["Main", "Cue", "Both"]), ("Quality", ["Lo", "Mid", "Hi"]), ("Input", ["Line", "Mic", "USB"])];
    let lever = |upright: bool| {
        let (w, h) = if upright { (26., 17.) } else { (17., 26.) };
        div()
            .w(z(w))
            .h(z(h))
            .rounded(z(4.))
            .flex()
            .when_flex(upright)
            .items_center()
            .justify_center()
            .gap(z(2.))
            .bg(linear_gradient(180., linear_color_stop(theme::rgb(255, 255, 255), 0.), linear_color_stop(theme::rgb(232, 232, 228), 1.)))
            .shadow(fx(vec![sh(0., 0., 0., 0.5, theme::rgba(0, 0, 0, 0.18)), sh(0., 2., 0., 0., theme::rgb(205, 205, 201)), sh(0., 4., 6., -1., theme::shade(0.25))]))
            .children((0..3).map(move |_| {
                let line = div().bg(theme::rgb(196, 196, 191));
                if upright { line.w(z(16.)).h(z(1.)) } else { line.w(z(1.)).h(z(16.)) }
            }))
    };
    let switch = |s: usize, cx: &mut Context<H>| {
        let (name, stops) = names[s];
        let upright = s != 1;
        let at = r.slides_at[s];
        let chosen = r.slides[s];
        let stop = |i: usize, cx: &mut Context<H>| {
            let on = i == chosen;
            let tick = div().bg(if on { t.ink.hsla() } else { theme::rgb(160, 160, 155) });
            let tick = if upright { tick.w(z(10.)).h(z(1.5)) } else { tick.w(z(1.5)).h(z(8.)) };
            let text = div()
                .text_size(z(9.))
                .font_weight(FontWeight::MEDIUM)
                .text_color(if on { theme::text() } else { theme::rgb(150, 150, 145) })
                .child(stops[i].to_uppercase());
            let item = div().flex().items_center().gap(z(8.)).cursor_pointer().on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this: &mut H, _, _, cx| {
                    this.rack_mut(ri).slides[s] = i;
                    cx.notify();
                }),
            );
            match s {
                0 => item.flex_row().justify_end().w(z(60.)).h(z(34.)).child(text).child(tick),
                2 => item.flex_row().w(z(60.)).h(z(34.)).child(tick).child(text),
                _ => item.flex_col().w(z(70.)).child(tick).child(text),
            }
        };
        let stops_el: Vec<Div> = (0..3).map(|i| stop(i, cx)).collect();
        if upright {
            let slot = div()
                .relative()
                .w(z(26.))
                .h(z(102.))
                .flex()
                .justify_center()
                .child(div().w(z(8.)).h_full().rounded(z(4.)).bg(theme::rgb(28, 28, 27)))
                .child(div().absolute().left_0().top(z(8.5 + at * 34. - 8.5)).child(lever(true)));
            let col = div().flex().flex_col().children(stops_el);
            let row = if s == 0 { div().flex().gap(z(6.)).child(col).child(slot) } else { div().flex().gap(z(6.)).child(slot).child(col) };
            div().flex().flex_col().gap(z(14.)).child(label(name)).child(row)
        } else {
            let slot = div()
                .relative()
                .w(z(210.))
                .h(z(26.))
                .flex()
                .items_center()
                .child(div().w_full().h(z(8.)).rounded(z(4.)).bg(theme::rgb(28, 28, 27)))
                .child(div().absolute().top_0().left(z(26. + at * 70. - 8.5)).child(lever(false)));
            div().flex().flex_col().gap(z(10.)).child(label(name)).child(slot).child(div().flex().ml(z(-9.)).children(stops_el))
        }
    };
    let body = div().w(z(456.)).flex().items_center().justify_between().child(switch(0, cx)).child(switch(1, cx)).child(switch(2, cx));
    module(t, "57", "Slide switch", 492., body)
}

trait FlexDir {
    fn when_flex(self, upright: bool) -> Self;
}

impl FlexDir for Div {
    /// Lever ridges run across the lever: stacked for an upright lever.
    fn when_flex(self, upright: bool) -> Self {
        if upright { self.flex_col() } else { self.flex_row() }
    }
}

/// 12 Tabs: keys in a dark frame; the chosen one raised and marked.
pub fn tabs_module<H: Host>(r: &Rack, ri: usize, t: &Theme, cx: &mut Context<H>) -> Div {
    const BODIES: [&str; 3] = [
        "Levels, pan and sends for every track on the current pattern.",
        "Trim, loop and reverse the selected sample. Non-destructive.",
        "Two effect slots per track: filters, delay and tape stop.",
    ];
    let keys = ["Mix", "Edit", "FX"].iter().enumerate().map(|(i, name)| {
        let on = r.tab == i;
        let key = div()
            .flex_1()
            .h(z(26.))
            .rounded(z(4.))
            .flex()
            .items_center()
            .justify_center()
            .cursor_pointer()
            .on_mouse_down(MouseButton::Left, cx.listener(move |this: &mut H, _, _, cx| {
                this.rack_mut(ri).tab = i;
                cx.notify();
            }));
        let key = if on {
            key.bg(linear_gradient(180., linear_color_stop(theme::rgb(255, 255, 255), 0.), linear_color_stop(theme::rgb(242, 242, 239), 1.)))
                .shadow(fx(vec![sh(0., 2., 0., 0., theme::rgb(205, 205, 201))]))
                .child(div().w(z(12.)).h(z(1.5)).bg(t.ink.hsla()))
        } else {
            key.bg(linear_gradient(180., linear_color_stop(theme::rgb(232, 232, 228), 0.), linear_color_stop(theme::rgb(218, 218, 214), 1.)))
                .child(div().w(z(12.)).h(z(1.5)).bg(theme::rgb(176, 176, 171)))
        };
        (key, name, on)
    });
    let (mut row, mut names) = (div().w(z(456.)).h(z(34.)).p(z(4.)).gap(z(4.)).flex().rounded(z(7.)).bg(theme::rgb(28, 28, 27)), div().w(z(456.)).flex());
    for (key, name, on) in keys {
        row = row.child(key);
        names = names.child(
            div()
                .flex_1()
                .flex()
                .justify_center()
                .text_size(z(7.5))
                .font_weight(FontWeight::MEDIUM)
                .text_color(if on { theme::text() } else { theme::dim() })
                .child(name.to_uppercase()),
        );
    }
    let body = div()
        .flex()
        .flex_col()
        .gap(z(8.))
        .child(row)
        .child(names)
        .child(div().pt(z(8.)).text_size(z(11.)).text_color(theme::rgb(90, 90, 86)).child(BODIES[r.tab]));
    module(t, "12", "Tabs", 492., body)
}
