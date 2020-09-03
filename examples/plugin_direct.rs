use {
    egui::vec2,
    emigui_miniquad::UiPlugin,
    miniquad::{
        self as mq, conf, Context, EventHandler, EventHandlerFree, KeyCode, KeyMods, MouseButton,
        TouchPhase,
    },
};

struct Stage {
    ui: UiPlugin,
}

impl EventHandler for Stage {
    fn resize_event(&mut self, _: &mut Context, width: f32, height: f32) {
        self.ui.resize_event(width, height);
    }

    fn mouse_motion_event(&mut self, _: &mut Context, x: f32, y: f32) {
        self.ui.mouse_motion_event(x, y);
    }
    fn mouse_wheel_event(&mut self, _: &mut Context, x: f32, y: f32) {
        self.ui.mouse_wheel_event(x, y);
    }
    fn mouse_button_down_event(&mut self, _: &mut Context, btn: MouseButton, x: f32, y: f32) {
        self.ui.mouse_button_down_event(btn, x, y);
    }
    fn mouse_button_up_event(&mut self, _: &mut Context, btn: MouseButton, x: f32, y: f32) {
        self.ui.mouse_button_up_event(btn, x, y);
    }

    fn char_event(&mut self, _: &mut Context, character: char, modifiers: KeyMods, repeat: bool) {
        self.ui.char_event(character, modifiers, repeat);
    }

    fn key_down_event(
        &mut self,
        _: &mut Context,
        keycode: KeyCode,
        modifiers: KeyMods,
        repeat: bool,
    ) {
        self.ui.key_down_event(keycode, modifiers, repeat);
    }
    fn key_up_event(&mut self, _: &mut Context, keycode: KeyCode, modifiers: KeyMods) {
        self.ui.key_up_event(keycode, modifiers);
    }

    fn touch_event(&mut self, _: &mut Context, phase: TouchPhase, id: u64, x: f32, y: f32) {
        self.ui.touch_event(phase, id, x, y);
    }

    fn update(&mut self, _: &mut Context) {}

    fn draw(&mut self, ctx: &mut Context) {
        //ctx.clear(Some((1., 1., 1., 1.)), None, None);
        ctx.begin_default_pass(mq::PassAction::clear_color(0.0, 0.0, 0.0, 1.0));
        ctx.end_render_pass();

        self.ui.ui(ctx, |ui| {
            egui::Window::new("Debug")
                .default_size(vec2(200.0, 100.0))
                .show(ui.ctx(), |ui| {
                    ui.add(
                        egui::Label::new("Egui on Miniquad").text_style(egui::TextStyle::Heading),
                    );
                    ui.separator();
                    ui.label("Woooohoooo!");
                    if ui.button("Quit").clicked {
                        std::process::exit(0);
                    }
                });
        });

        self.ui.draw(ctx);
    }
}

fn main() {
    miniquad::start(conf::Conf::default(), |mut ctx| {
        let ui_plugin = Stage {
            ui: UiPlugin::new(&mut ctx),
        };
        mq::UserData::owning(ui_plugin, ctx)
    });
}
