mod input;
mod painter;

// ----------------------------------------------------------------------------

use miniquad as mq;

#[cfg(target_os = "macos")] // https://github.com/not-fl3/miniquad/issues/172
use clipboard::ClipboardProvider;

/// egui bindings for miniquad
pub struct EguiMq {
    egui_ctx: egui::CtxRef,
    egui_input: egui::RawInput,
    painter: painter::Painter,
    #[cfg(target_os = "macos")]
    clipboard: Option<clipboard::ClipboardContext>,
}

impl EguiMq {
    pub fn new(mq_ctx: &mut mq::Context) -> Self {
        Self {
            egui_ctx: egui::CtxRef::default(),
            painter: painter::Painter::new(mq_ctx),
            egui_input: Default::default(),
            #[cfg(target_os = "macos")]
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
            open_url,
            copied_text,
            needs_repaint: _, // miniquad always runs at full framerate
        } = output;

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(url) = open_url {
            if let Err(err) = webbrowser::open(&url) {
                eprintln!("Failed to open url: {}", err);
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = open_url; // unused // TODO: navigate to web page
        }

        if !copied_text.is_empty() {
            self.set_clipboard(mq_ctx, copied_text);
        }

        self.painter
            .paint(mq_ctx, paint_jobs, &self.egui_ctx.texture());
    }

    pub fn mouse_motion_event(&mut self, ctx: &mut mq::Context, x: f32, y: f32) {
        let pos = egui::pos2(x as f32 / ctx.dpi_scale(), y as f32 / ctx.dpi_scale());
        self.egui_input.events.push(egui::Event::PointerMoved(pos))
    }

    pub fn mouse_wheel_event(&mut self, ctx: &mut mq::Context, dx: f32, dy: f32) {
        self.egui_input.scroll_delta += egui::vec2(dx, dy) * ctx.dpi_scale(); // not quite right speed on Mac for some reason
    }

    pub fn mouse_button_down_event(
        &mut self,
        ctx: &mut mq::Context,
        mb: mq::MouseButton,
        x: f32,
        y: f32,
    ) {
        let pos = egui::pos2(x as f32 / ctx.dpi_scale(), y as f32 / ctx.dpi_scale());
        let button = to_egui_button(mb);
        self.egui_input.events.push(egui::Event::PointerButton {
            pos,
            button,
            pressed: true,
            modifiers: self.egui_input.modifiers,
        })
    }

    pub fn mouse_button_up_event(
        &mut self,
        ctx: &mut mq::Context,
        mb: mq::MouseButton,
        x: f32,
        y: f32,
    ) {
        let pos = egui::pos2(x as f32 / ctx.dpi_scale(), y as f32 / ctx.dpi_scale());
        let button = to_egui_button(mb);

        self.egui_input.events.push(egui::Event::PointerButton {
            pos,
            button,
            pressed: false,
            modifiers: self.egui_input.modifiers,
        })
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

    pub fn key_down_event(
        &mut self,
        mq_ctx: &mut mq::Context,
        keycode: mq::KeyCode,
        keymods: mq::KeyMods,
    ) {
        let modifiers = input::egui_modifiers_from_mq_modifiers(keymods);
        self.egui_input.modifiers = modifiers;

        if modifiers.command && keycode == mq::KeyCode::X {
            self.egui_input.events.push(egui::Event::Cut);
        } else if modifiers.command && keycode == mq::KeyCode::C {
            self.egui_input.events.push(egui::Event::Copy);
        } else if modifiers.command && keycode == mq::KeyCode::V {
            if let Some(text) = self.get_clipboard(mq_ctx) {
                self.egui_input.events.push(egui::Event::Text(text));
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

    #[cfg(not(target_os = "macos"))]
    fn set_clipboard(&mut self, mq_ctx: &mut mq::Context, text: String) {
        mq::clipboard::set(mq_ctx, text.as_str());
    }

    #[cfg(not(target_os = "macos"))]
    fn get_clipboard(&mut self, mq_ctx: &mut mq::Context) -> Option<String> {
        mq::clipboard::get(mq_ctx)
    }

    #[cfg(target_os = "macos")]
    fn set_clipboard(&mut self, _mq_ctx: &mut mq::Context, text: String) {
        if let Some(clipboard) = &mut self.clipboard {
            if let Err(err) = clipboard.set_contents(text) {
                eprintln!("Copy/Cut error: {}", err);
            }
        }
    }

    #[cfg(target_os = "macos")]
    fn get_clipboard(&mut self, _mq_ctx: &mut mq::Context) -> Option<String> {
        if let Some(clipboard) = &mut self.clipboard {
            match clipboard.get_contents() {
                Ok(contents) => Some(contents),
                Err(err) => {
                    eprintln!("Paste error: {}", err);
                    None
                }
            }
        } else {
            None
        }
    }
}

#[cfg(target_os = "macos")]
fn init_clipboard() -> Option<clipboard::ClipboardContext> {
    match clipboard::ClipboardContext::new() {
        Ok(clipboard) => Some(clipboard),
        Err(err) => {
            eprintln!("Failed to initialize clipboard: {}", err);
            None
        }
    }
}

fn to_egui_button(mb: mq::MouseButton) -> egui::PointerButton {
    match mb {
        mq::MouseButton::Left => egui::PointerButton::Primary,
        mq::MouseButton::Right => egui::PointerButton::Secondary,
        mq::MouseButton::Middle => egui::PointerButton::Middle,
        mq::MouseButton::Unknown => egui::PointerButton::Primary,
    }
}
