
use nalgebra_glm as glm;
pub mod buffer;
pub mod vertex_array_object;
pub mod types;
pub mod program;
pub mod drawable;
pub mod texture;
pub mod uniform;
pub mod fonts;

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

pub enum Capability {
    Blend,
	CullFace,
	DepthTest,
	StencilTest,
	ScissorTest,
}

impl GlEnum for Capability {
    fn from_gl_enum(value: u32) -> Option<Self> {
		match value {
			gl::BLEND => Some(Self::Blend),
			gl::CULL_FACE => Some(Self::CullFace),
			gl::DEPTH_TEST => Some(Self::DepthTest),
			gl::STENCIL_TEST => Some(Self::StencilTest),
			gl::SCISSOR_TEST => Some(Self::ScissorTest),
			_ => None,
		}
	}

	fn to_gl_enum(&self) -> u32 {
		match self {
			Self::Blend => gl::BLEND,
			Self::CullFace => gl::CULL_FACE,
			Self::DepthTest => gl::DEPTH_TEST,
			Self::StencilTest => gl::STENCIL_TEST,
			Self::ScissorTest => gl::SCISSOR_TEST,
		}
	}
}


pub enum BlendFactor {
	Zero,
	One,
	SrcColor,
	OneMinusSrcColor,
	DstColor,
	OneMinusDstColor,
	SrcAlpha,
	OneMinusSrcAlpha,
	DstAlpha,
	OneMinusDstAlpha,
}

impl GlEnum for BlendFactor {
	fn from_gl_enum(value: u32) -> Option<Self> {
		match value {
			gl::ZERO => Some(Self::Zero),
			gl::ONE => Some(Self::One),
			gl::SRC_COLOR => Some(Self::SrcColor),
			gl::ONE_MINUS_SRC_COLOR => Some(Self::OneMinusSrcColor),
			gl::DST_COLOR => Some(Self::DstColor),
			gl::ONE_MINUS_DST_COLOR => Some(Self::OneMinusDstColor),
			gl::SRC_ALPHA => Some(Self::SrcAlpha),
			gl::ONE_MINUS_SRC_ALPHA => Some(Self::OneMinusSrcAlpha),
			gl::DST_ALPHA => Some(Self::DstAlpha),
			gl::ONE_MINUS_DST_ALPHA => Some(Self::OneMinusDstAlpha),
			_ => None,
		}
	}

	fn to_gl_enum(&self) -> u32 {
		match self {
			Self::Zero => gl::ZERO,
			Self::One => gl::ONE,
			Self::SrcColor => gl::SRC_COLOR,
			Self::OneMinusSrcColor => gl::ONE_MINUS_SRC_COLOR,
			Self::DstColor => gl::DST_COLOR,
			Self::OneMinusDstColor => gl::ONE_MINUS_DST_COLOR,
			Self::SrcAlpha => gl::SRC_ALPHA,
			Self::OneMinusSrcAlpha => gl::ONE_MINUS_SRC_ALPHA,
			Self::DstAlpha => gl::DST_ALPHA,
			Self::OneMinusDstAlpha => gl::ONE_MINUS_DST_ALPHA,
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

    pub fn enable(&self, capability: Capability) {
        unsafe {
            gl::Enable(capability.to_gl_enum());
        }
    }

    pub fn disable(&self, capability: Capability) {
        unsafe {
            gl::Disable(capability.to_gl_enum());
        }
    }

    pub fn blend_func(&self, s_factor: BlendFactor, d_factor: BlendFactor) {
		unsafe {
			gl::BlendFunc(s_factor.to_gl_enum(), d_factor.to_gl_enum());
		}
	}
}