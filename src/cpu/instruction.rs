use std::fmt;

use super::super::bus::Bus;
use super::registers::{Register16, Register8};
use super::io::{In8, Out8};
use super::State;

/// Operand8 (8-bits)
#[derive(Debug)]
pub enum Operand8 {
    Register(Register8),
    Memory(Address),
}

/// Address (16-bits)
#[derive(Debug, Clone, Copy)]
pub enum Address {
    Direct(u16),
}

#[derive(Debug)]
pub enum Instruction {
    Undefined(u8),
    Nop,
    Load8(Operand8, Operand8),
    Load8Immediate(Operand8, u8),
    Load16Immediate(Register16, u16),
    Jp(Address),
}

impl From<Register8> for Operand8 {
    fn from(value: Register8) -> Self {
        Operand8::Register(value)
    }
}

impl From<Address> for Operand8 {
    fn from(value: Address) -> Self {
        Operand8::Memory(value)
    }
}

impl From<u16> for Address {
    fn from(value: u16) -> Self {
        Address::Direct(value)
    }
}

impl In8 for Address {
    #[inline]
    fn read8<B: Bus>(&self, _: &mut State, bus: &mut B) -> u8 {
        match *self {
            Address::Direct(address) => bus.read8(address),
        }
    }
}

impl fmt::Display for Operand8 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Operand8::*;

        match *self {
            Register(register) => write!(f, "{:?}", register),
            _ => unimplemented!(),
        }
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Instruction::*;
        use self::Address::*;

        match *self {
            Nop => write!(f, "NOP"),
            Jp(Direct(address)) => write!(f, "JP {:04x}", address),
            Load8(ref src, ref dst) => write!(f, "LD {}, {}", src, dst),
            Load8Immediate(ref dst, value) => write!(f, "LD {}, {:04x}", dst, value),
            Load16Immediate(dst, value) => write!(f, "LD {:?}, {:04x}", dst, value),
            Undefined(opcode) => write!(f, "UNDEF {:02x}", opcode),
        }
    }
}
