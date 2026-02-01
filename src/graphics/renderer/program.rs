use std::ffi::CStr;
use std::marker::PhantomData;
use crate::graphics::renderer::{GlEnum, Renderer};
use anyhow::Result;

#[derive(Debug, Clone, Copy)]
pub enum ShaderType {
	Vertex,
	TesselationControl,
	TesselationEvaluation,
	Geometry,
	Fragment,
	Compute
}

impl GlEnum for ShaderType {
	fn from_gl_enum(value: u32) -> Option<Self> {
		match value {
			gl::VERTEX_SHADER => Some(Self::Vertex),
			gl::TESS_CONTROL_SHADER => Some(Self::TesselationControl),
			gl::TESS_EVALUATION_SHADER => Some(Self::TesselationEvaluation),
			gl::GEOMETRY_SHADER => Some(Self::Geometry),
			gl::FRAGMENT_SHADER => Some(Self::Fragment),
			gl::COMPUTE_SHADER => Some(Self::Compute),
			_ => None,
		}
	}

	fn to_gl_enum(&self) -> u32 {
		match self {
			Self::Vertex => gl::VERTEX_SHADER,
			Self::TesselationControl => gl::TESS_CONTROL_SHADER,
			Self::TesselationEvaluation => gl::TESS_EVALUATION_SHADER,
			Self::Geometry => gl::GEOMETRY_SHADER,
			Self::Fragment => gl::FRAGMENT_SHADER,
			Self::Compute => gl::COMPUTE_SHADER,
		}
	}
}

pub struct Program<'a> {
	id: u32,
	_marker: PhantomData<&'a Renderer>,
}

impl<'a> Program<'a> {
	pub fn new(_renderer: &'a Renderer, sources: &[(&CStr, ShaderType)]) -> Result<Self> {
        unsafe {
            let program_id = gl::CreateProgram();
            let mut shader_ids = Vec::with_capacity(sources.len());

            for (source, shader_type) in sources {
                match Self::compile_shader(source, shader_type) {
                    Ok(id) => {
                        shader_ids.push(id);
                        gl::AttachShader(program_id, id);
                    }
                    Err(e) => {
                        for id in shader_ids { gl::DeleteShader(id); }
                        gl::DeleteProgram(program_id);
                        return Err(e);
                    }
                }
            }

            gl::LinkProgram(program_id);

            let mut success = gl::FALSE as gl::types::GLint;
            gl::GetProgramiv(program_id, gl::LINK_STATUS, &mut success);

            if success == gl::FALSE as gl::types::GLint {
                let mut len = 0;
                gl::GetProgramiv(program_id, gl::INFO_LOG_LENGTH, &mut len);
                let mut buffer = vec![0u8; len as usize];
                gl::GetProgramInfoLog(program_id, len, std::ptr::null_mut(), buffer.as_mut_ptr() as *mut i8);
                
                let error_msg = String::from_utf8_lossy(&buffer).to_string();
                
                for id in shader_ids { gl::DeleteShader(id); }
                gl::DeleteProgram(program_id);
                
                anyhow::bail!(error_msg);
            }

            for id in shader_ids {
                gl::DetachShader(program_id, id);
                gl::DeleteShader(id);
            }

            return Ok(Self { id: program_id, _marker: PhantomData });
        }
    }

    fn compile_shader(source: &CStr, shader_type: &ShaderType) -> Result<u32> {
        unsafe {
            let id = gl::CreateShader(shader_type.to_gl_enum());
            gl::ShaderSource(id, 1, &source.as_ptr(), std::ptr::null());
            gl::CompileShader(id);

            let mut success = gl::FALSE as gl::types::GLint;
            gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success);

            if success == gl::FALSE as gl::types::GLint {
                let mut len = 0;
                gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);
                let mut buffer = vec![0u8; len as usize];
                gl::GetShaderInfoLog(id, len, std::ptr::null_mut(), buffer.as_mut_ptr() as *mut i8);
                
                let error_msg = String::from_utf8_lossy(&buffer).to_string();
                gl::DeleteShader(id);
                
				anyhow::bail!("Compilation error ({:?}) {}", shader_type, error_msg);
            }
            
			return Ok(id);
        }
    }

	pub fn get_id(&self) -> u32 {
		self.id
	}

	pub fn bind(&self) {
		unsafe { gl::UseProgram(self.id); }
	}
}

impl<'a> Drop for Program<'a> {
	fn drop(&mut self) {
		unsafe { gl::DeleteProgram(self.id); }
	}
}
