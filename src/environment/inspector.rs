use revm::interpreter::*;
use revm::primitives::*;
use revm::{create_evm_impl, Database, EVMData, Inspector};

pub trait Defender {
    type State;
    // initialize a state
    fn init(&mut self, contracts: Vec<(B160, Bytes)>) -> Self::State;
    // check current contract
    fn check(&self, state: &Self::State, inputs: &CallInputs) -> (Self::State, bool);
}

pub trait Attacker {
    type State;
    // initialize a state
    fn init(&mut self, contracts: Vec<(B160, Bytes)>) -> Self::State;
    // generate a sequence of function calls
    fn make_call(&self, state: &mut Self::State) -> Option<CallInputs>;
    // process a call return
    fn take_return(&self, state: &mut Self::State, ret: InstructionResult, gas: Gas, out: Bytes);
    // check made by attacker
    fn check(&self, state: &mut Self::State) -> bool;
}

pub struct GameInspector<DP: Defender, AP: Attacker> {
    // adimistrator account
    admin: B160,
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

lazy_static::lazy_static!{
    static ref INIT_CODE: Bytes = 
        Bytes::from(hex::decode(
            "6080604052348015600f57600080fd5b50603f80601d6000396000f3fe6080604052600080fdfea26469706673582212203e9cd2e4b65a21d520d56991fdd1bc3ef05a91ad81ef47da4ff48e58372c440164736f6c63430008120033"
        ).unwrap());
}

impl<DB: Database, DP: Defender, AP: Attacker> 
    Inspector<DB> for GameInspector<DP, AP> 
{
    fn call(
        &mut self,
        data: &mut EVMData<'_, DB>,
        inputs: &mut CallInputs,
    ) -> (InstructionResult, Gas, Bytes) {
        if self.admin == inputs.contract {
            let admin = inputs.contract;
            // administrator action: just call the attacker
            let mut evm = create_evm_impl::<DB, true>(data, self);
            // call account creating
            let Some(address) = evm.create(
                &mut CreateInputs {
                    caller: admin, 
                    scheme: CreateScheme::Create, 
                    value: U256::MAX / U256::from(2), 
                    init_code: INIT_CODE.clone(), 
                    gas_limit: 1 << 25,
                }
            ).1
            else {
                panic!("attacker account cannot be created. ")
            };
            // call attacker code
            evm.call(&mut CallInputs {
                // attacker account
                contract: address, 
                transfer: Transfer { source: admin, target: address, value: U256::ZERO },
                input: Bytes::new(), 
                gas_limit: 1 << 25, 
                context: CallContext {
                    address, 
                    caller: admin, 
                    code_address: address, 
                    apparent_value: U256::ZERO, 
                    scheme: CallScheme::Call,
                }, 
                is_static: false, 
            })
        }
        else if self.accounts.0.contains(&inputs.contract) {
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