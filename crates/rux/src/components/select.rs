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
/// The handle across the top of a sheet, and the room it takes. The same 36 by 4 as a dialog's.
const SHEET_GRABBER: Vec2 = Vec2::new(36.0, 4.0);
const SHEET_GRAB_ROOM: f32 = 20.0;

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
    /// How much larger than the stylesheet's sizes it is drawn: its words, its padding, its chevron and
    /// its menu. A consumer whose pane zooms by a multiplier, rather than through a layer's transform,
    /// passes the multiplier here so the select grows with what is round it.
    zoom: f32,
    /// The menu's shadow, when the consumer wants a quieter one than `--e-raised-lg`.
    menu_elevation: Option<crate::Elevation>,
    /// Drawn as the Agent-Tasks design's chip, with a mark before the value. See [`Select::chip`].
    chip: Option<SelectLead>,
}

/// The mark before a chip select's value: the ticket modal's status, assignee, model, project and epic.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SelectLead {
    /// A mark from the set, in a colour.
    Mark(Icon, egui::Color32),
    /// A lane's own mark, which is what a status is drawn with.
    Lane(super::LaneMark),
    /// No mark: the value starts where a mark would.
    None,
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
            zoom: 1.0,
            menu_elevation: None,
            chip: None,
        }
    }

    /// The Agent-Tasks design's field: a pill 38 points tall on the chip surface, `padding: 0 14px 0
    /// 12px; gap: 10px`, an 18 point mark, the value in 12.5 point words and a 10 point chevron. `task-2241`.
    pub fn chip(mut self, lead: SelectLead) -> Self {
        self.chip = Some(lead);
        self
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

    /// Draw it this many times the stylesheet's size. See the field.
    pub fn zoom(mut self, zoom: f32) -> Self {
        self.zoom = zoom.max(0.1);
        self
    }

    /// Give the menu this shadow instead of `--e-raised-lg`, for a consumer whose surfaces sit closer to
    /// the page than the reference's do.
    pub fn menu_elevation(mut self, elevation: crate::Elevation) -> Self {
        self.menu_elevation = Some(elevation);
        self
    }

    /// What a screen reader calls this control.
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = label;
        self
    }

    fn style(&self) -> Style {
        if self.chip.is_some() {
            return Style::sans(12.5 * self.zoom);
        }
        let style = match self.mono {
            // `.ic-select--mono .ic-select__trigger { font-size: 11px; letter-spacing: 0.01em }`.
            true => Style::mono(11.0).medium().tracking(0.01),
            false => Style::CONTROL,
        };
        style.at(style.size * self.zoom)
    }

    /// `padding: 9px 12px` plus the tallest thing in the row.
    pub fn measure(&self, rux: &Rux<'_>) -> Vec2 {
        let value = self.value();
        let words = rux.measure(self.style(), value);
        let z = self.zoom;
        Vec2::new(
            words.x + (12.0 * 2.0 + 8.0 + 13.0) * z,
            rux.screen().target_height((words.y + 9.0 * 2.0 * z).max(34.0 * z)),
        )
    }

    fn value(&self) -> &str {
        self.selected
            .and_then(|index| self.options.get(index))
            .map(String::as_str)
            .unwrap_or(self.placeholder)
    }

    pub fn show(self, rux: &mut Rux<'_>, rect: Rect, state: &mut SelectState) -> SelectOutcome {
        let response = interact(rux, rect, "rux-select", Sense::click());
        let visual = Visual::of(&response, self.enabled, rux.screen().pointer);
        pointing(rux, &response, self.enabled);
        let mut outcome = SelectOutcome::default();
        if response.clicked() && self.enabled {
            state.toggle();
            outcome.toggled = true;
        }

        if let Some(lead) = self.chip {
            self.show_chip(rux, rect, state, &response, visual, lead);
            if state.open {
                outcome.chosen = self.show_menu(rux, rect, state, &response);
            }
            return outcome;
        }

        let theme = rux.theme();
        // `:hover { box-shadow: var(--e-raised) }` and `.is-open { box-shadow: var(--e-pressed-sm) }`.
        let elevation = match (state.open, visual) {
            (true, _) => theme.elevation.pressed_sm,
            (false, Visual::Hover | Visual::Active) => theme.elevation.raised,
            _ => theme.elevation.raised_sm,
        };
        let z = self.zoom;
        rux.chrome.surface(rect, theme.radius.md * z, theme.surface.s1, elevation);

        let [value, chev] = Flex::row(rect)
            .pad(Pad::axes(9.0 * z, 12.0 * z))
            .gap(8.0 * z)
            .cross(Cross::Centre)
            .fill(1.0)
            .sized(Vec2::splat(13.0 * z))
            .solve();

        let ink = match self.selected.is_some() {
            true => theme.ink.i900,
            // `.ic-select__value--empty { color: var(--ink-400) }`.
            false => theme.ink.i400,
        }
        .gamma_multiply(visual.fade());
        let galley = rux.elided(self.style(), self.value(), ink, value.width());
        // **By its capitals**, so the words and the chevron beside them share a middle whatever face the
        // consumer bound to the family. See [`crate::text::draw_left_capitals`].
        crate::text::draw_left_capitals(
            rux.ui.painter(),
            egui::pos2(value.left(), chev.center().y),
            galley,
            self.style(),
            ink,
        );

        // `.ic-select.is-open .ic-select__chev { transform: rotate(180deg) }`.
        let turn = match state.open {
            true => std::f32::consts::PI,
            false => 0.0,
        };
        rux.mark(
            Mark::new(Icon::ChevDown, 13.0 * z).stroke(2.0 * z).turn(turn),
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

    /// The chip trigger: carved in while its menu is open, sky words under the pointer.
    fn show_chip(
        &self,
        rux: &mut Rux<'_>,
        rect: Rect,
        state: &SelectState,
        response: &Response,
        visual: Visual,
        lead: SelectLead,
    ) {
        let z = self.zoom;
        let chat = rux.theme().chat;
        let theme = rux.theme();
        let height = (38.0 * z).min(rect.height());
        let rect = Rect::from_center_size(rect.center(), Vec2::new(rect.width(), height));
        let radius = height / 2.0;
        match state.open {
            true => rux.chrome.surface(rect, radius, chat.well, chat.carved_sm()),
            false => rux.chrome.surface(rect, radius, super::chip(&chat, rect), chat.raised_chip()),
        }
        let fade = visual.fade();
        let mark = egui::pos2(rect.left() + 12.0 * z + 9.0 * z, rect.center().y);
        let value_left = match lead {
            SelectLead::Mark(icon, colour) => {
                rux.mark(Mark::new(icon, 15.0 * z), mark, colour.gamma_multiply(fade));
                rect.left() + (12.0 + 18.0 + 10.0) * z
            }
            SelectLead::Lane(lane) => {
                lane.paint(rux, mark, z);
                rect.left() + (12.0 + 18.0 + 10.0) * z
            }
            SelectLead::None => rect.left() + 14.0 * z,
        };
        let chev = egui::pos2(rect.right() - 14.0 * z - 5.0 * z, rect.center().y);
        let ink = match (self.selected.is_some(), response.hovered() && rux.screen().pointer.hovers()) {
            (_, true) => chat.sky,
            (true, false) => theme.ink.i900,
            (false, false) => theme.ink.i400,
        }
        .gamma_multiply(fade);
        let room = (chev.x - 5.0 * z - 10.0 * z - value_left).max(0.0);
        let galley = rux.elided(self.style(), self.value(), ink, room);
        crate::text::draw_left_capitals(
            rux.ui.painter(),
            egui::pos2(value_left, rect.center().y),
            galley,
            self.style(),
            ink,
        );
        let turn = match state.open {
            true => std::f32::consts::PI,
            false => 0.0,
        };
        rux.mark(
            Mark::new(Icon::ChevDown, 10.0 * z).stroke(1.6 * z).turn(turn),
            chev,
            theme.ink.i400.gamma_multiply(fade),
        );
        response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::ComboBox, self.enabled, self.label));
    }

    /// `.ic-select__menu` — the list that opens under the trigger.
    fn show_menu(
        &self,
        rux: &mut Rux<'_>,
        trigger: Rect,
        state: &mut SelectState,
        trigger_response: &Response,
    ) -> Option<usize> {
        let z = self.zoom;
        let row = self.row_height(rux);
        let content = row * self.options.len() as f32;
        let screen_rect = rux.ui.ctx().content_rect();

        // On a compact screen the menu is a sheet from the bottom edge rather than a list anchored under
        // its trigger — the same shape `Modal` takes there, drawn by the same function, because a menu
        // and a dialog should be one idea on a phone. A 200 point list anchored to a control halfway
        // down a 390 point screen has nowhere to go and flips about as the page scrolls.
        let sheet = rux.screen().is_compact();
        let inset = match sheet {
            true => rux.screen().insets.bottom,
            false => 0.0,
        };
        let menu_pad = MENU_PAD * z;
        let pad_top = match sheet {
            true => SHEET_GRAB_ROOM,
            false => menu_pad,
        };
        let height = match sheet {
            true => (content + pad_top + menu_pad + inset).min(screen_rect.height() * 0.6),
            false => (content + menu_pad * 2.0).min(SELECT_MENU_MAX_HEIGHT * z),
        };

        // `.ic-select--up` when it is asked for, and also when there is no room below — a menu drawn
        // off the bottom of the window is a menu nobody can reach.
        let below = trigger.bottom() + MENU_OFFSET * z;
        let upwards = !sheet && (self.up || below + height > screen_rect.bottom());
        let area = match sheet {
            true => super::sheet_frame(screen_rect, height, false),
            false => {
                let top = match upwards {
                    true => trigger.top() - MENU_OFFSET * z - height,
                    false => below,
                };
                Rect::from_min_size(
                    egui::pos2(trigger.left(), top),
                    Vec2::new(trigger.width(), height),
                )
            }
        };

        let id = rux.ui.id().with(("rux-select-menu", trigger.min.x.to_bits(), trigger.min.y.to_bits()));
        let options = self.options;
        let selected = self.selected;
        let style = self.style();
        let scroll = &mut state.scroll;
        let max_scroll = (content + pad_top + menu_pad + inset - height).max(0.0);
        let menu_elevation = self.menu_elevation;

        let (chosen, pointer_inside) = rux.overlay(id, area, egui::Order::Foreground, |rux| {
            let theme = rux.theme();
            // A sheet is square along the edge it came from. See [`super::Present`].
            let corners = match sheet {
                true => crate::Corners::top(theme.radius.xl),
                false => crate::Corners::all(theme.radius.md * z),
            };
            let elevation = menu_elevation.unwrap_or(theme.elevation.raised_lg);
            rux.chrome.surface(area, corners, theme.surface.s1, elevation);
            if sheet {
                let grab = Rect::from_center_size(
                    egui::pos2(area.center().x, area.top() + 10.0),
                    SHEET_GRABBER,
                );
                rux.chrome.rect(grab, SHEET_GRABBER.y / 2.0, theme.surface.sunken);
            }

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
            rux.chrome.clip(area, corners);
            let was = rux.ui.clip_rect();
            rux.ui.set_clip_rect(area.intersect(was));

            let mut chosen = None;
            for (index, option) in options.iter().enumerate() {
                let rect = Rect::from_min_size(
                    egui::pos2(area.left() + menu_pad, area.top() + pad_top + index as f32 * row - *scroll),
                    Vec2::new(area.width() - menu_pad * 2.0, row),
                );
                if rect.bottom() < area.top() || rect.top() > area.bottom() {
                    continue;
                }
                let response = interact(rux, rect, "rux-select-item", Sense::click());
                pointing(rux, &response, true);
                let is_selected = selected == Some(index);
                let hovered = super::hovering(rux, &response);
                // `.ic-select__item.is-selected { background: rgba(47,107,255,0.04) }` and
                // `:hover { background: rgba(47,107,255,0.06) }`.
                if hovered {
                    rux.wash(rect, theme.radius.sm * z, theme.interaction.accent_wash);
                } else if is_selected {
                    rux.wash(rect, theme.radius.sm * z, theme.interaction.accent_wash_soft);
                }
                let ink = match is_selected || hovered {
                    true => theme.accent.blue,
                    false => theme.ink.i700,
                };
                let row_style = match is_selected {
                    true => style.medium(),
                    false => style,
                };
                let text_rect = Pad::axes(8.0 * z, 12.0 * z).inside(rect);
                let galley = rux.elided(row_style, option, ink, text_rect.width() - 18.0 * z);
                crate::text::draw_left_capitals(
                    rux.ui.painter(),
                    text_rect.left_center(),
                    galley,
                    row_style,
                    ink,
                );
                if is_selected {
                    rux.mark(
                        Mark::new(Icon::Check, 12.0 * z),
                        egui::pos2(text_rect.right() - 6.0 * z, rect.center().y),
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
        rux.screen().target_height(rux.measure(self.style(), "Ag").y + 16.0 * self.zoom)
    }
}
