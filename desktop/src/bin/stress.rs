//! Stress test: a huge scrolling window of SPEC-01 racks (five components
//! each), with a frame-time readout.
//!
//! Two layouts:
//! - VIRTUAL: a `uniform_list`; only the racks on screen are built and drawn.
//! - ALL: a plain scrolling column; every rack is built and laid out every
//!   frame, and gpui culls what's off screen only when painting.
//!
//! ANIMATE moves every control in every rack each frame; SCROLL scrolls the
//! list continuously. Both keep frames coming, so the readout shows what the
//! machine sustains.
//!
//!     stress [--racks N] [--all] [--animate] [--scroll] [--bench SECONDS]
//!
//! `--bench` turns on animate and scroll, prints a frame-time summary after
//! that many seconds, and quits.

use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use gpui::{
    prelude::FluentBuilder, App, AppContext, Application, Bounds, Context, Div, FontWeight, InteractiveElement,
    IntoElement, MouseButton, MouseMoveEvent, ParentElement, Render, ScrollHandle,
    StatefulInteractiveElement, Styled, TitlebarOptions, UniformListScrollHandle, Window,
    WindowBounds, WindowOptions, div, point, px, size, uniform_list,
};
use spec01::{
    RACK_HEIGHT,
    rack::{Drag, Host, Rack},
    rack_view,
    theme::{self, KEYS, Theme},
    ui::{self, FONT, sh},
};

/// A rack plus its numbered header and spacing: one list item.
const ITEM_HEIGHT: f32 = 24. + RACK_HEIGHT + 28.;
const COUNTS: [usize; 4] = [10, 100, 1_000, 10_000];
const SCROLL_SPEED: f32 = 1500.; // px/s

#[derive(Clone, Copy, PartialEq)]
enum Layout {
    Virtual,
    All,
}

struct Stress {
    key: usize,
    theme: Theme,
    racks: Vec<Rack>,
    drag: Drag,
    layout: Layout,
    animate: bool,
    scroll: bool,
    list_scroll: UniformListScrollHandle,
    all_scroll: ScrollHandle,
    started: Instant,
    last_frame: Instant,
    /// Frame start times over the last second, for the readout.
    recent: VecDeque<Instant>,
    bench: Option<Bench>,
}

struct Bench {
    until: Instant,
    warmup_until: Instant,
    frame_ms: Vec<f32>,
}

impl Stress {
    fn set_count(&mut self, n: usize) {
        self.racks = (0..n).map(Rack::varied).collect();
    }

    fn scroll_handle(&self) -> ScrollHandle {
        match self.layout {
            Layout::Virtual => self.list_scroll.0.borrow().base_handle.clone(),
            Layout::All => self.all_scroll.clone(),
        }
    }

    /// Moves every control of every rack, each a little out of phase.
    fn drive(&mut self) {
        let t = self.started.elapsed().as_secs_f32();
        for (i, r) in self.racks.iter_mut().enumerate() {
            let p = i as f32 * 0.37;
            r.volume = (50. + 50. * (t * 1.1 + p).sin()).round();
            for k in 0..4 {
                r.knobs[k] = (50. + 50. * (t * 1.7 + p + k as f32).sin()).round();
            }
            r.power = (t * 2. + p).sin() > 0.;
            r.dials[0].chosen = ((t * 1.5 + p) as usize) % 5;
            r.dials[1].chosen = ((t * 0.9 + p * 2.) as usize) % 5;
        }
    }

    fn chip(&self, text: String, on: bool, cx: &mut Context<Self>, act: impl Fn(&mut Self) + 'static) -> Div {
        let t = &self.theme;
        div()
            .h(px(24.))
            .px(px(9.))
            .flex()
            .items_center()
            .rounded(px(7.))
            .text_size(px(9.))
            .font_weight(FontWeight::MEDIUM)
            .cursor_pointer()
            .map(|d| {
                if on {
                    d.bg(t.cap.hsla()).text_color(t.on_accent)
                } else {
                    d.bg(theme::rgb(250, 250, 248)).text_color(theme::text()).shadow(vec![
                        sh(0., 0., 0., 1., theme::shade(0.08)),
                        sh(0., 1.5, 0., 0., theme::rgb(220, 220, 216)),
                    ])
                }
            })
            .child(text)
            .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                act(this);
                cx.notify();
            }))
    }

    fn controls(&self, cx: &mut Context<Self>) -> Div {
        let n = self.racks.len();
        let frames: Vec<_> = self.recent.iter().collect();
        let fps = frames.len();
        let worst = frames.windows(2).map(|w| (*w[1] - *w[0]).as_secs_f32() * 1000.).fold(0., f32::max);
        // gpui only draws when something changes, so the rate means
        // something only while frames are being driven.
        let readout = if self.animate || self.scroll {
            format!("{fps:>3} FPS  WORST {worst:>5.1} MS  {} COMPONENTS", group(n * 5))
        } else {
            format!("IDLE  {} COMPONENTS", group(n * 5))
        };

        let mut bar = div().flex().items_center().gap(px(6.));
        bar = bar
            .child(self.chip("VIRTUAL".into(), self.layout == Layout::Virtual, cx, |s| s.layout = Layout::Virtual))
            .child(self.chip("ALL".into(), self.layout == Layout::All, cx, |s| s.layout = Layout::All))
            .child(div().w(px(8.)));
        for c in COUNTS {
            bar = bar.child(self.chip(group(c), n == c, cx, move |s| s.set_count(c)));
        }
        bar.child(div().w(px(8.)))
            .child(self.chip("ANIMATE".into(), self.animate, cx, |s| s.animate = !s.animate))
            .child(self.chip("SCROLL".into(), self.scroll, cx, |s| s.scroll = !s.scroll))
            .child(
                div()
                    .ml(px(10.))
                    .w(px(330.))
                    .text_size(px(10.))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(self.theme.ink.hsla())
                    .child(readout),
            )
    }

    fn rack_item(&self, i: usize, cx: &mut Context<Self>) -> Div {
        div().h(px(ITEM_HEIGHT)).w_full().flex().justify_center().child(
            div()
                .flex()
                .flex_col()
                .child(
                    div()
                        .h(px(24.))
                        .text_size(px(10.))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme::dim())
                        .child(format!("RACK {:05}", i + 1)),
                )
                .child(rack_view(&self.racks[i], i, &self.theme, cx)),
        )
    }
}

impl Host for Stress {
    fn rack_mut(&mut self, rack: usize) -> &mut Rack {
        &mut self.racks[rack]
    }
    fn drag_mut(&mut self) -> &mut Drag {
        &mut self.drag
    }
    fn set_key(&mut self, key: usize) {
        self.key = key;
        self.theme = Theme::new(KEYS[key].1);
    }
}

impl Render for Stress {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let now = Instant::now();
        let dt = (now - self.last_frame).as_secs_f32();
        self.last_frame = now;

        // Frame timing: the readout keeps the last second; a bench keeps all.
        self.recent.push_back(now);
        while self.recent.front().is_some_and(|f| now - *f > Duration::from_secs(1)) {
            self.recent.pop_front();
        }
        if let Some(b) = &mut self.bench {
            if now > b.warmup_until {
                b.frame_ms.push(dt * 1000.);
            }
            if now > b.until {
                report(&b.frame_ms, self.racks.len(), self.layout);
                cx.quit();
            }
        }

        if self.animate {
            self.drive();
        }
        let k = 1. - (-dt.min(0.05) * 18.).exp();
        let mut moving = false;
        for r in &mut self.racks {
            moving |= r.ease(k);
        }
        if self.scroll {
            let h = self.scroll_handle();
            let max = f32::from(h.max_offset().height);
            if max > 0. {
                let y = (-f32::from(h.offset().y) + SCROLL_SPEED * dt.min(0.05)) % max;
                h.set_offset(point(px(0.), px(-y)));
            }
        }
        if moving || self.animate || self.scroll || self.bench.is_some() {
            window.request_animation_frame();
        }

        let n = self.racks.len();
        let content = match self.layout {
            Layout::Virtual => div().size_full().child(
                uniform_list(
                    "racks",
                    n,
                    cx.processor(|this, range: std::ops::Range<usize>, _, cx| range.map(|i| this.rack_item(i, cx)).collect::<Vec<_>>()),
                )
                .track_scroll(self.list_scroll.clone())
                .size_full(),
            ),
            Layout::All => div().size_full().child(
                div()
                    .id("all-racks")
                    .size_full()
                    .overflow_y_scroll()
                    .track_scroll(&self.all_scroll)
                    .children((0..n).map(|i| self.rack_item(i, cx))),
            ),
        };

        let controls = self.controls(cx);
        div()
            .size_full()
            .bg(theme::page())
            .font_family(FONT)
            .text_color(theme::text())
            .flex()
            .flex_col()
            .on_mouse_move(cx.listener(|this, e: &MouseMoveEvent, _, cx| {
                if this.drag_move(e) {
                    cx.notify();
                }
            }))
            .on_mouse_up(MouseButton::Left, cx.listener(|this, _, _, cx| {
                this.drag = Drag::None;
                for r in &mut this.racks {
                    r.pressed = None;
                }
                cx.notify();
            }))
            .child(div().w_full().flex().justify_center().py(px(12.)).child(div().w(px(1000.)).child(ui::navbar(self.key, controls, cx))))
            .child(div().flex_1().w_full().overflow_hidden().child(content))
    }
}

/// 10000 -> "10,000"
fn group(n: usize) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out
}

fn report(frame_ms: &[f32], racks: usize, layout: Layout) {
    let mut v = frame_ms.to_vec();
    v.sort_by(|a, b| a.total_cmp(b));
    let pct = |p: f32| v.get(((v.len() as f32 - 1.) * p) as usize).copied().unwrap_or(0.);
    let total: f32 = v.iter().sum();
    let fps = if total > 0. { v.len() as f32 * 1000. / total } else { 0. };
    println!(
        "layout={} racks={} components={} frames={} avg_fps={:.1} p50_ms={:.2} p95_ms={:.2} p99_ms={:.2} max_ms={:.2}",
        if layout == Layout::Virtual { "virtual" } else { "all" },
        racks,
        racks * 5,
        v.len(),
        fps,
        pct(0.5),
        pct(0.95),
        pct(0.99),
        v.last().copied().unwrap_or(0.)
    );
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let flag = |f: &str| args.iter().any(|a| a == f);
    let value = |f: &str| args.iter().position(|a| a == f).and_then(|i| args.get(i + 1)).and_then(|v| v.parse::<f32>().ok());
    let racks = value("--racks").map(|v| v as usize).unwrap_or(1_000);
    let bench = value("--bench");
    let (all, animate, scroll) = (flag("--all"), flag("--animate"), flag("--scroll"));

    Application::new().run(move |cx: &mut App| {
        spec01::load_fonts(cx);
        let bounds = Bounds::centered(None, size(px(1064.), px(900.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions { title: Some("SPEC-01 / GPUI stress test".into()), ..Default::default() }),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|_| {
                    let now = Instant::now();
                    let mut s = Stress {
                        key: 0,
                        theme: Theme::new(KEYS[0].1),
                        racks: Vec::new(),
                        drag: Drag::None,
                        layout: if all { Layout::All } else { Layout::Virtual },
                        animate: animate || bench.is_some(),
                        scroll: scroll || bench.is_some(),
                        list_scroll: UniformListScrollHandle::new(),
                        all_scroll: ScrollHandle::new(),
                        started: now,
                        last_frame: now,
                        recent: VecDeque::new(),
                        bench: bench.map(|secs| Bench {
                            warmup_until: now + Duration::from_secs(2),
                            until: now + Duration::from_secs(2) + Duration::from_secs_f32(secs),
                            frame_ms: Vec::new(),
                        }),
                    };
                    s.set_count(racks);
                    s
                })
            },
        )
        .expect("open window");
        cx.activate(true);
    });
}
