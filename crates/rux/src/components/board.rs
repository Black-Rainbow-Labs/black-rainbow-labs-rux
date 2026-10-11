//! The Agent-Tasks design's parts: the view switch, the sprint ring, the search well, a lane and a
//! task card.
//!
//! `task-2241`. The source is the Claude Design canvas "Unluminous", page Agent-Tasks, artboard
//! "Agent-Tasks board · Dark" (`TasksBoard.dc.html`). It is drawn in the Agent-Chat design's values,
//! [`crate::theme::Chat`], with one addition the board needs: the chip, a flatter raised surface than a
//! card, which is what a lane's mark, a count, the chosen view and a card's round buttons are made of.
//!
//! Every part here draws and answers what was pressed. Which lane a ticket is in, what a search
//! matches and what Start does are the application's.

use egui::{Color32, Pos2, Rect, Response, Sense, Shape, Stroke, Vec2};

use super::round::{card, surface};
use super::{pointing, Visual};
use crate::icon::{Icon, Mark};
use crate::render::Fill;
use crate::text::Style;
use crate::theme::Chat;
use crate::Rux;

/// The chip's own gradient across `rect`.
pub fn chip(chat: &Chat, rect: Rect) -> Fill {
    Fill::gradient(chat.chip, rect)
}

/// A round chip with a mark in it: the disc that holds a lane's mark, and a card's assignee.
pub fn chip_disc(rux: &mut Rux<'_>, rect: Rect) {
    let chat = rux.theme().chat;
    rux.chrome.surface(rect, rect.width() / 2.0, chip(&chat, rect), chat.raised_chip());
}

/// The views of a board, as a pill switch: a carved track holding one pill per view, each a mark and a
/// word, with the chosen one raised out of it.
///
/// `padding: 5px; gap: 4px` round pills `height: 34px; padding: 0 14px 0 12px; gap: 7px` with a 13
/// point mark and 12 point words. Unlike [`super::PillSwitch`] each pill is as wide as what is in it.
/// When the switch is given less room than its words need it drops the words and keeps the marks.
pub struct ViewSwitch<'a> {
    views: &'a [(Icon, &'a str)],
    chosen: usize,
}

impl<'a> ViewSwitch<'a> {
    pub fn new(views: &'a [(Icon, &'a str)], chosen: usize) -> Self {
        Self { views, chosen }
    }

    fn pill_width(rux: &Rux<'_>, word: &str, words: bool) -> f32 {
        match words {
            true => rux.z(12.0 + 13.0 + 7.0 + 14.0) + rux.measure(rux.zs(Self::style()), word).x,
            false => rux.z(34.0),
        }
    }

    fn style() -> Style {
        Style::sans(12.0).medium()
    }

    /// How wide the switch is with its words, and 44 tall.
    pub fn measure(&self, rux: &Rux<'_>) -> Vec2 {
        self.measure_with(rux, true)
    }

    /// How wide the switch is with only its marks.
    pub fn measure_marks(&self, rux: &Rux<'_>) -> Vec2 {
        self.measure_with(rux, false)
    }

    fn measure_with(&self, rux: &Rux<'_>, words: bool) -> Vec2 {
        let pills: f32 = self.views.iter().map(|(_, word)| Self::pill_width(rux, word, words)).sum();
        let gaps = rux.z(4.0) * self.views.len().saturating_sub(1) as f32;
        Vec2::new(pills + gaps + rux.z(10.0), rux.z(44.0))
    }

    /// Draws the switch at the left of `rect` and answers the view pressed this frame, if any.
    pub fn show(self, rux: &mut Rux<'_>, rect: Rect) -> Option<usize> {
        let words = rect.width() >= self.measure(rux).x;
        let size = self.measure_with(rux, words);
        let track = Rect::from_min_size(Pos2::new(rect.left(), rect.center().y - size.y / 2.0), size);
        let chat = rux.theme().chat;
        let theme = rux.theme();
        rux.chrome.surface(track, track.height() / 2.0, chat.well, chat.carved_sm());
        let pad = rux.z(5.0);
        let mut left = track.left() + pad;
        let mut picked = None;
        for (index, (icon, word)) in self.views.iter().enumerate() {
            let width = Self::pill_width(rux, word, words);
            let pill = Rect::from_min_size(Pos2::new(left, track.top() + pad), Vec2::new(width, rux.z(34.0)));
            left += width + rux.z(4.0);
            let response = super::interact(rux, pill, "rux-view-switch", Sense::click());
            pointing(rux, &response, true);
            let chosen = self.chosen == index;
            if chosen {
                rux.chrome.surface(pill, pill.height() / 2.0, chip(&chat, pill), chat.raised_chip());
            }
            let ink = match (chosen, super::hovering(rux, &response)) {
                (true, _) => theme.ink.i900,
                (false, true) => chat.sky,
                (false, false) => theme.ink.i500,
            };
            match words {
                true => {
                    let centre = Pos2::new(pill.left() + rux.z(12.0 + 6.5), pill.center().y);
                    rux.mark(Mark::new(*icon, rux.z(13.0)), centre, ink);
                    let style = rux.zs(Self::style());
                    let galley = rux.text(style, word, ink);
                    crate::text::draw_left_capitals(
                        rux.painter(),
                        Pos2::new(pill.left() + rux.z(12.0 + 13.0 + 7.0), pill.center().y),
                        galley,
                        style,
                        ink,
                    );
                }
                false => rux.mark(Mark::new(*icon, rux.z(13.0)), pill.center(), ink),
            }
            let name = word.to_string();
            response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Button, true, chosen, name.clone()));
            if response.clicked() {
                picked = Some(index);
            }
        }
        picked
    }
}

/// The sprint's progress: a 44 point chip with a ring of how much of its work is done and the share
/// written in the middle.
///
/// The ring is `r="16"`, four points wide, round at its ends, in the action's gradient over the track
/// colour, starting at twelve o'clock.
pub struct SprintRing {
    fraction: f32,
}

impl SprintRing {
    pub fn new(fraction: f32) -> Self {
        Self { fraction: fraction.clamp(0.0, 1.0) }
    }

    pub fn measure(&self, rux: &Rux<'_>) -> Vec2 {
        Vec2::splat(rux.z(44.0))
    }

    pub fn show(self, rux: &mut Rux<'_>, rect: Rect) -> Response {
        let side = rux.z(44.0).min(rect.width()).min(rect.height());
        let rect = Rect::from_center_size(rect.center(), Vec2::splat(side));
        let response = super::interact(rux, rect, "rux-sprint-ring", Sense::hover());
        let chat = rux.theme().chat;
        let theme = rux.theme();
        chip_disc(rux, rect);
        let unit = side / 44.0;
        let centre = rect.center();
        let radius = 16.0 * unit;
        let width = 4.0 * unit;
        rux.chrome.ring(centre, radius, width, chat.track);
        if self.fraction > 0.0 {
            // `x1=0 y1=0 x2=1 y2=1` on an `<svg>` turned a quarter back: on the screen the gradient runs
            // from the bottom left of the ring to its top right.
            let box_ = Rect::from_center_size(centre, Vec2::splat(radius * 2.0));
            let fill = Fill::Linear {
                from: box_.left_bottom(),
                to: box_.right_top(),
                start: chat.action.from,
                end: chat.action.to,
            };
            let sweep = std::f32::consts::TAU * self.fraction;
            rux.chrome.arc(centre, radius, width, 0.0, sweep, fill);
        }
        let percent = format!("{}%", (self.fraction * 100.0).round() as i64);
        let style = rux.zs(Style::sans(10.0).semibold());
        let galley = rux.text(style, &percent, theme.ink.i900);
        crate::text::draw_centred(rux.painter(), rect, galley, theme.ink.i900);
        response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::ProgressIndicator, true, format!("Sprint {percent} done")));
        response
    }
}

/// A pill well with a magnifier at its left: where a search field goes.
///
/// `height: 42px; padding: 0 16px; gap: 10px`, carved with `--in-sm`, a 14 point mark in the quiet ink.
/// Answers the rectangle the field's words go in, which is the caller's to fill with a text box.
pub fn search_well(rux: &mut Rux<'_>, rect: Rect) -> Rect {
    let chat = rux.theme().chat;
    let theme = rux.theme();
    let height = rux.z(42.0).min(rect.height());
    let well = Rect::from_center_size(rect.center(), Vec2::new(rect.width(), height));
    rux.chrome.surface(well, well.height() / 2.0, chat.well, chat.carved_sm());
    let mark = Pos2::new(well.left() + rux.z(16.0 + 7.0), well.center().y);
    rux.mark(Mark::new(Icon::Search, rux.z(14.0)), mark, theme.ink.i400);
    Rect::from_min_max(
        Pos2::new(well.left() + rux.z(16.0 + 14.0 + 10.0), well.top()),
        Pos2::new(well.right() - rux.z(16.0), well.bottom()),
    )
}

/// What a lane's mark says, which is what the lane is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LaneMark {
    /// A clock in the quiet ink: not started.
    New,
    /// A cross in the slower colour: sent back.
    Failed,
    /// A quarter of a ring in the sky colour over the track: being worked on.
    Running,
    /// A tick in the faster colour: finished.
    Done,
}

impl LaneMark {
    /// Draw the mark in a 16 unit box centred on `centre`, `unit` points to the unit.
    ///
    /// Stroked with `egui` rather than recorded as decoration, so it sits on top of the chip it is in.
    pub fn paint(self, rux: &Rux<'_>, centre: Pos2, unit: f32) {
        let chat = rux.theme().chat;
        let theme = rux.theme();
        let painter = rux.painter();
        let at = |x: f32, y: f32| Pos2::new(centre.x + (x - 8.0) * unit, centre.y + (y - 8.0) * unit);
        let line = |points: Vec<Pos2>, width: f32, colour: Color32| {
            painter.add(Shape::line(points, Stroke::new(width * unit, colour)));
        };
        match self {
            LaneMark::New => {
                let ink = theme.ink.i400;
                painter.circle_stroke(centre, 6.0 * unit, Stroke::new(1.6 * unit, ink));
                line(vec![at(8.0, 4.9), at(8.0, 8.0), at(10.1, 9.4)], 1.6, ink);
            }
            LaneMark::Failed => {
                line(vec![at(5.2, 5.2), at(10.8, 10.8)], 1.9, chat.slower);
                line(vec![at(10.8, 5.2), at(5.2, 10.8)], 1.9, chat.slower);
            }
            LaneMark::Running => {
                painter.circle_stroke(centre, 6.0 * unit, Stroke::new(2.2 * unit, chat.track));
                // `M8 2a6 6 0 0 1 5.91 7.04`: from twelve o'clock, clockwise, to a tenth past three.
                let end = (1.04f32).atan2(5.91) + std::f32::consts::FRAC_PI_2;
                let points = (0..=16)
                    .map(|step| {
                        let angle = end * step as f32 / 16.0;
                        Pos2::new(centre.x + 6.0 * unit * angle.sin(), centre.y - 6.0 * unit * angle.cos())
                    })
                    .collect();
                line(points, 2.2, chat.sky);
            }
            LaneMark::Done => line(vec![at(4.3, 8.3), at(6.8, 10.8), at(11.7, 5.7)], 1.9, chat.faster),
        }
    }
}

/// One lane of a board: a well carved into the page, a heading of a round mark, the lane's name and a
/// count, and the room under the heading where its cards go.
///
/// `padding: 14px; gap: 14px; border-radius: 26px` with `--in`. The heading is `padding: 2px 2px 0`, a
/// 30 point chip with the mark in it, the name in 13 point semibold and the count in a chip at least
/// 26 points across.
pub struct Lane<'a> {
    name: &'a str,
    count: usize,
    mark: LaneMark,
}

/// The heading of a lane and the room under it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LaneLayout {
    /// The well itself.
    pub well: Rect,
    /// Where the cards go: inside the padding, under the heading and its gap.
    pub cards: Rect,
}

impl<'a> Lane<'a> {
    pub fn new(name: &'a str, count: usize, mark: LaneMark) -> Self {
        Self { name, count, mark }
    }

    /// How far the cards start below the top of the lane: the padding, the heading and the gap.
    pub fn heading(rux: &Rux<'_>) -> f32 {
        rux.z(14.0 + 2.0 + 30.0 + 14.0)
    }

    /// The padding round the lane's contents.
    pub fn padding(rux: &Rux<'_>) -> f32 {
        rux.z(14.0)
    }

    pub fn layout(rux: &Rux<'_>, rect: Rect) -> LaneLayout {
        let pad = Self::padding(rux);
        LaneLayout {
            well: rect,
            cards: Rect::from_min_max(
                Pos2::new(rect.left() + pad, rect.top() + Self::heading(rux)),
                Pos2::new(rect.right() - pad, rect.bottom() - pad),
            ),
        }
    }

    /// Draws the well and its heading and answers where the cards go.
    pub fn show(self, rux: &mut Rux<'_>, rect: Rect) -> LaneLayout {
        let chat = rux.theme().chat;
        let theme = rux.theme();
        rux.chrome.surface(rect, rux.z(26.0), chat.well, chat.carved());
        let pad = Self::padding(rux);
        let top = rect.top() + pad + rux.z(2.0);
        let middle = top + rux.z(15.0);
        let disc = Rect::from_center_size(Pos2::new(rect.left() + pad + rux.z(2.0 + 15.0), middle), Vec2::splat(rux.z(30.0)));
        chip_disc(rux, disc);
        self.mark.paint(rux, disc.center(), rux.z(1.0));
        let count = self.count.to_string();
        let count_style = rux.zs(Style::sans(11.5));
        let count_galley = rux.text(count_style, &count, theme.ink.i500);
        let width = (count_galley.size().x + rux.z(16.0)).max(rux.z(26.0));
        let chip_rect = Rect::from_min_size(
            Pos2::new(rect.right() - pad - rux.z(2.0) - width, middle - rux.z(13.0)),
            Vec2::new(width, rux.z(26.0)),
        );
        rux.chrome.surface(chip_rect, chip_rect.height() / 2.0, chip(&chat, chip_rect), chat.raised_chip());
        crate::text::draw_centred(rux.painter(), chip_rect, count_galley, theme.ink.i500);
        let name_left = disc.right() + rux.z(10.0);
        let name_style = rux.zs(Style::sans(13.0).semibold());
        let room = (chip_rect.left() - rux.z(10.0) - name_left).max(0.0);
        let name = rux.elided(name_style, self.name, theme.ink.i900, room);
        crate::text::draw_left_capitals(rux.painter(), Pos2::new(name_left, middle), name, name_style, theme.ink.i900);
        Self::layout(rux, rect)
    }
}

/// Who a ticket belongs to, drawn as a mark in a round chip.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Assignee {
    /// An agent: the spark, in the colour the application gives agents.
    Agent(Color32),
    /// A person: the figure, in the quiet ink.
    Person,
}

/// What the round button at the end of a card does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardAction {
    /// The play mark on a chip: start the ticket's agent.
    Start,
    /// The pause mark carved in, in the sky colour: the agent is running and this stops it.
    Pause,
    /// No button, because nothing could be done.
    None,
}

/// What a task card reported.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TaskCardOutcome {
    /// The round button was pressed.
    pub action: bool,
}

/// One ticket on a board.
///
/// `padding: 16px 16px 14px; gap: 14px; border-radius: 20px` on the card gradient with `--out`. The
/// title is 13 point semibold at a line height of 1.4, two lines at most. Under it the todos as a track
/// eight points tall with the count beside it in 11 points, and at the foot the key in the code face,
/// the assignee in a 28 point chip and the round button, 32 points.
///
/// The card does not claim the pointer for itself: the application owns what a press or a drag on a
/// card means, so it interacts with the card's rectangle **before** showing it, and the round button
/// added here, after it, is what egui gives a press on the button to.
pub struct TaskCard<'a> {
    title: &'a str,
    key: &'a str,
    done: u32,
    total: u32,
    assignee: Assignee,
    action: CardAction,
    action_label: &'a str,
    hovered: bool,
    chosen: bool,
}

impl<'a> TaskCard<'a> {
    pub fn new(title: &'a str, key: &'a str) -> Self {
        Self {
            title,
            key,
            done: 0,
            total: 0,
            assignee: Assignee::Person,
            action: CardAction::None,
            action_label: "",
            hovered: false,
            chosen: false,
        }
    }

    /// How many of the ticket's todos are done, of how many.
    pub fn todos(mut self, done: u32, total: u32) -> Self {
        self.done = done;
        self.total = total;
        self
    }

    pub fn assignee(mut self, assignee: Assignee) -> Self {
        self.assignee = assignee;
        self
    }

    /// The round button, and the name a screen reader is told it has.
    pub fn action(mut self, action: CardAction, label: &'a str) -> Self {
        self.action = action;
        self.action_label = label;
        self
    }

    /// The pointer is over the card, or it is being carried: a wash over it.
    pub fn hovered(mut self, hovered: bool) -> Self {
        self.hovered = hovered;
        self
    }

    /// The keyboard is on this card: the focus ring round it.
    pub fn chosen(mut self, chosen: bool) -> Self {
        self.chosen = chosen;
        self
    }

    fn title_style() -> Style {
        Style::sans(13.0).semibold().leading(1.4)
    }

    fn title_galley(&self, rux: &Rux<'_>, width: f32) -> std::sync::Arc<egui::Galley> {
        Self::title_in(rux.painter(), rux.zoom(), self.title, width, rux.theme().ink.i900)
    }

    fn title_in(
        painter: &egui::Painter,
        zoom: f32,
        title: &str,
        width: f32,
        ink: Color32,
    ) -> std::sync::Arc<egui::Galley> {
        let style = Self::title_style();
        let style = style.at(style.size * zoom);
        let mut job = egui::text::LayoutJob::single_section(title.to_owned(), style.format(ink));
        job.wrap = egui::text::TextWrapping {
            max_width: width.max(1.0),
            max_rows: 2,
            break_anywhere: false,
            overflow_character: Some('\u{2026}'),
        };
        painter.layout_job(job)
    }

    /// How tall the card is at `width`: one or two lines of title, then the track and the foot.
    pub fn height(&self, rux: &Rux<'_>, width: f32) -> f32 {
        Self::height_of(rux.painter(), rux.zoom(), self.title, width)
    }

    /// The same, for a caller that has to know how tall its cards are before it opens a layer: a board
    /// works out how far each lane can scroll before it draws any of them.
    pub fn height_of(painter: &egui::Painter, zoom: f32, title: &str, width: f32) -> f32 {
        let title = Self::title_in(painter, zoom, title, width - 32.0 * zoom, Color32::WHITE).size().y;
        (16.0 + 14.0 + 14.0 + 14.0 + 32.0 + 14.0) * zoom + title
    }

    /// The fill of the todo track, as a share of its length.
    pub fn fraction(done: u32, total: u32) -> f32 {
        match total {
            0 => 0.0,
            total => (done as f32 / total as f32).clamp(0.0, 1.0),
        }
    }

    pub fn show(self, rux: &mut Rux<'_>, rect: Rect) -> TaskCardOutcome {
        let chat = rux.theme().chat;
        let theme = rux.theme();
        let radius = rux.z(20.0);
        if self.chosen {
            super::round::focus_ring(rux, rect, radius, &chat);
        }
        rux.chrome.surface(rect, radius, card(&chat, rect), chat.raised());
        if self.hovered {
            rux.wash(rect, radius, theme.interaction.hover);
        }
        let inner = Rect::from_min_max(
            Pos2::new(rect.left() + rux.z(16.0), rect.top() + rux.z(16.0)),
            Pos2::new(rect.right() - rux.z(16.0), rect.bottom() - rux.z(14.0)),
        );
        let title = self.title_galley(rux, inner.width());
        let title_height = title.size().y;
        rux.painter().galley(inner.left_top(), title, theme.ink.i900);

        // The track and its count.
        let row = inner.top() + title_height + rux.z(14.0 + 7.0);
        let count = format!("{}/{}", self.done, self.total);
        let count_style = rux.zs(Style::sans(11.0));
        let count_galley = rux.text(count_style, &count, theme.ink.i400);
        let count_left = inner.right() - count_galley.size().x;
        crate::text::draw_left_capitals(rux.painter(), Pos2::new(count_left, row), count_galley, count_style, theme.ink.i400);
        let groove = Rect::from_min_max(
            Pos2::new(inner.left(), row - rux.z(4.0)),
            Pos2::new(count_left - rux.z(10.0), row + rux.z(4.0)),
        );
        if groove.width() > rux.z(8.0) {
            rux.chrome.surface(groove, groove.height() / 2.0, chat.well, chat.carved_sm());
            let fraction = Self::fraction(self.done, self.total);
            let inset = rux.z(2.0);
            let length = groove.width() * fraction - inset * 2.0;
            if length > 0.5 {
                let fill = Rect::from_min_size(
                    Pos2::new(groove.left() + inset, groove.top() + inset),
                    Vec2::new(length, groove.height() - inset * 2.0),
                );
                let gradient = match fraction >= 1.0 {
                    true => chat.done,
                    false => chat.action,
                };
                rux.chrome.rect(fill, fill.height() / 2.0, Fill::gradient(gradient, fill));
            }
        }

        // The foot: the key, who has it, and the round button.
        let foot = inner.bottom() - rux.z(16.0);
        let mut right = inner.right();
        let mut outcome = TaskCardOutcome::default();
        if self.action != CardAction::None {
            let button = Rect::from_center_size(Pos2::new(right - rux.z(16.0), foot), Vec2::splat(rux.z(32.0)));
            outcome.action = card_button(rux, button, self.action, self.action_label).clicked();
            right = button.left() - rux.z(10.0);
        }
        let badge = Rect::from_center_size(Pos2::new(right - rux.z(14.0), foot), Vec2::splat(rux.z(28.0)));
        chip_disc(rux, badge);
        let (icon, ink) = match self.assignee {
            Assignee::Agent(colour) => (Icon::Spark, colour),
            Assignee::Person => (Icon::User, theme.ink.i400),
        };
        rux.mark(Mark::new(icon, rux.z(12.0)).stroke(2.0), badge.center(), ink);
        let key_style = rux.zs(Style::mono(11.5));
        let key = rux.elided(key_style, self.key, theme.ink.i400, (badge.left() - rux.z(10.0) - inner.left()).max(0.0));
        crate::text::draw_left_capitals(rux.painter(), Pos2::new(inner.left(), foot), key, key_style, theme.ink.i400);
        outcome
    }
}

/// A round button on the chip surface with a mark in it.
///
/// The ticket modal's close button (36 points), the button that adds a todo (30) and the disc that folds the
/// todos away (24). At rest a raised chip with the mark in the quieter ink; under the pointer the mark is the
/// sky colour; held down it is carved in.
pub struct ChipButton<'a> {
    icon: Icon,
    diameter: f32,
    mark: f32,
    stroke: Option<f32>,
    turn: f32,
    label: &'a str,
    /// For a button that folds a section: whether the section is open, which a screen reader is told.
    open: Option<bool>,
}

impl<'a> ChipButton<'a> {
    pub fn new(icon: Icon, diameter: f32, label: &'a str) -> Self {
        Self { icon, diameter, mark: 12.0, stroke: None, turn: 0.0, label, open: None }
    }

    /// The button folds a section, which is open or shut.
    pub fn folds(mut self, open: bool) -> Self {
        self.open = Some(open);
        self
    }

    /// The mark's size, and its stroke when the mark's own is not wanted.
    pub fn mark(mut self, size: f32, stroke: Option<f32>) -> Self {
        self.mark = size;
        self.stroke = stroke;
        self
    }

    /// Turn the mark, as the fold disc's chevron turns when the list is open.
    pub fn turn(mut self, radians: f32) -> Self {
        self.turn = radians;
        self
    }

    pub fn measure(&self, rux: &Rux<'_>) -> Vec2 {
        Vec2::splat(rux.z(self.diameter))
    }

    pub fn show(self, rux: &mut Rux<'_>, rect: Rect) -> Response {
        let side = rux.z(self.diameter).min(rect.width()).min(rect.height());
        let rect = Rect::from_center_size(rect.center(), Vec2::splat(side));
        let response = super::interact_target(rux, rect, "rux-chip-button", Sense::click());
        let visual = Visual::of(&response, true, rux.screen().pointer);
        pointing(rux, &response, true);
        let chat = rux.theme().chat;
        let theme = rux.theme();
        match visual {
            Visual::Active => surface(rux, rect, side / 2.0, chat.well.into(), chat.carved_sm(), 1.0),
            _ => surface(rux, rect, side / 2.0, chip(&chat, rect), chat.raised_chip(), 1.0),
        }
        let ink = match visual {
            Visual::Hover => chat.sky,
            Visual::Active => theme.ink.i900,
            _ => theme.ink.i500,
        };
        let mut mark = Mark::new(self.icon, rux.z(self.mark)).turn(self.turn);
        if let Some(stroke) = self.stroke {
            mark = mark.stroke(stroke);
        }
        rux.mark(mark, rect.center(), ink);
        let label = self.label.to_owned();
        match self.open {
            Some(open) => response
                .widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Button, true, open, label.clone())),
            None => response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, label.clone())),
        }
        response
    }
}

/// A round tick box, 22 points: a raised chip while it is not ticked, and carved in with the faster colour's
/// tick once it is. The ticket modal's todos.
pub fn round_check(rux: &mut Rux<'_>, rect: Rect, ticked: bool, label: &str) -> Response {
    let side = rux.z(22.0).min(rect.width()).min(rect.height());
    let rect = Rect::from_center_size(rect.center(), Vec2::splat(side));
    let response = super::interact_target(rux, rect, "rux-round-check", Sense::click());
    pointing(rux, &response, true);
    let chat = rux.theme().chat;
    match ticked {
        true => {
            rux.chrome.surface(rect, side / 2.0, chat.well, chat.carved_sm());
            rux.mark(Mark::new(Icon::Tick, rux.z(12.0)).stroke(1.9), rect.center(), chat.faster);
        }
        false => rux.chrome.surface(rect, side / 2.0, chip(&chat, rect), chat.raised_chip()),
    }
    if super::hovering(rux, &response) {
        rux.wash(rect, side / 2.0, rux.theme().interaction.hover);
    }
    let label = label.to_owned();
    response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Checkbox, true, ticked, label.clone()));
    response
}

/// The 32 point round button at the end of a card: play on a chip, or pause carved in.
fn card_button(rux: &mut Rux<'_>, rect: Rect, action: CardAction, label: &str) -> Response {
    let response = super::interact_target(rux, rect, "rux-card-button", Sense::click());
    let visual = Visual::of(&response, true, rux.screen().pointer);
    pointing(rux, &response, true);
    let chat = rux.theme().chat;
    let theme = rux.theme();
    let radius = rect.width() / 2.0;
    let sunk = action == CardAction::Pause || matches!(visual, Visual::Active);
    match sunk {
        true => surface(rux, rect, radius, chat.well.into(), chat.carved_sm(), 1.0),
        false => surface(rux, rect, radius, chip(&chat, rect), chat.raised_chip(), 1.0),
    }
    let ink = match (action, visual) {
        (CardAction::Pause, _) => chat.sky,
        (_, Visual::Hover) => chat.sky,
        (_, Visual::Active) => theme.ink.i900,
        _ => theme.ink.i500,
    };
    let (icon, nudge) = match action {
        CardAction::Pause => (Icon::Pause, 0.0),
        _ => (Icon::Play, 2.0),
    };
    let centre = rect.center() + Vec2::new(rux.z(nudge) / 2.0, 0.0);
    rux.mark(Mark::new(icon, rux.z(12.0)).stroke(2.2), centre, ink);
    let label = label.to_owned();
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, label.clone()));
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_card_with_no_todos_has_an_empty_track() {
        assert_eq!(TaskCard::fraction(0, 0), 0.0);
        assert_eq!(TaskCard::fraction(6, 7), 6.0 / 7.0);
        assert_eq!(TaskCard::fraction(9, 9), 1.0);
        assert_eq!(TaskCard::fraction(12, 9), 1.0, "more done than there are is all of it");
    }
}
