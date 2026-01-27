use std::{marker::PhantomData, os::raw::c_void, ptr::null};

use crate::graphics::renderer::{self, Bindable, GlEnum, drawable::Drawable, vertex_array_object::{FieldType, StaticVertexLayout, VertexLayout}};


pub enum BufferUsage {
    StreamDraw,
    StreamRead,
    StreamCopy,
    StaticDraw,
    StaticRead,
    StaticCopy,
    DynamicDraw,
    DynamicRead,
    DynamicCopy
}

impl GlEnum for BufferUsage {
	fn to_gl_enum(&self) -> u32 {
		match self {
			BufferUsage::StreamDraw  => gl::STREAM_DRAW,
			BufferUsage::StreamRead  => gl::STREAM_READ,
			BufferUsage::StreamCopy  => gl::STREAM_COPY,
			BufferUsage::StaticDraw  => gl::STATIC_DRAW,
			BufferUsage::StaticRead  => gl::STATIC_READ,
			BufferUsage::StaticCopy  => gl::STATIC_COPY,
			BufferUsage::DynamicDraw => gl::DYNAMIC_DRAW,
			BufferUsage::DynamicRead => gl::DYNAMIC_READ,
			BufferUsage::DynamicCopy => gl::DYNAMIC_COPY,
		}
	}

	fn from_gl_enum(value: u32) -> Option<Self> {
		match value {
			gl::STREAM_DRAW  => Some(BufferUsage::StreamDraw),
			gl::STREAM_READ  => Some(BufferUsage::StreamRead),
			gl::STREAM_COPY  => Some(BufferUsage::StreamCopy),
			gl::STATIC_DRAW  => Some(BufferUsage::StaticDraw),
			gl::STATIC_READ  => Some(BufferUsage::StaticRead),
			gl::STATIC_COPY  => Some(BufferUsage::StaticCopy),
			gl::DYNAMIC_DRAW => Some(BufferUsage::DynamicDraw),
			gl::DYNAMIC_READ => Some(BufferUsage::DynamicRead),
			gl::DYNAMIC_COPY => Some(BufferUsage::DynamicCopy),
			_ => None,
		}
	}
}

pub trait BindableLayout: VertexLayout + Bindable {}
impl<T: VertexLayout + Bindable> BindableLayout for T {}

pub struct VertexBuffer<'a, T: StaticVertexLayout> {
    id: u32,
    divisor: u32,
    _marker: PhantomData<&'a renderer::Renderer>,
    _type_marker: PhantomData<T>,
}

impl<'a, T: StaticVertexLayout> VertexBuffer<'a, T> {
    pub fn new(_renderer: &'a renderer::Renderer) -> Self {
        let mut id = 0u32;
        unsafe {
            gl::GenBuffers(1, &mut id);
        }

        return Self {
            id,
            divisor: 0,
            _marker: PhantomData,
            _type_marker: PhantomData,
        };
    }

    pub fn set_instanced(mut self, divisor: u32) -> Self {
		self.divisor = divisor;
		return self;
	}

    pub fn set_data(&mut self, data: &[T], usage: BufferUsage) {
        self.bind();
        unsafe {
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (data.len() * std::mem::size_of::<T>()) as isize,
                data.as_ptr() as *const c_void,
                usage.to_gl_enum()
            );
        }
    }

    pub fn set_sub_data(&mut self, data: &[T], index_offset: usize) {
        self.bind();
        unsafe {
            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                (index_offset * std::mem::size_of::<T>()) as isize,
                (data.len() * std::mem::size_of::<T>()) as isize,
                data.as_ptr() as *const c_void
            );
        }
    }
}

impl<'a, T: StaticVertexLayout> Drop for VertexBuffer<'a, T> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &mut self.id);
        }
    }
}

impl<'a, T: StaticVertexLayout> Bindable for VertexBuffer<'a, T> {
    fn bind(&self) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.id);
        }
    }
}

impl<'a, T: StaticVertexLayout> VertexLayout for VertexBuffer<'a, T> {
    fn get_fields(&self) -> &'static [FieldType] {
        T::get_fields()
    }
    fn get_stride(&self) -> i32 {
        T::get_stride()
    }
    fn get_divisor(&self) -> u32 {
		self.divisor
	}
}


pub trait IndexType {
	fn to_gl_enum() -> u32;
}

impl IndexType for u8  { fn to_gl_enum() -> u32 { gl::UNSIGNED_BYTE  } }
impl IndexType for u16 { fn to_gl_enum() -> u32 { gl::UNSIGNED_SHORT } }
impl IndexType for u32 { fn to_gl_enum() -> u32 { gl::UNSIGNED_INT   } }

pub struct IndexBuffer<'a, T: IndexType> {
	id: u32,
	_marker: PhantomData<&'a renderer::Renderer>,
	_type_marker: PhantomData<T>,
}

impl<'a, T: IndexType> IndexBuffer<'a, T> {
	pub fn new(_renderer: &'a renderer::Renderer) -> Self {
		let mut id = 0u32;
		unsafe {
			gl::GenBuffers(1, &mut id);
		}

		Self {
			id,
			_marker: PhantomData,
			_type_marker: PhantomData,
		}
	}

	pub fn set_data(&mut self, data: &[T], usage: BufferUsage) {
		self.bind();
		unsafe {
			gl::BufferData(
				gl::ELEMENT_ARRAY_BUFFER,
				(data.len() * std::mem::size_of::<T>()) as isize,
				data.as_ptr() as *const c_void,
				usage.to_gl_enum(),
			);
		}
	}
}

impl<'a, T: IndexType> Drop for IndexBuffer<'a, T> {
	fn drop(&mut self) {
		unsafe {
			gl::DeleteBuffers(1, &self.id);
		}
	}
}

impl<'a, T: IndexType> Bindable for IndexBuffer<'a, T> {
    fn bind(&self) {
        unsafe {
			gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.id);
		}
    }
}

impl <'a, T: IndexType> Drawable for IndexBuffer<'a, T> {
    fn draw(&self, count: i32, draw_mode: renderer::drawable::DrawMode) {
        self.bind();
        unsafe {
            gl::DrawElements(draw_mode.to_gl_enum(), count, T::to_gl_enum(), null());
        }
    }
}