//! A table on a screen, and a timeline down a groove.
//!
//! The instrument language's two ways of listing things (see `instrument.rs`). A **table** is rows of
//! records on a recessed screen: silkscreen headers, hairline rows, numbers in tabular figures and right
//! aligned, a cell cut short rather than wrapped. Pressing a header reports it; the order is the
//! caller's. A **timeline** is a groove with an LED for each item: lit for done, breathing for the one
//! in progress, dark for what is to come.

use egui::{Color32, Id, Pos2, Rect, Sense, Vec2};

use super::instrument::{appearing, ease_out, led, screen, timing, Instrument, SILK};
use super::pointing;
use crate::render::Fill;
use crate::text::{self, Style};
use crate::Rux;

/// How a column's cells line up.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Align {
    #[default]
    Left,
    Right,
    Centre,
}

/// One column: its header and how its cells line up.
#[derive(Debug, Clone)]
pub struct Column<'a> {
    pub label: &'a str,
    pub align: Align,
}

/// What a table reports: a header that was pressed.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct TableOutcome {
    pub sort: Option<usize>,
}

/// A cell's words.
const CELL: Style = Style::sans(12.0);
/// A cell holding a number.
const FIGURE: Style = Style::mono(11.5);

/// Whether a cell reads as a number, which is what decides its face.
pub fn is_figure(cell: &str) -> bool {
    let trimmed = cell
        .trim()
        .trim_start_matches(['$', '£', '€', '+', '-', '\u{2212}']);
    trimmed.chars().next().is_some_and(|c| c.is_ascii_digit())
        && trimmed.chars().filter(|c| c.is_ascii_digit()).count() * 2
            >= trimmed.chars().filter(|c| !c.is_whitespace()).count()
}

/// A table.
pub struct Table<'a> {
    columns: &'a [Column<'a>],
    rows: &'a [Vec<String>],
    /// Which column the rows are ordered by, and whether it is descending.
    sorted: Option<(usize, bool)>,
    id: Id,
}

impl<'a> Table<'a> {
    pub fn new(id: Id, columns: &'a [Column<'a>], rows: &'a [Vec<String>]) -> Self {
        Self {
            columns,
            rows,
            sorted: None,
            id,
        }
    }

    pub fn sorted(mut self, sorted: Option<(usize, bool)>) -> Self {
        self.sorted = sorted;
        self
    }

    /// The least a row is: one line of words and the padding round it.
    fn row(rux: &Rux<'_>) -> f32 {
        rux.z(27.0)
    }

    fn head(rux: &Rux<'_>) -> f32 {
        rux.z(28.0)
    }

    /// How tall the whole table is at `width`, every row as tall as its tallest wrapped cell.
    pub fn height_at(&self, rux: &Rux<'_>, width: f32) -> f32 {
        let widths = self.widths(rux, width);
        let rows: f32 = self
            .rows
            .iter()
            .map(|cells| self.row_height(rux, cells, &widths))
            .sum();
        Self::head(rux) + rows + rux.z(4.0)
    }

    /// A cell laid out in its column: a figure on one line, words wrapped to at most [`MOST_LINES`].
    fn cell_galley(
        rux: &Rux<'_>,
        cell: &str,
        width: f32,
        colour: egui::Color32,
    ) -> std::sync::Arc<egui::Galley> {
        let figure = is_figure(cell);
        let style = rux.zs(if figure { FIGURE } else { CELL });
        match figure {
            true => text::elided(rux.painter(), style, cell, colour, width),
            false => {
                let mut job = egui::text::LayoutJob::single_section(
                    cell.to_owned(),
                    egui::TextFormat {
                        font_id: style.font(),
                        color: colour,
                        ..Default::default()
                    },
                );
                job.wrap = egui::text::TextWrapping {
                    max_width: width.max(1.0),
                    max_rows: MOST_LINES,
                    break_anywhere: false,
                    overflow_character: Some('\u{2026}'),
                };
                rux.painter().layout_job(job)
            }
        }
    }

    /// How tall one row is, given the column widths.
    fn row_height(&self, rux: &Rux<'_>, cells: &[String], widths: &[f32]) -> f32 {
        let pad = rux.z(12.0);
        let tallest = cells
            .iter()
            .zip(widths)
            .map(|(cell, width)| {
                Self::cell_galley(rux, cell, width - pad, egui::Color32::WHITE)
                    .size()
                    .y
            })
            .fold(0.0_f32, f32::max);
        (tallest + rux.z(12.0)).max(Self::row(rux))
    }

    /// How wide each column is.
    ///
    /// **A short column keeps its natural width and a long one gives way.** Shared out in proportion, a
    /// column of prose beside three short ones took nearly all of the row and left the names cut to four
    /// letters. So every column first gets what it needs or a fair share, whichever is less; what is
    /// left goes to the columns that wanted more, in proportion to what they wanted; and a prose column
    /// that is still short of room wraps rather than being cut.
    fn widths(&self, rux: &Rux<'_>, width: f32) -> Vec<f32> {
        let pad = rux.z(12.0);
        let wants: Vec<f32> = self
            .columns
            .iter()
            .enumerate()
            .map(|(at, column)| {
                let head = rux.measure(rux.zs(SILK), column.label).x;
                let cells = self
                    .rows
                    .iter()
                    .filter_map(|row| row.get(at))
                    .map(|cell| {
                        rux.measure(rux.zs(if is_figure(cell) { FIGURE } else { CELL }), cell)
                            .x
                    })
                    .fold(0.0_f32, f32::max);
                head.max(cells) + pad
            })
            .collect();
        let room = (width - pad).max(1.0);
        let total: f32 = wants.iter().sum();
        if wants.is_empty() {
            return wants;
        }
        if total <= room {
            // Room to spare goes to the first column, which is usually the name of the thing.
            let mut out = wants;
            out[0] += room - total;
            return out;
        }
        // Water filling: settle the columns that fit under an even share, then share what is left among
        // the rest, until every remaining column wants more than its share.
        let mut out = vec![0.0_f32; wants.len()];
        let mut open: Vec<usize> = (0..wants.len()).collect();
        let mut left = room;
        loop {
            let share = left / open.len().max(1) as f32;
            let (fits, rest): (Vec<usize>, Vec<usize>) =
                open.iter().partition(|at| wants[**at] <= share);
            if fits.is_empty() {
                let wanted: f32 = rest.iter().map(|at| wants[*at]).sum();
                for at in &rest {
                    out[*at] = left * wants[*at] / wanted.max(1.0);
                }
                break;
            }
            for at in &fits {
                out[*at] = wants[*at];
                left -= wants[*at];
            }
            open = rest;
            if open.is_empty() {
                break;
            }
        }
        let floor = rux.z(40.0);
        out.iter().map(|width| width.max(floor)).collect()
    }

    pub fn show(self, rux: &mut Rux<'_>, rect: Rect) -> TableOutcome {
        let theme = rux.theme();
        let colours = Instrument::of(theme);
        screen(rux, rect, rux.z(9.0), false);
        let painter = rux.painter().clone();
        let widths = self.widths(rux, rect.width());
        let pad = rux.z(12.0);
        let mut outcome = TableOutcome::default();
        let head = Self::head(rux);
        let cell_rect = |column: usize, top: f32, height: f32| {
            let left = rect.left() + pad / 2.0 + widths[..column].iter().sum::<f32>();
            Rect::from_min_size(Pos2::new(left, top), Vec2::new(widths[column], height))
        };
        // The headers, each a press target.
        for (at, column) in self.columns.iter().enumerate() {
            let place = cell_rect(at, rect.top(), head);
            let response = rux
                .ui
                .interact(place, self.id.with(("head", at)), Sense::click());
            pointing(rux, &response, true);
            if response.clicked() {
                outcome.sort = Some(at);
            }
            let sorted = self.sorted.filter(|(by, _)| *by == at);
            let ink = match (sorted.is_some(), response.hovered()) {
                (true, _) | (_, true) => theme.ink.i700,
                _ => theme.ink.i400,
            };
            let mut label = column.label.to_owned();
            if let Some((_, descending)) = sorted {
                label.push_str(if descending { " \u{25be}" } else { " \u{25b4}" });
            }
            let galley = text::elided(&painter, rux.zs(SILK), &label, ink, place.width() - pad);
            let x = aligned(column.align, place, galley.size().x, pad);
            painter.galley(
                Pos2::new(x, place.center().y - galley.size().y / 2.0),
                galley,
                ink,
            );
            if sorted.is_some() {
                led(
                    &painter,
                    Pos2::new(place.left() + pad / 2.0 - rux.z(3.0), place.center().y),
                    rux.z(2.0),
                    theme.accent.blue,
                    colours.led_off,
                    1.0,
                );
            }
            response.widget_info(|| {
                egui::WidgetInfo::labeled(
                    egui::WidgetType::Button,
                    true,
                    format!("Sort by {}", column.label),
                )
            });
        }
        // The rows, each as tall as its tallest cell.
        let mut top = rect.top() + head;
        for cells in self.rows.iter() {
            let row = self.row_height(rux, cells, &widths);
            let band = Rect::from_min_size(
                Pos2::new(rect.left() + rux.z(3.0), top),
                Vec2::new(rect.width() - rux.z(6.0), row),
            );
            painter.line_segment(
                [
                    Pos2::new(band.left() + pad / 2.0, top.round() + 0.5),
                    Pos2::new(band.right() - pad / 2.0, top.round() + 0.5),
                ],
                egui::Stroke::new(1.0, colours.rule),
            );
            if rux.ui.rect_contains_pointer(band) {
                painter.rect_filled(band, rux.z(4.0), theme.interaction.hover);
            }
            for (at, column) in self.columns.iter().enumerate() {
                let Some(cell) = cells.get(at) else { continue };
                let place = cell_rect(at, top, row);
                let figure = is_figure(cell);
                let ink = match (at, figure) {
                    (0, _) => theme.ink.i900,
                    (_, true) => theme.ink.i700,
                    _ => theme.ink.i500,
                };
                let galley = Self::cell_galley(rux, cell, place.width() - pad, ink);
                let align = match (column.align, figure) {
                    (Align::Left, true) if at > 0 => Align::Right,
                    (align, _) => align,
                };
                let x = aligned(align, place, galley.size().x, pad);
                // One line sits on the row's middle by its capitals; a wrapped cell starts at the top.
                if galley.rows.len() <= 1 {
                    let style = rux.zs(if figure { FIGURE } else { CELL });
                    text::draw_left_capitals(
                        &painter,
                        Pos2::new(x, place.center().y),
                        galley,
                        style,
                        ink,
                    );
                } else {
                    painter.galley(Pos2::new(x, top + rux.z(6.0)), galley, ink);
                }
            }
            top += row;
        }
        outcome
    }
}

/// The most lines a cell of words wraps to before it is cut with an ellipsis.
const MOST_LINES: usize = 3;

/// Where a cell's words start for an alignment.
fn aligned(align: Align, place: Rect, width: f32, pad: f32) -> f32 {
    match align {
        Align::Left => place.left() + pad / 2.0,
        Align::Right => place.right() - pad / 2.0 - width,
        Align::Centre => place.center().x - width / 2.0,
    }
}

/// Where an item on a timeline has got to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Stage {
    Done,
    Active,
    #[default]
    Todo,
}

/// One item on a timeline.
#[derive(Debug, Clone)]
pub struct Item<'a> {
    pub time: &'a str,
    pub title: &'a str,
    pub text: &'a str,
    pub stage: Stage,
}

/// A title on a timeline.
const ITEM_TITLE: Style = Style::sans(12.5).medium().leading(1.4);
/// The words under a title.
const ITEM_TEXT: Style = Style::sans(12.0).leading(1.45);
/// A time beside an item.
const ITEM_TIME: Style = Style::mono(10.5);

/// A timeline: a groove with an LED for each item.
pub struct Timeline<'a> {
    items: &'a [Item<'a>],
    id: Id,
}

impl<'a> Timeline<'a> {
    pub fn new(id: Id, items: &'a [Item<'a>]) -> Self {
        Self { items, id }
    }

    /// How far in the words start: past the groove, and past the time column when there is one.
    fn indent(&self, rux: &Rux<'_>) -> (f32, f32) {
        let time = self
            .items
            .iter()
            .filter(|item| !item.time.is_empty())
            .map(|item| rux.measure(rux.zs(ITEM_TIME), item.time).x)
            .fold(0.0_f32, f32::max);
        let time_column = if time > 0.0 { time + rux.z(12.0) } else { 0.0 };
        (time_column, time_column + rux.z(26.0))
    }

    fn item_height(&self, rux: &Rux<'_>, item: &Item<'_>, width: f32) -> f32 {
        let title = text::wrapped(
            rux.painter(),
            rux.zs(ITEM_TITLE),
            item.title,
            Color32::WHITE,
            width,
        )
        .size()
        .y;
        let words = match item.text.is_empty() {
            true => 0.0,
            false => {
                text::wrapped(
                    rux.painter(),
                    rux.zs(ITEM_TEXT),
                    item.text,
                    Color32::WHITE,
                    width,
                )
                .size()
                .y
            }
        };
        title + words
    }

    pub fn height_at(&self, rux: &Rux<'_>, width: f32) -> f32 {
        let (_, indent) = self.indent(rux);
        let words = (width - indent).max(rux.z(60.0));
        let gap = rux.z(12.0);
        self.items
            .iter()
            .map(|item| self.item_height(rux, item, words))
            .sum::<f32>()
            + gap * self.items.len().saturating_sub(1) as f32
    }

    pub fn show(self, rux: &mut Rux<'_>, rect: Rect) {
        let theme = rux.theme();
        let colours = Instrument::of(theme);
        let (time_column, indent) = self.indent(rux);
        let words = (rect.width() - indent).max(rux.z(60.0));
        let rail_x = rect.left() + time_column + rux.z(8.0);
        let first_line = rux.measure(rux.zs(ITEM_TITLE), "Ag").y;
        if self.items.len() > 1 {
            let last_top = rect.bottom()
                - self.item_height(rux, self.items.last().expect("more than one"), words);
            let groove = Rect::from_min_max(
                Pos2::new(rail_x - rux.z(1.5), rect.top() + first_line / 2.0),
                Pos2::new(rail_x + rux.z(1.5), last_top + first_line / 2.0),
            );
            rux.chrome.surface(
                groove,
                rux.z(1.5),
                Fill::Solid(colours.screen),
                theme.elevation.pressed_sm,
            );
        }
        let painter = rux.painter().clone();
        let mut pen = rect.top();
        let gap = rux.z(12.0);
        for (index, item) in self.items.iter().enumerate() {
            let height = self.item_height(rux, item, words);
            let centre = Pos2::new(rail_x, pen + first_line / 2.0);
            // Each light comes on in turn the first time the timeline is drawn.
            let on = ease_out(appearing(
                rux,
                self.id.with(("lamp", index)),
                index as f32 * timing::STAGGER * 3.0,
                timing::LIGHT * 2.0,
            ));
            let (colour, lit) = match item.stage {
                Stage::Done => (theme.accent.mint, on),
                Stage::Active => {
                    // An item in progress breathes, and stops breathing after a while so a finished
                    // answer does not keep the window drawing.
                    let age = super::instrument::age(rux, self.id.with(("breath", index))) as f32;
                    let breath = match age < timing::BREATHING_FOR && !rux.state.is_still() {
                        true => {
                            rux.ctx()
                                .request_repaint_after(std::time::Duration::from_millis(40));
                            0.75 + 0.25 * (age / timing::BREATH * std::f32::consts::TAU).cos()
                        }
                        false => 1.0,
                    };
                    (theme.accent.blue, on * breath)
                }
                Stage::Todo => (theme.ink.i300, 0.0),
            };
            if item.stage == Stage::Todo {
                // A dark lamp still has a rim, so the rail reads as a row of lamps rather than dots.
                painter.circle_stroke(centre, rux.z(4.5), egui::Stroke::new(1.0, theme.ink.i200));
            }
            led(&painter, centre, rux.z(4.0), colour, colours.led_off, lit);
            if !item.time.is_empty() {
                let galley = text::layout(&painter, rux.zs(ITEM_TIME), item.time, theme.ink.i400);
                text::draw_left_capitals(
                    &painter,
                    Pos2::new(rect.left(), centre.y),
                    galley,
                    rux.zs(ITEM_TIME),
                    theme.ink.i400,
                );
            }
            let left = rect.left() + indent;
            let title_ink = match item.stage {
                Stage::Todo => theme.ink.i500,
                _ => theme.ink.i900,
            };
            let title = text::wrapped(&painter, rux.zs(ITEM_TITLE), item.title, title_ink, words);
            let title_height = title.size().y;
            painter.galley(Pos2::new(left, pen), title, title_ink);
            if !item.text.is_empty() {
                let body = text::wrapped(
                    &painter,
                    rux.zs(ITEM_TEXT),
                    item.text,
                    theme.ink.i500,
                    words,
                );
                painter.galley(Pos2::new(left, pen + title_height), body, theme.ink.i500);
            }
            pen += height + gap;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_cell_is_a_figure_when_it_is_mostly_digits() {
        assert!(is_figure("142"));
        assert!(is_figure("61 s"));
        assert!(is_figure("$1,200.50"));
        assert!(is_figure("-7%"));
        assert!(!is_figure("canvas_realm"));
        assert!(!is_figure("3rd party plugins"));
        assert!(!is_figure(""));
    }
}
