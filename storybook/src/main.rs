//! The RUX storybook: every component, every state, both themes, and the example pages.
//!
//! ```text
//! rux-storybook                       open it
//! rux-storybook --theme light-neumorphic   start on a named theme
//! rux-storybook --measure             show what the library is holding, in the sidebar
//! rux-storybook --list                print every story and exit
//! ```
//!
//! A switch the program does not know is **refused with the list**, rather than being read as
//! something else — which is the rule the library keeps everywhere it is asked to name something
//! built in.

mod app;
mod stories;

use app::Storybook;

/// What the command line asked for.
#[derive(Debug)]
struct Arguments {
    theme: Option<String>,
    measure: bool,
    list: bool,
    /// Open without taking the foreground.
    ///
    /// Not a nicety: a script that drives this window would otherwise have to activate it, which on
    /// Windows switches the virtual desktop with it and takes the keyboard out of whatever the person
    /// was typing into. `with_active(false)` reaches `winit`, which turns it into `SW_SHOWNOACTIVATE`.
    background: bool,
    /// Open straight onto a story, by name.
    ///
    /// The other half of `--background`: a capture pass can reach any story with no pointer input at
    /// all, so photographing the whole board never touches the keyboard or the mouse.
    story: Option<String>,
    /// Open the states that need a press to reach: the dropdowns and the dialog.
    ///
    /// A storybook whose popup states could only be photographed by driving the pointer would be a
    /// storybook whose popup states nobody photographs, so they are reachable from the command line
    /// like everything else.
    open: bool,
    /// Start with the sidebar filtered, which is how the filter's own state is photographed.
    filter: Option<String>,
    /// Open the window at this inner size, in points, instead of 1440 by 920.
    ///
    /// A story is taller than the default window, and a photograph of the window shows only its top.
    /// `--size 1440x2400` lets a capture pass show a whole story without scrolling it.
    size: Option<[f32; 2]>,
    /// Open the story scrolled down by this many points.
    ///
    /// Windows will not make a window taller than the screen, so a story taller than that is
    /// photographed in slices, one `--scroll` each.
    scroll: Option<f32>,
}

fn main() -> eframe::Result<()> {
    let arguments = match read_arguments(std::env::args().skip(1)) {
        Ok(Some(arguments)) => arguments,
        // `--help`, `--list` and `--version` all print and stop.
        Ok(None) => return Ok(()),
        Err(problem) => {
            eprintln!("{problem}");
            std::process::exit(2);
        }
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(arguments.size.unwrap_or([1440.0, 920.0]))
            .with_min_inner_size([760.0, 560.0])
            .with_active(!arguments.background)
            .with_title("RUX Storybook"),
        ..Default::default()
    };
    eframe::run_native(
        "rux-storybook",
        options,
        Box::new(move |creation| {
            let state = rux::RuxState::new(
                rux::Theme::named("dark-neumorphic").unwrap_or_else(rux::theme::light),
            );
            let mut storybook = Storybook::new(&creation.egui_ctx, arguments.measure);
            if let Some(name) = &arguments.theme {
                if let Err(problem) = storybook.choose_theme(&state, name) {
                    eprintln!("{problem}");
                    std::process::exit(2);
                }
            }
            if let Some(name) = &arguments.story {
                if let Err(problem) = storybook.choose_story(name) {
                    eprintln!("{problem}");
                    std::process::exit(2);
                }
            }
            if arguments.open {
                storybook.open_everything();
            }
            if let Some(filter) = &arguments.filter {
                storybook.set_filter(filter);
            }
            if let Some(points) = arguments.scroll {
                storybook.scroll_to(points);
            }
            Ok(Box::new(Application { storybook, state }))
        }),
    )
}

/// The window.
///
/// **The library's state is owned here rather than by the storybook**, and that is not tidiness: the
/// storybook draws itself with `&mut self` while `rux::layer` needs a `&RuxState` for the length of
/// the drawing, and one struct cannot lend both halves of itself at once. Owning them side by side is
/// what a consumer of the library does anyway.
struct Application {
    storybook: Storybook,
    state: rux::RuxState,
}

impl eframe::App for Application {
    /// The `Ui` eframe hands over has no margin and no background of its own, which is what the
    /// library wants: it paints its own ground.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.storybook.ui(ui, &self.state);
        self.state.end_frame();
    }
}

/// Read the command line.
///
/// `Ok(None)` means something was printed and the program should stop, which is what `--help` and
/// `--list` are. An unknown switch is an error with a usage line rather than being taken as a value.
fn read_arguments(
    arguments: impl Iterator<Item = String>,
) -> Result<Option<Arguments>, String> {
    let mut out = Arguments {
        theme: None,
        measure: false,
        list: false,
        background: false,
        story: None,
        open: false,
        filter: None,
        size: None,
        scroll: None,
    };
    let mut arguments = arguments.peekable();
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--measure" => out.measure = true,
            "--list" => out.list = true,
            "--background" => out.background = true,
            "--open" => out.open = true,
            "--theme" => match arguments.next() {
                Some(name) => out.theme = Some(name),
                None => return Err("--theme needs a name after it".to_owned()),
            },
            "--story" => match arguments.next() {
                Some(name) => out.story = Some(name),
                None => return Err("--story needs a name after it".to_owned()),
            },
            "--filter" => match arguments.next() {
                Some(text) => out.filter = Some(text),
                None => return Err("--filter needs some text after it".to_owned()),
            },
            "--size" => match arguments.next().as_deref().map(read_size) {
                Some(Ok(size)) => out.size = Some(size),
                Some(Err(problem)) => return Err(problem),
                None => return Err("--size needs a width and a height after it, like 1440x2400".to_owned()),
            },
            "--scroll" => match arguments.next().map(|text| text.parse::<f32>()) {
                Some(Ok(points)) if points >= 0.0 => out.scroll = Some(points),
                _ => return Err("--scroll needs a number of points after it, like 1800".to_owned()),
            },
            "--help" | "-h" => {
                println!("{USAGE}");
                return Ok(None);
            }
            "--version" => {
                println!("rux-storybook {}", env!("CARGO_PKG_VERSION"));
                return Ok(None);
            }
            other => {
                return Err(format!("{USAGE}\n\nI do not know the switch {other:?}."));
            }
        }
    }
    if out.list {
        for story in stories::all() {
            println!("{:<14} {:<16} {}", story.section, story.name, story.rule);
        }
        return Ok(None);
    }
    Ok(Some(out))
}

/// Read `<width>x<height>` in points, such as `1440x2400`.
fn read_size(text: &str) -> Result<[f32; 2], String> {
    let refuse = || format!("--size takes a width and a height, like 1440x2400, and {text:?} is not one");
    let (width, height) = text.split_once(['x', 'X']).ok_or_else(refuse)?;
    let width: f32 = width.trim().parse().map_err(|_| refuse())?;
    let height: f32 = height.trim().parse().map_err(|_| refuse())?;
    if width < 760.0 || height < 560.0 {
        return Err(format!("--size {text} is smaller than the smallest window, 760x560"));
    }
    Ok([width, height])
}

const USAGE: &str = "\
rux-storybook — every rux component, every state, both themes.

    --theme <name>   start on a theme. One of: light-neumorphic, dark-neumorphic
    --story <name>   open straight onto a story, by name. `--list` prints them
    --open           open the dropdowns and the dialog, for photographing those states
    --filter <text>  start with the sidebar filtered
    --size <WxH>     open the window at this size in points, such as 1440x2400
    --scroll <n>     open the story scrolled down by this many points
    --measure        show what the library is holding, along the bottom of the sidebar
    --background     open without taking the foreground, for a script that drives it
    --list           print every story and exit
    --help           this
    --version        the version";

#[cfg(test)]
mod tests {
    use super::*;

    fn read(arguments: &[&str]) -> Result<Option<Arguments>, String> {
        read_arguments(arguments.iter().map(|argument| (*argument).to_owned()))
    }

    #[test]
    fn a_switch_the_program_does_not_know_is_refused_rather_than_read_as_a_value() {
        let problem = read(&["--colour"]).unwrap_err();
        assert!(problem.contains("--colour"));
        assert!(problem.contains("--theme"), "the refusal names what there is");
    }

    #[test]
    fn the_switches_are_read() {
        let arguments = read(&["--measure", "--theme", "light-neumorphic"]).unwrap().unwrap();
        assert!(arguments.measure);
        assert_eq!(arguments.theme.as_deref(), Some("light-neumorphic"));
        assert!(read(&["--theme"]).is_err(), "a theme with no name after it is a mistake");
    }

    #[test]
    fn a_size_is_read_as_a_width_and_a_height() {
        let arguments = read(&["--size", "1440x2400"]).unwrap().unwrap();
        assert_eq!(arguments.size, Some([1440.0, 2400.0]));
        assert!(read(&["--size", "wide"]).is_err());
        assert!(read(&["--size", "300x200"]).is_err(), "smaller than the smallest window");
        assert!(read(&["--size"]).is_err());
    }

    #[test]
    fn a_scroll_is_a_number_of_points() {
        let arguments = read(&["--scroll", "1800"]).unwrap().unwrap();
        assert_eq!(arguments.scroll, Some(1800.0));
        assert!(read(&["--scroll", "down"]).is_err());
        assert!(read(&["--scroll", "-5"]).is_err());
    }
}
