use revm::interpreter::*;
use revm::primitives::*;
use revm::primitives::bitvec::view::AsBits;
use super::bitwig;
use bitvec::vec::BitVec;
use revm::{Inspector, create_evm_impl, Database, EVMData};

pub struct GameInspector {
    // adimistrator account
    admin: B160,
    // set of function calls
    target: Vec<(B160, u32)>,
    // defender's action
    defender: HashMap<(B160, u32), Vec<bitwig::Instruction>>,
    // defender state stack
    dfdstate: Vec<Vec<bool>>,
    // attacker address, code for calls and code for returns
    attacker: (B160, Vec<bitwig::Instruction>, Vec<bitwig::Instruction>),
    // attacker state in bitlanguage
    attstate: Vec<bool>,
    // the rest chances of the attacker
    attacker_rest_calls: usize,
}

impl<DB: Database> Inspector<DB> for GameInspector {
    fn call(&mut self, data: &mut EVMData<'_,DB> , inputs: &mut CallInputs) -> (InstructionResult, Gas, Bytes) {
        let contract = inputs.contract;
        let bytes = &inputs.input;
        let funct: u32 = u32::from_be_bytes(bytes.get(..4)
            .map(|x| x.try_into().expect("length don't match"))
            .unwrap_or([0u8; 4]));
        if let Some(wig) = self.defender.get(&(contract, funct)) {
            let prev = self.dfdstate.last().unwrap();
            let next = bitwig::evaluate(bytes.as_bits(), prev.clone(), wig);
            let x = next[0]; self.dfdstate.push(next);
            (if x { InstructionResult::Continue } else { InstructionResult::Revert }, 
                Gas::new(0), Bytes::default())
        } else if contract == self.attacker.0 {
            // a state change using function 1
            self.attstate = bitwig::evaluate(
                &BitVec::from_element(1u8), 
                self.attstate.clone(), 
                &self.attacker.1
            );
            // struct EVMImpl is tweaked, we can execute a function call from here
            let mut evm = create_evm_impl::<DB, true>(data, self);
            evm.call(inputs);
            todo!()
        } else {
            (InstructionResult::Continue, Gas::new(0), Bytes::default())
        }
    }
    fn call_end(
        &mut self, _data: &mut EVMData<'_,DB>, inputs: &CallInputs, 
        remaining_gas: Gas, ret: InstructionResult, out: Bytes
    ) -> (InstructionResult, Gas, Bytes) {
        let contract = inputs.contract;
        let bytes = &inputs.input;
        let funct: u32 = u32::from_be_bytes(bytes.get(..4)
            .map(|x| x.try_into().expect("length don't match"))
            .unwrap_or([0u8; 4]));
        if self.defender.contains_key(&(contract, funct)) { self.dfdstate.pop(); }
        if inputs.context.caller == self.attacker.0 {
            self.attstate = bitwig::evaluate(out.as_bits(), self.attstate.clone(), &self.attacker.2);
        }
        (ret, remaining_gas, out)
    }
}