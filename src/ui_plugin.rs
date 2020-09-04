use egui::PaintJobs;
use super::{convert_keycode, EGUI_KEYS, Painter};

/// The number of unique miniquad::KeyCodes
const KEY_COUNT: usize = 128;

// This has to be in a static so that the hook can access it
static mut CONTEXT: Option<Context> = None;

fn get_context() -> &'static mut Context {
    unsafe { CONTEXT.as_mut().expect("somehow the context was removed/never initialized") }
}

struct Context {
    texture: Option<egui::Texture>,
    paint_jobs: PaintJobs,
    painter: Painter,
}

impl Context {
    fn draw(&mut self, ctx: &mut miniquad::Context) {
        if let Some(tex) = &self.texture {
            self.painter.paint(
                ctx,
                &mut self.paint_jobs,
                tex,
            );
        }
    }
}

pub struct UiPlugin {
    pub egui_ctx: std::sync::Arc<egui::Context>,
    pub raw_input: egui::RawInput,
    pub tracked_keys: [bool; KEY_COUNT],
    pub start_time: f64,
}

impl UiPlugin {
    pub fn new() -> Self {
        unsafe {
            let s = Self::from_mq(macroquad::get_internal_mq());

            if CONTEXT.is_none() {
                CONTEXT = Some(Context {
                    texture: None,
                    paint_jobs: Vec::with_capacity(1_000),
                    painter: Painter::new(macroquad::get_internal_mq()),
                });

                macroquad::hook_post_render(|ctx| get_context().draw(ctx))
            }

            s
        }
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
            start_time: miniquad::date::now(),
            tracked_keys: [false; KEY_COUNT],
            raw_input,
        }
    }

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
                .push(egui::Event::Text(typed_characters.to_string()));
        }
    }

    pub fn macroquad(&mut self, f: impl FnOnce(&mut egui::Ui)) {
        self.collect_input();
        let output = self.ui(f);

        unsafe { self.apply_output(macroquad::get_internal_mq(), output); }
    }

    pub fn ui(&mut self, f: impl FnOnce(&mut egui::Ui)) -> egui::Output {
        self.raw_input.time = miniquad::date::now() - self.start_time;

        let (output, paint_jobs) = {
            let mut ui = self.egui_ctx.begin_frame(self.raw_input.take());
            f(&mut ui);
            self.egui_ctx.end_frame()
        };

        let new_tex = self.egui_ctx.texture();
        let now_tex = &mut get_context().texture;
        if now_tex.as_ref().filter(|t| t.id != new_tex.id).is_none() {
            *now_tex = Some(new_tex.clone());
        }
        get_context().paint_jobs = paint_jobs;

        output
    }

    pub fn apply_output(&mut self, ctx: &mut miniquad::Context, output: egui::Output) {
        if !output.copied_text.is_empty() {
            miniquad::clipboard::set(ctx, &output.copied_text)
        }
    }
}
