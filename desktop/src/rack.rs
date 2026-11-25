//! A rack: one set of the five components and their state. The gallery
//! shows one; the stress test shows thousands. Views that host racks
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
        }
    }
}

impl Rack {
    /// A rack with its controls set somewhere different for each `seed`, so
    /// a long list doesn't look like one rack repeated.
    pub fn varied(seed: usize) -> Self {
        let mut r = Rack::default();
        let f = |k: usize| ((seed * 7919 + k * 104_729) % 101) as f32;
        r.volume = f(1);
        r.knobs = [f(2), f(3), f(4), f(5)];
        r.power = seed % 3 != 0;
        r.switch_at = if r.power { 1. } else { 0. };
        r.dials[0].chosen = seed % 5;
        r.dials[1].chosen = (seed / 5) % 5;
        for d in &mut r.dials {
            d.at = d.angle(d.chosen);
        }
        r
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
