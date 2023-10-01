use revm::interpreter::*;
use revm::primitives::*;
use revm::{create_evm_impl, Database, EVMData, Inspector};
use super::interfaces::*;

pub struct GameInspector<DP: Defender, AP: Attacker> {
    // the upper limit of attacker actions
    limit: usize,
    // set of targets, attacker account
    accounts: (HashSet<B160>, B160),
    // pure strategy of a defender
    defender: DP,
    defstate: Vec<DP::State>,
    // pure strategy of an attacker
    attacker: AP,
    attstate: AP::State,
}

impl<DP: Defender, AP: Attacker> GameInspector<DP, AP> {
}

impl<DB: Database, DP: Defender, AP: Attacker> 
    Inspector<DB> for GameInspector<DP, AP> 
{
    fn call(
        &mut self,
        data: &mut EVMData<'_, DB>,
        inputs: &mut CallInputs,
    ) -> (InstructionResult, Gas, Bytes) {
        if self.accounts.0.contains(&inputs.contract) {
            // check before function calls
            let (state, ok) = self.defender.check(self.defstate.last().unwrap_or_else(|| panic!()), inputs);
            self.defstate.push(state);
            let ok = if ok { InstructionResult::Continue } else { InstructionResult::Revert };
            (ok, Gas::new(0), Bytes::default())
        }
        else if self.accounts.1 == inputs.contract {
            // iterate over attacker calls, let attacker process input
            while self.limit > 0 {
                let Some(mut call) = self.attacker.make_call(&mut self.attstate)
                    else { break };
                self.limit -= 1;
                let (ret, gas, out) = create_evm_impl::<DB, true>(data, self).call(&mut call);
                self.attacker.take_return(&mut self.attstate, ret, gas, out);
            }
            // decide whether a call should fail
            let ok = self.attacker.check(&mut self.attstate);
            let ok = if ok { InstructionResult::Continue } else { InstructionResult::Revert };
            (ok, Gas::new(0), Bytes::default())
        }
        else {
            // default: do nothing
            (InstructionResult::Continue, Gas::new(0), Bytes::default())
        }
    }
    fn call_end(
        &mut self,
        _data: &mut EVMData<'_, DB>,
        inputs: &CallInputs,
        remaining_gas: Gas,
        ret: InstructionResult,
        out: Bytes,
    ) -> (InstructionResult, Gas, Bytes) {
        if self.accounts.0.contains(&inputs.contract) {
            self.defstate.pop();
        }
        (ret, remaining_gas, out)
    }
}