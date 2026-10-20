//! A layer's decoration as `egui` meshes, with every shadow a small cached sprite (`task-2288`).
//!
//! [`super::Canvas`] rasterises a layer's whole decoration with `vello_cpu` into one texture the size of the
//! layer, on the window's thread, every time anything in it changes. Unluminous measured what that costs a
//! person: opening its Settings window spent 62 to 77 ms rasterising, and a pane appearing 32 to 44 ms, all
//! of it on the thread that draws the window, and 2.3 times as long again when the machine is busy and the
//! thread is moved to an efficient core. Nearly every pixel of it was a flat fill or the middle of a shadow
//! whose every row is the same as the one beside it.
//!
//! So a layer is drawn the way `egui` draws everything else, as meshes:
//!
//! - **A fill is a polygon** with `egui`'s own one pixel feather round its edge: a rounded rectangle with each
//!   corner its own radius, a disc, a ring, a line. A two stop linear gradient is a colour on each vertex,
//!   which is exact, because the colour of such a gradient is affine in position and the GPU interpolates
//!   vertex colours affinely. Nothing about a fill is rasterised on the processor.
//! - **A shadow is a nine patch.** A blurred rounded rectangle is the same along each edge once it is far
//!   enough from the corners, so it is rasterised once, at a size just large enough to hold its four corners
//!   and a band of edge between them, and drawn at any size by stretching the band. It is rasterised by
//!   `vello_cpu`'s own blurred rectangle, the call the canvas makes, so its pixels are the canvas's. It is
//!   rasterised in white and tinted by the vertex colour, which is the same premultiplied arithmetic, so one
//!   sprite serves every colour of shadow with that shape. It is kept while it is used and
//!   [`super::canvas::KEEP_UNUSED`] after.
//!
//! A layer this cannot draw exactly answers `None`, and the caller rasterises it on the canvas as it always
//! did: an arc (a spinner), an inset shadow outside a carved surface, and a rounded clip that something
//! inside it reaches the corners of.

use std::collections::HashMap;

use egui::epaint::{Mesh, Vertex, WHITE_UV};
use egui::{Color32, Pos2, Rect, Vec2};
use vello_cpu::kurbo::{Affine, RoundedRect, RoundedRectRadii, Shape as _};
use vello_cpu::peniko::color::{AlphaColor, Srgb};
use vello_cpu::{
    CompositeMode, PixelFormat, Pixmap, RasterizerSettings, RenderContext, RenderMode,
    RenderSettings, Resources,
};

use super::decor::{Corners, Decor, Fill};

/// How many pixels of constant edge a sprite keeps between its corners, to be stretched.
const BAND: u32 = 2;

/// Values in a sprite's key are kept to a quarter of a pixel, so a shadow that moves by less than that, or
/// is a hair different in size, is drawn from the same sprite.
fn quarter(value: f32) -> i32 {
    (value * 4.0).round() as i32
}

/// What a sprite is the picture of, in device pixels, to a quarter of a pixel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Key {
    /// A blurred rounded rectangle, outside its shape. `width` and `height` are `None` when the sprite is
    /// stretched along that side, and the side's own length when it is too short to be.
    Outer {
        radius: i32,
        deviation: i32,
        width: Option<i32>,
        height: Option<i32>,
    },
    /// The inside of a carved surface: an inverted blurred rounded rectangle cut to the surface. `gaps` is how
    /// far the shadow's rectangle is inside the surface's on each side (left, top, right, bottom), which is its
    /// offset and its spread.
    Inset {
        clip: [i32; 4],
        radius: i32,
        deviation: i32,
        gaps: [i32; 4],
        width: Option<i32>,
        height: Option<i32>,
    },
}

/// One rasterised sprite: its texture, how big it is, and where the stretchable band is in it.
struct Sprite {
    texture: egui::TextureHandle,
    /// The picture's size in pixels.
    size: [u32; 2],
    /// Pixels from each side of the picture to the band, along each axis: left, top, right, bottom. A side
    /// that is not stretched has its whole length here and no band.
    fixed: [u32; 4],
    /// The frame it was last drawn on and when, for [`Sprites::tidy`].
    used: u64,
    used_at: std::time::Instant,
    /// The pixels, kept in a test so the picture a mesh makes can be worked out with no graphics card.
    #[cfg(test)]
    pixels: Vec<Color32>,
}

/// Every sprite the window has made, and the context that rasterises them.
pub struct Sprites {
    by_key: HashMap<Key, Sprite>,
    context: RenderContext,
    resources: Resources,
    frame: u64,
}

impl std::fmt::Debug for Sprites {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        out.debug_struct("Sprites")
            .field("sprites", &self.by_key.len())
            .finish()
    }
}

impl Default for Sprites {
    fn default() -> Self {
        Self::new(RenderSettings::default())
    }
}

impl Sprites {
    /// A store whose sprites rasterise at the SIMD level `settings` names: the machine's in a window, the
    /// target's baseline in a test.
    pub fn new(settings: RenderSettings) -> Self {
        Self {
            by_key: HashMap::new(),
            context: RenderContext::new_with(1, 1, settings),
            resources: Resources::default(),
            frame: 0,
        }
    }

    /// How many sprites are kept and how many pixels they hold, for a measurement pass.
    pub fn footprint(&self) -> (usize, usize) {
        let pixels = self
            .by_key
            .values()
            .map(|sprite| (sprite.size[0] * sprite.size[1]) as usize)
            .sum();
        (self.by_key.len(), pixels)
    }

    /// Forget the sprites not drawn for [`super::canvas::KEEP_UNUSED`]. Called once a frame.
    pub fn tidy(&mut self) {
        let frame = self.frame;
        self.by_key.retain(|_, sprite| {
            sprite.used == frame || sprite.used_at.elapsed() < super::canvas::KEEP_UNUSED
        });
        self.frame = self.frame.wrapping_add(1);
    }

    /// The sprite for `key`, rasterised now if it has not been yet.
    fn sprite(&mut self, ctx: &egui::Context, key: Key) -> &Sprite {
        let frame = self.frame;
        if !self.by_key.contains_key(&key) {
            let started = std::time::Instant::now();
            let (size, fixed, pixels) = self.rasterise(key);
            // **The size the picture was really drawn at.** `draw` makes a side of nothing one pixel, so a
            // surface squeezed to no width came back as one column of pixels under a size of nothing, and the
            // renderer refused the texture and ended the process (Unluminous `task-2317`).
            let size = size.map(|side| side.clamp(1, 4096));
            super::cost::rasterised(started, u64::from(size[0]) * u64::from(size[1]));
            #[cfg(test)]
            let kept = pixels.clone();
            let image = egui::ColorImage::new([size[0] as usize, size[1] as usize], pixels);
            let texture = ctx.load_texture(
                format!("rux-sprite-{key:?}"),
                image,
                egui::TextureOptions::LINEAR,
            );
            self.by_key.insert(
                key,
                Sprite {
                    texture,
                    size,
                    fixed,
                    used: frame,
                    used_at: std::time::Instant::now(),
                    #[cfg(test)]
                    pixels: kept,
                },
            );
        }
        let sprite = self.by_key.get_mut(&key).expect("it was just made");
        sprite.used = frame;
        sprite.used_at = std::time::Instant::now();
        sprite
    }

    /// Draw a sprite's shape in white, and answer its size, where its band is, and its pixels.
    fn rasterise(&mut self, key: Key) -> ([u32; 2], [u32; 4], Vec<Color32>) {
        let px = |value: i32| f64::from(value) / 4.0;
        match key {
            Key::Outer {
                radius,
                deviation,
                width,
                height,
            } => {
                let reach = reach(px(deviation));
                let corner = corner(px(radius), px(deviation), 0.0);
                let across = width.map_or(f64::from(2 * corner + BAND), px);
                let down = height.map_or(f64::from(2 * corner + BAND), px);
                let size = [
                    (across + 2.0 * f64::from(reach)).ceil() as u32,
                    (down + 2.0 * f64::from(reach)).ceil() as u32,
                ];
                let shape = vello_cpu::kurbo::Rect::new(
                    f64::from(reach),
                    f64::from(reach),
                    f64::from(reach) + across,
                    f64::from(reach) + down,
                );
                let pixels = self.draw(size, |context| {
                    context.fill_blurred_rounded_rect(
                        &shape,
                        px(radius).max(0.0) as f32,
                        px(deviation).max(0.01) as f32,
                        false,
                    );
                });
                let side = reach + corner;
                let fixed = [
                    width.map_or(side, |_| size[0]),
                    height.map_or(side, |_| size[1]),
                    width.map_or(size[0] - side - BAND, |_| 0),
                    height.map_or(size[1] - side - BAND, |_| 0),
                ];
                (size, fixed, pixels)
            }
            Key::Inset {
                clip,
                radius,
                deviation,
                gaps,
                width,
                height,
            } => {
                let widest = gaps.iter().map(|gap| px(*gap).abs()).fold(0.0, f64::max);
                let rounder = clip
                    .iter()
                    .map(|corner| px(*corner))
                    .fold(px(radius), f64::max);
                let corner = corner(rounder, px(deviation), widest);
                let across = width.map_or(f64::from(2 * corner + BAND), px);
                let down = height.map_or(f64::from(2 * corner + BAND), px);
                let size = [across.ceil() as u32, down.ceil() as u32];
                let surface = vello_cpu::kurbo::Rect::new(0.0, 0.0, across, down);
                let shadow = vello_cpu::kurbo::Rect::new(
                    px(gaps[0]),
                    px(gaps[1]),
                    across - px(gaps[2]),
                    down - px(gaps[3]),
                );
                let most = across.min(down) / 2.0;
                let radii = RoundedRectRadii::new(
                    px(clip[0]).clamp(0.0, most),
                    px(clip[1]).clamp(0.0, most),
                    px(clip[2]).clamp(0.0, most),
                    px(clip[3]).clamp(0.0, most),
                );
                let cut = RoundedRect::from_rect(surface, radii).to_path(0.1);
                let pixels = self.draw(size, |context| {
                    context.push_clip_path(&cut);
                    context.fill_blurred_rounded_rect(
                        &shadow,
                        px(radius).max(0.0) as f32,
                        px(deviation).max(0.01) as f32,
                        true,
                    );
                    context.pop_clip_path();
                });
                let fixed = [
                    width.map_or(corner, |_| size[0]),
                    height.map_or(corner, |_| size[1]),
                    width.map_or(size[0].saturating_sub(corner + BAND), |_| 0),
                    height.map_or(size[1].saturating_sub(corner + BAND), |_| 0),
                ];
                (size, fixed, pixels)
            }
        }
    }

    /// Rasterise one picture of `size` with white paint, and answer its pixels as `egui`'s colours.
    fn draw(&mut self, size: [u32; 2], paint: impl FnOnce(&mut RenderContext)) -> Vec<Color32> {
        let (width, height) = (size[0].clamp(1, 4096) as u16, size[1].clamp(1, 4096) as u16);
        let mut pixmap = Pixmap::new(width, height);
        self.context.reset_and_resize(width, height);
        self.context.set_transform(Affine::IDENTITY);
        self.context
            .set_paint(AlphaColor::<Srgb>::from_rgba8(255, 255, 255, 255));
        paint(&mut self.context);
        self.context.flush();
        self.context.render_with(
            pixmap.as_mut(),
            &mut self.resources,
            RasterizerSettings {
                render_mode: RenderMode::OptimizeSpeed,
                composite_mode: CompositeMode::Replace,
                pixel_format: PixelFormat::Rgba8,
                offset: (0, 0),
            },
        );
        pixmap
            .data_as_u8_slice()
            .chunks_exact(4)
            .map(|pixel| Color32::from_rgba_premultiplied(pixel[0], pixel[1], pixel[2], pixel[3]))
            .collect()
    }
}

/// How far past a blurred rectangle's edge its blur reaches, in whole pixels: three deviations, past which
/// a Gaussian is under a third of a percent.
fn reach(deviation: f64) -> u32 {
    (3.0 * deviation).ceil() as u32
}

/// How far from a sprite's edge its corners reach before the edge is the same all along: the corner's radius,
/// the blur's reach, and for an inset shadow how far its rectangle is inside the surface's, plus a pixel.
fn corner(radius: f64, deviation: f64, gap: f64) -> u32 {
    (radius.max(0.0) + 3.0 * deviation + gap).ceil() as u32 + 1
}

/// A layer's decoration as shapes, or `None` when one of its items is one this cannot draw exactly.
///
/// `scale` is the pixels a point the layer is composited at (`render::composited_scale`), which is what a
/// sprite is rasterised at and what a feather is a pixel of.
///
/// @param ctx - the context, which the sprites are uploaded through
/// @param sprites - the window's sprites
/// @param scale - pixels a point
/// @param items - the decoration, in the order it was recorded
/// @param area - what the decoration may cover, as the canvas cuts it
pub fn compose(
    ctx: &egui::Context,
    sprites: &mut Sprites,
    scale: f32,
    items: &[Decor],
    area: Rect,
) -> Option<Vec<egui::Shape>> {
    let plan = plan(items)?;
    let mut shapes = Vec::with_capacity(items.len());
    for step in plan {
        let mesh = match step.item {
            Decor::Rect {
                rect,
                corners,
                fill,
            } => fill_mesh(&rounded_outline(rect, corners, scale), fill, scale),
            Decor::Disc {
                centre,
                radius,
                fill,
            } => fill_mesh(&circle_outline(centre, radius, scale), fill, scale),
            Decor::Ring {
                centre,
                radius,
                width,
                colour,
            } => ring_mesh(centre, radius, width, colour, scale),
            Decor::Line {
                from,
                to,
                width,
                colour,
            } => line_mesh(from, to, width, colour, scale),
            Decor::Arc {
                centre,
                radius,
                width,
                start,
                sweep,
                fill,
                round,
            } => arc_mesh(centre, radius, width, start, sweep, fill, round, scale),
            Decor::Shadow {
                rect,
                radius,
                deviation,
                colour,
                inset: false,
            } => outer_shadow(ctx, sprites, rect, radius, deviation, colour, scale),
            Decor::Shadow {
                rect,
                radius,
                deviation,
                colour,
                inset: true,
            } => {
                let (surface, corners) = step.carved?;
                inset_shadow(
                    ctx, sprites, surface, corners, rect, radius, deviation, colour, scale,
                )
            }
            _ => continue,
        };
        let bound = step.cut.map_or(area, |cut| cut.intersect(area));
        let mesh = match bound.contains_rect(step.item.bounds()) {
            true => mesh,
            false => super::glass::clip_to_rect(&mesh, bound),
        };
        if !mesh.is_empty() {
            shapes.push(egui::Shape::mesh(mesh));
        }
    }
    Some(shapes)
}

/// One item to draw, with what the clips round it mean for it.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Step {
    item: Decor,
    /// The rectangle the clips open round it cut it to, when there are any.
    cut: Option<Rect>,
    /// For an inset shadow, the carved surface it is cut to and that surface's corners.
    carved: Option<(Rect, Corners)>,
}

/// Work out what each item needs from the clips round it, or `None` when a clip asks for something meshes
/// cannot do. Split from [`compose`] so a test can read the plan with no context and no sprites.
fn plan(items: &[Decor]) -> Option<Vec<Step>> {
    #[derive(Clone, Copy)]
    enum Open {
        /// A clip that only cuts a shadow from under the opaque surface drawn straight after it, which the
        /// surface covers anyway. Nothing is cut.
        Covered,
        /// The inside of a carved surface, for its inset shadows.
        Carved(Rect, Corners),
        /// A rounded rectangle everything inside is cut to.
        General(Rect, Corners),
    }
    let mut open: Vec<Open> = Vec::new();
    let mut steps = Vec::with_capacity(items.len());
    for (at, item) in items.iter().enumerate() {
        match *item {
            Decor::Glass { .. } => return None,
            // A round cap is a disc laid over the stroke's end, which only looks right when neither shows
            // through the other.
            Decor::Arc {
                fill, round: true, ..
            } if !fill.is_opaque() => return None,
            Decor::Clip {
                rect,
                corners,
                outside: true,
                ..
            } => {
                if !covered_after(items, at, rect, corners) {
                    return None;
                }
                open.push(Open::Covered);
            }
            Decor::Clip {
                rect,
                corners,
                outside: false,
                ..
            } => {
                let carved = items[..at].last().is_some_and(|before| {
                    matches!(*before, Decor::Rect { rect: surface, .. } if surface == rect)
                }) && only_inset_shadows_until_unclip(items, at);
                open.push(match carved {
                    true => Open::Carved(rect, corners),
                    false => Open::General(rect, corners),
                });
            }
            Decor::Unclip => {
                open.pop();
            }
            drawn => {
                let mut cut: Option<Rect> = None;
                let mut carved = None;
                for clip in &open {
                    match *clip {
                        Open::Covered => {}
                        Open::Carved(rect, corners) => carved = Some((rect, corners)),
                        Open::General(rect, corners) => {
                            if reaches_a_corner(drawn.bounds(), rect, corners) {
                                return None;
                            }
                            cut = Some(cut.map_or(rect, |bound| bound.intersect(rect)));
                        }
                    }
                }
                if matches!(drawn, Decor::Shadow { inset: true, .. }) && carved.is_none() {
                    return None;
                }
                steps.push(Step {
                    item: drawn,
                    cut: cut.filter(|bound| !bound.contains_rect(drawn.bounds())),
                    carved,
                });
            }
        }
    }
    Some(steps)
}

/// Whether the clip opened at `at` cuts only shadows, closes, and is followed by an opaque surface that covers
/// the clip's shape, which is what `Chrome::surface` records for an opaque raised surface. Unluminous's own
/// raised surface puts a hairline a little larger than the surface first, which covers it as well.
fn covered_after(items: &[Decor], at: usize, rect: Rect, corners: Corners) -> bool {
    let mut next = at + 1;
    while let Some(Decor::Shadow { inset: false, .. }) = items.get(next) {
        next += 1;
    }
    if items.get(next) != Some(&Decor::Unclip) {
        return false;
    }
    let Some(Decor::Rect {
        rect: surface,
        corners: round,
        fill,
    }) = items.get(next + 1)
    else {
        return false;
    };
    // A rounded rectangle grown by `grown` on every side covers the one inside it when no corner of it is
    // rounder than the inner corner grown by the same amount.
    let grown = (rect.min.x - surface.min.x)
        .min(rect.min.y - surface.min.y)
        .min(surface.max.x - rect.max.x)
        .min(surface.max.y - rect.max.y);
    let fits = |outer: f32, inner: f32| outer <= inner + grown + 0.01;
    fill.is_opaque()
        && grown >= -0.01
        && fits(round.nw, corners.nw)
        && fits(round.ne, corners.ne)
        && fits(round.se, corners.se)
        && fits(round.sw, corners.sw)
}

/// Whether the clip opened at `at` holds nothing but inset shadows before it closes.
fn only_inset_shadows_until_unclip(items: &[Decor], at: usize) -> bool {
    let mut next = at + 1;
    let mut any = false;
    while let Some(Decor::Shadow { inset: true, .. }) = items.get(next) {
        next += 1;
        any = true;
    }
    any && items.get(next) == Some(&Decor::Unclip)
}

/// Whether something with these bounds reaches into a rounded corner of a clip, where a rectangle would not
/// cut it as the rounded clip does.
fn reaches_a_corner(bounds: Rect, clip: Rect, corners: Corners) -> bool {
    let squares = [
        (
            corners.nw,
            Rect::from_min_size(clip.left_top(), Vec2::splat(corners.nw)),
        ),
        (
            corners.ne,
            Rect::from_min_size(
                clip.right_top() - Vec2::new(corners.ne, 0.0),
                Vec2::splat(corners.ne),
            ),
        ),
        (
            corners.se,
            Rect::from_min_size(
                clip.right_bottom() - Vec2::splat(corners.se),
                Vec2::splat(corners.se),
            ),
        ),
        (
            corners.sw,
            Rect::from_min_size(
                clip.left_bottom() - Vec2::new(0.0, corners.sw),
                Vec2::splat(corners.sw),
            ),
        ),
    ];
    squares
        .iter()
        .any(|(radius, square)| *radius > 0.0 && square.intersects(bounds))
}

/// The outline of a rounded rectangle, clockwise from the top left, with each corner its own radius clamped to
/// half the shorter side, as the canvas clamps it.
fn rounded_outline(rect: Rect, corners: Corners, scale: f32) -> Vec<Pos2> {
    let most = (rect.width().min(rect.height()) / 2.0).max(0.0);
    let one = |radius: f32| radius.clamp(0.0, most);
    let mut points = Vec::new();
    let mut corner = |centre: Pos2, radius: f32, from: f32| {
        if radius <= 0.0 {
            points.push(centre);
            return;
        }
        let steps = segments(radius, scale, std::f32::consts::FRAC_PI_2);
        for step in 0..=steps {
            let angle = from + std::f32::consts::FRAC_PI_2 * step as f32 / steps as f32;
            points.push(centre + radius * Vec2::angled(angle));
        }
    };
    use std::f32::consts::PI;
    let (nw, ne, se, sw) = (
        one(corners.nw),
        one(corners.ne),
        one(corners.se),
        one(corners.sw),
    );
    corner(rect.left_top() + Vec2::splat(nw), nw, PI);
    corner(rect.right_top() + Vec2::new(-ne, ne), ne, 1.5 * PI);
    corner(rect.right_bottom() - Vec2::splat(se), se, 0.0);
    corner(rect.left_bottom() + Vec2::new(sw, -sw), sw, 0.5 * PI);
    points
}

/// The outline of a circle, clockwise.
fn circle_outline(centre: Pos2, radius: f32, scale: f32) -> Vec<Pos2> {
    let steps = segments(radius, scale, std::f32::consts::TAU).max(8);
    (0..steps)
        .map(|step| {
            centre + radius * Vec2::angled(std::f32::consts::TAU * step as f32 / steps as f32)
        })
        .collect()
}

/// How many straight pieces an arc of `radius` points through `sweep` radians is drawn with: one every two
/// pixels of its length, which is closer than a pixel to the true curve at any radius here.
fn segments(radius: f32, scale: f32, sweep: f32) -> usize {
    ((radius * scale * sweep / 2.0).ceil() as usize).clamp(2, 256)
}

/// The colour of a fill at a point: the colour, or the gradient's colour where the point projects onto its
/// axis, both premultiplied, which is how the canvas interpolates.
fn colour_at(fill: Fill, at: Pos2) -> Color32 {
    match fill {
        Fill::Solid(colour) => colour,
        Fill::Linear {
            from,
            to,
            start,
            end,
        } => {
            let axis = to - from;
            let length = axis.length_sq();
            let t = if length > 0.0 {
                ((at - from).dot(axis) / length).clamp(0.0, 1.0)
            } else {
                0.0
            };
            start.lerp_to_gamma(end, t)
        }
    }
}

/// The mesh of a convex outline filled with `fill`, with a feather one pixel wide across its edge.
fn fill_mesh(outline: &[Pos2], fill: Fill, scale: f32) -> Mesh {
    let mut mesh = Mesh::default();
    let outline = without_repeats(outline);
    let count = outline.len();
    if count < 3 {
        return mesh;
    }
    let half = 0.5 / scale;
    let normals = normals(&outline);
    // An inner ring at full colour and an outer ring transparent, half a pixel either side of the edge, which
    // is the feather `egui` gives every shape it fills.
    for (point, normal) in outline.iter().zip(&normals) {
        let inner = *point - *normal * half;
        mesh.vertices.push(Vertex {
            pos: inner,
            uv: WHITE_UV,
            color: colour_at(fill, inner),
        });
        mesh.vertices.push(Vertex {
            pos: *point + *normal * half,
            uv: WHITE_UV,
            color: Color32::TRANSPARENT,
        });
    }
    for at in 0..count as u32 {
        let next = (at + 1) % count as u32;
        let (inner, outer, inner_next, outer_next) = (2 * at, 2 * at + 1, 2 * next, 2 * next + 1);
        mesh.indices
            .extend([inner, outer, outer_next, inner, outer_next, inner_next]);
    }
    for at in 1..count as u32 - 1 {
        mesh.indices.extend([0, 2 * at, 2 * (at + 1)]);
    }
    mesh
}

/// A ring of `width` round `centre`, feathered on both sides.
fn ring_mesh(centre: Pos2, radius: f32, width: f32, colour: Color32, scale: f32) -> Mesh {
    let steps = segments(radius + width / 2.0, scale, std::f32::consts::TAU).max(8);
    let half = 0.5 / scale;
    let radii = [
        radius - width / 2.0 - half,
        radius - width / 2.0 + half,
        radius + width / 2.0 - half,
        radius + width / 2.0 + half,
    ];
    let mut mesh = Mesh::default();
    let thin = width < 2.0 / scale;
    for step in 0..steps {
        let direction = Vec2::angled(std::f32::consts::TAU * step as f32 / steps as f32);
        for (ring, at) in radii.iter().enumerate() {
            // A stroke thinner than its feathers fades its colour rather than drawing two edges that overlap.
            let shown = if thin {
                colour.gamma_multiply((width * scale / 2.0).min(1.0))
            } else {
                colour
            };
            let colour = if ring == 0 || ring == 3 {
                Color32::TRANSPARENT
            } else {
                shown
            };
            mesh.vertices.push(Vertex {
                pos: centre + direction * at.max(0.0),
                uv: WHITE_UV,
                color: colour,
            });
        }
    }
    let steps = steps as u32;
    for step in 0..steps {
        let next = (step + 1) % steps;
        for ring in 0..3u32 {
            let (a, b) = (4 * step + ring, 4 * step + ring + 1);
            let (c, d) = (4 * next + ring, 4 * next + ring + 1);
            mesh.indices.extend([a, b, d, a, d, c]);
        }
    }
    mesh
}

/// An arc of a circle stroked `width` wide, feathered on both sides, from `start` through `sweep` radians
/// clockwise from twelve o'clock, as `Decor::Arc` measures them. A gradient colours each vertex where it is. A
/// round cap is a disc at each end, which is why a translucent arc with round caps is left to the canvas.
#[allow(clippy::too_many_arguments)]
fn arc_mesh(
    centre: Pos2,
    radius: f32,
    width: f32,
    start: f32,
    sweep: f32,
    fill: Fill,
    round: bool,
    scale: f32,
) -> Mesh {
    let mut mesh = Mesh::default();
    let steps = segments(radius + width / 2.0, scale, sweep.abs()).max(2);
    let half = 0.5 / scale;
    let offsets = [
        -width / 2.0 - half,
        -width / 2.0 + half,
        width / 2.0 - half,
        width / 2.0 + half,
    ];
    let thin = width < 2.0 / scale;
    let fade = (width * scale / 2.0).min(1.0);
    let point_at = |angle: f32| Vec2::new(angle.sin(), -angle.cos());
    for step in 0..=steps {
        let direction = point_at(start + sweep * step as f32 / steps as f32);
        for (ring, offset) in offsets.iter().enumerate() {
            let pos = centre + direction * (radius + offset).max(0.0);
            let shown = colour_at(fill, pos);
            let shown = if thin {
                shown.gamma_multiply(fade)
            } else {
                shown
            };
            let colour = if ring == 0 || ring == 3 {
                Color32::TRANSPARENT
            } else {
                shown
            };
            mesh.vertices.push(Vertex {
                pos,
                uv: WHITE_UV,
                color: colour,
            });
        }
    }
    for step in 0..steps as u32 {
        for ring in 0..3u32 {
            let (a, b) = (4 * step + ring, 4 * step + ring + 1);
            let (c, d) = (4 * (step + 1) + ring, 4 * (step + 1) + ring + 1);
            mesh.indices.extend([a, b, d, a, d, c]);
        }
    }
    if round {
        for angle in [start, start + sweep] {
            let end = centre + point_at(angle) * radius;
            mesh.append(fill_mesh(
                &circle_outline(end, width / 2.0, scale),
                Fill::Solid(colour_at(fill, end)),
                scale,
            ));
        }
    }
    mesh
}

/// A straight line of `width`, its ends squared off half its width past each end, feathered all round.
fn line_mesh(from: Pos2, to: Pos2, width: f32, colour: Color32, scale: f32) -> Mesh {
    let along = (to - from).normalized();
    if !along.is_finite() || along == Vec2::ZERO {
        return Mesh::default();
    }
    let across = along.rot90() * (width / 2.0);
    let (start, end) = (from - along * (width / 2.0), to + along * (width / 2.0));
    // Clockwise on the screen, where y runs down, as every outline here is: `across` is `along` turned a
    // quarter anticlockwise, so `start + across` is on the side the outline sets off along.
    let outline = [start + across, end + across, end - across, start - across];
    let shown = if width * scale < 1.0 {
        colour.gamma_multiply(width * scale)
    } else {
        colour
    };
    fill_mesh(&outline, Fill::Solid(shown), scale)
}

/// The points of an outline with any point that repeats the one before it left out.
fn without_repeats(outline: &[Pos2]) -> Vec<Pos2> {
    let mut points: Vec<Pos2> = Vec::with_capacity(outline.len());
    for point in outline {
        if points
            .last()
            .is_none_or(|last| (*last - *point).length_sq() > 1e-6)
        {
            points.push(*point);
        }
    }
    while points.len() > 1 && (points[0] - points[points.len() - 1]).length_sq() <= 1e-6 {
        points.pop();
    }
    points
}

/// The outward normal at each point of a clockwise convex outline, scaled so that moving each point along it
/// by a distance moves each edge by that distance.
fn normals(outline: &[Pos2]) -> Vec<Vec2> {
    let count = outline.len();
    (0..count)
        .map(|at| {
            let before = outline[(at + count - 1) % count];
            let after = outline[(at + 1) % count];
            let point = outline[at];
            // Clockwise in screen coordinates, where y runs down, so the outward normal of an edge is its
            // direction turned a quarter anticlockwise on the screen, which is egui's `rot90`: it turns
            // `(1, 0)` into `(0, -1)`.
            let first = (point - before).normalized().rot90();
            let second = (after - point).normalized().rot90();
            let sum = first + second;
            let length = sum.length_sq();
            if length < 1e-6 {
                first
            } else {
                // The miter: the normal whose projection onto each edge's normal is one.
                (sum * (2.0 / length)).clamp(Vec2::splat(-4.0), Vec2::splat(4.0))
            }
        })
        .collect()
}

/// An outer shadow as a nine patch.
fn outer_shadow(
    ctx: &egui::Context,
    sprites: &mut Sprites,
    rect: Rect,
    radius: f32,
    deviation: f32,
    colour: Color32,
    scale: f32,
) -> Mesh {
    let most = rect.width().min(rect.height()) / 2.0;
    let (radius, deviation) = (
        quarter(radius.clamp(0.0, most.max(0.0)) * scale),
        quarter(deviation.max(0.01) * scale),
    );
    // Measured from the quarter pixel values the sprite is rasterised from, so the two agree on where the band is.
    let corner = corner(f64::from(radius) / 4.0, f64::from(deviation) / 4.0, 0.0);
    let stretched = |length: f32| length * scale >= (2 * corner + BAND) as f32;
    let key = Key::Outer {
        radius,
        deviation,
        width: (!stretched(rect.width())).then(|| quarter(rect.width() * scale)),
        height: (!stretched(rect.height())).then(|| quarter(rect.height() * scale)),
    };
    let reach = reach(f64::from(deviation) / 4.0) as f32 / scale;
    let target = rect.expand(reach);
    let sprite = sprites.sprite(ctx, key);
    nine_patch(sprite, target, scale, colour, true)
}

/// An inset shadow, cut to the carved surface it is in, as a nine patch with no middle.
#[allow(clippy::too_many_arguments)]
fn inset_shadow(
    ctx: &egui::Context,
    sprites: &mut Sprites,
    surface: Rect,
    corners: Corners,
    rect: Rect,
    radius: f32,
    deviation: f32,
    colour: Color32,
    scale: f32,
) -> Mesh {
    // A carved surface with no width or no height has no inside to shade.
    if surface.width() * scale < 1.0 || surface.height() * scale < 1.0 {
        return Mesh::default();
    }
    let gaps = [
        rect.min.x - surface.min.x,
        rect.min.y - surface.min.y,
        surface.max.x - rect.max.x,
        surface.max.y - rect.max.y,
    ]
    .map(|gap| quarter(gap * scale));
    let deviation = quarter(deviation.max(0.01) * scale);
    let radius = quarter(radius.max(0.0) * scale);
    let clip =
        [corners.nw, corners.ne, corners.se, corners.sw].map(|corner| quarter(corner * scale));
    // From the quarter pixel values, as `Sprites::rasterise` measures it.
    let widest = gaps
        .iter()
        .map(|gap| f64::from(*gap).abs() / 4.0)
        .fold(0.0, f64::max);
    let rounder = clip.iter().fold(radius, |most, corner| most.max(*corner));
    let reach_px = corner(f64::from(rounder) / 4.0, f64::from(deviation) / 4.0, widest);
    let stretched = |length: f32| length * scale >= (2 * reach_px + BAND) as f32;
    let key = Key::Inset {
        clip,
        radius,
        deviation,
        gaps,
        width: (!stretched(surface.width())).then(|| quarter(surface.width() * scale)),
        height: (!stretched(surface.height())).then(|| quarter(surface.height() * scale)),
    };
    let sprite = sprites.sprite(ctx, key);
    nine_patch(sprite, surface, scale, colour, false)
}

/// The quads that draw `sprite` over `target`: its corners at their own size, its edges stretched along them,
/// and its middle stretched both ways when `middle`, tinted `colour`.
fn nine_patch(sprite: &Sprite, target: Rect, scale: f32, colour: Color32, middle: bool) -> Mesh {
    let mut mesh = Mesh::with_texture(sprite.texture.id());
    let [width, height] = sprite.size.map(|side| side as f32);
    let [left, top, right, bottom] = sprite.fixed.map(|side| side as f32);
    // Where each column and row starts and ends, in the picture's pixels and on the screen.
    let columns = cuts(width, left, right, target.min.x, target.max.x, scale);
    let rows = cuts(height, top, bottom, target.min.y, target.max.y, scale);
    for (row, (v0, v1, y0, y1)) in rows.iter().enumerate() {
        for (column, (u0, u1, x0, x1)) in columns.iter().enumerate() {
            let centre = row == 1 && column == 1 && rows.len() == 3 && columns.len() == 3;
            if centre && !middle {
                continue;
            }
            let at = Rect::from_min_max(Pos2::new(*x0, *y0), Pos2::new(*x1, *y1));
            let uv = Rect::from_min_max(
                Pos2::new(u0 / width, v0 / height),
                Pos2::new(u1 / width, v1 / height),
            );
            if at.width() > 0.0 && at.height() > 0.0 {
                mesh.add_rect_with_uv(at, uv, colour);
            }
        }
    }
    mesh
}

/// The pieces one axis of a nine patch is cut into: for each, where it starts and ends in the picture and on
/// the screen. One piece when the axis is not stretched, three when it is.
fn cuts(
    size: f32,
    near: f32,
    far: f32,
    from: f32,
    to: f32,
    scale: f32,
) -> Vec<(f32, f32, f32, f32)> {
    if far <= 0.0 || near >= size {
        return vec![(0.0, size, from, to)];
    }
    let (near_points, far_points) = (near / scale, far / scale);
    vec![
        (0.0, near, from, from + near_points),
        (near, size - far, from + near_points, to - far_points),
        (size - far, size, to - far_points, to),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::Chrome;
    use crate::theme::light;
    use egui::{pos2, vec2};

    fn card() -> Rect {
        Rect::from_min_size(pos2(40.0, 30.0), vec2(300.0, 120.0))
    }

    /// An opaque raised surface and a carved one, as `Chrome::surface` records them, are drawn by meshes: the
    /// clip that cut the raised one's shadow is not needed, and the carved one's shadows are cut to it.
    #[test]
    fn the_surfaces_chrome_records_are_planned_without_the_canvas() {
        let theme = light();
        let chrome = Chrome::recording();
        chrome.surface(card(), 12.0, theme.surface.s1, theme.elevation.raised);
        chrome.surface(
            card().translate(vec2(0.0, 200.0)),
            12.0,
            theme.surface.s1,
            theme.elevation.pressed,
        );
        let plan = plan(&chrome.take()).expect("both surfaces can be drawn as meshes");
        let insets: Vec<_> = plan
            .iter()
            .filter(|step| matches!(step.item, Decor::Shadow { inset: true, .. }))
            .collect();
        assert!(!insets.is_empty());
        assert!(insets
            .iter()
            .all(|step| step.carved.map(|(rect, _)| rect)
                == Some(card().translate(vec2(0.0, 200.0)))));
        assert!(
            plan.iter().all(|step| step.cut.is_none()),
            "nothing here is cut by a rectangle"
        );
    }

    /// An opaque arc is a mesh, and a translucent one with round caps is left to the canvas, because its caps
    /// would show through its ends.
    #[test]
    fn a_translucent_round_capped_arc_is_left_to_the_canvas() {
        let chrome = Chrome::recording();
        chrome.arc(pos2(50.0, 50.0), 20.0, 3.0, 0.0, 1.0, Color32::WHITE);
        assert!(plan(&chrome.take()).is_some());
        chrome.arc(
            pos2(50.0, 50.0),
            20.0,
            3.0,
            0.0,
            1.0,
            Color32::from_white_alpha(128),
        );
        assert!(plan(&chrome.take()).is_none());
        chrome.arc_square(
            pos2(50.0, 50.0),
            20.0,
            3.0,
            0.0,
            1.0,
            Color32::from_white_alpha(128),
        );
        assert!(plan(&chrome.take()).is_some());
    }

    /// An arc drawn as a mesh is the canvas's arc except along its feathered edges.
    #[test]
    fn an_arc_drawn_as_a_mesh_is_the_canvas_s_arc() {
        let area = Rect::from_min_size(pos2(0.0, 0.0), vec2(160.0, 160.0));
        let gradient = Fill::Linear {
            from: pos2(20.0, 140.0),
            to: pos2(140.0, 20.0),
            start: Color32::from_rgb(61, 120, 255),
            end: Color32::from_rgb(92, 182, 255),
        };
        for (fill, round) in [
            (Fill::from(Color32::from_rgb(58, 64, 74)), true),
            (gradient, true),
            (gradient, false),
        ] {
            let items = [Decor::Arc {
                centre: pos2(80.0, 80.0),
                radius: 55.0,
                width: 10.0,
                start: 0.14,
                sweep: 4.2,
                fill,
                round,
            }];
            let ctx = egui::Context::default();
            let mut sprites = pinned();
            let shapes =
                compose(&ctx, &mut sprites, 1.5, &items, area).expect("an opaque arc is a mesh");
            let ([width, _], canvas) = super::super::canvas::Canvas::picture(area, 1.5, &items);
            let meshes = software(&shapes, &sprites, area, 1.5);
            let (most, mean, worst) = difference_at(&canvas, &meshes, width);
            assert!(
                most <= 160.0 && mean <= 0.5,
                "round {round}: largest {most} {worst}, mean {mean}"
            );
        }
    }

    /// A list in a rounded panel is cut to the panel by a rectangle while nothing in it reaches a corner, and
    /// left to the canvas once something does.
    #[test]
    fn a_rounded_clip_is_a_rectangle_until_something_reaches_its_corner() {
        let panel = Rect::from_min_size(pos2(0.0, 0.0), vec2(400.0, 300.0));
        let row = |y: f32| Decor::Rect {
            rect: Rect::from_min_size(pos2(40.0, y), vec2(300.0, 40.0)),
            corners: Corners::from(6.0),
            fill: Fill::from(Color32::GRAY),
        };
        let clip = Decor::Clip {
            rect: panel,
            corners: Corners::from(20.0),
            outside: false,
            bound: panel,
        };
        let inside =
            plan(&[clip, row(100.0), row(280.0), Decor::Unclip]).expect("no row reaches a corner");
        assert_eq!(
            inside[1].cut,
            Some(panel),
            "the row that crosses the bottom edge is cut to the panel"
        );
        assert_eq!(
            inside[0].cut, None,
            "the row wholly inside is left as it is"
        );
        let corner_row = Decor::Rect {
            rect: Rect::from_min_size(pos2(0.0, 280.0), vec2(300.0, 40.0)),
            corners: Corners::NONE,
            fill: Fill::from(Color32::GRAY),
        };
        assert!(plan(&[clip, corner_row, Decor::Unclip]).is_none());
    }

    /// A gradient's colour on a vertex is the gradient's colour at that point.
    #[test]
    fn a_gradient_is_coloured_by_where_each_vertex_is_on_its_axis() {
        let fill = Fill::Linear {
            from: pos2(0.0, 0.0),
            to: pos2(100.0, 0.0),
            start: Color32::BLACK,
            end: Color32::WHITE,
        };
        assert_eq!(colour_at(fill, pos2(-10.0, 0.0)), Color32::BLACK);
        assert_eq!(colour_at(fill, pos2(200.0, 50.0)), Color32::WHITE);
        let middle = colour_at(fill, pos2(50.0, 30.0));
        assert!((127..=128).contains(&middle.r()), "{middle:?}");
    }

    /// A fill's mesh covers its outline: every vertex is within a pixel of it and the inner ring has the colour.
    #[test]
    fn a_rounded_rectangle_mesh_lies_on_its_outline() {
        let rect = card();
        let mesh = fill_mesh(
            &rounded_outline(rect, Corners::from(12.0), 2.0),
            Fill::from(Color32::RED),
            2.0,
        );
        assert!(mesh
            .vertices
            .iter()
            .all(|vertex| rect.expand(0.5).contains(vertex.pos)));
        assert!(mesh
            .vertices
            .iter()
            .step_by(2)
            .all(|vertex| vertex.color == Color32::RED));
        assert!(mesh.is_valid());
    }

    /// Draw `shapes` the way a GPU would, for a picture of `rect` at `scale`: each triangle filled at the centre of
    /// each pixel, its colour and texture coordinates interpolated, the texture sampled linearly, and the result
    /// laid over what is there with premultiplied alpha.
    fn software(
        shapes: &[egui::Shape],
        sprites: &Sprites,
        rect: Rect,
        scale: f32,
    ) -> Vec<[f32; 4]> {
        let size = [
            (rect.width() * scale).round() as usize,
            (rect.height() * scale).round() as usize,
        ];
        let mut out = vec![[0.0f32; 4]; size[0] * size[1]];
        let textures: HashMap<egui::TextureId, &Sprite> = sprites
            .by_key
            .values()
            .map(|sprite| (sprite.texture.id(), sprite))
            .collect();
        for shape in shapes {
            let egui::Shape::Mesh(mesh) = shape else {
                panic!("compose answers meshes")
            };
            let texture = textures.get(&mesh.texture_id);
            for triangle in mesh.indices.chunks_exact(3) {
                let v = [0, 1, 2].map(|i| mesh.vertices[triangle[i] as usize]);
                let p = v.map(|vertex| ((vertex.pos - rect.min) * scale).to_pos2());
                let area = (p[1] - p[0]).x * (p[2] - p[0]).y - (p[1] - p[0]).y * (p[2] - p[0]).x;
                if area.abs() < 1e-9 {
                    continue;
                }
                // Wound the same way every time, so the fill rule below means the same thing for every triangle.
                let (p, v) = if area < 0.0 {
                    ([p[0], p[2], p[1]], [v[0], v[2], v[1]])
                } else {
                    (p, v)
                };
                let area = area.abs();
                // The top left rule a GPU uses: a pixel centre exactly on an edge belongs to the triangle only
                // when the edge is a top or a left one, so two triangles sharing an edge never both draw it.
                let owns = |a: Pos2, b: Pos2| {
                    let d = b - a;
                    (d.y == 0.0 && d.x < 0.0) || d.y > 0.0
                };
                let low = p
                    .iter()
                    .fold(Pos2::new(f32::MAX, f32::MAX), |a, b| a.min(*b));
                let high = p
                    .iter()
                    .fold(Pos2::new(f32::MIN, f32::MIN), |a, b| a.max(*b));
                for y in
                    (low.y.floor().max(0.0) as usize)..(high.y.ceil().min(size[1] as f32) as usize)
                {
                    for x in (low.x.floor().max(0.0) as usize)
                        ..(high.x.ceil().min(size[0] as f32) as usize)
                    {
                        let at = Pos2::new(x as f32 + 0.5, y as f32 + 0.5);
                        let edge =
                            |a: Pos2, b: Pos2| (b - a).x * (at - a).y - (b - a).y * (at - a).x;
                        let w = [
                            edge(p[1], p[2]) / area,
                            edge(p[2], p[0]) / area,
                            edge(p[0], p[1]) / area,
                        ];
                        let edges = [(p[1], p[2]), (p[2], p[0]), (p[0], p[1])];
                        let inside = w.iter().zip(edges).all(|(weight, (a, b))| {
                            *weight > 1e-6 || (weight.abs() <= 1e-6 && owns(a, b))
                        });
                        if !inside {
                            continue;
                        }
                        let colour: [f32; 4] = std::array::from_fn(|c| {
                            (0..3)
                                .map(|i| w[i] * f32::from(v[i].color.to_array()[c]))
                                .sum::<f32>()
                                / 255.0
                        });
                        let sampled = match texture {
                            None => [1.0; 4],
                            Some(sprite) => {
                                let u = (0..3).map(|i| w[i] * v[i].uv.x).sum::<f32>()
                                    * sprite.size[0] as f32
                                    - 0.5;
                                let t = (0..3).map(|i| w[i] * v[i].uv.y).sum::<f32>()
                                    * sprite.size[1] as f32
                                    - 0.5;
                                bilinear(&sprite.pixels, sprite.size, u, t)
                            }
                        };
                        let source: [f32; 4] = std::array::from_fn(|c| colour[c] * sampled[c]);
                        let target = &mut out[y * size[0] + x];
                        for c in 0..4 {
                            target[c] = source[c] + target[c] * (1.0 - source[3]);
                        }
                    }
                }
            }
        }
        out
    }

    /// A texture sampled with a linear filter at pixel coordinates, clamped at its edges.
    fn bilinear(pixels: &[Color32], size: [u32; 2], u: f32, v: f32) -> [f32; 4] {
        let clamp = |value: f32, most: u32| value.clamp(0.0, (most - 1) as f32);
        let (u, v) = (clamp(u, size[0]), clamp(v, size[1]));
        let (x0, y0) = (u.floor() as u32, v.floor() as u32);
        let (x1, y1) = ((x0 + 1).min(size[0] - 1), (y0 + 1).min(size[1] - 1));
        let (fx, fy) = (u - x0 as f32, v - y0 as f32);
        let at = |x: u32, y: u32| {
            pixels[(y * size[0] + x) as usize]
                .to_array()
                .map(|value| f32::from(value) / 255.0)
        };
        let (a, b, c, d) = (at(x0, y0), at(x1, y0), at(x0, y1), at(x1, y1));
        std::array::from_fn(|i| {
            (a[i] * (1.0 - fx) + b[i] * fx) * (1.0 - fy) + (c[i] * (1.0 - fx) + d[i] * fx) * fy
        })
    }

    /// The largest and the mean difference a channel between two pictures, in units of 255, and where the
    /// largest is, with both pictures' values there, so a failure says what to look at.
    fn difference_at(canvas: &[Color32], meshes: &[[f32; 4]], width: usize) -> (f32, f32, String) {
        let mut most = 0.0f32;
        let mut sum = 0.0f32;
        let mut worst = String::new();
        for (at, (a, b)) in canvas.iter().zip(meshes).enumerate() {
            for c in 0..4 {
                let gap = (f32::from(a.to_array()[c]) - b[c] * 255.0).abs();
                if gap > most {
                    most = gap;
                    let mine = b.map(|v| (v * 255.0).round() as u8);
                    worst = format!(
                        "at ({}, {}) canvas {:?} meshes {:?}",
                        at % width.max(1),
                        at / width.max(1),
                        a.to_array(),
                        mine
                    );
                }
                sum += gap;
            }
        }
        (most, sum / (canvas.len() * 4) as f32, worst)
    }

    /// The pixel store the tests rasterise with: the target's own SIMD level, as every rendering test uses.
    fn pinned() -> Sprites {
        Sprites::new(RenderSettings {
            level: vello_cpu::Level::baseline(),
            num_threads: 0,
        })
    }

    /// The shadows of a raised card and a carved well, drawn from sprites, are the canvas's own shadows to within
    /// a few units a channel: the claim the whole of this module rests on.
    #[test]
    fn a_shadow_drawn_from_a_sprite_is_the_canvas_s_shadow() {
        let theme = light();
        let area = Rect::from_min_size(pos2(0.0, 0.0), vec2(420.0, 300.0));
        for scale in [1.0, 1.5, 2.0] {
            for size in [vec2(300.0, 120.0), vec2(140.0, 36.0), vec2(20.0, 20.0)] {
                let rect = Rect::from_min_size(pos2(60.0, 80.0), size);
                let chrome = Chrome::recording();
                chrome.surface(rect, 14.0, theme.surface.s1, theme.elevation.raised);
                // The shadows alone, without their clip or the surface, so the edge of a fill does not decide it.
                let shadows: Vec<Decor> = chrome
                    .take()
                    .into_iter()
                    .filter(|item| matches!(item, Decor::Shadow { inset: false, .. }))
                    .collect();
                let ctx = egui::Context::default();
                let mut sprites = pinned();
                let shapes = compose(&ctx, &mut sprites, scale, &shadows, area)
                    .expect("shadows are sprites");
                let ([width, _], canvas) =
                    super::super::canvas::Canvas::picture(area, scale, &shadows);
                let meshes = software(&shapes, &sprites, area, scale);
                let (most, mean, worst) = difference_at(&canvas, &meshes, width);
                assert!(
                    most <= 4.0 && mean <= 0.25,
                    "outer at {scale}x {size:?}: largest {most} {worst}, mean {mean}"
                );
            }
        }
    }

    /// The inside of a carved well, drawn from a sprite, is the canvas's: the fill and the edge of the well are
    /// left out of the comparison by drawing the same fill both ways underneath.
    #[test]
    fn a_carved_well_drawn_from_a_sprite_is_the_canvas_s_well() {
        let theme = light();
        let area = Rect::from_min_size(pos2(0.0, 0.0), vec2(420.0, 300.0));
        for scale in [1.0, 1.5] {
            for size in [vec2(300.0, 120.0), vec2(140.0, 36.0)] {
                let rect = Rect::from_min_size(pos2(60.0, 80.0), size);
                let chrome = Chrome::recording();
                chrome.surface(rect, 14.0, Color32::TRANSPARENT, theme.elevation.pressed);
                let items = chrome.take();
                let ctx = egui::Context::default();
                let mut sprites = pinned();
                let shapes =
                    compose(&ctx, &mut sprites, scale, &items, area).expect("a well is meshes");
                let ([width, _], canvas) =
                    super::super::canvas::Canvas::picture(area, scale, &items);
                let meshes = software(&shapes, &sprites, area, scale);
                let (most, mean, worst) = difference_at(&canvas, &meshes, width);
                assert!(
                    most <= 6.0 && mean <= 0.25,
                    "inset at {scale}x {size:?}: largest {most} {worst}, mean {mean}"
                );
            }
        }
    }

    /// A whole raised card, fill and all, is the canvas's picture except along the edge of the fill, where
    /// `egui`'s feather and `vello_cpu`'s coverage differ by part of a pixel.
    #[test]
    fn a_surface_drawn_as_meshes_is_the_canvas_s_surface() {
        let theme = light();
        let area = Rect::from_min_size(pos2(0.0, 0.0), vec2(420.0, 300.0));
        for elevation in [theme.elevation.raised, theme.elevation.pressed] {
            let chrome = Chrome::recording();
            chrome.surface(
                Rect::from_min_size(pos2(60.0, 80.0), vec2(300.0, 120.0)),
                14.0,
                theme.surface.s1,
                elevation,
            );
            let items = chrome.take();
            let ctx = egui::Context::default();
            let mut sprites = pinned();
            let shapes =
                compose(&ctx, &mut sprites, 1.5, &items, area).expect("a surface is meshes");
            let ([width, _], canvas) = super::super::canvas::Canvas::picture(area, 1.5, &items);
            let meshes = software(&shapes, &sprites, area, 1.5);
            let (most, mean, worst) = difference_at(&canvas, &meshes, width);
            assert!(
                most <= 96.0 && mean <= 0.1,
                "largest {most} {worst}, mean {mean}"
            );
        }
    }

    /// The axis of a nine patch is cut so its fixed ends keep their size on the screen.
    #[test]
    fn a_stretched_axis_keeps_its_ends_at_their_own_size() {
        let pieces = cuts(100.0, 30.0, 30.0, 0.0, 500.0, 2.0);
        assert_eq!(
            pieces,
            vec![
                (0.0, 30.0, 0.0, 15.0),
                (30.0, 70.0, 15.0, 485.0),
                (70.0, 100.0, 485.0, 500.0)
            ]
        );
        assert_eq!(
            cuts(100.0, 100.0, 0.0, 0.0, 50.0, 2.0),
            vec![(0.0, 100.0, 0.0, 50.0)]
        );
    }

    /// A sprite's size is the size its picture was drawn at, even for a side of nothing, so the texture and its
    /// pixels always agree. Unluminous `task-2317`: a carved well squeezed to no width made a sprite whose size
    /// said 0 by 106 while its picture held 106 pixels, and the renderer ended the process on it.
    #[test]
    fn a_sprite_of_no_width_still_agrees_with_its_own_pixels() {
        let ctx = egui::Context::default();
        let mut sprites = pinned();
        for (width, height) in [
            (Some(0), Some(424)),
            (Some(424), Some(0)),
            (Some(0), Some(0)),
        ] {
            let key = Key::Inset {
                clip: [0; 4],
                radius: 40,
                deviation: 20,
                gaps: [0; 4],
                width,
                height,
            };
            let sprite = sprites.sprite(&ctx, key);
            assert_eq!(
                (sprite.size[0] * sprite.size[1]) as usize,
                sprite.pixels.len(),
                "{width:?} by {height:?}"
            );
        }
        // And a carved surface with no width draws no shadow at all.
        let mesh = inset_shadow(
            &ctx,
            &mut sprites,
            Rect::from_min_size(pos2(10.0, 10.0), vec2(0.0, 106.0)),
            Corners::all(8.0),
            Rect::from_min_size(pos2(10.0, 10.0), vec2(0.0, 106.0)),
            8.0,
            4.0,
            Color32::BLACK,
            1.0,
        );
        assert!(mesh.is_empty());
    }
}
