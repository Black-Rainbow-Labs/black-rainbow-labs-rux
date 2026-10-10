//! The renderer: a `Decor` list into pixels, and the pixels into one `egui` texture.
//!
//! The whole of the cleverness is in [`Canvas::texture_for`], and it is a cache: a frame where nothing
//! moved must cost a comparison and nothing else.

use std::collections::HashMap;

use egui::{Color32, Rect};
use vello_cpu::kurbo::{Affine, Arc, BezPath, Circle, Point, RoundedRect, Shape as _, Stroke};
use vello_cpu::peniko::color::{AlphaColor, Srgb};
use vello_cpu::peniko::Fill as FillRule;
use vello_cpu::peniko::{ColorStop, ColorStops, Gradient, GradientKind, LinearGradientPosition};
use vello_cpu::{
    CompositeMode, PixelFormat, Pixmap, RasterizerSettings, RenderContext, RenderMode,
    RenderSettings, Resources,
};

use super::decor::{bounds_of, Corners, Decor, Fill};

/// The largest canvas that will ever be rasterised, in physical pixels a side.
///
/// Here so that an absurd size — a bug elsewhere, a screen configuration nobody has — degrades to *no
/// decoration* rather than to a four hundred megabyte allocation.
const MAX_SIDE: u16 = 4096;

/// The most pixels a point of decoration is ever rasterised at.
///
/// The decoration's highest-frequency feature is the edge of a rounded rectangle; everything else in
/// it is a Gaussian or a gradient, which is exactly the content that survives being drawn slightly
/// coarser and filtered up. The text over it is unaffected, because `egui` draws the text at the
/// display's own resolution.
///
/// The number is a cost. Rasterising is per pixel, so a display at two pixels a point is four times
/// the work; capping at 1.5 is the difference between a full screen of components that drops frames
/// while it is being dragged and one that does not.
pub const MAX_SCALE: f32 = 1.5;

/// What a canvas's current texture was made from.
///
/// **All of it, not a hash of the shapes.** The size and the scale are in it because two fractional
/// pixel densities can round to the same pixel dimensions and still need different transforms, and the
/// rectangle is in it because the canvas is a bounding box intersected with the layer, so the same
/// shapes in a moved layer are a different picture. And the list is **kept and compared** rather than
/// trusted to a 64-bit hash: a collision would serve stale pixels, and comparing a few hundred `Copy`
/// values costs less than the rasterisation it is deciding to skip.
#[derive(Debug, Clone, PartialEq)]
struct Drawn {
    items: Vec<Decor>,
    rect: Rect,
    scale: f32,
    width: u16,
    height: u16,
}

/// One layer's canvas: the renderer, the pixels and the texture they were uploaded as.
///
/// Kept between frames, which is the whole point: the texture is what a frame where nothing moved
/// reuses.
pub struct Canvas {
    context: RenderContext,
    resources: Resources,
    pixmap: Pixmap,
    texture: Option<egui::TextureHandle>,
    drawn: Option<Drawn>,
    /// How many times this canvas has actually rasterised. Read by the test that says a still layer is
    /// free, and by `--measure`.
    rasterisations: u64,
    /// The frame this canvas was last asked for, which is how [`Canvases::tidy`] knows it is wanted.
    last_used: u64,
}

impl std::fmt::Debug for Canvas {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        out.debug_struct("Canvas")
            .field("size", &(self.pixmap.width(), self.pixmap.height()))
            .field("rasterisations", &self.rasterisations)
            .finish()
    }
}

impl Default for Canvas {
    fn default() -> Self {
        Self::new(RenderSettings::default())
    }
}

impl Canvas {
    fn new(settings: RenderSettings) -> Self {
        Self {
            context: RenderContext::new_with(1, 1, settings),
            resources: Resources::new(),
            pixmap: Pixmap::new(1, 1),
            texture: None,
            drawn: None,
            rasterisations: 0,
            last_used: 0,
        }
    }

    /// A canvas whose output is the same on every machine.
    ///
    /// `RenderSettings::default()` detects the widest SIMD this processor has, and whether two levels
    /// are bit-identical is not something `vello_cpu` promises — so a picture accepted on one machine
    /// could fail on another for a reason that is not a fault in the library. The tests pin the
    /// **baseline** level, which is the set of features the target itself guarantees. `Level::fallback`
    /// is deliberately not used: it is compiled out on a target whose baseline is already above it.
    pub fn deterministic() -> Self {
        Self::new(RenderSettings { level: vello_cpu::Level::baseline(), num_threads: 0 })
    }

    pub fn rasterisations(&self) -> u64 {
        self.rasterisations
    }

    /// How much memory this canvas is holding, in bytes. Read by `--measure`.
    pub fn bytes(&self) -> usize {
        usize::from(self.pixmap.width()) * usize::from(self.pixmap.height()) * 4
    }

    /// Rasterise `items` if they or the size have changed, and answer the texture to paint.
    ///
    /// `None` when there is nothing to draw or the layer is an impossible size, which the caller draws
    /// as nothing rather than as a blank rectangle.
    pub fn texture_for(
        &mut self,
        ctx: &egui::Context,
        id: egui::Id,
        rect: Rect,
        pixels_per_point: f32,
        items: &[Decor],
    ) -> Option<(egui::TextureId, Rect)> {
        if items.is_empty() {
            return None;
        }
        // **The canvas is the decoration's own bounding box, not the layer's.** Rasterising has a
        // floor of about two nanoseconds a pixel whether anything is drawn there or not, so an empty
        // canvas the size of a full window costs milliseconds before a single shape.
        let rect = bounds_of(items).map(|bounds| bounds.intersect(rect))?;
        if rect.width() < 1.0 || rect.height() < 1.0 {
            return None;
        }
        let scale = pixels_per_point.min(MAX_SCALE);
        let width = (rect.width() * scale).round();
        let height = (rect.height() * scale).round();
        if !(width.is_finite() && height.is_finite()) || width < 1.0 || height < 1.0 {
            return None;
        }
        if width > f32::from(MAX_SIDE) || height > f32::from(MAX_SIDE) {
            return None;
        }
        let (width, height) = (width as u16, height as u16);
        // Compared without building anything: a frame where nothing moved must not allocate, and the
        // list is only copied on the frame it is actually drawn from.
        let same = self.drawn.as_ref().is_some_and(|drawn| {
            drawn.width == width
                && drawn.height == height
                && drawn.scale == scale
                && drawn.rect == rect
                && drawn.items == items
        });
        if same {
            if let Some(texture) = &self.texture {
                return Some((texture.id(), rect));
            }
        }
        self.rasterise(rect, scale, width, height, items);
        // Built here and handed straight over. `TextureHandle::set` takes ownership and the upload
        // happens at the end of the frame, so there is no buffer worth keeping and no reason to fill
        // one and then copy it.
        let image = egui::ColorImage::new([usize::from(width), usize::from(height)], self.pixels());
        match &mut self.texture {
            // Set rather than load, so a layer being typed into does not allocate a texture a keystroke.
            Some(texture) => texture.set(image, egui::TextureOptions::LINEAR),
            // The name is built once, when the texture is first made. Formatting one every frame would
            // be an allocation on a frame that draws nothing new.
            none => {
                *none = Some(ctx.load_texture(
                    format!("{id:?}-rux-chrome"),
                    image,
                    egui::TextureOptions::LINEAR,
                ));
            }
        }
        self.drawn = Some(Drawn { items: items.to_vec(), rect, scale, width, height });
        self.texture.as_ref().map(|texture| (texture.id(), rect))
    }

    /// Replay the list into the renderer and take the pixels out.
    fn rasterise(
        &mut self,
        rect: Rect,
        pixels_per_point: f32,
        width: u16,
        height: u16,
        items: &[Decor],
    ) {
        self.rasterisations += 1;
        // **Resized rather than rebuilt.** `RenderContext::new_with` allocates the whole dispatcher,
        // and the canvas is the decoration's bounding box, which moves by a point whenever anything
        // grows — so rebuilding on a size change would mean rebuilding on nearly every changed frame.
        if self.pixmap.width() != width || self.pixmap.height() != height {
            self.pixmap.resize(width, height);
        }
        self.context.reset_and_resize(width, height);
        // Points to physical pixels, and the layer's own top left to the origin. Every `Decor` is in
        // the window's coordinates, so this is the only place the two spaces meet.
        let to_pixels = Affine::scale(f64::from(pixels_per_point))
            * Affine::translate((-f64::from(rect.min.x), -f64::from(rect.min.y)));
        self.context.set_transform(to_pixels);
        let mut clips = 0usize;
        for item in items {
            self.draw(*item, &mut clips);
        }
        // A component that clipped and did not unclip leaves a clip on the stack. `push_clip_path`
        // allows that where `push_clip_layer` would panic, and it is popped anyway so the next frame
        // starts clean.
        for _ in 0..clips {
            self.context.pop_clip_path();
        }
        self.context.flush();
        self.context.render_with(
            self.pixmap.as_mut(),
            &mut self.resources,
            RasterizerSettings {
                render_mode: RenderMode::OptimizeSpeed,
                composite_mode: CompositeMode::Replace,
                pixel_format: PixelFormat::Rgba8,
                offset: (0, 0),
            },
        );
    }

    /// The pixmap as egui's own colour.
    ///
    /// `PremulRgba8` and `Color32` are the same four bytes in the same order, which
    /// `a_pixmap_is_the_same_four_bytes_as_egui` asserts rather than assumes — so this is a copy rather
    /// than a conversion, and it is written as a copy.
    fn pixels(&self) -> Vec<Color32> {
        let bytes = self.pixmap.data_as_u8_slice();
        let mut colours = Vec::with_capacity(bytes.len() / 4);
        colours.extend(bytes.chunks_exact(4).map(|pixel| {
            Color32::from_rgba_premultiplied(pixel[0], pixel[1], pixel[2], pixel[3])
        }));
        colours
    }

    fn draw(&mut self, item: Decor, clips: &mut usize) {
        match item {
            Decor::Rect { rect, corners, fill } => {
                self.set_fill(fill);
                self.context.fill_path(&rounded(rect, corners));
            }
            Decor::Shadow { rect, radius, deviation, colour, inset } => {
                self.context.set_paint(colour_of(colour));
                self.context.fill_blurred_rounded_rect(
                    &box_of(rect),
                    radius.max(0.0),
                    deviation.max(0.01),
                    inset,
                );
            }
            Decor::Disc { centre, radius, fill } => {
                self.set_fill(fill);
                let circle = Circle::new(point(centre), f64::from(radius));
                self.context.fill_path(&circle.to_path(0.1));
            }
            Decor::Ring { centre, radius, width, colour } => {
                self.context.set_paint(colour_of(colour));
                self.context.set_stroke(Stroke::new(f64::from(width)));
                let circle = Circle::new(point(centre), f64::from(radius));
                self.context.stroke_path(&circle.to_path(0.1));
            }
            Decor::Arc { centre, radius, width, start, sweep, fill, round } => {
                self.set_fill(fill);
                let caps = match round {
                    true => vello_cpu::kurbo::Cap::Round,
                    false => vello_cpu::kurbo::Cap::Butt,
                };
                self.context.set_stroke(
                    Stroke::new(f64::from(width))
                        .with_caps(caps)
                        .with_join(vello_cpu::kurbo::Join::Round),
                );
                // `kurbo::Arc` measures from three o'clock and anticlockwise; the library's own angles
                // are clockwise from twelve, which is how the reference's arc path reads.
                let arc = Arc::new(
                    point(centre),
                    (f64::from(radius), f64::from(radius)),
                    f64::from(start) - std::f64::consts::FRAC_PI_2,
                    f64::from(sweep),
                    0.0,
                );
                self.context.stroke_path(&arc.to_path(0.05));
            }
            Decor::Line { from, to, width, colour } => {
                self.context.set_paint(colour_of(colour));
                self.context.set_stroke(Stroke::new(f64::from(width)));
                let mut path = BezPath::new();
                path.move_to(point(from));
                path.line_to(point(to));
                self.context.stroke_path(&path);
            }
            Decor::Clip { rect, corners, outside, bound } => {
                // `push_clip_path` rather than `push_clip_layer`: a clip **layer** renders its contents
                // into a layer of their own and composites them, and nothing here needs that. A clip
                // **path** is intersected while the strips are generated, so a tile outside it costs
                // nothing at all — which is what makes the shadow band cheap rather than merely
                // correct.
                let (path, rule) = match outside {
                    false => (rounded(rect, corners), FillRule::NonZero),
                    // Everything within `bound` and outside the shape: two subpaths read even-odd.
                    true => {
                        let mut path = rounded(bound, Corners::NONE);
                        path.extend(rounded(rect, corners));
                        (path, FillRule::EvenOdd)
                    }
                };
                self.context.set_fill_rule(rule);
                self.context.push_clip_path(&path);
                // Back to the rule everything else is filled with, or the next shape would be read
                // even-odd.
                self.context.set_fill_rule(FillRule::NonZero);
                *clips += 1;
            }
            Decor::Unclip => {
                if *clips > 0 {
                    self.context.pop_clip_path();
                    *clips -= 1;
                }
            }
        }
    }

    fn set_fill(&mut self, fill: Fill) {
        match fill {
            Fill::Solid(colour) => self.context.set_paint(colour_of(colour)),
            Fill::Linear { from, to, start, end } => {
                let mut stops = ColorStops::default();
                stops.0.push(ColorStop::from((0.0, colour_of(start))));
                stops.0.push(ColorStop::from((1.0, colour_of(end))));
                self.context.set_paint(Gradient {
                    kind: GradientKind::Linear(LinearGradientPosition::new(
                        point(from),
                        point(to),
                    )),
                    stops,
                    ..Default::default()
                });
            }
        }
    }
}

/// egui's premultiplied colour as Vello's un-premultiplied one.
///
/// The one conversion in the file, so the premultiplication is undone in one place. A fully
/// transparent colour has no hue to recover, so it stays black.
fn colour_of(colour: Color32) -> AlphaColor<Srgb> {
    let [r, g, b, a] = colour.to_srgba_unmultiplied();
    AlphaColor::from_rgba8(r, g, b, a)
}

fn point(at: egui::Pos2) -> Point {
    Point::new(f64::from(at.x), f64::from(at.y))
}

fn box_of(rect: Rect) -> vello_cpu::kurbo::Rect {
    vello_cpu::kurbo::Rect::new(
        f64::from(rect.min.x),
        f64::from(rect.min.y),
        f64::from(rect.max.x),
        f64::from(rect.max.y),
    )
}

/// The path of a rounded rectangle, with each corner its own radius.
///
/// Every radius is clamped to half the shorter side, because a larger one describes an arc that
/// overruns the side it is on and `kurbo` draws a shape nobody asked for rather than refusing. That is
/// also what makes `--r-pill: 999px` a legal value at every size.
fn rounded(rect: Rect, corners: Corners) -> BezPath {
    let most = f64::from(rect.width().min(rect.height())) / 2.0;
    let one = |radius: f32| f64::from(radius).clamp(0.0, most.max(0.0));
    let radii = vello_cpu::kurbo::RoundedRectRadii::new(
        one(corners.nw),
        one(corners.ne),
        one(corners.se),
        one(corners.sw),
    );
    RoundedRect::from_rect(box_of(rect), radii).to_path(0.1)
}

/// Every canvas in this window, one per layer that draws decoration.
///
/// Keyed on the `egui::Id` the layer was drawn with. A canvas whose id was not drawn this frame is
/// dropped, so closing a popup gives its texture back.
#[derive(Debug, Default)]
pub struct Canvases {
    by_id: HashMap<egui::Id, Canvas>,
    /// Which frame this is, counted by [`Canvases::tidy`].
    ///
    /// A counter rather than a list of what was drawn this frame: the list version compared lengths to
    /// decide whether anything had gone, so one layer asked for twice would have masked another that
    /// had gone away. Each canvas remembers the frame it was last asked for, and one that missed a
    /// frame is dropped.
    frame: u64,
    /// Built with the tests' pinned SIMD level rather than the machine's.
    deterministic: bool,
}

impl Canvases {
    /// The store the rendering tests use, whose canvases rasterise the same on every machine.
    pub fn deterministic() -> Self {
        Self { deterministic: true, ..Self::default() }
    }

    /// Rasterise one layer's decoration and answer the texture to paint and where to paint it.
    ///
    /// `pixels_per_point` is what the layer is composited at, which is `render::composited_scale`: the
    /// display's density, times the zoom of a layer drawn through a transform.
    pub fn texture_for(
        &mut self,
        ctx: &egui::Context,
        id: egui::Id,
        rect: Rect,
        pixels_per_point: f32,
        items: &[Decor],
    ) -> Option<(egui::TextureId, Rect)> {
        let deterministic = self.deterministic;
        let frame = self.frame;
        let canvas = self.by_id.entry(id).or_insert_with(|| match deterministic {
            true => Canvas::deterministic(),
            false => Canvas::default(),
        });
        canvas.last_used = frame;
        canvas.texture_for(ctx, id, rect, pixels_per_point, items)
    }

    /// Forget the canvases of layers that were not drawn this frame. Called once, at the end of a frame.
    pub fn tidy(&mut self) {
        let frame = self.frame;
        self.by_id.retain(|_, canvas| canvas.last_used == frame);
        self.frame = self.frame.wrapping_add(1);
    }

    /// How many canvases are alive and how many bytes of pixmap they are holding, for `--measure`.
    pub fn footprint(&self) -> (usize, usize) {
        (self.by_id.len(), self.by_id.values().map(Canvas::bytes).sum())
    }

    /// How many times the canvas with this id has rasterised.
    pub fn rasterisations(&self, id: egui::Id) -> u64 {
        self.by_id.get(&id).map(Canvas::rasterisations).unwrap_or_default()
    }

    /// Every rasterisation this store has done, which is what `--measure` reports a frame.
    pub fn total_rasterisations(&self) -> u64 {
        self.by_id.values().map(Canvas::rasterisations).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::Chrome;
    use crate::theme::light;

    #[test]
    fn a_pixmap_is_the_same_four_bytes_as_egui() {
        let mut pixmap = Pixmap::new(1, 1);
        pixmap.data_as_u8_slice_mut().copy_from_slice(&[10, 20, 30, 40]);
        let bytes = pixmap.data_as_u8_slice();
        let colour = Color32::from_rgba_premultiplied(bytes[0], bytes[1], bytes[2], bytes[3]);
        assert_eq!(colour.to_array(), [10, 20, 30, 40]);
    }

    #[test]
    fn a_still_list_is_not_rasterised_twice() {
        let ctx = egui::Context::default();
        let theme = light();
        let chrome = Chrome::recording();
        let rect = Rect::from_min_size(egui::pos2(20.0, 20.0), egui::vec2(120.0, 60.0));
        chrome.surface(rect, 12.0, theme.surface.s1, theme.elevation.raised);
        let items = chrome.take();

        let mut canvas = Canvas::deterministic();
        let id = egui::Id::new("still");
        ctx.begin_pass(egui::RawInput::default());
        assert!(canvas.texture_for(&ctx, id, rect.expand(40.0), 1.0, &items).is_some());
        assert_eq!(canvas.rasterisations(), 1);
        assert!(canvas.texture_for(&ctx, id, rect.expand(40.0), 1.0, &items).is_some());
        assert_eq!(canvas.rasterisations(), 1, "an unchanged list must not rasterise again");
        // `TexturesDelta` panics when it is dropped with an upload nobody applied, which in a real
        // window is the renderer's job. There is no renderer here, so the deltas are dropped by hand.
        ctx.end_pass().textures_delta.clear();
    }

    #[test]
    fn an_empty_list_draws_nothing_at_all() {
        let ctx = egui::Context::default();
        let mut canvas = Canvas::deterministic();
        let rect = Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(100.0, 100.0));
        assert!(canvas.texture_for(&ctx, egui::Id::new("empty"), rect, 1.0, &[]).is_none());
        assert_eq!(canvas.rasterisations(), 0);
    }

    #[test]
    fn a_canvas_covers_the_decoration_rather_than_the_layer() {
        let chrome = Chrome::recording();
        chrome.rect(
            Rect::from_min_size(egui::pos2(10.0, 10.0), egui::vec2(20.0, 20.0)),
            4.0,
            Color32::WHITE,
        );
        let items = chrome.take();
        let bounds = bounds_of(&items).unwrap();
        assert_eq!(bounds, Rect::from_min_size(egui::pos2(10.0, 10.0), egui::vec2(20.0, 20.0)));
    }
}
