use revm::interpreter::*;
use revm::primitives::*;
use revm::{create_evm_impl, Database, EVMData, Inspector};
use super::interfaces::*;

pub struct GameInspector<DP: Defender, AP: Attacker, const TRACE: bool = false> {
    // the upper limit of attacker actions
    pub limit: usize,
    // set of targets, attacker account
    pub accounts: (HashSet<B160>, B160),
    // pure strategy of a defender
    pub defender: DP,
    pub defstate: Vec<DP::State>,
    // pure strategy of an attacker
    pub attacker: AP,
    pub attstate: AP::State,
}

impl<DB: Database, DP: Defender, AP: Attacker, const TRACE: bool> 
    Inspector<DB> for GameInspector<DP, AP, TRACE>
{
    fn initialize_interp(&mut self, interp: &mut Interpreter, _data: &mut EVMData<'_,DB> ,) -> InstructionResult {
        if !TRACE { return InstructionResult::Continue }
        println!("===");
        if interp.contract.address == self.accounts.1 {
            println!("contract attacker");
        }
        else if self.accounts.0.contains(&interp.contract.address) {
            println!("contract defender");
        }
        else {
            println!("contract {}", interp.contract.address);
        }
        InstructionResult::Continue
    }
    fn step(&mut self, interp: &mut Interpreter, _data: &mut EVMData<'_,DB> ) -> InstructionResult {
        if !TRACE { return InstructionResult::Continue }
        if interp.contract.address == self.accounts.1 {
            InstructionResult::Stop
        } else {
            InstructionResult::Continue
        }
    }
    fn step_end(&mut self, interp: &mut Interpreter,_data: &mut EVMData<'_,DB> ,_eval:InstructionResult,) -> InstructionResult {
        if !TRACE { return InstructionResult::Continue }
        use std::fmt::Write;
        let code_nr = interp.program_counter();
        let code_nm = revm::interpreter::opcode::OPCODE_JUMPMAP[interp.current_opcode() as usize].unwrap_or("UNKNOWN");
        let mut string = String::new();
        writeln!(&mut string, "{code_nr:<5}{code_nm}").unwrap();
        for d in interp.stack().data() {
            let d = d.as_limbs();
            writeln!(&mut string, "        {:016x}:{:016x}:{:016x}:{:016x}  [STACK]", d[0], d[1], d[2], d[3]).unwrap();
        }
        print!("{string}");
        InstructionResult::Continue
    }
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
                let Some((addr, value, input)) = self.attacker.make_call(&mut self.attstate)
                    else { break };
                self.limit -= 1;
                // boilerplate call
                let (ret, gas, out) = create_evm_impl::<DB, true>(data, self)
                    .call(&mut CallInputs {
                        contract: addr, transfer: Transfer { source: inputs.contract, target: addr, value }, 
                        input, gas_limit: 10_000_000, is_static: false, 
                        context: CallContext { 
                            address: addr, caller: inputs.contract, 
                            code_address: addr, 
                            apparent_value: value, scheme: CallScheme::Call, 
                        }, 
                    });
                self.attacker.take_return(&mut self.attstate, ret, gas, out);
            }
            // decide whether a call should fail
            let ok = self.attacker.check(&mut self.attstate);
            let ok = if ok { InstructionResult::Continue } else {InstructionResult::Revert };
            (ok, Gas::new(0), Bytes::default())
        }
        else {
            println!("????");
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