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

mod avatar;
mod button;
mod chip;
mod field;
mod icon_button;
mod list;
mod loader;
mod modal;
mod panel;
mod project_menu;
mod scrollbar;
mod select;
mod stepper;
mod tile;
mod toggle;
mod top_nav;

pub use avatar::Avatar;
pub use button::{Button, ButtonSize, ButtonVariant};
pub use chip::{Chip, ChipVariant};
pub use field::{TextArea, TextAreaOutcome, TextInput, TextInputOutcome};
pub use icon_button::{IconButton, IconButtonSize, IconButtonVariant};
pub use list::{menu_heading, ListItem, ListItemOutcome};
pub use loader::{elapsed_text, Progress, Skeleton, Spinner, SPINNER_DIAMETER};
pub use modal::{split_body, Modal, ModalOutcome};
pub use panel::{Divider, Panel, PanelKind, SubGroup, SubGroupOutcome, Well};
pub use project_menu::{ProjectMenu, ProjectMenuOutcome};
pub use scrollbar::Scrollbar;
pub use select::{Select, SelectOutcome, SelectState, SELECT_MENU_MAX_HEIGHT};
pub use stepper::{Stepper, StepperOutcome, STEPPER_WIDTH};
pub use tile::{Tile, TileKind};
pub use toggle::{DiceToggle, Segmented, SegmentedOutcome, Switch};
pub use top_nav::{NavItem, TopNav, TopNavOutcome};

use egui::{Rect, Response, Sense};

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
    pub fn of(response: &Response, enabled: bool) -> Self {
        match (enabled, response.is_pointer_button_down_on(), response.hovered()) {
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
