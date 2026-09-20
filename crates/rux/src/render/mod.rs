//! Turning depth into pixels: what a component records, and where it ends up.
//!
//! A layer's decoration is **one texture painted behind that layer's widgets**, so the shape has to be
//! reserved in the painter's list before any widget is added and filled in afterwards, once the
//! drawing is over and the list is complete. That is what [`layer`] does, and it is why the library
//! has a notion of a layer at all.
//!
//! **Every popup and every modal is its own layer.** `egui` draws them on foreground layers the
//! parent's painter cannot reach, and `Painter::set` writes to the layer its own painter belongs to —
//! so a dropdown menu with its parent's painter would fill in a slot on the wrong layer and appear
//! behind the page. Opening an `egui::Area` and calling [`layer`] inside it is the ordinary path here
//! rather than a special case.

mod canvas;
mod chrome;
mod decor;

pub use canvas::{Canvas, Canvases, MAX_SCALE};
pub use chrome::Chrome;
pub use decor::{bounds_of, Corners, Decor, Fill};

use egui::Rect;

use crate::{Rux, RuxState};

/// Draw into `ui` with a decoration canvas behind it.
///
/// Reserves the slot, makes a recording [`Chrome`], runs `add`, rasterises what it recorded and fills
/// the slot in. `area` is what the canvas may cover — the decoration's own bounding box is
/// intersected with it, so a component that draws outside its layer is cut rather than growing the
/// texture.
pub fn layer<R>(
    ui: &mut egui::Ui,
    state: &RuxState,
    id: egui::Id,
    area: Rect,
    add: impl FnOnce(&mut Rux<'_>) -> R,
) -> R {
    // A `Painter` is an owned handle, so it can be kept across the drawing and used again afterwards.
    let painter = ui.painter().clone();
    let slot = painter.add(egui::Shape::Noop);
    let chrome = Chrome::recording();
    let result = {
        let mut rux = Rux { ui, state, chrome: &chrome };
        add(&mut rux)
    };
    let items = chrome.take();
    if !items.is_empty() {
        let drawn = state.canvases.borrow_mut().texture_for(painter.ctx(), id, area, &items);
        if let Some((texture, rect)) = drawn {
            let uv = Rect::from_min_max(egui::Pos2::ZERO, egui::Pos2::new(1.0, 1.0));
            painter.set(slot, egui::Shape::image(texture, rect, uv, egui::Color32::WHITE));
        }
    }
    result
}
