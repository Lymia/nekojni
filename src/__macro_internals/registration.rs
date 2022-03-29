use crate::errors::*;

pub trait Registration<const ID: usize> {
    fn run_chain(self) -> Result<()>;
}

pub struct DerefRamp<const ID: usize, T>(pub T);

pub trait DerefRampChainA {
    fn run_chain(self) -> Result<()>;
}
impl<const ID: usize, T> DerefRampChainA for DerefRamp<ID, T>
where T: Registration<ID>
{
    fn run_chain(self) -> Result<()> {
        self.0.run_chain()
    }
}

pub trait DerefRampChainB {
    fn run_chain(self) -> Result<()>;
}
impl<const ID: usize, T> DerefRampChainB for &DerefRamp<ID, T> {
    fn run_chain(self) -> Result<()> {
        Ok(())
    }
}
