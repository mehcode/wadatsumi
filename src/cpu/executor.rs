use super::super::bus::Bus;
use super::io::{In16, In8, Out16, Out8};
use super::operations;
use super::operands::{Address, Condition, Register16};
use super::operands::Register8::*;
use super::State;
use super::state::Flags;

pub struct Executor<'a, B: Bus + 'a>(pub &'a mut State, pub &'a mut B);

impl<'a, B: Bus> Executor<'a, B> {
    #[inline]
    fn add16_sp_e(&mut self) -> u16 {
        let sp = Register16::SP.read16(self.0, self.1);
        let value = ((self.0.next8(self.1)) as i8) as i32;
        let result = ((sp as i32) + value) as u16;

        self.0.f.set(Flags::CARRY, (result & 0xFF) < (sp & 0xFF));
        self.0.f.set(Flags::HALF_CARRY, (result & 0xF) < (sp & 0xF));
        self.0.f.set(Flags::ZERO, false);
        self.0.f.set(Flags::ADD_SUBTRACT, false);

        // TODO: extra cycle goes here

        result
    }
}

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

        self.0.f.set(
            Flags::HALF_CARRY,
            ((hl ^ value ^ ((result & 0xFFFF) as u16)) & 0x1000) != 0,
        );
        self.0.f.set(Flags::CARRY, result > 0xFFFF);
        self.0.f.set(Flags::ADD_SUBTRACT, false);

        Register16::HL.write16(self.0, self.1, (result & 0xFFFF) as u16);
        // TODO: Extra cycle goes here
    }

    #[inline]
    fn add16_sp_e(&mut self) {
        let result = self.add16_sp_e();

        Register16::SP.write16(self.0, self.1, result);
    }

    #[inline]
    fn load16_hl_sp_e(&mut self) {
        let result = self.add16_sp_e();

        Register16::HL.write16(self.0, self.1, result);
    }

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

    #[inline]
    fn adc8<I: In8>(&mut self, src: I) {
        let a = self.0.a as u16;
        let value = src.read8(self.0, self.1) as u16;
        let carry = self.0.f.contains(Flags::CARRY) as u16;
        let result = a + value + carry;

        self.0.f.set(Flags::ZERO, (result & 0xff) == 0);
        self.0.f.set(Flags::ADD_SUBTRACT, false);
        self.0.f.set(Flags::CARRY, result > 0xFF);
        self.0.f.set(
            Flags::HALF_CARRY,
            ((a & 0x0F) + (value & 0x0F) + carry) > 0x0F,
        );

        self.0.a = result as u8;
    }

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

    #[inline]
    fn sbc<I: In8>(&mut self, src: I) {
        let a = self.0.a as i16;
        let value = src.read8(self.0, self.1) as i16;
        let carry = self.0.f.contains(Flags::CARRY) as i16;
        let result = a- value- carry;

        self.0.f.set(Flags::CARRY, result < 0);
        self.0.f.set(Flags::ZERO, (result & 0xFF) == 0);
        self.0.f.set(Flags::ADD_SUBTRACT, true);
        self.0.f.set(Flags::HALF_CARRY, ((((a as i16) & 0x0F) - ((value as i16) & 0x0F) - (carry as i16)) < 0));

        self.0.a = (result & 0xFF) as u8;
    }

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
    fn cpl(&mut self) {
        self.0.a ^= 0xff;
        self.0.f.set(Flags::ADD_SUBTRACT, true);
        self.0.f.set(Flags::HALF_CARRY, true);
    }

    #[inline]
    fn ccf(&mut self) {
        self.0.f.set(Flags::ADD_SUBTRACT, false);
        self.0.f.set(Flags::HALF_CARRY, false);
        self.0.f.toggle(Flags::CARRY);
    }

    #[inline]
    fn scf(&mut self) {
        self.0.f.set(Flags::ADD_SUBTRACT, false);
        self.0.f.set(Flags::HALF_CARRY, false);
        self.0.f.set(Flags::CARRY, true);
    }

    fn daa(&mut self) {
        // When this instruction is executed, the A register is BCD corrected
        // using the contents of the flags. The exact process is the following:
        // if the least significant four bits of A contain a non-BCD digit (i. e.
        // it is greater than 9) or the H flag is set, then $06 is added to the
        // register. Then the four most significant bits are checked. If this
        // more significant digit also happens to be greater than 9 or the C
        // flag is set, then $60 is added.
        //
        // If the N flag is set, subtract instead of add.
        //
        // If the lower 4 bits form a number greater than 9 or H is set,
        // add $06 to the accumulator

        let mut r = self.0.a as u16;
        let mut correction = if self.0.f.contains(Flags::CARRY) {
            0x60u16
        } else {
            0x00u16
        };

        if self.0.f.contains(Flags::HALF_CARRY) || ((!self.0.f.contains(Flags::ADD_SUBTRACT)) && ((r & 0x0F) > 9)) {
            correction |= 0x06;
        }

        if self.0.f.contains(Flags::CARRY) || ((!self.0.f.contains(Flags::ADD_SUBTRACT)) && (r > 0x99)) {
            correction |= 0x60;
        }

        if self.0.f.contains(Flags::ADD_SUBTRACT) {
            r = r.wrapping_sub(correction);
        } else {
            r = r.wrapping_add(correction);
        }

        if ((correction << 2) & 0x100) != 0 {
            self.0.f.set(Flags::CARRY, true);
        }

        // Half-carry is always unset (unlike a Z-80)
        self.0.f.set(Flags::HALF_CARRY, false);
        self.0.f.set(Flags::ZERO, (r & 0xFF) == 0);

        self.0.a = (r & 0xFF) as u8;
    }

    #[inline]
    fn undefined(&mut self, opcode: u8) {
        panic!("undefined opcode {:02x}", opcode);
    }
}
