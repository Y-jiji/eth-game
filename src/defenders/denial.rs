use crate::environment::interfaces::Defender;
use revm::interpreter::*;
use revm::primitives::*;

pub struct DefenderDenial;

impl Defender for DefenderDenial {
    type State = ();
    fn check(&self, _state: &Self::State, _inputs: &CallInputs) -> (Self::State, bool) {
        ((), false)
    }
    fn init(&mut self, _contracts: &[(B160, Bytes)]) -> Self::State {
        ()
    }
}