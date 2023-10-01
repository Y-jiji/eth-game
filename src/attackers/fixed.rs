use crate::environment::interfaces::Attacker;
use revm::interpreter::*;
use revm::primitives::*;

pub struct AttackerFixed {
    count: usize,
    group: Vec<Vec<CallInputs>>,
}

impl Attacker for AttackerFixed {
    type State = ();
    fn init(&mut self, _contracts: &[(B160, Bytes)]) -> Self::State {}
    fn check(&self, _state: &mut Self::State) -> bool { true }
    fn make_call(&self, state: &mut Self::State) -> Option<CallInputs> {
        None
    }
    fn take_return(&self, state: &mut Self::State, ret: InstructionResult, gas: Gas, out: Bytes) {
    }
}