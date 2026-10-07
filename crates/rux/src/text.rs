//! Which face, at what size, with what tracking.
//!
//! `epaint` shapes, wraps, measures and hit-tests the text; this module decides what to hand it. Two
//! things it carries that a naive port would drop, and both are load-bearing in the reference:
//!
//! **Weight is a face, not an axis.** `ab_glyph` has no variable-font axes, and the design uses four
//! weights of Inter and three of JetBrains Mono as genuinely different faces — a `600` heading beside
//! a `400` body is most of what the typography *is*. So seven static faces are registered, one per
//! weight, and [`Weight`] picks between them.
//!
//! **`letter-spacing` and `line-height` are carried through.** The reference's kickers are
//! `font-size: 9.5px; letter-spacing: 0.14em` uppercase mono, which without the tracking is a
//! different design. CSS gives `em` and `epaint` takes points, so the conversion is `em × size`.

use egui::text::{LayoutJob, TextFormat, TextWrapping};
use egui::{Color32, FontFamily, FontId, Galley, Pos2, Rect, Vec2};
use std::sync::Arc;

/// Inter, at the four weights the reference uses.
const INTER_400: &[u8] = include_bytes!("../assets/fonts/Inter-Regular.ttf");
const INTER_500: &[u8] = include_bytes!("../assets/fonts/Inter-Medium.ttf");
const INTER_600: &[u8] = include_bytes!("../assets/fonts/Inter-SemiBold.ttf");
const INTER_700: &[u8] = include_bytes!("../assets/fonts/Inter-Bold.ttf");
/// JetBrains Mono, at the three the reference uses.
const MONO_400: &[u8] = include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf");
const MONO_500: &[u8] = include_bytes!("../assets/fonts/JetBrainsMono-Medium.ttf");
const MONO_600: &[u8] = include_bytes!("../assets/fonts/JetBrainsMono-SemiBold.ttf");

/// `--font-sans` and `--font-mono`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Family {
    /// Inter. `--font-sans` and `--font-display` are the same face in the token file.
    Sans,
    /// JetBrains Mono. Every kicker, label, count and timestamp in the reference is set in it.
    Mono,
}

/// A CSS `font-weight`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Weight {
    Regular,
    Medium,
    SemiBold,
    Bold,
}

impl Weight {
    /// The number the stylesheet writes.
    pub fn number(self) -> u16 {
        match self {
            Weight::Regular => 400,
            Weight::Medium => 500,
            Weight::SemiBold => 600,
            Weight::Bold => 700,
        }
    }
}

/// The name a family and a weight are registered under.
///
/// **JetBrains Mono at `700` falls back to `600`**, which is stated rather than silently done: the
/// reference never sets a bold mono, so the face is not vendored, and a caller that asks for one gets
/// the heaviest there is rather than a missing glyph.
fn family_name(family: Family, weight: Weight) -> &'static str {
    match (family, weight) {
        (Family::Sans, Weight::Regular) => "rux-sans-400",
        (Family::Sans, Weight::Medium) => "rux-sans-500",
        (Family::Sans, Weight::SemiBold) => "rux-sans-600",
        (Family::Sans, Weight::Bold) => "rux-sans-700",
        (Family::Mono, Weight::Regular) => "rux-mono-400",
        (Family::Mono, Weight::Medium) => "rux-mono-500",
        (Family::Mono, Weight::SemiBold | Weight::Bold) => "rux-mono-600",
    }
}

/// Register the library's faces with an `egui` context.
///
/// Called once, before the first frame. It also points `Proportional` and `Monospace` at the library's
/// own faces, so anything `egui` draws for itself — a tooltip, the text inside a `TextEdit` before a
/// component restyles it — is in the same typeface as everything else.
pub fn install(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    let faces: [(&str, &'static [u8]); 7] = [
        ("rux-sans-400", INTER_400),
        ("rux-sans-500", INTER_500),
        ("rux-sans-600", INTER_600),
        ("rux-sans-700", INTER_700),
        ("rux-mono-400", MONO_400),
        ("rux-mono-500", MONO_500),
        ("rux-mono-600", MONO_600),
    ];
    for (name, bytes) in faces {
        fonts.font_data.insert(name.to_owned(), Arc::new(egui::FontData::from_static(bytes)));
        fonts.families.insert(FontFamily::Name(name.into()), vec![name.to_owned()]);
    }
    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "rux-sans-400".to_owned());
    fonts
        .families
        .entry(FontFamily::Monospace)
        .or_default()
        .insert(0, "rux-mono-400".to_owned());
    ctx.set_fonts(fonts);
}

/// One complete answer to "how is this text set".
///
/// A value rather than five arguments, so a kicker is a named style at one call site instead of the
/// same five numbers repeated at forty.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    pub family: Family,
    pub weight: Weight,
    /// `font-size`, in points.
    pub size: f32,
    /// `letter-spacing`, in **em** — the unit the stylesheet writes it in.
    pub tracking: f32,
    /// `line-height` as a multiplier, or `None` for the face's own.
    pub line_height: Option<f32>,
    /// `text-transform: uppercase`.
    pub upper: bool,
}

impl Style {
    /// Inter at a size, regular weight, no tracking.
    pub const fn sans(size: f32) -> Self {
        Self {
            family: Family::Sans,
            weight: Weight::Regular,
            size,
            tracking: 0.0,
            line_height: None,
            upper: false,
        }
    }

    /// JetBrains Mono at a size.
    pub const fn mono(size: f32) -> Self {
        Self { family: Family::Mono, ..Self::sans(size) }
    }

    pub const fn weight(mut self, weight: Weight) -> Self {
        self.weight = weight;
        self
    }

    pub const fn medium(self) -> Self {
        self.weight(Weight::Medium)
    }

    pub const fn semibold(self) -> Self {
        self.weight(Weight::SemiBold)
    }

    pub const fn bold(self) -> Self {
        self.weight(Weight::Bold)
    }

    /// `letter-spacing`, in em.
    pub const fn tracking(mut self, em: f32) -> Self {
        self.tracking = em;
        self
    }

    /// `line-height`, as a multiplier of the size.
    pub const fn leading(mut self, multiplier: f32) -> Self {
        self.line_height = Some(multiplier);
        self
    }

    pub const fn upper(mut self) -> Self {
        self.upper = true;
        self
    }

    /// The same style at another size, which is what a zoom or a compact variant asks for.
    pub const fn at(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn font(self) -> FontId {
        FontId::new(self.size, FontFamily::Name(family_name(self.family, self.weight).into()))
    }

    /// `letter-spacing` in points, which is what `epaint` takes.
    pub fn extra_letter_spacing(self) -> f32 {
        self.tracking * self.size
    }

    fn format(self, colour: Color32) -> TextFormat {
        TextFormat {
            font_id: self.font(),
            extra_letter_spacing: self.extra_letter_spacing(),
            line_height: self.line_height.map(|multiplier| multiplier * self.size),
            color: colour,
            ..Default::default()
        }
    }

    fn transform(self, text: &str) -> String {
        match self.upper {
            true => text.to_uppercase(),
            false => text.to_owned(),
        }
    }

    // ---- The reference's own named styles ----------------------------------------------------

    /// `.t-display` — 600 48px/1.05, `-0.03em`.
    pub const DISPLAY: Self = Self::sans(48.0).semibold().tracking(-0.03).leading(1.05);
    /// `.t-h1` — 600 32px/1.2, `-0.02em`.
    pub const H1: Self = Self::sans(32.0).semibold().tracking(-0.02).leading(1.2);
    /// `.t-h2` — 600 24px/1.3, `-0.015em`.
    pub const H2: Self = Self::sans(24.0).semibold().tracking(-0.015).leading(1.3);
    /// `.t-h3` — 600 18px/1.35, `-0.01em`.
    pub const H3: Self = Self::sans(18.0).semibold().tracking(-0.01).leading(1.35);
    /// `.t-body` — 400 15px/1.5.
    pub const BODY: Self = Self::sans(15.0).leading(1.5);
    /// `html, body` — 400 14px/1.5, `-0.005em`. The size everything in the page inherits from.
    pub const BASE: Self = Self::sans(14.0).tracking(-0.005).leading(1.5);
    /// `.t-label` — 500 13px/1.4.
    pub const LABEL: Self = Self::sans(13.0).medium().leading(1.4);
    /// `.t-caption` — 500 11px/1.4 mono, uppercase, `0.14em`. The kicker over a section heading.
    pub const CAPTION: Self = Self::mono(11.0).medium().tracking(0.14).leading(1.4).upper();
    /// `.ic-labelled__label` — the small uppercase mono label over a control.
    pub const FIELD_LABEL: Self = Self::mono(9.5).tracking(0.14).upper();
    /// `.ic-ctrl__label` — the same, one step smaller, over a control in a two-column grid.
    pub const CTRL_LABEL: Self = Self::mono(9.0).tracking(0.14).upper();
    /// `.ic-panel__title` — a panel's own heading.
    pub const PANEL_TITLE: Self = Self::mono(10.5).medium().tracking(0.16).upper();
    /// The text on a control: `.topnav__item`, `.ic-select__trigger`, `.ic-mini-btn`.
    pub const CONTROL: Self = Self::sans(12.0).medium().tracking(-0.005);
    /// The text on a larger control: `.proj__name`, `.ic-clear`, a list row.
    pub const CONTROL_LG: Self = Self::sans(13.0).medium().tracking(-0.01);
    /// `.ic-create` — the word on a primary button.
    pub const BUTTON: Self = Self::sans(14.0).semibold().tracking(-0.005);
    /// `.ic-prompt`, `.ic-form-textarea` — running text in a well.
    pub const PROSE: Self = Self::sans(13.0).leading(1.6);
    /// `.ic-count input`, `.ic-elapsed` — a number.
    pub const NUMBER: Self = Self::mono(12.0).semibold();
}

/// Lay text out with no wrapping at all.
pub fn layout(painter: &egui::Painter, style: Style, text: &str, colour: Color32) -> Arc<Galley> {
    let mut job = LayoutJob::single_section(style.transform(text), style.format(colour));
    job.wrap = TextWrapping::no_max_width();
    painter.layout_job(job)
}

/// Lay text out, wrapping at `width`.
pub fn wrapped(
    painter: &egui::Painter,
    style: Style,
    text: &str,
    colour: Color32,
    width: f32,
) -> Arc<Galley> {
    let mut job = LayoutJob::single_section(style.transform(text), style.format(colour));
    job.wrap = TextWrapping::from_wrap_mode_and_width(egui::TextWrapMode::Wrap, width);
    painter.layout_job(job)
}

/// Lay text out on one line, cutting it with `…` when it does not fit.
///
/// `text-overflow: ellipsis`, which `.ic-select__value`, `.proj__name` and `.ic-saveto__value` all
/// declare and which a component library has to have: a page can be sure its label fits and a library
/// cannot.
pub fn elided(
    painter: &egui::Painter,
    style: Style,
    text: &str,
    colour: Color32,
    width: f32,
) -> Arc<Galley> {
    let mut job = LayoutJob::single_section(style.transform(text), style.format(colour));
    job.wrap = TextWrapping {
        max_width: width.max(0.0),
        max_rows: 1,
        break_anywhere: true,
        overflow_character: Some('…'),
    };
    painter.layout_job(job)
}

/// How much room some text takes, without drawing it.
pub fn measure(painter: &egui::Painter, style: Style, text: &str) -> Vec2 {
    layout(painter, style, text, Color32::WHITE).size()
}

/// Draw a galley with its **left edge and vertical centre** at `at`.
///
/// Which is how nearly every label in the reference is placed: a row is `display: flex;
/// align-items: center`, so what a caller knows is the row's middle and where the text starts.
pub fn draw_left_centre(painter: &egui::Painter, at: Pos2, galley: Arc<Galley>, colour: Color32) {
    let top = at.y - galley.size().y / 2.0;
    painter.galley(Pos2::new(at.x, top.round()), galley, colour);
}

/// Draw a one line galley with its left edge at `at.x` and its capitals centred on `at.y`.
///
/// [`draw_left_centre`] centres the galley's box, and a line box is taller than the letters in it: the
/// baseline sits the font's ascent from the top and the extra leading is all below. Beside a mark that is
/// centred on the same point, such as a select's chevron, the words then sit visibly high or low, by an
/// amount that depends on the face a consumer has bound to the family. The band from the top of a capital
/// down to the baseline is measured off the same font from an `H`, so every row is placed the same
/// whatever letters are in it.
pub fn draw_left_capitals(
    painter: &egui::Painter,
    at: Pos2,
    galley: Arc<Galley>,
    style: Style,
    colour: Color32,
) {
    let probe = painter.layout_no_wrap("H".to_owned(), style.font(), Color32::WHITE);
    let top = match capital_band(&probe) {
        Some((cap, baseline)) => at.y - (cap + baseline) / 2.0,
        None => at.y - galley.size().y / 2.0,
    };
    painter.galley(Pos2::new(at.x, top.round()), galley, colour);
}

/// From the top of a galley, where its first glyph's ink starts and where its baseline is.
///
/// `Glyph::pos` is the baseline relative to its row, and `uv_rect.offset` is where the ink starts relative
/// to that, which is the sum `epaint`'s own tessellator makes when it places the glyph.
fn capital_band(galley: &Galley) -> Option<(f32, f32)> {
    let row = galley.rows.first()?;
    let glyph = row.row.glyphs.first()?;
    let baseline = row.pos.y + glyph.pos.y;
    let cap = baseline + glyph.uv_rect.offset.y;
    (baseline > cap).then_some((cap, baseline))
}

/// Draw a galley centred in `rect` on both axes.
pub fn draw_centred(painter: &egui::Painter, rect: Rect, galley: Arc<Galley>, colour: Color32) {
    let at = rect.center() - galley.size() / 2.0;
    painter.galley(Pos2::new(at.x.round(), at.y.round()), galley, colour);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracking_is_converted_from_em_to_points() {
        // `.t-caption` is `font-size: 11px; letter-spacing: 0.14em`, which is 1.54 points of tracking.
        let caption = Style::CAPTION;
        assert!((caption.extra_letter_spacing() - 1.54).abs() < 0.001);
        // And the body's own is a hair of negative tracking rather than none.
        assert!(Style::BASE.extra_letter_spacing() < 0.0);
    }

    #[test]
    fn the_named_styles_are_the_stylesheet_s() {
        assert_eq!((Style::H2.size, Style::H2.weight.number()), (24.0, 600));
        assert_eq!((Style::BODY.size, Style::BODY.weight.number()), (15.0, 400));
        assert_eq!(Style::CAPTION.family, Family::Mono);
        assert!(Style::CAPTION.upper);
        assert_eq!(Style::PROSE.line_height, Some(1.6));
    }

    #[test]
    fn an_uppercase_style_upper_cases_the_words() {
        assert_eq!(Style::CAPTION.transform("Recent projects"), "RECENT PROJECTS");
        assert_eq!(Style::BODY.transform("Recent projects"), "Recent projects");
    }

    #[test]
    fn mono_has_no_bold_face_and_says_so_by_falling_back() {
        assert_eq!(family_name(Family::Mono, Weight::Bold), "rux-mono-600");
        assert_eq!(family_name(Family::Sans, Weight::Bold), "rux-sans-700");
    }
}
