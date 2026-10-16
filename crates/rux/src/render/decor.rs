//! One piece of decoration, and what fills it.
//!
//! Deliberately as small and as plain as it can be: seven kinds of shape and a clip, in points,
//! absolute in the window's own coordinates, with no behaviour on any of them. That is what makes a
//! drawing something a test can assert on with no window, no graphics card and no fonts, and it is
//! what lets [`super::Canvas`] decide whether to rasterise by comparing two lists.

use std::hash::{Hash, Hasher};

use egui::{Color32, Pos2, Rect, Vec2};

use crate::theme::Gradient;

/// How round each corner of a surface is.
///
/// Four rather than one, because the reference squares off single corners: `.messageWrapper` in the
/// chat design, and the segmented control's end caps. `From<f32>` so a caller that means all four
/// writes one number.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Corners {
    pub nw: f32,
    pub ne: f32,
    pub se: f32,
    pub sw: f32,
}

impl Corners {
    pub const NONE: Self = Self::all(0.0);

    pub const fn all(radius: f32) -> Self {
        Self { nw: radius, ne: radius, se: radius, sw: radius }
    }

    /// Rounded at the top, square at the bottom.
    pub const fn top(radius: f32) -> Self {
        Self { nw: radius, ne: radius, se: 0.0, sw: 0.0 }
    }

    /// Rounded at the bottom, square at the top.
    pub const fn bottom(radius: f32) -> Self {
        Self { nw: 0.0, ne: 0.0, se: radius, sw: radius }
    }

    /// The one number a **shadow** is drawn with.
    ///
    /// `vello_cpu`'s blurred rounded rectangle takes a single radius and there is no per-corner form.
    /// That costs nothing worth having: a shadow is a Gaussian several points wide, and a few points
    /// of difference on one corner of it is not visible. The surface itself — the edge a person reads
    /// — is drawn from all four.
    pub fn representative(self) -> f32 {
        (self.nw + self.ne + self.se + self.sw) / 4.0
    }
}

impl From<f32> for Corners {
    fn from(radius: f32) -> Self {
        Self::all(radius)
    }
}

impl From<egui::CornerRadius> for Corners {
    fn from(corners: egui::CornerRadius) -> Self {
        Self {
            nw: f32::from(corners.nw),
            ne: f32::from(corners.ne),
            se: f32::from(corners.se),
            sw: f32::from(corners.sw),
        }
    }
}

/// What fills a shape: a colour, or a linear gradient given as the two ends of its axis.
///
/// The axis rather than an angle, so a diagonal is not a special case at the point of drawing. A
/// [`Gradient`] is turned into one of these by [`Fill::gradient`], which is where CSS's angle
/// convention is applied.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Fill {
    Solid(Color32),
    Linear { from: Pos2, to: Pos2, start: Color32, end: Color32 },
}

impl Fill {
    /// A theme gradient across `rect`, with CSS's own angle convention.
    pub fn gradient(gradient: Gradient, rect: Rect) -> Self {
        let (from, to) = gradient.axis(rect);
        Self::Linear { from, to, start: gradient.from, end: gradient.to }
    }

    /// The colour this fill reads as.
    pub fn representative(self) -> Color32 {
        match self {
            Self::Solid(colour) => colour,
            Self::Linear { start, .. } => start,
        }
    }

    /// Whether the surface hides whatever is behind it.
    ///
    /// Read by [`super::Chrome::surface`] before it cuts an outer shadow to the band around the shape:
    /// that cut is only sound when the surface really does cover its own shadow, and a translucent
    /// one does not.
    pub fn is_opaque(self) -> bool {
        match self {
            Self::Solid(colour) => colour.a() == 255,
            Self::Linear { start, end, .. } => start.a() == 255 && end.a() == 255,
        }
    }
}

impl From<Color32> for Fill {
    fn from(colour: Color32) -> Self {
        Self::Solid(colour)
    }
}

/// One piece of decoration.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Decor {
    /// A rounded rectangle: every panel, card, button, well and chip.
    Rect { rect: Rect, corners: Corners, fill: Fill },
    /// A Gaussian blur of a rounded rectangle, outside the shape or inside it.
    ///
    /// `inset` is `vello_cpu`'s own `invert` argument, whose documentation says in as many words that
    /// it is how an inset box shadow is drawn. It is the reason this module exists.
    ///
    /// `deviation` is the Gaussian's standard deviation, already halved from the CSS blur radius by
    /// [`crate::theme::Shadow::deviation`].
    Shadow { rect: Rect, radius: f32, deviation: f32, colour: Color32, inset: bool },
    /// A filled circle: a status dot, a round badge, the project icon.
    Disc { centre: Pos2, radius: f32, fill: Fill },
    /// An unfilled circle: a spinner's track, a focus ring round a round button.
    Ring { centre: Pos2, radius: f32, width: f32, colour: Color32 },
    /// Part of a circle, stroked. The spinner's arc, which is the one stroke in the library whose
    /// paint is a gradient.
    ///
    /// Angles in radians, clockwise from twelve o'clock, which is how the reference's
    /// `M22 4 a18 18 0 0 1 18 18` reads.
    ///
    /// `round` is the stroke's caps: round for every arc the reference draws, square-ended for a ring
    /// chart's slice too short to carry two round caps without growing past its own share.
    Arc { centre: Pos2, radius: f32, width: f32, start: f32, sweep: f32, fill: Fill, round: bool },
    /// A straight line: every divider and hairline.
    Line { from: Pos2, to: Pos2, width: f32, colour: Color32 },
    /// Everything after this is cut to this rounded rectangle, until [`Decor::Unclip`].
    ///
    /// `outside` cuts to everything **outside** it instead, bounded by `bound` — which is the band a
    /// raised surface's shadow is drawn in, and what makes elevation affordable rather than merely
    /// correct.
    Clip { rect: Rect, corners: Corners, outside: bool, bound: Rect },
    Unclip,
    /// Glass: the picture behind the layer, bent through this rounded rectangle (`task-2281`).
    ///
    /// Not rasterised. [`super::layer`] cuts the list at each one and draws the bent picture there as
    /// an `egui` mesh, so whatever was recorded after it is painted over it and whatever was recorded
    /// before it is under it. `bevel` is how deep the rim is and `strength` how far it bends, both in
    /// points; see [`crate::theme::Glass`].
    Glass { rect: Rect, corners: Corners, bevel: f32, strength: f32 },
}

impl Decor {
    /// The rectangle this item covers, shadow reach and all.
    ///
    /// A blurred rectangle reaches `2.5` standard deviations past its own edge, which is where
    /// `vello_cpu` cuts the Gaussian off.
    pub fn bounds(self) -> Rect {
        match self {
            Self::Rect { rect, .. } => rect,
            Self::Shadow { rect, deviation, inset, .. } => match inset {
                true => rect,
                false => rect.expand(deviation * 2.5),
            },
            Self::Disc { centre, radius, .. } => {
                Rect::from_center_size(centre, Vec2::splat(radius * 2.0))
            }
            Self::Ring { centre, radius, width, .. } | Self::Arc { centre, radius, width, .. } => {
                Rect::from_center_size(centre, Vec2::splat(radius * 2.0 + width))
            }
            Self::Line { from, to, width, .. } => Rect::from_two_pos(from, to).expand(width),
            Self::Clip { bound, .. } => bound,
            Self::Unclip => Rect::NOTHING,
            Self::Glass { rect, .. } => rect,
        }
    }
}

/// The rectangle everything in `items` fits inside, or `None` when there is nothing to draw.
pub fn bounds_of(items: &[Decor]) -> Option<Rect> {
    items
        .iter()
        .filter(|item| !matches!(item, Decor::Unclip))
        .map(|item| item.bounds())
        .reduce(|so_far, next| so_far.union(next))
}

/// Hashed by bit pattern, because `f32` is not `Eq`.
///
/// Nothing decides whether to redraw from a hash — [`super::Canvas`] keeps the list itself and
/// compares it, so a collision cannot serve stale pixels. This is here because a `Decor` is a value,
/// and a test that wants to say "these two drawings are the same drawing" should not have to spell out
/// how.
impl Hash for Decor {
    fn hash<H: Hasher>(&self, into: &mut H) {
        fn f(value: f32, into: &mut impl Hasher) {
            value.to_bits().hash(into);
        }
        fn pos(at: Pos2, into: &mut impl Hasher) {
            f(at.x, into);
            f(at.y, into);
        }
        fn rect(area: Rect, into: &mut impl Hasher) {
            pos(area.min, into);
            pos(area.max, into);
        }
        fn corners(of: Corners, into: &mut impl Hasher) {
            f(of.nw, into);
            f(of.ne, into);
            f(of.se, into);
            f(of.sw, into);
        }
        fn fill(of: Fill, into: &mut impl Hasher) {
            match of {
                Fill::Solid(colour) => {
                    0u8.hash(into);
                    colour.to_array().hash(into);
                }
                Fill::Linear { from, to, start, end } => {
                    1u8.hash(into);
                    pos(from, into);
                    pos(to, into);
                    start.to_array().hash(into);
                    end.to_array().hash(into);
                }
            }
        }
        std::mem::discriminant(self).hash(into);
        match *self {
            Decor::Rect { rect: area, corners: round, fill: paint } => {
                rect(area, into);
                corners(round, into);
                fill(paint, into);
            }
            Decor::Shadow { rect: area, radius, deviation, colour, inset } => {
                rect(area, into);
                f(radius, into);
                f(deviation, into);
                colour.to_array().hash(into);
                inset.hash(into);
            }
            Decor::Disc { centre, radius, fill: paint } => {
                pos(centre, into);
                f(radius, into);
                fill(paint, into);
            }
            Decor::Ring { centre, radius, width, colour } => {
                pos(centre, into);
                f(radius, into);
                f(width, into);
                colour.to_array().hash(into);
            }
            Decor::Arc { centre, radius, width, start, sweep, fill: paint, round } => {
                pos(centre, into);
                f(radius, into);
                f(width, into);
                f(start, into);
                f(sweep, into);
                fill(paint, into);
                round.hash(into);
            }
            Decor::Line { from, to, width, colour } => {
                pos(from, into);
                pos(to, into);
                f(width, into);
                colour.to_array().hash(into);
            }
            Decor::Clip { rect: area, corners: round, outside, bound } => {
                rect(area, into);
                corners(round, into);
                outside.hash(into);
                rect(bound, into);
            }
            Decor::Unclip => {}
            Decor::Glass { rect: area, corners: round, bevel, strength } => {
                rect(area, into);
                corners(round, into);
                f(bevel, into);
                f(strength, into);
            }
        }
    }
}
