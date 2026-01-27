use std::{marker::PhantomData};

use crate::graphics::renderer::{Bindable, GlEnum, buffer::BindableLayout, drawable::{DrawMode, Drawable}, types::{GlType, GlTypeEnum}};


pub struct FieldType {
    normalized: bool,
    gl_type: GlTypeEnum,
    size: i32,
}

impl FieldType {
    pub const fn new<T: GlType>(normalized: bool) -> Self {
        Self {
            normalized,
            size: T::FIELD_TYPE_SIZE,
            gl_type: T::ENUM
        }
    }
}

pub trait StaticVertexLayout: Sized {
    fn get_fields() -> &'static [FieldType];
    fn get_stride() -> i32 {
        std::mem::size_of::<Self>() as i32
    }
}

pub trait VertexLayout {
    fn get_fields(&self) -> &'static [FieldType];
    fn get_stride(&self) -> i32;
    fn get_divisor(&self) -> u32 { 0 }
}

pub struct VertexArrayObject<'a> {
    id: u32,
    _marker: PhantomData<&'a dyn BindableLayout>,
}

impl<'a> VertexArrayObject<'a> {
    pub fn new(buffers: &[&'a dyn BindableLayout]) -> Self {
        let mut id: u32 = 0u32;
        unsafe {
            gl::GenVertexArrays(1, &mut id);
            gl::BindVertexArray(id); 
        }

        let mut attribute = 0;

        for buffer in buffers {
            buffer.bind();
            let fields = buffer.get_fields();
            let stride = buffer.get_stride();
            let divisor = buffer.get_divisor();
            let mut current_offset = 0usize;

            for field in fields {
                unsafe {
                    gl::EnableVertexAttribArray(attribute);

                    match field.gl_type {
                        GlTypeEnum::Int | GlTypeEnum::UnsignedInt => {
                            gl::VertexAttribIPointer(
                                attribute,
                                field.size,
                                field.gl_type.to_gl_enum(),
                                stride,
                                current_offset as *const std::ffi::c_void
                            );
                        }
                        _ => {
                            gl::VertexAttribPointer(
                                attribute,
                                field.size,
                                field.gl_type.to_gl_enum(),
                                field.normalized as u8,
                                stride,
                                current_offset as *const std::ffi::c_void
                            );
                        }
                    }

                    if divisor > 0 {
                        gl::VertexAttribDivisor(attribute, divisor);
                    }
                }

				current_offset += field.gl_type.get_size() * (field.size as usize);
                attribute += 1;
            }
        }

        return Self { id, _marker: PhantomData };
    }

    pub fn draw_instanced(&self, vertex_count: i32, instance_count: i32, draw_mode: DrawMode) {
		self.bind();
		unsafe {
			gl::DrawArraysInstanced(
				draw_mode.to_gl_enum(),
				0,
				vertex_count,
				instance_count
			);
		}
	}
}

impl<'a> Bindable for VertexArrayObject<'a> {
    fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.id);
        }
    }
}

impl<'a> Drawable for VertexArrayObject<'a> {
    fn draw(&self, count: i32, draw_mode: DrawMode) {
        unsafe {
            gl::DrawArrays(draw_mode.to_gl_enum(), 0, count);
        }
    }
}

impl<'a> Drop for VertexArrayObject<'a> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &mut self.id);
        }
    }
}