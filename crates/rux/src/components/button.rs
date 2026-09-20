//! A button with a word on it.
//!
//! Seven variants and four sizes, and every one of them is a rule in the reference rather than a
//! shape somebody liked: `.ic-create`, `.render-btn`, `.ic-mini-btn`, `.add-scene`, `.ic-fbtn`,
//! `.ic-fbtn--danger`, `.ic-fbtn--primary`, `.ic-clear` and `.ic-add-person`.

use egui::{Color32, Rect, Response, Sense, Vec2};

use super::{interact, pointing, Visual};
use crate::icon::{Icon, Mark};
use crate::layout::{Cross, Flex};
use crate::text::Style;
use crate::theme::{Elevation, Opacity as _, Theme};
use crate::{Fill, Rux};

/// What kind of button this is, which decides its fill, its ink and its depth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonVariant {
    /// `.ic-create` and `.render-btn` — the blue gradient, white on it, the deepest shadow in the
    /// design. One to a screen.
    Primary,
    /// `.ic-mini-btn`, `.ic-fbtn`, `.ic-clear` — the surface itself, raised a little, ink on it, and
    /// the accent only on hover.
    #[default]
    Secondary,
    /// `.add-scene` — the same idea one step further off the page, for the one action a section is
    /// about.
    Raised,
    /// `.ic-fbtn--primary` — the mint gradient, which the reference uses for *save* where blue is
    /// *create*.
    Mint,
    /// `.ic-fbtn--danger` — a secondary button whose ink is the coral, not a red fill. The reference
    /// never draws a filled destructive button.
    Danger,
    /// `.ic-add-person` — a dashed outline and no depth at all: an invitation rather than an action.
    Dashed,
    /// A button with no surface under it. Not in the reference as a button, but `.scene__menu-btn` and
    /// `.proj__new` are both this shape.
    Ghost,
}

/// How big, which is the padding, the type and the mark's size together.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonSize {
    /// `.ic-mini-btn` — `padding: 9px 12px`, 12px medium, a 12px mark, a 6px gap.
    Mini,
    /// `.ic-fbtn`, `.ic-clear` — `padding: 9px 14px`, 12px medium, a 12px mark, a 6px gap.
    #[default]
    Small,
    /// `.render-btn`, `.add-scene` — `padding: 10px 20px`, 13px medium, a 13px mark, a 10px gap.
    Medium,
    /// `.ic-create` — `padding: 14px 20px`, 14px semibold, a 15px mark, an 8px gap.
    Large,
}

impl ButtonSize {
    /// `padding`, as block and inline.
    fn padding(self) -> (f32, f32) {
        match self {
            ButtonSize::Mini => (9.0, 12.0),
            ButtonSize::Small => (9.0, 14.0),
            ButtonSize::Medium => (10.0, 20.0),
            ButtonSize::Large => (14.0, 20.0),
        }
    }

    fn gap(self) -> f32 {
        match self {
            ButtonSize::Mini | ButtonSize::Small => 6.0,
            ButtonSize::Medium => 10.0,
            ButtonSize::Large => 8.0,
        }
    }

    fn style(self) -> Style {
        match self {
            ButtonSize::Mini | ButtonSize::Small => Style::CONTROL,
            ButtonSize::Medium => Style::CONTROL_LG,
            ButtonSize::Large => Style::BUTTON,
        }
    }

    fn mark(self) -> f32 {
        match self {
            ButtonSize::Mini | ButtonSize::Small => 12.0,
            ButtonSize::Medium => 13.0,
            ButtonSize::Large => 15.0,
        }
    }
}

/// A button with a word on it.
pub struct Button<'a> {
    label: &'a str,
    icon: Option<Icon>,
    /// A mark after the words rather than before them, which is what a disclosure on a button is.
    trailing: Option<Icon>,
    variant: ButtonVariant,
    size: ButtonSize,
    enabled: bool,
    /// `flex: 1` — the button fills the width it is given rather than hugging its words.
    stretch: bool,
}

impl<'a> Button<'a> {
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            icon: None,
            trailing: None,
            variant: ButtonVariant::default(),
            size: ButtonSize::default(),
            enabled: true,
            stretch: false,
        }
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Sugar for the commonest variant.
    pub fn primary(self) -> Self {
        self.variant(ButtonVariant::Primary)
    }

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn trailing(mut self, icon: Icon) -> Self {
        self.trailing = Some(icon);
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// `flex: 1` — the two buttons in `.ic-btn-row` are each half the row.
    pub fn stretch(mut self) -> Self {
        self.stretch = true;
        self
    }

    /// The size this button wants: its padding, its marks, its gaps and its words.
    pub fn measure(&self, rux: &Rux<'_>) -> Vec2 {
        let (block, inline) = self.size.padding();
        let style = self.size.style();
        let words = rux.measure(style, self.label);
        let mark = self.size.mark();
        let marks = f32::from(self.icon.is_some()) * mark + f32::from(self.trailing.is_some()) * mark;
        let gaps = self.size.gap()
            * (f32::from(self.icon.is_some()) + f32::from(self.trailing.is_some()));
        Vec2::new(
            words.x + marks + gaps + inline * 2.0,
            (words.y.max(mark) + block * 2.0).max(self.min_height()),
        )
    }

    /// The height the rule declares even when the words in it are shorter.
    fn min_height(&self) -> f32 {
        match self.size {
            ButtonSize::Mini | ButtonSize::Small => 34.0,
            ButtonSize::Medium => 40.0,
            ButtonSize::Large => 48.0,
        }
    }

    pub fn show(self, rux: &mut Rux<'_>, rect: Rect) -> Response {
        let response = interact(rux, rect, "rux-button", Sense::click());
        let visual = Visual::of(&response, self.enabled);
        pointing(rux, &response, self.enabled);

        let theme = rux.theme();
        let radius = match self.variant {
            // `.ic-add-person` is the one button in the design that is not a pill.
            ButtonVariant::Dashed => theme.radius.md,
            _ => theme.radius.pill,
        };
        // `:active { transform: translateY(0.5px) }` — the whole control moves, which is what makes a
        // press read as a press rather than as a colour change.
        let rect = match visual {
            Visual::Active => rect.translate(Vec2::new(0.0, 0.5)),
            _ => rect,
        };
        let (fill, ink, elevation) = self.skin(theme, visual, rect);
        let fade = visual.fade();

        match self.variant {
            ButtonVariant::Ghost => {
                if matches!(visual, Visual::Hover | Visual::Active) {
                    let wash = match visual {
                        Visual::Active => theme.interaction.pressed,
                        _ => theme.interaction.hover,
                    };
                    rux.wash(rect, radius, wash);
                }
            }
            ButtonVariant::Dashed => {
                // A dashed outline is stroked with `egui`, not recorded as decoration: it carries no
                // shadow, and a hover that changed its colour would otherwise rasterise the layer.
                dashed_outline(rux, rect, radius, ink.gamma_multiply(fade));
                if matches!(visual, Visual::Hover) {
                    rux.wash(rect, radius, theme.interaction.accent_wash);
                }
            }
            _ => rux.chrome.surface(rect, radius, fill, elevation),
        }

        let (block, inline) = self.size.padding();
        let mark = self.size.mark();
        let style = self.size.style();
        let words = rux.measure(style, self.label);
        let inner = Flex::row(rect)
            .pad(crate::layout::Pad::axes(block, inline))
            .gap(self.size.gap())
            .cross(Cross::Centre)
            .justify(crate::layout::Justify::Centre);
        let inner = match self.icon {
            Some(_) => inner.sized(Vec2::splat(mark)),
            None => inner,
        };
        let inner = inner.hug(words.x);
        let inner = match self.trailing {
            Some(_) => inner.sized(Vec2::splat(mark)),
            None => inner,
        };
        let cells = inner.solve_all();

        let ink = ink.gamma_multiply(fade);
        let mut cell = cells.iter();
        if let Some(icon) = self.icon {
            if let Some(at) = cell.next() {
                rux.mark_in(Mark::new(icon, mark), *at, ink);
            }
        }
        if let Some(at) = cell.next() {
            let galley = rux.text(style, self.label, ink);
            crate::text::draw_centred(rux.ui.painter(), *at, galley, ink);
        }
        if let Some(icon) = self.trailing {
            if let Some(at) = cell.next() {
                rux.mark_in(Mark::new(icon, mark), *at, ink);
            }
        }

        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Button, self.enabled, self.label)
        });
        response
    }

    /// The fill, the ink and the depth this variant has in this state.
    ///
    /// Every arm names the rule it came from, because that is the only way a later reader can check
    /// one against the stylesheet.
    fn skin(
        &self,
        theme: &'static Theme,
        visual: Visual,
        rect: Rect,
    ) -> (Fill, Color32, Elevation) {
        let pressed = matches!(visual, Visual::Active);
        match self.variant {
            // `.ic-create` / `.render-btn`
            ButtonVariant::Primary => {
                let elevation = match (pressed, self.size) {
                    (true, _) => theme.elevation.primary_pressed,
                    (false, ButtonSize::Large) => theme.elevation.primary,
                    (false, _) => theme.elevation.primary_sm,
                };
                (Fill::gradient(theme.gradient.primary, rect), Color32::WHITE, elevation)
            }
            // `.ic-fbtn--primary`
            ButtonVariant::Mint => (
                Fill::gradient(theme.gradient.mint, rect),
                Color32::WHITE,
                theme.glowing(theme.gradient.mint.to.at(0.35), 3.0),
            ),
            // `.ic-mini-btn`, `.ic-fbtn`, `.ic-clear`
            ButtonVariant::Secondary => {
                let ink = match visual {
                    Visual::Hover => theme.accent.blue,
                    _ => theme.ink.i700,
                };
                let elevation = match pressed {
                    true => theme.elevation.pressed_sm,
                    false => theme.elevation.raised_sm,
                };
                (Fill::Solid(theme.surface.s1), ink, elevation)
            }
            // `.add-scene`
            ButtonVariant::Raised => {
                let ink = match visual {
                    Visual::Hover => theme.accent.blue,
                    _ => theme.ink.i900,
                };
                let elevation = match pressed {
                    true => theme.elevation.pressed_sm,
                    false => theme.elevation.raised,
                };
                (Fill::Solid(theme.surface.s1), ink, elevation)
            }
            // `.ic-fbtn--danger` — coral ink on the ordinary surface.
            ButtonVariant::Danger => {
                let elevation = match pressed {
                    true => theme.elevation.pressed_sm,
                    false => theme.elevation.raised_sm,
                };
                (Fill::Solid(theme.surface.s1), theme.accent.coral, elevation)
            }
            // `.ic-add-person`
            ButtonVariant::Dashed => {
                let ink = match visual {
                    Visual::Hover => theme.accent.blue,
                    _ => theme.ink.i500,
                };
                (Fill::Solid(Color32::TRANSPARENT), ink, Elevation::NONE)
            }
            ButtonVariant::Ghost => {
                let ink = match visual {
                    Visual::Idle => theme.ink.i500,
                    _ => theme.ink.i900,
                };
                (Fill::Solid(Color32::TRANSPARENT), ink, Elevation::NONE)
            }
        }
    }
}

/// `border: 1px dashed var(--ink-300)` round a rounded rectangle.
///
/// `epaint` has no dash pattern, so the outline is walked as one continuous path — four sides **and
/// four corner arcs** — and cut into four-on, three-off segments, which is what a browser draws for
/// `dashed` at this width.
///
/// **The corners are not skipped**, and that is the whole reason this is fifty lines rather than
/// four. Drawing only the straight sides leaves the four corners open, and a pill-shaped control has
/// no straight sides on its short axis at all — so the outline came out as two dashes along the top
/// and bottom and nothing else, which reads as a control that failed to draw rather than as a dashed
/// border.
fn dashed_outline(rux: &mut Rux<'_>, rect: Rect, radius: f32, colour: Color32) {
    let radius = radius.min(rect.width() / 2.0).min(rect.height() / 2.0).max(0.0);
    let (l, r, t, b) = (rect.left(), rect.right(), rect.top(), rect.bottom());
    // The whole perimeter as a polyline, clockwise from the top left, with each corner flattened into
    // short chords so the dashes run round it at an even pace.
    let mut path = Vec::with_capacity(4 * 9 + 4);
    let mut corner = |centre: egui::Pos2, from: f32| {
        const STEPS: usize = 8;
        for step in 0..=STEPS {
            let angle = from + std::f32::consts::FRAC_PI_2 * step as f32 / STEPS as f32;
            path.push(egui::pos2(
                centre.x + radius * angle.cos(),
                centre.y + radius * angle.sin(),
            ));
        }
    };
    // Angles measured with `y` pointing down the screen, so π is the top left corner's start.
    corner(egui::pos2(l + radius, t + radius), std::f32::consts::PI);
    corner(egui::pos2(r - radius, t + radius), -std::f32::consts::FRAC_PI_2);
    corner(egui::pos2(r - radius, b - radius), 0.0);
    corner(egui::pos2(l + radius, b - radius), std::f32::consts::FRAC_PI_2);
    if let Some(first) = path.first().copied() {
        path.push(first);
    }

    let painter = rux.ui.painter();
    let stroke = egui::Stroke::new(1.0, colour);
    let (on, off) = (4.0, 3.0);
    // How far into the current dash we are, carried from one segment to the next so a dash is not
    // restarted at every vertex.
    let mut walked = 0.0_f32;
    for pair in path.windows(2) {
        let (from, to) = (pair[0], pair[1]);
        let span = to - from;
        let length = span.length();
        if length <= f32::EPSILON {
            continue;
        }
        let step = span / length;
        let mut at = 0.0;
        while at < length {
            let cycle = (walked + at) % (on + off);
            if cycle < on {
                let end = (at + (on - cycle)).min(length);
                painter.line_segment([from + step * at, from + step * end], stroke);
                at = end;
            } else {
                at += off - (cycle - on);
            }
        }
        walked += length;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_sizes_are_the_stylesheet_s() {
        // `.ic-create { padding: 14px 20px }`, `.ic-mini-btn { padding: 9px 12px; gap: 6px }`.
        assert_eq!(ButtonSize::Large.padding(), (14.0, 20.0));
        assert_eq!(ButtonSize::Mini.padding(), (9.0, 12.0));
        assert_eq!(ButtonSize::Mini.gap(), 6.0);
        assert_eq!(ButtonSize::Large.style().size, 14.0);
        assert_eq!(ButtonSize::Large.style().weight, crate::text::Weight::SemiBold);
    }

    #[test]
    fn a_pressed_primary_button_swaps_to_the_pressed_recipe() {
        // `.ic-create:active` is two *inset* shadows where the idle state has two outer ones and two
        // inset — a genuinely different declaration, not the same one dimmed.
        let theme = crate::theme::light();
        let rect = Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(120.0, 48.0));
        let button = Button::new("Create").primary().size(ButtonSize::Large);
        let (_, _, idle) = button.skin(theme, Visual::Idle, rect);
        let (_, _, active) = button.skin(theme, Visual::Active, rect);
        assert_ne!(idle, active);
        assert!(active.shadows().all(|shadow| shadow.inset));
        assert_eq!(idle.shadows().count(), 4);
    }

    #[test]
    fn a_secondary_button_takes_the_accent_only_on_hover() {
        // `.ic-mini-btn { color: var(--ink-700) } .ic-mini-btn:hover { color: var(--accent-blue) }`.
        let theme = crate::theme::light();
        let rect = Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(100.0, 34.0));
        let button = Button::new("Prompts");
        assert_eq!(button.skin(theme, Visual::Idle, rect).1, theme.ink.i700);
        assert_eq!(button.skin(theme, Visual::Hover, rect).1, theme.accent.blue);
    }

    #[test]
    fn a_dashed_button_has_no_depth_at_all() {
        // `.ic-add-person { box-shadow: none }` — an invitation rather than an action.
        let theme = crate::theme::light();
        let rect = Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(100.0, 34.0));
        let (_, _, elevation) =
            Button::new("Add Person").variant(ButtonVariant::Dashed).skin(theme, Visual::Idle, rect);
        assert!(elevation.is_empty());
    }
}
