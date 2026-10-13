//! The storybook's own window: a theme picker at the top left, a filtered sidebar, and the story.
//!
//! The shell is built out of `rux` itself, deliberately — the theme picker is the library's own
//! `Select` and the filter is its own `TextInput`, so the storybook is its own first consumer and a
//! fault in either is visible the moment the application starts.

use egui::{Rect, Vec2};
use rux::components::{ButtonSize, ButtonVariant, PanelKind, SelectState};
use rux::layout::{Cross, Flex, Pad, Pen};
use rux::prelude::*;
use rux::text::Style;
use rux::RuxState;

use crate::stories::{self, Demo, Story, PAGE_PAD};

/// `.topbar { padding: 16px 28px }` — the same strip the reference's own pages have.
const TOPBAR: f32 = 64.0;
/// Wide enough for a section heading and a story name without eliding either.
const SIDEBAR: f32 = 264.0;

pub struct Storybook {
    stories: Vec<Story>,
    chosen: usize,
    filter: String,
    theme_names: Vec<String>,
    theme_index: usize,
    theme_select: SelectState,
    demo: Demo,
    /// How far the story's own page is scrolled, kept per story so leaving one and coming back does
    /// not lose the place.
    scroll: Vec<f32>,
    /// Whether the measurement strip along the bottom of the sidebar is showing.
    measuring: bool,
    /// How far to scroll the chosen story on the first frame, which `--scroll` sets so a capture pass
    /// can photograph a story taller than the screen in slices.
    pending_scroll: Option<f32>,
}

impl Storybook {
    pub fn new(ctx: &egui::Context, measuring: bool) -> Self {
        rux::text::install(ctx);
        let stories = stories::all();
        let scroll = vec![0.0; stories.len()];
        let names: Vec<String> =
            Theme::all().iter().map(|theme| theme.label.to_owned()).collect();
        Self {
            stories,
            chosen: 0,
            filter: String::new(),
            theme_names: names,
            theme_index: 1,
            theme_select: SelectState::default(),
            demo: Demo::default(),
            scroll,
            measuring,
            pending_scroll: None,
        }
    }

    /// Which stories the filter leaves, as indices into `stories`.
    ///
    /// Two rules, and the second one is why there are two: a **subsequence** over the section and the
    /// name, which is what a fuzzy file finder does and is what makes `crtbtn` find `Create Button`;
    /// and a plain **substring** over the rule and the state names.
    ///
    /// A subsequence over everything was the first version and it was useless. The haystack included a
    /// rule like `.t-display / .t-h1 / .t-body / .t-caption`, and a long haystack contains nearly every
    /// short subsequence — `sel` matched `Type`, because there is an `s` in `t-display`, an `e`
    /// somewhere after it and an `l` after that. A filter that leaves nineteen of twenty-four rows is
    /// a filter nobody uses.
    fn matching(&self) -> Vec<usize> {
        let needle = self.filter.trim().to_lowercase();
        if needle.is_empty() {
            return (0..self.stories.len()).collect();
        }
        self.stories
            .iter()
            .enumerate()
            .filter(|(_, story)| {
                let name = format!("{} {}", story.section, story.name).to_lowercase();
                if is_subsequence(&needle, &name) {
                    return true;
                }
                if story.rule.to_lowercase().contains(&needle) {
                    return true;
                }
                story.states.iter().any(|state| state.to_lowercase().contains(&needle))
            })
            .map(|(index, _)| index)
            .collect()
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, state: &RuxState) {
        let area = ui.max_rect();
        let id = egui::Id::new("rux-storybook");
        // Everything outside a popup is one layer, so the whole window's decoration is one texture
        // and a frame where nothing moved costs a list comparison.
        let visible = self.matching();
        rux::layer(ui, state, id, area, |rux| {
            rux.ground(area);
            let [topbar, body] = Flex::column(area).fixed(TOPBAR).fill(1.0).solve();
            self.show_topbar(rux, topbar, state);
            let [sidebar, content] = Flex::row(body).fixed(SIDEBAR).fill(1.0).solve();
            self.show_sidebar(rux, sidebar, &visible, state);
            self.show_story(rux, content);
        });
    }

    fn show_topbar(&mut self, rux: &mut Rux<'_>, rect: Rect, state: &RuxState) {
        let theme = rux.theme();
        Divider::new().faded().show(
            rux,
            Rect::from_min_size(rect.left_bottom(), Vec2::new(rect.width(), 1.0)),
        );
        let count = format!("{} stories", self.stories.len());
        let count_width = rux.measure(Style::mono(10.5).tracking(0.1).upper(), &count).x;

        // **The theme picker is at the top left**, which is what the ticket asks for, and it is the
        // library's own `Select`.
        let [picker, title, tally] = Flex::row(rect)
            .pad(Pad::axes(14.0, 20.0))
            .gap(20.0)
            .cross(Cross::Centre)
            .hug(220.0)
            .fill(1.0)
            .hug(count_width)
            .solve();

        let picker = Rect::from_center_size(picker.center(), Vec2::new(220.0, 34.0));
        if let Some(index) = Select::new(&self.theme_names, Some(self.theme_index))
            .label("Theme")
            .show(rux, picker, &mut self.theme_select)
            .chosen
        {
            self.theme_index = index;
            if let Some(theme) = Theme::all().get(index) {
                state.set_theme(theme);
            }
        }

        let ink = theme.ink.i900;
        let galley = rux.text(Style::sans(15.0).semibold().tracking(-0.01), "RUX Storybook", ink);
        rux::text::draw_left_centre(rux.ui.painter(), title.left_center(), galley, ink);

        let dim = theme.ink.i400;
        let galley = rux.text(Style::mono(10.5).tracking(0.1).upper(), &count, dim);
        rux::text::draw_left_centre(rux.ui.painter(), tally.left_center(), galley, dim);
    }

    fn show_sidebar(&mut self, rux: &mut Rux<'_>, rect: Rect, visible: &[usize], state: &RuxState) {
        let theme = rux.theme();
        Panel::new().kind(PanelKind::Flat).pad(Pad::NONE).show(rux, rect);
        rux.chrome.line(rect.right_top(), rect.right_bottom(), 1.0, theme.interaction.hairline);

        let measure_height = match self.measuring {
            true => 76.0,
            false => 0.0,
        };
        let [filter, list, measure] = Flex::column(rect)
            .fixed(56.0)
            .fill(1.0)
            .fixed(measure_height)
            .solve();

        let field = Pad::axes(11.0, 14.0).inside(filter);
        TextInput::search(&mut self.filter).hint("Filter stories…").label("Filter").show(rux, field);

        // The tree scrolls, because there are more stories than a short window has room for.
        let inner = Pad::sides(6.0, 10.0, 10.0, 10.0).inside(list);
        let content = self.sidebar_height(visible);
        let bar = rux::components::Scrollbar::new(list, content);
        let offset_id = rux.ui.id().with("rux-storybook-sidebar-scroll");
        let mut offset = rux.ui.memory(|memory| memory.data.get_temp(offset_id).unwrap_or(0.0));
        offset = bar.grab(rux, offset);
        if rux.ui.ctx().pointer_latest_pos().is_some_and(|at| list.contains(at)) {
            let wheel = rux.ui.ctx().input(|input| input.smooth_scroll_delta.y);
            offset = (offset - wheel).clamp(0.0, (content - list.height()).max(0.0));
        }

        let was = rux.ui.clip_rect();
        rux.ui.set_clip_rect(list.intersect(was));
        let mut pen = Pen::new(Rect::from_min_size(
            egui::pos2(inner.left(), inner.top() - offset),
            Vec2::new(inner.width(), content),
        ));
        let mut section = "";
        for index in visible {
            let story = &self.stories[*index];
            if story.section != section {
                section = story.section;
                pen.gap(12.0);
                let strip = pen.strip(18.0);
                let dim = theme.ink.i400;
                let galley = rux.text(Style::CAPTION.at(9.5), section, dim);
                rux.ui.painter().galley(egui::pos2(strip.left() + 8.0, strip.top()), galley, dim);
                pen.gap(4.0);
            }
            let row = pen.strip(30.0);
            if ListItem::new(story.name, *index == self.chosen)
                .pressed_when_selected()
                .pad(Pad::axes(7.0, 10.0))
                .show(rux, row)
                .clicked
            {
                self.chosen = *index;
            }
            pen.gap(1.0);
        }
        rux.ui.set_clip_rect(was);
        bar.paint(rux, offset);
        rux.ui.memory_mut(|memory| memory.data.insert_temp(offset_id, offset));

        if self.measuring {
            self.show_measurements(rux, measure, state);
        }
    }

    /// How tall the sidebar's tree comes out, which the scrollbar needs before anything is drawn.
    fn sidebar_height(&self, visible: &[usize]) -> f32 {
        let mut height = 10.0;
        let mut section = "";
        for index in visible {
            let story = &self.stories[*index];
            if story.section != section {
                section = story.section;
                height += 12.0 + 18.0 + 4.0;
            }
            height += 31.0;
        }
        height
    }

    /// `--measure` — what the library is holding and how often it has rasterised.
    ///
    /// Here rather than in a log so the numbers stay measured rather than remembered, which is the
    /// same argument the cost examples in Unluminous make.
    fn show_measurements(&mut self, rux: &mut Rux<'_>, rect: Rect, state: &RuxState) {
        let theme = rux.theme();
        rux.chrome.line(rect.left_top(), rect.right_top(), 1.0, theme.interaction.hairline);
        let footprint = state.footprint();
        let lines = [
            format!(
                "canvases {} · {} KB",
                footprint.canvases,
                footprint.canvas_bytes / 1024
            ),
            format!("marks {} · {} KB", footprint.marks, footprint.mark_bytes / 1024),
            format!(
                "rasterised {} / {}",
                footprint.canvas_rasterisations, footprint.mark_rasterisations
            ),
        ];
        let inner = Pad::axes(10.0, 14.0).inside(rect);
        let mut pen = Pen::new(inner);
        let dim = theme.ink.i400;
        for line in lines {
            let galley = rux.text(Style::mono(9.5), &line, dim);
            let strip = pen.strip(galley.size().y + 2.0);
            rux.ui.painter().galley(strip.left_top(), galley, dim);
        }
    }

    fn show_story(&mut self, rux: &mut Rux<'_>, rect: Rect) {
        let theme = rux.theme();
        rux.ground_sunken(rect);
        let Some(story) = self.stories.get(self.chosen) else { return };

        // A page fills the content area; a component story runs down it under a heading.
        if story.full_bleed {
            let draw = story.draw;
            draw(rux, &mut self.demo, rect);
            return;
        }

        let head_height = 66.0;
        let [head, body] = Flex::column(rect).fixed(head_height).fill(1.0).solve();
        let inner = Pad::sides(22.0, 40.0, 10.0, 40.0).inside(head);
        let ink = theme.ink.i900;
        let galley = rux.text(Style::H2.at(22.0), story.name, ink);
        rux.ui.painter().galley(inner.left_top(), galley, ink);
        let dim = theme.ink.i400;
        let galley = rux.elided(Style::mono(10.0), story.rule, dim, inner.width());
        rux.ui.painter().galley(egui::pos2(inner.left(), inner.top() + 30.0), galley, dim);

        // The page is drawn into a clip and lifted by however far it is scrolled, so no story has to
        // know it is being scrolled.
        let page = PAGE_PAD.inside(body);
        let offset_id = rux.ui.id().with(("rux-storybook-page-scroll", self.chosen));
        let mut offset = rux.ui.memory(|memory| memory.data.get_temp(offset_id).unwrap_or(0.0));
        let content = self.scroll[self.chosen].max(1.0) + PAGE_PAD.vertical();
        let bar = rux::components::Scrollbar::new(body, content);
        offset = bar.grab(rux, offset);
        if rux.ui.ctx().pointer_latest_pos().is_some_and(|at| body.contains(at)) {
            let wheel = rux.ui.ctx().input(|input| input.smooth_scroll_delta.y);
            if wheel != 0.0 {
                self.pending_scroll = None;
            }
            offset = (offset - wheel).clamp(0.0, (content - body.height()).max(0.0));
        }
        // `--scroll` holds the page where it was asked to be until the wheel moves it. It is applied
        // after the scrollbar's own clamp, because on the first frame the story's height is not known
        // and the clamp would put it back at the top.
        if let Some(points) = self.pending_scroll {
            offset = points.max(0.0);
        }

        let was = rux.ui.clip_rect();
        rux.ui.set_clip_rect(body.intersect(was));
        let drawn = {
            let draw = story.draw;
            let area = Rect::from_min_size(
                egui::pos2(page.left(), page.top() - offset),
                Vec2::new(page.width(), page.height().max(1.0)),
            );
            draw(rux, &mut self.demo, area)
        };
        rux.ui.set_clip_rect(was);
        bar.paint(rux, offset);
        rux.ui.memory_mut(|memory| memory.data.insert_temp(offset_id, offset));
        // What the page really came out at, which the next frame's scrollbar reads. Measured rather
        // than declared, because a story's height depends on the fonts and the width.
        self.scroll[self.chosen] = drawn;
    }

    /// Start with the chosen story scrolled down by this many points.
    ///
    /// The offset is held until the wheel moves the page, and it is not limited to the story's height.
    /// A capture past the end of a story shows an empty page, which is how a capture pass knows it has
    /// reached the end.
    pub fn scroll_to(&mut self, points: f32) {
        self.pending_scroll = Some(points);
    }

    /// Start with the sidebar filtered, as though it had been typed in.
    pub fn set_filter(&mut self, filter: &str) {
        self.filter = filter.to_owned();
    }

    /// Open every state that a press would otherwise be needed to reach.
    ///
    /// The dropdowns and the dialog. `--open` is what calls it, so a capture pass can photograph an
    /// open menu without a synthetic click — which on Windows would mean activating the window, and
    /// activating a window moves the virtual desktop with it.
    pub fn open_everything(&mut self) {
        self.demo.select.open = true;
        self.demo.select_mono.open = true;
        self.demo.select_long.open = true;
        self.demo.project_open = true;
        self.demo.modal_open = true;
        self.theme_select.open = false;
    }

    /// Open straight onto a story, by name, case insensitively.
    ///
    /// A name that is not there is refused with the list, which is the rule the library keeps
    /// everywhere it is asked to name something built in.
    pub fn choose_story(&mut self, name: &str) -> Result<(), String> {
        let wanted = name.to_lowercase();
        match self
            .stories
            .iter()
            .position(|story| story.name.to_lowercase() == wanted)
        {
            Some(index) => {
                self.chosen = index;
                Ok(())
            }
            None => {
                let names: Vec<&str> = self.stories.iter().map(|story| story.name).collect();
                Err(format!("there is no story called {name:?}. There is: {}", names.join(", ")))
            }
        }
    }

    /// A refusal that names what there is, for a theme asked for on the command line.
    pub fn choose_theme(&mut self, state: &RuxState, name: &str) -> Result<(), String> {
        match Theme::named(name) {
            Some(theme) => {
                state.set_theme(theme);
                self.theme_index =
                    Theme::all().iter().position(|entry| entry.name == name).unwrap_or(0);
                Ok(())
            }
            None => Err(format!(
                "there is no theme called {name:?}. There is: {}",
                Theme::names().join(", ")
            )),
        }
    }
}

/// Whether every character of `needle` appears in `haystack`, in order.
///
/// The same rule a fuzzy file finder uses, so `crtbtn` finds `Create Button`.
fn is_subsequence(needle: &str, haystack: &str) -> bool {
    let mut characters = haystack.chars();
    needle
        .chars()
        .filter(|character| !character.is_whitespace())
        .all(|wanted| characters.any(|character| character == wanted))
}

/// Unused-import guard: the shell names these in its own signatures.
const _: Option<(ButtonSize, ButtonVariant)> = None;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_filter_matches_a_subsequence() {
        assert!(is_subsequence("btn", "components button"));
        assert!(is_subsequence("crtbtn", "create button"));
        assert!(is_subsequence("", "anything"));
        assert!(!is_subsequence("zzz", "create button"));
        // Out of order does not match, which is what makes it a subsequence rather than a bag of
        // letters.
        assert!(!is_subsequence("nttub", "button"));
    }

    #[test]
    fn a_short_filter_does_not_match_every_story() {
        // The fault the two-rule matcher exists for: a subsequence over a long haystack matches
        // nearly anything, so `sel` used to leave nineteen of the twenty-four rows on the board.
        let stories = stories::all();
        let left = |filter: &str| {
            let storybook = Storybook {
                stories: stories::all(),
                chosen: 0,
                filter: filter.to_owned(),
                theme_names: Vec::new(),
                theme_index: 0,
                theme_select: SelectState::default(),
                demo: Demo::default(),
                scroll: vec![0.0; stories.len()],
                measuring: false,
                pending_scroll: None,
            };
            let names: Vec<&str> =
                storybook.matching().into_iter().map(|index| stories[index].name).collect();
            names
        };
        assert_eq!(left("").len(), stories.len(), "an empty filter leaves everything");
        let some = left("sel");
        assert!(some.contains(&"Select"), "`sel` has to find Select, and left {some:?}");
        assert!(
            some.len() * 2 < stories.len(),
            "`sel` left {} of {} rows: {some:?}",
            some.len(),
            stories.len()
        );
        assert!(left("zzzz").is_empty());
        // And the point of the subsequence half: initials still find the story.
        assert!(left("iconbtn").contains(&"Icon Button"));
        assert!(left("imgcrt").contains(&"Image Creator"));
    }
}
