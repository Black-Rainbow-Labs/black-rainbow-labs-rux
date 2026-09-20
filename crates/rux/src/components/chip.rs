//! A small pill that says something rather than does something.
//!
//! Six of them in the reference and they are one component: `.meta-chip`, `.volume-chip`,
//! `.scene__badge`, `.ic-elapsed`, `.ic-saveto` and `.ic-preset-chip`. What differs between them is
//! the depth, the fill and whether there is a dot, a mark or a dismiss on it.

use egui::{Color32, Rect, Response, Sense, Vec2};

use super::{interact, pointing};
use crate::icon::{Icon, Mark};
use crate::layout::{Cross, Flex, Pad};
use crate::text::Style;
use crate::theme::Opacity as _;
use crate::{Fill, Rux};

/// How the chip sits on the page.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ChipVariant {
    /// `.meta-chip`, `.volume-chip`, `.scene__badge`, `.ic-saveto` — pressed into the surface, which
    /// is how the reference says "this is a reading, not a control".
    #[default]
    Sunken,
    /// `.ic-elapsed` — raised off it, for something floating over a picture.
    Raised,
    /// `.ic-preset-chip` — a mint gradient with a glow. Something that is on.
    Accent,
}

/// A small pill.
pub struct Chip<'a> {
    label: &'a str,
    /// `.meta-chip b` — a second, darker run after the label.
    value: Option<&'a str>,
    /// `.volume-chip__dot` — a 7px disc with a halo of its own colour.
    dot: Option<Color32>,
    icon: Option<Icon>,
    /// `.ic-preset-chip__x` — a round dismiss at the right hand end.
    dismissible: bool,
    variant: ChipVariant,
    /// `.volume-chip` and `.ic-saveto__label` are set in mono; the rest are not.
    mono: bool,
    /// Which accent an `Accent` chip is. The reference's is mint.
    accent: Option<crate::theme::Gradient>,
}

impl<'a> Chip<'a> {
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            value: None,
            dot: None,
            icon: None,
            dismissible: false,
            variant: ChipVariant::default(),
            mono: false,
            accent: None,
        }
    }

    pub fn variant(mut self, variant: ChipVariant) -> Self {
        self.variant = variant;
        self
    }

    /// `.meta-chip b { color: var(--ink-900); font-weight: 600 }`.
    pub fn value(mut self, value: &'a str) -> Self {
        self.value = Some(value);
        self
    }

    pub fn dot(mut self, colour: Color32) -> Self {
        self.dot = Some(colour);
        self
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn dismissible(mut self, dismissible: bool) -> Self {
        self.dismissible = dismissible;
        self
    }

    pub fn mono(mut self, mono: bool) -> Self {
        self.mono = mono;
        self
    }

    /// Which gradient an `Accent` chip is filled with.
    pub fn accent(mut self, gradient: crate::theme::Gradient) -> Self {
        self.accent = Some(gradient);
        self
    }

    fn style(&self) -> Style {
        match self.mono {
            true => Style::mono(11.0).medium().tracking(0.02),
            false => Style::sans(11.0).medium(),
        }
    }

    /// `padding: 8px 14px` on the biggest of them, `4px 10px` on `.scene__badge`.
    fn pad(&self) -> Pad {
        match self.variant {
            ChipVariant::Accent => Pad::sides(6.0, 8.0, 6.0, 12.0),
            _ => Pad::axes(7.0, 14.0),
        }
    }

    pub fn measure(&self, rux: &Rux<'_>) -> Vec2 {
        let style = self.style();
        let mut width = rux.measure(style, self.label).x;
        if let Some(value) = self.value {
            width += 4.0 + rux.measure(style.semibold(), value).x;
        }
        let gap = 8.0;
        if self.dot.is_some() {
            width += 7.0 + gap;
        }
        if self.icon.is_some() {
            width += 12.0 + gap;
        }
        if self.dismissible {
            width += 18.0 + gap;
        }
        let pad = self.pad();
        Vec2::new(width + pad.horizontal(), rux.measure(style, "Ag").y + pad.vertical())
    }

    /// Draws, and answers whether the dismiss was pressed.
    pub fn show(self, rux: &mut Rux<'_>, rect: Rect) -> (Response, bool) {
        let theme = rux.theme();
        let radius = theme.radius.pill;
        let response = interact(rux, rect, "rux-chip", Sense::hover());

        let ink = match self.variant {
            ChipVariant::Accent => Color32::WHITE,
            _ => theme.ink.i700,
        };
        match self.variant {
            ChipVariant::Sunken => {
                rux.chrome.surface(rect, radius, theme.surface.s1, theme.elevation.pressed_sm);
            }
            ChipVariant::Raised => {
                rux.chrome.surface(rect, radius, theme.surface.s1, theme.elevation.raised_sm);
            }
            ChipVariant::Accent => {
                let gradient = self.accent.unwrap_or(theme.gradient.mint);
                rux.chrome.surface(
                    rect,
                    radius,
                    Fill::gradient(gradient, rect),
                    theme.glowing(gradient.to.at(0.30), 2.0),
                );
            }
        }

        let style = self.style();
        let pad = self.pad();
        let gap = 8.0;
        let mut line = Flex::row(rect).pad(pad).gap(gap).cross(Cross::Centre);
        if self.dot.is_some() {
            line = line.sized(Vec2::splat(7.0));
        }
        if self.icon.is_some() {
            line = line.sized(Vec2::splat(12.0));
        }
        let words = rux.measure(style, self.label).x;
        let value_width = self.value.map(|value| 4.0 + rux.measure(style.semibold(), value).x);
        line = line.hug(words + value_width.unwrap_or(0.0));
        if self.dismissible {
            line = line.sized(Vec2::splat(18.0));
        }
        let cells = line.solve_all();
        let mut cell = cells.iter();

        if let Some(colour) = self.dot {
            if let Some(at) = cell.next() {
                // `box-shadow: 0 0 6px <colour>` — the dot glows in its own colour.
                rux.chrome.glow(*at, 3.5, colour, 6.0);
                rux.chrome.disc(at.center(), 3.5, colour);
            }
        }
        if let Some(icon) = self.icon {
            if let Some(at) = cell.next() {
                rux.mark_in(Mark::new(icon, 12.0), *at, ink);
            }
        }
        if let Some(at) = cell.next() {
            let galley = rux.text(style, self.label, ink);
            let size = galley.size();
            let top = (at.center().y - size.y / 2.0).round();
            rux.ui.painter().galley(egui::pos2(at.left(), top), galley, ink);
            if let Some(value) = self.value {
                let strong = theme.ink.i900;
                let galley = rux.text(style.semibold(), value, strong);
                rux.ui.painter().galley(
                    egui::pos2(at.left() + size.x + 4.0, top),
                    galley,
                    strong,
                );
            }
        }

        let mut dismissed = false;
        if self.dismissible {
            if let Some(at) = cell.next() {
                // `.ic-preset-chip__x { width: 18px; border-radius: 50%; background: rgba(255,255,255,0.25) }`.
                let x = interact(rux, *at, "rux-chip-dismiss", Sense::click());
                pointing(rux, &x, true);
                let wash = match x.hovered() {
                    true => 0.4,
                    false => 0.25,
                };
                rux.chrome.disc(at.center(), 9.0, Color32::WHITE.at(wash));
                rux.mark(Mark::new(Icon::X, 9.0).stroke(2.4), at.center(), Color32::WHITE);
                dismissed = x.clicked();
                x.widget_info(|| {
                    egui::WidgetInfo::labeled(egui::WidgetType::Button, true, "Dismiss")
                });
            }
        }

        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Label, true, self.label)
        });
        (response, dismissed)
    }
}
