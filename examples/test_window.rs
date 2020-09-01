use {
    egui::{pos2, vec2},
    emigui_miniquad::Painter,
    miniquad::{self as mq, conf, Context, EventHandler},
    std::time::Instant,
};

struct Stage {
    egui_ctx: std::sync::Arc<egui::Context>,
    raw_input: egui::RawInput,
    start_time: Instant,
    painter: Painter,
}

impl EventHandler for Stage {
    fn update(&mut self, _ctx: &mut Context) {}

    fn resize_event(&mut self, _ctx: &mut Context, width: f32, height: f32) {
        self.raw_input.screen_size = vec2(width, height);
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        _: miniquad::MouseButton,
        _x: f32,
        _y: f32,
    ) {
        self.raw_input.mouse_down = true;
    }
    fn mouse_button_up_event(
        &mut self,
        _ctx: &mut Context,
        _: miniquad::MouseButton,
        _x: f32,
        _y: f32,
    ) {
        self.raw_input.mouse_down = false;
    }

    fn mouse_motion_event(&mut self, _ctx: &mut Context, x: f32, y: f32) {
        self.raw_input.mouse_pos = Some(pos2(x as f32, y as f32));
    }

    fn draw(&mut self, ctx: &mut Context) {
        ctx.clear(Some((1., 1., 1., 1.)), None, None);
        ctx.begin_default_pass(mq::PassAction::clear_color(0.0, 0.0, 0.0, 1.0));
        ctx.end_render_pass();

        // TODO: give all of the raw_input information egui wants so everything works properly
        self.raw_input.time = self.start_time.elapsed().as_nanos() as f64 * 1e-9;

        let ui = self.egui_ctx.begin_frame(self.raw_input.take());
        egui::Window::new("Debug").default_size(vec2(200.0, 100.0)).show(ui.ctx(), |ui| {
            ui.add(
                egui::Label::new("Egui on Miniquad")
                    .text_style(egui::TextStyle::Heading),
            );
            ui.separator();
            ui.label("Woooohoooo!");
            if ui.button("Quit").clicked {
                std::process::exit(0);
            }
        });
        // TODO: handle this output so that hyperlinks, etc. work
        let (_, paint_jobs) = self.egui_ctx.end_frame();

        self.painter.paint(ctx, paint_jobs, self.egui_ctx.texture());
    }
}

fn main() {
    miniquad::start(conf::Conf::default(), |mut ctx| {
        mq::UserData::owning(
            {
                let egui_ctx = egui::Context::new();

                let pixels_per_point = ctx.dpi_scale();
                let (width, height) = ctx.screen_size();
                let screen_size = vec2(width as f32, height as f32) / pixels_per_point;

                let raw_input = egui::RawInput {
                    screen_size,
                    pixels_per_point: Some(pixels_per_point),
                    ..Default::default()
                };

                Stage {
                    egui_ctx,
                    painter: Painter::new(&mut ctx),
                    raw_input,
                    start_time: Instant::now(),
                }
            },
            ctx,
        )
    });
}
