mod input;
mod painter;

// ----------------------------------------------------------------------------

use miniquad as mq;

use clipboard::ClipboardProvider;

/// egui bindings for miniquad
pub struct EguiMq {
    egui_ctx: egui::CtxRef,
    egui_input: egui::RawInput,
    painter: painter::Painter,
    clipboard: Option<clipboard::ClipboardContext>,
}

impl EguiMq {
    pub fn new(mq_ctx: &mut mq::Context) -> Self {
        Self {
            egui_ctx: egui::CtxRef::default(),
            painter: painter::Painter::new(mq_ctx),
            egui_input: Default::default(),
            clipboard: init_clipboard(),
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
        let (output, shapes) = self.egui_ctx.end_frame();
        let paint_jobs = self.egui_ctx.tessellate(shapes);

        let egui::Output {
            cursor_icon: _, // https://github.com/not-fl3/miniquad/issues/171
            open_url: _,    // TODO: open hyperlinks
            copied_text,
            needs_repaint: _, // miniquad always runs at full framerate
        } = output;

        if !copied_text.is_empty() {
            if let Some(clipboard) = &mut self.clipboard {
                if let Err(err) = clipboard.set_contents(copied_text) {
                    eprintln!("Copy/Cut error: {}", err);
                }
            }
        }

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

    pub fn char_event(&mut self, chr: char) {
        if input::is_printable_char(chr)
            && !self.egui_input.modifiers.ctrl
            && !self.egui_input.modifiers.mac_cmd
        {
            self.egui_input
                .events
                .push(egui::Event::Text(chr.to_string()));
        }
    }

    pub fn key_down_event(&mut self, keycode: mq::KeyCode, keymods: mq::KeyMods) {
        let modifiers = input::egui_modifiers_from_mq_modifiers(keymods);
        self.egui_input.modifiers = modifiers;

        if modifiers.command && keycode == mq::KeyCode::X {
            self.egui_input.events.push(egui::Event::Cut);
        } else if modifiers.command && keycode == mq::KeyCode::C {
            self.egui_input.events.push(egui::Event::Copy);
        } else if modifiers.command && keycode == mq::KeyCode::V {
            if let Some(clipboard) = &mut self.clipboard {
                match clipboard.get_contents() {
                    Ok(contents) => {
                        self.egui_input.events.push(egui::Event::Text(contents));
                    }
                    Err(err) => {
                        eprintln!("Paste error: {}", err);
                    }
                }
            }
        } else if let Some(key) = input::egui_key_from_mq_key(keycode) {
            self.egui_input.events.push(egui::Event::Key {
                key,
                pressed: true,
                modifiers,
            })
        }
    }

    pub fn key_up_event(&mut self, keycode: mq::KeyCode, keymods: mq::KeyMods) {
        let modifiers = input::egui_modifiers_from_mq_modifiers(keymods);
        self.egui_input.modifiers = modifiers;
        if let Some(key) = input::egui_key_from_mq_key(keycode) {
            self.egui_input.events.push(egui::Event::Key {
                key,
                pressed: false,
                modifiers,
            })
        }
    }
}

fn init_clipboard() -> Option<clipboard::ClipboardContext> {
    match clipboard::ClipboardContext::new() {
        Ok(clipboard) => Some(clipboard),
        Err(err) => {
            eprintln!("Failed to initialize clipboard: {}", err);
            None
        }
    }
}
