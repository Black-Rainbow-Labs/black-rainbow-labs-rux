//! A number with an up and a down tick beside it.
//!
//! `.ic-count` and `.num-input`. The rule that decides its shape is `width: 66px; flex-shrink: 0` —
//! it is the one control in the reference that refuses to give way when the row it is in runs out of
//! room, because a number squeezed to nothing is a number nobody can read. [`crate::layout::Length::Fixed`]
//! is how a caller says that.

use egui::{Rect, Sense, Vec2};

use super::{interact, pointing, Visual};
use crate::icon::{Icon, Mark};
use crate::layout::{Cross, Flex, Pad};
use crate::text::Style;
use crate::Rux;

/// What happened this frame.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct StepperOutcome {
    /// The new value, if a tick moved it.
    pub changed: Option<i64>,
}

/// `.ic-count { width: 66px }`.
pub const STEPPER_WIDTH: f32 = 66.0;
/// `.ic-count__steps button { width: 18px; height: 11px }`, `gap: 1px`.
const TICK: Vec2 = Vec2::new(18.0, 11.0);
const TICK_GAP: f32 = 1.0;
/// How much room the number keeps between the two ticks of the wide form. Four digits of
/// [`Style::NUMBER`] and a little air.
const NUMBER_WIDTH: f32 = 34.0;

/// A number with two ticks.
pub struct Stepper {
    value: i64,
    min: i64,
    max: i64,
    enabled: bool,
    label: &'static str,
}

impl Stepper {
    pub fn new(value: i64, min: i64, max: i64) -> Self {
        Self { value, min, max, enabled: true, label: "Count" }
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn label(mut self, label: &'static str) -> Self {
        self.label = label;
        self
    }

    /// The room it wants.
    ///
    /// On a touch screen it asks for the wide form in [`Stepper::show_touch`] instead of the stacked
    /// one, because a stacked tick is 18 by 11 points and no amount of growing the control round it
    /// makes an 11 point tick pressable.
    pub fn measure(&self, rux: &Rux<'_>) -> Vec2 {
        match rux.screen().min_target() {
            target if target > 0.0 => {
                Vec2::new((target * 2.0 + NUMBER_WIDTH).max(STEPPER_WIDTH), target)
            }
            _ => Vec2::new(STEPPER_WIDTH, TICK.y * 2.0 + TICK_GAP + 8.0),
        }
    }

    pub fn show(self, rux: &mut Rux<'_>, rect: Rect) -> StepperOutcome {
        let theme = rux.theme();
        // `.ic-count { border-radius: var(--r-md); box-shadow: var(--e-pressed-sm) }`.
        rux.chrome.surface(rect, theme.radius.md, theme.surface.s1, theme.elevation.pressed_sm);
        if rux.screen().min_target() > 0.0 {
            return self.show_touch(rux, rect);
        }

        // `padding: 4px 4px 4px 10px; gap: 4px`, with the number 24 wide and the ticks 18.
        let [number, ticks] = Flex::row(rect)
            .pad(Pad::sides(4.0, 4.0, 4.0, 10.0))
            .gap(4.0)
            .cross(Cross::Centre)
            .fill(1.0)
            .sized(Vec2::new(TICK.x, TICK.y * 2.0 + TICK_GAP))
            .solve();

        self.draw_number(rux, number);

        let [up, down] = Flex::column(ticks).gap(TICK_GAP).fixed(TICK.y).fixed(TICK.y).solve();
        let mut outcome = StepperOutcome::default();
        // `.ic-count__steps button:first-child svg { transform: rotate(180deg) }` — the up tick is the
        // same chevron turned over, which is why there is no separate `chevUp` in the icon set.
        if self.tick(rux, up, true, Icon::ChevDown, std::f32::consts::PI) && self.value < self.max {
            outcome.changed = Some((self.value + 1).min(self.max));
        }
        if self.tick(rux, down, false, Icon::ChevDown, 0.0) && self.value > self.min {
            outcome.changed = Some((self.value - 1).max(self.min));
        }
        outcome
    }

    /// The wide form: a decrement, the number, an increment, each tick a full minimum target.
    ///
    /// **The two chevrons are the reference's own `chevLeft` and `chevRight` marks** and the tick keeps
    /// the same `--e-raised-sm` / `--e-pressed-sm` pair it has on a desktop, so nothing here is a new
    /// drawing — it is the same two shapes at the size a finger needs. The reference has no minus mark
    /// and this does not invent one.
    fn show_touch(self, rux: &mut Rux<'_>, rect: Rect) -> StepperOutcome {
        let target = rux.screen().min_target();
        let [down, number, up] = Flex::row(rect)
            .pad(Pad::all(3.0))
            .gap(2.0)
            .cross(Cross::Stretch)
            .fixed(target - 6.0)
            .fill(1.0)
            .fixed(target - 6.0)
            .solve();

        self.draw_number(rux, number);

        let mut outcome = StepperOutcome::default();
        if self.tick(rux, down, false, Icon::ChevLeft, 0.0) && self.value > self.min {
            outcome.changed = Some((self.value - 1).max(self.min));
        }
        if self.tick(rux, up, true, Icon::ChevRight, 0.0) && self.value < self.max {
            outcome.changed = Some((self.value + 1).min(self.max));
        }
        outcome
    }

    /// `.num-input` — the value, centred in whatever room is left.
    fn draw_number(&self, rux: &mut Rux<'_>, rect: Rect) {
        let ink = rux.theme().ink.i900.gamma_multiply(Visual::Idle.fade());
        let text = self.value.to_string();
        let galley = rux.text(Style::NUMBER, &text, ink);
        crate::text::draw_centred(rux.ui.painter(), rect, galley, ink);
    }

    /// One tick: `width: 18px; height: 11px; border-radius: 3px; box-shadow: var(--e-raised-sm)`.
    fn tick(&self, rux: &mut Rux<'_>, rect: Rect, up: bool, icon: Icon, turn: f32) -> bool {
        let at_limit = match up {
            true => self.value >= self.max,
            false => self.value <= self.min,
        };
        let enabled = self.enabled && !at_limit;
        let salt = match up {
            true => "rux-stepper-up",
            false => "rux-stepper-down",
        };
        let response = interact(rux, rect, salt, Sense::click());
        let visual = Visual::of(&response, enabled, rux.screen().pointer);
        pointing(rux, &response, enabled);

        let theme = rux.theme();
        let elevation = match visual {
            Visual::Active => theme.elevation.pressed_sm,
            _ => theme.elevation.raised_sm,
        };
        rux.chrome.surface(rect, 3.0, theme.surface.s1, elevation);
        let ink = match visual {
            Visual::Hover => theme.accent.blue,
            _ => theme.ink.i500,
        }
        .gamma_multiply(visual.fade());
        // `<SbIcon name="chevDown" size={9} stroke={2.4}/>`, and the same stroke at the size a finger
        // needs on the wide form.
        let mark = match rect.height() > TICK.y * 2.0 {
            true => 15.0,
            false => 9.0,
        };
        rux.mark(Mark::new(icon, mark).stroke(2.4).turn(turn), rect.center(), ink);
        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, self.label)
        });
        response.clicked() && enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_measurements_are_the_stylesheet_s() {
        // `.ic-count { width: 66px }`, `.ic-count__steps button { width: 18px; height: 11px }`.
        assert_eq!(STEPPER_WIDTH, 66.0);
        assert_eq!(TICK, Vec2::new(18.0, 11.0));
        assert_eq!(TICK_GAP, 1.0);
    }
}
