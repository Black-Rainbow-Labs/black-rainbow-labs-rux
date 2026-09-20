//! The library's values, checked against the stylesheet they were transcribed from.
//!
//! This is what "pixel perfect" means here, and it is stronger than comparing two pictures: a
//! screenshot comparison can only say that two images differ, where this says **which declaration is
//! wrong**. The files in `reference/` are read at test time and parsed, so a token that drifts, a
//! padding that is mistyped or a hex string with two digits swapped is a named failure rather than a
//! slightly wrong colour nobody notices.
//!
//! It also means the reference is a dependency of the build rather than a folder somebody once
//! looked at. Change `reference/neumorphic-tokens.css` and this test tells you what else has to move.

use std::collections::HashMap;
use std::path::PathBuf;

use rux::theme::{self, Theme};

fn reference(name: &str) -> String {
    let path: PathBuf =
        [env!("CARGO_MANIFEST_DIR"), "..", "..", "reference", name].iter().collect();
    std::fs::read_to_string(&path)
        .unwrap_or_else(|problem| panic!("could not read {}: {problem}", path.display()))
}

/// Every `--name: value;` in a stylesheet, last one winning — which is what the cascade does for a
/// file that is read top to bottom with one selector.
fn custom_properties(css: &str) -> HashMap<String, String> {
    let mut out = HashMap::new();
    for line in css.lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix("--") else { continue };
        let Some((name, value)) = rest.split_once(':') else { continue };
        // Cut at the semicolon rather than trimming it off the end: the token file writes a comment
        // after most of its declarations, and `#E4E9F0; /* … */` is not a colour.
        let value = value.split(';').next().unwrap_or("").trim();
        if value.is_empty() {
            continue;
        }
        out.insert(name.trim().to_owned(), value.to_owned());
    }
    out
}

/// A `#RRGGBB` as the library writes it, so a failure prints two comparable strings.
fn hex(colour: egui::Color32) -> String {
    let [r, g, b, _] = colour.to_srgba_unmultiplied();
    format!("#{r:02X}{g:02X}{b:02X}")
}

/// The declarations inside one rule, found by its selector.
///
/// A deliberately small reader: it finds `<selector> {` and takes `property: value` pairs up to the
/// closing brace. Enough for the rules this test names, and it fails loudly on a selector that is not
/// there rather than quietly checking nothing.
fn rule(css: &str, selector: &str) -> HashMap<String, String> {
    // The selector, then whatever whitespace is between it and its brace — the token file lines its
    // type-scale rules up in a column, so `.t-h1      {` is the same rule as `.t-h1 {`. The selector
    // has to be followed by a space or a brace, or `.topnav` would match `.topnav__item`.
    let start = css
        .match_indices(selector)
        .find_map(|(at, _)| {
            let rest = &css[at + selector.len()..];
            let after = rest.trim_start();
            let gap = rest.len() - after.len();
            let boundary = gap > 0 || after.starts_with('{');
            match boundary && after.starts_with('{') {
                true => Some(at + selector.len() + gap + 1),
                false => None,
            }
        })
        .unwrap_or_else(|| panic!("{selector} is not in the stylesheet any more"));
    let end = start + css[start..].find('}').expect("a rule that is never closed");
    let mut out = HashMap::new();
    for declaration in css[start..end].split(';') {
        if let Some((property, value)) = declaration.split_once(':') {
            out.insert(property.trim().to_owned(), value.trim().to_owned());
        }
    }
    out
}

/// One `<n>px` as a number.
fn px(value: &str) -> f32 {
    value
        .trim()
        .trim_end_matches("px")
        .parse()
        .unwrap_or_else(|_| panic!("{value:?} is not a length in points"))
}

/// The lengths in a shorthand like `padding: 9px 12px`.
fn lengths(value: &str) -> Vec<f32> {
    value.split_whitespace().map(px).collect()
}

#[test]
fn every_colour_in_the_light_theme_is_the_token_file_s() {
    let tokens = custom_properties(&reference("neumorphic-tokens.css"));
    let theme = theme::light();
    let pairs: [(&str, egui::Color32); 25] = [
        ("surface-0", theme.surface.s0),
        ("surface-1", theme.surface.s1),
        ("surface-2", theme.surface.s2),
        ("surface-3", theme.surface.s3),
        ("surface-sunken", theme.surface.sunken),
        ("shadow-light", theme.shadow.light),
        ("shadow-dark", theme.shadow.dark),
        ("shadow-darker", theme.shadow.darker),
        ("ink-900", theme.ink.i900),
        ("ink-700", theme.ink.i700),
        ("ink-500", theme.ink.i500),
        ("ink-400", theme.ink.i400),
        ("ink-300", theme.ink.i300),
        ("ink-200", theme.ink.i200),
        ("accent-blue", theme.accent.blue),
        ("accent-blue-deep", theme.accent.blue_deep),
        ("accent-blue-soft", theme.accent.blue_soft),
        ("accent-coral", theme.accent.coral),
        ("accent-violet", theme.accent.violet),
        ("accent-mint", theme.accent.mint),
        ("accent-amber", theme.accent.amber),
        ("accent-rose", theme.accent.rose),
        ("success", theme.semantic.success),
        ("warning", theme.semantic.warning),
        ("danger", theme.semantic.danger),
    ];
    for (name, colour) in pairs {
        let declared = tokens
            .get(name)
            .unwrap_or_else(|| panic!("--{name} is not in neumorphic-tokens.css any more"));
        assert_eq!(
            hex(colour).to_uppercase(),
            declared.to_uppercase(),
            "--{name}"
        );
    }
}

#[test]
fn every_colour_in_the_dark_theme_is_the_incognito_file_s() {
    let tokens = custom_properties(&reference("incognito-theme.css"));
    let theme = theme::dark();
    let pairs: [(&str, egui::Color32); 25] = [
        ("surface-0", theme.surface.s0),
        ("surface-1", theme.surface.s1),
        ("surface-2", theme.surface.s2),
        ("surface-3", theme.surface.s3),
        ("surface-sunken", theme.surface.sunken),
        ("shadow-light", theme.shadow.light),
        ("shadow-dark", theme.shadow.dark),
        ("shadow-darker", theme.shadow.darker),
        ("ink-900", theme.ink.i900),
        ("ink-700", theme.ink.i700),
        ("ink-500", theme.ink.i500),
        ("ink-400", theme.ink.i400),
        ("ink-300", theme.ink.i300),
        ("ink-200", theme.ink.i200),
        ("accent-blue", theme.accent.blue),
        ("accent-blue-deep", theme.accent.blue_deep),
        ("accent-blue-soft", theme.accent.blue_soft),
        ("accent-coral", theme.accent.coral),
        ("accent-violet", theme.accent.violet),
        ("accent-mint", theme.accent.mint),
        ("accent-amber", theme.accent.amber),
        ("accent-rose", theme.accent.rose),
        ("success", theme.semantic.success),
        ("warning", theme.semantic.warning),
        ("danger", theme.semantic.danger),
    ];
    for (name, colour) in pairs {
        let declared = tokens
            .get(name)
            .unwrap_or_else(|| panic!("--{name} is not in incognito-theme.css any more"));
        assert_eq!(hex(colour).to_uppercase(), declared.to_uppercase(), "--{name}");
    }
}

#[test]
fn the_radii_are_the_token_file_s() {
    let tokens = custom_properties(&reference("neumorphic-tokens.css"));
    let radius = theme::light().radius;
    for (name, value) in [
        ("r-sm", radius.sm),
        ("r-md", radius.md),
        ("r-lg", radius.lg),
        ("r-xl", radius.xl),
        ("r-2xl", radius.xxl),
        ("r-pill", radius.pill),
    ] {
        assert_eq!(px(&tokens[name]), value, "--{name}");
    }
}

#[test]
fn the_elevations_are_the_token_file_s_declarations() {
    // `--e-raised: -6px -6px 14px var(--shadow-light), 6px 6px 14px var(--shadow-dark);`
    //
    // Only the **lengths** are read, because the colours are `var(…)` references the theme has
    // already been checked against above. What this catches is an offset or a blur that drifted —
    // which is invisible in a screenshot and obvious here.
    let tokens = custom_properties(&reference("neumorphic-tokens.css"));
    let theme = theme::light();
    let declared_lengths = |name: &str| -> Vec<Vec<f32>> {
        tokens[name]
            .split(',')
            .map(|shadow| {
                shadow
                    .split_whitespace()
                    .filter(|word| word.ends_with("px"))
                    .map(px)
                    .collect()
            })
            .collect()
    };
    let ours = |elevation: rux::Elevation| -> Vec<Vec<f32>> {
        elevation.shadows().map(|shadow| vec![shadow.dx, shadow.dy, shadow.blur]).collect()
    };
    for (name, elevation) in [
        ("e-raised-sm", theme.elevation.raised_sm),
        ("e-raised", theme.elevation.raised),
        ("e-raised-lg", theme.elevation.raised_lg),
        ("e-pressed-sm", theme.elevation.pressed_sm),
        ("e-pressed", theme.elevation.pressed),
        ("e-pressed-deep", theme.elevation.pressed_deep),
    ] {
        assert_eq!(declared_lengths(name), ours(elevation), "--{name}");
    }
}

#[test]
fn a_blur_radius_is_half_the_gaussian_the_renderer_is_given() {
    // The one conversion between the stylesheet and the renderer, checked rather than assumed: CSS
    // defines the blur radius as twice the standard deviation.
    for shadow in theme::light().elevation.raised.shadows() {
        assert_eq!(shadow.deviation(), shadow.blur / 2.0, "the conversion drifted");
    }
}

#[test]
fn the_component_measurements_are_the_page_stylesheet_s() {
    let css = reference("image-creator.css");

    // `.ic-count { width: 66px }` — the control that must not shrink.
    assert_eq!(px(&rule(&css, ".ic-count")["width"]), rux::components::STEPPER_WIDTH);

    // `.ic-count__steps button { width: 18px; height: 11px }`.
    let tick = rule(&css, ".ic-count__steps button");
    assert_eq!((px(&tick["width"]), px(&tick["height"])), (18.0, 11.0));

    // `.ic-spinner { width: 56px; height: 56px }`.
    let spinner = rule(&css, ".ic-spinner");
    assert_eq!(px(&spinner["width"]), rux::components::SPINNER_DIAMETER);

    // `.topnav { gap: 4px; padding: 6px }` and `.topnav__item { padding: 9px 14px }`.
    let strip = rule(&css, ".topnav");
    assert_eq!((px(&strip["gap"]), px(&strip["padding"])), (4.0, 6.0));
    assert_eq!(lengths(&rule(&css, ".topnav__item")["padding"]), vec![9.0, 14.0]);
    assert_eq!(lengths(&rule(&css, ".topnav__item.is-active")["padding"]), vec![9.0, 18.0]);

    // `.ic-create { padding: 14px 20px }` — the large primary button.
    assert_eq!(lengths(&rule(&css, ".ic-create")["padding"]), vec![14.0, 20.0]);
    // `.ic-mini-btn { padding: 9px 12px; gap: 6px }`.
    let mini = rule(&css, ".ic-mini-btn");
    assert_eq!(lengths(&mini["padding"]), vec![9.0, 12.0]);
    assert_eq!(px(&mini["gap"]), 6.0);
    // `.ic-fbtn { padding: 9px 14px }`.
    assert_eq!(lengths(&rule(&css, ".ic-fbtn")["padding"]), vec![9.0, 14.0]);

    // `.ic-select__trigger { padding: 9px 12px; gap: 8px }`.
    let trigger = rule(&css, ".ic-select__trigger");
    assert_eq!(lengths(&trigger["padding"]), vec![9.0, 12.0]);
    assert_eq!(px(&trigger["gap"]), 8.0);
    // `.ic-select__menu { max-height: 280px; padding: 6px }`.
    let menu = rule(&css, ".ic-select__menu");
    assert_eq!(px(&menu["max-height"]), rux::components::SELECT_MENU_MAX_HEIGHT);
    assert_eq!(px(&menu["padding"]), 6.0);
    // `.ic-select__item { padding: 8px 12px }`.
    assert_eq!(lengths(&rule(&css, ".ic-select__item")["padding"]), vec![8.0, 12.0]);

    // `.proj__trigger { min-width: 240px; padding: 8px 14px 8px 8px; gap: 12px }`.
    let project = rule(&css, ".proj__trigger");
    assert_eq!(px(&project["min-width"]), 240.0);
    assert_eq!(lengths(&project["padding"]), vec![8.0, 14.0, 8.0, 8.0]);
    assert_eq!(px(&project["gap"]), 12.0);
    // `.proj__icon { width: 36px }`.
    assert_eq!(px(&rule(&css, ".proj__icon")["width"]), 36.0);

    // `.ic-panel--prompt { width: 300px }` and `.ic-panel--controls { width: 360px }`.
    assert_eq!(px(&rule(&css, ".ic-panel--prompt")["width"]), 300.0);
    assert_eq!(px(&rule(&css, ".ic-panel--controls")["width"]), 360.0);

    // The five round buttons, at the five diameters the library offers.
    for (selector, diameter) in [
        (".ic-chev", 24.0),
        (".ic-dice", 28.0),
        (".ic-prompt-shuffle", 28.0),
        (".ic-modal__close", 32.0),
        (".toolbtn", 40.0),
        (".iconbtn", 40.0),
        (".dock__btn", 48.0),
        (".avatar", 40.0),
    ] {
        assert_eq!(px(&rule(&css, selector)["width"]), diameter, "{selector}");
    }

    // `.ic-form-input { padding: 12px 16px }` and `.ic-prompt { padding: 14px 40px 28px 14px }`.
    assert_eq!(lengths(&rule(&css, ".ic-form-input")["padding"]), vec![12.0, 16.0]);
    assert_eq!(lengths(&rule(&css, ".ic-prompt")["padding"]), vec![14.0, 40.0, 28.0, 14.0]);

    // `.ic-canvas { padding: 24px 28px }` and `.topbar { padding: 16px 28px }`.
    assert_eq!(lengths(&rule(&css, ".ic-canvas")["padding"]), vec![24.0, 28.0]);
    assert_eq!(lengths(&rule(&css, ".topbar")["padding"]), vec![16.0, 28.0]);
}

#[test]
fn the_type_is_the_page_stylesheet_s() {
    let css = reference("image-creator.css");
    let style = |selector: &str| rule(&css, selector);

    // `.ic-create { font-size: 14px; font-weight: 600 }`.
    let create = style(".ic-create");
    assert_eq!(px(&create["font-size"]), rux::Style::BUTTON.size);
    assert_eq!(create["font-weight"], "600");

    // `.ic-labelled__label { font-size: 9.5px; letter-spacing: 0.14em }`.
    let label = style(".ic-labelled__label");
    assert_eq!(px(&label["font-size"]), rux::Style::FIELD_LABEL.size);
    assert_eq!(label["letter-spacing"], "0.14em");
    assert_eq!(rux::Style::FIELD_LABEL.tracking, 0.14);
    assert_eq!(label["text-transform"], "uppercase");
    assert!(rux::Style::FIELD_LABEL.upper);

    // `.ic-ctrl__label { font-size: 9px; letter-spacing: 0.14em }`.
    let ctrl = style(".ic-ctrl__label");
    assert_eq!(px(&ctrl["font-size"]), rux::Style::CTRL_LABEL.size);

    // `.ic-panel__title { font-size: 10.5px; letter-spacing: 0.16em }`.
    let panel = style(".ic-panel__title");
    assert_eq!(px(&panel["font-size"]), rux::Style::PANEL_TITLE.size);
    assert_eq!(rux::Style::PANEL_TITLE.tracking, 0.16);

    // `.ic-prompt { font-size: 13px; line-height: 1.6 }`.
    let prompt = style(".ic-prompt");
    assert_eq!(px(&prompt["font-size"]), rux::Style::PROSE.size);
    assert_eq!(rux::Style::PROSE.line_height, Some(1.6));

    // `.topnav__item { font-size: 12px; font-weight: 500 }`.
    let item = style(".topnav__item");
    assert_eq!(px(&item["font-size"]), rux::Style::CONTROL.size);
    assert_eq!(item["font-weight"], "500");
}

#[test]
fn the_named_type_scale_is_the_token_file_s() {
    // `.t-h2 { font: 600 24px/1.3 var(--font-display); letter-spacing: -0.015em }`.
    let css = reference("neumorphic-tokens.css");
    let scale: [(&str, rux::Style); 5] = [
        (".t-display", rux::Style::DISPLAY),
        (".t-h1", rux::Style::H1),
        (".t-h2", rux::Style::H2),
        (".t-h3", rux::Style::H3),
        (".t-body", rux::Style::BODY),
    ];
    for (selector, style) in scale {
        let declared = rule(&css, selector);
        let font = &declared["font"];
        let mut words = font.split_whitespace();
        let weight: u16 = words.next().unwrap().parse().unwrap();
        let (size, leading) = words.next().unwrap().split_once('/').unwrap();
        assert_eq!(style.weight.number(), weight, "{selector} weight");
        assert_eq!(style.size, px(size), "{selector} size");
        assert_eq!(
            style.line_height,
            Some(leading.parse::<f32>().unwrap()),
            "{selector} line height"
        );
        if let Some(tracking) = declared.get("letter-spacing") {
            let declared: f32 = tracking.trim_end_matches("em").parse().unwrap();
            assert!(
                (style.tracking - declared).abs() < 0.0001,
                "{selector} tracking: {} against {declared}",
                style.tracking
            );
        }
    }
}

#[test]
fn every_icon_in_the_set_is_in_the_reference_s_own_source() {
    // The path data is transcribed rather than generated, so the thing that can go wrong is a mark
    // being renamed or dropped. Every name the library knows has to appear as a key in `SbIcon`.
    let source = reference("shared-components.jsx");
    for icon in rux::Icon::ALL {
        let key = format!("{}: <>", icon.name());
        assert!(source.contains(&key), "{} is not a key in SbIcon any more", icon.name());
    }
}

#[test]
fn the_dark_theme_changes_no_recipe_at_all() {
    // The property the whole theme design rests on, stated as a test: the incognito stylesheet
    // overrides values and not rules, so both themes must declare identically shaped elevations.
    let shape = |elevation: rux::Elevation| {
        elevation
            .shadows()
            .map(|shadow| (shadow.dx, shadow.dy, shadow.blur, shadow.spread, shadow.inset))
            .collect::<Vec<_>>()
    };
    let light = theme::light().elevation;
    let dark = theme::dark().elevation;
    for (name, a, b) in [
        ("raised_sm", light.raised_sm, dark.raised_sm),
        ("raised", light.raised, dark.raised),
        ("raised_lg", light.raised_lg, dark.raised_lg),
        ("pressed_sm", light.pressed_sm, dark.pressed_sm),
        ("pressed", light.pressed, dark.pressed),
        ("pressed_deep", light.pressed_deep, dark.pressed_deep),
        ("modal", light.modal, dark.modal),
        ("primary", light.primary, dark.primary),
        ("primary_pressed", light.primary_pressed, dark.primary_pressed),
        ("primary_sm", light.primary_sm, dark.primary_sm),
        ("nav_active", light.nav_active, dark.nav_active),
        ("image_well", light.image_well, dark.image_well),
    ] {
        assert_eq!(shape(a), shape(b), "{name}");
    }
    // And the gradients' angles, which are geometry rather than colour.
    for (name, a, b) in [
        ("primary", theme::light().gradient.primary, theme::dark().gradient.primary),
        (
            "primary_diagonal",
            theme::light().gradient.primary_diagonal,
            theme::dark().gradient.primary_diagonal,
        ),
        ("nav_active", theme::light().gradient.nav_active, theme::dark().gradient.nav_active),
        ("knob", theme::light().gradient.knob, theme::dark().gradient.knob),
    ] {
        assert_eq!(a.angle, b.angle, "{name}");
    }
}

#[test]
fn a_theme_is_named_and_an_unknown_one_is_refused_with_the_list() {
    assert!(Theme::named("light-neumorphic").is_some());
    assert!(Theme::named("dark-neumorphic").is_some());
    assert!(Theme::named("solarized").is_none());
    assert_eq!(Theme::names(), vec!["light-neumorphic", "dark-neumorphic"]);
}
