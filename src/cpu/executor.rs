use super::super::bus::Bus;
use super::io::{In16, In8, Out16, Out8};
use super::operations;
use super::operands::{Condition, Address, Register16};
use super::operands::Register8::*;
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
    fn load16<I: In16, O: Out16>(&mut self, dst: O, src: I) {
        let value = src.read16(self.0, self.1);
        dst.write16(self.0, self.1, value);
    }

    #[inline]
    fn jp<C: Condition>(&mut self, cond: C, address: Address) {
        if cond.check(self.0) {
            let address = self.0.indirect(self.1, address);

            self.0.pc = address;
        } else if address == Address::Direct {
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

    #[inline]
    fn add16_hl(&mut self, r: Register16) {
        let hl = Register16::HL.read16(self.0, self.1);
        let value = r.read16(self.0, self.1);
        let result = hl as u32 + value as u32;

        self.0.f.set(Flags::HALF_CARRY, ((hl ^ value ^ ((result & 0xFFFF) as u16)) & 0x1000) != 0);
        self.0.f.set(Flags::CARRY, result > 0xFFFF);
        self.0.f.set(Flags::ADD_SUBTRACT, false);

        Register16::HL.write16(self.0, self.1, (result & 0xFFFF) as u16);
        // TODO: Extra cycle goes here
    }

    // ADD _
    // A = A + _
    #[inline]
    fn add8<I: In8>(&mut self, src: I) {
        let a = self.0.a as u16;
        let value = src.read8(self.0, self.1) as u16;
        let result = a + value;

        self.0.f.set(Flags::ZERO, (result & 0xff) == 0);
        self.0.f.set(Flags::ADD_SUBTRACT, false);
        self.0.f.set(Flags::CARRY, result > 0xFF);
        self.0
            .f
            .set(Flags::HALF_CARRY, ((a & 0x0F) + (value & 0x0F)) > 0x0F);

        self.0.a = result as u8;
    }

    // ADC _
    // A = A + _ + CARRY
    #[inline]
    fn adc8<I: In8>(&mut self, src: I) {
        let a = self.0.a as u16;
        let value = src.read8(self.0, self.1) as u16;
        let carry = self.0.f.contains(Flags::CARRY) as u16;
        let result = a + value + carry;

        self.0.f.set(Flags::ZERO, (result & 0xff) == 0);
        self.0.f.set(Flags::ADD_SUBTRACT, false);
        self.0.f.set(Flags::CARRY, result > 0xFF);
        self.0
            .f
            .set(Flags::HALF_CARRY,((a & 0x0F) + (value & 0x0F) + carry) > 0x0F);

        self.0.a = result as u8;
    }

    // SUB _
    // A = A - _
    #[inline]
    fn sub<I: In8>(&mut self, src: I) {
        // FIXME: Duplicate code with `compare`

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

        self.0.a = result as u8;
    }

    // CP _
    #[inline]
    fn cp<I: In8>(&mut self, src: I) {
        // FIXME: Duplicate code with `sub`

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
    fn inc16(&mut self, r: Register16) {
        let value = r.read16(self.0, self.1).wrapping_add(1);

        r.write16(self.0, self.1, value);
    }

    #[inline]
    fn dec16(&mut self, r: Register16) {
        let value = r.read16(self.0, self.1).wrapping_sub(1);

        r.write16(self.0, self.1, value);
    }

    #[inline]
    fn push16(&mut self, r: Register16) {
        let value = r.read16(self.0, self.1);
        self.0.push16(self.1, value);
    }

    #[inline]
    fn pop16(&mut self, r: Register16) {
        let value = self.0.pop16(self.1);
        r.write16(self.0, self.1, value);
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
    fn rla(&mut self) {
        // `RLA` is exactly `RL A` with the ZERO flag always reset
        self.rl(A);
        self.0.f.set(Flags::ZERO, false);
    }

    #[inline]
    fn rlca(&mut self) {
        // `RLCA` is exactly `RLC A` with the ZERO flag always reset
        self.rlc(A);
        self.0.f.set(Flags::ZERO, false);
    }

    #[inline]
    fn rra(&mut self) {
        // `RRA` is exactly `RR A` with the ZERO flag always reset
        self.rr(A);
        self.0.f.set(Flags::ZERO, false);
    }

    #[inline]
    fn rrca(&mut self) {
        // `RRCA` is exactly `RRC A` with the ZERO flag always reset
        self.rrc(A);
        self.0.f.set(Flags::ZERO, false);
    }

    #[inline]
    fn rl<IO: In8 + Out8>(&mut self, io: IO) {
        let value = io.read8(self.0, self.1);
        let result = (value << 1) | (self.0.f.contains(Flags::CARRY) as u8);

        self.0.f.set(Flags::ZERO, result == 0);
        self.0.f.set(Flags::ADD_SUBTRACT, false);
        self.0.f.set(Flags::HALF_CARRY, false);
        self.0.f.set(Flags::CARRY, ((value & 0x80) != 0));

        io.write8(self.0, self.1, result);
    }

    #[inline]
    fn rlc<IO: In8 + Out8>(&mut self, io: IO) {
        let value = io.read8(self.0, self.1);
        let result = (value << 1) | (value >> 7);

        self.0.f.set(Flags::ZERO, result == 0);
        self.0.f.set(Flags::ADD_SUBTRACT, false);
        self.0.f.set(Flags::HALF_CARRY, false);
        self.0.f.set(Flags::CARRY, ((value & 0x80) != 0));

        io.write8(self.0, self.1, result);
    }

    #[inline]
    fn rr<IO: In8 + Out8>(&mut self, io: IO) {
        let value = io.read8(self.0, self.1);
        let result = (value >> 1) | ((self.0.f.contains(Flags::CARRY) as u8) << 7);

        self.0.f.set(Flags::ZERO, result == 0);
        self.0.f.set(Flags::ADD_SUBTRACT, false);
        self.0.f.set(Flags::HALF_CARRY, false);
        self.0.f.set(Flags::CARRY, ((value & 0x01) != 0));

        io.write8(self.0, self.1, result);
    }

    #[inline]
    fn rrc<IO: In8 + Out8>(&mut self, io: IO) {
        let value = io.read8(self.0, self.1);
        let result = (value >> 1) | (value << 7);

        self.0.f.set(Flags::ZERO, result == 0);
        self.0.f.set(Flags::ADD_SUBTRACT, false);
        self.0.f.set(Flags::HALF_CARRY, false);
        self.0.f.set(Flags::CARRY, ((value & 0x01) != 0));

        io.write8(self.0, self.1, result);
    }

    #[inline]
    fn swap<IO: In8 + Out8>(&mut self, io: IO) {
        let value = io.read8(self.0, self.1);
        let result = (value >> 4) | ((value << 4) & 0xF0);

        self.0.f.set(Flags::ZERO, result == 0);
        self.0.f.set(Flags::ADD_SUBTRACT, false);
        self.0.f.set(Flags::HALF_CARRY, false);
        self.0.f.set(Flags::CARRY, false);

        io.write8(self.0, self.1, result);
    }

    #[inline]
    fn sla<IO: In8 + Out8>(&mut self, io: IO) {
        let value = io.read8(self.0, self.1);
        let result = value << 1;

        self.0.f.set(Flags::ZERO, result == 0);
        self.0.f.set(Flags::ADD_SUBTRACT, false);
        self.0.f.set(Flags::HALF_CARRY, false);
        self.0.f.set(Flags::CARRY, (value & 0x80) != 0);

        io.write8(self.0, self.1, result);
    }

    #[inline]
    fn sra<IO: In8 + Out8>(&mut self, io: IO) {
        let value = io.read8(self.0, self.1);
        let result = if (value & 0x80) != 0 {
            (value >> 1) | 0x80
        } else {
            (value >> 1)
        };

        self.0.f.set(Flags::ZERO, result == 0);
        self.0.f.set(Flags::ADD_SUBTRACT, false);
        self.0.f.set(Flags::HALF_CARRY, false);
        self.0.f.set(Flags::CARRY, (value & 0x01) != 0);

        io.write8(self.0, self.1, result);
    }

    #[inline]
    fn srl<IO: In8 + Out8>(&mut self, io: IO) {
        let value = io.read8(self.0, self.1);
        let result = value >> 1;

        self.0.f.set(Flags::ZERO, result == 0);
        self.0.f.set(Flags::ADD_SUBTRACT, false);
        self.0.f.set(Flags::HALF_CARRY, false);
        self.0.f.set(Flags::CARRY, (value & 0x01) != 0);

        io.write8(self.0, self.1, result);
    }

    #[inline]
    fn bit<I: In8>(&mut self, bit: u8, src: I) {
        let value = src.read8(self.0, self.1);

        self.0.f.set(Flags::ZERO, (value & (1 << bit)) == 0);
        self.0.f.set(Flags::ADD_SUBTRACT, false);
        self.0.f.set(Flags::HALF_CARRY, true);
    }

    #[inline]
    fn set<IO: In8 + Out8>(&mut self, bit: u8, io: IO) {
        let mut value = io.read8(self.0, self.1);
        value |= 1 << bit;

        io.write8(self.0, self.1, value);
    }

    #[inline]
    fn res<IO: In8 + Out8>(&mut self, bit: u8, io: IO) {
        let mut value = io.read8(self.0, self.1);
        value &= !(1 << bit);

        io.write8(self.0, self.1, value);
    }

    #[inline]
    fn rst(&mut self, address: u8) {
        let pc = self.0.pc;
        self.0.push16(self.1, pc);

        self.0.pc = address as u16;
    }

    #[inline]
    fn undefined(&mut self, opcode: u8) {
        panic!("undefined opcode {:02x}", opcode);
    }
}
