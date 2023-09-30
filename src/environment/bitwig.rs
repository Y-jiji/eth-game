use bitvec::slice::*;

// a bit manipulation language for the defender
pub enum Instruction {
    And(usize, usize),
    Not(usize),
    Load(usize, usize),
    Swap(usize, usize),
}

pub fn evaluate(input: &BitSlice<u8>, mut state: Vec<bool>, instr: &[Instruction]) -> Vec<bool> {
    use Instruction::*;
    assert!(state.len().count_ones() == 1);
    let n = (1 << state.len().trailing_zeros() + 1) - 1;
    for i in instr { match i {
        And(x, y) => state[x & n] &= state[y & n],
        Not(x) => state[x & n] = !state[x & n],
        Load(x, y) => state[x & n] = if *y < input.len() { input[*y] } else { false },
        Swap(x, y) => {
            let t = state[x & n];
            state[x & n] = state[y & n]; 
            state[y & n] = t
        },
    }}
    return state;
}