use revm::primitives::*;
use revm::interpreter::*;
use crate::environment::interfaces::Attacker;
use bitvec::view::AsBits;

pub struct AttackerNeural {
    script_make_call: Vec<u32>,
    script_proj_data: Vec<u32>,
    proj_skip_call: [u32; 1024],
    proj_value: [u32; 1024],
    proj_contract: [u32; 1024],
    contracts: Vec<B160>,
    script_take_return: Vec<u32>,
    script_check: Vec<u32>,
    proj_check: [u32; 1024],
}

impl Attacker for AttackerNeural {
    type State = [u32; 1024];
    fn init(&mut self, contracts: &[(B160, Bytes)]) -> Self::State {
        todo!()
    }
    fn check(&self, state: &mut Self::State) -> bool {
        apply(state, Bytes::default(), &self.script_check);
        cast_bool(state, &self.proj_check)
    }
    fn make_mal_call(&self, state: &mut Self::State) -> Option<(B160, U256, Bytes)> {
        let skip = cast_bool(state, &self.proj_skip_call);
        apply(state, Bytes::default(), &self.script_make_call);
        if skip { None }
        else {
            let x = cast_byte(2, state, &self.proj_contract);
            let contract = self.contracts[(x[0] as usize * 256 + x[1] as usize) % self.contracts.len()];
            let x = cast_byte(32, state, &self.proj_value);
            let value = U256::try_from_be_slice(&x).unwrap();
            let mut lstate = state.clone();
            apply(&mut lstate, Bytes::default(), &self.script_proj_data);
            let x = cast_byte(usize::MAX, &lstate, state);
            let input = Bytes::from(x);
            Some((contract, value, input))
        }
    }
    fn take_return(&self, state: &mut Self::State, _ret: InstructionResult, _gas: Gas, out: Bytes) {
        apply(state, out, &self.script_take_return);
    }
}

pub fn cast_byte<const N: usize>(m: usize, a: &[u32; N], b: &[u32; N]) -> Vec<u8> {
    let mut bytes = vec![0u8; m];
    let mut k = 0;
    for i in 0..m*8 {
        if k >= N * 32 { break; }
        if b[k / 32] >> k % 32 & 1 == 0 { k += 1; }
        bytes[i / 8] |= ((a[k / 32] >> k % 32 & 1) as u8) << i % 8;
    }
    return bytes;
}

pub fn cast_bool<const N: usize>(a: &[u32; N], b: &[u32; N]) -> bool {
    let mut c = 0u32;
    for i in 0..N { c |= a[i] & b[i]; }
    c.count_ones() == 0
}

// A state is a byte string
// A script manipulates the byte string
pub fn apply<const N: usize>(state: &mut [u32; N], bytes: Bytes, script: &[u32]) {
    let mut script = script.into_iter();
    let len = state.len();
    assert!(len.count_ones() == 1);
    let mask = 1 << len.trailing_zeros() - 1;
    let get = move |script: &mut std::slice::Iter<'_, u32>| {
        script.next().copied().unwrap_or(0)
    };
    let mut x = get(&mut script);
    let mut y = get(&mut script);
    while let Some(instr) = script.next() {
        match instr % 19 {
            0 => { y = y.overflowing_add(x).0 }
            1 => { y = y.overflowing_sub(x).0 }
            2 => { y = y.overflowing_mul(x).0 }
            3 => { y = y.overflowing_div(x).0 }
            4 => { y = y.overflowing_rem(x).0 }
            6 => { y = y.overflowing_shl(x).0 }
            7 => { y = y.overflowing_shr(x).0 }
            8 => { y = y & x }
            9 => { y = y | x }
            10 => { y = y ^ x }
            11 => { y = !y }
            12 => { std::mem::swap(&mut x, &mut y); }
            13 => { state[x as usize & mask] = y; }
            14 => { y = state[y as usize & mask]; }
            15 => { y = bytes[y as usize % bytes.len()] as u32; }
            16 => { y = bytes[y as usize % bytes.len()] as u32 * 256; }
            17 => { y = bytes[y as usize % bytes.len()] as u32 * 256 * 256; }
            18 => { y = bytes[y as usize % bytes.len()] as u32 * 256 * 256 * 256; }
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use tch::{Kind, Device};

    #[test]
    fn test_tch_works() {
        let a = tch::Tensor::rand([1,5], (Kind::Double, Device::Cuda(0)));
        let b = tch::Tensor::rand([1,5], (Kind::Double, Device::Cuda(0)));
        println!("{} \n+ {} \n= {}", a, b, &a + &b);
    }
}