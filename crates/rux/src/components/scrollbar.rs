//! The thin bar down the right hand edge of anything that scrolls.
//!
//! `.ic-panel__scroll::-webkit-scrollbar { width: 6px }` with a thumb in `var(--surface-sunken)` at
//! `border-radius: 3px` and a track that is not drawn at all.
//!
//! **It is two calls rather than one**, and that is the only place a component here is. `grab` takes
//! the pointer at the same moment the scrolling area takes its own, because `egui` hands a drag to the
//! last widget that asked for the point; `paint` draws at the end, once the wheel and any keyboard
//! movement have had their say, because a thumb drawn from the offset the frame opened with is a frame
//! behind the content.

use egui::{Rect, Sense, Vec2};

use super::interact;
use crate::Rux;

/// `width: 6px`, and six points in from the right so it clears a panel's own padding.
const WIDTH: f32 = 6.0;
const INSET: f32 = 6.0;

/// A scrollbar for an area that is taller than the room it has.
pub struct Scrollbar {
    /// The whole area being scrolled.
    area: Rect,
    /// How tall the content is.
    content: f32,
}

impl Scrollbar {
    pub fn new(area: Rect, content: f32) -> Self {
        Self { area, content }
    }

    /// Whether there is anything to scroll. A caller draws no bar at all when there is not, which is
    /// what the reference does — the track is never painted.
    pub fn needed(&self) -> bool {
        self.content > self.area.height() + 0.5
    }

    fn thumb(&self, offset: f32) -> Rect {
        let travel = (self.content - self.area.height()).max(1.0);
        let fraction = (self.area.height() / self.content).clamp(0.08, 1.0);
        let height = (self.area.height() * fraction).max(24.0);
        let top = self.area.top()
            + (self.area.height() - height) * (offset / travel).clamp(0.0, 1.0);
        Rect::from_min_size(
            egui::pos2(self.area.right() - INSET - WIDTH, top),
            Vec2::new(WIDTH, height),
        )
    }

    /// Take the drag, and answer the offset it moved to.
    ///
    /// Called **before** the content is drawn, for the reason in this module's own comment.
    pub fn grab(&self, rux: &mut Rux<'_>, offset: f32) -> f32 {
        if !self.needed() {
            return 0.0;
        }
        let thumb = self.thumb(offset);
        // The lane is as wide as the pointer needs, not as wide as the thumb is drawn. A six point
        // strip is a mouse target; a finger wants the minimum target, and the lane is at the panel's
        // own edge so there is no neighbouring control for the extra width to take a press from.
        let grip = WIDTH.max(rux.screen().min_target());
        let lane = Rect::from_min_max(
            egui::pos2(thumb.right() - grip, self.area.top()),
            egui::pos2(thumb.right(), self.area.bottom()),
        );
        let response = interact(rux, lane, "rux-scrollbar", Sense::click_and_drag());
        if response.dragged() {
            let travel = (self.content - self.area.height()).max(1.0);
            let reach = (self.area.height() - thumb.height()).max(1.0);
            return (offset + response.drag_delta().y / reach * travel).clamp(0.0, travel);
        }
        offset.clamp(0.0, (self.content - self.area.height()).max(0.0))
    }

    /// Draw the thumb, at the end of the frame.
    pub fn paint(&self, rux: &mut Rux<'_>, offset: f32) {
        if !self.needed() {
            return;
        }
        let theme = rux.theme();
        let thumb = self.thumb(offset);
        let hovered = rux.screen().pointer.hovers()
            && rux
                .ui
                .ctx()
                .pointer_latest_pos()
                .is_some_and(|at| thumb.expand2(Vec2::new(8.0, 0.0)).contains(at));
        // Quiet it is the sunken surface; under the pointer it is the ink, which is the one place the
        // reference's own `::-webkit-scrollbar-thumb` colour is not quite enough to see.
        let colour = match hovered {
            true => theme.ink.i300,
            false => theme.surface.sunken,
        };
        rux.chrome.rect(thumb, WIDTH / 2.0, colour);
    }
}
