use crate::kernel::io;
use core::cmp;

pub struct Take<T> {
    inner: T,
    limit: u64,
}

impl<T> Take<T> {
    pub fn new(inner: T, limit: u64) -> Take<T> {
        Take {
            inner: inner,
            limit: limit,
        }
    }

    pub fn limit(&self) -> u64 {
        self.limit
    }
}

impl<T: io::Read> io::Read for Take<T> {
    type Err = T::Err;

    fn read(&mut self, buf: &mut [u8]) -> Result<usize, T::Err> {
        if self.limit == 0 {
            return Ok(0);
        }

        let max = cmp::min(buf.len() as u64, self.limit) as usize;
        let n = self.inner.read(&mut buf[..max])?;
        self.limit -= n as u64;
        Ok(n)
    }
}
