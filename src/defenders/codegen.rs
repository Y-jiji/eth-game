use revm::interpreter::*;

pub struct DefenderCodeGen {
}

impl<DB: revm::Database> revm::Inspector<DB> for DefenderCodeGen {
    fn step(
        &mut self,
        _interp: &mut Interpreter,
        _data: &mut revm::EVMData<'_, DB>,
    ) -> InstructionResult {
        todo!()
    }
    fn step_end(
        &mut self,
        _interp: &mut Interpreter,
        _data: &mut revm::EVMData<'_, DB>,
        _eval: InstructionResult,
    ) -> InstructionResult {
        todo!()
    }
}