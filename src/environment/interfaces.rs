use revm::primitives::B160;
use super::bitwig;

pub struct AttackerInfo {
    // target contracts
    targets: Vec<(B160, Vec<u8>, Vec<(u64, u64)>)>,
    // the number of functions to use
    chances: usize,
}

pub struct AttackerAction {
    // if the attacker is malicious for this action
    malicious: bool,
    // a pack of function calls
    fcallpack: Vec<Vec<(B160, u64, Vec<u8>)>>,
}

pub trait Attacker {
    fn act(&mut self, x: AttackerInfo) -> AttackerAction;
    fn improve(&mut self);
}

pub struct DefenderInfo {
    // target contracts
    targets: Vec<(B160, Vec<u8>, Vec<(u64, u64)>)>,
}

pub struct DefenderAction {
    // defending conditions for each function in each contract
    hook: Vec<(B160, Vec<(u64, Vec<bitwig::Instruction>)>)>,
}

pub trait Defender {
    fn act(&mut self, x: DefenderInfo) -> DefenderAction;
    fn improve(&mut self);
}

pub trait Runner {
    // get attacker information
    fn attacker_info(&self) -> AttackerInfo;
    // get defender information
    fn defender_info(&self) -> DefenderInfo;
    // load attacker action
    fn attacker_load(&mut self, action: AttackerAction);
    // load defender action
    fn defender_load(&mut self, action: DefenderAction);
    // return attacker money delta and execution trace
    fn evaluate_util(&mut self) -> (i128, Vec<u8>);
}