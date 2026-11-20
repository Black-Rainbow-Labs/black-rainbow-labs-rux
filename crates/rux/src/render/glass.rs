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
//! copy of its own mesh at the picture's opacity, and then adds red, green and blue, each sampled at its
//! own strength. The order of the three does not matter, which is what lets them be cut across the
//! picture's strips independently.
//!
//! **The cover is the passes' own mesh, so the two fade out together at the edge.** It used to be an
//! `egui` rounded rectangle, whose one pixel feather is centred on the edge, while the passes stay whole
//! up to the edge and fade only outside it. Along the outline the picture was then added over a cover
//! that was already partly gone, and every piece of glass had a thin ring brighter than both sides of
//! it: the white edge reported on task-2281.
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

/// How far a corner's straight segments may stray from the true curve, in physical pixels.
///
/// The surface's tint under the glass is a true curve, so where the glass's outline is a polygon the two
/// disagree along each flat and the edge reads as faceted. At one segment every four points, at most eight
/// a corner, a chip's outline strayed a third of a pixel at 1.5 pixels a point, and Jason saw the facets
/// (`task-2281`, fifth pass). A twentieth of a pixel is below what the one pixel feather can show.
const CURVE_TOLERANCE: f32 = 0.05;

/// How many segments a corner whose radius is `radius` physical pixels is cut into, so no segment strays more than
/// [`CURVE_TOLERANCE`] from the curve.
fn segments(radius: f32) -> usize {
    if !radius.is_finite() || radius <= CURVE_TOLERANCE {
        return 2;
    }
    let step = 2.0 * (1.0 - CURVE_TOLERANCE / radius).acos();
    ((std::f32::consts::FRAC_PI_2 / step).ceil() as usize).clamp(2, 64)
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
/// `feather` is how wide the faded band just inside the edge is, in the layer's points: one
/// physical pixel, which is the feather `egui` gives its own rounded rectangles.
pub fn geometry(glass: &Glass, pane: &Pane, feather: f32) -> (Vec<Sample>, Vec<u32>) {
    let half = pane.rect.width().min(pane.rect.height()) / 2.0;
    let radii = radii(pane.rect, pane.corners);
    // `feather` is one physical pixel in the layer's points, so this is each corner's radius in pixels.
    let cuts = radii.map(|radius| segments(radius / feather.max(0.001)));
    let deepest = pane.bevel.min(half - 0.25).max(0.0);
    let mut samples = Vec::new();
    let mut rings = Vec::new();
    // The faded ring on the edge, the whole ring one pixel inside it, then deeper rings across the rim.
    // **The glass fades inside the edge**, under the tint's own antialiased edge (`task-2281`): with the
    // glass whole up to the edge or past it, the edge pixel had more of the picture than of the tint's
    // darkening, a light line on a bright picture. Measured with `glass-rims.py` on a bright picture, the
    // fade centred on the edge left +4.6 at two pixels a point once the outline was a true curve; inside
    // it, +2.3, and +0.3 at 1.5.
    rings.push(samples.len());
    samples.extend(ring(pane, glass, 0.0, cuts, true));
    for step in 0..=RINGS {
        rings.push(samples.len());
        let inset = (deepest * step as f32 / RINGS as f32).max(feather);
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
    shapes_cut(glass, pane, backdrop, to_global, pixels_per_point, Seams::Apart)
}

/// How a piece of glass is cut at the seams between its picture's strips (`task-2810`).
///
/// Both draw the same triangles. [`Seams::Apart`] gives every triangle three vertices of its own, which is how
/// glass has always been cut. [`Seams::Shared`] keeps one vertex for each point of the mesh inside a strip and
/// one for each place an edge crosses a seam, the way the mesh was built. Measured in Chordical at two pixels a
/// point over a picture uploaded as 32 strips: a 28 point round button was 9,425 vertices apart, and the
/// window's own glass 30,277, because every triangle a strip keeps was copied out whole.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Seams {
    /// Every triangle its own three vertices.
    #[default]
    Apart,
    /// Triangles that meet share the vertices they meet at.
    Shared,
}

/// [`shapes`], cut at the strips the way `seams` says.
///
/// @param glass - the material
/// @param pane - the piece of glass, in the layer's points
/// @param backdrop - the picture it bends
/// @param to_global - from the layer's points to the window's
/// @param pixels_per_point - the display's density
/// @param seams - whether the triangles a strip keeps share their vertices
pub fn shapes_cut(
    glass: &Glass,
    pane: &Pane,
    backdrop: &Backdrop,
    to_global: egui::emath::TSTransform,
    pixels_per_point: f32,
    seams: Seams,
) -> Vec<Shape> {
    if pane.rect.width() < 1.0 || pane.rect.height() < 1.0 || backdrop.bands.is_empty() {
        return Vec::new();
    }
    let scale = (pixels_per_point * to_global.scaling).max(0.1);
    let (samples, indices) = geometry(glass, pane, 1.0 / scale);
    let opacity = (backdrop.opacity.clamp(0.0, 1.0) * 255.0).round() as u8;
    let mut shapes = Vec::with_capacity(1 + backdrop.bands.len());
    shapes.push(cover(&samples, &indices, opacity));

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
        let mut piece = match seams {
            Seams::Apart => cut(&whole, band.drawn.0, band.drawn.1),
            Seams::Shared => cut_sharing_vertices(&whole, band.drawn.0, band.drawn.1),
        };
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
    note_the_vertices(pane, &shapes);
    shapes
}

/// Append one line to the file `RUX_DEBUG_GLASS` names for a piece of glass just built: its rectangle, its
/// corners, how many shapes it is and how many vertices each has (`task-2810`).
///
/// A piece of glass is built once and kept while nothing it is built from moves, so the file has one line for
/// each piece of glass a window shows and one more each time one moves. That is how the vertices of a frame
/// under glass are traced to the surfaces they belong to.
///
/// @param pane - the piece of glass
/// @param shapes - what it was built as
fn note_the_vertices(pane: &Pane, shapes: &[Shape]) {
    let Some(file) = std::env::var_os("RUX_DEBUG_GLASS") else {
        return;
    };
    let counts: Vec<usize> = shapes
        .iter()
        .map(|shape| match shape {
            Shape::Mesh(mesh) => mesh.vertices.len(),
            _ => 0,
        })
        .collect();
    let total: usize = counts.iter().sum();
    let rect = pane.rect;
    let line = format!(
        "glass {:.0}x{:.0} at {:.0},{:.0} corners {:.1} vertices {total} shapes {} each {counts:?}\n",
        rect.width(),
        rect.height(),
        rect.min.x,
        rect.min.y,
        pane.corners.nw,
        shapes.len(),
    );
    use std::io::Write;
    if let Ok(mut out) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(file)
    {
        let _ = out.write_all(line.as_bytes());
    }
}

/// What covers what is behind the glass, by the picture's own opacity, so the three passes that follow
/// can add: the passes' own mesh in black, fading on the same feathered ring, so the cover and the
/// passes leave exactly the same share of the background at every point of the edge.
///
/// @param samples - the points of the glass, from [`geometry`]
/// @param indices - its triangles
/// @param opacity - how much of what is behind it the cover hides, 0 to 255
fn cover(samples: &[Sample], indices: &[u32], opacity: u8) -> Shape {
    let dark = Color32::from_rgba_premultiplied(0, 0, 0, opacity);
    let vertices = samples
        .iter()
        .map(|sample| Vertex {
            pos: sample.at,
            uv: egui::epaint::WHITE_UV,
            color: if sample.feather { Color32::TRANSPARENT } else { dark },
        })
        .collect();
    Shape::mesh(Mesh { indices: indices.to_vec(), vertices, ..Mesh::default() })
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
        let row = |v: &Vertex| v.uv.y;
        let kept = clip(&clip(&corners, row, top, true), row, bottom, false);
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

/// A point of a mesh being cut at a band's two lines: one of the mesh's own vertices, or the place one of its
/// edges crosses a line. An edge is named by its two vertices, lower index first, so the two triangles that
/// share an edge name the same crossing and are given the same vertex for it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Point {
    Own(u32),
    /// The edge's two vertices, and whether the line is the band's bottom.
    Crossing(u32, u32, bool),
}

/// [`cut`], with the triangles it keeps sharing their vertices the way they did in `mesh` (`task-2810`).
///
/// A vertex of `mesh` inside the band is kept once, however many triangles use it, and the place an edge crosses
/// the band's top or bottom is made once, from the edge's own two vertices, for both triangles either side of it.
/// The triangles are the ones [`cut`] keeps.
///
/// @param mesh - the whole piece, in picture coordinates
/// @param top - the band's top, as a fraction of the picture's height
/// @param bottom - its bottom
pub fn cut_sharing_vertices(mesh: &Mesh, top: f32, bottom: f32) -> Mesh {
    let mut out = Mesh::default();
    let mut placed: std::collections::HashMap<Point, u32> = std::collections::HashMap::new();
    let lines = (top, bottom);
    for triangle in mesh.indices.chunks_exact(3) {
        let rows = [0, 1, 2].map(|i| mesh.vertices[triangle[i] as usize].uv.y);
        if rows.iter().all(|&y| y < top) || rows.iter().all(|&y| y > bottom) {
            continue;
        }
        let corners = [0, 1, 2].map(|i| Point::Own(triangle[i]));
        let kept = match rows.iter().all(|&y| y >= top && y <= bottom) {
            true => corners.to_vec(),
            false => {
                let below_top = clip_points(&corners, mesh, lines, false);
                clip_points(&below_top, mesh, lines, true)
            }
        };
        if kept.len() < 3 {
            continue;
        }
        let ids: Vec<u32> = kept
            .iter()
            .map(|&point| place(&mut out, &mut placed, mesh, lines, point))
            .collect();
        for next in 1..ids.len() - 1 {
            out.indices.extend([ids[0], ids[next], ids[next + 1]]);
        }
    }
    out
}

/// The texture row a [`Point`] is at: its vertex's, or the line it was made on.
fn row_of(mesh: &Mesh, lines: (f32, f32), point: Point) -> f32 {
    match point {
        Point::Own(index) => mesh.vertices[index as usize].uv.y,
        Point::Crossing(_, _, bottom) => {
            if bottom {
                lines.1
            } else {
                lines.0
            }
        }
    }
}

/// The edge of `mesh` the side of a polygon from `from` to `to` lies on, lower index first.
fn edge_under(from: Point, to: Point) -> Option<(u32, u32)> {
    let (a, b) = match (from, to) {
        (Point::Own(a), Point::Own(b)) => (a, b),
        (Point::Crossing(a, b, _), Point::Own(_)) | (Point::Own(_), Point::Crossing(a, b, _)) => {
            (a, b)
        }
        // Two crossings on one line: that side lies along the line and crosses no other.
        _ => return None,
    };
    Some((a.min(b), a.max(b)))
}

/// One step of Sutherland and Hodgman's clipping over [`Point`]s: the part of `polygon` at or below the band's
/// top line when `bottom` is false, and at or above its bottom line when it is true.
fn clip_points(polygon: &[Point], mesh: &Mesh, lines: (f32, f32), bottom: bool) -> Vec<Point> {
    let inside = |point: Point| {
        let row = row_of(mesh, lines, point);
        if bottom {
            row <= lines.1
        } else {
            row >= lines.0
        }
    };
    let mut out = Vec::with_capacity(polygon.len() + 2);
    for (i, &current) in polygon.iter().enumerate() {
        let previous = polygon[(i + polygon.len() - 1) % polygon.len()];
        if inside(current) != inside(previous) {
            if let Some((a, b)) = edge_under(previous, current) {
                out.push(Point::Crossing(a, b, bottom));
            }
        }
        if inside(current) {
            out.push(current);
        }
    }
    out
}

/// The index in `out` of `point`, adding its vertex the first time it is asked for.
fn place(
    out: &mut Mesh,
    placed: &mut std::collections::HashMap<Point, u32>,
    mesh: &Mesh,
    lines: (f32, f32),
    point: Point,
) -> u32 {
    if let Some(&index) = placed.get(&point) {
        return index;
    }
    let vertex = match point {
        Point::Own(index) => mesh.vertices[index as usize],
        Point::Crossing(a, b, _) => {
            let (from, to) = (mesh.vertices[a as usize], mesh.vertices[b as usize]);
            let at = row_of(mesh, lines, point);
            between(&from, &to, (at - from.uv.y) / (to.uv.y - from.uv.y))
        }
    };
    let index = out.vertices.len() as u32;
    out.vertices.push(vertex);
    placed.insert(point, index);
    index
}

/// The vertex the fraction `t` of the way from `from` to `to`: position, texture coordinate and colour.
fn between(from: &Vertex, to: &Vertex, t: f32) -> Vertex {
    let mix = |a: u8, b: u8| (f32::from(a) + (f32::from(b) - f32::from(a)) * t).round() as u8;
    let [pa, pb] = [from.color.to_array(), to.color.to_array()];
    Vertex {
        pos: from.pos + (to.pos - from.pos) * t,
        uv: from.uv + (to.uv - from.uv) * t,
        color: Color32::from_rgba_premultiplied(
            mix(pa[0], pb[0]),
            mix(pa[1], pb[1]),
            mix(pa[2], pb[2]),
            mix(pa[3], pb[3]),
        ),
    }
}

/// The part of `mesh` inside `rect`, in the mesh's own positions, cut where its triangles cross the edge.
///
/// A piece of glass is an `egui` mesh drawn beside the layer's rasterised runs, so a rounded clip the
/// layer has open, such as a board lane round its scrolling cards, does not reach it the way it reaches
/// the runs: a card scrolled past the top of its lane drew its glass over the lane's heading
/// (`task-2281`). The glass is cut to the clip's rectangle. Its rounded corners are not followed, so a
/// piece of glass passing under a clip's corner shows by the corner's radius at most.
///
/// @param mesh - a piece of glass's shape
/// @param rect - what it may be drawn inside
pub fn clip_to_rect(mesh: &Mesh, rect: Rect) -> Mesh {
    let mut out = Mesh { texture_id: mesh.texture_id, ..Mesh::default() };
    let x = |v: &Vertex| v.pos.x;
    let y = |v: &Vertex| v.pos.y;
    for triangle in mesh.indices.chunks_exact(3) {
        let corners = [0, 1, 2].map(|i| mesh.vertices[triangle[i] as usize]);
        let inside = corners.iter().all(|v| rect.contains(v.pos));
        let first = out.vertices.len() as u32;
        if inside {
            out.vertices.extend(corners);
            out.indices.extend([first, first + 1, first + 2]);
            continue;
        }
        let kept = clip(&corners, x, rect.min.x, true);
        let kept = clip(&kept, x, rect.max.x, false);
        let kept = clip(&kept, y, rect.min.y, true);
        let kept = clip(&kept, y, rect.max.y, false);
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
/// `value` of a vertex is `at`, the side at or above it when `above` and at or below it otherwise.
fn clip(polygon: &[Vertex], value: impl Fn(&Vertex) -> f32, at: f32, above: bool) -> Vec<Vertex> {
    let inside = |v: &Vertex| if above { value(v) >= at } else { value(v) <= at };
    let mut out = Vec::with_capacity(polygon.len() + 2);
    for (i, current) in polygon.iter().enumerate() {
        let previous = &polygon[(i + polygon.len() - 1) % polygon.len()];
        if inside(current) != inside(previous) {
            let t = (at - value(previous)) / (value(current) - value(previous));
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
    /// How it is cut at the picture's strips (`task-2810`).
    pub seams: Seams,
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
            .skip(1)
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
        assert!(
            matches!(&drawn[0], Shape::Mesh(mesh) if mesh.texture_id == TextureId::default()),
            "the cover goes first, untextured"
        );
    }

    #[test]
    fn glass_cut_to_a_clip_stays_inside_it_and_keeps_its_area_there() {
        // A card scrolled half past the top of its lane (task-2281): only the part inside the lane is drawn.
        let pane = card();
        let backdrop = Backdrop::single(
            TextureId::User(1),
            Rect::from_min_size(Pos2::ZERO, Vec2::new(500.0, 200.0)),
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            1.0,
        );
        let drawn = shapes(&Glass::DESIGN, &pane, &backdrop, Default::default(), 1.0);
        let Shape::Mesh(whole) = &drawn[1] else { panic!("expected the bent picture") };
        let lane = Rect::from_min_max(Pos2::new(-10.0, 76.0), Pos2::new(600.0, 400.0));
        let kept = clip_to_rect(whole, lane);
        assert!(kept.vertices.iter().all(|v| lane.expand(0.01).contains(v.pos)));
        let area = |mesh: &Mesh| -> f32 {
            mesh.indices
                .chunks_exact(3)
                .map(|t| {
                    let [a, b, c] = [0, 1, 2].map(|i| mesh.vertices[t[i] as usize].pos);
                    ((b - a).x * (c - a).y - (b - a).y * (c - a).x).abs() / 2.0
                })
                .sum()
        };
        let half = area(whole) * (152.0 - 76.0) / 152.0;
        assert!((area(&kept) - half).abs() < half * 0.05, "{} against {half}", area(&kept));
    }

    #[test]
    fn the_cover_fades_out_on_exactly_the_vertices_the_passes_do() {
        // The white edge of task-2281: a cover that fades on a different line from the passes leaves a
        // ring where the picture is added over a background the cover has not hidden.
        let pane = card();
        let backdrop = Backdrop::single(
            TextureId::User(1),
            Rect::from_min_size(Pos2::ZERO, Vec2::new(500.0, 200.0)),
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            0.8,
        );
        let drawn = shapes(&Glass::DESIGN, &pane, &backdrop, Default::default(), 1.0);
        let Shape::Mesh(cover) = &drawn[0] else { panic!("the cover is a mesh") };
        let (samples, indices) = geometry(&Glass::DESIGN, &pane, 1.0);
        assert_eq!(cover.indices, indices);
        assert_eq!(cover.vertices.len(), samples.len());
        for (vertex, sample) in cover.vertices.iter().zip(&samples) {
            assert_eq!(vertex.pos, sample.at);
            // The passes are transparent on the feather and whole everywhere else; so is the cover.
            assert_eq!(vertex.color.a(), if sample.feather { 0 } else { 204 });
        }
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

    /// A backdrop of `strips` equal strips over a 500 by 200 point picture.
    fn striped(strips: usize) -> Backdrop {
        let bands = (0..strips)
            .map(|i| {
                let (top, bottom) = (i as f32 / strips as f32, (i + 1) as f32 / strips as f32);
                Band {
                    texture: TextureId::User(i as u64),
                    drawn: (top, bottom),
                    held: (top, bottom),
                }
            })
            .collect();
        Backdrop {
            bands,
            rect: Rect::from_min_size(Pos2::ZERO, Vec2::new(500.0, 200.0)),
            uv: Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            opacity: 0.75,
        }
    }

    /// Every triangle of `mesh` as its three corners' positions, texture coordinates and colours.
    fn triangles(mesh: &Mesh) -> Vec<[Vertex; 3]> {
        mesh.indices
            .chunks_exact(3)
            .map(|t| [0, 1, 2].map(|i| mesh.vertices[t[i] as usize]))
            .collect()
    }

    #[test]
    fn shared_seams_draw_the_same_triangles_with_fewer_vertices() {
        // task-2810: a 28 point round button and a card, over a picture in 32 strips, at two pixels a point.
        let button = Pane {
            rect: Rect::from_min_size(Pos2::new(208.0, 56.0), Vec2::splat(28.0)),
            corners: Corners::all(14.0),
            bevel: 14.0,
            strength: 20.0,
        };
        for pane in [button, card()] {
            let backdrop = striped(32);
            let apart = shapes_cut(
                &Glass::DESIGN,
                &pane,
                &backdrop,
                Default::default(),
                2.0,
                Seams::Apart,
            );
            let shared = shapes_cut(
                &Glass::DESIGN,
                &pane,
                &backdrop,
                Default::default(),
                2.0,
                Seams::Shared,
            );
            assert_eq!(apart.len(), shared.len(), "the same pieces, one a strip");
            let count = |drawn: &[Shape]| -> usize {
                drawn
                    .iter()
                    .map(|s| {
                        if let Shape::Mesh(m) = s {
                            m.vertices.len()
                        } else {
                            0
                        }
                    })
                    .sum()
            };
            assert!(
                count(&shared) * 2 < count(&apart),
                "{} against {}",
                count(&shared),
                count(&apart)
            );
            for (a, b) in apart.iter().zip(&shared) {
                let (Shape::Mesh(a), Shape::Mesh(b)) = (a, b) else {
                    panic!("glass is meshes")
                };
                assert_eq!(a.texture_id, b.texture_id);
                let (a, b) = (triangles(a), triangles(b));
                assert_eq!(a.len(), b.len());
                for (x, y) in a.iter().zip(&b) {
                    for (p, q) in x.iter().zip(y) {
                        assert!((p.pos - q.pos).length() < 1e-3, "{p:?} against {q:?}");
                        assert!((p.uv - q.uv).length() < 1e-5, "{p:?} against {q:?}");
                        let [c, d] = [p.color.to_array(), q.color.to_array()];
                        assert!(
                            c.iter().zip(d).all(|(c, d)| c.abs_diff(d) <= 1),
                            "{p:?} against {q:?}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn a_crossing_shared_by_two_triangles_is_one_vertex() {
        // Two triangles making a square across one seam: the two points on the shared diagonal's crossing and
        // the corners above the seam are each kept once.
        let at = |x: f32, y: f32| Vertex {
            pos: Pos2::new(x, y),
            uv: Pos2::new(x, y),
            color: Color32::WHITE,
        };
        let square = Mesh {
            vertices: vec![at(0.0, 0.0), at(1.0, 0.0), at(1.0, 1.0), at(0.0, 1.0)],
            indices: vec![0, 1, 2, 0, 2, 3],
            ..Mesh::default()
        };
        let upper = cut_sharing_vertices(&square, 0.0, 0.5);
        // The two top corners, the right edge's and the left edge's crossings, and the diagonal's crossing.
        assert_eq!(upper.vertices.len(), 5);
        assert_eq!(cut(&square, 0.0, 0.5).vertices.len(), 7);
    }

    #[test]
    fn a_round_chips_outline_stays_within_a_twentieth_of_a_pixel_of_the_circle() {
        // task-2281, fifth pass: a 34 point chip at 1.5 pixels a point had flats a third of a pixel deep.
        for scale in [1.0_f32, 1.5, 2.0, 3.0] {
            let rect = Rect::from_center_size(Pos2::new(100.0, 100.0), Vec2::splat(34.0));
            let pane = Pane { rect, corners: Corners::from(17.0), bevel: 8.0, strength: 20.0 };
            let (samples, _) = geometry(&Glass::DESIGN, &pane, 1.0 / scale);
            // The first whole ring is the one on the edge; it is the second ring in the list.
            let around = samples.len() / (RINGS + 2);
            let edge = &samples[around..around * 2];
            let radius = (edge[0].at - rect.center()).length();
            for pair in edge.windows(2) {
                let middle = pair[0].at + (pair[1].at - pair[0].at) * 0.5;
                let sag = (radius - (middle - rect.center()).length()) * scale;
                assert!(sag <= CURVE_TOLERANCE + 0.001, "a flat {sag} pixels deep at {scale} pixels a point");
            }
        }
    }
}
