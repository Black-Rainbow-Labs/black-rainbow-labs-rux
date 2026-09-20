//! Three things that switch: a segmented control, a round toggle and a switch.
//!
//! `.view-toggle` and `.ic-modal-tabs` are the same control at two sizes — a pressed track with a
//! raised pill on the chosen one, which is the neumorphic way of saying "this one" without a colour.
//! `.ic-dice` is a round toggle whose on state is a violet gradient with a glow. The switch is the
//! style guide's `.toggle__knob` with `--grad-knob`.

use egui::{Color32, Rect, Response, Sense, Vec2};

use super::{interact, pointing, Visual};
use crate::icon::{Icon, Mark};
use crate::layout::{Cross, Flex, Pad};
use crate::text::Style;
use crate::theme::Opacity as _;
use crate::{Fill, Rux};

/// What happened to a segmented control this frame.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SegmentedOutcome {
    pub chosen: Option<usize>,
}

/// One row of mutually exclusive choices inside a pressed track.
pub struct Segmented<'a> {
    items: &'a [(&'a str, Option<Icon>)],
    active: usize,
    /// `.ic-modal-tabs` is the smaller of the two: `padding: 4px`, rows of `6px 10px` at 11px.
    compact: bool,
    /// `.ic-modal-tab.is-active[data-kind="image"] { color: var(--accent-blue) }` — the chosen row can
    /// take an accent of its own, which is how the modal's four tabs are told apart.
    accents: Option<&'a [Color32]>,
}

impl<'a> Segmented<'a> {
    pub fn new(items: &'a [(&'a str, Option<Icon>)], active: usize) -> Self {
        Self { items, active, compact: false, accents: None }
    }

    pub fn compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        self
    }

    pub fn accents(mut self, accents: &'a [Color32]) -> Self {
        self.accents = Some(accents);
        self
    }

    fn track_pad(&self) -> f32 {
        match self.compact {
            true => 4.0,
            false => 5.0,
        }
    }

    fn item_pad(&self) -> Pad {
        match self.compact {
            true => Pad::axes(6.0, 10.0),
            false => Pad::axes(7.0, 14.0),
        }
    }

    fn style(&self) -> Style {
        match self.compact {
            true => Style::sans(11.0).medium(),
            false => Style::CONTROL,
        }
    }

    fn mark(&self) -> f32 {
        match self.compact {
            true => 11.0,
            false => 13.0,
        }
    }

    fn widths(&self, rux: &Rux<'_>) -> Vec<f32> {
        let pad = self.item_pad();
        let gap = match self.compact {
            true => 5.0,
            false => 6.0,
        };
        self.items
            .iter()
            .map(|(label, icon)| {
                let words = rux.measure(self.style(), label).x;
                let mark = icon.map(|_| self.mark() + gap).unwrap_or(0.0);
                words + mark + pad.horizontal()
            })
            .collect()
    }

    pub fn measure(&self, rux: &Rux<'_>) -> Vec2 {
        let widths = self.widths(rux);
        // `gap: 2px` between the rows in the track.
        let gaps = 2.0 * widths.len().saturating_sub(1) as f32;
        let line = rux.measure(self.style(), "Ag").y;
        Vec2::new(
            widths.iter().sum::<f32>() + gaps + self.track_pad() * 2.0,
            line.max(self.mark()) + self.item_pad().vertical() + self.track_pad() * 2.0,
        )
    }

    pub fn show(self, rux: &mut Rux<'_>, rect: Rect) -> SegmentedOutcome {
        let theme = rux.theme();
        rux.chrome.surface(rect, theme.radius.pill, theme.surface.s1, theme.elevation.pressed_sm);

        let widths = self.widths(rux);
        let mut line =
            Flex::row(rect).pad(Pad::all(self.track_pad())).gap(2.0).cross(Cross::Stretch);
        for width in &widths {
            line = line.hug(*width);
        }
        let cells = line.solve_all();

        let mut outcome = SegmentedOutcome::default();
        let gap = match self.compact {
            true => 5.0,
            false => 6.0,
        };
        for (index, ((label, icon), cell)) in self.items.iter().zip(cells).enumerate() {
            let response = interact(rux, cell, "rux-segment", Sense::click());
            pointing(rux, &response, true);
            if response.clicked() {
                outcome.chosen = Some(index);
            }
            let active = index == self.active;
            let ink = match (active, self.accents.and_then(|list| list.get(index)), response.hovered())
            {
                (true, Some(accent), _) => *accent,
                (true, None, _) => theme.ink.i900,
                (false, _, true) => theme.ink.i900,
                (false, _, false) => theme.ink.i500,
            };
            if active {
                // `.is-active { background: var(--surface-1); box-shadow: var(--e-raised-sm) }`.
                rux.chrome.surface(
                    cell,
                    theme.radius.pill,
                    theme.surface.s1,
                    theme.elevation.raised_sm,
                );
            }
            let mut inner = Flex::row(cell).pad(self.item_pad()).gap(gap).cross(Cross::Centre);
            if icon.is_some() {
                inner = inner.sized(Vec2::splat(self.mark()));
            }
            let words = rux.measure(self.style(), label).x;
            let cells = inner.hug(words).solve_all();
            let mut slot = cells.iter();
            if let (Some(icon), Some(at)) = (icon, slot.next()) {
                rux.mark_in(Mark::new(*icon, self.mark()), *at, ink);
            }
            if let Some(at) = slot.next() {
                let galley = rux.text(self.style(), label, ink);
                crate::text::draw_left_centre(rux.ui.painter(), at.left_center(), galley, ink);
            }
            response.widget_info(|| {
                egui::WidgetInfo::selected(egui::WidgetType::SelectableLabel, true, active, *label)
            });
        }
        outcome
    }
}

/// `.ic-dice` — a round toggle beside a control, for "randomise this one".
///
/// Off it is pressed into the surface and faint; on it is a violet gradient with a glow under it and a
/// lit top edge. Two genuinely different declarations, which is what makes the state readable without
/// a label.
pub struct DiceToggle {
    on: bool,
    icon: Icon,
    label: &'static str,
    enabled: bool,
}

impl DiceToggle {
    /// `.ic-dice { width: 28px; height: 28px }`.
    pub const SIZE: f32 = 28.0;

    pub fn new(on: bool) -> Self {
        Self { on, icon: Icon::Cube, label: "Randomize", enabled: true }
    }

    /// The reference draws a die; the icon set has no die, so a caller picks the mark. `Icon::Cube` is
    /// the default because it is the nearest thing to one in the set.
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = icon;
        self
    }

    pub fn label(mut self, label: &'static str) -> Self {
        self.label = label;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn measure(&self, _rux: &Rux<'_>) -> Vec2 {
        Vec2::splat(Self::SIZE)
    }

    pub fn show(self, rux: &mut Rux<'_>, rect: Rect) -> Response {
        let side = Self::SIZE.min(rect.width()).min(rect.height());
        let rect = Rect::from_center_size(rect.center(), Vec2::splat(side));
        let response = interact(rux, rect, "rux-dice", Sense::click());
        let visual = Visual::of(&response, self.enabled);
        pointing(rux, &response, self.enabled);

        let theme = rux.theme();
        let radius = side / 2.0;
        let ink = match (self.on, visual) {
            (true, _) => Color32::WHITE,
            (false, Visual::Hover) => theme.accent.violet,
            (false, _) => theme.ink.i400,
        }
        .gamma_multiply(visual.fade());

        if self.on {
            let gradient = theme.gradient.violet;
            rux.chrome.surface(
                rect,
                radius,
                Fill::gradient(gradient, rect),
                crate::theme::Elevation::of([
                    crate::theme::Shadow::outer(-2.0, -2.0, 4.0, theme.shadow.light),
                    crate::theme::Shadow::outer(2.0, 2.0, 6.0, gradient.to.at(0.4)),
                    crate::theme::Shadow::inset(1.0, 1.0, 1.0, Color32::WHITE.at(0.3)),
                ]),
            );
        } else {
            rux.chrome.surface(rect, radius, theme.surface.s1, theme.elevation.pressed_sm);
        }
        rux.mark(Mark::new(self.icon, 13.0), rect.center(), ink);
        response.widget_info(|| {
            egui::WidgetInfo::selected(egui::WidgetType::Checkbox, self.enabled, self.on, self.label)
        });
        response
    }
}

/// A switch: a pressed track with a raised knob that slides.
///
/// Not on the Image Creator page, but `--grad-knob` and `.toggle__knob` are in the token file and a
/// component library without a switch is a component library somebody has to work around.
pub struct Switch {
    on: bool,
    enabled: bool,
    label: &'static str,
}

impl Switch {
    /// Wide enough for the knob to have somewhere to go: 44 by 24 is the ratio the token file's knob
    /// gradient was drawn for.
    pub const SIZE: Vec2 = Vec2::new(44.0, 24.0);

    pub fn new(on: bool) -> Self {
        Self { on, enabled: true, label: "Toggle" }
    }

    pub fn label(mut self, label: &'static str) -> Self {
        self.label = label;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn measure(&self, _rux: &Rux<'_>) -> Vec2 {
        Self::SIZE
    }

    pub fn show(self, rux: &mut Rux<'_>, rect: Rect) -> Response {
        let rect = Rect::from_center_size(rect.center(), Self::SIZE.min(rect.size()));
        let response = interact(rux, rect, "rux-switch", Sense::click());
        let visual = Visual::of(&response, self.enabled);
        pointing(rux, &response, self.enabled);

        let theme = rux.theme();
        let radius = rect.height() / 2.0;
        let track = match self.on {
            true => Fill::gradient(theme.gradient.primary, rect),
            false => Fill::Solid(theme.surface.s1),
        };
        rux.chrome.surface(rect, radius, track, theme.elevation.pressed_sm);

        let knob = radius - 3.0;
        let centre = match self.on {
            true => egui::pos2(rect.right() - 3.0 - knob, rect.center().y),
            false => egui::pos2(rect.left() + 3.0 + knob, rect.center().y),
        };
        let knob_rect = Rect::from_center_size(centre, Vec2::splat(knob * 2.0));
        rux.chrome.surface(
            knob_rect,
            knob,
            Fill::gradient(theme.gradient.knob, knob_rect),
            theme.elevation.raised_sm,
        );
        let _ = visual;
        response.widget_info(|| {
            egui::WidgetInfo::selected(egui::WidgetType::Checkbox, self.enabled, self.on, self.label)
        });
        response
    }
}
