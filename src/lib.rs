use egui::PaintJobs;
use miniquad::{KeyCode, KeyMods, MouseButton};
mod painter;

pub use painter::Painter;

pub struct UiPlugin {
    pub egui_ctx: std::sync::Arc<egui::Context>,
    pub raw_input: egui::RawInput,
    pub paint_jobs: PaintJobs,
    pub painter: Painter,
}

impl UiPlugin {
    pub fn new(ctx: &mut miniquad::Context) -> Self {
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
            raw_input,
        }
    }

    pub fn draw(&mut self, ctx: &mut miniquad::Context) {
        self.painter.paint(
            ctx,
            std::mem::take(&mut self.paint_jobs),
            self.egui_ctx.texture()
        );
    }

    pub fn ui(&mut self, ctx: &mut miniquad::Context, f: impl FnOnce(&mut egui::Ui)) {
        let input = self.raw_input.take();
        let mut ui = self.egui_ctx.begin_frame(input);
        f(&mut ui);
        let (output, paint_jobs) = self.egui_ctx.end_frame();
        if !output.copied_text.is_empty() {
            miniquad::clipboard::set(ctx, &output.copied_text)
        }

        self.paint_jobs = paint_jobs;
    }
}

#[cfg(feature = "macroquad-plugin")]
pub fn ui(f: impl FnOnce(&mut egui::Ui)) {
    macroquad::custom_ui::<UiPlugin, _>(|ctx, plugin| plugin.ui(ctx, f));
}

#[cfg(feature = "macroquad-plugin")]
impl macroquad::drawing::DrawableUi for UiPlugin {
    fn draw(&mut self, _: &mut quad_gl::QuadGl, ctx: &mut miniquad::Context) {
        self.draw(ctx);
    }
    fn mut_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

pub fn convert_keycode(keycode: KeyCode) -> Option<egui::Key> {
    Some(match keycode {
        KeyCode::Up => egui::Key::Up,
        KeyCode::Down => egui::Key::Down,
        KeyCode::Right => egui::Key::Right,
        KeyCode::Left => egui::Key::Left,
        KeyCode::Home => egui::Key::Home,
        KeyCode::End => egui::Key::End,
        KeyCode::Delete => egui::Key::Delete,
        KeyCode::Backspace => egui::Key::Backspace,
        KeyCode::Enter => egui::Key::Enter,
        KeyCode::Tab => egui::Key::Tab,
        _ => return None,
    })
}

impl miniquad::EventHandlerFree for UiPlugin {
    fn resize_event(&mut self, width: f32, height: f32) {
        self.raw_input.screen_size = egui::vec2(width, height);
    }

    fn mouse_motion_event(&mut self, x: f32, y: f32) {
        self.raw_input.mouse_pos = Some(egui::pos2(x, y));
    }
    fn mouse_wheel_event(&mut self, x: f32, y: f32) {
        self.raw_input.scroll_delta = egui::vec2(x, y);
    }
    fn mouse_button_down_event(&mut self, _btn: MouseButton, x: f32, y: f32) {
        self.raw_input.mouse_pos = Some(egui::pos2(x, y));
        self.raw_input.mouse_down = true;
    }
    fn mouse_button_up_event(&mut self, _btn: MouseButton, x: f32, y: f32) {
        self.raw_input.mouse_pos = Some(egui::pos2(x, y));
        self.raw_input.mouse_down = false;
    }

    fn char_event(&mut self, character: char, _modifiers: KeyMods, _repeat: bool) {
        self.raw_input.events.push(egui::Event::Text(String::from(character)));
    }

    fn key_down_event(&mut self, keycode: KeyCode, _modifiers: KeyMods, _repeat: bool) {
        if let Some(key) = convert_keycode(keycode) {
            self.raw_input.events.push(egui::Event::Key {
                key,
                pressed: true,
            });
        }
    }
    fn key_up_event(&mut self, keycode: KeyCode, _: KeyMods) {
        if let Some(key) = convert_keycode(keycode) {
            self.raw_input.events.push(egui::Event::Key {
                key,
                pressed: false,
            });
        }
    }

    fn draw(&mut self) {}
    fn update(&mut self) {}
}

