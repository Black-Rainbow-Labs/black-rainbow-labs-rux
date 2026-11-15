//! A single line field's text, typed or the placeholder, sits in the middle of the field's height.
//!
//! chordical-sheet-music `task-2690`: the library's search field is 50 points tall, and its placeholder and the
//! typed text were drawn about 10 points above the middle. `TextInput` worked out the right top and bottom margin
//! and gave it to the `TextEdit` with `.margin(..)`, but egui 0.36 uses that margin only when the `TextEdit` builds
//! its own frame, and the field passes `Frame::NONE`, so the margin was dropped and the text sat at the top.

use egui::{Rect, Vec2};
use rux::components::TextInput;
use rux::layout::Pad;
use rux::screen::Screen;
use rux::{RuxState, Theme};

/// The vertical middle of every piece of text laid out in a frame, with the text it was.
fn middles_drawn(body: impl Fn(&mut rux::Rux<'_>)) -> Vec<(String, f32)> {
    let ctx = egui::Context::default();
    rux::text::install(&ctx);
    let state = RuxState::new(<&Theme>::default());
    let screen = Screen::desktop(Vec2::new(800.0, 600.0));
    state.set_screen(screen);
    let mut input = egui::RawInput::default();
    input.screen_rect = Some(Rect::from_min_size(egui::pos2(0.0, 0.0), screen.size));
    let mut found = Vec::new();
    // Twice: the first frame builds the font atlas and the second draws with it.
    for _ in 0..2 {
        let mut output = ctx.run_ui(input.clone(), |ui| {
            let area = ui.max_rect();
            rux::layer(ui, &state, egui::Id::new("centre"), area, |rux| body(rux));
        });
        output.textures_delta.clear();
        found.clear();
        for clipped in &output.shapes {
            collect(&clipped.shape, &mut found);
        }
    }
    state.end_frame();
    found
}

fn collect(shape: &egui::Shape, out: &mut Vec<(String, f32)>) {
    match shape {
        egui::Shape::Text(text) => {
            let rect = text.galley.rect.translate(text.pos.to_vec2());
            out.push((text.galley.job.text.clone(), rect.center().y));
        }
        egui::Shape::Vec(shapes) => shapes.iter().for_each(|one| collect(one, out)),
        _ => {}
    }
}

fn middle_of(found: &[(String, f32)], text: &str) -> f32 {
    found
        .iter()
        .find(|(words, _)| words == text)
        .map(|(_, y)| *y)
        .unwrap_or_else(|| panic!("{text:?} was not drawn; drawn: {found:?}"))
}

#[test]
fn a_tall_fields_placeholder_and_typed_text_sit_in_its_middle() {
    let field = Rect::from_min_size(egui::pos2(20.0, 100.0), Vec2::new(300.0, 50.0));
    let found = middles_drawn(|rux| {
        let mut empty = String::new();
        TextInput::new(&mut empty).hint("Search title").pad(Pad::sides(6.0, 18.0, 6.0, 46.0)).show(rux, field);
    });
    let hint = middle_of(&found, "Search title");
    assert!((hint - field.center().y).abs() < 1.0, "the placeholder's middle is at {hint}, the field's at {}", field.center().y);
    let below = Rect::from_min_size(egui::pos2(20.0, 200.0), Vec2::new(300.0, 50.0));
    let found = middles_drawn(|rux| {
        let mut typed = String::from("Bach");
        TextInput::new(&mut typed).pad(Pad::sides(6.0, 18.0, 6.0, 46.0)).show(rux, below);
    });
    let text = middle_of(&found, "Bach");
    assert!((text - below.center().y).abs() < 1.0, "the typed text's middle is at {text}, the field's at {}", below.center().y);
}
