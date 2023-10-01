use ethers::abi::Abi;
use revm::primitives::Bytes;
use std::process::Command;

pub fn compile_solidity(solc: &str, source: &str) -> (Bytes, Abi) {
    Command::new("rm")
        .args(&["-rf", "tmp"])
        .spawn().unwrap()
        .wait().unwrap();
    Command::new(solc)
        .args(&[source, "--abi", "--bin", "-o", "tmp"])
        .spawn().unwrap()
        .wait().unwrap();
    let dir = std::fs::read_dir("tmp/").unwrap().map(|x| x.unwrap().path()).collect::<Vec<_>>();
    let (mut a, mut b) = (0, 1);
    if dir[a].to_str().unwrap().ends_with(".bin") {
        std::mem::swap(&mut a, &mut b);
    }
    let abi = ethers::abi::Abi::load(
        std::fs::OpenOptions::new().read(true).open(&dir[a]).unwrap()).unwrap();
    let bin = std::fs::read(&dir[b]).unwrap();
    Command::new("rm")
        .args(&["-rf", "tmp"])
        .spawn().unwrap()
        .wait().unwrap();
    (bin.into(), abi)
}