pub trait Driver {
    fn init(&self) -> Result<(), &'static str> {
        Ok(())
    }

    fn name(&self) -> &str;
}