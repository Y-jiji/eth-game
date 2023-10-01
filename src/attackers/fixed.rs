use crate::environment::interfaces::Attacker;
use revm::interpreter::*;
use revm::primitives::*;

pub struct AttackerFixed {
    group: Vec<Vec<(B160, U256, Bytes)>>,
}

impl AttackerFixed {
    pub fn new(group: Vec<Vec<(B160, U256, Bytes)>>) -> Self {
        AttackerFixed { group }
    }
}

impl Attacker for AttackerFixed {
    type State = (usize, Vec<Vec<(B160, U256, Bytes)>>);
    fn init(&mut self, _contracts: &[(B160, Bytes)]) -> Self::State {
        (0, self.group.clone())
    }
    fn check(&self, _state: &mut Self::State) -> bool { true }
    fn make_mal_call(&self, state: &mut Self::State) -> Option<(B160, U256, Bytes)> {
        let (count, group) = state; *count += 1;
        if *count > group.len() { None }
        else {
            group[*count - 1].pop()
        }
    }
    fn take_return(&self, state: &mut Self::State, _ret: InstructionResult, _gas: Gas, _out: Bytes) {
        let (count, _group) = state;
        *count -= 1;
    }
}