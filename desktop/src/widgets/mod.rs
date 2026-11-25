//! The components, and a catalogue of them ([`Kind`]) for views that lay
//! out many: each kind's name, panel size, and how to draw it.

pub mod button;
pub mod controls;
pub mod keys;
pub mod knob;
pub mod mode_dial;
pub mod screens;
pub mod slider;
pub mod switch;

use gpui::{Context, Div, Styled};

use crate::{
    rack::{Host, Rack},
    theme::Theme,
    ui::z,
};

/// Panel widths in the skin: one column or two.
pub const NARROW: f32 = 238.;
pub const WIDE: f32 = 492.;
pub const GAP: f32 = 16.;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub enum Kind {
    Button,
    Switch,
    Checkbox,
    Segmented,
    Slider,
    Knob,
    TextField,
    Meter,
    Tabs,
    Calendar,
    Radio,
    Stepper,
    Badge,
    Progress,
    Alert,
    Toggles,
    Bars,
    Display,
    ItemList,
    DPad,
    ModeDial,
    Lcd,
    SlideSwitch,
}

impl Kind {
    pub const ALL: [Kind; 23] = [
        Kind::Button,
        Kind::Switch,
        Kind::Checkbox,
        Kind::Segmented,
        Kind::Slider,
        Kind::Knob,
        Kind::TextField,
        Kind::Meter,
        Kind::Tabs,
        Kind::Calendar,
        Kind::Radio,
        Kind::Stepper,
        Kind::Badge,
        Kind::Progress,
        Kind::Alert,
        Kind::Toggles,
        Kind::Bars,
        Kind::Display,
        Kind::ItemList,
        Kind::DPad,
        Kind::ModeDial,
        Kind::Lcd,
        Kind::SlideSwitch,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Kind::Button => "Button",
            Kind::Switch => "Switch",
            Kind::Checkbox => "Checkbox",
            Kind::Segmented => "Segmented control",
            Kind::Slider => "Slider",
            Kind::Knob => "Flat knob",
            Kind::TextField => "Text field",
            Kind::Meter => "Meter",
            Kind::Tabs => "Tabs",
            Kind::Calendar => "Calendar",
            Kind::Radio => "Radio group",
            Kind::Stepper => "Number stepper",
            Kind::Badge => "Badge",
            Kind::Progress => "Progress",
            Kind::Alert => "Alert",
            Kind::Toggles => "Toggle group",
            Kind::Bars => "Bar graph",
            Kind::Display => "Display",
            Kind::ItemList => "Item list",
            Kind::DPad => "D-pad",
            Kind::ModeDial => "Mode dial",
            Kind::Lcd => "LCD",
            Kind::SlideSwitch => "Slide switch",
        }
    }

    pub fn wide(self) -> bool {
        matches!(
            self,
            Kind::Slider | Kind::Knob | Kind::Tabs | Kind::Calendar | Kind::Alert | Kind::Display | Kind::ItemList | Kind::DPad | Kind::ModeDial | Kind::Lcd | Kind::SlideSwitch
        )
    }

    /// Panel width in skin units.
    pub fn width(self) -> f32 {
        if self.wide() { WIDE } else { NARROW }
    }

    /// Panel height in skin units.
    pub fn height(self) -> f32 {
        match self {
            Kind::Bars | Kind::ModeDial | Kind::DPad | Kind::ItemList => 272.,
            Kind::Lcd | Kind::Display | Kind::SlideSwitch => 300.,
            _ => 190.,
        }
    }

    /// The kind's panel, sized, for rack `ri`.
    pub fn render<H: Host>(self, r: &Rack, ri: usize, t: &Theme, cx: &mut Context<H>) -> Div {
        let panel = match self {
            Kind::Button => button::button_module(r, ri, t, cx),
            Kind::Switch => switch::switch_module(r, ri, t, cx),
            Kind::Checkbox => keys::checkbox_module(r, ri, t, cx),
            Kind::Segmented => keys::segmented_module(r, ri, t, cx),
            Kind::Slider => slider::slider_module(r, ri, t, cx),
            Kind::Knob => knob::knob_module(r, ri, t, cx),
            Kind::TextField => screens::text_field_module(r, ri, t, cx),
            Kind::Meter => screens::meter_module(r, ri, t, cx),
            Kind::Tabs => controls::tabs_module(r, ri, t, cx),
            Kind::Calendar => screens::calendar_module(r, ri, t, cx),
            Kind::Radio => keys::radio_module(r, ri, t, cx),
            Kind::Stepper => keys::stepper_module(r, ri, t, cx),
            Kind::Badge => keys::badge_module(r, ri, t, cx),
            Kind::Progress => screens::progress_module(r, ri, t, cx),
            Kind::Alert => screens::alert_module(r, ri, t, cx),
            Kind::Toggles => keys::toggles_module(r, ri, t, cx),
            Kind::Bars => screens::bars_module(r, ri, t, cx),
            Kind::Display => screens::display_module(r, ri, t, cx),
            Kind::ItemList => screens::item_list_module(r, ri, t, cx),
            Kind::DPad => controls::dpad_module(r, ri, t, cx),
            Kind::ModeDial => mode_dial::mode_dial_module(r, ri, t, cx),
            Kind::Lcd => screens::lcd_module(r, ri, t, cx),
            Kind::SlideSwitch => controls::slide_switch_module(r, ri, t, cx),
        };
        panel.w(z(self.width())).h(z(self.height())).flex_none()
    }
}
