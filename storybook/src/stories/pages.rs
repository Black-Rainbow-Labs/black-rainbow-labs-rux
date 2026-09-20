//! The example pages, hosted full width and interactive.
//!
//! A page is shown **as it would be used**, not as a picture: its selects open, its panels collapse,
//! its stepper counts and its modal opens. That is what makes the page a test of the components, and
//! it is why it is in the storybook rather than only in a screenshot.

use egui::Rect;
use rux::Rux;

use super::{Demo, Story};

pub fn stories() -> Vec<Story> {
    vec![Story {
        section: "Pages",
        name: "Image Creator",
        rule: "examples/pages/image_creator.rs · the AI Studio reference page",
        states: &["prompt panel", "controls panel", "results canvas", "prompt manager"],
        draw: image_creator,
        full_bleed: true,
    }]
}

fn image_creator(rux: &mut Rux<'_>, demo: &mut Demo, area: Rect) -> f32 {
    demo.image_creator.show(rux, area);
    area.height()
}
