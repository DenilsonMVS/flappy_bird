use crate::graphics::renderer::program::Program;
use nalgebra_glm as glm;

pub trait UniformValue {
    fn set_program_uniform(&self, program_id: u32, location: i32);
}

impl UniformValue for i32 {
	fn set_program_uniform(&self, program_id: u32, location: i32) {
		unsafe { gl::ProgramUniform1i(program_id, location, *self); }
	}
}

impl UniformValue for f32 {
	fn set_program_uniform(&self, program_id: u32, location: i32) {
		unsafe { gl::ProgramUniform1f(program_id, location, *self); }
	}
}

impl UniformValue for glm::Vec2 {
	fn set_program_uniform(&self, program_id: u32, location: i32) {
		unsafe { gl::ProgramUniform2f(program_id, location, self[0], self[1]); }
	}
}

impl UniformValue for glm::Vec3 {
	fn set_program_uniform(&self, program_id: u32, location: i32) {
		unsafe { gl::ProgramUniform3f(program_id, location, self[0], self[1], self[2]); }
	}
}

impl UniformValue for glm::Mat4 {
	fn set_program_uniform(&self, program_id: u32, location: i32) {
		unsafe { gl::ProgramUniformMatrix4fv(program_id, location, 1, gl::FALSE, self.as_ptr()); }
	}
}

impl UniformValue for glm::UVec2 {
	fn set_program_uniform(&self, program_id: u32, location: i32) {
		unsafe { gl::ProgramUniform2ui(program_id, location, self[0], self[1]); }
	}
}

impl UniformValue for u32 {
	fn set_program_uniform(&self, program_id: u32, location: i32) {
		unsafe { gl::ProgramUniform1ui(program_id, location, *self); }
	}
}


pub struct Uniform<T: UniformValue> {
	location: i32,
	_marker: std::marker::PhantomData<T>,
}

impl<T: UniformValue> Uniform<T> {
	pub fn new(program: &Program, name: &std::ffi::CStr) -> Option<Self> {
		let location = unsafe { gl::GetUniformLocation(program.get_id(), name.as_ptr()) };

		if location == -1 {
			None
		} else {
			Some(Self {
				location,
				_marker: std::marker::PhantomData
			})
		}
	}

	pub fn set(&self, program_id: u32, value: &T) {
		value.set_program_uniform(program_id, self.location);
	}
}