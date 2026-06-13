use crate::raw_vec::CustomRawVec;

pub struct CustomIntoIter<T> {
    start: *const T,
    end: *const T,
    _buffer: CustomRawVec<T>,
}
impl<T> CustomIntoIter<T> {
    pub unsafe fn new(start: *const T, end: *const T, buffer: CustomRawVec<T>) -> Self {
        CustomIntoIter {
            start,
            end,
            _buffer: buffer,
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
            let element = std::ptr::read(self.start);
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
        if std::mem::needs_drop::<T>() {
            for _val in self {}
        }
    }
}
