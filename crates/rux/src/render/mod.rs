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
pub mod cost;
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

/// The scale to rasterise one run of a glass layer at: `scale`, or less for a run of nothing but outer
/// shadows (`task-2281`).
///
/// A Gaussian has no detail finer than its standard deviation, so two pixels to a deviation draw it as
/// well as twenty. A glass surface's fill is translucent, so its outer shadows are not cut at its outline
/// (`Chrome::surface` cuts only under an opaque fill), and what of them lies under the surface is hidden by
/// the glass's own cover: no edge in such a run is drawn at the canvas's resolution. On the Settings
/// window's first opening those runs were six of its canvases and most of its pixels.
///
/// Nothing else is drawn below the display's resolution. A surface's tint and its inner glow are cut at
/// its outline, and 0.92.0 drew them at half a pixel a point: the outline smeared a pixel or two outward
/// and read as a light ring round every chip over a bright picture, and edges looked pixelated.
fn glass_canvas_scale(run: &[Decor], scale: f32) -> f32 {
    let softest = run.iter().try_fold(f32::INFINITY, |least, item| match *item {
        Decor::Shadow { inset: false, deviation, .. } => Some(least.min(deviation)),
        _ => None,
    });
    match softest.filter(|deviation| deviation.is_finite() && *deviation > 0.0) {
        Some(deviation) => scale.min((2.0 / deviation).max(SHADOW_LEAST_SCALE)),
        None => scale,
    }
}

/// The least scale a run of outer shadows is rasterised at under glass: a pixel every four points.
pub const SHADOW_LEAST_SCALE: f32 = 0.25;

/// The rectangle the clips that keep their insides leave open, or `None` when there are none.
fn inside_the_clips(open: &[Decor]) -> Option<Rect> {
    open.iter().fold(None, |bound: Option<Rect>, item| match *item {
        Decor::Clip { rect, outside: false, .. } => {
            Some(bound.map_or(rect, |bound| bound.intersect(rect)))
        }
        _ => bound,
    })
}

#[cfg(test)]
mod glass_canvas_tests {
    use super::*;
    use egui::{pos2, vec2, Color32};

    fn shadow(deviation: f32) -> Decor {
        Decor::Shadow {
            rect: Rect::from_min_size(pos2(100.0, 100.0), vec2(300.0, 100.0)),
            radius: 22.0,
            deviation,
            colour: Color32::BLACK,
            inset: false,
        }
    }

    #[test]
    fn a_run_of_outer_shadows_is_rasterised_at_two_pixels_a_deviation() {
        assert_eq!(glass_canvas_scale(&[shadow(18.0)], 1.5), SHADOW_LEAST_SCALE);
        assert_eq!(glass_canvas_scale(&[shadow(2.0)], 1.5), 1.0);
    }

    #[test]
    fn anything_with_an_edge_is_rasterised_at_the_displays_own_scale() {
        // The white ring of task-2281's third review: a tint and its glow cut at the outline, drawn at half
        // a pixel a point, smeared outward.
        let rect = Rect::from_min_size(pos2(100.0, 100.0), vec2(300.0, 100.0));
        let fill = Decor::Rect { rect, corners: Corners::from(22.0), fill: Fill::from(Color32::GRAY) };
        let glow = Decor::Shadow { rect, radius: 22.0, deviation: 9.0, colour: Color32::WHITE, inset: true };
        let clip = Decor::Clip { rect, corners: Corners::from(22.0), outside: false, bound: rect };
        assert_eq!(glass_canvas_scale(&[fill, clip, glow, Decor::Unclip], 1.5), 1.5);
        assert_eq!(glass_canvas_scale(&[clip, shadow(18.0), Decor::Unclip], 1.5), 1.5);
        let wide = Rect::from_min_size(pos2(0.0, 0.0), vec2(1800.0, 1200.0));
        let big = Decor::Rect { rect: wide, corners: Corners::from(22.0), fill: Fill::from(Color32::GRAY) };
        assert_eq!(glass_canvas_scale(&[big], 1.5), 1.5, "a large surface keeps its resolution too");
    }
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
            let run_scale = glass_canvas_scale(run, scale);
            let drawn = state.canvases.borrow_mut().texture_for(ctx, run_id, area, run_scale, run);
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
                let glass = state.canvases.borrow_mut().glass_for(pane_id, built);
                // Cut to the clips open round it, which reach the rasterised runs but not an `egui` mesh.
                match inside_the_clips(&open) {
                    Some(bound) if !bound.contains_rect(rect) => {
                        shapes.extend(glass.into_iter().map(|shape| match shape {
                            egui::Shape::Mesh(mesh) => {
                                egui::Shape::mesh(glass::clip_to_rect(&mesh, bound))
                            }
                            other => other,
                        }));
                    }
                    _ => shapes.extend(glass),
                }
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
