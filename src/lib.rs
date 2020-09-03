use egui::PaintJobs;
use miniquad::KeyCode;
mod painter;

pub use painter::Painter;

/// The number of unique miniquad::KeyCodes
const KEY_COUNT: usize = 128;

pub struct UiPlugin {
    pub egui_ctx: std::sync::Arc<egui::Context>,
    pub raw_input: egui::RawInput,
    pub paint_jobs: PaintJobs,
    pub painter: Painter,
    #[cfg(feature = "macroquad")]
    pub tracked_keys: [bool; KEY_COUNT],
    pub start_time: f64,
}

impl UiPlugin {
    #[cfg(feature = "macroquad")]
    pub fn new() -> Self {
        unsafe { Self::from_mq(macroquad::get_internal_mq()) }
    }

    pub fn from_mq(ctx: &mut miniquad::Context) -> Self {
        let egui_ctx = egui::Context::new();

        let pixels_per_point = ctx.dpi_scale();
        let (width, height) = ctx.screen_size();
        let screen_size = egui::vec2(width as f32, height as f32) / pixels_per_point;

        let raw_input = egui::RawInput {
            screen_size,
            pixels_per_point: Some(pixels_per_point),
            ..Default::default()
        };

        Self {
            egui_ctx,
            painter: Painter::new(ctx),
            paint_jobs: Vec::with_capacity(10_000),
            start_time: miniquad::date::now(),
            #[cfg(feature = "macroquad")]
            tracked_keys: [false; KEY_COUNT],
            raw_input,
        }
    }

    pub fn draw(&mut self, ctx: &mut miniquad::Context) {
        self.painter.paint(
            ctx,
            std::mem::take(&mut self.paint_jobs),
            self.egui_ctx.texture(),
        );
    }

    #[cfg(feature = "macroquad")]
    fn collect_input(&mut self) {
        use macroquad::*;

        self.raw_input.screen_size = egui::vec2(screen_width(), screen_height());
        self.raw_input.mouse_pos = {
            let (x, y) = mouse_position();
            Some(egui::pos2(x, y))
        };
        self.raw_input.scroll_delta = {
            let (x, y) = mouse_wheel();
            egui::vec2(x, y)
        };
        self.raw_input.mouse_down = is_mouse_button_down(MouseButton::Left);
        for &kc in EGUI_KEYS.iter() {
            if let Some(key) = convert_keycode(kc).filter(|_| is_key_pressed(kc)) {
                self.tracked_keys[kc as usize] = true;
                self.raw_input
                    .events
                    .push(egui::Event::Key { key, pressed: true });
            }
        }
        for (i, pressed) in self
            .tracked_keys
            .iter_mut()
            .enumerate()
            .filter(|(_, p)| **p)
        {
            let kc = KeyCode::from(i as u32);
            if let Some(key) = convert_keycode(kc).filter(|_| !is_key_down(kc)) {
                *pressed = false;
                self.raw_input.events.push(egui::Event::Key {
                    key,
                    pressed: false,
                });
            }
        }
        let typed_characters = typed_characters();
        if !typed_characters.is_empty() {
            self.raw_input
                .events
                .push(egui::Event::Text(typed_characters));
        }
    }

    #[cfg(feature = "macroquad")]
    pub fn macroquad(&mut self, f: impl FnOnce(&mut egui::Ui)) {
        self.collect_input();
        let output = self.ui(f);

        unsafe {
            let ctx = macroquad::get_internal_mq();
            self.apply_output(ctx, output);
            self.draw(ctx);
        }
    }

    pub fn ui(&mut self, f: impl FnOnce(&mut egui::Ui)) -> egui::Output {
        self.raw_input.time = miniquad::date::now() - self.start_time;

        let (output, paint_jobs) = {
            let mut ui = self.egui_ctx.begin_frame(self.raw_input.take());
            f(&mut ui);
            self.egui_ctx.end_frame()
        };

        self.paint_jobs = paint_jobs;
        output
    }

    pub fn apply_output(&mut self, ctx: &mut miniquad::Context, output: egui::Output) {
        if !output.copied_text.is_empty() {
            miniquad::clipboard::set(ctx, &output.copied_text)
        }
    }
}

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
