//! 01 Button: raised keys standing in their holes. A primary key in the key
//! color, a secondary white key, and a round key with a record dot. Pressing
//! sinks the key onto its lip.

use gpui::{
    Context, Div, FontWeight, InteractiveElement, MouseButton, ParentElement, Styled,
    div, linear_color_stop, linear_gradient, px,
};

use crate::{
    gallery::Gallery,
    theme::{self, Theme},
    ui::{module, sh},
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Key {
    Play,
    Stop,
    Record,
}

impl Gallery {
    pub fn button_module(&self, t: &Theme, cx: &mut Context<Self>) -> Div {
        let body = div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(12.))
            .child(
                div()
                    .flex()
                    .gap(px(12.))
                    .child(self.key_button(t, Key::Play, cx))
                    .child(self.key_button(t, Key::Stop, cx)),
            )
            .child(self.key_button(t, Key::Record, cx));
        module(t, "01", "Button", 238., body).h(px(190.))
    }

    fn key_button(&self, t: &Theme, key: Key, cx: &mut Context<Self>) -> Div {
        let down = self.pressed == Some(key);
        let primary = key == Key::Play;
        let round = key == Key::Record;
        // Pressed, the key travels most of its 3px lip.
        let travel = if down { 2.0 } else { 0.0 };
        let lip = 3.0 - travel;

        let (bg, shadows) = if primary {
            let k = t.key;
            (
                linear_gradient(0., linear_color_stop(k.with_l(k.l + 0.06).hsla(), 0.), linear_color_stop(k.hsla(), 0.7)),
                vec![
                    sh(0., lip, 0., 0., k.with_l(k.l * 0.7).hsla()),
                    sh(0., lip + 0.5, 1., 0., theme::rgba(0, 0, 0, 0.11)),
                    sh(0., 6. - travel, 10., -3., k.alpha(if down { 0.3 } else { 0.468 })),
                    sh(0., 11. - travel * 2., 20., -8., k.alpha(if down { 0.2 } else { 0.342 })),
                ],
            )
        } else {
            (
                linear_gradient(0., linear_color_stop(theme::rgb(253, 253, 252), 0.), linear_color_stop(theme::rgb(239, 239, 236), 0.7)),
                vec![
                    sh(0., 0., 0., 1., theme::shade(0.07)),
                    sh(0., lip, 0., 0., theme::rgb(220, 220, 216)),
                    sh(0., lip + 0.5, 1., 0., theme::shade(0.118)),
                    sh(0., lip + 1.5, 3., 0., theme::shade(0.063)),
                    sh(0.5, 6. - travel, 9., -3., theme::shade(0.082)),
                ],
            )
        };

        let face = div()
            .flex()
            .items_center()
            .justify_center()
            .bg(bg)
            .shadow(shadows)
            .cursor_pointer()
            .text_size(px(10.))
            .font_weight(if primary { FontWeight::NORMAL } else { FontWeight::MEDIUM })
            .text_color(if primary { t.on_accent } else { theme::text() })
            .mt(px(travel))
            .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                this.pressed = Some(key);
                cx.notify();
            }));

        let face = if round {
            face.size(px(48.)).rounded_full().child(
                // The record dot, lit a little toward its top left.
                div()
                    .size(px(14.))
                    .rounded_full()
                    .bg(linear_gradient(
                        160.,
                        linear_color_stop(t.key.with_l(t.key.l + 0.07).hsla(), 0.),
                        linear_color_stop(t.key.hsla(), 0.7),
                    ))
                    .shadow(vec![sh(0., 0., 3., 0., t.key.alpha(if down { 0.6 } else { 0.0 }))]),
            )
        } else {
            face.w(px(72.)).h(px(44.)).rounded(px(12.)).child(match key {
                Key::Play => "PLAY",
                _ => "STOP",
            })
        };
        // A fixed slot, so a pressed key doesn't move its neighbours.
        div().h(px(if round { 51. } else { 47. })).child(face)
    }
}
