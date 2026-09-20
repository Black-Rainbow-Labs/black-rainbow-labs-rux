//! `.avatar` — a round chip standing in for a person.
//!
//! A violet gradient with their initials on it, raised a little, and an optional status dot at the
//! bottom right. The dot is `border: 2px solid var(--surface-1); box-sizing: content-box`, which is a
//! disc of the page's own colour behind a smaller coloured one — it reads as a hole punched in the
//! avatar rather than as an outline, and it works over any background because the ring is the page.

use egui::{Color32, Pos2, Rect, Response, Sense, Vec2};

use super::interact;
use crate::text::Style;
use crate::{Fill, Rux};

/// A round chip standing in for a person.
pub struct Avatar<'a> {
    /// What is drawn on it. The reference shows two letters.
    initials: &'a str,
    /// `.avatar__status` — mint when they are here.
    status: Option<Color32>,
    diameter: f32,
    name: &'a str,
}

impl<'a> Avatar<'a> {
    /// `.avatar { width: 40px; height: 40px }`.
    pub const SIZE: f32 = 40.0;

    pub fn new(initials: &'a str) -> Self {
        Self { initials, status: None, diameter: Self::SIZE, name: "Account" }
    }

    pub fn status(mut self, colour: Color32) -> Self {
        self.status = Some(colour);
        self
    }

    pub fn diameter(mut self, diameter: f32) -> Self {
        self.diameter = diameter;
        self
    }

    /// The whole name, for a screen reader and a tooltip — the initials are not a name.
    pub fn name(mut self, name: &'a str) -> Self {
        self.name = name;
        self
    }

    pub fn measure(&self, _rux: &Rux<'_>) -> Vec2 {
        Vec2::splat(self.diameter)
    }

    pub fn show(self, rux: &mut Rux<'_>, rect: Rect) -> Response {
        let side = self.diameter.min(rect.width()).min(rect.height());
        let rect = Rect::from_center_size(rect.center(), Vec2::splat(side));
        let response = interact(rux, rect, "rux-avatar", Sense::click());
        let theme = rux.theme();

        rux.chrome.surface(
            rect,
            side / 2.0,
            Fill::gradient(theme.gradient.avatar, rect),
            theme.elevation.raised_sm,
        );
        // `font-size: 13px; font-weight: 600` at the reference's own diameter, scaled with it.
        let style = Style::sans(13.0 * side / Self::SIZE).semibold();
        let galley = rux.text(style, self.initials, Color32::WHITE);
        crate::text::draw_centred(rux.ui.painter(), rect, galley, Color32::WHITE);

        if let Some(colour) = self.status {
            // `bottom: 1px; right: 1px; width: 10px` plus a 2px ring.
            let scale = side / Self::SIZE;
            let radius = 5.0 * scale;
            let centre = Pos2::new(
                rect.right() - 1.0 * scale - radius,
                rect.bottom() - 1.0 * scale - radius,
            );
            rux.chrome.disc(centre, radius + 2.0 * scale, theme.surface.s1);
            rux.chrome.disc(centre, radius, colour);
        }

        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Button, true, self.name)
        });
        response
    }
}
