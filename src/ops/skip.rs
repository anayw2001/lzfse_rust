use super::len::Len;

use std::mem;

pub trait Skip: Len {
    #[inline(always)]
    fn skip(&mut self, len: usize) {
        self.skip_unchecked(len)
    }

    /// Skip `len` bytes unchecked.
    fn skip_unchecked(&mut self, len: usize);
}

impl Skip for &[u8] {
    #[inline(always)]
    fn skip_unchecked(&mut self, len: usize) {
        assert!(len <= self.len());
        *self = &self[len..];
    }
}

impl Skip for &mut [u8] {
    #[inline(always)]
    fn skip_unchecked(&mut self, len: usize) {
        assert!(len <= self.len());
        *self = &mut mem::take(self)[len..];
    }
}

impl<T: Skip + ?Sized> Skip for &mut T {
    #[inline(always)]
    fn skip(&mut self, len: usize) {
        (**self).skip(len)
    }

    #[inline(always)]
    fn skip_unchecked(&mut self, len: usize) {
        (**self).skip_unchecked(len)
    }
}
