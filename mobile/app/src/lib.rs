//! The mobile application: `rux` components laid out for a phone.
//!
//! **There is no platform code in this crate.** It is an [`eframe::App`] and three screens, and each
//! shell is a dozen lines that construct it:
//!
//! | Shell | What it declares | What loads it |
//! |---|---|---|
//! | `mobile/ios` | a `bin`, and a `staticlib` exporting `rux_mobile_main` | the bundle's executable, or an Xcode project's `main.m` |
//! | `mobile/android/glue` | a `cdylib` exporting `android_main` | `NativeActivity`, by the name in `AndroidManifest.xml` |
//! | `mobile/preview` | a `bin` | nothing: it runs here |
//!
//! That is possible because `eframe` 0.36 supports both platforms: `egui-winit` reads
//! `UIWindow.safeAreaInsets` through `objc2-ui-kit` and feeds it to egui as `RawInput`, and `eframe`
//! has the `android-native-activity` feature and `NativeOptions::android_app`. So a phone runs the same
//! `eframe::App` the storybook runs, and the library underneath is the same `rux`.
//!
//! # The one thing this crate does that the storybook does not
//!
//! It tells `rux` what kind of device it is on, once a frame, before anything draws:
//!
//! ```no_run
//! # let state: rux::RuxState = Default::default();
//! # let size = egui::Vec2::new(393.0, 852.0);
//! let insets = rux::Insets::vertical(59.0, 34.0);   // the Dynamic Island, and the home indicator
//! state.set_screen(rux::Screen::phone(size, insets));
//! ```
//!
//! Every touch target, the sheet that a dialog becomes and the nav strip that fits a narrow width all
//! follow from that one call. Nothing else in the application asks about the platform.

#![allow(clippy::needless_pass_by_value)]

pub mod screens;

use egui::{Rect, Vec2};
use rux::components::{Fit, NavItem, TopNav};
use rux::icon::Icon;
use rux::{Insets, Pointer, Rux, RuxState, Screen, Theme};

use screens::{Controls, Create, Gallery};

/// Which screen is showing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Create,
    Gallery,
    Controls,
}

impl Tab {
    pub const ALL: &'static [Tab] = &[Tab::Create, Tab::Gallery, Tab::Controls];

    /// The name a command line switch uses.
    pub fn name(self) -> &'static str {
        match self {
            Tab::Create => "create",
            Tab::Gallery => "gallery",
            Tab::Controls => "controls",
        }
    }

    pub fn named(name: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|tab| tab.name() == name)
    }

    fn nav(self) -> NavItem {
        match self {
            Tab::Create => NavItem::new("create", Icon::Spark, "Create"),
            Tab::Gallery => NavItem::new("gallery", Icon::Image, "Gallery"),
            Tab::Controls => NavItem::new("controls", Icon::Layers, "Controls"),
        }
    }
}

/// The application.
///
/// It owns the three screens' state and which tab is showing, and nothing else. The library's own state
/// is owned beside it rather than inside it, for the reason the storybook records: a screen draws itself
/// with `&mut self` while `rux::layer` needs a `&RuxState` for the length of the drawing, and one struct
/// cannot lend both halves of itself at once.
pub struct Mobile {
    pub tab: Tab,
    pub create: Create,
    pub gallery: Gallery,
    pub controls: Controls,
    /// What to tell `rux` about the device, when the device cannot be read off egui.
    ///
    /// `None` means read it every frame, which is what a real phone does. The desktop preview sets it,
    /// because a Windows window has no notch to report and the whole point of the preview is to lay out
    /// as though it had one.
    pub forced: Option<Screen>,
    /// The Android readings egui does not have: insets, the keyboard and the back gesture.
    #[cfg(target_os = "android")]
    pub platform: rux_android::Platform,
}

impl Default for Mobile {
    fn default() -> Self {
        Self::new()
    }
}

impl Mobile {
    pub fn new() -> Self {
        Self {
            tab: Tab::Create,
            create: Create::new(),
            gallery: Gallery::new(),
            controls: Controls::new(),
            forced: None,
            #[cfg(target_os = "android")]
            platform: rux_android::Platform::new(),
        }
    }

    /// Lay out as though this were the given device, whatever the window really is.
    pub fn as_device(mut self, screen: Screen) -> Self {
        self.forced = Some(screen);
        self
    }

    pub fn on_tab(mut self, tab: Tab) -> Self {
        self.tab = tab;
        self
    }

    /// The device this frame: what was forced, or what egui reports.
    ///
    /// On a real phone the reading is egui's, with one correction — the pointer is set to `Touch`
    /// outright rather than waiting for `has_touch_screen` to become true, because that only turns true
    /// once a finger has landed and the first frame would otherwise be laid out for a mouse and jump.
    ///
    /// On Android the insets come from `rux-android`, which reads what `RuxActivity` was told by
    /// `onApplyWindowInsets`; egui has no Android reading of its own. An open keyboard counts as bottom
    /// inset, so the content above it stays reachable.
    fn screen(&mut self, ctx: &egui::Context) -> Screen {
        match self.forced {
            Some(screen) => screen,
            None => {
                let screen = Screen::read(ctx).with_pointer(Pointer::Touch);
                #[cfg(target_os = "android")]
                let screen = {
                    let edges = self.platform.screen(ctx);
                    let mut insets = edges.safe;
                    insets.bottom = insets.bottom.max(edges.keyboard);
                    screen.with_insets(insets)
                };
                screen
            }
        }
    }

    /// One frame.
    pub fn ui(&mut self, ui: &mut egui::Ui, state: &RuxState) {
        let screen = self.screen(ui.ctx());
        state.set_screen(screen);

        let window = ui.max_rect();
        // The forced device may be smaller than the window it is being previewed in, so the application
        // is drawn into a rectangle of the device's own size at the top left, and the preview draws the
        // frame round it.
        let area = match self.forced {
            Some(device) => Rect::from_min_size(window.min, device.size.min(window.size())),
            None => window,
        };

        rux::layer(ui, state, egui::Id::new("rux-mobile"), area, |rux| {
            rux.ground(area);
            self.chrome(rux, area, screen);
        });
    }

    /// The frame every screen sits in: the status bar's room, the nav strip, the screen, the home
    /// indicator's room.
    fn chrome(&mut self, rux: &mut Rux<'_>, area: Rect, screen: Screen) {
        // Above the nav is the notch, below the last control is the home indicator. Neither is room the
        // application may draw a control in, and both are room the *background* covers — which is why
        // the ground was painted over the whole area and only the content is inset.
        let safe = screen.safe(area);
        // **A phone application on a tablet is a column, not a stretched phone.** Past the compact
        // breakpoint the content keeps a readable width and is centred, which is what every one column
        // interface does when it is given more room than it can use. A layout that genuinely uses a
        // tablet's width — two columns, the canvas beside the controls — is a different design and is
        // listed in the TDD as not in this version.
        let safe = match screen.is_compact() {
            true => safe,
            false => Rect::from_center_size(
                egui::pos2(safe.center().x, safe.center().y),
                Vec2::new(safe.width().min(READABLE_WIDTH), safe.height()),
            ),
        };
        let items: Vec<NavItem> = Tab::ALL.iter().map(|tab| tab.nav()).collect();
        let active = Tab::ALL.iter().position(|tab| *tab == self.tab).unwrap_or(0);

        let strip = TopNav::new(&items, active);
        let nav_height = strip.measure(rux).y;
        let nav_row = Rect::from_min_size(
            egui::pos2(safe.left() + PAGE_PAD, safe.top() + PAGE_PAD),
            Vec2::new(safe.width() - PAGE_PAD * 2.0, nav_height),
        );
        // It is given the row rather than its own measurement, so `TopNav` is the thing that decides
        // what it gives up to fit — which it does by dropping the active item's label and then trimming
        // its padding. See `Fit`.
        if let Some(index) = strip.show(rux, nav_row).chosen {
            if let Some(tab) = Tab::ALL.get(index) {
                self.tab = *tab;
            }
        }

        let body = Rect::from_min_max(
            egui::pos2(safe.left(), nav_row.bottom() + PAGE_PAD),
            egui::pos2(safe.right(), safe.bottom()),
        );
        match self.tab {
            Tab::Create => self.create.show(rux, body),
            Tab::Gallery => self.gallery.show(rux, body),
            Tab::Controls => self.controls.show(rux, body),
        }
    }

    /// Which fit the nav strip comes out as at a given width, for the preview's own report.
    pub fn nav_fit(&self, rux: &Rux<'_>, width: f32) -> Fit {
        let items: Vec<NavItem> = Tab::ALL.iter().map(|tab| tab.nav()).collect();
        let active = Tab::ALL.iter().position(|tab| *tab == self.tab).unwrap_or(0);
        TopNav::new(&items, active).fit_for(rux, width - PAGE_PAD * 2.0)
    }
}

/// The margin round the page. 16 points is the side gutter every phone interface uses, and it is what
/// keeps a control's rounded corner off the screen's own.
pub const PAGE_PAD: f32 = 16.0;

/// The gap between the things stacked down a screen.
pub const STACK_GAP: f32 = 14.0;

/// How wide the column gets before it stops growing and centres instead.
///
/// 720 points is a little over the widest phone in landscape, so it never applies on a phone and always
/// applies on a tablet.
pub const READABLE_WIDTH: f32 = 720.0;

/// The devices `mobile/preview` can pretend to be, and what each one's screen edges cover.
///
/// The insets are Apple's and Google's published figures for the portrait orientation: an iPhone with a
/// Dynamic Island reserves 59 points at the top and every iPhone with no home button reserves 34 at the
/// bottom, where the indicator is.
pub const DEVICES: &[(&str, Vec2, Insets)] = &[
    ("iphone-16", Vec2::new(393.0, 852.0), Insets::vertical(59.0, 34.0)),
    ("iphone-16-pro-max", Vec2::new(440.0, 956.0), Insets::vertical(62.0, 34.0)),
    ("iphone-se", Vec2::new(375.0, 667.0), Insets::vertical(20.0, 0.0)),
    ("pixel-8", Vec2::new(412.0, 870.0), Insets::vertical(24.0, 24.0)),
];

/// A device by name, as a `Screen` with a touch pointer.
pub fn device(name: &str) -> Option<Screen> {
    DEVICES
        .iter()
        .find(|(id, _, _)| *id == name)
        .map(|(_, size, insets)| Screen::phone(*size, *insets))
}

/// Everything a shell needs to start: the fonts registered, the library's state, the application.
///
/// Every shell calls this, so "what a consumer of `rux` has to do on startup" is written once.
pub fn start(ctx: &egui::Context, theme: &str) -> (Mobile, RuxState) {
    rux::text::install(ctx);
    let theme = Theme::named(theme).unwrap_or_else(|| rux::theme::dark());
    let state = RuxState::new(theme);
    (Mobile::new(), state)
}

// ----------------------------------------------------------------------------------------------- iOS

/// Start the application on iOS.
///
/// It never returns: `winit`'s iOS backend calls `UIApplicationMain` inside `eframe::run_native`, and
/// `UIApplicationMain` does not come back.
///
/// `mobile/ios` is what gives this a symbol an Apple toolchain can call — `rux_mobile_main` for an
/// Xcode project that links the static library, and a `main` for the bundle this repository's own
/// script builds.
#[cfg(target_os = "ios")]
pub fn run_ios() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_title("RUX"),
        ..Default::default()
    };
    let result = eframe::run_native(
        "rux-mobile",
        options,
        Box::new(|creation| {
            let (app, state) = start(&creation.egui_ctx, "dark-neumorphic");
            Ok(Box::new(Shell { app, state }))
        }),
    );
    if let Err(problem) = result {
        // There is no console on a phone, so this reaches the device log through `NSLog`'s own stderr.
        eprintln!("rux-mobile could not start: {problem}");
    }
}

// ------------------------------------------------------------------------------------------- Android

/// Start the application on Android.
///
/// `mobile/android/glue` is what gives this the `android_main` symbol `android-activity` calls. The
/// `AndroidApp` it is handed has to reach `winit`, which is what `NativeOptions::android_app` is for,
/// and `rux_android::install` keeps a clone so the insets, the keyboard and the back gesture can be read
/// every frame.
#[cfg(target_os = "android")]
pub fn run_android(app: winit::platform::android::activity::AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Debug),
    );
    let handle = app.clone();

    let options = rux_android::native_options(app, "RUX");

    let result = eframe::run_native(
        "rux-mobile",
        options,
        Box::new(move |creation| {
            if let Err(problem) = rux_android::install(&handle, &creation.egui_ctx) {
                log::error!("rux-android could not reach RuxBridge: {problem}");
            }
            let (app, state) = start(&creation.egui_ctx, "dark-neumorphic");
            Ok(Box::new(Shell { app, state }))
        }),
    );
    if let Err(problem) = result {
        log::error!("rux-mobile could not start: {problem}");
    }
}

/// The window, for the two shells that have no `main` of their own to hold it.
///
/// `mobile/preview` has its own, because it also draws the device frame round the application.
pub struct Shell {
    pub app: Mobile,
    pub state: RuxState,
}

impl eframe::App for Shell {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        #[cfg(target_os = "android")]
        rux_android::set_light_bars(!self.state.theme().dark);
        self.app.ui(ui, &self.state);
        self.state.end_frame();
    }

    /// On Android, adds the soft keyboard's text to the frame's input, because winit drops it.
    #[cfg(target_os = "android")]
    fn raw_input_hook(&mut self, ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        self.app.platform.raw_input(ctx, raw_input);
    }
}
