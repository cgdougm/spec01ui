//! The window's one view: all control state, the navbar with the key
//! picker, the grid of modules, drags, and the eased animations.

use std::{cell::Cell, rc::Rc, time::Instant};

use gpui::{
    Bounds, Context, Div, FontWeight, InteractiveElement, IntoElement, MouseButton, MouseMoveEvent,
    ParentElement, Pixels, Render, Styled, Window, div, linear_color_stop, linear_gradient, px,
};

use crate::{
    theme::{self, KEYS, Theme},
    ui::{FONT, sh},
    widgets::{button::Key, mode_dial::Dial},
};

pub enum Drag {
    None,
    Slider,
    Knob { index: usize, start_y: f32, start: f32 },
}

pub struct Gallery {
    pub key: usize,
    pub theme: Theme,
    pub pressed: Option<Key>,
    pub power: bool,
    /// The switch thumb's drawn position, easing toward `power`.
    pub switch_at: f32,
    pub volume: f32,
    pub slider_bounds: Rc<Cell<Bounds<Pixels>>>,
    pub knobs: [f32; 4],
    pub dials: Vec<Dial>,
    pub drag: Drag,
    last_frame: Instant,
}

impl Gallery {
    pub fn new() -> Self {
        Gallery {
            key: 0,
            theme: Theme::new(KEYS[0].1),
            pressed: None,
            power: true,
            switch_at: 1.,
            volume: 64.,
            slider_bounds: Rc::default(),
            knobs: [30., 72., 50., 18.],
            dials: vec![
                Dial::new("Effort", &["low", "medium", "high", "xhigh", "max"], 240., 2),
                Dial::new("Bank", &["1", "2", "3", "4", "5"], 180., 2),
            ],
            drag: Drag::None,
            last_frame: Instant::now(),
        }
    }

    /// Eases animated values toward their targets; true while any is moving.
    fn animate(&mut self) -> bool {
        let now = Instant::now();
        let dt = (now - self.last_frame).as_secs_f32().min(0.05);
        self.last_frame = now;
        let k = 1. - (-dt * 18.).exp(); // ~150ms to settle
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

    fn navbar(&self, cx: &mut Context<Self>) -> Div {
        let swatches = KEYS.iter().enumerate().map(|(i, (_, key))| {
            let on = i == self.key;
            // The chosen key is ringed: a dark circle, a light gap, the key.
            // (Nested circles, as gpui keeps a shadow's corner radius as it spreads.)
            let ring = |size: f32, color| div().size(px(size)).rounded_full().flex().items_center().justify_center().bg(color);
            let swatch = div().size(px(18.)).rounded_full().bg(key.hsla()).border_1().border_color(theme::rgba(0, 0, 0, 0.12));
            let outer = if on { theme::text() } else { theme::rgba(0, 0, 0, 0.) };
            let gap = if on { theme::rgb(246, 246, 244) } else { theme::rgba(0, 0, 0, 0.) };
            ring(25., outer)
                .child(ring(23., gap).child(swatch))
                .cursor_pointer()
                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                    this.key = i;
                    this.theme = Theme::new(KEYS[i].1);
                    cx.notify();
                }))
        });
        div()
            .h(px(52.))
            .px(px(22.))
            .flex()
            .items_center()
            .justify_between()
            .rounded(px(32.))
            .bg(linear_gradient(180., linear_color_stop(theme::rgb(248, 248, 246), 0.), linear_color_stop(theme::rgb(242, 242, 240), 1.)))
            .border_1()
            .border_color(theme::rgba(255, 255, 255, 0.6))
            .shadow(vec![
                sh(0., 0., 0., 1., theme::rgba(0, 0, 0, 0.07)),
                sh(0., 1., 2., 0., theme::shade(0.11)),
                sh(0., 12., 28., -12., theme::shade(0.27)),
            ])
            .child(div().flex().items_center().gap(px(2.)).children(swatches))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .text_size(px(10.))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme::text())
                    .child(crate::ui::led(&self.theme, 5.))
                    .child("SPEC-01 / GPUI"),
            )
    }

    fn on_move(&mut self, e: &MouseMoveEvent, cx: &mut Context<Self>) {
        if e.pressed_button != Some(MouseButton::Left) {
            self.drag = Drag::None;
            return;
        }
        match self.drag {
            Drag::None => return,
            Drag::Slider => self.slide_to(e.position.x),
            // Two pixels of travel per step, as a scrub area turns.
            Drag::Knob { index, start_y, start } => self.set_knob(index, start + (start_y - f32::from(e.position.y)) / 2.),
        }
        cx.notify();
    }
}

impl Render for Gallery {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.animate() {
            window.request_animation_frame();
        }
        let t = self.theme;

        let row = || div().flex().items_start().gap(px(16.));
        div()
            .size_full()
            .bg(theme::page())
            .font_family(FONT)
            .text_color(theme::text())
            .flex()
            .flex_col()
            .items_center()
            .pt(px(12.))
            .on_mouse_move(cx.listener(|this, e: &MouseMoveEvent, _, cx| this.on_move(e, cx)))
            .on_mouse_up(MouseButton::Left, cx.listener(|this, _, _, cx| {
                this.drag = Drag::None;
                this.pressed = None;
                cx.notify();
            }))
            .child(div().w(px(1000.)).child(self.navbar(cx)))
            .child(
                div()
                    .w(px(1000.))
                    .mt(px(28.))
                    .flex()
                    .flex_col()
                    .gap(px(16.))
                    .child(row().child(self.button_module(&t, cx)).child(self.switch_module(&t, cx)).child(self.slider_module(&t, cx)))
                    .child(row().child(self.knob_module(&t, cx)).child(self.mode_dial_module(&t, cx))),
            )
    }
}
