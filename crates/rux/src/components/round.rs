//! The Agent-Chat design's buttons: pills with words in them, and round buttons with a mark.
//!
//! `task-2235`. The design's rule is that everything a person presses is round or a pill. Three pills
//! (primary, secondary, ghost) and three round buttons (38 for the header, 30 for the small keys on a
//! card and the gear, 44 for send), each with five looks: at rest, under the pointer, held down, with
//! the keyboard, and unavailable. The source is the canvas's "Chat parts" artboard, whose button grid
//! draws all thirty.
//!
//! Every surface reads [`crate::theme::Chat`], so the buttons follow whichever pane they are on.
//!
//! **A press sinks.** Held down, a raised button becomes a well carved into the pane with its ink at
//! full strength, and the action's gradient presses into itself. That is the one change that
//! rasterises the layer again; a hover changes only the ink, which is a vertex colour.

use egui::{Color32, Pos2, Rect, Response, Sense, Vec2};

use super::{pointing, Visual};
use crate::icon::{Icon, Mark};
use crate::render::{Corners, Fill};
use crate::text::Style;
use crate::theme::{Chat, Elevation, Gradient};
use crate::Rux;

/// What a pill is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PillVariant {
    /// The one thing the card is for: `Apply change`, `Allow once`. The action's gradient.
    Primary,
    /// The other thing worth doing: `Open in editor`, `Retry`. A raised card.
    #[default]
    Secondary,
    /// The way out: `Copy`, `Deny`, `Reject`. No surface until it is pressed.
    Ghost,
}

/// A pill with words in it, and a mark before them when one is given.
pub struct PillButton<'a> {
    text: &'a str,
    variant: PillVariant,
    mark: Option<Icon>,
    enabled: bool,
    /// Draw the focus ring whether or not the keyboard is here, which is what a sheet of states does.
    ring: bool,
    /// Draw it held down whatever the pointer is doing, for the same reason.
    held: bool,
    hovered: bool,
}

impl<'a> PillButton<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            text,
            variant: PillVariant::default(),
            mark: None,
            enabled: true,
            ring: false,
            held: false,
            hovered: false,
        }
    }

    pub fn variant(mut self, variant: PillVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn primary(self) -> Self {
        self.variant(PillVariant::Primary)
    }

    pub fn ghost(self) -> Self {
        self.variant(PillVariant::Ghost)
    }

    /// A mark before the words: `Retry` carries the rerun arrow.
    pub fn mark(mut self, icon: Icon) -> Self {
        self.mark = Some(icon);
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Show a state regardless of the pointer and the keyboard: `hover`, `held` and `ring`.
    pub fn showing(mut self, hovered: bool, held: bool, ring: bool) -> Self {
        self.hovered = hovered;
        self.held = held;
        self.ring = ring;
        self
    }

    fn style(&self) -> Style {
        // `font-size: 12.5px`, at 600 for the primary, 500 for the secondary and 400 for a ghost.
        let style = Style::sans(12.5);
        match self.variant {
            PillVariant::Primary => style.semibold(),
            PillVariant::Secondary => style.medium(),
            PillVariant::Ghost => style,
        }
    }

    /// `padding: 0 18px`, `0 16px` and `0 14px`, and `0 16px 0 12px` with a mark and a 6 point gap.
    fn padding(&self) -> (f32, f32) {
        match (self.mark.is_some(), self.variant) {
            (true, _) => (12.0, 16.0),
            (false, PillVariant::Primary) => (18.0, 18.0),
            (false, PillVariant::Secondary) => (16.0, 16.0),
            (false, PillVariant::Ghost) => (14.0, 14.0),
        }
    }

    /// `height: 36px`, as wide as its words.
    pub fn measure(&self, rux: &Rux<'_>) -> Vec2 {
        let words = rux.measure(rux.zs(self.style()), self.text).x;
        let (left, right) = self.padding();
        let mark = match self.mark {
            Some(_) => rux.z(17.0 + 6.0),
            None => 0.0,
        };
        rux.screen().target(Vec2::new(words + mark + rux.z(left + right), rux.z(36.0)))
    }

    pub fn show(self, rux: &mut Rux<'_>, rect: Rect) -> Response {
        let height = rux.z(36.0).min(rect.height());
        let rect = Rect::from_center_size(rect.center(), Vec2::new(rect.width(), height));
        let response = super::interact_target(rux, rect, "rux-pill", Sense::click());
        let visual = forced(Visual::of(&response, self.enabled, rux.screen().pointer), self.hovered, self.held);
        pointing(rux, &response, self.enabled);
        let chat = rux.theme().chat;
        let ring = self.ring || (response.has_focus() && self.enabled);
        let radius = rect.height() / 2.0;
        if ring {
            focus_ring(rux, rect, radius, &chat);
        }
        let fade = visual.fade();
        match (self.variant, visual) {
            (PillVariant::Primary, Visual::Active) => {
                surface(rux, rect, radius, action(&chat.action, rect), chat.action_pressed(), fade)
            }
            (PillVariant::Primary, Visual::Hover) => {
                surface(rux, rect, radius, action(&chat.action_hover, rect), chat.action_raised(), fade)
            }
            (PillVariant::Primary, _) => {
                surface(rux, rect, radius, action(&chat.action, rect), chat.action_raised(), fade)
            }
            (_, Visual::Active) => surface(rux, rect, radius, chat.well.into(), chat.carved_sm(), fade),
            (PillVariant::Secondary, _) => {
                surface(rux, rect, radius, card(&chat, rect), chat.raised_sm(), fade)
            }
            (PillVariant::Ghost, _) => {}
        }
        let theme = rux.theme();
        let ink = match (self.variant, visual) {
            (PillVariant::Primary, _) => Color32::WHITE,
            (_, Visual::Active) => theme.ink.i900,
            (_, Visual::Hover) => chat.sky,
            (PillVariant::Secondary, _) => theme.ink.i700,
            (PillVariant::Ghost, _) => theme.ink.i500,
        }
        .gamma_multiply(fade);
        let style = rux.zs(self.style());
        let galley = rux.text(style, self.text, ink);
        let (left, right) = self.padding();
        let mark_width = match self.mark {
            Some(_) => rux.z(17.0 + 6.0),
            None => 0.0,
        };
        let content = galley.size().x + mark_width;
        // Centred in whatever width it was handed, which is its own width when it was measured.
        let room = rect.width() - rux.z(left + right);
        let start = rect.left() + rux.z(left) + ((room - content) / 2.0).max(0.0);
        if let Some(icon) = self.mark {
            let centre = Pos2::new(start + rux.z(8.5), rect.center().y);
            rux.mark(Mark::new(icon, rux.z(17.0)), centre, ink);
        }
        crate::text::draw_left_capitals(
            rux.painter(),
            Pos2::new(start + mark_width, rect.center().y),
            galley,
            style,
            ink,
        );
        response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, self.enabled, self.text));
        response
    }
}

/// The round buttons' diameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RoundSize {
    /// 30: the gear, a card's chevron, copy and stop.
    Small,
    /// 34: the close button of the Chat settings dialog.
    Close,
    /// 38: history and new conversation in the header.
    #[default]
    Header,
    /// 44: send, and stop while an answer arrives.
    Send,
}

impl RoundSize {
    pub fn diameter(self) -> f32 {
        match self {
            RoundSize::Small => 30.0,
            RoundSize::Close => 34.0,
            RoundSize::Header => 38.0,
            RoundSize::Send => 44.0,
        }
    }
}

/// What is inside a round button.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RoundContent {
    /// A mark, at the size and stroke given.
    Mark(Mark),
    /// The paper airplane on the action's gradient: send.
    Send,
    /// A small square in the slower colour on a raised card: stop.
    Stop,
}

/// A round button.
pub struct RoundButton {
    content: RoundContent,
    size: RoundSize,
    label: &'static str,
    enabled: bool,
    /// Pressed in because what it opens is open, as the gear is while the settings are showing.
    on: bool,
    /// No surface at rest, as copy and run again are under an answer.
    ghost: bool,
    ring: bool,
    held: bool,
    hovered: bool,
}

impl RoundButton {
    pub fn new(content: RoundContent, label: &'static str) -> Self {
        Self {
            content,
            size: RoundSize::default(),
            label,
            enabled: true,
            on: false,
            ghost: false,
            ring: false,
            held: false,
            hovered: false,
        }
    }

    /// A round button with a mark in it.
    pub fn mark(icon: Icon, size: f32, label: &'static str) -> Self {
        Self::new(RoundContent::Mark(Mark::new(icon, size)), label)
    }

    /// The round send button.
    pub fn send(label: &'static str) -> Self {
        Self::new(RoundContent::Send, label).size(RoundSize::Send)
    }

    /// The round stop button, at the size of send.
    pub fn stop(label: &'static str) -> Self {
        Self::new(RoundContent::Stop, label).size(RoundSize::Send)
    }

    pub fn size(mut self, size: RoundSize) -> Self {
        self.size = size;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn on(mut self, on: bool) -> Self {
        self.on = on;
        self
    }

    pub fn ghost(mut self) -> Self {
        self.ghost = true;
        self
    }

    /// Show a state regardless of the pointer and the keyboard: `hover`, `held` and `ring`.
    pub fn showing(mut self, hovered: bool, held: bool, ring: bool) -> Self {
        self.hovered = hovered;
        self.held = held;
        self.ring = ring;
        self
    }

    pub fn measure(&self, rux: &Rux<'_>) -> Vec2 {
        rux.screen().target(Vec2::splat(rux.z(self.size.diameter())))
    }

    pub fn show(self, rux: &mut Rux<'_>, rect: Rect) -> Response {
        let side = rux.z(self.size.diameter()).min(rect.width()).min(rect.height());
        let rect = Rect::from_center_size(rect.center(), Vec2::splat(side));
        let response = super::interact_target(rux, rect, "rux-round", Sense::click());
        let visual = forced(Visual::of(&response, self.enabled, rux.screen().pointer), self.hovered, self.held);
        pointing(rux, &response, self.enabled);
        let chat = rux.theme().chat;
        let theme = rux.theme();
        let radius = side / 2.0;
        if self.ring || (response.has_focus() && self.enabled) {
            focus_ring(rux, rect, radius, &chat);
        }
        let fade = visual.fade();
        let sunk = matches!(visual, Visual::Active) || self.on;
        match self.content {
            RoundContent::Send => {
                let (fill, elevation) = match visual {
                    Visual::Active => (&chat.action, chat.action_pressed()),
                    Visual::Hover => (&chat.action_hover, chat.action_raised()),
                    _ => (&chat.action, chat.action_raised()),
                };
                surface(rux, rect, radius, action(fill, rect), elevation, fade);
            }
            _ if sunk => surface(rux, rect, radius, chat.well.into(), chat.carved_sm(), fade),
            _ if self.ghost => {
                if matches!(visual, Visual::Hover) {
                    rux.wash(rect, radius, theme.interaction.hover);
                }
            }
            _ => surface(rux, rect, radius, card(&chat, rect), chat.raised_sm(), fade),
        }
        let ink = match (self.content, visual) {
            (RoundContent::Send, _) => Color32::WHITE,
            (RoundContent::Stop, _) => chat.slower,
            (_, Visual::Hover) => chat.sky,
            (_, Visual::Active) if self.on => chat.sky,
            (_, Visual::Active) => theme.ink.i900,
            _ if self.on => chat.sky,
            _ if self.ghost => theme.ink.i400,
            _ => theme.ink.i500,
        }
        .gamma_multiply(fade);
        match self.content {
            RoundContent::Mark(mark) => {
                let mark = Mark { size: rux.z(mark.size), ..mark };
                rux.mark(mark, rect.center(), ink);
            }
            RoundContent::Send => {
                // `margin: 1px 2px 0 0` on the 18 point mark: the airplane's weight is up and to the
                // right, so it is moved down and left to look centred.
                let centre = rect.center() + Vec2::new(rux.z(-1.0), rux.z(0.5));
                rux.mark(Mark::new(Icon::Send, rux.z(18.0)), centre, ink);
            }
            RoundContent::Stop => {
                // `width: 12px; height: 12px; border-radius: 3px` in a 44, and 9 by 2 in a 30.
                let (square, round) = match self.size {
                    RoundSize::Small => (9.0, 2.0),
                    _ => (12.0, 3.0),
                };
                let square = Rect::from_center_size(rect.center(), Vec2::splat(rux.z(square)));
                rux.painter().rect_filled(square, rux.z(round), ink);
            }
        }
        response.widget_info(|| {
            egui::WidgetInfo::selected(egui::WidgetType::Button, self.enabled, self.on, self.label)
        });
        response
    }
}

/// A pill switch: a carved track holding one pill per choice, with the chosen one raised out of it.
///
/// The Chat settings dialog's model choice: `claude`, `codex`, `local qwen 27B`. The track is
/// `padding: 5px; gap: 4px` with the choices sharing its width equally, each `height: 36px`.
pub struct PillSwitch<'a> {
    choices: &'a [String],
    chosen: Option<usize>,
    label: &'static str,
}

impl<'a> PillSwitch<'a> {
    pub fn new(choices: &'a [String], chosen: Option<usize>, label: &'static str) -> Self {
        Self { choices, chosen, label }
    }

    /// As wide as it is given, 46 points tall.
    pub fn measure(&self, rux: &Rux<'_>) -> Vec2 {
        Vec2::new(rux.z(240.0), rux.z(46.0))
    }

    /// Draws the switch and answers the choice pressed this frame, if any.
    pub fn show(self, rux: &mut Rux<'_>, rect: Rect) -> Option<usize> {
        let chat = rux.theme().chat;
        let theme = rux.theme();
        let height = rux.z(46.0).min(rect.height());
        let track = Rect::from_center_size(rect.center(), Vec2::new(rect.width(), height));
        rux.chrome.surface(track, track.height() / 2.0, chat.well, chat.carved_sm());
        let count = self.choices.len().max(1) as f32;
        let pad = rux.z(5.0);
        let gap = rux.z(4.0);
        let width = (track.width() - pad * 2.0 - gap * (count - 1.0)) / count;
        let mut picked = None;
        for (index, choice) in self.choices.iter().enumerate() {
            let left = track.left() + pad + index as f32 * (width + gap);
            let pill = Rect::from_min_size(Pos2::new(left, track.top() + pad), Vec2::new(width, track.height() - pad * 2.0));
            let response = super::interact(rux, pill, "rux-pill-switch", Sense::click());
            pointing(rux, &response, true);
            let chosen = self.chosen == Some(index);
            if chosen {
                rux.chrome.surface(pill, pill.height() / 2.0, card(&chat, pill), chat.raised_sm());
            }
            let ink = match (chosen, super::hovering(rux, &response)) {
                (true, _) => theme.ink.i900,
                (false, true) => chat.sky,
                (false, false) => theme.ink.i500,
            };
            let style = rux.zs(Style::sans(12.0).medium());
            let galley = rux.elided(style, choice, ink, (pill.width() - rux.z(16.0)).max(0.0));
            crate::text::draw_centred(rux.painter(), pill, galley, ink);
            let name = format!("{}: {choice}", self.label);
            response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::RadioButton, true, chosen, name.clone()));
            if response.clicked() {
                picked = Some(index);
            }
        }
        picked
    }
}

/// The state a sheet of looks asked for, over the one the pointer is in.
fn forced(visual: Visual, hovered: bool, held: bool) -> Visual {
    match (visual, held, hovered) {
        (Visual::Disabled, _, _) => Visual::Disabled,
        (_, true, _) => Visual::Active,
        (_, false, true) => Visual::Hover,
        (visual, _, _) => visual,
    }
}

/// The card's own gradient across `rect`.
pub(crate) fn card(chat: &Chat, rect: Rect) -> Fill {
    Fill::gradient(chat.card, rect)
}

/// The action's gradient across `rect`.
fn action(gradient: &Gradient, rect: Rect) -> Fill {
    Fill::gradient(*gradient, rect)
}

/// A surface, at `fade` of its strength: `opacity: 0.4` on a disabled control takes its shadows with it.
pub(crate) fn surface(
    rux: &mut Rux<'_>,
    rect: Rect,
    radius: impl Into<Corners>,
    fill: Fill,
    elevation: Elevation,
    fade: f32,
) {
    if fade >= 1.0 {
        rux.chrome.surface(rect, radius, fill, elevation);
        return;
    }
    let fill = match fill {
        Fill::Solid(colour) => Fill::Solid(colour.gamma_multiply(fade)),
        Fill::Linear { from, to, start, end } => Fill::Linear {
            from,
            to,
            start: start.gamma_multiply(fade),
            end: end.gamma_multiply(fade),
        },
    };
    let elevation = Elevation::of(elevation.shadows().map(|mut shadow| {
        shadow.colour = shadow.colour.gamma_multiply(fade);
        shadow
    }));
    rux.chrome.surface(rect, radius, fill, elevation);
}

/// `--ring: 0 0 0 2px <pane>, 0 0 0 4px rgba(92, 182, 255, 0.6)`, drawn under the control.
pub(crate) fn focus_ring(rux: &mut Rux<'_>, rect: Rect, radius: f32, chat: &Chat) {
    let outer = rect.expand(rux.z(4.0));
    let inner = rect.expand(rux.z(2.0));
    rux.chrome.rect(outer, radius + rux.z(4.0), chat.focus());
    rux.chrome.rect(inner, radius + rux.z(2.0), chat.ground);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_diameters_are_the_canvas_s() {
        assert_eq!(RoundSize::Small.diameter(), 30.0);
        assert_eq!(RoundSize::Header.diameter(), 38.0);
        assert_eq!(RoundSize::Send.diameter(), 44.0);
    }

    #[test]
    fn a_disabled_control_stays_disabled_whatever_a_sheet_asks_for() {
        assert_eq!(forced(Visual::Disabled, true, true), Visual::Disabled);
        // `forced(visual, hovered, held)`.
        assert_eq!(forced(Visual::Idle, false, true), Visual::Active);
        assert_eq!(forced(Visual::Idle, true, false), Visual::Hover);
        assert_eq!(forced(Visual::Hover, false, false), Visual::Hover);
    }
}
