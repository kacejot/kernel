use super::result::KernelResult;

pub trait Driver {
    fn init(&self) -> KernelResult {
        Ok(())
    }

    fn name(&self) -> &str;
}