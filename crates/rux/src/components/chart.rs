//! A chart on a screen: bars made of lit segments, a glowing trace, an area under one, or a segmented
//! ring.
//!
//! The instrument language's data display (see `instrument.rs`). The plot is a recessed screen with a
//! graticule; gridlines fall on round numbers; the numbers on the axes are in the mono face; bars are
//! columns of segments the way a level meter is, rather than flat rectangles; a line is a trace with a
//! soft glow along it. The value under the pointer is read out in the screen's top right corner, which
//! is where an instrument shows a reading, rather than in a floating tooltip.
//!
//! **Everything that moves is drawn with `egui`.** The screen itself goes into the chrome once; the
//! bars rise, the trace draws itself and the pointer's reading changes on the painter, so none of that
//! rasterises the layer again.

use egui::{Color32, Id, Mesh, Pos2, Rect, Sense, Shape, Stroke, Vec2};

use super::instrument::{appearing, ease_back, ease_out, led, screen, timing, Instrument, SILK};
use crate::text::{self, Style};
use crate::Rux;

/// Which way up a chart is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ChartKind {
    #[default]
    Bar,
    Line,
    Area,
    Donut,
}

/// One series: its name, its values and its colour.
#[derive(Debug, Clone, Copy)]
pub struct Series<'a> {
    pub name: &'a str,
    pub values: &'a [f64],
    pub colour: Color32,
}

/// What the chart reports: which point is under the pointer, as (series, index).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ChartOutcome {
    pub hovered: Option<(usize, usize)>,
}

/// The numbers on the axes and in the reading.
const AXIS: Style = Style::mono(9.5);
/// A legend entry.
const LEGEND: Style = Style::sans(11.0);
/// The reading in the corner.
const READOUT: Style = Style::mono(10.5).medium();

/// A chart.
pub struct Chart<'a> {
    kind: ChartKind,
    labels: &'a [String],
    series: Vec<Series<'a>>,
    stacked: bool,
    format: &'a dyn Fn(f64) -> String,
    id: Id,
}

/// Round numbers to put gridlines at between `low` and `high`: about `count` of them, each a 1, 2, 2.5
/// or 5 times a power of ten apart, always covering the data.
pub fn nice_ticks(low: f64, high: f64, count: usize) -> Vec<f64> {
    let (low, high) = match (low.is_finite(), high.is_finite()) {
        (true, true) if high > low => (low, high),
        (true, true) => (low.min(0.0), low.max(0.0) + 1.0),
        _ => (0.0, 1.0),
    };
    let rough = (high - low) / count.max(1) as f64;
    let power = 10f64.powf(rough.log10().floor());
    let step = [1.0, 2.0, 2.5, 5.0, 10.0]
        .into_iter()
        .map(|factor| factor * power)
        .find(|step| *step >= rough)
        .unwrap_or(power * 10.0);
    let first = (low / step).floor() * step;
    let mut out = Vec::new();
    let mut at = first;
    while at < high + step * 0.999 && out.len() < 50 {
        out.push((at / step).round() * step);
        at += step;
    }
    out
}

impl<'a> Chart<'a> {
    pub fn new(
        id: Id,
        kind: ChartKind,
        labels: &'a [String],
        format: &'a dyn Fn(f64) -> String,
    ) -> Self {
        Self {
            kind,
            labels,
            series: Vec::new(),
            stacked: false,
            format,
            id,
        }
    }

    pub fn series(mut self, series: Series<'a>) -> Self {
        self.series.push(series);
        self
    }

    pub fn stacked(mut self, stacked: bool) -> Self {
        self.stacked = stacked;
        self
    }

    /// What the chart shows, in words, for an accessibility tree and a test that finds it by name.
    fn describe(&self) -> String {
        let names: Vec<&str> = self.series.iter().map(|one| one.name).collect();
        format!(
            "Chart: {} over {}",
            names.join(", "),
            self.labels.join(", ")
        )
    }

    fn has_legend(&self) -> bool {
        self.series.len() > 1 || self.kind == ChartKind::Donut
    }

    /// How tall the chart is at `width`: the screen, and the legend under it when there is one.
    pub fn height_at(&self, rux: &Rux<'_>, width: f32) -> f32 {
        let plot = match self.kind {
            ChartKind::Donut => rux.z(140.0).min(width * 0.6).max(rux.z(110.0)),
            _ => rux.z(160.0),
        };
        match self.has_legend() && self.kind != ChartKind::Donut {
            true => plot + rux.z(10.0) + self.legend_height(rux, width),
            false => plot,
        }
    }

    /// The legend, wrapped across `width`.
    fn legend_height(&self, rux: &Rux<'_>, width: f32) -> f32 {
        let row = rux.z(16.0);
        let mut rows = 1;
        let mut pen = 0.0;
        for one in &self.series {
            let need = rux.z(14.0) + rux.measure(rux.zs(LEGEND), one.name).x + rux.z(16.0);
            if pen > 0.0 && pen + need > width {
                rows += 1;
                pen = 0.0;
            }
            pen += need;
        }
        rows as f32 * row
    }

    pub fn show(self, rux: &mut Rux<'_>, rect: Rect) -> ChartOutcome {
        match self.kind {
            ChartKind::Donut => self.donut(rux, rect),
            _ => self.plot(rux, rect),
        }
    }

    /// The values the y axis has to cover: every value, or every stack's total.
    fn range(&self) -> (f64, f64) {
        let mut low: f64 = 0.0;
        let mut high: f64 = 0.0;
        let points = self.labels.len().max(
            self.series
                .iter()
                .map(|s| s.values.len())
                .max()
                .unwrap_or(0),
        );
        for index in 0..points {
            let values = self
                .series
                .iter()
                .filter_map(|s| s.values.get(index).copied());
            match self.stacked {
                true => {
                    let (up, down) = values.fold((0.0, 0.0), |(up, down), v| {
                        if v >= 0.0 {
                            (up + v, down)
                        } else {
                            (up, down + v)
                        }
                    });
                    high = high.max(up);
                    low = low.min(down);
                }
                false => {
                    for value in values {
                        high = high.max(value);
                        low = low.min(value);
                    }
                }
            }
        }
        (low, high)
    }

    fn plot(&self, rux: &mut Rux<'_>, rect: Rect) -> ChartOutcome {
        let theme = rux.theme();
        let colours = Instrument::of(theme);
        let legend = self.has_legend();
        let legend_height = match legend {
            true => self.legend_height(rux, rect.width()),
            false => 0.0,
        };
        let screen_rect = Rect::from_min_max(
            rect.min,
            Pos2::new(
                rect.right(),
                rect.bottom()
                    - if legend {
                        legend_height + rux.z(10.0)
                    } else {
                        0.0
                    },
            ),
        );
        screen(rux, screen_rect, rux.z(9.0), true);
        let painter = rux.painter().clone();
        let (low, high) = self.range();
        let ticks = nice_ticks(low, high, 5);
        let (bottom_value, top_value) = (
            ticks.first().copied().unwrap_or(0.0),
            ticks.last().copied().unwrap_or(1.0),
        );
        let axis = rux.zs(AXIS);
        let widest = ticks
            .iter()
            .map(|t| rux.measure(axis, &(self.format)(*t)).x)
            .fold(0.0_f32, f32::max);
        let plot = Rect::from_min_max(
            Pos2::new(
                screen_rect.left() + widest + rux.z(14.0),
                screen_rect.top() + rux.z(22.0),
            ),
            Pos2::new(
                screen_rect.right() - rux.z(12.0),
                screen_rect.bottom() - rux.z(22.0),
            ),
        );
        if plot.width() < 20.0 || plot.height() < 20.0 {
            return ChartOutcome::default();
        }
        let y_of = |value: f64| {
            plot.bottom()
                - ((value - bottom_value) / (top_value - bottom_value).max(f64::EPSILON)) as f32
                    * plot.height()
        };
        // Gridlines, dotted, with their numbers on the left.
        for tick in &ticks {
            let y = y_of(*tick);
            dotted(
                &painter,
                plot.left(),
                plot.right(),
                y,
                colours.rule,
                rux.z(2.0),
                rux.z(4.0),
            );
            let words = (self.format)(*tick);
            let galley = text::layout(&painter, axis, &words, theme.ink.i300);
            painter.galley(
                Pos2::new(
                    plot.left() - rux.z(8.0) - galley.size().x,
                    y - galley.size().y / 2.0,
                ),
                galley,
                theme.ink.i300,
            );
        }
        let zero = y_of(0.0_f64.clamp(bottom_value, top_value));
        let points = self.labels.len().max(
            self.series
                .iter()
                .map(|s| s.values.len())
                .max()
                .unwrap_or(0),
        );
        if points == 0 {
            return ChartOutcome::default();
        }
        let group = plot.width() / points as f32;
        // The category labels under the plot. When they do not fit a column each, every second, third
        // or fifth is written instead, always counting from the first, so a run of years or weeks reads
        // as a scale rather than as a row of ellipses.
        let widest_label = self
            .labels
            .iter()
            .map(|label| rux.measure(axis, label).x)
            .fold(0.0_f32, f32::max);
        let stride = match widest_label + rux.z(6.0) <= group {
            true => 1,
            false => {
                let need = ((widest_label + rux.z(6.0)) / group.max(1.0)).ceil() as usize;
                [2, 3, 4, 5, 10, 20, 25, 50]
                    .into_iter()
                    .find(|step| *step >= need)
                    .unwrap_or(need)
            }
        };
        for (index, label) in self.labels.iter().enumerate() {
            if index % stride != 0 {
                continue;
            }
            let centre = plot.left() + group * (index as f32 + 0.5);
            let room = group * stride as f32 - rux.z(4.0);
            let galley = text::elided(&painter, axis, label, theme.ink.i400, room);
            let x = (centre - galley.size().x / 2.0).max(plot.left() - rux.z(6.0));
            painter.galley(
                Pos2::new(x, plot.bottom() + rux.z(6.0)),
                galley,
                theme.ink.i400,
            );
        }
        let hover = rux
            .ui
            .interact(screen_rect, self.id.with("hover"), Sense::hover());
        hover.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Image, true, self.describe())
        });
        let pointer = hover.hover_pos();
        let mut outcome = ChartOutcome::default();
        match self.kind {
            ChartKind::Bar => self.bars(
                rux,
                &painter,
                plot,
                group,
                zero,
                &y_of,
                pointer,
                &mut outcome,
            ),
            _ => self.traces(
                rux,
                &painter,
                plot,
                group,
                zero,
                &y_of,
                pointer,
                &mut outcome,
            ),
        }
        // The reading in the corner, for the point under the pointer.
        if let Some((series, index)) = outcome.hovered {
            if let Some(one) = self.series.get(series) {
                let value = one.values.get(index).copied().unwrap_or(0.0);
                let label = self.labels.get(index).map(String::as_str).unwrap_or("");
                let words = match self.series.len() > 1 {
                    true => format!("{label} \u{b7} {} {}", one.name, (self.format)(value)),
                    false => format!("{label} {}", (self.format)(value)),
                };
                let galley = text::layout(&painter, rux.zs(READOUT), &words, theme.ink.i900);
                let at = Pos2::new(
                    screen_rect.right() - rux.z(10.0) - galley.size().x,
                    screen_rect.top() + rux.z(6.0),
                );
                led(
                    &painter,
                    Pos2::new(at.x - rux.z(8.0), at.y + galley.size().y / 2.0),
                    rux.z(2.5),
                    one.colour,
                    colours.led_off,
                    1.0,
                );
                painter.galley(at, galley, theme.ink.i900);
            }
        }
        if legend {
            self.legend(
                rux,
                &painter,
                Rect::from_min_max(
                    Pos2::new(rect.left(), screen_rect.bottom() + rux.z(10.0)),
                    rect.max,
                ),
            );
        }
        outcome
    }

    #[allow(clippy::too_many_arguments)]
    fn bars(
        &self,
        rux: &Rux<'_>,
        painter: &egui::Painter,
        plot: Rect,
        group: f32,
        zero: f32,
        y_of: &dyn Fn(f64) -> f32,
        pointer: Option<Pos2>,
        outcome: &mut ChartOutcome,
    ) {
        let count = self.series.len().max(1);
        let lanes = if self.stacked { 1 } else { count };
        let gap = rux.z(4.0);
        let width = ((group * 0.72 - gap * (lanes as f32 - 1.0)) / lanes as f32)
            .clamp(rux.z(3.0), rux.z(16.0));
        let total = width * lanes as f32 + gap * (lanes as f32 - 1.0);
        let segment = rux.z(5.0);
        let space = rux.z(2.0);
        let mut drawn = 0;
        for index in 0..self.labels.len().max(
            self.series
                .iter()
                .map(|s| s.values.len())
                .max()
                .unwrap_or(0),
        ) {
            let start = plot.left() + group * (index as f32 + 0.5) - total / 2.0;
            let mut stack_up = 0.0;
            let mut stack_down = 0.0;
            for (series, one) in self.series.iter().enumerate() {
                let Some(&value) = one.values.get(index) else {
                    continue;
                };
                let (from, to) = match self.stacked {
                    true => {
                        let base = if value >= 0.0 { stack_up } else { stack_down };
                        if value >= 0.0 {
                            stack_up += value
                        } else {
                            stack_down += value
                        }
                        (base, base + value)
                    }
                    false => (0.0, value),
                };
                let lane = if self.stacked { 0 } else { series };
                let left = start + lane as f32 * (width + gap);
                let rise = ease_back(appearing(
                    rux,
                    self.id.with(("bar", index, series)),
                    drawn as f32 * timing::STAGGER,
                    timing::RISE,
                ));
                drawn += 1;
                let y_from = y_of(from);
                let y_to = y_of(from + (to - from) * rise as f64);
                let column = Rect::from_min_max(
                    Pos2::new(left, y_from.min(y_to)),
                    Pos2::new(left + width, y_from.max(y_to)),
                );
                let hovered = pointer.is_some_and(|p| {
                    p.x >= left - gap / 2.0
                        && p.x <= left + width + gap / 2.0
                        && p.y >= plot.top()
                        && p.y <= plot.bottom() + rux.z(18.0)
                });
                if hovered {
                    outcome.hovered = Some((series, index));
                }
                let colour = match hovered {
                    true => brighten(one.colour),
                    false => one.colour,
                };
                // A soft glow round the lit column, the light a segment throws on the screen around it.
                painter.rect_filled(
                    column.expand(rux.z(2.0)),
                    rux.z(3.0),
                    colour.gamma_multiply(0.10),
                );
                // Segments from the baseline outward, the last one cut to the value.
                let upward = to >= from;
                let mut edge = if upward {
                    column.bottom()
                } else {
                    column.top()
                };
                let end = if upward {
                    column.top()
                } else {
                    column.bottom()
                };
                while (upward && edge > end + 0.5) || (!upward && edge < end - 0.5) {
                    let next = if upward {
                        (edge - segment).max(end)
                    } else {
                        (edge + segment).min(end)
                    };
                    let cell = Rect::from_min_max(
                        Pos2::new(left, edge.min(next)),
                        Pos2::new(left + width, edge.max(next)),
                    );
                    painter.rect_filled(cell, rux.z(1.0), colour);
                    edge = if upward { next - space } else { next + space };
                }
            }
        }
        let _ = zero;
    }

    #[allow(clippy::too_many_arguments)]
    fn traces(
        &self,
        rux: &Rux<'_>,
        painter: &egui::Painter,
        plot: Rect,
        group: f32,
        zero: f32,
        y_of: &dyn Fn(f64) -> f32,
        pointer: Option<Pos2>,
        outcome: &mut ChartOutcome,
    ) {
        let mut below = vec![
            0.0_f64;
            self.labels.len().max(
                self.series
                    .iter()
                    .map(|s| s.values.len())
                    .max()
                    .unwrap_or(0)
            )
        ];
        let hovered_index = pointer
            .filter(|p| plot.expand(rux.z(10.0)).contains(*p))
            .map(|p| (((p.x - plot.left()) / group) as usize).min(below.len().saturating_sub(1)));
        for (series, one) in self.series.iter().enumerate() {
            let draw = ease_out(appearing(
                rux,
                self.id.with(("trace", series)),
                series as f32 * timing::STAGGER * 3.0,
                timing::RISE * 1.6,
            ));
            let points: Vec<Pos2> = one
                .values
                .iter()
                .enumerate()
                .map(|(index, value)| {
                    let base = if self.stacked { below[index] } else { 0.0 };
                    Pos2::new(
                        plot.left() + group * (index as f32 + 0.5),
                        y_of(base + value),
                    )
                })
                .collect();
            let bases: Vec<f32> = (0..points.len())
                .map(|index| {
                    if self.stacked {
                        y_of(below[index])
                    } else {
                        zero
                    }
                })
                .collect();
            if self.stacked {
                for (index, value) in one.values.iter().enumerate() {
                    below[index] += value;
                }
            }
            if points.is_empty() {
                continue;
            }
            // How far along the trace has drawn itself, as an x.
            let reach = plot.left() + (plot.width()) * draw;
            let visible: Vec<Pos2> = clip_polyline(&points, reach);
            if self.kind == ChartKind::Area && visible.len() > 1 {
                let mut mesh = Mesh::default();
                for (at, point) in visible.iter().enumerate() {
                    let base = bases.get(at).copied().unwrap_or(zero);
                    mesh.colored_vertex(*point, one.colour.gamma_multiply(0.30));
                    mesh.colored_vertex(Pos2::new(point.x, base), one.colour.gamma_multiply(0.02));
                }
                for i in 0..(visible.len() as u32 - 1) {
                    let a = i * 2;
                    mesh.add_triangle(a, a + 1, a + 2);
                    mesh.add_triangle(a + 1, a + 2, a + 3);
                }
                painter.add(Shape::mesh(mesh));
            }
            if visible.len() > 1 {
                // The glow, then the trace.
                painter.add(Shape::line(
                    visible.clone(),
                    Stroke::new(rux.z(6.0), one.colour.gamma_multiply(0.08)),
                ));
                painter.add(Shape::line(
                    visible.clone(),
                    Stroke::new(rux.z(3.0), one.colour.gamma_multiply(0.18)),
                ));
                painter.add(Shape::line(
                    visible.clone(),
                    Stroke::new(rux.z(1.6), one.colour),
                ));
            }
            let screen_colour = Instrument::of(rux.theme()).screen;
            for (index, point) in points.iter().enumerate() {
                if point.x > reach + 0.5 {
                    break;
                }
                let lit = hovered_index == Some(index);
                if lit {
                    outcome.hovered = Some((series, index));
                }
                let radius = rux.z(if lit { 3.5 } else { 2.4 });
                painter.circle_filled(*point, radius + rux.z(1.2), screen_colour);
                painter.circle_filled(
                    *point,
                    radius,
                    if lit {
                        brighten(one.colour)
                    } else {
                        one.colour
                    },
                );
            }
        }
        if let Some(index) = hovered_index {
            let x = plot.left() + group * (index as f32 + 0.5);
            dotted_vertical(
                painter,
                x,
                plot.top(),
                plot.bottom(),
                Instrument::of(rux.theme()).rule.gamma_multiply(2.0),
                rux.z(2.0),
                rux.z(3.0),
            );
        }
    }

    fn legend(&self, rux: &Rux<'_>, painter: &egui::Painter, rect: Rect) {
        let theme = rux.theme();
        let off = Instrument::of(theme).led_off;
        let style = rux.zs(LEGEND);
        let row = rux.z(16.0);
        let mut pen = Pos2::new(rect.left() + rux.z(4.0), rect.top());
        for one in &self.series {
            let words = rux.measure(style, one.name).x;
            let need = rux.z(14.0) + words + rux.z(16.0);
            if pen.x > rect.left() + rux.z(4.0) && pen.x + need > rect.right() {
                pen = Pos2::new(rect.left() + rux.z(4.0), pen.y + row);
            }
            led(
                painter,
                Pos2::new(pen.x + rux.z(3.0), pen.y + row / 2.0),
                rux.z(3.0),
                one.colour,
                off,
                1.0,
            );
            let galley = text::layout(painter, style, one.name, theme.ink.i500);
            painter.galley(
                Pos2::new(pen.x + rux.z(12.0), pen.y + (row - galley.size().y) / 2.0),
                galley,
                theme.ink.i500,
            );
            pen.x += need;
        }
    }

    /// A segmented ring: each slice an arc with a small gap either side, sweeping round from the top
    /// the first time it is drawn.
    fn donut(&self, rux: &mut Rux<'_>, rect: Rect) -> ChartOutcome {
        let theme = rux.theme();
        let colours = Instrument::of(theme);
        let side = rect.height().min(rect.width() * 0.5);
        let dial = Rect::from_min_size(rect.min, Vec2::splat(side));
        screen(rux, dial, side / 2.0, false);
        let painter = rux.painter().clone();
        let centre = dial.center();
        let outer = side / 2.0 - rux.z(9.0);
        let inner = outer * 0.66;
        let values: Vec<f64> = self
            .series
            .first()
            .map(|s| s.values.iter().map(|v| v.max(0.0)).collect())
            .unwrap_or_default();
        let total: f64 = values.iter().sum();
        let sweep = ease_out(appearing(
            rux,
            self.id.with("sweep"),
            0.0,
            timing::RISE * 1.8,
        ));
        let hover = rux.ui.interact(rect, self.id.with("hover"), Sense::hover());
        hover.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Image, true, self.describe())
        });
        let pointer = hover.hover_pos();
        let mut outcome = ChartOutcome::default();
        let gap = (1.6_f32).to_radians();
        let mut angle = -std::f32::consts::FRAC_PI_2;
        let colour_of = |index: usize| colours.series(index);
        if total > 0.0 {
            for (index, value) in values.iter().enumerate() {
                let share = (*value / total) as f32 * std::f32::consts::TAU * sweep;
                if share <= gap * 2.0 {
                    angle += share;
                    continue;
                }
                let (from, to) = (angle + gap, angle + share - gap);
                let hovered = pointer.is_some_and(|p| {
                    let d = p - centre;
                    let r = d.length();
                    let mut a = d.y.atan2(d.x);
                    while a < -std::f32::consts::FRAC_PI_2 {
                        a += std::f32::consts::TAU;
                    }
                    r >= inner && r <= outer + rux.z(4.0) && a >= from && a <= to
                });
                if hovered {
                    outcome.hovered = Some((0, index));
                }
                let colour = if hovered {
                    brighten(colour_of(index))
                } else {
                    colour_of(index)
                };
                let reach = if hovered { rux.z(3.0) } else { 0.0 };
                ring_arc(
                    &painter,
                    centre,
                    inner - rux.z(2.0),
                    outer + rux.z(2.0) + reach,
                    from,
                    to,
                    colour.gamma_multiply(0.10),
                );
                ring_arc(&painter, centre, inner, outer + reach, from, to, colour);
                angle += share;
            }
        }
        // The middle reads the slice under the pointer, or the largest.
        let shown = outcome.hovered.map(|(_, i)| i).or_else(|| {
            values
                .iter()
                .enumerate()
                .max_by(|a, b| a.1.total_cmp(b.1))
                .map(|(i, _)| i)
        });
        if let (Some(index), true) = (shown, total > 0.0) {
            let percent = format!("{:.0}%", values[index] / total * 100.0);
            let big = text::layout(
                &painter,
                rux.zs(Style::mono(16.0).medium()),
                &percent,
                theme.ink.i900,
            );
            let label = self.labels.get(index).cloned().unwrap_or_default();
            let small = text::elided(&painter, rux.zs(SILK), &label, theme.ink.i400, inner * 1.6);
            let height = big.size().y + small.size().y;
            painter.galley(
                Pos2::new(centre.x - big.size().x / 2.0, centre.y - height / 2.0),
                big.clone(),
                theme.ink.i900,
            );
            painter.galley(
                Pos2::new(
                    centre.x - small.size().x / 2.0,
                    centre.y - height / 2.0 + big.size().y,
                ),
                small,
                theme.ink.i400,
            );
        }
        // The legend beside the dial, one row a slice.
        let style = rux.zs(LEGEND);
        let row = rux.z(18.0);
        let left = dial.right() + rux.z(18.0);
        let top = dial.center().y - row * values.len() as f32 / 2.0;
        for (index, label) in self.labels.iter().enumerate().take(values.len()) {
            let y = top + row * index as f32;
            if y < rect.top() - 1.0 || y + row > rect.bottom() + 1.0 {
                continue;
            }
            let lit = outcome.hovered.map(|(_, i)| i) == Some(index);
            led(
                &painter,
                Pos2::new(left + rux.z(3.0), y + row / 2.0),
                rux.z(3.0),
                colour_of(index),
                colours.led_off,
                1.0,
            );
            let ink = if lit { theme.ink.i900 } else { theme.ink.i700 };
            let share = if total > 0.0 {
                format!("{:.0}%", values[index] / total * 100.0)
            } else {
                String::new()
            };
            let number = text::layout(&painter, rux.zs(AXIS), &share, theme.ink.i400);
            let room = (rect.right() - left - rux.z(14.0) - number.size().x - rux.z(10.0))
                .max(rux.z(20.0));
            let galley = text::elided(&painter, style, label, ink, room);
            painter.galley(
                Pos2::new(left + rux.z(12.0), y + (row - galley.size().y) / 2.0),
                galley.clone(),
                ink,
            );
            painter.galley(
                Pos2::new(
                    left + rux.z(12.0) + galley.size().x + rux.z(8.0),
                    y + (row - number.size().y) / 2.0,
                ),
                number,
                theme.ink.i400,
            );
        }
        outcome
    }
}

/// A colour a little brighter, for the thing under the pointer.
fn brighten(colour: Color32) -> Color32 {
    let up = |c: u8| (c as f32 + (255.0 - c as f32) * 0.25) as u8;
    Color32::from_rgb(up(colour.r()), up(colour.g()), up(colour.b()))
}

/// The polyline through `points` cut off at `x`, the last point placed on the cut.
fn clip_polyline(points: &[Pos2], x: f32) -> Vec<Pos2> {
    let mut out = Vec::new();
    for pair in points.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        if out.is_empty() {
            if a.x > x {
                break;
            }
            out.push(a);
        }
        if b.x <= x {
            out.push(b);
        } else {
            let t = ((x - a.x) / (b.x - a.x).max(f32::EPSILON)).clamp(0.0, 1.0);
            out.push(a + (b - a) * t);
            break;
        }
    }
    if out.is_empty() && points.len() == 1 && points[0].x <= x {
        out.push(points[0]);
    }
    out
}

/// A horizontal dotted line.
fn dotted(
    painter: &egui::Painter,
    left: f32,
    right: f32,
    y: f32,
    colour: Color32,
    on: f32,
    off: f32,
) {
    let mut x = left;
    while x < right {
        let end = (x + on).min(right);
        painter.line_segment(
            [
                Pos2::new(x, y.round() + 0.5),
                Pos2::new(end, y.round() + 0.5),
            ],
            Stroke::new(1.0, colour),
        );
        x += on + off;
    }
}

/// A vertical dotted line.
fn dotted_vertical(
    painter: &egui::Painter,
    x: f32,
    top: f32,
    bottom: f32,
    colour: Color32,
    on: f32,
    off: f32,
) {
    let mut y = top;
    while y < bottom {
        let end = (y + on).min(bottom);
        painter.line_segment(
            [
                Pos2::new(x.round() + 0.5, y),
                Pos2::new(x.round() + 0.5, end),
            ],
            Stroke::new(1.0, colour),
        );
        y += on + off;
    }
}

/// A filled band of a ring between two angles.
fn ring_arc(
    painter: &egui::Painter,
    centre: Pos2,
    inner: f32,
    outer: f32,
    from: f32,
    to: f32,
    colour: Color32,
) {
    let steps = (((to - from).abs() / 0.06).ceil() as usize).max(2);
    let mut mesh = Mesh::default();
    for step in 0..=steps {
        let angle = from + (to - from) * step as f32 / steps as f32;
        let direction = Vec2::new(angle.cos(), angle.sin());
        mesh.colored_vertex(centre + direction * outer, colour);
        mesh.colored_vertex(centre + direction * inner, colour);
    }
    for i in 0..steps as u32 {
        let a = i * 2;
        mesh.add_triangle(a, a + 1, a + 2);
        mesh.add_triangle(a + 1, a + 2, a + 3);
    }
    painter.add(Shape::mesh(mesh));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gridlines_land_on_round_numbers() {
        assert_eq!(
            nice_ticks(0.0, 212.0, 5),
            vec![0.0, 50.0, 100.0, 150.0, 200.0, 250.0]
        );
        assert_eq!(nice_ticks(0.0, 1.0, 4), vec![0.0, 0.25, 0.5, 0.75, 1.0]);
        assert_eq!(nice_ticks(-30.0, 50.0, 4).first().copied(), Some(-40.0));
    }

    #[test]
    fn a_trace_is_cut_where_it_has_drawn_to() {
        let points = [
            Pos2::new(0.0, 0.0),
            Pos2::new(10.0, 10.0),
            Pos2::new(20.0, 0.0),
        ];
        assert_eq!(
            clip_polyline(&points, 5.0),
            vec![Pos2::new(0.0, 0.0), Pos2::new(5.0, 5.0)]
        );
        assert_eq!(clip_polyline(&points, 30.0).len(), 3);
    }
}
