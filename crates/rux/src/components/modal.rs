//! A dialog over a dimmed page.
//!
//! `.ic-modal-backdrop` and `.ic-modal`. The component owns the scrim, the frame, the header and the
//! footer, and hands back the three rectangles a caller fills in — because what is *in* a dialog is
//! the application's and what a dialog *is* is the library's.
//!
//! **One thing the reference has that this does not draw**: `backdrop-filter: blur(8px)` on the
//! scrim. Blurring whatever is behind a layer means reading the frame back and filtering it, which is
//! a render pass and a texture the size of the window, every frame the dialog is open. The scrim's
//! own opacity is what does the work; the blur is a refinement, and it is written down here rather
//! than quietly left out.

use egui::{Rect, Vec2};

use crate::icon::{Icon, Mark};
use crate::layout::{Cross, Flex, Pad};
use crate::text::Style;
use crate::Rux;

/// What happened this frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModalOutcome {
    /// The close button, the scrim or `Escape` — all three mean the same thing, so a caller handles
    /// one flag rather than three.
    pub dismissed: bool,
    /// The body, between the header and the footer.
    pub body: Rect,
    /// The footer, or an empty rectangle when there is none.
    pub foot: Rect,
}

/// `width: min(920px, calc(100vw - 48px)); height: min(640px, calc(100vh - 64px))`.
const WIDE: Vec2 = Vec2::new(920.0, 640.0);
const MARGIN: Vec2 = Vec2::new(48.0, 64.0);
/// How much room is left either side of a sheet, and above it at most.
const SHEET_MARGIN: f32 = 12.0;
/// `.ic-sheet__grab` — the handle across the top of a sheet. 36 by 4, in the sunken surface.
const GRABBER: Vec2 = Vec2::new(36.0, 4.0);

/// Where a dialog comes from.
///
/// **A sheet is this component with a different rectangle and a different corner set**, not a second
/// component: the scrim, the frame, the elevation, the header, the body and the footer are the same code
/// and the same tokens. That is why a select's menu and a dialog look like one idea on a phone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Present {
    /// Centred on the window, at `min(920, 100vw - 48)` by `min(640, 100vh - 64)`.
    Dialog,
    /// Up from the bottom edge, the full width less a 12 point margin, with only its top corners
    /// rounded and the device's bottom inset added to its footer.
    Sheet,
    /// A dialog on a mouse and a sheet on a compact screen, which is what a caller usually wants.
    #[default]
    Automatic,
}

impl Present {
    /// Which of the two shapes this comes out as on a given screen.
    fn resolve(self, screen: crate::Screen) -> Self {
        match self {
            Present::Automatic => match screen.is_compact() {
                true => Present::Sheet,
                false => Present::Dialog,
            },
            other => other,
        }
    }
}
/// `.ic-modal__head { padding: 18px 22px }` and `.ic-modal__foot { padding: 14px 22px }`.
const HEAD_PAD: Pad = Pad::axes(18.0, 22.0);
const FOOT_PAD: Pad = Pad::axes(14.0, 22.0);

/// A dialog.
pub struct Modal<'a> {
    title: &'a str,
    /// `.ic-modal__title-icon` — a 28px raised chip with a mark in it, left of the title.
    icon: Option<Icon>,
    /// The tint of that mark. The reference's Prompt Manager uses the amber.
    icon_tint: Option<egui::Color32>,
    size: Vec2,
    /// Whether to reserve a footer strip at the bottom.
    footer: bool,
    /// Whether a press on the scrim dismisses. A dialog asking a question it must have an answer to
    /// says no.
    light_dismiss: bool,
    /// Draw the frame with `.ic-modal`'s own `var(--e-raised-lg), 0 30px 80px …` instead of the token
    /// file's `--e-modal`. See [`Modal::raised`].
    raised: bool,
    present: Present,
}

impl<'a> Modal<'a> {
    pub fn new(title: &'a str) -> Self {
        Self {
            title,
            icon: None,
            icon_tint: None,
            size: WIDE,
            footer: false,
            light_dismiss: true,
            raised: false,
            present: Present::Automatic,
        }
    }

    /// Where it comes from: centred, up from the bottom edge, or whichever suits the screen.
    pub fn present(mut self, present: Present) -> Self {
        self.present = present;
        self
    }

    /// Draw the frame the way `.ic-modal` does, rather than the way the token file says to.
    ///
    /// **The two disagree, and the token file wins by default.** `.ic-modal` says
    /// `box-shadow: var(--e-raised-lg), 0 30px 80px rgba(20,25,36,0.4)`, and `--e-raised-lg` pairs a
    /// bright top-left glow with a grey bottom-right shadow. That is right on the light canvas it was
    /// drawn for and wrong over a dimmed backdrop, where the pale half reads as a grey halo bleeding
    /// out of the dialog — which is visible in the dark theme and is exactly what
    /// `reference/neumorphic-tokens.css` says `--e-modal` was added to fix, in its own comment.
    ///
    /// So the default is `--e-modal` and this is here for a caller that wants the page's own look.
    pub fn raised(mut self) -> Self {
        self.raised = true;
        self
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn icon_tint(mut self, colour: egui::Color32) -> Self {
        self.icon_tint = Some(colour);
        self
    }

    /// The size it would like, before it is clamped to the window.
    pub fn size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }

    pub fn footer(mut self, footer: bool) -> Self {
        self.footer = footer;
        self
    }

    pub fn light_dismiss(mut self, light_dismiss: bool) -> Self {
        self.light_dismiss = light_dismiss;
        self
    }

    /// Draw the scrim and the frame, and answer where the body and the footer are.
    ///
    /// `add` draws the contents, and is given a `Rux` whose chrome is the **dialog's own layer** —
    /// which it has to be, because `egui` draws a foreground area on a layer the page's painter cannot
    /// reach.
    pub fn show<R>(
        self,
        rux: &mut Rux<'_>,
        add: impl FnOnce(&mut Rux<'_>, ModalOutcome) -> R,
    ) -> (ModalOutcome, R) {
        // `content_rect` rather than `viewport_rect`: `100vw` in the reference means the part of the
        // window a person can actually see, which on a platform with a notch or a status bar is not
        // the whole viewport.
        let screen = rux.ui.ctx().content_rect();
        let present = self.present.resolve(rux.screen());
        let frame = match present {
            Present::Sheet => sheet_frame(screen, self.size.y, self.footer),
            // `Present::Automatic` has already been resolved to one of the two.
            _ => {
                let size = Vec2::new(
                    self.size.x.min(screen.width() - MARGIN.x),
                    self.size.y.min(screen.height() - MARGIN.y),
                );
                Rect::from_center_size(screen.center(), size)
            }
        };

        // The scrim is its own layer under the dialog, so a press anywhere on the page is caught.
        let scrim_id = egui::Id::new("rux-modal-scrim");
        let scrim = egui::Area::new(scrim_id)
            .order(egui::Order::Middle)
            .fixed_pos(screen.min)
            .show(rux.ui.ctx(), |ui| {
                let response = ui.allocate_rect(screen, egui::Sense::click());
                ui.painter().rect_filled(screen, 0, rux.state.theme().interaction.scrim);
                response
            })
            .inner;

        let escaped = rux.ui.ctx().input(|input| input.key_pressed(egui::Key::Escape));
        let mut dismissed = escaped || (self.light_dismiss && scrim.clicked());

        let id = egui::Id::new("rux-modal");
        let (closed, result) = rux.overlay(id, frame, egui::Order::Foreground, |rux| {
            let theme = rux.theme();
            // `--e-modal` by default; `.ic-modal`'s own recipe when a caller asked for it. See
            // [`Modal::raised`] for why the two differ and why the token file wins.
            let elevation = match self.raised {
                true => crate::theme::Elevation::of(
                    theme.elevation.raised_lg.shadows().chain([crate::theme::Shadow::outer(
                        0.0,
                        30.0,
                        80.0,
                        crate::theme::rgba(0x141924, 0.4),
                    )]),
                ),
                false => theme.elevation.modal,
            };
            // A sheet is square along the edge it came from, because it is against that edge.
            let corners = match present {
                Present::Sheet => crate::Corners::top(theme.radius.xl),
                _ => crate::Corners::all(theme.radius.xl),
            };
            rux.chrome.surface(frame, corners, theme.surface.s1, elevation);

            // The handle, and the room it takes above the header.
            let grab = match present {
                Present::Sheet => {
                    let at = Rect::from_center_size(
                        egui::pos2(frame.center().x, frame.top() + 10.0),
                        GRABBER,
                    );
                    rux.chrome.rect(at, GRABBER.y / 2.0, theme.surface.sunken);
                    20.0
                }
                _ => 0.0,
            };
            let frame = Rect::from_min_max(
                egui::pos2(frame.left(), frame.top() + grab),
                frame.max,
            );

            let head_height = HEAD_PAD.vertical() + 32.0;
            // On a sheet the device's bottom inset is added to the footer, so the last control in the
            // dialog is not under the home indicator. With no footer it is added to the body instead,
            // which is what `sheet_frame` has already allowed for.
            let inset = match present {
                Present::Sheet => rux.screen().insets.bottom,
                _ => 0.0,
            };
            let foot_height = match self.footer {
                true => FOOT_PAD.vertical() + 34.0 + inset,
                false => inset,
            };
            let head = Rect::from_min_size(frame.min, Vec2::new(frame.width(), head_height));
            let foot = match self.footer {
                true => Rect::from_min_size(
                    egui::pos2(frame.left(), frame.bottom() - foot_height),
                    Vec2::new(frame.width(), foot_height),
                ),
                false => Rect::from_min_size(frame.left_bottom(), Vec2::ZERO),
            };
            let body = Rect::from_min_max(
                egui::pos2(frame.left(), head.bottom()),
                egui::pos2(frame.right(), frame.bottom() - foot_height),
            );

            let closed = self.draw_head(rux, head);
            // `border-bottom: 1px solid var(--surface-sunken)`.
            rux.chrome.line(
                head.left_bottom(),
                head.right_bottom(),
                1.0,
                theme.surface.sunken,
            );
            if self.footer {
                // `.ic-modal__foot { background: var(--surface-0); border-top: … }`.
                let foot_corners = match present {
                    Present::Sheet => crate::Corners::NONE,
                    _ => crate::Corners::bottom(theme.radius.xl),
                };
                rux.chrome.rect(foot, foot_corners, theme.surface.s0);
                rux.chrome.line(foot.left_top(), foot.right_top(), 1.0, theme.surface.sunken);
            }

            // What a caller fills is the footer less the inset: the inset is clearance, not room.
            let foot = match inset > 0.0 && self.footer {
                true => Rect::from_min_max(
                    foot.min,
                    egui::pos2(foot.right(), foot.bottom() - inset),
                ),
                false => foot,
            };
            let outcome = ModalOutcome { dismissed: false, body, foot };
            (closed, add(rux, outcome))
        });
        dismissed |= closed;

        (ModalOutcome { dismissed, body: Rect::NOTHING, foot: Rect::NOTHING }, result)
    }

    /// `.ic-modal__head` — the chip, the title and the close button.
    fn draw_head(&self, rux: &mut Rux<'_>, head: Rect) -> bool {
        let theme = rux.theme();
        let words = rux.measure(Style::sans(15.0).semibold(), self.title);
        let mut line = Flex::row(head).pad(HEAD_PAD).gap(10.0).cross(Cross::Centre);
        if self.icon.is_some() {
            line = line.sized(Vec2::splat(28.0));
        }
        let cells = line.hug(words.x).fill(1.0).sized(Vec2::splat(32.0)).solve_all();
        let mut cell = cells.iter();

        // `cell.next()` only when there IS an icon: reading it inside a tuple pattern advanced the
        // iterator whether or not the branch matched, so a dialog with no icon lost its title.
        if let Some((icon, at)) = self.icon.and_then(|icon| cell.next().map(|at| (icon, at))) {
            // `.ic-modal__title-icon { width: 28px; border-radius: var(--r-sm); box-shadow: var(--e-raised-sm) }`.
            rux.chrome.surface(*at, theme.radius.sm, theme.surface.s1, theme.elevation.raised_sm);
            let tint = self.icon_tint.unwrap_or(theme.accent.amber);
            rux.mark_in(Mark::new(icon, 14.0), *at, tint);
        }
        if let Some(at) = cell.next() {
            let ink = theme.ink.i900;
            let galley = rux.text(Style::sans(15.0).semibold(), self.title, ink);
            crate::text::draw_left_centre(rux.ui.painter(), at.left_center(), galley, ink);
        }
        let _spacer = cell.next();
        match cell.next() {
            Some(at) => super::IconButton::new(Icon::X, "Close")
                .size(super::IconButtonSize::Medium)
                .show(rux, *at)
                .clicked(),
            None => false,
        }
    }
}

/// Where a sheet sits: against the bottom edge, the full width less a margin, as tall as it asked for.
///
/// It is a free function rather than a method so it can be tested with no window, which is the only way
/// the geometry gets checked at all.
///
/// The height is capped so that a margin's worth of the page is always still visible above it — a sheet
/// that reaches the top edge is a dialog with square corners, and a person cannot tell what dismissing
/// it would go back to.
pub fn sheet_frame(screen: Rect, wanted_height: f32, _footer: bool) -> Rect {
    let width = screen.width() - SHEET_MARGIN * 2.0;
    let height = wanted_height.min(screen.height() - SHEET_MARGIN * 4.0);
    Rect::from_min_size(
        egui::pos2(screen.left() + SHEET_MARGIN, screen.bottom() - height),
        Vec2::new(width, height),
    )
}

/// The two halves `.ic-modal__body` is split into: `grid-template-columns: 260px 1fr`.
///
/// A helper rather than a component, because what goes in each half is entirely the application's —
/// but the hairline between them and the 260 are the reference's and belong here.
pub fn split_body(rux: &mut Rux<'_>, body: Rect, left_width: f32) -> (Rect, Rect) {
    let theme = rux.theme();
    let [left, right] = Flex::row(body).fixed(left_width).fill(1.0).solve();
    rux.chrome.line(left.right_top(), left.right_bottom(), 1.0, theme.surface.sunken);
    (left, right)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::screen::{Insets, Screen};

    fn phone() -> Rect {
        Rect::from_min_size(egui::pos2(0.0, 0.0), Vec2::new(393.0, 852.0))
    }

    #[test]
    fn a_sheet_is_against_the_bottom_edge_and_the_full_width_less_its_margin() {
        let frame = sheet_frame(phone(), 520.0, false);
        assert_eq!(frame.left(), SHEET_MARGIN);
        assert_eq!(frame.right(), 393.0 - SHEET_MARGIN);
        assert_eq!(frame.bottom(), 852.0);
        assert_eq!(frame.height(), 520.0);
    }

    #[test]
    fn a_sheet_taller_than_the_screen_still_leaves_the_page_showing_above_it() {
        let frame = sheet_frame(phone(), 4000.0, true);
        assert_eq!(frame.bottom(), 852.0);
        assert_eq!(frame.top(), SHEET_MARGIN * 4.0);
        assert!(frame.top() > 0.0, "a sheet that reaches the top edge is not a sheet");
    }

    #[test]
    fn automatic_is_a_dialog_on_a_desktop_and_a_sheet_on_a_phone() {
        let desk = Screen::desktop(Vec2::new(1440.0, 900.0));
        let phone = Screen::phone(Vec2::new(393.0, 852.0), Insets::vertical(59.0, 34.0));
        assert_eq!(Present::Automatic.resolve(desk), Present::Dialog);
        assert_eq!(Present::Automatic.resolve(phone), Present::Sheet);
        // An explicit choice is never overridden by the screen.
        assert_eq!(Present::Dialog.resolve(phone), Present::Dialog);
        assert_eq!(Present::Sheet.resolve(desk), Present::Sheet);
    }
}
