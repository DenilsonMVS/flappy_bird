use std::{marker::PhantomData, os::raw::c_void};
use crate::graphics::renderer::{Renderer, vertex_array_object::{FieldType, StaticVertexLayout, VertexLayout}};

pub trait GenericBuffer: VertexLayout {
    fn get_id(&self) -> u32;
}

pub struct StaticBuffer<'a, T: StaticVertexLayout> {
    id: u32,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: StaticVertexLayout> VertexLayout for StaticBuffer<'a, T> {
    fn get_fields(&self) -> &'static [FieldType] {
        T::get_fields()
    }
    fn get_stride(&self) -> i32 {
        T::get_stride()
    }
    fn get_divisor(&self) -> u32 {
		T::get_divisor()
	}
}

impl<'a, T: StaticVertexLayout> GenericBuffer for StaticBuffer<'a, T> {
    fn get_id(&self) -> u32 {
        self.id
    }
}

impl<'a, T: StaticVertexLayout> Drop for StaticBuffer<'a, T> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.id);
        }
    }
}

impl<'a, T: StaticVertexLayout> StaticBuffer<'a, T> {
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
            _marker: PhantomData
        };
    }
}


pub struct DynamicBuffer<'a, T: StaticVertexLayout> {
    id: u32,
    ptr: *mut T,
    index: usize,
    sync_obj: gl::types::GLsync,
    _marker: PhantomData<&'a Renderer>,
}

impl<'a, T: StaticVertexLayout> VertexLayout for DynamicBuffer<'a, T> {
    fn get_fields(&self) -> &'static [FieldType] {
        T::get_fields()
    }
    fn get_stride(&self) -> i32 {
        T::get_stride()
    }
    fn get_divisor(&self) -> u32 {
		T::get_divisor()
	}
}

impl<'a, T: StaticVertexLayout> GenericBuffer for DynamicBuffer<'a, T> {
    fn get_id(&self) -> u32 {
        self.id
    }
}

impl<'a, T: StaticVertexLayout> Drop for DynamicBuffer<'a, T> {
    fn drop(&mut self) {
        unsafe {
            if !self.sync_obj.is_null() {
                gl::DeleteSync(self.sync_obj);
            }
            gl::UnmapNamedBuffer(self.id);
            gl::DeleteBuffers(1, &self.id);
        }
    }
}

impl<'a, T: StaticVertexLayout> DynamicBuffer<'a, T> {
    pub fn new(_renderer: &'a Renderer, size: usize) -> Self {
        let mut id = 0u32;
        let byte_size = (std::mem::size_of::<T>() * size) as isize;

        let storage_flags = gl::MAP_WRITE_BIT | gl::MAP_PERSISTENT_BIT | gl::MAP_COHERENT_BIT | gl::DYNAMIC_STORAGE_BIT;
        let map_flags = gl::MAP_WRITE_BIT | gl::MAP_PERSISTENT_BIT | gl::MAP_FLUSH_EXPLICIT_BIT;

        let mapped_ptr;
        unsafe {
            gl::CreateBuffers(1, &mut id);
            gl::NamedBufferStorage(id, byte_size, std::ptr::null(), storage_flags);
            mapped_ptr = gl::MapNamedBufferRange(id, 0, byte_size, map_flags) as *mut T;
        }

        Self {
            id,
            ptr: mapped_ptr,
            index: 0,
            sync_obj: std::ptr::null(),
            _marker: PhantomData,            
        }
    }

    pub fn write(&mut self, value: &T) {
        unsafe {
            let dst = self.ptr.add(self.index);
            std::ptr::copy_nonoverlapping(value as *const T, dst, 1);
            self.index += 1;
        }
    }

    pub fn reset_index(&mut self) {
        self.index = 0;
    }

    pub fn lock(&mut self) {
        unsafe {
            if !self.sync_obj.is_null() {
                gl::DeleteSync(self.sync_obj);
            }

            self.sync_obj = gl::FenceSync(gl::SYNC_GPU_COMMANDS_COMPLETE, 0);
        }
    }

    pub fn get_len(&self) -> usize {
        self.index
    }

    pub fn wait(&mut self) {
        unsafe {
            if !self.sync_obj.is_null() {
                gl::ClientWaitSync(self.sync_obj, gl::SYNC_FLUSH_COMMANDS_BIT, 1 << 30);
            }
        }
    }

    pub fn flush(&self) {
        unsafe {
            let length = (self.index * std::mem::size_of::<T>()) as isize;
            gl::FlushMappedNamedBufferRange(self.id, 0, length);
        }
    }
}
