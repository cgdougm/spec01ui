//! A rack: the state of one instance of every component. The gallery shows
//! one rack as five modules; the stress test gives each widget on its wall
//! a rack of its own and shows one module from it. Views that host racks
//! implement [`Host`], which the components' event handlers use.

use std::{cell::Cell, rc::Rc};

use gpui::{Bounds, MouseButton, MouseMoveEvent, Pixels};

use crate::widgets::{button::Key, mode_dial::Dial};

pub struct Rack {
    pub pressed: Option<Key>,
    pub power: bool,
    /// The switch thumb's drawn position, easing toward `power`.
    pub switch_at: f32,
    pub volume: f32,
    pub slider_bounds: Rc<Cell<Bounds<Pixels>>>,
    pub knobs: [f32; 4],
    pub dials: [Dial; 2],
    /// Latched checkbox keys: SYNC, LOOP, METRO.
    pub checks: [bool; 3],
    pub segment: usize,
    pub segment_at: f32,
    /// CPU meter and upload progress, 0..100.
    pub meter: f32,
    pub progress: f32,
    pub radio: usize,
    pub tempo: i32,
    /// Toggle group: which alignment is pressed.
    pub align: usize,
    /// Bar graph levels, 0..1.
    pub bars: [f32; 12],
    pub bpm: f32,
    /// D-pad position on its 5x3 grid.
    pub cell: (i32, i32),
    pub slides: [usize; 3],
    pub slides_at: [f32; 3],
    pub tab: usize,
    pub row: usize,
    pub day: u32,
    pub photo: usize,
    pub usage: f32,
}

impl Default for Rack {
    fn default() -> Self {
        Rack {
            pressed: None,
            power: true,
            switch_at: 1.,
            volume: 64.,
            slider_bounds: Rc::default(),
            knobs: [30., 72., 50., 18.],
            dials: [
                Dial::new("Effort", &["low", "medium", "high", "xhigh", "max"], 240., 2),
                Dial::new("Bank", &["1", "2", "3", "4", "5"], 180., 2),
            ],
            checks: [true, false, false],
            segment: 0,
            segment_at: 0.,
            meter: 68.,
            progress: 42.,
            radio: 0,
            tempo: 120,
            align: 0,
            bars: [0.08, 0.6, 0.98, 0.85, 0.92, 0.48, 0.65, 0.48, 0.98, 0.12, 0.6, 0.55],
            bpm: 128.,
            cell: (2, 1),
            slides: [1, 1, 0],
            slides_at: [1., 1., 0.],
            tab: 0,
            row: 1,
            day: 14,
            photo: 0,
            usage: 82.,
        }
    }
}

/// A cheap repeatable hash: 0..1 from an integer.
pub fn hash01(n: usize) -> f32 {
    let mut x = n as u64 ^ 0x9E37_79B9_7F4A_7C15;
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    ((x ^ (x >> 31)) % 10_000) as f32 / 10_000.
}

impl Rack {
    /// A rack with its controls set somewhere different for each `seed`, so
    /// a long list doesn't look like one rack repeated.
    pub fn varied(seed: usize) -> Self {
        let mut r = Rack::default();
        let f = |k: usize| hash01(seed * 31 + k);
        r.drive(f(0), f(1) * 100.);
        r.photo = seed % 2;
        r.settle();
        r
    }

    /// Sets every control from one value `v` (0..1) and a time `t`, each
    /// part a little out of phase: the motion modes use this.
    pub fn drive(&mut self, v: f32, t: f32) {
        let w = |k: f32| 0.5 + 0.5 * (v * std::f32::consts::TAU + k).sin();
        self.volume = (w(0.) * 100.).round();
        for k in 0..4 {
            self.knobs[k] = (w(k as f32 * 0.9) * 100.).round();
        }
        self.power = w(1.7) > 0.5;
        self.dials[0].chosen = ((w(0.4) * 4.99) as usize).min(4);
        self.dials[1].chosen = ((w(2.4) * 4.99) as usize).min(4);
        self.checks = [w(0.3) > 0.5, w(1.3) > 0.5, w(2.3) > 0.5];
        self.segment = ((v * 3.99) as usize).min(3);
        self.meter = (w(0.2) * 100.).round();
        self.progress = (v * 100.).round();
        self.radio = ((w(1.1) * 2.99) as usize).min(2);
        self.tempo = 60 + (w(0.6) * 120.) as i32;
        self.align = ((w(2.9) * 2.99) as usize).min(2);
        for (i, b) in self.bars.iter_mut().enumerate() {
            let p = i as f32 * 0.55;
            *b = (0.5 + 0.5 * (t * 3.1 + p + v * 6.).sin() * (t * 1.3 + p * 0.7).cos()).clamp(0.04, 1.);
        }
        self.bpm = 60. + w(0.8) * 120.;
        self.cell = (((w(0.1) * 4.99) as i32).min(4), ((w(3.1) * 2.99) as i32).min(2));
        self.slides = [((w(0.5) * 2.99) as usize).min(2), ((w(1.5) * 2.99) as usize).min(2), ((w(2.5) * 2.99) as usize).min(2)];
        self.tab = ((w(1.9) * 2.99) as usize).min(2);
        self.row = ((w(0.7) * 3.99) as usize).min(3);
        self.day = 1 + ((w(2.2) * 29.99) as u32).min(29);
        self.usage = (w(1.4) * 100.).round();
    }

    /// Jumps animated values straight to their targets.
    pub fn settle(&mut self) {
        self.switch_at = if self.power { 1. } else { 0. };
        self.segment_at = self.segment as f32;
        self.slides_at = self.slides.map(|s| s as f32);
        for d in &mut self.dials {
            d.at = d.angle(d.chosen);
        }
    }

    /// Eases animated values toward their targets by `k` (0..1); true while
    /// any is still moving.
    pub fn ease(&mut self, k: f32) -> bool {
        let mut moving = false;
        let mut ease = |x: &mut f32, to: f32, eps: f32| {
            if (*x - to).abs() < eps {
                *x = to;
            } else {
                *x += (to - *x) * k;
                moving = true;
            }
        };
        ease(&mut self.switch_at, if self.power { 1. } else { 0. }, 0.002);
        ease(&mut self.segment_at, self.segment as f32, 0.002);
        for i in 0..3 {
            ease(&mut self.slides_at[i], self.slides[i] as f32, 0.002);
        }
        for d in &mut self.dials {
            let to = d.angle(d.chosen);
            ease(&mut d.at, to, 0.05);
        }
        moving
    }

    pub fn set_knob(&mut self, i: usize, v: f32) {
        self.knobs[i] = v.clamp(0., 100.).round();
    }

    pub fn slide_to(&mut self, x: Pixels) {
        let b = self.slider_bounds.get();
        let w = f32::from(b.size.width).max(1.);
        self.volume = ((f32::from(x - b.origin.x) / w) * 100.).clamp(0., 100.).round();
    }
}

/// A drag in progress, routed from the window root so it keeps going when
/// the pointer leaves the control.
#[derive(Clone, Copy, Default)]
pub enum Drag {
    #[default]
    None,
    Slider { rack: usize },
    Knob { rack: usize, index: usize, start_y: f32, start: f32 },
}

/// A view that holds racks and the key color.
pub trait Host: 'static + Sized {
    fn rack_mut(&mut self, rack: usize) -> &mut Rack;
    fn drag_mut(&mut self) -> &mut Drag;
    fn set_key(&mut self, key: usize);

    /// Continues a drag; returns true if anything changed.
    fn drag_move(&mut self, e: &MouseMoveEvent) -> bool {
        if e.pressed_button != Some(MouseButton::Left) {
            *self.drag_mut() = Drag::None;
            return false;
        }
        match *self.drag_mut() {
            Drag::None => false,
            Drag::Slider { rack } => {
                self.rack_mut(rack).slide_to(e.position.x);
                true
            }
            // Two pixels of travel per step, as a scrub area turns.
            Drag::Knob { rack, index, start_y, start } => {
                self.rack_mut(rack).set_knob(index, start + (start_y - f32::from(e.position.y)) / 2.);
                true
            }
        }
    }
}
