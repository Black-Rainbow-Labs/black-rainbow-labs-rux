//! The soft keyboard's text, turned into egui events.
//!
//! GameActivity keeps the editor state the keyboard is typing into (the text, the selection and the word
//! being composed) on the native side. winit does not pass its changes on, so this reads the whole text
//! once a frame and compares it with the text it read last time. Whatever the two share at the start is
//! unchanged; the rest of the old text was deleted and the rest of the new text was typed. That covers
//! every way a keyboard edits: a letter, a backspace, autocorrect replacing a word, a suggestion picked,
//! and a word still being composed.
//!
//! The text is read from `RuxBridge`, which `RuxActivity.stateChanged` keeps up to date, rather than from
//! `GameActivity_getTextInputState`: before anything has been typed that call gives `android-activity`
//! 0.6.1 a null text pointer, which it turns into a slice (a panic in a debug build). The keyboard's action
//! key arrives through `RuxActivity.onEditorAction` the same way, because winit drops editor actions, and
//! becomes an Enter key press. A newline typed into the text becomes one too.

/// What one change of the keyboard's text means to egui.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Edit {
    /// Delete one character before the caret.
    Backspace,
    /// Type this text at the caret.
    Text(String),
    /// The keyboard's action key.
    Enter,
}

/// Works out the edits that turn `before` into `after`.
/// @param before - the keyboard's text last frame
/// @param after - the keyboard's text now
pub fn diff(before: &str, after: &str) -> Vec<Edit> {
    let shared = before.chars().zip(after.chars()).take_while(|(a, b)| a == b).count();
    let removed = before.chars().count() - shared;
    let mut edits: Vec<Edit> = (0..removed).map(|_| Edit::Backspace).collect();
    let typed: String = after.chars().skip(shared).collect();
    for (index, line) in typed.split('\n').enumerate() {
        if index > 0 {
            edits.push(Edit::Enter);
        }
        if !line.is_empty() {
            edits.push(Edit::Text(line.to_owned()));
        }
    }
    edits
}

/// The egui events for one edit.
/// @param edit - the edit
pub fn events(edit: &Edit) -> Vec<egui::Event> {
    match edit {
        Edit::Text(text) => vec![egui::Event::Text(text.clone())],
        Edit::Backspace => press(egui::Key::Backspace),
        Edit::Enter => press(egui::Key::Enter),
    }
}

/// A key going down and coming up.
/// @param key - the key
fn press(key: egui::Key) -> Vec<egui::Event> {
    [true, false]
        .into_iter()
        .map(|pressed| egui::Event::Key { key, physical_key: None, pressed, repeat: false, modifiers: egui::Modifiers::NONE })
        .collect()
}

/// The keyboard text read last frame, and whether a text field had focus.
#[derive(Default)]
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
pub struct Keyboard {
    last: String,
    focused: bool,
}

#[cfg(target_os = "android")]
impl Keyboard {
    /// Adds the keyboard's changes since last frame to egui's input.
    ///
    /// When a text field gains focus the keyboard's editor is emptied, so it never holds text from a
    /// field that is no longer there. After the action key it is emptied again, so a long session does not
    /// grow one buffer forever.
    /// @param app - the Android application, which owns the keyboard's settings
    /// @param wants_text - whether egui has a text field focused
    /// @param raw - the input egui is about to run the frame with
    pub fn feed(&mut self, app: &winit::platform::android::activity::AndroidApp, wants_text: bool, raw: &mut egui::RawInput) {
        use winit::platform::android::activity::input::{ImeOptions, InputType, TextInputAction};
        if wants_text && !self.focused {
            let kind = InputType::TYPE_CLASS_TEXT | InputType::TYPE_TEXT_FLAG_CAP_SENTENCES | InputType::TYPE_TEXT_FLAG_AUTO_CORRECT;
            app.set_ime_editor_info(kind, TextInputAction::Done, ImeOptions::IME_FLAG_NO_FULLSCREEN);
            crate::bridge::clear_keyboard();
            self.last.clear();
        }
        self.focused = wants_text;
        let actions = crate::bridge::take_editor_actions();
        if !wants_text {
            return;
        }
        let now = crate::bridge::keyboard_text().unwrap_or_else(|| self.last.clone());
        for edit in diff(&self.last, &now) {
            raw.events.extend(events(&edit));
        }
        self.last = now;
        for _ in 0..actions {
            raw.events.extend(events(&Edit::Enter));
        }
        if actions > 0 {
            crate::bridge::clear_keyboard();
            self.last.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_letter_is_typed() {
        assert_eq!(diff("hel", "hell"), vec![Edit::Text("l".into())]);
    }

    #[test]
    fn a_backspace_deletes() {
        assert_eq!(diff("hello", "hell"), vec![Edit::Backspace]);
    }

    #[test]
    fn autocorrect_replaces_the_word() {
        assert_eq!(
            diff("I teh", "I the "),
            vec![Edit::Backspace, Edit::Backspace, Edit::Text("he ".into())]
        );
    }

    #[test]
    fn the_action_key_is_enter() {
        assert_eq!(diff("hi", "hi\n"), vec![Edit::Enter]);
        assert_eq!(diff("", "a\nb"), vec![Edit::Text("a".into()), Edit::Enter, Edit::Text("b".into())]);
    }

    #[test]
    fn characters_are_counted_not_bytes() {
        assert_eq!(diff("café", "caf"), vec![Edit::Backspace]);
        assert_eq!(diff("caf", "café"), vec![Edit::Text("é".into())]);
    }
}
