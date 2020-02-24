use crate::kernel::io;

pub trait Read {
    type Err;

    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Err>;

    fn read_exact<E>(&mut self, mut buf: &mut [u8]) -> Result<(), E>
    where
        E: From<Self::Err> + From<io::EndOfFile>,
    {
        while buf.len() > 0 {
            match self.read(&mut buf)? {
                0 => return Err(E::from(io::EndOfFile)),
                n => {
                    let tmp = buf;
                    buf = &mut tmp[n..]
                }
            }
        }
        Ok(())
    }

    fn by_ref(&mut self) -> &mut Self
    where
        Self: Sized,
    {
        self
    }

    fn chain<R: Read, E>(self, next: R) -> io::Chain<Self, R, E>
    where
        Self: Sized,
        E: From<Self::Err> + From<R::Err>,
    {
        io::Chain::new(self, next)
    }

    fn take(self, limit: u64) -> io::Take<Self>
    where
        Self: Sized,
    {
        io::Take::new(self, limit)
    }
}
