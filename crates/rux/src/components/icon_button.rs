//! A round button with a mark in it, and nothing else.
//!
//! Six of them in the reference, at five diameters: `.toolbtn` and `.iconbtn` at 40, `.dock__btn` at
//! 48, `.ic-modal__close` at 32, `.ic-prompt-shuffle` and `.ic-dice` at 28, and `.ic-chev` at 24. They
//! differ in exactly two things — the diameter and whether the surface is raised or pressed — which is
//! why they are one component with a size rather than six.

use egui::{Color32, Pos2, Rect, Response, Sense, Vec2};

use super::{pointing, Visual};
use crate::icon::{Icon, Mark};
use crate::text::Style;
use crate::theme::Opacity as _;
use crate::{Fill, Rux};

/// The diameters the reference uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IconButtonSize {
    /// `.ic-chev` — 24, the collapse handle floating on a panel's border.
    Tiny,
    /// `.ic-prompt-shuffle`, `.ic-dice` — 28.
    Small,
    /// `.ic-modal__close` — 32.
    Medium,
    /// `.toolbtn`, `.iconbtn` — 40, which is the ordinary one.
    #[default]
    Large,
    /// `.dock__btn` — 48, a floating action.
    Huge,
}

impl IconButtonSize {
    pub fn diameter(self) -> f32 {
        match self {
            IconButtonSize::Tiny => 24.0,
            IconButtonSize::Small => 28.0,
            IconButtonSize::Medium => 32.0,
            IconButtonSize::Large => 40.0,
            IconButtonSize::Huge => 48.0,
        }
    }

    /// The mark inside, at the size the reference passes `<SbIcon size={…}/>`.
    fn mark(self) -> f32 {
        match self {
            IconButtonSize::Tiny => 12.0,
            IconButtonSize::Small => 13.0,
            IconButtonSize::Medium => 14.0,
            IconButtonSize::Large => 17.0,
            IconButtonSize::Huge => 19.0,
        }
    }
}

/// What the surface under the mark is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IconButtonVariant {
    /// `.toolbtn`, `.iconbtn`, `.ic-chev` — the surface, raised a little.
    #[default]
    Raised,
    /// `.ic-dice` when it is off — pressed into the surface, so it reads as a switch rather than as a
    /// button.
    Sunken,
    /// `.dock__btn--primary` — a coral gradient with a glow under it.
    Primary,
    /// `.scene__menu-btn` — no surface at all until the pointer is on it.
    Ghost,
}

/// A round button with a mark in it.
pub struct IconButton {
    icon: Icon,
    size: IconButtonSize,
    variant: IconButtonVariant,
    /// `.toolbtn__dot` — a status dot in the top right, with a halo and a ring in the page's colour.
    dot: Option<Color32>,
    /// `.dock__badge` — a count in the top right.
    badge: Option<u32>,
    /// Overrides the ink, for the marks the reference tints: `.ic-saveto__spark` is amber and a
    /// sub-group's own mark takes its section's accent.
    tint: Option<Color32>,
    /// `.ic-panel.is-collapsed .ic-chev svg { transform: rotate(180deg) }`.
    turn: f32,
    enabled: bool,
    /// What a screen reader is told this is. `SbIcon` names are not words a person would recognise.
    label: &'static str,
}

impl IconButton {
    pub fn new(icon: Icon, label: &'static str) -> Self {
        Self {
            icon,
            size: IconButtonSize::default(),
            variant: IconButtonVariant::default(),
            dot: None,
            badge: None,
            tint: None,
            turn: 0.0,
            enabled: true,
            label,
        }
    }

    pub fn size(mut self, size: IconButtonSize) -> Self {
        self.size = size;
        self
    }

    pub fn variant(mut self, variant: IconButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn dot(mut self, colour: Color32) -> Self {
        self.dot = Some(colour);
        self
    }

    pub fn badge(mut self, count: u32) -> Self {
        self.badge = Some(count);
        self
    }

    pub fn tint(mut self, colour: Color32) -> Self {
        self.tint = Some(colour);
        self
    }

    pub fn turn(mut self, radians: f32) -> Self {
        self.turn = radians;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// The room it wants, which on a touch screen is the minimum target rather than the circle.
    ///
    /// **The circle itself never grows.** Three of the five diameters — 24, 28 and 32 — are under the
    /// 44 points a finger needs, so on a phone the control claims a 44 point box and draws its own
    /// circle centred in it. Growing the layout rather than only the hit rectangle is what keeps two
    /// neighbouring icon buttons from claiming overlapping targets; see [`super::touch_target`].
    pub fn measure(&self, rux: &Rux<'_>) -> Vec2 {
        rux.screen().target(Vec2::splat(self.size.diameter()))
    }

    pub fn show(self, rux: &mut Rux<'_>, rect: Rect) -> Response {
        // Square and centred whatever it was handed, because a circle drawn into an oblong is an oval.
        let side = self.size.diameter().min(rect.width()).min(rect.height());
        let rect = Rect::from_center_size(rect.center(), Vec2::splat(side));
        // The drawn circle is `rect`; what a finger may press is that circle grown to the minimum
        // target. On a mouse the minimum is zero and the two are the same rectangle.
        let response = super::interact_target(rux, rect, "rux-icon-button", Sense::click());
        let visual = Visual::of(&response, self.enabled, rux.screen().pointer);
        pointing(rux, &response, self.enabled);

        let theme = rux.theme();
        let radius = side / 2.0;
        let pressed = matches!(visual, Visual::Active);

        let ink = match (self.tint, self.variant, visual) {
            (_, IconButtonVariant::Primary, _) => Color32::WHITE,
            (Some(tint), _, _) => tint,
            // `.toolbtn:hover { color: var(--accent-blue) }`.
            (None, _, Visual::Hover) => theme.accent.blue,
            (None, IconButtonVariant::Sunken, _) => theme.ink.i400,
            (None, _, _) => theme.ink.i700,
        }
        .gamma_multiply(visual.fade());

        match self.variant {
            IconButtonVariant::Ghost => {
                if matches!(visual, Visual::Hover | Visual::Active) {
                    rux.wash(rect, radius, theme.interaction.hover);
                }
            }
            IconButtonVariant::Primary => {
                let glow = theme.gradient.danger_diagonal.to.at(0.30);
                rux.chrome.surface(
                    rect,
                    radius,
                    Fill::gradient(theme.gradient.danger_diagonal, rect),
                    theme.glowing(glow, 3.0),
                );
            }
            IconButtonVariant::Sunken => {
                rux.chrome.surface(
                    rect,
                    radius,
                    theme.surface.s1,
                    theme.elevation.pressed_sm,
                );
            }
            IconButtonVariant::Raised => {
                // `:active { box-shadow: var(--e-pressed-sm) }` — every round button in the reference
                // presses in rather than dimming.
                let elevation = match pressed {
                    true => theme.elevation.pressed_sm,
                    false => theme.elevation.raised_sm,
                };
                rux.chrome.surface(rect, radius, theme.surface.s1, elevation);
            }
        }

        rux.mark(
            Mark::new(self.icon, self.size.mark()).turn(self.turn),
            rect.center(),
            ink,
        );

        if let Some(colour) = self.dot {
            self.paint_dot(rux, rect, colour);
        }
        if let Some(count) = self.badge {
            self.paint_badge(rux, rect, count);
        }

        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Button, self.enabled, self.label)
        });
        response
    }

    /// `.toolbtn__dot` — 8 across, `top: 8px; right: 8px`, a 6px halo and a 2px ring of the page.
    ///
    /// The ring is `border: 2px solid var(--surface-1); box-sizing: content-box`, which is a disc of
    /// the page's own colour behind a smaller coloured one — so it reads as a hole punched in the
    /// button rather than as an outline.
    fn paint_dot(&self, rux: &mut Rux<'_>, rect: Rect, colour: Color32) {
        let theme = rux.theme();
        let scale = self.size.diameter() / 40.0;
        let radius = 4.0 * scale;
        let centre = Pos2::new(rect.right() - 8.0 * scale - radius, rect.top() + 8.0 * scale + radius);
        rux.chrome.glow(
            Rect::from_center_size(centre, Vec2::splat(radius * 2.0)),
            radius,
            colour,
            6.0,
        );
        rux.chrome.disc(centre, radius + 2.0 * scale, theme.surface.s1);
        rux.chrome.disc(centre, radius, colour);
    }

    /// `.dock__badge` — `min-width: 18px; height: 18px; border-radius: 9px`, mono 10px semibold.
    fn paint_badge(&self, rux: &mut Rux<'_>, rect: Rect, count: u32) {
        let theme = rux.theme();
        let text = count.to_string();
        let style = Style::mono(10.0).semibold();
        let words = rux.measure(style, &text);
        let width = (words.x + 10.0).max(18.0);
        let badge = Rect::from_min_size(
            Pos2::new(rect.right() - width + 2.0, rect.top() - 2.0),
            Vec2::new(width, 18.0),
        );
        rux.chrome.glow(badge.translate(Vec2::new(0.0, 2.0)), 9.0, theme.accent.coral.at(0.4), 6.0);
        rux.chrome.rect(badge, 9.0, theme.accent.coral);
        let galley = rux.text(style, &text, Color32::WHITE);
        crate::text::draw_centred(rux.ui.painter(), badge, galley, Color32::WHITE);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_diameters_are_the_stylesheet_s() {
        // `.ic-chev` 24, `.ic-dice` 28, `.ic-modal__close` 32, `.toolbtn` 40, `.dock__btn` 48.
        let sizes = [
            (IconButtonSize::Tiny, 24.0),
            (IconButtonSize::Small, 28.0),
            (IconButtonSize::Medium, 32.0),
            (IconButtonSize::Large, 40.0),
            (IconButtonSize::Huge, 48.0),
        ];
        for (size, diameter) in sizes {
            assert_eq!(size.diameter(), diameter);
            assert!(size.mark() < diameter, "the mark has to fit inside the button");
        }
    }
}
