use crate::errors::*;

pub trait Registration<const ID: usize> {
    fn run_chain_fwd(&self);
    fn run_chain_rev(&self);
}

pub struct DerefRamp<'a, const ID: usize, T>(pub &'a T);
impl<'a, const ID: usize, T> Copy for DerefRamp<'a, ID, T> {}
impl<'a, const ID: usize, T> Clone for DerefRamp<'a, ID, T> {
    fn clone(&self) -> Self {
        *self
    }
}

pub trait DerefRampChainA {
    fn run_chain_fwd(self);
    fn run_chain_rev(self);
}
impl<'a, const ID: usize, T> DerefRampChainA for &DerefRamp<'a, ID, T>
where T: Registration<ID>
{
    #[inline(always)]
    fn run_chain_fwd(self) {
        self.0.run_chain_fwd()
    }
    #[inline(always)]
    fn run_chain_rev(self) {
        self.0.run_chain_rev()
    }
}

pub trait DerefRampChainB {
    fn run_chain_fwd(self);
    fn run_chain_rev(self);
}
impl<'a, const ID: usize, T> DerefRampChainB for DerefRamp<'a, ID, T> {
    #[inline(always)]
    fn run_chain_fwd(self) {}
    #[inline(always)]
    fn run_chain_rev(self) {}
}
