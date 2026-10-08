//! Key caps: a button that sinks when it is pressed, and a checkbox that stays down when it is ticked.
//!
//! The instrument language's controls (see `instrument.rs`). A key is a raised cap with a lit top
//! edge; pressing it sinks it a point and turns its shadow inward over [`timing::PRESS`]. A key that
//! matters more than its neighbours is not painted a colour: its LED is lit. A key that is chosen, in a
//! row of choices or a strip of tabs, stays down with its LED lit.

use egui::{Color32, Id, Pos2, Rect, Response, Sense, Vec2};

use super::instrument::{led_for, lit_edge, sinking, timing, toward, Instrument};
use super::{interact, pointing};
use crate::render::Fill;
use crate::text::{self, Style};
use crate::Rux;

/// The words on a key.
const LABEL: Style = Style::sans(12.0).medium().tracking(-0.005);

/// A key cap with a label and, optionally, an LED.
pub struct Key<'a> {
    label: &'a str,
    led: Option<Color32>,
    lit: bool,
    down: bool,
    enabled: bool,
    id: Option<Id>,
    compact: bool,
}

impl<'a> Key<'a> {
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            led: None,
            lit: false,
            down: false,
            enabled: true,
            id: None,
            compact: false,
        }
    }

    /// An LED in `colour` at the left of the label, lit or dark.
    pub fn led(mut self, colour: Color32, lit: bool) -> Self {
        self.led = Some(colour);
        self.lit = lit;
        self
    }

    /// Held down, as the chosen key in a row of choices is.
    pub fn down(mut self, down: bool) -> Self {
        self.down = down;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// The id its motion is remembered under. Without one it is worked out from where the key is,
    /// which is right for a key that does not move.
    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    /// Twenty four points tall rather than thirty, for a badge or a key in a dense row.
    pub fn compact(mut self) -> Self {
        self.compact = true;
        self
    }

    fn style(&self, rux: &Rux<'_>) -> Style {
        let style = match self.compact {
            true => LABEL.at(11.0),
            false => LABEL,
        };
        rux.zs(style)
    }

    /// The width the label and the LED need, and the key's height.
    pub fn measure(&self, rux: &Rux<'_>) -> Vec2 {
        let words = rux.measure(self.style(rux), self.label).x;
        let led = match self.led {
            Some(_) => rux.z(6.0 + 8.0),
            None => 0.0,
        };
        let pad = rux.z(if self.compact { 10.0 } else { 13.0 });
        let height = rux.z(if self.compact { 24.0 } else { 30.0 });
        Vec2::new(
            (words + led + pad * 2.0).ceil(),
            rux.screen().target_height(height),
        )
    }

    pub fn show(self, rux: &mut Rux<'_>, rect: Rect) -> Response {
        let response = match self.id {
            Some(id) => rux.ui.interact(rect, id, Sense::click()),
            None => interact(rux, rect, "rux-key", Sense::click()),
        };
        pointing(rux, &response, self.enabled);
        let theme = rux.theme();
        let colours = Instrument::of(theme);
        let held = self.down || (self.enabled && response.is_pointer_button_down_on());
        let sink = toward(rux, response.id.with("sink"), held, timing::PRESS);
        let radius = rux.z(if self.compact { 7.0 } else { 9.0 });
        let face = rect.translate(Vec2::new(0.0, sink * rux.z(1.0)));
        let lifted = match response.hovered() && self.enabled && !held {
            // A key under the pointer is a shade lighter, which is all a hover is on hardware.
            true => theme.surface.s3,
            false => colours.plate,
        };
        rux.chrome.surface(
            face,
            radius,
            Fill::Solid(lifted),
            sinking(theme.elevation.raised_sm, theme.elevation.pressed_sm, sink),
        );
        if sink < 0.5 {
            lit_edge(
                rux.painter(),
                face,
                radius,
                colours.edge.gamma_multiply(1.0 - sink * 2.0),
            );
        }
        let ink = match (self.enabled, self.lit || self.down) {
            (false, _) => theme.ink.i300,
            (true, true) => theme.ink.i900,
            (true, false) => theme.ink.i700,
        };
        let pad = rux.z(if self.compact { 10.0 } else { 13.0 });
        let mut pen = face.left() + pad;
        if let Some(colour) = self.led {
            let size = rux.z(if self.compact { 2.5 } else { 3.0 });
            led_for(
                rux,
                response.id,
                Pos2::new(pen + size, face.center().y),
                size,
                colour,
                self.lit || self.down,
            );
            pen += rux.z(6.0 + 8.0);
        }
        let style = self.style(rux);
        let galley = text::elided(
            rux.painter(),
            style,
            self.label,
            ink,
            (face.right() - pad - pen).max(0.0),
        );
        text::draw_left_capitals(
            rux.painter(),
            Pos2::new(pen, face.center().y),
            galley,
            style,
            ink,
        );
        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Button, self.enabled, self.label)
        });
        response
    }
}

/// A checkbox: a small key cap with an LED in it, down and lit when ticked, with its label beside it.
pub struct Checkbox<'a> {
    on: bool,
    label: &'a str,
    colour: Option<Color32>,
    id: Option<Id>,
    settled: bool,
}

/// The words beside a checkbox.
const CHECK_LABEL: Style = Style::sans(12.5).leading(1.45);

impl<'a> Checkbox<'a> {
    pub fn new(on: bool, label: &'a str) -> Self {
        Self {
            on,
            label,
            colour: None,
            id: None,
            settled: true,
        }
    }

    /// Whether ticking it quiets its words, as a done item on a checklist is. True unless told otherwise;
    /// a setting that is switched on is not finished with, so a form's toggle says false.
    pub fn settles(mut self, settled: bool) -> Self {
        self.settled = settled;
        self
    }

    /// The LED's colour when ticked; the theme's mint when left out.
    pub fn colour(mut self, colour: Color32) -> Self {
        self.colour = Some(colour);
        self
    }

    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    /// How tall it is at `width`, with its label wrapped.
    pub fn height_at(&self, rux: &Rux<'_>, width: f32) -> f32 {
        let cap = rux.z(20.0);
        let words = (width - cap - rux.z(10.0)).max(rux.z(40.0));
        let galley = text::wrapped(
            rux.painter(),
            rux.zs(CHECK_LABEL),
            self.label,
            Color32::WHITE,
            words,
        );
        galley.size().y.max(cap)
    }

    /// Draw it into `rect`. The whole row answers a press, so the words are as good a target as the cap.
    pub fn show(self, rux: &mut Rux<'_>, rect: Rect) -> Response {
        let response = match self.id {
            Some(id) => rux.ui.interact(rect, id, Sense::click()),
            None => interact(rux, rect, "rux-checkbox", Sense::click()),
        };
        pointing(rux, &response, true);
        let theme = rux.theme();
        let colours = Instrument::of(theme);
        let side = rux.z(20.0);
        let style = rux.zs(CHECK_LABEL);
        // The cap sits on the first line of the label, centred on its capitals.
        let first_line = rux.measure(style, "Ag").y;
        let cap = Rect::from_min_size(
            Pos2::new(
                rect.left(),
                rect.top() + ((first_line - side) / 2.0).max(0.0),
            ),
            Vec2::splat(side),
        );
        let held = self.on || response.is_pointer_button_down_on();
        let sink = toward(rux, response.id.with("sink"), held, timing::PRESS);
        let radius = rux.z(6.0);
        let fill = match response.hovered() && !held {
            true => theme.surface.s3,
            false => colours.plate,
        };
        rux.chrome.surface(
            cap,
            radius,
            Fill::Solid(fill),
            sinking(theme.elevation.raised_sm, theme.elevation.pressed_sm, sink),
        );
        if sink < 0.5 {
            lit_edge(
                rux.painter(),
                cap,
                radius,
                colours.edge.gamma_multiply(1.0 - sink * 2.0),
            );
        }
        let colour = self.colour.unwrap_or(theme.accent.mint);
        led_for(rux, response.id, cap.center(), rux.z(3.0), colour, self.on);
        let ink = match self.on && self.settled {
            true => theme.ink.i400,
            false => theme.ink.i700,
        };
        let left = cap.right() + rux.z(10.0);
        let galley = text::wrapped(
            rux.painter(),
            style,
            self.label,
            ink,
            (rect.right() - left).max(rux.z(40.0)),
        );
        rux.painter()
            .galley(Pos2::new(left, rect.top()), galley, ink);
        response.widget_info(|| {
            egui::WidgetInfo::selected(egui::WidgetType::Checkbox, true, self.on, self.label)
        });
        response
    }
}
