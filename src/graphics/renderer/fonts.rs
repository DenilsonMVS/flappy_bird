use std::marker::PhantomData;

use msdfgen::{Bitmap, FillRule, FontExt, MsdfGeneratorConfig, Range};
use ttf_parser::Face;
use macros::{GlVertex, program_interface};
use crate::graphics::renderer::{
    Renderer, 
    buffer::{self, Buffer, Dynamic, Static}, 
    drawable::DrawMode, 
    positioning::{BaseDimensions, PositionMode, generate_box}, 
    program::{Program, ShaderType}, 
    texture::Texture, 
    uniform::UniformValue, 
    vertex_array_object::{FieldType, StaticVertexLayout, VertexArrayObject}
};
use nalgebra_glm as glm;
use anyhow::Result;

const GLYPHS_PER_RENDER: usize = 1 << 10;

#[repr(C)]
#[derive(GlVertex)]
#[vertex(divisor = 1)]
pub struct GlyphAttrs {
    pub bound_min: glm::Vec2,
    pub bound_max: glm::Vec2,
    pub character_idx: u32,
    
    #[normalized]
    pub color: glm::U8Vec4,
}

#[derive(Clone, Copy, Default)]
struct GlyphInfo {
    bound_min: glm::Vec2,
    bound_max: glm::Vec2,
    advance: f32,
}

#[program_interface(
    vert = "../../../res/shaders/font.vert",
    frag = "../../../res/shaders/font.frag"
)]
struct FontProgram {
    u_texture: i32,
    u_projection: glm::Mat4,
    u_px_range: f32,
    u_glyph_size: u32,
    u_glyph_margin: u32,
}

pub struct Fonts<'a> {
    font_program: FontProgram<'a>,
    _marker: PhantomData<&'a Renderer>,
}

impl<'a> Fonts<'a> {
    pub fn new(renderer: &'a Renderer) -> Self {
        let font_program = FontProgram::init(renderer).unwrap();
        font_program.bind();
        font_program.set_u_texture(&0);
        font_program.set_u_px_range(&PX_RANGE);
        font_program.set_u_glyph_size(&(GLYPH_SIZE as u32));
        font_program.set_u_glyph_margin(&(GLYPYH_MARGIN as u32));

        return Self {
            font_program,
            _marker: PhantomData,
        };
    }

    pub fn new_font(&self, renderer: &'a Renderer, bytes: &[u8]) -> Result<Font<'a>> {
        Font::from_bytes(renderer, bytes)
    }

    pub fn draw_buffer(&self, font: &Font, buffer: &FontVbo, proj_matrix: &glm::Mat4) {
        self.font_program.bind();
        self.font_program.set_u_projection(proj_matrix);
        font.texture.bind_to_unit(0);
        buffer.vao.draw_instanced(4, buffer.amount, DrawMode::TriangleStrip);
    }

    pub fn draw(&self, font: &mut Font, proj_matrix: &glm::Mat4) {
        self.font_program.bind();
        self.font_program.set_u_projection(proj_matrix);
        font.texture.bind_to_unit(0);
        font.vao.draw_instanced(4, font.staging_area.len() as i32, DrawMode::TriangleStrip);
        font.staging_area.clear();
    }
}

pub struct FontVbo<'a> {
    _vbo: Buffer<'a, GlyphAttrs, buffer::Static>,
    vao: VertexArrayObject<'a>,
    amount: i32,
}

pub struct Font<'a> {
    texture: Texture<'a>,
    glyphs: [GlyphInfo; WESTERN_CHAR_COUNT],
    ascender: f32,
    descender: f32,
    staging_area: Vec<GlyphAttrs>,
    vbo: Buffer<'a, GlyphAttrs, buffer::Dynamic>,
    vao: VertexArrayObject<'a>,
}

const fn map_char_to_index(c: char) -> Option<usize> {
    let code = c as usize;
    if code >= UTF_8_START && code < UTF_8_END {
        Some(code - UTF_8_START)
    } else if code >= EXTRA_START {
        Some((code - EXTRA_START + UTF_8_END - UTF_8_START) as usize)
    } else {
        None
    }
}

fn get_western_iterator() -> impl Iterator<Item = char> {
    (UTF_8_START as u32..UTF_8_END as u32)
        .chain(EXTRA_START as u32..EXTRA_END as u32)
        .filter_map(std::char::from_u32)
}

const fn calculate_atlas_params(char_amount: usize) -> (usize, usize, usize) {
    let mut size = 1usize;
    while size * size < char_amount {
        size *= 2;
    }
    let columns = size;
    let rows = (char_amount + columns - 1) / columns;
    
    (rows * SPACE_PER_GLYPH, columns * SPACE_PER_GLYPH, columns)
}

pub const GLYPH_SIZE: usize = 31;
pub const GLYPYH_MARGIN: usize = 1;
pub const SPACE_PER_GLYPH: usize = GLYPH_SIZE + GLYPYH_MARGIN;
pub const PX_RANGE: f32 = 4.0;
pub const UTF_8_START: usize = 0x0020;
pub const UTF_8_END: usize = 0x007F;
pub const EXTRA_START: usize = 0x00A0;
pub const EXTRA_END: usize = 0x0100;
pub const WESTERN_CHAR_COUNT: usize = UTF_8_END - UTF_8_START + EXTRA_END - EXTRA_START;

const ATLAS_PARAMS: (usize, usize, usize) = calculate_atlas_params(WESTERN_CHAR_COUNT);
pub const ATLAS_HEIGHT: usize = ATLAS_PARAMS.0;
pub const ATLAS_WIDTH: usize = ATLAS_PARAMS.1;
pub const ATLAS_COLUMNS: usize = ATLAS_PARAMS.2;

#[derive(Debug, Clone, Copy)]
pub struct TextRenderConfig<'a> {
    pub text: &'a str,
    pub position: glm::Vec2,
    pub line_height: f32,
    pub position_mode: PositionMode,
    pub color: glm::U8Vec4,
}

impl<'a> TextRenderConfig<'a> {
    pub fn new(text: &'a str, position: glm::Vec2, line_height: f32, color: glm::U8Vec4) -> Self {
        Self {
            text,
            position,
            line_height,
            position_mode: PositionMode::Center,
            color,
        }
    }

    pub fn with_mode(mut self, mode: PositionMode) -> Self {
        self.position_mode = mode;
        self
    }
}

impl<'a> Font<'a> {
    fn from_bytes(renderer: &'a Renderer, bytes: &[u8]) -> Result<Self> {
        let font = Face::parse(bytes, 0)?;

        let mut raw_image_data = [glm::U8Vec3::new(0, 0, 0); ATLAS_HEIGHT * ATLAS_WIDTH];
        let mut glyphs = [GlyphInfo::default(); WESTERN_CHAR_COUNT];

        for (idx, c) in get_western_iterator().enumerate() {
            let info = &mut glyphs[idx];
            let idx = map_char_to_index(c).unwrap();

            if let Some(glyph_id) = font.glyph_index(c) {
                info.advance = font.glyph_hor_advance(glyph_id).unwrap_or(0) as f32;

                if let Some(mut shape) = font.glyph_shape(glyph_id) {
                    if let Some(framing) = shape.get_bound().autoframe(
                        GLYPH_SIZE as u32,
                        GLYPH_SIZE as u32,
                        Range::Px(PX_RANGE as f64),
                        None
                    ) {
                        let mut bitmap = Bitmap::new(GLYPH_SIZE as u32, GLYPH_SIZE as u32);
                        let config = MsdfGeneratorConfig::default();

                        shape.edge_coloring_simple(3.0, 0);
                        shape.generate_msdf(&mut bitmap, &framing, &config);
                        shape.correct_sign(&mut bitmap, &framing, FillRule::default());
                        shape.correct_msdf_error(&mut bitmap, &framing, &config);

                        let start_y = (idx / ATLAS_COLUMNS) * SPACE_PER_GLYPH;
                        let start_x = (idx % ATLAS_COLUMNS) * SPACE_PER_GLYPH;

                        for y in 0..GLYPH_SIZE {
                            for x in 0..GLYPH_SIZE {
                                let pixel = bitmap.pixel(x as u32, (GLYPH_SIZE - y - 1) as u32);
                                let r = (pixel.r * 255.0).clamp(0.0, 255.0) as u8;
                                let g = (pixel.g * 255.0).clamp(0.0, 255.0) as u8;
                                let b = (pixel.b * 255.0).clamp(0.0, 255.0) as u8;

                                raw_image_data[(start_y + y) * ATLAS_WIDTH + start_x + x] = 
                                    glm::U8Vec3::new(r, g, b);
                            }
                        }

                        let scale = framing.projection.scale;
                        let translate = framing.projection.translate;

                        info.bound_min = glm::vec2(-translate.x as f32, -translate.y as f32);
                        info.bound_max = info.bound_min + glm::vec2(
                            GLYPH_SIZE as f32 / scale.x as f32,
                            GLYPH_SIZE as f32 / scale.y as f32
                        );
                    }
                }
            }
        }

        let texture = Texture::from_font_raw(renderer, raw_image_data.as_slice(), ATLAS_WIDTH);
        let vbo = Buffer::<GlyphAttrs, Dynamic>::new(renderer, GLYPHS_PER_RENDER);
        let vao = VertexArrayObject::new(renderer, &[&vbo]);

        return Ok(Self {
            texture,
            glyphs,
            ascender: font.ascender() as f32,
            descender: font.descender() as f32,
            staging_area: Vec::with_capacity(GLYPHS_PER_RENDER),
            vbo,
            vao,
        });
    }

    pub fn bind_to_unit(&self, unit: u32) {
        self.texture.bind_to_unit(unit);
    }

    pub fn send(&mut self) {
        self.vbo.set_sub_data(&self.staging_area, 0);
    }

    pub fn add_text(&mut self, text: &TextRenderConfig) {
        let font_height = self.ascender - self.descender;
        let total_width = text.text.chars().fold(0.0f32, |acc, c| {
            if let Some(idx) = map_char_to_index(c) {
                acc + self.glyphs[idx].advance
            } else {
                acc
            }
        });

        let text_block_box = generate_box(
            text.position,
            text.position_mode,
            glm::vec2(total_width, font_height),
            BaseDimensions::Height(text.line_height),
        );

        let scale = text.line_height / font_height;
        let mut cursor_x = text_block_box.min.x;
        let baseline_y = text_block_box.min.y - (self.descender * scale);

        for c in text.text.chars() {
            if let Some(idx) = map_char_to_index(c) {
                let info = &self.glyphs[idx];
                
                let min_pos = glm::vec2(cursor_x, baseline_y) + info.bound_min * scale;
                let max_pos = glm::vec2(cursor_x, baseline_y) + info.bound_max * scale;

                self.staging_area.push(GlyphAttrs {
                    bound_min: min_pos,
                    bound_max: max_pos,
                    character_idx: idx as u32,
                    color: text.color
                });

                cursor_x += info.advance * scale;
            }
        }
    }

    pub fn create_text_vbo<'b>(&self, renderer: &'b Renderer, rendered_text: &[TextRenderConfig]) -> FontVbo<'b> {
        let mut glyph_buffer_data = Vec::with_capacity(
            rendered_text.iter().fold(0usize, |prev, cur| prev + cur.text.len())
        );
        
        let font_height = self.ascender - self.descender;

        for config in rendered_text {
            let total_width = config.text.chars().fold(0.0f32, |acc, c| {
                if let Some(idx) = map_char_to_index(c) {
                    acc + self.glyphs[idx].advance
                } else {
                    acc
                }
            });

            let text_block_box = generate_box(
                config.position,
                config.position_mode,
                glm::vec2(total_width, font_height),
                BaseDimensions::Height(config.line_height),
            );

            let scale = config.line_height / font_height;
            let mut cursor_x = text_block_box.min.x;
            let baseline_y = text_block_box.min.y - (self.descender * scale);

            for c in config.text.chars() {
                if let Some(idx) = map_char_to_index(c) {
                    let info = &self.glyphs[idx];
                    
                    let min_pos = glm::vec2(cursor_x, baseline_y) + info.bound_min * scale;
                    let max_pos = glm::vec2(cursor_x, baseline_y) + info.bound_max * scale;

                    glyph_buffer_data.push(GlyphAttrs {
                        bound_min: min_pos,
                        bound_max: max_pos,
                        character_idx: idx as u32,
                        color: config.color
                    });

                    cursor_x += info.advance * scale;
                }
            }
        }

        let vbo = Buffer::<GlyphAttrs, Static>::new(renderer, &glyph_buffer_data);
        let vao = VertexArrayObject::new(renderer, &[&vbo]);
        
        FontVbo {
            _vbo: vbo,
            vao,
            amount: glyph_buffer_data.len() as i32
        }
    }
}
