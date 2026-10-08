//! Every component, in every state the reference has a rule for.
//!
//! The states here are the table in §8.1 of the design, made live. A state is drawn with the real
//! component, so "open" is the open dropdown and "focused" is a field the keyboard is in.

use egui::{Rect, Vec2};
use rux::components::{
    ButtonSize, ButtonVariant, ChipVariant, DiceToggle, IconButtonSize, IconButtonVariant,
    PanelKind, TileKind,
};
use rux::layout::{Flex, Pad, Pen};
use rux::prelude::*;
use rux::text::Style;

use super::{group, note, samples, wide, Demo, Story, GROUP_GAP};

pub fn stories() -> Vec<Story> {
    vec![
        story("Top Nav", ".topnav / .topnav__item", &["active", "inactive", "hover"], top_nav),
        story(
            "Button",
            ".ic-create / .ic-mini-btn / .ic-fbtn / .add-scene / .ic-add-person",
            &["primary", "secondary", "raised", "mint", "danger", "dashed", "ghost", "disabled", "sizes"],
            button,
        ),
        story(
            "Icon Button",
            ".toolbtn / .iconbtn / .dock__btn / .ic-chev / .ic-modal__close",
            &["sizes", "raised", "sunken", "primary", "ghost", "dot", "badge"],
            icon_button,
        ),
        story(
            "Select",
            ".ic-select / .ic-select__menu / .ic-select__item",
            &["closed", "open", "up", "mono", "empty", "long", "disabled"],
            select,
        ),
        story(
            "Project Menu",
            ".proj / .proj__trigger / .proj__menu",
            &["closed", "open", "selected row", "new project"],
            project_menu,
        ),
        story(
            "Text Input",
            ".ic-form-input / .search-field",
            &["empty", "filled", "focused", "search", "disabled"],
            text_input,
        ),
        story(
            "Text Area",
            ".ic-prompt / .ic-form-textarea",
            &["empty", "filled", "with count", "with corner button"],
            text_area,
        ),
        story("Stepper", ".ic-count / .num-input", &["minimum", "middle", "maximum"], stepper),
        story(
            "Loader",
            ".ic-spinner / .ic-tile--loading / .ic-elapsed",
            &["spinner", "in a tile", "progress", "indeterminate", "skeleton"],
            loader,
        ),
        story(
            "Chip",
            ".meta-chip / .volume-chip / .ic-preset-chip / .ic-elapsed / .ic-saveto",
            &["sunken", "raised", "accent", "with dot", "with value", "dismissible"],
            chip,
        ),
        story(
            "Segmented",
            ".view-toggle / .ic-modal-tabs",
            &["two", "three", "four", "compact", "accented"],
            segmented,
        ),
        story("Toggle", ".ic-dice / .toggle__knob", &["dice off", "dice on", "switch"], toggle),
        story(
            "Panel",
            ".scene / .ic-panel / .timeline / .ic-prompt-wrap",
            &["card", "flat", "well", "shallow well", "divider"],
            panel,
        ),
        story("Sub Group", ".ic-sub", &["open", "collapsed", "four accents"], sub_group),
        story(
            "List Item",
            ".proj__item / .ic-modal__list-item",
            &["default", "selected", "with dot", "with action", "pressed"],
            list_item,
        ),
        story("Avatar", ".avatar / .avatar__status", &["with status", "without"], avatar),
        story(
            "Tile",
            ".ic-tile / .scene__image",
            &["well", "dropzone", "with spinner"],
            tile,
        ),
        story("Modal", ".ic-modal / .ic-modal-backdrop", &["closed", "open", "with footer"], modal),
        story(
            "Instruments",
            "no reference rule: composed from the theme's surfaces, shadows, ink and accents (task-2211)",
            &["plate", "screen", "led", "key", "checkbox", "fader", "meter", "chart", "table", "timeline"],
            instruments,
        ),
    ]
}

/// The instrument controls Unluminous's chat pane draws an answer's components with.
fn instruments(rux: &mut Rux<'_>, demo: &mut Demo, area: Rect) -> f32 {
    use rux::components::instrument;
    let mut pen = Pen::new(area);
    note(
        rux,
        &mut pen,
        "Plates with a lit top edge, recessed screens, key caps that sink, LEDs, faders and segmented          meters. Colour is light: a lamp or glowing data, never a filled control.",
    );
    let blue = rux.theme().accent.blue;
    group(rux, &mut pen, "Keys and checkboxes");
    let rect = wide(
        rux,
        &mut pen,
        30.0,
        "a key with its lamp lit, a key held down, a plain key",
    );
    let first = Key::new("Make the change").led(blue, true);
    let size = first.measure(rux);
    first.show(rux, Rect::from_min_size(rect.min, size));
    let second = Key::new("main").down(true).led(blue, true);
    let next = Rect::from_min_size(
        rect.min + Vec2::new(size.x + 10.0, 0.0),
        second.measure(rux),
    );
    second.show(rux, next);
    let third = Key::new("Open Cargo.toml");
    let last = Rect::from_min_size(next.right_top() + Vec2::new(10.0, 0.0), third.measure(rux));
    third.show(rux, last);
    pen.gap(GROUP_GAP / 2.0);
    for (index, label) in [
        "Commit the work",
        "Run the window suite",
        "Copy notarize.env",
    ]
    .into_iter()
    .enumerate()
    {
        let rect = pen.strip(20.0);
        if Checkbox::new(demo.ticks[index], label)
            .id(egui::Id::new(("story-tick", index)))
            .show(rux, Rect::from_min_size(rect.min, Vec2::new(320.0, 20.0)))
            .clicked()
        {
            demo.ticks[index] = !demo.ticks[index];
        }
        pen.gap(8.0);
    }
    pen.gap(GROUP_GAP);
    group(rux, &mut pen, "Fader and meter");
    let rect = wide(
        rux,
        &mut pen,
        Fader::height(rux),
        "drag, press the groove, or use the arrow keys",
    );
    let fader = Fader::new(egui::Id::new("story-fader"), demo.fader, 0.0, 2000.0).step(50.0);
    if let Some(next) = fader
        .show(
            rux,
            Rect::from_min_size(rect.min, Vec2::new(rect.width().min(420.0), rect.height())),
        )
        .changed
    {
        demo.fader = next;
    }
    let rect = wide(
        rux,
        &mut pen,
        Meter::height(rux),
        "lit to the fader's value",
    );
    Meter::new((demo.fader / 2000.0) as f32)
        .top(rux.theme().accent.amber)
        .show(
            rux,
            Rect::from_min_size(rect.min, Vec2::new(rect.width().min(420.0), rect.height())),
        );
    pen.gap(GROUP_GAP);
    group(rux, &mut pen, "Chart");
    let labels = vec!["core".to_owned(), "app".to_owned(), "cli".to_owned()];
    let format = |v: f64| format!("{v:.0}");
    let chart = Chart::new(
        egui::Id::new("story-chart"),
        ChartKind::Bar,
        &labels,
        &format,
    )
    .series(Series {
        name: "last week",
        values: &[41.0, 81.0, 9.0],
        colour: instrument::Instrument::of(rux.theme()).quiet,
    })
    .series(Series {
        name: "this week",
        values: &[38.0, 131.0, 9.0],
        colour: blue,
    });
    let height = chart.height_at(rux, 420.0);
    let rect = wide(
        rux,
        &mut pen,
        height,
        "segments rise from the baseline the first time a chart is drawn",
    );
    chart.show(
        rux,
        Rect::from_min_size(rect.min, Vec2::new(rect.width().min(420.0), height)),
    );
    pen.gap(GROUP_GAP);
    group(rux, &mut pen, "Timeline");
    let items = [
        Item {
            time: "09:00",
            title: "Freeze main",
            text: "",
            stage: Stage::Done,
        },
        Item {
            time: "11:30",
            title: "Run the window suite",
            text: "483 pictures, about 40 minutes",
            stage: Stage::Active,
        },
        Item {
            time: "14:00",
            title: "Publish",
            text: "",
            stage: Stage::Todo,
        },
    ];
    let line = Timeline::new(egui::Id::new("story-timeline"), &items);
    let height = line.height_at(rux, 420.0);
    let rect = wide(rux, &mut pen, height, "the item in progress breathes");
    line.show(
        rux,
        Rect::from_min_size(rect.min, Vec2::new(420.0_f32.min(rect.width()), height)),
    );
    pen.used()
}

fn story(
    name: &'static str,
    rule: &'static str,
    states: &'static [&'static str],
    draw: fn(&mut Rux<'_>, &mut Demo, Rect) -> f32,
) -> Story {
    Story { section: "Components", name, rule, states, draw, full_bleed: false }
}

const NAV: &[NavItem] = &[
    NavItem::new("home", Icon::Home, "Home"),
    NavItem::new("library", Icon::Layers, "Library"),
    NavItem::new("storyboard", Icon::Storyboard, "Storyboard"),
    NavItem::new("studio", Icon::Spark, "Studio"),
    NavItem::new("render", Icon::Film, "Render"),
];

fn top_nav(rux: &mut Rux<'_>, demo: &mut Demo, area: Rect) -> f32 {
    let mut pen = Pen::new(area);
    note(
        rux,
        &mut pen,
        "Only the chosen item shows its label — `.topnav__item span { display: none }` — so the strip \
         changes width with the choice and measures itself rather than being given one. Click an item.",
    );
    group(rux, &mut pen, "Interactive");
    let nav = TopNav::new(NAV, demo.nav);
    let size = nav.measure(rux);
    let rect = wide(rux, &mut pen, size.y, "click to change the active item");
    let rect = Rect::from_min_size(rect.min, size);
    if let Some(index) = nav.show(rux, rect).chosen {
        demo.nav = index;
    }
    pen.gap(GROUP_GAP);

    group(rux, &mut pen, "Each item active");
    for index in 0..NAV.len().min(3) {
        let nav = TopNav::new(NAV, index);
        let size = nav.measure(rux);
        let rect = wide(rux, &mut pen, size.y, NAV[index].label);
        nav.show(rux, Rect::from_min_size(rect.min, size));
        pen.gap(16.0);
    }
    pen.gap(GROUP_GAP);
    pen.used()
}

fn button(rux: &mut Rux<'_>, _demo: &mut Demo, area: Rect) -> f32 {
    let mut pen = Pen::new(area);
    note(
        rux,
        &mut pen,
        "Seven variants and four sizes, every one a rule in the reference. Press one: `:active` is a \
         genuinely different shadow declaration and the whole control moves half a point down, which \
         is the reference's own `transform: translateY(0.5px)`.",
    );

    let variants: [(&str, ButtonVariant, Option<Icon>); 7] = [
        ("primary", ButtonVariant::Primary, Some(Icon::Spark)),
        ("secondary", ButtonVariant::Secondary, Some(Icon::Folder)),
        ("raised", ButtonVariant::Raised, Some(Icon::Plus)),
        ("mint", ButtonVariant::Mint, Some(Icon::Check)),
        ("danger", ButtonVariant::Danger, Some(Icon::Trash)),
        ("dashed", ButtonVariant::Dashed, Some(Icon::Plus)),
        ("ghost", ButtonVariant::Ghost, Some(Icon::Dots)),
    ];
    group(rux, &mut pen, "Variants");
    let mut cells = Vec::new();
    for (name, variant, icon) in variants {
        let mut button = Button::new(label_for(variant)).variant(variant).size(ButtonSize::Medium);
        if let Some(icon) = icon {
            button = button.icon(icon);
        }
        cells.push((name, button.measure(rux).x));
    }
    let rects = samples(rux, &mut pen, 40.0, &cells);
    for (rect, (_, variant, icon)) in rects.iter().zip(variants) {
        let mut button = Button::new(label_for(variant)).variant(variant).size(ButtonSize::Medium);
        if let Some(icon) = icon {
            button = button.icon(icon);
        }
        button.show(rux, *rect);
    }
    pen.gap(GROUP_GAP);

    group(rux, &mut pen, "Sizes");
    let sizes: [(&str, ButtonSize, f32); 4] = [
        ("mini · 9/12", ButtonSize::Mini, 34.0),
        ("small · 9/14", ButtonSize::Small, 34.0),
        ("medium · 10/20", ButtonSize::Medium, 40.0),
        ("large · 14/20", ButtonSize::Large, 48.0),
    ];
    let cells: Vec<(&str, f32)> = sizes
        .iter()
        .map(|(name, size, _)| {
            (*name, Button::new("Create").icon(Icon::Spark).size(*size).measure(rux).x)
        })
        .collect();
    let rects = samples(rux, &mut pen, 48.0, &cells);
    for (rect, (_, size, height)) in rects.iter().zip(sizes) {
        let at = Rect::from_min_size(
            egui::pos2(rect.left(), rect.center().y - height / 2.0),
            Vec2::new(rect.width(), height),
        );
        Button::new("Create").icon(Icon::Spark).size(size).show(rux, at);
    }
    pen.gap(GROUP_GAP);

    group(rux, &mut pen, "Disabled, and without a mark");
    let cells = [("disabled primary", 130.0), ("disabled secondary", 120.0), ("no icon", 110.0)];
    let rects = samples(rux, &mut pen, 40.0, &cells);
    Button::new("Create")
        .primary()
        .icon(Icon::Spark)
        .size(ButtonSize::Medium)
        .enabled(false)
        .show(rux, rects[0]);
    Button::new("Prompts")
        .icon(Icon::Folder)
        .size(ButtonSize::Medium)
        .enabled(false)
        .show(rux, rects[1]);
    Button::new("Clear").size(ButtonSize::Medium).show(rux, rects[2]);
    pen.gap(GROUP_GAP);
    pen.used()
}

fn label_for(variant: ButtonVariant) -> &'static str {
    match variant {
        ButtonVariant::Primary => "Create",
        ButtonVariant::Secondary => "Prompts",
        ButtonVariant::Raised => "Add Scene",
        ButtonVariant::Mint => "Save",
        ButtonVariant::Danger => "Delete",
        ButtonVariant::Dashed => "Add Person",
        ButtonVariant::Ghost => "More",
    }
}

fn icon_button(rux: &mut Rux<'_>, _demo: &mut Demo, area: Rect) -> f32 {
    let theme = rux.theme();
    let mut pen = Pen::new(area);
    note(
        rux,
        &mut pen,
        "Five diameters and four surfaces. The mark is a tinted texture rather than a path in the \
         canvas, so hovering one — which changes its colour in nineteen rules of the reference — \
         costs a vertex colour and rasterises nothing.",
    );

    group(rux, &mut pen, "Sizes");
    let sizes: [(&str, IconButtonSize); 5] = [
        ("tiny · 24", IconButtonSize::Tiny),
        ("small · 28", IconButtonSize::Small),
        ("medium · 32", IconButtonSize::Medium),
        ("large · 40", IconButtonSize::Large),
        ("huge · 48", IconButtonSize::Huge),
    ];
    let cells: Vec<(&str, f32)> = sizes.iter().map(|(name, size)| (*name, size.diameter())).collect();
    let rects = samples(rux, &mut pen, 48.0, &cells);
    for (rect, (_, size)) in rects.iter().zip(sizes) {
        IconButton::new(Icon::Settings, "Settings").size(size).show(rux, *rect);
    }
    pen.gap(GROUP_GAP);

    group(rux, &mut pen, "Variants");
    let cells = [("raised", 48.0), ("sunken", 48.0), ("primary", 48.0), ("ghost", 48.0)];
    let rects = samples(rux, &mut pen, 48.0, &cells);
    let variants = [
        IconButtonVariant::Raised,
        IconButtonVariant::Sunken,
        IconButtonVariant::Primary,
        IconButtonVariant::Ghost,
    ];
    for (rect, variant) in rects.iter().zip(variants) {
        IconButton::new(Icon::Bell, "Notifications").variant(variant).show(rux, *rect);
    }
    pen.gap(GROUP_GAP);

    group(rux, &mut pen, "With a dot, a badge, and turned");
    let cells = [("status dot", 48.0), ("count badge", 56.0), ("chevron turned", 48.0), ("disabled", 48.0)];
    let rects = samples(rux, &mut pen, 48.0, &cells);
    IconButton::new(Icon::Bell, "Notifications").dot(theme.accent.coral).show(rux, rects[0]);
    IconButton::new(Icon::Chat, "Messages")
        .size(IconButtonSize::Huge)
        .variant(IconButtonVariant::Primary)
        .badge(12)
        .show(rux, rects[1]);
    IconButton::new(Icon::ChevLeft, "Collapse")
        .size(IconButtonSize::Tiny)
        .turn(std::f32::consts::PI)
        .show(rux, rects[2]);
    IconButton::new(Icon::Trash, "Delete").enabled(false).show(rux, rects[3]);
    pen.gap(GROUP_GAP);
    pen.used()
}

fn select(rux: &mut Rux<'_>, demo: &mut Demo, area: Rect) -> f32 {
    let mut pen = Pen::new(area);
    note(
        rux,
        &mut pen,
        "Open one. The menu is its own egui layer with its own decoration canvas, because a popup \
         drawn into the page's painter would appear behind the page. It opens upwards by itself when \
         there is no room below, which is `.ic-select--up` made automatic.",
    );

    group(rux, &mut pen, "Ordinary");
    let rect = wide(rux, &mut pen, 34.0, "click to open · chosen row is ticked and accented");
    let rect = Rect::from_min_size(rect.min, Vec2::new(280.0, 34.0));
    if let Some(index) =
        Select::new(&demo.options, demo.selected).show(rux, rect, &mut demo.select).chosen
    {
        demo.selected = Some(index);
    }
    pen.gap(GROUP_GAP);

    group(rux, &mut pen, "Mono, and empty");
    let cells = [("mono · 11px", 300.0), ("nothing chosen", 200.0), ("disabled", 200.0)];
    let rects = samples(rux, &mut pen, 34.0, &cells);
    if let Some(index) = Select::new(&demo.sizes, demo.selected_mono)
        .mono(true)
        .show(rux, rects[0], &mut demo.select_mono)
        .chosen
    {
        demo.selected_mono = Some(index);
    }
    let mut unused = rux::components::SelectState::default();
    Select::new(&demo.options, None).show(rux, rects[1], &mut unused);
    Select::new(&demo.options, Some(0)).enabled(false).show(rux, rects[2], &mut unused);
    pen.gap(GROUP_GAP);

    group(rux, &mut pen, "A list longer than the menu's 280 points");
    let rect = wide(rux, &mut pen, 34.0, "scrolls inside the menu");
    let rect = Rect::from_min_size(rect.min, Vec2::new(280.0, 34.0));
    if let Some(index) =
        Select::new(&demo.long, demo.selected_long).show(rux, rect, &mut demo.select_long).chosen
    {
        demo.selected_long = Some(index);
    }
    pen.gap(GROUP_GAP);

    group(rux, &mut pen, "Forced upwards · .ic-select--up");
    let rect = wide(rux, &mut pen, 34.0, "the menu opens above the trigger");
    let rect = Rect::from_min_size(rect.min, Vec2::new(280.0, 34.0));
    if let Some(index) = Select::new(&demo.options, demo.selected)
        .up(true)
        .show(rux, rect, &mut demo.select_up)
        .chosen
    {
        demo.selected = Some(index);
    }
    pen.gap(GROUP_GAP);
    pen.used()
}

fn project_menu(rux: &mut Rux<'_>, demo: &mut Demo, area: Rect) -> f32 {
    let mut pen = Pen::new(area);
    note(
        rux,
        &mut pen,
        "Not a Select with a bigger trigger: it has a gradient icon chip, a kicker over an elided \
         name, a minimum width, a heading on its menu and an action under it.",
    );
    group(rux, &mut pen, "Interactive");
    let rect = wide(rux, &mut pen, 52.0, "click the trigger");
    let rect = Rect::from_min_size(rect.min, Vec2::new(280.0, 52.0));
    let outcome = ProjectMenu::new(&demo.projects, demo.project).show(rux, rect, &mut demo.project_open);
    if let Some(index) = outcome.chosen {
        demo.project = index;
    }
    pen.gap(GROUP_GAP);
    pen.used()
}

fn text_input(rux: &mut Rux<'_>, demo: &mut Demo, area: Rect) -> f32 {
    let mut pen = Pen::new(area);
    note(
        rux,
        &mut pen,
        "The caret, the selection and the clipboard are egui's. What the library adds is the well, \
         the face and one detail: the box is given the field's whole rectangle with the padding as \
         its own margin, so a press in the padding is a press on the box rather than on nothing.",
    );

    group(rux, &mut pen, "Form field");
    let cells = [("empty", 320.0), ("filled · type in it", 320.0)];
    let rects = samples(rux, &mut pen, 44.0, &cells);
    let mut empty = String::new();
    TextInput::new(&mut empty).hint("Enter a title for this prompt…").show(rux, rects[0]);
    TextInput::new(&mut demo.input).label("Title").show(rux, rects[1]);
    pen.gap(GROUP_GAP);

    group(rux, &mut pen, "Search, and disabled");
    let cells = [("search · pill with a mark", 280.0), ("disabled", 240.0)];
    let rects = samples(rux, &mut pen, 40.0, &cells);
    TextInput::search(&mut demo.search).hint("Filter…").show(rux, rects[0]);
    let mut off = "Read only".to_owned();
    TextInput::new(&mut off).enabled(false).show(rux, rects[1]);
    pen.gap(GROUP_GAP);
    pen.used()
}

fn text_area(rux: &mut Rux<'_>, demo: &mut Demo, area: Rect) -> f32 {
    let mut pen = Pen::new(area);
    note(
        rux,
        &mut pen,
        "`.ic-prompt` has `padding: 14px 40px 28px 14px` — the right padding leaves room for the \
         shuffle button and the bottom padding for the character count, both painted over the well \
         rather than laid out in the flow.",
    );
    group(rux, &mut pen, "With a count and a corner button");
    let rect = wide(rux, &mut pen, 200.0, "type in it · the count follows");
    let rect = Rect::from_min_size(rect.min, Vec2::new(rect.width().min(360.0), 200.0));
    TextArea::new(&mut demo.prompt)
        .hint("Describe the image…")
        .count(true)
        .corner(Icon::Wand)
        .show(rux, rect);
    pen.gap(GROUP_GAP);

    group(rux, &mut pen, "Plain");
    let rect = wide(rux, &mut pen, 140.0, "no count, no corner");
    let rect = Rect::from_min_size(rect.min, Vec2::new(rect.width().min(360.0), 140.0));
    let mut empty = String::new();
    TextArea::new(&mut empty).hint("Enter the prompt text…").show(rux, rect);
    pen.gap(GROUP_GAP);
    pen.used()
}

fn stepper(rux: &mut Rux<'_>, demo: &mut Demo, area: Rect) -> f32 {
    let mut pen = Pen::new(area);
    note(
        rux,
        &mut pen,
        "`width: 66px; flex-shrink: 0` — the one control in the reference that refuses to give way \
         when its row runs out of room. Its up tick is the same chevron turned over, which is why the \
         icon set has no separate `chevUp`.",
    );
    group(rux, &mut pen, "Interactive");
    let cells = [
        ("at the minimum", rux::components::STEPPER_WIDTH),
        ("in the middle", rux::components::STEPPER_WIDTH),
        ("at the maximum", rux::components::STEPPER_WIDTH),
    ];
    let rects = samples(rux, &mut pen, 34.0, &cells);
    Stepper::new(1, 1, 9).show(rux, rects[0]);
    if let Some(value) = Stepper::new(demo.count, 1, 9).show(rux, rects[1]).changed {
        demo.count = value;
    }
    Stepper::new(9, 1, 9).show(rux, rects[2]);
    pen.gap(GROUP_GAP);
    pen.used()
}

fn loader(rux: &mut Rux<'_>, demo: &mut Demo, area: Rect) -> f32 {
    let mut pen = Pen::new(area);
    note(
        rux,
        &mut pen,
        "The spinner's arc is the one stroke in the library whose paint is a gradient, which is why \
         an arc is a shape of its own rather than an icon. The skeleton's sweep is an egui mesh, not \
         decoration: a moving gradient in the canvas would rasterise the layer sixty times a second.",
    );
    group(rux, &mut pen, "Spinner");
    let cells = [("in a well · 56", 56.0), ("bare · 40", 40.0), ("small · 28", 28.0)];
    let rects = samples(rux, &mut pen, 56.0, &cells);
    Spinner::new().show(rux, rects[0]);
    Spinner::new().diameter(40.0).bare().show(rux, rects[1]);
    Spinner::new().diameter(28.0).bare().show(rux, rects[2]);
    pen.gap(GROUP_GAP);

    group(rux, &mut pen, "Progress");
    let bar = wide(rux, &mut pen, 6.0, "62%");
    Progress::new(demo.progress).show(rux, Rect::from_min_size(bar.min, Vec2::new(280.0, 6.0)));
    pen.gap(14.0);
    let bar = wide(rux, &mut pen, 6.0, "indeterminate");
    Progress::indeterminate().show(rux, Rect::from_min_size(bar.min, Vec2::new(280.0, 6.0)));
    pen.gap(GROUP_GAP);

    group(rux, &mut pen, "Skeleton");
    let block = wide(rux, &mut pen, 72.0, "a block that has not arrived yet");
    Skeleton::new().show(rux, Rect::from_min_size(block.min, Vec2::new(320.0, 72.0)));
    pen.gap(GROUP_GAP);
    pen.used()
}

fn chip(rux: &mut Rux<'_>, _demo: &mut Demo, area: Rect) -> f32 {
    let theme = rux.theme();
    let mut pen = Pen::new(area);
    note(rux, &mut pen, "Six rules in the reference, one component. A chip says something; it does not do something.");

    group(rux, &mut pen, "Variants");
    let samples_list: Vec<(String, ChipVariant, Option<egui::Color32>, Option<Icon>, bool)> = vec![
        ("Sunken".to_owned(), ChipVariant::Sunken, None, None, false),
        ("Raised".to_owned(), ChipVariant::Raised, None, Some(Icon::Bell), false),
        ("3 fields randomized".to_owned(), ChipVariant::Accent, None, Some(Icon::Spark), true),
        ("Live".to_owned(), ChipVariant::Sunken, Some(theme.accent.coral), None, false),
        ("Ready".to_owned(), ChipVariant::Sunken, Some(theme.accent.mint), None, false),
    ];
    let mut cells = Vec::new();
    for (label, variant, dot, icon, dismiss) in &samples_list {
        let mut chip = Chip::new(label).variant(*variant).dismissible(*dismiss);
        if let Some(colour) = dot {
            chip = chip.dot(*colour);
        }
        if let Some(icon) = icon {
            chip = chip.icon(*icon);
        }
        cells.push((label.as_str(), chip.measure(rux).x));
    }
    let captions: Vec<(&str, f32)> = cells.iter().map(|(name, width)| (*name, *width)).collect();
    let rects = samples(rux, &mut pen, 32.0, &captions);
    for (rect, (label, variant, dot, icon, dismiss)) in rects.iter().zip(&samples_list) {
        let mut chip = Chip::new(label).variant(*variant).dismissible(*dismiss);
        if let Some(colour) = dot {
            chip = chip.dot(*colour);
        }
        if let Some(icon) = icon {
            chip = chip.icon(*icon);
        }
        chip.show(rux, *rect);
    }
    pen.gap(GROUP_GAP);

    group(rux, &mut pen, "With a value, and in mono");
    let meta = Chip::new("Scenes").value("12");
    let width = meta.measure(rux).x;
    let mono = Chip::new("7s").icon(Icon::Bell).mono(true).variant(ChipVariant::Raised);
    let mono_width = mono.measure(rux).x;
    let rects = samples(rux, &mut pen, 32.0, &[(".meta-chip b", width), (".ic-elapsed", mono_width)]);
    meta.show(rux, rects[0]);
    mono.show(rux, rects[1]);
    pen.gap(GROUP_GAP);
    pen.used()
}

fn segmented(rux: &mut Rux<'_>, demo: &mut Demo, area: Rect) -> f32 {
    let theme = rux.theme();
    let mut pen = Pen::new(area);
    note(
        rux,
        &mut pen,
        "A pressed track with a raised pill on the chosen one, which is the neumorphic way of saying \
         'this one' without a colour. The modal's tabs are the same control one size down, with an \
         accent per tab.",
    );

    const VIEWS: &[(&str, Option<Icon>)] = &[("Grid", Some(Icon::Storyboard)), ("List", Some(Icon::Layers))];
    group(rux, &mut pen, "View toggle");
    let control = Segmented::new(VIEWS, demo.segmented);
    let size = control.measure(rux);
    let rect = wide(rux, &mut pen, size.y, "click to change");
    if let Some(index) = control.show(rux, Rect::from_min_size(rect.min, size)).chosen {
        demo.segmented = index;
    }
    pen.gap(GROUP_GAP);

    const TABS: &[(&str, Option<Icon>)] = &[
        ("Image", Some(Icon::Image)),
        ("Video", Some(Icon::Video)),
        ("Vision", Some(Icon::User)),
        ("Chat", Some(Icon::Chat)),
    ];
    group(rux, &mut pen, "Modal tabs · compact, one accent each");
    let accents = [theme.accent.blue, theme.accent.violet, theme.accent.mint, theme.accent.coral];
    let control = Segmented::new(TABS, demo.tabs).compact(true).accents(&accents);
    let size = control.measure(rux);
    let rect = wide(rux, &mut pen, size.y, "the chosen tab takes its own colour");
    if let Some(index) = control.show(rux, Rect::from_min_size(rect.min, size)).chosen {
        demo.tabs = index;
    }
    pen.gap(GROUP_GAP);
    pen.used()
}

fn toggle(rux: &mut Rux<'_>, demo: &mut Demo, area: Rect) -> f32 {
    let mut pen = Pen::new(area);
    note(
        rux,
        &mut pen,
        "The dice is off when it is pressed into the surface and on when it is a violet gradient with \
         a glow under it — two genuinely different declarations, which is what makes the state \
         readable with no label.",
    );
    group(rux, &mut pen, "Dice");
    let cells = [("off", DiceToggle::SIZE), ("on", DiceToggle::SIZE), ("interactive", DiceToggle::SIZE)];
    let rects = samples(rux, &mut pen, DiceToggle::SIZE, &cells);
    DiceToggle::new(false).show(rux, rects[0]);
    DiceToggle::new(true).show(rux, rects[1]);
    if DiceToggle::new(demo.dice).show(rux, rects[2]).clicked() {
        demo.dice = !demo.dice;
    }
    pen.gap(GROUP_GAP);

    group(rux, &mut pen, "Switch");
    let cells = [("off", Switch::SIZE.x), ("on", Switch::SIZE.x), ("interactive", Switch::SIZE.x)];
    let rects = samples(rux, &mut pen, Switch::SIZE.y, &cells);
    Switch::new(false).show(rux, rects[0]);
    Switch::new(true).show(rux, rects[1]);
    if Switch::new(demo.switch).show(rux, rects[2]).clicked() {
        demo.switch = !demo.switch;
    }
    pen.gap(GROUP_GAP);
    pen.used()
}

fn panel(rux: &mut Rux<'_>, _demo: &mut Demo, area: Rect) -> f32 {
    let theme = rux.theme();
    let mut pen = Pen::new(area);
    note(rux, &mut pen, "Cards stand off the page, wells are pressed into it, and a divider fades out at both ends.");

    group(rux, &mut pen, "Cards and wells");
    let cells = [("card · raised", 200.0), ("card · raised sm", 200.0), ("well", 200.0), ("well · shallow", 200.0)];
    let rects = samples(rux, &mut pen, 96.0, &cells);
    Panel::new().show(rux, rects[0]);
    Panel::new().kind(PanelKind::RaisedSmall).show(rux, rects[1]);
    Well::new().radius(theme.radius.xl).show(rux, rects[2]);
    Well::new().radius(theme.radius.xl).shallow().show(rux, rects[3]);
    pen.gap(GROUP_GAP);

    group(rux, &mut pen, "Divider");
    let rule = wide(rux, &mut pen, 1.0, "faded at both ends · .topbar::after");
    Divider::new().faded().show(rux, rule);
    pen.gap(14.0);
    let rule = wide(rux, &mut pen, 1.0, "plain hairline");
    Divider::new().show(rux, rule);
    pen.gap(GROUP_GAP);
    pen.used()
}

fn sub_group(rux: &mut Rux<'_>, demo: &mut Demo, area: Rect) -> f32 {
    let theme = rux.theme();
    let mut pen = Pen::new(area);
    note(
        rux,
        &mut pen,
        "A heading with a disclosure and a hairline over it. Its own mark takes the section's accent \
         — violet, blue, amber and rose, which is the reference's own four.",
    );
    group(rux, &mut pen, "Four sections");
    let sections: [(&str, Icon, egui::Color32); 4] = [
        ("Qualities", Icon::Spark, theme.accent.violet),
        ("Scene", Icon::Image, theme.accent.blue),
        ("Lighting", Icon::Spark, theme.accent.amber),
        ("Person 1", Icon::User, theme.accent.rose),
    ];
    for (index, (title, icon, accent)) in sections.into_iter().enumerate() {
        let open = match index {
            // One shut, so both states are on the page at once.
            1 => false,
            _ => demo.dice || index != 0,
        };
        let head = SubGroup::new(title, icon, accent, open);
        let strip = pen.strip(head.head_height(rux) + if open { 46.0 } else { 0.0 });
        let outcome = head.first(index == 0).show(rux, strip.intersect(area));
        if open {
            let body = Rect::from_min_size(outcome.body.min, Vec2::new(280.0, 34.0));
            let mut state = rux::components::SelectState::default();
            Select::new(&demo.options, Some(0)).enabled(false).show(rux, body, &mut state);
        }
        pen.gap(14.0);
    }
    pen.gap(GROUP_GAP);
    pen.used()
}

fn list_item(rux: &mut Rux<'_>, demo: &mut Demo, area: Rect) -> f32 {
    let mut pen = Pen::new(area);
    note(
        rux,
        &mut pen,
        "A chosen `.proj__item` takes the accent and a glowing dot; a chosen `.ic-modal__list-item` \
         is pressed into the surface. That is the difference between 'this is selected' and 'this is \
         what you are editing', and it is a variant rather than two components.",
    );

    group(rux, &mut pen, "Menu row · accented when chosen");
    let width = area.width().min(320.0);
    for (index, name) in demo.projects.clone().iter().enumerate() {
        let strip = Rect::from_min_size(pen.strip(36.0).min, Vec2::new(width, 36.0));
        if ListItem::new(name, index == demo.project).dot(true).show(rux, strip).clicked {
            demo.project = index;
        }
        pen.gap(2.0);
    }
    pen.gap(GROUP_GAP);

    group(rux, &mut pen, "List row · pressed when chosen, with a trailing action");
    let rows = ["Detailed V1", "Outfit Description", "Simple Generic"];
    for (index, name) in rows.iter().enumerate() {
        let strip = Rect::from_min_size(pen.strip(38.0).min, Vec2::new(width, 38.0));
        let hit = ListItem::new(name, index == demo.list_row)
            .pressed_when_selected()
            .action(Icon::Check, "Apply")
            .pad(Pad::axes(10.0, 12.0))
            .show(rux, strip);
        if hit.clicked {
            demo.list_row = index;
        }
        pen.gap(2.0);
    }
    pen.gap(GROUP_GAP);
    pen.used()
}

fn avatar(rux: &mut Rux<'_>, _demo: &mut Demo, area: Rect) -> f32 {
    let theme = rux.theme();
    let mut pen = Pen::new(area);
    note(
        rux,
        &mut pen,
        "The status dot is a disc of the page's own colour behind a smaller coloured one, which is \
         what `border: 2px solid var(--surface-1); box-sizing: content-box` really means — it reads \
         as a hole punched in the avatar rather than as an outline.",
    );
    group(rux, &mut pen, "States");
    let cells = [("with status", 40.0), ("without", 40.0), ("large · 56", 56.0)];
    let rects = samples(rux, &mut pen, 56.0, &cells);
    Avatar::new("JM").status(theme.accent.mint).name("Jason").show(rux, rects[0]);
    Avatar::new("BR").name("Black Rainbow").show(rux, rects[1]);
    Avatar::new("AI").diameter(56.0).status(theme.accent.amber).show(rux, rects[2]);
    pen.gap(GROUP_GAP);
    pen.used()
}

fn tile(rux: &mut Rux<'_>, _demo: &mut Demo, area: Rect) -> f32 {
    let mut pen = Pen::new(area);
    note(
        rux,
        &mut pen,
        "The dropzone is dark in both themes on purpose: it stands in for a picture, and a picture is \
         not lighter because the interface is. What changes with the theme is the ink drawn over it.",
    );
    group(rux, &mut pen, "Kinds");
    let width = 280.0_f32.min(area.width());
    let height = Tile::height_at(width);
    let cells = [("well · with a spinner", width), ("dropzone · empty", width)];
    let rects = samples(rux, &mut pen, height, &cells);
    Tile::new().kind(TileKind::Well).show(rux, rects[0]);
    Spinner::new().show(rux, Rect::from_center_size(rects[0].center(), Vec2::splat(56.0)));
    Tile::new()
        .kind(TileKind::Dropzone)
        .empty_text("Drop an image", "or click to browse")
        .show(rux, rects[1]);
    pen.gap(GROUP_GAP);
    pen.used()
}

fn modal(rux: &mut Rux<'_>, demo: &mut Demo, area: Rect) -> f32 {
    let theme = rux.theme();
    let mut pen = Pen::new(area);
    note(
        rux,
        &mut pen,
        "The dialog owns the scrim, the frame, the header and the footer; what is in it is the \
         application's. Escape, the close button and a press on the scrim all mean the same thing, so \
         a caller handles one flag rather than three.",
    );
    group(rux, &mut pen, "Open it");
    let button = Button::new("Open the dialog").variant(ButtonVariant::Raised).icon(Icon::Folder);
    let size = button.measure(rux);
    let rect = wide(rux, &mut pen, size.y, "Escape closes it");
    if button.show(rux, Rect::from_min_size(rect.min, size)).clicked() {
        demo.modal_open = true;
    }
    pen.gap(GROUP_GAP);

    if demo.modal_open {
        let (outcome, _) = Modal::new("Prompt Manager")
            .icon(Icon::Folder)
            .icon_tint(theme.accent.amber)
            .size(Vec2::new(640.0, 420.0))
            .footer(true)
            .show(rux, |rux, layout| {
                let body = Pad::all(22.0).inside(layout.body);
                let ink = rux.theme().ink.i700;
                let galley = rux::text::wrapped(
                    rux.ui.painter(),
                    Style::sans(13.0).leading(1.6),
                    "A dialog draws its own frame, header and footer and hands back the rectangle in \
                     between. Everything here is the caller's.",
                    ink,
                    body.width(),
                );
                rux.ui.painter().galley(body.left_top(), galley, ink);

                let foot = Pad::axes(14.0, 22.0).inside(layout.foot);
                let save = Button::new("Save")
                    .variant(ButtonVariant::Mint)
                    .icon(Icon::Check)
                    .size(ButtonSize::Small);
                let size = save.measure(rux);
                save.show(
                    rux,
                    Rect::from_min_size(
                        egui::pos2(foot.right() - size.x, foot.center().y - size.y / 2.0),
                        size,
                    ),
                );
            });
        if outcome.dismissed {
            demo.modal_open = false;
        }
    }
    pen.used()
}

/// Needed so the module's own `Flex` import is not dead while every story lays out with `Pen`.
const _: Option<Flex> = None;
