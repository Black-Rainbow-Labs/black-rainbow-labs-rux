//! Somewhere to type: a one-line field and a well of running text.
//!
//! `.ic-form-input`, `.search-field`, `.scene__title-input`, `.ic-prompt` and `.ic-form-textarea`.
//!
//! **The caret, the selection, the clipboard and the input method are `egui`'s**, which is the whole
//! reason `egui` is under this library rather than only `vello_cpu`. What is written here is the well
//! the box sits in, the face the words are set in, and one detail that matters more than it sounds:
//!
//! **the box is given the field's whole rectangle, with the padding as its margin.** A text box laid
//! out as a one-line strip inside a taller control leaves its own padding unclaimed, so a press there
//! hits nothing, no box takes the keyboard, and the keystroke lands in whatever is behind — which is
//! a real fault Unluminous had to fix twice. With the margin inside the box, every point of the
//! control is the box.

use egui::{Color32, Rect, Vec2};

use crate::icon::{Icon, Mark};
use crate::layout::{Cross, Flex, Pad};
use crate::text::Style;
use crate::theme::Theme;
use crate::Rux;

/// What happened to a one-line field this frame.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct TextInputOutcome {
    pub changed: bool,
    pub focused: bool,
    /// Enter was pressed while the field had the keyboard.
    pub submitted: bool,
}

/// A one-line field.
pub struct TextInput<'a> {
    text: &'a mut String,
    hint: &'a str,
    /// A mark before the words, which is what makes `.search-field` a search field.
    icon: Option<Icon>,
    /// `border-radius` — `var(--r-md)` for a form field, `var(--r-pill)` for a search one.
    pill: bool,
    /// `font-family: var(--font-mono)` for a field that holds a number or a path.
    style: Style,
    pad: Pad,
    enabled: bool,
    label: &'a str,
}

impl<'a> TextInput<'a> {
    /// `.ic-form-input` — `padding: 12px 16px; border-radius: var(--r-md)`, 13px.
    pub fn new(text: &'a mut String) -> Self {
        Self {
            text,
            hint: "",
            icon: None,
            pill: false,
            style: Style::sans(13.0),
            pad: Pad::axes(12.0, 16.0),
            enabled: true,
            label: "Text",
        }
    }

    /// `.search-field` — a pill with a magnifier in it, `padding: 0 18px`, 13px.
    pub fn search(text: &'a mut String) -> Self {
        Self {
            icon: Some(Icon::Search),
            pill: true,
            pad: Pad::axes(10.0, 18.0),
            ..Self::new(text)
        }
    }

    pub fn hint(mut self, hint: &'a str) -> Self {
        self.hint = hint;
        self
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn pad(mut self, pad: Pad) -> Self {
        self.pad = pad;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = label;
        self
    }

    /// `padding` plus one line of the face it is set in.
    pub fn measure(&self, rux: &Rux<'_>) -> Vec2 {
        let line = rux.measure(self.style, "Ag").y;
        let mark = f32::from(self.icon.is_some()) * (13.0 + 10.0);
        Vec2::new(240.0 + mark, rux.screen().target_height(line + self.pad.vertical()))
    }

    pub fn show(self, rux: &mut Rux<'_>, rect: Rect) -> TextInputOutcome {
        let theme = rux.theme();
        let radius = match self.pill {
            true => theme.radius.pill,
            false => theme.radius.md,
        };
        let focused = rux.ui.memory(|memory| memory.has_focus(self.id(rux, rect)));
        // `.ic-form-input:focus { box-shadow: var(--e-pressed) }` — a focused field is pressed further
        // into the surface, which is the reference's whole focus treatment.
        let elevation = match focused {
            true => theme.elevation.pressed,
            false => theme.elevation.pressed_sm,
        };
        rux.chrome.surface(rect, radius, theme.surface.s1, elevation);

        let inner = self.pad.inside(rect);
        let (mark_rect, box_rect) = match self.icon {
            Some(_) => {
                let [mark, body] = Flex::row(inner)
                    .gap(10.0)
                    .cross(Cross::Centre)
                    .sized(Vec2::splat(13.0))
                    .fill(1.0)
                    .solve();
                (Some(mark), body)
            }
            None => (None, inner),
        };
        if let (Some(mark), Some(icon)) = (mark_rect, self.icon) {
            let ink = theme.ink.i500;
            rux.mark_in(Mark::new(icon, 13.0), mark, ink);
        }

        self.draw_box(rux, theme, rect, box_rect, focused)
    }

    /// The id the text box is given.
    ///
    /// Derived from where the control is rather than from `egui`'s auto-counter, which shifts the
    /// moment the number of widgets above it changes — and a shifting id silently moves the keyboard
    /// to a different field.
    fn id(&self, rux: &Rux<'_>, rect: Rect) -> egui::Id {
        rux.ui.id().with(("rux-text-input", rect.min.x.to_bits(), rect.min.y.to_bits()))
    }

    fn draw_box(
        self,
        rux: &mut Rux<'_>,
        theme: &'static Theme,
        whole: Rect,
        body: Rect,
        focused: bool,
    ) -> TextInputOutcome {
        let id = self.id(rux, whole);
        let line = rux.measure(self.style, "Ag").y;
        // The box gets the body's whole height, with the slack as its own margin, so a press anywhere
        // in the control is a press on the box.
        let slack = ((body.height() - line) / 2.0).max(0.0);
        let margin = egui::Margin {
            left: 0,
            right: 0,
            top: slack.round().clamp(0.0, 127.0) as i8,
            bottom: slack.round().clamp(0.0, 127.0) as i8,
        };
        style_the_box(rux, theme);
        let hint = self.hint;
        let response = rux
            .ui
            .put(
                body,
                egui::TextEdit::singleline(self.text)
                    .id(id)
                    .frame(egui::Frame::NONE)
                    .margin(margin)
                    .desired_width(body.width())
                    .font(self.style.font())
                    .text_color(theme.ink.i900)
                    .hint_text(hint)
                    .interactive(self.enabled),
            );
        response.widget_info(|| {
            egui::WidgetInfo::text_edit(self.enabled, "", self.text.as_str(), self.label)
        });
        TextInputOutcome {
            changed: response.changed(),
            focused,
            submitted: response.lost_focus()
                && rux.ui.ctx().input(|input| input.key_pressed(egui::Key::Enter)),
        }
    }
}

/// What happened to a well of running text this frame.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct TextAreaOutcome {
    pub changed: bool,
    pub focused: bool,
    /// The corner button was pressed, if there is one. `.ic-prompt-shuffle`.
    pub corner_pressed: bool,
}

/// A well of running text.
///
/// `.ic-prompt-wrap` and `.ic-prompt`: a pressed well with `padding: 14px 40px 28px 14px` — the right
/// padding leaves room for the shuffle button and the bottom padding for the character count, both of
/// which are painted over the well rather than laid out in the flow.
pub struct TextArea<'a> {
    text: &'a mut String,
    hint: &'a str,
    /// `.ic-prompt-count` — the number of characters, at the bottom right.
    count: bool,
    /// `.ic-prompt-shuffle` — a small round button in the top right corner.
    corner: Option<Icon>,
    style: Style,
    enabled: bool,
    label: &'a str,
}

impl<'a> TextArea<'a> {
    pub fn new(text: &'a mut String) -> Self {
        Self {
            text,
            hint: "",
            count: false,
            corner: None,
            style: Style::PROSE,
            enabled: true,
            label: "Text",
        }
    }

    pub fn hint(mut self, hint: &'a str) -> Self {
        self.hint = hint;
        self
    }

    /// Show the character count at the bottom right.
    pub fn count(mut self, count: bool) -> Self {
        self.count = count;
        self
    }

    /// A small round button in the top right corner of the well.
    pub fn corner(mut self, icon: Icon) -> Self {
        self.corner = Some(icon);
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = label;
        self
    }

    /// `.ic-prompt { min-height: 280px }`.
    pub fn measure(&self, _rux: &Rux<'_>) -> Vec2 {
        Vec2::new(260.0, 280.0)
    }

    pub fn show(self, rux: &mut Rux<'_>, rect: Rect) -> TextAreaOutcome {
        let theme = rux.theme();
        rux.chrome.surface(rect, theme.radius.md, theme.surface.s1, theme.elevation.pressed);

        // `padding: 14px 40px 28px 14px` when there is a corner button and a count, and the plain
        // `14px 16px` of `.ic-form-textarea` when there is neither.
        let pad = match (self.corner.is_some(), self.count) {
            (false, false) => Pad::axes(14.0, 16.0),
            _ => Pad::sides(14.0, 40.0, 28.0, 14.0),
        };
        let body = pad.inside(rect);
        let id = rux.ui.id().with(("rux-text-area", rect.min.x.to_bits(), rect.min.y.to_bits()));
        let focused = rux.ui.memory(|memory| memory.has_focus(id));

        style_the_box(rux, theme);
        let hint = self.hint;
        let response = rux.ui.put(
            body,
            egui::TextEdit::multiline(self.text)
                .id(id)
                .frame(egui::Frame::NONE)
                .margin(egui::Margin::ZERO)
                .desired_width(body.width())
                .desired_rows(1)
                .font(self.style.font())
                .text_color(theme.ink.i900)
                .hint_text(hint)
                .interactive(self.enabled),
        );
        response.widget_info(|| {
            egui::WidgetInfo::text_edit(self.enabled, "", self.text.as_str(), self.label)
        });

        let mut outcome = TextAreaOutcome {
            changed: response.changed(),
            focused,
            corner_pressed: false,
        };

        if let Some(icon) = self.corner {
            // `.ic-prompt-shuffle { top: 8px; right: 8px; width: 28px }`.
            let button = Rect::from_min_size(
                egui::pos2(rect.right() - 8.0 - 28.0, rect.top() + 8.0),
                Vec2::splat(28.0),
            );
            outcome.corner_pressed = super::IconButton::new(icon, "Regenerate")
                .size(super::IconButtonSize::Small)
                .show(rux, button)
                .clicked();
        }
        if self.count {
            // `.ic-prompt-count { bottom: 8px; right: 12px }`, mono 10px, ink-400.
            let text = self.text.chars().count().to_string();
            let style = Style::mono(10.0);
            let size = rux.measure(style, &text);
            let galley = rux.text(style, &text, theme.ink.i400);
            rux.ui.painter().galley(
                egui::pos2(rect.right() - 12.0 - size.x, rect.bottom() - 8.0 - size.y),
                galley,
                theme.ink.i400,
            );
        }
        outcome
    }
}

/// Make `egui`'s own text box look like the reference's.
///
/// The selection, the caret and the hint are `egui`'s to draw, and its defaults are its own blue and
/// its own grey. Set on the `Ui` rather than on the widget because `TextEdit` reads them from the
/// style, and put back by `egui` itself when the `Ui` ends.
fn style_the_box(rux: &mut Rux<'_>, theme: &'static Theme) {
    let visuals = &mut rux.ui.style_mut().visuals;
    visuals.selection.bg_fill = crate::theme::with_opacity(theme.accent.blue, 0.25);
    visuals.selection.stroke = egui::Stroke::new(1.0, theme.ink.i900);
    visuals.text_cursor.stroke = egui::Stroke::new(1.5, theme.accent.blue);
    // `::placeholder { color: var(--ink-400) }`.
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, theme.ink.i400);
    visuals.extreme_bg_color = Color32::TRANSPARENT;
}
