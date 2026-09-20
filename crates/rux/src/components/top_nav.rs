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
    fn widths(&self, rux: &Rux<'_>) -> Vec<f32> {
        self.items
            .iter()
            .enumerate()
            .map(|(index, item)| match index == self.active {
                true => {
                    let words = rux.measure(Style::CONTROL, item.label);
                    MARK + ITEM_GAP + words.x + ACTIVE_PAD_X * 2.0
                }
                false => MARK + ITEM_PAD_X * 2.0,
            })
            .collect()
    }

    pub fn measure(&self, rux: &Rux<'_>) -> Vec2 {
        let widths = self.widths(rux);
        let gaps = STRIP_GAP * widths.len().saturating_sub(1) as f32;
        Vec2::new(
            widths.iter().sum::<f32>() + gaps + STRIP_PAD * 2.0,
            MARK + ITEM_PAD_Y * 2.0 + STRIP_PAD * 2.0,
        )
    }

    pub fn show(self, rux: &mut Rux<'_>, rect: Rect) -> TopNavOutcome {
        let theme = rux.theme();
        // `.topnav { border-radius: var(--r-pill); box-shadow: var(--e-raised) }`.
        rux.chrome.surface(rect, theme.radius.pill, theme.surface.s1, theme.elevation.raised);

        let widths = self.widths(rux);
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
            self.draw_item(rux, cell, *item, active, response.hovered());
            response.widget_info(|| {
                egui::WidgetInfo::selected(egui::WidgetType::SelectableLabel, true, active, item.label)
            });
        }
        outcome
    }

    fn draw_item(&self, rux: &mut Rux<'_>, rect: Rect, item: NavItem, active: bool, hovered: bool) {
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
}
