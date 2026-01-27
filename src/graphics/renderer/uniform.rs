use crate::graphics::renderer::program::Program;
use nalgebra_glm as glm;

pub trait UniformValue {
    fn set_uniform(&self, location: i32);
}

impl UniformValue for i32 {
	fn set_uniform(&self, location: i32) {
		unsafe { gl::Uniform1i(location, *self); }
	}
}

impl UniformValue for f32 {
	fn set_uniform(&self, location: i32) {
		unsafe { gl::Uniform1f(location, *self); }
	}
}

impl UniformValue for glm::Vec2 {
	fn set_uniform(&self, location: i32) {
		unsafe { gl::Uniform2f(location, self[0], self[1]); }
	}
}

impl UniformValue for glm::Vec3 {
	fn set_uniform(&self, location: i32) {
		unsafe { gl::Uniform3f(location, self[0], self[1], self[2]); }
	}
}

impl UniformValue for glm::Mat4 {
	fn set_uniform(&self, location: i32) {
		unsafe { gl::UniformMatrix4fv(location, 1, gl::FALSE, self.as_ptr()); }
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

	pub fn set(&self, value: &T) {
		value.set_uniform(self.location);
	}
}