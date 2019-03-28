use super::Register;
use bitintr::Bextr;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::{
    convert::TryInto,
    fmt::{self, Display, Formatter},
};
use unchecked_unwrap::UncheckedUnwrap;
use crate::state::State;

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, FromPrimitive)]
#[repr(u8)]
pub enum ShiftType {
    LogicalLeft = 0b00,
    LogicalRight = 0b01,
    ArithmeticRight = 0b10,
    RotateRight = 0b11,
}

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
pub enum Shift {
    Immediate { amount: u8, type_: ShiftType },
    Register { s: Register, type_: ShiftType },
}

impl ShiftType {
    #[inline]
    pub fn decode(code: u32) -> Self {
        unsafe { ShiftType::from_u32(code).unchecked_unwrap() }
    }
}

impl Shift {
    pub fn apply(self, state: &State, value: u32) -> u32 {
        let (amount, type_) = match self {
            Shift::Immediate { amount, type_ } => {
                (amount as u32, type_)
            }

            Shift::Register { s, type_ } => {
                (*state.r(s), type_)
            }
        };

        match type_ {
            ShiftType::LogicalLeft => {
                value << amount
            }

            ShiftType::ArithmeticRight => {
                (( value as i32 ) >> amount) as u32
            }

            ShiftType::LogicalRight => {
                value >> amount
            }

            ShiftType::RotateRight => {
                value.rotate_right(amount)
            }
        }
    }

    #[inline]
    pub fn decode(code: u32) -> Self {
        let type_ = ShiftType::decode(code.bextr(1, 2));
        if code.bextr(0, 1) == 1 {
            Shift::Register {
                type_,
                s: Register::decode(code.bextr(4, 4)),
            }
        } else {
            Shift::Immediate {
                type_,
                amount: unsafe { code.bextr(3, 5).try_into().unchecked_unwrap() },
            }
        }
    }
}

impl Display for ShiftType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(match self {
            ShiftType::LogicalLeft => "LSL",
            ShiftType::LogicalRight => "LSR",
            ShiftType::ArithmeticRight => "ASR",
            ShiftType::RotateRight => "ROR",
        })
    }
}

impl Display for Shift {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Shift::Immediate { amount, type_ } if *amount > 0 => {
                write!(f, ", {} #{}", type_, amount)
            }

            Shift::Register { s, type_ } => write!(f, ", {} {}", type_, s),

            // 0-amount immediate shifts are the default value and do not get printed
            _ => Ok(()),
        }
    }
}
