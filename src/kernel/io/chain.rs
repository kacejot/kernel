use crate::kernel::io;
use core::marker;

pub struct Chain<T, U, E> {
    first: T,
    second: U,
    done_first: bool,
    _phantom: marker::PhantomData<fn() -> E>,
}

impl<T, U, E> Chain<T, U, E> {
    pub fn new(first: T, second: U) -> Chain<T, U, E> {
        Chain {
            first: first,
            second: second,
            done_first: false,
            _phantom: marker::PhantomData,
        }
    }
}

impl<T: io::Read, U: io::Read, E> io::Read for Chain<T, U, E>
    where E: From<T::Err> + From<U::Err>
{
    type Err = E;

    fn read(&mut self, buf: &mut [u8]) -> Result<usize, E> {
        if !self.done_first {
            match self.first.read(buf)? {
                0 => { self.done_first = true; }
                n => return Ok(n),
            }
        }
        self.second.read(buf).map_err(From::from)
    }
}
