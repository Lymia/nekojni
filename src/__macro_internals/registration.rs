use crate::errors::*;

pub trait Registration<const ID: usize> {
    fn run_chain_fwd(self) -> Result<()>;
    fn run_chain_rev(self) -> Result<()>;
}

pub struct DerefRamp<const ID: usize, T>(pub T);

pub trait DerefRampChainA {
    fn run_chain_fwd(self) -> Result<()>;
    fn run_chain_rev(self) -> Result<()>;
}
impl<const ID: usize, T> DerefRampChainA for DerefRamp<ID, T>
where T: Registration<ID>
{
    fn run_chain_fwd(self) -> Result<()> {
        self.0.run_chain_fwd()
    }
    fn run_chain_rev(self) -> Result<()> {
        self.0.run_chain_rev()
    }
}

pub trait DerefRampChainB {
    fn run_chain_fwd(self) -> Result<()>;
    fn run_chain_rev(self) -> Result<()>;
}
impl<const ID: usize, T> DerefRampChainB for &DerefRamp<ID, T> {
    fn run_chain_fwd(self) -> Result<()> {
        Ok(())
    }
    fn run_chain_rev(self) -> Result<()> {
        Ok(())
    }
}
