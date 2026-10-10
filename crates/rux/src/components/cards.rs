//! The Agent-Chat design's cards: a command that ran, a block of code, a change to apply, a request
//! for permission, and an error.
//!
//! `task-2235`. Every one is built the same way, read off the canvas's "Tool runs" and "Asking · code
//! · changes" panels: a card extruded from the pane with a 22 point radius and 12 points of padding; a
//! header row of a status well, a title and a detail line, and sometimes a small round button; a well
//! carved into the card holding monospaced lines; and a row of pills with the way out on the left and
//! the thing to do on the right.
//!
//! A card decides nothing. It answers which of its buttons was pressed, and the caller acts.

use egui::{Color32, Pos2, Rect, Response, Sense, Vec2};

use super::round::{card, PillButton, PillVariant, RoundButton, RoundSize};
use super::status::{Status, StatusWell};
use crate::icon::{Icon, Mark};
use crate::text::Style;
use crate::theme::Opacity as _;
use crate::Rux;

/// The card's corner radius: `border-radius: 22px`.
const RADIUS: f32 = 22.0;
/// The card's padding and the gap between its rows: `padding: 12px; gap: 12px`.
const PAD: f32 = 12.0;
/// The header row: as tall as its 34 point well.
const HEADER: f32 = 34.0;
/// A row of pills: `height: 36px`.
const ACTIONS: f32 = 36.0;

/// What a line in a card's well says, in pieces each with its own colour.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Line {
    /// The words, in runs. A run with no colour is drawn in the card's ordinary ink for that well.
    pub runs: Vec<(String, Option<Color32>)>,
    /// A line number, drawn in its own column.
    pub number: Option<String>,
    /// Whether the line was taken out or put in, for a diff.
    pub sign: Sign,
    /// Words at the right-hand end, quieter: how long a command took.
    pub trailing: Option<String>,
}

impl Line {
    /// A line of one run in the ordinary ink.
    pub fn plain(text: impl Into<String>) -> Self {
        Self { runs: vec![(text.into(), None)], ..Self::default() }
    }

    /// A line of one run in a colour of its own.
    pub fn coloured(text: impl Into<String>, colour: Color32) -> Self {
        Self { runs: vec![(text.into(), Some(colour))], ..Self::default() }
    }

    pub fn number(mut self, number: impl Into<String>) -> Self {
        self.number = Some(number.into());
        self
    }

    pub fn sign(mut self, sign: Sign) -> Self {
        self.sign = sign;
        self
    }

    pub fn trailing(mut self, trailing: impl Into<String>) -> Self {
        self.trailing = Some(trailing.into());
        self
    }

    /// The whole line's words, which is what a copy of it is.
    pub fn text(&self) -> String {
        self.runs.iter().map(|(text, _)| text.as_str()).collect()
    }
}

/// A diff line's sign.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Sign {
    #[default]
    Same,
    Removed,
    Added,
}

/// What a card's header leads with.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Lead {
    /// A status well.
    Status(Status),
    /// A well holding a mark in the quiet ink, as a code block and a diff hold a file.
    Mark(Icon),
}

/// The button at the right of a header.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Corner {
    None,
    /// The chevron that shows or hides the lines: pointing down while hidden, up while shown.
    Toggle { open: bool },
    /// Stop a command that is running.
    Stop,
    /// Copy what is in the card.
    Copy,
}

/// Which of a card's buttons was pressed this frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CardOutcome {
    /// The header's chevron.
    pub toggled: bool,
    /// The header's stop button.
    pub stopped: bool,
    /// The header's copy button.
    pub copied: bool,
    /// The pill on the left: `Copy`, `Deny`, `Reject`, `Show full log`.
    pub ghost: bool,
    /// The middle pill: `Open in editor`, `Always allow`, `Retry`.
    pub secondary: bool,
    /// The pill on the right: `Apply change`, `Allow once`.
    pub primary: bool,
}

/// How the well's lines are set.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Setting {
    size: f32,
    leading: f32,
    padding_y: f32,
    padding_x: f32,
    number_column: f32,
    sign_column: f32,
}

impl Setting {
    /// A command's output and the commands of a run: `font-size: 11.5px; line-height: 20px; padding:
    /// 12px 14px`.
    const OUTPUT: Self =
        Self { size: 11.5, leading: 20.0, padding_y: 12.0, padding_x: 14.0, number_column: 0.0, sign_column: 0.0 };
    /// A code block: `font-size: 11px; line-height: 20px; padding: 12px 0`, a 30 point number column.
    const CODE: Self =
        Self { size: 11.0, leading: 20.0, padding_y: 12.0, padding_x: 0.0, number_column: 30.0, sign_column: 0.0 };
    /// A diff: `font-size: 11.5px; line-height: 21px; padding: 10px 0`, 34 and 14 point columns.
    const DIFF: Self =
        Self { size: 11.5, leading: 21.0, padding_y: 10.0, padding_x: 0.0, number_column: 34.0, sign_column: 14.0 };
    /// An error's log and a command waiting for permission: `line-height: 1.65`, about 19 points.
    const LOG: Self =
        Self { size: 11.5, leading: 19.0, padding_y: 12.0, padding_x: 14.0, number_column: 0.0, sign_column: 0.0 };
}

/// The three pills a card can end with.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
struct Actions<'a> {
    ghost: Option<&'a str>,
    secondary: Option<(&'a str, Option<Icon>)>,
    primary: Option<&'a str>,
}

impl Actions<'_> {
    fn any(&self) -> bool {
        self.ghost.is_some() || self.secondary.is_some() || self.primary.is_some()
    }
}

/// Everything a card is made of, which each kind of card fills in.
struct Body<'a> {
    lead: Lead,
    title: &'a str,
    title_mono: bool,
    detail: &'a str,
    detail_colour: Option<Color32>,
    corner: Corner,
    lines: Option<(&'a [Line], Setting)>,
    progress: Option<f32>,
    actions: Actions<'a>,
    label: &'a str,
}

impl Body<'_> {
    fn height(&self, rux: &Rux<'_>) -> f32 {
        let mut rows = vec![rux.z(HEADER)];
        if self.progress.is_some() {
            rows.push(rux.z(22.0));
        }
        if let Some((lines, setting)) = self.lines.filter(|(lines, _)| !lines.is_empty()) {
            rows.push(rux.z(setting.padding_y * 2.0 + setting.leading * lines.len() as f32));
        }
        if self.actions.any() {
            rows.push(rux.z(ACTIONS));
        }
        let gaps = rux.z(PAD) * (rows.len() as f32 - 1.0);
        rows.iter().sum::<f32>() + gaps + rux.z(PAD) * 2.0
    }

    fn show(self, rux: &mut Rux<'_>, rect: Rect) -> (Response, CardOutcome) {
        let response = super::interact(rux, rect, "rux-card", Sense::hover());
        let chat = rux.theme().chat;
        rux.chrome.surface(rect, rux.z(RADIUS), card(&chat, rect), chat.raised());
        let inner = rect.shrink(rux.z(PAD));
        let mut outcome = CardOutcome::default();
        let header = Rect::from_min_size(inner.min, Vec2::new(inner.width(), rux.z(HEADER)));
        self.header(rux, header, &mut outcome);
        let mut top = header.bottom() + rux.z(PAD);
        if let Some(fraction) = self.progress {
            // `margin: 0 6px` round a 12 point track.
            let track = Rect::from_min_size(
                Pos2::new(inner.left() + rux.z(6.0), top),
                Vec2::new(inner.width() - rux.z(12.0), rux.z(22.0)),
            );
            super::dial::Track::new(fraction, 1.0).label(self.title).show(rux, track);
            top = track.bottom() + rux.z(PAD);
        }
        if let Some((lines, setting)) = self.lines.filter(|(lines, _)| !lines.is_empty()) {
            let height = rux.z(setting.padding_y * 2.0 + setting.leading * lines.len() as f32);
            let well = Rect::from_min_size(Pos2::new(inner.left(), top), Vec2::new(inner.width(), height));
            paint_lines(rux, well, lines, setting);
            top = well.bottom() + rux.z(PAD);
        }
        if self.actions.any() {
            let row = Rect::from_min_size(Pos2::new(inner.left(), top), Vec2::new(inner.width(), rux.z(ACTIONS)));
            actions(rux, row, self.actions, &mut outcome);
        }
        let label = self.label.to_owned();
        response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Other, true, label.clone()));
        (response, outcome)
    }

    fn header(&self, rux: &mut Rux<'_>, row: Rect, outcome: &mut CardOutcome) {
        let theme = rux.theme();
        let side = rux.z(HEADER);
        let well = Rect::from_min_size(row.min, Vec2::splat(side));
        match self.lead {
            Lead::Status(status) => {
                StatusWell::new(status).show(rux, well);
            }
            Lead::Mark(icon) => {
                let chat = theme.chat;
                rux.chrome.surface(well, side / 2.0, chat.well, chat.carved_sm());
                rux.mark(Mark::new(icon, rux.z(18.0)), well.center(), theme.ink.i500);
            }
        }
        let button = rux.z(30.0);
        let right = match self.corner {
            Corner::None => row.right(),
            _ => row.right() - button - rux.z(12.0),
        };
        let left = well.right() + rux.z(12.0);
        let room = (right - left).max(0.0);
        let title_style = match self.title_mono {
            true => rux.zs(Style::mono(12.0)),
            false => rux.zs(Style::sans(12.5)),
        };
        let detail_style = rux.zs(Style::sans(11.0));
        let title = rux.elided(title_style, self.title, theme.ink.i900, room);
        let detail_colour = self.detail_colour.unwrap_or(theme.ink.i400);
        let detail = rux.elided(detail_style, self.detail, detail_colour, room);
        let gap = rux.z(1.0);
        let block = title.size().y + gap + detail.size().y;
        let top = row.center().y - block / 2.0;
        let painter = rux.painter().clone();
        painter.galley(Pos2::new(left, top), title.clone(), theme.ink.i900);
        painter.galley(Pos2::new(left, top + title.size().y + gap), detail, detail_colour);
        let corner = Rect::from_center_size(Pos2::new(row.right() - button / 2.0, row.center().y), Vec2::splat(button));
        match self.corner {
            Corner::None => {}
            Corner::Toggle { open } => {
                let (label, turn) = match open {
                    true => ("Hide", std::f32::consts::PI),
                    false => ("Show", 0.0),
                };
                let mark = Mark::new(Icon::ChevDown, 12.0).stroke(2.0).turn(turn);
                let pressed = RoundButton::new(super::round::RoundContent::Mark(mark), label)
                    .size(RoundSize::Small)
                    .show(rux, corner)
                    .clicked();
                outcome.toggled |= pressed;
            }
            Corner::Stop => {
                let pressed = RoundButton::stop("Stop this command").size(RoundSize::Small).show(rux, corner).clicked();
                outcome.stopped |= pressed;
            }
            Corner::Copy => {
                let pressed = RoundButton::mark(Icon::Copy, 13.0, "Copy code")
                    .size(RoundSize::Small)
                    .show(rux, corner)
                    .clicked();
                outcome.copied |= pressed;
            }
        }
    }
}

/// The well and its lines.
fn paint_lines(rux: &mut Rux<'_>, well: Rect, lines: &[Line], setting: Setting) {
    let theme = rux.theme();
    let chat = theme.chat;
    rux.chrome.surface(well, rux.z(16.0), chat.well, chat.carved_sm());
    let style = rux.zs(Style::mono(setting.size));
    let painter = rux.painter().clone();
    let clip = painter.with_clip_rect(well.shrink(1.0).intersect(painter.clip_rect()));
    let leading = rux.z(setting.leading);
    let mut top = well.top() + rux.z(setting.padding_y);
    for line in lines {
        let row = Rect::from_min_size(Pos2::new(well.left(), top), Vec2::new(well.width(), leading));
        let wash = match line.sign {
            Sign::Same => None,
            Sign::Removed => Some(chat.slower.at(0.08)),
            Sign::Added => Some(chat.faster.at(0.08)),
        };
        if let Some(wash) = wash {
            clip.rect_filled(row, 0.0, wash);
        }
        let mut x = well.left() + rux.z(setting.padding_x);
        let middle = row.center().y;
        if setting.number_column > 0.0 {
            let column = rux.z(setting.number_column);
            if let Some(number) = &line.number {
                let galley = rux.text(style, number, theme.ink.i300);
                let at = Pos2::new(x + column - rux.z(10.0) - galley.size().x, middle - galley.size().y / 2.0);
                clip.galley(at, galley, theme.ink.i300);
            }
            x += column;
        }
        if setting.sign_column > 0.0 {
            let (sign, colour) = match line.sign {
                Sign::Same => ("", theme.ink.i400),
                Sign::Removed => ("\u{2212}", chat.slower),
                Sign::Added => ("+", chat.faster),
            };
            if !sign.is_empty() {
                let galley = rux.text(style, sign, colour);
                clip.galley(Pos2::new(x, middle - galley.size().y / 2.0), galley, colour);
            }
            x += rux.z(setting.sign_column);
        }
        let mut right = well.right() - rux.z(setting.padding_x.max(12.0));
        if let Some(trailing) = &line.trailing {
            let galley = rux.text(rux.zs(Style::sans(11.0)), trailing, theme.ink.i400);
            right -= galley.size().x;
            clip.galley(Pos2::new(right, middle - galley.size().y / 2.0), galley, theme.ink.i400);
            right -= rux.z(10.0);
        }
        let default = match setting.number_column > 0.0 && setting.sign_column == 0.0 {
            true => theme.ink.i700,
            false => theme.ink.i500,
        };
        // Laid out as one job with a section a run, so the runs share a baseline and one ellipsis.
        let mut job = egui::text::LayoutJob::default();
        for (words, colour) in &line.runs {
            job.append(words, 0.0, style.format(colour.unwrap_or(default)));
        }
        job.wrap = egui::text::TextWrapping {
            max_width: (right - x).max(0.0),
            max_rows: 1,
            break_anywhere: true,
            overflow_character: Some('\u{2026}'),
        };
        let galley = painter.layout_job(job);
        clip.galley(Pos2::new(x, middle - galley.size().y / 2.0), galley, default);
        top += leading;
    }
}

/// The row of pills: the ghost on the left, then a space, then the secondary and the primary.
fn actions(rux: &mut Rux<'_>, row: Rect, actions: Actions<'_>, outcome: &mut CardOutcome) {
    let gap = rux.z(8.0);
    if let Some(text) = actions.ghost {
        let pill = PillButton::new(text).variant(PillVariant::Ghost);
        let width = pill.measure(rux).x;
        let rect = Rect::from_min_size(row.min, Vec2::new(width, row.height()));
        outcome.ghost |= pill.show(rux, rect).clicked();
    }
    let mut right = row.right();
    if let Some(text) = actions.primary {
        let pill = PillButton::new(text).variant(PillVariant::Primary);
        let width = pill.measure(rux).x;
        let rect = Rect::from_min_size(Pos2::new(right - width, row.top()), Vec2::new(width, row.height()));
        outcome.primary |= pill.show(rux, rect).clicked();
        right = rect.left() - gap;
    }
    if let Some((text, icon)) = actions.secondary {
        let mut pill = PillButton::new(text).variant(PillVariant::Secondary);
        if let Some(icon) = icon {
            pill = pill.mark(icon);
        }
        let width = pill.measure(rux).x;
        let rect = Rect::from_min_size(Pos2::new(right - width, row.top()), Vec2::new(width, row.height()));
        outcome.secondary |= pill.show(rux, rect).clicked();
    }
}

/// A command the agent ran, or a run of them: collapsed, shown, running with its progress, or failed.
pub struct CommandCard<'a> {
    title: &'a str,
    mono: bool,
    detail: &'a str,
    status: Status,
    open: bool,
    lines: &'a [Line],
    progress: Option<f32>,
    stoppable: bool,
}

impl<'a> CommandCard<'a> {
    /// `title` is the command, set in the code font; `detail` is how long it took or what it is doing.
    pub fn new(title: &'a str, detail: &'a str, status: Status) -> Self {
        Self { title, mono: true, detail, status, open: false, lines: &[], progress: None, stoppable: false }
    }

    /// A title in the ordinary font, for a run: `Ran 3 commands`.
    pub fn words(mut self) -> Self {
        self.mono = false;
        self
    }

    /// Shown, with these lines in its well; hidden lines are not drawn and the chevron points down.
    pub fn lines(mut self, lines: &'a [Line], open: bool) -> Self {
        self.lines = lines;
        self.open = open;
        self
    }

    /// Running, this far through: a track under the header and a stop button in place of the chevron.
    pub fn progress(mut self, fraction: Option<f32>) -> Self {
        self.progress = fraction;
        self.stoppable = true;
        self
    }

    fn body(&self) -> Body<'a> {
        let corner = match (self.stoppable, self.lines.is_empty()) {
            (true, _) => Corner::Stop,
            (false, false) => Corner::Toggle { open: self.open },
            (false, true) => Corner::None,
        };
        Body {
            lead: Lead::Status(self.status),
            title: self.title,
            title_mono: self.mono,
            detail: self.detail,
            detail_colour: None,
            corner,
            lines: self.open.then_some((self.lines, Setting::OUTPUT)),
            progress: self.progress.filter(|_| self.status == Status::Running),
            actions: Actions::default(),
            label: self.title,
        }
    }

    pub fn height(&self, rux: &Rux<'_>) -> f32 {
        self.body().height(rux)
    }

    /// The detail line in a colour of its own: `Exited with 101` in the slower colour.
    pub fn show_with_detail(self, rux: &mut Rux<'_>, rect: Rect, colour: Color32) -> (Response, CardOutcome) {
        let mut body = self.body();
        body.detail_colour = Some(colour);
        body.show(rux, rect)
    }

    pub fn show(self, rux: &mut Rux<'_>, rect: Rect) -> (Response, CardOutcome) {
        let mut body = self.body();
        if self.status == Status::Failed {
            body.detail_colour = Some(rux.theme().chat.slower);
        }
        body.show(rux, rect)
    }
}

/// A block of code with a file's name, its lines numbered, and a copy button.
pub struct CodeBlock<'a> {
    name: &'a str,
    detail: &'a str,
    lines: &'a [Line],
}

impl<'a> CodeBlock<'a> {
    pub fn new(name: &'a str, detail: &'a str, lines: &'a [Line]) -> Self {
        Self { name, detail, lines }
    }

    fn body(&self) -> Body<'a> {
        Body {
            lead: Lead::Mark(Icon::File),
            title: self.name,
            title_mono: false,
            detail: self.detail,
            detail_colour: None,
            corner: Corner::Copy,
            lines: Some((self.lines, Setting::CODE)),
            progress: None,
            actions: Actions::default(),
            label: self.name,
        }
    }

    pub fn height(&self, rux: &Rux<'_>) -> f32 {
        self.body().height(rux)
    }

    pub fn show(self, rux: &mut Rux<'_>, rect: Rect) -> (Response, CardOutcome) {
        self.body().show(rux, rect)
    }
}

/// A change to one file, line by line, with Reject, Open in editor and Apply change.
pub struct DiffCard<'a> {
    name: &'a str,
    detail: &'a str,
    lines: &'a [Line],
    ghost: Option<&'a str>,
    secondary: Option<&'a str>,
    primary: Option<&'a str>,
}

impl<'a> DiffCard<'a> {
    pub fn new(name: &'a str, detail: &'a str, lines: &'a [Line]) -> Self {
        Self {
            name,
            detail,
            lines,
            ghost: Some("Reject"),
            secondary: Some("Open in editor"),
            primary: Some("Apply change"),
        }
    }

    /// Other words on the three pills, or `None` for a pill that is not offered.
    pub fn buttons(mut self, ghost: Option<&'a str>, secondary: Option<&'a str>, primary: Option<&'a str>) -> Self {
        self.ghost = ghost;
        self.secondary = secondary;
        self.primary = primary;
        self
    }

    fn body(&self) -> Body<'a> {
        Body {
            lead: Lead::Mark(Icon::File),
            title: self.name,
            title_mono: false,
            detail: self.detail,
            detail_colour: None,
            corner: Corner::None,
            lines: Some((self.lines, Setting::DIFF)),
            progress: None,
            actions: Actions {
                ghost: self.ghost,
                secondary: self.secondary.map(|text| (text, None)),
                primary: self.primary,
            },
            label: self.name,
        }
    }

    pub fn height(&self, rux: &Rux<'_>) -> f32 {
        self.body().height(rux)
    }

    pub fn show(self, rux: &mut Rux<'_>, rect: Rect) -> (Response, CardOutcome) {
        self.body().show(rux, rect)
    }
}

/// The agent asking before it does something: what, why, and Deny, Always allow, Allow once.
pub struct PermissionCard<'a> {
    title: &'a str,
    detail: &'a str,
    command: &'a [Line],
}

impl<'a> PermissionCard<'a> {
    pub fn new(title: &'a str, detail: &'a str, command: &'a [Line]) -> Self {
        Self { title, detail, command }
    }

    fn body(&self) -> Body<'a> {
        Body {
            lead: Lead::Status(Status::NeedsOk),
            title: self.title,
            title_mono: false,
            detail: self.detail,
            detail_colour: None,
            corner: Corner::None,
            lines: Some((self.command, Setting::LOG)),
            progress: None,
            actions: Actions {
                ghost: Some("Deny"),
                secondary: Some(("Always allow", None)),
                primary: Some("Allow once"),
            },
            label: self.title,
        }
    }

    pub fn height(&self, rux: &Rux<'_>) -> f32 {
        self.body().height(rux)
    }

    pub fn show(self, rux: &mut Rux<'_>, rect: Rect) -> (Response, CardOutcome) {
        self.body().show(rux, rect)
    }
}

/// Something that failed: what, the first lines of its log, and Show full log and Retry.
pub struct ErrorCard<'a> {
    title: &'a str,
    detail: &'a str,
    log: &'a [Line],
    retry: bool,
}

impl<'a> ErrorCard<'a> {
    pub fn new(title: &'a str, detail: &'a str, log: &'a [Line]) -> Self {
        Self { title, detail, log, retry: true }
    }

    /// No Retry, for a failure nothing can try again.
    pub fn without_retry(mut self) -> Self {
        self.retry = false;
        self
    }

    fn body(&self) -> Body<'a> {
        Body {
            lead: Lead::Status(Status::Failed),
            title: self.title,
            title_mono: false,
            detail: self.detail,
            detail_colour: None,
            corner: Corner::None,
            lines: Some((self.log, Setting::LOG)),
            progress: None,
            actions: Actions {
                ghost: Some("Show full log"),
                secondary: self.retry.then_some(("Retry", Some(Icon::Rerun))),
                primary: None,
            },
            label: self.title,
        }
    }

    pub fn height(&self, rux: &Rux<'_>) -> f32 {
        self.body().height(rux)
    }

    pub fn show(self, rux: &mut Rux<'_>, rect: Rect) -> (Response, CardOutcome) {
        self.body().show(rux, rect)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_line_keeps_its_runs_in_order() {
        let line = Line {
            runs: vec![("kurbo = ".into(), None), ("\"0.11\"".into(), Some(Color32::WHITE))],
            ..Line::default()
        }
        .number("14")
        .sign(Sign::Added);
        assert_eq!(line.text(), "kurbo = \"0.11\"");
        assert_eq!(line.number.as_deref(), Some("14"));
        assert_eq!(line.sign, Sign::Added);
    }

    #[test]
    fn the_settings_are_the_canvas_s() {
        assert_eq!((Setting::CODE.size, Setting::CODE.leading, Setting::CODE.number_column), (11.0, 20.0, 30.0));
        assert_eq!((Setting::DIFF.leading, Setting::DIFF.number_column, Setting::DIFF.sign_column), (21.0, 34.0, 14.0));
        assert_eq!((Setting::OUTPUT.padding_y, Setting::OUTPUT.padding_x), (12.0, 14.0));
    }
}
