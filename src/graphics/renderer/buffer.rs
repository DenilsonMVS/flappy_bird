use std::{marker::PhantomData, os::raw::c_void, ptr::null};

use crate::graphics::renderer::{GlEnum, Renderer, vertex_array_object::{FieldType, StaticVertexLayout, VertexLayout}};

pub enum ResizableBufferUsage {
    StreamDraw,
    StreamRead,
    StreamCopy,
    DynamicDraw,
    DynamicRead,
    DynamicCopy,
}

impl GlEnum for ResizableBufferUsage {
	fn to_gl_enum(&self) -> u32 {
		match self {
			Self::StreamDraw  => gl::STREAM_DRAW,
			Self::StreamRead  => gl::STREAM_READ,
			Self::StreamCopy  => gl::STREAM_COPY,
			Self::DynamicDraw => gl::DYNAMIC_DRAW,
			Self::DynamicRead => gl::DYNAMIC_READ,
			Self::DynamicCopy => gl::DYNAMIC_COPY,
		}
	}

	fn from_gl_enum(value: u32) -> Option<Self> {
		match value {
			gl::STREAM_DRAW  => Some(Self::StreamDraw),
			gl::STREAM_READ  => Some(Self::StreamRead),
			gl::STREAM_COPY  => Some(Self::StreamCopy),
			gl::DYNAMIC_DRAW => Some(Self::DynamicDraw),
			gl::DYNAMIC_READ => Some(Self::DynamicRead),
			gl::DYNAMIC_COPY => Some(Self::DynamicCopy),
			_ => None,
		}
	}
}

pub trait GenericBuffer: VertexLayout {
    fn get_id(&self) -> u32;
}

pub trait StorageStrategy {}

pub struct Static {}
impl StorageStrategy for Static {}

pub struct Dynamic {}
impl StorageStrategy for Dynamic {}

pub struct Resizable {}
impl StorageStrategy for Resizable {}

pub struct Buffer<'a, T: StaticVertexLayout, S: StorageStrategy> {
    id: u32,
    divisor: u32,
    _marker: PhantomData<&'a (S, T)>,
}

impl<'a, T: StaticVertexLayout, S: StorageStrategy> VertexLayout for Buffer<'a, T, S> {
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

impl<'a, T: StaticVertexLayout, S: StorageStrategy> GenericBuffer for Buffer<'a, T, S> {
    fn get_id(&self) -> u32 {
        self.id
    }
}

impl<'a, T: StaticVertexLayout, S: StorageStrategy> Buffer<'a, T, S> {
    pub fn set_divisor(mut self, divisor: u32) -> Self {
        self.divisor = divisor;
        return self;
    }
}

impl<'a, T: StaticVertexLayout, S: StorageStrategy> Drop for Buffer<'a, T, S> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.id);
        }
    }
}

impl<'a, T: StaticVertexLayout> Buffer<'a, T, Static> {
    pub fn new(_renderer: &'a Renderer, data: &[T]) -> Self {
        let mut id = 0u32;
        unsafe {
            gl::CreateBuffers(1, &mut id);
            gl::NamedBufferStorage(
                id,
                (std::mem::size_of::<T>() * data.len()) as isize,
                data.as_ptr() as *const c_void,
                0
            );
        }

        return Self {
            id,
            divisor: 0,
            _marker: PhantomData
        };
    }
}

impl<'a, T: StaticVertexLayout> Buffer<'a, T, Dynamic> {
    pub fn new(_renderer: &'a Renderer, size: usize) -> Self {
        let mut id = 0u32;

        unsafe {
            gl::CreateBuffers(1, &mut id);
            gl::NamedBufferStorage(
                id, 
                (std::mem::size_of::<T>() * size) as isize, 
                std::ptr::null(), 
                gl::DYNAMIC_STORAGE_BIT
            );
        }

        Self {
            id,
            divisor: 0,
            _marker: PhantomData,
        }
    }

    pub fn set_sub_data(&mut self, data: &[T], index_offset: usize) {
        unsafe {
            gl::NamedBufferSubData(
                self.id,
                (index_offset * std::mem::size_of::<T>()) as isize,
                (data.len() * std::mem::size_of::<T>()) as isize,
                data.as_ptr() as *const c_void
            );
        }
    }
}

impl<'a, T: StaticVertexLayout> Buffer<'a, T, Resizable> {
    pub fn new(_renderer: &'a Renderer) -> Self {
        let mut id = 0u32;
        unsafe {
            gl::CreateBuffers(1, &mut id);
        }

        return Self {
            id,
            divisor: 0,
            _marker: PhantomData
        };
    }

    pub fn reserve_data(&mut self, amount: usize, usage: ResizableBufferUsage) {
        unsafe {
            gl::NamedBufferData(
                self.id,
                (std::mem::size_of::<T>() * amount) as isize,
                null(),
                usage.to_gl_enum()
            );
        }
    }

    pub fn set_data(&mut self, data: &[T], usage: ResizableBufferUsage) {
        unsafe {
            gl::NamedBufferData(
                self.id,
                (data.len() * std::mem::size_of::<T>()) as isize,
                data.as_ptr() as *const c_void,
                usage.to_gl_enum()
            );
        }
    }

    pub fn set_sub_data(&mut self, data: &[T], index_offset: usize) {
        unsafe {
            gl::NamedBufferSubData(
                self.id,
                (index_offset * std::mem::size_of::<T>()) as isize,
                (data.len() * std::mem::size_of::<T>()) as isize,
                data.as_ptr() as *const c_void
            );
        }
    }
}
