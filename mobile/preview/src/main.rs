//! The mobile application, on this machine, inside a phone frame.
//!
//! ```text
//! rux-mobile-preview                          iPhone 16, dark
//! rux-mobile-preview --device iphone-se       another device
//! rux-mobile-preview --theme light-neumorphic
//! rux-mobile-preview --screen gallery         open straight onto a screen
//! rux-mobile-preview --wide                   the same application at 1200 points, to see the
//!                                             expanded size class
//! rux-mobile-preview --background             open without taking the foreground
//! rux-mobile-preview --list                   print the devices and screens and stop
//! ```
//!
//! **Why this exists at all.** The Android emulator is the real thing and it runs here, but it cannot
//! show an iPhone's Dynamic Island inset, and no emulator on this machine can run iOS. This program lays
//! the application out at a named device's logical size with that device's published safe area insets and
//! a touch pointer forced, so the geometry a phone produces is checkable on the machine it is written on.
//!
//! It is a preview of the **layout**, not of the platform: there is no status bar, no keyboard and no
//! system gesture. What it does show is exactly what the application does with the width and the insets it
//! is given, which is what nearly every mobile layout mistake is.
//!
//! A switch it does not know is refused with the list, the same rule the storybook keeps.

use egui::{Rect, Vec2};
use rux_mobile::{device, Mobile, Tab, DEVICES};

/// What the command line asked for.
#[derive(Debug)]
struct Arguments {
    device: String,
    theme: String,
    screen: Option<Tab>,
    /// Lay out at 1200 points instead of a phone, which is `SizeClass::Expanded`.
    wide: bool,
    background: bool,
}

fn main() -> eframe::Result<()> {
    let arguments = match read_arguments(std::env::args().skip(1)) {
        Ok(Some(arguments)) => arguments,
        Ok(None) => return Ok(()),
        Err(problem) => {
            eprintln!("{problem}");
            std::process::exit(2);
        }
    };

    let screen = match arguments.wide {
        // The expanded case is a touch screen that is not compact: nothing becomes a sheet, the nav keeps
        // its label, and every target is still 44. That combination is a tablet, and it is the one thing
        // a phone cannot show.
        true => rux::Screen::phone(Vec2::new(1200.0, 900.0), rux::Insets::NONE),
        false => match device(&arguments.device) {
            Some(screen) => screen,
            None => {
                eprintln!("{}", unknown_device(&arguments.device));
                std::process::exit(2);
            }
        },
    };

    // The window is the device plus the frame drawn round it.
    let window = screen.size + Vec2::new(FRAME * 2.0, FRAME * 2.0 + CAPTION);
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([window.x, window.y])
            .with_active(!arguments.background)
            .with_title("RUX Mobile Preview"),
        ..Default::default()
    };

    eframe::run_native(
        "rux-mobile-preview",
        options,
        Box::new(move |creation| {
            let (mut app, state) = rux_mobile::start(&creation.egui_ctx, &arguments.theme);
            app = app.as_device(screen);
            if let Some(tab) = arguments.screen {
                app = app.on_tab(tab);
            }
            Ok(Box::new(Preview { app, state, screen, device: arguments.device.clone() }))
        }),
    )
}

/// The window: the frame, the caption, and the application inside them.
struct Preview {
    app: Mobile,
    state: rux::RuxState,
    screen: rux::Screen,
    device: String,
}

impl eframe::App for Preview {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let window = ui.max_rect();
        let theme = self.state.theme();

        // Outside the device is a flat surface, so the phone's own background is readable as the phone's.
        ui.painter().rect_filled(window, 0, theme.surface.s0);

        let device = Rect::from_min_size(
            egui::pos2(window.left() + FRAME, window.top() + FRAME),
            self.screen.size,
        );

        // The bezel: a rounded frame with the device's corner radius, and the two insets marked with a
        // hairline so it is obvious which room the application is keeping clear.
        ui.painter().rect_filled(device.expand(FRAME), BEZEL_RADIUS, theme.ink.i900);

        let mut child =
            ui.new_child(egui::UiBuilder::new().max_rect(device).id_salt("rux-mobile-device"));
        child.set_clip_rect(device);
        self.app.ui(&mut child, &self.state);
        self.state.end_frame();

        self.inset_marks(ui, device, theme);
        self.caption(ui, window, device, theme);
    }
}

impl Preview {
    /// Where the notch and the home indicator are, drawn over the application so a control that strays
    /// into either is visible rather than merely wrong.
    fn inset_marks(&self, ui: &egui::Ui, device: Rect, theme: &rux::Theme) {
        let insets = self.screen.insets;
        let hairline = theme.accent.amber.gamma_multiply(0.5);
        if insets.top > 0.0 {
            let y = device.top() + insets.top;
            ui.painter().hline(device.x_range(), y, egui::Stroke::new(1.0, hairline));
        }
        if insets.bottom > 0.0 {
            let y = device.bottom() - insets.bottom;
            ui.painter().hline(device.x_range(), y, egui::Stroke::new(1.0, hairline));
            // The home indicator itself, so the bottom inset is recognisable rather than a stray line.
            let bar = Rect::from_center_size(
                egui::pos2(device.center().x, device.bottom() - insets.bottom / 2.0),
                Vec2::new(device.width() * 0.35, 5.0),
            );
            ui.painter().rect_filled(bar, 2.5, theme.ink.i300);
        }
    }

    /// What is being looked at, under the device: the name, the size, the size class and the nav's fit.
    ///
    /// The nav's fit is there because it is the one thing in this design that changes shape with width, so
    /// a caption that did not say which fit is showing would leave the reader guessing.
    fn caption(&self, ui: &mut egui::Ui, window: Rect, device: Rect, theme: &rux::Theme) {
        let row = Rect::from_min_max(
            egui::pos2(window.left() + FRAME, device.bottom() + FRAME),
            egui::pos2(window.right() - FRAME, window.bottom()),
        );
        let class = match self.screen.class() {
            rux::SizeClass::Compact => "compact",
            rux::SizeClass::Medium => "medium",
            rux::SizeClass::Expanded => "expanded",
        };
        let words = format!(
            "{}  ·  {:.0} x {:.0} pt  ·  {class}  ·  top {:.0} bottom {:.0}  ·  targets {:.0} pt",
            self.device,
            self.screen.size.x,
            self.screen.size.y,
            self.screen.insets.top,
            self.screen.insets.bottom,
            self.screen.min_target(),
        );
        let galley = ui.painter().layout_no_wrap(
            words,
            egui::FontId::new(11.0, egui::FontFamily::Name("rux-mono-500".into())),
            theme.ink.i500,
        );
        ui.painter().galley(
            egui::pos2(row.left(), row.center().y - galley.size().y / 2.0),
            galley,
            theme.ink.i500,
        );
    }
}

/// How much bezel is drawn round the device, and how much room the caption takes under it.
const FRAME: f32 = 14.0;
const CAPTION: f32 = 26.0;
const BEZEL_RADIUS: u8 = 40;

fn read_arguments(args: impl Iterator<Item = String>) -> Result<Option<Arguments>, String> {
    let mut arguments = Arguments {
        device: "iphone-16".to_owned(),
        theme: "dark-neumorphic".to_owned(),
        screen: None,
        wide: false,
        background: false,
    };
    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => {
                println!("{}", usage());
                return Ok(None);
            }
            "--list" => {
                println!("{}", list());
                return Ok(None);
            }
            "--wide" => arguments.wide = true,
            "--background" => arguments.background = true,
            "--device" => {
                let name = args.next().ok_or_else(|| "--device needs a name".to_owned())?;
                if device(&name).is_none() {
                    return Err(unknown_device(&name));
                }
                arguments.device = name;
            }
            "--theme" => {
                arguments.theme = args.next().ok_or_else(|| "--theme needs a name".to_owned())?;
            }
            "--screen" => {
                let name = args.next().ok_or_else(|| "--screen needs a name".to_owned())?;
                arguments.screen = Some(
                    Tab::named(&name).ok_or_else(|| format!("no screen called {name}\n{}", list()))?,
                );
            }
            other => return Err(format!("unknown switch {other}\n{}", usage())),
        }
    }
    Ok(Some(arguments))
}

fn unknown_device(name: &str) -> String {
    format!("no device called {name}\n{}", list())
}

fn usage() -> String {
    "rux-mobile-preview [--device <name>] [--theme <name>] [--screen <name>] [--wide] \
     [--background] [--list]"
        .to_owned()
}

fn list() -> String {
    let mut out = String::from("devices:\n");
    for (name, size, insets) in DEVICES {
        out.push_str(&format!(
            "  {name:<20} {:.0} x {:.0} pt, top {:.0}, bottom {:.0}\n",
            size.x, size.y, insets.top, insets.bottom
        ));
    }
    out.push_str("screens:\n");
    for tab in Tab::ALL {
        out.push_str(&format!("  {}\n", tab.name()));
    }
    out
}
