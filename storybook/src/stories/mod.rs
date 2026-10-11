//! What a story is, and the small helpers every one of them is written with.
//!
//! A story is a **list of named states, each drawn live**. Not a picture of a component in a state —
//! the real component, interactive, so "open" is the open dropdown and "focused" is a field you can
//! type into. That is the whole point of a storybook, and it is what finds the state that does not
//! work.

pub mod board;
pub mod chat;
pub mod components;
pub mod foundations;
pub mod pages;

use egui::{Rect, Vec2};
use rux::components::SelectState;
use rux::layout::{Cross, Flex, Pad, Pen};
use rux::text::Style;
use rux::{Icon, Rux};

/// Everything every story on the board needs to remember.
///
/// One bag rather than a per-story `Any`, because it is a storybook: the whole thing is known at
/// compile time and a typed field is easier to read than a downcast. State that is per-instance —
/// which select is open, what is in a field — lives here so that a story can be left and come back to.
pub struct Demo {
    pub nav: usize,
    pub segmented: usize,
    pub tabs: usize,
    pub dice: bool,
    pub switch: bool,
    pub count: i64,
    pub select: SelectState,
    pub select_up: SelectState,
    pub select_mono: SelectState,
    pub select_long: SelectState,
    pub selected: Option<usize>,
    pub selected_mono: Option<usize>,
    pub selected_long: Option<usize>,
    pub project_open: bool,
    pub project: usize,
    pub input: String,
    pub search: String,
    pub prompt: String,
    pub modal_open: bool,
    pub list_row: usize,
    pub progress: f32,
    pub options: Vec<String>,
    pub sizes: Vec<String>,
    pub long: Vec<String>,
    pub projects: Vec<String>,
    /// The instrument story's fader and checklist, so they can be played with.
    pub fader: f64,
    pub ticks: [bool; 3],
    /// The example page, so it keeps its own state while the storybook is used.
    pub image_creator: rux_examples::ImageCreator,
}

impl Default for Demo {
    fn default() -> Self {
        Self {
            nav: 3,
            fader: 300.0,
            ticks: [true, false, false],
            segmented: 0,
            tabs: 2,
            dice: true,
            switch: true,
            count: 1,
            select: SelectState::default(),
            select_up: SelectState::default(),
            select_mono: SelectState::default(),
            select_long: SelectState::default(),
            selected: Some(0),
            selected_mono: Some(0),
            selected_long: None,
            project_open: false,
            project: 0,
            input: "Cinematic 35mm".to_owned(),
            search: String::new(),
            prompt: "A wide cinematic shot of a lone figure at dusk.".to_owned(),
            modal_open: false,
            list_row: 1,
            progress: 0.62,
            options: ["film photography", "digital photo", "oil painting", "3D render"]
                .iter()
                .map(|option| (*option).to_owned())
                .collect(),
            sizes: [
                "1920 × 1080 · HD · 16:9 · 2.1 MP",
                "1024 × 1024 · 1:1 · 1.0 MP",
                "1536 × 1024 · 3:2 · 1.6 MP",
            ]
            .iter()
            .map(|size| (*size).to_owned())
            .collect(),
            long: (1..=14).map(|index| format!("Option number {index}")).collect(),
            projects: ["real-arcade v2", "mvl-arcade-v2", "Nocturne · music video"]
                .iter()
                .map(|name| (*name).to_owned())
                .collect(),
            image_creator: rux_examples::ImageCreator::new(),
        }
    }
}

/// One entry in the sidebar.
pub struct Story {
    /// Which heading it sits under: `Foundations`, `Components`, `Pages`.
    pub section: &'static str,
    pub name: &'static str,
    /// The rule in `reference/image-creator.css` this component comes from, shown under the title so
    /// the thing on the screen can be checked against the thing it was built from.
    pub rule: &'static str,
    /// The names of its states, so the filter matches them as well as the story's own name.
    pub states: &'static [&'static str],
    /// Draws the story into a rectangle and answers how tall it came out.
    pub draw: fn(&mut Rux<'_>, &mut Demo, Rect) -> f32,
    /// A page fills the whole content area rather than running down it.
    pub full_bleed: bool,
}

/// Every story, in the order the sidebar lists them.
pub fn all() -> Vec<Story> {
    let mut stories = foundations::stories();
    stories.extend(components::stories());
    stories.extend(chat::stories());
    stories.extend(board::stories());
    stories.extend(pages::stories());
    stories
}

// ---- The helpers every story is written with -------------------------------------------------

/// `gap` between one state group and the next.
pub const GROUP_GAP: f32 = 34.0;
/// How far a caption sits under the sample it names.
pub const CAPTION_GAP: f32 = 10.0;

/// A heading over a group of states: a rule with a word on it.
pub fn group(rux: &mut Rux<'_>, pen: &mut Pen, title: &str) {
    let theme = rux.theme();
    let strip = pen.strip(22.0);
    let ink = theme.ink.i500;
    let galley = rux.text(Style::CAPTION, title, ink);
    let width = galley.size().x;
    rux.ui.painter().galley(strip.left_top(), galley, ink);
    let line_y = (strip.top() + galley_height(rux) / 2.0).round();
    rux.chrome.line(
        egui::pos2(strip.left() + width + 12.0, line_y),
        egui::pos2(strip.right(), line_y),
        1.0,
        theme.interaction.hairline,
    );
    pen.gap(14.0);
}

fn galley_height(rux: &Rux<'_>) -> f32 {
    rux.measure(Style::CAPTION, "Ag").y
}

/// A row of samples, each with a caption under it.
///
/// Answers one rectangle per sample, at the width each asked for. A row that runs out of width wraps,
/// which is what `flex-wrap: wrap` does and what a storybook narrowed to half a screen needs.
pub fn samples(
    rux: &mut Rux<'_>,
    pen: &mut Pen,
    height: f32,
    cells: &[(&str, f32)],
) -> Vec<Rect> {
    let theme = rux.theme();
    let gap = 24.0;
    let caption_height = rux.measure(Style::mono(9.5).tracking(0.1).upper(), "Ag").y;
    let row_height = height + CAPTION_GAP + caption_height;
    let area = pen.rest();
    let mut out = Vec::with_capacity(cells.len());
    let mut x = area.left();
    let mut y = area.top();
    let mut rows = 1.0;
    for (caption, asked) in cells {
        // A cell is at least as wide as its own caption, or the captions run into one another.
        let width = asked.max(rux.measure(Style::mono(9.5).tracking(0.1).upper(), caption).x);
        if x > area.left() && x + width > area.right() {
            x = area.left();
            y += row_height + 20.0;
            rows += 1.0;
        }
        let sample = Rect::from_min_size(egui::pos2(x, y), Vec2::new(width, height));
        out.push(sample);
        let ink = theme.ink.i400;
        let galley = rux.text(Style::mono(9.5).tracking(0.1).upper(), caption, ink);
        rux.ui.painter().galley(
            egui::pos2(x, (y + height + CAPTION_GAP).round()),
            galley,
            ink,
        );
        x += width + gap;
    }
    pen.strip(rows * row_height + (rows - 1.0) * 20.0);
    out
}

/// One sample on a line of its own, full width.
pub fn wide(rux: &mut Rux<'_>, pen: &mut Pen, height: f32, caption: &str) -> Rect {
    let theme = rux.theme();
    let sample = pen.strip(height);
    pen.gap(CAPTION_GAP);
    let ink = theme.ink.i400;
    let galley = rux.text(Style::mono(9.5).tracking(0.1).upper(), caption, ink);
    let strip = pen.strip(galley.size().y);
    rux.ui.painter().galley(strip.left_top(), galley, ink);
    sample
}

/// A line of prose under a heading, for a story that needs a sentence of explanation.
pub fn note(rux: &mut Rux<'_>, pen: &mut Pen, text: &str) {
    let theme = rux.theme();
    let ink = theme.ink.i500;
    let area = pen.rest();
    let galley = rux::text::wrapped(
        rux.ui.painter(),
        Style::sans(12.5).leading(1.5),
        text,
        ink,
        area.width().min(720.0),
    );
    let height = galley.size().y;
    let strip = pen.strip(height);
    rux.ui.painter().galley(strip.left_top(), galley, ink);
    pen.gap(16.0);
}

/// A swatch: a colour, its token name and its hex, which is the whole of the colour foundation page.
pub fn swatch(rux: &mut Rux<'_>, rect: Rect, colour: egui::Color32, name: &str) {
    let theme = rux.theme();
    let [chip, words] = Flex::row(rect).gap(12.0).cross(Cross::Centre).sized(Vec2::splat(36.0)).fill(1.0).solve();
    rux.chrome.surface(chip, theme.radius.sm, colour, theme.elevation.raised_sm);
    let [top, bottom] = Flex::column(words).fixed(16.0).fixed(14.0).solve();
    let ink = theme.ink.i900;
    let galley = rux.text(Style::sans(12.0).medium(), name, ink);
    rux.ui.painter().galley(top.left_top(), galley, ink);
    let [r, g, b, _] = colour.to_srgba_unmultiplied();
    let hex = format!("#{r:02X}{g:02X}{b:02X}");
    let dim = theme.ink.i400;
    let galley = rux.text(Style::mono(10.5), &hex, dim);
    rux.ui.painter().galley(bottom.left_top(), galley, dim);
}

/// The mark and the two lines of an icon on the icon sheet.
pub fn icon_cell(rux: &mut Rux<'_>, rect: Rect, icon: Icon) {
    let theme = rux.theme();
    let [big, small, name] = Flex::column(rect).gap(6.0).fixed(32.0).fixed(16.0).fixed(14.0).solve();
    rux.mark_in(rux::Mark::new(icon, 24.0), big, theme.ink.i700);
    // **A mark is judged at both sizes or it is not judged.** A sheet at three times life size hides
    // exactly the faults that matter at the fourteen pixels a person really sees.
    rux.mark_in(rux::Mark::new(icon, 13.0), small, theme.ink.i700);
    let dim = theme.ink.i400;
    let galley = rux.elided(Style::mono(9.0), icon.name(), dim, rect.width());
    let at = egui::pos2((name.center().x - galley.size().x / 2.0).round(), name.top());
    rux.ui.painter().galley(at, galley, dim);
}

/// The padding a story's page is drawn inside.
pub const PAGE_PAD: Pad = Pad::sides(28.0, 40.0, 60.0, 40.0);
