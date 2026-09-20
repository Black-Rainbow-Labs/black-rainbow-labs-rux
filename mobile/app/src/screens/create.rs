//! The Create screen: the AI Studio Image Creator, laid out for a phone.
//!
//! **The same components as `examples/pages/image_creator.rs` and none of the same layout**, which is
//! the thing being demonstrated. The desktop page is three fixed columns — a 260 point prompt panel, a
//! 320 point controls panel and a canvas taking the rest. At 393 points that is not a layout, so here it
//! is one column that scrolls, the pickers open as sheets from the bottom edge, and the Create button is
//! pinned above the home indicator where a thumb can reach it.
//!
//! Nothing in this file asks what platform it is on. Every difference from the desktop page is either a
//! different arrangement of the same calls, or something `rux` did by itself because the screen it was
//! told about is a compact touch one.

use egui::{Rect, Vec2};
use rux::components::{
    Button, ButtonSize, Chip, ChipVariant, DiceToggle, IconButton, IconButtonSize, ListItem, Modal,
    ModalOutcome, Panel, PanelKind, Present, Select, SelectState, Spinner, Stepper, SubGroup, TextArea,
    Tile, TileKind,
};
use rux::icon::Icon;
use rux::layout::{Cross, Flex, Pad, Pen};
use rux::{Rux, Theme};

use super::{Scroll, KICKER_HEIGHT};
use crate::{PAGE_PAD, STACK_GAP};

/// One control: a labelled select, with its own open state.
#[derive(Debug, Clone)]
struct Control {
    label: &'static str,
    options: Vec<String>,
    chosen: Option<usize>,
    select: SelectState,
}

impl Control {
    fn new(label: &'static str, options: &[&str], chosen: Option<usize>) -> Self {
        Self {
            label,
            options: options.iter().map(|option| (*option).to_owned()).collect(),
            chosen,
            select: SelectState::default(),
        }
    }
}

/// One collapsible group of controls.
#[derive(Debug, Clone)]
struct Group {
    title: &'static str,
    icon: Icon,
    accent: fn(&Theme) -> egui::Color32,
    open: bool,
    controls: Vec<Control>,
}

/// Which picker is showing as a sheet, if any.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Sheet {
    Prompts,
    None,
}

/// The screen's own state. The application owns it, which is the library's rule about state applied to a
/// screen rather than to a control.
#[derive(Debug, Clone)]
pub struct Create {
    projects: Vec<String>,
    project: usize,
    project_open: bool,
    prompt: String,
    count: i64,
    sizes: Vec<String>,
    size: usize,
    size_select: SelectState,
    random: bool,
    groups: Vec<Group>,
    /// How long each result has been going, in seconds.
    results: Vec<u64>,
    sheet: Sheet,
    prompt_row: usize,
}

impl Default for Create {
    fn default() -> Self {
        Self::new()
    }
}

impl Create {
    pub fn new() -> Self {
        Self {
            projects: vec![
                "Black Rainbow Labs".to_owned(),
                "Unluminous".to_owned(),
                "Field notes".to_owned(),
            ],
            project: 0,
            project_open: false,
            prompt: "a woman in a bikini doing yoga on a headland at sunrise, close up, \
                     film photography, 35mm"
                .to_owned(),
            count: 4,
            sizes: vec![
                "1024 x 1024".to_owned(),
                "1280 x 704".to_owned(),
                "1920 x 1088".to_owned(),
                "832 x 1216".to_owned(),
            ],
            size: 1,
            size_select: SelectState::default(),
            random: false,
            groups: vec![
                Group {
                    title: "Qualities",
                    icon: Icon::Spark,
                    accent: |theme| theme.accent.violet,
                    open: true,
                    controls: vec![Control::new(
                        "Genre",
                        &["film photography", "digital photo", "oil painting", "3D render"],
                        Some(0),
                    )],
                },
                Group {
                    title: "Scene",
                    icon: Icon::Image,
                    accent: |theme| theme.accent.blue,
                    open: true,
                    controls: vec![
                        Control::new("Setting", &["headland", "studio", "kitchen", "forest"], Some(0)),
                        Control::new("Light", &["sunrise", "noon", "overcast", "tungsten"], Some(0)),
                    ],
                },
                Group {
                    title: "Camera",
                    icon: Icon::Film,
                    accent: |theme| theme.accent.amber,
                    open: false,
                    controls: vec![
                        Control::new("Lens", &["35mm", "50mm", "85mm", "135mm"], Some(0)),
                        Control::new("Angle", &["eye level", "low", "high", "dutch"], None),
                    ],
                },
            ],
            results: vec![18, 4],
            sheet: Sheet::None,
            prompt_row: 0,
        }
    }

    pub fn show(&mut self, rux: &mut Rux<'_>, area: Rect) {
        // The Create button is pinned rather than scrolled, because it is the screen's one action and a
        // person should not have to find it.
        let action_height = Button::new("Create").size(ButtonSize::Large).measure(rux).y;
        let [scroll, action] = Flex::column(area)
            .fill(1.0)
            .fixed(action_height + PAGE_PAD)
            .solve();

        let body = Pad::axes(0.0, PAGE_PAD).inside(scroll);
        let content = self.content_height(rux, body.width());
        Scroll::new(rux, scroll, body, content, "rux-mobile-create").show(rux, |rux, pen| {
            self.column(rux, pen, body.width());
        });

        self.action(rux, action);

        if self.sheet == Sheet::Prompts {
            self.prompts_sheet(rux);
        }
    }

    /// The scrolling column, in the order a phone reads it: what is being made, then what it is made
    /// from.
    fn column(&mut self, rux: &mut Rux<'_>, pen: &mut Pen, width: f32) {
        self.project_row(rux, pen.strip(PROJECT_HEIGHT));
        pen.gap(STACK_GAP);

        self.canvas(rux, pen.strip(canvas_height(width)));
        pen.gap(STACK_GAP);

        self.prompt_well(rux, pen.strip(PROMPT_HEIGHT));
        pen.gap(STACK_GAP);

        self.dimensions(rux, pen.strip(ROW_HEIGHT));
        pen.gap(STACK_GAP);

        for index in 0..self.groups.len() {
            let height = self.group_height(rux, index, width);
            let strip = pen.strip(height);
            self.group(rux, strip, index);
            pen.gap(STACK_GAP);
        }
    }

    /// The project, as a row rather than the desktop's 280 point trigger — there is no room beside it on
    /// a phone, so it gets the whole width and the list opens under it.
    fn project_row(&mut self, rux: &mut Rux<'_>, rect: Rect) {
        let theme = rux.theme();
        let inner = Panel::new().kind(PanelKind::Raised).pad(Pad::all(8.0)).show(rux, rect);
        let [chip, name, chevron] = Flex::row(inner)
            .gap(10.0)
            .cross(Cross::Centre)
            .sized(Vec2::splat(32.0))
            .fill(1.0)
            .sized(Vec2::splat(IconButtonSize::Small.diameter()))
            .solve();

        rux.chrome.surface(
            chip,
            theme.radius.sm,
            rux::Fill::gradient(theme.gradient.nav_active, chip),
            theme.elevation.raised_sm,
        );
        rux.mark_in(rux::Mark::new(Icon::Folder, 15.0), chip, egui::Color32::WHITE);

        let label = self.projects.get(self.project).cloned().unwrap_or_default();
        let ink = theme.ink.i900;
        let galley = rux.elided(rux::Style::sans(14.0).medium(), &label, ink, name.width());
        rux::text::draw_left_centre(rux.ui.painter(), name.left_center(), galley, ink);

        let turn = match self.project_open {
            true => std::f32::consts::PI,
            false => 0.0,
        };
        if IconButton::new(Icon::ChevDown, "Choose project")
            .size(IconButtonSize::Small)
            .turn(turn)
            .show(rux, chevron)
            .clicked()
        {
            self.project_open = !self.project_open;
        }

        if self.project_open {
            self.project_list(rux, rect);
        }
    }

    /// The project list, in its own layer under the row.
    fn project_list(&mut self, rux: &mut Rux<'_>, row: Rect) {
        let row_height = ListItem::new("Ag", false).measure(rux).y;
        let height = row_height * self.projects.len() as f32 + 12.0;
        let area = Rect::from_min_size(
            egui::pos2(row.left(), row.bottom() + 6.0),
            Vec2::new(row.width(), height),
        );
        let id = rux.ui.id().with("rux-mobile-projects");
        let projects = self.projects.clone();
        let current = self.project;
        let chosen = rux.overlay(id, area, egui::Order::Foreground, |rux| {
            let theme = rux.theme();
            rux.chrome.surface(area, theme.radius.md, theme.surface.s1, theme.elevation.raised_lg);
            let mut chosen = None;
            for (index, name) in projects.iter().enumerate() {
                let at = Rect::from_min_size(
                    egui::pos2(area.left() + 6.0, area.top() + 6.0 + index as f32 * row_height),
                    Vec2::new(area.width() - 12.0, row_height),
                );
                let outcome =
                    ListItem::new(name, index == current).tick(true).dot(true).show(rux, at);
                if outcome.clicked {
                    chosen = Some(index);
                }
            }
            chosen
        });
        if let Some(index) = chosen {
            self.project = index;
            self.project_open = false;
        }
    }

    /// The canvas: the results, or one big well waiting for them.
    fn canvas(&mut self, rux: &mut Rux<'_>, rect: Rect) {
        let mut tile = Tile::new().kind(TileKind::Dropzone).label("Canvas");
        // The two lines a dropzone shows are what it says when it is empty, so they go the moment there
        // is something in it — otherwise "Nothing yet" is printed behind the things there are.
        if self.results.is_empty() {
            tile = tile.empty_text("Nothing yet", "Press Create");
        }
        let (_, inside) = tile.show(rux, rect);

        // Each result is a spinner in the corner of the well with its elapsed time beside it, which is
        // `.ic-tile` plus `.ic-elapsed` — the same two things the desktop page shows, stacked instead of
        // laid across a grid because there is one column.
        let mut pen = Pen::new(Pad::all(10.0).inside(inside));
        for seconds in self.results.clone() {
            let strip = pen.strip(28.0);
            let [dot, words] = Flex::row(strip)
                .gap(8.0)
                .cross(Cross::Centre)
                .sized(Vec2::splat(20.0))
                .fill(1.0)
                .solve();
            Spinner::new().diameter(20.0).show(rux, dot);
            let ink = egui::Color32::WHITE;
            let text = rux::components::elapsed_text(seconds);
            let galley = rux.text(rux::Style::mono(11.0).medium(), &text, ink);
            rux::text::draw_left_centre(rux.ui.painter(), words.left_center(), galley, ink);
            pen.gap(6.0);
        }
    }

    /// The prompt, with the character count and the two buttons under it.
    fn prompt_well(&mut self, rux: &mut Rux<'_>, rect: Rect) {
        let buttons = Button::new("Prompts").size(ButtonSize::Small).measure(rux).y;
        let [well, row] = Flex::column(rect).gap(10.0).fill(1.0).fixed(buttons).solve();
        TextArea::new(&mut self.prompt)
            .hint("Describe the image…")
            .count(true)
            .corner(Icon::Wand)
            .label("Prompt")
            .show(rux, well);

        let [prompts, save] = Flex::row(row).gap(8.0).fill(1.0).fill(1.0).solve();
        if Button::new("Prompts")
            .icon(Icon::Folder)
            .size(ButtonSize::Small)
            .show(rux, prompts)
            .clicked()
        {
            self.sheet = Sheet::Prompts;
        }
        Button::new("Save").icon(Icon::Download).size(ButtonSize::Small).show(rux, save);
    }

    /// The dimension picker, the count and the dice — one row, because they belong together.
    ///
    /// On a desktop this is a 34 point select, a 66 point stepper and a 28 point dice. Every one of the
    /// three is a different size here and **not one number in this function changed**: `Select`,
    /// `Stepper` and `DiceToggle` each measured themselves against the screen they were told about.
    fn dimensions(&mut self, rux: &mut Rux<'_>, rect: Rect) {
        let stepper = Stepper::new(self.count, 1, 8).measure(rux);
        let dice = DiceToggle::new(self.random).measure(rux);
        let [size, count, random] = Flex::row(rect)
            .gap(8.0)
            .cross(Cross::Centre)
            .fill(1.0)
            .fixed(stepper.x)
            .fixed(dice.x)
            .solve();

        let outcome = Select::new(&self.sizes, Some(self.size))
            .mono(true)
            .label("Dimensions")
            .show(rux, size, &mut self.size_select);
        if let Some(index) = outcome.chosen {
            self.size = index;
        }

        if let Some(value) = Stepper::new(self.count, 1, 8).show(rux, count).changed {
            self.count = value;
        }
        if DiceToggle::new(self.random).show(rux, random).clicked() {
            self.random = !self.random;
        }
    }

    /// One collapsible group, its controls in a single column.
    fn group(&mut self, rux: &mut Rux<'_>, rect: Rect, index: usize) {
        let (title, icon, accent, open) = {
            let group = &self.groups[index];
            (group.title, group.icon, group.accent, group.open)
        };
        let theme = rux.theme();
        let inner = Panel::new()
            .kind(PanelKind::Flat)
            .pad(Pad::axes(12.0, 14.0))
            .show(rux, rect);

        let head = SubGroup::new(title, icon, accent(theme), open).first(true);
        let head_height = head.head_height(rux);
        let mut pen = Pen::new(inner);
        let strip = pen.strip(head_height);
        if head.show(rux, strip).toggled {
            self.groups[index].open = !open;
        }
        if !open {
            return;
        }

        for control in &mut self.groups[index].controls {
            pen.gap(8.0);
            let label = pen.strip(KICKER_HEIGHT);
            super::kicker(rux, label, control.label);
            let field = pen.strip(Select::new(&control.options, control.chosen).measure(rux).y);
            let outcome = Select::new(&control.options, control.chosen)
                .label(control.label)
                .show(rux, field, &mut control.select);
            if let Some(chosen) = outcome.chosen {
                control.chosen = Some(chosen);
            }
        }
    }

    /// The Create button, and the chip that says where the result goes.
    fn action(&mut self, rux: &mut Rux<'_>, rect: Rect) {
        let inner = Pad::sides(PAGE_PAD, PAGE_PAD, 0.0, PAGE_PAD).inside(rect);
        let label = self.projects.get(self.project).cloned().unwrap_or_default();
        let chip = Chip::new("to").value(&label).variant(ChipVariant::Sunken).icon(Icon::Spark);
        let chip_size = chip.measure(rux);
        let [button, to] = Flex::row(inner)
            .gap(10.0)
            .cross(Cross::Centre)
            .fill(1.0)
            .fixed(chip_size.x.min(inner.width() * 0.45))
            .solve();

        if Button::new("Create")
            .icon(Icon::Spark)
            .size(ButtonSize::Large)
            .variant(rux::components::ButtonVariant::Primary)
            .show(rux, button)
            .clicked()
        {
            self.results.insert(0, 0);
        }
        chip.show(rux, Rect::from_center_size(to.center(), Vec2::new(to.width(), chip_size.y)));
    }

    /// The Prompt Manager, as a sheet.
    ///
    /// `Present::Sheet` is passed rather than left `Automatic` only so the screen reads as what it is;
    /// `Automatic` would choose the same thing, because this screen is only ever compact.
    fn prompts_sheet(&mut self, rux: &mut Rux<'_>) {
        let saved = ["Golden hour portrait", "Studio, hard light", "Anamorphic wide"];
        let (outcome, closed) = Modal::new("Prompts")
            .icon(Icon::Folder)
            .size(Vec2::new(360.0, 420.0))
            .present(Present::Sheet)
            .footer(true)
            .show(rux, |rux, outcome: ModalOutcome| {
                let body = Pad::all(14.0).inside(outcome.body);
                let mut pen = Pen::new(body);
                let row_height = ListItem::new("Ag", false).measure(rux).y;
                for (index, name) in saved.iter().enumerate() {
                    let at = pen.strip(row_height);
                    if ListItem::new(name, index == self.prompt_row).tick(true).show(rux, at).clicked
                    {
                        self.prompt_row = index;
                    }
                    pen.gap(4.0);
                }

                let foot = Pad::axes(0.0, 14.0).inside(outcome.foot);
                let [cancel, use_it] = Flex::row(foot)
                    .gap(8.0)
                    .cross(Cross::Centre)
                    .fill(1.0)
                    .fill(1.0)
                    .solve();
                let height = Button::new("Use").size(ButtonSize::Small).measure(rux).y;
                let shrink = |cell: Rect| {
                    Rect::from_center_size(cell.center(), Vec2::new(cell.width(), height))
                };
                let dismissed = Button::new("Cancel")
                    .size(ButtonSize::Small)
                    .show(rux, shrink(cancel))
                    .clicked();
                let taken = Button::new("Use")
                    .icon(Icon::Check)
                    .size(ButtonSize::Small)
                    .variant(rux::components::ButtonVariant::Primary)
                    .show(rux, shrink(use_it))
                    .clicked();
                if taken {
                    self.prompt = saved[self.prompt_row].to_owned();
                }
                dismissed || taken
            });
        if outcome.dismissed || closed {
            self.sheet = Sheet::None;
        }
    }

    /// How tall the whole column comes to, which the scrollbar needs before anything is drawn.
    fn content_height(&self, rux: &Rux<'_>, width: f32) -> f32 {
        let mut total = PROJECT_HEIGHT
            + STACK_GAP
            + canvas_height(width)
            + STACK_GAP
            + PROMPT_HEIGHT
            + STACK_GAP
            + ROW_HEIGHT
            + STACK_GAP;
        for index in 0..self.groups.len() {
            total += self.group_height(rux, index, width) + STACK_GAP;
        }
        total
    }

    fn group_height(&self, rux: &Rux<'_>, index: usize, _width: f32) -> f32 {
        let group = &self.groups[index];
        let theme = rux.theme();
        let head = SubGroup::new(group.title, group.icon, (group.accent)(theme), group.open)
            .first(true)
            .head_height(rux);
        let mut total = head + 24.0;
        if group.open {
            for control in &group.controls {
                let field = Select::new(&control.options, control.chosen).measure(rux).y;
                total += 8.0 + KICKER_HEIGHT + field;
            }
        }
        total
    }
}

/// How tall the canvas is at a given width.
///
/// `16 / 9` of the column, capped: at 393 points that is 221 and the cap does nothing, and on a tablet's
/// 720 point column it would be 405 — most of the screen given to an empty box, with the prompt pushed
/// off the bottom.
fn canvas_height(width: f32) -> f32 {
    Tile::height_at(width).min(CANVAS_MAX_HEIGHT)
}

const CANVAS_MAX_HEIGHT: f32 = 300.0;

/// The project row: a 32 point chip and 8 points of padding either side.
const PROJECT_HEIGHT: f32 = 48.0;
/// The prompt well and the two buttons under it.
const PROMPT_HEIGHT: f32 = 190.0;
/// The dimension row. `Stepper` is the tallest thing in it on a touch screen, at the minimum target.
const ROW_HEIGHT: f32 = 44.0;
