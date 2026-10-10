//! A status well: a small round well carved into a card, holding one mark that says what state
//! something is in.
//!
//! `task-2235`. The Agent-Chat design has no status dots: "state is an icon in a carved well". Five
//! states, each a mark and a colour, read off the canvas's "Status" panel: waiting is a clock in the
//! quiet ink, running is an arc turning in a track, done is a tick in the faster colour, needs your OK
//! is a key in the waiting colour, and failed is a cross in the slower colour.
//!
//! **The running arc turns as a texture**, through [`crate::icon::paint_arc`], for the reason the
//! spinner does: an arc recorded as decoration would change the decoration on every frame and
//! rasterise the whole layer sixty times a second.

use egui::{Color32, Rect, Response, Sense, Vec2};

use crate::icon::{Icon, Mark};
use crate::Rux;

/// What state something is in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Status {
    /// Queued behind the current step.
    Waiting,
    /// A command or a reply in progress.
    Running,
    /// Finished without an error.
    Done,
    /// Paused until somebody allows it.
    NeedsOk,
    /// Stopped; the error says why.
    Failed,
}

impl Status {
    /// What a person is told this state is, which is also what a screen reader is told.
    pub fn label(self) -> &'static str {
        match self {
            Status::Waiting => "Waiting",
            Status::Running => "Running",
            Status::Done => "Done",
            Status::NeedsOk => "Needs your OK",
            Status::Failed => "Failed",
        }
    }

    /// The mark in the well, or `None` for the arc that turns.
    pub fn icon(self) -> Option<Icon> {
        match self {
            Status::Waiting => Some(Icon::Clock),
            Status::Running => None,
            Status::Done => Some(Icon::Tick),
            Status::NeedsOk => Some(Icon::Key),
            Status::Failed => Some(Icon::Cross),
        }
    }
}

/// How long the running arc takes to go once round.
const TURN_SECONDS: f64 = 1.1;

/// A 34 point well with a state in it.
pub struct StatusWell {
    status: Status,
    diameter: f32,
}

impl StatusWell {
    pub fn new(status: Status) -> Self {
        Self { status, diameter: 34.0 }
    }

    /// A different diameter; the mark and the arc keep their proportion to it.
    pub fn diameter(mut self, diameter: f32) -> Self {
        self.diameter = diameter;
        self
    }

    pub fn measure(&self, rux: &Rux<'_>) -> Vec2 {
        Vec2::splat(rux.z(self.diameter))
    }

    /// The colour the state is drawn in.
    pub fn colour(rux: &Rux<'_>, status: Status) -> Color32 {
        let theme = rux.theme();
        match status {
            Status::Waiting => theme.ink.i400,
            Status::Running => theme.chat.sky,
            Status::Done => theme.chat.faster,
            Status::NeedsOk => theme.chat.waiting,
            Status::Failed => theme.chat.slower,
        }
    }

    pub fn show(self, rux: &mut Rux<'_>, rect: Rect) -> Response {
        let side = rux.z(self.diameter).min(rect.width()).min(rect.height());
        let rect = Rect::from_center_size(rect.center(), Vec2::splat(side));
        let response = super::interact(rux, rect, "rux-status-well", Sense::hover());
        let chat = rux.theme().chat;
        rux.chrome.surface(rect, side / 2.0, chat.well, chat.carved_sm());
        // The design draws an 18 point mark in a 34 point well, and a 20 point arc box.
        let unit = side / 34.0;
        let colour = Self::colour(rux, self.status);
        match self.status.icon() {
            Some(icon) => rux.mark(Mark::new(icon, 18.0 * unit), rect.center(), colour),
            None => {
                // `r="7.5" stroke-width="2.5"`, the track in `#2c323b` and a 14 of 47 dash in sky.
                let radius = 7.5 * unit;
                let width = 2.5 * unit;
                rux.chrome.ring(rect.center(), radius, width, chat.track);
                let sweep = 14.0 / 7.5;
                let turn = match rux.state.is_still() {
                    true => 0.0,
                    false => {
                        let time = rux.ctx().input(|input| input.time);
                        rux.ctx().request_repaint();
                        ((time / TURN_SECONDS).fract() * std::f64::consts::TAU) as f32
                    }
                };
                let painter = rux.painter().clone();
                crate::icon::paint_arc(
                    &painter,
                    &mut rux.state.icons.borrow_mut(),
                    rect.center(),
                    radius,
                    width,
                    sweep,
                    turn,
                    colour,
                    colour,
                );
            }
        }
        let label = self.status.label();
        response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Label, true, label));
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_state_but_running_has_a_mark_and_every_state_has_words() {
        let all = [Status::Waiting, Status::Running, Status::Done, Status::NeedsOk, Status::Failed];
        for status in all {
            assert!(!status.label().is_empty());
            assert_eq!(status.icon().is_none(), status == Status::Running);
        }
    }
}
