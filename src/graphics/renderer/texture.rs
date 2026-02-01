use std::{marker::PhantomData, os::raw::c_void};

use image::{GenericImageView, load_from_memory};
use nalgebra_glm as glm;
use crate::graphics::renderer::{GlEnum, Renderer};
use anyhow::Result;

pub enum MagFiltering {
    Nearest,
    Linear
}

impl GlEnum for MagFiltering {
    fn from_gl_enum(value: u32) -> Option<Self> {
        match value {
			gl::NEAREST => Some(Self::Nearest),
			gl::LINEAR => Some(Self::Linear),
			_ => None,
		}
    }

    fn to_gl_enum(&self) -> u32 {
        match self {
			Self::Nearest => gl::NEAREST,
			Self::Linear => gl::LINEAR,
		}
    }
}

pub enum MinFiltering {
    Nearest,
	Linear,
	NearestMipmapNearest,
	LinearMipmapNearest,
	NearestMipmapLinear,
	LinearMipmapLinear,
}

impl MinFiltering {
    fn uses_mipmap(&self) -> bool {
        match self {
			Self::Nearest | Self::Linear => false,
			_ => true,
		}
    }
}

impl GlEnum for MinFiltering {
    fn from_gl_enum(value: u32) -> Option<Self> {
		match value {
			gl::NEAREST => Some(Self::Nearest),
			gl::LINEAR => Some(Self::Linear),
			gl::NEAREST_MIPMAP_NEAREST => Some(Self::NearestMipmapNearest),
			gl::LINEAR_MIPMAP_NEAREST => Some(Self::LinearMipmapNearest),
			gl::NEAREST_MIPMAP_LINEAR => Some(Self::NearestMipmapLinear),
			gl::LINEAR_MIPMAP_LINEAR => Some(Self::LinearMipmapLinear),
			_ => None,
		}
	}

	fn to_gl_enum(&self) -> u32 {
		match self {
			Self::Nearest => gl::NEAREST,
			Self::Linear => gl::LINEAR,
			Self::NearestMipmapNearest => gl::NEAREST_MIPMAP_NEAREST,
			Self::LinearMipmapNearest => gl::LINEAR_MIPMAP_NEAREST,
			Self::NearestMipmapLinear => gl::NEAREST_MIPMAP_LINEAR,
			Self::LinearMipmapLinear => gl::LINEAR_MIPMAP_LINEAR,
		}
	}
}

pub enum TextureWrap {
    Repeat,
	MirroredRepeat,
	ClampToEdge,
	ClampToBorder,
}

impl GlEnum for TextureWrap {
    fn from_gl_enum(value: u32) -> Option<Self> {
		match value {
			gl::REPEAT => Some(Self::Repeat),
			gl::MIRRORED_REPEAT => Some(Self::MirroredRepeat),
			gl::CLAMP_TO_EDGE => Some(Self::ClampToEdge),
			gl::CLAMP_TO_BORDER => Some(Self::ClampToBorder),
			_ => None,
		}
	}

	fn to_gl_enum(&self) -> u32 {
		match self {
			Self::Repeat => gl::REPEAT,
			Self::MirroredRepeat => gl::MIRRORED_REPEAT,
			Self::ClampToEdge => gl::CLAMP_TO_EDGE,
			Self::ClampToBorder => gl::CLAMP_TO_BORDER,
		}
	}
}

pub struct Texture<'a> {
    id: u32,
    _marker: PhantomData<&'a Renderer>,
}

impl<'a> Texture<'a> {
    pub fn from_image_bytes(
        _renderer: &'a Renderer,
        bytes: &[u8],
        mag_filter: MagFiltering,
        min_filter: MinFiltering,
        wrap: TextureWrap
    ) -> Result<Self> {
        let img = load_from_memory(bytes)?;
        let (width, height) = img.dimensions();

        let (internal_format, data_format, raw_data) = match img {
            image::DynamicImage::ImageLuma8(buf) => (gl::RED, gl::RED, buf.into_raw()),
            image::DynamicImage::ImageLumaA8(buf) => (gl::RG, gl::RG, buf.into_raw()),
            image::DynamicImage::ImageRgb8(buf) => (gl::RGB8, gl::RGB, buf.into_raw()),
            image::DynamicImage::ImageRgba8(buf) => (gl::RGBA8, gl::RGBA, buf.into_raw()),
            _ => anyhow::bail!("Unsupported format"), 
        };

        let mut id = 0u32;

        unsafe {
            gl::GenTextures(1, &mut id);
			gl::BindTexture(gl::TEXTURE_2D, id);

            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                internal_format as i32,
                width as i32,
                height as i32,
                0,
                data_format,
                gl::UNSIGNED_BYTE,
                raw_data.as_ptr() as *const c_void
            );

            if min_filter.uses_mipmap() {
                gl::GenerateMipmap(gl::TEXTURE_2D);
            }

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, min_filter.to_gl_enum() as i32);
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, mag_filter.to_gl_enum() as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, wrap.to_gl_enum() as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, wrap.to_gl_enum() as i32);
        }

        return Ok(Self { id, _marker: PhantomData });
    }

    pub fn bind_to_unit(&self, unit: u32) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + unit);
            gl::BindTexture(gl::TEXTURE_2D, self.id);
        }
    }

    pub fn from_font_raw(_renderer: &'a Renderer, bytes: &[glm::U8Vec3], width: usize) -> Self {
        let mut id = 0u32;

        let height = bytes.len() / width;

        unsafe {
            gl::GenTextures(1, &mut id);
			gl::BindTexture(gl::TEXTURE_2D, id);

            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGB8 as i32,
                width as i32,
                height as i32,
                0,
                gl::RGB,
                gl::UNSIGNED_BYTE,
                bytes.as_ptr() as *const c_void
            );

            gl::GenerateMipmap(gl::TEXTURE_2D);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32);
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
        }

        return Self {
            id,
            _marker: PhantomData
        };
    }
}

impl<'a> Drop for Texture<'a>  {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.id);
        }
    }
}
