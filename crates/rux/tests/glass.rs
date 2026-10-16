//! Glass (`task-2281`): what a glazed theme records, and what a layer with glass in it paints.
//!
//! The arithmetic of the bend is tested beside it in `render::glass`. These are the rules around it: which
//! surfaces are glass, that a layer is cut at each piece of glass in order, that nothing changes for a theme
//! that is not glass, and that a still layer builds nothing again.

use egui::{Pos2, Rect, Shape, TextureId, Vec2};
use rux::render::{Backdrop, Plane};
use rux::{Chrome, Decor, RuxState, Theme};

fn glazed() -> &'static Theme {
    let dark = Theme::named("dark-neumorphic").expect("the dark theme");
    Box::leak(Box::new(dark.glazed("dark-glass", dark.surface.s1)))
}

fn a_card() -> Rect {
    Rect::from_min_size(Pos2::new(40.0, 40.0), Vec2::new(300.0, 120.0))
}

#[test]
fn only_a_glazed_theme_is_glass() {
    for name in Theme::names() {
        assert!(Theme::named(name).unwrap().glass.is_none(), "{name} shipped as glass");
    }
    assert!(glazed().glass.is_some());
}

#[test]
fn a_raised_translucent_surface_is_glass_and_a_well_is_not() {
    let theme = glazed();
    let chat = theme.chat;
    let chrome = Chrome::recording().with_glass(theme.glass);
    chrome.surface(a_card(), 22.0, rux::Fill::gradient(chat.card, a_card()), chat.raised());
    let raised = chrome.take();
    let glass = raised.iter().position(|item| matches!(item, Decor::Glass { .. }));
    let fill = raised.iter().position(|item| matches!(item, Decor::Rect { .. }));
    assert!(glass.is_some(), "a raised card is glass");
    assert!(glass < fill, "and the bent picture is under its tint");

    chrome.surface(a_card(), 16.0, chat.well, chat.carved_sm());
    assert!(
        !chrome.take().iter().any(|item| matches!(item, Decor::Glass { .. })),
        "a carved well is tint only"
    );

    chrome.surface(a_card(), 16.0, egui::Color32::BLACK, chat.raised());
    assert!(
        !chrome.take().iter().any(|item| matches!(item, Decor::Glass { .. })),
        "an opaque fill hides the picture anyway"
    );
}

#[test]
fn a_theme_that_is_not_glass_records_no_glass() {
    let theme = Theme::named("dark-neumorphic").unwrap();
    let chrome = Chrome::recording().with_glass(theme.glass);
    let translucent = egui::Color32::from_rgba_premultiplied(10, 10, 10, 100);
    chrome.surface(a_card(), 22.0, translucent, theme.chat.raised());
    assert!(!chrome.take().iter().any(|item| matches!(item, Decor::Glass { .. })));
}

/// Draw a card with a pill in a well on it, the way a chosen pill in a carved track is, and answer what
/// the layer painted.
fn painted(theme: &'static Theme, backdrop: Option<Backdrop>, passes: usize) -> (Vec<Shape>, u64) {
    let ctx = egui::Context::default();
    rux::text::install(&ctx);
    rux::render::set_backdrop(&ctx, Plane::Page, backdrop);
    let state = RuxState::deterministic(theme);
    let input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(800.0, 600.0))),
        ..Default::default()
    };
    let mut shapes = Vec::new();
    for _ in 0..passes {
        let mut output = ctx.run_ui(input.clone(), |ui| {
            let area = ui.max_rect();
            rux::layer(ui, &state, egui::Id::new("glass"), area, |rux| {
                let chat = rux.theme().chat;
                let card = a_card();
                rux.chrome.surface(card, 22.0, rux::Fill::gradient(chat.card, card), chat.raised());
                let well = card.shrink(20.0);
                rux.chrome.surface(well, 30.0, chat.well, chat.carved_sm());
                let pill = Rect::from_min_size(well.min + Vec2::splat(4.0), Vec2::new(100.0, 36.0));
                rux.chrome.surface(
                    pill,
                    18.0,
                    rux::Fill::gradient(chat.card, pill),
                    chat.raised_sm(),
                );
            });
        });
        output.textures_delta.clear();
        shapes = output.shapes.into_iter().map(|clipped| clipped.shape).collect();
        state.end_frame();
    }
    let rasterised = state.canvases.borrow().total_rasterisations();
    (shapes, rasterised)
}

fn flat(shapes: Vec<Shape>) -> Vec<Shape> {
    shapes
        .into_iter()
        .flat_map(|shape| match shape {
            Shape::Vec(inner) => flat(inner),
            other => vec![other],
        })
        .collect()
}

fn a_picture() -> Backdrop {
    Backdrop::single(
        TextureId::User(7),
        Rect::from_min_size(Pos2::ZERO, Vec2::new(800.0, 600.0)),
        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
        1.0,
    )
}

#[test]
fn a_layer_is_cut_at_each_piece_of_glass_in_order() {
    let (shapes, _) = painted(glazed(), Some(a_picture()), 2);
    // What each shape is: an image of a rasterised run, or the bent picture.
    let kinds: Vec<&str> = flat(shapes)
        .iter()
        .filter_map(|shape| match shape {
            Shape::Mesh(mesh) if mesh.texture_id == TextureId::User(7) => Some("glass"),
            Shape::Rect(rect) if rect.fill.a() == 255 && rect.fill.r() == 0 => Some("cover"),
            Shape::Rect(rect) if rect.brush.is_some() => Some("run"),
            Shape::Mesh(_) => Some("run"),
            _ => None,
        })
        .collect();
    // The card's shadow, its glass, the card's tint and the well, the pill's glass, the pill's tint.
    let glass = kinds.iter().filter(|kind| **kind == "glass").count();
    assert_eq!(glass, 2, "{kinds:?}");
    let first_glass = kinds.iter().position(|kind| *kind == "glass").unwrap();
    let last_glass = kinds.iter().rposition(|kind| *kind == "glass").unwrap();
    assert!(
        kinds[first_glass..last_glass].contains(&"run"),
        "the well is between the two: {kinds:?}"
    );
    assert_eq!(kinds.last(), Some(&"run"), "and the pill's tint is over its glass: {kinds:?}");
}

#[test]
fn with_no_picture_a_glazed_layer_is_one_texture_as_before() {
    let (shapes, _) = painted(glazed(), None, 2);
    let flat = flat(shapes);
    assert!(!flat
        .iter()
        .any(|shape| matches!(shape, Shape::Mesh(mesh) if mesh.texture_id == TextureId::User(7))));
}

#[test]
fn a_still_glazed_layer_rasterises_each_run_once() {
    let (_, once) = painted(glazed(), Some(a_picture()), 2);
    let (_, thrice) = painted(glazed(), Some(a_picture()), 4);
    assert_eq!(once, thrice, "two more frames with nothing moving rasterised again");
}
