//! Every component actually drawn, in a debug build, with no window.
//!
//! **This exists because a release build hid a crash for a whole ticket.** `epaint::Mesh::colored_vertex`
//! carries a `debug_assert` that the mesh has no texture, and both places in `rux::icon` that draw a
//! *turned* mark called it and then wrote the texture coordinate over the vertex afterwards. That
//! produces the right picture with assertions off, and kills the process with them on:
//!
//! ```text
//! thread '<unnamed>' panicked at epaint-0.36.2/src/mesh.rs:170:
//! Mesh has an assigned texture
//! ```
//!
//! Every test in this repository ran the geometry rather than the drawing, and every program that ran
//! was built `--release`, so nothing had ever executed that line with assertions on. The Android
//! application's first frame did — a select's chevron, a stepper's chevron and a spinner's head are all
//! turned marks — and the process died before anything reached the screen.
//!
//! So: this draws each component for real. `cargo test` is a debug build, which is the whole point.

use egui::{Rect, Vec2};
use rux::components::{
    Avatar, Button, ButtonSize, Chip, DiceToggle, Divider, IconButton, IconButtonSize, ListItem,
    Modal, ModalOutcome, Panel, Progress, Segmented, Select, SelectState, Skeleton, Spinner,
    Stepper, SubGroup, Switch, TextArea, TextInput, Tile, TopNav,
};
use rux::components::NavItem;
use rux::icon::{Icon, Mark};
use rux::screen::{Insets, Screen};
use rux::{RuxState, Theme};

/// Draw `body` for real, on a given screen and theme, with no window.
fn drawn(screen: Screen, theme: &str, body: impl FnOnce(&mut rux::Rux<'_>)) {
    let ctx = egui::Context::default();
    rux::text::install(&ctx);
    let state = RuxState::new(Theme::named(theme).expect("a theme"));
    state.set_screen(screen);

    let mut input = egui::RawInput::default();
    input.screen_rect = Some(Rect::from_min_size(egui::pos2(0.0, 0.0), screen.size));

    let mut once = Some(body);
    // Three passes: the first builds the font atlas, the second draws with it, and the third lets a
    // popup's own layer be laid out where the frame before it said it would be.
    for _ in 0..3 {
        let mut output = ctx.run_ui(input.clone(), |ui| {
            let area = ui.max_rect();
            rux::layer(ui, &state, egui::Id::new("draw"), area, |rux| {
                let _ = rux.measure(rux::Style::CONTROL, "Ag");
                if let Some(body) = once.take() {
                    body(rux);
                }
            });
        });
        // Nothing here paints to a screen, so the atlas epaint built has nobody to hand its texture to.
        output.textures_delta.clear();
    }
    state.end_frame();
}

fn desktop() -> Screen {
    Screen::desktop(Vec2::new(1440.0, 900.0))
}

fn phone() -> Screen {
    Screen::phone(Vec2::new(393.0, 852.0), Insets::vertical(59.0, 34.0))
}

/// Everything, drawn into one column. The rectangles overlap and that does not matter: what is being
/// proved is that each one's drawing code runs.
fn every_component(rux: &mut rux::Rux<'_>) {
    let at = |y: f32, w: f32, h: f32| Rect::from_min_size(egui::pos2(20.0, y), Vec2::new(w, h));

    rux.ground(Rect::from_min_size(egui::pos2(0.0, 0.0), Vec2::new(400.0, 900.0)));

    for (index, size) in
        [ButtonSize::Mini, ButtonSize::Small, ButtonSize::Medium, ButtonSize::Large]
            .into_iter()
            .enumerate()
    {
        let button = Button::new("Create").icon(Icon::Spark).trailing(Icon::ArrowRight).size(size);
        let want = button.measure(rux);
        button.show(rux, at(20.0 + index as f32 * 60.0, want.x, want.y));
    }

    for (index, size) in [
        IconButtonSize::Tiny,
        IconButtonSize::Small,
        IconButtonSize::Medium,
        IconButtonSize::Large,
        IconButtonSize::Huge,
    ]
    .into_iter()
    .enumerate()
    {
        let button = IconButton::new(Icon::Plus, "Add").size(size);
        let want = button.measure(rux);
        button.show(rux, at(280.0 + index as f32 * 56.0, want.x, want.y));
        // And the same mark turned over, which is the drawing that used to kill the process.
        IconButton::new(Icon::ChevDown, "Collapse")
            .size(size)
            .turn(std::f32::consts::PI)
            .show(rux, at(280.0 + index as f32 * 56.0, want.x, want.y).translate(Vec2::new(60.0, 0.0)));
    }

    // A bare turned mark, and one at every quarter turn, because the rotation is what was broken.
    for (index, turn) in [0.0_f32, 1.5707964, 3.1415927, 4.712389].into_iter().enumerate() {
        rux.mark(
            Mark::new(Icon::ChevDown, 13.0).stroke(2.0).turn(turn),
            egui::pos2(300.0 + index as f32 * 20.0, 30.0),
            rux.theme().ink.i500,
        );
    }

    let options = vec!["Krea 2".to_owned(), "Ideogram 4".to_owned(), "MiniMax H3".to_owned()];
    let mut open = SelectState { open: true, scroll: 0.0 };
    let want = Select::new(&options, Some(0)).measure(rux);
    // An open select: its trigger's chevron is turned by 180 degrees, and its menu is a whole nested
    // layer — which on a compact screen is a sheet.
    Select::new(&options, Some(0)).show(rux, at(580.0, 220.0, want.y), &mut open);

    let mut shut = SelectState::default();
    Select::new(&options, None).mono(true).show(rux, at(640.0, 220.0, want.y), &mut shut);

    let mut text = "Black Rainbow".to_owned();
    let want = TextInput::new(&mut text).measure(rux);
    TextInput::new(&mut text).icon(Icon::Search).show(rux, at(700.0, 240.0, want.y));
    let mut search = String::new();
    TextInput::search(&mut search).show(rux, at(760.0, 240.0, want.y));

    let mut prose = "a woman doing yoga at sunrise".to_owned();
    TextArea::new(&mut prose)
        .hint("Describe the image…")
        .count(true)
        .corner(Icon::Wand)
        .show(rux, at(820.0, 260.0, 180.0));

    // The stepper: two turned chevrons on a desktop, two more on a phone, in the wide form.
    let want = Stepper::new(4, 1, 8).measure(rux);
    Stepper::new(4, 1, 8).show(rux, at(1020.0, want.x, want.y));
    Stepper::new(1, 1, 8).show(rux, at(1080.0, want.x, want.y));
    Stepper::new(8, 1, 8).show(rux, at(1140.0, want.x, want.y));

    Switch::new(true).show(rux, at(1200.0, 60.0, 60.0));
    Switch::new(false).show(rux, at(1260.0, 60.0, 60.0));
    DiceToggle::new(true).show(rux, at(1320.0, 60.0, 60.0));
    DiceToggle::new(false).show(rux, at(1380.0, 60.0, 60.0));

    let items = [("Fast", None), ("Quality", Some(Icon::Spark))];
    let want = Segmented::new(&items, 0).measure(rux);
    Segmented::new(&items, 0).show(rux, at(1440.0, want.x, want.y));
    Segmented::new(&items, 1).compact(true).show(rux, at(1500.0, want.x, want.y));

    Chip::new("1280 x 704").mono(true).dismissible(true).icon(Icon::Image).show(
        rux,
        at(1560.0, 180.0, Chip::new("x").measure(rux).y),
    );

    ListItem::new("Black Rainbow Labs", true).tick(true).dot(true).show(
        rux,
        at(1620.0, 260.0, ListItem::new("Ag", false).measure(rux).y),
    );
    rux::components::menu_heading(rux, at(1680.0, 260.0, 20.0), "Projects");

    // A spinner's head is a turned arc: the other half of the same defect.
    Spinner::new().show(rux, at(1720.0, 60.0, 60.0));
    Spinner::new().diameter(20.0).bare().show(rux, at(1790.0, 20.0, 20.0));
    Progress::new(0.4).show(rux, at(1820.0, 200.0, 8.0));
    Progress::indeterminate().show(rux, at(1840.0, 200.0, 8.0));
    Skeleton::new().show(rux, at(1860.0, 200.0, 40.0));

    Avatar::new("JM").show(rux, at(1920.0, 36.0, 36.0));
    Divider::new().faded().show(rux, at(1980.0, 300.0, 1.0));
    Tile::new().empty_text("Nothing yet", "Press Create").show(rux, at(2000.0, 300.0, 170.0));
    Panel::new().show(rux, at(2200.0, 300.0, 120.0));

    let head = SubGroup::new("Qualities", Icon::Spark, rux.theme().accent.violet, true).first(true);
    let height = head.head_height(rux);
    head.show(rux, at(2340.0, 300.0, height));
    // And shut, which turns the chevron by a quarter.
    SubGroup::new("Camera", Icon::Film, rux.theme().accent.amber, false)
        .show(rux, at(2380.0, 300.0, height));

    let nav = [
        NavItem::new("create", Icon::Spark, "Create"),
        NavItem::new("gallery", Icon::Image, "Gallery"),
        NavItem::new("scenes", Icon::Film, "Scenes"),
    ];
    let want = TopNav::new(&nav, 0).measure(rux);
    TopNav::new(&nav, 0).show(rux, at(2440.0, want.x, want.y));
    // And squeezed, so the fit that drops the label is drawn too.
    TopNav::new(&nav, 0).show(rux, at(2500.0, 200.0, want.y));
}

#[test]
fn every_component_draws_on_a_desktop() {
    drawn(desktop(), "light-neumorphic", every_component);
    drawn(desktop(), "dark-neumorphic", every_component);
}

#[test]
fn every_component_draws_on_a_phone() {
    drawn(phone(), "dark-neumorphic", every_component);
    drawn(phone(), "light-neumorphic", every_component);
}

#[test]
fn a_dialog_draws_as_a_dialog_and_as_a_sheet() {
    for screen in [desktop(), phone()] {
        drawn(screen, "dark-neumorphic", |rux| {
            let (_, _) = Modal::new("Prompts")
                .icon(Icon::Folder)
                .footer(true)
                .show(rux, |rux, outcome: ModalOutcome| {
                    let (left, right) = rux::components::split_body(rux, outcome.body, 260.0);
                    let _ = (left, right);
                });
            // The same dialog with no icon at all, which is the case that lost its title.
            let (_, _) = Modal::new("No icon").show(rux, |_rux, _outcome: ModalOutcome| {});
        });
    }
}

/// A select drawn at twice the stylesheet's size measures twice as wide and draws, open, with a
/// quieter menu shadow — what the Unluminous chat pane asks of it when the pane is zoomed.
#[test]
fn a_zoomed_select_is_measured_larger_and_draws_open() {
    drawn(desktop(), "dark-neumorphic", |rux| {
        let options = vec!["claude".to_owned(), "codex".to_owned()];
        let plain = Select::new(&options, Some(0)).measure(rux);
        let zoomed = Select::new(&options, Some(0)).zoom(2.0).measure(rux);
        assert!(zoomed.x > plain.x * 1.8, "{zoomed:?} against {plain:?}");
        assert!(zoomed.y > plain.y * 1.8, "{zoomed:?} against {plain:?}");
        let quiet = rux.theme().elevation.raised_sm;
        let mut open = SelectState { open: true, scroll: 0.0 };
        Select::new(&options, Some(0)).zoom(2.0).menu_elevation(quiet).show(
            rux,
            Rect::from_min_size(egui::pos2(20.0, 20.0), zoomed),
            &mut open,
        );
    });
}

/// The instrument controls `task-2211` added for Unluminous's chat pane, each drawn for real, at the
/// reference size and zoomed, in both themes, on both screens.
fn every_instrument(rux: &mut rux::Rux<'_>) {
    use rux::components::{
        instrument, Align, Chart, ChartKind, Checkbox, Column, Fader, Item, Key, Meter, Series,
        Stage, Table, Timeline,
    };
    let at = |y: f32, w: f32, h: f32| Rect::from_min_size(egui::pos2(20.0, y), Vec2::new(w, h));
    let inside = instrument::plate(rux, at(0.0, 360.0, 900.0), 14.0, 14.0);
    let _ = inside;
    instrument::screen(rux, at(10.0, 300.0, 60.0), 9.0, true);
    let blue = rux.theme().accent.blue;
    let size = Key::new("Make the change").led(blue, true).measure(rux);
    Key::new("Make the change")
        .led(blue, true)
        .show(rux, at(80.0, size.x, size.y));
    Key::new("main")
        .down(true)
        .compact()
        .show(rux, at(120.0, 80.0, 24.0));
    let tick = Checkbox::new(true, "Commit the task's own work");
    let height = tick.height_at(rux, 300.0);
    tick.show(rux, at(150.0, 300.0, height));
    Checkbox::new(false, "Run the window suite").show(rux, at(180.0, 300.0, 20.0));
    let fader = Fader::new(egui::Id::new("fader"), 5.0, 0.0, 12.0).step(0.5);
    fader.show(rux, at(210.0, 300.0, Fader::height(rux)));
    Meter::new(0.85)
        .top(rux.theme().accent.amber)
        .id(egui::Id::new("meter"))
        .show(rux, at(250.0, 300.0, Meter::height(rux)));
    let labels = vec!["core".to_owned(), "app".to_owned(), "cli".to_owned()];
    let format = |v: f64| format!("{v:.0}");
    for (index, kind) in [
        ChartKind::Bar,
        ChartKind::Line,
        ChartKind::Area,
        ChartKind::Donut,
    ]
    .into_iter()
    .enumerate()
    {
        let chart = Chart::new(egui::Id::new(("chart", index)), kind, &labels, &format)
            .series(Series {
                name: "last week",
                values: &[41.0, 81.0, 9.0],
                colour: blue,
            })
            .series(Series {
                name: "this week",
                values: &[38.0, 131.0, 9.0],
                colour: rux.theme().accent.mint,
            })
            .stacked(index == 2);
        let height = chart.height_at(rux, 320.0);
        chart.show(rux, at(280.0 + index as f32 * 10.0, 320.0, height));
    }
    let columns = [
        Column {
            label: "Binary",
            align: Align::Left,
        },
        Column {
            label: "Tests",
            align: Align::Right,
        },
    ];
    let rows = vec![
        vec!["canvas_realm".to_owned(), "97".to_owned()],
        vec!["chat".to_owned(), "142".to_owned()],
    ];
    let table = Table::new(egui::Id::new("table"), &columns, &rows).sorted(Some((1, true)));
    let height = table.height_at(rux, 320.0);
    table.show(rux, at(600.0, 320.0, height));
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
            text: "483 pictures",
            stage: Stage::Active,
        },
        Item {
            time: "",
            title: "Publish",
            text: "",
            stage: Stage::Todo,
        },
    ];
    let line = Timeline::new(egui::Id::new("timeline"), &items);
    let height = line.height_at(rux, 320.0);
    line.show(rux, at(720.0, 320.0, height));
}

#[test]
fn every_instrument_draws_in_both_themes_on_both_screens_and_zoomed() {
    for theme in ["dark-neumorphic", "light-neumorphic"] {
        for screen in [desktop(), phone()] {
            drawn(screen, theme, every_instrument);
        }
    }
    // Zoomed, which is how Unluminous's chat pane draws them at a size other than the reference.
    let ctx = egui::Context::default();
    rux::text::install(&ctx);
    let state = RuxState::new(Theme::named("dark-neumorphic").expect("a theme"));
    state.set_zoom(1.75);
    state.set_still(true);
    let mut input = egui::RawInput::default();
    input.screen_rect = Some(Rect::from_min_size(
        egui::pos2(0.0, 0.0),
        Vec2::new(900.0, 1400.0),
    ));
    for _ in 0..2 {
        let mut output = ctx.run_ui(input.clone(), |ui| {
            let area = ui.max_rect();
            rux::layer(ui, &state, egui::Id::new("zoomed"), area, |rux| {
                assert_eq!(rux.z(10.0), 17.5);
                every_instrument(rux);
            });
        });
        output.textures_delta.clear();
    }
}

/// The Agent-Chat design's parts (`task-2235`): every button in every state, the status wells, the
/// charts and the cards.
fn every_chat_part(rux: &mut rux::Rux<'_>) {
    use rux::components::{
        AreaChart, CodeBlock, CommandCard, Dial, DiffCard, ErrorCard, Line, PermissionCard,
        PillButton, PillVariant, Ring, RoundButton, RoundSize, Sign, Slice, SlicePaint, Status,
        StatusWell, Track,
    };
    let at = |x: f32, y: f32, w: f32, h: f32| Rect::from_min_size(egui::pos2(x, y), Vec2::new(w, h));
    rux.ground(at(0.0, 0.0, 1400.0, 900.0));
    let states = [(false, false, false, true), (true, false, false, true), (false, true, false, true), (false, false, true, true), (false, false, false, false)];
    for (index, (hovered, held, ring, enabled)) in states.into_iter().enumerate() {
        let x = 20.0 + index as f32 * 150.0;
        for (row, variant) in [PillVariant::Primary, PillVariant::Secondary, PillVariant::Ghost].into_iter().enumerate() {
            let pill = PillButton::new("Apply change").variant(variant).enabled(enabled).showing(hovered, held, ring);
            let size = pill.measure(rux);
            pill.show(rux, at(x, 20.0 + row as f32 * 50.0, size.x, size.y));
        }
        RoundButton::mark(Icon::Clock, 20.0, "History").enabled(enabled).showing(hovered, held, ring).show(rux, at(x, 170.0, 38.0, 38.0));
        RoundButton::mark(Icon::Settings, 13.0, "Settings").size(RoundSize::Small).on(held).enabled(enabled).show(rux, at(x, 220.0, 30.0, 30.0));
        RoundButton::send("Send").enabled(enabled).showing(hovered, held, ring).show(rux, at(x, 260.0, 44.0, 44.0));
        RoundButton::stop("Stop").show(rux, at(x + 60.0, 260.0, 44.0, 44.0));
    }
    for (index, status) in [Status::Waiting, Status::Running, Status::Done, Status::NeedsOk, Status::Failed].into_iter().enumerate() {
        StatusWell::new(status).show(rux, at(20.0 + index as f32 * 50.0, 320.0, 34.0, 34.0));
    }
    let chat = rux.theme().chat;
    Dial::new(128.0, 50.0).centre("178", " s").caption("+39%", chat.slower).show(rux, at(20.0, 370.0, 150.0, 150.0));
    Ring::new(vec![Slice::new(131.0, SlicePaint::Action), Slice::new(38.0, SlicePaint::Colour(chat.slices[0])), Slice::new(1.0, SlicePaint::Colour(chat.slices[1]))])
        .centre("74%", "")
        .show(rux, at(200.0, 370.0, 150.0, 150.0));
    Track::new(131.0, 140.0).marker(81.0).show(rux, at(380.0, 380.0, 300.0, 22.0));
    Track::new(0.0, 140.0).without_knob().show(rux, at(380.0, 420.0, 300.0, 22.0));
    let days = [119.0, 121.0, 118.0, 124.0, 151.0, 170.0, 178.0];
    let chart = AreaChart::new(&days).marker(4, "rux added").ends("26 Sep", "9 Oct");
    let size = chart.measure(rux);
    chart.show(rux, at(380.0, 460.0, 360.0, size.y));
    let lines = [Line::plain("cargo build --release").trailing("2m 58s").number("1").sign(Sign::Added)];
    let card = CommandCard::new("cargo build", "0:47", Status::Running).progress(Some(0.36));
    let height = card.height(rux);
    card.show(rux, at(780.0, 20.0, 380.0, height));
    let card = CommandCard::new("Ran 1 command", "3m", Status::Done).words().lines(&lines, true);
    let height = card.height(rux);
    card.show(rux, at(780.0, 120.0, 380.0, height));
    let card = CodeBlock::new("icon.rs", "Rust", &lines);
    let height = card.height(rux);
    card.show(rux, at(780.0, 260.0, 380.0, height));
    let card = DiffCard::new("icon.rs", "1 line changed", &lines);
    let height = card.height(rux);
    card.show(rux, at(780.0, 400.0, 380.0, height));
    let card = PermissionCard::new("claude wants to run a command", "It changes Cargo.lock", &lines);
    let height = card.height(rux);
    card.show(rux, at(780.0, 560.0, 380.0, height));
    let models = vec!["claude".to_owned(), "codex".to_owned(), "local qwen 27B".to_owned()];
    rux::components::PillSwitch::new(&models, Some(1), "Model").show(rux, at(20.0, 560.0, 380.0, 46.0));
    let card = ErrorCard::new("cargo exited with code 101", "cargo build", &lines);
    let height = card.height(rux);
    card.show(rux, at(780.0, 720.0, 380.0, height));
}

#[test]
fn every_chat_part_draws_in_both_themes_on_both_screens() {
    for theme in ["dark-neumorphic", "light-neumorphic"] {
        for screen in [desktop(), phone()] {
            drawn(screen, theme, every_chat_part);
        }
    }
}
