// a bit manipulation language for the defender
pub enum Instruction {
    And(usize, usize),
    Not(usize),
    Swp(usize, usize),
}

pub fn evaluate(mut input: Vec<bool>, instr: Vec<Instruction>) -> Vec<bool> {
    use Instruction::*;
    for i in instr { match i {
        And(x, y) => input[x] &= input[y],
        Not(x) => input[x] = !input[x],
        Swp(x, y) => {let t = input[x]; input[x] = input[y]; input[y] = t},
    }}
    return input;
}