use super::State;
use super::io::{In8, Out8};
use super::instruction::Address;
use super::registers::Register8;
use super::registers::Register8::*;

/// Defines a visitor for a CPU (micro) operation.
pub trait Operations {
    type Output;

    /// No Operation ~ NOP
    fn nop(&mut self) -> Self::Output;

    /// 8-bit Loads ::
    ///     LD r8, r8
    ///     LD (r16), r8
    ///     LD r8, (r16)
    fn load8<I: In8, O: Out8>(&mut self, destination: O, source: I) -> Self::Output;

    /// Absolute Jump ~ JP #16
    fn jp(&mut self) -> Self::Output;

    /// Undefined operation; an unmapped opcode.
    fn undefined(&mut self, opcode: u8) -> Self::Output;
}

#[inline]
pub fn visit<O: Operations>(mut ops: O, opcode: u8) -> O::Output {
    match opcode {
        // 8-bit Loads
        // ===========

        // LD B, _
        0x40 => ops.load8(B, B),
        0x41 => ops.load8(B, C),
        0x42 => ops.load8(B, D),
        0x43 => ops.load8(B, E),
        0x44 => ops.load8(B, H),
        0x45 => ops.load8(B, L),
        // TODO: 0x46 => ops.load8(B, Address::HL),
        0x47 => ops.load8(B, A),

        // LD C, _
        0x48 => ops.load8(C, B),
        0x49 => ops.load8(C, C),
        0x4a => ops.load8(C, D),
        0x4b => ops.load8(C, E),
        0x4c => ops.load8(C, H),
        0x4d => ops.load8(C, L),
        // TODO: 0x4e => ops.load8(C, Address::HL),
        0x4f => ops.load8(C, A),

        // LD D, _
        0x50 => ops.load8(D, B),
        0x51 => ops.load8(D, C),
        0x52 => ops.load8(D, D),
        0x53 => ops.load8(D, E),
        0x54 => ops.load8(D, H),
        0x55 => ops.load8(D, L),
        // TODO: 0x56 => ops.load8(D, Address::HL),
        0x57 => ops.load8(D, A),

        // LD E, _
        0x58 => ops.load8(E, B),
        0x59 => ops.load8(E, C),
        0x5a => ops.load8(E, D),
        0x5b => ops.load8(E, E),
        0x5c => ops.load8(E, H),
        0x5d => ops.load8(E, L),
        // TODO: 0x5e => ops.load8(E, Address::HL),
        0x5f => ops.load8(E, A),

        // LD H, _
        0x60 => ops.load8(H, B),
        0x61 => ops.load8(H, C),
        0x62 => ops.load8(H, D),
        0x63 => ops.load8(H, E),
        0x64 => ops.load8(H, H),
        0x65 => ops.load8(H, L),
        // TODO: 0x66 => ops.load8(H, Address::HL),
        0x67 => ops.load8(H, A),

        // LD L, _
        0x68 => ops.load8(L, B),
        0x69 => ops.load8(L, C),
        0x6a => ops.load8(L, D),
        0x6b => ops.load8(L, E),
        0x6c => ops.load8(L, H),
        0x6d => ops.load8(L, L),
        // TODO: 0x6e => ops.load8(L, Address::HL),
        0x6f => ops.load8(L, A),

        // Jumps
        // =====

        0xc3 => ops.jp(),

        // Miscellaneous
        // =============

        0x00 => ops.nop(),
        _ => ops.undefined(opcode),
    }
}
