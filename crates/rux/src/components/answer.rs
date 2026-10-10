//! The smaller parts of the Agent-Chat design that an answer is built from: the card and the well
//! every part sits on, a readout with its sparkline, the rows of a chart's legend, a file chip, the
//! pill that says the agent is thinking, and the note that says the context was compacted.
//!
//! `task-2235`. Read off the canvas's "Readouts", "Charts", "Messages" and "Tool runs" panels. A card
//! is extruded from the pane with the design's paired shadow and a 22 point radius; a well is carved
//! into it. Nothing here is a dot or a segmented bar: an amount is a line or an arc, and state is a
//! mark in a well.

use egui::epaint::{PathShape, PathStroke};
use egui::{Color32, Pos2, Rect, Response, Sense, Shape, Vec2};

use super::cards::Lead;
use super::dial::{smooth, SlicePaint};
use super::round::card;
use super::status::StatusWell;
use crate::icon::{Icon, Mark};
use crate::render::Fill;
use crate::text::Style;
use crate::Rux;

/// A card's corner radius: `border-radius: 22px`.
pub const CARD_RADIUS: f32 = 22.0;
/// A chart card's corner radius: `border-radius: 26px`.
pub const CHART_RADIUS: f32 = 26.0;
/// A well's corner radius inside a card: `border-radius: 16px`.
pub const WELL_RADIUS: f32 = 16.0;

/// A card extruded from the pane: the card's gradient and the big raised pair, `--out`.
pub fn chat_card(rux: &mut Rux<'_>, rect: Rect, radius: f32) {
    let chat = rux.theme().chat;
    rux.chrome.surface(rect, radius, card(&chat, rect), chat.raised());
}

/// A small raised surface: a pill, a chip, a bubble, `--out-sm`.
pub fn chat_raised_sm(rux: &mut Rux<'_>, rect: Rect, radius: f32) {
    let chat = rux.theme().chat;
    rux.chrome.surface(rect, radius, card(&chat, rect), chat.raised_sm());
}

/// A well carved into a card, `--in-sm`.
pub fn chat_well(rux: &mut Rux<'_>, rect: Rect, radius: f32) {
    let chat = rux.theme().chat;
    rux.chrome.surface(rect, radius, chat.well, chat.carved_sm());
}

/// A deeper well, `--in`: the prompt, the channel of a ring.
pub fn chat_well_deep(rux: &mut Rux<'_>, rect: Rect, radius: f32) {
    let chat = rux.theme().chat;
    rux.chrome.surface(rect, radius, chat.well, chat.carved());
}

/// The header row of a card: a 34 point well holding a state or a mark, a title, and a quieter line
/// under it. `row` is the whole row; the words are elided to fit `row` less the well.
///
/// The same header every card in [`super::cards`] has, for an answer's own parts.
pub fn card_header(
    rux: &mut Rux<'_>,
    row: Rect,
    lead: Lead,
    title: &str,
    detail: &str,
    detail_colour: Option<Color32>,
) {
    let theme = rux.theme();
    let side = rux.z(34.0);
    let well = Rect::from_min_size(Pos2::new(row.left(), row.center().y - side / 2.0), Vec2::splat(side));
    match lead {
        Lead::Status(status) => {
            StatusWell::new(status).show(rux, well);
        }
        Lead::Mark(icon) => {
            chat_well(rux, well, side / 2.0);
            rux.mark(Mark::new(icon, rux.z(18.0)), well.center(), theme.ink.i500);
        }
    }
    let left = well.right() + rux.z(12.0);
    let room = (row.right() - left).max(0.0);
    let title = rux.elided(rux.zs(Style::sans(12.5)), title, theme.ink.i900, room);
    let colour = detail_colour.unwrap_or(theme.ink.i400);
    let painter = rux.painter().clone();
    match detail.is_empty() {
        true => {
            painter.galley(Pos2::new(left, row.center().y - title.size().y / 2.0), title, theme.ink.i900);
        }
        false => {
            let detail = rux.elided(rux.zs(Style::sans(11.0)), detail, colour, room);
            let gap = rux.z(1.0);
            let top = row.center().y - (title.size().y + gap + detail.size().y) / 2.0;
            let below = top + title.size().y + gap;
            painter.galley(Pos2::new(left, top), title, theme.ink.i900);
            painter.galley(Pos2::new(left, below), detail, colour);
        }
    }
}

/// A sparkline: a smooth line through the values, two points wide, in one colour.
pub struct Sparkline<'a> {
    values: &'a [f32],
    colour: Color32,
}

impl<'a> Sparkline<'a> {
    pub fn new(values: &'a [f32], colour: Color32) -> Self {
        Self { values, colour }
    }

    /// The canvas's sparkline: 26 points tall, as wide as it is given.
    pub fn measure(&self, rux: &Rux<'_>) -> Vec2 {
        Vec2::new(rux.z(100.0), rux.z(26.0))
    }

    pub fn show(self, rux: &mut Rux<'_>, rect: Rect) {
        let points = spark_points(self.values, rect);
        if points.len() < 2 {
            return;
        }
        let curve = smooth(&points);
        let stroke = PathStroke::new(rux.z(2.0), self.colour);
        rux.painter().add(Shape::Path(PathShape::line(curve, stroke)));
    }
}

/// Each value's place in a sparkline's box: `x` from 2 to 98 of 100 and `y` from 22 at the lowest
/// value to 4 at the highest, of 26. A line of equal values runs along the middle.
pub fn spark_points(values: &[f32], rect: Rect) -> Vec<Pos2> {
    let low = values.iter().copied().fold(f32::INFINITY, f32::min);
    let high = values.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let span = high - low;
    let count = values.len().max(2) - 1;
    values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let x = rect.left() + rect.width() * (2.0 + index as f32 * 96.0 / count as f32) / 100.0;
            let share = match span > f32::EPSILON {
                true => (value - low) / span,
                false => 0.5,
            };
            let y = rect.top() + rect.height() * (22.0 - share * 18.0) / 26.0;
            Pos2::new(x, y)
        })
        .collect()
}

/// A readout: what it is, a number with its unit, how it changed, and a sparkline of the last few.
pub struct Readout<'a> {
    label: &'a str,
    value: &'a str,
    unit: &'a str,
    delta: Option<(&'a str, Color32)>,
    note: &'a str,
    spark: &'a [f32],
}

impl<'a> Readout<'a> {
    pub fn new(label: &'a str, value: &'a str, unit: &'a str) -> Self {
        Self { label, value, unit, delta: None, note: "", spark: &[] }
    }

    /// How it changed: `+39%` in the slower colour, `−7%` in the faster.
    pub fn delta(mut self, text: &'a str, colour: Color32) -> Self {
        if !text.is_empty() {
            self.delta = Some((text, colour));
        }
        self
    }

    /// A quiet line under the change.
    pub fn note(mut self, note: &'a str) -> Self {
        self.note = note;
        self
    }

    /// The values the sparkline is drawn through, in the delta's colour.
    pub fn spark(mut self, values: &'a [f32]) -> Self {
        self.spark = values;
        self
    }

    /// `padding: 16px 14px 14px; gap: 6px`, the value 24 points, the sparkline 26 tall with 4 above.
    pub fn height(&self, rux: &Rux<'_>) -> f32 {
        let mut height = rux.z(16.0 + 14.0);
        height += rux.measure(rux.zs(Style::sans(11.0)), "A").y;
        height += rux.z(6.0) + rux.measure(rux.zs(value_style()), "0").y;
        if self.delta.is_some() {
            height += rux.z(6.0) + rux.measure(rux.zs(Style::sans(11.0)), "0").y;
        }
        if !self.note.is_empty() {
            height += rux.z(4.0) + rux.measure(rux.zs(Style::sans(11.0)), "0").y;
        }
        if self.spark.len() >= 2 {
            height += rux.z(6.0 + 4.0 + 26.0);
        }
        height
    }

    pub fn show(self, rux: &mut Rux<'_>, rect: Rect) -> Response {
        let response = super::interact(rux, rect, "rux-readout", Sense::hover());
        chat_card(rux, rect, rux.z(CARD_RADIUS));
        let theme = rux.theme();
        let left = rect.left() + rux.z(14.0);
        let room = rect.width() - rux.z(28.0);
        let painter = rux.painter().clone();
        let mut top = rect.top() + rux.z(16.0);
        let small = rux.zs(Style::sans(11.0));
        let label = rux.elided(small, self.label, theme.ink.i400, room);
        top += label.size().y;
        painter.galley(Pos2::new(left, rect.top() + rux.z(16.0)), label, theme.ink.i400);
        top += rux.z(6.0);
        let value = rux.elided(rux.zs(value_style()), self.value, theme.ink.i900, room);
        let unit = rux.text(rux.zs(Style::sans(12.0)), self.unit, theme.ink.i400);
        let value_base = value.rows.first().map_or(value.size().y, |row| row.pos.y + row.row.height());
        let unit_base = unit.rows.first().map_or(unit.size().y, |row| row.pos.y + row.row.height());
        let value_height = value.size().y;
        let value_width = value.size().x;
        painter.galley(Pos2::new(left, top), value, theme.ink.i900);
        if !self.unit.is_empty() && value_width + unit.size().x <= room {
            painter.galley(Pos2::new(left + value_width, top + value_base - unit_base), unit, theme.ink.i400);
        }
        top += value_height;
        let spark_colour = self.delta.map_or(theme.chat.sky, |(_, colour)| colour);
        if let Some((text, colour)) = self.delta {
            top += rux.z(6.0);
            let galley = rux.elided(small, text, colour, room);
            let height = galley.size().y;
            painter.galley(Pos2::new(left, top), galley, colour);
            top += height;
        }
        if !self.note.is_empty() {
            top += rux.z(4.0);
            let galley = rux.elided(small, self.note, theme.ink.i400, room);
            let height = galley.size().y;
            painter.galley(Pos2::new(left, top), galley, theme.ink.i400);
            top += height;
        }
        if self.spark.len() >= 2 {
            top += rux.z(6.0 + 4.0);
            let line = Rect::from_min_size(Pos2::new(left, top), Vec2::new(room, rux.z(26.0)));
            Sparkline::new(self.spark, spark_colour).show(rux, line);
        }
        let label = format!("{}: {}{}", self.label, self.value, self.unit);
        response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Label, true, label.clone()));
        response
    }
}

/// The readout's number: `font-size: 24px; font-weight: 500; letter-spacing: -0.03em`.
fn value_style() -> Style {
    Style::sans(24.0).medium().tracking(-0.03).leading(1.1)
}

/// One row of a ring chart's legend: a swatch, a name and a value.
#[derive(Debug, Clone, PartialEq)]
pub struct LegendRow {
    pub name: String,
    pub value: String,
    pub paint: SlicePaint,
}

/// A ring chart's legend: `gap: 12px` between rows of a ten point swatch, the name in 12 point ink and
/// the value in 13 point medium at the right.
pub struct Legend<'a> {
    rows: &'a [LegendRow],
}

impl<'a> Legend<'a> {
    pub fn new(rows: &'a [LegendRow]) -> Self {
        Self { rows }
    }

    pub fn height(&self, rux: &Rux<'_>) -> f32 {
        let line = rux.measure(rux.zs(Style::sans(13.0).medium()), "0").y;
        let count = self.rows.len() as f32;
        (line * count + rux.z(12.0) * (count - 1.0)).max(0.0)
    }

    pub fn show(self, rux: &mut Rux<'_>, rect: Rect) {
        let theme = rux.theme();
        let chat = theme.chat;
        let line = rux.measure(rux.zs(Style::sans(13.0).medium()), "0").y;
        let mut top = rect.top();
        for row in self.rows {
            let middle = top + line / 2.0;
            let swatch = Rect::from_center_size(Pos2::new(rect.left() + rux.z(5.0), middle), Vec2::splat(rux.z(10.0)));
            let fill = match row.paint {
                SlicePaint::Action => Fill::gradient(chat.action, swatch),
                SlicePaint::Colour(colour) => Fill::Solid(colour),
            };
            rux.chrome.rect(swatch, swatch.width() / 2.0, fill);
            let value = rux.text(rux.zs(Style::sans(13.0).medium()), &row.value, theme.ink.i900);
            let value_left = rect.right() - value.size().x;
            let name_left = swatch.right() + rux.z(10.0);
            let name = rux.elided(
                rux.zs(Style::sans(12.0)),
                &row.name,
                theme.ink.i700,
                (value_left - name_left - rux.z(8.0)).max(0.0),
            );
            let painter = rux.painter().clone();
            painter.galley(Pos2::new(name_left, middle - name.size().y / 2.0), name, theme.ink.i700);
            painter.galley(Pos2::new(value_left, middle - value.size().y / 2.0), value, theme.ink.i900);
            top += line + rux.z(12.0);
        }
    }
}

/// A file named in a run of reads, or in an answer: a raised pill with a file mark and the name.
pub struct FileChip<'a> {
    name: &'a str,
    label: String,
}

impl<'a> FileChip<'a> {
    pub fn new(name: &'a str) -> Self {
        Self { name, label: format!("Open {name}") }
    }

    /// What a screen reader is told pressing it does.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// `height: 30px; padding: 0 12px 0 8px; gap: 4px`, a 15 point mark.
    pub fn measure(&self, rux: &Rux<'_>) -> Vec2 {
        let words = rux.measure(rux.zs(Style::sans(11.5)), self.name).x;
        Vec2::new(rux.z(8.0 + 15.0 + 4.0 + 12.0) + words, rux.z(30.0))
    }

    pub fn show(self, rux: &mut Rux<'_>, rect: Rect) -> Response {
        let response = super::interact(rux, rect, "rux-file-chip", Sense::click());
        let theme = rux.theme();
        let hovered = response.hovered();
        if hovered {
            rux.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        let chat = theme.chat;
        match response.is_pointer_button_down_on() {
            true => rux.chrome.surface(rect, rect.height() / 2.0, chat.well, chat.carved_sm()),
            false => chat_raised_sm(rux, rect, rect.height() / 2.0),
        }
        let mark_centre = Pos2::new(rect.left() + rux.z(8.0 + 7.5), rect.center().y);
        let mark_colour = if hovered { chat.sky } else { theme.ink.i400 };
        rux.mark(Mark::new(Icon::File, rux.z(15.0)), mark_centre, mark_colour);
        let words_colour = if hovered { chat.sky } else { theme.ink.i700 };
        let left = rect.left() + rux.z(8.0 + 15.0 + 4.0);
        let galley =
            rux.elided(rux.zs(Style::sans(11.5)), self.name, words_colour, (rect.right() - left - rux.z(12.0)).max(0.0));
        rux.painter().galley(Pos2::new(left, rect.center().y - galley.size().y / 2.0), galley, words_colour);
        let label = self.label;
        response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, label.clone()));
        response
    }
}

/// How long the thinking pill's arc takes to go once round.
const THINK_TURN: f64 = 1.4;

/// The pill that says the agent is thinking: its avatar with an arc going round it, what it is doing,
/// and for how long. While it is not running the arc stands still and the pill reads as a record.
pub struct ThinkingPill<'a> {
    words: &'a str,
    time: &'a str,
    colour: Color32,
    running: bool,
}

impl<'a> ThinkingPill<'a> {
    /// `colour` is the agent's own, which the avatar's spark and the arc are drawn in.
    pub fn new(words: &'a str, time: &'a str, colour: Color32) -> Self {
        Self { words, time, colour, running: true }
    }

    pub fn running(mut self, running: bool) -> Self {
        self.running = running;
        self
    }

    /// `padding: 6px 18px 6px 6px; gap: 12px`, a 30 point avatar.
    pub fn measure(&self, rux: &Rux<'_>) -> Vec2 {
        let words = rux.measure(rux.zs(Style::sans(12.0)), self.words).x;
        let time = match self.time.is_empty() {
            true => 0.0,
            false => rux.z(12.0) + rux.measure(rux.zs(Style::sans(11.0)), self.time).x,
        };
        Vec2::new(rux.z(6.0 + 30.0 + 12.0 + 18.0) + words + time, rux.z(42.0))
    }

    pub fn show(self, rux: &mut Rux<'_>, rect: Rect) -> Response {
        let response = super::interact(rux, rect, "rux-thinking", Sense::click());
        let theme = rux.theme();
        let chat = theme.chat;
        chat_raised_sm(rux, rect, rect.height() / 2.0);
        let avatar = Rect::from_min_size(rect.min + Vec2::splat(rux.z(6.0)), Vec2::splat(rux.z(30.0)));
        let centre = avatar.center();
        // `r="13" stroke-width="2"`, the track and a 22 of 82 dash in the agent's colour.
        let radius = rux.z(13.0);
        let width = rux.z(2.0);
        rux.chrome.ring(centre, radius, width, chat.track);
        let turn = match self.running && !rux.state.is_still() {
            true => {
                let time = rux.ctx().input(|input| input.time);
                rux.ctx().request_repaint();
                ((time / THINK_TURN).fract() * std::f64::consts::TAU) as f32
            }
            false => 0.0,
        };
        let painter = rux.painter().clone();
        crate::icon::paint_arc(
            &painter,
            &mut rux.state.icons.borrow_mut(),
            centre,
            radius,
            width,
            22.0 / 13.0,
            turn,
            self.colour,
            self.colour,
        );
        rux.mark(Mark::new(Icon::Spark, rux.z(13.0)).stroke(2.0), centre, self.colour);
        let mut x = avatar.right() + rux.z(12.0);
        let words = rux.text(rux.zs(Style::sans(12.0)), self.words, theme.ink.i700);
        let wide = words.size().x;
        painter.galley(Pos2::new(x, rect.center().y - words.size().y / 2.0), words, theme.ink.i700);
        x += wide + rux.z(12.0);
        if !self.time.is_empty() {
            let time = rux.text(rux.zs(Style::sans(11.0)), self.time, theme.ink.i400);
            painter.galley(Pos2::new(x, rect.center().y - time.size().y / 2.0), time, theme.ink.i400);
        }
        let label = self.words.to_owned();
        response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, label.clone()));
        response
    }
}

/// A quiet note in a well: `Context compacted · 41k → 9k tokens`.
pub struct NotePill<'a> {
    words: &'a str,
}

impl<'a> NotePill<'a> {
    pub fn new(words: &'a str) -> Self {
        Self { words }
    }

    /// `padding: 7px 16px`, 11 point words.
    pub fn measure(&self, rux: &Rux<'_>) -> Vec2 {
        let size = rux.measure(rux.zs(Style::sans(11.0)), self.words);
        Vec2::new(size.x + rux.z(32.0), size.y + rux.z(14.0))
    }

    pub fn show(self, rux: &mut Rux<'_>, rect: Rect) -> Response {
        let response = super::interact(rux, rect, "rux-note", Sense::hover());
        let theme = rux.theme();
        chat_well(rux, rect, rect.height() / 2.0);
        let galley = rux.elided(rux.zs(Style::sans(11.0)), self.words, theme.ink.i400, rect.width() - rux.z(32.0));
        rux.painter().galley(rect.center() - galley.size() / 2.0, galley, theme.ink.i400);
        let label = self.words.to_owned();
        response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Label, true, label.clone()));
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_sparkline_spans_its_box_and_puts_the_highest_value_at_the_top() {
        let rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(100.0, 26.0));
        let points = spark_points(&[1.0, 3.0, 2.0], rect);
        assert_eq!(points[0].x, 2.0);
        assert_eq!(points[2].x, 98.0);
        assert_eq!(points[1].y, 4.0);
        assert_eq!(points[0].y, 22.0);
    }

    #[test]
    fn a_flat_sparkline_runs_along_the_middle() {
        let rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(100.0, 26.0));
        let points = spark_points(&[5.0, 5.0, 5.0], rect);
        assert!(points.iter().all(|point| point.y == 13.0));
    }
}
