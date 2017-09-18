use std::fmt;
use super::super::bus::Bus;
use super::operands::{Address, Register16, Register8};
use super::io::{In8, Out8};
use super::State;

// Data8 ------------------------------------------------------------------------------------------

/// Wraps `u8` to format as hexadecimal.
#[derive(Debug)]
pub struct Data8(pub u8);

impl fmt::Display for Data8 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:02x}", self.0)
    }
}

// SignedData8 ------------------------------------------------------------------------------------

/// Wraps `i8` to format as signed hexadecimal.
#[derive(Debug)]
pub struct SignedData8(pub i8);

impl fmt::Display for SignedData8 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.0 < 0 {
            write!(f, "{:02x}", self.0 * -1)
        } else {
            write!(f, "{:02x}", self.0)
        }
    }
}

// Data16 -----------------------------------------------------------------------------------------

/// Wraps `u16` to format as hexadecimal.
#[derive(Debug)]
pub struct Data16(pub u16);

impl fmt::Display for Data16 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:04x}", self.0)
    }
}

// Operand8 ---------------------------------------------------------------------------------------

/// Describes a valid operand for an 8-bit instruction.
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

// Condition --------------------------------------------------------------------------------------

/// Describes a condition that may be around a conditional instruction such as `JP` or `CALL`.
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

// Instruction ------------------------------------------------------------------------------------

#[derive(Debug)]
pub enum Instruction {
    Undefined(Data8),
    Nop,
    Load8(Operand8, Operand8),
    Load16Immediate(Register16, Data16),
    JumpRelative(Option<Condition>, SignedData8),
    Jump(Option<Condition>, Data16),
    Call(Option<Condition>, Data16),
    Return(Option<Condition>),
    ReturnAndEnableInterrupts,
    Increment8(Operand8),
    Decrement8(Operand8),
    And(Operand8),
    Or(Operand8),
    Xor(Operand8),
    EnableInterrupts,
    DisableInterrupts,
    Reset(Data8),
}

#[inline]
fn unary<T: fmt::Display>(f: &mut fmt::Formatter, instr: &str, arg: T) -> fmt::Result {
    write!(f, "{} {}", instr, arg)
}

#[inline]
fn binary<T: fmt::Display, U: fmt::Display>(
    f: &mut fmt::Formatter,
    instr: &str,
    arg0: T,
    arg1: U,
) -> fmt::Result {
    write!(f, "{} {}, {}", instr, arg0, arg1)
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Instruction::*;

        match *self {
            // Unit (0-argument)
            Nop => write!(f, "NOP"),
            EnableInterrupts => write!(f, "EI"),
            DisableInterrupts => write!(f, "DI"),
            Return(None) => write!(f, "RET"),
            ReturnAndEnableInterrupts => write!(f, "RETI"),

            // Unary (1-argument)
            Jump(None, ref address) => unary(f, "JP", address),
            JumpRelative(None, ref offset) => unary(f, "JR", offset),
            Call(None, ref address) => unary(f, "CALL", address),
            Return(Some(ref cond)) => unary(f, "RET", cond),
            And(ref operand) => unary(f, "AND", operand),
            Or(ref operand) => unary(f, "OR", operand),
            Xor(ref operand) => unary(f, "XOR", operand),
            Increment8(ref operand) => unary(f, "INC", operand),
            Decrement8(ref operand) => unary(f, "DEC", operand),
            Reset(ref address) => unary(f, "RST", address),
            Undefined(ref opcode) => unary(f, "UNDEF", opcode),

            // Binary (2-argument)
            Jump(Some(ref cond), ref address) => binary(f, "JP", cond, address),
            JumpRelative(Some(ref cond), ref offset) => binary(f, "JR", cond, offset),
            Call(Some(ref cond), ref address) => binary(f, "CALL", cond, address),
            Load8(ref src, ref dst) => binary(f, "LD", src, dst),
            Load16Immediate(dst, ref value) => binary(f, "LD", dst, value),
        }
    }
}
