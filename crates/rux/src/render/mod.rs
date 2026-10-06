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

/// How many pixels one point of `layer` is composited at: the display's own density times the scale of the
/// transform the layer is drawn through, in quarter steps.
///
/// **A layer can be scaled.** Unluminous zooms a modal and its canvas nodes by drawing them into a layer
/// with a `TSTransform`, and `epaint` applies that to the finished shapes. A decoration or an icon
/// rasterised at the display's density alone is then a bitmap magnified by the zoom, which reads as a
/// soft edge on every well and every mark. So the texture is made for the size it is seen at. Quarter
/// steps rather than the exact number, because a zoom that glides would otherwise make a new texture on
/// every frame of the glide; rounded up, so the transform only ever shrinks a texture.
pub fn composited_scale(ctx: &egui::Context, layer: egui::LayerId) -> f32 {
    let zoom = ctx.layer_transform_to_global(layer).map(|to| to.scaling).unwrap_or(1.0);
    let zoom = match zoom.is_finite() && zoom > 0.0 {
        true => ((zoom * 4.0).ceil() / 4.0).clamp(0.25, 4.0),
        false => 1.0,
    };
    ctx.pixels_per_point() * zoom
}

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
        let scale = composited_scale(painter.ctx(), painter.layer_id());
        let drawn = state.canvases.borrow_mut().texture_for(painter.ctx(), id, area, scale, &items);
        if let Some((texture, rect)) = drawn {
            let uv = Rect::from_min_max(egui::Pos2::ZERO, egui::Pos2::new(1.0, 1.0));
            painter.set(slot, egui::Shape::image(texture, rect, uv, egui::Color32::WHITE));
        }
    }
    result
}
