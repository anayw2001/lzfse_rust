use crate::match_kit;
use crate::types::Idx;

use super::ring_box::RingBox;
use super::ring_size::RingSize;
use super::ring_type::RingType;
use super::ring_view::RingView;

use std::marker::PhantomData;
use std::mem;
use std::ops::{Deref, DerefMut};

pub const OVERMATCH_LEN: usize = 5 * mem::size_of::<usize>();

pub struct Ring<'a, T>(&'a mut [u8], PhantomData<T>);

impl<'a, T: RingType> Ring<'a, T> {
    /// Returns a raw mut pointer to the start of the nominal data range.
    #[inline(always)]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        unsafe { self.0.as_mut_ptr().add(T::RING_LIMIT as usize) }
    }

    /// Returns a raw pointer to the start of the nominal data range.
    #[inline(always)]
    pub fn as_ptr(&self) -> *const u8 {
        unsafe { self.0.as_ptr().add(T::RING_LIMIT as usize) }
    }

    #[inline(always)]
    pub fn nominal_slice(&self) -> &[u8] {
        let start = T::RING_LIMIT as usize;
        &self.0[start..start + T::RING_SIZE as usize]
    }
}

// Implementation notes:
//
// Hybrid ring buffer.
//
// |tttt|HHHH|...............................................|TTTT|hhhh|S|
// <-------------------------- RING_CAPACITY ---------------------------->
//      <----------------------- RING_SIZE ----------------------->
//      ^ PTR Nominal start (offset RING_LIMIT into the slice)
//
// Tag  | Zone           | Size
// ----------------------------------
// HHHH | head           | RING_LIMIT
// TTTT | tail           | RING_LIMIT
// hhhh | head shadow    | RING_LIMIT
// tttt | tail shadow    | RING_LIMIT
// S    | Slack          | WIDTH

impl<'a, T: RingType> Ring<'a, T> {
    /// May overmatch `max` by  `LEN + OVERMATCH_LEN` bytes
    #[inline(always)]
    pub fn match_inc_coarse<const LEN: usize>(&self, idxs: (Idx, Idx), max: usize) -> usize {
        assert!(LEN + OVERMATCH_LEN <= T::RING_LIMIT as usize);
        debug_assert!(self.head_shadowed_len(LEN + OVERMATCH_LEN));
        let indexes = (
            (usize::from(idxs.0)) % T::RING_SIZE as usize,
            (usize::from(idxs.1)) % T::RING_SIZE as usize,
        );
        let u_0 = unsafe {
            self.0
                .as_ptr()
                .add(T::RING_LIMIT as usize + indexes.0 + LEN)
                .cast::<usize>()
                .read_unaligned()
        };
        let u_1 = unsafe {
            self.0
                .as_ptr()
                .add(T::RING_LIMIT as usize + indexes.1 + LEN)
                .cast::<usize>()
                .read_unaligned()
        };
        let x = u_0 ^ u_1;
        if x != 0 {
            // Likely
            LEN + match_kit::nclz_bytes(x) as usize
        } else {
            // Unlikely.
            self.match_inc_coarse_cont::<LEN>(indexes, max)
        }
    }

    fn match_inc_coarse_cont<const LEN: usize>(
        &self,
        mut indexes: (usize, usize),
        max: usize,
    ) -> usize {
        let mut len = LEN + mem::size_of::<usize>();
        loop {
            for i in 0..4 {
                let off = LEN + mem::size_of::<usize>() + i * mem::size_of::<usize>();
                let u_0 = unsafe {
                    self.0
                        .as_ptr()
                        .add(T::RING_LIMIT as usize + indexes.0 + off)
                        .cast::<usize>()
                        .read_unaligned()
                };
                let u_1 = unsafe {
                    self.0
                        .as_ptr()
                        .add(T::RING_LIMIT as usize + indexes.1 + off)
                        .cast::<usize>()
                        .read_unaligned()
                };
                let x = u_0 ^ u_1;
                if x != 0 {
                    return len + i * mem::size_of::<usize>() + match_kit::nclz_bytes(x) as usize;
                }
            }
            if len >= max {
                break;
            }
            len += 4 * mem::size_of::<usize>();
            indexes = (
                indexes.0.wrapping_add(4 * mem::size_of::<usize>()) % T::RING_SIZE as usize,
                indexes.1.wrapping_add(4 * mem::size_of::<usize>()) % T::RING_SIZE as usize,
            );
        }
        max
    }

    /// May overmatch `max` by  `LEN + OVERMATCH_LEN` bytes
    #[inline(always)]
    pub fn match_dec_coarse<const LEN: usize>(&self, idxs: (Idx, Idx), max: usize) -> usize {
        assert!(LEN + OVERMATCH_LEN <= T::RING_LIMIT as usize);
        debug_assert!(self.head_shadowed_len(LEN + OVERMATCH_LEN));
        let off = LEN + OVERMATCH_LEN;
        let indexes = (
            (usize::from(idxs.0).wrapping_sub(off)) % T::RING_SIZE as usize,
            (usize::from(idxs.1).wrapping_sub(off)) % T::RING_SIZE as usize,
        );
        let off = 4 * mem::size_of::<usize>();
        let u_0 = unsafe {
            self.0
                .as_ptr()
                .add(T::RING_LIMIT as usize + indexes.0 + off)
                .cast::<usize>()
                .read_unaligned()
        };
        let u_1 = unsafe {
            self.0
                .as_ptr()
                .add(T::RING_LIMIT as usize + indexes.1 + off)
                .cast::<usize>()
                .read_unaligned()
        };
        let x = u_0 ^ u_1;
        if x != 0 {
            // Likely
            LEN + match_kit::nctz_bytes(x) as usize
        } else {
            // Unlikely.
            self.match_dec_cont::<LEN>(indexes, max)
        }
    }

    fn match_dec_cont<const LEN: usize>(&self, mut indexes: (usize, usize), max: usize) -> usize {
        let mut len = LEN + mem::size_of::<usize>();
        loop {
            for i in 0..4 {
                let off = (3 - i) * mem::size_of::<usize>();
                let u_0 = unsafe {
                    self.0
                        .as_ptr()
                        .add(T::RING_LIMIT as usize + indexes.0 + off)
                        .cast::<usize>()
                        .read_unaligned()
                };
                let u_1 = unsafe {
                    self.0
                        .as_ptr()
                        .add(T::RING_LIMIT as usize + indexes.1 + off)
                        .cast::<usize>()
                        .read_unaligned()
                };
                let x = u_0 ^ u_1;
                if x != 0 {
                    return len + i * mem::size_of::<usize>() + match_kit::nctz_bytes(x) as usize;
                }
            }
            if len >= max {
                break;
            }
            len += 4 * mem::size_of::<usize>();
            indexes = (
                indexes.0.wrapping_sub(4 * mem::size_of::<usize>()) % T::RING_SIZE as usize,
                indexes.1.wrapping_sub(4 * mem::size_of::<usize>()) % T::RING_SIZE as usize,
            );
        }
        max
    }

    pub fn head_shadowed(&self) -> bool {
        self.head_shadowed_len(T::RING_LIMIT as usize)
    }

    #[inline(always)]
    pub fn head_shadowed_len(&self, len: usize) -> bool {
        assert!(len <= T::RING_LIMIT as usize);
        let start = T::RING_LIMIT as usize;
        let shadow_start = start + T::RING_SIZE as usize;
        self.0[start..start + len] == self.0[shadow_start..shadow_start + len]
    }

    #[inline(always)]
    pub fn head_copy_out(&mut self) {
        self.head_copy_out_len(T::RING_LIMIT as usize);
    }

    /// Copy head -> head shadow
    #[inline(always)]
    pub fn head_copy_out_len(&mut self, len: usize) {
        assert!(len <= T::RING_LIMIT as usize);
        let start = T::RING_LIMIT as usize;
        let shadow_start = start + T::RING_SIZE as usize;
        self.0.copy_within(start..start + len, shadow_start);
    }

    #[inline(always)]
    #[allow(dead_code)]
    pub fn head_copy_in(&mut self) {
        self.head_copy_in_len(T::RING_LIMIT as usize);
    }

    /// Copy head shadow -> head
    #[inline(always)]
    pub fn head_copy_in_len(&mut self, len: usize) {
        assert!(len <= T::RING_LIMIT as usize);
        let start = T::RING_LIMIT as usize;
        let shadow_start = start + T::RING_SIZE as usize;
        self.0.copy_within(shadow_start..shadow_start + len, start);
    }

    #[allow(dead_code)]
    pub fn tail_shadowed(&self) -> bool {
        self.tail_shadowed_len(T::RING_LIMIT as usize)
    }

    #[allow(dead_code)]
    #[inline(always)]
    pub fn tail_shadowed_len(&self, len: usize) -> bool {
        assert!(len <= T::RING_LIMIT as usize);
        let nominal_start = T::RING_LIMIT as usize;
        let tail_shadow_start = 0;
        let tail_start = nominal_start + T::RING_SIZE as usize - T::RING_LIMIT as usize;
        self.0[tail_shadow_start..tail_shadow_start + len] == self.0[tail_start..tail_start + len]
    }

    #[inline(always)]
    pub fn tail_copy_out(&mut self) {
        self.tail_copy_out_len(T::RING_LIMIT as usize);
    }

    /// Copy tail -> tail shadow
    #[inline(always)]
    pub fn tail_copy_out_len(&mut self, len: usize) {
        assert!(len <= T::RING_LIMIT as usize);
        let nominal_start = T::RING_LIMIT as usize;
        let tail_shadow_start = 0;
        let tail_start = nominal_start + T::RING_SIZE as usize - T::RING_LIMIT as usize;
        self.0.copy_within(tail_start..tail_start + len, tail_shadow_start);
    }

    #[allow(dead_code)]
    #[inline(always)]
    pub fn tail_copy_in(&mut self) {
        self.tail_copy_in_len(T::RING_LIMIT as usize);
    }

    /// Copy tail shadow -> tail
    #[allow(dead_code)]
    #[inline(always)]
    pub fn tail_copy_in_len(&mut self, len: usize) {
        assert!(len <= T::RING_LIMIT as usize);
        let nominal_start = T::RING_LIMIT as usize;
        let tail_shadow_start = 0;
        let tail_start = nominal_start + T::RING_SIZE as usize - T::RING_LIMIT as usize;
        self.0.copy_within(tail_shadow_start..tail_shadow_start + len, tail_start);
    }

    #[inline(always)]
    pub fn view(&self, head: Idx, tail: Idx) -> RingView<'_, T> {
        RingView::new(self, head, tail)
    }
}

impl<'a, T: RingSize + RingType> Ring<'a, T> {
    #[inline(always)]
    pub fn get_u32(&self, idx: Idx) -> u32 {
        let index = idx % T::RING_SIZE;
        unsafe {
            self.0
                .as_ptr()
                .add(T::RING_LIMIT as usize + index as usize)
                .cast::<u32>()
                .read_unaligned()
        }
    }

    #[inline(always)]
    pub fn set_quad_index(&mut self, index: usize, u: u32) {
        debug_assert!(index < T::RING_SIZE as usize);
        unsafe {
            self.0
                .as_mut_ptr()
                .add(T::RING_LIMIT as usize + index)
                .cast::<u32>()
                .write_unaligned(u);
        }
    }
}

impl<'a, T: RingType> Deref for Ring<'a, T> {
    type Target = [u8];

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        let start = T::RING_LIMIT as usize;
        &self.0[start..start + T::RING_SIZE as usize]
    }
}

impl<'a, T: RingType> DerefMut for Ring<'a, T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        let start = T::RING_LIMIT as usize;
        &mut self.0[start..start + T::RING_SIZE as usize]
    }
}

impl<'a, T: RingType> From<&'a mut RingBox<T>> for Ring<'a, T> {
    #[inline(always)]
    fn from(ring_box: &'a mut RingBox<T>) -> Self {
        Self(&mut ring_box.0, PhantomData)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Copy, Clone, Debug)]
    struct T;

    unsafe impl RingSize for T {
        const RING_SIZE: u32 = 0x0100;
    }

    unsafe impl RingType for T {
        const RING_LIMIT: u32 = 0x0040;
    }

    // Cycling match_index, match_distance and match_len combinations.
    #[test]
    fn match_inc_1() {
        let mut ring_box = RingBox::<T>::default();
        let mut ring = Ring::from(&mut ring_box);
        for match_index in 0x0000..0x0100 {
            for match_distance in 0x0001..0x0100 {
                ring.fill(0xFF);
                for n in 0..match_distance {
                    ring[(match_index + n) % 0x0100] = (n + 1) as u8;
                }
                let index = match_index + match_distance;
                for match_len in 0..0x0100 - match_distance {
                    ring.head_copy_out();
                    ring.tail_copy_out();
                    let n = ring.match_inc_coarse::<0>(
                        (Idx::new(index as u32), Idx::new(match_index as u32)),
                        0x100,
                    );
                    assert_eq!(n, match_len);
                    ring[(index + match_len) % 0x0100] = (match_len % match_distance + 1) as u8;
                }
            }
        }
    }

    // Cycling match_index, match_distance combinations with overmatch limit checking.
    #[test]
    fn match_inc_2() {
        let mut ring_box = RingBox::<T>::default();
        let mut ring = Ring::from(&mut ring_box);
        for match_index in 0x0000..0x0100 {
            for match_distance in 0x0001..0x0100 {
                ring.fill(0xFF);
                for n in 0..match_distance {
                    ring[(match_index + n) % 0x0100] = (n + 1) as u8;
                }
                let index = match_index + match_distance;
                for match_len in 0..0x0100 - match_distance {
                    ring[(index + match_len) % 0x0100] = (match_len % match_distance + 1) as u8;
                }
                ring.head_copy_out();
                ring.tail_copy_out();
                let match_len = 0x0100 - match_distance;
                let n = ring.match_inc_coarse::<0>(
                    (Idx::new(index as u32), Idx::new(match_index as u32)),
                    0,
                );
                assert!(n <= match_len);
                assert!(n <= OVERMATCH_LEN);
                let n = ring.match_inc_coarse::<4>(
                    (Idx::new(index as u32), Idx::new(match_index as u32)),
                    0,
                );
                assert!(n <= match_len + 4);
                assert!(n <= 4 + OVERMATCH_LEN);
            }
        }
    }

    // Cycling match_index, match_distance and match_len combinations.
    #[test]
    fn match_dec_1() {
        let mut ring_box = RingBox::<T>::default();
        let mut ring = Ring::from(&mut ring_box);
        for match_index in 0x0000..0x0100usize {
            for match_distance in 0x0001..0x0100 {
                ring.fill(0xFF);
                for n in 1..=match_distance {
                    ring[(match_index.wrapping_sub(n)) % 0x0100] = n as u8;
                }
                let index = match_index.wrapping_sub(match_distance);
                for match_len in 0..0x0100 - match_distance {
                    ring.head_copy_out();
                    ring.tail_copy_out();
                    let n = ring.match_dec_coarse::<0>(
                        (Idx::new(index as u32), Idx::new(match_index as u32)),
                        0x100,
                    );
                    assert_eq!(n, match_len);
                    ring[index.wrapping_sub(match_len + 1) % 0x0100] =
                        (match_len % match_distance + 1) as u8;
                }
            }
        }
    }

    // Cycling match_index, match_distance combinations with overmatch limit checking.
    #[test]
    fn match_dec_2() {
        let mut ring_box = RingBox::<T>::default();
        let mut ring = Ring::from(&mut ring_box);
        for match_index in 0x0000..0x0100usize {
            for match_distance in 0x0001..0x0100 {
                ring.fill(0xFF);
                for n in 1..=match_distance {
                    ring[(match_index.wrapping_sub(n)) % 0x0100] = n as u8;
                }
                let index = match_index.wrapping_sub(match_distance);
                for match_len in 0..0x0100 - match_distance {
                    ring[index.wrapping_sub(match_len + 1) % 0x0100] =
                        (match_len % match_distance + 1) as u8;
                }
                ring.head_copy_out();
                ring.tail_copy_out();
                let match_len = 0x0100 - match_distance;
                let n = ring.match_dec_coarse::<0>(
                    (Idx::new(index as u32), Idx::new(match_index as u32)),
                    0,
                );
                assert!(n <= match_len);
                assert!(n <= OVERMATCH_LEN);
                let n = ring.match_dec_coarse::<4>(
                    (Idx::new(index as u32), Idx::new(match_index as u32)),
                    0,
                );
                assert!(n <= match_len + 4);
                assert!(n <= 4 + OVERMATCH_LEN);
            }
        }
    }
}
