use egui::DroppedFile;
use miniquad::Context;
use {egui_miniquad as egui_mq, miniquad as mq};

struct Stage {
    egui_mq: egui_mq::EguiMq,
    show_egui_demo_windows: bool,
    egui_demo_windows: egui_demo_lib::DemoWindows,
    color_test: egui_demo_lib::ColorTest,
    dropped_files: Vec<DroppedFile>,
}

impl Stage {
    fn new(ctx: &mut mq::Context) -> Self {
        Self {
            egui_mq: egui_mq::EguiMq::new(ctx),
            show_egui_demo_windows: true,
            egui_demo_windows: Default::default(),
            color_test: Default::default(),
            dropped_files: Default::default(),
        }
    }
}

impl mq::EventHandler for Stage {
    fn update(&mut self, _ctx: &mut mq::Context) {}

    fn draw(&mut self, ctx: &mut mq::Context) {
        ctx.clear(Some((1., 1., 1., 1.)), None, None);
        ctx.begin_default_pass(mq::PassAction::clear_color(0.0, 0.0, 0.0, 1.0));
        ctx.end_render_pass();

        // Run the UI code:
        self.egui_mq.run(ctx, |egui_ctx| {
            if self.show_egui_demo_windows {
                self.egui_demo_windows.ui(egui_ctx);
            }

            egui::Window::new("egui ❤ miniquad").show(egui_ctx, |ui| {
                egui::widgets::global_dark_light_mode_buttons(ui);
                ui.checkbox(&mut self.show_egui_demo_windows, "Show egui demo windows");

                #[cfg(not(target_arch = "wasm32"))]
                {
                    if ui.button("Quit").clicked() {
                        std::process::exit(0);
                    }
                }
            });

            egui::Window::new("Color Test").show(egui_ctx, |ui| {
                egui::ScrollArea::both()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        self.color_test.ui(ui);
                    });
            });

            // Collect dropped files:
            if !egui_ctx.input().raw.dropped_files.is_empty() {
                self.dropped_files = egui_ctx.input().raw.dropped_files.clone();
            }

            // Show dropped files (if any):
            if !self.dropped_files.is_empty() {
                let mut open = true;
                egui::Window::new("Dropped files")
                    .open(&mut open)
                    .show(egui_ctx, |ui| {
                        for file in &self.dropped_files {
                            let mut info = if let Some(path) = &file.path {
                                path.display().to_string()
                            } else if !file.name.is_empty() {
                                file.name.clone()
                            } else {
                                "???".to_owned()
                            };
                            if let Some(bytes) = &file.bytes {
                                info += &format!(" ({} bytes)", bytes.len());
                            }
                            ui.label(info);
                        }
                    });
                if !open {
                    self.dropped_files.clear();
                }
            }
        });

        // Draw things behind egui here

        self.egui_mq.draw(ctx);

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

    fn files_dropped_event(&mut self, _ctx: &mut Context) {
        self.egui_mq.files_dropped_event();
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
