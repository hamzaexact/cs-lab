use crate::{into_iter::CustomIntoIter, raw_vec::CustomRawVec};
use std::{
    alloc::{Layout, alloc, realloc},
    ptr::NonNull,
};

pub(crate) struct CustomVec<T> {
    buffer: CustomRawVec<T>,
    len: usize, // The number of elements that have been initialized.
}
impl<T> CustomVec<T> {
    pub fn new() -> Self {
        assert_ne!(size_of::<T>(), 0, "We're not ready to handle ZSTs");
        Self {
            buffer: CustomRawVec::new(),
            len: 0,
        }
    }
    pub fn ptr(&self) -> *mut T {
        self.buffer.ptr.as_ptr()
    }

    fn cap(&self) -> usize {
        self.buffer.cap
    }

    pub fn grow(&mut self) {
        let align = align_of::<T>();
        let elem_size = size_of::<T>();
        let layout: Layout = unsafe { Layout::from_size_align_unchecked(elem_size, align) };
        let (new_cap, ptr) = {
            if self.buffer.cap == 0 {
                // unsafe
                unsafe {
                    let ptr = alloc(layout);
                    (1, ptr)
                }
            } else {
                let layout = Layout::array::<T>(self.buffer.cap).unwrap();
                let old_num_bytes = self.buffer.cap * elem_size;
                assert!(old_num_bytes * 2 <= isize::MAX as usize);

                let new_cap = self.buffer.cap * 2;
                // re-allocation
                let new_num_bytes = old_num_bytes * 2;
                let ptr =
                    unsafe { realloc(self.buffer.ptr.as_ptr() as *mut _, layout, new_num_bytes) };
                (new_cap, ptr)
            }
        };
        // Out of std::memory
        //
        if ptr.is_null() {
            std::alloc::handle_alloc_error(layout);
        }
        self.buffer.cap = new_cap;
        self.buffer.ptr = unsafe { NonNull::new_unchecked(ptr as *mut _) };
    }

    pub fn push(&mut self, elem: T) {
        if self.len == self.buffer.cap {
            self.grow();
        }

        unsafe {
            std::ptr::write(self.buffer.ptr.offset(self.len as isize).as_ptr(), elem);
        }
        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        unsafe {
            Some(std::ptr::read(
                self.buffer.ptr.offset(self.len as isize).as_ptr(),
            ))
        }
    }
    pub fn insert(&mut self, index: usize, elem: T) {
        assert!(index <= self.len, "index out of bounds");
        let mut read_ptr = (self.len() - 1) as isize;
        unsafe {
            if index < self.len {
                let offset = self.len - index;
                for _ in 0..offset {
                    let element = std::ptr::read(self.buffer.ptr.as_ptr().offset(read_ptr));
                    std::ptr::write(self.buffer.ptr.as_ptr().offset(read_ptr + 1), element);
                    read_ptr -= 1;
                }
            }

            #[allow(clippy::ptr_offset_with_cast)]
            std::ptr::write(self.buffer.ptr.as_ptr().offset(index as isize), elem);
        }
        self.len += 1;
    }
    pub fn remove(&mut self, index: usize) {
        assert!(index < self.len, "index out of bounds");
        if self.len == self.buffer.cap {
            self.grow();
        }
        self.len -= 1;
        let mut read_ptr = index as isize;
        unsafe {
            let offset = self.len - index;
            for _ in 0..offset {
                let element = std::ptr::read(self.buffer.ptr.as_ptr().offset(read_ptr + 1));
                std::ptr::write(self.buffer.ptr.as_ptr().offset(read_ptr), element);
                read_ptr += 1;
            }
        }
    }
}

impl<T> CustomVec<T> {
    pub fn into_iter(self) -> CustomIntoIter<T> {
        unsafe {
            let buffer = std::ptr::read(&self.buffer);
            let len = self.len();
            std::mem::forget(self);
            CustomIntoIter::new(
                buffer.ptr.as_ptr(),
                buffer.ptr.offset(len as isize).as_ptr(),
                buffer,
            )
        }
    }
    pub fn from_raw_parts(ptr: *mut T, len: usize, cap: usize) -> Self {
        assert!(len <= cap);
        unsafe {
            Self {
                buffer: CustomRawVec {
                    ptr: NonNull::new_unchecked(ptr),
                    cap,
                },
                len,
            }
        }
    }
}
impl<T> Drop for CustomVec<T> {
    fn drop(&mut self) {
        #[allow(clippy::redundant_pattern_matching)]
        // Skip drop process for scalar types
        if std::mem::needs_drop::<T>() {
            while let Some(_) = self.pop() {}
        }
    }
}

impl<T> std::ops::Deref for CustomVec<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        unsafe { std::slice::from_raw_parts(self.buffer.ptr.as_ptr(), self.len) }
    }
}

impl<T> std::ops::DerefMut for CustomVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { std::slice::from_raw_parts_mut(self.buffer.ptr.as_ptr(), self.len) }
    }
}
