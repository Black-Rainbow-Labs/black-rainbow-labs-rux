//! Glass: the picture behind a layer, bent through a rounded rectangle (`task-2281`).
//!
//! The Claude Design canvas "Unluminous", page "Agent-Chat 2", builds glass in a browser with an SVG
//! backdrop filter on every element: a displacement map made from the element's rounded rectangle, and
//! three `feDisplacementMap` passes at 1.0, 0.92 and 0.84 of the strength for red, green and blue. This
//! is the same arithmetic drawn as an `egui` mesh.
//!
//! **Why a mesh.** A displacement changes *where the picture is sampled*, and a mesh already says that:
//! every vertex carries a texture coordinate and the graphics card interpolates between them. So each
//! piece of glass is a small mesh whose vertices lie on rings parallel to its edge, and whose texture
//! coordinates are where the bent sample lands in the picture. The picture's own textures are drawn,
//! nothing new is uploaded, and the mesh goes through `egui`'s own pipeline, so a window and a test
//! that renders offscreen draw it the same way. `tasks/task-2281-glass-theme-tdd.md` in Unluminous
//! weighs this against a shader and against bending on the processor.
//!
//! **Why it is accurate.** Along a straight edge the normal is constant and the lens term is linear in
//! the position, so the offset changes linearly and needs no vertex between the corners. Round a corner
//! the normal turns, so each corner is cut into segments. Across the rim the offset follows `t^2.2`,
//! which [`RINGS`] rings follow to within a fraction of a point. Inside the innermost ring only the lens
//! term is left, and it is linear, so the fan to the centre is exact.
//!
//! **The colour split is three passes that add.** `egui` blends premultiplied, `src + dst × (1 − src.a)`,
//! so a vertex colour whose alpha is zero adds. The glass first covers what is behind it with a dark
//! rounded rectangle at the picture's opacity, and then adds red, green and blue, each sampled at its
//! own strength. The order of the three does not matter, which is what lets them be cut across the
//! picture's strips independently.
//!
//! **What is bent is a picture the application hands over, not the frame.** `egui` draws every layer
//! straight into one render pass and cannot read back what is already there, so glass cannot sample
//! "whatever is under it". An application sets a [`Backdrop`] once a frame with [`set_backdrop`]: the
//! picture its panes are drawn on, and, for whatever floats above them, a photograph of the window.
//! With no backdrop, glass is its tint and its edges and nothing is bent.

use std::sync::Arc;

use egui::epaint::Vertex;
use egui::{Color32, Mesh, Pos2, Rect, Shape, TextureId, Vec2};

use super::decor::Corners;
use crate::theme::Glass;

/// How many rings cross the rim. Six follow `t^2.2` to within a point at the strongest bend the design
/// uses, which is the modal's, and a point of bend is not something anybody can see in a refraction.
pub const RINGS: usize = 6;

/// One texture of a [`Backdrop`]: the rows of the picture it is drawn for and the rows it holds.
///
/// A picture is often uploaded in strips, so a graphics driver never has to keep one large upload's
/// staging memory, and a strip holds a row more above and below than it is drawn for, so a linear
/// filter at a seam reads its neighbour. Both ranges are fractions of the whole picture's height.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Band {
    pub texture: TextureId,
    /// The rows this band is drawn for, `0.0..1.0` being the whole picture.
    pub drawn: (f32, f32),
    /// The rows its texture holds.
    pub held: (f32, f32),
}

/// The picture glass bends: its textures, where it is drawn and which part of it.
#[derive(Debug, Clone, PartialEq)]
pub struct Backdrop {
    /// The picture, top to bottom.
    pub bands: Vec<Band>,
    /// Where the part [`Backdrop::uv`] of the picture is drawn, in the window's points.
    pub rect: Rect,
    /// Which part of the picture is drawn over [`Backdrop::rect`], as fractions of it.
    pub uv: Rect,
    /// How much of what is behind it the picture covers where it is drawn. Glass covers the same.
    pub opacity: f32,
}

impl Backdrop {
    /// A picture held as one texture.
    pub fn single(texture: TextureId, rect: Rect, uv: Rect, opacity: f32) -> Self {
        let whole = Band { texture, drawn: (0.0, 1.0), held: (0.0, 1.0) };
        Self { bands: vec![whole], rect, uv, opacity }
    }

    /// Where in the picture a point of the window is, as fractions of the picture, kept inside it.
    ///
    /// A sample that would land off the picture is held at its edge, which is what a displacement filter
    /// does in a browser too.
    pub fn picture_at(&self, at: Pos2) -> Pos2 {
        let size = self.rect.size();
        let along = Vec2::new(
            (at.x - self.rect.min.x) / size.x.max(f32::EPSILON),
            (at.y - self.rect.min.y) / size.y.max(f32::EPSILON),
        );
        let picture = self.uv.min + along * self.uv.size();
        Pos2::new(picture.x.clamp(0.0, 1.0), picture.y.clamp(0.0, 1.0))
    }
}

/// Which backdrop a layer bends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Plane {
    /// The page: every layer drawn in `egui`'s background and middle orders.
    Page,
    /// Whatever floats above the page: a modal, a popup, a menu. Falls back to [`Plane::Page`]'s
    /// backdrop when the application set none.
    Above,
}

impl Plane {
    /// The plane a layer of this order is drawn on.
    pub fn of(order: egui::Order) -> Self {
        match order {
            egui::Order::Background | egui::Order::Middle => Plane::Page,
            _ => Plane::Above,
        }
    }
}

fn slot(plane: Plane) -> egui::Id {
    egui::Id::new(("rux-glass-backdrop", plane))
}

/// Say what glass on `plane` bends, for the rest of this window's life or until it is set again.
///
/// Kept in `egui`'s own memory, so every `RuxState` in the window sees it with nothing passed through.
/// `None` takes it away: glass is then its tint and edges.
pub fn set_backdrop(ctx: &egui::Context, plane: Plane, backdrop: Option<Backdrop>) {
    ctx.data_mut(|data| match backdrop {
        Some(backdrop) => {
            let same =
                data.get_temp::<Arc<Backdrop>>(slot(plane)).is_some_and(|kept| *kept == backdrop);
            if !same {
                data.insert_temp(slot(plane), Arc::new(backdrop));
            }
        }
        None => data.remove::<Arc<Backdrop>>(slot(plane)),
    });
}

/// The backdrop glass in `layer` bends, if there is one.
pub fn backdrop_for(ctx: &egui::Context, layer: egui::LayerId) -> Option<Arc<Backdrop>> {
    let read = |plane: Plane| ctx.data(|data| data.get_temp::<Arc<Backdrop>>(slot(plane)));
    match Plane::of(layer.order) {
        Plane::Page => read(Plane::Page),
        Plane::Above => read(Plane::Above).or_else(|| read(Plane::Page)),
    }
}

/// One piece of glass, as [`super::Decor::Glass`] records it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pane {
    pub rect: Rect,
    pub corners: Corners,
    pub bevel: f32,
    pub strength: f32,
}

/// A vertex of the glass before it is turned into three passes: where it is in the layer, and how far
/// the picture is bent there, in the layer's points, at full strength.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sample {
    pub at: Pos2,
    pub offset: Vec2,
    /// Whether this vertex is on the feathered edge outside the shape, where the glass fades to nothing.
    pub feather: bool,
}

/// How far the picture is bent at a point of a piece of glass: the design's displacement map.
///
/// `depth` is the signed distance to the edge, negative inside, and `normal` the outward direction of
/// the edge there. The rim pulls along the normal by `t^2.2`, where `t` runs from nought at `bevel`
/// points inside to one at the edge; the lens pulls away from the centre in proportion to the distance
/// from it, fading in across the rim. The result is in points at full strength: the map's `0..255`
/// becomes `−0.5..0.5`, which is why the strength is halved.
///
/// @param glass - the material
/// @param pane - the piece of glass
/// @param at - the point, in the layer's points
/// @param normal - the outward direction of the edge nearest the point, or zero at the centre
/// @param depth - the signed distance to the edge, negative inside
pub fn offset_at(glass: &Glass, pane: &Pane, at: Pos2, normal: Vec2, depth: f32) -> Vec2 {
    let bevel = pane.bevel.max(0.5);
    let t = (1.0 + depth / bevel).clamp(0.0, 1.0);
    let rim = t.powf(2.2);
    let inner = (-depth / bevel).clamp(0.0, 1.0);
    let centre = pane.rect.center();
    let half = pane.rect.size() / 2.0;
    let across = (at.x - centre.x) / half.x.max(f32::EPSILON);
    let down = (at.y - centre.y) / half.y.max(f32::EPSILON);
    let squash = (pane.rect.height() / pane.rect.width().max(f32::EPSILON) * 2.2).min(1.0);
    let lens = Vec2::new(across * glass.lens * inner, down * glass.lens * inner * squash);
    (-normal * rim + lens) * pane.strength * 0.5
}

/// The corners of a rectangle each clamped to half its shorter side, in the order the rings walk them.
fn radii(rect: Rect, corners: Corners) -> [f32; 4] {
    let most = (rect.width().min(rect.height()) / 2.0).max(0.0);
    [corners.nw, corners.ne, corners.se, corners.sw].map(|radius| radius.clamp(0.0, most))
}

/// How many segments a corner of this radius is cut into.
fn segments(radius: f32) -> usize {
    ((radius / 4.0).ceil() as usize).clamp(2, 8)
}

/// The vertices of one ring: the rounded rectangle moved `inset` points inwards (outwards when
/// negative), walked clockwise from the left end of the top left corner.
fn ring(pane: &Pane, glass: &Glass, inset: f32, cuts: [usize; 4], feather: bool) -> Vec<Sample> {
    let radii = radii(pane.rect, pane.corners);
    let shape = pane.rect.shrink(inset);
    // A corner's centre stays where it was while the corner is still round, and moves in with the
    // sides once the ring is deeper than the corner's radius.
    let corner = |index: usize| {
        let radius = (radii[index] - inset).max(0.0);
        let centre = match index {
            0 => Pos2::new(shape.min.x + radius, shape.min.y + radius),
            1 => Pos2::new(shape.max.x - radius, shape.min.y + radius),
            2 => Pos2::new(shape.max.x - radius, shape.max.y - radius),
            _ => Pos2::new(shape.min.x + radius, shape.max.y - radius),
        };
        (centre, radius)
    };
    let mut samples = Vec::with_capacity(cuts.iter().map(|c| c + 1).sum());
    for (index, &cut) in cuts.iter().enumerate() {
        let (centre, radius) = corner(index);
        // Clockwise with y down: the top left corner turns from pointing left to pointing up.
        let start = std::f32::consts::PI * (1.0 + 0.5 * index as f32);
        for step in 0..=cut {
            let angle = start + std::f32::consts::FRAC_PI_2 * step as f32 / cut as f32;
            let normal = Vec2::new(angle.cos(), angle.sin());
            let at = centre + normal * radius;
            let offset = offset_at(glass, pane, at, normal, -inset);
            samples.push(Sample { at, offset, feather });
        }
    }
    samples
}

/// The glass as rings and a centre, and the triangles that join them.
///
/// `feather` is how far outside the edge the faded ring is, in the layer's points: one physical pixel,
/// which is the feather `egui` gives its own rounded rectangles.
pub fn geometry(glass: &Glass, pane: &Pane, feather: f32) -> (Vec<Sample>, Vec<u32>) {
    let half = pane.rect.width().min(pane.rect.height()) / 2.0;
    let radii = radii(pane.rect, pane.corners);
    let cuts = radii.map(segments);
    let deepest = pane.bevel.min(half - 0.25).max(0.0);
    let mut samples = Vec::new();
    let mut rings = Vec::new();
    // The faded ring outside the edge, then the edge, then deeper rings across the rim.
    rings.push(samples.len());
    samples.extend(ring(pane, glass, -feather, cuts, true));
    for step in 0..=RINGS {
        rings.push(samples.len());
        let inset = deepest * step as f32 / RINGS as f32;
        samples.extend(ring(pane, glass, inset, cuts, false));
    }
    let around = samples.len() / rings.len();
    let centre = samples.len() as u32;
    samples.push(Sample { at: pane.rect.center(), offset: Vec2::ZERO, feather: false });

    let mut indices = Vec::with_capacity(rings.len() * around * 6 + around * 3);
    for pair in rings.windows(2) {
        let (outer, inner) = (pair[0] as u32, pair[1] as u32);
        for i in 0..around as u32 {
            let j = (i + 1) % around as u32;
            indices.extend([outer + i, outer + j, inner + j, outer + i, inner + j, inner + i]);
        }
    }
    let last = *rings.last().unwrap_or(&0) as u32;
    for i in 0..around as u32 {
        let j = (i + 1) % around as u32;
        indices.extend([last + i, last + j, centre]);
    }
    (samples, indices)
}

/// The shapes that draw one piece of glass: the cover, then red, green and blue bent through it, each
/// cut at the backdrop's strips.
///
/// `to_global` is the layer's transform, so a layer drawn zoomed samples the picture where it really is
/// on the screen. `pixels_per_point` sets the feather.
///
/// @param glass - the material
/// @param pane - the piece of glass, in the layer's points
/// @param backdrop - the picture it bends
/// @param to_global - from the layer's points to the window's
/// @param pixels_per_point - the display's density
pub fn shapes(
    glass: &Glass,
    pane: &Pane,
    backdrop: &Backdrop,
    to_global: egui::emath::TSTransform,
    pixels_per_point: f32,
) -> Vec<Shape> {
    if pane.rect.width() < 1.0 || pane.rect.height() < 1.0 || backdrop.bands.is_empty() {
        return Vec::new();
    }
    let scale = (pixels_per_point * to_global.scaling).max(0.1);
    let (samples, indices) = geometry(glass, pane, 1.0 / scale);
    let opacity = (backdrop.opacity.clamp(0.0, 1.0) * 255.0).round() as u8;
    let mut shapes = Vec::with_capacity(1 + backdrop.bands.len());
    shapes.push(cover(pane, opacity));

    // All three passes in one mesh, in picture coordinates, before they are cut at the strips.
    let mut whole = Mesh::default();
    for (channel, &share) in glass.chroma.iter().enumerate() {
        let first = whole.vertices.len() as u32;
        for sample in &samples {
            let bent = sample.at + sample.offset * share;
            let uv = backdrop.picture_at(to_global * bent);
            let colour = match sample.feather {
                true => Color32::TRANSPARENT,
                false => {
                    let mut rgba = [0u8; 4];
                    rgba[channel] = opacity;
                    Color32::from_rgba_premultiplied(rgba[0], rgba[1], rgba[2], rgba[3])
                }
            };
            whole.vertices.push(Vertex { pos: sample.at, uv, color: colour });
        }
        whole.indices.extend(indices.iter().map(|index| index + first));
    }
    for band in &backdrop.bands {
        let mut piece = cut(&whole, band.drawn.0, band.drawn.1);
        if piece.indices.is_empty() {
            continue;
        }
        let span = (band.held.1 - band.held.0).max(f32::EPSILON);
        for vertex in &mut piece.vertices {
            vertex.uv.y = (vertex.uv.y - band.held.0) / span;
        }
        piece.texture_id = band.texture;
        shapes.push(Shape::mesh(piece));
    }
    shapes
}

/// The dark rounded rectangle that covers what is behind the glass, by the picture's own opacity, so
/// the three passes that follow can add.
fn cover(pane: &Pane, opacity: u8) -> Shape {
    let [nw, ne, se, sw] =
        radii(pane.rect, pane.corners).map(|r| r.round().clamp(0.0, 255.0) as u8);
    Shape::rect_filled(
        pane.rect,
        egui::CornerRadius { nw, ne, se, sw },
        Color32::from_rgba_premultiplied(0, 0, 0, opacity),
    )
}

/// The triangles of `mesh` whose texture rows lie between `top` and `bottom`, cut where they cross.
///
/// Each triangle is clipped against the two lines and what is left, up to five corners, is fanned back
/// into triangles. Position, texture coordinate and colour all change in a straight line across a
/// triangle, so where an edge meets a line is found by the same fraction for all three.
///
/// @param mesh - the whole piece, in picture coordinates
/// @param top - the band's top, as a fraction of the picture's height
/// @param bottom - its bottom
pub fn cut(mesh: &Mesh, top: f32, bottom: f32) -> Mesh {
    let mut out = Mesh::default();
    for triangle in mesh.indices.chunks_exact(3) {
        let corners = [0, 1, 2].map(|i| mesh.vertices[triangle[i] as usize]);
        let inside = corners.iter().all(|v| v.uv.y >= top && v.uv.y <= bottom);
        let outside =
            corners.iter().all(|v| v.uv.y < top) || corners.iter().all(|v| v.uv.y > bottom);
        if outside {
            continue;
        }
        let first = out.vertices.len() as u32;
        if inside {
            out.vertices.extend(corners);
            out.indices.extend([first, first + 1, first + 2]);
            continue;
        }
        let kept = clip(&clip(&corners, top, true), bottom, false);
        if kept.len() < 3 {
            continue;
        }
        out.vertices.extend(kept.iter().copied());
        for next in 1..kept.len() as u32 - 1 {
            out.indices.extend([first, first + next, first + next + 1]);
        }
    }
    out
}

/// One step of Sutherland and Hodgman's clipping: the part of a polygon on one side of the line where
/// the texture's row is `at`.
fn clip(polygon: &[Vertex], at: f32, below: bool) -> Vec<Vertex> {
    let inside = |v: &Vertex| if below { v.uv.y >= at } else { v.uv.y <= at };
    let mut out = Vec::with_capacity(polygon.len() + 2);
    for (i, current) in polygon.iter().enumerate() {
        let previous = &polygon[(i + polygon.len() - 1) % polygon.len()];
        if inside(current) != inside(previous) {
            let t = (at - previous.uv.y) / (current.uv.y - previous.uv.y);
            let mix =
                |a: u8, b: u8| (f32::from(a) + (f32::from(b) - f32::from(a)) * t).round() as u8;
            let [pa, pb] = [previous.color.to_array(), current.color.to_array()];
            out.push(Vertex {
                pos: previous.pos + (current.pos - previous.pos) * t,
                uv: previous.uv + (current.uv - previous.uv) * t,
                color: Color32::from_rgba_premultiplied(
                    mix(pa[0], pb[0]),
                    mix(pa[1], pb[1]),
                    mix(pa[2], pb[2]),
                    mix(pa[3], pb[3]),
                ),
            });
        }
        if inside(current) {
            out.push(*current);
        }
    }
    out
}

/// What a piece of glass's shapes were built from. Kept beside them, so a still frame compares and
/// builds nothing.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Built {
    pub glass: Glass,
    pub pane: Pane,
    pub backdrop: Arc<Backdrop>,
    pub to_global: egui::emath::TSTransform,
    pub pixels_per_point: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn card() -> Pane {
        Pane {
            rect: Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(424.0, 152.0)),
            corners: Corners::all(22.0),
            bevel: 28.0,
            strength: 61.2,
        }
    }

    /// The handoff's Python, at three points: `R = 128 + (−gx × mag + lx) × 127`, read back as an offset
    /// of `S × (R/255 − 0.5)`.
    fn the_designs_offset(x: f32, y: f32, gx: f32, gy: f32, d: f32) -> Vec2 {
        let (w, h, bevel, lens, s) = (424.0f32, 152.0f32, 28.0f32, 0.42f32, 61.2f32);
        let t = (1.0 + d / bevel).clamp(0.0, 1.0);
        let mag = t.powf(2.2);
        let inner = (-d / bevel).clamp(0.0, 1.0);
        let hx = (x - w / 2.0) / (w / 2.0);
        let hy = (y - h / 2.0) / (h / 2.0);
        let lx = hx * lens * inner;
        let ly = hy * lens * inner * (h / w * 2.2).min(1.0);
        let r = (128.0 + (-gx * mag + lx) * 127.0).clamp(0.0, 255.0);
        let g = (128.0 + (-gy * mag + ly) * 127.0).clamp(0.0, 255.0);
        Vec2::new(s * (r / 255.0 - 0.5), s * (g / 255.0 - 0.5))
    }

    #[test]
    fn the_bend_is_the_designs_displacement_map() {
        let glass = Glass::DESIGN;
        let pane = card();
        for (x, y, gx, gy, d) in [
            (424.0, 76.0, 1.0, 0.0, 0.0),
            (410.0, 76.0, 1.0, 0.0, -14.0),
            (300.0, 60.0, 0.0, -1.0, -60.0),
            (100.0, 140.0, 0.0, 1.0, -12.0),
        ] {
            let ours = offset_at(&glass, &pane, Pos2::new(x, y), Vec2::new(gx, gy), d);
            let theirs = the_designs_offset(x, y, gx, gy, d);
            // Within the map's own rounding: one step of 255 at this strength is a quarter of a point.
            assert!((ours - theirs).length() < 0.4, "at {x},{y}: {ours:?} against {theirs:?}");
        }
    }

    #[test]
    fn the_centre_is_not_bent_and_the_edge_is_bent_most() {
        let glass = Glass::DESIGN;
        let pane = card();
        let centre = offset_at(&glass, &pane, pane.rect.center(), Vec2::ZERO, -76.0);
        assert_eq!(centre, Vec2::ZERO);
        let edge = offset_at(&glass, &pane, Pos2::new(424.0, 76.0), Vec2::new(1.0, 0.0), 0.0);
        let inside = offset_at(&glass, &pane, Pos2::new(404.0, 76.0), Vec2::new(1.0, 0.0), -20.0);
        assert!(edge.x.abs() > inside.x.abs(), "the rim bends hardest at the edge");
    }

    #[test]
    fn every_vertex_is_inside_the_shape_or_its_feather() {
        let glass = Glass::DESIGN;
        for pane in [
            card(),
            Pane { corners: Corners::all(19.0), ..card() },
            Pane {
                rect: Rect::from_min_size(Pos2::new(5.0, 5.0), Vec2::new(38.0, 38.0)),
                corners: Corners::all(19.0),
                bevel: 16.0,
                strength: 27.0,
            },
            Pane {
                rect: Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(300.0, 64.0)),
                corners: Corners { nw: 22.0, ne: 22.0, se: 8.0, sw: 22.0 },
                bevel: 20.0,
                strength: 40.0,
            },
        ] {
            let (samples, indices) = geometry(&glass, &pane, 1.0);
            let room = pane.rect.expand(1.01);
            assert!(samples.iter().all(|s| room.contains(s.at)), "{pane:?}");
            assert!(indices.iter().all(|&i| (i as usize) < samples.len()));
            assert_eq!(indices.len() % 3, 0);
        }
    }

    #[test]
    fn the_strips_together_paint_what_the_whole_mesh_paints() {
        let glass = Glass::DESIGN;
        let pane = card();
        let backdrop = Backdrop {
            bands: [(0.0, 0.3), (0.3, 0.55), (0.55, 1.0)]
                .iter()
                .enumerate()
                .map(|(i, &(top, bottom))| Band {
                    texture: TextureId::User(i as u64),
                    drawn: (top, bottom),
                    held: (top, bottom),
                })
                .collect(),
            rect: Rect::from_min_size(Pos2::ZERO, Vec2::new(500.0, 200.0)),
            uv: Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            opacity: 1.0,
        };
        let drawn = shapes(&glass, &pane, &backdrop, Default::default(), 1.0);
        let area = |mesh: &Mesh| -> f32 {
            mesh.indices
                .chunks_exact(3)
                .map(|t| {
                    let [a, b, c] = [0, 1, 2].map(|i| mesh.vertices[t[i] as usize].pos);
                    ((b - a).x * (c - a).y - (b - a).y * (c - a).x).abs() / 2.0
                })
                .sum()
        };
        let pieces: f32 = drawn
            .iter()
            .filter_map(|shape| match shape {
                Shape::Mesh(mesh) => Some(area(mesh)),
                _ => None,
            })
            .sum();
        // Three passes, each covering the piece of glass once.
        let one_pass = pane.rect.area();
        assert!(
            (pieces / 3.0 - one_pass).abs() < one_pass * 0.02,
            "{pieces} against three of {one_pass}"
        );
        assert!(matches!(drawn[0], Shape::Rect(_)), "the cover goes first");
    }

    #[test]
    fn the_three_passes_add_rather_than_cover() {
        let glass = Glass::DESIGN;
        let pane = card();
        let backdrop = Backdrop::single(
            TextureId::User(1),
            Rect::from_min_size(Pos2::ZERO, Vec2::new(500.0, 200.0)),
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            1.0,
        );
        let drawn = shapes(&glass, &pane, &backdrop, Default::default(), 1.0);
        let Shape::Mesh(mesh) = &drawn[1] else { panic!("expected the bent picture") };
        assert!(mesh.vertices.iter().all(|v| v.color.a() == 0), "alpha zero is what makes it add");
        let channels: Vec<usize> = mesh
            .vertices
            .iter()
            .filter(|v| v.color != Color32::TRANSPARENT)
            .map(|v| v.color.to_array().iter().position(|&c| c > 0).unwrap())
            .collect();
        assert!(channels.contains(&0) && channels.contains(&1) && channels.contains(&2));
    }
}
