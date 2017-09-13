use super::State;
use super::io::{In8, Out8};
use super::operands::{Address, Immediate16, Immediate8, Register16, Register8};

/// Defines a visitor for a CPU (micro) operation.
pub trait Operations {
    type Output;

    /// No Operation
    fn nop(&mut self) -> Self::Output;

    /// 8-bit Loads
    fn load8<I: In8, O: Out8>(&mut self, O, I) -> Self::Output;

    /// 16-bit Immediate Load
    fn load16_immediate(&mut self, Register16) -> Self::Output;

    /// Absolute Jump
    fn jp(&mut self) -> Self::Output;

    /// Bitwise AND
    fn and<IO: In8 + Out8>(&mut self, IO) -> Self::Output;

    /// Bitwise OR
    fn or<IO: In8 + Out8>(&mut self, IO) -> Self::Output;

    /// Bitwise XOR
    fn xor<IO: In8 + Out8>(&mut self, IO) -> Self::Output;

    /// Undefined operation; an unmapped opcode.
    fn undefined(&mut self, opcode: u8) -> Self::Output;
}

#[inline]
pub fn visit<O: Operations>(mut ops: O, opcode: u8) -> O::Output {
    use self::Register8::*;
    use self::Register16::*;

    match opcode {
        // 8-bit Loads ---------------------------------------------------------
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

        // LD A, _
        0x3e => ops.load8(A, Immediate8),

        // LD (r16), _
        0x02 => ops.load8(Address::BC, Immediate8),
        0x12 => ops.load8(Address::DE, Immediate8),
        0x22 => ops.load8(Address::HLI, Immediate8),
        0x32 => ops.load8(Address::HLD, Immediate8),
        0x36 => ops.load8(Address::HL, Immediate8),

        // 16-bit Immediate Loads ----------------------------------------------
        0x01 => ops.load16_immediate(BC),
        0x11 => ops.load16_immediate(DE),
        0x21 => ops.load16_immediate(HL),
        // TOOD: 0x31 => ops.load16_immediate(SP),

        // Jumps ---------------------------------------------------------------
        0xc3 => ops.jp(),

        // Arithmetic ----------------------------------------------------------
        // AND _
        0xa0 => ops.and(B),
        0xa1 => ops.and(C),
        0xa2 => ops.and(D),
        0xa3 => ops.and(E),
        0xa4 => ops.and(H),
        0xa5 => ops.and(L),
        0xa6 => ops.and(Address::HL),
        0xa7 => ops.and(A),

        // XOR _
        0xa8 => ops.xor(B),
        0xa9 => ops.xor(C),
        0xaa => ops.xor(D),
        0xab => ops.xor(E),
        0xac => ops.xor(H),
        0xad => ops.xor(L),
        0xae => ops.xor(Address::HL),
        0xaf => ops.xor(A),

        // OR _
        0xb0 => ops.or(B),
        0xb1 => ops.or(C),
        0xb2 => ops.or(D),
        0xb3 => ops.or(E),
        0xb4 => ops.or(H),
        0xb5 => ops.or(L),
        0xb6 => ops.or(Address::HL),
        0xb7 => ops.or(A),

        // Miscellaneous -------------------------------------------------------
        0x00 => ops.nop(),
        _ => ops.undefined(opcode),
    }
}
