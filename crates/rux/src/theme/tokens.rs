//! The values a theme is made of: colours, radii, shadows and gradients.
//!
//! Every type here is a transcription of a CSS custom property in `reference/neumorphic-tokens.css`,
//! and the names are that file's names with the dashes turned into Rust. That is deliberate: somebody
//! reading a rule in `reference/image-creator.css` should be able to find the value here without a
//! translation table.
//!
//! **The list is closed.** A theme says what a name *means*; it cannot add a name. That is what makes it
//! true that a component reads `theme.ink.i500` and gets the right colour in both themes, and it is why
//! there is no `Color32::from_rgb` anywhere in `components/`.

use egui::Color32;

/// An opaque colour written the way CSS writes one: `rgb(0xECEFF3)`.
pub const fn rgb(hex: u32) -> Color32 {
    Color32::from_rgb((hex >> 16) as u8, (hex >> 8) as u8, hex as u8)
}

/// A colour at an opacity, which is how every `rgba(…)` in the reference is written.
pub fn rgba(hex: u32, opacity: f32) -> Color32 {
    with_opacity(rgb(hex), opacity)
}

/// The same colour at a different opacity.
///
/// `Color32` is premultiplied, so this un-premultiplies, replaces the alpha and premultiplies again —
/// which is what lets a component write `theme.gradient.primary.end.at(0.35)` and mean exactly what
/// `rgba(29, 79, 219, 0.35)` means in the stylesheet.
pub fn with_opacity(colour: Color32, opacity: f32) -> Color32 {
    let [r, g, b, _] = colour.to_srgba_unmultiplied();
    let alpha = (opacity.clamp(0.0, 1.0) * 255.0).round() as u8;
    Color32::from_rgba_unmultiplied(r, g, b, alpha)
}

/// Sugar so a shadow recipe reads like the declaration it came from.
pub trait Opacity {
    fn at(self, opacity: f32) -> Color32;
}

impl Opacity for Color32 {
    fn at(self, opacity: f32) -> Color32 {
        with_opacity(self, opacity)
    }
}

/// `--surface-0` … `--surface-sunken`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Surfaces {
    /// Deepest panel base and the canvas backdrop.
    pub s0: Color32,
    /// The body's own background — the main neumorphic canvas everything is lit against.
    pub s1: Color32,
    /// Slightly raised.
    pub s2: Color32,
    /// Raised: modals and popovers.
    pub s3: Color32,
    /// Recessed wells, slider tracks, hairline dividers between sections.
    pub sunken: Color32,
}

/// `--shadow-light`, `--shadow-dark`, `--shadow-darker`.
///
/// In dark neumorphism the "light" one is a **lifted grey rather than white**, which the incognito
/// stylesheet says about itself. That one substitution is most of what makes the whole system re-skin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShadowTones {
    pub light: Color32,
    pub dark: Color32,
    pub darker: Color32,
}

/// `--ink-900` … `--ink-200`: text and icons, darkest to faintest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ink {
    pub i900: Color32,
    pub i700: Color32,
    pub i500: Color32,
    pub i400: Color32,
    pub i300: Color32,
    pub i200: Color32,
}

/// `--accent-*`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Accents {
    pub blue: Color32,
    pub blue_deep: Color32,
    pub blue_soft: Color32,
    pub coral: Color32,
    pub violet: Color32,
    pub mint: Color32,
    pub amber: Color32,
    pub rose: Color32,
}

/// `--success`, `--warning`, `--danger`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Semantic {
    pub success: Color32,
    pub warning: Color32,
    pub danger: Color32,
}

/// `--r-sm` … `--r-pill`.
///
/// `pill` is the stylesheet's `999px`, and it stays that number rather than becoming "half the height":
/// the renderer clamps every radius to half the shorter side, so one value is right at every size.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Radii {
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
    pub xxl: f32,
    pub pill: f32,
}

impl Radii {
    /// The reference's own six, which no theme changes.
    pub const REFERENCE: Self =
        Self { sm: 8.0, md: 12.0, lg: 18.0, xl: 24.0, xxl: 32.0, pill: 999.0 };
}

/// One CSS `box-shadow`.
///
/// The fields are the declaration's, in the declaration's order: `<dx> <dy> <blur> <spread> <colour>`,
/// with `inset` as a flag. Nothing is converted here — the conversion to a Gaussian's standard
/// deviation happens once, in the renderer, so the value a reader compares against the stylesheet is
/// the value that is stored.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shadow {
    pub dx: f32,
    pub dy: f32,
    /// The CSS blur **radius**. The renderer halves it, because CSS defines the radius as twice the
    /// Gaussian's standard deviation.
    pub blur: f32,
    /// How much larger than the box the shadow's own shape is, before it is blurred.
    pub spread: f32,
    pub colour: Color32,
    pub inset: bool,
}

impl Shadow {
    /// An outer shadow: `<dx> <dy> <blur> <colour>`.
    pub const fn outer(dx: f32, dy: f32, blur: f32, colour: Color32) -> Self {
        Self { dx, dy, blur, spread: 0.0, colour, inset: false }
    }

    /// An inner shadow: `inset <dx> <dy> <blur> <colour>`.
    pub const fn inset(dx: f32, dy: f32, blur: f32, colour: Color32) -> Self {
        Self { dx, dy, blur, spread: 0.0, colour, inset: true }
    }

    /// The same shadow with a spread, which is the fourth length in the declaration.
    pub const fn spread(mut self, spread: f32) -> Self {
        self.spread = spread;
        self
    }

    /// The Gaussian standard deviation this blur radius means.
    ///
    /// CSS defines the blur radius as **twice** the standard deviation, and
    /// `vello_cpu::fill_blurred_rounded_rect` takes the standard deviation. `--e-raised`'s `14px` is
    /// therefore `σ = 7`.
    pub fn deviation(self) -> f32 {
        (self.blur / 2.0).max(0.01)
    }
}

/// A whole `box-shadow` declaration: up to six shadows, in the order they are written.
///
/// Six because `--e-modal` has five and the deepest recipe on a button has four. `Copy`, so an
/// elevation is passed by value everywhere and a `Theme` stays a plain value.
///
/// **Order matters and is the stylesheet's.** CSS paints the *first* shadow in the list on top of the
/// ones after it, so the renderer replays the list in reverse.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Elevation {
    shadows: [Option<Shadow>; 6],
}

impl Elevation {
    /// No shadow at all — a flat surface.
    pub const NONE: Self = Self { shadows: [None; 6] };

    /// The declaration, written in the order CSS writes it.
    pub fn of(shadows: impl IntoIterator<Item = Shadow>) -> Self {
        let mut slots = [None; 6];
        for (slot, shadow) in slots.iter_mut().zip(shadows) {
            *slot = Some(shadow);
        }
        Self { shadows: slots }
    }

    /// The shadows, in the order they were declared.
    pub fn shadows(&self) -> impl DoubleEndedIterator<Item = Shadow> + '_ {
        self.shadows.iter().flatten().copied()
    }

    /// Whether there is anything to draw.
    pub fn is_empty(&self) -> bool {
        self.shadows[0].is_none()
    }

    /// How far outside its own edge this elevation's outer shadows reach.
    ///
    /// A caller has to leave room for it: a layer's canvas is cut to the layer, so a surface closer to
    /// the edge than this has its shadow clipped and reads as stuck to the side rather than as
    /// floating. `2.5` standard deviations is where `vello_cpu` itself cuts the Gaussian off.
    pub fn reach(&self) -> f32 {
        self.shadows()
            .filter(|shadow| !shadow.inset)
            .map(|shadow| {
                shadow.dx.abs().max(shadow.dy.abs()) + shadow.spread + shadow.deviation() * 2.5
            })
            .fold(0.0, f32::max)
    }
}

/// A CSS `linear-gradient(<angle>, <from>, <to>)`.
///
/// The angle is kept in CSS's own convention — degrees clockwise from "to top" — so a reader can check
/// it against the declaration. The conversion to an axis happens in [`Gradient::axis`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Gradient {
    pub angle: f32,
    pub from: Color32,
    pub to: Color32,
}

impl Gradient {
    /// `linear-gradient(<angle>deg, <from>, <to>)`.
    pub const fn at(angle: f32, from: Color32, to: Color32) -> Self {
        Self { angle, from, to }
    }

    /// `linear-gradient(<from>, <to>)`, which CSS reads as `180deg` — top to bottom.
    pub const fn down(from: Color32, to: Color32) -> Self {
        Self { angle: 180.0, from, to }
    }

    /// The two ends of the gradient's axis across `rect`, in the window's own coordinates.
    ///
    /// CSS puts the gradient line through the box's centre, pointing at the end colour, and makes it
    /// long enough that the corners are fully covered: `|W·sin a| + |H·cos a|`. With `y` pointing down
    /// the screen, `0deg` (to top) is `(0, -1)`, `90deg` (to right) is `(1, 0)` and `180deg` (to
    /// bottom) is `(0, 1)` — which is the direction vector `(sin a, -cos a)`.
    pub fn axis(self, rect: egui::Rect) -> (egui::Pos2, egui::Pos2) {
        let radians = self.angle.to_radians();
        let direction = egui::vec2(radians.sin(), -radians.cos());
        let length =
            (rect.width() * radians.sin()).abs() + (rect.height() * radians.cos()).abs();
        let half = direction * (length / 2.0);
        (rect.center() - half, rect.center() + half)
    }
}

/// `--grad-*`, plus the handful the page's own stylesheet writes inline.
///
/// The page-specific ones are here rather than in the components that use them because more than one
/// component uses each: the mint gradient is on the preset chip *and* the primary footer button, and
/// the diagonal blue is on the project icon, the `+` badge and the active timeline segment.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Gradients {
    /// `--grad-primary` — the Create button and the render button.
    pub primary: Gradient,
    /// `--grad-primary-160` — the project icon, the `+` badge, an active timeline segment.
    pub primary_diagonal: Gradient,
    /// `--grad-danger`.
    pub danger: Gradient,
    /// `linear-gradient(160deg, #ff8f82, #e2433f)` — the dock's primary button.
    pub danger_diagonal: Gradient,
    /// `linear-gradient(160deg, #3ad9af, #1fa37e)` — the preset chip and the modal's Save button.
    pub mint: Gradient,
    /// `linear-gradient(160deg, var(--accent-violet), #6d4fd4)` — the dice toggle when it is on.
    pub violet: Gradient,
    /// `--grad-nav-active` — the selected top-nav item.
    pub nav_active: Gradient,
    /// `--grad-knob` — a slider thumb or a toggle knob.
    pub knob: Gradient,
    /// `linear-gradient(160deg, #a88cff, #6b49e8)` — an avatar with no picture.
    pub avatar: Gradient,
    /// `linear-gradient(160deg, #1a202d 0%, #0c1018 100%)` — an image dropzone, which is dark in
    /// both themes because it stands in for a picture.
    pub image_well: Gradient,
    /// `--grad-progress`.
    pub progress: Gradient,
    /// `--grad-multistop`, as its two ends; the spinner's arc uses it.
    pub multistop: Gradient,
}

/// Every named elevation a component can ask for.
///
/// The first seven are the token file's. The rest are recipes the page's stylesheet writes more than
/// once, built from the theme's own colours so that they shift with it — `rgba(29, 79, 219, 0.35)`
/// under the primary button is `--accent-blue-deep` at `0.35`, and in the dark theme that is a
/// different blue.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Elevations {
    pub raised_sm: Elevation,
    pub raised: Elevation,
    pub raised_lg: Elevation,
    pub pressed_sm: Elevation,
    pub pressed: Elevation,
    pub pressed_deep: Elevation,
    /// `--e-modal`: a purely downward, layered ambient shadow plus a hairline lit top edge. Unlike the
    /// raised tokens it does **not** pair a bright top-left glow with a grey bottom-right shadow, which
    /// on a dimmed backdrop reads as a bleeding halo.
    pub modal: Elevation,
    /// `.ic-create` — the large primary button.
    pub primary: Elevation,
    /// `.ic-create:active` — pressed into itself rather than merely dimmer.
    pub primary_pressed: Elevation,
    /// `.render-btn` and `.dock__btn--primary`'s shape: a smaller primary.
    pub primary_sm: Elevation,
    /// `.topnav__item.is-active`.
    pub nav_active: Elevation,
    /// `.scene__image` — a dark well that is both pressed into itself and raised off the page.
    pub image_well: Elevation,
}
