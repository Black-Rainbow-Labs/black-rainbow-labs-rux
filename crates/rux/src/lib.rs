//! **rux** — Black Rainbow Labs' component library.
//!
//! Neumorphic components drawn the way the Unluminous editor draws its own: `egui` for input, hit
//! testing, scrolling and text, and `vello_cpu` for the four things `epaint` cannot draw at all — an
//! inset shadow, a diagonal gradient, a glow round a circle, and a clip with rounded corners. Those
//! four are what the design is made of, so the second renderer is not an embellishment.
//!
//! `tasks/task-2008-rux-tdd.md` is the design and `reference/` is what it is measured against.
//!
//! # Using it
//!
//! ```no_run
//! # use rux::{RuxState, Theme};
//! # struct App { rux: RuxState }
//! # impl App {
//! fn setup(ctx: &egui::Context) -> RuxState {
//!     rux::text::install(ctx);                 // the seven faces, once
//!     RuxState::new(Theme::named("dark-neumorphic").unwrap())
//! }
//!
//! fn frame(&mut self, ui: &mut egui::Ui) {
//!     let area = ui.max_rect();
//!     rux::layer(ui, &self.rux, egui::Id::new("page"), area, |rux| {
//!         rux.ground(area);
//!         // …components…
//!     });
//!     self.rux.end_frame();                    // gives back the canvases nothing drew into
//! }
//! # }
//! ```
//!
//! # The shape every component has
//!
//! ```ignore
//! Button::new("Create").variant(ButtonVariant::Primary).icon(Icon::Spark);
//!
//! fn measure(&self, rux: &Rux) -> Vec2;                  // what it wants, before anybody places it
//! fn show(self, rux: &mut Rux, rect: Rect) -> Response;  // draws into the rect it is given
//! ```
//!
//! Three rules hold for all of them, and they are what makes the library reusable rather than a page
//! taken apart:
//!
//! - **A component decides nothing about the application.** It draws, and it answers what happened.
//!   State that has to survive a frame is the caller's, passed in by `&mut`, which is what lets the
//!   same component appear twice on one screen.
//! - **A component reads the theme and never a colour.** There is no `Color32::from_rgb` anywhere in
//!   `components/`. A name the theme has not got is a name the theme should gain.
//! - **A component sets the cursor the stylesheet sets** — `cursor: pointer` is
//!   `CursorIcon::PointingHand`, `cursor: text` is `CursorIcon::Text`, `cursor: grab` is
//!   `CursorIcon::Grab`.

#![forbid(unsafe_code)]

pub mod components;
pub mod icon;
pub mod layout;
pub mod render;
pub mod screen;
pub mod text;
pub mod theme;

use std::cell::{Cell, RefCell};

use egui::{Color32, Pos2, Rect, Vec2};

pub use icon::{Icon, IconAtlas, Mark};
pub use layout::{Cross, Flex, Grid, Justify, Length, Pad, Pen};
pub use render::{layer, Canvases, Chrome, Corners, Decor, Fill};
pub use screen::{Insets, Pointer, Screen, SizeClass};
pub use text::Style;
pub use theme::{Elevation, Shadow, Theme};

/// Everything the library keeps between frames.
///
/// Two caches and the active theme. Owned by the application rather than hidden in a global, so a
/// consumer can hold more than one — and behind `RefCell`s rather than `&mut`, because a popup opens a
/// **nested** layer while the one that opened it is still being drawn, and a `&mut` borrow could not
/// reach it.
///
/// The borrows are short: a canvas is borrowed for the length of one rasterisation and an atlas for
/// the length of one lookup. Neither is held across a closure the caller supplied.
#[derive(Debug, Default)]
pub struct RuxState {
    pub canvases: RefCell<Canvases>,
    pub icons: RefCell<IconAtlas>,
    theme: Cell<&'static Theme>,
    screen: Cell<Screen>,
    /// How much larger than its reference size everything is drawn. Zero means one, so a state built
    /// with `Default` draws at the reference size.
    zoom: Cell<f32>,
    /// Whether controls animate. On unless a consumer turns it off, for a person who asked for less
    /// motion or for a test that photographs a frame mid gesture.
    still: Cell<bool>,
}

impl RuxState {
    pub fn new(theme: &'static Theme) -> Self {
        Self {
            canvases: RefCell::new(Canvases::default()),
            icons: RefCell::new(IconAtlas::default()),
            theme: Cell::new(theme),
            screen: Cell::new(Screen::default()),
            zoom: Cell::new(1.0),
            still: Cell::new(false),
        }
    }

    /// A state whose canvases and marks rasterise the same on every machine, for the rendering tests.
    pub fn deterministic(theme: &'static Theme) -> Self {
        Self {
            canvases: RefCell::new(Canvases::deterministic()),
            icons: RefCell::new(IconAtlas::deterministic()),
            theme: Cell::new(theme),
            screen: Cell::new(Screen::default()),
            zoom: Cell::new(1.0),
            still: Cell::new(false),
        }
    }

    pub fn theme(&self) -> &'static Theme {
        self.theme.get()
    }

    /// The device the components are being drawn on: how wide, what is covering the edges, and whether
    /// the thing pointing at it is a finger.
    ///
    /// Beside the theme and behind a `Cell` for the same reason: `Rux` is built inside
    /// [`render::layer`] and again inside [`Rux::overlay`], so a field on it would have to be threaded
    /// through every call site including a consumer's own — and a popup opens a nested layer while the
    /// one that opened it is still drawing.
    pub fn screen(&self) -> Screen {
        self.screen.get()
    }

    /// Say what device this is, once a frame, before anything draws.
    ///
    /// An application that does not call this gets [`Screen::default`], which is a desktop mouse — so
    /// a program written before this existed behaves exactly as it did.
    pub fn set_screen(&self, screen: Screen) {
        self.screen.set(screen);
    }

    /// Switch themes. Every canvas is rasterised again on the next frame, because every colour in
    /// every `Decor` changed — which is the comparison in `Canvas::texture_for` doing its job rather
    /// than anything having to be invalidated by hand.
    pub fn set_theme(&self, theme: &'static Theme) {
        self.theme.set(theme);
    }

    /// How much larger than its reference size everything is drawn.
    ///
    /// For a consumer whose pane zooms by a multiplier rather than through a layer's transform: a chat
    /// pane zoomed to 1.5 sets 1.5 here once a frame and every component that reads [`Rux::z`] grows
    /// with it. A layer drawn through a transform needs nothing here; [`render::composited_scale`]
    /// already keeps it sharp.
    pub fn zoom(&self) -> f32 {
        match self.zoom.get() {
            zoom if zoom > 0.0 && zoom.is_finite() => zoom,
            _ => 1.0,
        }
    }

    /// Set the zoom, clamped to a tenth at the least.
    pub fn set_zoom(&self, zoom: f32) {
        self.zoom.set(zoom.max(0.1));
    }

    /// Whether controls should jump to where they are going rather than move there.
    pub fn is_still(&self) -> bool {
        self.still.get()
    }

    /// Turn motion off or back on.
    pub fn set_still(&self, still: bool) {
        self.still.set(still);
    }

    /// Give back the canvases of layers that were not drawn this frame. Called once, at the end.
    pub fn end_frame(&self) {
        self.canvases.borrow_mut().tidy();
    }

    /// What the library is holding, for a measurement pass: canvases, their pixels, marks, their
    /// pixels, and how many rasterisations have happened since the window opened.
    pub fn footprint(&self) -> Footprint {
        let (canvases, canvas_bytes) = self.canvases.borrow().footprint();
        let (marks, mark_bytes) = self.icons.borrow().footprint();
        Footprint {
            canvases,
            canvas_bytes,
            marks,
            mark_bytes,
            canvas_rasterisations: self.canvases.borrow().total_rasterisations(),
            mark_rasterisations: self.icons.borrow().rasterisations(),
        }
    }
}

/// What [`RuxState::footprint`] answers with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Footprint {
    pub canvases: usize,
    pub canvas_bytes: usize,
    pub marks: usize,
    pub mark_bytes: usize,
    pub canvas_rasterisations: u64,
    pub mark_rasterisations: u64,
}

/// What a component is handed: somewhere to put widgets, the theme, and somewhere to record depth.
///
/// Passed by `&mut` down through a page. `ui` is `egui`'s, so anything `egui` can do is still
/// available and a consumer is never boxed in by the library.
pub struct Rux<'a> {
    pub ui: &'a mut egui::Ui,
    pub state: &'a RuxState,
    pub chrome: &'a Chrome,
}

impl<'a> Rux<'a> {
    pub fn theme(&self) -> &'static Theme {
        self.state.theme()
    }

    /// The device this is being drawn on. See [`RuxState::screen`].
    pub fn screen(&self) -> Screen {
        self.state.screen()
    }

    /// The zoom everything is drawn at. See [`RuxState::zoom`].
    pub fn zoom(&self) -> f32 {
        self.state.zoom()
    }

    /// A reference measurement at the current zoom.
    pub fn z(&self, points: f32) -> f32 {
        points * self.state.zoom()
    }

    /// A text style at the current zoom.
    pub fn zs(&self, style: Style) -> Style {
        style.at(style.size * self.state.zoom())
    }

    pub fn painter(&self) -> &egui::Painter {
        self.ui.painter()
    }

    pub fn ctx(&self) -> &egui::Context {
        self.ui.ctx()
    }

    /// How many physical pixels one point of this layer covers, which is what a hairline and a mark
    /// snapped to the pixel grid are measured in.
    pub fn pixels_per_point(&self) -> f32 {
        render::composited_scale(self.ctx(), self.painter().layer_id())
    }

    /// Fill a rectangle with the page's own background: `body { background: var(--surface-1) }`.
    ///
    /// **Recorded into the chrome rather than painted with `egui`, and that ordering is the whole
    /// reason this is a method rather than a `rect_filled` at the call site.** The decoration is one
    /// texture filled into a slot reserved *before* any widget is added, so anything a caller paints
    /// with `egui` lands on top of it — and a ground painted on top of the decoration hides every
    /// shadow in the layer. It looks exactly like the renderer not working.
    ///
    /// It costs nothing to put it here: a flat fill is one `Decor::Rect`, and the canvas already
    /// covers the layer because the layer's own dividers do.
    pub fn ground(&mut self, rect: Rect) {
        let colour = self.theme().surface.s1;
        self.chrome.rect(rect, 0.0, colour);
    }

    /// The same for a recessed area: `.ic-canvas { background: var(--surface-0) }`.
    pub fn ground_sunken(&mut self, rect: Rect) {
        let colour = self.theme().surface.s0;
        self.chrome.rect(rect, 0.0, colour);
    }

    /// Paint a mark, centred, tinted.
    pub fn mark(&mut self, mark: Mark, centre: Pos2, colour: Color32) {
        let painter = self.ui.painter().clone();
        icon::paint(&painter, &mut self.state.icons.borrow_mut(), mark, centre, colour);
    }

    /// Paint a mark centred in a rectangle.
    pub fn mark_in(&mut self, mark: Mark, rect: Rect, colour: Color32) {
        self.mark(mark, rect.center(), colour);
    }

    /// Lay text out with no wrapping.
    pub fn text(&self, style: Style, text: &str, colour: Color32) -> std::sync::Arc<egui::Galley> {
        text::layout(self.ui.painter(), style, text, colour)
    }

    /// Lay text out on one line, cut with `…` when it does not fit.
    pub fn elided(
        &self,
        style: Style,
        text: &str,
        colour: Color32,
        width: f32,
    ) -> std::sync::Arc<egui::Galley> {
        text::elided(self.ui.painter(), style, text, colour, width)
    }

    /// How much room some text takes, without drawing it.
    pub fn measure(&self, style: Style, text: &str) -> Vec2 {
        text::measure(self.ui.painter(), style, text)
    }

    /// A flat wash over a surface: `background: rgba(0, 0, 0, 0.025)` on a hovered row.
    ///
    /// Drawn with `egui`, deliberately — a hover that reached the chrome would rasterise the whole
    /// layer every time the pointer moved down a menu.
    pub fn wash(&mut self, rect: Rect, radius: f32, colour: Color32) {
        self.ui.painter().rect_filled(rect, radius, colour);
    }

    /// Where the pointer last was, in the points this `Ui`'s layer is drawn in.
    ///
    /// `Context::pointer_latest_pos` answers in the window's points, and a layer drawn through a transform,
    /// such as a zoomed modal, a canvas node or an overlay opened from either, lays its rectangles out in its
    /// own. Compared directly, the two agree only at a zoom of one, so a select's menu in a modal zoomed to
    /// 1.25 took a press on its own rows for a press outside and closed before the row was chosen (Unluminous
    /// `task-2277`). A widget that asks "is the pointer over this rectangle" asks this.
    pub fn pointer(&self) -> Option<egui::Pos2> {
        let ctx = self.ui.ctx();
        let at = ctx.pointer_latest_pos()?;
        Some(match ctx.layer_transform_to_global(self.ui.layer_id()) {
            Some(to_global) => to_global.inverse() * at,
            None => at,
        })
    }

    /// Open a foreground layer of this library's own — a dropdown menu, a context menu, a modal.
    ///
    /// It is a layer rather than a painted rectangle because `egui` orders and hit-tests foreground
    /// areas properly, and because the decoration behind a popup has to be filled into **that layer's**
    /// painter. `area` is where it goes; the popup draws into exactly that rectangle.
    pub fn overlay<R>(
        &mut self,
        id: egui::Id,
        area: Rect,
        order: egui::Order,
        add: impl FnOnce(&mut Rux<'_>) -> R,
    ) -> R {
        let state = self.state;
        // **The overlay is drawn through the same transform as the layer it was opened from.** `area` is in
        // that layer's points, so a dropdown opened inside a zoomed modal or a canvas node would otherwise
        // be placed in the window's points, somewhere else entirely, and drawn at the wrong size.
        let ctx = self.ui.ctx().clone();
        if let Some(to_global) = ctx.layer_transform_to_global(self.ui.layer_id()) {
            ctx.set_transform_layer(egui::LayerId::new(order, id), to_global);
        }
        let response = egui::Area::new(id)
            .order(order)
            .fixed_pos(area.min)
            .constrain(false)
            .show(self.ui.ctx(), |ui| {
                ui.set_clip_rect(area.expand(64.0));
                let mut child =
                    ui.new_child(egui::UiBuilder::new().max_rect(area).id_salt("rux-overlay"));
                render::layer(&mut child, state, id.with("chrome"), area.expand(48.0), add)
            });
        response.inner
    }
}

/// Everything a page needs, in one `use`.
pub mod prelude {
    pub use crate::components::*;
    pub use crate::icon::{Icon, Mark};
    pub use crate::layout::{Cross, Flex, Grid, Justify, Pad, Pen};
    pub use crate::text::{Style, Weight};
    pub use crate::screen::{Insets, Pointer, Screen, SizeClass};
    pub use crate::theme::Opacity as _;
    pub use crate::{Rux, RuxState, Theme};
}
