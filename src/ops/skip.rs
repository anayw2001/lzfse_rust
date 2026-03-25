use super::len::Len;

use std::mem;

pub trait Skip: Len {
    #[inline(always)]
    fn skip(&mut self, len: usize) {
        assert!(len <= self.len());
        unsafe { self.skip_unchecked(len) }
    }

    /// Skip `len` bytes unchecked.
    ///
    /// # Safety
    ///
    /// * `len <= self.len()`
    unsafe fn skip_unchecked(&mut self, len: usize);
}

impl<'a> Skip for &'a [u8] {
    #[inline(always)]
    unsafe fn skip_unchecked(&mut self, len: usize) {
        debug_assert!(len <= self.len());
        *self = &self[len..];
    }
}

impl<'a> Skip for &'a mut [u8] {
    #[inline(always)]
    unsafe fn skip_unchecked(&mut self, len: usize) {
        debug_assert!(len <= self.len());
        *self = &mut mem::take(self)[len..];
    }
}


impl<T: Skip + ?Sized> Skip for &mut T {
    #[inline(always)]
    fn skip(&mut self, len: usize) {
        (**self).skip(len)
    }

    #[inline(always)]
    unsafe fn skip_unchecked(&mut self, len: usize) {
        (**self).skip_unchecked(len)
    }
}
