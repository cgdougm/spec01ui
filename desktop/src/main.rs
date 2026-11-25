//! SPEC-01/HARDWARE in gpui: a native, GPU-rendered port of five components
//! of Shu Ding's hardware skin (https://github.com/shuding/spec-01).

// No console window behind the app in release builds on Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod gallery;
mod theme;
mod ui;
mod widgets;

use std::borrow::Cow;

use gpui::{App, AppContext, Application, Bounds, TitlebarOptions, WindowBounds, WindowOptions, px, size};

const FONTS: [&[u8]; 4] = [
    include_bytes!("../assets/fonts/IBMPlexMono-ExtraLight.ttf"),
    include_bytes!("../assets/fonts/IBMPlexMono-Light.ttf"),
    include_bytes!("../assets/fonts/IBMPlexMono-Regular.ttf"),
    include_bytes!("../assets/fonts/IBMPlexMono-Medium.ttf"),
];

fn main() {
    Application::new().run(|cx: &mut App| {
        cx.text_system()
            .add_fonts(FONTS.iter().map(|f| Cow::Borrowed(*f)).collect())
            .expect("embedded fonts load");

        let bounds = Bounds::centered(None, size(px(1064.), px(610.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions { title: Some("SPEC-01 / GPUI".into()), ..Default::default() }),
                window_min_size: Some(size(px(1064.), px(610.))),
                ..Default::default()
            },
            |_, cx| cx.new(|_| gallery::Gallery::new()),
        )
        .expect("open window");
        cx.activate(true);
    });
}
