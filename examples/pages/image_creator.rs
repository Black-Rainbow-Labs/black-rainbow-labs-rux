//! The AI Studio Image Creator, rebuilt out of the `rux` components.
//!
//! `reference/image-creator.css` and `reference/image-creator-page.jsx.html` are what this is against:
//! a top bar with a project picker, the nav and a save-to chip; two collapsible panels; and a results
//! canvas. Every measurement below is a declaration in that stylesheet, named in the comment beside it
//! so a later reader can check one against the other.
//!
//! The one thing this deliberately does not copy is the CSS transitions. `.ic-panel { transition:
//! width 0.28s }` and the menus' fades are real and worth having; the library draws the end states and
//! §14 of the design says why.

use rux::prelude::*;
use rux::components::{
    ButtonSize, ButtonVariant, ChipVariant, IconButtonSize, PanelKind,
    SelectState, TileKind,
};
use rux::layout::{Justify, Length};
use rux::text::Weight;

use egui::{Rect, Vec2};

/// `.topbar { padding: 16px 28px }` plus the tallest thing in it, which is the project trigger.
const TOPBAR_HEIGHT: f32 = 73.0;
/// `.ic-panel--prompt { width: 300px }` and `.ic-panel--controls { width: 360px }`.
const PROMPT_WIDTH: f32 = 300.0;
const CONTROLS_WIDTH: f32 = 360.0;
/// `.ic-panel__scroll { padding: 16px 16px 10px; gap: 14px }`.
const SCROLL_PAD: Pad = Pad::sides(16.0, 16.0, 10.0, 16.0);
const SCROLL_GAP: f32 = 14.0;
/// `.ic-canvas { padding: 24px 28px }` and `.ic-results { gap: 20px; minmax(260px, 1fr) }`.
const CANVAS_PAD: Pad = Pad::axes(24.0, 28.0);
const RESULT_MIN: f32 = 260.0;
const RESULT_GAP: f32 = 20.0;

/// One labelled select with a dice beside it: `.ic-ctrl`.
#[derive(Debug, Clone)]
struct Control {
    label: &'static str,
    options: Vec<String>,
    chosen: Option<usize>,
    random: bool,
    select: SelectState,
}

impl Control {
    fn new(label: &'static str, options: &[&str], chosen: Option<usize>, random: bool) -> Self {
        Self {
            label,
            options: options.iter().map(|option| (*option).to_owned()).collect(),
            chosen,
            random,
            select: SelectState::default(),
        }
    }
}

/// One collapsible group of controls: `.ic-sub`.
#[derive(Debug, Clone)]
struct Group {
    title: &'static str,
    icon: Icon,
    /// Which accent the group's own mark takes. `.ic-sub--scene svg { color: var(--accent-blue) }`
    /// and its three siblings.
    accent: fn(&Theme) -> egui::Color32,
    open: bool,
    /// `.ic-ctrl-grid { grid-template-columns: 1fr 1fr }` for the groups that have four controls.
    columns: usize,
    controls: Vec<Control>,
    /// `.ic-add-person` under the last group.
    add_person: bool,
}

/// Everything the page remembers between frames.
///
/// The caller owns it. That is the library's rule about state applied to a page: two of these can be
/// on one screen and neither knows about the other.
#[derive(Debug, Clone)]
pub struct ImageCreator {
    projects: Vec<String>,
    project: usize,
    project_open: bool,
    nav: usize,
    prompt: String,
    prompt_open: bool,
    controls_open: bool,
    count: i64,
    sizes: Vec<String>,
    size: usize,
    size_select: SelectState,
    workflows: Vec<String>,
    workflow: usize,
    workflow_select: SelectState,
    groups: Vec<Group>,
    /// How long each of the three results has been going, in seconds. `.ic-elapsed`.
    results: Vec<u64>,
    modal_open: bool,
    /// The Prompt Manager's own state.
    modal_tab: usize,
    modal_row: usize,
    modal_title: String,
    modal_body: String,
    save_to_open: bool,
}

impl Default for ImageCreator {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageCreator {
    /// The page with the reference's own contents in it, down to the prompt text.
    pub fn new() -> Self {
        let groups = vec![
            Group {
                title: "Qualities",
                icon: Icon::Spark,
                accent: |theme| theme.accent.violet,
                open: true,
                columns: 1,
                controls: vec![Control::new(
                    "Genre",
                    &[
                        "film photography",
                        "digital photo",
                        "oil painting",
                        "3D render",
                        "illustration",
                    ],
                    Some(0),
                    false,
                )],
                add_person: false,
            },
            Group {
                title: "Scene",
                icon: Icon::Image,
                accent: |theme| theme.accent.blue,
                open: true,
                columns: 2,
                controls: vec![
                    Control::new("Background col", &["urban", "nature", "studio", "abstract"], None, false),
                    Control::new("Indoor", &["—", "Option A", "Option B", "Option C"], None, false),
                    Control::new(
                        "Outdoor scenery",
                        &["mountain", "coastal", "desert", "forest", "cityscape"],
                        None,
                        false,
                    ),
                    Control::new("Interior", &["—", "Option A", "Option B", "Option C"], None, false),
                ],
                add_person: false,
            },
            Group {
                title: "Lighting",
                icon: Icon::Spark,
                accent: |theme| theme.accent.amber,
                open: true,
                columns: 2,
                controls: vec![
                    Control::new("Natural light", &["natural", "studio", "cinematic", "harsh"], None, false),
                    Control::new(
                        "Quality",
                        &["high-key lighting, soft", "low-key", "chiaroscuro", "flat"],
                        Some(0),
                        true,
                    ),
                    Control::new("Weather", &["clear", "overcast", "storm", "fog"], None, false),
                    Control::new("Colour", &["warm", "cool", "muted", "vivid"], None, false),
                ],
                add_person: false,
            },
            Group {
                title: "Person 1",
                icon: Icon::User,
                accent: |theme| theme.accent.rose,
                open: true,
                columns: 2,
                controls: vec![
                    Control::new(
                        "Ethnicity",
                        &["Caucasian", "East Asian", "South Asian", "African", "Latinx", "Mixed"],
                        Some(0),
                        false,
                    ),
                    Control::new(
                        "Skin color",
                        &["Fair", "Light", "Sun-kissed", "Olive", "Deep", "Dark"],
                        Some(2),
                        true,
                    ),
                    Control::new(
                        "Hair color",
                        &["Jet black", "Dark brown", "Chestnut", "Blonde", "Red", "Silver"],
                        Some(0),
                        true,
                    ),
                    Control::new(
                        "Body build",
                        &["Extremely thin, b…", "Athletic", "Average", "Muscular", "Heavyset"],
                        Some(0),
                        true,
                    ),
                ],
                add_person: true,
            },
        ];
        Self {
            projects: [
                "real-arcade v2",
                "mvl-arcade-v2",
                "Nocturne · music video",
                "Ember campaign · Q2",
                "Brand reel 2026",
            ]
            .iter()
            .map(|name| (*name).to_owned())
            .collect(),
            project: 0,
            project_open: false,
            nav: 3,
            prompt: "A wide cinematic shot of a lone figure standing in front of a grand \
neoclassical building at dusk.\n\nOverhead storm clouds part to reveal soft golden light spilling \
across the steps.\n\nRealistic photo, professional, 8K, film grain, natural lighting."
                .to_owned(),
            prompt_open: true,
            controls_open: true,
            count: 1,
            sizes: [
                "1920 × 1080 · HD · 16:9 · 2.1 MP",
                "1024 × 1024 · 1:1 · 1.0 MP",
                "1536 × 1024 · 3:2 · 1.6 MP",
                "1080 × 1920 · 9:16 · 2.1 MP",
            ]
            .iter()
            .map(|size| (*size).to_owned())
            .collect(),
            size: 0,
            size_select: SelectState::default(),
            workflows: ["/all", "/portrait", "/scene", "/product", "/concept"]
                .iter()
                .map(|flow| (*flow).to_owned())
                .collect(),
            workflow: 0,
            workflow_select: SelectState::default(),
            groups,
            results: vec![1, 3, 7],
            modal_open: false,
            modal_tab: 2,
            modal_row: 0,
            modal_title: String::new(),
            modal_body: String::new(),
            save_to_open: false,
        }
    }

    /// The seven entries in `.topnav`.
    fn nav_items() -> &'static [NavItem] {
        const ITEMS: &[NavItem] = &[
            NavItem::new("home", Icon::Home, "Home"),
            NavItem::new("library", Icon::Layers, "Library"),
            NavItem::new("storyboard", Icon::Storyboard, "Storyboard"),
            NavItem::new("studio", Icon::Spark, "Studio"),
            NavItem::new("render", Icon::Film, "Render"),
            NavItem::new("assets", Icon::Image, "Assets"),
            NavItem::new("settings", Icon::Settings, "Settings"),
        ];
        ITEMS
    }

    /// How many fields have their dice on, which is what the preset chip counts.
    fn randomised(&self) -> usize {
        self.groups
            .iter()
            .flat_map(|group| &group.controls)
            .filter(|control| control.random)
            .count()
    }

    /// Draw the whole page into `area`.
    pub fn show(&mut self, rux: &mut Rux<'_>, area: Rect) {
        rux.ground(area);
        let [topbar, body] = Flex::column(area).fixed(TOPBAR_HEIGHT).fill(1.0).solve();
        self.show_topbar(rux, topbar);

        // `.ic-main { display: flex }` — two panels of fixed width and a canvas taking the rest.
        // A collapsed panel is `width: 0`, which is a `Fixed(0.0)` rather than a branch in the layout.
        let prompt_width = match self.prompt_open {
            true => PROMPT_WIDTH,
            false => 0.0,
        };
        let controls_width = match self.controls_open {
            true => CONTROLS_WIDTH,
            false => 0.0,
        };
        let [prompt, controls, canvas] = Flex::row(body)
            .fixed(prompt_width)
            .fixed(controls_width)
            .fill(1.0)
            .solve();
        self.show_prompt_panel(rux, prompt);
        self.show_controls_panel(rux, controls);
        self.show_canvas(rux, canvas);

        if self.modal_open {
            self.show_prompt_manager(rux);
        }
    }

    /// `.topbar { display: grid; grid-template-columns: 1fr auto 1fr }`.
    fn show_topbar(&mut self, rux: &mut Rux<'_>, rect: Rect) {
        let theme = rux.theme();
        // `.topbar::after` — a hairline that fades out at both ends.
        Divider::new().faded().show(
            rux,
            Rect::from_min_size(rect.left_bottom(), Vec2::new(rect.width(), 1.0)),
        );

        let nav_width = TopNav::new(Self::nav_items(), self.nav).measure(rux).x;
        let [left, middle, right] = Flex::row(rect)
            .pad(Pad::axes(16.0, 28.0))
            .gap(24.0)
            .cross(Cross::Centre)
            .fill(1.0)
            .hug(nav_width)
            .fill(1.0)
            .solve();

        let trigger = Rect::from_min_size(left.left_top(), Vec2::new(280.0, 52.0));
        let trigger = Rect::from_center_size(
            egui::pos2(left.left() + trigger.width() / 2.0, left.center().y),
            trigger.size(),
        );
        let picked = ProjectMenu::new(&self.projects, self.project)
            .show(rux, trigger, &mut self.project_open);
        if let Some(index) = picked.chosen {
            self.project = index;
        }

        let nav = Rect::from_center_size(
            middle.center(),
            Vec2::new(nav_width, TopNav::new(Self::nav_items(), self.nav).measure(rux).y),
        );
        if let Some(index) = TopNav::new(Self::nav_items(), self.nav).show(rux, nav).chosen {
            self.nav = index;
        }

        // `.ic-saveto` — a sunken chip at the right hand end saying where results go.
        let label = self.projects.get(self.project).cloned().unwrap_or_default();
        let chip = Chip::new("Save to")
            .value(&label)
            .icon(Icon::Spark)
            .mono(false)
            .variant(ChipVariant::Sunken);
        let size = chip.measure(rux);
        let chip_rect = Rect::from_min_size(
            egui::pos2(right.right() - size.x, right.center().y - size.y / 2.0),
            size,
        );
        let (response, _) = chip.show(rux, chip_rect);
        if response.clicked() {
            self.save_to_open = !self.save_to_open;
        }
        let _ = theme;
    }

    /// `.ic-panel--prompt` — the prompt well, two mini buttons, and a pinned foot.
    fn show_prompt_panel(&mut self, rux: &mut Rux<'_>, rect: Rect) {
        if rect.width() < 1.0 {
            self.show_collapse_handle(rux, rect, true);
            return;
        }
        let theme = rux.theme();
        Panel::new().kind(PanelKind::Flat).pad(Pad::NONE).show(rux, rect);
        // `.ic-panel { border-right: 1px solid rgba(0,0,0,0.04) }`.
        rux.chrome.line(rect.right_top(), rect.right_bottom(), 1.0, theme.interaction.hairline);

        // `.ic-panel__foot` is pinned to the bottom; the scroll region takes the rest.
        let foot_height = 16.0 + 34.0 + 10.0 + 14.0 + 34.0 + 10.0 + 48.0 + 16.0;
        let [head, scroll, foot] = Flex::column(rect)
            .fixed(32.0)
            .fill(1.0)
            .fixed(foot_height)
            .solve();

        panel_title(rux, head, Icon::Edit, "Prompt", theme.accent.blue);

        let body = SCROLL_PAD.inside(scroll);
        let buttons_height = 34.0;
        let [well, buttons] = Flex::column(body)
            .gap(SCROLL_GAP)
            .fill(1.0)
            .fixed(buttons_height)
            .solve();
        TextArea::new(&mut self.prompt)
            .hint("Describe the image…")
            .count(true)
            .corner(Icon::Wand)
            .label("Prompt")
            .show(rux, well);

        // `.ic-btn-row { display: flex; gap: 8px }`, each button `flex: 1`.
        let [prompts, save] = Flex::row(buttons).gap(8.0).fill(1.0).fill(1.0).solve();
        if Button::new("Prompts")
            .icon(Icon::Folder)
            .size(ButtonSize::Mini)
            .show(rux, prompts)
            .clicked()
        {
            self.modal_open = true;
        }
        Button::new("Save").icon(Icon::Download).size(ButtonSize::Mini).show(rux, save);

        self.show_prompt_foot(rux, foot);
        self.show_collapse_handle(rux, rect, true);
    }

    /// `.ic-panel__foot` — the dimension row, the workflow select and the Create button.
    fn show_prompt_foot(&mut self, rux: &mut Rux<'_>, rect: Rect) {
        let theme = rux.theme();
        rux.chrome.line(rect.left_top(), rect.right_top(), 1.0, theme.surface.sunken);

        let inner = Pad::sides(12.0, 16.0, 16.0, 16.0).inside(rect);
        let mut pen = Pen::new(inner);

        // `.ic-dim-row` — the size select and the count, which does not shrink.
        let dim = pen.strip(34.0);
        let [size, count] = Flex::row(dim)
            .gap(8.0)
            .fill(1.0)
            .fixed(rux::components::STEPPER_WIDTH)
            .solve();
        let chosen = Select::new(&self.sizes, Some(self.size))
            .mono(true)
            .up(true)
            .label("Image size")
            .show(rux, size, &mut self.size_select)
            .chosen;
        if let Some(index) = chosen {
            self.size = index;
        }
        if let Some(value) = Stepper::new(self.count, 1, 9).label("Images").show(rux, count).changed {
            self.count = value;
        }

        pen.gap(10.0);
        // `.ic-labelled` — a mono label over the control.
        let label = pen.strip(14.0);
        field_label(rux, label, "Workflow");
        pen.gap(6.0);
        let flow = pen.strip(34.0);
        let chosen = Select::new(&self.workflows, Some(self.workflow))
            .up(true)
            .label("Workflow")
            .show(rux, flow, &mut self.workflow_select)
            .chosen;
        if let Some(index) = chosen {
            self.workflow = index;
        }

        pen.gap(10.0);
        let create = pen.strip(48.0);
        Button::new("Create")
            .variant(ButtonVariant::Primary)
            .size(ButtonSize::Large)
            .icon(Icon::Spark)
            .show(rux, create);
    }

    /// `.ic-panel--controls` — the preset chip and the four collapsible groups.
    fn show_controls_panel(&mut self, rux: &mut Rux<'_>, rect: Rect) {
        if rect.width() < 1.0 {
            self.show_collapse_handle(rux, rect, false);
            return;
        }
        let theme = rux.theme();
        Panel::new().kind(PanelKind::Flat).pad(Pad::NONE).show(rux, rect);
        rux.chrome.line(rect.right_top(), rect.right_bottom(), 1.0, theme.interaction.hairline);

        let [head, scroll] = Flex::column(rect).fixed(32.0).fill(1.0).solve();
        panel_title(rux, head, Icon::Settings, "Controls", theme.accent.violet);

        // The panel scrolls, so the whole column is drawn into a clip and moved by an offset.
        let body = SCROLL_PAD.inside(scroll);
        let content = self.controls_height(rux, body.width());
        let bar = Scrollbar::new(scroll, content + SCROLL_PAD.vertical());
        let offset_id = rux.ui.id().with("rux-image-creator-controls-scroll");
        let mut offset = rux.ui.memory(|memory| memory.data.get_temp(offset_id).unwrap_or(0.0));
        offset = bar.grab(rux, offset);
        if rux
            .ui
            .ctx()
            .pointer_latest_pos()
            .is_some_and(|at| scroll.contains(at))
        {
            let wheel = rux.ui.ctx().input(|input| input.smooth_scroll_delta.y);
            offset = (offset - wheel).clamp(0.0, (content + SCROLL_PAD.vertical() - scroll.height()).max(0.0));
        }

        let was = rux.ui.clip_rect();
        rux.ui.set_clip_rect(scroll.intersect(was));
        rux.chrome.clip(scroll, 0.0);
        let mut pen = Pen::new(Rect::from_min_size(
            egui::pos2(body.left(), body.top() - offset),
            Vec2::new(body.width(), content),
        ));

        if self.randomised() > 0 {
            let text = format!("{} fields randomized", self.randomised());
            let chip = Chip::new(&text)
                .icon(Icon::Spark)
                .variant(ChipVariant::Accent)
                .dismissible(true);
            let size = chip.measure(rux);
            let strip = pen.strip(size.y);
            // `align-self: flex-start` — the chip hugs its words rather than filling the column.
            let (_, dismissed) =
                chip.show(rux, Rect::from_min_size(strip.min, size));
            if dismissed {
                for group in &mut self.groups {
                    for control in &mut group.controls {
                        control.random = false;
                    }
                }
            }
            pen.gap(SCROLL_GAP);
        }

        for index in 0..self.groups.len() {
            let first = index == 0;
            let height = self.group_height(rux, index, body.width());
            let strip = pen.strip(height);
            self.show_group(rux, strip, index, first);
            pen.gap(SCROLL_GAP);
        }

        rux.chrome.unclip();
        rux.ui.set_clip_rect(was);
        bar.paint(rux, offset);
        rux.ui.memory_mut(|memory| memory.data.insert_temp(offset_id, offset));
        self.show_collapse_handle(rux, rect, false);
    }

    /// How tall the whole controls column comes out, which the scrollbar needs before anything is
    /// drawn.
    fn controls_height(&self, rux: &Rux<'_>, width: f32) -> f32 {
        let mut height = 0.0;
        if self.randomised() > 0 {
            height += 30.0 + SCROLL_GAP;
        }
        for index in 0..self.groups.len() {
            height += self.group_height(rux, index, width) + SCROLL_GAP;
        }
        height
    }

    /// `.ic-sub` plus its body: a heading, then either one control or a two-column grid of them.
    fn group_height(&self, _rux: &Rux<'_>, index: usize, _width: f32) -> f32 {
        let group = &self.groups[index];
        let head = match index == 0 {
            true => SubGroup::HEAD_HEIGHT,
            false => SubGroup::HEAD_HEIGHT + 14.0,
        };
        if !group.open {
            return head;
        }
        // `.ic-ctrl` is a label over a select, with a dice beside them: 14 + 4 + 34.
        let control = 52.0;
        let rows = group.controls.len().div_ceil(group.columns.max(1));
        let grid = rows as f32 * control + (rows.saturating_sub(1)) as f32 * 12.0;
        let person = match group.add_person {
            true => 12.0 + 36.0,
            false => 0.0,
        };
        head + 12.0 + grid + person
    }

    fn show_group(&mut self, rux: &mut Rux<'_>, rect: Rect, index: usize, first: bool) {
        let theme = rux.theme();
        let (title, icon, accent, open, columns, add_person) = {
            let group = &self.groups[index];
            (group.title, group.icon, (group.accent)(theme), group.open, group.columns, group.add_person)
        };
        let outcome = SubGroup::new(title, icon, accent, open).first(first).show(rux, rect);
        if outcome.toggled {
            self.groups[index].open = !open;
        }
        if !open {
            return;
        }

        // `.ic-ctrl-grid { grid-template-columns: 1fr 1fr; gap: 12px 10px }`.
        let grid = Grid::new(outcome.body, columns).gap(12.0, 10.0).row_height(52.0);
        for slot in 0..self.groups[index].controls.len() {
            let cell = grid.cell(slot);
            self.show_control(rux, cell, index, slot);
        }

        if add_person {
            let rows = self.groups[index].controls.len().div_ceil(columns.max(1));
            let top = outcome.body.top() + rows as f32 * 52.0 + (rows.saturating_sub(1)) as f32 * 12.0 + 12.0;
            let button = Rect::from_min_size(
                egui::pos2(outcome.body.left(), top),
                Vec2::new(outcome.body.width(), 36.0),
            );
            Button::new("Add Person")
                .variant(ButtonVariant::Dashed)
                .icon(Icon::Plus)
                .size(ButtonSize::Small)
                .show(rux, button);
        }
    }

    /// `.ic-ctrl { display: grid; grid-template-columns: 1fr auto; align-items: end }`.
    fn show_control(&mut self, rux: &mut Rux<'_>, rect: Rect, group: usize, slot: usize) {
        let [column, dice] = Flex::row(rect)
            .gap(8.0)
            .cross(Cross::End)
            .fill(1.0)
            .sized(Vec2::splat(DiceToggle::SIZE))
            .solve();
        let [label, select] = Flex::column(column).gap(4.0).fixed(14.0).fixed(34.0).solve();

        ctrl_label(rux, label, self.groups[group].controls[slot].label);
        let chosen = {
            let control = &mut self.groups[group].controls[slot];
            Select::new(&control.options, control.chosen)
                .label(control.label)
                .show(rux, select, &mut control.select)
                .chosen
        };
        if let Some(index) = chosen {
            self.groups[group].controls[slot].chosen = Some(index);
        }
        let on = self.groups[group].controls[slot].random;
        if DiceToggle::new(on).show(rux, dice).clicked() {
            self.groups[group].controls[slot].random = !on;
        }
    }

    /// `.ic-chev` — the round handle floating on a panel's border.
    fn show_collapse_handle(&mut self, rux: &mut Rux<'_>, rect: Rect, prompt: bool) {
        let open = match prompt {
            true => self.prompt_open,
            false => self.controls_open,
        };
        // `right: -12px; top: 50%` — half off the panel's own edge.
        let at = Rect::from_center_size(
            egui::pos2(rect.right(), rect.center().y),
            Vec2::splat(24.0),
        );
        let turn = match open {
            true => 0.0,
            false => std::f32::consts::PI,
        };
        let label = match prompt {
            true => "Collapse prompt",
            false => "Collapse controls",
        };
        if IconButton::new(Icon::ChevLeft, label)
            .size(IconButtonSize::Tiny)
            .turn(turn)
            .tint(rux.theme().ink.i500)
            .show(rux, at)
            .clicked()
        {
            match prompt {
                true => self.prompt_open = !self.prompt_open,
                false => self.controls_open = !self.controls_open,
            }
        }
    }

    /// `.ic-canvas` — a recessed area holding the results grid.
    fn show_canvas(&mut self, rux: &mut Rux<'_>, rect: Rect) {
        let theme = rux.theme();
        rux.ground_sunken(rect);
        let inner = CANVAS_PAD.inside(rect);
        let [head, results] = Flex::column(inner).gap(20.0).fixed(36.0).fill(1.0).solve();

        // `.ic-canvas-kicker` — "<b>3</b> images generated".
        let count = self.results.len().to_string();
        let strong = theme.ink.i900;
        let quiet = theme.ink.i500;
        let number = rux.text(Style::sans(13.0).semibold(), &count, strong);
        let width = number.size().x;
        let top = (head.center().y - number.size().y / 2.0).round();
        rux.ui.painter().galley(egui::pos2(head.left(), top), number, strong);
        let rest = rux.text(Style::sans(13.0), " images generated", quiet);
        rux.ui.painter().galley(egui::pos2(head.left() + width, top), rest, quiet);

        let clear = Button::new("Clear")
            .icon(Icon::Trash)
            .size(ButtonSize::Small)
            .variant(ButtonVariant::Secondary);
        let size = clear.measure(rux);
        let clear_rect = Rect::from_min_size(
            egui::pos2(head.right() - size.x, head.center().y - size.y / 2.0),
            size,
        );
        if clear.show(rux, clear_rect).clicked() {
            self.results.clear();
        }

        // `.ic-results { grid-template-columns: repeat(auto-fill, minmax(260px, 1fr)); gap: 20px }`.
        let columns = Grid::auto_fill(results.width(), RESULT_MIN, RESULT_GAP);
        let grid = Grid::new(results, columns).gap(RESULT_GAP, RESULT_GAP);
        let tile_height = Tile::height_at(grid.column_width());
        let grid = grid.row_height(tile_height);
        for (index, seconds) in self.results.clone().into_iter().enumerate() {
            let cell = grid.cell(index);
            if cell.bottom() > results.bottom() + tile_height {
                break;
            }
            Tile::new().kind(TileKind::Well).label("Result").show(rux, cell);
            Spinner::new().show(rux, Rect::from_center_size(cell.center(), Vec2::splat(56.0)));

            // `.ic-elapsed { bottom: 14px; left: 50% }`.
            let text = rux::components::elapsed_text(seconds);
            let chip = Chip::new(&text).icon(Icon::Bell).mono(true).variant(ChipVariant::Raised);
            let size = chip.measure(rux);
            let at = Rect::from_min_size(
                egui::pos2(cell.center().x - size.x / 2.0, cell.bottom() - 14.0 - size.y),
                size,
            );
            chip.show(rux, at);
        }
    }

    /// The Prompt Manager: a modal with a tabbed list on the left and a form on the right.
    fn show_prompt_manager(&mut self, rux: &mut Rux<'_>) {
        const TABS: &[(&str, Option<Icon>)] = &[
            ("Image", Some(Icon::Image)),
            ("Video", Some(Icon::Video)),
            ("Vision", Some(Icon::User)),
            ("Chat", Some(Icon::Chat)),
        ];
        let samples: [&[&str]; 4] = [
            &["Cinematic 35mm", "Studio portrait", "Product hero"],
            &["Slow dolly in", "Handheld chase", "Static wide"],
            &["Detailed V1", "Outfit Description", "Simple Generic"],
            &["System · Director", "System · Stylist"],
        ];
        let theme = rux.theme();
        let accents = [theme.accent.blue, theme.accent.violet, theme.accent.mint, theme.accent.coral];

        let tab = self.modal_tab;
        let row = self.modal_row;
        let list = samples[tab];
        let mut next_tab = tab;
        let mut next_row = row;

        let (outcome, _) = Modal::new("Prompt Manager")
            .icon(Icon::Folder)
            .icon_tint(theme.accent.amber)
            .footer(true)
            .show(rux, |rux, layout| {
                // `.ic-modal__body { grid-template-columns: 260px 1fr }`.
                let (left, right) = rux::components::split_body(rux, layout.body, 260.0);

                let inner = Pad::all(16.0).inside(left);
                let [tabs_rect, rows] = Flex::column(inner).gap(12.0).fixed(30.0).fill(1.0).solve();
                if let Some(index) = Segmented::new(TABS, tab)
                    .compact(true)
                    .accents(&accents)
                    .show(rux, tabs_rect)
                    .chosen
                {
                    next_tab = index;
                    next_row = 0;
                }
                let mut pen = Pen::new(rows);
                for (index, name) in list.iter().enumerate() {
                    let strip = pen.strip(38.0);
                    let hit = ListItem::new(name, index == row)
                        .pressed_when_selected()
                        .action(Icon::Check, "Apply")
                        .pad(Pad::axes(10.0, 12.0))
                        .show(rux, strip);
                    if hit.clicked {
                        next_row = index;
                    }
                    pen.gap(2.0);
                }

                // `.ic-modal__form { padding: 22px; gap: 16px }`.
                let form = Pad::all(22.0).inside(right);
                let [title_label, title, body_label, body] = Flex::column(form)
                    .gap(6.0)
                    .fixed(16.0)
                    .fixed(44.0)
                    .fixed(16.0)
                    .fill(1.0)
                    .solve();
                form_label(rux, title_label, "Title");
                TextInput::new(&mut self.modal_title)
                    .hint("Enter a title for this prompt…")
                    .label("Title")
                    .show(rux, title);
                form_label(rux, body_label, "Prompt Text");
                TextArea::new(&mut self.modal_body)
                    .hint("Enter the prompt text…")
                    .label("Prompt text")
                    .show(rux, body);

                // `.ic-modal__foot` — Delete on the left, New and Save on the right.
                let foot = Pad::axes(14.0, 22.0).inside(layout.foot);
                let delete = Button::new("Delete")
                    .variant(ButtonVariant::Danger)
                    .icon(Icon::Trash)
                    .size(ButtonSize::Small);
                let size = delete.measure(rux);
                delete.show(
                    rux,
                    Rect::from_min_size(
                        egui::pos2(foot.left(), foot.center().y - size.y / 2.0),
                        size,
                    ),
                );
                let new = Button::new("New").icon(Icon::Plus).size(ButtonSize::Small);
                let save = Button::new("Save")
                    .variant(ButtonVariant::Mint)
                    .icon(Icon::Check)
                    .size(ButtonSize::Small);
                let new_size = new.measure(rux);
                let save_size = save.measure(rux);
                let [new_rect, save_rect] = Flex::row(Rect::from_min_size(
                    egui::pos2(
                        foot.right() - new_size.x - 8.0 - save_size.x,
                        foot.center().y - new_size.y / 2.0,
                    ),
                    Vec2::new(new_size.x + 8.0 + save_size.x, new_size.y),
                ))
                .gap(8.0)
                .hug(new_size.x)
                .hug(save_size.x)
                .solve();
                new.show(rux, new_rect);
                save.show(rux, save_rect);
            });

        self.modal_tab = next_tab;
        self.modal_row = next_row;
        if outcome.dismissed {
            self.modal_open = false;
        }
    }
}

/// `.ic-panel__title` — a mark and a mono heading at the top of a panel.
fn panel_title(rux: &mut Rux<'_>, rect: Rect, icon: Icon, text: &str, tint: egui::Color32) {
    let theme = rux.theme();
    let inner = Pad::sides(14.0, 16.0, 4.0, 16.0).inside(rect);
    let [mark, label] = Flex::row(inner)
        .gap(8.0)
        .cross(Cross::Centre)
        .sized(Vec2::splat(12.0))
        .fill(1.0)
        .solve();
    rux.mark_in(Mark::new(icon, 12.0), mark, tint);
    let ink = theme.ink.i700;
    let galley = rux.text(Style::PANEL_TITLE, text, ink);
    rux::text::draw_left_centre(rux.ui.painter(), label.left_center(), galley, ink);
}

/// `.ic-labelled__label` — mono 9.5px uppercase at `0.14em`, indented 4.
fn field_label(rux: &mut Rux<'_>, rect: Rect, text: &str) {
    let ink = rux.theme().ink.i500;
    let galley = rux.text(Style::FIELD_LABEL, text, ink);
    rux.ui.painter().galley(egui::pos2(rect.left() + 4.0, rect.top()), galley, ink);
}

/// `.ic-ctrl__label` — the same, one step smaller, indented 2, and elided.
fn ctrl_label(rux: &mut Rux<'_>, rect: Rect, text: &str) {
    let ink = rux.theme().ink.i500;
    let galley = rux.elided(Style::CTRL_LABEL, text, ink, rect.width() - 2.0);
    rux.ui.painter().galley(egui::pos2(rect.left() + 2.0, rect.top()), galley, ink);
}

/// `.ic-form-label` — mono 10px uppercase at `0.14em`, over a field in the modal.
fn form_label(rux: &mut Rux<'_>, rect: Rect, text: &str) {
    let ink = rux.theme().ink.i500;
    let galley = rux.text(Style::mono(10.0).tracking(0.14).upper(), text, ink);
    rux.ui.painter().galley(rect.left_top(), galley, ink);
}

/// Unused imports kept honest: the page names these types in its own signatures.
const _: Option<(Length, Justify, Weight)> = None;
