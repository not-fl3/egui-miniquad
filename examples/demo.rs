use {egui_miniquad as egui_mq, miniquad as mq};

struct Stage {
    egui_mq: egui_mq::EguiMq,
    show_egui_demo_windows: bool,
    egui_demo_windows: egui_demo_lib::DemoWindows,
}

impl Stage {
    fn new(ctx: &mut mq::Context) -> Self {
        Self {
            egui_mq: egui_mq::EguiMq::new(ctx),
            show_egui_demo_windows: true,
            egui_demo_windows: Default::default(),
        }
    }

    fn ui(&mut self) {
        let Self {
            egui_mq,
            show_egui_demo_windows,
            egui_demo_windows,
        } = self;

        let egui_ctx = egui_mq.egui_ctx();

        if *show_egui_demo_windows {
            egui_demo_windows.ui(egui_ctx);
        }

        egui::Window::new("egui ❤ miniquad").show(egui_ctx, |ui| {
            ui.checkbox(show_egui_demo_windows, "Show egui demo windows");

            #[cfg(not(target_arch = "wasm32"))]
            {
                if ui.button("Quit").clicked() {
                    std::process::exit(0);
                }
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

        // Draw things behind egui here

        self.egui_mq.begin_frame(ctx);
        self.ui();
        self.egui_mq.end_frame(ctx);

        // Draw things in front of egui here

        ctx.commit_frame();
    }

    fn mouse_motion_event(&mut self, ctx: &mut mq::Context, x: f32, y: f32) {
        self.egui_mq.mouse_motion_event(ctx, x, y);
    }

    fn mouse_wheel_event(&mut self, ctx: &mut mq::Context, dx: f32, dy: f32) {
        self.egui_mq.mouse_wheel_event(ctx, dx, dy);
    }

    fn mouse_button_down_event(
        &mut self,
        ctx: &mut mq::Context,
        mb: mq::MouseButton,
        x: f32,
        y: f32,
    ) {
        self.egui_mq.mouse_button_down_event(ctx, mb, x, y);
    }

    fn mouse_button_up_event(
        &mut self,
        ctx: &mut mq::Context,
        mb: mq::MouseButton,
        x: f32,
        y: f32,
    ) {
        self.egui_mq.mouse_button_up_event(ctx, mb, x, y);
    }

    fn char_event(
        &mut self,
        _ctx: &mut mq::Context,
        character: char,
        _keymods: mq::KeyMods,
        _repeat: bool,
    ) {
        self.egui_mq.char_event(character);
    }

    fn key_down_event(
        &mut self,
        ctx: &mut mq::Context,
        keycode: mq::KeyCode,
        keymods: mq::KeyMods,
        _repeat: bool,
    ) {
        self.egui_mq.key_down_event(ctx, keycode, keymods);
    }

    fn key_up_event(&mut self, _ctx: &mut mq::Context, keycode: mq::KeyCode, keymods: mq::KeyMods) {
        self.egui_mq.key_up_event(keycode, keymods);
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
