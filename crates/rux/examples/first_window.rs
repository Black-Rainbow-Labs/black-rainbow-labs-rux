//! The smallest whole rux program: a window with a search field, a select, a switch and a button.
//!
//! `docs/getting-started.md` walks through it line by line, and it is compiled with the crate's examples
//! so the page cannot drift from the API.
//!
//! ```text
//! cargo run --release -p rux --example first_window
//! cargo run --release -p rux --example first_window -- light-neumorphic
//! ```

use rux::prelude::*;

/// The window: the library's state beside the page's.
///
/// Two fields, because `rux::layer` borrows the `RuxState` for the whole frame while the page is drawn
/// with `&mut`. Keeping them side by side lets the closure borrow both.
struct App {
    rux: RuxState,
    page: Page,
}

/// Everything the page shows. A component keeps no state of its own, so it all lives here.
struct Page {
    query: String,
    styles: Vec<String>,
    style: Option<usize>,
    style_menu: SelectState,
    hd: bool,
    renders: u32,
}

impl App {
    /// Install the faces once and choose the theme.
    fn new(ctx: &egui::Context, theme: &'static Theme) -> Self {
        rux::text::install(ctx);
        Self { rux: RuxState::new(theme), page: Page::new() }
    }
}

impl Page {
    /// The values the page opens with.
    fn new() -> Self {
        Self {
            query: String::new(),
            styles: ["film photography", "digital photo", "oil painting"].iter().map(|style| style.to_string()).collect(),
            style: Some(0),
            style_menu: SelectState::default(),
            hd: true,
            renders: 0,
        }
    }

    /// One frame: a panel in the middle of the window with a heading and four controls in a column.
    fn draw(&mut self, rux: &mut Rux<'_>, area: egui::Rect) {
        rux.ground(area);
        let card = egui::Rect::from_center_size(area.center(), egui::vec2(420.0, 300.0));
        let inside = Panel::new().show(rux, card);
        let title = rux.text(Style::H3, "Render settings", rux.theme().ink.i900);
        let [heading, search, select, switch_row, button_row] = Flex::column(inside)
            .gap(16.0)
            .fixed(title.size().y)
            .fixed(42.0)
            .fixed(40.0)
            .fixed(24.0)
            .fill(1.0)
            .solve();
        rux.ui.painter().galley(heading.left_top(), title, rux.theme().ink.i900);

        TextInput::search(&mut self.query).hint("Search presets").show(rux, search);

        let outcome = Select::new(&self.styles, self.style).label("Style").show(rux, select, &mut self.style_menu);
        if let Some(index) = outcome.chosen {
            self.style = Some(index);
        }

        self.draw_switch_row(rux, switch_row);

        let words = if self.renders == 0 { "Create".to_owned() } else { format!("Create again ({})", self.renders) };
        let create = Button::new(&words).primary().icon(Icon::Spark).size(ButtonSize::Large).stretch();
        let wanted = create.measure(rux);
        let [_, button] = Flex::column(button_row).cross(Cross::Stretch).fill(1.0).fixed(wanted.y).solve();
        if create.show(rux, button).clicked() {
            self.renders += 1;
        }
    }

    /// A label on the left and a switch on the right.
    fn draw_switch_row(&mut self, rux: &mut Rux<'_>, row: egui::Rect) {
        let switch = Switch::new(self.hd).label("High definition");
        let [label, control] = Flex::row(row).cross(Cross::Centre).fill(1.0).sized(switch.measure(rux)).solve();
        let ink = rux.theme().ink.i700;
        let words = rux.text(Style::LABEL, "High definition", ink);
        rux::text::draw_left_centre(rux.painter(), label.left_center(), words, ink);
        if switch.show(rux, control).clicked() {
            self.hd = !self.hd;
        }
    }
}

impl eframe::App for App {
    /// `rux::layer` reserves the slot the decoration goes in, runs the drawing, and fills the slot.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let area = ui.max_rect();
        rux::layer(ui, &self.rux, egui::Id::new("page"), area, |rux| self.page.draw(rux, area));
        self.rux.end_frame();
    }
}

/// Open the window on the theme named on the command line, or the dark one.
///
/// `--background` opens it without taking the foreground, which is how `docs/` photographs it.
fn main() -> eframe::Result<()> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let background = arguments.iter().any(|argument| argument == "--background");
    let name = arguments.iter().find(|argument| !argument.starts_with("--")).cloned().unwrap_or_else(|| "dark-neumorphic".to_owned());
    let Some(theme) = Theme::named(&name) else {
        eprintln!("There is no theme called {name:?}. These exist: {}", Theme::names().join(", "));
        std::process::exit(2);
    };
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([720.0, 520.0]).with_active(!background).with_title("rux: first window"),
        ..Default::default()
    };
    eframe::run_native("rux-first-window", options, Box::new(move |creation| Ok(Box::new(App::new(&creation.egui_ctx, theme)))))
}
