use std::io;

pub trait Allocate {
    /// Allocate `len` bytes returning `io::ErrorKind::Other` in case of failure.
    fn allocate(&mut self, len: usize) -> io::Result<()>;
}

impl<T: Allocate + ?Sized> Allocate for &mut T {
    #[inline(always)]
    fn allocate(&mut self, additional: usize) -> io::Result<()> {
        (**self).allocate(additional)
    }
}


impl Allocate for Vec<u8> {
    // TODO: Issue 48043, use `vec::try_reserve`
    #[cfg(target_pointer_width = "32")]
    fn allocate(&mut self, len: usize) -> io::Result<()> {
        let index = self.len();
        if !self.is_allocated(len) {
            if index + len > isize::MAX as usize {
                // Unlikely.
                return Err(io::ErrorKind::Other.into());
            }
            self.reserve(len);
        }
        Ok(())
    }

    #[inline(always)]
    fn allocate(&mut self, additional: usize) -> io::Result<()> {
        self.reserve(additional);
        Ok(())
    }
    }

