//! The Agent-Chat design's parts, laid out the way the design canvas's "Chat parts" artboard lays them
//! out, so the two can be put side by side (`task-2235`).

use egui::{Pos2, Rect, Vec2};
use rux::components::{
    AreaChart, CodeBlock, CommandCard, Dial, DiffCard, ErrorCard, Line, PermissionCard, PillButton,
    PillVariant, Ring, RoundButton, RoundSize, Sign, Slice, SlicePaint, Status, StatusWell, Track,
};
use rux::layout::Pen;
use rux::text::Style;
use rux::{Icon, Rux};

use super::{group, note, Demo, Story, GROUP_GAP};

pub fn stories() -> Vec<Story> {
    vec![Story {
        section: "Components",
        name: "Chat Parts",
        rule: "Claude Design canvas Unluminous, page Agent-Chat, artboard Chat parts",
        states: &["rest", "hover", "pressed", "focus", "disabled", "dial", "ring", "track", "card"],
        draw: parts,
        full_bleed: false,
    }]
}

/// The canvas's 1320 points of content, drawn at whatever width the story has.
fn parts(rux: &mut Rux<'_>, _demo: &mut Demo, area: Rect) -> f32 {
    let mut pen = Pen::new(area);
    note(
        rux,
        &mut pen,
        "One light, top left. Cards and controls are extruded from the pane; wells and tracks are \
         carved into it. Everything pressed is round or a pill, state is an icon in a carved well, \
         and an amount is a dial, a ring or a slider.",
    );
    // The charts first, because the ticket's check is about them: arcs that do not touch at the joins
    // and a smallest slice in proportion to its share.
    group(rux, &mut pen, "Charts");
    charts(rux, &mut pen);
    pen.gap(GROUP_GAP);
    group(rux, &mut pen, "Buttons");
    buttons(rux, &mut pen);
    pen.gap(GROUP_GAP);
    group(rux, &mut pen, "Status");
    statuses(rux, &mut pen);
    pen.gap(GROUP_GAP);
    group(rux, &mut pen, "Cards");
    cards(rux, &mut pen);
    pen.used()
}

/// Six rows of five states: primary, secondary, ghost, round, settings and send.
fn buttons(rux: &mut Rux<'_>, pen: &mut Pen) {
    let theme = rux.theme();
    let column = 150.0;
    let label = 100.0;
    let head = pen.strip(18.0);
    for (index, word) in ["Rest", "Hover", "Pressed", "Focus", "Disabled"].iter().enumerate() {
        let galley = rux.text(Style::sans(11.0), word, theme.ink.i400);
        let at = Pos2::new(head.left() + label + index as f32 * column, head.top());
        rux.painter().galley(at, galley, theme.ink.i400);
    }
    pen.gap(12.0);
    let states = [(false, false, false, true), (true, false, false, true), (false, true, false, true), (false, false, true, true), (false, false, false, false)];
    let rows: [&str; 6] = ["Primary", "Secondary", "Ghost", "Round", "Settings", "Send"];
    for (row, name) in rows.iter().enumerate() {
        let strip = pen.strip(44.0);
        let galley = rux.text(Style::sans(12.0), name, theme.ink.i500);
        rux::text::draw_left_centre(rux.painter(), Pos2::new(strip.left(), strip.center().y), galley, theme.ink.i500);
        for (index, (hovered, held, ring, enabled)) in states.into_iter().enumerate() {
            let left = strip.left() + label + index as f32 * column;
            let cell = Rect::from_min_size(Pos2::new(left, strip.top()), Vec2::new(column - 16.0, strip.height()));
            match row {
                0..=2 => {
                    let (text, variant) = match row {
                        0 => ("Apply change", PillVariant::Primary),
                        1 => ("Open in editor", PillVariant::Secondary),
                        _ => ("Copy", PillVariant::Ghost),
                    };
                    let pill = PillButton::new(text).variant(variant).enabled(enabled).showing(hovered, held, ring);
                    let size = pill.measure(rux);
                    pill.show(rux, Rect::from_min_size(Pos2::new(left, cell.center().y - size.y / 2.0), size));
                }
                3 => {
                    let rect = Rect::from_min_size(Pos2::new(left, cell.center().y - 19.0), Vec2::splat(38.0));
                    RoundButton::mark(Icon::Clock, 20.0, "History").enabled(enabled).showing(hovered, held, ring).show(rux, rect);
                }
                4 => {
                    let rect = Rect::from_min_size(Pos2::new(left, cell.center().y - 15.0), Vec2::splat(30.0));
                    RoundButton::mark(Icon::Settings, 13.0, "Chat settings")
                        .size(RoundSize::Small)
                        .on(held)
                        .enabled(enabled)
                        .showing(hovered, false, ring)
                        .show(rux, rect);
                }
                _ => {
                    let rect = Rect::from_min_size(Pos2::new(left, cell.center().y - 22.0), Vec2::splat(44.0));
                    RoundButton::send("Send").enabled(enabled).showing(hovered, held, ring).show(rux, rect);
                }
            }
        }
        pen.gap(14.0);
    }
}

/// The five states in their wells, each with its words.
fn statuses(rux: &mut Rux<'_>, pen: &mut Pen) {
    let theme = rux.theme();
    let states = [
        (Status::Waiting, "Queued behind the current step"),
        (Status::Running, "A command or a reply in progress"),
        (Status::Done, "Finished without an error"),
        (Status::NeedsOk, "Paused until you allow it"),
        (Status::Failed, "Stopped; the error says why"),
    ];
    let area = pen.rest();
    let width = 240.0;
    let strip = pen.strip(40.0);
    for (index, (status, detail)) in states.into_iter().enumerate() {
        let left = area.left() + index as f32 * width;
        let well = Rect::from_min_size(Pos2::new(left, strip.top() + 3.0), Vec2::splat(34.0));
        StatusWell::new(status).show(rux, well);
        let title = rux.text(Style::sans(12.5), status.label(), theme.ink.i900);
        let line = rux.text(Style::sans(11.0), detail, theme.ink.i400);
        let top = well.center().y - (title.size().y + 1.0 + line.size().y) / 2.0;
        rux.painter().galley(Pos2::new(well.right() + 14.0, top), title.clone(), theme.ink.i900);
        rux.painter().galley(Pos2::new(well.right() + 14.0, top + title.size().y + 1.0), line, theme.ink.i400);
    }
}

/// The dial, the ring chart, the tracks and the area chart, each on a card as the canvas sets them.
fn charts(rux: &mut Rux<'_>, pen: &mut Pen) {
    let theme = rux.theme();
    let chat = theme.chat;
    let area = pen.rest();
    let half = ((area.width() - 28.0) / 2.0).min(480.0);
    let top = pen.strip(190.0);
    let left_card = Rect::from_min_size(top.min, Vec2::new(half, 190.0));
    let right_card = Rect::from_min_size(Pos2::new(top.left() + half + 28.0, top.top()), Vec2::new(half, 190.0));
    for card in [left_card, right_card] {
        rux.chrome.surface(card, 26.0, rux::Fill::gradient(chat.card, card), chat.raised());
    }
    let dial = Rect::from_min_size(left_card.min + Vec2::new(20.0, 20.0), Vec2::splat(150.0));
    Dial::new(128.0, 50.0)
        .centre("178", " s")
        .caption("+39%", chat.slower)
        .label("Total release build 178 seconds: 128 seconds last week plus 50 seconds added")
        .show(rux, dial);
    legend(rux, Pos2::new(dial.right() + 22.0, dial.top() + 18.0), &[("Last week", "128 s", false), ("Added this week", "50 s", true)], "Release build");
    let ring = Rect::from_min_size(right_card.min + Vec2::new(20.0, 20.0), Vec2::splat(150.0));
    Ring::new(vec![
        Slice::new(131.0, SlicePaint::Action),
        Slice::new(38.0, SlicePaint::Colour(chat.slices[0])),
        Slice::new(9.0, SlicePaint::Colour(chat.slices[1])),
    ])
    .centre("74%", "")
    .caption("app", theme.ink.i400)
    .label("Where this week's 178 seconds went: app 131, core 38, cli 9")
    .show(rux, ring);
    let rows = [
        rux::components::LegendRow { name: "unluminous-app".into(), value: "131 s".into(), paint: SlicePaint::Action },
        rux::components::LegendRow { name: "unluminous-core".into(), value: "38 s".into(), paint: SlicePaint::Colour(chat.slices[0]) },
        rux::components::LegendRow { name: "unluminous-cli".into(), value: "9 s".into(), paint: SlicePaint::Colour(chat.slices[1]) },
    ];
    let legend = rux::components::Legend::new(&rows);
    let height = legend.height(rux);
    let galley = rux.text(Style::sans(12.0), "Where the time goes", theme.ink.i500);
    rux.painter().galley(Pos2::new(ring.right() + 22.0, ring.top() + 24.0), galley, theme.ink.i500);
    let rect = Rect::from_min_size(Pos2::new(ring.right() + 22.0, ring.top() + 52.0), Vec2::new(right_card.right() - ring.right() - 44.0, height));
    legend.show(rux, rect);
    pen.gap(28.0);
    let bottom = pen.strip(250.0);
    let tracks = Rect::from_min_size(bottom.min, Vec2::new(half, 250.0));
    rux.chrome.surface(tracks, 26.0, rux::Fill::gradient(chat.card, tracks), chat.raised());
    let crates: [(&str, f32, f32); 3] = [("unluminous-app", 81.0, 131.0), ("unluminous-core", 41.0, 38.0), ("unluminous-cli", 6.0, 9.0)];
    for (index, (name, last, now)) in crates.into_iter().enumerate() {
        let row = tracks.top() + 20.0 + index as f32 * 66.0;
        let delta = (now - last) / last * 100.0;
        let colour = if now > last { chat.slower } else { chat.faster };
        let galley = rux.text(Style::sans(12.5), name, theme.ink.i900);
        rux.painter().galley(Pos2::new(tracks.left() + 22.0, row), galley, theme.ink.i900);
        let numbers = format!("{last:.0} s \u{2192}  {now:.0} s   {}{:.0}%", if delta > 0.0 { "+" } else { "\u{2212}" }, delta.abs());
        let galley = rux.text(Style::sans(11.0), &numbers, colour);
        let at = Pos2::new(tracks.right() - 22.0 - galley.size().x, row);
        rux.painter().galley(at, galley, colour);
        let track = Rect::from_min_size(Pos2::new(tracks.left() + 22.0, row + 24.0), Vec2::new(tracks.width() - 44.0, 22.0));
        Track::new(now, 140.0).marker(last).label(name).show(rux, track);
    }
    let line = Rect::from_min_size(Pos2::new(bottom.left() + half + 28.0, bottom.top()), Vec2::new(half, 250.0));
    rux.chrome.surface(line, 26.0, rux::Fill::gradient(chat.card, line), chat.raised());
    let days = [119.0, 121.0, 118.0, 124.0, 122.0, 126.0, 125.0, 131.0, 129.0, 134.0, 151.0, 170.0, 176.0, 178.0];
    let chart = AreaChart::new(&days).range(110.0, 185.0).marker(10, "rux added").ends("26 Sep", "9 Oct");
    let size = chart.measure(rux);
    let inner = Rect::from_min_size(line.min + Vec2::new(22.0, 50.0), Vec2::new(line.width() - 44.0, size.y));
    let galley = rux.text(Style::sans(12.0), "Release build \u{00B7} 14 days", theme.ink.i500);
    rux.painter().galley(line.min + Vec2::new(22.0, 22.0), galley, theme.ink.i500);
    chart.label("Release build over the last 14 days").show(rux, inner);
    pen.gap(28.0);
    // The readouts: a number, how it changed and a sparkline of the week, each on a card of its own.
    let strip = pen.strip(150.0);
    let each = ((strip.width() - 28.0) / 3.0).min(200.0);
    let readouts: [(&str, &str, &str, [f32; 7], bool); 3] = [
        ("Total", "178", "+39%", [126.0, 131.0, 129.0, 134.0, 151.0, 170.0, 178.0], true),
        ("App", "131", "+62%", [81.0, 84.0, 83.0, 88.0, 104.0, 122.0, 131.0], true),
        ("Core", "38", "\u{2212}7%", [41.0, 41.0, 40.0, 40.0, 39.0, 38.0, 38.0], false),
    ];
    for (index, (label, value, delta, spark, up)) in readouts.iter().enumerate() {
        let colour = if *up { chat.slower } else { chat.faster };
        let readout = rux::components::Readout::new(label, value, " s").delta(delta, colour).spark(spark);
        let height = readout.height(rux);
        let rect = Rect::from_min_size(Pos2::new(strip.left() + index as f32 * (each + 14.0), strip.top()), Vec2::new(each, height));
        readout.show(rux, rect);
    }
    pen.gap(28.0);
    // The thinking pill, the compacted note and the file chips of a run of reads.
    let row = pen.strip(42.0);
    let pill = rux::components::ThinkingPill::new("claude is thinking", "0:04", rux.theme().chat.sky);
    let size = pill.measure(rux);
    pill.show(rux, Rect::from_min_size(row.min, size));
    let note = rux::components::NotePill::new("Context compacted \u{b7} 41k \u{2192} 9k tokens");
    let note_size = note.measure(rux);
    note.show(rux, Rect::from_min_size(Pos2::new(row.left() + size.x + 28.0, row.center().y - note_size.y / 2.0), note_size));
    pen.gap(20.0);
    let chips = pen.strip(30.0);
    let mut x = chips.left();
    for name in ["Cargo.toml", "Cargo.lock", "theme/mod.rs", "theme/icon.rs"] {
        let chip = rux::components::FileChip::new(name);
        let size = chip.measure(rux);
        chip.show(rux, Rect::from_min_size(Pos2::new(x, chips.top()), size));
        x += size.x + 8.0;
    }
}

/// The words beside a dial: a label, then each series' swatch, name and value.
fn legend(rux: &mut Rux<'_>, at: Pos2, rows: &[(&str, &str, bool)], title: &str) {
    let theme = rux.theme();
    let chat = theme.chat;
    let galley = rux.text(Style::sans(12.0), title, theme.ink.i500);
    rux.painter().galley(at, galley, theme.ink.i500);
    for (index, (name, value, action)) in rows.iter().enumerate() {
        let top = at.y + 30.0 + index as f32 * 50.0;
        let swatch = Rect::from_min_size(Pos2::new(at.x, top + 5.0), Vec2::new(14.0, 4.0));
        match action {
            true => rux.chrome.rect(swatch, 2.0, rux::Fill::gradient(chat.action, swatch)),
            false => rux.chrome.rect(swatch, 2.0, chat.quiet),
        }
        let galley = rux.text(Style::sans(11.0), name, theme.ink.i400);
        rux.painter().galley(Pos2::new(at.x + 22.0, top), galley, theme.ink.i400);
        let galley = rux.text(Style::sans(17.0).medium(), value, theme.ink.i900);
        rux.painter().galley(Pos2::new(at.x + 22.0, top + 16.0), galley, theme.ink.i900);
    }
}

/// The run cards, the permission prompt, the code block, the diff and the error, in three columns.
fn cards(rux: &mut Rux<'_>, pen: &mut Pen) {
    let theme = rux.theme();
    let chat = theme.chat;
    let area = pen.rest();
    let column = ((area.width() - 80.0) / 3.0).min(400.0);
    let lefts = [area.left(), area.left() + column + 40.0, area.left() + 2.0 * (column + 40.0)];
    let mut tops = [area.top(); 3];
    let mut place = |rux: &mut Rux<'_>, which: usize, height: f32, draw: &dyn Fn(&mut Rux<'_>, Rect)| {
        let rect = Rect::from_min_size(Pos2::new(lefts[which], tops[which]), Vec2::new(column, height));
        draw(rux, rect);
        tops[which] += height + 20.0;
    };
    let runs = [
        Line::plain("cargo build --release --timings").trailing("2m 58s"),
        Line::plain("cargo tree --duplicates -e normal").trailing("1.4s"),
        Line::plain("git log --since=7.days -- Cargo.lock").trailing("0.2s"),
    ];
    let card = CommandCard::new("cargo build --release", "Compiling 212 of 580 \u{00B7} 0:47", Status::Running).progress(Some(0.36));
    place(rux, 0, card.height(rux), &|rux, rect| {
        CommandCard::new("cargo build --release", "Compiling 212 of 580 \u{00B7} 0:47", Status::Running).progress(Some(0.36)).show(rux, rect);
    });
    let card = CommandCard::new("Ran 3 commands", "3m 00s", Status::Done).words().lines(&runs, false);
    place(rux, 0, card.height(rux), &|rux, rect| {
        CommandCard::new("Ran 3 commands", "3m 00s", Status::Done).words().lines(&runs, false).show(rux, rect);
    });
    let card = CommandCard::new("Ran 3 commands", "3m 00s", Status::Done).words().lines(&runs, true);
    place(rux, 0, card.height(rux), &|rux, rect| {
        CommandCard::new("Ran 3 commands", "3m 00s", Status::Done).words().lines(&runs, true).show(rux, rect);
    });
    let failed = [Line::plain("test theme::tests::the_set_is_the_reference_s_own ... FAILED")];
    let card = CommandCard::new("cargo test -p unluminous-app", "Exited with 101 \u{00B7} 2 tests failed", Status::Failed).lines(&failed, false);
    place(rux, 0, card.height(rux), &|rux, rect| {
        CommandCard::new("cargo test -p unluminous-app", "Exited with 101 \u{00B7} 2 tests failed", Status::Failed).lines(&failed, false).show(rux, rect);
    });
    let command = [Line::coloured("cargo update -p kurbo --precise 0.11.1", theme.ink.i900)];
    let card = PermissionCard::new("claude wants to run a command", "It changes Cargo.lock", &command);
    place(rux, 1, card.height(rux), &|rux, rect| {
        PermissionCard::new("claude wants to run a command", "It changes Cargo.lock", &command).show(rux, rect);
    });
    let keyword = rux::theme::rgb(0xFF79C6);
    let function = rux::theme::rgb(0x50FA7B);
    let kind = rux::theme::rgb(0x8BE9FD);
    let code = [
        Line { runs: vec![("pub fn ".into(), Some(keyword)), ("more".into(), Some(function)), ("(painter: &".into(), None), ("Painter".into(), Some(kind)), (") {".into(), None)], ..Line::default() }.number("1"),
        Line { runs: vec![("    for ".into(), Some(keyword)), ("step in [-4.5, 0.0, 4.5] {".into(), None)], ..Line::default() }.number("2"),
        Line::plain("        painter.circle_filled(dot, 1.4, INK);").number("3"),
        Line::plain("    }").number("4"),
        Line::plain("}").number("5"),
    ];
    let card = CodeBlock::new("icon.rs", "Rust \u{00B7} lines 1\u{2013}5", &code);
    place(rux, 1, card.height(rux), &|rux, rect| {
        CodeBlock::new("icon.rs", "Rust \u{00B7} lines 1\u{2013}5", &code).show(rux, rect);
    });
    let diff = [
        Line::plain("use egui::{Color32, Pos2, Stroke};").number("3"),
        Line::plain("use kurbo::BezPath;").number("4").sign(Sign::Removed),
        Line::plain("use rux::kurbo::BezPath;").number("4").sign(Sign::Added),
        Line::plain("").number("5"),
    ];
    let card = DiffCard::new("icon.rs", "1 line changed", &diff);
    place(rux, 1, card.height(rux), &|rux, rect| {
        DiffCard::new("icon.rs", "1 line changed", &diff).show(rux, rect);
    });
    let log = [
        Line::coloured("error[E0432]: unresolved import `kurbo::BezPath`", chat.slower),
        Line::coloured("  --> crates/unluminous-app/src/theme/icon.rs:4:5", theme.ink.i400),
    ];
    let card = ErrorCard::new("cargo exited with code 101", "cargo build --release", &log);
    place(rux, 2, card.height(rux), &|rux, rect| {
        ErrorCard::new("cargo exited with code 101", "cargo build --release", &log).show(rux, rect);
    });
    let tallest = tops.iter().copied().fold(area.top(), f32::max);
    pen.strip(tallest - area.top());
}
