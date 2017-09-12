use super::super::bus::Bus;
use super::io::{In8, Out8, In16};
use super::operations;
use super::instruction::Instruction;
use super::State;

pub struct Executor<'a, B: Bus + 'a>(pub &'a mut State, pub &'a mut B);

impl<'a, B: Bus> operations::Operations for Executor<'a, B> {
    type Output = ();

    fn nop(&mut self) {
        // No Operation
    }

    fn load8<I: In8, O: Out8>(&mut self, destination: O, source: I) {
        let value = source.read8(self.0, self.1);
        destination.write8(self.0, self.1, value);
    }

    fn jp(&mut self) {
        self.0.pc = self.0.next16(self.1);
    }

    fn undefined(&mut self, opcode: u8) {
        panic!("undefined opcode {:02x}", opcode);
    }
}
