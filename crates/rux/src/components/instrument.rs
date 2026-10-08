//! The instrument language: plates, screens, LEDs, and the motion they share.
//!
//! Unluminous's chat pane asked for answers drawn as hardware (`task-2211`): an answer's components
//! read as machined modules on a studio desk, after Teenage Engineering and Braun. The controls in
//! this group are built from five pieces, and this file is the five:
//!
//! - a **plate**, raised, with a lit edge along its top where machined metal catches the light;
//! - a **screen**, recessed into a plate, darker than anything else, with a faint dot graticule, which
//!   is where data is shown;
//! - an **LED**, a small lit dot with a bloom, which is where colour lives. A control is never filled
//!   with an accent colour; it lights one;
//! - **encoder colours** for data series, the theme's own accents in the order an OP-1's four encoders
//!   are coloured;
//! - **physical motion**: a press sinks, a light fades on, a value rises from its baseline.
//!
//! **None of it is a new token.** Every colour here is worked out from the theme's own surfaces,
//! shadows, ink and accents, so a theme that changes those changes this, and `tests/reference.rs` has
//! nothing new to check. The derivations say what they are made of where they are made.

use egui::{Color32, Id, Mesh, Pos2, Rect, Shape, Stroke, Vec2};

use crate::render::Fill;
use crate::theme::{Elevation, Shadow, Theme};
use crate::Rux;

/// The colours the instrument controls draw in, worked out from a theme.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Instrument {
    /// A screen's ground: darker than the deepest surface, toward the theme's dark shadow.
    pub screen: Color32,
    /// The graticule's dots on a screen.
    pub dots: Color32,
    /// The lit edge along the top of a plate.
    pub edge: Color32,
    /// A plate's own ground.
    pub plate: Color32,
    /// An LED that is off.
    pub led_off: Color32,
    /// Gridlines and hairlines on a screen.
    pub rule: Color32,
    /// Data series, in order: blue, mint, amber, coral, violet.
    pub series: [Color32; 5],
    /// The series a comparison is made against, which is a quiet grey rather than a colour.
    pub quiet: Color32,
}

impl Instrument {
    /// The instrument colours of `theme`.
    pub fn of(theme: &Theme) -> Self {
        let lerp = |a: Color32, b: Color32, t: f32| {
            let mix = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
            Color32::from_rgb(mix(a.r(), b.r()), mix(a.g(), b.g()), mix(a.b(), b.b()))
        };
        let (screen, dots, edge, led_off) = match theme.dark {
            // Most of the way to the dark shadow, so a screen reads as a hole in the plate.
            true => (
                lerp(theme.surface.s0, theme.shadow.dark, 0.75),
                Color32::from_white_alpha(9),
                Color32::from_white_alpha(18),
                lerp(theme.surface.s3, theme.shadow.dark, 0.25),
            ),
            false => (
                theme.surface.sunken,
                Color32::from_black_alpha(14),
                Color32::from_white_alpha(230),
                theme.ink.i200,
            ),
        };
        let accent = theme.accent;
        Self {
            screen,
            dots,
            edge,
            plate: theme.surface.s2,
            led_off,
            rule: match theme.dark {
                true => Color32::from_white_alpha(14),
                false => Color32::from_black_alpha(18),
            },
            series: [
                accent.blue,
                accent.mint,
                accent.amber,
                accent.coral,
                accent.violet,
            ],
            quiet: lerp(theme.ink.i300, theme.ink.i200, 0.4),
        }
    }

    /// The colour of series `index`, wrapping round the five.
    pub fn series(&self, index: usize) -> Color32 {
        self.series[index % self.series.len()]
    }
}

/// How fast the instrument moves. One set of numbers, so a press feels the same on every control.
pub mod timing {
    /// A key sinking or coming back up.
    pub const PRESS: f32 = 0.09;
    /// An LED fading on or off.
    pub const LIGHT: f32 = 0.16;
    /// A bar, a segment run or a trace rising from the baseline.
    pub const RISE: f32 = 0.42;
    /// Between one bar starting to rise and the next.
    pub const STAGGER: f32 = 0.03;
    /// A number rolling to its value.
    pub const ROLL: f32 = 0.6;
    /// One breath of an LED that is working.
    pub const BREATH: f32 = 2.4;
    /// How long an LED goes on breathing before it holds still, so a finished answer does not keep
    /// the window drawing for ever.
    pub const BREATHING_FOR: f32 = 12.0;
}

/// Ease out, exponential: quick to start and slow to settle, which is how a physical thing stops.
pub fn ease_out(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(3)
}

/// Ease out with a small overshoot, for something rising into place.
pub fn ease_back(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let c = 1.2;
    1.0 + (c + 1.0) * (t - 1.0).powi(3) + c * (t - 1.0).powi(2)
}

/// A value moving toward `on` or off over `seconds`, from 0 to 1. Still when the state says so.
pub fn toward(rux: &Rux<'_>, id: Id, on: bool, seconds: f32) -> f32 {
    match rux.state.is_still() {
        true => f32::from(u8::from(on)),
        false => rux.ctx().animate_bool_with_time(id, on, seconds),
    }
}

/// How far through its first appearance `id` is, from 0 to 1, over `seconds` after `delay`.
///
/// The first frame that asks about an id writes down the time, and a frame is asked for while the
/// answer is short of one. An id is the caller's, so it has to be stable: a component scrolled across
/// the pane must not appear afresh on every frame.
pub fn appearing(rux: &Rux<'_>, id: Id, delay: f32, seconds: f32) -> f32 {
    if rux.state.is_still() {
        return 1.0;
    }
    let now = rux.ctx().input(|input| input.time);
    let since = rux
        .ctx()
        .data_mut(|data| *data.get_temp_mut_or_insert_with(id.with("rux-appear"), || now));
    let progress = ((now - since) as f32 - delay) / seconds.max(0.001);
    if progress < 1.0 {
        rux.ctx().request_repaint();
    }
    progress.clamp(0.0, 1.0)
}

/// How long ago `id` first appeared, in seconds. The first frame to ask answers zero.
pub fn age(rux: &Rux<'_>, id: Id) -> f64 {
    let now = rux.ctx().input(|input| input.time);
    let since = rux
        .ctx()
        .data_mut(|data| *data.get_temp_mut_or_insert_with(id.with("rux-appear"), || now));
    now - since
}

/// An elevation part of the way from raised to pressed: the outer shadows fading out while the inner
/// ones fade in. This is what a key does while it sinks.
pub fn sinking(raised: Elevation, pressed: Elevation, t: f32) -> Elevation {
    let t = t.clamp(0.0, 1.0);
    let fade = |shadow: Shadow, by: f32| Shadow {
        colour: shadow.colour.gamma_multiply(by),
        ..shadow
    };
    let outer = raised
        .shadows()
        .filter(|s| !s.inset)
        .map(|s| fade(s, 1.0 - t));
    let inner = pressed.shadows().filter(|s| s.inset).map(|s| fade(s, t));
    let all: Vec<Shadow> = outer.chain(inner).filter(|s| s.colour.a() > 0).collect();
    Elevation::of(all)
}

/// A raised plate with a lit top edge. Answers the rectangle inside its padding.
pub fn plate(rux: &mut Rux<'_>, rect: Rect, radius: f32, padding: f32) -> Rect {
    let theme = rux.theme();
    let colours = Instrument::of(theme);
    rux.chrome.surface(
        rect,
        radius,
        Fill::Solid(colours.plate),
        theme.elevation.raised_sm,
    );
    lit_edge(rux.painter(), rect, radius, colours.edge);
    rect.shrink(padding)
}

/// The lit edge along the top of a raised thing: brightest a third of the way along, where light from
/// the top left lands, and gone before the corners.
pub fn lit_edge(painter: &egui::Painter, rect: Rect, radius: f32, colour: Color32) {
    let left = rect.left() + radius * 0.8;
    let right = rect.right() - radius * 0.8;
    if right - left < 4.0 {
        return;
    }
    let y = rect.top() + 0.5;
    let peak = left + (right - left) * 0.35;
    let mut mesh = Mesh::default();
    let height = 1.0;
    for (x, alpha) in [(left, 0.0), (peak, 1.0), (right, 0.0)] {
        let tint = colour.gamma_multiply(alpha);
        mesh.colored_vertex(Pos2::new(x, y - height / 2.0), tint);
        mesh.colored_vertex(Pos2::new(x, y + height / 2.0), tint);
    }
    for i in 0..2u32 {
        let a = i * 2;
        mesh.add_triangle(a, a + 1, a + 2);
        mesh.add_triangle(a + 1, a + 2, a + 3);
    }
    painter.add(Shape::mesh(mesh));
}

/// A recessed screen: the darkest ground there is, sunk into the plate, with a dot graticule when
/// `graticule` is true.
pub fn screen(rux: &mut Rux<'_>, rect: Rect, radius: f32, graticule: bool) {
    let theme = rux.theme();
    let colours = Instrument::of(theme);
    rux.chrome.surface(
        rect,
        radius,
        Fill::Solid(colours.screen),
        theme.elevation.pressed_sm,
    );
    if graticule {
        dots(rux, rect.shrink(radius * 0.5), colours.dots);
    }
}

/// The graticule: one dot every six points, painted as a single repeating texture so a screen of any
/// size is one quad.
fn dots(rux: &mut Rux<'_>, rect: Rect, colour: Color32) {
    let ctx = rux.ctx().clone();
    let key = Id::new("rux-graticule");
    // Looked up and stored in two steps: loading a texture takes the context's lock, which the
    // memory's own closure is already holding.
    let texture = match ctx.data(|data| data.get_temp::<egui::TextureHandle>(key)) {
        Some(texture) => texture,
        None => {
            let mut image = egui::ColorImage::new([6, 6], vec![Color32::TRANSPARENT; 36]);
            image.pixels[0] = Color32::WHITE;
            let texture = ctx.load_texture(
                "rux-graticule",
                image,
                egui::TextureOptions {
                    magnification: egui::TextureFilter::Nearest,
                    minification: egui::TextureFilter::Nearest,
                    wrap_mode: egui::TextureWrapMode::Repeat,
                    mipmap_mode: None,
                },
            );
            ctx.data_mut(|data| data.insert_temp(key, texture.clone()));
            texture
        }
    };
    let step = 6.0 * rux.zoom().max(1.0).round();
    let uv = Rect::from_min_max(
        Pos2::ZERO,
        Pos2::new(rect.width() / step, rect.height() / step),
    );
    rux.painter().image(texture.id(), rect, uv, colour);
}

/// An LED at `centre`: off, or lit in `colour` at `brightness` from 0 to 1, with a bloom round it.
///
/// Drawn with `egui` rather than into the chrome, because a light fades and a breathing one changes on
/// every frame; recorded into the chrome it would rasterise the whole layer each time.
pub fn led(
    painter: &egui::Painter,
    centre: Pos2,
    radius: f32,
    colour: Color32,
    off: Color32,
    brightness: f32,
) {
    let lit = brightness.clamp(0.0, 1.0);
    if lit > 0.01 {
        // The bloom: rings of falling alpha, wider than the lamp.
        for ring in (1..=6).rev() {
            let reach = radius * (1.0 + ring as f32 * 0.42);
            let alpha = lit * 0.13 * (1.0 - ring as f32 / 7.0);
            painter.circle_filled(centre, reach, colour.gamma_multiply(alpha));
        }
    }
    let lerp = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * lit).round() as u8;
    let core = Color32::from_rgb(
        lerp(off.r(), colour.r()),
        lerp(off.g(), colour.g()),
        lerp(off.b(), colour.b()),
    );
    painter.circle_filled(centre, radius, core);
    // The rim: a lamp sits in a hole, so its lower right edge catches no light.
    painter.circle_stroke(
        centre,
        radius,
        Stroke::new(0.6, Color32::from_black_alpha(90)),
    );
    // The glint on the glass, brighter when lit.
    let glint = Color32::from_white_alpha((40.0 + 120.0 * lit) as u8);
    painter.circle_filled(centre + Vec2::splat(-radius * 0.32), radius * 0.32, glint);
}

/// An LED for a control, at the theme's own off colour, fading with the control's state.
pub fn led_for(rux: &Rux<'_>, id: Id, centre: Pos2, radius: f32, colour: Color32, on: bool) {
    let off = Instrument::of(rux.theme()).led_off;
    let lit = toward(rux, id.with("rux-led"), on, timing::LIGHT);
    led(rux.painter(), centre, radius, colour, off, lit);
}

/// The text style for silkscreen labels printed on a plate: small, spaced capitals.
pub const SILK: crate::text::Style = crate::text::Style::sans(9.5)
    .semibold()
    .tracking(0.12)
    .upper();

/// The text style for a reading on a screen.
pub const READING: crate::text::Style = crate::text::Style::mono(21.0).medium().tracking(-0.02);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_instrument_colours_come_from_the_theme() {
        let dark = Instrument::of(crate::theme::dark());
        let light = Instrument::of(crate::theme::light());
        // A screen is darker than every surface of its theme.
        let lum = |c: Color32| c.r() as u32 + c.g() as u32 + c.b() as u32;
        assert!(lum(dark.screen) < lum(crate::theme::dark().surface.s0));
        assert_eq!(dark.series[0], crate::theme::dark().accent.blue);
        assert_ne!(dark.screen, light.screen);
    }

    #[test]
    fn a_key_sinking_trades_its_outer_shadows_for_inner_ones() {
        let theme = crate::theme::dark();
        let up = sinking(theme.elevation.raised_sm, theme.elevation.pressed_sm, 0.0);
        let down = sinking(theme.elevation.raised_sm, theme.elevation.pressed_sm, 1.0);
        assert!(up.shadows().all(|s| !s.inset));
        assert!(down.shadows().all(|s| s.inset));
        let half = sinking(theme.elevation.raised_sm, theme.elevation.pressed_sm, 0.5);
        assert_eq!(half.shadows().count(), 4);
    }

    #[test]
    fn the_easings_start_at_zero_and_end_at_one() {
        assert_eq!(ease_out(0.0), 0.0);
        assert!((ease_out(1.0) - 1.0).abs() < 1e-6);
        assert!((ease_back(1.0) - 1.0).abs() < 1e-6);
        // And the back easing goes past the end on the way.
        assert!((0..100)
            .map(|i| ease_back(i as f32 / 100.0))
            .any(|v| v > 1.0));
    }
}
