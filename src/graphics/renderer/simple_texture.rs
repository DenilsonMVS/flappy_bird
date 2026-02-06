use macros::{GlVertex, program_interface};
use nalgebra_glm as glm;
use crate::graphics::renderer::{Renderer, atlas::{FrameInfo, TypedAtlas, UvInfo}, buffer::{Buffer, Dynamic}, drawable::DrawMode, positioning::{BaseDimensions, OrientedBox, PositionMode, RenderBox, SCALE_FACTOR, SimpleTransform}, program::{Program, ShaderType}, texture::{MagFiltering, MinFiltering, Texture, TextureWrap}, uniform::UniformValue, vertex_array_object::{FieldType, StaticVertexLayout, VertexArrayObject}};
use anyhow::Result;

const QUADS_PER_RENDER: usize = 1 << 10;

#[repr(C)]
#[derive(GlVertex, Debug)]
#[vertex(divisor = 1)]
struct TextureVertex {
    center: glm::I16Vec2,
    top_edge: glm::I16Vec2,
    uv_min: glm::U16Vec2,
    uv_max: glm::U16Vec2,
}

#[program_interface(
	vert = "../../../res/shaders/texture.vert",
	frag = "../../../res/shaders/texture.frag"
)]
struct TextureProgram {
    u_projection: glm::Mat4,
    u_texture: i32,
    u_world_scale: f32,
}

pub struct SimpleTextureRenderer<'a> {
    program: TextureProgram<'a>,
}

impl<'a> SimpleTextureRenderer<'a> {
    pub fn new(renderer: &'a Renderer) -> Self {
        let program = TextureProgram::init(renderer).unwrap();
        program.set_u_texture(&0);
        program.set_u_world_scale(&(1.0 / SCALE_FACTOR));

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

    pub fn get_frame_info(&self, frame: Atlas::Frame) -> FrameInfo {
        self.atlas.get_info(frame)
    }

    pub fn add_quad(&mut self,
        position: glm::Vec2,
        position_mode: PositionMode,
        base_dimension: BaseDimensions,
        frame_info: &FrameInfo,
    ) {
        let original_size = frame_info.get_original_dimensions();
        let uv_data = frame_info.get_uv();
        let simple_box = RenderBox::new(position, position_mode, original_size, base_dimension);
        
        self.staging_area.push(TextureVertex {
            center: simple_box.get_center(),
            top_edge: simple_box.get_top_edge(),
            uv_min: uv_data.min,
            uv_max: uv_data.max,
        });
    }

    pub fn add_quad_simple_transform(
        &mut self,
        position: glm::Vec2,
        position_mode: PositionMode,
        base_dimension: BaseDimensions,
        frame_info: &FrameInfo,
        transform: SimpleTransform,
    ) {
        let original_size = frame_info.get_original_dimensions();
        let uv_data = frame_info.get_uv();
        let simple_box = RenderBox::new(position, position_mode, original_size, base_dimension);

        let (top_edge, uv_min, uv_max) = match transform {
            SimpleTransform::None => (
                simple_box.get_top_edge(),
                uv_data.min, 
                uv_data.max
            ),
            SimpleTransform::FlipHorizontal => (
                simple_box.get_top_edge(),
                glm::vec2(uv_data.max.x, uv_data.min.y),
                glm::vec2(uv_data.min.x, uv_data.max.y),
            ),
            SimpleTransform::FlipVertical => (
                simple_box.get_top_edge(),
                glm::vec2(uv_data.min.x, uv_data.max.y),
                glm::vec2(uv_data.max.x, uv_data.min.y),
            ),
            SimpleTransform::Rotate180 => (
                -simple_box.get_top_edge(),
                uv_data.min,
                uv_data.max
            )
        };

        self.staging_area.push(TextureVertex {
            center: simple_box.get_center(),
            top_edge,
            uv_min,
            uv_max,
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
        let oriented_box = OrientedBox::new(
            position, position_mode, original_size, base_dimension, up_vector);
        
        self.staging_area.push(TextureVertex {
            center: oriented_box.get_center(),
            top_edge: oriented_box.get_top_edge(),
            uv_min: uv_data.min,
            uv_max: uv_data.max,
        });
    }
}
