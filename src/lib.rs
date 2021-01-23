mod input;
mod painter;

// ----------------------------------------------------------------------------

use miniquad as mq;

/// egui bindings for miniquad
pub struct EguiMq {
    egui_ctx: egui::CtxRef,
    egui_input: egui::RawInput,
    painter: painter::Painter,
}

impl EguiMq {
    pub fn new(mq_ctx: &mut mq::Context) -> Self {
        Self {
            egui_ctx: egui::CtxRef::default(),
            painter: painter::Painter::new(mq_ctx),
            egui_input: Default::default(),
        }
    }

    /// Use this to open egui windows, panels etc.
    /// Can only be used between [`Self::begin_frmae`] and [`Self::end_frame`].
    pub fn egui_ctx(&self) -> &egui::CtxRef {
        &self.egui_ctx
    }

    /// Call this at the start of each `draw` call.
    pub fn begin_frame(&mut self, mq_ctx: &mut mq::Context) {
        input::on_frame_start(&mut self.egui_input, mq_ctx);
        self.egui_ctx.begin_frame(self.egui_input.take());
    }

    /// Call this at the end of each `draw` call.
    pub fn end_frame(&mut self, mq_ctx: &mut mq::Context) {
        // TODO: handle this output so that hyperlinks, etc. work
        let (_output, shapes) = self.egui_ctx.end_frame();
        let paint_jobs = self.egui_ctx.tessellate(shapes);

        self.painter
            .paint(mq_ctx, paint_jobs, &self.egui_ctx.texture());
    }

    pub fn mouse_motion_event(&mut self, ctx: &mut mq::Context, x: f32, y: f32) {
        self.egui_input.mouse_pos = Some(egui::pos2(
            x as f32 / ctx.dpi_scale(),
            y as f32 / ctx.dpi_scale(),
        ));
    }

    pub fn mouse_wheel_event(&mut self, ctx: &mut mq::Context, dx: f32, dy: f32) {
        self.egui_input.scroll_delta += egui::vec2(dx, dy) * ctx.dpi_scale(); // not quite right speed on Mac for some reason
    }

    pub fn mouse_button_down_event(&mut self, _mb: mq::MouseButton, _x: f32, _y: f32) {
        self.egui_input.mouse_down = true;
    }

    pub fn mouse_button_up_event(&mut self, _mb: mq::MouseButton, _x: f32, _y: f32) {
        self.egui_input.mouse_down = false;
    }

    pub fn char_event(&mut self, character: char) {
        input::char_event(&mut self.egui_input, character);
    }

    pub fn key_down_event(&mut self, keycode: mq::KeyCode, keymods: mq::KeyMods) {
        input::key_down_event(&mut self.egui_input, keycode, keymods);
    }

    pub fn key_up_event(&mut self, keycode: mq::KeyCode, keymods: mq::KeyMods) {
        input::key_up_event(&mut self.egui_input, keycode, keymods);
    }
}
