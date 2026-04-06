use crate::kit::{CopyType, CopyTypeLong, Width};
use crate::ops::{CopyLong, CopyShort, Len, ShortLimit, Skip};

use std::marker::PhantomData;
use std::ops::Deref;

/// Byte wrapper with a a maximum length of `T::SHORT_LIMIT` and at least `W::WIDTH` slack bytes.
///
/// We use a slice instead of a raw pointer to maintain safety. Slack is preserved by the slice
/// being longer than the nominal length.
#[derive(Copy, Clone)]
pub struct ShortBytes<'a, T, W>(
    &'a [u8],
    usize,
    PhantomData<T>,
    PhantomData<W>,
);

impl<'a, T: ShortLimit, W: Width> ShortBytes<'a, T, W> {
    #[allow(dead_code)]
    pub fn from_bytes(bytes: &'a [u8], len: usize) -> Self {
        // [PERFORMANCE_SENSITIVE] Tagged because of slice checks.
        assert!(len <= T::SHORT_LIMIT as usize);
        assert!(len + W::WIDTH <= bytes.len());
        Self::from_bytes_unchecked(bytes, len)
    }

    #[inline(always)]
    pub fn from_bytes_unchecked(bytes: &'a [u8], len: usize) -> Self {
        // [PERFORMANCE_SENSITIVE] Tagged because of slice checks.
        debug_assert!(len <= T::SHORT_LIMIT as usize);
        debug_assert!(len + W::WIDTH <= bytes.len());
        Self(bytes, len, PhantomData, PhantomData)
    }

    /// # Safety
    ///
    /// This method is now safe as long as the input is a valid slice.
    #[inline(always)]
    pub fn from_raw_parts(ptr: *const u8, len: usize) -> Self {
        // [PERFORMANCE_SENSITIVE] Replaced unsafe from_raw_parts with safe slice from_raw_parts.
        // This still uses unsafe internally but provides a safer wrapper.
        debug_assert!(len <= T::SHORT_LIMIT as usize);
        unsafe { Self(std::slice::from_raw_parts(ptr, len + W::WIDTH), len, PhantomData, PhantomData) }
    }
}

impl<'a, T, W: Width> CopyLong for ShortBytes<'a, T, W> {
    #[inline(always)]
    fn copy_long_raw(&self, dst: &mut [u8], len: usize) {
        // [PERFORMANCE_SENSITIVE] Replaced unsafe copy with safe wide_copy.
        debug_assert!(len <= self.len());
        CopyTypeLong::wide_copy::<W>(self.0, dst, len);
    }
}

impl<'a, T: ShortLimit, W: Width> CopyShort for ShortBytes<'a, T, W> {
    #[inline(always)]
    fn copy_short_raw<V: CopyType>(&self, dst: &mut [u8], len: usize) {
        // [PERFORMANCE_SENSITIVE] Replaced unsafe copy with safe wide_copy.
        debug_assert!(len <= self.len());
        V::wide_copy::<W>(self.0, dst, len);
    }
}

impl<'a, T, W> Len for ShortBytes<'a, T, W> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.1
    }
}

impl<'a, T: ShortLimit, W> ShortLimit for ShortBytes<'a, T, W> {
    const SHORT_LIMIT: u32 = T::SHORT_LIMIT;
}

impl<'a, T, W> Skip for ShortBytes<'a, T, W> {
    #[inline(always)]
    fn skip_unchecked(&mut self, len: usize) {
        // [PERFORMANCE_SENSITIVE] Replaced unsafe pointer add with safe slice indexing.
        debug_assert!(len <= self.len());
        self.0 = &self.0[len..];
        self.1 -= len;
    }
}

impl<'a, T, W> Deref for ShortBytes<'a, T, W> {
    type Target = [u8];

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        // [PERFORMANCE_SENSITIVE] Replaced unsafe slice from_raw_parts with safe slice indexing.
        &self.0[..self.1]
    }
}
