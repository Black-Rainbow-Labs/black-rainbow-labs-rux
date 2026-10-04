//! What a rux application needs from Android that `egui` and `winit` do not give it.
//!
//! | Need | Where Android keeps it | What this crate does |
//! |---|---|---|
//! | The safe area | `View.onApplyWindowInsets`, Kotlin only | reads `RuxBridge.insets()` once a frame and turns pixels into points |
//! | The soft keyboard's height | the same insets, `Type.ime()` | the same call, fifth value |
//! | The soft keyboard's text | GameActivity's `stateChanged`, Kotlin only | `RuxActivity` records it, this diffs it once a frame into `egui` text and backspace events |
//! | The back gesture | `OnBackPressedDispatcher`, Kotlin only | counts presses in Kotlin, hands them out here, and says when the app wants them |
//! | A repaint when Kotlin changes something | nothing | registers `RuxBridge.nativeWake` with `RegisterNatives`, which asks egui for a frame |
//! | The application's own Kotlin | the activity's class loader | [`with_activity`] and [`load_class`] |
//!
//! The Kotlin half is `mobile/android/rux-shell` (`RuxActivity`, `RuxBridge`). An application calls
//! [`install`] once from its eframe creation closure, [`Platform::screen`] once a frame, and
//! [`Platform::raw_input`] from `eframe::App::raw_input_hook`.
//!
//! **Why winit is not enough.** winit 0.30 shows and hides GameActivity's keyboard when egui asks for it
//! (`set_ime_allowed` calls `show_soft_input`), but it drops every `InputEvent::TextEvent`, so the text the
//! keyboard types never reaches egui. `RuxActivity` records the whole editor text each time it changes, so
//! reading it once a frame and diffing it against the last one recovers every insertion and deletion
//! without touching winit's event loop.
//!
//! Off Android every call is a no-op, so a crate that depends on this compiles on the desktop unchanged.

#[cfg(target_os = "android")]
mod bridge;
pub mod keyboard;

#[cfg(target_os = "android")]
mod options;

#[cfg(target_os = "android")]
pub use bridge::{install, load_class, set_handles_back, with_activity};
#[cfg(target_os = "android")]
pub use options::native_options;
#[cfg(target_os = "android")]
pub use jni;
pub use keyboard::Keyboard;

use rux::Insets;

/// One frame's reading of the device edges, in points.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Edges {
    /// What the system bars and the display cutout cover.
    pub safe: Insets,
    /// How tall the soft keyboard is, 0 when it is closed. It is measured from the bottom of the window,
    /// so it already includes the navigation bar it covers.
    pub keyboard: f32,
}

/// The per application Android state: the keyboard diff and the last edges read.
#[derive(Default)]
pub struct Platform {
    #[cfg(target_os = "android")]
    keyboard: keyboard::Keyboard,
    /// The edges read on the last frame, kept so a frame where JNI fails still has a reading.
    pub edges: Edges,
}

impl Platform {
    /// Creates the state. Call [`install`] separately, once, with the activity.
    pub fn new() -> Self {
        Self::default()
    }

    /// Reads the insets and the keyboard height for this frame, in points.
    /// @param ctx - the egui context, for its pixels per point
    #[cfg(target_os = "android")]
    pub fn screen(&mut self, ctx: &egui::Context) -> Edges {
        if let Some(edges) = bridge::read_edges(ctx.pixels_per_point()) {
            self.edges = edges;
        }
        self.edges
    }

    /// Off Android there are no edges to read.
    /// @param _ctx - unused
    #[cfg(not(target_os = "android"))]
    pub fn screen(&mut self, _ctx: &egui::Context) -> Edges {
        self.edges
    }

    /// Adds this frame's keyboard text to egui's input. Call from `eframe::App::raw_input_hook`.
    /// @param ctx - the egui context, which says whether a text field has focus
    /// @param raw - the input egui is about to run the frame with
    #[cfg(target_os = "android")]
    pub fn raw_input(&mut self, ctx: &egui::Context, raw: &mut egui::RawInput) {
        if let Some(app) = bridge::app() {
            self.keyboard.feed(&app, ctx.egui_wants_keyboard_input(), raw);
        }
    }

    /// Off Android the platform's own keyboard already reaches egui.
    /// @param _ctx - unused
    /// @param _raw - unused
    #[cfg(not(target_os = "android"))]
    pub fn raw_input(&mut self, _ctx: &egui::Context, _raw: &mut egui::RawInput) {}

    /// How many back gestures arrived since the last call.
    #[cfg(target_os = "android")]
    pub fn take_back(&mut self) -> u32 {
        bridge::take_back_presses()
    }

    /// Off Android there is no back gesture.
    #[cfg(not(target_os = "android"))]
    pub fn take_back(&mut self) -> u32 {
        0
    }
}

/// Off Android, whether the app wants the back gesture is nobody's business.
/// @param _handles - unused
#[cfg(not(target_os = "android"))]
pub fn set_handles_back(_handles: bool) {}
