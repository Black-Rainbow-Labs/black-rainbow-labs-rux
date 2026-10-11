//! The Agent-Tasks design's parts, laid out the way the design canvas's "Agent-Tasks board" artboard lays
//! them out, so the two can be put side by side (`task-2241`).

use egui::{Rect, Vec2};
use rux::components::{
    round_check, search_well, Assignee, CardAction, ChipButton, Lane, LaneMark, PillButton, PillSwitch,
    RoundButton, Select, SelectLead, SelectState, SprintRing, TaskCard, ViewSwitch,
};
use rux::layout::Pen;
use rux::{Icon, Rux};

use super::{group, note, Demo, Story, GROUP_GAP};

pub fn stories() -> Vec<Story> {
    vec![Story {
        section: "Components",
        name: "Board Parts",
        rule: "Claude Design canvas Unluminous, page Agent-Tasks, artboards Agent-Tasks board and Edit task",
        states: &["view switch", "sprint", "search", "lane", "card", "start", "pause", "chip", "priority"],
        draw: parts,
        full_bleed: false,
    }]
}

/// The views, the sprint, a row of lanes with their cards, and the ticket's own controls.
fn parts(rux: &mut Rux<'_>, _demo: &mut Demo, area: Rect) -> f32 {
    let mut pen = Pen::new(area);
    note(
        rux,
        &mut pen,
        "A lane is a well carved into the page and a card is extruded from it. The small raised surfaces \
         are chips: the chosen view, a lane's mark and count, a card's round buttons and the ticket's \
         fields. One blue gradient for the action and the track of work done; mint once it is all done.",
    );
    group(rux, &mut pen, "Header");
    header(rux, &mut pen);
    pen.gap(GROUP_GAP);
    group(rux, &mut pen, "Lanes");
    lanes(rux, &mut pen);
    pen.gap(GROUP_GAP);
    group(rux, &mut pen, "Ticket");
    ticket(rux, &mut pen);
    pen.used()
}

/// The view switch with its words and with only its marks, the sprint ring, the search well and `Add Task`.
fn header(rux: &mut Rux<'_>, pen: &mut Pen) {
    let views = [(Icon::Storyboard, "Board"), (Icon::Layers, "Backlog"), (Icon::Check, "Completed"), (Icon::Cube, "Epics")];
    let row = pen.strip(44.0);
    let switch = ViewSwitch::new(&views, 0);
    let full = switch.measure(rux).x;
    switch.show(rux, Rect::from_min_size(row.left_top(), Vec2::new(full, 44.0)));
    let ring = Rect::from_min_size(row.left_top() + Vec2::new(full + 22.0, 0.0), Vec2::splat(44.0));
    SprintRing::new(0.95).show(rux, ring);
    let well = Rect::from_min_size(ring.right_top() + Vec2::new(22.0, 1.0), Vec2::new(260.0, 42.0));
    search_well(rux, well);
    let add = PillButton::new("Add Task").primary().large().mark(Icon::Plus);
    let size = add.measure(rux);
    add.show(rux, Rect::from_min_size(well.right_top() + Vec2::new(22.0, 0.0), size));
    pen.gap(14.0);
    let row = pen.strip(44.0);
    let marks = ViewSwitch::new(&views, 2).measure_marks(rux).x;
    ViewSwitch::new(&views, 2).show(rux, Rect::from_min_size(row.left_top(), Vec2::new(marks, 44.0)));
    RoundButton::action(Icon::Plus, 16.0, "Add Task")
        .show(rux, Rect::from_min_size(row.left_top() + Vec2::new(marks + 22.0, 0.0), Vec2::splat(44.0)));
}

/// The four lanes of the artboard, with the artboard's own tickets in them.
fn lanes(rux: &mut Rux<'_>, pen: &mut Pen) {
    let strip = pen.strip(380.0);
    let width = ((strip.width() - 22.0 * 3.0) / 4.0).max(240.0);
    let agent = rux.theme().chat.sky;
    let lanes: [(&str, LaneMark, &[(&str, &str, u32, u32, bool)]); 4] = [
        ("New", LaneMark::New, &[("Unluminous: icons for the realm nodes", "task-2231", 0, 0, false)]),
        ("QA failed", LaneMark::Failed, &[]),
        ("In progress", LaneMark::Running, &[("Unluminous Design Updates Prep", "task-2230", 6, 7, true)]),
        ("Agent done", LaneMark::Done, &[("Jupyter notebooks", "task-2220", 9, 9, false), ("Light theme", "task-2215", 5, 5, false)]),
    ];
    for (index, (name, mark, cards)) in lanes.into_iter().enumerate() {
        let lane = Rect::from_min_size(
            strip.left_top() + Vec2::new(index as f32 * (width + 22.0), 0.0),
            Vec2::new(width, strip.height()),
        );
        let layout = Lane::new(name, cards.len(), mark).show(rux, lane);
        let mut top = layout.cards.top();
        for (title, key, done, total, running) in cards {
            let action = match running {
                true => CardAction::Pause,
                false => CardAction::Start,
            };
            let card = TaskCard::new(title, key).todos(*done, *total).assignee(Assignee::Agent(agent)).action(action, "Start Work");
            let height = card.height(rux, layout.cards.width());
            card.show(rux, Rect::from_min_size(egui::pos2(layout.cards.left(), top), Vec2::new(layout.cards.width(), height)));
            top += height + 14.0;
        }
    }
}

/// The ticket modal's controls: the close button, the fold disc, the tick boxes, two chip fields and the
/// priority.
fn ticket(rux: &mut Rux<'_>, pen: &mut Pen) {
    let row = pen.strip(38.0);
    let at = |x: f32, w: f32, h: f32| Rect::from_min_size(row.left_top() + Vec2::new(x, (38.0 - h) / 2.0), Vec2::new(w, h));
    ChipButton::new(Icon::X, 36.0, "Close").mark(12.0, Some(1.8)).show(rux, at(0.0, 36.0, 36.0));
    ChipButton::new(Icon::ChevDown, 24.0, "Todos").folds(true).turn(std::f32::consts::PI).show(rux, at(52.0, 24.0, 24.0));
    round_check(rux, at(92.0, 22.0, 22.0), true, "Export the window marks as SVG");
    round_check(rux, at(124.0, 22.0, 22.0), false, "Carry the redesign back to Rust");
    let statuses = vec!["New".to_owned(), "In progress".to_owned(), "Agent done".to_owned()];
    let mut status = SelectState::default();
    Select::new(&statuses, Some(1)).label("Status").chip(SelectLead::Lane(LaneMark::Running)).show(rux, at(170.0, 234.0, 38.0), &mut status);
    let epics = vec!["general".to_owned()];
    let mut epic = SelectState::default();
    let ink = rux.theme().ink.i400;
    Select::new(&epics, Some(0)).label("Epic").chip(SelectLead::Mark(Icon::Layers, ink)).show(rux, at(420.0, 234.0, 38.0), &mut epic);
    let names = vec!["Low".to_owned(), "Medium".to_owned(), "High".to_owned()];
    PillSwitch::new(&names, Some(1), "Priority").small().show(rux, at(670.0, 234.0, 38.0));
}
