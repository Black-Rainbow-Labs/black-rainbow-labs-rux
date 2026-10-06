//! The icon set, and the atlas that turns one into pixels once.
//!
//! **An icon is a tinted texture, not a path in the chrome canvas.** That is how an icon font works and
//! it buys the one thing the whole rendering design rests on: a hover that changes an icon's colour
//! costs a vertex colour rather than a rasterisation of the layer it is on. The reference changes an
//! icon's colour on hover in nineteen separate rules, so this is not a micro-optimisation — it is the
//! difference between moving the pointer across a toolbar being free and it rasterising a full-window
//! canvas nineteen times.
//!
//! The rasteriser is `vello_cpu`, so the caps, the joins and the curve flattening are the same ones
//! everything else here is drawn with.

pub mod paths;

use std::collections::HashMap;

use egui::{Color32, Mesh, Pos2, Rect, Vec2};
use vello_cpu::kurbo::{Affine, BezPath, Circle, RoundedRect, Shape as _, Stroke};
use vello_cpu::peniko::color::AlphaColor;
use vello_cpu::{
    CompositeMode, PixelFormat, Pixmap, RasterizerSettings, RenderContext, RenderMode,
    RenderSettings, Resources,
};

pub use paths::{Icon, Part, Shape, DEFAULT_STROKE, VIEW_BOX};

/// A pixel of slack round every mark, so a round cap that reaches the viewBox's edge is not clipped
/// by the texture it is drawn into.
const PAD: u16 = 1;

/// What a cached texture was made for.
///
/// The stroke width is quantised, because a caller that computes one from a zoom would otherwise make
/// a new texture on every frame of a pinch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Key {
    icon: Icon,
    /// The mark's side in **physical pixels**, which is what actually gets rasterised.
    pixels: u16,
    /// The stroke width in viewBox units, in hundredths.
    stroke: u16,
}

/// What a cached **arc** texture was made for.
///
/// The gradient is baked into the pixels, so the two colours are part of the key. The angles are not:
/// an arc is rasterised once pointing at twelve o'clock and **turned** when it is drawn, which is what
/// makes a spinner cost a mesh a frame rather than a rasterisation. See [`paint_arc`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ArcKey {
    /// The centreline radius in physical pixels.
    radius: u16,
    /// The stroke width in physical pixels, in tenths.
    stroke: u16,
    /// The sweep in hundredths of a radian.
    sweep: u16,
    from: [u8; 4],
    to: [u8; 4],
}

/// Every icon texture this window is holding.
///
/// One entry per (icon, size, stroke, pixel density), shared by every use — so a toolbar of eight
/// different marks at one size is eight small alpha textures and nothing else, however many times each
/// is drawn.
pub struct IconAtlas {
    context: RenderContext,
    resources: Resources,
    pixmap: Pixmap,
    by_key: HashMap<Key, egui::TextureHandle>,
    arcs: HashMap<ArcKey, egui::TextureHandle>,
    /// How many marks have actually been rasterised, for `--measure` and for the test that says the
    /// second draw of an icon is free.
    rasterisations: u64,
    deterministic: bool,
}

impl std::fmt::Debug for IconAtlas {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        out.debug_struct("IconAtlas")
            .field("marks", &self.by_key.len())
            .field("rasterisations", &self.rasterisations)
            .finish()
    }
}

impl Default for IconAtlas {
    fn default() -> Self {
        Self::build(false)
    }
}

impl IconAtlas {
    /// An atlas whose marks rasterise the same on every machine. See `Canvas::deterministic`.
    pub fn deterministic() -> Self {
        Self::build(true)
    }

    fn build(deterministic: bool) -> Self {
        let settings = match deterministic {
            true => RenderSettings { level: vello_cpu::Level::baseline(), num_threads: 0 },
            false => RenderSettings::default(),
        };
        Self {
            context: RenderContext::new_with(1, 1, settings),
            resources: Resources::new(),
            pixmap: Pixmap::new(1, 1),
            by_key: HashMap::new(),
            arcs: HashMap::new(),
            rasterisations: 0,
            deterministic,
        }
    }

    pub fn rasterisations(&self) -> u64 {
        self.rasterisations
    }

    /// How many marks are held and how many bytes of texture they come to, for `--measure`.
    ///
    /// Four bytes a pixel, because `egui` keeps every texture as `Color32` whatever is in it.
    pub fn footprint(&self) -> (usize, usize) {
        let marks: usize = self
            .by_key
            .keys()
            .map(|key| {
                let side = usize::from(key.pixels + PAD * 2);
                side * side * 4
            })
            .sum();
        let arcs: usize = self
            .arcs
            .keys()
            .map(|key| {
                let side = usize::from(key.radius) * 2 + usize::from(key.stroke) / 10 + 2;
                side * side * 4
            })
            .sum();
        (self.by_key.len() + self.arcs.len(), marks + arcs)
    }

    /// The texture for this mark at this size, making it if it is not held yet.
    ///
    /// `scale` is how many pixels a point is composited at — see `render::composited_scale`.
    fn texture(
        &mut self,
        ctx: &egui::Context,
        icon: Icon,
        size: f32,
        stroke: f32,
        scale: f32,
    ) -> Option<(egui::TextureId, u16)> {
        let pixels = (size * scale).round();
        if !pixels.is_finite() || pixels < 1.0 || pixels > 512.0 {
            return None;
        }
        let key = Key {
            icon,
            pixels: pixels as u16,
            stroke: (stroke.clamp(0.1, 8.0) * 100.0).round() as u16,
        };
        if let Some(texture) = self.by_key.get(&key) {
            return Some((texture.id(), key.pixels));
        }
        let image = self.rasterise(key);
        let texture = ctx.load_texture(
            format!("rux-icon-{}-{}-{}", icon.name(), key.pixels, key.stroke),
            image,
            egui::TextureOptions::LINEAR,
        );
        let id = texture.id();
        self.by_key.insert(key, texture);
        Some((id, key.pixels))
    }

    /// Draw the mark in white, so the alpha channel is the mask every tint multiplies.
    fn rasterise(&mut self, key: Key) -> egui::ColorImage {
        self.rasterisations += 1;
        let side = key.pixels + PAD * 2;
        if self.pixmap.width() != side || self.pixmap.height() != side {
            self.pixmap.resize(side, side);
        }
        self.context.reset_and_resize(side, side);
        let scale = f64::from(key.pixels) / f64::from(VIEW_BOX);
        self.context
            .set_transform(Affine::translate((f64::from(PAD), f64::from(PAD))) * Affine::scale(scale));
        self.context.set_paint(AlphaColor::from_rgba8(255, 255, 255, 255));
        // The stroke is given in viewBox units and the transform scales it, exactly as the browser
        // scales an SVG's `stroke-width`. Round caps and joins are the `<svg>`'s own.
        let stroke = f64::from(key.stroke) / 100.0;
        self.context.set_stroke(
            Stroke::new(stroke)
                .with_caps(vello_cpu::kurbo::Cap::Round)
                .with_join(vello_cpu::kurbo::Join::Round),
        );
        for part in key.icon.parts() {
            let path = path_of(part.shape);
            if part.fill {
                self.context.fill_path(&path);
            }
            if part.stroke {
                self.context.stroke_path(&path);
            }
        }
        self.context.flush();
        self.context.render_with(
            self.pixmap.as_mut(),
            &mut self.resources,
            RasterizerSettings {
                render_mode: RenderMode::OptimizeQuality,
                composite_mode: CompositeMode::Replace,
                pixel_format: PixelFormat::Rgba8,
                offset: (0, 0),
            },
        );
        let bytes = self.pixmap.data_as_u8_slice();
        let pixels: Vec<Color32> = bytes
            .chunks_exact(4)
            .map(|pixel| Color32::from_rgba_premultiplied(pixel[0], pixel[1], pixel[2], pixel[3]))
            .collect();
        egui::ColorImage::new([usize::from(side), usize::from(side)], pixels)
    }

    /// Whether this atlas rasterises the same on every machine.
    pub fn is_deterministic(&self) -> bool {
        self.deterministic
    }

    /// The texture for a stroked, gradient-filled arc, making it if it is not held yet.
    ///
    /// Rasterised once pointing at twelve o'clock; the turn is applied to the mesh when it is drawn.
    fn arc_texture(
        &mut self,
        ctx: &egui::Context,
        radius: f32,
        stroke: f32,
        sweep: f32,
        from: Color32,
        to: Color32,
        scale: f32,
    ) -> Option<(egui::TextureId, u16)> {
        let radius_px = (radius * scale).round();
        let stroke_px = (stroke * scale).max(0.5);
        if !radius_px.is_finite() || radius_px < 1.0 || radius_px > 512.0 {
            return None;
        }
        let key = ArcKey {
            radius: radius_px as u16,
            stroke: (stroke_px * 10.0).round() as u16,
            sweep: (sweep.abs() * 100.0).round().min(u16::from(u16::MAX)as f32) as u16,
            from: from.to_array(),
            to: to.to_array(),
        };
        let side = (f32::from(key.radius) * 2.0 + stroke_px).ceil() as u16 + PAD * 2;
        if let Some(texture) = self.arcs.get(&key) {
            return Some((texture.id(), side));
        }
        let image = self.rasterise_arc(key, side);
        let texture = ctx.load_texture(
            format!("rux-arc-{}-{}-{}", key.radius, key.stroke, key.sweep),
            image,
            egui::TextureOptions::LINEAR,
        );
        let id = texture.id();
        self.arcs.insert(key, texture);
        Some((id, side))
    }

    fn rasterise_arc(&mut self, key: ArcKey, side: u16) -> egui::ColorImage {
        self.rasterisations += 1;
        if self.pixmap.width() != side || self.pixmap.height() != side {
            self.pixmap.resize(side, side);
        }
        self.context.reset_and_resize(side, side);
        self.context.set_transform(Affine::IDENTITY);
        let centre = f64::from(side) / 2.0;
        let radius = f64::from(key.radius);
        let stroke = f64::from(key.stroke) / 10.0;
        // `<linearGradient x1="0" x2="1" y1="0" y2="1">` — the diagonal of the object's own box, which
        // is what the reference's spinner uses.
        let box_min = centre - radius - stroke / 2.0;
        let box_max = centre + radius + stroke / 2.0;
        let mut stops = vello_cpu::peniko::ColorStops::default();
        // The key holds the premultiplied bytes `Color32` stores, so they are un-premultiplied again
        // here — the one conversion between egui's colour and Vello's, the same one `render::canvas`
        // makes.
        let colour = |bytes: [u8; 4]| {
            let [r, g, b, a] =
                Color32::from_rgba_premultiplied(bytes[0], bytes[1], bytes[2], bytes[3])
                    .to_srgba_unmultiplied();
            AlphaColor::from_rgba8(r, g, b, a)
        };
        stops.0.push(vello_cpu::peniko::ColorStop::from((0.0, colour(key.from))));
        stops.0.push(vello_cpu::peniko::ColorStop::from((1.0, colour(key.to))));
        self.context.set_paint(vello_cpu::peniko::Gradient {
            kind: vello_cpu::peniko::GradientKind::Linear(
                vello_cpu::peniko::LinearGradientPosition::new(
                    vello_cpu::kurbo::Point::new(box_min, box_min),
                    vello_cpu::kurbo::Point::new(box_max, box_max),
                ),
            ),
            stops,
            ..Default::default()
        });
        self.context.set_stroke(
            Stroke::new(stroke)
                .with_caps(vello_cpu::kurbo::Cap::Round)
                .with_join(vello_cpu::kurbo::Join::Round),
        );
        // `kurbo::Arc` measures from three o'clock, so twelve is a quarter turn back.
        let arc = vello_cpu::kurbo::Arc::new(
            vello_cpu::kurbo::Point::new(centre, centre),
            (radius, radius),
            -std::f64::consts::FRAC_PI_2,
            f64::from(key.sweep) / 100.0,
            0.0,
        );
        self.context.stroke_path(&arc.to_path(0.05));
        self.context.flush();
        self.context.render_with(
            self.pixmap.as_mut(),
            &mut self.resources,
            RasterizerSettings {
                render_mode: RenderMode::OptimizeQuality,
                composite_mode: CompositeMode::Replace,
                pixel_format: PixelFormat::Rgba8,
                offset: (0, 0),
            },
        );
        let bytes = self.pixmap.data_as_u8_slice();
        let pixels: Vec<Color32> = bytes
            .chunks_exact(4)
            .map(|pixel| Color32::from_rgba_premultiplied(pixel[0], pixel[1], pixel[2], pixel[3]))
            .collect();
        egui::ColorImage::new([usize::from(side), usize::from(side)], pixels)
    }
}

/// Paint a stroked, gradient-filled arc, turned.
///
/// **This exists so a spinner does not rasterise the whole layer sixty times a second.** The arc is
/// the one stroke in the library whose paint is a gradient and the one shape that moves every frame;
/// recorded as decoration it changed the `Decor` list on every frame, which is exactly the comparison
/// that decides whether to rasterise — measured at 822 ms of processor time a second on a page with
/// three spinners on it, which is most of a core to draw three circles. As a texture with a rotated
/// mesh it costs four vertices.
///
/// The gradient turns with the shape, which is what the CSS animation does too: it rotates the whole
/// `<svg>`, and the gradient is defined in the object's own box.
pub fn paint_arc(
    painter: &egui::Painter,
    atlas: &mut IconAtlas,
    centre: Pos2,
    radius: f32,
    stroke: f32,
    sweep: f32,
    turn: f32,
    from: Color32,
    to: Color32,
) {
    let scale = crate::render::composited_scale(painter.ctx(), painter.layer_id());
    let Some((texture, side)) =
        atlas.arc_texture(painter.ctx(), radius, stroke, sweep, from, to, scale)
    else {
        return;
    };
    let points = f32::from(side) / scale;
    let rect = Rect::from_center_size(centre, Vec2::splat(points));
    let uv = Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0));
    let mut mesh = Mesh::with_texture(texture);
    let (sin, cos) = turn.sin_cos();
    let corners = [rect.left_top(), rect.right_top(), rect.right_bottom(), rect.left_bottom()];
    let uvs = [uv.left_top(), uv.right_top(), uv.right_bottom(), uv.left_bottom()];
    for (corner, uv) in corners.into_iter().zip(uvs) {
        let offset = corner - centre;
        // A vertex with its texture coordinate, pushed whole. See [`textured_vertex`].
        mesh.vertices.push(textured_vertex(
            centre + Vec2::new(offset.x * cos - offset.y * sin, offset.x * sin + offset.y * cos),
            uv,
            Color32::WHITE,
        ));
    }
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    painter.add(egui::Shape::mesh(mesh));
}

/// How an icon is asked for: which mark, how big, how heavy, and whether it is turned.
///
/// `size` is the `width`/`height` the reference's `<SbIcon size={…}/>` passes, and `stroke` is its
/// `stroke` prop — so `<SbIcon name="chevDown" size={9} stroke={2.4}/>` is
/// `Mark::new(Icon::ChevDown, 9.0).stroke(2.4)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mark {
    pub icon: Icon,
    pub size: f32,
    pub stroke: f32,
    /// Clockwise, in radians. `.ic-count__steps button:first-child svg { transform: rotate(180deg) }`
    /// is the one place the reference turns a mark, and it is how the stepper's up tick is drawn.
    pub turn: f32,
}

impl Mark {
    pub fn new(icon: Icon, size: f32) -> Self {
        Self { icon, size, stroke: DEFAULT_STROKE, turn: 0.0 }
    }

    pub fn stroke(mut self, stroke: f32) -> Self {
        self.stroke = stroke;
        self
    }

    pub fn turn(mut self, radians: f32) -> Self {
        self.turn = radians;
        self
    }

    /// Half a turn, which is what a chevron pointing the other way is.
    pub fn flipped(self) -> Self {
        self.turn(std::f32::consts::PI)
    }

    /// The square this mark occupies.
    pub fn size(self) -> Vec2 {
        Vec2::splat(self.size)
    }
}

/// Paint a mark centred in `rect`, tinted.
///
/// The texture is white with an alpha mask, and `epaint` multiplies a mesh's vertex colour by the
/// texture — so the tint is exact and costs nothing.
pub fn paint(
    painter: &egui::Painter,
    atlas: &mut IconAtlas,
    mark: Mark,
    centre: Pos2,
    colour: Color32,
) {
    let scale = crate::render::composited_scale(painter.ctx(), painter.layer_id());
    let Some((texture, pixels)) =
        atlas.texture(painter.ctx(), mark.icon, mark.size, mark.stroke, scale)
    else {
        return;
    };
    // The texture is the mark plus a pixel of slack each way, so the rectangle it is drawn into is
    // proportionally larger than the mark's own size.
    let side = mark.size * f32::from(pixels + PAD * 2) / f32::from(pixels);
    let rect = Rect::from_center_size(centre, Vec2::splat(side));
    let uv = Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0));
    if mark.turn == 0.0 {
        painter.add(egui::Shape::image(texture, rect, uv, colour));
        return;
    }
    // A turned mark cannot be an image shape, whose rectangle is axis-aligned, so the four corners are
    // rotated about the centre by hand. The texture coordinates stay where they are, which is what
    // makes this a rotation of the picture rather than of the sampling.
    let mut mesh = Mesh::with_texture(texture);
    let (sin, cos) = mark.turn.sin_cos();
    let corners = [rect.left_top(), rect.right_top(), rect.right_bottom(), rect.left_bottom()];
    let uvs = [uv.left_top(), uv.right_top(), uv.right_bottom(), uv.left_bottom()];
    for (corner, uv) in corners.into_iter().zip(uvs) {
        let offset = corner - centre;
        mesh.vertices.push(textured_vertex(
            centre + Vec2::new(offset.x * cos - offset.y * sin, offset.x * sin + offset.y * cos),
            uv,
            colour,
        ));
    }
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    painter.add(egui::Shape::mesh(mesh));
}

/// A vertex carrying a texture coordinate.
///
/// **`Mesh::colored_vertex` may not be used on a mesh that has a texture, and `epaint` says so with a
/// `debug_assert`.** Both turned marks here used to call it and then write the texture coordinate over
/// the vertex afterwards, which produced the right picture in a release build and killed the process in
/// a debug one:
///
/// ```text
/// thread '<unnamed>' panicked at epaint-0.36.2/src/mesh.rs:170:
/// Mesh has an assigned texture
/// ```
///
/// Nothing found it for as long as every build that ran was a release build. The first debug build that
/// drew a turned chevron — the Android application's first frame, where a select, a stepper and a
/// spinner all draw one — died before it reached the screen.
fn textured_vertex(pos: Pos2, uv: Pos2, colour: Color32) -> egui::epaint::Vertex {
    egui::epaint::Vertex { pos, uv, color: colour }
}

/// One icon part as a path in the viewBox's own units.
fn path_of(shape: Shape) -> BezPath {
    match shape {
        Shape::Path(d) => BezPath::from_svg(d).unwrap_or_default(),
        Shape::Circle { cx, cy, r } => {
            Circle::new((f64::from(cx), f64::from(cy)), f64::from(r)).to_path(0.01)
        }
        Shape::Rect { x, y, w, h, rx } => RoundedRect::new(
            f64::from(x),
            f64::from(y),
            f64::from(x + w),
            f64::from(y + h),
            f64::from(rx),
        )
        .to_path(0.01),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_icon_s_path_data_parses() {
        // The path strings are copied from the reference verbatim, so the thing that can go wrong is a
        // transcription error — a missing quote, a dropped command letter. This is that check.
        for icon in Icon::ALL {
            for part in icon.parts() {
                if let Shape::Path(d) = part.shape {
                    let path = BezPath::from_svg(d)
                        .unwrap_or_else(|error| panic!("{}: {d:?}: {error}", icon.name()));
                    assert!(!path.elements().is_empty(), "{} drew nothing", icon.name());
                }
            }
        }
    }

    #[test]
    fn every_mark_stays_inside_its_own_view_box() {
        // A mark whose ink reaches past `0 0 24 24` would be clipped by the texture it is rasterised
        // into. The stroke is centred on the path, so half of the widest stroke the library uses is the
        // slack that has to fit — which is what `PAD` is for, and this is the check that one pixel is
        // enough.
        let widest = 2.4_f64 / 2.0;
        for icon in Icon::ALL {
            for part in icon.parts() {
                let bounds = path_of(part.shape).bounding_box();
                let margin = if part.stroke { widest } else { 0.0 };
                assert!(
                    bounds.x0 - margin >= -0.6
                        && bounds.y0 - margin >= -0.6
                        && bounds.x1 + margin <= 24.6
                        && bounds.y1 + margin <= 24.6,
                    "{} reaches {bounds:?}",
                    icon.name()
                );
            }
        }
    }

    #[test]
    fn a_mark_carries_the_reference_s_own_props() {
        let mark = Mark::new(Icon::ChevDown, 9.0).stroke(2.4);
        assert_eq!(mark.size, 9.0);
        assert_eq!(mark.stroke, 2.4);
        assert_eq!(Mark::new(Icon::ChevDown, 13.0).stroke, DEFAULT_STROKE);
        assert_eq!(mark.flipped().turn, std::f32::consts::PI);
    }
}
