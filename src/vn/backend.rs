use crate::encode::Backend;
use crate::kit::{CopyTypeIndex, WIDE};
use crate::lmd::MatchDistance;
use crate::ops::CopyShort;
use crate::types::{Idx, ShortBuffer, ShortWriter};

use super::block::VnBlock;
use super::constants::*;
use super::object::Vn;
use super::opc;

use std::io;

// Working slack: max op len + max literal len + EOS tag len
const SLACK: u32 = 0x03 + 0x010F + 0x08;

fn n_allocate(len: usize) -> usize {
    // Assuming a functional front end that pushes literals correctly:
    // * literals, at worst, cost 1 byte plus an additional 1 byte per 4 literals.
    // * match runs are always cost neutral.
    VN_HEADER_SIZE as usize + (len / 4) * 5 + 32 + SLACK as usize + WIDE
}

#[derive(Default)]
pub struct VnBackend {
    mark: Idx,
    match_distance: u32,
    n_literals: u32,
    n_match_bytes: u32,
}

/// VN backend.
///
/// Memory is allocated in advance for the entire block based on the worst scenario.
/// Pushing literals with a block length of less than 4 more than once may overflow our memory
/// allocation.
impl Backend for VnBackend {
    type Type = Vn;

    #[inline(always)]
    fn init<O: ShortWriter>(&mut self, dst: &mut O, len: Option<usize>) -> io::Result<()> {
        // TODO MAX len
        if let Some(u) = len {
            self.mark = dst.pos();
            self.match_distance = 0;
            self.n_literals = 0;
            self.n_match_bytes = 0;
            let n = n_allocate(u);
            dst.allocate(n)?;
            dst.write_short_bytes(&[0u8; VN_HEADER_SIZE as usize])?;
            Ok(())
        } else {
            Err(io::ErrorKind::Other.into())
        }
    }

    #[inline(always)]
    fn push_literals<I: ShortBuffer, O: ShortWriter>(
        &mut self,
        dst: &mut O,
        mut literals: I,
    ) -> io::Result<()> {
        assert!(I::SHORT_LIMIT >= 0x010F);
        self.n_literals += literals.len() as u32;
        while literals.len() >= 0x10 {
            let len = literals.len().min(0x10F) as u32;
            lrg_l(dst, &mut literals, len);
        }
        if literals.len() > 0 {
            let len = literals.len() as u32;
            sml_l(dst, &mut literals, len);
        }
        Ok(())
    }

    #[inline(always)]
    fn push_match<I, O: ShortWriter>(
        &mut self,
        dst: &mut O,
        mut literals: I,
        mut match_len: u32,
        match_distance: MatchDistance<Vn>,
    ) -> io::Result<()>
    where
        I: ShortBuffer,
    {
        assert!(I::SHORT_LIMIT >= 0x010F);
        let match_distance = match_distance.get();
        self.n_literals += literals.len() as u32;
        self.n_match_bytes += match_len;
        while literals.len() >= 0x10 {
            let len = literals.len().min(0x10F) as u32;
            lrg_l(dst, &mut literals, len);
        }
        if literals.len() >= 0x04 {
            let len = literals.len() as u32;
            sml_l(dst, &mut literals, len);
        }
        let literal_len = literals.len();
        let n = opc::match_len_x(literal_len as u32).min(match_len);
        match_len -= n;
        if match_distance == self.match_distance {
            if literal_len == 0 {
                sml_m(dst, n);
            } else {
                pre_d(dst, &mut literals, literal_len as u32, n);
            }
        } else if match_distance < 0x600 {
            sml_d(dst, &mut literals, literal_len as u32, n, match_distance);
        } else if match_distance >= 0x4000 || match_len == 0 || n + match_len > 0x22 {
            lrg_d(dst, &mut literals, literal_len as u32, n, match_distance);
        } else {
            med_d(dst, &mut literals, literal_len as u32, n, match_distance);
        }
        self.match_distance = match_distance;
        while match_len > 0x0F {
            let limit = match_len.min(0x10F);
            lrg_m(dst, limit);
            match_len -= limit;
        }
        if match_len > 0 {
            sml_m(dst, match_len);
        }
        Ok(())
    }

    #[inline(always)]
    fn finalize<O: ShortWriter>(&mut self, dst: &mut O) -> io::Result<()> {
        dst.write_short_u64(EOS as u64)?;
        let n_payload_bytes = (dst.pos() - self.mark) as u32 - VN_HEADER_SIZE;
        let buf = dst.patch_into(self.mark, VN_HEADER_SIZE as usize);
        let n_raw_bytes = self.n_literals + self.n_match_bytes;
        VnBlock::new(n_raw_bytes, n_payload_bytes).expect("internal error").store(buf);
        Ok(())
    }
}

fn sml_l<I: CopyShort, O: ShortWriter>(dst: &mut O, src: &mut I, literal_len: u32) {
    l(dst, src, literal_len, opc::encode_sml_l(literal_len), 1)
}

fn lrg_l<I: CopyShort, O: ShortWriter>(dst: &mut O, src: &mut I, literal_len: u32) {
    l(dst, src, literal_len, opc::encode_lrg_l(literal_len), 2)
}

fn l<I: CopyShort, O: ShortWriter>(
    dst: &mut O,
    src: &mut I,
    literal_len: u32,
    opu: u32,
    op_len: u32,
) {
    // [PERFORMANCE_SENSITIVE] Replaced unsafe pointer manipulation with safe short_wide_block and truncate.
    debug_assert!(literal_len <= 0x10F);
    debug_assert!(literal_len as usize <= src.len());
    debug_assert!(op_len <= 3);
    let start_pos = dst.pos();
    let mut block = dst.short_wide_block(op_len + literal_len).expect("internal error");
    block.poke_at(0, &opu.to_le_bytes());
    let full_block = block.into_full_slice();
    src.read_short_raw::<CopyTypeIndex>(&mut full_block[op_len as usize..], literal_len as usize);
    let _ = dst.truncate(start_pos + (op_len + literal_len));
}

fn sml_m<O: ShortWriter>(dst: &mut O, match_len: u32) {
    m(dst, opc::encode_sml_m(match_len), 1);
}

fn lrg_m<O: ShortWriter>(dst: &mut O, match_len: u32) {
    m(dst, opc::encode_lrg_m(match_len), 2);
}

fn m<O: ShortWriter>(dst: &mut O, opu: u32, op_len: u32) {
    // [PERFORMANCE_SENSITIVE] Replaced unsafe pointer manipulation with safe short_wide_block and truncate.
    debug_assert!(op_len <= 3);
    let start_pos = dst.pos();
    let mut block = dst.short_wide_block(op_len).expect("internal error");
    block.poke_at(0, &opu.to_le_bytes());
    let _ = dst.truncate(start_pos + op_len);
}

fn pre_d<I: ShortBuffer, O: ShortWriter>(
    dst: &mut O,
    src: &mut I,
    literal_len: u32,
    match_len: u32,
) {
    lmd(dst, src, literal_len, opc::encode_pre_d(literal_len, match_len), 1)
}

fn sml_d<I: ShortBuffer, O: ShortWriter>(
    dst: &mut O,
    src: &mut I,
    literal_len: u32,
    match_len: u32,
    match_distance: u32,
) {
    lmd(dst, src, literal_len, opc::encode_sml_d(literal_len, match_len, match_distance), 2)
}

fn med_d<I: ShortBuffer, O: ShortWriter>(
    dst: &mut O,
    src: &mut I,
    literal_len: u32,
    match_len: u32,
    match_distance: u32,
) {
    lmd(dst, src, literal_len, opc::encode_med_d(literal_len, match_len, match_distance), 3)
}

fn lrg_d<I: ShortBuffer, O: ShortWriter>(
    dst: &mut O,
    src: &mut I,
    literal_len: u32,
    match_len: u32,
    match_distance: u32,
) {
    lmd(dst, src, literal_len, opc::encode_lrg_d(literal_len, match_len, match_distance), 3)
}

fn lmd<I: ShortBuffer, O: ShortWriter>(
    dst: &mut O,
    src: &mut I,
    literal_len: u32,
    opu: u32,
    op_len: u32,
) {
    // [PERFORMANCE_SENSITIVE] Replaced unsafe pointer manipulation with safe short_wide_block.
    debug_assert!(literal_len <= 4);
    debug_assert!(literal_len as usize <= src.len());
    debug_assert!(op_len <= 3);
    let literal_bytes = src.peek_u32();
    let start_pos = dst.pos();
    let mut block = dst.short_wide_block(op_len + literal_len).expect("internal error");
    block.poke_at(0, &opu.to_le_bytes());
    block.poke_at(op_len as usize, &literal_bytes.to_le_bytes());
    let _ = dst.truncate(start_pos + (op_len + literal_len));
}
