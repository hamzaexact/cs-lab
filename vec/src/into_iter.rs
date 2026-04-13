use std::{
    alloc::Layout,
    ptr::{self, NonNull},
};

#[derive(Debug)]
pub struct CustomIntoIter<T> {
    start: *const T,
    end: *const T,
    buffer: NonNull<T>,
    cap: usize,
}
impl<T> CustomIntoIter<T> {
    pub unsafe fn new(ptr: NonNull<T>, cap: usize, len: usize) -> Self {
        CustomIntoIter {
            start: ptr.as_ptr(),
            end: {
                if len == 0 {
                    ptr.as_ptr()
                } else {
                    #[allow(clippy::ptr_offset_with_cast)]
                    unsafe {
                        ptr.as_ptr().offset(len as isize)
                    }
                }
            },
            buffer: ptr,
            cap,
        }
    }
}
impl<T> Iterator for CustomIntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.start == self.end {
            return None;
        }
        unsafe {
            let element = ptr::read(self.start);
            self.start = self.start.offset(1);
            Some(element)
        }
    }
}
impl<T> DoubleEndedIterator for CustomIntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.start == self.end {
            return None;
        }
        unsafe {
            self.end = self.end.offset(-1);
            Some(std::ptr::read(self.end))
        }
    }
}

impl<T> Drop for CustomIntoIter<T> {
    fn drop(&mut self) {
        let cap = self.cap;
        let buffer = self.buffer;
        if self.cap != 0 {
            if std::mem::needs_drop::<T>() {
                #[allow(unused_variables)]
                for val in self {}
            }
            let layout = Layout::array::<T>(cap).unwrap();
            unsafe {
                std::alloc::dealloc(buffer.as_ptr() as *mut u8, layout);
            }
        }
    }
}

