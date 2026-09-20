//! The project picker at the top left of the page.
//!
//! `.proj`, `.proj__trigger` and `.proj__menu`. It is not a [`super::Select`] with a bigger trigger:
//! the trigger has a gradient icon chip, a two-line body with a kicker over a name, and a minimum
//! width, and the menu has a heading and a footer action. Those are enough differences that folding
//! them into one component would be a component with six flags.

use egui::{Rect, Vec2};

use super::{interact, pointing};
use crate::icon::{Icon, Mark};
use crate::layout::{Cross, Flex, Pad};
use crate::text::Style;
use crate::{Fill, Rux};

/// What happened this frame.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ProjectMenuOutcome {
    pub chosen: Option<usize>,
    /// `.proj__new` — the action under the list.
    pub create: bool,
    pub toggled: bool,
}

/// `.proj__trigger { min-width: 240px; padding: 8px 14px 8px 8px; gap: 12px }`.
const TRIGGER_PAD: Pad = Pad::sides(8.0, 14.0, 8.0, 8.0);
const MIN_WIDTH: f32 = 240.0;
/// `.proj__icon { width: 36px; height: 36px }`.
const CHIP: f32 = 36.0;
/// `.proj__menu { top: calc(100% + 10px); min-width: 280px; padding: 8px }`.
const MENU_OFFSET: f32 = 10.0;
const MENU_MIN_WIDTH: f32 = 280.0;
const MENU_PAD: f32 = 8.0;

/// The project picker.
pub struct ProjectMenu<'a> {
    projects: &'a [String],
    selected: usize,
    /// `.proj__kicker` — the word over the name. The reference's is "Project".
    kicker: &'a str,
    /// `.proj__menu-head`.
    heading: &'a str,
    /// `.proj__new`.
    create_label: &'a str,
}

impl<'a> ProjectMenu<'a> {
    pub fn new(projects: &'a [String], selected: usize) -> Self {
        Self {
            projects,
            selected,
            kicker: "Project",
            heading: "Recent projects",
            create_label: "New project",
        }
    }

    pub fn kicker(mut self, kicker: &'a str) -> Self {
        self.kicker = kicker;
        self
    }

    pub fn heading(mut self, heading: &'a str) -> Self {
        self.heading = heading;
        self
    }

    pub fn create_label(mut self, label: &'a str) -> Self {
        self.create_label = label;
        self
    }

    pub fn measure(&self, _rux: &Rux<'_>) -> Vec2 {
        Vec2::new(MIN_WIDTH, CHIP + TRIGGER_PAD.vertical())
    }

    pub fn show(
        self,
        rux: &mut Rux<'_>,
        rect: Rect,
        open: &mut bool,
    ) -> ProjectMenuOutcome {
        let rect = Rect::from_min_size(
            rect.min,
            Vec2::new(rect.width().max(MIN_WIDTH), rect.height()),
        );
        let response = interact(rux, rect, "rux-project-trigger", egui::Sense::click());
        pointing(rux, &response, true);
        let mut outcome = ProjectMenuOutcome::default();
        if response.clicked() {
            *open = !*open;
            outcome.toggled = true;
        }

        let theme = rux.theme();
        // `.proj.is-open .proj__trigger { box-shadow: var(--e-pressed-sm) }`.
        let elevation = match *open {
            true => theme.elevation.pressed_sm,
            false => theme.elevation.raised_sm,
        };
        rux.chrome.surface(rect, theme.radius.lg, theme.surface.s1, elevation);

        let [chip, body, chev] = Flex::row(rect)
            .pad(TRIGGER_PAD)
            .gap(12.0)
            .cross(Cross::Centre)
            .sized(Vec2::splat(CHIP))
            .fill(1.0)
            .sized(Vec2::splat(15.0))
            .solve();

        // `.proj__icon { border-radius: var(--r-md); background: linear-gradient(160deg, …);
        //   box-shadow: inset 1px 1px 2px rgba(255,255,255,0.2) }`.
        rux.chrome.surface(
            chip,
            theme.radius.md,
            Fill::gradient(theme.gradient.primary_diagonal, chip),
            crate::theme::Elevation::of([crate::theme::Shadow::inset(
                1.0,
                1.0,
                2.0,
                crate::theme::rgba(0xFFFFFF, 0.2),
            )]),
        );
        rux.mark_in(Mark::new(Icon::Folder, 16.0), chip, egui::Color32::WHITE);

        // `.proj__kicker` over `.proj__name`, which is `align-items: flex-start` in a column.
        let kicker_style = Style::mono(9.0).tracking(0.14).upper();
        let name_style = Style::sans(13.0).semibold().tracking(-0.01);
        let kicker_size = rux.measure(kicker_style, self.kicker);
        let name_size = rux.measure(name_style, "Ag");
        let stack = kicker_size.y + name_size.y;
        let top = (body.center().y - stack / 2.0).round();
        let dim = theme.ink.i400;
        let galley = rux.text(kicker_style, self.kicker, dim);
        rux.ui.painter().galley(egui::pos2(body.left(), top), galley, dim);
        let name = self.projects.get(self.selected).map(String::as_str).unwrap_or("—");
        let ink = theme.ink.i900;
        // `.proj__name { max-width: 160px; text-overflow: ellipsis }`.
        let galley = rux.elided(name_style, name, ink, body.width().min(160.0));
        rux.ui.painter().galley(egui::pos2(body.left(), top + kicker_size.y), galley, ink);

        let turn = match *open {
            true => std::f32::consts::PI,
            false => 0.0,
        };
        rux.mark_in(Mark::new(Icon::ChevDown, 15.0).turn(turn), chev, theme.ink.i500);

        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::ComboBox, true, name)
        });

        if *open {
            self.show_menu(rux, rect, open, &response, &mut outcome);
        }
        outcome
    }

    fn show_menu(
        &self,
        rux: &mut Rux<'_>,
        trigger: Rect,
        open: &mut bool,
        trigger_response: &egui::Response,
        outcome: &mut ProjectMenuOutcome,
    ) {
        let head_height = 28.0;
        let row = 36.0;
        let foot_height = 1.0 + 4.0 + 36.0 + 4.0;
        let height = MENU_PAD * 2.0
            + head_height
            + row * self.projects.len() as f32
            + foot_height;
        let width = trigger.width().max(MENU_MIN_WIDTH);
        let area = Rect::from_min_size(
            egui::pos2(trigger.left(), trigger.bottom() + MENU_OFFSET),
            Vec2::new(width, height),
        );
        let id = rux.ui.id().with("rux-project-menu");
        let projects = self.projects;
        let selected = self.selected;
        let heading = self.heading;
        let create_label = self.create_label;

        let (chosen, create, inside) =
            rux.overlay(id, area, egui::Order::Foreground, |rux| {
                let theme = rux.theme();
                rux.chrome.surface(
                    area,
                    theme.radius.lg,
                    theme.surface.s1,
                    theme.elevation.raised_lg,
                );
                let inner = Pad::all(MENU_PAD).inside(area);
                let mut pen = crate::layout::Pen::new(inner);
                super::list::menu_heading(rux, pen.strip(head_height), heading);

                let mut chosen = None;
                for (index, project) in projects.iter().enumerate() {
                    let rect = pen.strip(row);
                    let hit = super::ListItem::new(project, index == selected)
                        .dot(true)
                        .show(rux, rect);
                    if hit.clicked {
                        chosen = Some(index);
                    }
                }

                // `.proj__menu-foot { margin-top: 4px; border-top: 1px solid rgba(0,0,0,0.04) }`.
                pen.gap(4.0);
                let rule = pen.strip(1.0);
                rux.chrome.line(rule.left_top(), rule.right_top(), 1.0, theme.interaction.hairline);
                pen.gap(4.0);
                let create_rect = pen.strip(36.0);
                let create = super::Button::new(create_label)
                    .variant(super::ButtonVariant::Ghost)
                    .icon(Icon::Plus)
                    .size(super::ButtonSize::Small)
                    .show(rux, create_rect)
                    .clicked();
                // `.proj__new { color: var(--accent-blue) }` — the one ghost button in the design that
                // is drawn in the accent rather than in ink, so it is painted over the ghost's own.
                let ink = theme.accent.blue;
                let style = Style::sans(13.0).medium();
                let words = rux.measure(style, create_label);
                let [mark, label] = Flex::row(create_rect)
                    .pad(Pad::axes(10.0, 14.0))
                    .gap(8.0)
                    .cross(Cross::Centre)
                    .sized(Vec2::splat(14.0))
                    .hug(words.x)
                    .solve();
                rux.mark_in(Mark::new(Icon::Plus, 14.0), mark, ink);
                let galley = rux.text(style, create_label, ink);
                crate::text::draw_left_centre(rux.ui.painter(), label.left_center(), galley, ink);

                let inside = rux
                    .ui
                    .ctx()
                    .pointer_latest_pos()
                    .is_some_and(|at| area.contains(at));
                (chosen, create, inside)
            });

        outcome.chosen = chosen;
        outcome.create = create;
        if chosen.is_some() || create {
            *open = false;
        } else if rux.ui.ctx().input(|input| input.pointer.any_pressed())
            && !inside
            && !trigger_response.hovered()
        {
            *open = false;
        }
    }
}
