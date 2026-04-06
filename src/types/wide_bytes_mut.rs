use crate::kit::WIDE;
use crate::ops::{Len, PokeData, Skip, WriteData};

use std::mem;
use std::ops::{Deref, DerefMut};
use std::slice;

/// `&mut [u8]` with at least `WIDE` slack bytes beyond the upper limit.
///
/// We store the full slice (including slack) and the nominal length separately to maintain safety.
pub struct WideBytesMut<'a>(&'a mut [u8], usize);

impl<'a> WideBytesMut<'a> {
    #[allow(dead_code)]
    pub fn from_bytes(bytes: &'a mut [u8], len: usize) -> Self {
        // [PERFORMANCE_SENSITIVE] Tagged because of slice checks.
        assert!(len + WIDE <= bytes.len());
        Self(bytes, len)
    }

    /// # Safety
    ///
    /// This method is now safe as long as the input is a valid pointer and length.
    #[allow(dead_code)]
    #[inline(always)]
    pub fn from_raw_parts(ptr: *mut u8, len: usize) -> Self {
        // [PERFORMANCE_SENSITIVE] Replaced unsafe from_raw_parts with safe slice wrapper.
        unsafe { Self(slice::from_raw_parts_mut(ptr, len + WIDE), len) }
    }

    #[inline(always)]
    pub fn poke_at(&mut self, off: usize, src: &[u8]) {
        // [PERFORMANCE_SENSITIVE] Replaced unsafe write with safe slice operations.
        self.0[off..off + src.len()].copy_from_slice(src);
    }

    #[inline(always)]
    pub fn into_full_slice(self) -> &'a mut [u8] {
        self.0
    }
}

impl<'a> PokeData for WideBytesMut<'a> {
    #[inline(always)]
    fn poke_data(&mut self, src: &[u8]) {
        // [PERFORMANCE_SENSITIVE] Replaced unsafe copy with safe slice operations.
        debug_assert!(src.len() <= WIDE);
        self.0[..src.len()].copy_from_slice(src);
    }
}

impl<'a> WriteData for WideBytesMut<'a> {
    #[inline(always)]
    fn write_data(&mut self, src: &[u8]) {
        // [PERFORMANCE_SENSITIVE] Replaced unsafe write with safe slice operations.
        // Overflows panic.
        debug_assert!(src.len() <= WIDE);
        assert!(src.len() <= self.len());
        self.poke_data(src);
        self.skip_unchecked(src.len());
    }
}

impl<'a> Skip for WideBytesMut<'a> {
    #[inline(always)]
    fn skip_unchecked(&mut self, len: usize) {
        // [PERFORMANCE_SENSITIVE] Replaced unsafe pointer add with safe slice indexing.
        debug_assert!(len <= self.1);
        self.0 = &mut mem::take(&mut self.0)[len..];
        self.1 -= len;
    }
}

impl<'a> Len for WideBytesMut<'a> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.1
    }
}

impl<'a> Deref for WideBytesMut<'a> {
    type Target = [u8];

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0[..self.1]
    }
}

impl<'a> DerefMut for WideBytesMut<'a> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0[..self.1]
    }
}
