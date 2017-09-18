use super::State;
use super::io::{In16, In8, Out16, Out8};
use super::operands::{self, condition, Address, Condition, Immediate16, Immediate8, Register16,
                      Register8};

/// Defines a visitor for a CPU (micro) operation.
pub trait Operations {
    type Output;

    /// No operation
    fn nop(&mut self) -> Self::Output;

    /// 8-bit load
    fn load8<I: In8, O: Out8>(&mut self, O, I) -> Self::Output;

    /// 16-bit immediate load
    fn load16_immediate(&mut self, Register16) -> Self::Output;

    /// Relative jump
    fn jr<C: Condition>(&mut self, C) -> Self::Output;

    /// Absolute jump
    fn jp<C: Condition>(&mut self, C, Address) -> Self::Output;

    /// Call (subroutine)
    fn call<C: Condition>(&mut self, C) -> Self::Output;

    /// Return (from subroutine)
    fn ret<C: Condition>(&mut self, C) -> Self::Output;

    /// Return (from subroutine) and enable interrupts
    fn reti(&mut self) -> Self::Output;

    /// Addition
    fn add<I: In8>(&mut self, I) -> Self::Output;

    /// Addition (with carry)
    fn adc<I: In8>(&mut self, I) -> Self::Output;

    /// Subtraction
    fn sub<I: In8>(&mut self, I) -> Self::Output;

    /// Compare
    fn cp<I: In8>(&mut self, I) -> Self::Output;

    /// Bitwise AND
    fn and<I: In8>(&mut self, I) -> Self::Output;

    /// Bitwise OR
    fn or<I: In8>(&mut self, I) -> Self::Output;

    /// Bitwise XOR
    fn xor<I: In8>(&mut self, I) -> Self::Output;

    /// 8-bit increment
    fn inc8<IO: In8 + Out8>(&mut self, IO) -> Self::Output;

    /// 8-bit decrement
    fn dec8<IO: In8 + Out8>(&mut self, IO) -> Self::Output;

    /// 16-bit increment
    fn inc16(&mut self, Register16) -> Self::Output;

    /// 16-bit decrement
    fn dec16(&mut self, Register16) -> Self::Output;

    /// 16-bit push
    fn push16(&mut self, Register16) -> Self::Output;

    /// 16-bit pop
    fn pop16(&mut self, Register16) -> Self::Output;

    /// Enable interrupts
    fn ei(&mut self) -> Self::Output;

    /// Disable dnterrupts
    fn di(&mut self) -> Self::Output;

    /// Rotate accumulator (A) left (through carry)
    fn rla(&mut self) -> Self::Output;

    /// Rotate accumulator (A) left
    fn rlca(&mut self) -> Self::Output;

    /// Rotate accumulator (A) right (through carry)
    fn rra(&mut self) -> Self::Output;

    /// Rotate accumulator (A) right
    fn rrca(&mut self) -> Self::Output;

    /// Rotate left (through carry)
    fn rl<IO: In8 + Out8>(&mut self, IO) -> Self::Output;

    /// Rotate left
    fn rlc<IO: In8 + Out8>(&mut self, IO) -> Self::Output;

    /// Rotate right (through carry)
    fn rr<IO: In8 + Out8>(&mut self, IO) -> Self::Output;

    /// Rotate right
    fn rrc<IO: In8 + Out8>(&mut self, IO) -> Self::Output;

    /// Swap, exchange low/hi-nibble
    fn swap<IO: In8 + Out8>(&mut self, IO) -> Self::Output;

    /// Shift left (arithmetic)
    fn sla<IO: In8 + Out8>(&mut self, IO) -> Self::Output;

    /// Shift right (arithmetic)
    fn sra<IO: In8 + Out8>(&mut self, IO) -> Self::Output;

    /// Shift right (logical)
    fn srl<IO: In8 + Out8>(&mut self, IO) -> Self::Output;

    /// Bit test
    fn bit<I: In8>(&mut self, u8, I) -> Self::Output;

    /// Bit set
    fn set<IO: In8 + Out8>(&mut self, u8, IO) -> Self::Output;

    /// Bit reset
    fn res<IO: In8 + Out8>(&mut self, u8, IO) -> Self::Output;

    /// Reset
    fn rst(&mut self, address: u8) -> Self::Output;

    /// Undefined operation
    fn undefined(&mut self, opcode: u8) -> Self::Output;
}

#[inline]
pub fn visit<O: Operations>(mut ops: O, opcode: u8) -> O::Output {
    use self::Register8::*;
    use self::Register16::*;

    match opcode {
        // 8-bit Loads ----------------------------------------------------------------------------
        // LD A, _
        0x0a => ops.load8(A, Address::BC),
        0x1a => ops.load8(A, Address::DE),
        0x2a => ops.load8(A, Address::HLI),
        0x3a => ops.load8(A, Address::HLD),
        0x3e => ops.load8(A, Immediate8),
        0x78 => ops.load8(A, B),
        0x79 => ops.load8(A, C),
        0x7a => ops.load8(A, D),
        0x7b => ops.load8(A, E),
        0x7c => ops.load8(A, H),
        0x7d => ops.load8(A, L),
        0x7e => ops.load8(A, Address::HL),
        0x7f => ops.load8(A, A),
        0xe0 => ops.load8(Address::ZeroPage, A),
        0xf0 => ops.load8(A, Address::ZeroPage),
        0xe2 => ops.load8(Address::ZeroPageC, A),
        0xf2 => ops.load8(A, Address::ZeroPageC),
        0xea => ops.load8(Address::Direct, A),
        0xfa => ops.load8(A, Address::Direct),

        // LD B, _
        0x06 => ops.load8(B, Immediate8),
        0x40 => ops.load8(B, B),
        0x41 => ops.load8(B, C),
        0x42 => ops.load8(B, D),
        0x43 => ops.load8(B, E),
        0x44 => ops.load8(B, H),
        0x45 => ops.load8(B, L),
        0x46 => ops.load8(B, Address::HL),
        0x47 => ops.load8(B, A),

        // LD C, _
        0x0e => ops.load8(C, Immediate8),
        0x48 => ops.load8(C, B),
        0x49 => ops.load8(C, C),
        0x4a => ops.load8(C, D),
        0x4b => ops.load8(C, E),
        0x4c => ops.load8(C, H),
        0x4d => ops.load8(C, L),
        0x4e => ops.load8(C, Address::HL),
        0x4f => ops.load8(C, A),

        // LD D, _
        0x16 => ops.load8(D, Immediate8),
        0x50 => ops.load8(D, B),
        0x51 => ops.load8(D, C),
        0x52 => ops.load8(D, D),
        0x53 => ops.load8(D, E),
        0x54 => ops.load8(D, H),
        0x55 => ops.load8(D, L),
        0x56 => ops.load8(D, Address::HL),
        0x57 => ops.load8(D, A),

        // LD E, _
        0x1e => ops.load8(E, Immediate8),
        0x58 => ops.load8(E, B),
        0x59 => ops.load8(E, C),
        0x5a => ops.load8(E, D),
        0x5b => ops.load8(E, E),
        0x5c => ops.load8(E, H),
        0x5d => ops.load8(E, L),
        0x5e => ops.load8(E, Address::HL),
        0x5f => ops.load8(E, A),

        // LD H, _
        0x26 => ops.load8(H, Immediate8),
        0x60 => ops.load8(H, B),
        0x61 => ops.load8(H, C),
        0x62 => ops.load8(H, D),
        0x63 => ops.load8(H, E),
        0x64 => ops.load8(H, H),
        0x65 => ops.load8(H, L),
        0x66 => ops.load8(H, Address::HL),
        0x67 => ops.load8(H, A),

        // LD L, _
        0x2e => ops.load8(L, Immediate8),
        0x68 => ops.load8(L, B),
        0x69 => ops.load8(L, C),
        0x6a => ops.load8(L, D),
        0x6b => ops.load8(L, E),
        0x6c => ops.load8(L, H),
        0x6d => ops.load8(L, L),
        0x6e => ops.load8(L, Address::HL),
        0x6f => ops.load8(L, A),

        // LD (HL), _
        0x36 => ops.load8(Address::HL, Immediate8),
        0x70 => ops.load8(Address::HL, B),
        0x71 => ops.load8(Address::HL, C),
        0x72 => ops.load8(Address::HL, D),
        0x73 => ops.load8(Address::HL, E),
        0x74 => ops.load8(Address::HL, H),
        0x75 => ops.load8(Address::HL, L),
        0x77 => ops.load8(Address::HL, A),

        // LD (r16), _
        0x02 => ops.load8(Address::BC, A),
        0x12 => ops.load8(Address::DE, A),
        0x22 => ops.load8(Address::HLI, A),
        0x32 => ops.load8(Address::HLD, A),

        // 16-bit Immediate Loads -----------------------------------------------------------------
        0x01 => ops.load16_immediate(BC),
        0x11 => ops.load16_immediate(DE),
        0x21 => ops.load16_immediate(HL),
        0x31 => ops.load16_immediate(SP),

        // Relative Jumps -------------------------------------------------------------------------
        0x18 => ops.jr(()),
        0x20 => ops.jr(condition::NOT_ZERO),
        0x28 => ops.jr(condition::ZERO),
        0x30 => ops.jr(condition::NOT_CARRY),
        0x38 => ops.jr(condition::CARRY),

        // Absolute Jumps -------------------------------------------------------------------------
        0xc3 => ops.jp((), Address::Direct),
        0xc2 => ops.jp(condition::NOT_ZERO, Address::Direct),
        0xca => ops.jp(condition::ZERO, Address::Direct),
        0xd2 => ops.jp(condition::NOT_CARRY, Address::Direct),
        0xda => ops.jp(condition::CARRY, Address::Direct),
        0xe9 => ops.jp((), Address::HL),

        // Calls ----------------------------------------------------------------------------------
        0xcd => ops.call(()),
        0xc4 => ops.call(condition::NOT_ZERO),
        0xcc => ops.call(condition::ZERO),
        0xd4 => ops.call(condition::NOT_CARRY),
        0xdc => ops.call(condition::CARRY),

        // Returns --------------------------------------------------------------------------------
        0xc9 => ops.ret(()),
        0xc0 => ops.ret(condition::NOT_ZERO),
        0xc8 => ops.ret(condition::ZERO),
        0xd0 => ops.ret(condition::NOT_CARRY),
        0xd8 => ops.ret(condition::CARRY),
        0xd9 => ops.reti(),

        // 8-bit increment ------------------------------------------------------------------------
        0x04 => ops.inc8(B),
        0x0c => ops.inc8(C),
        0x14 => ops.inc8(D),
        0x1c => ops.inc8(E),
        0x24 => ops.inc8(H),
        0x2c => ops.inc8(L),
        0x34 => ops.inc8(Address::HL),
        0x3c => ops.inc8(A),

        // 8-bit decrement ------------------------------------------------------------------------
        0x05 => ops.dec8(B),
        0x0d => ops.dec8(C),
        0x15 => ops.dec8(D),
        0x1d => ops.dec8(E),
        0x25 => ops.dec8(H),
        0x2d => ops.dec8(L),
        0x35 => ops.dec8(Address::HL),
        0x3d => ops.dec8(A),

        // 16-bit increment -----------------------------------------------------------------------
        0x03 => ops.inc16(BC),
        0x13 => ops.inc16(DE),
        0x23 => ops.inc16(HL),
        0x33 => ops.inc16(SP),

        // 16-bit decrement 0----------------------------------------------------------------------
        0x0b => ops.dec16(BC),
        0x1b => ops.dec16(DE),
        0x2b => ops.dec16(HL),
        0x3b => ops.dec16(SP),

        // 16-bit push ----------------------------------------------------------------------------
        0xc5 => ops.push16(BC),
        0xd5 => ops.push16(DE),
        0xe5 => ops.push16(HL),
        0xf5 => ops.push16(AF),

        // 16-bit pop -----------------------------------------------------------------------------
        0xc1 => ops.pop16(BC),
        0xd1 => ops.pop16(DE),
        0xe1 => ops.pop16(HL),
        0xf1 => ops.pop16(AF),

        // Rotate accumulator ---------------------------------------------------------------------
        0x07 => ops.rlca(),
        0x0f => ops.rrca(),
        0x17 => ops.rla(),
        0x1f => ops.rra(),

        // Addition -------------------------------------------------------------------------------
        0xc6 => ops.add(Immediate8),
        0x80 => ops.add(B),
        0x81 => ops.add(C),
        0x82 => ops.add(D),
        0x83 => ops.add(E),
        0x84 => ops.add(H),
        0x85 => ops.add(L),
        0x86 => ops.add(Address::HL),
        0x87 => ops.add(A),

        // Addition (with carry) ------------------------------------------------------------------
        0xce => ops.adc(Immediate8),
        0x88 => ops.adc(B),
        0x89 => ops.adc(C),
        0x8a => ops.adc(D),
        0x8b => ops.adc(E),
        0x8c => ops.adc(H),
        0x8d => ops.adc(L),
        0x8e => ops.adc(Address::HL),
        0x8f => ops.adc(A),

        // Subtraction ----------------------------------------------------------------------------
        0xd6 => ops.sub(Immediate8),
        0x90 => ops.sub(B),
        0x91 => ops.sub(C),
        0x92 => ops.sub(D),
        0x93 => ops.sub(E),
        0x94 => ops.sub(H),
        0x95 => ops.sub(L),
        0x96 => ops.sub(Address::HL),
        0x97 => ops.sub(A),

        // Compare --------------------------------------------------------------------------------
        0xfe => ops.cp(Immediate8),
        0xb8 => ops.cp(B),
        0xb9 => ops.cp(C),
        0xba => ops.cp(D),
        0xbb => ops.cp(E),
        0xbc => ops.cp(H),
        0xbd => ops.cp(L),
        0xbe => ops.cp(Address::HL),
        0xbf => ops.cp(A),

        // Bitwise AND ----------------------------------------------------------------------------
        0xe6 => ops.and(Immediate8),
        0xa0 => ops.and(B),
        0xa1 => ops.and(C),
        0xa2 => ops.and(D),
        0xa3 => ops.and(E),
        0xa4 => ops.and(H),
        0xa5 => ops.and(L),
        0xa6 => ops.and(Address::HL),
        0xa7 => ops.and(A),

        // Bitwise XOR ----------------------------------------------------------------------------
        0xee => ops.xor(Immediate8),
        0xa8 => ops.xor(B),
        0xa9 => ops.xor(C),
        0xaa => ops.xor(D),
        0xab => ops.xor(E),
        0xac => ops.xor(H),
        0xad => ops.xor(L),
        0xae => ops.xor(Address::HL),
        0xaf => ops.xor(A),

        // Bitwise OR -----------------------------------------------------------------------------
        0xf6 => ops.or(Immediate8),
        0xb0 => ops.or(B),
        0xb1 => ops.or(C),
        0xb2 => ops.or(D),
        0xb3 => ops.or(E),
        0xb4 => ops.or(H),
        0xb5 => ops.or(L),
        0xb6 => ops.or(Address::HL),
        0xb7 => ops.or(A),

        // Reset ----------------------------------------------------------------------------------
        0xc7 => ops.rst(0x00),
        0xcf => ops.rst(0x08),
        0xd7 => ops.rst(0x10),
        0xdf => ops.rst(0x18),
        0xe7 => ops.rst(0x20),
        0xef => ops.rst(0x28),
        0xf7 => ops.rst(0x30),
        0xff => ops.rst(0x38),

        // Miscellaneous --------------------------------------------------------------------------
        0x00 => ops.nop(),
        0xf3 => ops.di(),
        0xfb => ops.ei(),
        _ => ops.undefined(opcode),
    }
}

#[inline]
pub fn visit_cb<O: Operations>(mut ops: O, opcode: u8) -> O::Output {
    use self::Register8::*;
    use self::Register16::*;

    match opcode {
        // Rotate left ----------------------------------------------------------------------------
        0x00 => ops.rlc(B),
        0x01 => ops.rlc(C),
        0x02 => ops.rlc(D),
        0x03 => ops.rlc(E),
        0x04 => ops.rlc(H),
        0x05 => ops.rlc(L),
        0x06 => ops.rlc(Address::HL),
        0x07 => ops.rlc(A),

        // Rotate left through carry --------------------------------------------------------------
        0x10 => ops.rl(B),
        0x11 => ops.rl(C),
        0x12 => ops.rl(D),
        0x13 => ops.rl(E),
        0x14 => ops.rl(H),
        0x15 => ops.rl(L),
        0x16 => ops.rl(Address::HL),
        0x17 => ops.rl(A),

        // Rotate left ----------------------------------------------------------------------------
        0x08 => ops.rrc(B),
        0x09 => ops.rrc(C),
        0x0a => ops.rrc(D),
        0x0b => ops.rrc(E),
        0x0c => ops.rrc(H),
        0x0d => ops.rrc(L),
        0x0e => ops.rrc(Address::HL),
        0x0f => ops.rrc(A),

        // Rotate left through carry --------------------------------------------------------------
        0x18 => ops.rr(B),
        0x19 => ops.rr(C),
        0x1a => ops.rr(D),
        0x1b => ops.rr(E),
        0x1c => ops.rr(H),
        0x1d => ops.rr(L),
        0x1e => ops.rr(Address::HL),
        0x1f => ops.rr(A),

        // Shift left (arithmetic) ----------------------------------------------------------------
        0x20 => ops.sla(B),
        0x21 => ops.sla(C),
        0x22 => ops.sla(D),
        0x23 => ops.sla(E),
        0x24 => ops.sla(H),
        0x25 => ops.sla(L),
        0x26 => ops.sla(Address::HL),
        0x27 => ops.sla(A),

        // Shift right (arithmetic) ---------------------------------------------------------------
        0x28 => ops.sra(B),
        0x29 => ops.sra(C),
        0x2a => ops.sra(D),
        0x2b => ops.sra(E),
        0x2c => ops.sra(H),
        0x2d => ops.sra(L),
        0x2e => ops.sra(Address::HL),
        0x2f => ops.sra(A),

        // Shift right (logical) ------------------------------------------------------------------
        0x38 => ops.srl(B),
        0x39 => ops.srl(C),
        0x3a => ops.srl(D),
        0x3b => ops.srl(E),
        0x3c => ops.srl(H),
        0x3d => ops.srl(L),
        0x3e => ops.srl(Address::HL),
        0x3f => ops.srl(A),

        // Swap -----------------------------------------------------------------------------------
        0x30 => ops.swap(B),
        0x31 => ops.swap(C),
        0x32 => ops.swap(D),
        0x33 => ops.swap(E),
        0x34 => ops.swap(H),
        0x35 => ops.swap(L),
        0x36 => ops.swap(Address::HL),
        0x37 => ops.swap(A),

        // Bit test -------------------------------------------------------------------------------
        // BIT 0, _
        0x40 => ops.bit(0, B),
        0x41 => ops.bit(0, C),
        0x42 => ops.bit(0, D),
        0x43 => ops.bit(0, E),
        0x44 => ops.bit(0, H),
        0x45 => ops.bit(0, L),
        0x46 => ops.bit(0, Address::HL),
        0x47 => ops.bit(0, A),

        // BIT 1, _
        0x48 => ops.bit(1, B),
        0x49 => ops.bit(1, C),
        0x4a => ops.bit(1, D),
        0x4b => ops.bit(1, E),
        0x4c => ops.bit(1, H),
        0x4d => ops.bit(1, L),
        0x4e => ops.bit(1, Address::HL),
        0x4f => ops.bit(1, A),

        // BIT 2, _
        0x50 => ops.bit(2, B),
        0x51 => ops.bit(2, C),
        0x52 => ops.bit(2, D),
        0x53 => ops.bit(2, E),
        0x54 => ops.bit(2, H),
        0x55 => ops.bit(2, L),
        0x56 => ops.bit(2, Address::HL),
        0x57 => ops.bit(2, A),

        // BIT 3, _
        0x58 => ops.bit(3, B),
        0x59 => ops.bit(3, C),
        0x5a => ops.bit(3, D),
        0x5b => ops.bit(3, E),
        0x5c => ops.bit(3, H),
        0x5d => ops.bit(3, L),
        0x5e => ops.bit(3, Address::HL),
        0x5f => ops.bit(3, A),

        // BIT 4, _
        0x60 => ops.bit(4, B),
        0x61 => ops.bit(4, C),
        0x62 => ops.bit(4, D),
        0x63 => ops.bit(4, E),
        0x64 => ops.bit(4, H),
        0x65 => ops.bit(4, L),
        0x66 => ops.bit(4, Address::HL),
        0x67 => ops.bit(4, A),

        // BIT 5, _
        0x68 => ops.bit(5, B),
        0x69 => ops.bit(5, C),
        0x6a => ops.bit(5, D),
        0x6b => ops.bit(5, E),
        0x6c => ops.bit(5, H),
        0x6d => ops.bit(5, L),
        0x6e => ops.bit(5, Address::HL),
        0x6f => ops.bit(5, A),

        // BIT 6, _
        0x70 => ops.bit(6, B),
        0x71 => ops.bit(6, C),
        0x72 => ops.bit(6, D),
        0x73 => ops.bit(6, E),
        0x74 => ops.bit(6, H),
        0x75 => ops.bit(6, L),
        0x76 => ops.bit(6, Address::HL),
        0x77 => ops.bit(6, A),

        // BIT 7, _
        0x78 => ops.bit(7, B),
        0x79 => ops.bit(7, C),
        0x7a => ops.bit(7, D),
        0x7b => ops.bit(7, E),
        0x7c => ops.bit(7, H),
        0x7d => ops.bit(7, L),
        0x7e => ops.bit(7, Address::HL),
        0x7f => ops.bit(7, A),

        // Bit Set --------------------------------------------------------------------------------
        // SET 0, _
        0xc0 => ops.set(0, B),
        0xc1 => ops.set(0, C),
        0xc2 => ops.set(0, D),
        0xc3 => ops.set(0, E),
        0xc4 => ops.set(0, H),
        0xc5 => ops.set(0, L),
        0xc6 => ops.set(0, Address::HL),
        0xc7 => ops.set(0, A),

        // SET 1, _
        0xc8 => ops.set(1, B),
        0xc9 => ops.set(1, C),
        0xca => ops.set(1, D),
        0xcb => ops.set(1, E),
        0xcc => ops.set(1, H),
        0xcd => ops.set(1, L),
        0xce => ops.set(1, Address::HL),
        0xcf => ops.set(1, A),

        // SET 2, _
        0xd0 => ops.set(2, B),
        0xd1 => ops.set(2, C),
        0xd2 => ops.set(2, D),
        0xd3 => ops.set(2, E),
        0xd4 => ops.set(2, H),
        0xd5 => ops.set(2, L),
        0xd6 => ops.set(2, Address::HL),
        0xd7 => ops.set(2, A),

        // SET 3, _
        0xd8 => ops.set(3, B),
        0xd9 => ops.set(3, C),
        0xda => ops.set(3, D),
        0xdb => ops.set(3, E),
        0xdc => ops.set(3, H),
        0xdd => ops.set(3, L),
        0xde => ops.set(3, Address::HL),
        0xdf => ops.set(3, A),

        // SET 4, _
        0xe0 => ops.set(4, B),
        0xe1 => ops.set(4, C),
        0xe2 => ops.set(4, D),
        0xe3 => ops.set(4, E),
        0xe4 => ops.set(4, H),
        0xe5 => ops.set(4, L),
        0xe6 => ops.set(4, Address::HL),
        0xe7 => ops.set(4, A),

        // SET 5, _
        0xe8 => ops.set(5, B),
        0xe9 => ops.set(5, C),
        0xea => ops.set(5, D),
        0xeb => ops.set(5, E),
        0xec => ops.set(5, H),
        0xed => ops.set(5, L),
        0xee => ops.set(5, Address::HL),
        0xef => ops.set(5, A),

        // SET 6, _
        0xf0 => ops.set(6, B),
        0xf1 => ops.set(6, C),
        0xf2 => ops.set(6, D),
        0xf3 => ops.set(6, E),
        0xf4 => ops.set(6, H),
        0xf5 => ops.set(6, L),
        0xf6 => ops.set(6, Address::HL),
        0xf7 => ops.set(6, A),

        // SET 7, _
        0xf8 => ops.set(7, B),
        0xf9 => ops.set(7, C),
        0xfa => ops.set(7, D),
        0xfb => ops.set(7, E),
        0xfc => ops.set(7, H),
        0xfd => ops.set(7, L),
        0xfe => ops.set(7, Address::HL),
        0xff => ops.set(7, A),

        // Bit Reset ------------------------------------------------------------------------------
        // RES 0, _
        0x80 => ops.res(0, B),
        0x81 => ops.res(0, C),
        0x82 => ops.res(0, D),
        0x83 => ops.res(0, E),
        0x84 => ops.res(0, H),
        0x85 => ops.res(0, L),
        0x86 => ops.res(0, Address::HL),
        0x87 => ops.res(0, A),

        // RES 1, _
        0x88 => ops.res(1, B),
        0x89 => ops.res(1, C),
        0x8a => ops.res(1, D),
        0x8b => ops.res(1, E),
        0x8c => ops.res(1, H),
        0x8d => ops.res(1, L),
        0x8e => ops.res(1, Address::HL),
        0x8f => ops.res(1, A),

        // RES 2, _
        0x90 => ops.res(2, B),
        0x91 => ops.res(2, C),
        0x92 => ops.res(2, D),
        0x93 => ops.res(2, E),
        0x94 => ops.res(2, H),
        0x95 => ops.res(2, L),
        0x96 => ops.res(2, Address::HL),
        0x97 => ops.res(2, A),

        // RES 3, _
        0x98 => ops.res(3, B),
        0x99 => ops.res(3, C),
        0x9a => ops.res(3, D),
        0x9b => ops.res(3, E),
        0x9c => ops.res(3, H),
        0x9d => ops.res(3, L),
        0x9e => ops.res(3, Address::HL),
        0x9f => ops.res(3, A),

        // RES 4, _
        0xa0 => ops.res(4, B),
        0xa1 => ops.res(4, C),
        0xa2 => ops.res(4, D),
        0xa3 => ops.res(4, E),
        0xa4 => ops.res(4, H),
        0xa5 => ops.res(4, L),
        0xa6 => ops.res(4, Address::HL),
        0xa7 => ops.res(4, A),

        // RES 5, _
        0xa8 => ops.res(5, B),
        0xa9 => ops.res(5, C),
        0xaa => ops.res(5, D),
        0xab => ops.res(5, E),
        0xac => ops.res(5, H),
        0xad => ops.res(5, L),
        0xae => ops.res(5, Address::HL),
        0xaf => ops.res(5, A),

        // RES 6, _
        0xb0 => ops.res(6, B),
        0xb1 => ops.res(6, C),
        0xb2 => ops.res(6, D),
        0xb3 => ops.res(6, E),
        0xb4 => ops.res(6, H),
        0xb5 => ops.res(6, L),
        0xb6 => ops.res(6, Address::HL),
        0xb7 => ops.res(6, A),

        // RES 7, _
        0xb8 => ops.res(7, B),
        0xb9 => ops.res(7, C),
        0xba => ops.res(7, D),
        0xbb => ops.res(7, E),
        0xbc => ops.res(7, H),
        0xbd => ops.res(7, L),
        0xbe => ops.res(7, Address::HL),
        0xbf => ops.res(7, A),

        _ => {
            unreachable!("unknown opcode: cb {:02x}", opcode)
        }
    }
}
