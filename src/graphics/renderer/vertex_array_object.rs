use std::{marker::PhantomData};

use crate::graphics::renderer::{self, Bindable, GlEnum, buffer::BindableLayout, types::{GlType, GlTypeEnum}};


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
}

pub struct VertexArrayObject {
    id: u32,
    _marker: PhantomData<renderer::Renderer>,
}

impl VertexArrayObject {
    pub fn new(buffers: &[&dyn BindableLayout]) -> Self {
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
            let mut current_offset = 0usize;

            for field in fields {
                unsafe {
                    gl::EnableVertexAttribArray(attribute);
                    gl::VertexAttribPointer(
                        attribute,
                        field.size,
                        field.gl_type.to_gl_enum(),
                        field.normalized as u8,
                        stride,
                        current_offset as *const std::ffi::c_void
                    );
                }

				current_offset += field.gl_type.get_size() * (field.size as usize);
                attribute += 1;
            }
        }

        return Self { id, _marker: PhantomData };
    }

    pub fn draw(&self, count: i32) {
        self.bind();
        unsafe {
            gl::DrawArrays(gl::TRIANGLES, 0, count);
        }
    }
}

impl Bindable for VertexArrayObject {
    fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.id);
        }
    }
}