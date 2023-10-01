use crate::{environment, attackers::{self, AttackerFixed}, defenders::{self, DefenderPermissive}};
use ethers::abi::Token;

#[test]
fn test_environment() {
    use ethers::abi::parse_abi;
    use revm::primitives::*;
    let abi = parse_abi(&[
        "function donate(address _to) public payable",
        "function balanceOf(address _who) public view returns (uint balance)",
        "function withdraw(uint _amount) public",
    ]).unwrap();
    let bin = hex::decode(include_str!("../../test-resources/Reentrance.bin")).unwrap().into();
    let mut env = environment::Environment::<AttackerFixed, DefenderPermissive, true>::new(10);
    env.load_contracts(vec![bin]);
    let target = env.get_contracts()[0].0;
    let attacker_account = env.create_attacker_account();
    let x = env.attacker_balance();
    let donate : Bytes = abi.function("donate").unwrap().encode_input(&[Token::Address(attacker_account.into())]).unwrap().into();
    let withdraw: Bytes = abi.function("withdraw").unwrap().encode_input(&[Token::Uint(U256::from(10000).into())]).unwrap().into();
    let attacker = attackers::AttackerFixed::new(vec![
        vec![(target, U256::from(0), withdraw.clone()), (target, U256::from(10000), donate)],
        vec![(target, U256::from(0), withdraw.clone())],
        vec![(target, U256::from(0), withdraw.clone())],
        vec![(target, U256::from(0), withdraw.clone())],
        vec![(target, U256::from(0), withdraw.clone())],
    ]);
    let defender = defenders::DefenderPermissive;
    env.load_attacker(attacker);
    env.load_defender(defender);
    let y = env.compute();
    println!("===");
    println!("attacker balance:\n\t{x} -> {y}");
}