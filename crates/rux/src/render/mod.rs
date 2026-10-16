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
pub mod glass;

pub use canvas::{Canvas, Canvases, MAX_SCALE};
pub use chrome::Chrome;
pub use decor::{bounds_of, Corners, Decor, Fill};
pub use glass::{backdrop_for, set_backdrop, Backdrop, Band, Plane};

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
    let chrome = Chrome::recording().with_glass(state.theme().glass);
    let result = {
        let mut rux = Rux { ui, state, chrome: &chrome };
        add(&mut rux)
    };
    let items = chrome.take();
    if items.is_empty() {
        return result;
    }
    let ctx = painter.ctx().clone();
    let scale = composited_scale(&ctx, painter.layer_id());
    let has_glass = items.iter().any(|item| matches!(item, Decor::Glass { .. }));
    let backdrop = match has_glass {
        true => glass::backdrop_for(&ctx, painter.layer_id()),
        false => None,
    };
    // **One texture when there is nothing to bend**, which is every layer in a theme that is not glass
    // and every layer with no picture behind it: the canvas draws nothing for a glass item.
    let Some(backdrop) = backdrop else {
        let drawn = state.canvases.borrow_mut().texture_for(&ctx, id, area, scale, &items);
        if let Some((texture, rect)) = drawn {
            painter.set(slot, image(texture, rect));
        }
        return result;
    };
    painter.set(
        slot,
        egui::Shape::Vec(glazed(&ctx, &painter, state, id, area, scale, &items, backdrop)),
    );
    result
}

fn image(texture: egui::TextureId, rect: Rect) -> egui::Shape {
    let uv = Rect::from_min_max(egui::Pos2::ZERO, egui::Pos2::new(1.0, 1.0));
    egui::Shape::image(texture, rect, uv, egui::Color32::WHITE)
}

/// A layer's decoration with glass in it: the list cut at each piece of glass, each run rasterised as
/// a texture of its own and each piece of glass drawn as the bent picture between them (`task-2281`).
///
/// That order is what puts a carved well over the card it is carved into and under the chosen pill
/// raised out of it. A rounded clip still open when the list is cut is closed at the end of one run and
/// opened again at the start of the next, so each run is a complete drawing.
#[allow(clippy::too_many_arguments)]
fn glazed(
    ctx: &egui::Context,
    painter: &egui::Painter,
    state: &RuxState,
    id: egui::Id,
    area: Rect,
    scale: f32,
    items: &[Decor],
    backdrop: std::sync::Arc<glass::Backdrop>,
) -> Vec<egui::Shape> {
    let material = state.theme().glass.unwrap_or(crate::theme::Glass::DESIGN);
    let to_global = ctx.layer_transform_to_global(painter.layer_id()).unwrap_or_default();
    let pixels_per_point = ctx.pixels_per_point();
    let mut shapes = Vec::new();
    let mut open: Vec<Decor> = Vec::new();
    let mut run: Vec<Decor> = Vec::new();
    let mut runs = 0usize;
    let mut panes = 0usize;
    let mut flush = |run: &mut Vec<Decor>, open: &[Decor], shapes: &mut Vec<egui::Shape>| {
        let draws = run.iter().any(|item| !matches!(item, Decor::Clip { .. } | Decor::Unclip));
        if draws {
            run.extend(open.iter().map(|_| Decor::Unclip));
            let run_id = match runs {
                0 => id,
                n => id.with(("rux-glass-run", n)),
            };
            let drawn = state.canvases.borrow_mut().texture_for(ctx, run_id, area, scale, run);
            if let Some((texture, rect)) = drawn {
                shapes.push(image(texture, rect));
            }
        }
        runs += 1;
        run.clear();
        run.extend(open.iter().copied());
    };
    for &item in items {
        match item {
            Decor::Glass { rect, corners, bevel, strength } => {
                flush(&mut run, &open, &mut shapes);
                let built = glass::Built {
                    glass: material,
                    pane: glass::Pane { rect, corners, bevel, strength },
                    backdrop: backdrop.clone(),
                    to_global,
                    pixels_per_point,
                };
                let pane_id = id.with(("rux-glass", panes));
                panes += 1;
                shapes.extend(state.canvases.borrow_mut().glass_for(pane_id, built));
            }
            Decor::Clip { .. } => {
                open.push(item);
                run.push(item);
            }
            Decor::Unclip => {
                open.pop();
                run.push(item);
            }
            other => run.push(other),
        }
    }
    flush(&mut run, &open, &mut shapes);
    shapes
}
