//! A small flex solver, because the reference is flexbox and grid.
//!
//! It is a measure-and-arrange pass that produces rectangles and nothing else, so it runs with no
//! window and is a unit test. Nothing about it is clever: it exists so a page reads like the markup it
//! was built from instead of like a list of magic numbers, and so a component can answer how big it
//! wants to be before anybody decides where to put it.

use egui::{Pos2, Rect, Vec2};

/// `padding`, in the order CSS writes it.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Pad {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Pad {
    pub const NONE: Self = Self { top: 0.0, right: 0.0, bottom: 0.0, left: 0.0 };

    /// `padding: <all>`.
    pub const fn all(all: f32) -> Self {
        Self { top: all, right: all, bottom: all, left: all }
    }

    /// `padding: <block> <inline>`.
    pub const fn axes(block: f32, inline: f32) -> Self {
        Self { top: block, right: inline, bottom: block, left: inline }
    }

    /// `padding: <top> <right> <bottom> <left>`.
    pub const fn sides(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self { top, right, bottom, left }
    }

    pub fn horizontal(self) -> f32 {
        self.left + self.right
    }

    pub fn vertical(self) -> f32 {
        self.top + self.bottom
    }

    /// The rectangle left inside `rect` once this padding is taken off it.
    pub fn inside(self, rect: Rect) -> Rect {
        Rect::from_min_max(
            Pos2::new(rect.min.x + self.left, rect.min.y + self.top),
            Pos2::new(rect.max.x - self.right, rect.max.y - self.bottom),
        )
    }
}

/// How much of the main axis an item takes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Length {
    /// A number of points, which is `flex: 0 0 <n>` — or `width: <n>; flex-shrink: 0`, which is what
    /// `.ic-count` says about itself.
    Fixed(f32),
    /// As much as the item itself said it needs: `flex: 0 0 auto`.
    Hug(f32),
    /// A share of whatever is left, which is `flex-grow`.
    Fill(f32),
}

/// Where an item sits across the axis, which is `align-items`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Cross {
    /// `align-items: stretch` — the item is as tall as the row.
    #[default]
    Stretch,
    /// `align-items: center`.
    Centre,
    /// `align-items: flex-start`.
    Start,
    /// `align-items: flex-end`, which is what `.board-head` uses to sit a title on a baseline.
    End,
}

/// Where the items as a whole sit along the axis, which is `justify-content`.
///
/// Only meaningful when nothing is `Fill`, because a `Fill` item has already eaten the slack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Justify {
    #[default]
    Start,
    Centre,
    End,
    /// `justify-content: space-between`.
    Between,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Axis {
    Row,
    Column,
}

/// One item in a flex line.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Item {
    length: Length,
    cross: Option<f32>,
    align: Option<Cross>,
}

/// A flex row or column: `display: flex` with a gap, a padding and an alignment.
///
/// ```no_run
/// # use rux::layout::{Flex, Cross, Pad};
/// # let area = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(240.0, 52.0));
/// # let chevron = 15.0;
/// let [icon, body, chev] = Flex::row(area)
///     .pad(Pad::sides(8.0, 14.0, 8.0, 8.0))
///     .gap(12.0)
///     .cross(Cross::Centre)
///     .fixed(36.0)
///     .fill(1.0)
///     .hug(chevron)
///     .solve();
/// ```
#[derive(Debug, Clone)]
pub struct Flex {
    area: Rect,
    axis: Axis,
    gap: f32,
    pad: Pad,
    cross: Cross,
    justify: Justify,
    items: Vec<Item>,
}

impl Flex {
    /// `flex-direction: row`.
    pub fn row(area: Rect) -> Self {
        Self::new(area, Axis::Row)
    }

    /// `flex-direction: column`.
    pub fn column(area: Rect) -> Self {
        Self::new(area, Axis::Column)
    }

    fn new(area: Rect, axis: Axis) -> Self {
        Self {
            area,
            axis,
            gap: 0.0,
            pad: Pad::NONE,
            cross: Cross::Stretch,
            justify: Justify::Start,
            items: Vec::new(),
        }
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    pub fn pad(mut self, pad: Pad) -> Self {
        self.pad = pad;
        self
    }

    /// `align-items`.
    pub fn cross(mut self, cross: Cross) -> Self {
        self.cross = cross;
        self
    }

    /// `justify-content`.
    pub fn justify(mut self, justify: Justify) -> Self {
        self.justify = justify;
        self
    }

    fn push(mut self, length: Length) -> Self {
        self.items.push(Item { length, cross: None, align: None });
        self
    }

    /// An item of exactly this many points.
    pub fn fixed(self, points: f32) -> Self {
        self.push(Length::Fixed(points))
    }

    /// An item as big as it said it needs to be.
    pub fn hug(self, points: f32) -> Self {
        self.push(Length::Hug(points))
    }

    /// An item taking a share of what is left.
    pub fn fill(self, weight: f32) -> Self {
        self.push(Length::Fill(weight))
    }

    /// An item of a whole size: its main length *and* its cross size, which overrides the line's
    /// `align-items` for that one item. What a round button in a row of stretched things is.
    pub fn sized(mut self, size: Vec2) -> Self {
        let (main, cross) = match self.axis {
            Axis::Row => (size.x, size.y),
            Axis::Column => (size.y, size.x),
        };
        self.items.push(Item {
            length: Length::Hug(main),
            cross: Some(cross),
            align: Some(Cross::Centre),
        });
        self
    }

    /// The last item pushed, aligned across the axis differently from the rest of the line.
    pub fn align(mut self, align: Cross) -> Self {
        if let Some(item) = self.items.last_mut() {
            item.align = Some(align);
        }
        self
    }

    /// The rectangles, one per item, in the order they were pushed.
    ///
    /// **`Fill` shares what is left after the fixed items, the gaps and the padding**, in proportion
    /// to the weights. With nothing left there is nothing to share, and a `Fill` item comes out zero
    /// wide rather than negative — which is what makes a collapsed panel legal rather than a source of
    /// inverted rectangles.
    pub fn solve<const N: usize>(&self) -> [Rect; N] {
        let solved = self.solve_all();
        let mut out = [Rect::NOTHING; N];
        for (slot, rect) in out.iter_mut().zip(solved) {
            *slot = rect;
        }
        out
    }

    /// The same, for a line whose length is only known at run time.
    pub fn solve_all(&self) -> Vec<Rect> {
        let inner = self.pad.inside(self.area);
        let (available, cross_span) = match self.axis {
            Axis::Row => (inner.width(), inner.height()),
            Axis::Column => (inner.height(), inner.width()),
        };
        let gaps = self.gap * self.items.len().saturating_sub(1) as f32;
        let taken: f32 = self
            .items
            .iter()
            .map(|item| match item.length {
                Length::Fixed(points) | Length::Hug(points) => points,
                Length::Fill(_) => 0.0,
            })
            .sum();
        let weights: f32 = self
            .items
            .iter()
            .filter_map(|item| match item.length {
                Length::Fill(weight) => Some(weight),
                _ => None,
            })
            .sum();
        let slack = (available - taken - gaps).max(0.0);

        let mut lengths: Vec<f32> = self
            .items
            .iter()
            .map(|item| match item.length {
                Length::Fixed(points) | Length::Hug(points) => points,
                Length::Fill(weight) if weights > 0.0 => slack * weight / weights,
                Length::Fill(_) => 0.0,
            })
            .collect();

        // With no `Fill` item there is slack to distribute, which is what `justify-content` decides.
        let used: f32 = lengths.iter().sum::<f32>() + gaps;
        let (mut cursor, spread) = match (weights > 0.0, self.justify) {
            (true, _) | (false, Justify::Start) => (0.0, 0.0),
            (false, Justify::Centre) => ((available - used) / 2.0, 0.0),
            (false, Justify::End) => (available - used, 0.0),
            (false, Justify::Between) if self.items.len() > 1 => {
                (0.0, (available - used).max(0.0) / (self.items.len() - 1) as f32)
            }
            (false, Justify::Between) => (0.0, 0.0),
        };

        // A line whose items do not fit shrinks them in proportion, which is `flex-shrink: 1` — except
        // that `Fixed` says `flex-shrink: 0` and keeps its length. `.ic-count` depends on that.
        let overflow = used - available;
        if overflow > 0.0 {
            let shrinkable: f32 = self
                .items
                .iter()
                .zip(&lengths)
                .filter(|(item, _)| !matches!(item.length, Length::Fixed(_)))
                .map(|(_, length)| *length)
                .sum();
            if shrinkable > 0.0 {
                let scale = ((shrinkable - overflow) / shrinkable).max(0.0);
                for (item, length) in self.items.iter().zip(lengths.iter_mut()) {
                    if !matches!(item.length, Length::Fixed(_)) {
                        *length *= scale;
                    }
                }
            }
        }

        let mut out = Vec::with_capacity(self.items.len());
        for (item, length) in self.items.iter().zip(lengths) {
            let align = item.align.unwrap_or(self.cross);
            let size = item.cross.unwrap_or(cross_span).min(cross_span);
            let offset = match align {
                Cross::Stretch if item.cross.is_none() => 0.0,
                Cross::Stretch | Cross::Centre => (cross_span - size) / 2.0,
                Cross::Start => 0.0,
                Cross::End => cross_span - size,
            };
            let across = match align {
                Cross::Stretch if item.cross.is_none() => cross_span,
                _ => size,
            };
            let rect = match self.axis {
                Axis::Row => Rect::from_min_size(
                    Pos2::new(inner.min.x + cursor, inner.min.y + offset),
                    Vec2::new(length, across),
                ),
                Axis::Column => Rect::from_min_size(
                    Pos2::new(inner.min.x + offset, inner.min.y + cursor),
                    Vec2::new(across, length),
                ),
            };
            out.push(rect);
            cursor += length + self.gap + spread;
        }
        out
    }
}

/// `display: grid; grid-template-columns: repeat(<columns>, 1fr)` with a row and a column gap.
///
/// Two of them in the reference: the controls panel's `.ic-ctrl-grid` and the results canvas's
/// `.ic-results`, which is `repeat(auto-fill, minmax(260px, 1fr))` — so [`Grid::auto_fill`] answers
/// how many columns fit before the cells are laid out.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Grid {
    area: Rect,
    columns: usize,
    column_gap: f32,
    row_gap: f32,
    row_height: Option<f32>,
}

impl Grid {
    pub fn new(area: Rect, columns: usize) -> Self {
        Self { area, columns: columns.max(1), column_gap: 0.0, row_gap: 0.0, row_height: None }
    }

    /// `gap: <row> <column>`.
    pub fn gap(mut self, row: f32, column: f32) -> Self {
        self.row_gap = row;
        self.column_gap = column;
        self
    }

    /// Every row this tall. Without it a row is as tall as the area divided by the row count, which is
    /// what a grid filling a fixed box does.
    pub fn row_height(mut self, height: f32) -> Self {
        self.row_height = Some(height);
        self
    }

    /// How many columns of at least `minimum` points fit, which is `repeat(auto-fill, minmax(…))`.
    pub fn auto_fill(width: f32, minimum: f32, gap: f32) -> usize {
        if minimum <= 0.0 {
            return 1;
        }
        let fits = ((width + gap) / (minimum + gap)).floor() as i64;
        fits.max(1) as usize
    }

    pub fn column_width(&self) -> f32 {
        let gaps = self.column_gap * (self.columns - 1) as f32;
        ((self.area.width() - gaps) / self.columns as f32).max(0.0)
    }

    /// The rectangle of the cell at this index, counting across then down.
    pub fn cell(&self, index: usize) -> Rect {
        let column = index % self.columns;
        let row = index / self.columns;
        let width = self.column_width();
        let height = self.row_height.unwrap_or_else(|| self.area.height().max(0.0));
        Rect::from_min_size(
            Pos2::new(
                self.area.min.x + column as f32 * (width + self.column_gap),
                self.area.min.y + row as f32 * (height + self.row_gap),
            ),
            Vec2::new(width, height),
        )
    }

    /// How tall a grid of `count` cells comes out, which a scrolling parent needs before it draws.
    pub fn height_for(&self, count: usize) -> f32 {
        if count == 0 {
            return 0.0;
        }
        let rows = count.div_ceil(self.columns);
        let height = self.row_height.unwrap_or_else(|| self.area.height().max(0.0));
        rows as f32 * height + (rows - 1) as f32 * self.row_gap
    }
}

/// A cursor running down a column, which is what a settings page, a storybook page and a panel of
/// stacked sections all are.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pen {
    area: Rect,
    y: f32,
}

impl Pen {
    pub fn new(area: Rect) -> Self {
        Self { area, y: area.min.y }
    }

    /// A strip this tall across the whole width, and the pen moves past it.
    pub fn strip(&mut self, height: f32) -> Rect {
        let rect = Rect::from_min_size(
            Pos2::new(self.area.min.x, self.y),
            Vec2::new(self.area.width(), height),
        );
        self.y += height;
        rect
    }

    /// Nothing drawn, just space.
    pub fn gap(&mut self, height: f32) {
        self.y += height;
    }

    /// Where the pen is now.
    pub fn at(&self) -> f32 {
        self.y
    }

    /// How far it has come, which is what a page reports so a scrollbar knows how long it is.
    pub fn used(&self) -> f32 {
        self.y - self.area.min.y
    }

    /// What is left below the pen.
    pub fn rest(&self) -> Rect {
        Rect::from_min_max(Pos2::new(self.area.min.x, self.y), self.area.max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn area(width: f32, height: f32) -> Rect {
        Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(width, height))
    }

    #[test]
    fn a_fill_item_takes_what_is_left_after_the_fixed_ones_and_the_gaps() {
        let [icon, body, chev] = Flex::row(area(240.0, 52.0))
            .pad(Pad::sides(8.0, 14.0, 8.0, 8.0))
            .gap(12.0)
            .fixed(36.0)
            .fill(1.0)
            .hug(15.0)
            .solve();
        // 240 - 8 - 14 = 218 of room, less 36 and 15 and two 12-point gaps = 143.
        assert_eq!(icon.width(), 36.0);
        assert_eq!(body.width(), 143.0);
        assert_eq!(chev.width(), 15.0);
        assert_eq!(icon.min.x, 8.0);
        assert_eq!(body.min.x, 8.0 + 36.0 + 12.0);
        assert_eq!(chev.max.x, 240.0 - 14.0);
    }

    #[test]
    fn two_fill_items_share_in_proportion_to_their_weights() {
        // 300 less one 20-point gap is 280 to share two parts to one.
        let [left, right] = Flex::row(area(300.0, 20.0)).gap(20.0).fill(2.0).fill(1.0).solve();
        assert!((left.width() - 560.0 / 3.0).abs() < 0.01);
        assert!((right.width() - 280.0 / 3.0).abs() < 0.01);
        assert!((left.width() + right.width() + 20.0 - 300.0).abs() < 0.01);
    }

    #[test]
    fn a_fixed_item_does_not_shrink_when_the_line_overflows() {
        // `.ic-count { width: 66px; flex-shrink: 0 }` beside a select that must give way.
        let [select, count] = Flex::row(area(80.0, 20.0)).gap(8.0).hug(200.0).fixed(66.0).solve();
        assert_eq!(count.width(), 66.0);
        assert!(select.width() < 200.0);
        assert!(select.width() >= 0.0);
    }

    #[test]
    fn a_collapsed_panel_is_zero_wide_rather_than_negative() {
        let [only] = Flex::row(area(0.0, 40.0)).pad(Pad::all(16.0)).fill(1.0).solve();
        assert_eq!(only.width(), 0.0);
    }

    #[test]
    fn centred_and_end_alignment_place_a_shorter_item_across_the_line() {
        let [centred] = Flex::row(area(100.0, 40.0)).cross(Cross::Centre).sized(Vec2::splat(10.0)).solve();
        assert_eq!(centred.min.y, 15.0);
        let [ended] =
            Flex::row(area(100.0, 40.0)).sized(Vec2::splat(10.0)).align(Cross::End).solve();
        assert_eq!(ended.min.y, 30.0);
    }

    #[test]
    fn space_between_spreads_the_slack_into_the_gaps() {
        let [a, b] = Flex::row(area(100.0, 20.0))
            .justify(Justify::Between)
            .hug(20.0)
            .hug(20.0)
            .solve();
        assert_eq!(a.min.x, 0.0);
        assert_eq!(b.min.x, 80.0);
    }

    #[test]
    fn a_column_runs_down_the_area() {
        let [head, body] =
            Flex::column(area(200.0, 100.0)).gap(10.0).fixed(30.0).fill(1.0).solve();
        assert_eq!(head.height(), 30.0);
        assert_eq!(body.height(), 60.0);
        assert_eq!(body.min.y, 40.0);
        assert_eq!(head.width(), 200.0);
    }

    #[test]
    fn an_auto_fill_grid_counts_the_columns_that_fit() {
        // `.ic-results { grid-template-columns: repeat(auto-fill, minmax(260px, 1fr)); gap: 20px }`.
        // Three columns need 3×260 + 2×20 = 820, so 880 fits three and 819 fits two.
        assert_eq!(Grid::auto_fill(880.0, 260.0, 20.0), 3);
        assert_eq!(Grid::auto_fill(819.0, 260.0, 20.0), 2);
        assert_eq!(Grid::auto_fill(540.0, 260.0, 20.0), 2);
        assert_eq!(Grid::auto_fill(539.0, 260.0, 20.0), 1);
        // Never zero, however narrow the pane gets.
        assert_eq!(Grid::auto_fill(10.0, 260.0, 20.0), 1);
    }

    #[test]
    fn a_grid_lays_its_cells_across_then_down() {
        let grid = Grid::new(area(220.0, 200.0), 2).gap(12.0, 10.0).row_height(50.0);
        assert_eq!(grid.column_width(), 105.0);
        assert_eq!(grid.cell(0).min, Pos2::new(0.0, 0.0));
        assert_eq!(grid.cell(1).min, Pos2::new(115.0, 0.0));
        assert_eq!(grid.cell(2).min, Pos2::new(0.0, 62.0));
        assert_eq!(grid.height_for(4), 112.0);
        assert_eq!(grid.height_for(0), 0.0);
    }

    #[test]
    fn a_pen_reports_how_far_it_has_come() {
        let mut pen = Pen::new(area(300.0, 400.0));
        let first = pen.strip(40.0);
        pen.gap(14.0);
        let second = pen.strip(28.0);
        assert_eq!(first.height(), 40.0);
        assert_eq!(second.min.y, 54.0);
        assert_eq!(pen.used(), 82.0);
        assert_eq!(pen.rest().min.y, 82.0);
    }
}
