//! SPEC-01/HARDWARE in gpui: a native, GPU-rendered port of five components
//! of Shu Ding's hardware skin (https://github.com/shuding/spec-01).

// No console window behind the app in release builds on Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::time::Instant;

use gpui::{
    App, AppContext, Application, Bounds, Context, FontWeight, InteractiveElement, IntoElement,
    MouseButton, MouseMoveEvent, ParentElement, Render, Styled, TitlebarOptions, Window,
    WindowBounds, WindowOptions, div, px, size,
};
use spec01::{
    rack::{Drag, Host, Rack},
    rack_view,
    theme::{self, KEYS, Theme},
    ui::{self, FONT},
};

struct Gallery {
    key: usize,
    theme: Theme,
    rack: Rack,
    drag: Drag,
    last_frame: Instant,
}

impl Host for Gallery {
    fn rack_mut(&mut self, _: usize) -> &mut Rack {
        &mut self.rack
    }
    fn drag_mut(&mut self) -> &mut Drag {
        &mut self.drag
    }
    fn set_key(&mut self, key: usize) {
        self.key = key;
        self.theme = Theme::new(KEYS[key].1);
    }
}

impl Render for Gallery {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let now = Instant::now();
        let dt = (now - self.last_frame).as_secs_f32().min(0.05);
        self.last_frame = now;
        if self.rack.ease(1. - (-dt * 18.).exp()) {
            window.request_animation_frame();
        }
        let t = self.theme;
        let title = div()
            .flex()
            .items_center()
            .gap(px(8.))
            .text_size(px(10.))
            .font_weight(FontWeight::MEDIUM)
            .child(ui::led(&t, 5.))
            .child("SPEC-01 / GPUI");

        div()
            .size_full()
            .bg(theme::page())
            .font_family(FONT)
            .text_color(theme::text())
            .flex()
            .flex_col()
            .items_center()
            .pt(px(12.))
            .on_mouse_move(cx.listener(|this, e: &MouseMoveEvent, _, cx| {
                if this.drag_move(e) {
                    cx.notify();
                }
            }))
            .on_mouse_up(MouseButton::Left, cx.listener(|this, _, _, cx| {
                this.drag = Drag::None;
                this.rack.pressed = None;
                cx.notify();
            }))
            .child(div().w(px(1000.)).child(ui::navbar(self.key, title, cx)))
            .child(div().mt(px(28.)).child(rack_view(&self.rack, 0, &t, cx)))
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        spec01::load_fonts(cx);
        let bounds = Bounds::centered(None, size(px(1064.), px(640.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions { title: Some("SPEC-01 / GPUI".into()), ..Default::default() }),
                window_min_size: Some(size(px(1064.), px(640.))),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|_| Gallery { key: 0, theme: Theme::new(KEYS[0].1), rack: Rack::default(), drag: Drag::None, last_frame: Instant::now() })
            },
        )
        .expect("open window");
        cx.activate(true);
    });
}
