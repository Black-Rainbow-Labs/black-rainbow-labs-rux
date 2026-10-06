//! A field's placeholder is set in the field's own font, whatever the `Ui` around it is set to.
//!
//! Unluminous `task-2198`: *"Placeholder text in various places is not sized like the other font sizes around
//! it."* egui 0.36 lays a `TextEdit`'s hint out as atoms, and an atom with no size of its own takes the
//! `Ui`'s Body size rather than the `TextEdit`'s font. Unluminous sets Body from a person's interface size,
//! which is 24 points on the machine the report came from, so `Add a todo` in a 13 point field was drawn at
//! 24. This draws both fields with Body at 24 and asks what size each hint was laid out at.

use egui::{Rect, Vec2};
use rux::components::{TextArea, TextInput};
use rux::screen::Screen;
use rux::{RuxState, Theme};

/// The font size every piece of laid out text in one frame was set in, with the text it was.
fn sizes_drawn(body: impl Fn(&mut rux::Rux<'_>)) -> Vec<(String, f32)> {
    let ctx = egui::Context::default();
    rux::text::install(&ctx);
    // The interface size of the machine the report came from.
    ctx.all_styles_mut(|style| {
        if let Some(font) = style.text_styles.get_mut(&egui::TextStyle::Body) {
            font.size = 24.0;
        }
    });
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
            rux::layer(ui, &state, egui::Id::new("hint"), area, |rux| body(rux));
        });
        // Nothing here paints to a screen, so the atlas epaint built has nobody to hand its texture to.
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
            for section in &text.galley.job.sections {
                out.push((text.galley.job.text.clone(), section.format.font_id.size));
            }
        }
        egui::Shape::Vec(shapes) => shapes.iter().for_each(|one| collect(one, out)),
        _ => {}
    }
}

fn size_of(found: &[(String, f32)], text: &str) -> f32 {
    found
        .iter()
        .find(|(words, _)| words == text)
        .map(|(_, size)| *size)
        .unwrap_or_else(|| panic!("{text:?} was not drawn; drawn: {found:?}"))
}

#[test]
fn a_text_inputs_hint_is_set_in_the_fields_font() {
    let found = sizes_drawn(|rux| {
        let mut empty = String::new();
        let field = Rect::from_min_size(egui::pos2(20.0, 20.0), Vec2::new(300.0, 40.0));
        TextInput::new(&mut empty).hint("Add a todo").show(rux, field);
    });
    assert_eq!(size_of(&found, "Add a todo"), 13.0, "the hint took the Ui's Body size");
}

#[test]
fn a_text_areas_hint_is_set_in_the_fields_font() {
    let found = sizes_drawn(|rux| {
        let mut empty = String::new();
        let field = Rect::from_min_size(egui::pos2(20.0, 80.0), Vec2::new(300.0, 120.0));
        TextArea::new(&mut empty).hint("Describe the image").show(rux, field);
    });
    let size = size_of(&found, "Describe the image");
    assert!(size < 24.0, "the hint took the Ui's Body size: {size}");
}
