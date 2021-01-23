use {emigui_miniquad as egui_mq, miniquad as mq};

struct Stage {
    egui_ctx: egui::CtxRef,
    egui_input: egui::RawInput,
    painter: egui_mq::Painter,

    show_egui_demo_windows: bool,

    egui_demo_windows: egui_demo_lib::DemoWindows,
}

impl Stage {
    fn new(ctx: &mut mq::Context) -> Self {
        Self {
            egui_ctx: egui::CtxRef::default(),
            painter: egui_mq::Painter::new(ctx),
            egui_input: Default::default(),
            show_egui_demo_windows: true,
            egui_demo_windows: Default::default(),
        }
    }

    fn ui(&mut self) {
        let Self {
            egui_ctx,
            show_egui_demo_windows,
            egui_demo_windows,
            ..
        } = self;

        if *show_egui_demo_windows {
            egui_demo_windows.ui(&egui_ctx);
        }

        egui::Window::new("Debug").show(&egui_ctx, |ui| {
            ui.add(egui::Label::new("Egui on Miniquad").text_style(egui::TextStyle::Heading));
            ui.separator();
            ui.checkbox(show_egui_demo_windows, "Show egui demo windows");
            ui.label("Woooohoooo!");
            if ui.button("Quit").clicked {
                std::process::exit(0);
            }
        });
    }
}

impl mq::EventHandler for Stage {
    fn update(&mut self, _ctx: &mut mq::Context) {}

    fn draw(&mut self, ctx: &mut mq::Context) {
        ctx.clear(Some((1., 1., 1., 1.)), None, None);
        ctx.begin_default_pass(mq::PassAction::clear_color(0.0, 0.0, 0.0, 1.0));
        ctx.end_render_pass();

        egui_mq::on_frame_start(&mut self.egui_input, ctx);
        self.egui_ctx.begin_frame(self.egui_input.take());

        self.ui();

        // TODO: handle this output so that hyperlinks, etc. work
        let (_, shapes) = self.egui_ctx.end_frame();
        let paint_jobs = self.egui_ctx.tessellate(shapes);

        self.painter
            .paint(ctx, paint_jobs, &self.egui_ctx.texture());
    }

    fn mouse_motion_event(&mut self, ctx: &mut mq::Context, x: f32, y: f32) {
        self.egui_input.mouse_pos = Some(egui::pos2(
            x as f32 / ctx.dpi_scale(),
            y as f32 / ctx.dpi_scale(),
        ));
    }

    fn mouse_wheel_event(&mut self, ctx: &mut mq::Context, dx: f32, dy: f32) {
        self.egui_input.scroll_delta += egui::vec2(dx, dy) * ctx.dpi_scale(); // not quite right speed on Mac for some reason
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut mq::Context,
        _: mq::MouseButton,
        _x: f32,
        _y: f32,
    ) {
        self.egui_input.mouse_down = true;
    }

    fn mouse_button_up_event(
        &mut self,
        _ctx: &mut mq::Context,
        _: mq::MouseButton,
        _x: f32,
        _y: f32,
    ) {
        self.egui_input.mouse_down = false;
    }

    fn char_event(
        &mut self,
        _ctx: &mut mq::Context,
        character: char,
        _keymods: mq::KeyMods,
        _repeat: bool,
    ) {
        egui_mq::char_event(&mut self.egui_input, character);
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut mq::Context,
        keycode: mq::KeyCode,
        keymods: mq::KeyMods,
        _repeat: bool,
    ) {
        egui_mq::key_down_event(&mut self.egui_input, keycode, keymods);
    }

    fn key_up_event(&mut self, _ctx: &mut mq::Context, keycode: mq::KeyCode, keymods: mq::KeyMods) {
        egui_mq::key_up_event(&mut self.egui_input, keycode, keymods);
    }
}

fn main() {
    let conf = mq::conf::Conf {
        high_dpi: true,
        ..Default::default()
    };
    mq::start(conf, |mut ctx| {
        mq::UserData::owning(Stage::new(&mut ctx), ctx)
    });
}
