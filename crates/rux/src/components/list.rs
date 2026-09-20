//! A row in a list or a menu.
//!
//! `.proj__item` and `.ic-modal__list-item` are the same row: a label, an optional leading dot, an
//! optional trailing action, and three states. What differs is that a chosen `.proj__item` takes the
//! accent while a chosen `.ic-modal__list-item` is pressed into the surface — which is the difference
//! between *this is selected* and *this is what you are editing*, and is a variant rather than two
//! components.

use egui::{Rect, Sense, Vec2};

use super::{interact, pointing};
use crate::icon::{Icon, Mark};
use crate::layout::{Cross, Flex, Pad};
use crate::text::Style;
use crate::Rux;

/// What happened this frame.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ListItemOutcome {
    pub clicked: bool,
    /// The trailing action was pressed rather than the row itself.
    pub action: bool,
}

/// One row.
pub struct ListItem<'a> {
    label: &'a str,
    selected: bool,
    /// `.proj__item-dot` — a 6px disc that takes the accent and a halo when the row is the chosen one.
    dot: bool,
    /// A mark at the right hand end that does something of its own: `.ic-modal__apply`.
    action: Option<(Icon, &'a str)>,
    /// `.proj__item.is-selected svg` — a tick at the right hand end of the chosen row.
    tick: bool,
    /// `.ic-modal__list-item.is-active { box-shadow: var(--e-pressed-sm) }` rather than the accent.
    pressed_when_selected: bool,
    style: Style,
    pad: Pad,
}

impl<'a> ListItem<'a> {
    /// `.proj__item { padding: 10px 14px; border-radius: var(--r-md); font-size: 13px }`.
    pub fn new(label: &'a str, selected: bool) -> Self {
        Self {
            label,
            selected,
            dot: false,
            action: None,
            tick: false,
            pressed_when_selected: false,
            style: Style::sans(13.0),
            pad: Pad::axes(10.0, 14.0),
        }
    }

    pub fn dot(mut self, dot: bool) -> Self {
        self.dot = dot;
        self
    }

    pub fn tick(mut self, tick: bool) -> Self {
        self.tick = tick;
        self
    }

    pub fn action(mut self, icon: Icon, label: &'a str) -> Self {
        self.action = Some((icon, label));
        self
    }

    /// `.ic-modal__list-item.is-active` — pressed into the surface rather than accented.
    pub fn pressed_when_selected(mut self) -> Self {
        self.pressed_when_selected = true;
        self
    }

    pub fn pad(mut self, pad: Pad) -> Self {
        self.pad = pad;
        self
    }

    pub fn measure(&self, rux: &Rux<'_>) -> Vec2 {
        let line = rux.measure(self.style, "Ag").y;
        Vec2::new(
            rux.measure(self.style, self.label).x + self.pad.horizontal() + 40.0,
            line + self.pad.vertical(),
        )
    }

    pub fn show(self, rux: &mut Rux<'_>, rect: Rect) -> ListItemOutcome {
        let response = interact(rux, rect, "rux-list-item", Sense::click());
        pointing(rux, &response, true);
        let hovered = response.hovered();
        let theme = rux.theme();
        let radius = theme.radius.md;

        if self.selected && self.pressed_when_selected {
            rux.chrome.surface(rect, radius, theme.surface.s1, theme.elevation.pressed_sm);
        } else if hovered {
            // `.proj__item:hover { background: rgba(0,0,0,0.025) }`, drawn with `egui` so moving the
            // pointer down a menu rasterises nothing.
            rux.wash(rect, radius, theme.interaction.hover);
        }

        let ink = match (self.selected, self.pressed_when_selected, hovered) {
            (true, false, _) => theme.accent.blue,
            (true, true, _) => theme.ink.i900,
            (false, _, true) => theme.ink.i900,
            (false, _, false) => theme.ink.i700,
        };
        let style = match self.selected {
            true => self.style.medium(),
            false => self.style,
        };

        let mut line = Flex::row(rect).pad(self.pad).gap(10.0).cross(Cross::Centre);
        if self.dot {
            line = line.sized(Vec2::splat(6.0));
        }
        line = line.fill(1.0);
        let trailing = self.tick || self.action.is_some();
        if trailing {
            line = line.sized(Vec2::splat(14.0));
        }
        let cells = line.solve_all();
        let mut cell = cells.iter();

        if self.dot {
            if let Some(at) = cell.next() {
                let colour = match self.selected {
                    true => theme.accent.blue,
                    false => theme.ink.i300,
                };
                if self.selected {
                    // `box-shadow: 0 0 8px var(--accent-blue)` on the chosen row's dot.
                    rux.chrome.glow(*at, 3.0, colour, 8.0);
                }
                rux.chrome.disc(at.center(), 3.0, colour);
            }
        }
        if let Some(at) = cell.next() {
            let galley = rux.elided(style, self.label, ink, at.width());
            crate::text::draw_left_centre(rux.ui.painter(), at.left_center(), galley, ink);
        }

        let mut outcome = ListItemOutcome { clicked: response.clicked(), action: false };
        if let Some(at) = cell.next() {
            match self.action {
                Some((icon, label)) => {
                    // `.ic-modal__apply { opacity: 0 }` until the row is hovered or active — an action
                    // that is always visible on every row is a list of buttons rather than a list.
                    if hovered || self.selected {
                        let hit = interact(rux, at.expand(4.0), "rux-list-action", Sense::click());
                        pointing(rux, &hit, true);
                        let tint = match hit.hovered() {
                            true => theme.accent.mint,
                            false => crate::theme::with_opacity(theme.accent.mint, 0.75),
                        };
                        rux.mark_in(Mark::new(icon, 12.0).stroke(2.4), *at, tint);
                        outcome.action = hit.clicked();
                        hit.widget_info(|| {
                            egui::WidgetInfo::labeled(egui::WidgetType::Button, true, label)
                        });
                    }
                }
                None if self.selected => {
                    rux.mark_in(Mark::new(Icon::Check, 14.0), *at, theme.accent.blue);
                }
                None => {}
            }
        }

        response.widget_info(|| {
            egui::WidgetInfo::selected(
                egui::WidgetType::SelectableLabel,
                true,
                self.selected,
                self.label,
            )
        });
        outcome
    }
}

/// A heading over a group of rows: `.proj__menu-head`, mono 9px uppercase at `0.14em`.
pub fn menu_heading(rux: &mut Rux<'_>, rect: Rect, text: &str) {
    let theme = rux.theme();
    let ink = theme.ink.i400;
    let galley = rux.text(Style::mono(9.0).tracking(0.14).upper(), text, ink);
    let inner = Pad::sides(10.0, 14.0, 8.0, 14.0).inside(rect);
    rux.ui.painter().galley(inner.left_top(), galley, ink);
}
