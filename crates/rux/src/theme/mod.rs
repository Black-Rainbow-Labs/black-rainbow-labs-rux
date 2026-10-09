//! What a colour means, and the two themes that answer.
//!
//! A [`Theme`] is a plain value. Everything a component draws reads one, so re-skinning the whole
//! library is swapping the value — which is exactly what `reference/incognito-theme.css` does to
//! ai-service: it overrides surfaces, shadow tones, ink, accents and gradients and changes **no
//! component rule at all**. That property is the design, and it is kept here by the token list being
//! closed: a theme says what a name means and cannot add a name.
//!
//! Two themes ship, and both are transcriptions rather than interpretations. See [`light`] and
//! [`dark`].

pub mod tokens;

use std::sync::OnceLock;

use egui::Color32;

pub use tokens::{
    rgb, rgba, with_opacity, Accents, Elevation, Elevations, Gradient, Gradients, Ink, Opacity,
    Radii, Semantic, Shadow, ShadowTones, Surfaces,
};

/// The washes and hairlines a rule writes as a bare `rgba(…)` rather than as a token.
///
/// The reference writes `rgba(0, 0, 0, 0.025)` for a menu row's hover and `rgba(0, 0, 0, 0.04)` for a
/// hairline, which are fine on a light canvas and **invisible** on a dark one — which is why
/// `incognito-theme.css` has a whole section re-toning them to faint whites. They are tokens here so
/// that section is not needed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Interaction {
    /// `.proj__item:hover`, `.scene__title:hover` — a barely-there wash over a surface.
    pub hover: Color32,
    /// The same, one step stronger: a row that is being held.
    pub pressed: Color32,
    /// `border-top: 1px solid rgba(0,0,0,0.04)` and the `.topbar::after` rule.
    pub hairline: Color32,
    /// `.ic-select__item:hover` — an accent-tinted wash on a row that would be chosen.
    pub accent_wash: Color32,
    /// `.ic-select__item.is-selected` — the same, quieter, on the row that already is.
    pub accent_wash_soft: Color32,
    /// `.ic-modal-backdrop` — what a modal dims the page behind it with.
    pub scrim: Color32,
    /// The ring drawn round whatever has the keyboard. Not in the reference, which has no focus
    /// styling at all; a library that shipped without one would be a library nobody can use by keyboard.
    pub focus_ring: Color32,
}

/// One complete answer to "what does this name mean".
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Theme {
    /// The name a consumer asks for and the storybook's dropdown lists: `light-neumorphic`.
    pub name: &'static str,
    /// What that name is called in front of a person: `Light Neumorphic`.
    pub label: &'static str,
    /// Whether this is a dark theme. Read only where a component genuinely needs to know — an image
    /// dropzone is dark in both, so what changes is the ink drawn over it.
    pub dark: bool,
    pub surface: Surfaces,
    pub shadow: ShadowTones,
    pub ink: Ink,
    pub accent: Accents,
    pub semantic: Semantic,
    pub interaction: Interaction,
    pub radius: Radii,
    pub elevation: Elevations,
    pub gradient: Gradients,
}

impl Theme {
    /// A raised surface whose bottom-right shadow is a colour rather than the theme's grey.
    ///
    /// The shape `-2px -2px 4px light, <dx> <dy> <blur> <glow>, inset 1px 1px 1px white` is written
    /// four times in the reference — the dice toggle when it is on, the preset chip, the modal's Save
    /// button and the dock's primary button — each with a different accent under it. One function, so a
    /// fifth one cannot drift.
    pub fn glowing(&self, glow: Color32, lift: f32) -> Elevation {
        Elevation::of([
            Shadow::outer(-lift, -lift, lift * 2.0, self.shadow.light),
            Shadow::outer(lift, lift, lift * 3.0, glow),
            Shadow::inset(1.0, 1.0, 1.0, Color32::WHITE.at(0.25)),
        ])
    }

    /// This theme with another application's surfaces, ink and accent, and its elevations worked out
    /// again from them.
    ///
    /// For a consumer whose own palette is the truth and whose components have to sit beside its own
    /// surfaces: Unluminous draws a chat bubble in its theme's card colour, and a `rux` plate drawn beside it
    /// in `rux`'s grey reads as a second design. Every recipe is unchanged — only values move, which is the
    /// property [`Theme`] exists to keep. `shadow` is the pair the elevations are made from.
    pub fn retoned(
        &self,
        name: &'static str,
        surface: Surfaces,
        shadow: ShadowTones,
        ink: Ink,
        blue: Color32,
    ) -> Theme {
        let mut theme = *self;
        theme.name = name;
        theme.surface = surface;
        theme.shadow = shadow;
        theme.ink = ink;
        theme.accent.blue = blue;
        theme.interaction.accent_wash = with_opacity(blue, 0.06);
        theme.interaction.accent_wash_soft = with_opacity(blue, 0.04);
        theme.interaction.focus_ring = with_opacity(blue, 0.65);
        theme.elevation = elevations(&theme.shadow, &theme.accent, &theme.gradient);
        theme
    }

    /// Every theme that ships, in the order the storybook lists them.
    pub fn all() -> &'static [&'static Theme] {
        static ALL: OnceLock<[&'static Theme; 2]> = OnceLock::new();
        ALL.get_or_init(|| [light(), dark()])
    }

    /// The theme with this name, or `None`.
    ///
    /// A name that does not exist is refused rather than silently falling back, which is the rule the
    /// whole library keeps about naming something built in: the caller is told what there is.
    pub fn named(name: &str) -> Option<&'static Theme> {
        Theme::all().iter().copied().find(|theme| theme.name == name)
    }

    /// What `named` would have accepted, for a refusal that is useful.
    pub fn names() -> Vec<&'static str> {
        Theme::all().iter().map(|theme| theme.name).collect()
    }
}

impl Default for &'static Theme {
    fn default() -> Self {
        light()
    }
}

/// The light neumorphic theme: ai-service's normal mode.
///
/// Every value is `reference/neumorphic-tokens.css`, transcribed. The elevations below the token
/// file's seven come from `reference/image-creator.css`, where they are written out per rule.
pub fn light() -> &'static Theme {
    static THEME: OnceLock<Theme> = OnceLock::new();
    THEME.get_or_init(|| {
        let shadow =
            ShadowTones { light: rgb(0xFFFFFF), dark: rgb(0xB8BFCC), darker: rgb(0xA8B0BE) };
        let accent = Accents {
            blue: rgb(0x2F6BFF),
            blue_deep: rgb(0x1D4FDB),
            blue_soft: rgb(0x6F96FF),
            coral: rgb(0xFF6B5B),
            violet: rgb(0x8B6BFF),
            mint: rgb(0x2FCFA6),
            amber: rgb(0xFFB648),
            rose: rgb(0xFF4F7A),
        };
        let gradient = Gradients {
            primary: Gradient::down(rgb(0x3A78FF), rgb(0x1D4FDB)),
            primary_diagonal: Gradient::at(160.0, rgb(0x4A82FF), rgb(0x1D4FDB)),
            danger: Gradient::down(rgb(0xFF7A78), rgb(0xE2433F)),
            danger_diagonal: Gradient::at(160.0, rgb(0xFF8F82), rgb(0xE2433F)),
            mint: Gradient::at(160.0, rgb(0x3AD9AF), rgb(0x1FA37E)),
            violet: Gradient::at(160.0, accent.violet, rgb(0x6D4FD4)),
            nav_active: Gradient::at(165.0, rgb(0x2A3140), rgb(0x0F131C)),
            knob: Gradient::at(145.0, rgb(0xFFFFFF), rgb(0xE3E8EF)),
            avatar: Gradient::at(160.0, rgb(0xA88CFF), rgb(0x6B49E8)),
            image_well: Gradient::at(160.0, rgb(0x1A202D), rgb(0x0C1018)),
            progress: Gradient::at(90.0, rgb(0x2F6BFF), rgb(0x6F96FF)),
            multistop: Gradient::at(90.0, rgb(0x2F6BFF), rgb(0x8B6BFF)),
        };
        Theme {
            name: "light-neumorphic",
            label: "Light Neumorphic",
            dark: false,
            surface: Surfaces {
                s0: rgb(0xE4E9F0),
                s1: rgb(0xECEFF3),
                s2: rgb(0xF1F4F8),
                s3: rgb(0xF7F9FC),
                sunken: rgb(0xDCE1E9),
            },
            shadow,
            ink: Ink {
                i900: rgb(0x1E2530),
                i700: rgb(0x3A4454),
                i500: rgb(0x6B7686),
                i400: rgb(0x8A94A3),
                i300: rgb(0xAAB2BF),
                i200: rgb(0xC7CDD7),
            },
            accent,
            semantic: Semantic {
                success: rgb(0x2FB67C),
                warning: rgb(0xFFB648),
                danger: rgb(0xF25F5C),
            },
            interaction: Interaction {
                hover: rgba(0x000000, 0.025),
                pressed: rgba(0x000000, 0.055),
                hairline: rgba(0x000000, 0.04),
                accent_wash: with_opacity(accent.blue, 0.06),
                accent_wash_soft: with_opacity(accent.blue, 0.04),
                scrim: rgba(0x141924, 0.55),
                focus_ring: with_opacity(accent.blue, 0.65),
            },
            radius: Radii::REFERENCE,
            elevation: elevations(&shadow, &accent, &gradient),
            gradient,
        }
    })
}

/// The dark neumorphic theme: ai-service's incognito mode.
///
/// `reference/incognito-theme.css` over `reference/neumorphic-tokens.css`. **The recipes are
/// identical** — only the values differ — which is the property this whole module exists to keep.
///
/// The four page-level gradients the incognito file does not override keep their own colours here, for
/// the same reason they keep them there: they are written inline in `image-creator.css` with literal
/// hex, so the only part of them that moves is the token one of them names (`--accent-violet` in the
/// dice toggle).
pub fn dark() -> &'static Theme {
    static THEME: OnceLock<Theme> = OnceLock::new();
    THEME.get_or_init(|| {
        // In dark neumorphism the "light" highlight is a lifted grey, not white. The incognito
        // stylesheet says this about itself, and it is the single substitution that makes the system
        // re-skin with no component change.
        let shadow =
            ShadowTones { light: rgb(0x2F353E), dark: rgb(0x14171B), darker: rgb(0x0C0E11) };
        let accent = Accents {
            blue: rgb(0x4C86FF),
            blue_deep: rgb(0x2A5FE6),
            blue_soft: rgb(0x7FA3FF),
            coral: rgb(0xFF7A6B),
            violet: rgb(0x9B80FF),
            mint: rgb(0x34DBB0),
            amber: rgb(0xFFC062),
            rose: rgb(0xFF6791),
        };
        let gradient = Gradients {
            primary: Gradient::down(rgb(0x4C86FF), rgb(0x2A5FE6)),
            primary_diagonal: Gradient::at(160.0, rgb(0x5A90FF), rgb(0x2A5FE6)),
            danger: Gradient::down(rgb(0xFF8A88), rgb(0xE0413E)),
            // Not overridden by the incognito file: written inline in the page's own stylesheet.
            danger_diagonal: Gradient::at(160.0, rgb(0xFF8F82), rgb(0xE2433F)),
            mint: Gradient::at(160.0, rgb(0x3AD9AF), rgb(0x1FA37E)),
            violet: Gradient::at(160.0, accent.violet, rgb(0x6D4FD4)),
            nav_active: Gradient::at(165.0, rgb(0x3A78FF), rgb(0x1D4FDB)),
            knob: Gradient::at(145.0, rgb(0x363C46), rgb(0x21252C)),
            avatar: Gradient::at(160.0, rgb(0xA88CFF), rgb(0x6B49E8)),
            image_well: Gradient::at(160.0, rgb(0x1A202D), rgb(0x0C1018)),
            progress: Gradient::at(90.0, rgb(0x4C86FF), rgb(0x7FA3FF)),
            multistop: Gradient::at(90.0, rgb(0x4C86FF), rgb(0x9B80FF)),
        };
        Theme {
            name: "dark-neumorphic",
            label: "Dark Neumorphic",
            dark: true,
            surface: Surfaces {
                s0: rgb(0x1C1F25),
                s1: rgb(0x23272E),
                s2: rgb(0x282D35),
                s3: rgb(0x2F353E),
                sunken: rgb(0x191C21),
            },
            shadow,
            ink: Ink {
                i900: rgb(0xEDF0F5),
                i700: rgb(0xC9D0DA),
                i500: rgb(0x939DAC),
                i400: rgb(0x79828F),
                i300: rgb(0x5C6470),
                i200: rgb(0x3C434E),
            },
            accent,
            semantic: Semantic {
                success: rgb(0x36C98A),
                warning: rgb(0xFFC062),
                danger: rgb(0xFF6B68),
            },
            interaction: Interaction {
                // The light theme's near-black washes vanish on a dark surface, so they are faint
                // whites instead — which is the substitution `incognito-theme.css` makes by hand for
                // the Tailwind compatibility layer.
                hover: rgba(0xFFFFFF, 0.04),
                pressed: rgba(0xFFFFFF, 0.08),
                hairline: rgba(0xFFFFFF, 0.08),
                accent_wash: with_opacity(accent.blue, 0.10),
                accent_wash_soft: with_opacity(accent.blue, 0.07),
                scrim: rgba(0x080A0E, 0.55),
                focus_ring: with_opacity(accent.blue, 0.65),
            },
            radius: Radii::REFERENCE,
            elevation: elevations(&shadow, &accent, &gradient),
            gradient,
        }
    })
}

/// The twelve named elevations, built from whichever theme's colours are handed in.
///
/// One function for both themes, because the whole point is that the *recipes* do not differ. Each
/// entry names the rule in `reference/` it was read out of.
fn elevations(shadow: &ShadowTones, accent: &Accents, gradient: &Gradients) -> Elevations {
    Elevations {
        // --e-raised-sm
        raised_sm: Elevation::of([
            Shadow::outer(-3.0, -3.0, 6.0, shadow.light),
            Shadow::outer(3.0, 3.0, 6.0, shadow.dark),
        ]),
        // --e-raised
        raised: Elevation::of([
            Shadow::outer(-6.0, -6.0, 14.0, shadow.light),
            Shadow::outer(6.0, 6.0, 14.0, shadow.dark),
        ]),
        // --e-raised-lg
        raised_lg: Elevation::of([
            Shadow::outer(-10.0, -10.0, 24.0, shadow.light),
            Shadow::outer(12.0, 12.0, 24.0, shadow.dark),
        ]),
        // --e-pressed-sm
        pressed_sm: Elevation::of([
            Shadow::inset(2.0, 2.0, 4.0, shadow.dark),
            Shadow::inset(-2.0, -2.0, 4.0, shadow.light),
        ]),
        // --e-pressed
        pressed: Elevation::of([
            Shadow::inset(4.0, 4.0, 8.0, shadow.dark),
            Shadow::inset(-4.0, -4.0, 8.0, shadow.light),
        ]),
        // --e-pressed-deep
        pressed_deep: Elevation::of([
            Shadow::inset(6.0, 6.0, 12.0, shadow.darker),
            Shadow::inset(-6.0, -6.0, 12.0, shadow.light),
        ]),
        // --e-modal
        modal: Elevation::of([
            Shadow::inset(0.0, 1.0, 0.0, Color32::WHITE.at(0.7)),
            Shadow::outer(0.0, 1.0, 2.0, rgba(0x0F1626, 0.06)),
            Shadow::outer(0.0, 6.0, 16.0, rgba(0x0F1626, 0.12)),
            Shadow::outer(0.0, 18.0, 40.0, rgba(0x0F1626, 0.20)),
            Shadow::outer(0.0, 40.0, 80.0, rgba(0x0F1626, 0.16)),
        ]),
        // .ic-create
        primary: Elevation::of([
            Shadow::outer(-4.0, -4.0, 10.0, shadow.light),
            Shadow::outer(4.0, 4.0, 14.0, gradient.primary.to.at(0.35)),
            Shadow::inset(1.0, 1.0, 1.0, Color32::WHITE.at(0.28)),
            Shadow::inset(-1.0, -1.0, 2.0, Color32::BLACK.at(0.15)),
        ]),
        // .ic-create:active
        primary_pressed: Elevation::of([
            Shadow::inset(3.0, 3.0, 8.0, Color32::BLACK.at(0.25)),
            Shadow::inset(-2.0, -2.0, 4.0, accent.blue_soft.at(0.30)),
        ]),
        // .render-btn
        primary_sm: Elevation::of([
            Shadow::outer(-3.0, -3.0, 6.0, shadow.light),
            Shadow::outer(4.0, 4.0, 10.0, gradient.primary.to.at(0.35)),
            Shadow::inset(1.0, 1.0, 1.0, Color32::WHITE.at(0.25)),
        ]),
        // .topnav__item.is-active
        nav_active: Elevation::of([
            Shadow::outer(-2.0, -2.0, 5.0, shadow.light),
            Shadow::outer(3.0, 3.0, 8.0, gradient.nav_active.to.at(0.35)),
            Shadow::inset(1.0, 1.0, 2.0, Color32::WHITE.at(0.08)),
        ]),
        // .scene__image
        image_well: Elevation::of([
            Shadow::inset(4.0, 4.0, 10.0, Color32::BLACK.at(0.35)),
            Shadow::inset(-3.0, -3.0, 8.0, Color32::WHITE.at(0.02)),
            Shadow::outer(-3.0, -3.0, 8.0, shadow.light),
            Shadow::outer(3.0, 3.0, 10.0, shadow.dark),
        ]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_blur_radius_is_twice_the_gaussian_it_becomes() {
        // `--e-raised: … 14px …` is the widest of the three raised tokens, and CSS defines the blur
        // radius as twice the standard deviation.
        let raised = light().elevation.raised;
        assert_eq!(raised.shadows().next().unwrap().deviation(), 7.0);
    }

    #[test]
    fn both_themes_declare_the_same_recipes() {
        // Not the same *values* — the same shape. A dark theme that grew a shadow of its own would be
        // a dark theme that needs a component change, which is the thing this design refuses.
        for (light, dark) in [
            (light().elevation.raised, dark().elevation.raised),
            (light().elevation.pressed, dark().elevation.pressed),
            (light().elevation.primary, dark().elevation.primary),
            (light().elevation.nav_active, dark().elevation.nav_active),
        ] {
            let shape = |elevation: Elevation| {
                elevation
                    .shadows()
                    .map(|s| (s.dx, s.dy, s.blur, s.spread, s.inset))
                    .collect::<Vec<_>>()
            };
            assert_eq!(shape(light), shape(dark));
        }
    }

    #[test]
    fn a_gradient_axis_is_css_s_own_convention() {
        // Compared approximately, because `sin(180°)` is not exactly zero in binary floating point and
        // an axis that is a ten-thousandth of a point off vertical is the same gradient.
        let near = |at: egui::Pos2, x: f32, y: f32| {
            assert!((at.x - x).abs() < 0.001 && (at.y - y).abs() < 0.001, "{at:?} is not {x},{y}");
        };
        let rect = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(100.0, 100.0));
        // 180deg is "to bottom": the axis runs down the box.
        let (from, to) = Gradient::down(Color32::WHITE, Color32::BLACK).axis(rect);
        near(from, 50.0, 0.0);
        near(to, 50.0, 100.0);
        // 0deg is "to top", which is the other way up.
        let (from, to) = Gradient::at(0.0, Color32::WHITE, Color32::BLACK).axis(rect);
        near(from, 50.0, 100.0);
        near(to, 50.0, 0.0);
        // 90deg is "to right".
        let (from, to) = Gradient::at(90.0, Color32::WHITE, Color32::BLACK).axis(rect);
        near(from, 0.0, 50.0);
        near(to, 100.0, 50.0);
        // And 160deg — the one every diagonal in the reference uses — runs down and to the right.
        let (from, to) = Gradient::at(160.0, Color32::WHITE, Color32::BLACK).axis(rect);
        assert!(to.x > from.x && to.y > from.y);
    }

    #[test]
    fn a_theme_is_found_by_name_and_an_unknown_one_is_refused() {
        assert_eq!(Theme::named("dark-neumorphic").unwrap().label, "Dark Neumorphic");
        assert!(Theme::named("solarized").is_none());
        assert_eq!(Theme::names(), vec!["light-neumorphic", "dark-neumorphic"]);
    }

    #[test]
    fn the_tokens_are_the_stylesheet_s() {
        // Spot-checked against `reference/neumorphic-tokens.css` and `reference/incognito-theme.css`.
        // A typo in a hex string is a failing test rather than a slightly wrong colour.
        assert_eq!(light().surface.s1, rgb(0xECEFF3));
        assert_eq!(light().shadow.dark, rgb(0xB8BFCC));
        assert_eq!(light().ink.i900, rgb(0x1E2530));
        assert_eq!(light().accent.blue, rgb(0x2F6BFF));
        assert_eq!(dark().surface.s1, rgb(0x23272E));
        assert_eq!(dark().shadow.light, rgb(0x2F353E));
        assert_eq!(dark().ink.i900, rgb(0xEDF0F5));
        assert_eq!(dark().accent.blue, rgb(0x4C86FF));
        assert_eq!(light().radius.lg, 18.0);
    }
}
