use super::super::bus::Bus;
use super::io::{In8, Out16, Out8};
use super::operations;
use super::operands::{Condition, Register16};
use super::operands::Register8::*;
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
        } else {
            self.0.pc = self.0.pc.wrapping_add(2);
        }
    }

    #[inline]
    fn jr<C: Condition>(&mut self, cond: C) {
        if cond.check(self.0) {
            // Take the _signed_ 8-bit immediate value and extend to 32-bits
            let offset = (self.0.next8(self.1) as i8) as i32;

            // Perform signed addition to offset from PC
            self.0.pc = ((self.0.pc as i32) + offset) as u16;
        } else {
            self.0.pc = self.0.pc.wrapping_add(1);
        }
    }

    #[inline]
    fn call<C: Condition>(&mut self, cond: C) {
        if cond.check(self.0) {
            let address = self.0.next16(self.1);
            let pc = self.0.pc;

            self.0.push16(self.1, pc);

            self.0.pc = address;
        } else {
            self.0.pc = self.0.pc.wrapping_add(2);
        }
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

    // ADD _
    // A = A + _
    #[inline]
    fn add<I: In8>(&mut self, src: I) {
        let a = self.0.a as u16;
        let value = src.read8(self.0, self.1) as u16;
        let result = a + value;

        self.0.f.set(Flags::ZERO, result == 0);
        self.0.f.set(Flags::ADD_SUBTRACT, false);
        self.0
            .f
            .set(Flags::HALF_CARRY, ((a & 0x0F) + (value & 0x0F)) > 0x0F);
        self.0.f.set(Flags::CARRY, result > 0xFF);

        self.0.a = result as u8;
    }

    // CP _
    #[inline]
    fn compare<I: In8>(&mut self, src: I) {
        let a = self.0.a as i16;
        let value = src.read8(self.0, self.1) as i16;
        let result = a - value;

        self.0.f.set(Flags::CARRY, result < 0);
        self.0.f.set(Flags::ZERO, (result & 0xFF) == 0);
        self.0.f.set(Flags::ADD_SUBTRACT, true);
        self.0.f.set(
            Flags::HALF_CARRY,
            ((((a as i16) & 0x0F) - ((value as i16) & 0x0F)) < 0),
        );
    }

    // AND _
    // A = A & _
    #[inline]
    fn and<I: In8>(&mut self, src: I) {
        let value = src.read8(self.0, self.1);
        let result = self.0.a & value;

        self.0.f.set(Flags::ZERO, result == 0);
        self.0.f.set(Flags::ADD_SUBTRACT, false);
        self.0.f.set(Flags::HALF_CARRY, true);
        self.0.f.set(Flags::CARRY, false);

        self.0.a = result;
    }

    // OR _
    // A = A | _
    #[inline]
    fn or<I: In8>(&mut self, src: I) {
        let value = src.read8(self.0, self.1);
        let result = self.0.a | value;

        self.0.f.set(Flags::ZERO, result == 0);
        self.0.f.set(Flags::ADD_SUBTRACT, false);
        self.0.f.set(Flags::HALF_CARRY, false);
        self.0.f.set(Flags::CARRY, false);

        self.0.a = result;
    }

    // XOR _
    // A = A ^ _
    #[inline]
    fn xor<I: In8>(&mut self, src: I) {
        let value = src.read8(self.0, self.1);
        let result = self.0.a ^ value;

        self.0.f.set(Flags::ZERO, result == 0);
        self.0.f.set(Flags::ADD_SUBTRACT, false);
        self.0.f.set(Flags::HALF_CARRY, false);
        self.0.f.set(Flags::CARRY, false);

        self.0.a = result;
    }

    #[inline]
    fn inc8<IO: In8 + Out8>(&mut self, io: IO) {
        let value = io.read8(self.0, self.1).wrapping_add(1);

        self.0.f.set(Flags::ZERO, value == 0);
        self.0.f.set(Flags::ADD_SUBTRACT, false);
        self.0.f.set(Flags::HALF_CARRY, value & 0x0F == 0x00);

        io.write8(self.0, self.1, value);
    }

    #[inline]
    fn dec8<IO: In8 + Out8>(&mut self, io: IO) {
        let value = io.read8(self.0, self.1).wrapping_sub(1);

        self.0.f.set(Flags::ZERO, value == 0);
        self.0.f.set(Flags::ADD_SUBTRACT, true);
        self.0.f.set(Flags::HALF_CARRY, value & 0x0F == 0x0F);

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
