use std::{marker::PhantomData};

use crate::graphics::renderer::{GlEnum, Renderer, buffer::GenericBuffer, drawable::DrawMode, types::{GlType, GlTypeEnum}};


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
    fn get_divisor() -> u32;
}

pub trait VertexLayout {
    fn get_fields(&self) -> &'static [FieldType];
    fn get_stride(&self) -> i32;
    fn get_divisor(&self) -> u32 { 0 }
}

pub struct VertexArrayObject<'a> {
    id: u32,
    _marker: PhantomData<&'a Renderer>,
}

impl<'a> VertexArrayObject<'a> {
    pub fn new(_renderer: &'a Renderer, buffers: &[& (dyn GenericBuffer + 'a)]) -> Self {
        let mut id: u32 = 0u32;
        unsafe {
            gl::CreateVertexArrays(1, &mut id);
        }

        let mut attribute_index = 0;
        for (binding_index, buffer) in buffers.iter().enumerate() {
            let binding_index = binding_index as u32;
            let fields = buffer.get_fields();
            let stride = buffer.get_stride();
            let divisor = buffer.get_divisor();
            let mut relative_offset = 0u32;

            unsafe {
                gl::VertexArrayVertexBuffer(
                    id,
                    binding_index,
                    buffer.get_id(),
                    0,
                    stride as i32
                );

                gl::VertexArrayBindingDivisor(id, binding_index, divisor);
            }

            for field in fields {
                unsafe {
                    gl::EnableVertexArrayAttrib(id, attribute_index);

                    match field.gl_type {
                        GlTypeEnum::Int | GlTypeEnum::UnsignedInt => {
                            gl::VertexArrayAttribIFormat(
                                id,
                                attribute_index,
                                field.size,
                                field.gl_type.to_gl_enum(),
                                relative_offset
                            );
                        }
                        _ => {
                            gl::VertexArrayAttribFormat(
                                id,
                                attribute_index,
                                field.size,
                                field.gl_type.to_gl_enum(),
                                field.normalized as u8,
                                relative_offset
                            );
                        }
                    }

                    gl::VertexArrayAttribBinding(id, attribute_index, binding_index);
                }

                relative_offset += (field.gl_type.get_size() * (field.size as usize)) as u32;
                attribute_index += 1;
            }
        }

        return Self { id, _marker: PhantomData };
    }

    pub fn draw_instanced(&self, vertex_count: i32, instance_count: i32, draw_mode: DrawMode) {
		unsafe {
            gl::BindVertexArray(self.id);
			gl::DrawArraysInstanced(
				draw_mode.to_gl_enum(),
				0,
				vertex_count,
				instance_count
			);
		}
	}

    pub fn draw(&self, vertex_count: i32, draw_mode: DrawMode) {
        unsafe {
            gl::BindVertexArray(self.id);            
            gl::DrawArrays(
                draw_mode.to_gl_enum(),
                0,
                vertex_count
            );
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