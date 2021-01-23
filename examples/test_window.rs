use {
    egui::{pos2, vec2},
    emigui_miniquad::Painter,
    miniquad::{self as mq, conf, Context, EventHandler},
};

struct Stage {
    egui_ctx: egui::CtxRef,
    raw_input: egui::RawInput,
    painter: Painter,
}

impl Stage {
    fn new(ctx: &mut mq::Context) -> Self {
        let egui_ctx = egui::CtxRef::default();

        let pixels_per_point = ctx.dpi_scale();
        let (width, height) = ctx.screen_size();
        let screen_size = vec2(width as f32, height as f32) / pixels_per_point;

        let raw_input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(Default::default(), screen_size)),
            pixels_per_point: Some(pixels_per_point),
            ..Default::default()
        };

        Self {
            egui_ctx,
            painter: Painter::new(ctx),
            raw_input,
        }
    }
}

impl EventHandler for Stage {
    fn update(&mut self, _ctx: &mut Context) {}

    fn resize_event(&mut self, _ctx: &mut Context, width: f32, height: f32) {
        self.raw_input.screen_rect = Some(egui::Rect::from_min_size(
            Default::default(),
            egui::vec2(width, height),
        ));
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
        self.raw_input.time = Some(miniquad::date::now());

        self.egui_ctx.begin_frame(self.raw_input.take());
        egui::Window::new("Debug")
            .default_size(vec2(200.0, 100.0))
            .show(&self.egui_ctx, |ui| {
                ui.add(egui::Label::new("Egui on Miniquad").text_style(egui::TextStyle::Heading));
                ui.separator();
                ui.label("Woooohoooo!");
                if ui.button("Quit").clicked {
                    std::process::exit(0);
                }
            });
        // TODO: handle this output so that hyperlinks, etc. work
        let (_, shapes) = self.egui_ctx.end_frame();
        let paint_jobs = self.egui_ctx.tessellate(shapes);

        self.painter
            .paint(ctx, paint_jobs, &self.egui_ctx.texture());
    }
}

fn main() {
    let conf = conf::Conf {
        // high_dpi: true, // TODO after https://github.com/not-fl3/miniquad/issues/169 is fixed
        ..Default::default()
    };
    miniquad::start(conf, |mut ctx| {
        mq::UserData::owning(Stage::new(&mut ctx), ctx)
    });
}
