//! The Controls screen: every control at touch size, on the device.
//!
//! The storybook is how the library is checked on a desktop, and its sidebar and filter are a desktop
//! shape. This is the same idea for a phone: one scrolling column with every control in it, so the thing
//! that decides whether a target is big enough is a thumb rather than a number.
//!
//! It also carries the sheet demonstration, because a sheet is the one piece of the design that cannot be
//! seen on a desktop at all.

use egui::{Rect, Vec2};
use rux::components::{
    Button, ButtonSize, ButtonVariant, Chip, ChipVariant, DiceToggle, IconButton, IconButtonSize,
    ListItem, Modal, ModalOutcome, Panel, PanelKind, Present, Progress, Segmented, Select,
    SelectState, Spinner, Stepper, Switch, TextInput,
};
use rux::icon::Icon;
use rux::layout::{Cross, Flex, Pad, Pen};
use rux::Rux;

use super::{Scroll, KICKER_HEIGHT};
use crate::{PAGE_PAD, STACK_GAP};

#[derive(Debug, Clone)]
pub struct Controls {
    search: String,
    field: String,
    count: i64,
    on: bool,
    dice: bool,
    segment: usize,
    options: Vec<String>,
    chosen: Option<usize>,
    select: SelectState,
    row: usize,
    sheet_open: bool,
    dialog_open: bool,
}

impl Default for Controls {
    fn default() -> Self {
        Self::new()
    }
}

impl Controls {
    pub fn new() -> Self {
        Self {
            search: String::new(),
            field: "Black Rainbow".to_owned(),
            count: 3,
            on: true,
            dice: false,
            segment: 1,
            options: vec![
                "Krea 2".to_owned(),
                "Ideogram 4".to_owned(),
                "Qwen Image 2.1".to_owned(),
                "MiniMax H3".to_owned(),
            ],
            chosen: Some(0),
            select: SelectState::default(),
            row: 0,
            sheet_open: false,
            dialog_open: false,
        }
    }

    pub fn show(&mut self, rux: &mut Rux<'_>, area: Rect) {
        let body = Pad::axes(0.0, PAGE_PAD).inside(area);
        let content = self.content_height(rux);
        Scroll::new(rux, area, body, content, "rux-mobile-controls").show(rux, |rux, pen| {
            self.column(rux, pen);
        });

        if self.sheet_open {
            self.sheet(rux);
        }
        if self.dialog_open {
            self.dialog(rux);
        }
    }

    fn column(&mut self, rux: &mut Rux<'_>, pen: &mut Pen) {
        self.section(rux, pen, "Buttons", |rux, pen, this| {
            let height = Button::new("Create").size(ButtonSize::Large).measure(rux).y;
            let row = pen.strip(height);
            let [primary, plain] = Flex::row(row).gap(8.0).fill(1.0).fill(1.0).solve();
            Button::new("Create")
                .icon(Icon::Spark)
                .size(ButtonSize::Large)
                .variant(ButtonVariant::Primary)
                .show(rux, primary);
            Button::new("Cancel").size(ButtonSize::Large).show(rux, plain);
            pen.gap(8.0);

            let small = Button::new("Save").size(ButtonSize::Small).measure(rux).y;
            let row = pen.strip(small);
            let [save, prompts, off] = Flex::row(row).gap(8.0).fill(1.0).fill(1.0).fill(1.0).solve();
            Button::new("Save").icon(Icon::Download).size(ButtonSize::Small).show(rux, save);
            Button::new("Prompts").icon(Icon::Folder).size(ButtonSize::Small).show(rux, prompts);
            Button::new("Off").size(ButtonSize::Small).enabled(false).show(rux, off);
            let _ = this;
        });

        self.section(rux, pen, "Icon buttons, every diameter", |rux, pen, this| {
            // Each one draws at its own diameter and claims the minimum target, which is what makes a 24
            // point collapse handle pressable without making it look like a 44 point button.
            let sizes = [
                IconButtonSize::Tiny,
                IconButtonSize::Small,
                IconButtonSize::Medium,
                IconButtonSize::Large,
                IconButtonSize::Huge,
            ];
            let tallest = sizes
                .iter()
                .map(|size| IconButton::new(Icon::Plus, "Add").size(*size).measure(rux).y)
                .fold(0.0_f32, f32::max);
            let row = pen.strip(tallest);
            let mut line = Flex::row(row).gap(8.0).cross(Cross::Centre);
            for size in sizes {
                let want = IconButton::new(Icon::Plus, "Add").size(size).measure(rux);
                line = line.sized(want);
            }
            let cells = line.solve_all();
            for (size, cell) in sizes.iter().zip(cells) {
                IconButton::new(Icon::Plus, "Add").size(*size).show(rux, cell);
            }
            let _ = this;
        });

        self.section(rux, pen, "Fields", |rux, pen, this| {
            let height = TextInput::new(&mut this.field).measure(rux).y;
            let row = pen.strip(height);
            TextInput::search(&mut this.search).show(rux, row);
            pen.gap(8.0);
            let row = pen.strip(height);
            TextInput::new(&mut this.field).show(rux, row);
        });

        self.section(rux, pen, "Choosing", |rux, pen, this| {
            let height = Select::new(&this.options, this.chosen).measure(rux).y;
            let row = pen.strip(height);
            // On this screen the menu comes up as a sheet, because the screen is compact. The call is the
            // same one the desktop storybook makes.
            let outcome = Select::new(&this.options, this.chosen)
                .label("Model")
                .show(rux, row, &mut this.select);
            if let Some(index) = outcome.chosen {
                this.chosen = Some(index);
            }
            pen.gap(8.0);

            let items = [("Fast", None), ("Quality", Some(Icon::Spark))];
            let segmented = Segmented::new(&items, this.segment);
            let row = pen.strip(segmented.measure(rux).y);
            if let Some(index) = segmented.show(rux, row).chosen {
                this.segment = index;
            }
        });

        self.section(rux, pen, "Numbers and switches", |rux, pen, this| {
            let stepper = Stepper::new(this.count, 1, 8).measure(rux);
            let switch = Switch::new(this.on).measure(rux);
            let dice = DiceToggle::new(this.dice).measure(rux);
            let row = pen.strip(stepper.y.max(switch.y).max(dice.y));
            let [count, toggle, random, spare] = Flex::row(row)
                .gap(12.0)
                .cross(Cross::Centre)
                .fixed(stepper.x)
                .fixed(switch.x)
                .fixed(dice.x)
                .fill(1.0)
                .solve();
            if let Some(value) = Stepper::new(this.count, 1, 8).show(rux, count).changed {
                this.count = value;
            }
            if Switch::new(this.on).show(rux, toggle).clicked() {
                this.on = !this.on;
            }
            if DiceToggle::new(this.dice).show(rux, random).clicked() {
                this.dice = !this.dice;
            }
            let _ = spare;
        });

        self.section(rux, pen, "Chips", |rux, pen, this| {
            let chips = [
                ("1280 x 704", ChipVariant::Sunken),
                ("4 images", ChipVariant::Accent),
                ("seed 8812", ChipVariant::Sunken),
            ];
            let tallest = chips
                .iter()
                .map(|(label, variant)| Chip::new(label).variant(*variant).measure(rux).y)
                .fold(0.0_f32, f32::max);
            let row = pen.strip(tallest);
            let mut line = Flex::row(row).gap(8.0).cross(Cross::Centre);
            for (label, variant) in chips {
                line = line.hug(Chip::new(label).variant(variant).measure(rux).x);
            }
            let cells = line.solve_all();
            for ((label, variant), cell) in chips.iter().zip(cells) {
                Chip::new(label).variant(*variant).mono(true).show(rux, cell);
            }
            let _ = this;
        });

        self.section(rux, pen, "Waiting", |rux, pen, this| {
            let row = pen.strip(48.0);
            let [spinner, bar] = Flex::row(row).gap(14.0).cross(Cross::Centre).fixed(48.0).fill(1.0).solve();
            Spinner::new().diameter(40.0).show(rux, spinner);
            Progress::new(0.62).show(
                rux,
                Rect::from_center_size(bar.center(), Vec2::new(bar.width(), 8.0)),
            );
            let _ = this;
        });

        self.section(rux, pen, "Rows", |rux, pen, this| {
            let names = ["Black Rainbow Labs", "Unluminous", "Field notes"];
            let height = ListItem::new("Ag", false).measure(rux).y;
            for (index, name) in names.iter().enumerate() {
                let at = pen.strip(height);
                if ListItem::new(name, index == this.row)
                    .tick(true)
                    .dot(true)
                    .show(rux, at)
                    .clicked
                {
                    this.row = index;
                }
                pen.gap(4.0);
            }
        });

        self.section(rux, pen, "Sheets", |rux, pen, this| {
            let height = Button::new("Open").size(ButtonSize::Small).measure(rux).y;
            let row = pen.strip(height);
            let [sheet, dialog] = Flex::row(row).gap(8.0).fill(1.0).fill(1.0).solve();
            if Button::new("Sheet").icon(Icon::Layers).size(ButtonSize::Small).show(rux, sheet).clicked()
            {
                this.sheet_open = true;
            }
            // The same component with `Present::Dialog` forced, so the two shapes can be compared on the
            // device rather than described.
            if Button::new("Dialog").icon(Icon::Cube).size(ButtonSize::Small).show(rux, dialog).clicked()
            {
                this.dialog_open = true;
            }
        });
    }

    /// One titled section in a flat panel, with the title above it.
    fn section(
        &mut self,
        rux: &mut Rux<'_>,
        pen: &mut Pen,
        title: &str,
        body: impl FnOnce(&mut Rux<'_>, &mut Pen, &mut Self),
    ) {
        let label = pen.strip(KICKER_HEIGHT);
        super::kicker(rux, label, title);
        pen.gap(6.0);

        let height = self.section_height(rux, title);
        let panel = pen.strip(height);
        let inner = Panel::new().kind(PanelKind::Flat).pad(Pad::all(12.0)).show(rux, panel);
        let mut inner_pen = Pen::new(inner);
        body(rux, &mut inner_pen, self);
        pen.gap(STACK_GAP);
    }

    /// A sheet, which is what a dialog is on a compact screen.
    fn sheet(&mut self, rux: &mut Rux<'_>) {
        let (outcome, closed) = Modal::new("A sheet")
            .icon(Icon::Layers)
            .size(Vec2::new(360.0, 300.0))
            .present(Present::Sheet)
            .footer(true)
            .show(rux, |rux, outcome: ModalOutcome| {
                let body = Pad::all(14.0).inside(outcome.body);
                let ink = rux.theme().ink.i700;
                let galley = rux.text(
                    rux::Style::PROSE,
                    "Square along the edge it came from, a handle at the top, and the home \
                     indicator's room added under the footer.",
                    ink,
                );
                rux.ui.painter().galley(body.left_top(), galley, ink);

                let foot = Pad::axes(0.0, 14.0).inside(outcome.foot);
                let height = Button::new("Done").size(ButtonSize::Small).measure(rux).y;
                Button::new("Done")
                    .variant(ButtonVariant::Primary)
                    .size(ButtonSize::Small)
                    .show(rux, Rect::from_center_size(foot.center(), Vec2::new(foot.width(), height)))
                    .clicked()
            });
        if outcome.dismissed || closed {
            self.sheet_open = false;
        }
    }

    /// The same component with the dialog shape forced, for the comparison.
    fn dialog(&mut self, rux: &mut Rux<'_>) {
        let (outcome, closed) = Modal::new("A dialog")
            .icon(Icon::Cube)
            .size(Vec2::new(320.0, 260.0))
            .present(Present::Dialog)
            .show(rux, |rux, outcome: ModalOutcome| {
                let body = Pad::all(14.0).inside(outcome.body);
                let ink = rux.theme().ink.i700;
                let galley = rux.text(
                    rux::Style::PROSE,
                    "Centred, rounded on all four corners. This is what the same call gives on a \
                     desktop.",
                    ink,
                );
                rux.ui.painter().galley(body.left_top(), galley, ink);
                false
            });
        if outcome.dismissed || closed {
            self.dialog_open = false;
        }
    }

    /// How tall each section's panel is. Measured rather than guessed, because a touch screen changes
    /// every one of these numbers.
    fn section_height(&self, rux: &Rux<'_>, title: &str) -> f32 {
        let pad = 24.0;
        let large = Button::new("Create").size(ButtonSize::Large).measure(rux).y;
        let small = Button::new("Save").size(ButtonSize::Small).measure(rux).y;
        let field = TextInput::search(&mut String::new()).measure(rux).y;
        let select = Select::new(&self.options, self.chosen).measure(rux).y;
        let items = [("Fast", None), ("Quality", Some(Icon::Spark))];
        let segmented = Segmented::new(&items, self.segment).measure(rux).y;
        let row = ListItem::new("Ag", false).measure(rux).y;
        let stepper = Stepper::new(self.count, 1, 8).measure(rux).y;
        let chip = Chip::new("1280 x 704").measure(rux).y;
        let biggest_icon = IconButton::new(Icon::Plus, "Add")
            .size(IconButtonSize::Huge)
            .measure(rux)
            .y;

        pad + match title {
            "Buttons" => large + 8.0 + small,
            "Icon buttons, every diameter" => biggest_icon,
            "Fields" => field * 2.0 + 8.0,
            "Choosing" => select + 8.0 + segmented,
            "Numbers and switches" => stepper.max(44.0),
            "Chips" => chip,
            "Waiting" => 48.0,
            "Rows" => row * 3.0 + 8.0,
            "Sheets" => small,
            _ => 44.0,
        }
    }

    fn content_height(&self, rux: &Rux<'_>) -> f32 {
        let titles = [
            "Buttons",
            "Icon buttons, every diameter",
            "Fields",
            "Choosing",
            "Numbers and switches",
            "Chips",
            "Waiting",
            "Rows",
            "Sheets",
        ];
        titles
            .iter()
            .map(|title| KICKER_HEIGHT + 6.0 + self.section_height(rux, title) + STACK_GAP)
            .sum()
    }
}
