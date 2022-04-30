use egui::epaint::Vertex;
use miniquad::{
    Bindings, BlendFactor, BlendState, BlendValue, Buffer, BufferLayout, BufferType, Context,
    Equation, Pipeline, PipelineParams, Shader, VertexAttribute, VertexFormat,
};

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

                        let gamma = 1.0;
                        let data: Vec<u8> = image
                            .srgba_pixels(gamma)
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

                    let gamma = 1.0;
                    let data: Vec<u8> = image
                        .srgba_pixels(gamma)
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
    ) {
        for (id, image_delta) in &textures_delta.set {
            self.set_texture(ctx, *id, image_delta);
        }

        self.paint(ctx, primtives);

        for &id in &textures_delta.free {
            self.free_texture(id);
        }
    }

    pub fn paint(&mut self, ctx: &mut Context, primtives: Vec<egui::ClippedPrimitive>) {
        ctx.begin_default_pass(miniquad::PassAction::Nothing);
        ctx.apply_pipeline(&self.pipeline);

        let screen_size_in_pixels = ctx.screen_size();
        let screen_size_in_points = (
            screen_size_in_pixels.0 / ctx.dpi_scale(),
            screen_size_in_pixels.1 / ctx.dpi_scale(),
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
                    self.paint_job(ctx, clip_rect, mesh);
                }
                egui::epaint::Primitive::Callback(callback) => {
                    let info = egui::PaintCallbackInfo {
                        viewport: callback.rect,
                        clip_rect,
                        pixels_per_point: ctx.dpi_scale(),
                        screen_size_px: [
                            screen_size_in_pixels.0.round() as _,
                            screen_size_in_pixels.1.round() as _,
                        ],
                    };
                    callback.call(&info, ctx)
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
    ) {
        let screen_size_in_pixels = ctx.screen_size();
        let pixels_per_point = ctx.dpi_scale();

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
    varying vec4 v_rgba;

    // 0-1 linear  from  0-255 sRGB
    vec3 linear_from_srgb(vec3 srgb) {
        bvec3 cutoff = lessThan(srgb, vec3(10.31475));
        vec3 lower = srgb / vec3(3294.6);
        vec3 higher = pow((srgb + vec3(14.025)) / vec3(269.025), vec3(2.4));
        return mix(higher, lower, vec3(cutoff));
    }

    // 0-1 linear  from  0-255 sRGBA
    vec4 linear_from_srgba(vec4 srgba) {
        return vec4(linear_from_srgb(srgba.rgb), srgba.a / 255.0);
    }

    // 0-255 sRGB  from  0-1 linear
    vec3 srgb_from_linear(vec3 rgb) {
        bvec3 cutoff = lessThan(rgb, vec3(0.0031308));
        vec3 lower = rgb * vec3(3294.6);
        vec3 higher = vec3(269.025) * pow(rgb, vec3(1.0 / 2.4)) - vec3(14.025);
        return mix(higher, lower, vec3(cutoff));
    }

    // 0-255 sRGBA  from  0-1 linear
    vec4 srgba_from_linear(vec4 rgba) {
        return vec4(srgb_from_linear(rgba.rgb), 255.0 * rgba.a);
    }

    void main() {
        gl_Position = vec4(
            2.0 * a_pos.x / u_screen_size.x - 1.0,
            1.0 - 2.0 * a_pos.y / u_screen_size.y,
            0.0,
            1.0);

        v_tc = a_tc;
        v_rgba = a_srgba / 255.0;
        v_rgba.a = pow(v_rgba.a, 1.6);
    }
    "#;

    pub const FRAGMENT: &str = r#"
    #version 100
    uniform sampler2D u_sampler;
    precision highp float;

    varying vec2 v_tc;
    varying vec4 v_rgba;

    // 0-1 linear  from  0-255 sRGB
    vec3 linear_from_srgb(vec3 srgb) {
        bvec3 cutoff = lessThan(srgb, vec3(10.31475));
        vec3 lower = srgb / vec3(3294.6);
        vec3 higher = pow((srgb + vec3(14.025)) / vec3(269.025), vec3(2.4));
        return mix(higher, lower, vec3(cutoff));
    }

    // 0-1 linear  from  0-255 sRGBA
    vec4 linear_from_srgba(vec4 srgba) {
        return vec4(linear_from_srgb(srgba.rgb), srgba.a / 255.0);
    }

    // 0-255 sRGB  from  0-1 linear
    vec3 srgb_from_linear(vec3 rgb) {
        bvec3 cutoff = lessThan(rgb, vec3(0.0031308));
        vec3 lower = rgb * vec3(3294.6);
        vec3 higher = vec3(269.025) * pow(rgb, vec3(1.0 / 2.4)) - vec3(14.025);
        return mix(higher, lower, vec3(cutoff));
    }

    // 0-255 sRGBA  from  0-1 linear
    vec4 srgba_from_linear(vec4 rgba) {
        return vec4(srgb_from_linear(rgba.rgb), 255.0 * rgba.a);
    }

    void main() {
        // We must decode the colors, since WebGL1 doesn't come with sRGBA textures:
        vec4 texture_rgba = linear_from_srgba(texture2D(u_sampler, v_tc) * 255.0);

        // WebGL1 doesn't support linear blending in the framebuffer,
        // so we do a hack here where we change the premultiplied alpha
        // to do the multiplication in gamma space instead:

        // Unmultiply alpha:
        if (texture_rgba.a > 0.0) {
            texture_rgba.rgb /= texture_rgba.a;
        }

        // Empiric tweak to make e.g. shadows look more like they should:
        texture_rgba.a *= sqrt(texture_rgba.a);

        // To gamma:
        texture_rgba = srgba_from_linear(texture_rgba) / 255.0;

        // Premultiply alpha, this time in gamma space:
        if (texture_rgba.a > 0.0) {
            texture_rgba.rgb *= texture_rgba.a;
        }

        /// Multiply vertex color with texture color (in linear space).
        gl_FragColor = v_rgba * texture_rgba;
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
