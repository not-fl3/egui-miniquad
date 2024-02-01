use miniquad as mq;

pub fn on_frame_start(egui_input: &mut egui::RawInput, equi_ctx: &egui::Context) {
    let screen_size_in_pixels = mq::window::screen_size();
    let screen_size_in_points =
        egui::vec2(screen_size_in_pixels.0, screen_size_in_pixels.1) / equi_ctx.pixels_per_point();
    egui_input.screen_rect = Some(egui::Rect::from_min_size(
        Default::default(),
        screen_size_in_points,
    ));
    egui_input.time = Some(mq::date::now());
}

/// miniquad sends special keys (backspace, delete, F1, ...) as characters.
/// Ignore those.
/// We also ignore '\r', '\n', '\t'.
/// Newlines are handled by the `Key::Enter` event.
pub fn is_printable_char(chr: char) -> bool {
    #![allow(clippy::manual_range_contains)]

    let is_in_private_use_area = '\u{e000}' <= chr && chr <= '\u{f8ff}'
        || '\u{f0000}' <= chr && chr <= '\u{ffffd}'
        || '\u{100000}' <= chr && chr <= '\u{10fffd}';

    !is_in_private_use_area && !chr.is_ascii_control()
}

pub fn egui_modifiers_from_mq_modifiers(keymods: mq::KeyMods) -> egui::Modifiers {
    egui::Modifiers {
        alt: keymods.alt,
        ctrl: keymods.ctrl,
        shift: keymods.shift,
        mac_cmd: keymods.logo && cfg!(target_os = "macos"),
        command: if cfg!(target_os = "macos") {
            keymods.logo
        } else {
            keymods.ctrl
        },
    }
}

pub fn egui_key_from_mq_key(key: mq::KeyCode) -> Option<egui::Key> {
    Some(match key {
        mq::KeyCode::Down => egui::Key::ArrowDown,
        mq::KeyCode::Left => egui::Key::ArrowLeft,
        mq::KeyCode::Right => egui::Key::ArrowRight,
        mq::KeyCode::Up => egui::Key::ArrowUp,

        mq::KeyCode::Escape => egui::Key::Escape,
        mq::KeyCode::Tab => egui::Key::Tab,
        mq::KeyCode::Backspace => egui::Key::Backspace,
        mq::KeyCode::Enter => egui::Key::Enter,
        mq::KeyCode::Space => egui::Key::Space,

        mq::KeyCode::Insert => egui::Key::Insert,
        mq::KeyCode::Delete => egui::Key::Delete,
        mq::KeyCode::Home => egui::Key::Home,
        mq::KeyCode::End => egui::Key::End,
        mq::KeyCode::PageUp => egui::Key::PageUp,
        mq::KeyCode::PageDown => egui::Key::PageDown,

        mq::KeyCode::Key0 => egui::Key::Num0,
        mq::KeyCode::Key1 => egui::Key::Num1,
        mq::KeyCode::Key2 => egui::Key::Num2,
        mq::KeyCode::Key3 => egui::Key::Num3,
        mq::KeyCode::Key4 => egui::Key::Num4,
        mq::KeyCode::Key5 => egui::Key::Num5,
        mq::KeyCode::Key6 => egui::Key::Num6,
        mq::KeyCode::Key7 => egui::Key::Num7,
        mq::KeyCode::Key8 => egui::Key::Num8,
        mq::KeyCode::Key9 => egui::Key::Num9,

        mq::KeyCode::A => egui::Key::A,
        mq::KeyCode::B => egui::Key::B,
        mq::KeyCode::C => egui::Key::C,
        mq::KeyCode::D => egui::Key::D,
        mq::KeyCode::E => egui::Key::E,
        mq::KeyCode::F => egui::Key::F,
        mq::KeyCode::G => egui::Key::G,
        mq::KeyCode::H => egui::Key::H,
        mq::KeyCode::I => egui::Key::I,
        mq::KeyCode::J => egui::Key::J,
        mq::KeyCode::K => egui::Key::K,
        mq::KeyCode::L => egui::Key::L,
        mq::KeyCode::M => egui::Key::M,
        mq::KeyCode::N => egui::Key::N,
        mq::KeyCode::O => egui::Key::O,
        mq::KeyCode::P => egui::Key::P,
        mq::KeyCode::Q => egui::Key::Q,
        mq::KeyCode::R => egui::Key::R,
        mq::KeyCode::S => egui::Key::S,
        mq::KeyCode::T => egui::Key::T,
        mq::KeyCode::U => egui::Key::U,
        mq::KeyCode::V => egui::Key::V,
        mq::KeyCode::W => egui::Key::W,
        mq::KeyCode::X => egui::Key::X,
        mq::KeyCode::Y => egui::Key::Y,
        mq::KeyCode::Z => egui::Key::Z,

        mq::KeyCode::Comma => egui::Key::Comma,
        mq::KeyCode::Backslash => egui::Key::Backslash,
        mq::KeyCode::LeftBracket => egui::Key::OpenBracket,
        mq::KeyCode::RightBracket => egui::Key::CloseBracket,
        mq::KeyCode::GraveAccent => egui::Key::Backtick,
        mq::KeyCode::Minus => egui::Key::Minus,
        mq::KeyCode::Period => egui::Key::Period,
        mq::KeyCode::Equal => egui::Key::Equals,
        mq::KeyCode::Semicolon => egui::Key::Semicolon,

        _other => {
            // mq::trace!("egui-miniquad: Unhandled key: {:?}", _other);
            return None;
        }
    })
}
