//! The components.
//!
//! One file each, named after the rule in `reference/image-creator.css` it was read out of, and every
//! one the same shape:
//!
//! ```ignore
//! fn measure(&self, rux: &Rux) -> Vec2;                  // what it wants, before anybody places it
//! fn show(self, rux: &mut Rux, rect: Rect) -> Response;  // draws into the rect it is given
//! ```
//!
//! Three rules hold for all of them. A component **decides nothing about the application** — it draws,
//! it answers what happened, and state that has to survive a frame is the caller's. It **reads the
//! theme and never a colour**. And it **sets the cursor the stylesheet sets**, because the ticket asks
//! for the right cursor and `cursor: pointer`, `cursor: text` and `cursor: grab` are all written down
//! in the reference.

mod answer;
mod avatar;
mod button;
mod cards;
mod chart;
mod chip;
mod dial;
mod field;
mod fader;
mod icon_button;
pub mod instrument;
mod key;
mod list;
mod loader;
mod modal;
mod panel;
mod project_menu;
mod round;
mod scrollbar;
mod select;
mod status;
mod stepper;
mod table;
mod tile;
mod toggle;
mod top_nav;

pub use answer::{
    card_header, chat_card, chat_raised_sm, chat_well, chat_well_deep, spark_points, FileChip, Legend, LegendRow,
    NotePill, Readout, Sparkline, ThinkingPill, CARD_RADIUS, CHART_RADIUS, WELL_RADIUS,
};
pub use avatar::Avatar;
pub use button::{Button, ButtonSize, ButtonVariant};
pub use cards::{CardOutcome, CodeBlock, CommandCard, Corner, DiffCard, ErrorCard, Lead, Line, PermissionCard, Sign};
pub use chart::{nice_ticks, Chart, ChartKind, ChartOutcome, Series};
pub use chip::{Chip, ChipVariant};
pub use dial::{arc_segments, smooth, AreaChart, Dial, Ring, Segment, Slice, SlicePaint, Track};
pub use fader::{Fader, FaderOutcome, Meter};
pub use instrument::Instrument;
pub use key::{Checkbox, Key};
pub use field::{TextArea, TextAreaOutcome, TextInput, TextInputOutcome};
pub use icon_button::{IconButton, IconButtonSize, IconButtonVariant};
pub use list::{menu_heading, ListItem, ListItemOutcome};
pub use loader::{elapsed_text, Progress, Skeleton, Spinner, SPINNER_DIAMETER};
pub use modal::{sheet_frame, split_body, Modal, ModalOutcome, Present};
pub use panel::{Divider, Panel, PanelKind, SubGroup, SubGroupOutcome, Well};
pub use project_menu::{ProjectMenu, ProjectMenuOutcome};
pub use round::{PillButton, PillSwitch, PillVariant, RoundButton, RoundContent, RoundSize};
pub use scrollbar::Scrollbar;
pub use select::{Select, SelectOutcome, SelectState, SELECT_MENU_MAX_HEIGHT};
pub use status::{Status, StatusWell};
pub use stepper::{Stepper, StepperOutcome, STEPPER_WIDTH};
pub use table::{is_figure, Align, Column, Item, Stage, Table, TableOutcome, Timeline};
pub use tile::{Tile, TileKind};
pub use toggle::{DiceToggle, Segmented, SegmentedOutcome, Switch};
pub use top_nav::{Fit, NavItem, TopNav, TopNavOutcome};

use egui::{Rect, Response, Sense, Vec2};

use crate::screen::Pointer;
use crate::Rux;

/// Which of a control's four states it is in this frame.
///
/// The reference writes three of them as pseudo-classes — `:hover`, `:active` and `[disabled]` — and a
/// component reads this rather than asking about the pointer four times.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visual {
    Idle,
    Hover,
    /// `:active` — held down. This is the one that changes the elevation, so it is also the one that
    /// legitimately rasterises the layer again.
    Active,
    Disabled,
}

impl Visual {
    /// Which state a control is in, for the pointer that is pointing at it.
    ///
    /// **`Pointer::Touch` never answers `Hover`.** A finger is either on the control or off the screen,
    /// and egui keeps a touch's last position after the finger lifts — so on a phone a hover state
    /// would stay behind on whatever was last tapped. Nineteen rules in `reference/image-creator.css`
    /// change a colour on `:hover`, and this is the one place that decides whether any of them apply.
    pub fn of(response: &Response, enabled: bool, pointer: Pointer) -> Self {
        match (enabled, response.is_pointer_button_down_on(), response.hovered() && pointer.hovers())
        {
            (false, _, _) => Visual::Disabled,
            (true, true, _) => Visual::Active,
            (true, false, true) => Visual::Hover,
            _ => Visual::Idle,
        }
    }

    pub fn is_disabled(self) -> bool {
        matches!(self, Visual::Disabled)
    }

    /// How much of its colour a disabled control keeps. `epaint` multiplies, so this is applied to
    /// whatever the theme said rather than to a second, dimmer palette.
    pub fn fade(self) -> f32 {
        match self {
            Visual::Disabled => 0.4,
            _ => 1.0,
        }
    }
}

/// Claim a rectangle for a control, with an id derived from where it is.
///
/// A component library cannot make the caller supply an id for every control, and `egui`'s own
/// auto-counter shifts the moment the number of widgets above changes — which silently moves a text
/// box's focus to a different box. Deriving it from the position is stable across frames and unique
/// within a layer, which is what the id is for.
pub(crate) fn interact(rux: &mut Rux<'_>, rect: Rect, salt: &str, sense: Sense) -> Response {
    let id = rux
        .ui
        .id()
        .with((salt, rect.min.x.to_bits(), rect.min.y.to_bits(), rect.width().to_bits()));
    rux.ui.interact(rect, id, sense)
}

/// `cursor: pointer`.
pub(crate) fn pointing(rux: &Rux<'_>, response: &Response, enabled: bool) {
    if response.hovered() && enabled {
        rux.ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
}

/// Whether a control should draw itself as pointed at.
///
/// `Response::hovered` on its own is not the question, because a finger does not hover. See
/// [`Visual::of`].
pub(crate) fn hovering(rux: &Rux<'_>, response: &Response) -> bool {
    response.hovered() && rux.screen().pointer.hovers()
}

/// Grow a rectangle about its own centre until neither side is under the pointer's minimum target.
///
/// On a mouse the minimum is zero and this hands back exactly what it was given, which is why every
/// desktop measurement is unchanged.
///
/// **Use this for a control whose drawn shape must not grow** — a circle, a switch track, a scrollbar
/// lane. A control that can simply be taller grows in `measure` instead, and that is the better of the
/// two: two 28 point circles six points apart claim 44 point targets that overlap by ten, and egui
/// gives an overlapping point to whichever widget asked last, so the second one silently eats a strip
/// of the first. Growing the layout cannot do that, because the layout is what keeps controls apart.
/// The mobile screens therefore place with `measure`.
pub(crate) fn touch_target(rux: &Rux<'_>, rect: Rect) -> Rect {
    let min = rux.screen().min_target();
    Rect::from_center_size(
        rect.center(),
        Vec2::new(rect.width().max(min), rect.height().max(min)),
    )
}

/// Claim a rectangle for a control whose drawn shape is smaller than its target.
pub(crate) fn interact_target(
    rux: &mut Rux<'_>,
    rect: Rect,
    salt: &str,
    sense: Sense,
) -> Response {
    let target = touch_target(rux, rect);
    interact(rux, target, salt, sense)
}
