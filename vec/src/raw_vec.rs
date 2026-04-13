use std::ptr::NonNull;

pub(crate) struct CustomRawVec<T> {
    pub ptr: NonNull<T>, // NonNull pointer to the allocation
    pub cap: usize,      // Size of the allocation
}

impl<T> CustomRawVec<T> {
    pub fn new() -> Self {
        Self {
            ptr: NonNull::dangling(),
            cap: 0,
        }
    }
}

impl<T> Drop for CustomRawVec<T> {
    fn drop(&mut self) {
        if self.cap != 0 {
            if std::mem::needs_drop::<T>() {
                let layout = std::alloc::Layout::array::<T>(self.cap).unwrap();
                unsafe {
                    std::alloc::dealloc(self.ptr.as_ptr() as *mut u8, layout);
                }
            }
        }
    }
}
