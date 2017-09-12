use std::fmt;

use super::super::bus::Bus;
use super::operands::{Address, Register16, Register8};
use super::io::{In8, Out8};
use super::State;

#[derive(Debug)]
pub enum Operand8 {
    Register(Register8),
    Immediate(u8),
    Memory(Address),
}

#[derive(Debug)]
pub enum Instruction {
    Undefined(u8),
    Nop,
    Load8(Operand8, Operand8),
    Load16Immediate(Register16, u16),
    Jp(u16),
    And(Operand8),
    Or(Operand8),
    Xor(Operand8),
}

impl From<Register8> for Operand8 {
    fn from(value: Register8) -> Self {
        Operand8::Register(value)
    }
}

impl fmt::Display for Operand8 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Operand8::*;

        match *self {
            Register(register) => write!(f, "{:?}", register),
            Immediate(value) => write!(f, "{:02x}", value),
            Memory(address) => write!(f, "({:?})", address),
        }
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Instruction::*;

        match *self {
            Nop => write!(f, "NOP"),
            Jp(address) => write!(f, "JP {:04x}", address),
            Load8(ref src, ref dst) => write!(f, "LD {}, {}", src, dst),
            Load16Immediate(dst, value) => write!(f, "LD {:?}, {:04x}", dst, value),
            And(ref operand) => write!(f, "AND {}", operand),
            Or(ref operand) => write!(f, "OR {}", operand),
            Xor(ref operand) => write!(f, "XOR {}", operand),
            Undefined(opcode) => write!(f, "UNDEF {:02x}", opcode),
        }
    }
}
