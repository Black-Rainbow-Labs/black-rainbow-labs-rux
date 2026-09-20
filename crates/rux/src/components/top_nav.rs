//! The pill of navigation items along the top of the page.
//!
//! `.topnav` and `.topnav__item`. One rule in it decides the whole shape of the component:
//! `.topnav__item span { display: none }` with `.topnav__item.is-active span { display: inline }` —
//! **only the chosen item shows its label**, so the strip breathes and the chosen one is unmistakable.
//! That also means the component's width changes with which item is active, which is why it measures
//! itself rather than being given a width.

use egui::{Color32, Rect, Sense, Vec2};

use super::{interact, pointing};
use crate::icon::{Icon, Mark};
use crate::layout::{Cross, Flex, Pad};
use crate::text::Style;
use crate::{Fill, Rux};

/// One entry in the strip.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NavItem {
    pub id: &'static str,
    pub icon: Icon,
    pub label: &'static str,
}

impl NavItem {
    pub const fn new(id: &'static str, icon: Icon, label: &'static str) -> Self {
        Self { id, icon, label }
    }
}

/// What happened this frame.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct TopNavOutcome {
    pub chosen: Option<usize>,
}

/// `padding: 6px` on the strip, `gap: 4px` between items.
const STRIP_PAD: f32 = 6.0;
const STRIP_GAP: f32 = 4.0;
/// `.topnav__item { padding: 9px 14px }`, and `.is-active { padding: 9px 18px }`.
const ITEM_PAD_Y: f32 = 9.0;
const ITEM_PAD_X: f32 = 14.0;
const ACTIVE_PAD_X: f32 = 18.0;
/// `<SbIcon size={20}/>`, and `gap: 8px` between a mark and its label.
const MARK: f32 = 20.0;
const ITEM_GAP: f32 = 8.0;
/// How far an item's side padding may be trimmed before the strip stops shrinking.
///
/// **The one number in this file the stylesheet does not contain.** 10 leaves a 40 point item, which is
/// the diameter of `.toolbtn` — the reference's own smallest comfortable round control — so five items
/// and their gaps come to 232 points and fit any phone in portrait.
const TRIM_PAD_X: f32 = 10.0;

/// How much of itself the strip is showing.
///
/// The strip's width changes with which item is active, because only the active one has a label. When
/// there is not enough room for that, these are the two things it gives up, in order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fit {
    /// Every rule as written: the active item carries its label.
    Full,
    /// `.topnav__item span { display: none }` applied to the active item as well — marks only.
    IconsOnly,
    /// Marks only, and the side padding trimmed to [`TRIM_PAD_X`].
    Trimmed,
}

/// The row of navigation items.
pub struct TopNav<'a> {
    items: &'a [NavItem],
    active: usize,
}

impl<'a> TopNav<'a> {
    pub fn new(items: &'a [NavItem], active: usize) -> Self {
        Self { items, active }
    }

    /// How wide each item comes out, which both the measurement and the drawing read.
    ///
    /// `fit` is how it behaves when the strip is wider than the room it has, and there are only two
    /// things to give up. See [`Fit`].
    fn widths(&self, rux: &Rux<'_>, fit: Fit) -> Vec<f32> {
        let label = matches!(fit, Fit::Full);
        let pad_x = match fit {
            Fit::Trimmed => TRIM_PAD_X,
            _ => ITEM_PAD_X,
        };
        self.items
            .iter()
            .enumerate()
            .map(|(index, item)| match label && index == self.active {
                true => {
                    let words = rux.measure(Style::CONTROL, item.label);
                    MARK + ITEM_GAP + words.x + ACTIVE_PAD_X * 2.0
                }
                false => MARK + pad_x * 2.0,
            })
            .collect()
    }

    /// How wide the strip is in a given fit.
    fn width(&self, rux: &Rux<'_>, fit: Fit) -> f32 {
        let widths = self.widths(rux, fit);
        let gaps = STRIP_GAP * widths.len().saturating_sub(1) as f32;
        widths.iter().sum::<f32>() + gaps + STRIP_PAD * 2.0
    }

    /// How tall it is.
    ///
    /// The item is what a finger presses, so the minimum target applies to the item and the strip's own
    /// padding is added on top of it.
    fn height(&self, rux: &Rux<'_>) -> f32 {
        rux.screen().target_height(MARK + ITEM_PAD_Y * 2.0) + STRIP_PAD * 2.0
    }

    /// The widest fit that gets the strip inside `room`, or the narrowest one there is.
    ///
    /// **Both steps walk back a rule the reference itself wrote**, so a narrow nav is a state the design
    /// already has rather than a second drawing: `.topnav__item span { display: none }` is the idle
    /// item, which is what dropping the active label reaches.
    pub fn fit_for(&self, rux: &Rux<'_>, room: f32) -> Fit {
        for fit in [Fit::Full, Fit::IconsOnly, Fit::Trimmed] {
            if self.width(rux, fit) <= room {
                return fit;
            }
        }
        Fit::Trimmed
    }

    /// The room it wants with the label it would like to show.
    pub fn measure(&self, rux: &Rux<'_>) -> Vec2 {
        Vec2::new(self.width(rux, Fit::Full), self.height(rux))
    }

    /// The least room it can be drawn in with nothing overlapping.
    pub fn measure_min(&self, rux: &Rux<'_>) -> Vec2 {
        Vec2::new(self.width(rux, Fit::Trimmed), self.height(rux))
    }

    pub fn show(self, rux: &mut Rux<'_>, rect: Rect) -> TopNavOutcome {
        let theme = rux.theme();
        // `.topnav { border-radius: var(--r-pill); box-shadow: var(--e-raised) }`.
        rux.chrome.surface(rect, theme.radius.pill, theme.surface.s1, theme.elevation.raised);

        let fit = self.fit_for(rux, rect.width());
        let widths = self.widths(rux, fit);
        let mut line = Flex::row(rect)
            .pad(Pad::all(STRIP_PAD))
            .gap(STRIP_GAP)
            .cross(Cross::Stretch);
        for width in &widths {
            line = line.hug(*width);
        }
        let cells = line.solve_all();

        let mut outcome = TopNavOutcome::default();
        for (index, (item, cell)) in self.items.iter().zip(cells).enumerate() {
            let response = interact(rux, cell, "rux-nav-item", Sense::click());
            pointing(rux, &response, true);
            if response.clicked() {
                outcome.chosen = Some(index);
            }
            let active = index == self.active;
            let hovered = super::hovering(rux, &response);
            self.draw_item(rux, cell, *item, active, hovered, fit);
            response.widget_info(|| {
                egui::WidgetInfo::selected(egui::WidgetType::SelectableLabel, true, active, item.label)
            });
        }
        outcome
    }

    fn draw_item(
        &self,
        rux: &mut Rux<'_>,
        rect: Rect,
        item: NavItem,
        active: bool,
        hovered: bool,
        fit: Fit,
    ) {
        let theme = rux.theme();
        let ink = match (active, hovered) {
            // `.is-active { color: #fff }`.
            (true, _) => Color32::WHITE,
            // `.topnav__item:hover { color: var(--ink-900) }`.
            (false, true) => theme.ink.i900,
            (false, false) => theme.ink.i500,
        };
        if active {
            rux.chrome.surface(
                rect,
                theme.radius.pill,
                Fill::gradient(theme.gradient.nav_active, rect),
                theme.elevation.nav_active,
            );
            if !matches!(fit, Fit::Full) {
                // The chosen item keeps its own pill and its own gradient; what it gives up when there
                // is no room is the label, which is the idle item's own rule.
                rux.mark_in(Mark::new(item.icon, MARK), rect, ink);
                return;
            }
            let words = rux.measure(Style::CONTROL, item.label);
            let [mark, label] = Flex::row(rect)
                .pad(Pad::axes(ITEM_PAD_Y, ACTIVE_PAD_X))
                .gap(ITEM_GAP)
                .cross(Cross::Centre)
                .sized(Vec2::splat(MARK))
                .hug(words.x)
                .solve();
            rux.mark_in(Mark::new(item.icon, MARK), mark, ink);
            let galley = rux.text(Style::CONTROL, item.label, ink);
            crate::text::draw_left_centre(rux.ui.painter(), label.left_center(), galley, ink);
        } else {
            // No label at all: `.topnav__item span { display: none }`.
            rux.mark_in(Mark::new(item.icon, MARK), rect, ink);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_measurements_are_the_stylesheet_s() {
        // `.topnav { gap: 4px; padding: 6px }`, `.topnav__item { padding: 9px 14px }`,
        // `.is-active { padding: 9px 18px }`, `<SbIcon size={20}/>`, `gap: 8px`.
        assert_eq!((STRIP_PAD, STRIP_GAP), (6.0, 4.0));
        assert_eq!((ITEM_PAD_Y, ITEM_PAD_X, ACTIVE_PAD_X), (9.0, 14.0, 18.0));
        assert_eq!((MARK, ITEM_GAP), (20.0, 8.0));
    }

    #[test]
    fn a_trimmed_item_is_as_wide_as_the_reference_s_own_round_button() {
        // `.toolbtn { width: 40px }` is the smallest comfortable round control in the reference, and a
        // trimmed nav item comes to exactly that.
        assert_eq!(MARK + TRIM_PAD_X * 2.0, 40.0);
    }
}
