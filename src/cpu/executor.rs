use super::super::bus::Bus;
use super::io::{In8, Out16, Out8};
use super::operations;
use super::registers::Register16;
use super::instruction::Instruction;
use super::State;

pub struct Executor<'a, B: Bus + 'a>(pub &'a mut State, pub &'a mut B);

impl<'a, B: Bus> operations::Operations for Executor<'a, B> {
    type Output = ();

    fn nop(&mut self) {
        // No Operation
    }

    fn load8<I: In8, O: Out8>(&mut self, dst: O, src: I) {
        let value = src.read8(self.0, self.1);
        dst.write8(self.0, self.1, value);
    }

    fn load8_immediate<O: Out8>(&mut self, dst: O) {
        let value = self.0.next8(self.1);
        dst.write8(self.0, self.1, value);
    }

    fn load16_immediate(&mut self, register: Register16) {
        let value = self.0.next16(self.1);
        register.write16(self.0, self.1, value);
    }

    fn jp(&mut self) {
        self.0.pc = self.0.next16(self.1);
    }

    fn undefined(&mut self, opcode: u8) {
        panic!("undefined opcode {:02x}", opcode);
    }
}
