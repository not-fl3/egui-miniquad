use {
    emigui::{
        label,
        math::vec2,
        widgets::{Button, Label},
        Align, Emigui,
    },
    emigui_miniquad::Painter,
    miniquad::{self as mq, conf, Context, EventHandler},
};

struct Stage {
    emigui: Emigui,
    raw_input: emigui::RawInput,
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
        self.raw_input.mouse_pos = Some(vec2(x as f32, y as f32));
    }

    fn draw(&mut self, ctx: &mut Context) {
        ctx.clear(Some((0., 0., 0., 1.)), None, None);

        self.emigui.new_frame(self.raw_input);
        let mut region = self.emigui.whole_screen_region();
        let mut region = region.left_column(region.width().min(480.0));
        region.set_align(Align::Min);
        region.add(
            label!("Emigui running inside of Miniquad").text_style(emigui::TextStyle::Heading),
        );
        if region.add(Button::new("Quit")).clicked {
            std::process::exit(0);
        }
        self.emigui.example(&mut region);
        let mesh = self.emigui.paint();
        let texture = self.emigui.texture();

        self.painter.paint(ctx, mesh, texture);
    }
}

fn main() {
    miniquad::start(conf::Conf::default(), |mut ctx| {
        mq::UserData::owning(
            {
                let pixels_per_point = ctx.dpi_scale();
                let raw_input = emigui::RawInput {
                    screen_size: {
                        let (width, height) = ctx.screen_size();
                        vec2(width as f32, height as f32) / pixels_per_point
                    },
                    pixels_per_point,
                    ..Default::default()
                };

                Stage {
                    emigui: Emigui::new(ctx.dpi_scale()),
                    painter: Painter::new(&mut ctx),
                    raw_input,
                }
            },
            ctx,
        )
    });
}
