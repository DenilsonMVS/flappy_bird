use std::marker::PhantomData;

use msdfgen::{Bitmap, FillRule, FontExt, MsdfGeneratorConfig, Vector2};
use ttf_parser::Face;
use vertex_derive::{GlVertex, program_interface};
use crate::graphics::renderer::{Bindable, Renderer, buffer::{BufferUsage, VertexBuffer}, drawable::DrawMode, program::{Program, ShaderType}, texture::Texture, uniform::UniformValue, vertex_array_object::{FieldType, StaticVertexLayout, VertexArrayObject}};
use nalgebra_glm as glm;

#[repr(C)]
#[derive(GlVertex)]
pub struct GlyphAttrs {
    bound_min: glm::Vec2,
    bound_max: glm::Vec2,
    character_idx: u32,
}

#[repr(C)]
#[derive(GlVertex)]
pub struct BaseFormat {
    offsets: glm::Vec2,
}

struct GlyphInfo {
    bound_min: glm::Vec2,
    bound_max: glm::Vec2,
	advance: f32,
}

struct Glyph {
	character: char,
	info: GlyphInfo,
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

impl<'a> Fonts<'a>  {
    pub fn new(renderer: &'a Renderer) -> Self {
        let font_program = FontProgram::init(renderer).unwrap();
        font_program.bind();
        font_program.set_u_texture(&0);
        font_program.set_u_px_range(&PX_RANGE);
        font_program.set_u_glyph_size(&(GLYPH_SIZE as u32));
        font_program.set_u_glyph_margin(&(GLYPYH_MARGIN as u32));

        return Self {
            font_program,
            _marker: PhantomData
        };
    }

    pub fn new_font(&self, renderer: &'a Renderer, bytes: &[u8]) -> Option<Font<'a>> {
        Font::from_bytes(renderer, bytes)
    }

    pub fn draw_buffer(&self, buffer: &FontVbo, proj_matrix: &glm::Mat4) {
        self.font_program.bind();
        self.font_program.set_u_projection(proj_matrix);
        buffer.texture.bind_to_unit(0);
        buffer.vao.draw_instanced(4, buffer.amount, DrawMode::TriangleFan);
    }
}

pub struct FontVbo<'a> {
    _vbo: VertexBuffer<'a, GlyphAttrs>,
    vao: VertexArrayObject<'a>,
    amount: i32,
    texture: &'a Texture<'a>,
}

pub struct Font<'a> {
    texture: Texture<'a>,
    glyphs: Vec<Glyph>,
    ascender: f32,
	descender: f32
}

fn get_western_iterator() -> impl Iterator<Item = char> {
	(0x0020u32..=0x007E)
		.chain(0x00A0..=0x00FF)
		.filter_map(std::char::from_u32)
}

pub const GLYPH_SIZE: usize = 31;
pub const GLYPYH_MARGIN: usize = 1;
pub const SPACE_PER_GLYPH: usize = GLYPH_SIZE + GLYPYH_MARGIN;

fn calculate_atlas_dimensions(char_amount: usize) -> (usize, usize) {
    let mut size = 1usize;
    while size * size < char_amount {
        size *= 2;
    }

    let line_amount = (char_amount + size - 1) / size;
    return (line_amount * SPACE_PER_GLYPH, size * SPACE_PER_GLYPH);
}

pub const PX_RANGE: f32 = 4.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PositionMode {
	TopLeft,    TopCenter,    TopRight,
	CenterLeft, Center,       CenterRight,
	BottomLeft, BottomCenter, BottomRight,
}

impl PositionMode {
	pub fn get_offsets(&self) -> glm::Vec2 {
		match self {
			PositionMode::TopLeft      => glm::vec2(0.0, 1.0),
			PositionMode::TopCenter    => glm::vec2(0.5, 1.0),
			PositionMode::TopRight     => glm::vec2(1.0, 1.0),
			PositionMode::CenterLeft   => glm::vec2(0.0, 0.5),
			PositionMode::Center       => glm::vec2(0.5, 0.5),
			PositionMode::CenterRight  => glm::vec2(1.0, 0.5),
			PositionMode::BottomLeft   => glm::vec2(0.0, 0.0),
			PositionMode::BottomCenter => glm::vec2(0.5, 0.0),
			PositionMode::BottomRight  => glm::vec2(1.0, 0.0),
		}
	}
}

pub struct TextRenderConfig<'a> {
    pub text: &'a str,
    pub position: glm::Vec2,
    pub line_height: f32,
    pub position_mode: PositionMode,
}

impl<'a> TextRenderConfig<'a> {
	pub fn new(text: &'a str, position: glm::Vec2, line_height: f32) -> Self {
		Self {
			text,
			position,
			line_height,
			position_mode: PositionMode::Center,
		}
	}

	pub fn with_mode(mut self, mode: PositionMode) -> Self {
		self.position_mode = mode;
		self
	}
}

impl<'a> Font<'a> {
    fn from_bytes(renderer: &'a Renderer, bytes: &[u8]) -> Option<Self> {
        let font = Face::parse(bytes, 0).ok()?;

        let amount_supported_characters = get_western_iterator()
            .filter(|&c| font.glyph_index(c).is_some())
            .count();

        let (atlas_height, atlas_width) = calculate_atlas_dimensions(amount_supported_characters);
        let columns = atlas_width / SPACE_PER_GLYPH;

        let mut raw_image_data = vec![glm::U8Vec3::new(0, 0, 0); atlas_height * atlas_width];
        let mut glyphs = Vec::with_capacity(amount_supported_characters);

        for (position, c) in get_western_iterator()
            .filter(|&c| font.glyph_index(c).is_some())
            .enumerate()
        {
            let glyph = font.glyph_index(c).unwrap();
            let mut shape = match font.glyph_shape(glyph) {
                Some(s) => s,
                None => {
                    glyphs.push(Glyph {
                        character: c,
                        info: GlyphInfo {
                            bound_min: glm::vec2(0.0, 0.0),
                            bound_max: glm::vec2(0.0, 0.0),
                            advance: font.glyph_hor_advance(glyph).unwrap_or(0) as f32,
                        },
                    });
                    continue;
                },
            };

            let bound = shape.get_bound();
            let glyph_width = bound.right - bound.left;
            let glyph_height = bound.top - bound.bottom;

            let internal_margin = PX_RANGE as f64; 
            let usable_size = GLYPH_SIZE as f64 - (internal_margin * 2.0);

            let scale_x = if glyph_width > 0.0 { usable_size / glyph_width } else { 0.0 };
            let scale_y = if glyph_height > 0.0 { usable_size / glyph_height } else { 0.0 };

            let translate_x = if scale_x > 0.0 { (internal_margin / scale_x) - bound.left } else { 0.0 };
            let translate_y = if scale_y > 0.0 { (internal_margin / scale_y) - bound.bottom } else { 0.0 };

            let projection = msdfgen::Projection {
                scale: Vector2::new(scale_x, scale_y),
                translate: Vector2::new(translate_x, translate_y),
            };

            let range_in_font_units = PX_RANGE as f64 / f64::min(scale_x, scale_y);
            let framing = msdfgen::Framing {
                projection,
                range: range_in_font_units, 
            };
            
            let mut bitmap = Bitmap::new(GLYPH_SIZE as u32, GLYPH_SIZE as u32);

            let config = MsdfGeneratorConfig::default();
            shape.edge_coloring_simple(3.0, 0);
            shape.generate_msdf(&mut bitmap, &framing, &config);
            shape.correct_sign(&mut bitmap, &framing, FillRule::default());
            shape.correct_msdf_error(&mut bitmap, &framing, &config);

            let start_y = (position / columns) * SPACE_PER_GLYPH;
            let start_x = (position % columns) * SPACE_PER_GLYPH;

            for y in 0..GLYPH_SIZE {
                for x in 0..GLYPH_SIZE {
                    let pixel = bitmap.pixel(x as u32, (GLYPH_SIZE - y - 1) as u32);
                    let r = (pixel.r * 255.0).clamp(0.0, 255.0) as u8;
                    let g = (pixel.g * 255.0).clamp(0.0, 255.0) as u8;
                    let b = (pixel.b * 255.0).clamp(0.0, 255.0) as u8;

                    raw_image_data[(start_y + y) * atlas_width + start_x + x] = glm::U8Vec3::new(r, g, b);
                }
            }

            let slot_min_font = glm::vec2(-translate_x as f32, -translate_y as f32);
            let slot_max_font = slot_min_font + glm::vec2(
                GLYPH_SIZE as f32 / scale_x as f32,
                GLYPH_SIZE as f32 / scale_y as f32
            );

            glyphs.push(Glyph {
                character: c,
                info: GlyphInfo {
                    bound_min: slot_min_font,
                    bound_max: slot_max_font,
                    advance: font.glyph_hor_advance(glyph).unwrap_or(0) as f32,
                },
            });
        }

        let texture = Texture::from_font_raw(renderer, raw_image_data.as_slice(), atlas_width);

        return Some(Self {
            texture,
            glyphs,
            ascender: font.ascender() as f32,
			descender: font.descender() as f32,
        });
    }

    pub fn bind_to_unit(&self, unit: u32) {
        self.texture.bind_to_unit(unit);
    }

    pub fn create_text_vbo(&'a self, renderer: &'a Renderer, rendered_text: &[TextRenderConfig<'a>]) -> FontVbo<'a> {
        let mut glyph_buffer_data = Vec::with_capacity(rendered_text
            .iter()
            .fold(0usize, |prev, cur| prev + cur.text.len())
        );

        let font_height = self.ascender - self.descender;

        for TextRenderConfig {
            text,
            position,
            line_height,
            position_mode
        } in rendered_text {
            let total_width = text.chars().fold(0.0f32, |acc, c|
                acc + self.glyphs
                    .binary_search_by_key(&c, |g| g.character)
                    .map(|idx| self.glyphs[idx].info.advance)
                    .unwrap_or(0.0)
            );

            let scale = line_height / font_height;
            let offsets = position_mode.get_offsets();

            let mut cursor_x = position.x - (total_width * scale * offsets.x);

            let baseline_offset = self.descender * scale;
            let cursor_y = position.y - (line_height * offsets.y) - baseline_offset;

            for c in text.chars() {
                if let Ok(idx) = self.glyphs.binary_search_by_key(&c, |g| g.character) {
                    let info = &self.glyphs[idx].info;
                    
                    let min_pos = glm::vec2(cursor_x, cursor_y) + info.bound_min * scale;
                    let max_pos = glm::vec2(cursor_x, cursor_y) + info.bound_max * scale;

                    glyph_buffer_data.push(GlyphAttrs {
                        bound_min: min_pos,
                        bound_max: max_pos,
                        character_idx: idx as u32
                    });

                    cursor_x += info.advance * scale;
                }
            }
        }

        let mut vbo = VertexBuffer::new(renderer)
            .set_instanced(1);
        vbo.set_data(&glyph_buffer_data, BufferUsage::StaticDraw);

        let vao = VertexArrayObject::new(renderer, &[&vbo]);
        
        return FontVbo {
            _vbo: vbo,
            vao,
            amount: glyph_buffer_data.len() as i32,
            texture: &self.texture
        };
    }
}