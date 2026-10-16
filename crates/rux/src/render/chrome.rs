//! What a component talks to when it wants depth.
//!
//! A component says *what kind of surface* it is drawing — a rounded rectangle at an elevation, a
//! disc, a ring, a line, a rounded clip — and never holds a shadow offset. The elevation recipe lives
//! in the theme, and turning one into shapes happens here, once.

use egui::{Color32, Pos2, Rect, Vec2};

use super::decor::{Corners, Decor, Fill};
use crate::theme::{Elevation, Glass};

/// The recorder every component draws its decoration into.
///
/// A `Mutex` rather than a `RefCell` so that [`Chrome::off`] can be a `static`: a component takes a
/// `&Chrome` and a caller that wants no decoration at all needs one that outlives everything. It is
/// never contended — a layer is drawn on the thread that owns the window — so the lock is an
/// uncontended atomic.
#[derive(Debug, Default)]
pub struct Chrome {
    recording: bool,
    items: std::sync::Mutex<Vec<Decor>>,
    /// The material of the theme the layer is drawn in. `Some` makes a raised translucent surface
    /// glass; see [`Chrome::surface`].
    glass: Option<Glass>,
}

impl Chrome {
    /// A chrome that records nothing, whatever is asked of it.
    ///
    /// `const`, so it can be a `static` — which is what lets a test build a component context with no
    /// canvas behind it.
    pub const fn off() -> Self {
        Self { recording: false, items: std::sync::Mutex::new(Vec::new()), glass: None }
    }

    /// A chrome that records, for a layer that has a canvas behind it.
    pub fn recording() -> Self {
        Self { recording: true, items: std::sync::Mutex::new(Vec::new()), glass: None }
    }

    /// The same chrome drawing in a material: `Some` is glass (`task-2281`). [`super::layer`] sets the
    /// theme's own, so a component never asks.
    pub fn with_glass(mut self, glass: Option<Glass>) -> Self {
        self.glass = glass;
        self
    }

    /// The material this chrome draws raised translucent surfaces in.
    pub fn glass(&self) -> Option<Glass> {
        self.glass
    }

    /// Whether anything drawn into this will be seen.
    ///
    /// A component asks so it can draw the flat form instead: with the decoration off, a card is a
    /// filled rectangle with a hairline border, which is what somebody who turned the decoration off
    /// asked for.
    pub fn is_recording(&self) -> bool {
        self.recording
    }

    /// Take what was recorded, leaving the chrome empty for the next frame.
    pub fn take(&self) -> Vec<Decor> {
        match self.items.lock() {
            Ok(mut items) => std::mem::take(&mut *items),
            // A poisoned lock means a component panicked mid-drawing. Drawing the layer without
            // decoration is a better report than a second panic on top of the first.
            Err(_) => Vec::new(),
        }
    }

    fn push(&self, item: Decor) {
        if self.recording {
            if let Ok(mut items) = self.items.lock() {
                items.push(item);
            }
        }
    }

    /// A rounded rectangle with no depth at all.
    pub fn rect(&self, rect: Rect, radius: impl Into<Corners>, fill: impl Into<Fill>) {
        self.push(Decor::Rect { rect, corners: radius.into(), fill: fill.into() });
    }

    /// A surface at an elevation: the whole of what a card, a button, a well or a chip is.
    ///
    /// This is CSS's own painting order, and each step is that order rather than a choice:
    ///
    /// 1. **the outer shadows, last-declared first**, because CSS paints the first shadow in a list on
    ///    top of the ones after it;
    /// 2. **the surface**, over them;
    /// 3. **the inner shadows**, clipped to the surface, again last-declared first.
    ///
    /// Two details are the difference between this looking right and looking nearly right.
    ///
    /// **An outer shadow is cut to the band around the shape when the fill is opaque.** The surface is
    /// painted over its own shadow, so every pixel of Gaussian inside it is computed and then thrown
    /// away — measured in Unluminous at 3.3 ms for a single lane-sized surface. The cut is `Clip {
    /// outside: true }`, an even-odd frame. It is **not** applied to a translucent surface, which
    /// really does show its own shadow through itself.
    ///
    /// **An inner shadow is clipped to the border box, and it has to be.**
    /// `fill_blurred_rounded_rect`'s inverted form paints the *complement* of the blur: opaque
    /// everywhere outside the rectangle and fading to nothing inside it. Unclipped, a well would wash
    /// its own colour across the whole layer. CSS clips an inset shadow to the border box for exactly
    /// this reason.
    pub fn surface(
        &self,
        rect: Rect,
        radius: impl Into<Corners>,
        fill: impl Into<Fill>,
        elevation: Elevation,
    ) {
        let corners = radius.into();
        let fill = fill.into();
        let representative = corners.representative();

        let outer: Vec<_> = elevation.shadows().filter(|shadow| !shadow.inset).collect();
        let has_outer = !outer.is_empty();
        if has_outer {
            let cut = fill.is_opaque();
            if cut {
                self.push(Decor::Clip {
                    rect,
                    corners,
                    outside: true,
                    bound: rect.expand(elevation.reach()),
                });
            }
            for shadow in outer.into_iter().rev() {
                self.push(Decor::Shadow {
                    rect: rect
                        .translate(Vec2::new(shadow.dx, shadow.dy))
                        .expand(shadow.spread),
                    radius: representative + shadow.spread,
                    deviation: shadow.deviation(),
                    colour: shadow.colour,
                    inset: false,
                });
            }
            if cut {
                self.push(Decor::Unclip);
            }
        }

        // **Glass**, under a glass theme (`task-2281`): a raised surface (one with a shadow outside it)
        // whose fill can be seen through bends the picture behind it, and its fill, its edges and its
        // inner glow are painted over the bent picture. The rule is here rather than in a component, so
        // no component branches on the theme. An opaque fill hides the picture anyway, and a carved
        // well has no outer shadow, so neither is glass.
        if let Some(glass) = self.glass.filter(|_| has_outer && !fill.is_opaque()) {
            let size = rect.size();
            self.push(Decor::Glass {
                rect,
                corners,
                bevel: glass.bevel_for(size),
                strength: glass.strength_for(size),
            });
        }

        self.push(Decor::Rect { rect, corners, fill });

        let inner: Vec<_> = elevation.shadows().filter(|shadow| shadow.inset).collect();
        if !inner.is_empty() {
            self.push(Decor::Clip { rect, corners, outside: false, bound: rect });
            for shadow in inner.into_iter().rev() {
                self.push(Decor::Shadow {
                    rect: rect
                        .translate(Vec2::new(shadow.dx, shadow.dy))
                        .shrink(shadow.spread),
                    radius: (representative - shadow.spread).max(0.0),
                    deviation: shadow.deviation(),
                    colour: shadow.colour,
                    inset: true,
                });
            }
            self.push(Decor::Unclip);
        }
    }

    /// A coloured halo behind something: `box-shadow: 0 0 6px var(--accent-coral)` under a status dot,
    /// and the blue under the primary button.
    pub fn glow(&self, rect: Rect, radius: f32, colour: Color32, blur: f32) {
        self.push(Decor::Shadow {
            rect,
            radius,
            deviation: (blur / 2.0).max(0.01),
            colour,
            inset: false,
        });
    }

    pub fn disc(&self, centre: Pos2, radius: f32, fill: impl Into<Fill>) {
        self.push(Decor::Disc { centre, radius, fill: fill.into() });
    }

    pub fn ring(&self, centre: Pos2, radius: f32, width: f32, colour: Color32) {
        self.push(Decor::Ring { centre, radius, width, colour });
    }

    /// Part of a circle, stroked. Angles in radians, clockwise from twelve o'clock.
    pub fn arc(
        &self,
        centre: Pos2,
        radius: f32,
        width: f32,
        start: f32,
        sweep: f32,
        fill: impl Into<Fill>,
    ) {
        self.push(Decor::Arc { centre, radius, width, start, sweep, fill: fill.into(), round: true });
    }

    /// The same arc with square ends, for a slice too short to carry round caps. See [`Decor::Arc`].
    pub fn arc_square(
        &self,
        centre: Pos2,
        radius: f32,
        width: f32,
        start: f32,
        sweep: f32,
        fill: impl Into<Fill>,
    ) {
        self.push(Decor::Arc {
            centre,
            radius,
            width,
            start,
            sweep,
            fill: fill.into(),
            round: false,
        });
    }

    pub fn line(&self, from: Pos2, to: Pos2, width: f32, colour: Color32) {
        self.push(Decor::Line { from, to, width, colour });
    }

    /// A one-point hairline across the whole width of `rect`, at its top edge.
    pub fn hairline_top(&self, rect: Rect, colour: Color32) {
        self.line(rect.left_top(), rect.right_top(), 1.0, colour);
    }

    /// Everything drawn until [`Self::unclip`] is cut to this rounded rectangle.
    ///
    /// The one thing `egui` cannot do at all: its clip rectangle is axis-aligned and square, so a row
    /// scrolled to the bottom of a panel pokes its own square corner out of the panel's curve.
    pub fn clip(&self, rect: Rect, radius: impl Into<Corners>) {
        self.push(Decor::Clip { rect, corners: radius.into(), outside: false, bound: rect });
    }

    pub fn unclip(&self) {
        self.push(Decor::Unclip);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{light, Shadow};

    fn a_rect() -> Rect {
        Rect::from_min_size(Pos2::new(10.0, 20.0), Vec2::new(100.0, 40.0))
    }

    #[test]
    fn a_raised_surface_records_the_stylesheet_s_two_shadows_at_the_stylesheet_s_offsets() {
        let chrome = Chrome::recording();
        let theme = light();
        chrome.surface(a_rect(), 12.0, theme.surface.s1, theme.elevation.raised);
        let items = chrome.take();
        // clip, two shadows, unclip, the surface.
        assert_eq!(items.len(), 5);
        let Decor::Shadow { rect, deviation, colour, inset, .. } = items[1] else {
            panic!("expected a shadow, got {:?}", items[1]);
        };
        // The list is replayed in reverse, so the *second* declaration — `6px 6px 14px shadow-dark` —
        // is drawn first and ends up underneath.
        assert!(!inset);
        assert_eq!(colour, theme.shadow.dark);
        assert_eq!(deviation, 7.0);
        assert_eq!(rect.min, Pos2::new(16.0, 26.0));
    }

    #[test]
    fn an_outer_shadow_is_cut_to_its_band_only_when_the_surface_is_opaque() {
        let theme = light();
        let opaque = Chrome::recording();
        opaque.surface(a_rect(), 12.0, theme.surface.s1, theme.elevation.raised);
        assert!(matches!(opaque.take()[0], Decor::Clip { outside: true, .. }));

        let translucent = Chrome::recording();
        translucent.surface(
            a_rect(),
            12.0,
            Color32::from_rgba_unmultiplied(0, 0, 0, 128),
            theme.elevation.raised,
        );
        // Straight to the shadow: a surface you can see through does not cover its own.
        assert!(matches!(translucent.take()[0], Decor::Shadow { .. }));
    }

    #[test]
    fn an_inner_shadow_is_clipped_to_the_border_box() {
        let chrome = Chrome::recording();
        let theme = light();
        chrome.surface(a_rect(), 12.0, theme.surface.s1, theme.elevation.pressed);
        let items = chrome.take();
        // The surface, then a clip, two shadows and the unclip.
        assert!(matches!(items[0], Decor::Rect { .. }));
        assert!(matches!(items[1], Decor::Clip { outside: false, .. }));
        assert!(matches!(items[2], Decor::Shadow { inset: true, .. }));
        assert!(matches!(items[3], Decor::Shadow { inset: true, .. }));
        assert!(matches!(items[4], Decor::Unclip));
    }

    #[test]
    fn a_spread_grows_the_shadow_s_own_shape_before_it_is_blurred() {
        let chrome = Chrome::recording();
        chrome.surface(
            a_rect(),
            10.0,
            Color32::WHITE,
            Elevation::of([Shadow::outer(0.0, 0.0, 0.0, Color32::BLACK).spread(4.0)]),
        );
        let items = chrome.take();
        let Decor::Shadow { rect, radius, .. } = items[1] else { panic!("expected a shadow") };
        assert_eq!(rect, a_rect().expand(4.0));
        assert_eq!(radius, 14.0);
    }

    #[test]
    fn a_chrome_that_is_off_records_nothing() {
        static OFF: Chrome = Chrome::off();
        OFF.surface(a_rect(), 12.0, Color32::WHITE, light().elevation.raised);
        assert!(OFF.take().is_empty());
        assert!(!OFF.is_recording());
    }
}
