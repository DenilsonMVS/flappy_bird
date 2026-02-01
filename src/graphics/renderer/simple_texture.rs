use macros::{GlVertex, program_interface};
use nalgebra_glm as glm;
use crate::graphics::renderer::{Renderer, atlas::{TypedAtlas, UvInfo}, buffer::{Buffer, Dynamic}, drawable::DrawMode, positioning::{BaseDimensions, PositionMode, SimpleTransform, generate_box, generate_oriented_box}, program::{Program, ShaderType}, texture::{MagFiltering, MinFiltering, Texture, TextureWrap}, uniform::UniformValue, vertex_array_object::{FieldType, StaticVertexLayout, VertexArrayObject}};
use anyhow::Result;

const QUADS_PER_RENDER: usize = 1 << 10;

#[repr(C)]
#[derive(GlVertex, Debug)]
#[vertex(divisor = 1)]
struct TextureVertex {
    top_left: glm::Vec2,
    top_right: glm::Vec2,
    bot_left: glm::Vec2,
    bot_right: glm::Vec2,
    uv_min: glm::Vec2,
    uv_max: glm::Vec2,
}

#[program_interface(
	vert = "../../../res/shaders/texture.vert",
	frag = "../../../res/shaders/texture.frag"
)]
struct TextureProgram {
    u_projection: glm::Mat4,
    u_texture: i32,
}

pub struct SimpleTextureRenderer<'a> {
    program: TextureProgram<'a>,
}

impl<'a> SimpleTextureRenderer<'a> {
    pub fn new(renderer: &'a Renderer) -> Self {
        let program = TextureProgram::init(renderer).unwrap();
        program.set_u_texture(&0);

        return Self { program };
    }

    pub fn draw<Atlas: TypedAtlas>(&self, projection_matrix: &glm::Mat4, simple_texture: &mut SimpleTexture<Atlas>) {
        self.program.bind();
        self.program.set_u_projection(projection_matrix);
        
        simple_texture.get_texture().bind_to_unit(0);
        simple_texture.get_vao().draw_instanced(4, simple_texture.get_quad_amount() as i32, DrawMode::TriangleStrip);
        simple_texture.clear_staging_area();
    }
}

pub struct SimpleTexture<'a, Atlas: TypedAtlas> {
    texture: Texture<'a>,
    vbo: Buffer<'a, TextureVertex, Dynamic>,
    vao: VertexArrayObject<'a>,
    staging_area: Vec<TextureVertex>,
    atlas: Atlas,
}

impl<'a, Atlas: TypedAtlas> SimpleTexture<'a, Atlas> {
    pub fn new(
        renderer: &'a Renderer,
        texture: &[u8],
        mag_filter: MagFiltering,
        min_filter: MinFiltering,
        wrap: TextureWrap,
        atlas: &[u8],
    ) -> Result<Self> {
        let texture = Texture::from_image_bytes(renderer, texture, mag_filter, min_filter, wrap)?;
        let vbo = Buffer::<TextureVertex, Dynamic>::new(renderer, QUADS_PER_RENDER * 4);
        let vao = VertexArrayObject::new(renderer, &[&vbo]);
        let staging_area = Vec::with_capacity(QUADS_PER_RENDER * 4);
        let atlas = Atlas::new(atlas)?;
        return Ok(Self { texture, vao, vbo, staging_area, atlas });
    }

    pub fn send(&mut self) {
        self.vbo.set_sub_data(&self.staging_area, 0);
    }

    fn get_texture(&'a self) -> &'a Texture<'a> {
        &self.texture
    }

    fn get_vao(&'a self) -> &'a VertexArrayObject<'a> {
        &self.vao
    }

    fn get_quad_amount(&self) -> usize {
        self.staging_area.len()
    }

    fn clear_staging_area(&mut self) {
        self.staging_area.clear();
    }

    pub fn get_frame_info(&self, frame: Atlas::Frame) -> (UvInfo, glm::Vec2) {
        self.atlas.get_info(frame)
    }

    pub fn add_quad(&mut self,
        position: glm::Vec2,
        position_mode: PositionMode,
        original_size: glm::Vec2,
        base_dimension: BaseDimensions,
        uv_data: &UvInfo,
    ) {
        let simple_box = generate_box(position, position_mode, original_size, base_dimension);

        self.staging_area.push(TextureVertex {
            top_left: glm::vec2(simple_box.min.x, simple_box.max.y),
            top_right: glm::vec2(simple_box.max.x, simple_box.max.y),
            bot_left: glm::vec2(simple_box.min.x, simple_box.min.y),
            bot_right: glm::vec2(simple_box.max.x, simple_box.min.y),
            uv_min: uv_data.min,
            uv_max: uv_data.max,
        });
    }

    pub fn add_quad_simple_transform(&mut self,
        position: glm::Vec2,
        position_mode: PositionMode,
        original_size: glm::Vec2,
        base_dimension: BaseDimensions,
        uv_data: &UvInfo,
        transform: SimpleTransform,
    ) {
        let simple_box = generate_box(position, position_mode, original_size, base_dimension);

        let (l, r) = (simple_box.min.x, simple_box.max.x);
        let (b, t) = (simple_box.min.y, simple_box.max.y);

        let (tl, tr, bl, br) = match transform {
            SimpleTransform::None => (
                glm::vec2(l, t), glm::vec2(r, t),
                glm::vec2(l, b), glm::vec2(r, b),
            ),
            SimpleTransform::FlipHorizontal => (
                glm::vec2(r, t), glm::vec2(l, t),
                glm::vec2(r, b), glm::vec2(l, b),
            ),
            SimpleTransform::FlipVertical => (
                glm::vec2(l, b), glm::vec2(r, b),
                glm::vec2(l, t), glm::vec2(r, t),
            ),
            SimpleTransform::Rotate180 => (
                glm::vec2(r, b), glm::vec2(l, b),
                glm::vec2(r, t), glm::vec2(l, t),
            )
        };

        self.staging_area.push(TextureVertex {
            top_left: tl,
            top_right: tr,
            bot_left: bl,
            bot_right: br,
            uv_min: uv_data.min,
            uv_max: uv_data.max,
        });
    }

    pub fn add_oriented_quad(&mut self,
        position: glm::Vec2,
        position_mode: PositionMode,
        original_size: glm::Vec2,
        base_dimension: BaseDimensions,
        up_vector: glm::Vec2,
        uv_data: &UvInfo,
    ) {
        let oriented_box = generate_oriented_box(
            position, position_mode, original_size, base_dimension, up_vector);

        self.staging_area.push(TextureVertex {
            top_left: oriented_box.top_left,
            top_right: oriented_box.top_right,
            bot_left: oriented_box.bot_left,
            bot_right: oriented_box.bot_right,
            uv_min: uv_data.min,
            uv_max: uv_data.max,
        });
    }
}
