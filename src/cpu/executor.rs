use super::super::bus::Bus;
use super::io::{In8, Out16, Out8};
use super::operations;
use super::operands::{Condition, Register16};
use super::instruction::Instruction;
use super::State;
use super::state::Flags;

pub struct Executor<'a, B: Bus + 'a>(pub &'a mut State, pub &'a mut B);

impl<'a, B: Bus> operations::Operations for Executor<'a, B> {
    type Output = ();

    #[inline]
    fn nop(&mut self) {
        // No Operation
    }

    #[inline]
    fn load8<I: In8, O: Out8>(&mut self, dst: O, src: I) {
        let value = src.read8(self.0, self.1);
        dst.write8(self.0, self.1, value);
    }

    #[inline]
    fn load16_immediate(&mut self, dst: Register16) {
        let value = self.0.next16(self.1);
        dst.write16(self.0, self.1, value);
    }

    #[inline]
    fn jp<C: Condition>(&mut self, cond: C) {
        if cond.check(self.0) {
            self.0.pc = self.0.next16(self.1);
        }
    }

    #[inline]
    fn jr<C: Condition>(&mut self, cond: C) {
        if cond.check(self.0) {
            // Take the _signed_ 8-bit immediate value and extend to 32-bits
            let offset = (self.0.next8(self.1) as i8) as i32;

            // Perform signed addition to offset from PC
            self.0.pc = ((self.0.pc as i32) + offset) as u16;
        }
    }

    #[inline]
    fn call<C: Condition>(&mut self, cond: C) {
        if cond.check(self.0) {
            let address = self.0.next16(self.1);
            let pc = self.0.pc;

            self.0.push16(self.1, pc);

            self.0.pc = address;
        }

        // TODO: On a missed branch there are 2 additional cycles
    }

    #[inline]
    fn ret<C: Condition>(&mut self, cond: C) {
        if cond.check(self.0) {
            let address = self.0.pop16(self.1);

            self.0.pc = address;
        }
    }

    #[inline]
    fn reti(&mut self) {
        self.ret(());
        self.ei();
    }

    #[inline]
    fn and<IO: In8 + Out8>(&mut self, io: IO) {
        let mut value = io.read8(self.0, self.1);
        value &= self.0.a;

        io.write8(self.0, self.1, value);
    }

    #[inline]
    fn or<IO: In8 + Out8>(&mut self, io: IO) {
        let mut value = io.read8(self.0, self.1);
        value |= self.0.a;

        io.write8(self.0, self.1, value);
    }

    #[inline]
    fn xor<IO: In8 + Out8>(&mut self, io: IO) {
        let mut value = io.read8(self.0, self.1);
        value ^= self.0.a;

        io.write8(self.0, self.1, value);
    }

    #[inline]
    fn inc8<IO: In8 + Out8>(&mut self, io: IO) {
        let mut value = io.read8(self.0, self.1);
        value = value.wrapping_add(1);

        // TODO: Set HALF_CARRY
        self.0.f.set(Flags::ZERO, value == 0);
        self.0.f.set(Flags::ADD_SUBTRACT, false);

        io.write8(self.0, self.1, value);
    }

    #[inline]
    fn dec8<IO: In8 + Out8>(&mut self, io: IO) {
        let mut value = io.read8(self.0, self.1);
        value = value.wrapping_sub(1);

        // TODO: Set HALF_CARRY
        self.0.f.set(Flags::ZERO, value == 0);
        self.0.f.set(Flags::ADD_SUBTRACT, true);

        io.write8(self.0, self.1, value);
    }

    #[inline]
    fn ei(&mut self) {
        info!("unimplemented: EI");
    }

    #[inline]
    fn di(&mut self) {
        info!("unimplemented: EI");
    }

    #[inline]
    fn reset(&mut self, address: u8) {
        self.0.pc = address as u16;
    }

    #[inline]
    fn undefined(&mut self, opcode: u8) {
        panic!("undefined opcode {:02x}", opcode);
    }
}
