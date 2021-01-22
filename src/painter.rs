use egui::{
    math::clamp,
    paint::{PaintJob, PaintJobs, Vertex},
    Texture,
};

use miniquad::{
    Bindings, BlendFactor, BlendState, BlendValue, Buffer, BufferLayout, BufferType, Context,
    Equation, Pipeline, PipelineParams, Shader, VertexAttribute, VertexFormat,
};

pub struct Painter {
    pipeline: Pipeline,
    bindings: Bindings,
    vertex_buffer_size: usize,
    index_buffer_size: usize,
    texture_hash: u64,
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
                VertexAttribute::new("a_color", VertexFormat::Byte4),
            ],
            shader.expect("couldn't make shader"),
            PipelineParams {
                color_blend: Some(BlendState::new(
                    Equation::Add,
                    BlendFactor::One,
                    BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                )),
                ..Default::default()
            },
        );

        let vertex_buffer_size = 10 * 1024;
        let vertex_buffer = Buffer::stream(
            ctx,
            BufferType::VertexBuffer,
            vertex_buffer_size * std::mem::size_of::<Vertex>(),
        );
        let index_buffer_size = 10 * 1024;
        let index_buffer = Buffer::stream(
            ctx,
            BufferType::IndexBuffer,
            index_buffer_size * std::mem::size_of::<u16>(),
        );
        let bindings = Bindings {
            vertex_buffers: vec![vertex_buffer],
            index_buffer,
            images: vec![miniquad::Texture::empty()],
        };

        Painter {
            pipeline,
            bindings,
            vertex_buffer_size,
            index_buffer_size,
            texture_hash: 0,
        }
    }

    fn rebuild_texture(&mut self, ctx: &mut Context, texture: &Texture) {
        self.texture_hash = texture.version;

        self.bindings.images[0].delete();

        let mut texture_data = Vec::new();
        for pixel in texture.srgba_pixels() {
            texture_data.push(pixel.r());
            texture_data.push(pixel.g());
            texture_data.push(pixel.b());
            texture_data.push(pixel.a());
        }
        assert_eq!(texture_data.len(), texture.width * texture.height * 4);
        self.bindings.images[0] = miniquad::Texture::from_data_and_format(
            ctx,
            &texture_data,
            miniquad::TextureParams {
                format: miniquad::TextureFormat::RGBA8,
                wrap: miniquad::TextureWrap::Clamp,
                filter: miniquad::FilterMode::Linear,
                width: texture.width as _,
                height: texture.height as _,
            },
        );
    }

    pub fn paint(&mut self, ctx: &mut Context, jobs: PaintJobs, texture: &Texture) {
        if texture.version != self.texture_hash {
            self.rebuild_texture(ctx, texture);
        }

        ctx.begin_default_pass(miniquad::PassAction::Nothing);

        for paint_job in jobs {
            self.paint_job(ctx, paint_job);
        }

        ctx.end_render_pass();
        ctx.commit_frame();
    }

    pub fn paint_job(&mut self, ctx: &mut Context, (clip_rect, mesh): PaintJob) {
        if self.vertex_buffer_size < mesh.vertices.len() {
            self.vertex_buffer_size = mesh.vertices.len();
            self.bindings.vertex_buffers[0].delete();
            self.bindings.vertex_buffers[0] = Buffer::stream(
                ctx,
                BufferType::VertexBuffer,
                self.vertex_buffer_size * std::mem::size_of::<Vertex>(),
            );
        }
        if self.index_buffer_size < mesh.indices.len() {
            self.index_buffer_size = mesh.indices.len();
            self.bindings.index_buffer.delete();
            self.bindings.index_buffer = Buffer::stream(
                ctx,
                BufferType::IndexBuffer,
                self.index_buffer_size * std::mem::size_of::<u16>(),
            );
        }

        self.bindings.vertex_buffers[0].update(ctx, &mesh.vertices);

        let screen_size = ctx.screen_size();

        // TODO: support u32 indices in miniquad and just use "mesh.indices"
        for mesh in mesh.split_to_u16() {
            let indices = mesh.indices.iter().map(|x| *x as u16).collect::<Vec<u16>>();
            self.bindings.index_buffer.update(ctx, &indices);

            ctx.apply_pipeline(&self.pipeline);

            let (width_pixels, height_pixels) = screen_size;
            // https://github.com/emilk/egui/blob/master/egui_glium/src/painter.rs#L276
            let pixels_per_point = ctx.dpi_scale();
            let clip_min_x = pixels_per_point * clip_rect.min.x;
            let clip_min_y = pixels_per_point * clip_rect.min.y;
            let clip_max_x = pixels_per_point * clip_rect.max.x;
            let clip_max_y = pixels_per_point * clip_rect.max.y;
            let clip_min_x = clamp(clip_min_x, 0.0..=width_pixels as f32);
            let clip_min_y = clamp(clip_min_y, 0.0..=height_pixels as f32);
            let clip_max_x = clamp(clip_max_x, clip_min_x..=width_pixels as f32);
            let clip_max_y = clamp(clip_max_y, clip_min_y..=height_pixels as f32);
            let clip_min_x = clip_min_x.round() as u32;
            let clip_min_y = clip_min_y.round() as u32;
            let clip_max_x = clip_max_x.round() as u32;
            let clip_max_y = clip_max_y.round() as u32;

            ctx.apply_scissor_rect(
                clip_min_x as i32,
                (height_pixels as u32 - clip_max_y) as i32,
                (clip_max_x - clip_min_x) as i32,
                (clip_max_y - clip_min_y) as i32,
            );
            ctx.apply_bindings(&self.bindings);
            ctx.apply_uniforms(&shader::Uniforms { screen_size });
            ctx.draw(0, mesh.indices.len() as i32, 1);
        }
    }
}

mod shader {
    use miniquad::{ShaderMeta, UniformBlockLayout, UniformDesc, UniformType};

    pub const VERTEX: &str = r#"#version 100
    uniform vec2 u_screen_size;

    attribute vec2 a_pos;
    attribute vec2 a_tc;
    attribute vec4 a_color;

    varying vec2 v_tc;
    varying vec4 v_color;

    void main() {
        gl_Position = vec4(
            2.0 * a_pos.x / u_screen_size.x - 1.0,
            1.0 - 2.0 * a_pos.y / u_screen_size.y,
            0.0,
            1.0);

        v_tc = a_tc;
        v_color = a_color / 255.0;
    }"#;

    pub const FRAGMENT: &str = r#"#version 100
    uniform sampler2D u_sampler;
    precision highp float;
    varying vec2 v_tc;
    varying vec4 v_color;
    void main() {
        gl_FragColor = v_color;
        // gl_FragColor *= texture2D(u_sampler, v_tc); // TODO: this is what we *should* do, but the texture is always black on my mac
        gl_FragColor.r *= texture2D(u_sampler, v_tc).r;
    }
    "#;

    pub fn meta() -> ShaderMeta {
        ShaderMeta {
            images: vec![],
            uniforms: UniformBlockLayout {
                uniforms: vec![UniformDesc::new("u_screen_size", UniformType::Float2)],
            },
        }
    }

    #[repr(C)]
    #[derive(Debug)]
    pub struct Uniforms {
        pub screen_size: (f32, f32),
    }
}
