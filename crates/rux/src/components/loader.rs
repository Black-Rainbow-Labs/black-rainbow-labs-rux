//! Waiting: a spinner, a progress bar and a skeleton.
//!
//! `.ic-spinner`, `.ic-spinner__ring` and `@keyframes ic-spin`. The spinner is the one place in the
//! library where a **stroke takes a gradient** — its arc runs blue into violet — which is why
//! [`crate::Decor::Arc`] exists as a shape of its own rather than being an icon.

use egui::{Rect, Vec2};

use crate::theme::Opacity as _;
use crate::{Fill, Rux};

/// `.ic-spinner { width: 56px; height: 56px }`.
pub const SPINNER_DIAMETER: f32 = 56.0;
/// `.ic-spinner__ring { inset: 8px }`.
const RING_INSET: f32 = 8.0;
/// The ring's own `viewBox` is `0 0 44 44` with `r="18"` and `stroke-width="3"`, so both scale from 44
/// onto whatever the ring box really is.
const VIEW_BOX: f32 = 44.0;
const VIEW_RADIUS: f32 = 18.0;
const VIEW_STROKE: f32 = 3.0;
/// `animation: ic-spin 1.4s linear infinite`.
const PERIOD: f32 = 1.4;

/// The neumorphic spinner: a pressed disc with a turning arc inside it.
pub struct Spinner {
    /// The whole control's diameter. `SPINNER_DIAMETER` unless a caller wants a smaller one.
    diameter: f32,
    /// Whether to draw the pressed disc behind the arc. `.ic-spinner` has one; a spinner inside a
    /// button does not.
    well: bool,
}

impl Default for Spinner {
    fn default() -> Self {
        Self::new()
    }
}

impl Spinner {
    pub fn new() -> Self {
        Self { diameter: SPINNER_DIAMETER, well: true }
    }

    pub fn diameter(mut self, diameter: f32) -> Self {
        self.diameter = diameter;
        self
    }

    /// Without the pressed disc behind it.
    pub fn bare(mut self) -> Self {
        self.well = false;
        self
    }

    pub fn measure(&self, _rux: &Rux<'_>) -> Vec2 {
        Vec2::splat(self.diameter)
    }

    pub fn show(self, rux: &mut Rux<'_>, rect: Rect) {
        let theme = rux.theme();
        let side = self.diameter.min(rect.width()).min(rect.height());
        let rect = Rect::from_center_size(rect.center(), Vec2::splat(side));
        let scale = side / SPINNER_DIAMETER;
        if self.well {
            rux.chrome.surface(
                rect,
                side / 2.0,
                theme.surface.s1,
                theme.elevation.pressed,
            );
        }

        let ring = rect.shrink(RING_INSET * scale);
        let radius = VIEW_RADIUS / VIEW_BOX * ring.width();
        let stroke = VIEW_STROKE / VIEW_BOX * ring.width();

        // `<circle stroke="rgba(30,37,48,0.08)"/>` — the track is the ink at an eighth, so it is the
        // right tone in both themes without a token of its own. It does not move, so it is decoration.
        rux.chrome.ring(ring.center(), radius, stroke, theme.ink.i900.at(0.08));

        // **The arc is a texture, not decoration**, and that is the difference between a page with
        // three spinners on it costing 822 ms of processor time a second and costing nothing. The
        // `Decor` list is what decides whether the layer is rasterised again, so a shape whose angle
        // changes every frame rasterises the whole layer every frame. Rasterised once and turned, it
        // is four vertices. See `icon::paint_arc`.
        //
        // The arc is a quarter turn: `M22 4 a18 18 0 0 1 18 18` starts at twelve o'clock and ends at
        // three.
        let turn = rux.ui.input(|input| input.time) as f32 / PERIOD * std::f32::consts::TAU;
        let gradient = theme.gradient.multistop;
        let painter = rux.ui.painter().clone();
        crate::icon::paint_arc(
            &painter,
            &mut rux.state.icons.borrow_mut(),
            ring.center(),
            radius,
            stroke,
            std::f32::consts::FRAC_PI_2,
            turn % std::f32::consts::TAU,
            gradient.from,
            gradient.to,
        );
        rux.ui.ctx().request_repaint();
    }
}

/// A bar that fills as something finishes: `--grad-progress` in a sunken track.
pub struct Progress {
    /// 0 to 1, or `None` for a bar that does not know how far along it is.
    fraction: Option<f32>,
    height: f32,
}

impl Progress {
    pub fn new(fraction: f32) -> Self {
        Self { fraction: Some(fraction.clamp(0.0, 1.0)), height: 6.0 }
    }

    /// A bar with no idea how far along it is, which sweeps instead of filling.
    pub fn indeterminate() -> Self {
        Self { fraction: None, height: 6.0 }
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    pub fn measure(&self, _rux: &Rux<'_>) -> Vec2 {
        Vec2::new(200.0, self.height)
    }

    pub fn show(self, rux: &mut Rux<'_>, rect: Rect) {
        let theme = rux.theme();
        let track = Rect::from_center_size(
            rect.center(),
            Vec2::new(rect.width(), self.height.min(rect.height())),
        );
        let radius = track.height() / 2.0;
        rux.chrome.surface(track, radius, theme.surface.sunken, theme.elevation.pressed_sm);
        match self.fraction {
            Some(fraction) => {
                // A bar at a fixed fraction does not move between frames, so it is decoration.
                let filled = Rect::from_min_size(
                    track.min,
                    Vec2::new(track.width() * fraction, track.height()),
                );
                if filled.width() > 0.5 {
                    rux.chrome.rect(filled, radius, Fill::gradient(theme.gradient.progress, track));
                }
            }
            None => {
                // A sweep a fifth of the track wide, going back and forth, which is what an
                // indeterminate bar is everywhere — **drawn as an `egui` mesh**, for the reason the
                // spinner's arc is a texture: a shape that moves every frame must not reach the
                // `Decor` list, or the whole layer is rasterised every frame.
                let time = rux.ui.input(|input| input.time) as f32;
                let span = track.width() * 0.2;
                let travel = (time * 0.6).sin() * 0.5 + 0.5;
                rux.ui.ctx().request_repaint();
                let band = Rect::from_min_size(
                    egui::pos2(track.left() + (track.width() - span) * travel, track.top()),
                    Vec2::new(span, track.height()),
                );
                let gradient = theme.gradient.progress;
                let mut mesh = egui::Mesh::default();
                for (at, colour) in [
                    (band.left_top(), gradient.from),
                    (band.right_top(), gradient.to),
                    (band.right_bottom(), gradient.to),
                    (band.left_bottom(), gradient.from),
                ] {
                    mesh.colored_vertex(at, colour);
                }
                mesh.add_triangle(0, 1, 2);
                mesh.add_triangle(0, 2, 3);
                rux.ui.painter().add(egui::Shape::mesh(mesh));
            }
        }
    }
}

/// A sunken block standing in for something that has not arrived yet, with a highlight sweeping over
/// it.
///
/// **The sweep is an `egui` mesh, not decoration.** It moves every frame, and a moving gradient in the
/// canvas would rasterise the whole layer sixty times a second — so the well is decoration and the
/// thing that moves is four vertices.
pub struct Skeleton {
    radius: Option<f32>,
}

impl Default for Skeleton {
    fn default() -> Self {
        Self::new()
    }
}

impl Skeleton {
    pub fn new() -> Self {
        Self { radius: None }
    }

    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = Some(radius);
        self
    }

    pub fn show(self, rux: &mut Rux<'_>, rect: Rect) {
        let theme = rux.theme();
        let radius = self.radius.unwrap_or(theme.radius.md);
        rux.chrome.surface(rect, radius, theme.surface.sunken, theme.elevation.pressed_sm);

        let time = rux.ui.input(|input| input.time) as f32;
        let travel = (time * 0.8).fract();
        let span = rect.width() * 0.35;
        let left = rect.left() - span + (rect.width() + span * 2.0) * travel;
        let band = Rect::from_min_size(egui::pos2(left, rect.top()), Vec2::new(span, rect.height()))
            .intersect(rect);
        if band.width() > 0.5 {
            let clear = theme.surface.s2.at(0.0);
            let lit = theme.surface.s2.at(0.55);
            let mut mesh = egui::Mesh::default();
            for (at, colour) in [
                (band.left_top(), clear),
                (band.center_top(), lit),
                (band.right_top(), clear),
                (band.left_bottom(), clear),
                (band.center_bottom(), lit),
                (band.right_bottom(), clear),
            ] {
                mesh.colored_vertex(at, colour);
            }
            mesh.add_triangle(0, 1, 4);
            mesh.add_triangle(0, 4, 3);
            mesh.add_triangle(1, 2, 5);
            mesh.add_triangle(1, 5, 4);
            rux.ui.painter().add(egui::Shape::mesh(mesh));
        }
        rux.ui.ctx().request_repaint();
    }
}

/// The `.ic-elapsed` chip: how long something has been going, in mono, under a tile.
pub fn elapsed_text(seconds: u64) -> String {
    match seconds {
        seconds if seconds < 60 => format!("{seconds}s"),
        seconds => format!("{}m {}s", seconds / 60, seconds % 60),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_spinner_is_the_stylesheet_s_size() {
        // `.ic-spinner { width: 56px }`, `.ic-spinner__ring { inset: 8px }`, `viewBox="0 0 44 44"`
        // with `r="18"` and `stroke-width="3"`.
        assert_eq!(SPINNER_DIAMETER, 56.0);
        assert_eq!(RING_INSET, 8.0);
        // The ring box is 40 across, so the drawn radius is 18/44 of that.
        let ring = SPINNER_DIAMETER - RING_INSET * 2.0;
        assert!((VIEW_RADIUS / VIEW_BOX * ring - 16.3636).abs() < 0.001);
        assert!((VIEW_STROKE / VIEW_BOX * ring - 2.7272).abs() < 0.001);
    }

    #[test]
    fn elapsed_reads_as_a_person_would_say_it() {
        assert_eq!(elapsed_text(7), "7s");
        assert_eq!(elapsed_text(95), "1m 35s");
    }
}
