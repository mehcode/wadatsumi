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

impl fmt::Display for Operand8 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Operand8::*;

        match *self {
            Register(register) => write!(f, "{:?}", register),
            Immediate(value) => write!(f, "{:02x}", value),
            Memory(address) => write!(f, "({})", address),
        }
    }
}

#[derive(Debug)]
pub enum Condition {
    Zero,
    NotZero,
    Carry,
    NotCarry,
}

impl fmt::Display for Condition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Condition::*;

        match *self {
            Zero => write!(f, "Z"),
            NotZero => write!(f, "NZ"),
            Carry => write!(f, "C"),
            NotCarry => write!(f, "NC"),
        }
    }
}

#[derive(Debug)]
pub enum Instruction {
    Undefined(u8),
    Nop,
    Load8(Operand8, Operand8),
    Load16Immediate(Register16, u16),
    Jr(Option<Condition>, i8),
    Jp(Option<Condition>, u16),
    Call(Option<Condition>, u16),
    And(Operand8),
    Or(Operand8),
    Xor(Operand8),
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Instruction::*;

        match *self {
            Nop => write!(f, "NOP"),
            Jp(None, address) => write!(f, "JP {:04x}", address),
            Jr(None, offset) => write!(f, "JR {}", offset),
            Call(None, address) => write!(f, "CALL {:04x}", address),
            Jp(Some(ref cond), address) => write!(f, "JP {} {:04x}", cond, address),
            Jr(Some(ref cond), offset) => write!(f, "JR {} {}", cond, offset),
            Call(Some(ref cond), address) => write!(f, "CALL {} {:04x}", cond, address),
            Load8(ref src, ref dst) => write!(f, "LD {}, {}", src, dst),
            Load16Immediate(dst, value) => write!(f, "LD {:?}, {:04x}", dst, value),
            And(ref operand) => write!(f, "AND {}", operand),
            Or(ref operand) => write!(f, "OR {}", operand),
            Xor(ref operand) => write!(f, "XOR {}", operand),
            Undefined(opcode) => write!(f, "UNDEF {:02x}", opcode),
        }
    }
}
