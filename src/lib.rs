//! [egui](https://github.com/emilk/egui) bindings for [miniquad](https://github.com/not-fl3/miniquad).
//!
//! ## Usage
//! Create an instance of [`EguiMq`] and call its event-handler from
//! your `miniquad::EventHandler` implementation.
//!
//! In your `miniquad::EventHandler::draw` method do this:
//!
//! ```
//! use miniquad as mq;
//!
//! struct MyMiniquadApp {
//!     egui_mq: egui_miniquad::EguiMq,
//!     mq_ctx: Box<dyn mq::RenderingBackend>
//! }
//!
//! impl MyMiniquadApp {
//!     fn new() -> Self {
//!        let mut mq_ctx = mq::window::new_rendering_backend();
//!         Self {
//!             egui_mq: egui_miniquad::EguiMq::new(&mut *mq_ctx),
//!             mq_ctx,
//!         }
//!     }
//! }
//!
//! impl mq::EventHandler for MyMiniquadApp {
//!     fn update(&mut self) {}
//!
//!     fn draw(&mut self) {
//!         self.mq_ctx.begin_default_pass(mq::PassAction::clear_color(0.0, 0.0, 0.0, 1.0));
//!         self.mq_ctx.end_render_pass();
//!
//!         self.egui_mq.run(&mut *self.mq_ctx, |_mq_ctx, egui_ctx|{
//!             egui::Window::new("Egui Window").show(egui_ctx, |ui| {
//!                 ui.heading("Hello World!");
//!             });
//!         });
//!
//!         // Draw things behind egui here
//!
//!         self.egui_mq.draw(&mut *self.mq_ctx);
//!
//!         // Draw things in front of egui here
//!
//!         self.mq_ctx.commit_frame();
//!     }
//!
//!     fn mouse_motion_event(&mut self, x: f32, y: f32) {
//!         self.egui_mq.mouse_motion_event(x, y);
//!     }
//!
//!     fn mouse_wheel_event(&mut self, dx: f32, dy: f32) {
//!         self.egui_mq.mouse_wheel_event(dx, dy);
//!     }
//!
//!     fn mouse_button_down_event(
//!         &mut self,
//!         mb: mq::MouseButton,
//!         x: f32,
//!         y: f32,
//!     ) {
//!         self.egui_mq.mouse_button_down_event(mb, x, y);
//!     }
//!
//!     fn mouse_button_up_event(
//!         &mut self,
//!         mb: mq::MouseButton,
//!         x: f32,
//!         y: f32,
//!     ) {
//!         self.egui_mq.mouse_button_up_event(mb, x, y);
//!     }
//!
//!     fn char_event(
//!         &mut self,
//!         character: char,
//!         _keymods: mq::KeyMods,
//!         _repeat: bool,
//!     ) {
//!         self.egui_mq.char_event(character);
//!     }
//!
//!     fn key_down_event(
//!         &mut self,
//!         keycode: mq::KeyCode,
//!         keymods: mq::KeyMods,
//!         _repeat: bool,
//!     ) {
//!         self.egui_mq.key_down_event(keycode, keymods);
//!     }
//!
//!     fn key_up_event(&mut self, keycode: mq::KeyCode, keymods: mq::KeyMods) {
//!         self.egui_mq.key_up_event(keycode, keymods);
//!     }
//! }
//! ```

mod input;
mod painter;

// ----------------------------------------------------------------------------

/// Required by `getrandom` crate.
#[cfg(target_arch = "wasm32")]
fn getrandom(buf: &mut [u8]) -> Result<(), getrandom::Error> {
    // TODO: higher quality random function, e.g. by defining this in JavaScript
    for value in buf {
        *value = quad_rand::rand() as u8;
    }
    Ok(())
}
#[cfg(target_arch = "wasm32")]
getrandom::register_custom_getrandom!(getrandom);

// ----------------------------------------------------------------------------

use egui::CursorIcon;
use miniquad as mq;

pub use painter::CallbackFn;

#[cfg(target_os = "macos")] // https://github.com/not-fl3/miniquad/issues/172
use copypasta::ClipboardProvider;

/// egui bindings for miniquad.
///
///
pub struct EguiMq {
    /// The DPI as reported by miniquad.
    native_dpi_scale: f32,
    /// Pixels per point from egui. Can differ from native DPI because egui allows zooming.
    pixels_per_point: f32,
    egui_ctx: egui::Context,
    egui_input: egui::RawInput,
    painter: painter::Painter,
    #[cfg(target_os = "macos")]
    clipboard: Option<copypasta::ClipboardContext>,
    shapes: Option<Vec<egui::epaint::ClippedShape>>,
    textures_delta: egui::TexturesDelta,
}

impl EguiMq {
    pub fn new(mq_ctx: &mut dyn mq::RenderingBackend) -> Self {
        let native_dpi_scale = miniquad::window::dpi_scale();

        Self {
            native_dpi_scale,
            pixels_per_point: native_dpi_scale,
            egui_ctx: egui::Context::default(),
            painter: painter::Painter::new(mq_ctx),
            egui_input: egui::RawInput::default(),
            #[cfg(target_os = "macos")]
            clipboard: init_clipboard(),
            shapes: None,
            textures_delta: Default::default(),
        }
    }

    /// Use this to open egui windows, panels etc.
    ///
    /// May only be used from inside the callback given to [`Self::run`].
    pub fn egui_ctx(&self) -> &egui::Context {
        &self.egui_ctx
    }

    /// Run the ui code for one frame.
    pub fn run(
        &mut self,
        mq_ctx: &mut dyn mq::RenderingBackend,
        mut run_ui: impl FnMut(&mut dyn mq::RenderingBackend, &egui::Context),
    ) {
        input::on_frame_start(&mut self.egui_input, &self.egui_ctx);

        if self.native_dpi_scale != miniquad::window::dpi_scale() {
            // DPI scale change (maybe new monitor?). Tell egui to change:
            self.native_dpi_scale = miniquad::window::dpi_scale();
            self.egui_input
                .viewports
                .get_mut(&self.egui_input.viewport_id)
                .unwrap()
                .native_pixels_per_point = Some(self.native_dpi_scale);
        }

        let full_output = self
            .egui_ctx
            .run(self.egui_input.take(), |egui_ctx| run_ui(mq_ctx, egui_ctx));

        let egui::FullOutput {
            platform_output,
            textures_delta,
            shapes,
            pixels_per_point,
            viewport_output: _viewport_output, // we only support one viewport
        } = full_output;

        if self.shapes.is_some() {
            eprintln!("Egui contents not drawn. You need to call `draw` after calling `run`");
        }
        self.shapes = Some(shapes);
        self.pixels_per_point = pixels_per_point;
        self.textures_delta.append(textures_delta);

        let egui::PlatformOutput {
            commands,
            cursor_icon,
            events: _,                    // no screen reader
            ime: _,                       // no IME
            mutable_text_under_cursor: _, // no IME
            ..
        } = platform_output;

        for command in commands {
            match command {
                egui::OutputCommand::OpenUrl(open_url) => {
                    quad_url::link_open(&open_url.url, open_url.new_tab);
                }
                egui::OutputCommand::CopyText(copied_text) => {
                    self.set_clipboard(copied_text);
                }
                egui::OutputCommand::CopyImage(_) => (), // No implementation for miniquad
            }
        }

        if cursor_icon == egui::CursorIcon::None {
            miniquad::window::show_mouse(false);
        } else {
            miniquad::window::show_mouse(true);
            let mq_cursor_icon = to_mq_cursor_icon(cursor_icon);
            let mq_cursor_icon = mq_cursor_icon.unwrap_or(mq::CursorIcon::Default);
            miniquad::window::set_mouse_cursor(mq_cursor_icon);
        }
    }

    /// Call this when you need to draw egui.
    /// Must be called after `end_frame`.
    pub fn draw(&mut self, mq_ctx: &mut dyn mq::RenderingBackend) {
        if let Some(shapes) = self.shapes.take() {
            let meshes = self.egui_ctx.tessellate(shapes, self.pixels_per_point);
            self.painter.paint_and_update_textures(
                mq_ctx,
                meshes,
                &self.textures_delta,
                &self.egui_ctx,
            );
            self.textures_delta.clear();
        } else {
            eprintln!("Failed to draw egui. You need to call `end_frame` before calling `draw`");
        }
    }

    /// Call from your [`miniquad::EventHandler`].
    pub fn mouse_motion_event(&mut self, x: f32, y: f32) {
        let pos = egui::pos2(
            x / self.egui_ctx.pixels_per_point(),
            y / self.egui_ctx.pixels_per_point(),
        );
        self.egui_input.events.push(egui::Event::PointerMoved(pos))
    }

    /// Call from your [`miniquad::EventHandler`].
    pub fn mouse_wheel_event(&mut self, dx: f32, dy: f32) {
        let delta = egui::vec2(dx, dy);
        let modifiers = self.egui_input.modifiers;

        self.egui_input.events.push(egui::Event::MouseWheel {
            modifiers,
            unit: egui::MouseWheelUnit::Line,
            delta,
        });
    }

    /// Call from your [`miniquad::EventHandler`].
    pub fn mouse_button_down_event(&mut self, mb: mq::MouseButton, x: f32, y: f32) {
        let pos = egui::pos2(
            x / self.egui_ctx.pixels_per_point(),
            y / self.egui_ctx.pixels_per_point(),
        );
        let button = to_egui_button(mb);
        self.egui_input.events.push(egui::Event::PointerButton {
            pos,
            button,
            pressed: true,
            modifiers: self.egui_input.modifiers,
        })
    }

    /// Call from your [`miniquad::EventHandler`].
    pub fn mouse_button_up_event(&mut self, mb: mq::MouseButton, x: f32, y: f32) {
        let pos = egui::pos2(
            x / self.egui_ctx.pixels_per_point(),
            y / self.egui_ctx.pixels_per_point(),
        );
        let button = to_egui_button(mb);

        self.egui_input.events.push(egui::Event::PointerButton {
            pos,
            button,
            pressed: false,
            modifiers: self.egui_input.modifiers,
        })
    }

    /// Call from your [`miniquad::EventHandler`].
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

    /// Call from your [`miniquad::EventHandler`].
    pub fn key_down_event(&mut self, keycode: mq::KeyCode, keymods: mq::KeyMods) {
        let modifiers = input::egui_modifiers_from_mq_modifiers(keymods);
        self.egui_input.modifiers = modifiers;

        if modifiers.command && keycode == mq::KeyCode::X {
            self.egui_input.events.push(egui::Event::Cut);
        } else if modifiers.command && keycode == mq::KeyCode::C {
            self.egui_input.events.push(egui::Event::Copy);
        } else if modifiers.command && keycode == mq::KeyCode::V {
            if let Some(text) = self.get_clipboard() {
                self.egui_input.events.push(egui::Event::Text(text));
            }
        } else if let Some(key) = input::egui_key_from_mq_key(keycode) {
            self.egui_input.events.push(egui::Event::Key {
                key,
                pressed: true,
                modifiers,
                repeat: false,      // egui will set this for us
                physical_key: None, // unsupported
            })
        }
    }

    /// Call from your [`miniquad::EventHandler`].
    pub fn key_up_event(&mut self, keycode: mq::KeyCode, keymods: mq::KeyMods) {
        let modifiers = input::egui_modifiers_from_mq_modifiers(keymods);
        self.egui_input.modifiers = modifiers;
        if let Some(key) = input::egui_key_from_mq_key(keycode) {
            self.egui_input.events.push(egui::Event::Key {
                key,
                pressed: false,
                modifiers,
                repeat: false,      // egui will set this for us
                physical_key: None, // unsupported
            })
        }
    }

    #[cfg(not(target_os = "macos"))]
    fn set_clipboard(&mut self, text: String) {
        mq::window::clipboard_set(&text);
    }

    #[cfg(not(target_os = "macos"))]
    fn get_clipboard(&mut self) -> Option<String> {
        mq::window::clipboard_get()
    }

    #[cfg(target_os = "macos")]
    fn set_clipboard(&mut self, text: String) {
        if let Some(clipboard) = &mut self.clipboard {
            if let Err(err) = clipboard.set_contents(text) {
                eprintln!("Copy/Cut error: {}", err);
            }
        }
    }

    #[cfg(target_os = "macos")]
    fn get_clipboard(&mut self) -> Option<String> {
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
fn init_clipboard() -> Option<copypasta::ClipboardContext> {
    match copypasta::ClipboardContext::new() {
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

fn to_mq_cursor_icon(cursor_icon: egui::CursorIcon) -> Option<mq::CursorIcon> {
    match cursor_icon {
        // Handled outside this function
        CursorIcon::None => None,

        egui::CursorIcon::Default => Some(mq::CursorIcon::Default),
        egui::CursorIcon::PointingHand => Some(mq::CursorIcon::Pointer),
        egui::CursorIcon::Text => Some(mq::CursorIcon::Text),
        egui::CursorIcon::ResizeHorizontal => Some(mq::CursorIcon::EWResize),
        egui::CursorIcon::ResizeVertical => Some(mq::CursorIcon::NSResize),
        egui::CursorIcon::ResizeNeSw => Some(mq::CursorIcon::NESWResize),
        egui::CursorIcon::ResizeNwSe => Some(mq::CursorIcon::NWSEResize),
        egui::CursorIcon::Help => Some(mq::CursorIcon::Help),
        egui::CursorIcon::Wait => Some(mq::CursorIcon::Wait),
        egui::CursorIcon::Crosshair => Some(mq::CursorIcon::Crosshair),
        egui::CursorIcon::Move => Some(mq::CursorIcon::Move),
        egui::CursorIcon::NotAllowed => Some(mq::CursorIcon::NotAllowed),

        // Similar enough
        egui::CursorIcon::AllScroll => Some(mq::CursorIcon::Move),
        egui::CursorIcon::Progress => Some(mq::CursorIcon::Wait),

        // Not implemented, see https://github.com/not-fl3/miniquad/pull/173 and https://github.com/not-fl3/miniquad/issues/171
        egui::CursorIcon::Grab | egui::CursorIcon::Grabbing => None,

        // Also not implemented:
        egui::CursorIcon::Alias
        | egui::CursorIcon::Cell
        | egui::CursorIcon::ContextMenu
        | egui::CursorIcon::Copy
        | egui::CursorIcon::NoDrop
        | egui::CursorIcon::ResizeColumn
        | egui::CursorIcon::ResizeEast
        | egui::CursorIcon::ResizeNorth
        | egui::CursorIcon::ResizeNorthEast
        | egui::CursorIcon::ResizeNorthWest
        | egui::CursorIcon::ResizeRow
        | egui::CursorIcon::ResizeSouth
        | egui::CursorIcon::ResizeSouthEast
        | egui::CursorIcon::ResizeSouthWest
        | egui::CursorIcon::ResizeWest
        | egui::CursorIcon::VerticalText
        | egui::CursorIcon::ZoomIn
        | egui::CursorIcon::ZoomOut => None,
    }
}
