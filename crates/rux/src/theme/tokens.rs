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

/// The values the Agent-Chat design draws in (`task-2235`).
///
/// That design is lit from the top left like the rest of the library, but harder: a card is a
/// diagonal gradient extruded from the pane with a wide pair of shadows, a well is carved into it with
/// the same pair inset, and every control is round or a pill. Its source is the Claude Design canvas
/// "Unluminous", page Agent-Chat, which writes the dark values as literal hex; [`Chat::lit_from`] is
/// those values written as steps from the pane's own colour, so the same design follows any ground.
///
/// Only colours are kept here. The shadows are made from them by the methods below, so both themes
/// declare the same recipes, which is the property [`super::Theme`] exists to keep.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Chat {
    /// The pane the design sits on. The focus ring's gap is painted in it.
    pub ground: Color32,
    /// `--card: linear-gradient(145deg, #252a32, #1c2026)`: every card, bubble and round button.
    pub card: Gradient,
    /// `--well: #1a1e24`: everything carved in, from the prompt box to a slider's groove.
    pub well: Color32,
    /// The lit half of every shadow pair: `#2a2f37`.
    pub light: Color32,
    /// The dark half of every shadow pair: `#0c0f13`.
    pub dark: Color32,
    /// `--grad: linear-gradient(135deg, #3d78ff, #5cb6ff)`: the one action, and every data series.
    pub action: Gradient,
    /// `--grad-hover`: the action under the pointer.
    pub action_hover: Gradient,
    /// The blue glow under the action: `rgba(20, 60, 160, 0.45)`.
    pub action_glow: Color32,
    /// The pale edge inside a pressed action: `rgba(140, 200, 255, 0.25)`.
    pub action_sheen: Color32,
    /// `--sky: #5cb6ff`: a control under the pointer, a running arc, the focus ring.
    pub sky: Color32,
    /// The empty part of a small ring: `#2c323b`.
    pub track: Color32,
    /// The quiet series a comparison is made against: `#3a404a`.
    pub quiet: Color32,
    /// The second and third slices of a ring chart: `#4a515c` and `#363c46`.
    pub slices: [Color32; 2],
    /// `--up: #ff8f73`: slower, failed, a line taken out.
    pub slower: Color32,
    /// `--down: #3ad6ad`: faster, done, a line put in.
    pub faster: Color32,
    /// `--wait: #ffc062`: waiting for somebody's OK.
    pub waiting: Color32,
    /// `--chip: linear-gradient(145deg, #262b33, #20242b)`: the Agent-Tasks design's small raised
    /// surfaces, flatter than a card: a lane's mark and count, a chosen view, a card's round buttons.
    pub chip: Gradient,
    /// The lit half of a chip's shadow pair: `rgba(46, 52, 61, 0.7)`.
    pub chip_light: Color32,
    /// The dark half of a chip's shadow pair: `#0b0d11`.
    pub chip_dark: Color32,
    /// The hairline inside a chip's edge: `inset 0 0 0 1px rgba(255, 255, 255, 0.045)`.
    pub chip_edge: Color32,
    /// The lit line along a chip's top: `inset 0 1px 0 rgba(255, 255, 255, 0.05)`.
    pub chip_top: Color32,
    /// `linear-gradient(135deg, #2fb98f, #3ad6ad)`: a track whose work is all done.
    pub done: Gradient,
}

impl Chat {
    /// The design's values for a pane of colour `ground`.
    ///
    /// For a dark ground each colour is the canvas's own value written as a step from the canvas's
    /// pane, `#1f232a`, so on that pane this answers exactly the hex the canvas writes and on any other
    /// dark pane it answers the same steps. A light ground has no canvas of its own; its steps are the
    /// same design turned the right way up for a light surface, with darker accents so that words in
    /// them stay readable on it.
    pub fn lit_from(ground: Color32, dark: bool) -> Self {
        let step = |colour: Color32, by: [i16; 3]| {
            let channel = |value: u8, by: i16| (i16::from(value) + by).clamp(0, 255) as u8;
            Color32::from_rgb(
                channel(colour.r(), by[0]),
                channel(colour.g(), by[1]),
                channel(colour.b(), by[2]),
            )
        };
        match dark {
            true => Self {
                ground,
                card: Gradient::at(145.0, step(ground, [6, 7, 8]), step(ground, [-3, -3, -4])),
                well: step(ground, [-5, -5, -6]),
                light: step(ground, [11, 12, 13]),
                dark: step(ground, [-19, -20, -23]),
                action: Gradient::at(135.0, rgb(0x3D78FF), rgb(0x5CB6FF)),
                action_hover: Gradient::at(135.0, rgb(0x4D86FF), rgb(0x70C2FF)),
                action_glow: rgba(0x143CA0, 0.45),
                action_sheen: rgba(0x8CC8FF, 0.25),
                sky: rgb(0x5CB6FF),
                track: step(ground, [13, 15, 17]),
                quiet: step(ground, [27, 29, 32]),
                slices: [step(ground, [43, 46, 50]), step(ground, [23, 25, 28])],
                slower: rgb(0xFF8F73),
                faster: rgb(0x3AD6AD),
                waiting: rgb(0xFFC062),
                chip: Gradient::at(145.0, step(ground, [7, 8, 9]), step(ground, [1, 1, 1])),
                chip_light: step(ground, [15, 17, 19]).at(0.7),
                chip_dark: step(ground, [-20, -22, -25]),
                chip_edge: Color32::WHITE.at(0.045),
                chip_top: Color32::WHITE.at(0.05),
                done: Gradient::at(135.0, rgb(0x2FB98F), rgb(0x3AD6AD)),
            },
            false => Self {
                ground,
                card: Gradient::at(145.0, step(ground, [10, 9, 7]), step(ground, [-6, -6, -5])),
                well: step(ground, [-5, -5, -4]),
                light: rgb(0xFFFFFF),
                dark: step(ground, [-44, -40, -34]),
                action: Gradient::at(135.0, rgb(0x2F6BFF), rgb(0x4AA3FF)),
                action_hover: Gradient::at(135.0, rgb(0x447BFF), rgb(0x62B1FF)),
                action_glow: rgba(0x1E50C8, 0.35),
                action_sheen: rgba(0xC8E4FF, 0.35),
                sky: rgb(0x2A7FE0),
                track: step(ground, [-18, -16, -13]),
                quiet: step(ground, [-34, -31, -27]),
                slices: [step(ground, [-52, -47, -40]), step(ground, [-24, -22, -18])],
                slower: rgb(0xD9543C),
                faster: rgb(0x0F9A76),
                waiting: rgb(0xB97A0E),
                chip: Gradient::at(145.0, step(ground, [8, 7, 6]), step(ground, [-2, -2, -2])),
                chip_light: rgb(0xFFFFFF).at(0.9),
                chip_dark: step(ground, [-44, -40, -34]).at(0.75),
                chip_edge: Color32::BLACK.at(0.04),
                chip_top: Color32::WHITE.at(0.6),
                done: Gradient::at(135.0, rgb(0x0C8A69), rgb(0x0F9A76)),
            },
        }
    }

    /// `--out: -7px -7px 16px light, 8px 8px 18px dark`: a card.
    pub fn raised(&self) -> Elevation {
        Elevation::of([
            Shadow::outer(-7.0, -7.0, 16.0, self.light),
            Shadow::outer(8.0, 8.0, 18.0, self.dark),
        ])
    }

    /// `--out-sm: -4px -4px 9px light, 4px 4px 10px dark`: a bubble, a round button, a knob.
    pub fn raised_sm(&self) -> Elevation {
        Elevation::of([
            Shadow::outer(-4.0, -4.0, 9.0, self.light),
            Shadow::outer(4.0, 4.0, 10.0, self.dark),
        ])
    }

    /// The Agent-Tasks board's `--out-sm`: `-2px -2px 5px`, `3px 3px 6px` and a hairline inside the
    /// edge. Tighter than a bubble's, for a chip, a count, a chosen view and a card's round buttons.
    pub fn raised_chip(&self) -> Elevation {
        Elevation::of([
            Shadow::outer(-2.0, -2.0, 5.0, self.chip_light),
            Shadow::outer(3.0, 3.0, 6.0, self.chip_dark),
            Shadow::inset(0.0, 0.0, 0.0, self.chip_edge).spread(1.0),
            Shadow::inset(0.0, 1.0, 0.0, self.chip_top),
        ])
    }

    /// `--in: inset 5px 5px 10px dark, inset -5px -5px 10px light`: the prompt box, a dial's channel.
    pub fn carved(&self) -> Elevation {
        Elevation::of([
            Shadow::inset(5.0, 5.0, 10.0, self.dark),
            Shadow::inset(-5.0, -5.0, 10.0, self.light),
        ])
    }

    /// `--in-sm: inset 2px 2px 5px dark, inset -2px -2px 5px light`: a status well, a code block.
    pub fn carved_sm(&self) -> Elevation {
        Elevation::of([
            Shadow::inset(2.0, 2.0, 5.0, self.dark),
            Shadow::inset(-2.0, -2.0, 5.0, self.light),
        ])
    }

    /// `--grad-out`: the action, lifted, with its blue glow and a lit top edge.
    pub fn action_raised(&self) -> Elevation {
        Elevation::of([
            Shadow::outer(-4.0, -4.0, 9.0, self.light),
            Shadow::outer(4.0, 5.0, 12.0, self.action_glow),
            Shadow::inset(0.0, 1.0, 0.0, Color32::WHITE.at(0.25)),
        ])
    }

    /// `--grad-in`: the action, pressed into itself.
    pub fn action_pressed(&self) -> Elevation {
        Elevation::of([
            Shadow::inset(3.0, 3.0, 7.0, Color32::BLACK.at(0.35)),
            Shadow::inset(-2.0, -2.0, 5.0, self.action_sheen),
        ])
    }

    /// The Chat settings dialog: a lit edge up and left, and a long soft fall below.
    pub fn floating(&self) -> Elevation {
        Elevation::of([
            Shadow::outer(-6.0, -6.0, 16.0, self.light.at(0.6)),
            Shadow::outer(0.0, 24.0, 48.0, Color32::BLACK.at(0.55)),
        ])
    }

    /// `--ring`'s outer half: `rgba(92, 182, 255, 0.6)`.
    pub fn focus(&self) -> Color32 {
        self.sky.at(0.6)
    }
}
