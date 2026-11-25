//! Colors. SPEC-01 defines its key color as OKLCH and derives every themed
//! color from it (tokens.css); this does the same math, then converts to sRGB
//! for gpui, clipping out-of-gamut colors the way the browser does on an sRGB
//! display.

use gpui::{Hsla, Rgba};

/// An OKLCH color: lightness 0..1, chroma, hue in degrees.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Oklch {
    pub l: f32,
    pub c: f32,
    pub h: f32,
}

pub const fn oklch(l: f32, c: f32, h: f32) -> Oklch {
    Oklch { l, c, h }
}

impl Oklch {
    pub fn with_l(self, l: f32) -> Self {
        Self { l, ..self }
    }

    pub fn rgba(self, alpha: f32) -> Rgba {
        let (a, b) = {
            let h = self.h.to_radians();
            (self.c * h.cos(), self.c * h.sin())
        };
        let l_ = self.l + 0.396_337_78 * a + 0.215_803_76 * b;
        let m_ = self.l - 0.105_561_346 * a - 0.063_854_17 * b;
        let s_ = self.l - 0.089_484_18 * a - 1.291_485_5 * b;
        let (l, m, s) = (l_.powi(3), m_.powi(3), s_.powi(3));
        let r = 4.076_741_7 * l - 3.307_711_6 * m + 0.230_969_94 * s;
        let g = -1.268_438 * l + 2.609_757_4 * m - 0.341_319_38 * s;
        let bl = -0.004_196_086_3 * l - 0.703_418_6 * m + 1.707_614_7 * s;
        let enc = |x: f32| {
            let x = x.clamp(0.0, 1.0);
            if x <= 0.003_130_8 { 12.92 * x } else { 1.055 * x.powf(1.0 / 2.4) - 0.055 }
        };
        Rgba { r: enc(r), g: enc(g), b: enc(bl), a: alpha }
    }

    pub fn hsla(self) -> Hsla {
        self.rgba(1.0).into()
    }

    pub fn alpha(self, a: f32) -> Hsla {
        self.rgba(a).into()
    }
}

/// Plain sRGB with alpha, for the skin's fixed neutrals.
pub fn rgb(r: u8, g: u8, b: u8) -> Hsla {
    rgba(r, g, b, 1.0)
}

pub fn rgba(r: u8, g: u8, b: u8, a: f32) -> Hsla {
    Rgba { r: r as f32 / 255.0, g: g as f32 / 255.0, b: b as f32 / 255.0, a }.into()
}

/// The six keys of the site's key picker.
pub const KEYS: [(&str, Oklch); 6] = [
    ("Orange-red", oklch(0.64, 0.31, 48.0)),
    ("Yellow", oklch(0.94, 0.23, 105.21)),
    ("Green", oklch(0.7, 0.29, 147.6)),
    ("Blue", oklch(0.56, 0.37, 260.0)),
    ("Violet", oklch(0.56, 0.3, 296.49)),
    ("Black", oklch(0.24, 0.0, 0.0)),
];

/// The themed colors, as tokens.css works them out from the one key color.
#[derive(Clone, Copy)]
pub struct Theme {
    /// --key-color: key caps, fills, anything large.
    pub key: Oklch,
    /// --key-ink: text and thin marks on light panels.
    pub ink: Oklch,
    /// --key-lamp: lit LEDs.
    pub lamp: Oklch,
    /// --key-cap: a fill carrying a small legend (the mode dial's chosen chip).
    pub cap: Oklch,
    /// --on-accent: text on a key cap.
    pub on_accent: Hsla,
}

impl Theme {
    pub fn new(key: Oklch) -> Self {
        let light = key.l > 0.68;
        let ink = if light { oklch(0.22, 0.01, key.h) } else { key.with_l(key.l.min(0.62)) };
        Theme {
            key,
            ink,
            lamp: key.with_l(key.l.min(0.86)),
            cap: if light { key } else { key.with_l(key.l.min(0.55)) },
            on_accent: if key.l < 0.8 { rgb(255, 255, 255) } else { ink.hsla() },
        }
    }
}

// The skin's neutrals.
pub fn page() -> Hsla { rgb(238, 238, 236) }
pub fn panel() -> Hsla { rgb(251, 251, 250) }
pub fn text() -> Hsla { rgb(29, 29, 28) }
pub fn dim() -> Hsla { rgb(143, 141, 135) }
pub fn print() -> Hsla { rgb(106, 105, 100) }
pub fn shade(a: f32) -> Hsla { rgba(83, 80, 70, a) }
