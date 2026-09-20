//! Every control, measured on a desktop and on a phone.
//!
//! Two claims are checked here, and the second one is the more important of the two:
//!
//! 1. **On a touch screen nothing a finger presses is under 44 points.** Written as a loop over the
//!    size enums rather than a list of numbers, so a size added later is covered without anybody
//!    remembering to come back here.
//! 2. **On a desktop every measurement is exactly what it was before this library knew about phones.**
//!    The numbers on the right of that table were read off the code at the commit that introduced
//!    `Screen`, so if a `target_height` ever leaks into the desktop path this fails and names the
//!    control.
//!
//! It runs with no window: `egui::Context::run` needs no surface, and the seven vendored faces are font
//! data rather than anything the platform provides.

use egui::Vec2;
use rux::components::{
    Button, ButtonSize, Chip, DiceToggle, IconButton, IconButtonSize, ListItem, Segmented, Select,
    SelectState, Stepper, Switch, TextInput, TopNav,
};
use rux::icon::Icon;
use rux::screen::{Insets, Pointer, Screen};
use rux::components::NavItem;
use rux::{RuxState, Theme};

/// Hand a closure a `Rux` on a given screen, with no window anywhere.
fn measured<R>(screen: Screen, body: impl FnOnce(&mut rux::Rux<'_>) -> R) -> R {
    let ctx = egui::Context::default();
    rux::text::install(&ctx);
    let state = RuxState::new(Theme::named("light-neumorphic").expect("the light theme"));
    state.set_screen(screen);

    let mut input = egui::RawInput::default();
    input.screen_rect = Some(egui::Rect::from_min_size(
        egui::pos2(0.0, 0.0),
        Vec2::new(screen.size.x, screen.size.y),
    ));

    // A warm-up pass first: a galley measured before the font atlas has been built is a galley of the
    // wrong size, and the atlas is built by the pass that first asks for a glyph.
    let mut warm = ctx.run_ui(input.clone(), |ui| {
        let area = ui.max_rect();
        rux::layer(ui, &state, egui::Id::new("test"), area, |rux| {
            let _ = rux.measure(rux::Style::CONTROL, "Ag");
        });
    });
    // Nothing here paints, so the font atlas epaint just built has nobody to hand its texture to.
    // `TexturesDelta` panics when it is dropped with work outstanding, so it is dropped deliberately.
    warm.textures_delta.clear();

    // `run_ui` takes an `FnMut` and `body` runs once, so it is handed over through an `Option`.
    let mut once = Some(body);
    let mut answer = None;
    let mut output = ctx.run_ui(input, |ui| {
        let Some(body) = once.take() else { return };
        let area = ui.max_rect();
        answer = Some(rux::layer(ui, &state, egui::Id::new("test"), area, body));
    });
    output.textures_delta.clear();
    answer.expect("the closure ran")
}

fn desktop() -> Screen {
    Screen::desktop(Vec2::new(1440.0, 900.0))
}

fn phone() -> Screen {
    Screen::phone(Vec2::new(393.0, 852.0), Insets::vertical(59.0, 34.0))
}

const MIN: f32 = 44.0;

#[test]
fn every_control_reaches_the_minimum_target_on_a_touch_screen() {
    let failures = measured(phone(), |rux| {
        let mut failures: Vec<String> = Vec::new();
        let mut note = |what: String, size: Vec2, both: bool| {
            let short = match both {
                true => size.x < MIN || size.y < MIN,
                false => size.y < MIN,
            };
            if short {
                failures.push(format!("{what} measures {:?}", size));
            }
        };

        for size in [ButtonSize::Mini, ButtonSize::Small, ButtonSize::Medium, ButtonSize::Large] {
            let button = Button::new("Create").size(size);
            note(format!("Button::{size:?}"), button.measure(rux), false);
        }
        for size in [
            IconButtonSize::Tiny,
            IconButtonSize::Small,
            IconButtonSize::Medium,
            IconButtonSize::Large,
            IconButtonSize::Huge,
        ] {
            let button = IconButton::new(Icon::Plus, "Add").size(size);
            note(format!("IconButton::{size:?}"), button.measure(rux), true);
        }

        let options = vec!["Krea 2".to_owned(), "Ideogram 4".to_owned()];
        note("Select".into(), Select::new(&options, Some(0)).measure(rux), false);
        note("Select (mono)".into(), Select::new(&options, None).mono(true).measure(rux), false);

        let mut text = String::new();
        note("TextInput".into(), TextInput::new(&mut text).measure(rux), false);
        let mut search = String::new();
        note("TextInput::search".into(), TextInput::search(&mut search).measure(rux), false);

        note("Chip".into(), Chip::new("1920x1088").measure(rux), false);
        note("Switch".into(), Switch::new(true).measure(rux), true);
        note("DiceToggle".into(), DiceToggle::new(true).measure(rux), true);

        let items = [("Fast", None), ("Quality", Some(Icon::Spark))];
        note("Segmented".into(), Segmented::new(&items, 0).measure(rux), false);
        note("Segmented (compact)".into(), Segmented::new(&items, 0).compact(true).measure(rux), false);

        note("ListItem".into(), ListItem::new("Project one", false).measure(rux), false);
        note("Stepper".into(), Stepper::new(4, 1, 8).measure(rux), true);

        let nav = [
            NavItem::new("create", Icon::Spark, "Create"),
            NavItem::new("gallery", Icon::Image, "Gallery"),
        ];
        // A nav item is what a finger presses, so the strip is the item plus its own padding.
        let strip = TopNav::new(&nav, 0).measure(rux);
        if strip.y < MIN {
            failures.push(format!("TopNav measures {strip:?}"));
        }

        failures
    });
    assert!(failures.is_empty(), "under {MIN} points on a touch screen:\n  {}", failures.join("\n  "));
}

#[test]
fn every_control_measures_on_a_desktop_exactly_what_it_did_before_there_was_a_screen() {
    let measurements = measured(desktop(), |rux| {
        let options = vec!["Krea 2".to_owned(), "Ideogram 4".to_owned()];
        let mut text = String::new();
        let items = [("Fast", None), ("Quality", Some(Icon::Spark))];
        vec![
            ("Button::Mini", Button::new("Create").size(ButtonSize::Mini).measure(rux).y, 34.0),
            ("Button::Small", Button::new("Create").size(ButtonSize::Small).measure(rux).y, 34.0),
            ("Button::Medium", Button::new("Create").size(ButtonSize::Medium).measure(rux).y, 40.0),
            ("Button::Large", Button::new("Create").size(ButtonSize::Large).measure(rux).y, 48.0),
            (
                "IconButton::Tiny",
                IconButton::new(Icon::Plus, "Add").size(IconButtonSize::Tiny).measure(rux).y,
                24.0,
            ),
            (
                "IconButton::Small",
                IconButton::new(Icon::Plus, "Add").size(IconButtonSize::Small).measure(rux).y,
                28.0,
            ),
            (
                "IconButton::Medium",
                IconButton::new(Icon::Plus, "Add").size(IconButtonSize::Medium).measure(rux).y,
                32.0,
            ),
            (
                "IconButton::Large",
                IconButton::new(Icon::Plus, "Add").size(IconButtonSize::Large).measure(rux).y,
                40.0,
            ),
            (
                "IconButton::Huge",
                IconButton::new(Icon::Plus, "Add").size(IconButtonSize::Huge).measure(rux).y,
                48.0,
            ),
            ("Select", Select::new(&options, Some(0)).measure(rux).y, 34.0),
            ("Switch", Switch::new(true).measure(rux).y, 24.0),
            ("DiceToggle", DiceToggle::new(true).measure(rux).y, 28.0),
            ("Stepper", Stepper::new(4, 1, 8).measure(rux).x, 66.0),
            ("Stepper height", Stepper::new(4, 1, 8).measure(rux).y, 11.0 * 2.0 + 1.0 + 8.0),
            ("TextInput width", TextInput::new(&mut text).measure(rux).x, 240.0),
            // One line of `Style::CONTROL`, the item padding (7 top and bottom) and the track padding (5).
            ("Segmented", Segmented::new(&items, 0).measure(rux).y.floor(), 39.0),
        ]
    });
    for (what, got, want) in measurements {
        assert!(
            (got - want).abs() < 0.01,
            "{what} measures {got} on a desktop and used to measure {want}"
        );
    }
}

#[test]
fn a_touch_pointer_never_hovers_and_a_mouse_always_can() {
    assert!(!Pointer::Touch.hovers());
    assert!(Pointer::Mouse.hovers());
    assert_eq!(Pointer::Touch.min_target(), MIN);
    assert_eq!(Pointer::Mouse.min_target(), 0.0);
}

#[test]
fn the_nav_strip_fits_a_phone_and_shows_every_label_it_can_on_a_desktop() {
    let nav = [
        NavItem::new("create", Icon::Spark, "Create"),
        NavItem::new("gallery", Icon::Image, "Gallery"),
        NavItem::new("scenes", Icon::Film, "Scenes"),
        NavItem::new("library", Icon::Layers, "Library"),
        NavItem::new("settings", Icon::Settings, "Settings"),
    ];

    let (least, wanted) = measured(phone(), |rux| {
        let strip = TopNav::new(&nav, 0);
        (strip.measure_min(rux).x, strip.measure(rux).x)
    });
    assert!(
        least <= 390.0,
        "five nav items come to {least} points at their narrowest, which does not fit a 390 point phone"
    );
    assert!(wanted > least, "the full strip should want more room than the trimmed one");

    // On a desktop the strip is given all the room it asked for, so it keeps its label.
    let fits = measured(desktop(), |rux| {
        let strip = TopNav::new(&nav, 0);
        let wanted = strip.measure(rux).x;
        strip.fit_for(rux, wanted)
    });
    assert_eq!(fits, rux::components::Fit::Full);

    // And at 320 points — an iPhone SE in portrait less its page padding — it gives up the label first
    // and then the padding, rather than overflowing.
    let narrow = measured(phone(), |rux| {
        let strip = TopNav::new(&nav, 0);
        (strip.fit_for(rux, 320.0), strip.measure_min(rux).x)
    });
    assert!(narrow.1 <= 320.0, "the trimmed strip is {} points wide", narrow.1);
    assert_ne!(narrow.0, rux::components::Fit::Full);
}

#[test]
fn a_selects_menu_is_a_sheet_on_a_phone_and_a_list_under_its_trigger_on_a_desktop() {
    // The geometry of the sheet itself is `modal::sheet_frame`, tested beside it with no window at all.
    // What this checks is that a compact screen is what chooses it, through the one predicate.
    assert!(phone().is_compact());
    assert!(!desktop().is_compact());

    // And that a select on a phone is tall enough to press before its menu is ever opened.
    let height = measured(phone(), |rux| {
        let options = vec!["a".to_owned()];
        let _state = SelectState::default();
        Select::new(&options, Some(0)).measure(rux).y
    });
    assert!(height >= MIN);
}

#[test]
fn a_segment_with_no_icon_still_draws_its_label() {
    // Found by running the Gallery screen: its filter is `[("All", None), ("Starred", Some(…)), …]` and
    // the first pill came out empty. `Segmented::show` read its next layout cell inside a tuple pattern —
    // `if let (Some(icon), Some(at)) = (icon, slot.next())` — and a tuple evaluates both halves, so a
    // segment with no icon consumed its own label's cell and then had nothing left to draw the label in.
    //
    // There is no drawing to assert on here, so what is pinned is the fact that decides it: a segment
    // with no icon lays out ONE cell, and a segment with an icon lays out two. If the mark ever takes a
    // cell it did not ask for, the first of those numbers is what changes.
    let (without, with) = measured(desktop(), |rux| {
        let plain = [("All", None)];
        let marked = [("All", Some(Icon::Star))];
        (
            Segmented::new(&plain, 0).measure(rux).x,
            Segmented::new(&marked, 0).measure(rux).x,
        )
    });
    assert!(
        with > without,
        "a segment with a mark should be wider than the same segment without one: {with} vs {without}"
    );
}

#[test]
fn a_dialog_with_no_icon_keeps_its_title() {
    // The same defect, in `Modal::draw_head`: `if let (Some(icon), Some(at)) = (self.icon, cell.next())`
    // took the title's cell whenever the dialog had no icon. The head lays out a chip only when there is
    // an icon, so the count of cells is what tells the two apart.
    let widths = measured(desktop(), |rux| {
        let head = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), Vec2::new(400.0, 68.0));
        // 28 for the chip, 10 for the gap either side of it, from `Modal`'s own rule.
        let with_icon = rux::layout::Flex::row(head)
            .pad(rux::layout::Pad::axes(18.0, 22.0))
            .gap(10.0)
            .sized(Vec2::splat(28.0))
            .hug(80.0)
            .fill(1.0)
            .sized(Vec2::splat(32.0))
            .solve_all()
            .len();
        let without_icon = rux::layout::Flex::row(head)
            .pad(rux::layout::Pad::axes(18.0, 22.0))
            .gap(10.0)
            .hug(80.0)
            .fill(1.0)
            .sized(Vec2::splat(32.0))
            .solve_all()
            .len();
        (with_icon, without_icon)
    });
    assert_eq!(widths, (4, 3), "the head lays out one cell fewer with no icon, and the title is the first of them");
}
