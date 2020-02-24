use crate::kernel::io;
use core::fmt;

pub trait Write {
    type Err;

    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Err>;

    fn write_all<E>(&mut self, mut buf: &[u8]) -> Result<(), E>
    where
        E: From<Self::Err> + From<io::EndOfFile>,
    {
        while buf.len() > 0 {
            match self.write(buf)? {
                0 => return Err(E::from(io::EndOfFile)),
                n => buf = &buf[n..],
            }
        }
        Ok(())
    }

    fn write_fmt<E>(&mut self, fmt: fmt::Arguments) -> Result<(), E>
    where
        E: From<Self::Err> + From<io::EndOfFile>,
    {
        // Create a shim which translates a Write to a fmt::Write and saves
        // off I/O errors. instead of discarding them
        struct Adaptor<'a, T: ?Sized + 'a, E> {
            inner: &'a mut T,
            result: Result<(), E>,
        }

        impl<'a, T: ?Sized, F> fmt::Write for Adaptor<'a, T, F>
        where
            T: Write,
            F: From<io::EndOfFile> + From<T::Err>,
        {
            fn write_str(&mut self, s: &str) -> fmt::Result {
                match self.inner.write_all(s.as_bytes()) {
                    Ok(()) => Ok(()),
                    Err(e) => {
                        self.result = Err(e);
                        Err(fmt::Error)
                    }
                }
            }
        }

        let mut output = Adaptor {
            inner: self,
            result: Ok(()),
        };

        match fmt::write(&mut output, fmt) {
            Ok(()) => Ok(()),
            Err(..) => {
                if output.result.is_err() {
                    output.result
                } else {
                    output.result
                    //Err(Error::new(ErrorKind::Other, "formatter error"))
                }
            }
        }
    }
}
