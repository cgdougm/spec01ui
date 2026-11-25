//! Stress test: a wall of SPEC-01 widgets of 23 kinds, packed into rows
//! that reflow to the window width and zoom, virtualized so only rows on
//! screen are built, with a scrollbar, zoom and motion modes.
//!
//!     stress [--widgets N] [--layout mixed|grouped] [--zoom Z]
//!            [--motion off|noise|wave] [--hue] [--scroll] [--no-cull]
//!            [--bench SECONDS]
//!
//! Keys: Ctrl+= / Ctrl++ zoom in, Ctrl+- zoom out, Ctrl+0 reset (Ctrl+wheel
//! zooms too). PgUp/PgDn, Home/End, arrows scroll. M cycles motion, H hue,
//! S auto-scroll, L layout.
//!
//! `--bench` turns on wave motion and scrolling, prints a frame-time
//! summary after that many seconds, and quits.
//!
//! `--suite [FILE]` runs a fixed set of scenarios one after another and
//! appends one JSON line per scenario to FILE (default bench-results.jsonl),
//! tagged with the platform, for `tools/compare-bench.py`. `--label NAME`
//! names the machine in the results.

use std::{
    collections::VecDeque,
    ops::Range,
    time::{Duration, Instant},
};

use gpui::{
    App, AppContext, Application, Bounds, Context, Div, FocusHandle, FontWeight,
    InteractiveElement, IntoElement, KeyDownEvent, MouseButton, MouseMoveEvent, ParentElement,
    Pixels, Render, ScrollDelta, ScrollWheelEvent, Styled, TitlebarOptions, Window, WindowBounds,
    WindowOptions, canvas, div, fill, point, px, size,
};
use spec01::{
    rack::{Drag, Host, Rack, hash01},
    theme::{self, KEYS, Theme, oklch},
    ui::{self, FONT, bar_shell, sh_px, z},
    widgets::{GAP, Kind},
};

const HUD_H: f32 = 108.;
const SCROLLBAR_W: f32 = 16.;
const SIDE_PAD: f32 = 24.;
const COUNTS: [usize; 5] = [100, 1_000, 5_000, 20_000, 50_000];
const SCROLL_SPEED: f32 = 900.; // px/s at zoom 1
const HEADER_H: f32 = 40.;

#[derive(Clone, Copy, PartialEq)]
enum Layout {
    Mixed,
    Grouped,
}

#[derive(Clone, Copy, PartialEq)]
enum Motion {
    Off,
    Noise,
    Wave,
}

/// A packed row of the wall, in zoomed pixels.
struct Row {
    y: f32,
    h: f32,
    items: Range<usize>, // into `Stress::placed`
    header: Option<Kind>,
}

struct Stress {
    key: usize,
    theme: Theme,
    kinds: Vec<Kind>,
    racks: Vec<Rack>,
    drag: Drag,
    focus: FocusHandle,
    focused: bool,

    layout: Layout,
    motion: Motion,
    hue: bool,
    scroll: bool,
    cull: bool,
    zoom: f32,

    // The packed wall, rebuilt when width, zoom, layout or count change.
    placed: Vec<usize>,
    /// Each widget's place on the wall, in skin units, for the wave.
    spot: Vec<(f32, f32)>,
    rows: Vec<Row>,
    content_h: f32,
    packed_for: (f32, f32, usize, bool),
    scroll_y: f32,
    view_h: f32,
    thumb_drag: Option<(f32, f32)>,

    started: Instant,
    last_frame: Instant,
    /// Recent frame times (ms), newest last, for the graph and readout.
    frames: VecDeque<f32>,
    shown: usize,
    /// Only this kind, if set (--kind).
    only: Option<Kind>,
    suite: Option<Suite>,
    scale: f32,
    viewport: (f32, f32),
    /// How long building the frame's elements took (ms), recent frames.
    build: VecDeque<f32>,
    bench: Option<Bench>,
}

/// One suite scenario.
#[derive(Clone, Copy)]
struct Scenario {
    name: &'static str,
    widgets: usize,
    zoom: f32,
    motion: Motion,
    layout: Layout,
    hue: bool,
}

const SUITE: [Scenario; 8] = [
    Scenario { name: "scroll-z100", widgets: 2_000, zoom: 1., motion: Motion::Off, layout: Layout::Mixed, hue: false },
    Scenario { name: "wave-z100", widgets: 2_000, zoom: 1., motion: Motion::Wave, layout: Layout::Mixed, hue: false },
    Scenario { name: "wave-z60", widgets: 2_000, zoom: 0.6, motion: Motion::Wave, layout: Layout::Mixed, hue: false },
    Scenario { name: "wave-z35", widgets: 2_000, zoom: 0.35, motion: Motion::Wave, layout: Layout::Mixed, hue: false },
    Scenario { name: "wave-z35-20k", widgets: 20_000, zoom: 0.35, motion: Motion::Wave, layout: Layout::Mixed, hue: false },
    Scenario { name: "wave-z25-50k", widgets: 50_000, zoom: 0.25, motion: Motion::Wave, layout: Layout::Mixed, hue: false },
    Scenario { name: "noise-hue-z35", widgets: 2_000, zoom: 0.35, motion: Motion::Noise, layout: Layout::Mixed, hue: true },
    Scenario { name: "grouped-z50", widgets: 2_000, zoom: 0.5, motion: Motion::Wave, layout: Layout::Grouped, hue: false },
];
const SUITE_WARMUP: f32 = 2.;
const SUITE_SECS: f32 = 5.;

struct Suite {
    next: usize,
    file: String,
    label: String,
    current: &'static str,
}

struct Bench {
    warmup_until: Instant,
    until: Instant,
    frame_ms: Vec<f32>,
    shown: Vec<usize>,
    build_ms: Vec<f32>,
}

impl Stress {
    fn set_count(&mut self, n: usize) {
        // A mix that cycles through all kinds in a scrambled order.
        self.kinds = match self.only {
            Some(k) => vec![k; n],
            None => (0..n).map(|i| Kind::ALL[(i * 7 + (hash01(i / 23) * 23.) as usize) % Kind::ALL.len()]).collect(),
        };
        self.racks = (0..n).map(Rack::varied).collect();
        self.spot = vec![(0., 0.); n];
        self.packed_for = (0., 0., usize::MAX, false);
        self.scroll_y = 0.;
    }

    /// Starts the next suite scenario; false when there are none left.
    fn next_scenario(&mut self) -> bool {
        let Some(suite) = &mut self.suite else { return false };
        let Some(sc) = SUITE.get(suite.next).copied() else { return false };
        suite.next += 1;
        suite.current = sc.name;
        self.layout = sc.layout;
        self.motion = sc.motion;
        self.hue = sc.hue;
        self.scroll = true;
        self.zoom = sc.zoom;
        self.set_count(sc.widgets);
        let now = Instant::now();
        self.bench = Some(Bench {
            warmup_until: now + Duration::from_secs_f32(SUITE_WARMUP),
            until: now + Duration::from_secs_f32(SUITE_WARMUP + SUITE_SECS),
            frame_ms: Vec::new(),
            shown: Vec::new(),
            build_ms: Vec::new(),
        });
        true
    }

    fn set_zoom(&mut self, zoom: f32) {
        let zoom = zoom.clamp(0.2, 3.);
        // Keep the same part of the wall in view.
        self.scroll_y *= zoom / self.zoom;
        self.zoom = zoom;
    }

    /// Packs widgets into rows that fit `width` (zoomed px). A wide panel
    /// that doesn't fit lets a narrow one from just behind it in first, so
    /// rows close up.
    fn pack(&mut self, width: f32) {
        let k = self.zoom;
        let key = (width, k, self.kinds.len(), self.layout == Layout::Grouped);
        if self.packed_for == key {
            return;
        }
        self.packed_for = key;
        let gap = GAP * k;
        let mut order: Vec<usize> = (0..self.kinds.len()).collect();
        if self.layout == Layout::Grouped {
            order.sort_by_key(|&i| (self.kinds[i], i));
        }
        self.placed.clear();
        self.rows.clear();
        let mut y = 0.;
        let mut pending: VecDeque<usize> = VecDeque::new();
        let mut it = order.into_iter().peekable();
        let mut group: Option<Kind> = None;
        loop {
            // Refill the look-ahead window from the same group.
            while pending.len() < 12 {
                match it.peek() {
                    Some(&i) if self.layout == Layout::Mixed || group.is_none() || Some(self.kinds[i]) == group => {
                        pending.push_back(i);
                        it.next();
                    }
                    _ => break,
                }
            }
            if pending.is_empty() {
                match it.peek() {
                    Some(&i) => {
                        // A new group: a header row naming it.
                        let kind = self.kinds[i];
                        group = Some(kind);
                        let start = self.placed.len();
                        self.rows.push(Row { y, h: HEADER_H * k, items: start..start, header: Some(kind) });
                        y += HEADER_H * k;
                        continue;
                    }
                    None => break,
                }
            }
            let start = self.placed.len();
            let (mut used, mut h) = (0f32, 0f32);
            loop {
                let room = width - used - if used > 0. { gap } else { 0. };
                let Some(pos) = pending.iter().position(|&i| self.kinds[i].width() * k <= room) else { break };
                let i = pending.remove(pos).unwrap();
                let w = self.kinds[i].width() * k;
                let x = used + if used > 0. { gap } else { 0. };
                self.spot[i] = ((x + w / 2.) / k, (y + h / 2.) / k);
                used = x + w;
                h = h.max(self.kinds[i].height() * k);
                self.placed.push(i);
                if pending.is_empty() {
                    break;
                }
            }
            if self.placed.len() == start {
                // Narrower than one panel: place it anyway.
                let i = pending.pop_front().unwrap();
                h = self.kinds[i].height() * k;
                self.spot[i] = (self.kinds[i].width() / 2., y / k);
                self.placed.push(i);
            }
            self.rows.push(Row { y, h, items: start..self.placed.len(), header: None });
            y += h + gap;
        }
        self.content_h = y + 40.;
    }

    fn max_scroll(&self) -> f32 {
        (self.content_h - self.view_h).max(0.)
    }

    fn scroll_by(&mut self, dy: f32) {
        self.scroll_y = (self.scroll_y + dy).clamp(0., self.max_scroll());
    }

    /// Moves every control of the widgets on screen: noise gives each its
    /// own phase; the wave rolls across the wall by position. Motion is a
    /// function of time and place, so widgets off screen needn't be driven:
    /// they pick up where they should be when they scroll in.
    fn drive(&mut self, t: f32, on_screen: &[usize]) {
        for &i in on_screen {
            match self.motion {
                Motion::Off => {}
                Motion::Noise => self.racks[i].drive((t * 0.25 + hash01(i)).fract(), t + i as f32),
                Motion::Wave => {
                    let (x, y) = self.spot[i];
                    let v = ((x * 0.0011 + y * 0.0016) - t * 0.35).rem_euclid(1.);
                    self.racks[i].drive(v, t + (x + y) * 0.002);
                }
            }
        }
    }

    fn chip(&self, text: &str, on: bool, cx: &mut Context<Self>, act: impl Fn(&mut Self) + 'static) -> Div {
        let t = &self.theme;
        let d = div().h(px(22.)).px(px(8.)).flex().items_center().rounded(px(7.)).text_size(px(9.)).font_weight(FontWeight::MEDIUM).cursor_pointer();
        let d = if on {
            d.bg(t.cap.hsla()).text_color(t.on_accent)
        } else {
            d.bg(theme::rgb(250, 250, 248)).text_color(theme::text()).shadow(vec![sh_px(0., 0., 0., 1., theme::shade(0.08)), sh_px(0., 1.5, 0., 0., theme::rgb(220, 220, 216))])
        };
        d.child(text.to_string()).on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
            act(this);
            cx.notify();
        }))
    }

    fn group_label(text: &str) -> Div {
        div().text_size(px(8.)).font_weight(FontWeight::MEDIUM).text_color(theme::dim()).child(text.to_string())
    }

    fn hud(&self, cx: &mut Context<Self>) -> Div {
        let n = self.kinds.len();
        let line1 = div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(px(6.))
            .child(ui::key_picker(self.key, cx))
            .child(div().w(px(8.)))
            .child(Self::group_label("LAYOUT"))
            .child(self.chip("MIXED", self.layout == Layout::Mixed, cx, |s| s.layout = Layout::Mixed))
            .child(self.chip("GROUPED", self.layout == Layout::Grouped, cx, |s| s.layout = Layout::Grouped))
            .child(div().w(px(8.)))
            .child(Self::group_label("MOTION"))
            .child(self.chip("OFF", self.motion == Motion::Off, cx, |s| s.motion = Motion::Off))
            .child(self.chip("NOISE", self.motion == Motion::Noise, cx, |s| s.motion = Motion::Noise))
            .child(self.chip("WAVE", self.motion == Motion::Wave, cx, |s| s.motion = Motion::Wave))
            .child(div().w(px(8.)))
            .child(self.chip("HUE", self.hue, cx, |s| s.hue = !s.hue))
            .child(self.chip("SCROLL", self.scroll, cx, |s| s.scroll = !s.scroll))
            .child(self.chip("CULL", self.cull, cx, |s| s.cull = !s.cull));

        let mut counts = div().flex().items_center().gap(px(6.)).child(Self::group_label("WIDGETS"));
        for c in COUNTS {
            counts = counts.child(self.chip(&group(c), n == c, cx, move |s| s.set_count(c)));
        }
        let zoom = div()
            .flex()
            .items_center()
            .gap(px(6.))
            .child(Self::group_label("ZOOM"))
            .child(self.chip("−", false, cx, |s| s.set_zoom(s.zoom / 1.15)))
            .child(div().w(px(36.)).flex().justify_center().text_size(px(10.)).font_weight(FontWeight::MEDIUM).child(format!("{}%", (self.zoom * 100.).round())))
            .child(self.chip("+", false, cx, |s| s.set_zoom(s.zoom * 1.15)))
            .child(self.chip("1:1", false, cx, |s| s.set_zoom(1.)));

        let frames: Vec<f32> = self.frames.iter().copied().collect();
        let recent = &frames[frames.len().saturating_sub(60)..];
        let avg = if recent.is_empty() { 0. } else { recent.iter().sum::<f32>() / recent.len() as f32 };
        let worst = recent.iter().copied().fold(0., f32::max);
        let driven = self.motion != Motion::Off || self.scroll || self.hue;
        let build = if self.build.is_empty() { 0. } else { self.build.iter().sum::<f32>() / self.build.len() as f32 };
        let readout = if driven {
            format!("{:>3.0} FPS  {:>5.1} MS  WORST {:>5.1}  BUILD {:>4.1} MS", if avg > 0. { 1000. / avg } else { 0. }, avg, worst, build)
        } else {
            "IDLE".to_string()
        };
        let stats = format!("{} WIDGETS  {} ON SCREEN  {} ROWS", group(n), group(self.shown), group(self.rows.len()));
        let ink = self.theme.ink.hsla();
        let line2 = div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(px(14.))
            .child(counts)
            .child(zoom)
            .child(graph(frames, self.theme))
            .child(div().flex().flex_col().text_size(px(9.)).font_weight(FontWeight::MEDIUM).child(div().text_color(ink).child(readout)).child(div().text_color(theme::dim()).child(stats)));
        bar_shell().px(px(18.)).py(px(10.)).flex().flex_col().gap(px(8.)).child(line1).child(line2)
    }

    fn scrollbar(&self, cx: &mut Context<Self>) -> Div {
        let track_h = self.view_h.max(1.);
        let ratio = (self.view_h / self.content_h.max(1.)).min(1.);
        let thumb_h = (track_h * ratio).max(36.);
        let max = self.max_scroll();
        let thumb_y = if max > 0. { self.scroll_y / max * (track_h - thumb_h) } else { 0. };
        let t = &self.theme;
        let thumb = div()
            .absolute()
            .left(px(2.))
            .top(px(thumb_y))
            .w(px(SCROLLBAR_W - 4.))
            .h(px(thumb_h))
            .rounded(px(6.))
            .bg(gpui::linear_gradient(90., gpui::linear_color_stop(theme::rgb(255, 255, 255), 0.), gpui::linear_color_stop(theme::rgb(236, 236, 233), 1.)))
            .shadow(vec![sh_px(0., 0., 0., 0.5, theme::rgba(0, 0, 0, 0.2)), sh_px(0., 2., 4., 0., theme::shade(0.25))])
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(2.))
            .children((0..3).map(|_| div().w(px(6.)).h(px(1.)).bg(theme::rgb(190, 190, 185))))
            .child(div().absolute().left(px(4.)).top(px(-1.)).w(px(4.)).h(px(2.)).bg(t.lamp.hsla()))
            .on_mouse_down(MouseButton::Left, cx.listener(|this, e: &gpui::MouseDownEvent, _, cx| {
                this.thumb_drag = Some((f32::from(e.position.y), this.scroll_y));
                cx.stop_propagation();
                cx.notify();
            }));
        div()
            .relative()
            .w(px(SCROLLBAR_W))
            .h(px(track_h))
            .rounded(px(8.))
            .bg(theme::rgb(226, 226, 222))
            .border_1()
            .border_color(theme::rgba(0, 0, 0, 0.06))
            .cursor_pointer()
            // A click on the track pages toward it.
            .on_mouse_down(MouseButton::Left, cx.listener(move |this, e: &gpui::MouseDownEvent, _, cx| {
                let y = f32::from(e.position.y) - HUD_H;
                let page = this.view_h * 0.9;
                this.scroll_by(if y < thumb_y { -page } else { page });
                cx.notify();
            }))
            .child(thumb)
    }

    fn on_key(&mut self, e: &KeyDownEvent, cx: &mut Context<Self>) {
        let k = &e.keystroke;
        let ctrl = k.modifiers.control || k.modifiers.platform;
        let page = self.view_h * 0.9;
        match (ctrl, k.key.as_str()) {
            (true, "=" | "+") => self.set_zoom(self.zoom * 1.15),
            (true, "-") => self.set_zoom(self.zoom / 1.15),
            (true, "0") => self.set_zoom(1.),
            (false, "pagedown" | "space") => self.scroll_by(page),
            (false, "pageup") => self.scroll_by(-page),
            (false, "down") => self.scroll_by(60.),
            (false, "up") => self.scroll_by(-60.),
            (false, "home") => self.scroll_y = 0.,
            (false, "end") => self.scroll_y = self.max_scroll(),
            (false, "m") => self.motion = match self.motion { Motion::Off => Motion::Noise, Motion::Noise => Motion::Wave, Motion::Wave => Motion::Off },
            (false, "h") => self.hue = !self.hue,
            (false, "s") => self.scroll = !self.scroll,
            (false, "l") => self.layout = if self.layout == Layout::Mixed { Layout::Grouped } else { Layout::Mixed },
            _ => return,
        }
        cx.notify();
    }

    fn on_wheel(&mut self, e: &ScrollWheelEvent, cx: &mut Context<Self>) {
        let dy = match e.delta {
            ScrollDelta::Pixels(p) => -f32::from(p.y),
            ScrollDelta::Lines(p) => -p.y * 60.,
        };
        if e.modifiers.control {
            self.set_zoom(self.zoom * if dy < 0. { 1.1 } else { 1. / 1.1 });
        } else {
            self.scroll_by(dy);
        }
        cx.notify();
    }

    fn row_view(&self, row: &Row, top: f32, cx: &mut Context<Self>) -> Div {
        let el = div().absolute().left_0().top(px(top)).w_full().h(px(row.h));
        if let Some(kind) = row.header {
            let count = self.kinds.iter().filter(|&&k| k == kind).count();
            return el.flex().items_end().justify_center().pb(z(10.)).child(
                div()
                    .text_size(z(12.))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme::dim())
                    .child(format!("{}  ×  {}", kind.name().to_uppercase(), group(count))),
            );
        }
        let t = self.theme;
        el.flex().justify_center().items_start().gap(z(GAP)).children(self.placed[row.items.clone()].iter().map(|&i| self.kinds[i].render(&self.racks[i], i, &t, cx)))
    }
}

/// The frame-time graph: the last 120 frames as bars, green to red.
fn graph(frames: Vec<f32>, t: Theme) -> impl IntoElement {
    canvas(
        |_, _, _| {},
        move |b: Bounds<Pixels>, _, window, _| {
            window.paint_quad(fill(b, theme::rgb(24, 24, 23)));
            let n = 120usize;
            let w = f32::from(b.size.width) / n as f32;
            let h = f32::from(b.size.height);
            // A line at 16.7 ms (60 FPS) and the bars, scaled to 50 ms.
            let line = h - h * (16.7 / 50.);
            window.paint_quad(fill(Bounds::new(point(b.origin.x, b.origin.y + px(line)), size(b.size.width, px(1.))), theme::rgb(70, 70, 66)));
            for (i, ms) in frames.iter().rev().take(n).enumerate() {
                let bh = (ms / 50. * h).clamp(1., h);
                let x = b.origin.x + px(f32::from(b.size.width) - (i + 1) as f32 * w);
                let color = if *ms <= 18. { t.lamp.hsla() } else if *ms <= 34. { oklch(0.8, 0.16, 75.).hsla() } else { oklch(0.62, 0.24, 25.).hsla() };
                window.paint_quad(fill(Bounds::new(point(x, b.origin.y + px(h - bh)), size(px((w - 0.5).max(0.5)), px(bh))), color));
            }
        },
    )
    .w(px(180.))
    .h(px(30.))
    .rounded(px(4.))
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
        if !self.focused {
            window.focus(&self.focus);
            self.focused = true;
        }
        let now = Instant::now();
        let built = now;
        let dt = (now - self.last_frame).as_secs_f32();
        self.last_frame = now;
        let t = self.started.elapsed().as_secs_f32();
        ui::set_now(t);
        ui::set_zoom(self.zoom);

        self.frames.push_back(dt * 1000.);
        if self.frames.len() > 240 {
            self.frames.pop_front();
        }

        // Size and pack for the window as it is now: resizing reflows.
        let viewport = window.viewport_size();
        self.scale = window.scale_factor();
        self.viewport = (f32::from(viewport.width), f32::from(viewport.height));
        self.view_h = (f32::from(viewport.height) - HUD_H).max(50.);
        let width = f32::from(viewport.width) - SCROLLBAR_W - SIDE_PAD * 3.;
        self.pack(width.max(100.));

        if self.scroll {
            let max = self.max_scroll();
            if max > 0. {
                self.scroll_y = (self.scroll_y + SCROLL_SPEED * self.zoom * dt.min(0.05)) % max;
            }
        }
        self.scroll_y = self.scroll_y.clamp(0., self.max_scroll());

        // The rows in view, with a little overdraw.
        let (top, bottom) = if self.cull { (self.scroll_y - 200., self.scroll_y + self.view_h + 200.) } else { (f32::MIN, f32::MAX) };
        let first = self.rows.partition_point(|r| r.y + r.h < top);
        let last = self.rows.partition_point(|r| r.y <= bottom).max(first);
        let on_screen: Vec<usize> = self.rows[first..last].iter().flat_map(|r| self.placed[r.items.clone()].iter().copied()).collect();
        self.shown = on_screen.len();

        self.drive(t, &on_screen);
        if self.hue {
            let base = KEYS[self.key].1;
            let c = if base.c < 0.05 { 0.25 } else { base.c };
            self.theme = Theme::new(oklch(base.l.clamp(0.5, 0.75), c, (base.h + t * 40.) % 360.));
        }
        let k = 1. - (-dt.min(0.05) * 18.).exp();
        let mut moving = false;
        for &i in &on_screen {
            moving |= self.racks[i].ease(k);
        }

        if let Some(b) = &mut self.bench {
            if now > b.warmup_until {
                b.frame_ms.push(dt * 1000.);
                b.shown.push(self.shown);
                if let Some(&ms) = self.build.back() {
                    b.build_ms.push(ms);
                }
            }
            if now > b.until {
                let stats = report(b, self.only, self.kinds.len(), self.zoom, self.layout);
                if let Some(s) = &self.suite {
                    save(s, &stats, self.scale, self.viewport);
                }
                self.bench = None;
                if !self.next_scenario() {
                    cx.quit();
                }
            }
        }
        if moving || self.motion != Motion::Off || self.scroll || self.hue || self.bench.is_some() {
            window.request_animation_frame();
        }

        let rows: Vec<Div> = (first..last).map(|r| self.row_view(&self.rows[r], self.rows[r].y - self.scroll_y, cx)).collect();

        let wall = div()
            .relative()
            .flex_1()
            .h(px(self.view_h))
            .overflow_hidden()
            .children(rows)
            .on_scroll_wheel(cx.listener(|this, e: &ScrollWheelEvent, _, cx| this.on_wheel(e, cx)));

        let hud = self.hud(cx);
        let scrollbar = self.scrollbar(cx);
        self.build.push_back(built.elapsed().as_secs_f32() * 1000.);
        if self.build.len() > 60 {
            self.build.pop_front();
        }
        div()
            .size_full()
            .bg(theme::page())
            .font_family(FONT)
            .text_color(theme::text())
            .track_focus(&self.focus)
            .on_key_down(cx.listener(|this, e: &KeyDownEvent, _, cx| this.on_key(e, cx)))
            .on_mouse_move(cx.listener(|this, e: &MouseMoveEvent, _, cx| {
                if let Some((y0, s0)) = this.thumb_drag {
                    if e.pressed_button != Some(MouseButton::Left) {
                        this.thumb_drag = None;
                        return;
                    }
                    let ratio = (this.view_h / this.content_h.max(1.)).min(1.);
                    let thumb_h = (this.view_h * ratio).max(36.);
                    let per_px = this.max_scroll() / (this.view_h - thumb_h).max(1.);
                    this.scroll_y = (s0 + (f32::from(e.position.y) - y0) * per_px).clamp(0., this.max_scroll());
                    cx.notify();
                } else if this.drag_move(e) {
                    cx.notify();
                }
            }))
            .on_mouse_up(MouseButton::Left, cx.listener(|this, _, _, cx| {
                this.drag = Drag::None;
                this.thumb_drag = None;
                for r in &mut this.racks {
                    r.pressed = None;
                }
                cx.notify();
            }))
            .flex()
            .flex_col()
            .child(div().h(px(HUD_H)).px(px(SIDE_PAD)).pt(px(10.)).child(hud))
            .child(div().flex().px(px(SIDE_PAD)).gap(px(SIDE_PAD)).child(wall).child(scrollbar))
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

struct Stats {
    widgets: usize,
    zoom: f32,
    on_screen: usize,
    build_ms: f32,
    frames: usize,
    fps: f32,
    p50: f32,
    p95: f32,
    p99: f32,
    max: f32,
}

/// Appends one scenario's results as a JSON line, with the platform.
fn save(s: &Suite, st: &Stats, scale: f32, viewport: (f32, f32)) {
    use std::io::Write;
    let cpus = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(0);
    let line = format!(
        "{{\"label\":\"{}\",\"scenario\":\"{}\",\"os\":\"{}\",\"arch\":\"{}\",\"cpus\":{},\"scale\":{:.2},\"viewport\":[{:.0},{:.0}],\"widgets\":{},\"zoom\":{:.2},\"on_screen\":{},\"build_ms\":{:.2},\"frames\":{},\"fps\":{:.1},\"p50_ms\":{:.2},\"p95_ms\":{:.2},\"p99_ms\":{:.2},\"max_ms\":{:.2},\"version\":\"{}\"}}\n",
        s.label.replace('"', "'"),
        s.current,
        std::env::consts::OS,
        std::env::consts::ARCH,
        cpus,
        scale,
        viewport.0,
        viewport.1,
        st.widgets,
        st.zoom,
        st.on_screen,
        st.build_ms,
        st.frames,
        st.fps,
        st.p50,
        st.p95,
        st.p99,
        st.max,
        env!("CARGO_PKG_VERSION"),
    );
    match std::fs::OpenOptions::new().create(true).append(true).open(&s.file) {
        Ok(mut f) => {
            let _ = f.write_all(line.as_bytes());
        }
        Err(e) => eprintln!("can't write {}: {e}", s.file),
    }
}

fn report(b: &Bench, only: Option<Kind>, widgets: usize, zoom: f32, layout: Layout) -> Stats {
    let mut v = b.frame_ms.clone();
    v.sort_by(|a, b| a.total_cmp(b));
    let pct = |p: f32| v.get(((v.len() as f32 - 1.) * p) as usize).copied().unwrap_or(0.);
    let total: f32 = v.iter().sum();
    let fps = if total > 0. { v.len() as f32 * 1000. / total } else { 0. };
    let shown = if b.shown.is_empty() { 0 } else { b.shown.iter().sum::<usize>() / b.shown.len() };
    let build = if b.build_ms.is_empty() { 0. } else { b.build_ms.iter().sum::<f32>() / b.build_ms.len() as f32 };
    let kind = only.map(|k| k.name().replace(' ', "-").to_lowercase()).unwrap_or_else(|| "all".into());
    println!(
        "kind={} layout={} widgets={} zoom={:.2} on_screen_avg={} build_ms={:.2} frames={} avg_fps={:.1} p50_ms={:.2} p95_ms={:.2} p99_ms={:.2} max_ms={:.2}",
        kind,
        if layout == Layout::Mixed { "mixed" } else { "grouped" },
        widgets,
        zoom,
        shown,
        build,
        v.len(),
        fps,
        pct(0.5),
        pct(0.95),
        pct(0.99),
        v.last().copied().unwrap_or(0.)
    );
    Stats { widgets, zoom, on_screen: shown, build_ms: build, frames: v.len(), fps, p50: pct(0.5), p95: pct(0.95), p99: pct(0.99), max: v.last().copied().unwrap_or(0.) }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let flag = |f: &str| args.iter().any(|a| a == f);
    let value = |f: &str| args.iter().position(|a| a == f).and_then(|i| args.get(i + 1)).cloned();
    let widgets = value("--widgets").and_then(|v| v.parse::<usize>().ok()).unwrap_or(2_000);
    let zoom = value("--zoom").and_then(|v| v.parse::<f32>().ok()).unwrap_or(1.);
    let bench = value("--bench").and_then(|v| v.parse::<f32>().ok());
    let layout = if value("--layout").as_deref() == Some("grouped") { Layout::Grouped } else { Layout::Mixed };
    let motion = match value("--motion").as_deref() {
        Some("noise") => Motion::Noise,
        Some("wave") => Motion::Wave,
        Some(_) => Motion::Off,
        None if bench.is_some() => Motion::Wave,
        None => Motion::Off,
    };
    // --kind "Flat knob" or flat-knob: a wall of one kind.
    let only = value("--kind").and_then(|v| {
        let want = v.to_lowercase().replace('-', " ");
        Kind::ALL.into_iter().find(|k| k.name().to_lowercase() == want || format!("{k:?}").to_lowercase() == want.replace(' ', ""))
    });
    // --suite [FILE]: the file is optional.
    let suite = args.iter().position(|a| a == "--suite").map(|i| Suite {
        next: 0,
        file: args.get(i + 1).filter(|v| !v.starts_with("--")).cloned().unwrap_or_else(|| "bench-results.jsonl".into()),
        label: value("--label").unwrap_or_else(|| std::env::consts::OS.to_string()),
        current: "",
    });
    let (hue, scroll, cull) = (flag("--hue"), flag("--scroll") || bench.is_some(), !flag("--no-cull"));

    Application::new().run(move |cx: &mut App| {
        spec01::load_fonts(cx);
        let bounds = Bounds::centered(None, size(px(1280.), px(900.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions { title: Some("SPEC-01 / GPUI stress test".into()), ..Default::default() }),
                window_min_size: Some(size(px(420.), px(360.))),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|cx| {
                    let now = Instant::now();
                    let mut s = Stress {
                        key: 0,
                        theme: Theme::new(KEYS[0].1),
                        kinds: Vec::new(),
                        racks: Vec::new(),
                        drag: Drag::None,
                        focus: cx.focus_handle(),
                        focused: false,
                        layout,
                        motion,
                        hue,
                        scroll,
                        cull,
                        zoom: zoom.clamp(0.2, 3.),
                        placed: Vec::new(),
                        spot: Vec::new(),
                        rows: Vec::new(),
                        content_h: 0.,
                        packed_for: (0., 0., usize::MAX, false),
                        scroll_y: 0.,
                        view_h: 600.,
                        thumb_drag: None,
                        started: now,
                        last_frame: now,
                        frames: VecDeque::new(),
                        shown: 0,
                        only,
                        suite,
                        scale: 1.,
                        viewport: (0., 0.),
                        build: VecDeque::new(),
                        bench: bench.map(|secs| Bench {
                            warmup_until: now + Duration::from_secs(2),
                            until: now + Duration::from_secs(2) + Duration::from_secs_f32(secs),
                            frame_ms: Vec::new(),
                            shown: Vec::new(),
                            build_ms: Vec::new(),
                        }),
                    };
                    s.set_count(widgets);
                    if s.suite.is_some() {
                        s.next_scenario();
                        eprintln!("suite: {} scenarios, {:.0}s each, results -> {}", SUITE.len(), SUITE_WARMUP + SUITE_SECS, s.suite.as_ref().unwrap().file);
                    }
                    s
                })
            },
        )
        .expect("open window");
        cx.activate(true);
    });
}
