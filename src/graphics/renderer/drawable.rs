use crate::graphics::renderer::GlEnum;


pub enum DrawMode {
	Points,
	LineStrip,
	LineLoop,
	Lines,
	TriangleStrip,
	TriangleFan,
	Triangles,
}

impl GlEnum for DrawMode {
	fn from_gl_enum(value: u32) -> Option<Self> {
		match value {
			gl::POINTS         => Some(Self::Points),
			gl::LINE_STRIP     => Some(Self::LineStrip),
			gl::LINE_LOOP      => Some(Self::LineLoop),
			gl::LINES          => Some(Self::Lines),
			gl::TRIANGLE_STRIP => Some(Self::TriangleStrip),
			gl::TRIANGLE_FAN   => Some(Self::TriangleFan),
			gl::TRIANGLES      => Some(Self::Triangles),
			_ => None,
		}
	}

	fn to_gl_enum(&self) -> u32 {
		match self {
			Self::Points        => gl::POINTS,
			Self::LineStrip     => gl::LINE_STRIP,
			Self::LineLoop      => gl::LINE_LOOP,
			Self::Lines         => gl::LINES,
			Self::TriangleStrip => gl::TRIANGLE_STRIP,
			Self::TriangleFan   => gl::TRIANGLE_FAN,
			Self::Triangles     => gl::TRIANGLES,
		}
	}
}
