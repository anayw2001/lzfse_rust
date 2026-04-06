use crate::types::WideBytesMut;

use super::len::Len;
use super::skip::Skip;

/// Copy long: eager, high volume, higher latency.
pub trait CopyLong: Len + Skip {
    #[allow(dead_code)]
    #[inline(always)]
    fn read_long(&mut self, mut dst: WideBytesMut) {
        assert!(dst.len() <= self.len());
        self.read_long_unchecked(&mut dst);
    }

    #[allow(dead_code)]
    #[inline(always)]
    fn read_long_unchecked(&mut self, dst: &mut [u8]) {
        // [PERFORMANCE_SENSITIVE] Tagged because of slice checks.
        self.read_long_raw(dst, dst.len());
    }

    /// * `dst` is valid for `len + WIDE` byte writes.
    /// * `len <= Self::len()`
    #[inline(always)]
    fn read_long_raw(&mut self, dst: &mut [u8], len: usize) {
        // [PERFORMANCE_SENSITIVE] Tagged because of slice checks.
        self.copy_long_raw(dst, len);
        self.skip_unchecked(len);
    }

    #[allow(dead_code)]
    #[inline(always)]
    fn copy_long(&self, dst: WideBytesMut) {
        assert!(dst.len() <= self.len());
        self.copy_long_unchecked(dst);
    }

    #[allow(dead_code)]
    #[inline(always)]
    fn copy_long_unchecked(&self, mut dst: WideBytesMut) {
        // [PERFORMANCE_SENSITIVE] Tagged because of slice checks.
        let len = dst.len();
        self.copy_long_raw(&mut dst, len)
    }

    /// * `dst` is valid for `len + WIDE` byte writes.
    /// * `len <= Self::len()`
    fn copy_long_raw(&self, dst: &mut [u8], len: usize);
}

impl<T: CopyLong + ?Sized> CopyLong for &mut T {
    #[allow(dead_code)]
    #[inline(always)]
    fn read_long(&mut self, dst: WideBytesMut) {
        (**self).read_long(dst)
    }

    #[allow(dead_code)]
    #[inline(always)]
    fn read_long_unchecked(&mut self, dst: &mut [u8]) {
        (**self).read_long_unchecked(dst)
    }

    #[inline(always)]
    fn read_long_raw(&mut self, dst: &mut [u8], len: usize) {
        (**self).read_long_raw(dst, len)
    }

    #[allow(dead_code)]
    #[inline(always)]
    fn copy_long(&self, dst: WideBytesMut) {
        (**self).copy_long(dst)
    }

    #[allow(dead_code)]
    #[inline(always)]
    fn copy_long_unchecked(&self, dst: WideBytesMut) {
        (**self).copy_long_unchecked(dst)
    }

    #[inline(always)]
    fn copy_long_raw(&self, dst: &mut [u8], len: usize) {
        (**self).copy_long_raw(dst, len)
    }
}

impl CopyLong for &[u8] {
    #[inline(always)]
    fn copy_long_raw(&self, dst: &mut [u8], len: usize) {
        // [PERFORMANCE_SENSITIVE] Replaced unsafe copy with safe slice operations.
        debug_assert!(len <= self.len());
        dst[..len].copy_from_slice(&self[..len]);
    }
}

#[cfg(test)]
mod tests {
    use crate::kit::WIDE;

    use super::*;

    #[test]
    fn test_seq() {
        let mut bytes = [0u8; 1 + WIDE];
        let vec: Vec<u8> = (0u8..=255).collect();
        let mut literals = vec.as_slice();
        for i in 0..=255 {
            let dst = WideBytesMut::from_raw_parts(bytes.as_mut_ptr(), 1);
            literals.read_long(dst);
            assert_eq!(bytes[0], i);
        }
    }

    #[allow(clippy::needless_range_loop)]
    #[test]
    fn test_inc() {
        let mut bytes = [0u8; 255 + WIDE];
        for i in 0..=255 {
            let vec: Vec<u8> = (0u8..=255).collect();
            let mut literals = vec.as_slice();
            let dst = WideBytesMut::from_raw_parts(bytes.as_mut_ptr(), i);
            literals.read_long(dst);
            for j in 0..i {
                assert_eq!(bytes[j], j as u8);
            }
        }
    }
}
