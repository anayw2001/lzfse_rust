use crate::kit::WIDE;
use crate::types::WideBytesMut;

use super::allocate::Allocate;
use super::short_limit::ShortLimit;

use std::io;

/// Low-level write buffer access methods.
pub trait WriteShort: Allocate + ShortLimit {
    // Little endian.
    #[allow(dead_code)]
    #[inline(always)]
    fn write_short_u8(&mut self, u: u8) -> io::Result<()> {
        self.write_short_bytes(&u.to_le_bytes())
    }

    // Little endian.
    #[allow(dead_code)]
    #[inline(always)]
    fn write_short_u16(&mut self, u: u16) -> io::Result<()> {
        self.write_short_bytes(&u.to_le_bytes())
    }

    // Little endian.
    #[inline(always)]
    fn write_short_u32(&mut self, u: u32) -> io::Result<()> {
        self.write_short_bytes(&u.to_le_bytes())
    }

    // Little endian.
    #[inline(always)]
    fn write_short_u64(&mut self, u: u64) -> io::Result<()> {
        self.write_short_bytes(&u.to_le_bytes())
    }

    // Little endian.
    #[allow(dead_code)]
    #[inline(always)]
    fn write_short_u128(&mut self, u: u128) -> io::Result<()> {
        self.write_short_bytes(&u.to_le_bytes())
    }

    // Little endian.
    #[allow(dead_code)]
    #[inline(always)]
    fn write_short_usize(&mut self, u: usize) -> io::Result<()> {
        self.write_short_bytes(&u.to_le_bytes())
    }

    #[inline(always)]
    fn write_short_bytes(&mut self, bytes: &[u8]) -> io::Result<()> {
        // [PERFORMANCE_SENSITIVE] Replaced unsafe write with safe slice operations.
        assert!(bytes.len() <= Self::SHORT_LIMIT as usize);
        let len = bytes.len() as u32;
        let block = self.short_block(len)?;
        block.copy_from_slice(bytes);
        Ok(())
    }

    fn short_block(&mut self, len: u32) -> io::Result<&mut [u8]>;

    #[allow(dead_code)]
    fn short_wide_block(&mut self, len: u32) -> io::Result<WideBytesMut<'_>>;
}

impl<T: WriteShort + ?Sized> WriteShort for &mut T {
    #[allow(dead_code)]
    #[inline(always)]
    fn write_short_u8(&mut self, u: u8) -> io::Result<()> {
        (**self).write_short_u8(u)
    }

    #[allow(dead_code)]
    #[inline(always)]
    fn write_short_u16(&mut self, u: u16) -> io::Result<()> {
        (**self).write_short_u16(u)
    }

    #[inline(always)]
    fn write_short_u32(&mut self, u: u32) -> io::Result<()> {
        (**self).write_short_u32(u)
    }

    #[inline(always)]
    fn write_short_u64(&mut self, u: u64) -> io::Result<()> {
        (**self).write_short_u64(u)
    }

    #[allow(dead_code)]
    #[inline(always)]
    fn write_short_u128(&mut self, u: u128) -> io::Result<()> {
        (**self).write_short_u128(u)
    }

    #[allow(dead_code)]
    #[inline(always)]
    fn write_short_usize(&mut self, u: usize) -> io::Result<()> {
        (**self).write_short_usize(u)
    }

    #[inline(always)]
    fn write_short_bytes(&mut self, bytes: &[u8]) -> io::Result<()> {
        (**self).write_short_bytes(bytes)
    }

    #[inline(always)]
    fn short_block(&mut self, len: u32) -> io::Result<&mut [u8]> {
        (**self).short_block(len)
    }

    #[allow(dead_code)]
    #[inline(always)]
    fn short_wide_block(&mut self, len: u32) -> io::Result<WideBytesMut<'_>> {
        (**self).short_wide_block(len)
    }
}

impl WriteShort for Vec<u8> {
    #[inline(always)]
    fn short_block(&mut self, len: u32) -> io::Result<&mut [u8]> {
        // [PERFORMANCE_SENSITIVE] Replaced unsafe manual length management with safe resize.
        let len = len as usize;
        self.allocate(len)?;
        let index = self.len();
        self.resize(index + len, 0);
        Ok(&mut self[index..index + len])
    }

    #[inline(always)]
    fn short_wide_block(&mut self, len: u32) -> io::Result<WideBytesMut<'_>> {
        // [PERFORMANCE_SENSITIVE] Replaced unsafe manual length management with safe resize.
        let len = len as usize;
        self.allocate(len + WIDE)?;
        let index = self.len();
        self.resize(index + len + WIDE, 0);
        Ok(WideBytesMut::from_bytes(&mut self[index..index + len + WIDE], len))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vec_0() -> crate::Result<()> {
        let mut vec = vec![0, 1, 2, 3, 4];
        let bytes = vec.short_block(0)?;
        assert_eq!(bytes.len(), 0);
        assert_eq!(vec, vec![0, 1, 2, 3, 4]);
        Ok(())
    }

    #[test]
    fn vec_1() -> crate::Result<()> {
        let mut vec = vec![0, 1, 2, 3, 4];
        let bytes = vec.short_block(1)?;
        assert_eq!(bytes.len(), 1);
        bytes[0] = 5;
        assert_eq!(vec, vec![0, 1, 2, 3, 4, 5]);
        Ok(())
    }

    #[test]
    fn vec_2() -> crate::Result<()> {
        let mut vec = vec![0, 1, 2, 3, 4];
        let bytes = vec.short_block(1)?;
        bytes[0] = 5;
        let bytes = vec.short_block(2)?;
        bytes[0] = 6;
        bytes[1] = 7;
        assert_eq!(vec, vec![0, 1, 2, 3, 4, 5, 6, 7]);
        Ok(())
    }

    #[test]
    fn vec_3() -> crate::Result<()> {
        let mut vec = vec![0, 1, 2, 3, 4];
        vec.write_short_bytes(&[])?;
        assert_eq!(vec, vec![0, 1, 2, 3, 4]);
        Ok(())
    }

    #[test]
    fn vec_4() -> crate::Result<()> {
        let mut vec = vec![0, 1, 2, 3, 4];
        vec.write_short_bytes(&[5])?;
        assert_eq!(vec, vec![0, 1, 2, 3, 4, 5]);
        Ok(())
    }

    #[test]
    fn vec_5() -> crate::Result<()> {
        let mut vec = vec![0, 1, 2, 3, 4];
        vec.write_short_bytes(&[5])?;
        vec.write_short_bytes(&[6])?;
        assert_eq!(vec, vec![0, 1, 2, 3, 4, 5, 6]);
        Ok(())
    }
}
