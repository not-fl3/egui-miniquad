use miniquad::KeyCode;

#[cfg(feature = "macroquad")]
mod ui_plugin;
#[cfg(feature = "macroquad")]
pub use ui_plugin::UiPlugin;

mod painter;
pub use painter::Painter;

/// All of the keys Egui is interested in receiving, as miniquad KeyCodes.
pub const EGUI_KEYS: &[KeyCode] = &[
    KeyCode::LeftAlt,
    KeyCode::RightAlt,
    KeyCode::Backspace,
    KeyCode::LeftControl,
    KeyCode::RightControl,
    KeyCode::Delete,
    KeyCode::Down,
    KeyCode::End,
    KeyCode::Escape,
    KeyCode::Home,
    KeyCode::Insert,
    KeyCode::Left,
    KeyCode::LeftSuper,
    KeyCode::RightSuper,
    KeyCode::PageDown,
    KeyCode::PageUp,
    KeyCode::Enter,
    KeyCode::Right,
    KeyCode::LeftShift,
    KeyCode::RightShift,
    KeyCode::Tab,
    KeyCode::Up,
];
pub fn convert_keycode(keycode: KeyCode) -> Option<egui::Key> {
    use KeyCode::*;
    Some(match keycode {
        LeftAlt | RightAlt => egui::Key::Alt,
        Backspace => egui::Key::Backspace,
        LeftControl | RightControl => egui::Key::Control,
        Delete => egui::Key::Delete,
        Down => egui::Key::Down,
        End => egui::Key::End,
        Escape => egui::Key::Escape,
        Home => egui::Key::Home,
        Insert => egui::Key::Insert,
        Left => egui::Key::Left,
        LeftSuper | RightSuper => egui::Key::Logo,
        PageDown => egui::Key::PageDown,
        PageUp => egui::Key::PageUp,
        Enter => egui::Key::Enter,
        Right => egui::Key::Right,
        LeftShift | RightShift => egui::Key::Shift,
        Tab => egui::Key::Tab,
        Up => egui::Key::Up,
        _ => return None,
    })
}
