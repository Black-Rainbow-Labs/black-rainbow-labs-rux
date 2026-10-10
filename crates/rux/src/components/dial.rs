//! The Agent-Chat design's charts: a dial, a ring chart, a slider track and a smooth area chart.
//!
//! `task-2235`. The canvas's "Charts" panel draws four, and the design's rule is that there are no
//! segmented meters: an amount is an arc, a slider or a line.
//!
//! **The dial and the ring chart are one drawing.** A raised rim, a channel carved into it, arcs ten
//! points wide with round ends laid in the channel, and a raised disc in the middle carrying the number.
//! A dial is a ring chart of two parts, the amount before and the amount added.
//!
//! **The gap between two arcs is the same everywhere, and a short slice keeps its share.** A round cap
//! reaches half the stroke past the end of the arc it is drawn on, so each arc is shortened by the
//! stroke and by the gap; what is seen of a slice is then exactly its share less the gap. A slice whose
//! share is shorter than the stroke cannot carry two round caps without growing past its share, so it
//! is drawn with square ends at exactly that length instead. [`arc_segments`] is that arithmetic, with
//! no window behind it, and its tests are the two checks the ticket names.

use std::sync::Arc;

use egui::epaint::{ColorMode, PathShape, PathStroke};
use egui::{Color32, Mesh, Pos2, Rect, Response, Sense, Shape, Stroke, Vec2};

use super::round::card;
use crate::render::Fill;
use crate::text::Style;
use crate::theme::Opacity as _;
use crate::Rux;

/// One arc of a ring, in radians clockwise from twelve o'clock.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Segment {
    pub start: f32,
    pub sweep: f32,
    /// Round ends, or square ends for a slice too short for round ones.
    pub round: bool,
}

impl Segment {
    /// How much of the circle's centreline this arc covers once its caps are drawn, in points.
    pub fn seen(&self, radius: f32, width: f32) -> f32 {
        self.sweep * radius + if self.round { width } else { 0.0 }
    }
}

/// Where each slice of a ring of `radius` goes, with `width` stroke and `gap` points between slices.
///
/// The slices go round in order from twelve o'clock. A value that is zero or negative takes no room
/// and draws nothing, which is what `None` in its place says.
pub fn arc_segments(values: &[f32], radius: f32, width: f32, gap: f32) -> Vec<Option<Segment>> {
    let total: f32 = values.iter().filter(|value| **value > 0.0).sum();
    let circumference = std::f32::consts::TAU * radius;
    let alone = values.iter().filter(|value| **value > 0.0).count() == 1;
    let mut around = 0.0;
    values
        .iter()
        .map(|&value| {
            if value <= 0.0 || total <= 0.0 {
                return None;
            }
            let share = circumference * value / total;
            let start = around;
            around += share;
            // One slice is the whole circle and has no neighbour to keep a gap from.
            if alone {
                return Some(Segment { start: 0.0, sweep: std::f32::consts::TAU, round: false });
            }
            let seen = (share - gap).max(0.0);
            let segment = match seen > width {
                true => Segment {
                    start: (start + gap / 2.0 + width / 2.0) / radius,
                    sweep: (seen - width) / radius,
                    round: true,
                },
                false => Segment { start: (start + gap / 2.0) / radius, sweep: seen / radius, round: false },
            };
            Some(segment)
        })
        .collect()
}

/// What paints one slice.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SlicePaint {
    /// The action's gradient, which is what the slice a chart is about is drawn in.
    Action,
    /// A colour of its own: the quiet grey of "last week", or a ring chart's other slices.
    Colour(Color32),
}

/// One slice of a ring.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Slice {
    pub value: f32,
    pub paint: SlicePaint,
}

impl Slice {
    pub fn new(value: f32, paint: SlicePaint) -> Self {
        Self { value, paint }
    }
}

/// The ring chart: a rim, a channel, gapped arcs and a disc with the number on it.
pub struct Ring<'a> {
    slices: Vec<Slice>,
    diameter: f32,
    value: &'a str,
    unit: &'a str,
    caption: Option<(&'a str, Color32)>,
    label: &'a str,
}

impl<'a> Ring<'a> {
    /// A ring of `slices`, 150 points across.
    pub fn new(slices: Vec<Slice>) -> Self {
        Self { slices, diameter: 150.0, value: "", unit: "", caption: None, label: "Ring chart" }
    }

    pub fn diameter(mut self, diameter: f32) -> Self {
        self.diameter = diameter;
        self
    }

    /// The number on the disc, and a smaller unit after it: `178` and ` s`.
    pub fn centre(mut self, value: &'a str, unit: &'a str) -> Self {
        self.value = value;
        self.unit = unit;
        self
    }

    /// The line under the number: `+39%` in the slower colour, or `app` in the quiet ink.
    pub fn caption(mut self, text: &'a str, colour: Color32) -> Self {
        self.caption = Some((text, colour));
        self
    }

    /// What a screen reader is told the chart says.
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = label;
        self
    }

    pub fn measure(&self, rux: &Rux<'_>) -> Vec2 {
        Vec2::splat(rux.z(self.diameter))
    }

    pub fn show(self, rux: &mut Rux<'_>, rect: Rect) -> Response {
        let side = rux.z(self.diameter).min(rect.width()).min(rect.height());
        let rect = Rect::from_center_size(rect.center(), Vec2::splat(side));
        let response = super::interact(rux, rect, "rux-ring", Sense::hover());
        let chat = rux.theme().chat;
        let centre = rect.center();
        // The canvas's 150 point ring: a 132 point channel, arcs on a 55 point radius, an 88 point disc.
        let unit = side / 150.0;
        rux.chrome.surface(rect, side / 2.0, card(&chat, rect), chat.raised_sm());
        let channel = Rect::from_center_size(centre, Vec2::splat(132.0 * unit));
        rux.chrome.surface(channel, channel.width() / 2.0, chat.well, chat.carved());
        let radius = 55.0 * unit;
        let width = 10.0 * unit;
        let gap = 5.0 * unit;
        let values: Vec<f32> = self.slices.iter().map(|slice| slice.value).collect();
        // `linearGradient x1=0 y1=0 x2=1 y2=1` on an `<svg>` turned a quarter back, so on the screen the
        // gradient runs from the bottom left of the ring to its top right.
        let ring = Rect::from_center_size(centre, Vec2::splat(radius * 2.0));
        let action =
            Fill::Linear { from: ring.left_bottom(), to: ring.right_top(), start: chat.action.from, end: chat.action.to };
        for (slice, segment) in self.slices.iter().zip(arc_segments(&values, radius, width, gap)) {
            let Some(segment) = segment else { continue };
            let fill = match slice.paint {
                SlicePaint::Action => action,
                SlicePaint::Colour(colour) => Fill::Solid(colour),
            };
            match segment.round {
                true => rux.chrome.arc(centre, radius, width, segment.start, segment.sweep, fill),
                false => rux.chrome.arc_square(centre, radius, width, segment.start, segment.sweep, fill),
            }
        }
        let disc = Rect::from_center_size(centre, Vec2::splat(88.0 * unit));
        rux.chrome.surface(disc, disc.width() / 2.0, card(&chat, disc), chat.raised_sm());
        self.paint_words(rux, disc, unit);
        let label = self.label.to_owned();
        response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Label, true, label.clone()));
        response
    }

    /// The number, its unit and the caption, centred on the disc.
    fn paint_words(&self, rux: &mut Rux<'_>, disc: Rect, unit: f32) {
        let theme = rux.theme();
        let value_style = Style::sans(25.0 * unit).medium().tracking(-0.03);
        let unit_style = Style::sans(12.0 * unit);
        let caption_style = Style::sans(10.5 * unit);
        let value = rux.text(value_style, self.value, theme.ink.i900);
        let suffix = rux.text(unit_style, self.unit, theme.ink.i400);
        let caption = self.caption.map(|(text, colour)| (rux.text(caption_style, text, colour), colour));
        let gap = 2.0 * unit;
        let line = value.size().y.max(suffix.size().y);
        let height = line + caption.as_ref().map_or(0.0, |(galley, _)| galley.size().y + gap);
        let top = disc.center().y - height / 2.0;
        let width = value.size().x + suffix.size().x;
        let left = disc.center().x - width / 2.0;
        let painter = rux.painter().clone();
        // The value and its unit share a baseline, which is how `178<span> s</span>` sets.
        let value_base = value.rows.first().map_or(line, |row| row.pos.y + row.row.height());
        let suffix_base = suffix.rows.first().map_or(line, |row| row.pos.y + row.row.height());
        painter.galley(Pos2::new(left, top), value.clone(), theme.ink.i900);
        painter.galley(
            Pos2::new(left + value.size().x, top + value_base - suffix_base),
            suffix,
            theme.ink.i400,
        );
        if let Some((galley, colour)) = caption {
            let at = Pos2::new(disc.center().x - galley.size().x / 2.0, top + line + gap);
            painter.galley(at, galley, colour);
        }
    }
}

/// A dial: what something was, and what was added to it, as a ring of two.
pub struct Dial<'a> {
    before: f32,
    added: f32,
    value: &'a str,
    unit: &'a str,
    caption: Option<(&'a str, Color32)>,
    diameter: f32,
    label: &'a str,
}

impl<'a> Dial<'a> {
    pub fn new(before: f32, added: f32) -> Self {
        Self { before, added, value: "", unit: "", caption: None, diameter: 150.0, label: "Dial" }
    }

    pub fn diameter(mut self, diameter: f32) -> Self {
        self.diameter = diameter;
        self
    }

    pub fn centre(mut self, value: &'a str, unit: &'a str) -> Self {
        self.value = value;
        self.unit = unit;
        self
    }

    pub fn caption(mut self, text: &'a str, colour: Color32) -> Self {
        self.caption = Some((text, colour));
        self
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = label;
        self
    }

    pub fn measure(&self, rux: &Rux<'_>) -> Vec2 {
        Vec2::splat(rux.z(self.diameter))
    }

    pub fn show(self, rux: &mut Rux<'_>, rect: Rect) -> Response {
        let quiet = rux.theme().chat.quiet;
        let mut ring = Ring::new(vec![
            Slice::new(self.before, SlicePaint::Colour(quiet)),
            Slice::new(self.added, SlicePaint::Action),
        ])
        .diameter(self.diameter)
        .centre(self.value, self.unit)
        .label(self.label);
        if let Some((text, colour)) = self.caption {
            ring = ring.caption(text, colour);
        }
        ring.show(rux, rect)
    }
}

/// A slider track: a groove, filled with the action's gradient up to the value, with a raised knob at
/// the end of the fill and a quiet mark where the value was before.
///
/// This replaces the segmented bar. It shows one amount against a scale, and the mark answers "and
/// last week?" without a second bar.
pub struct Track {
    value: f32,
    max: f32,
    marker: Option<f32>,
    knob: bool,
    label: String,
}

impl Track {
    pub fn new(value: f32, max: f32) -> Self {
        Self { value, max, marker: None, knob: true, label: String::new() }
    }

    /// Where the value was before, on the same scale.
    pub fn marker(mut self, before: f32) -> Self {
        self.marker = Some(before);
        self
    }

    /// No knob, for a track that only shows an amount.
    pub fn without_knob(mut self) -> Self {
        self.knob = false;
        self
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// As wide as it is given, 22 points tall so the knob fits.
    pub fn measure(&self, rux: &Rux<'_>) -> Vec2 {
        Vec2::new(rux.z(120.0), rux.z(22.0))
    }

    /// The fraction of the track the value fills, from 0 to 1.
    pub fn fraction(value: f32, max: f32) -> f32 {
        match max > 0.0 {
            true => (value / max).clamp(0.0, 1.0),
            false => 0.0,
        }
    }

    pub fn show(self, rux: &mut Rux<'_>, rect: Rect) -> Response {
        let response = super::interact(rux, rect, "rux-track", Sense::hover());
        let chat = rux.theme().chat;
        let theme = rux.theme();
        // `height: 12px` groove, the fill inset two points, the 22 point knob centred on the fill's end.
        let groove = Rect::from_center_size(rect.center(), Vec2::new(rect.width(), rux.z(12.0)));
        rux.chrome.surface(groove, groove.height() / 2.0, chat.well, chat.carved_sm());
        let at = |fraction: f32| groove.left() + groove.width() * fraction;
        let fraction = Self::fraction(self.value, self.max);
        let inset = rux.z(2.0);
        let end = (at(fraction) - inset).max(groove.left() + inset);
        let fill = Rect::from_min_max(
            Pos2::new(groove.left() + inset, groove.top() + inset),
            Pos2::new(end, groove.bottom() - inset),
        );
        if fill.width() > 0.5 {
            rux.chrome.rect(fill, fill.height() / 2.0, Fill::gradient(chat.action, fill));
        }
        if let Some(before) = self.marker {
            // `width: 2px; height: 18px; top: -3px`, the quiet ink at 0.6.
            let x = at(Self::fraction(before, self.max));
            let mark = Rect::from_min_size(
                Pos2::new(x - rux.z(1.0), groove.top() - rux.z(3.0)),
                Vec2::new(rux.z(2.0), rux.z(18.0)),
            );
            // Into the decoration, under the knob, which is the order the canvas draws them in.
            rux.chrome.rect(mark, rux.z(1.0), theme.ink.i400.at(0.6));
        }
        if self.knob {
            let centre = Pos2::new((at(fraction) - rux.z(2.0)).max(groove.left() + rux.z(11.0)), groove.center().y);
            let knob = Rect::from_center_size(centre, Vec2::splat(rux.z(22.0)));
            rux.chrome.surface(knob, knob.width() / 2.0, card(&chat, knob), chat.raised_sm());
        }
        let label = self.label;
        response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::ProgressIndicator, true, label.clone()));
        response
    }
}

/// A smooth area chart in a well: one series as a curve with a fading fill under it, a dashed line at
/// a moment worth naming, and the last value marked.
pub struct AreaChart<'a> {
    values: &'a [f32],
    range: Option<(f32, f32)>,
    marker: Option<(usize, &'a str)>,
    ends: Option<(&'a str, &'a str)>,
    label: &'a str,
}

/// The chart's own height inside the well, which is the canvas's `viewBox="0 0 360 150"`.
const AREA_HEIGHT: f32 = 150.0;

impl<'a> AreaChart<'a> {
    pub fn new(values: &'a [f32]) -> Self {
        Self { values, range: None, marker: None, ends: None, label: "Area chart" }
    }

    /// The values at the bottom and the top of the chart. Without it the chart fits the values.
    pub fn range(mut self, low: f32, high: f32) -> Self {
        self.range = Some((low, high));
        self
    }

    /// A dashed line at value `index`, named: `rux added`.
    pub fn marker(mut self, index: usize, text: &'a str) -> Self {
        self.marker = Some((index, text));
        self
    }

    /// The words under the first and the last value: `26 Sep` and `9 Oct`.
    pub fn ends(mut self, first: &'a str, last: &'a str) -> Self {
        self.ends = Some((first, last));
        self
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = label;
        self
    }

    /// The well's padding, the chart and the row of words under it.
    pub fn measure(&self, rux: &Rux<'_>) -> Vec2 {
        let words = match self.ends {
            Some(_) => rux.z(6.0) + rux.measure(rux.zs(Style::sans(10.5)), "9").y,
            None => 0.0,
        };
        Vec2::new(rux.z(320.0), rux.z(14.0 + AREA_HEIGHT + 10.0) + words)
    }

    pub fn show(self, rux: &mut Rux<'_>, rect: Rect) -> Response {
        let response = super::interact(rux, rect, "rux-area-chart", Sense::hover());
        let chat = rux.theme().chat;
        let theme = rux.theme();
        rux.chrome.surface(rect, rux.z(18.0), chat.well, chat.carved_sm());
        let inner = Rect::from_min_max(
            rect.min + Vec2::splat(rux.z(14.0)),
            Pos2::new(rect.right() - rux.z(14.0), rect.top() + rux.z(14.0 + AREA_HEIGHT)),
        );
        let points = self.points(inner, rux.zoom());
        let painter = rux.painter().clone();
        if points.len() >= 2 {
            let curve = smooth(&points);
            self.paint_area(&painter, &curve, &points, inner, rux.zoom(), &chat);
            let (from, to) = (chat.action.from, chat.action.to);
            let (left, right) = (inner.left(), inner.right());
            let stroke = PathStroke {
                width: rux.z(3.0),
                color: ColorMode::UV(Arc::new(move |_, at: Pos2| {
                    lerp(from, to, ((at.x - left) / (right - left).max(1.0)).clamp(0.0, 1.0))
                })),
                kind: egui::StrokeKind::Middle,
            };
            painter.add(Shape::Path(PathShape::line(curve, stroke)));
            if let Some((index, text)) = self.marker.filter(|(index, _)| *index < points.len()) {
                let x = points[index].x;
                let dashes = Shape::dashed_line(
                    &[Pos2::new(x, inner.top() + rux.z(6.0)), Pos2::new(x, inner.top() + rux.z(140.0))],
                    Stroke::new(1.0, theme.ink.i400.at(0.4)),
                    rux.z(3.0),
                    rux.z(4.0),
                );
                painter.extend(dashes);
                let galley = rux.text(rux.zs(Style::sans(10.0)), text, theme.ink.i400);
                let at = Pos2::new(x - rux.z(6.0) - galley.size().x, inner.top() + rux.z(4.0));
                painter.galley(at, galley, theme.ink.i400);
            }
            if let Some(last) = points.last() {
                // `r="7"` filled with the card, ringed in sky three points wide.
                painter.circle(
                    *last,
                    rux.z(7.0),
                    chat.card.from.lerp_to_gamma(chat.card.to, 0.5),
                    Stroke::new(rux.z(3.0), chat.sky),
                );
            }
        }
        if let Some((first, last)) = self.ends {
            let style = rux.zs(Style::sans(10.5));
            let top = inner.bottom() + rux.z(6.0);
            let first = rux.text(style, first, theme.ink.i400);
            let last = rux.text(style, last, theme.ink.i400);
            painter.galley(Pos2::new(inner.left(), top), first, theme.ink.i400);
            let right = Pos2::new(inner.right() - last.size().x, top);
            painter.galley(right, last, theme.ink.i400);
        }
        let label = self.label.to_owned();
        response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Label, true, label.clone()));
        response
    }

    /// Each value's place, on the canvas's own plot inside a 360 by 150 box: `x` from 8 to 350 and
    /// `y` from 138 at the bottom of the range to 26 at the top.
    fn points(&self, inner: Rect, zoom: f32) -> Vec<Pos2> {
        let (low, high) = self.range.unwrap_or_else(|| {
            let low = self.values.iter().copied().fold(f32::INFINITY, f32::min);
            let high = self.values.iter().copied().fold(f32::NEG_INFINITY, f32::max);
            let pad = ((high - low) * 0.15).max(1.0);
            (low - pad, high + pad)
        });
        let across = inner.width() / 360.0;
        let count = self.values.len().max(2) - 1;
        self.values
            .iter()
            .enumerate()
            .map(|(index, value)| {
                let x = inner.left() + (8.0 + index as f32 * 342.0 / count as f32) * across;
                let share = ((value - low) / (high - low).max(f32::EPSILON)).clamp(-0.1, 1.1);
                let y = inner.top() + (138.0 - share * 112.0) * zoom;
                Pos2::new(x, y)
            })
            .collect()
    }

    /// The fill under the curve: the action's first colour at 0.4 under the highest point, fading to
    /// nothing six points below the plot's floor.
    fn paint_area(
        &self,
        painter: &egui::Painter,
        curve: &[Pos2],
        points: &[Pos2],
        inner: Rect,
        zoom: f32,
        chat: &crate::theme::Chat,
    ) {
        let floor = inner.top() + (138.0 + 6.0) * zoom;
        let top = points.iter().map(|point| point.y).fold(f32::INFINITY, f32::min);
        let colour = |y: f32| {
            let t = ((y - top) / (floor - top).max(1.0)).clamp(0.0, 1.0);
            chat.action.from.at(0.4 * (1.0 - t))
        };
        let mut mesh = Mesh::default();
        for point in curve {
            mesh.colored_vertex(*point, colour(point.y));
            mesh.colored_vertex(Pos2::new(point.x, floor), Color32::TRANSPARENT);
        }
        for index in 0..curve.len().saturating_sub(1) as u32 {
            let base = index * 2;
            mesh.add_triangle(base, base + 1, base + 2);
            mesh.add_triangle(base + 1, base + 3, base + 2);
        }
        painter.add(Shape::mesh(mesh));
    }
}

/// The curve through `points`, as the canvas draws it: each span a cubic whose handles are a sixth of
/// the way along the line between the points either side, then flattened into short lines.
pub fn smooth(points: &[Pos2]) -> Vec<Pos2> {
    const STEPS: usize = 12;
    let mut curve = Vec::with_capacity(points.len() * STEPS + 1);
    if let Some(first) = points.first() {
        curve.push(*first);
    }
    for index in 0..points.len().saturating_sub(1) {
        let p0 = points[index.saturating_sub(1)];
        let p1 = points[index];
        let p2 = points[index + 1];
        let p3 = points[(index + 2).min(points.len() - 1)];
        let c1 = p1 + (p2 - p0) / 6.0;
        let c2 = p2 - (p3 - p1) / 6.0;
        for step in 1..=STEPS {
            let t = step as f32 / STEPS as f32;
            let u = 1.0 - t;
            let at = p1.to_vec2() * (u * u * u)
                + c1.to_vec2() * (3.0 * u * u * t)
                + c2.to_vec2() * (3.0 * u * t * t)
                + p2.to_vec2() * (t * t * t);
            curve.push(at.to_pos2());
        }
    }
    curve
}

fn lerp(from: Color32, to: Color32, t: f32) -> Color32 {
    let mix = |a: u8, b: u8| (f32::from(a) + (f32::from(b) - f32::from(a)) * t).round() as u8;
    Color32::from_rgba_premultiplied(
        mix(from.r(), to.r()),
        mix(from.g(), to.g()),
        mix(from.b(), to.b()),
        mix(from.a(), to.a()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const RADIUS: f32 = 55.0;
    const WIDTH: f32 = 10.0;
    const GAP: f32 = 5.0;

    fn seen(segments: &[Option<Segment>]) -> Vec<f32> {
        segments.iter().flatten().map(|segment| segment.seen(RADIUS, WIDTH)).collect()
    }

    /// The ticket's first check: arcs must not touch at the joins. Each arc's ends, caps included, are
    /// `GAP` points from the next arc's along the centreline, all the way round.
    #[test]
    fn every_join_has_the_same_gap_and_no_two_arcs_touch() {
        for values in [vec![128.0, 50.0], vec![131.0, 38.0, 9.0], vec![1.0, 1.0, 1.0, 1.0]] {
            let segments = arc_segments(&values, RADIUS, WIDTH, GAP);
            let ends: Vec<(f32, f32)> = segments
                .iter()
                .flatten()
                .map(|segment| {
                    let cap = if segment.round { WIDTH / 2.0 } else { 0.0 };
                    (segment.start * RADIUS - cap, (segment.start + segment.sweep) * RADIUS + cap)
                })
                .collect();
            let circumference = std::f32::consts::TAU * RADIUS;
            for (index, (_, end)) in ends.iter().enumerate() {
                let next = ends[(index + 1) % ends.len()].0;
                let next = if index + 1 == ends.len() { next + circumference } else { next };
                assert!((next - end - GAP).abs() < 0.01, "{values:?}: gap after slice {index} is {}", next - end);
            }
        }
    }

    /// The ticket's second check: the smallest slice stays in proportion to its share. What is seen of
    /// every slice is its share of the circle less one gap, so the ratio between any two slices' seen
    /// lengths is the ratio of their shares once the gap is put back.
    #[test]
    fn the_smallest_slice_keeps_its_share() {
        let values = [131.0, 38.0, 9.0, 1.0];
        let circumference = std::f32::consts::TAU * RADIUS;
        let total: f32 = values.iter().sum();
        let segments = arc_segments(&values, RADIUS, WIDTH, GAP);
        for (value, seen) in values.iter().zip(seen(&segments)) {
            let share = circumference * value / total;
            assert!((seen - (share - GAP).max(0.0)).abs() < 0.01, "{value}: seen {seen} of share {share}");
        }
        // The one-in-179 slice is shorter than the stroke, so it is drawn square rather than grown into
        // a ten point dot by its caps.
        assert!(!segments[3].unwrap().round);
        assert!(segments[2].unwrap().round);
    }

    #[test]
    fn a_dial_is_the_canvas_s_two_arcs() {
        // `last = 128, total = 178`: the quiet arc first from twelve o'clock, then the blue one.
        let segments = arc_segments(&[128.0, 50.0], RADIUS, WIDTH, GAP);
        let quiet = segments[0].unwrap();
        let added = segments[1].unwrap();
        assert!(quiet.round && added.round);
        assert!(quiet.start < added.start);
        assert!((quiet.start * RADIUS - (GAP / 2.0 + WIDTH / 2.0)).abs() < 0.001);
    }

    #[test]
    fn nothing_and_one_slice_are_drawn_sensibly() {
        assert_eq!(arc_segments(&[0.0, 0.0], RADIUS, WIDTH, GAP), vec![None, None]);
        let whole = arc_segments(&[0.0, 7.0], RADIUS, WIDTH, GAP);
        assert_eq!(whole[0], None);
        assert!((whole[1].unwrap().sweep - std::f32::consts::TAU).abs() < 0.001);
    }

    #[test]
    fn a_track_fills_its_share_and_no_more() {
        assert_eq!(Track::fraction(70.0, 140.0), 0.5);
        assert_eq!(Track::fraction(200.0, 140.0), 1.0);
        assert_eq!(Track::fraction(5.0, 0.0), 0.0);
    }

    #[test]
    fn the_curve_passes_through_every_point() {
        let points = [Pos2::new(0.0, 10.0), Pos2::new(10.0, 0.0), Pos2::new(20.0, 5.0)];
        let curve = smooth(&points);
        assert_eq!(curve.first(), Some(&points[0]));
        assert!((curve[12] - points[1]).length() < 0.001);
        assert!((*curve.last().unwrap() - points[2]).length() < 0.001);
    }
}
