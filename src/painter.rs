use egui::epaint::Vertex;
use miniquad::{
    Bindings, BlendFactor, BlendState, BlendValue, Buffer, BufferLayout, BufferType, Context,
    Equation, GraphicsContext, Pipeline, PipelineParams, Shader, VertexAttribute, VertexFormat,
};

/// A callback function that can be used to compose an [`egui::PaintCallback`] for custom rendering
/// with [`egui-miniquad`].
///
/// The callback is passed, the [`egui::PaintCallbackInfo`] and the [`GraphicsContext`] which can be used to
/// access the OpenGL context.
///
/// # Example
///
/// See the [`custom3d_glow`](https://github.com/emilk/egui/blob/master/crates/egui_demo_app/src/apps/custom3d_wgpu.rs) demo source for a detailed usage example.
pub struct CallbackFn {
    #[allow(clippy::type_complexity)]
    f: Box<dyn Fn(egui::PaintCallbackInfo, &mut GraphicsContext) + Sync + Send>,
}

impl CallbackFn {
    pub fn new(
        callback: impl Fn(egui::PaintCallbackInfo, &mut GraphicsContext) + Sync + Send + 'static,
    ) -> Self {
        let f = Box::new(callback);
        CallbackFn { f }
    }
}

pub struct Painter {
    pipeline: Pipeline,
    bindings: Bindings,
    textures: std::collections::HashMap<egui::TextureId, miniquad::Texture>,
}

impl Painter {
    pub fn new(ctx: &mut Context) -> Painter {
        let shader = Shader::new(ctx, shader::VERTEX, shader::FRAGMENT, shader::meta());

        let pipeline = Pipeline::with_params(
            ctx,
            &[BufferLayout::default()],
            &[
                VertexAttribute::new("a_pos", VertexFormat::Float2),
                VertexAttribute::new("a_tc", VertexFormat::Float2),
                VertexAttribute::new("a_srgba", VertexFormat::Byte4),
            ],
            shader.expect("couldn't make shader"),
            PipelineParams {
                color_blend: Some(BlendState::new(
                    Equation::Add,
                    BlendFactor::One,
                    BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                )),
                cull_face: miniquad::CullFace::Nothing,
                ..Default::default()
            },
        );

        let vertex_buffer = Buffer::stream(
            ctx,
            BufferType::VertexBuffer,
            32 * 1024 * std::mem::size_of::<Vertex>(),
        );
        let index_buffer = Buffer::stream(
            ctx,
            BufferType::IndexBuffer,
            32 * 1024 * std::mem::size_of::<u16>(),
        );

        let bindings = Bindings {
            vertex_buffers: vec![vertex_buffer],
            index_buffer,
            images: vec![miniquad::Texture::empty()],
        };

        Painter {
            pipeline,
            bindings,
            textures: Default::default(),
        }
    }

    pub fn set_texture(
        &mut self,
        ctx: &mut Context,
        tex_id: egui::TextureId,
        delta: &egui::epaint::ImageDelta,
    ) {
        let [w, h] = delta.image.size();

        if let Some([x, y]) = delta.pos {
            // Partial update
            if let Some(texture) = self.textures.get(&tex_id) {
                match &delta.image {
                    egui::ImageData::Color(image) => {
                        assert_eq!(
                            image.width() * image.height(),
                            image.pixels.len(),
                            "Mismatch between texture size and texel count"
                        );
                        let data: &[u8] = bytemuck::cast_slice(image.pixels.as_ref());
                        texture.update_texture_part(ctx, x as _, y as _, w as _, h as _, data);
                    }
                    egui::ImageData::Font(image) => {
                        assert_eq!(
                            image.width() * image.height(),
                            image.pixels.len(),
                            "Mismatch between texture size and texel count"
                        );

                        let data: Vec<u8> = image
                            .srgba_pixels(None)
                            .flat_map(|a| a.to_array())
                            .collect();

                        texture.update_texture_part(ctx, x as _, y as _, w as _, h as _, &data);
                    }
                }
            } else {
                eprintln!("Failed to find egui texture {:?}", tex_id);
            }
        } else {
            // New texture (or full update).
            let params = miniquad::TextureParams {
                format: miniquad::TextureFormat::RGBA8,
                wrap: miniquad::TextureWrap::Clamp,
                filter: miniquad::FilterMode::Linear,
                width: w as _,
                height: h as _,
            };

            let texture = match &delta.image {
                egui::ImageData::Color(image) => {
                    assert_eq!(
                        image.width() * image.height(),
                        image.pixels.len(),
                        "Mismatch between texture size and texel count"
                    );
                    let data: &[u8] = bytemuck::cast_slice(image.pixels.as_ref());
                    miniquad::Texture::from_data_and_format(ctx, data, params)
                }
                egui::ImageData::Font(image) => {
                    assert_eq!(
                        image.width() * image.height(),
                        image.pixels.len(),
                        "Mismatch between texture size and texel count"
                    );

                    let data: Vec<u8> = image
                        .srgba_pixels(None)
                        .flat_map(|a| a.to_array())
                        .collect();

                    miniquad::Texture::from_data_and_format(ctx, &data, params)
                }
            };

            let previous = self.textures.insert(tex_id, texture);
            if let Some(previous) = previous {
                previous.delete();
            }
        }
    }

    pub fn free_texture(&mut self, tex_id: egui::TextureId) {
        if let Some(old_tex) = self.textures.remove(&tex_id) {
            old_tex.delete();
        }
    }

    pub fn paint_and_update_textures(
        &mut self,
        ctx: &mut Context,
        primtives: Vec<egui::ClippedPrimitive>,
        textures_delta: &egui::TexturesDelta,
        egui_ctx: &egui::Context,
    ) {
        for (id, image_delta) in &textures_delta.set {
            self.set_texture(ctx, *id, image_delta);
        }

        self.paint(ctx, primtives, egui_ctx);

        for &id in &textures_delta.free {
            self.free_texture(id);
        }
    }

    pub fn paint(
        &mut self,
        ctx: &mut Context,
        primtives: Vec<egui::ClippedPrimitive>,
        egui_ctx: &egui::Context,
    ) {
        ctx.begin_default_pass(miniquad::PassAction::Nothing);
        ctx.apply_pipeline(&self.pipeline);

        let screen_size_in_pixels = ctx.screen_size();
        let screen_size_in_points = (
            screen_size_in_pixels.0 / egui_ctx.pixels_per_point(),
            screen_size_in_pixels.1 / egui_ctx.pixels_per_point(),
        );
        ctx.apply_uniforms(&shader::Uniforms {
            u_screen_size: screen_size_in_points,
        });

        for egui::ClippedPrimitive {
            clip_rect,
            primitive,
        } in primtives
        {
            match primitive {
                egui::epaint::Primitive::Mesh(mesh) => {
                    self.paint_job(ctx, clip_rect, mesh, egui_ctx);
                }
                egui::epaint::Primitive::Callback(callback) => {
                    let info = egui::PaintCallbackInfo {
                        viewport: callback.rect,
                        clip_rect,
                        pixels_per_point: egui_ctx.pixels_per_point(),
                        screen_size_px: [
                            screen_size_in_pixels.0.round() as _,
                            screen_size_in_pixels.1.round() as _,
                        ],
                    };

                    if let Some(callback) = callback.callback.downcast_ref::<CallbackFn>() {
                        (callback.f)(info, ctx);
                    } else {
                        eprintln!(
                            "Warning: Unsupported render callback. Expected egui_miniquad::CallbackFn"
                        );
                    }
                }
            }
        }

        ctx.end_render_pass();
    }

    pub fn paint_job(
        &mut self,
        ctx: &mut Context,
        clip_rect: egui::Rect,
        mesh: egui::epaint::Mesh,
        egui_ctx: &egui::Context,
    ) {
        let screen_size_in_pixels = ctx.screen_size();
        let pixels_per_point = egui_ctx.pixels_per_point();

        // TODO: support u32 indices in miniquad and just use "mesh.indices" without a need for `split_to_u16`
        let meshes = mesh.split_to_u16();
        for mesh in meshes {
            assert!(mesh.is_valid());
            let vertices_size_bytes = mesh.vertices.len() * std::mem::size_of::<Vertex>();
            if self.bindings.vertex_buffers[0].size() < vertices_size_bytes {
                self.bindings.vertex_buffers[0].delete();
                self.bindings.vertex_buffers[0] =
                    Buffer::stream(ctx, BufferType::VertexBuffer, vertices_size_bytes);
            }
            self.bindings.vertex_buffers[0].update(ctx, &mesh.vertices);

            let indices_size_bytes = mesh.indices.len() * std::mem::size_of::<u16>();
            if self.bindings.index_buffer.size() < indices_size_bytes {
                self.bindings.index_buffer.delete();
                self.bindings.index_buffer =
                    Buffer::stream(ctx, BufferType::IndexBuffer, indices_size_bytes);
            }
            self.bindings.index_buffer.update(ctx, &mesh.indices);

            self.bindings.images[0] = match mesh.texture_id {
                egui::TextureId::Managed(id) => {
                    if let Some(tex) = self.textures.get(&mesh.texture_id) {
                        *tex
                    } else {
                        eprintln!("Texture {:?} not found", id);
                        continue;
                    }
                }
                egui::TextureId::User(id) => unsafe { miniquad::Texture::from_raw_id(id as u32) },
            };

            let (width_in_pixels, height_in_pixels) = screen_size_in_pixels;

            // From https://github.com/emilk/egui/blob/master/egui_glium/src/painter.rs#L233

            // Transform clip rect to physical pixels:
            let clip_min_x = pixels_per_point * clip_rect.min.x;
            let clip_min_y = pixels_per_point * clip_rect.min.y;
            let clip_max_x = pixels_per_point * clip_rect.max.x;
            let clip_max_y = pixels_per_point * clip_rect.max.y;

            // Make sure clip rect can fit withing an `u32`:
            let clip_min_x = clip_min_x.clamp(0.0, width_in_pixels as f32);
            let clip_min_y = clip_min_y.clamp(0.0, height_in_pixels as f32);
            let clip_max_x = clip_max_x.clamp(clip_min_x, width_in_pixels as f32);
            let clip_max_y = clip_max_y.clamp(clip_min_y, height_in_pixels as f32);

            let clip_min_x = clip_min_x.round() as u32;
            let clip_min_y = clip_min_y.round() as u32;
            let clip_max_x = clip_max_x.round() as u32;
            let clip_max_y = clip_max_y.round() as u32;

            ctx.apply_scissor_rect(
                clip_min_x as i32,
                (height_in_pixels as u32 - clip_max_y) as i32,
                (clip_max_x - clip_min_x) as i32,
                (clip_max_y - clip_min_y) as i32,
            );
            ctx.apply_bindings(&self.bindings);
            ctx.draw(0, mesh.indices.len() as i32, 1);
        }
    }
}

mod shader {
    use miniquad::{ShaderMeta, UniformBlockLayout, UniformDesc, UniformType};

    pub const VERTEX: &str = r#"
    #version 100
    uniform vec2 u_screen_size;

    attribute vec2 a_pos;
    attribute vec2 a_tc;
    attribute vec4 a_srgba;

    varying vec2 v_tc;
    varying vec4 v_rgba_in_gamma;

    void main() {
        gl_Position = vec4(
            2.0 * a_pos.x / u_screen_size.x - 1.0,
            1.0 - 2.0 * a_pos.y / u_screen_size.y,
            0.0,
            1.0);
            v_rgba_in_gamma = a_srgba / 255.0;
            v_tc = a_tc;
    }
    "#;

    pub const FRAGMENT: &str = r#"
    #version 100
    uniform sampler2D u_sampler;
    precision highp float;

    varying vec2 v_tc;
    varying vec4 v_rgba_in_gamma;

    void main() {
        vec4 texture_in_gamma = texture2D(u_sampler, v_tc);
        gl_FragColor = v_rgba_in_gamma * texture_in_gamma;
    }
    "#;

    pub fn meta() -> ShaderMeta {
        ShaderMeta {
            images: vec!["u_sampler".to_string()],
            uniforms: UniformBlockLayout {
                uniforms: vec![UniformDesc::new("u_screen_size", UniformType::Float2)],
            },
        }
    }

    #[repr(C)]
    #[derive(Debug)]
    pub struct Uniforms {
        pub u_screen_size: (f32, f32),
    }
}
