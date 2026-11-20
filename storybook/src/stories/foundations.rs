//! The theme itself: every colour, the whole type scale, every elevation and every mark.
//!
//! These four pages are the ones that answer "is the theme right" rather than "is the component
//! right", and they are what a person looks at first after switching themes.

use egui::{Rect, Vec2};
use rux::components::Switch;
use rux::layout::{Grid, Pen};
use rux::text::Style;
use rux::{Icon, Rux};

use super::{group, icon_cell, note, samples, swatch, Demo, Story, GROUP_GAP};

pub fn stories() -> Vec<Story> {
    vec![
        Story {
            section: "Foundations",
            name: "Colour",
            rule: "reference/neumorphic-tokens.css · reference/incognito-theme.css",
            states: &["surfaces", "ink", "accents", "semantic", "shadow tones"],
            draw: colour,
            full_bleed: false,
        },
        Story {
            section: "Foundations",
            name: "Type",
            rule: ".t-display / .t-h1 / .t-body / .t-caption",
            states: &["display", "headings", "body", "caption", "tracking"],
            draw: typography,
            full_bleed: false,
        },
        Story {
            section: "Foundations",
            name: "Elevation",
            rule: "--e-raised-* / --e-pressed-* / --e-modal",
            states: &["raised", "pressed", "modal", "primary", "nav active"],
            draw: elevation,
            full_bleed: false,
        },
        Story {
            section: "Foundations",
            name: "Icons",
            rule: "SbIcon — reference/shared-components.jsx",
            states: &["all 37 marks", "agent-chat", "editing code", "settings", "24px", "13px"],
            draw: icons,
            full_bleed: false,
        },
        Story {
            section: "Foundations",
            name: "Drawing cost",
            rule: "Canvases::set_keep_composed · Canvases::set_glass_seams (task-2810)",
            states: &["composed shapes kept", "glass seams shared", "vertices"],
            draw: drawing_cost,
            full_bleed: false,
        },
        Story {
            section: "Foundations",
            name: "Glass rims",
            rule: "Canvases::set_glass_rims (task-2815)",
            states: &["rims squashed", "a long row", "a round button", "a card"],
            draw: glass_rims,
            full_bleed: false,
        },
        Story {
            section: "Foundations",
            name: "Radii",
            rule: "--r-sm … --r-pill",
            states: &["sm", "md", "lg", "xl", "2xl", "pill"],
            draw: radii,
            full_bleed: false,
        },
    ]
}

fn colour(rux: &mut Rux<'_>, _demo: &mut Demo, area: Rect) -> f32 {
    let theme = rux.theme();
    let mut pen = Pen::new(area);
    note(
        rux,
        &mut pen,
        "Every colour the library has a name for. The list is closed: a theme says what a name means \
         and cannot add one, which is what makes it true that a component reads a name and gets the \
         right colour in both themes.",
    );

    let rows: [(&str, Vec<(&str, egui::Color32)>); 5] = [
        (
            "Surfaces",
            vec![
                ("surface.s0", theme.surface.s0),
                ("surface.s1", theme.surface.s1),
                ("surface.s2", theme.surface.s2),
                ("surface.s3", theme.surface.s3),
                ("surface.sunken", theme.surface.sunken),
            ],
        ),
        (
            "Shadow tones",
            vec![
                ("shadow.light", theme.shadow.light),
                ("shadow.dark", theme.shadow.dark),
                ("shadow.darker", theme.shadow.darker),
            ],
        ),
        (
            "Ink",
            vec![
                ("ink.i900", theme.ink.i900),
                ("ink.i700", theme.ink.i700),
                ("ink.i500", theme.ink.i500),
                ("ink.i400", theme.ink.i400),
                ("ink.i300", theme.ink.i300),
                ("ink.i200", theme.ink.i200),
            ],
        ),
        (
            "Accents",
            vec![
                ("accent.blue", theme.accent.blue),
                ("accent.blue_deep", theme.accent.blue_deep),
                ("accent.blue_soft", theme.accent.blue_soft),
                ("accent.violet", theme.accent.violet),
                ("accent.mint", theme.accent.mint),
                ("accent.coral", theme.accent.coral),
                ("accent.amber", theme.accent.amber),
                ("accent.rose", theme.accent.rose),
            ],
        ),
        (
            "Semantic",
            vec![
                ("semantic.success", theme.semantic.success),
                ("semantic.warning", theme.semantic.warning),
                ("semantic.danger", theme.semantic.danger),
            ],
        ),
    ];

    for (title, colours) in rows {
        group(rux, &mut pen, title);
        let columns = Grid::auto_fill(area.width(), 200.0, 16.0);
        let grid = Grid::new(pen.rest(), columns).gap(14.0, 16.0).row_height(40.0);
        for (index, (name, colour)) in colours.iter().enumerate() {
            swatch(rux, grid.cell(index), *colour, name);
        }
        pen.strip(grid.height_for(colours.len()));
        pen.gap(GROUP_GAP);
    }
    pen.used()
}

fn typography(rux: &mut Rux<'_>, _demo: &mut Demo, area: Rect) -> f32 {
    let theme = rux.theme();
    let mut pen = Pen::new(area);
    note(
        rux,
        &mut pen,
        "Inter at four weights and JetBrains Mono at three, every one a separate face because \
         ab_glyph has no variable-font axes. letter-spacing and line-height are carried through from \
         the stylesheet, which is what makes a 9.5px mono kicker at 0.14em that rather than a \
         resemblance.",
    );

    let scale: [(&str, Style, &str); 9] = [
        ("display · 600 48/1.05 · -0.03em", Style::DISPLAY, "Black Rainbow"),
        ("h1 · 600 32/1.2 · -0.02em", Style::H1, "Image Creator"),
        ("h2 · 600 24/1.3 · -0.015em", Style::H2, "Scene controls"),
        ("h3 · 600 18/1.35 · -0.01em", Style::H3, "Person 1"),
        ("body · 400 15/1.5", Style::BODY, "A wide cinematic shot of a lone figure at dusk."),
        ("base · 400 14/1.5 · -0.005em", Style::BASE, "The size everything on the page inherits."),
        ("label · 500 13/1.4", Style::LABEL, "Save to project"),
        ("caption · mono 500 11 · 0.14em", Style::CAPTION, "Recent projects"),
        ("field label · mono 9.5 · 0.14em", Style::FIELD_LABEL, "Workflow"),
    ];

    group(rux, &mut pen, "The scale");
    for (caption, style, sample) in scale {
        let height = rux.measure(style, sample).y;
        let strip = pen.strip(height);
        let ink = theme.ink.i900;
        let galley = rux.text(style, sample, ink);
        rux.ui.painter().galley(strip.left_top(), galley, ink);
        pen.gap(4.0);
        let dim = theme.ink.i400;
        let galley = rux.text(Style::mono(9.5), caption, dim);
        let strip = pen.strip(galley.size().y);
        rux.ui.painter().galley(strip.left_top(), galley, dim);
        pen.gap(20.0);
    }

    pen.gap(GROUP_GAP - 20.0);
    group(rux, &mut pen, "Weights");
    for weight in [
        rux::text::Weight::Regular,
        rux::text::Weight::Medium,
        rux::text::Weight::SemiBold,
        rux::text::Weight::Bold,
    ] {
        let style = Style::sans(18.0).weight(weight);
        let strip = pen.strip(rux.measure(style, "Ag").y);
        let ink = theme.ink.i900;
        let text = format!("{}  ·  The quick brown fox", weight.number());
        let galley = rux.text(style, &text, ink);
        rux.ui.painter().galley(strip.left_top(), galley, ink);
        pen.gap(10.0);
    }
    pen.gap(GROUP_GAP);
    pen.used()
}

fn elevation(rux: &mut Rux<'_>, _demo: &mut Demo, area: Rect) -> f32 {
    let theme = rux.theme();
    let mut pen = Pen::new(area);
    note(
        rux,
        &mut pen,
        "An elevation is the CSS box-shadow declaration, not an approximation of it: the blur radius \
         halves to a Gaussian standard deviation, spread grows the shape before the blur, and the \
         list is replayed in reverse because CSS paints the first shadow on top.",
    );

    let lifted: [(&str, rux::Elevation); 6] = [
        ("raised sm", theme.elevation.raised_sm),
        ("raised", theme.elevation.raised),
        ("raised lg", theme.elevation.raised_lg),
        ("pressed sm", theme.elevation.pressed_sm),
        ("pressed", theme.elevation.pressed),
        ("pressed deep", theme.elevation.pressed_deep),
    ];
    group(rux, &mut pen, "The token file's six");
    // Wide cells, because a raised shadow reaches about forty points past its own edge and a sample
    // whose neighbour's halo lands on it is a sample nobody can judge. The same reason the board's
    // rail in Unluminous is inset further than everything else.
    let cells: Vec<(&str, f32)> = lifted.iter().map(|(name, _)| (*name, 170.0)).collect();
    let rects = samples(rux, &mut pen, 80.0, &cells);
    for (rect, (_, elevation)) in rects.iter().zip(lifted) {
        let shrunk = Rect::from_center_size(rect.center(), egui::vec2(120.0, 80.0));
        rux.chrome.surface(shrunk, theme.radius.lg, theme.surface.s1, elevation);
    }
    pen.gap(GROUP_GAP);

    let mixed: [(&str, rux::Elevation); 4] = [
        ("primary", theme.elevation.primary),
        ("primary pressed", theme.elevation.primary_pressed),
        ("nav active", theme.elevation.nav_active),
        ("modal", theme.elevation.modal),
    ];
    group(rux, &mut pen, "The recipes that mix an accent");
    let cells: Vec<(&str, f32)> = mixed.iter().map(|(name, _)| (*name, 190.0)).collect();
    let rects = samples(rux, &mut pen, 80.0, &cells);
    for (index, (rect, (_, elevation))) in rects.iter().zip(mixed).enumerate() {
        let shrunk = Rect::from_center_size(rect.center(), egui::vec2(140.0, 80.0));
        let fill = match index {
            0 | 1 => rux::Fill::gradient(theme.gradient.primary, shrunk),
            2 => rux::Fill::gradient(theme.gradient.nav_active, shrunk),
            _ => rux::Fill::Solid(theme.surface.s1),
        };
        rux.chrome.surface(shrunk, theme.radius.lg, fill, elevation);
    }
    pen.gap(GROUP_GAP);

    group(rux, &mut pen, "Gradients");
    let gradients: [(&str, rux::theme::Gradient); 8] = [
        ("primary", theme.gradient.primary),
        ("primary diagonal", theme.gradient.primary_diagonal),
        ("mint", theme.gradient.mint),
        ("violet", theme.gradient.violet),
        ("danger diagonal", theme.gradient.danger_diagonal),
        ("nav active", theme.gradient.nav_active),
        ("knob", theme.gradient.knob),
        ("avatar", theme.gradient.avatar),
    ];
    let cells: Vec<(&str, f32)> = gradients.iter().map(|(name, _)| (*name, 120.0)).collect();
    let rects = samples(rux, &mut pen, 56.0, &cells);
    for (rect, (_, gradient)) in rects.iter().zip(gradients) {
        rux.chrome.rect(*rect, theme.radius.md, rux::Fill::gradient(gradient, *rect));
    }
    pen.gap(GROUP_GAP);
    pen.used()
}

fn icons(rux: &mut Rux<'_>, _demo: &mut Demo, area: Rect) -> f32 {
    let mut pen = Pen::new(area);
    note(
        rux,
        &mut pen,
        "All thirty-seven SbIcon marks, with their path data transcribed verbatim. Each is shown at \
         24 points and again at 13 — the size it is really drawn at inside a control — because a mark \
         judged only at three times life size is a mark nobody has judged.",
    );
    group(rux, &mut pen, "The set");
    let columns = Grid::auto_fill(area.width(), 92.0, 12.0);
    let grid = Grid::new(pen.rest(), columns).gap(22.0, 12.0).row_height(76.0);
    for (index, icon) in Icon::ALL.iter().enumerate() {
        icon_cell(rux, grid.cell(index), *icon);
    }
    pen.strip(grid.height_for(Icon::ALL.len()));
    pen.gap(GROUP_GAP);
    // The Agent-Chat design's marks, which are not `SbIcon`'s (`task-2235`).
    group(rux, &mut pen, "The Agent-Chat marks");
    let grid = Grid::new(pen.rest(), columns).gap(22.0, 12.0).row_height(76.0);
    for (index, icon) in Icon::CHAT.iter().enumerate() {
        icon_cell(rux, grid.cell(index), *icon);
    }
    pen.strip(grid.height_for(Icon::CHAT.len()));
    pen.gap(GROUP_GAP);
    // The Editing code design's marks (`task-2241`), the Settings design's (`task-2246`) and the DAW set (`task-2344`), so the
    // sheet shows every mark the library can draw.
    for (title, set) in [("The Editing code marks", Icon::WINDOW), ("The Settings marks", Icon::SETTINGS), ("The DAW marks", Icon::DAW)] {
        group(rux, &mut pen, title);
        let grid = Grid::new(pen.rest(), columns).gap(22.0, 12.0).row_height(76.0);
        for (index, icon) in set.iter().enumerate() {
            icon_cell(rux, grid.cell(index), *icon);
        }
        pen.strip(grid.height_for(set.len()));
        pen.gap(GROUP_GAP);
    }
    pen.used()
}

fn radii(rux: &mut Rux<'_>, _demo: &mut Demo, area: Rect) -> f32 {
    let theme = rux.theme();
    let mut pen = Pen::new(area);
    note(
        rux,
        &mut pen,
        "The six radii, and the rule that a radius larger than half the shorter side is clamped — \
         which is what makes --r-pill: 999px a legal value at every size rather than a shape kurbo \
         draws wrong.",
    );
    group(rux, &mut pen, "The six");
    let sizes: [(&str, f32); 6] = [
        ("sm · 8", theme.radius.sm),
        ("md · 12", theme.radius.md),
        ("lg · 18", theme.radius.lg),
        ("xl · 24", theme.radius.xl),
        ("2xl · 32", theme.radius.xxl),
        ("pill · 999", theme.radius.pill),
    ];
    let cells: Vec<(&str, f32)> = sizes.iter().map(|(name, _)| (*name, 110.0)).collect();
    let rects = samples(rux, &mut pen, 72.0, &cells);
    for (rect, (_, radius)) in rects.iter().zip(sizes) {
        rux.chrome.surface(*rect, radius, theme.surface.s1, theme.elevation.raised_sm);
    }
    pen.gap(GROUP_GAP);

    group(rux, &mut pen, "Per-corner");
    let corners = super::samples(
        rux,
        &mut pen,
        72.0,
        &[("top only", 110.0), ("bottom only", 110.0), ("one squared", 110.0)],
    );
    let shapes = [
        rux::Corners::top(theme.radius.xl),
        rux::Corners::bottom(theme.radius.xl),
        rux::Corners { nw: 6.0, ne: 18.0, se: 18.0, sw: 18.0 },
    ];
    for (rect, shape) in corners.iter().zip(shapes) {
        rux.chrome.surface(*rect, shape, theme.surface.s1, theme.elevation.raised_sm);
    }
    pen.gap(GROUP_GAP);
    let _ = Vec2::ZERO;
    pen.used()
}

/// The two options that draw the same picture for less (`task-2810`), each a switch on this storybook's own
/// state, and what a piece of glass costs in vertices with its seams apart and shared.
fn drawing_cost(rux: &mut Rux<'_>, _demo: &mut Demo, area: Rect) -> f32 {
    let theme = rux.theme();
    let mut pen = Pen::new(area);
    note(
        rux,
        &mut pen,
        "Both options are off unless an application turns them on, and neither changes the picture. Keeping \
         composed shapes draws a layer whose Decor list, scale and area have not changed from the meshes it made \
         last time. Sharing glass seams cuts a piece of glass at its picture's strips with one vertex for each \
         point and each crossing, instead of three for every triangle.",
    );
    group(rux, &mut pen, "The options, on this storybook");
    let (keep, seams) = {
        let canvases = rux.state.canvases.borrow();
        (canvases.keeps_composed(), canvases.glass_seams())
    };
    let shared = seams == rux::render::glass::Seams::Shared;
    let cells = [
        ("composed shapes kept", Switch::SIZE.x),
        ("glass seams shared", Switch::SIZE.x),
    ];
    let rects = samples(rux, &mut pen, Switch::SIZE.y, &cells);
    if Switch::new(keep).show(rux, rects[0]).clicked() {
        rux.state.canvases.borrow_mut().set_keep_composed(!keep);
    }
    if Switch::new(shared).show(rux, rects[1]).clicked() {
        let next = if shared {
            rux::render::glass::Seams::Apart
        } else {
            rux::render::glass::Seams::Shared
        };
        rux.state.canvases.borrow_mut().set_glass_seams(next);
    }
    pen.gap(GROUP_GAP);

    group(
        rux,
        &mut pen,
        "Vertices in one piece of glass, over a picture in 32 strips, at 2 pixels a point",
    );
    for (name, size, radius) in [
        ("a round button", 28.0, 14.0),
        ("a card", 424.0, 22.0),
        ("a window", 1280.0, 12.0),
    ] {
        let [apart, shared] = [
            rux::render::glass::Seams::Apart,
            rux::render::glass::Seams::Shared,
        ]
        .map(|seams| glass_vertices(size, radius, seams));
        let line = format!("{name}, {size:.0} points: {apart} apart, {shared} shared");
        let ink = theme.ink.i700;
        let galley = rux.text(Style::BASE, &line, ink);
        let strip = pen.strip(galley.size().y);
        rux.ui.painter().galley(strip.left_top(), galley, ink);
        pen.gap(8.0);
    }
    pen.gap(GROUP_GAP);
    pen.used()
}

/// The option that squashes a glass rim's pull across the long sides of a shape much longer than it is tall
/// (`task-2815`), a switch on this storybook's own state, over a long row, a round button and a card. Seen under
/// the dark glass theme with a picture behind it.
fn glass_rims(rux: &mut Rux<'_>, _demo: &mut Demo, area: Rect) -> f32 {
    let theme = rux.theme();
    let mut pen = Pen::new(area);
    note(
        rux,
        &mut pen,
        "The design's rim pulls the picture inward more steeply than its band is deep, so the picture folds \
         back on itself near the edge. Round a corner that is the ring that reads as glass; along the straight \
         side of a long, thin row it stretches every speck of the picture into a streak. Squashed rims multiply \
         the pull across the long sides by min(short / long x 2.2, 1), the factor the design already puts on the \
         lens. Off unless an application turns it on; a shape no more than 2.2 times as long as it is tall is \
         unchanged.",
    );
    group(rux, &mut pen, "The option, on this storybook");
    let squashed = rux.state.canvases.borrow().glass_rims() == rux::render::glass::Rims::Squashed;
    let rects = samples(rux, &mut pen, Switch::SIZE.y, &[("rims squashed", Switch::SIZE.x)]);
    if Switch::new(squashed).show(rux, rects[0]).clicked() {
        let next = match squashed {
            true => rux::render::glass::Rims::Design,
            false => rux::render::glass::Rims::Squashed,
        };
        rux.state.canvases.borrow_mut().set_glass_rims(next);
    }
    pen.gap(GROUP_GAP);

    group(rux, &mut pen, "Raised surfaces under glass");
    let cells = [("a long row", 520.0), ("a round button", 90.0), ("a card", 260.0)];
    let rects = samples(rux, &mut pen, 140.0, &cells);
    let sizes = [Vec2::new(500.0, 28.0), Vec2::splat(28.0), Vec2::new(240.0, 120.0)];
    for (rect, size) in rects.iter().zip(sizes) {
        let shape = Rect::from_center_size(rect.center(), size);
        let radius = if size.y < 40.0 { size.y / 2.0 } else { theme.radius.lg };
        let fill = rux::Fill::gradient(theme.chat.chip, shape);
        rux.chrome.surface(shape, radius, fill, theme.chat.raised_chip());
    }
    pen.gap(GROUP_GAP);
    pen.used()
}

/// How many vertices one square piece of glass `size` points a side is built as, cut the way `seams` says.
fn glass_vertices(size: f32, radius: f32, seams: rux::render::glass::Seams) -> usize {
    use rux::render::glass::{shapes_cut, Pane};
    let strips = 32;
    let bands = (0..strips)
        .map(|i| {
            let (top, bottom) = (i as f32 / strips as f32, (i + 1) as f32 / strips as f32);
            rux::render::Band {
                texture: egui::TextureId::User(i),
                drawn: (top, bottom),
                held: (top, bottom),
            }
        })
        .collect();
    let room = Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1280.0, 780.0));
    let backdrop = rux::render::Backdrop {
        bands,
        rect: room,
        uv: Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
        opacity: 0.75,
    };
    let rect = Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(size, size.min(780.0)));
    let pane = Pane {
        rect,
        corners: rux::Corners::all(radius),
        bevel: radius.min(28.0),
        strength: 40.0,
    };
    let glass = rux::theme::Glass::DESIGN;
    shapes_cut(&glass, &pane, &backdrop, Default::default(), 2.0, seams)
        .iter()
        .map(|shape| match shape {
            egui::Shape::Mesh(mesh) => mesh.vertices.len(),
            _ => 0,
        })
        .sum()
}
