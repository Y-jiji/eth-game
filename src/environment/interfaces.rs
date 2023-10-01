use revm::interpreter::*;
use revm::primitives::*;

pub trait Defender {
    type State;
    // initialize a state
    fn init(&mut self, contracts: &[(B160, Bytes)]) -> Self::State;
    // check current contract
    fn check(&self, state: &Self::State, inputs: &CallInputs) -> (Self::State, bool);
}

pub trait Attacker {
    type State;
    // initialize a state
    fn init(&mut self, contracts: &[(B160, Bytes)]) -> Self::State;
    // generate a sequence of function calls
    fn make_mal_call(&self, state: &mut Self::State) -> Option<(B160, U256, Bytes)>;
    // process a call return
    fn take_return(&self, state: &mut Self::State, ret: InstructionResult, gas: Gas, out: Bytes);
    // check made by attacker
    fn check(&self, state: &mut Self::State) -> bool;
}
