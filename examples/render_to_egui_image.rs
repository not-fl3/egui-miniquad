use egui::load::SizedTexture;
use glam::{vec3, EulerRot, Mat4};
use {egui_miniquad as egui_mq, miniquad as mq};

struct Stage {
    egui_mq: egui_mq::EguiMq,
    offscreen_pipeline: mq::Pipeline,
    offscreen_bind: mq::Bindings,
    offscreen_pass: mq::RenderPass,
    rx: f32,
    ry: f32,
    mq_ctx: Box<dyn mq::RenderingBackend>,
}

impl Stage {
    pub fn new() -> Stage {
        let mut mq_ctx = mq::window::new_rendering_backend();

        let color_img = mq_ctx.new_render_texture(mq::TextureParams {
            width: 256,
            height: 256,
            format: mq::TextureFormat::RGBA8,
            ..Default::default()
        });
        let depth_img = mq_ctx.new_render_texture(mq::TextureParams {
            width: 256,
            height: 256,
            format: mq::TextureFormat::Depth,
            ..Default::default()
        });

        let offscreen_pass = mq_ctx.new_render_pass(color_img, Some(depth_img));

        #[rustfmt::skip]
        let vertices: &[f32] = &[
            /* pos               color                   uvs */
            -1.0, -1.0, -1.0,    1.0, 0.5, 0.5, 1.0,     0.0, 0.0,
             1.0, -1.0, -1.0,    1.0, 0.5, 0.5, 1.0,     1.0, 0.0,
             1.0,  1.0, -1.0,    1.0, 0.5, 0.5, 1.0,     1.0, 1.0,
            -1.0,  1.0, -1.0,    1.0, 0.5, 0.5, 1.0,     0.0, 1.0,

            -1.0, -1.0,  1.0,    0.5, 1.0, 0.5, 1.0,     0.0, 0.0,
             1.0, -1.0,  1.0,    0.5, 1.0, 0.5, 1.0,     1.0, 0.0,
             1.0,  1.0,  1.0,    0.5, 1.0, 0.5, 1.0,     1.0, 1.0,
            -1.0,  1.0,  1.0,    0.5, 1.0, 0.5, 1.0,     0.0, 1.0,

            -1.0, -1.0, -1.0,    0.5, 0.5, 1.0, 1.0,     0.0, 0.0,
            -1.0,  1.0, -1.0,    0.5, 0.5, 1.0, 1.0,     1.0, 0.0,
            -1.0,  1.0,  1.0,    0.5, 0.5, 1.0, 1.0,     1.0, 1.0,
            -1.0, -1.0,  1.0,    0.5, 0.5, 1.0, 1.0,     0.0, 1.0,

             1.0, -1.0, -1.0,    1.0, 0.5, 0.0, 1.0,     0.0, 0.0,
             1.0,  1.0, -1.0,    1.0, 0.5, 0.0, 1.0,     1.0, 0.0,
             1.0,  1.0,  1.0,    1.0, 0.5, 0.0, 1.0,     1.0, 1.0,
             1.0, -1.0,  1.0,    1.0, 0.5, 0.0, 1.0,     0.0, 1.0,

            -1.0, -1.0, -1.0,    0.0, 0.5, 1.0, 1.0,     0.0, 0.0,
            -1.0, -1.0,  1.0,    0.0, 0.5, 1.0, 1.0,     1.0, 0.0,
             1.0, -1.0,  1.0,    0.0, 0.5, 1.0, 1.0,     1.0, 1.0,
             1.0, -1.0, -1.0,    0.0, 0.5, 1.0, 1.0,     0.0, 1.0,

            -1.0,  1.0, -1.0,    1.0, 0.0, 0.5, 1.0,     0.0, 0.0,
            -1.0,  1.0,  1.0,    1.0, 0.0, 0.5, 1.0,     1.0, 0.0,
             1.0,  1.0,  1.0,    1.0, 0.0, 0.5, 1.0,     1.0, 1.0,
             1.0,  1.0, -1.0,    1.0, 0.0, 0.5, 1.0,     0.0, 1.0
        ];

        let vertex_buffer = mq_ctx.new_buffer(
            mq::BufferType::VertexBuffer,
            mq::BufferUsage::Immutable,
            mq::BufferSource::slice(vertices),
        );

        #[rustfmt::skip]
        let indices: &[u16] = &[
            0, 1, 2,  0, 2, 3,
            6, 5, 4,  7, 6, 4,
            8, 9, 10,  8, 10, 11,
            14, 13, 12,  15, 14, 12,
            16, 17, 18,  16, 18, 19,
            22, 21, 20,  23, 22, 20
        ];

        let index_buffer = mq_ctx.new_buffer(
            mq::BufferType::IndexBuffer,
            mq::BufferUsage::Immutable,
            mq::BufferSource::slice(indices),
        );

        let offscreen_bind = mq::Bindings {
            vertex_buffers: vec![vertex_buffer],
            index_buffer,
            images: vec![],
        };

        let offscreen_shader = mq_ctx
            .new_shader(
                mq::ShaderSource::Glsl {
                    vertex: offscreen_shader::VERTEX,
                    fragment: offscreen_shader::FRAGMENT,
                },
                offscreen_shader::meta(),
            )
            .unwrap();

        let offscreen_pipeline = mq_ctx.new_pipeline(
            &[mq::BufferLayout {
                stride: 36,
                ..Default::default()
            }],
            &[
                mq::VertexAttribute::new("pos", mq::VertexFormat::Float3),
                mq::VertexAttribute::new("color0", mq::VertexFormat::Float4),
            ],
            offscreen_shader,
            mq::PipelineParams {
                depth_test: mq::Comparison::LessOrEqual,
                depth_write: true,
                ..Default::default()
            },
        );

        Stage {
            egui_mq: egui_mq::EguiMq::new(&mut *mq_ctx),
            offscreen_pipeline,
            offscreen_bind,
            offscreen_pass,
            rx: 0.,
            ry: 0.,
            mq_ctx,
        }
    }
}

impl mq::EventHandler for Stage {
    fn update(&mut self) {}

    fn draw(&mut self) {
        let (width, height) = mq::window::screen_size();
        let proj = Mat4::perspective_rh_gl(60.0f32.to_radians(), width / height, 0.01, 10.0);
        let view = Mat4::look_at_rh(
            vec3(0.0, 1.5, 3.0),
            vec3(0.0, 0.0, 0.0),
            vec3(0.0, 1.0, 0.0),
        );
        let view_proj = proj * view;

        self.rx += 0.01;
        self.ry += 0.03;
        let model = Mat4::from_euler(EulerRot::YXZ, self.rx, self.ry, 0.);

        let vs_params = offscreen_shader::Uniforms {
            mvp: view_proj * model,
        };

        // the offscreen pass, rendering an rotating, untextured cube into a render target image
        self.mq_ctx.begin_pass(
            Some(self.offscreen_pass),
            mq::PassAction::clear_color(1.0, 1.0, 1.0, 1.),
        );
        self.mq_ctx.apply_pipeline(&self.offscreen_pipeline);
        self.mq_ctx.apply_bindings(&self.offscreen_bind);
        self.mq_ctx
            .apply_uniforms(mq::UniformsSource::table(&vs_params));
        self.mq_ctx.draw(0, 36, 1);
        self.mq_ctx.end_render_pass();

        // Extract texture from offscreen render pass
        let mq_texture = self.mq_ctx.render_pass_texture(self.offscreen_pass);

        // create egui TextureId from Miniquad GL texture Id
        let raw_id = match unsafe { self.mq_ctx.texture_raw_id(mq_texture) } {
            mq::RawId::OpenGl(id) => id as u64,
        };
        let egui_texture_id = egui::TextureId::User(raw_id);

        self.mq_ctx
            .begin_default_pass(mq::PassAction::clear_color(0.0, 0.0, 0.0, 1.0));
        self.mq_ctx.end_render_pass();

        // Run the UI code:
        self.egui_mq.run(&mut *self.mq_ctx, |_mq_ctx, egui_ctx| {
            egui::Window::new("egui ❤ miniquad").show(egui_ctx, |ui| {
                let img =
                    egui::Image::from_texture(SizedTexture::new(egui_texture_id, [140.0, 140.0]));
                ui.add(img);
                #[cfg(not(target_arch = "wasm32"))]
                {
                    if ui.button("Quit").clicked() {
                        std::process::exit(0);
                    }
                }
            });
        });

        // Draw things behind egui here

        self.egui_mq.draw(&mut *self.mq_ctx);

        // Draw things in front of egui here

        self.mq_ctx.commit_frame();
    }

    fn mouse_motion_event(&mut self, x: f32, y: f32) {
        self.egui_mq.mouse_motion_event(x, y);
    }

    fn mouse_wheel_event(&mut self, dx: f32, dy: f32) {
        self.egui_mq.mouse_wheel_event(dx, dy);
    }

    fn mouse_button_down_event(&mut self, mb: mq::MouseButton, x: f32, y: f32) {
        self.egui_mq.mouse_button_down_event(mb, x, y);
    }

    fn mouse_button_up_event(&mut self, mb: mq::MouseButton, x: f32, y: f32) {
        self.egui_mq.mouse_button_up_event(mb, x, y);
    }

    fn char_event(&mut self, character: char, _keymods: mq::KeyMods, _repeat: bool) {
        self.egui_mq.char_event(character);
    }

    fn key_down_event(&mut self, keycode: mq::KeyCode, keymods: mq::KeyMods, _repeat: bool) {
        self.egui_mq.key_down_event(keycode, keymods);
    }

    fn key_up_event(&mut self, keycode: mq::KeyCode, keymods: mq::KeyMods) {
        self.egui_mq.key_up_event(keycode, keymods);
    }
}

fn main() {
    let conf = mq::conf::Conf {
        high_dpi: true,
        ..Default::default()
    };
    mq::start(conf, || Box::new(Stage::new()));
}

mod offscreen_shader {
    use miniquad as mq;

    pub const VERTEX: &str = r#"#version 100
    attribute vec4 pos;
    attribute vec4 color0;

    varying lowp vec4 color;

    uniform mat4 mvp;

    void main() {
        gl_Position = mvp * pos;
        color = color0;
    }
    "#;

    pub const FRAGMENT: &str = r#"#version 100
    varying lowp vec4 color;

    void main() {
        gl_FragColor = color;
    }
    "#;

    pub fn meta() -> mq::ShaderMeta {
        mq::ShaderMeta {
            images: vec![],
            uniforms: mq::UniformBlockLayout {
                uniforms: vec![mq::UniformDesc::new("mvp", mq::UniformType::Mat4)],
            },
        }
    }

    #[repr(C)]
    pub struct Uniforms {
        pub mvp: glam::Mat4,
    }
}
