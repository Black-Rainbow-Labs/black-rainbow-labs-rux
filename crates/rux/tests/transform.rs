//! A layer drawn through a transform: its popups follow it, and its textures are made for the size it is
//! seen at.
//!
//! Unluminous `task-2198` zooms a modal by drawing it into a layer with a `TSTransform`, which is how it
//! already zooms the nodes on its canvas. Two things in this library assumed every layer was drawn at the
//! window's own scale: an overlay was opened at its rectangle read as window points, so a dropdown opened
//! inside a zoomed modal appeared somewhere else; and the decoration and the icons were rasterised at the
//! display's density, so the transform magnified them.

use egui::{Rect, Vec2};
use rux::render::composited_scale;
use rux::screen::Screen;
use rux::{RuxState, Theme};

/// Run one frame with a layer drawn through `zoom`, and answer what `inside` saw.
fn in_a_zoomed_layer<R>(zoom: f32, inside: impl FnOnce(&mut rux::Rux<'_>) -> R) -> (egui::Context, R) {
    let ctx = egui::Context::default();
    rux::text::install(&ctx);
    let state = RuxState::new(<&Theme>::default());
    let screen = Screen::desktop(Vec2::new(800.0, 600.0));
    state.set_screen(screen);
    let layer = egui::LayerId::new(egui::Order::Foreground, egui::Id::new("zoomed"));
    ctx.set_transform_layer(layer, egui::emath::TSTransform::from_scaling(zoom));
    let mut input = egui::RawInput::default();
    input.screen_rect = Some(Rect::from_min_size(egui::pos2(0.0, 0.0), screen.size));
    let mut inside = Some(inside);
    let mut answer = None;
    let mut output = ctx.run_ui(input, |ui| {
        let mut zoomed = ui.new_child(egui::UiBuilder::new().layer_id(layer).max_rect(ui.max_rect()));
        let area = zoomed.max_rect();
        answer = rux::layer(&mut zoomed, &state, egui::Id::new("zoomed-rux"), area, |rux| {
            inside.take().map(|inside| inside(rux))
        });
    });
    output.textures_delta.clear();
    state.end_frame();
    (ctx, answer.expect("the layer was drawn"))
}

#[test]
fn an_overlay_is_drawn_through_the_transform_of_the_layer_it_was_opened_from() {
    let menu = egui::Id::new("a dropdown");
    let (ctx, ()) = in_a_zoomed_layer(1.5, |rux| {
        let rect = Rect::from_min_size(egui::pos2(40.0, 40.0), Vec2::new(120.0, 80.0));
        rux.overlay(menu, rect, egui::Order::Foreground, |_| {});
    });
    let overlay = egui::LayerId::new(egui::Order::Foreground, menu);
    let to_global = ctx.layer_transform_to_global(overlay).expect("the overlay has a transform");
    assert_eq!(to_global.scaling, 1.5);
}

#[test]
fn a_zoomed_layer_is_rasterised_at_the_size_it_is_seen_at() {
    let (_, seen) = in_a_zoomed_layer(1.3, |rux| composited_scale(rux.ui.ctx(), rux.ui.layer_id()));
    // In quarter steps, rounded up: 1.3 is drawn from a texture made at 1.5.
    assert_eq!(seen, 1.5);
    let (_, plain) = in_a_zoomed_layer(1.0, |rux| composited_scale(rux.ui.ctx(), rux.ui.layer_id()));
    assert_eq!(plain, 1.0, "and a layer with no zoom is the display's own density");
}
