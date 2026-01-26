
use nalgebra_glm as glm;
pub mod buffer;
pub mod vertex_array_object;
pub mod types;
pub mod program;
pub mod drawable;

pub trait Bindable {
    fn bind(&self);
}

trait GlEnum: Sized {
    fn to_gl_enum(&self) -> u32;
    fn from_gl_enum(value: u32) -> Option<Self>;
}


pub enum ClearField {
    Color,
    Depth,
    Stencil
}

impl GlEnum for ClearField {
    fn to_gl_enum(&self) -> u32 {
        match self {
			ClearField::Color => gl::COLOR_BUFFER_BIT,
			ClearField::Depth => gl::DEPTH_BUFFER_BIT,
			ClearField::Stencil => gl::STENCIL_BUFFER_BIT,
		}
    }

    fn from_gl_enum(value: u32) -> Option<Self> {
        match value {
			gl::COLOR_BUFFER_BIT => Some(ClearField::Color),
			gl::DEPTH_BUFFER_BIT => Some(ClearField::Depth),
			gl::STENCIL_BUFFER_BIT => Some(ClearField::Stencil),
			_ => None,
		}    
    }
}

pub struct Renderer;

impl Renderer {
    pub(super) fn new() -> Self {
        Self
    }

    pub fn clear_color(&self, color: &glm::Vec4) {
        unsafe {
            gl::ClearColor(color[0], color[1], color[2], color[3]);
        }
    }

    pub fn clear(&self, fields: &[ClearField]) {
        let combined_mask = fields.iter().fold(0u32, |prev, cur| prev | cur.to_gl_enum());
        unsafe {
            gl::Clear(combined_mask);
        }
    }
}