use super::ops;

use std::convert::TryInto;
use std::mem;

#[allow(dead_code)]
#[inline(always)]
pub fn fast_match_inc(
    bytes: &[u8],
    index: usize,
    match_index: usize,
    len: usize,
    max: usize,
) -> usize {
    assert!(match_index < index);
    assert!(index <= bytes.len());
    assert!(max <= bytes.len() - index);
    unsafe { fast_match_inc_unchecked(bytes, index, match_index, len, max) }
}

/// # Safety
///
/// * `match_index < index`
/// * `index <= bytes.len()`
/// * `max <= bytes.len() - index`
#[allow(clippy::missing_safety_doc)]
#[inline(always)]
pub unsafe fn fast_match_inc_unchecked(
    bytes: &[u8],
    index: usize,
    match_index: usize,
    mut len: usize,
    max: usize,
) -> usize {
    debug_assert!(match_index < index);
    debug_assert!(index <= bytes.len());
    debug_assert!(max <= bytes.len() - index);
    while len + mem::size_of::<usize>() <= max {
        let u_0 = usize::from_ne_bytes(
            bytes[index + len..index + len + mem::size_of::<usize>()]
                .try_into()
                .unwrap(),
        );
        let u_1 = usize::from_ne_bytes(
            bytes[match_index + len..match_index + len + mem::size_of::<usize>()]
                .try_into()
                .unwrap(),
        );
        let x = u_0 ^ u_1;
        if x != 0 {
            return len + ops::nclz_bytes(x) as usize;
        }
        len += mem::size_of::<usize>();
    }
    while len < max {
        if bytes[index + len] != bytes[match_index + len] {
            return len;
        }
        len += 1;
    }
    max
}

#[allow(dead_code)]
#[inline(always)]
pub fn fast_match_dec(bytes: &[u8], index: usize, match_index: usize, max: usize) -> usize {
    assert!(max <= match_index);
    assert!(match_index < index);
    assert!(index <= bytes.len());
    unsafe { fast_match_dec_unchecked(bytes, index, match_index, max) }
}

/// # Safety
///
/// * `max <= match_index`
/// * `match_index < index`
/// * `index <= bytes.len()`
#[inline(always)]
pub unsafe fn fast_match_dec_unchecked(
    bytes: &[u8],
    index: usize,
    match_index: usize,
    max: usize,
) -> usize {
    debug_assert!(max <= match_index);
    debug_assert!(match_index < index);
    debug_assert!(index <= bytes.len());
    let mut len = mem::size_of::<usize>();
    while len <= max {
        let u_0 = usize::from_ne_bytes(
            bytes[index - len..index - len + mem::size_of::<usize>()]
                .try_into()
                .unwrap(),
        );
        let u_1 = usize::from_ne_bytes(
            bytes[match_index - len..match_index - len + mem::size_of::<usize>()]
                .try_into()
                .unwrap(),
        );
        let x = u_0 ^ u_1;
        if x != 0 {
            return len - mem::size_of::<usize>() + ops::nctz_bytes(x) as usize;
        }
        len += mem::size_of::<usize>();
    }
    len -= mem::size_of::<usize>();
    while len != max {
        if bytes[index - len - 1] != bytes[match_index - len - 1] {
            break;
        }
        len += 1;
    }
    len
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "expensive"]
    fn match_inc_1() {
        let mut buf = [0u8; 64];
        for index in 1..buf.len() {
            let max = buf.len() - index;
            for match_index in 0..index {
                for match_len in 0..max {
                    for len in 0..match_len {
                        test_kit::build_match_inc(&mut buf, index, match_index, match_len);
                        let n = fast_match_inc(&buf, index, match_index, len, max);
                        assert_eq!(n, match_len);
                    }
                }
            }
        }
    }

    #[test]
    #[ignore = "expensive"]
    fn match_inc_2() {
        let mut buf = [0u8; 64];
        for index in 1..buf.len() {
            let match_len = buf.len() - index;
            for match_index in 0..index {
                test_kit::build_match_inc(&mut buf, index, match_index, match_len);
                for max in 0..match_len {
                    for len in 0..match_len {
                        let n = fast_match_inc(&buf, index, match_index, len, max);
                        assert_eq!(n, max);
                    }
                }
            }
        }
    }

    #[test]
    #[ignore = "expensive"]
    fn match_dec_1() {
        let mut buf = [0u8; 64];
        for index in 1..buf.len() {
            for match_index in 0..index {
                let max = match_index;
                for match_len in 0..max {
                    test_kit::build_match_dec(&mut buf, index, match_index, match_len);
                    let n = fast_match_dec(&buf, index, match_index, max);
                    assert_eq!(n, match_len);
                }
            }
        }
    }

    #[test]
    #[ignore = "expensive"]
    fn match_dec_2() {
        let mut buf = [0u8; 64];
        for index in 1..buf.len() {
            for match_index in 0..index {
                let match_len = match_index;
                test_kit::build_match_dec(&mut buf, index, match_index, match_len);
                for max in 0..match_len {
                    let n = fast_match_dec(&buf, index, match_index, max);
                    assert_eq!(n, max);
                }
            }
        }
    }
}
