use super::wide::Width;

pub trait CopyType {
    fn wide_copy<W: Width>(src: &[u8], dst: &mut [u8], len: usize);
}

#[derive(Copy, Clone, Debug)]
pub struct CopyTypeIndex;

impl CopyType for CopyTypeIndex {
    #[inline(always)]
    fn wide_copy<W: Width>(src: &[u8], dst: &mut [u8], len: usize) {
        // [PERFORMANCE_SENSITIVE] Replaced unsafe wide copy with safe slice operations.
        let mut off = 0;
        loop {
            let chunk_len = W::WIDTH;
            if off + chunk_len <= src.len() && off + chunk_len <= dst.len() {
                dst[off..off + chunk_len].copy_from_slice(&src[off..off + chunk_len]);
            } else {
                let remaining = len.saturating_sub(off);
                if remaining > 0 {
                    dst[off..off + remaining].copy_from_slice(&src[off..off + remaining]);
                }
                break;
            }
            off += chunk_len;
            if off >= len {
                break;
            }
        }
    }
}

#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
pub struct CopyTypePtr;

impl CopyType for CopyTypePtr {
    #[inline(always)]
    fn wide_copy<W: Width>(src: &[u8], dst: &mut [u8], len: usize) {
        // [PERFORMANCE_SENSITIVE] Replaced unsafe wide copy with safe slice operations.
        let mut off = 0;
        loop {
            let chunk_len = W::WIDTH;
            if off + chunk_len <= src.len() && off + chunk_len <= dst.len() {
                dst[off..off + chunk_len].copy_from_slice(&src[off..off + chunk_len]);
            } else {
                let remaining = len.saturating_sub(off);
                if remaining > 0 {
                    dst[off..off + remaining].copy_from_slice(&src[off..off + remaining]);
                }
                break;
            }
            off += chunk_len;
            if off >= len {
                break;
            }
        }
    }
}

// High latency, high throughput.
#[derive(Copy, Clone, Debug)]
pub struct CopyTypeLong;

impl CopyType for CopyTypeLong {
    #[inline(always)]
    fn wide_copy<W: Width>(src: &[u8], dst: &mut [u8], len: usize) {
        // [PERFORMANCE_SENSITIVE] Replaced unsafe wide copy with safe slice operations.
        const K: usize = 8;
        let mut off = 0;
        if len >= W::WIDTH * K {
            let chunk_len = W::WIDTH * K;
            let wide_len = (len / chunk_len) * chunk_len;
            loop {
                if off + chunk_len <= src.len() && off + chunk_len <= dst.len() {
                    dst[off..off + chunk_len].copy_from_slice(&src[off..off + chunk_len]);
                } else {
                    break;
                }
                off += chunk_len;
                if off == wide_len {
                    break;
                }
            }
        }
        loop {
            let chunk_len = W::WIDTH;
            if off + chunk_len <= src.len() && off + chunk_len <= dst.len() {
                dst[off..off + chunk_len].copy_from_slice(&src[off..off + chunk_len]);
            } else {
                let remaining = len.saturating_sub(off);
                if remaining > 0 {
                    dst[off..off + remaining].copy_from_slice(&src[off..off + remaining]);
                }
                break;
            }
            off += chunk_len;
            if off >= len {
                break;
            }
        }
    }
}
