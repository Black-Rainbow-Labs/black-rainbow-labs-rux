//! A fader and a segment meter: the two ways the instrument shows a value along a line.
//!
//! A **fader** is a value somebody sets: a groove sunk into the plate, tick marks under it, the part of
//! the groove the value has travelled lit in its colour, and a raised rectangular cap with an engraved
//! line across it. It is dragged, pressed anywhere along the groove, or moved with the arrow keys.
//!
//! A **meter** is a value somebody reads: a row of segments on a screen, lit up to a fraction, the way a
//! level meter on a mixing desk is.

use egui::{Color32, Id, Pos2, Rect, Sense, Stroke, Vec2};

use super::instrument::{appearing, ease_out, lit_edge, screen, timing, Instrument};
use super::pointing;
use crate::render::Fill;
use crate::Rux;

/// What a fader reports.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct FaderOutcome {
    /// The new value, when it moved this frame.
    pub changed: Option<f64>,
    /// Whether it is being dragged.
    pub dragging: bool,
}

/// A fader over `min..=max` in steps of `step`.
pub struct Fader {
    value: f64,
    min: f64,
    max: f64,
    step: f64,
    colour: Option<Color32>,
    id: Id,
    label: String,
}

impl Fader {
    pub fn new(id: Id, value: f64, min: f64, max: f64) -> Self {
        Self {
            value,
            min,
            max: if max > min { max } else { min + 1.0 },
            step: 0.0,
            colour: None,
            id,
            label: "Fader".into(),
        }
    }

    pub fn step(mut self, step: f64) -> Self {
        self.step = step.max(0.0);
        self
    }

    pub fn colour(mut self, colour: Color32) -> Self {
        self.colour = Some(colour);
        self
    }

    /// The name an accessibility tree and a test find it by.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// Sixteen points of cap, three of gap and five of the tallest tick, and six to spare below.
    pub fn height(rux: &Rux<'_>) -> f32 {
        rux.z(30.0)
    }

    /// `value` snapped to the step and kept in range.
    pub fn snap(&self, value: f64) -> f64 {
        let clamped = value.clamp(self.min, self.max);
        match self.step > 0.0 {
            true => {
                let steps = ((clamped - self.min) / self.step).round();
                (self.min + steps * self.step).clamp(self.min, self.max)
            }
            false => clamped,
        }
    }

    /// The value at `x` across a groove running from `left` to `right`.
    pub fn value_at(&self, x: f32, left: f32, right: f32) -> f64 {
        let fraction = ((x - left) / (right - left).max(1.0)).clamp(0.0, 1.0) as f64;
        self.snap(self.min + fraction * (self.max - self.min))
    }

    /// How long the lit run is at `fraction` of a groove whose ticks run from `left` to `right`.
    ///
    /// Nothing at the smallest value and the whole of the ticks at the largest: the groove is the scale, so
    /// the run never starts before the first notch or ends after the last.
    pub fn lit_run(fraction: f32, left: f32, right: f32) -> f32 {
        (right - left).max(0.0) * fraction.clamp(0.0, 1.0)
    }

    pub fn show(self, rux: &mut Rux<'_>, rect: Rect) -> FaderOutcome {
        let theme = rux.theme();
        let colours = Instrument::of(theme);
        let colour = self.colour.unwrap_or(theme.accent.blue);
        // The cap is twelve points wide and eighteen tall; the value runs between the cap's centre at
        // each end, so the cap never leaves the rectangle.
        let half = rux.z(6.0);
        let left = rect.left() + half;
        let right = rect.right() - half;
        let middle = rect.top() + rux.z(8.0);
        let response = rux.ui.interact(rect, self.id, Sense::click_and_drag());
        pointing(rux, &response, true);
        if response.dragged() {
            rux.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
        }
        let mut outcome = FaderOutcome {
            changed: None,
            dragging: response.dragged(),
        };
        let mut value = self.value;
        if response.clicked() || response.dragged() {
            if let Some(at) = response.interact_pointer_pos() {
                let next = self.value_at(at.x, left, right);
                if (next - value).abs() > f64::EPSILON {
                    value = next;
                    outcome.changed = Some(next);
                }
            }
        }
        if response.has_focus() {
            let step = if self.step > 0.0 {
                self.step
            } else {
                (self.max - self.min) / 100.0
            };
            let (less, more) = rux.ctx().input(|input| {
                (
                    input.key_pressed(egui::Key::ArrowLeft)
                        || input.key_pressed(egui::Key::ArrowDown),
                    input.key_pressed(egui::Key::ArrowRight)
                        || input.key_pressed(egui::Key::ArrowUp),
                )
            });
            let next = self.snap(
                value
                    + if more {
                        step
                    } else if less {
                        -step
                    } else {
                        0.0
                    },
            );
            if (next - value).abs() > f64::EPSILON {
                value = next;
                outcome.changed = Some(next);
            }
        }
        if response.clicked() {
            response.request_focus();
        }

        // **The groove is the scale.** It runs from the centre of the cap at the smallest value to the
        // centre of the cap at the largest, exactly as the ticks and the value do, so at the smallest
        // value nothing is lit and nothing runs on past the last notch. It used to run the whole width of
        // the rectangle while the ticks ran between the cap's centres, so the lit run started seven points
        // left of the zero notch: Unluminous's `task-2219` reported it as the bar going over the notches.
        let snap = |v: f32| (v * rux.pixels_per_point()).round() / rux.pixels_per_point();
        let left = snap(left);
        let right = snap(right);
        let middle = snap(middle);
        let depth = rux.z(4.0);
        let groove = Rect::from_min_max(
            Pos2::new(left - depth / 2.0, middle - depth / 2.0),
            Pos2::new(right + depth / 2.0, middle + depth / 2.0),
        );
        rux.chrome.surface(
            groove,
            depth / 2.0,
            Fill::Solid(colours.screen),
            theme.elevation.pressed_sm,
        );
        let fraction = ((value - self.min) / (self.max - self.min)).clamp(0.0, 1.0) as f32;
        let at = snap(left + Fader::lit_run(fraction, left, right));
        // The run the value has travelled: a crisp line in the groove with a glow one point wide either
        // side and nothing wider. On the painter rather than in the chrome, because it moves under a drag
        // and the chrome would rasterise on every frame of one.
        let painter = rux.painter().clone();
        if at - left > 0.5 {
            let line = Rect::from_min_max(
                Pos2::new(left, middle - rux.z(1.0)),
                Pos2::new(at, middle + rux.z(1.0)),
            );
            painter.rect_filled(line.expand(rux.z(1.0)), rux.z(2.0), colour.gamma_multiply(0.22));
            painter.rect_filled(line, rux.z(1.0), colour);
        }
        // The cap: raised, rounded, with a lit edge and a grip of two engraved lines.
        let cap = Vec2::new(rux.z(12.0), rux.z(16.0));
        let cap_rect = Rect::from_center_size(Pos2::new(at, middle), cap);
        // Tick marks under the cap's travel, starting far enough below the cap that its shadow never
        // reaches them: eleven, the ends and the middle taller, each on a whole pixel.
        let ticks = snap(cap_rect.bottom() + rux.z(3.0));
        let hair = 1.0 / rux.pixels_per_point();
        for tick in 0..=10 {
            let x = snap(left + (right - left) * tick as f32 / 10.0) + hair / 2.0;
            let tall = if tick % 5 == 0 { 5.0 } else { 3.0 };
            painter.line_segment(
                [Pos2::new(x, ticks), Pos2::new(x, ticks + rux.z(tall))],
                Stroke::new(hair, match tick % 5 == 0 { true => theme.ink.i300, false => theme.ink.i200 }),
            );
        }
        let held = response.dragged() || response.is_pointer_button_down_on();
        let fill = match response.hovered() || held {
            true => theme.surface.s3,
            false => colours.plate,
        };
        rux.chrome.surface(
            cap_rect,
            rux.z(4.0),
            Fill::Solid(fill),
            theme.elevation.raised_sm,
        );
        // A hairline round the cap, because on a light plate the cap is the plate's own colour and only
        // its shadow said where it was: the grip lines then looked as if they floated beside it.
        painter.rect_stroke(
            cap_rect,
            rux.z(4.0),
            Stroke::new(hair, theme.ink.i200),
            egui::StrokeKind::Inside,
        );
        lit_edge(&painter, cap_rect.shrink(hair), rux.z(4.0), colours.edge);
        let grip = match held {
            true => colour,
            false => theme.ink.i300,
        };
        for dy in [-1.25_f32, 1.25] {
            let y = snap(middle + rux.z(dy)) + hair / 2.0;
            painter.line_segment(
                [
                    Pos2::new(cap_rect.left() + rux.z(3.5), y),
                    Pos2::new(cap_rect.right() - rux.z(3.5), y),
                ],
                Stroke::new(rux.z(1.0).max(hair), grip),
            );
        }
        if response.has_focus() {
            painter.rect_stroke(
                cap_rect.expand(rux.z(2.0)),
                rux.z(5.0),
                Stroke::new(1.0, theme.interaction.focus_ring),
                egui::StrokeKind::Outside,
            );
        }
        response.widget_info(|| egui::WidgetInfo::slider(true, value, self.label.clone()));
        outcome
    }
}

/// A row of segments lit up to a fraction.
pub struct Meter {
    fraction: f32,
    colour: Option<Color32>,
    /// The colour of the last tenth, as a level meter goes amber near the top.
    top: Option<Color32>,
    id: Option<Id>,
}

impl Meter {
    pub fn new(fraction: f32) -> Self {
        Self {
            fraction: fraction.clamp(0.0, 1.0),
            colour: None,
            top: None,
            id: None,
        }
    }

    pub fn colour(mut self, colour: Color32) -> Self {
        self.colour = Some(colour);
        self
    }

    pub fn top(mut self, colour: Color32) -> Self {
        self.top = Some(colour);
        self
    }

    /// The id the meter rises from zero under, the first time it is drawn.
    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    pub fn height(rux: &Rux<'_>) -> f32 {
        rux.z(21.0)
    }

    pub fn show(self, rux: &mut Rux<'_>, rect: Rect) {
        let theme = rux.theme();
        let colours = Instrument::of(theme);
        screen(rux, rect, rux.z(7.0), false);
        let inside = rect.shrink2(Vec2::new(rux.z(7.0), rux.z(6.0)));
        let gap = rux.z(2.0);
        let count = ((inside.width() + gap) / rux.z(8.0)).floor().max(4.0) as usize;
        let width = (inside.width() - gap * (count as f32 - 1.0)) / count as f32;
        let rise = match self.id {
            Some(id) => ease_out(appearing(rux, id, 0.0, timing::RISE * 1.5)),
            None => 1.0,
        };
        let lit = (self.fraction * rise * count as f32).round() as usize;
        let colour = self.colour.unwrap_or(theme.accent.blue);
        let painter = rux.painter().clone();
        for segment in 0..count {
            let x = inside.left() + segment as f32 * (width + gap);
            let cell = Rect::from_min_size(
                Pos2::new(x, inside.top()),
                Vec2::new(width, inside.height()),
            );
            let near_top = segment as f32 >= count as f32 * 0.9;
            let tint = match (segment < lit, near_top, self.top) {
                (true, true, Some(top)) => top,
                (true, _, _) => colour,
                (false, _, _) => colours.led_off,
            };
            if segment < lit {
                painter.rect_filled(
                    cell.expand(rux.z(1.5)),
                    rux.z(2.5),
                    tint.gamma_multiply(0.14),
                );
            }
            painter.rect_filled(cell, rux.z(1.5), tint);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_lit_run_starts_at_the_first_notch_and_ends_at_the_last() {
        assert_eq!(Fader::lit_run(0.0, 10.0, 110.0), 0.0, "nothing is lit at the smallest value");
        assert_eq!(Fader::lit_run(1.0, 10.0, 110.0), 100.0, "the largest value lights the ticks and no more");
        assert_eq!(Fader::lit_run(1.4, 10.0, 110.0), 100.0, "and never runs past the last notch");
        assert_eq!(Fader::lit_run(0.5, 10.0, 110.0), 50.0);
    }

    #[test]
    fn a_fader_snaps_to_its_step_and_stays_in_range() {
        let fader = Fader::new(Id::new("f"), 5.0, 0.0, 12.0).step(0.5);
        assert_eq!(fader.snap(5.26), 5.5);
        assert_eq!(fader.snap(40.0), 12.0);
        assert_eq!(fader.value_at(50.0, 0.0, 100.0), 6.0);
        assert_eq!(fader.value_at(-10.0, 0.0, 100.0), 0.0);
    }
}
