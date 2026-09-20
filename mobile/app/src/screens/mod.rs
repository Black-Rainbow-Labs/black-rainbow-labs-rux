//! The three screens, and the one thing they share: a column that scrolls.

mod controls;
mod create;
mod gallery;

pub use controls::Controls;
pub use create::Create;
pub use gallery::Gallery;

use egui::{Rect, Vec2};
use rux::components::Scrollbar;
use rux::layout::Pen;
use rux::Rux;

/// A column taller than the screen it is on.
///
/// The same idiom `examples/pages/image_creator.rs` uses for its panels — a clip, an offset and `rux`'s
/// own `Scrollbar` — with one thing added for a phone: **a drag anywhere the content did not claim
/// scrolls the column.**
///
/// The drag rectangle is allocated *before* the content, so every control drawn afterwards wins the
/// point where the two overlap; egui hands an overlapping point to whichever widget asked last. The
/// consequence is worth knowing: a finger that lands on a select and then drags does not scroll, it
/// operates the select. On a phone that is the wrong behaviour and fixing it properly means egui's own
/// `ScrollArea`, which tracks whether a drag began on a widget and has momentum. It is not used here
/// because the decoration `rux` draws has to be recorded into this layer's chrome, and a `ScrollArea`
/// would move the content underneath the one texture the layer has — so the whole column would be
/// rasterised again on every frame of every scroll.
pub struct Scroll {
    /// The whole visible area, which is also where the bar goes.
    area: Rect,
    /// The area inside the padding, which is where the content goes.
    body: Rect,
    /// How tall the content is.
    content: f32,
    id: egui::Id,
}

impl Scroll {
    pub fn new(rux: &Rux<'_>, area: Rect, body: Rect, content: f32, name: &str) -> Self {
        Self { area, body, content, id: rux.ui.id().with(name) }
    }

    /// Run `draw` with a pen over the scrolled content, and draw the bar afterwards.
    pub fn show(self, rux: &mut Rux<'_>, draw: impl FnOnce(&mut Rux<'_>, &mut Pen)) {
        let reach = (self.content - self.area.height()).max(0.0);
        let bar = Scrollbar::new(self.area, self.content);
        let mut offset: f32 = rux.ui.memory(|memory| memory.data.get_temp(self.id).unwrap_or(0.0));

        // The bar first, so a drag on the bar itself is the bar's rather than the column's.
        offset = bar.grab(rux, offset);

        // Then the column, for a finger. `Sense::drag` only, so a tap that hits nothing does nothing.
        let sheet = rux.ui.interact(self.area, self.id.with("drag"), egui::Sense::drag());
        if sheet.dragged() {
            offset -= sheet.drag_delta().y;
        }

        // Then the wheel, which is what a desktop preview and a trackpad send.
        let inside = rux
            .ui
            .ctx()
            .pointer_latest_pos()
            .is_some_and(|at| self.area.contains(at));
        if inside {
            let wheel = rux.ui.ctx().input(|input| input.smooth_scroll_delta.y);
            offset -= wheel;
        }
        offset = offset.clamp(0.0, reach);

        let was = rux.ui.clip_rect();
        rux.ui.set_clip_rect(self.area.intersect(was));
        rux.chrome.clip(self.area, 0.0);
        let mut pen = Pen::new(Rect::from_min_size(
            egui::pos2(self.body.left(), self.body.top() - offset),
            Vec2::new(self.body.width(), self.content),
        ));
        draw(rux, &mut pen);
        rux.chrome.unclip();
        rux.ui.set_clip_rect(was);

        bar.paint(rux, offset);
        rux.ui.memory_mut(|memory| memory.data.insert_temp(self.id, offset));
    }
}

/// The kicker over a section: 10.5px mono, tracked out, upper case.
///
/// `.ic-sub__title`'s own style, used on its own where a whole `SubGroup` would be too much furniture.
pub fn kicker(rux: &mut Rux<'_>, rect: Rect, words: &str) {
    let style = rux::Style::mono(10.5).medium().tracking(0.12).upper();
    let ink = rux.theme().ink.i500;
    let galley = rux.text(style, words, ink);
    rux::text::draw_left_centre(rux.ui.painter(), rect.left_center(), galley, ink);
}

/// How tall a kicker's own strip is.
pub const KICKER_HEIGHT: f32 = 18.0;
