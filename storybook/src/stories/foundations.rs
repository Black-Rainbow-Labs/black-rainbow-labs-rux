//! The theme itself: every colour, the whole type scale, every elevation and every mark.
//!
//! These four pages are the ones that answer "is the theme right" rather than "is the component
//! right", and they are what a person looks at first after switching themes.

use egui::{Rect, Vec2};
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
    // The Editing code design's marks (`task-2241`) and the Settings design's (`task-2246`), so the
    // sheet shows every mark the library can draw.
    for (title, set) in [("The Editing code marks", Icon::WINDOW), ("The Settings marks", Icon::SETTINGS)] {
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
