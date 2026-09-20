//! What a component is allowed to know about the device it is being drawn on.
//!
//! Three facts, and nothing else: how wide the screen is, what is covering its edges, and whether the
//! thing pointing at it is a finger. A component reads them through [`crate::Rux::screen`] and decides
//! its own size; nothing here reaches back into a component.
//!
//! `Screen::desktop` is a mouse with no insets and no minimum target, which is exactly what this
//! library did before it knew about phones — so every measurement on a desktop is the number it always
//! was, and the reference tests prove that rather than this comment.

use egui::{Rect, Vec2};

/// What is pointing at the screen.
///
/// There is no third case. A stylus and a trackpad both behave like a mouse for the two things this
/// distinction decides — whether there is a hover state and how big a target has to be.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Pointer {
    #[default]
    Mouse,
    Touch,
}

impl Pointer {
    /// The smallest square a person can reliably hit.
    ///
    /// **44 points, which is Apple's number.** Material asks for 48 and both are defensible; 44 is
    /// chosen because it is the smaller of the two and so leaves the reference's own 40 point
    /// `.toolbtn` closest to the size it was drawn at. A control never shrinks to meet this — it only
    /// grows.
    pub fn min_target(self) -> f32 {
        match self {
            Pointer::Mouse => 0.0,
            Pointer::Touch => 44.0,
        }
    }

    /// Whether a control has an idle-but-pointed-at state at all.
    ///
    /// A finger has no hover: it is either on the control or not on the screen. Nineteen rules in
    /// `reference/image-creator.css` change a colour on `:hover`, and a touch pointer keeps its last
    /// position after the finger lifts — so without this the wash stays behind on whatever was tapped.
    pub fn hovers(self) -> bool {
        matches!(self, Pointer::Mouse)
    }
}

/// How much room there is across the screen, in Material's own three bands.
///
/// The numbers are Material's window size classes, because they are the published breakpoints the rest
/// of the industry lays out against: **compact under 600 points, medium 600 to 840, expanded 840 and
/// up**. Every phone in portrait is compact, a phone in landscape and a small tablet are medium, and a
/// desktop window is expanded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SizeClass {
    Compact,
    Medium,
    Expanded,
}

impl SizeClass {
    pub const COMPACT_MAX: f32 = 600.0;
    pub const MEDIUM_MAX: f32 = 840.0;

    pub fn of(width: f32) -> Self {
        if width < Self::COMPACT_MAX {
            SizeClass::Compact
        } else if width < Self::MEDIUM_MAX {
            SizeClass::Medium
        } else {
            SizeClass::Expanded
        }
    }
}

/// What is covering the edges of the screen: a notch, a status bar, a home indicator.
///
/// On iOS these come from `UIWindow.safeAreaInsets`, which `egui-winit` reads through `objc2-ui-kit`
/// and hands to egui as `RawInput::safe_area_insets`. On Android they are the display cutout and the
/// navigation bar. On a desktop they are all zero.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Insets {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Insets {
    pub const NONE: Self = Self { top: 0.0, right: 0.0, bottom: 0.0, left: 0.0 };

    pub const fn new(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self { top, right, bottom, left }
    }

    /// Just the two that nearly always matter: the notch and the home indicator.
    pub const fn vertical(top: f32, bottom: f32) -> Self {
        Self { top, right: 0.0, bottom, left: 0.0 }
    }

    pub fn is_none(self) -> bool {
        self == Self::NONE
    }

    /// The rectangle left once the insets are taken off.
    pub fn inside(self, rect: Rect) -> Rect {
        Rect::from_min_max(
            egui::pos2(rect.min.x + self.left, rect.min.y + self.top),
            egui::pos2(rect.max.x - self.right, rect.max.y - self.bottom),
        )
    }
}

impl From<egui::SafeAreaInsets> for Insets {
    fn from(insets: egui::SafeAreaInsets) -> Self {
        Self {
            top: insets.0.top,
            right: insets.0.right,
            bottom: insets.0.bottom,
            left: insets.0.left,
        }
    }
}

/// The device, as far as a component is concerned.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Screen {
    /// The whole window, including anything behind a notch.
    pub size: Vec2,
    pub insets: Insets,
    pub pointer: Pointer,
}

impl Default for Screen {
    fn default() -> Self {
        Self::desktop(Vec2::new(1440.0, 900.0))
    }
}

impl Screen {
    /// A mouse, no insets, no minimum target — the behaviour this library had before it knew about
    /// phones.
    pub fn desktop(size: Vec2) -> Self {
        Self { size, insets: Insets::NONE, pointer: Pointer::Mouse }
    }

    /// A touch screen with the edges its device covers.
    pub fn phone(size: Vec2, insets: Insets) -> Self {
        Self { size, insets, pointer: Pointer::Touch }
    }

    /// Read it off egui.
    ///
    /// **A mobile application should call [`Screen::phone`] instead, or pass its own pointer.**
    /// `has_touch_screen` only becomes true once a touch has actually happened, so on a phone's first
    /// frame this answers `Mouse` and every control is laid out small, then jumps a frame later when
    /// the first finger lands. This is the right call for a desktop program that might also be run on
    /// a touch screen; it is the wrong one for a program that knows it is on a phone.
    pub fn read(ctx: &egui::Context) -> Self {
        let (size, insets, touch) = ctx.input(|input| {
            (
                input.viewport_rect().size(),
                Insets::from(input.safe_area_insets()),
                input.has_touch_screen(),
            )
        });
        Self {
            size,
            insets,
            pointer: match touch {
                true => Pointer::Touch,
                false => Pointer::Mouse,
            },
        }
    }

    /// The same screen with its pointer overridden, which is what the preview harness does.
    pub fn with_pointer(mut self, pointer: Pointer) -> Self {
        self.pointer = pointer;
        self
    }

    pub fn with_insets(mut self, insets: Insets) -> Self {
        self.insets = insets;
        self
    }

    pub fn class(&self) -> SizeClass {
        SizeClass::of(self.size.x)
    }

    pub fn is_compact(&self) -> bool {
        self.class() == SizeClass::Compact
    }

    /// The smallest square a control may claim. Zero on a mouse.
    pub fn min_target(&self) -> f32 {
        self.pointer.min_target()
    }

    /// Grow a size so neither side is under the minimum target.
    pub fn target(&self, size: Vec2) -> Vec2 {
        let min = self.min_target();
        Vec2::new(size.x.max(min), size.y.max(min))
    }

    /// Grow just the height, which is what a control that owns its width wants.
    pub fn target_height(&self, height: f32) -> f32 {
        height.max(self.min_target())
    }

    /// The part of `rect` nothing is covering.
    pub fn safe(&self, rect: Rect) -> Rect {
        self.insets.inside(rect)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_breakpoints_are_material_s_window_size_classes() {
        assert_eq!(SizeClass::of(0.0), SizeClass::Compact);
        assert_eq!(SizeClass::of(390.0), SizeClass::Compact);
        assert_eq!(SizeClass::of(599.9), SizeClass::Compact);
        assert_eq!(SizeClass::of(600.0), SizeClass::Medium);
        assert_eq!(SizeClass::of(839.9), SizeClass::Medium);
        assert_eq!(SizeClass::of(840.0), SizeClass::Expanded);
        assert_eq!(SizeClass::of(1440.0), SizeClass::Expanded);
    }

    #[test]
    fn a_desktop_screen_asks_for_nothing_at_all() {
        let screen = Screen::desktop(Vec2::new(1440.0, 900.0));
        assert_eq!(screen.min_target(), 0.0);
        assert!(screen.insets.is_none());
        assert!(screen.pointer.hovers());
        assert_eq!(screen.class(), SizeClass::Expanded);
        // The whole point: growing a size on a desktop changes nothing.
        assert_eq!(screen.target(Vec2::new(28.0, 28.0)), Vec2::new(28.0, 28.0));
        assert_eq!(screen.target_height(34.0), 34.0);
    }

    #[test]
    fn a_phone_grows_a_target_to_forty_four_and_never_shrinks_one() {
        let screen = Screen::phone(Vec2::new(393.0, 852.0), Insets::vertical(59.0, 34.0));
        assert_eq!(screen.min_target(), 44.0);
        assert!(!screen.pointer.hovers());
        assert!(screen.is_compact());
        assert_eq!(screen.target(Vec2::new(28.0, 28.0)), Vec2::new(44.0, 44.0));
        // A 48 point control is already legal and is left alone.
        assert_eq!(screen.target(Vec2::new(48.0, 48.0)), Vec2::new(48.0, 48.0));
        assert_eq!(screen.target_height(34.0), 44.0);
        assert_eq!(screen.target_height(52.0), 52.0);
    }

    #[test]
    fn the_safe_area_is_the_rect_less_the_insets() {
        let whole = Rect::from_min_size(egui::pos2(0.0, 0.0), Vec2::new(393.0, 852.0));
        let phone = Screen::phone(whole.size(), Insets::vertical(59.0, 34.0));
        let safe = phone.safe(whole);
        assert_eq!(safe.top(), 59.0);
        assert_eq!(safe.bottom(), 852.0 - 34.0);
        assert_eq!(safe.width(), 393.0);

        // And a desktop screen hands back what it was given.
        let desk = Screen::desktop(whole.size());
        assert_eq!(desk.safe(whole), whole);
    }

    #[test]
    fn the_insets_conversion_reads_egui_s_own_margin() {
        let egui_insets = egui::SafeAreaInsets(egui::epaint::MarginF32 {
            top: 59.0,
            right: 1.0,
            bottom: 34.0,
            left: 2.0,
        });
        assert_eq!(Insets::from(egui_insets), Insets::new(59.0, 1.0, 34.0, 2.0));
    }
}
