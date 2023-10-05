use crate::environment::{self, interfaces::Defender};
use crate::{attackers, defenders};
use ethers::abi::Token;

fn test_environment_with<const TRACE: bool>(defender: impl Defender) {
    use ethers::abi::parse_abi;
    use revm::primitives::*;
    let abi = parse_abi(&[
        "function donate(address _to) public payable",
        "function balanceOf(address _who) public view returns (uint balance)",
        "function withdraw(uint _amount) public",
    ]).unwrap();
    let bin = hex::decode(include_str!("../../test-resources/Reentrance.bin")).unwrap().into();
    let mut env = environment::Environment::<_, _, TRACE>::new(10);
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
    env.load_attacker(attacker);
    env.load_defender(defender);
    let y = env.compute();
    println!("\n===");
    println!("attacker balance:\n\t{x} -> {y}");
}

#[test]
fn test_environment() {
    test_environment_with::<false>(defenders::DefenderPermissive);
    test_environment_with::<false>(defenders::DefenderDenial);
}