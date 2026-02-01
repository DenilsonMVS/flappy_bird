
use nalgebra_glm as glm;

use crate::graphics::renderer::GlEnum;

pub enum GlTypeEnum {
	Byte,
	UnsignedByte,
	Short,
	UnsignedShort,
	Int,
	UnsignedInt,
	Float,
	Double
}

impl GlTypeEnum {
	pub const fn get_size(&self) -> usize {
		match self {
			GlTypeEnum::Byte | GlTypeEnum::UnsignedByte => 1,
			GlTypeEnum::Short | GlTypeEnum::UnsignedShort => 2,
			GlTypeEnum::Int | GlTypeEnum::UnsignedInt | GlTypeEnum::Float => 4,
			GlTypeEnum::Double => 8,
		}
	}

	pub const fn is_integer(&self) -> bool {
		matches!(*self,
			Self::Byte |
			Self::UnsignedByte |
			Self::Short |
			Self::UnsignedShort |
			Self::Int |
			Self::UnsignedInt)
	}
}

impl GlEnum for GlTypeEnum {
	fn from_gl_enum(value: u32) -> Option<Self> {
		match value {
			gl::BYTE => Some(GlTypeEnum::Byte),
			gl::UNSIGNED_BYTE => Some(GlTypeEnum::UnsignedByte),
			gl::SHORT => Some(GlTypeEnum::Short),
			gl::UNSIGNED_SHORT => Some(GlTypeEnum::UnsignedShort),
			gl::INT => Some(GlTypeEnum::Int),
			gl::UNSIGNED_INT => Some(GlTypeEnum::UnsignedInt),
			gl::FLOAT => Some(GlTypeEnum::Float),
			gl::DOUBLE => Some(GlTypeEnum::Double),
			_ => None,
		}
	}

	fn to_gl_enum(&self) -> u32 {
		match self {
			GlTypeEnum::Byte => gl::BYTE,
			GlTypeEnum::UnsignedByte => gl::UNSIGNED_BYTE,
			GlTypeEnum::Short => gl::SHORT,
			GlTypeEnum::UnsignedShort => gl::UNSIGNED_SHORT,
			GlTypeEnum::Int => gl::INT,
			GlTypeEnum::UnsignedInt => gl::UNSIGNED_INT,
			GlTypeEnum::Float => gl::FLOAT,
			GlTypeEnum::Double => gl::DOUBLE,
		}
	}
}


pub trait GlType {
	const ENUM: GlTypeEnum;
	const FIELD_TYPE_SIZE: i32;
}

impl GlType for u8 {
	const ENUM: GlTypeEnum = GlTypeEnum::UnsignedByte;
    const FIELD_TYPE_SIZE: i32 = 1;
}

impl GlType for glm::U8Vec3 {
	const ENUM: GlTypeEnum = GlTypeEnum::UnsignedByte;
    const FIELD_TYPE_SIZE: i32 = 3;
}

impl GlType for glm::U8Vec4 {
	const ENUM: GlTypeEnum = GlTypeEnum::UnsignedByte;
    const FIELD_TYPE_SIZE: i32 = 4;
}

impl GlType for glm::U16Vec2 {
	const ENUM: GlTypeEnum = GlTypeEnum::UnsignedShort;
    const FIELD_TYPE_SIZE: i32 = 2;
}

impl GlType for glm::I16Vec2 {
	const ENUM: GlTypeEnum = GlTypeEnum::Short;
    const FIELD_TYPE_SIZE: i32 = 2;
}

impl GlType for f32 {
	const ENUM: GlTypeEnum = GlTypeEnum::Float;
    const FIELD_TYPE_SIZE: i32 = 1;
}

impl GlType for glm::Vec2 {
	const ENUM: GlTypeEnum = GlTypeEnum::Float;
    const FIELD_TYPE_SIZE: i32 = 2;
}

impl GlType for u32 {
	const ENUM: GlTypeEnum = GlTypeEnum::UnsignedInt;
    const FIELD_TYPE_SIZE: i32 = 1;
}
