//! A dropdown: a trigger that says what is chosen, and a menu that opens under it.
//!
//! `.ic-select`, `.ic-select__trigger`, `.ic-select__menu` and `.ic-select__item`, plus the
//! `.ic-select--up` variant that opens upwards when there is no room below.
//!
//! **The menu is its own `egui` layer**, because `egui` draws foreground areas above everything and
//! because the decoration behind a menu has to be filled into that layer's own painter. See
//! [`crate::Rux::overlay`].

use egui::{Rect, Response, Sense, Vec2};

use super::{interact, pointing, Visual};
use crate::icon::{Icon, Mark};
use crate::layout::{Cross, Flex, Pad};
use crate::text::Style;
use crate::Rux;

/// Whether the menu is open, and where it is scrolled to. The caller owns it, which is what lets two
/// selects appear on one screen.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SelectState {
    pub open: bool,
    /// How far the menu is scrolled, for a list longer than `max-height: 280px`.
    pub scroll: f32,
}

impl SelectState {
    pub fn toggle(&mut self) {
        self.open = !self.open;
        self.scroll = 0.0;
    }
}

/// What happened this frame.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SelectOutcome {
    /// The index of the row that was chosen, if one was.
    pub chosen: Option<usize>,
    /// Whether the trigger was pressed, which is what opens and closes it.
    pub toggled: bool,
}

/// `max-height: 280px` on `.ic-select__menu`.
pub const SELECT_MENU_MAX_HEIGHT: f32 = 280.0;
/// `top: calc(100% + 6px)`.
const MENU_OFFSET: f32 = 6.0;
/// `padding: 6px` on the menu, `padding: 8px 12px` on a row.
const MENU_PAD: f32 = 6.0;

/// A dropdown.
pub struct Select<'a> {
    options: &'a [String],
    selected: Option<usize>,
    /// `.ic-select__value--empty` — what is shown with nothing chosen. The reference's is `—`.
    placeholder: &'a str,
    /// `.ic-select--mono` — the dimension picker, which is set in JetBrains Mono at 11px.
    mono: bool,
    /// `.ic-select--up` — open upwards. The reference sets it by hand on the two selects pinned to the
    /// bottom of the prompt panel; here it is also chosen automatically when there is no room below.
    up: bool,
    enabled: bool,
    label: &'a str,
}

impl<'a> Select<'a> {
    pub fn new(options: &'a [String], selected: Option<usize>) -> Self {
        Self {
            options,
            selected,
            placeholder: "—",
            mono: false,
            up: false,
            enabled: true,
            label: "Select",
        }
    }

    pub fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = placeholder;
        self
    }

    pub fn mono(mut self, mono: bool) -> Self {
        self.mono = mono;
        self
    }

    /// Force the menu upwards, which is `.ic-select--up`.
    pub fn up(mut self, up: bool) -> Self {
        self.up = up;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// What a screen reader calls this control.
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = label;
        self
    }

    fn style(&self) -> Style {
        match self.mono {
            // `.ic-select--mono .ic-select__trigger { font-size: 11px; letter-spacing: 0.01em }`.
            true => Style::mono(11.0).medium().tracking(0.01),
            false => Style::CONTROL,
        }
    }

    /// `padding: 9px 12px` plus the tallest thing in the row.
    pub fn measure(&self, rux: &Rux<'_>) -> Vec2 {
        let value = self.value();
        let words = rux.measure(self.style(), value);
        Vec2::new(words.x + 12.0 * 2.0 + 8.0 + 13.0, (words.y + 9.0 * 2.0).max(34.0))
    }

    fn value(&self) -> &str {
        self.selected
            .and_then(|index| self.options.get(index))
            .map(String::as_str)
            .unwrap_or(self.placeholder)
    }

    pub fn show(self, rux: &mut Rux<'_>, rect: Rect, state: &mut SelectState) -> SelectOutcome {
        let response = interact(rux, rect, "rux-select", Sense::click());
        let visual = Visual::of(&response, self.enabled);
        pointing(rux, &response, self.enabled);
        let mut outcome = SelectOutcome::default();
        if response.clicked() && self.enabled {
            state.toggle();
            outcome.toggled = true;
        }

        let theme = rux.theme();
        // `:hover { box-shadow: var(--e-raised) }` and `.is-open { box-shadow: var(--e-pressed-sm) }`.
        let elevation = match (state.open, visual) {
            (true, _) => theme.elevation.pressed_sm,
            (false, Visual::Hover | Visual::Active) => theme.elevation.raised,
            _ => theme.elevation.raised_sm,
        };
        rux.chrome.surface(rect, theme.radius.md, theme.surface.s1, elevation);

        let [value, chev] = Flex::row(rect)
            .pad(Pad::axes(9.0, 12.0))
            .gap(8.0)
            .cross(Cross::Centre)
            .fill(1.0)
            .sized(Vec2::splat(13.0))
            .solve();

        let ink = match self.selected.is_some() {
            true => theme.ink.i900,
            // `.ic-select__value--empty { color: var(--ink-400) }`.
            false => theme.ink.i400,
        }
        .gamma_multiply(visual.fade());
        let galley = rux.elided(self.style(), self.value(), ink, value.width());
        crate::text::draw_left_centre(rux.ui.painter(), value.left_center(), galley, ink);

        // `.ic-select.is-open .ic-select__chev { transform: rotate(180deg) }`.
        let turn = match state.open {
            true => std::f32::consts::PI,
            false => 0.0,
        };
        rux.mark(
            Mark::new(Icon::ChevDown, 13.0).stroke(2.0).turn(turn),
            chev.center(),
            theme.ink.i500.gamma_multiply(visual.fade()),
        );

        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::ComboBox, self.enabled, self.label)
        });

        if state.open {
            let chosen = self.show_menu(rux, rect, state, &response);
            outcome.chosen = chosen;
        }
        outcome
    }

    /// `.ic-select__menu` — the list that opens under the trigger.
    fn show_menu(
        &self,
        rux: &mut Rux<'_>,
        trigger: Rect,
        state: &mut SelectState,
        trigger_response: &Response,
    ) -> Option<usize> {
        let row = self.row_height(rux);
        let content = row * self.options.len() as f32;
        let height = (content + MENU_PAD * 2.0).min(SELECT_MENU_MAX_HEIGHT);
        let screen = rux.ui.ctx().content_rect();

        // `.ic-select--up` when it is asked for, and also when there is no room below — a menu drawn
        // off the bottom of the window is a menu nobody can reach.
        let below = trigger.bottom() + MENU_OFFSET;
        let upwards = self.up || below + height > screen.bottom();
        let top = match upwards {
            true => trigger.top() - MENU_OFFSET - height,
            false => below,
        };
        let area = Rect::from_min_size(
            egui::pos2(trigger.left(), top),
            Vec2::new(trigger.width(), height),
        );

        let id = rux.ui.id().with(("rux-select-menu", trigger.min.x.to_bits(), trigger.min.y.to_bits()));
        let options = self.options;
        let selected = self.selected;
        let style = self.style();
        let scroll = &mut state.scroll;
        let max_scroll = (content + MENU_PAD * 2.0 - height).max(0.0);

        let (chosen, pointer_inside) = rux.overlay(id, area, egui::Order::Foreground, |rux| {
            let theme = rux.theme();
            rux.chrome.surface(area, theme.radius.md, theme.surface.s1, theme.elevation.raised_lg);

            let pointer_inside = rux
                .ui
                .ctx()
                .pointer_latest_pos()
                .is_some_and(|at| area.contains(at));
            if pointer_inside && max_scroll > 0.0 {
                let wheel = rux.ui.ctx().input(|input| input.smooth_scroll_delta.y);
                *scroll = (*scroll - wheel).clamp(0.0, max_scroll);
            }

            // Rounded corners are the one clip `egui` cannot do, so the rows are cut by the chrome and
            // by `egui`'s own square clip together: the chrome's for the decoration, `egui`'s for the
            // text.
            rux.chrome.clip(area, theme.radius.md);
            let was = rux.ui.clip_rect();
            rux.ui.set_clip_rect(area.intersect(was));

            let mut chosen = None;
            for (index, option) in options.iter().enumerate() {
                let rect = Rect::from_min_size(
                    egui::pos2(area.left() + MENU_PAD, area.top() + MENU_PAD + index as f32 * row - *scroll),
                    Vec2::new(area.width() - MENU_PAD * 2.0, row),
                );
                if rect.bottom() < area.top() || rect.top() > area.bottom() {
                    continue;
                }
                let response = interact(rux, rect, "rux-select-item", Sense::click());
                pointing(rux, &response, true);
                let is_selected = selected == Some(index);
                let hovered = response.hovered();
                // `.ic-select__item.is-selected { background: rgba(47,107,255,0.04) }` and
                // `:hover { background: rgba(47,107,255,0.06) }`.
                if hovered {
                    rux.wash(rect, theme.radius.sm, theme.interaction.accent_wash);
                } else if is_selected {
                    rux.wash(rect, theme.radius.sm, theme.interaction.accent_wash_soft);
                }
                let ink = match is_selected || hovered {
                    true => theme.accent.blue,
                    false => theme.ink.i700,
                };
                let row_style = match is_selected {
                    true => style.medium(),
                    false => style,
                };
                let text_rect = Pad::axes(8.0, 12.0).inside(rect);
                let galley = rux.elided(row_style, option, ink, text_rect.width() - 18.0);
                crate::text::draw_left_centre(
                    rux.ui.painter(),
                    text_rect.left_center(),
                    galley,
                    ink,
                );
                if is_selected {
                    rux.mark(
                        Mark::new(Icon::Check, 12.0),
                        egui::pos2(text_rect.right() - 6.0, rect.center().y),
                        theme.accent.blue,
                    );
                }
                if response.clicked() {
                    chosen = Some(index);
                }
            }
            rux.ui.set_clip_rect(was);
            rux.chrome.unclip();
            (chosen, pointer_inside)
        });

        if chosen.is_some() {
            state.open = false;
        } else {
            // A press anywhere else closes it, which is what the reference's `mousedown` listener on
            // the document does. The trigger is excluded because its own click already toggled.
            let clicked_away = rux.ui.ctx().input(|input| input.pointer.any_pressed())
                && !pointer_inside
                && !trigger_response.hovered();
            if clicked_away {
                state.open = false;
            }
        }
        chosen
    }

    /// `.ic-select__item { padding: 8px 12px }` plus the line the words take.
    fn row_height(&self, rux: &Rux<'_>) -> f32 {
        rux.measure(self.style(), "Ag").y + 16.0
    }
}
