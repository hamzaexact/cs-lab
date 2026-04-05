// #![allow(dead_code)]

use std::{
    alloc::{Layout, alloc, realloc},
    mem,
    ptr::NonNull,
};

// #[derive(Debug)]
// Layout
pub(crate) struct CustomVec<T> {
    ptr: NonNull<T>, // NonNull pointer to the allocation
    cap: usize,      // Size of the allocation
    len: usize,      // The number of elements that have been initialized.
}

impl<T> CustomVec<T> {
    pub fn new() -> Self {
        assert!(mem::size_of::<T>() != 0, "We're not ready to handle ZSTs");
        Self {
            ptr: NonNull::dangling(),
            cap: 0,
            len: 0,
        }
    }
    pub fn grow(&mut self) {
        let align = mem::align_of::<T>();
        let elem_size = mem::size_of::<T>();
        // let layout = Layout::new::<T>();
        let layout = unsafe { Layout::from_size_align_unchecked(elem_size, align) };
        let (new_cap, ptr) = {
            if self.cap == 0 {
                unsafe {
                    let ptr = alloc(layout);
                    (1, ptr)
                }
            } else {
                let old_num_bytes = self.cap * elem_size;
                assert!(old_num_bytes <= isize::MAX as usize);
                let new_cap = self.cap * 2;

                // re-allocation
                let new_num_bytes = old_num_bytes * 2;
                let ptr = unsafe { realloc(self.ptr.as_ptr() as *mut _, layout, new_num_bytes) };
                (new_cap, ptr)
            }
        };
        // Out of memory
        if ptr.is_null() {
            panic!()
        }
        self.cap = new_cap;
        self.ptr = unsafe { NonNull::new_unchecked(ptr as *mut _) }
    }
}
