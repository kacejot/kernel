mod read;
pub use read::Read;

mod write;
pub use write::Write;

mod chain;
pub use chain::Chain;

mod take;
pub use take::Take;

// TODO: derieve from error::Error

#[derive(Copy, Clone, Debug)]
pub struct EndOfFile;

#[derive(Copy, Clone, Debug)]
pub struct OutOfBounds;

#[derive(Copy, Clone, Debug)]
pub struct FormatError;

pub trait Console = Read + Write;
