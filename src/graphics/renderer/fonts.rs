use msdfgen::{Bitmap, FillRule, FontExt, MsdfGeneratorConfig};
use ttf_parser::Face;
use crate::graphics::renderer::{Renderer, texture::Texture};
use nalgebra_glm as glm;

pub struct Font<'a> {
    texture: Texture<'a>,
    supported_characters: Vec<char>,
}

fn get_western_iterator() -> impl Iterator<Item = char> {
	(0x0020u32..=0x007E)
		.chain(0x00A0..=0x00FF)
		.filter_map(std::char::from_u32)
}

const GLYPH_SIZE: usize = 31;
const GLYPYH_MARGIN: usize = 1;
const SPACE_PER_GLYPH: usize = GLYPH_SIZE + GLYPYH_MARGIN;

fn calculate_atlas_dimensions(char_amount: usize) -> (usize, usize) {
    let mut size = 1usize;
    while size * size < char_amount {
        size *= 2;
    }

    let line_amount = (char_amount + size - 1) / size;
    return (line_amount * SPACE_PER_GLYPH, size * SPACE_PER_GLYPH);
}

impl<'a> Font<'a> {
    pub fn from_bytes(renderer: &'a Renderer, bytes: &[u8]) -> Option<Self> {
        let font = Face::parse(bytes, 0).ok()?;

        let supported_characters: Vec<char> = get_western_iterator()
            .filter(|&c| font.glyph_index(c).is_some())
            .collect();

        let (atlas_height, atlas_width) = calculate_atlas_dimensions(supported_characters.len());
        let columns = atlas_width / SPACE_PER_GLYPH;

        let mut raw_image_data = vec![glm::U8Vec3::new(0, 0, 0); atlas_height * atlas_width];

        for (position, c) in supported_characters.iter().enumerate() {
            let glyph = font.glyph_index(*c).unwrap();
            let mut shape = match font.glyph_shape(glyph) {
                Some(s) => s,
                None => continue,
            };

            let bound = shape.get_bound();
            let framing = bound.autoframe(GLYPH_SIZE as u32, GLYPH_SIZE as u32, msdfgen::Range::Px(4.0), None).unwrap();
            let fill_rule = FillRule::default();

            let mut bitmap = Bitmap::new(GLYPH_SIZE as u32, GLYPH_SIZE as u32);
            shape.edge_coloring_simple(3.0, 0);

            let config = MsdfGeneratorConfig::default();
            shape.generate_msdf(&mut bitmap, &framing, &config);
            shape.correct_sign(&mut bitmap, &framing, fill_rule);
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
        }

        let texture = Texture::from_font_raw(renderer, raw_image_data.as_slice(), atlas_width);
        
        return Some(Self {
            texture,
            supported_characters
        });
    }

    pub fn bind_to_unit(&self, unit: u32) {
        self.texture.bind_to_unit(unit);
    }
}