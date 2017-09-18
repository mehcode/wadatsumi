use super::State;
use super::io::{In8, Out8};
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
    fn jp<C: Condition>(&mut self, C) -> Self::Output;

    /// Call (subroutine)
    fn call<C: Condition>(&mut self, C) -> Self::Output;

    /// Return (from subroutine)
    fn ret<C: Condition>(&mut self, C) -> Self::Output;

    /// Return (from subroutine) and enable interrupts
    fn reti(&mut self) -> Self::Output;

    /// Addition
    fn add<I: In8>(&mut self, I) -> Self::Output;

    /// Compare
    fn compare<I: In8>(&mut self, I) -> Self::Output;

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

    /// Enable interrupts
    fn ei(&mut self) -> Self::Output;

    /// Disable dnterrupts
    fn di(&mut self) -> Self::Output;

    /// Reset
    fn reset(&mut self, address: u8) -> Self::Output;

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

        // LD (r16), _
        0x02 => ops.load8(Address::BC, A),
        0x12 => ops.load8(Address::DE, A),
        0x22 => ops.load8(Address::HLI, A),
        0x32 => ops.load8(Address::HLD, A),
        0x36 => ops.load8(Address::HL, Immediate8),

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
        0xc3 => ops.jp(()),
        0xc2 => ops.jp(condition::NOT_ZERO),
        0xca => ops.jp(condition::ZERO),
        0xd2 => ops.jp(condition::NOT_CARRY),
        0xda => ops.jp(condition::CARRY),

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
        0x3c => ops.inc8(Address::HL),

        // 8-bit decrement ------------------------------------------------------------------------
        0x05 => ops.dec8(B),
        0x0d => ops.dec8(C),
        0x15 => ops.dec8(D),
        0x1d => ops.dec8(E),
        0x25 => ops.dec8(H),
        0x2d => ops.dec8(L),
        0x35 => ops.dec8(Address::HL),
        0x3d => ops.dec8(Address::HL),

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

        // Compare --------------------------------------------------------------------------------
        0xfe => ops.compare(Immediate8),
        0xb8 => ops.compare(B),
        0xb9 => ops.compare(C),
        0xba => ops.compare(D),
        0xbb => ops.compare(E),
        0xbc => ops.compare(H),
        0xbd => ops.compare(L),
        0xbe => ops.compare(Address::HL),
        0xbf => ops.compare(A),

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
        0xc7 => ops.reset(0x00),
        0xcf => ops.reset(0x08),
        0xd7 => ops.reset(0x10),
        0xdf => ops.reset(0x18),
        0xe7 => ops.reset(0x20),
        0xef => ops.reset(0x28),
        0xf7 => ops.reset(0x30),
        0xff => ops.reset(0x38),

        // Miscellaneous --------------------------------------------------------------------------
        0x00 => ops.nop(),
        0xf3 => ops.di(),
        0xfb => ops.ei(),
        _ => ops.undefined(opcode),
    }
}
