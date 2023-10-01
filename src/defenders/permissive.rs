use crate::environment::interfaces::Defender;
use revm::interpreter::*;
use revm::primitives::*;

pub struct DefenderPermissive;

impl Defender for DefenderPermissive {
    type State = ();
    fn check(&self, _state: &Self::State, _inputs: &CallInputs) -> (Self::State, bool) {
        ((), true)
    }
    fn init(&mut self, _contracts: &[(B160, Bytes)]) -> Self::State {
        ()
    }
}