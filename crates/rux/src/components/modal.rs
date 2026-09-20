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
        }
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
        let size = Vec2::new(
            self.size.x.min(screen.width() - MARGIN.x),
            self.size.y.min(screen.height() - MARGIN.y),
        );
        let frame = Rect::from_center_size(screen.center(), size);

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
            rux.chrome.surface(frame, theme.radius.xl, theme.surface.s1, elevation);

            let head_height = HEAD_PAD.vertical() + 32.0;
            let foot_height = match self.footer {
                true => FOOT_PAD.vertical() + 34.0,
                false => 0.0,
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
                rux.chrome.rect(foot, crate::Corners::bottom(theme.radius.xl), theme.surface.s0);
                rux.chrome.line(foot.left_top(), foot.right_top(), 1.0, theme.surface.sunken);
            }

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

        if let (Some(icon), Some(at)) = (self.icon, cell.next()) {
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
