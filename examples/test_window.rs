use {
    emigui_miniquad::{convert_keycode, Painter},
    miniquad::{self as mq, conf, Context, EventHandler, KeyCode, KeyMods, MouseButton},
};

struct Stage {
    egui_ctx: std::sync::Arc<egui::Context>,
    raw_input: egui::RawInput,
    start_time: f64,
    painter: Painter,
}

impl EventHandler for Stage {
    fn resize_event(&mut self, _: &mut Context, width: f32, height: f32) {
        self.raw_input.screen_size = egui::vec2(width, height);
    }

    fn mouse_motion_event(&mut self, _: &mut Context, x: f32, y: f32) {
        self.raw_input.mouse_pos = Some(egui::pos2(x, y));
    }
    fn mouse_wheel_event(&mut self, _: &mut Context, x: f32, y: f32) {
        self.raw_input.scroll_delta = egui::vec2(x, y);
    }
    fn mouse_button_down_event(&mut self, _: &mut Context, _btn: MouseButton, x: f32, y: f32) {
        self.raw_input.mouse_pos = Some(egui::pos2(x, y));
        self.raw_input.mouse_down = true;
    }
    fn mouse_button_up_event(&mut self, _: &mut Context, _btn: MouseButton, x: f32, y: f32) {
        self.raw_input.mouse_pos = Some(egui::pos2(x, y));
        self.raw_input.mouse_down = false;
    }

    fn char_event(&mut self, _: &mut Context, character: char, _modifiers: KeyMods, _repeat: bool) {
        self.raw_input
            .events
            .push(egui::Event::Text(String::from(character)));
    }

    fn key_down_event(
        &mut self,
        _: &mut Context,
        keycode: KeyCode,
        _modifiers: KeyMods,
        _repeat: bool,
    ) {
        if let Some(key) = convert_keycode(keycode) {
            self.raw_input
                .events
                .push(egui::Event::Key { key, pressed: true });
        }
    }
    fn key_up_event(&mut self, _: &mut Context, keycode: KeyCode, _: KeyMods) {
        if let Some(key) = convert_keycode(keycode) {
            self.raw_input.events.push(egui::Event::Key {
                key,
                pressed: false,
            });
        }
    }

    fn update(&mut self, _: &mut Context) {}

    fn draw(&mut self, ctx: &mut Context) {
        ctx.clear(Some((1., 1., 1., 1.)), None, None);
        ctx.begin_default_pass(mq::PassAction::clear_color(0.0, 0.0, 0.0, 1.0));
        ctx.end_render_pass();

        // TODO: give all of the raw_input information egui wants so everything works properly
        self.raw_input.time = miniquad::date::now() - self.start_time;

        let ui = self.egui_ctx.begin_frame(self.raw_input.take());
        egui::Window::new("Debug")
            .default_size(egui::vec2(200.0, 100.0))
            .show(ui.ctx(), |ui| {
                ui.add(egui::Label::new("Egui on Miniquad").text_style(egui::TextStyle::Heading));
                ui.separator();
                ui.label("Woooohoooo!");
                if ui.button("Quit").clicked {
                    std::process::exit(0);
                }
            });
        // TODO: handle this output so that hyperlinks, etc. work
        let (_, mut paint_jobs) = self.egui_ctx.end_frame();

        self.painter.paint(ctx, &mut paint_jobs, self.egui_ctx.texture());
    }
}

fn main() {
    miniquad::start(conf::Conf::default(), |mut ctx| {
        mq::UserData::owning(
            {
                let egui_ctx = egui::Context::new();

                let pixels_per_point = ctx.dpi_scale();
                let (width, height) = ctx.screen_size();
                let screen_size = egui::vec2(width as f32, height as f32) / pixels_per_point;

                let raw_input = egui::RawInput {
                    screen_size,
                    pixels_per_point: Some(pixels_per_point),
                    ..Default::default()
                };

                Stage {
                    egui_ctx,
                    painter: Painter::new(&mut ctx),
                    raw_input,
                    start_time: miniquad::date::now(),
                }
            },
            ctx,
        )
    });
}
