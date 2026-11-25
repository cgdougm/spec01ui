//! Screens and readouts: Meter, Progress, Bar graph, Text field, LCD, Item
//! list, Calendar, Display, Alert.

use std::sync::{Arc, OnceLock};

use gpui::{
    Bounds, Context, Div, FontWeight, Image, ImageFormat, InteractiveElement, MouseButton,
    ObjectFit, ParentElement, Pixels, Styled, StyledImage, canvas, div, fill, img,
    linear_color_stop, linear_gradient, point, size,
};

use crate::{
    rack::{Host, Rack},
    theme::{self, Theme, oklch},
    ui::{cells, fx, dark_glass, label, lcd_glass, lod, module, now, sh, visible, z, zoom},
};

/// A row of round lamps in a pill track, lit up to `value` (0..100).
fn dot_meter(t: &Theme, name: &str, value: f32, dots: usize) -> Div {
    let lit = ((value / 100.) * dots as f32).round() as usize;
    let track = div()
        .w(z(200.))
        .h(z(18.))
        .px(z(7.))
        .rounded(z(9.))
        .bg(theme::rgb(236, 236, 233))
        .border_1()
        .border_color(theme::rgba(0, 0, 0, 0.07))
        .child({
            let (on, off) = (t.lamp.hsla(), theme::rgb(206, 206, 202));
            let pitch = 186. / (dots as f32 - 1.) - 0.01;
            cells(dots, 1, (6., 6.), (pitch, 0.), 3., move |c, _| if c < lit { (on, true) } else { (off, false) }, t.lamp.alpha(0.35)).size_full()
        });
    let head = div()
        .w(z(200.))
        .flex()
        .justify_between()
        .items_end()
        .child(div().text_size(z(10.)).font_weight(FontWeight::MEDIUM).text_color(theme::text()).child(name.to_uppercase()))
        .child(div().text_size(z(10.)).font_weight(FontWeight::MEDIUM).text_color(theme::dim()).child(format!("{}%", value as i32)));
    div().flex().flex_col().gap(z(10.)).child(head).child(track)
}

/// 09 Meter.
pub fn meter_module<H: Host>(r: &Rack, _ri: usize, t: &Theme, _cx: &mut Context<H>) -> Div {
    module(t, "09", "Meter", 238., dot_meter(t, "CPU", r.meter, 20))
}

/// 37 Progress.
pub fn progress_module<H: Host>(r: &Rack, _ri: usize, t: &Theme, _cx: &mut Context<H>) -> Div {
    module(t, "37", "Progress", 238., dot_meter(t, "Uploading", r.progress, 19))
}

/// 44 Bar graph: twelve LED columns of fourteen segments on a black screen,
/// drawn as quads straight onto the GPU, no layout.
pub fn bars_module<H: Host>(r: &Rack, _ri: usize, t: &Theme, _cx: &mut Context<H>) -> Div {
    let bars = r.bars;
    let (on, off, glow) = (t.lamp.hsla(), theme::rgb(44, 44, 42), t.lamp.alpha(0.35));
    let screen = dark_glass(200., 92.).child(
        canvas(
            |_, _, _| {},
            move |b: Bounds<Pixels>, _, window, _| {
                if !visible(b, window) {
                    return;
                }
                // Per column: the unlit stack, the lit part over it (with a
                // halo), then the gaps between segments painted across all
                // columns at once in the screen's black. ~50 quads, not 336.
                let k = zoom();
                let (cols, rows) = (12usize, 14usize);
                let (x0, y0) = (b.origin.x + Pixels::from(13. * k), b.origin.y + Pixels::from(12. * k));
                let (cw, ch) = (14.5 * k, 4.9 * k);
                let (bw, total) = (cw - 3. * k, ch * rows as f32);
                for c in 0..cols {
                    let x = x0 + Pixels::from(c as f32 * cw);
                    window.paint_quad(fill(Bounds::new(point(x, y0), size(Pixels::from(bw), Pixels::from(total))), off));
                    let lit = (bars[c] * rows as f32).round();
                    if lit > 0. {
                        let top = y0 + Pixels::from(total - lit * ch);
                        let lit_b = Bounds::new(point(x, top), size(Pixels::from(bw), Pixels::from(lit * ch)));
                        if lod() > 0 {
                            window.paint_quad(fill(lit_b.dilate(Pixels::from(1.2 * k)), glow));
                        }
                        window.paint_quad(fill(lit_b, on));
                    }
                }
                let gap = theme::rgb(22, 22, 21);
                for s in 1..rows {
                    let y = y0 + Pixels::from(s as f32 * ch - 1.8 * k);
                    window.paint_quad(fill(Bounds::new(point(x0 - Pixels::from(2. * k), y), size(Pixels::from(cols as f32 * cw + 2. * k), Pixels::from(1.8 * k))), gap));
                }
            },
        )
        .size_full(),
    );
    module(t, "44", "Bar graph", 238., screen)
}

/// 07 Text field: a recessed black field with a blinking block caret.
pub fn text_field_module<H: Host>(_r: &Rack, ri: usize, t: &Theme, _cx: &mut Context<H>) -> Div {
    let blink = (now() * 1.8 + ri as f32 * 0.13).fract() < 0.55;
    let field = div()
        .w(z(200.))
        .h(z(40.))
        .px(z(14.))
        .rounded(z(10.))
        .bg(theme::rgb(16, 16, 15))
        .shadow(fx(vec![sh(0., 1., 0., 0., theme::rgb(255, 255, 255)), sh(0., 0., 0., 2., theme::rgba(0, 0, 0, 0.05))]))
        .flex()
        .items_center()
        .text_size(z(13.))
        .text_color(theme::rgb(240, 240, 236))
        .child("untitled_04")
        .child(div().ml(z(1.)).w(z(8.)).h(z(16.)).bg(if blink { t.lamp.hsla() } else { theme::rgba(0, 0, 0, 0.) }));
    module(t, "07", "Text field", 238., div().flex().flex_col().gap(z(10.)).child(label("Project")).child(field))
}

/// An LCD readout: a small tag over a value, with unlit 8s ghosted behind.
fn lcd(tag: &str, value: String, ghost: &str, w: f32, h: f32, big: f32) -> Div {
    let ink = theme::rgb(28, 29, 27);
    let digits = div()
        .relative()
        .text_size(z(big))
        .font_weight(FontWeight::NORMAL)
        .child(div().absolute().top_0().left_0().text_color(theme::rgba(0, 0, 0, 0.07)).child(ghost.to_string()))
        .child(div().text_color(ink).child(value));
    lcd_glass(w, h)
        .px(z(13.))
        .py(z(9.))
        .flex()
        .flex_col()
        .justify_between()
        .child(div().text_size(z(7.5)).font_weight(FontWeight::MEDIUM).text_color(theme::rgb(70, 71, 67)).child(tag.to_uppercase()))
        .child(digits)
}

/// 56 LCD: four segment-free readouts; the timer runs.
pub fn lcd_module<H: Host>(r: &Rack, ri: usize, t: &Theme, _cx: &mut Context<H>) -> Div {
    let secs = (now() + ri as f32 * 7.) as i32 + 272;
    let body = div()
        .flex()
        .flex_col()
        .items_center()
        .gap(z(18.))
        .child(
            div()
                .flex()
                .items_center()
                .gap(z(14.))
                .child(lcd("Tempo", format!("{:.1}", r.bpm), "888.8", 108., 62., 24.))
                .child(lcd("Status", if r.power { "Synced to MIDI".into() } else { "Free running".into() }, "", 180., 50., 12.))
                .child(lcd("Battery", format!("{}%", (r.usage as i32).clamp(1, 99)), "88%", 64., 40., 13.)),
        )
        .child(lcd("Timer", format!("{:02}:{:02}", (secs / 60) % 100, secs % 60), "88:88", 186., 90., 44.));
    module(t, "56", "LCD", 492., body)
}

/// 51 Item list: a table on an LCD; the chosen row printed in the key ink.
pub fn item_list_module<H: Host>(r: &Rack, ri: usize, t: &Theme, cx: &mut Context<H>) -> Div {
    const ROWS: [[&str; 5]; 4] = [
        ["d41", "web", "main", "Ready", "2m"],
        ["d40", "api", "main", "Ready", "18m"],
        ["d39", "web", "fix/nav", "Error", "1h"],
        ["d38", "docs", "main", "Ready", "3h"],
    ];
    // The font is monospaced, so columns are just padding: one text run
    // per row instead of an element per cell.
    let fmt = |c: &[&str; 5], mark: &str| format!("{mark} {:<6}{:<12}{:<12}{:<9}{:>4} {mark}", c[0], c[1], c[2], c[3], c[4]);
    let head = div()
        .text_size(z(7.5))
        .font_weight(FontWeight::MEDIUM)
        .text_color(theme::rgb(70, 71, 67))
        .child(format!("  {:<9}{:<18}{:<18}{:<13}{:>5}  ", "ID", "PROJECT", "BRANCH", "STATUS", "AGE"));
    let rows = ROWS.iter().enumerate().map(|(i, cells)| {
        let on = r.row == i;
        div()
            .h(z(26.))
            .flex()
            .items_center()
            .cursor_pointer()
            .text_size(z(11.))
            .font_weight(if on { FontWeight::BOLD } else { FontWeight::NORMAL })
            .text_color(if on { t.ink.hsla() } else { theme::rgb(82, 83, 79) })
            .child(fmt(cells, if on { "—" } else { " " }))
            .on_mouse_down(MouseButton::Left, cx.listener(move |this: &mut H, _, _, cx| {
                this.rack_mut(ri).row = i;
                cx.notify();
            }))
    });
    let screen = lcd_glass(456., 160.)
        .px(z(8.))
        .py(z(12.))
        .flex()
        .flex_col()
        .items_center()
        .child(div().pb(z(6.)).w_full().flex().justify_center().border_b_1().border_dashed().border_color(theme::rgba(0, 0, 0, 0.3)).child(head))
        .child(div().pt(z(4.)).flex().flex_col().children(rows));
    module(t, "51", "Item list", 492., screen)
}

/// 13 Calendar: a date on a black LCD, with a knurled jog wheel to set it.
pub fn calendar_module<H: Host>(r: &Rack, ri: usize, t: &Theme, cx: &mut Context<H>) -> Div {
    let weekday = ["SAT", "SUN", "MON", "TUE", "WED", "THU", "FRI"][(r.day as usize + 6) % 7];
    let field = |s: String, on: bool| {
        let d = div().px(z(3.)).rounded(z(2.)).child(s);
        if on { d.bg(t.lamp.hsla()).text_color(t.on_accent) } else { d }
    };
    let (day, on, off) = (r.day as usize, t.lamp.hsla(), theme::rgb(58, 58, 56));
    let grid = div().w(z(54.)).h(z(36.)).child(cells(7, 5, (5., 5.), (7., 7.), 0., move |c, row| if row * 7 + c + 1 == day { (on, true) } else { (off, false) }, t.lamp.alpha(0.4)).size_full());
    let screen = dark_glass(300., 112.)
        .px(z(16.))
        .py(z(12.))
        .flex()
        .flex_col()
        .gap(z(6.))
        .child(
            div()
                .flex()
                .justify_between()
                .text_size(z(7.5))
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme::rgb(150, 150, 145))
                .child("SET DATE")
                .child(weekday),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap(z(6.))
                .text_size(z(26.))
                .font_weight(FontWeight::LIGHT)
                .text_color(theme::rgb(236, 236, 230))
                .child(field("2026".into(), false))
                .child(div().text_color(theme::rgb(110, 110, 105)).child("·"))
                .child(field("11".into(), false))
                .child(div().text_color(theme::rgb(110, 110, 105)).child("·"))
                .child(field(format!("{:02}", r.day), true)),
        )
        .child(grid);
    let jog = div()
        .size(z(104.))
        .rounded_full()
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .bg(linear_gradient(180., linear_color_stop(theme::rgb(250, 250, 248), 0.), linear_color_stop(theme::rgb(226, 226, 222), 1.)))
        .shadow(fx(vec![sh(0., 0., 0., 1., theme::shade(0.08)), sh(0., 3., 0., 0., theme::rgb(214, 214, 210)), sh(0., 8., 14., -4., theme::shade(0.2))]))
        .child(
            div()
                .size(z(46.))
                .rounded_full()
                .flex()
                .items_center()
                .justify_center()
                .bg(linear_gradient(180., linear_color_stop(theme::rgb(64, 64, 62), 0.), linear_color_stop(theme::rgb(30, 30, 29), 1.)))
                .shadow(fx(vec![sh(0., 3., 0., 0., theme::rgb(12, 12, 11)), sh(0., 5., 8., -2., theme::rgba(0, 0, 0, 0.4))]))
                .text_size(z(8.))
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme::rgb(236, 236, 230))
                .child("SET"),
        )
        .on_mouse_down(MouseButton::Left, cx.listener(move |this: &mut H, _, _, cx| {
            let r = this.rack_mut(ri);
            r.day = r.day % 30 + 1;
            cx.notify();
        }));
    module(t, "13", "Calendar", 492., div().flex().items_center().gap(z(20.)).child(screen).child(jog))
}

/// The two photos the original kit ships (Unsplash License, via Lorem Picsum).
fn photo(i: usize) -> Arc<Image> {
    static PHOTOS: OnceLock<[Arc<Image>; 2]> = OnceLock::new();
    PHOTOS.get_or_init(|| {
        [
            Arc::new(Image::from_bytes(ImageFormat::Jpeg, include_bytes!("../../assets/photos/highlands.jpg").to_vec())),
            Arc::new(Image::from_bytes(ImageFormat::Jpeg, include_bytes!("../../assets/photos/river.jpg").to_vec())),
        ]
    })[i % 2]
        .clone()
}

/// 45 Display: a photo on a screen in a black bezel.
pub fn display_module<H: Host>(r: &Rack, _ri: usize, t: &Theme, _cx: &mut Context<H>) -> Div {
    let bezel = div()
        .w(z(456.))
        .h(z(196.))
        .p(z(10.))
        .rounded(z(18.))
        .bg(linear_gradient(180., linear_color_stop(theme::rgb(34, 34, 33), 0.), linear_color_stop(theme::rgb(14, 14, 13), 1.)))
        .shadow(fx(vec![sh(0., 1., 0., 0., theme::rgb(255, 255, 255)), sh(0., 10., 20., -8., theme::shade(0.35))]))
        .overflow_hidden()
        .child(img(photo(r.photo)).w(z(436.)).h(z(176.)).rounded(z(9.)).object_fit(ObjectFit::Cover));
    let caption = div().text_size(z(9.)).font_weight(FontWeight::MEDIUM).text_color(theme::dim()).child(if r.photo % 2 == 0 { "HIGHLANDS" } else { "RIVER" });
    module(t, "45", "Display", 492., div().flex().flex_col().items_center().gap(z(12.)).child(bezel).child(caption))
}

/// 33 Alert: a legend in a bracketed frame, its lamp amber or red.
pub fn alert_module<H: Host>(r: &Rack, _ri: usize, t: &Theme, _cx: &mut Context<H>) -> Div {
    let hot = r.usage >= 90.;
    let lamp = if hot { oklch(0.62, 0.24, 25.) } else { oklch(0.8, 0.16, 75.) };
    let rule = || div().flex_1().h(z(1.5)).bg(theme::rgb(176, 176, 171));
    let tick = || div().w(z(1.5)).h(z(6.)).bg(theme::rgb(176, 176, 171));
    let top = div()
        .w(z(456.))
        .flex()
        .items_start()
        .child(tick())
        .child(rule())
        .child(
            div()
                .px(z(12.))
                .mt(z(-5.))
                .flex()
                .items_center()
                .gap(z(8.))
                .child(div().size(z(6.)).rounded_full().bg(lamp.hsla()).shadow(fx(vec![sh(0., 0., 5., 0., lamp.alpha(0.8))])))
                .child(div().text_size(z(9.)).font_weight(FontWeight::MEDIUM).text_color(theme::text()).child(format!("USAGE AT {}%", r.usage as i32))),
        )
        .child(rule())
        .child(tick());
    let bottom = div().w(z(456.)).flex().items_end().child(tick()).child(rule()).child(tick());
    let days = ((100. - r.usage) / 4.5).max(0.).round() as i32;
    let text = div()
        .py(z(16.))
        .text_size(z(12.))
        .text_color(theme::rgb(110, 110, 105))
        .child(if days == 0 { "You've reached your plan's limit.".to_string() } else { format!("You'll reach your plan's limit in about {days} days.") });
    module(t, "33", "Alert", 492., div().flex().flex_col().items_center().child(top).child(text).child(bottom))
}
