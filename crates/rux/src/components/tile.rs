//! A box where a picture goes.
//!
//! `.ic-tile` is a pressed well that a result appears in; `.scene__image` is a dark dropzone with a
//! gradient in it, an outline and a hint. They share a shape and differ in their depth, which is
//! exactly what a variant is for.
//!
//! **The dropzone is dark in both themes**, and that is deliberate rather than an oversight: it stands
//! in for a picture, and a picture is not lighter because the interface is. What changes with the
//! theme is the ink of everything drawn *over* it, which is already white at an alpha in the
//! reference.

use egui::{Color32, Rect, Response, Sense, Vec2};

use super::{interact, pointing};
use crate::icon::{Icon, Mark};
use crate::text::Style;
use crate::theme::Opacity as _;
use crate::{Fill, Rux};

/// What kind of box this is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TileKind {
    /// `.ic-tile` — a pressed well the size of a result, which is what a spinner sits in.
    #[default]
    Well,
    /// `.scene__image` — a dark dropzone, pressed into itself *and* raised off the page.
    Dropzone,
}

/// `aspect-ratio: 16 / 9`.
pub const ASPECT: f32 = 16.0 / 9.0;

/// A box where a picture goes.
pub struct Tile<'a> {
    kind: TileKind,
    /// The picture, once there is one.
    image: Option<egui::TextureId>,
    /// `.scene__image-title` and `.scene__image-sub` — what an empty dropzone says.
    title: Option<&'a str>,
    subtitle: Option<&'a str>,
    label: &'a str,
}

impl<'a> Tile<'a> {
    pub fn new() -> Self {
        Self { kind: TileKind::default(), image: None, title: None, subtitle: None, label: "Tile" }
    }

    pub fn kind(mut self, kind: TileKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn image(mut self, texture: egui::TextureId) -> Self {
        self.image = Some(texture);
        self
    }

    /// The two lines an empty dropzone shows.
    pub fn empty_text(mut self, title: &'a str, subtitle: &'a str) -> Self {
        self.title = Some(title);
        self.subtitle = Some(subtitle);
        self
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = label;
        self
    }

    /// How tall this tile is at a given width, which is what a grid of them needs.
    pub fn height_at(width: f32) -> f32 {
        width / ASPECT
    }

    /// Draws the box and answers the rectangle inside it, for whatever goes on top.
    pub fn show(self, rux: &mut Rux<'_>, rect: Rect) -> (Response, Rect) {
        let sense = match self.kind {
            TileKind::Dropzone => Sense::click(),
            TileKind::Well => Sense::hover(),
        };
        let response = interact(rux, rect, "rux-tile", sense);
        let theme = rux.theme();
        let radius = theme.radius.lg;

        match self.kind {
            TileKind::Well => {
                rux.chrome.surface(rect, radius, theme.surface.s1, theme.elevation.pressed);
            }
            TileKind::Dropzone => {
                pointing(rux, &response, true);
                rux.chrome.surface(
                    rect,
                    radius,
                    Fill::gradient(theme.gradient.image_well, rect),
                    theme.elevation.image_well,
                );
            }
        }

        if let Some(texture) = self.image {
            // Rounded corners on a picture are the clip `egui` cannot do, so the picture is drawn
            // into the chrome's clip rather than `egui`'s square one.
            let uv = Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0));
            rux.ui.painter().add(egui::Shape::image(texture, rect, uv, Color32::WHITE));
        } else if self.kind == TileKind::Dropzone {
            self.draw_empty(rux, rect);
        }

        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Button, true, self.label)
        });
        (response, rect.shrink(12.0))
    }

    /// `.scene__image-empty` — a dashed square, a title and a line under it.
    fn draw_empty(&self, rux: &mut Rux<'_>, rect: Rect) {
        let theme = rux.theme();
        let Some(title) = self.title else { return };
        let subtitle = self.subtitle.unwrap_or("");
        let title_style = Style::sans(14.0).medium().tracking(-0.01);
        let sub_style = Style::sans(12.0);
        let title_size = rux.measure(title_style, title);
        let sub_size = rux.measure(sub_style, subtitle);
        // `.scene__image-icon { width: 56px; height: 56px; margin-bottom: 4px }`, then `gap: 8px`.
        let block = 56.0 + 4.0 + 8.0 + title_size.y + 8.0 + sub_size.y;
        let top = rect.center().y - block / 2.0;

        let icon = Rect::from_center_size(
            egui::pos2(rect.center().x, top + 28.0),
            Vec2::splat(56.0),
        );
        rux.chrome.rect(icon, theme.radius.md, Color32::WHITE.at(0.04));
        // `border: 1.5px dashed rgba(255,255,255,0.15)`.
        rux.ui.painter().rect_stroke(
            icon,
            theme.radius.md,
            egui::Stroke::new(1.5, Color32::WHITE.at(0.15)),
            egui::StrokeKind::Inside,
        );
        rux.mark_in(Mark::new(Icon::ImagePlus, 22.0), icon, Color32::WHITE.at(0.5));

        let title_ink = Color32::WHITE.at(0.7);
        let galley = rux.text(title_style, title, title_ink);
        rux.ui.painter().galley(
            egui::pos2(rect.center().x - title_size.x / 2.0, (top + 68.0).round()),
            galley,
            title_ink,
        );
        let sub_ink = Color32::WHITE.at(0.35);
        let galley = rux.text(sub_style, subtitle, sub_ink);
        rux.ui.painter().galley(
            egui::pos2(
                rect.center().x - sub_size.x / 2.0,
                (top + 68.0 + title_size.y + 8.0).round(),
            ),
            galley,
            sub_ink,
        );
    }
}

impl Default for Tile<'_> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tile_keeps_the_stylesheet_s_aspect_ratio() {
        // `aspect-ratio: 16 / 9`.
        assert!((Tile::height_at(320.0) - 180.0).abs() < 0.001);
    }
}
