//! SPEC-01/HARDWARE components in gpui, shared by the gallery (`spec01`) and
//! the stress test (`stress`).

pub mod rack;
pub mod theme;
pub mod ui;
pub mod widgets;

/// Hybrid-GPU hints, exported from the executables by build.rs: 1 asks the
/// NVIDIA / AMD driver to run the app on the discrete GPU.
pub mod gpu {
    #[unsafe(no_mangle)]
    #[used]
    pub static NvOptimusEnablement: u32 = 1;
    #[unsafe(no_mangle)]
    #[used]
    pub static AmdPowerXpressRequestHighPerformance: u32 = 1;
}

use std::borrow::Cow;

use gpui::{App, Div, ParentElement, Styled, div};

use rack::{Host, Rack};
use theme::Theme;
use ui::z;

const FONTS: [&[u8]; 4] = [
    include_bytes!("../assets/fonts/IBMPlexMono-ExtraLight.ttf"),
    include_bytes!("../assets/fonts/IBMPlexMono-Light.ttf"),
    include_bytes!("../assets/fonts/IBMPlexMono-Regular.ttf"),
    include_bytes!("../assets/fonts/IBMPlexMono-Medium.ttf"),
];

/// Registers the embedded IBM Plex Mono weights.
pub fn load_fonts(cx: &mut App) {
    cx.text_system()
        .add_fonts(FONTS.iter().map(|f| Cow::Borrowed(*f)).collect())
        .expect("embedded fonts load");
}

/// Height of a rack laid out by [`rack_view`].
pub const RACK_HEIGHT: f32 = 190. + 16. + 272.;

/// The five modules of one rack, in two rows, 1000px wide.
pub fn rack_view<H: Host>(r: &Rack, ri: usize, t: &Theme, cx: &mut gpui::Context<H>) -> Div {
    let row = || div().flex().items_start().gap(z(16.));
    div()
        .w(z(1000.))
        .flex()
        .flex_col()
        .gap(z(16.))
        .child(
            row()
                .child(widgets::button::button_module(r, ri, t, cx))
                .child(widgets::switch::switch_module(r, ri, t, cx))
                .child(widgets::slider::slider_module(r, ri, t, cx)),
        )
        .child(
            row()
                .child(widgets::knob::knob_module(r, ri, t, cx).h(z(272.)))
                .child(widgets::mode_dial::mode_dial_module(r, ri, t, cx).h(z(272.))),
        )
}
