//! The Gallery screen: a grid of results.
//!
//! It is here for two reasons. It gives the scroll something to do that is longer than one screen, and it
//! is the one place the layout solver's `repeat(auto-fill, minmax(…))` is exercised at a phone's width —
//! `Grid::auto_fill` answers **two** columns at 393 points and four on a tablet, from the same call.

use egui::Rect;
use rux::components::{IconButton, IconButtonSize, Segmented, Tile, TileKind};
use rux::icon::Icon;
use rux::layout::{Cross, Flex, Grid, Pad, Pen};
use rux::Rux;

use super::{Scroll, KICKER_HEIGHT};
use crate::{PAGE_PAD, STACK_GAP};

/// One result.
#[derive(Debug, Clone)]
struct Shot {
    title: &'static str,
    size: &'static str,
    starred: bool,
}

#[derive(Debug, Clone)]
pub struct Gallery {
    filter: usize,
    shots: Vec<Shot>,
}

impl Default for Gallery {
    fn default() -> Self {
        Self::new()
    }
}

impl Gallery {
    pub fn new() -> Self {
        let titles = [
            ("Headland, sunrise", "1280 x 704"),
            ("Studio, hard light", "1024 x 1024"),
            ("Yoga, close up", "832 x 1216"),
            ("Two dancers", "1920 x 1088"),
            ("Kitchen, noon", "1024 x 1024"),
            ("Anamorphic wide", "1920 x 1088"),
            ("Forest, overcast", "1280 x 704"),
            ("Portrait, 85mm", "832 x 1216"),
        ];
        Self {
            filter: 0,
            shots: titles
                .iter()
                .enumerate()
                .map(|(index, (title, size))| Shot {
                    title,
                    size,
                    starred: index % 3 == 0,
                })
                .collect(),
        }
    }

    pub fn show(&mut self, rux: &mut Rux<'_>, area: Rect) {
        let body = Pad::axes(0.0, PAGE_PAD).inside(area);
        let columns = Grid::auto_fill(body.width(), MIN_COLUMN, CELL_GAP);
        let content = self.content_height(rux, body.width(), columns);
        Scroll::new(rux, area, body, content, "rux-mobile-gallery").show(rux, |rux, pen| {
            self.column(rux, pen, columns);
        });
    }

    fn column(&mut self, rux: &mut Rux<'_>, pen: &mut Pen, columns: usize) {
        let items = [("All", None), ("Starred", Some(Icon::Star)), ("Recent", Some(Icon::Spark))];
        let filter = Segmented::new(&items, self.filter);
        let strip = pen.strip(filter.measure(rux).y);
        if let Some(index) = filter.show(rux, strip).chosen {
            self.filter = index;
        }
        pen.gap(STACK_GAP);

        let shown = self.shown();
        let count = pen.strip(KICKER_HEIGHT);
        let words = format!("{} results", shown.len());
        super::kicker(rux, count, &words);
        pen.gap(8.0);

        // One grid, and the cells are walked rather than the rows, because `Grid::cell` counts across then
        // down and that is the reading order.
        let cell_width = (pen.rest().width() - CELL_GAP * (columns - 1) as f32) / columns as f32;
        let cell_height = Tile::height_at(cell_width) + CAPTION_HEIGHT;
        let rows = shown.len().div_ceil(columns);
        let grid_area = pen.strip(rows as f32 * cell_height + (rows.max(1) - 1) as f32 * CELL_GAP);
        let grid = Grid::new(grid_area, columns).gap(CELL_GAP, CELL_GAP).row_height(cell_height);
        for (index, shot) in shown.iter().enumerate() {
            self.cell(rux, grid.cell(index), shot);
        }
    }

    /// One result: the picture, and its name and size under it.
    fn cell(&self, rux: &mut Rux<'_>, rect: Rect, shot: &Shot) {
        let [picture, caption] = Flex::column(rect)
            .gap(6.0)
            .fill(1.0)
            .fixed(CAPTION_HEIGHT - 6.0)
            .solve();

        let (_, inside) = Tile::new().kind(TileKind::Well).label(shot.title).show(rux, picture);
        if shot.starred {
            // The star sits in the corner of the picture, at the size the reference draws a mark inside a
            // control. It is an `IconButton` rather than a bare mark because it is pressable, and on a
            // phone that means it claims the minimum target while still drawing a 28 point circle.
            let button = IconButton::new(Icon::Star, "Starred")
                .size(IconButtonSize::Small)
                .tint(rux.theme().accent.amber);
            let size = button.measure(rux);
            let at = Rect::from_min_size(
                egui::pos2(inside.right() - size.x - 4.0, inside.top() + 4.0),
                size,
            );
            button.show(rux, at);
        }

        // The caption is **text, not controls**. A `Chip` would say the same thing and would be measured
        // as a 44 point target on a touch screen, which is twice as tall as a caption line — the size of
        // a target is a property of things a finger presses, and nothing here is pressed.
        let theme = rux.theme();
        let size_style = rux::Style::mono(10.0).medium().tracking(0.02);
        let size_width = rux.measure(size_style, shot.size).x;
        let [name, dimensions] = Flex::row(caption)
            .gap(6.0)
            .cross(Cross::Centre)
            .fill(1.0)
            .hug(size_width.min(caption.width() * 0.5))
            .solve();

        let ink = theme.ink.i700;
        let galley = rux.elided(rux::Style::sans(12.0).medium(), shot.title, ink, name.width());
        rux::text::draw_left_centre(rux.ui.painter(), name.left_center(), galley, ink);

        let faint = theme.ink.i400;
        let galley = rux.elided(size_style, shot.size, faint, dimensions.width());
        rux::text::draw_left_centre(rux.ui.painter(), dimensions.left_center(), galley, faint);
    }

    fn shown(&self) -> Vec<Shot> {
        match self.filter {
            1 => self.shots.iter().filter(|shot| shot.starred).cloned().collect(),
            2 => self.shots.iter().take(4).cloned().collect(),
            _ => self.shots.clone(),
        }
    }

    fn content_height(&self, rux: &Rux<'_>, width: f32, columns: usize) -> f32 {
        let items = [("All", None), ("Starred", Some(Icon::Star)), ("Recent", Some(Icon::Spark))];
        let filter = Segmented::new(&items, self.filter).measure(rux).y;
        let cell_width = (width - CELL_GAP * (columns - 1) as f32) / columns as f32;
        let cell_height = Tile::height_at(cell_width) + CAPTION_HEIGHT;
        let rows = self.shown().len().div_ceil(columns.max(1));
        filter
            + STACK_GAP
            + KICKER_HEIGHT
            + 8.0
            + rows as f32 * cell_height
            + (rows.max(1) - 1) as f32 * CELL_GAP
    }
}

/// `repeat(auto-fill, minmax(150px, 1fr))` — two columns at 393 points, four at 840.
const MIN_COLUMN: f32 = 150.0;
const CELL_GAP: f32 = 10.0;
/// The name and the size under each picture.
const CAPTION_HEIGHT: f32 = 26.0;
