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
        // cap * T.size?
        // let lay = Layout::new::<Arr
        // let layout = Layout::new::<T>();
        let layout = unsafe { Layout::from_size_align_unchecked(elem_size, align) };
        let (new_cap, ptr) = {
            if self.cap == 0 {
                unsafe {
                    let ptr = alloc(layout);
                    (1, ptr)
                }
            } else {
                let layout = Layout::array::<T>(self.cap).unwrap();
                let old_num_bytes = self.cap * elem_size;
                assert!(old_num_bytes * 2 <= isize::MAX as usize);

                let new_cap = self.cap * 2;
                // re-allocation
                let new_num_bytes = old_num_bytes * 2;
                let ptr = unsafe { realloc(self.ptr.as_ptr() as *mut _, layout, new_num_bytes) };
                (new_cap, ptr)
            }
        };
        // Out of Memory
        if ptr.is_null() {
            std::alloc::handle_alloc_error(layout);
        }
        self.cap = new_cap;
        self.ptr = unsafe { NonNull::new_unchecked(ptr as *mut _) };
    }

    pub fn push(&mut self, elem: T) {
        if self.len == self.cap {
            self.grow();
        }

        unsafe {
            std::ptr::write(self.ptr.offset(self.len as isize).as_ptr(), elem);
        }
        self.len += 1;
    }
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        unsafe { Some(std::ptr::read(self.ptr.offset(self.len as isize).as_ptr())) }
    }
    pub fn insert(&mut self, index: usize, elem: T) {
        assert!(index <= self.len, "index out of bounds");
        if self.len == self.cap {
            self.grow();
        }
        let mut read_ptr = (self.len() - 1) as isize;
        unsafe {
            if index < self.len {
                let offset = self.len - index;
                for _ in 0..offset {
                    // save the data at idx len - 1;
                    let element = std::ptr::read(self.ptr.as_ptr().offset(read_ptr));
                    // write the data at idx read_ptr + 1;
                    std::ptr::write(self.ptr.as_ptr().offset(read_ptr + 1), element);
                    read_ptr -= 1;
                }

                // std::ptr::write(
                //     self.ptr.as_ptr().offset(read_ptr + 1),
                //     std::ptr::read(self.ptr.as_ptr().offset(read_ptr)),
                // )
            }
            // we already check if index <= self.len
            // so worst case is self.len == 0 & index = self.len - N
            #[allow(clippy::ptr_offset_with_cast)]
            std::ptr::write(self.ptr.as_ptr().offset(index as isize), elem);
        }

        // read ptr at the exact indext we want
        self.len += 1;
    }
}

impl<T> Drop for CustomVec<T> {
    fn drop(&mut self) {
        if self.len != 0 {
            #[allow(clippy::redundant_pattern_matching)]
            while let Some(_) = self.pop() {}
            unsafe {
                let layout = Layout::array::<T>(self.cap).unwrap();
                std::alloc::dealloc(self.ptr.as_ptr() as *mut u8, layout);
            }
        }
    }
}

impl<T> std::ops::Deref for CustomVec<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }
}

impl<T> std::ops::DerefMut for CustomVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }
}
