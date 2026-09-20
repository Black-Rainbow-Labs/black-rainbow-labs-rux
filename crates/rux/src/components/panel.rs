//! Surfaces to put things on: a card, a well, a collapsible group and a divider.
//!
//! `.scene` and `.ic-panel` are cards, `.timeline` and `.ic-prompt-wrap` are wells, `.ic-sub` is a
//! collapsible group with a hairline over it, and `.topbar::after` is a divider.

use egui::{Color32, Rect, Sense, Vec2};

use super::{interact, pointing};
use crate::icon::{Icon, Mark};
use crate::layout::{Cross, Flex, Pad};
use crate::text::Style;
use crate::Rux;

/// Whether a surface stands off the page or is pressed into it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PanelKind {
    /// `.scene { box-shadow: var(--e-raised) }` — a card.
    #[default]
    Raised,
    /// The same, one step quieter: `--e-raised-sm`.
    RaisedSmall,
    /// `.ic-panel` — flat, with a hairline down the edge that faces the page.
    Flat,
}

/// A card or a flat panel.
///
/// Draws the surface and answers the rectangle left inside its padding, so a caller writes
/// `let body = Panel::new().show(rux, rect);` and carries on.
pub struct Panel {
    kind: PanelKind,
    radius: Option<f32>,
    pad: Pad,
    fill: Option<Color32>,
}

impl Default for Panel {
    fn default() -> Self {
        Self::new()
    }
}

impl Panel {
    /// `.scene { border-radius: var(--r-xl); padding: 22px }`.
    pub fn new() -> Self {
        Self { kind: PanelKind::default(), radius: None, pad: Pad::all(22.0), fill: None }
    }

    pub fn kind(mut self, kind: PanelKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = Some(radius);
        self
    }

    pub fn pad(mut self, pad: Pad) -> Self {
        self.pad = pad;
        self
    }

    pub fn fill(mut self, fill: Color32) -> Self {
        self.fill = Some(fill);
        self
    }

    /// Draws the surface and answers what is left inside the padding.
    pub fn show(self, rux: &mut Rux<'_>, rect: Rect) -> Rect {
        let theme = rux.theme();
        let radius = self.radius.unwrap_or(theme.radius.xl);
        let fill = self.fill.unwrap_or(theme.surface.s1);
        match self.kind {
            PanelKind::Raised => {
                rux.chrome.surface(rect, radius, fill, theme.elevation.raised);
            }
            PanelKind::RaisedSmall => {
                rux.chrome.surface(rect, radius, fill, theme.elevation.raised_sm);
            }
            // Into the chrome rather than through `egui`, for the reason `Rux::ground` records: the
            // decoration's texture is filled into a slot reserved before any widget, so a surface
            // painted with `egui` would sit on top of every shadow in the layer.
            PanelKind::Flat => rux.chrome.rect(rect, radius, fill),
        }
        self.pad.inside(rect)
    }
}

/// A surface pressed into the page: `.timeline`, `.ic-prompt-wrap`, `.ic-tile`.
pub struct Well {
    radius: Option<f32>,
    pad: Pad,
    /// `--e-pressed-sm` rather than `--e-pressed`, which is the shallower of the two.
    shallow: bool,
    fill: Option<Color32>,
}

impl Default for Well {
    fn default() -> Self {
        Self::new()
    }
}

impl Well {
    pub fn new() -> Self {
        Self { radius: None, pad: Pad::NONE, shallow: false, fill: None }
    }

    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = Some(radius);
        self
    }

    pub fn pad(mut self, pad: Pad) -> Self {
        self.pad = pad;
        self
    }

    pub fn shallow(mut self) -> Self {
        self.shallow = true;
        self
    }

    pub fn fill(mut self, fill: Color32) -> Self {
        self.fill = Some(fill);
        self
    }

    pub fn show(self, rux: &mut Rux<'_>, rect: Rect) -> Rect {
        let theme = rux.theme();
        let radius = self.radius.unwrap_or(theme.radius.md);
        let elevation = match self.shallow {
            true => theme.elevation.pressed_sm,
            false => theme.elevation.pressed,
        };
        rux.chrome.surface(rect, radius, self.fill.unwrap_or(theme.surface.s1), elevation);
        self.pad.inside(rect)
    }
}

/// What happened to a collapsible group this frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SubGroupOutcome {
    pub toggled: bool,
    /// The rectangle the body may use, which is empty when the group is shut.
    pub body: Rect,
}

/// `.ic-sub` — a heading with a disclosure and a hairline over it.
///
/// The heading is `font-family: var(--font-mono); font-size: 10.5px; text-transform: uppercase;
/// letter-spacing: 0.12em`, and its own mark takes the section's accent: violet for Qualities, blue
/// for Scene, amber for Lighting, rose for Person.
pub struct SubGroup<'a> {
    title: &'a str,
    icon: Icon,
    accent: Color32,
    open: bool,
    /// `.ic-sub:first-of-type { border-top: none; padding-top: 0 }`.
    first: bool,
}

impl<'a> SubGroup<'a> {
    /// `padding-top: 14px` above the heading, and the heading itself is one line of 10.5px mono.
    pub const HEAD_HEIGHT: f32 = 20.0;

    pub fn new(title: &'a str, icon: Icon, accent: Color32, open: bool) -> Self {
        Self { title, icon, accent, open, first: false }
    }

    pub fn first(mut self, first: bool) -> Self {
        self.first = first;
        self
    }

    /// How tall the heading and its hairline come to, which a caller needs before it lays the body out.
    pub fn head_height(&self) -> f32 {
        match self.first {
            true => Self::HEAD_HEIGHT,
            false => Self::HEAD_HEIGHT + 14.0,
        }
    }

    pub fn show(self, rux: &mut Rux<'_>, rect: Rect) -> SubGroupOutcome {
        let theme = rux.theme();
        let top = match self.first {
            true => rect.top(),
            false => {
                // `border-top: 1px solid var(--surface-sunken)` above `padding-top: 14px`.
                rux.chrome.line(
                    rect.left_top(),
                    rect.right_top(),
                    1.0,
                    theme.surface.sunken,
                );
                rect.top() + 14.0
            }
        };
        let head = Rect::from_min_size(
            egui::pos2(rect.left(), top),
            Vec2::new(rect.width(), Self::HEAD_HEIGHT),
        );
        let response = interact(rux, head, "rux-subgroup", Sense::click());
        pointing(rux, &response, true);

        let [mark, label, chev] = Flex::row(head)
            .gap(8.0)
            .cross(Cross::Centre)
            .sized(Vec2::splat(13.0))
            .fill(1.0)
            .sized(Vec2::splat(11.0))
            .solve();
        rux.mark_in(Mark::new(self.icon, 13.0), mark, self.accent);
        let style = Style::mono(10.5).medium().tracking(0.12).upper();
        let ink = theme.ink.i700;
        let galley = rux.elided(style, self.title, ink, label.width());
        crate::text::draw_left_centre(rux.ui.painter(), label.left_center(), galley, ink);
        // `.is-collapsed .ic-sub__chev { transform: rotate(-90deg) }`.
        let turn = match self.open {
            true => 0.0,
            false => -std::f32::consts::FRAC_PI_2,
        };
        rux.mark_in(
            Mark::new(Icon::ChevDown, 11.0).stroke(2.2).turn(turn),
            chev,
            theme.ink.i400,
        );

        response.widget_info(|| {
            egui::WidgetInfo::selected(
                egui::WidgetType::Button,
                true,
                self.open,
                self.title,
            )
        });

        let body = match self.open {
            // `.ic-sub__body { gap: 12px }` under the heading.
            true => Rect::from_min_max(
                egui::pos2(rect.left(), head.bottom() + 12.0),
                rect.max,
            ),
            false => Rect::from_min_size(egui::pos2(rect.left(), head.bottom()), Vec2::ZERO),
        };
        SubGroupOutcome { toggled: response.clicked(), body }
    }
}

/// A hairline across a rectangle.
///
/// `.topbar::after` is `linear-gradient(90deg, transparent, rgba(0,0,0,0.06), transparent)` — a rule
/// that fades out at both ends rather than meeting the window's edges, which is why this is a
/// gradient rather than a line.
pub struct Divider {
    faded: bool,
    colour: Option<Color32>,
}

impl Default for Divider {
    fn default() -> Self {
        Self::new()
    }
}

impl Divider {
    pub fn new() -> Self {
        Self { faded: false, colour: None }
    }

    /// `.topbar::after` — fading out at both ends.
    pub fn faded(mut self) -> Self {
        self.faded = true;
        self
    }

    pub fn colour(mut self, colour: Color32) -> Self {
        self.colour = Some(colour);
        self
    }

    /// Draws a one-point rule along the top of `rect`.
    pub fn show(self, rux: &mut Rux<'_>, rect: Rect) {
        let theme = rux.theme();
        let colour = self.colour.unwrap_or(theme.interaction.hairline);
        if !self.faded {
            rux.chrome.line(rect.left_top(), rect.right_top(), 1.0, colour);
            return;
        }
        // Two gradients back to back, transparent at the ends and the colour in the middle.
        let clear = crate::theme::with_opacity(colour, 0.0);
        let middle = rect.center().x;
        let line = Rect::from_min_size(rect.left_top(), Vec2::new(rect.width(), 1.0));
        let left = Rect::from_min_max(line.left_top(), egui::pos2(middle, line.bottom()));
        let right = Rect::from_min_max(egui::pos2(middle, line.top()), line.right_bottom());
        rux.chrome.rect(
            left,
            0.0,
            crate::Fill::Linear {
                from: left.left_center(),
                to: left.right_center(),
                start: clear,
                end: colour,
            },
        );
        rux.chrome.rect(
            right,
            0.0,
            crate::Fill::Linear {
                from: right.left_center(),
                to: right.right_center(),
                start: colour,
                end: clear,
            },
        );
    }
}
