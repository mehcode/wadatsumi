use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::fmt::{self, Display, Formatter};
use unchecked_unwrap::UncheckedUnwrap;

/// A condition that must pass before an instruction can be executed.
/// ARM instructions require this property but can use `AL` or `Always` to nop out of
/// a condition check.
#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, FromPrimitive)]
#[repr(u8)]
pub enum Condition {
    Equal = 0b0000,
    NotEqual = 0b0001,
    UnsignedHigherOrSame = 0b0010,
    UnsignedLower = 0b0011,
    Negative = 0b0100,
    PositiveOrZero = 0b0101,
    Overflow = 0b0110,
    NoOverflow = 0b0111,
    UnsignedHigher = 0b1000,
    UnsignedLowerOrSame = 0b1001,
    GreaterOrEqual = 0b1010,
    LessThan = 0b1011,
    GreaterThan = 0b1100,
    LessThanOrEqual = 0b1101,
    Always = 0b1110,
}

impl Condition {
    #[inline]
    pub fn decode(code: u32) -> Self {
        unsafe { Condition::from_u32(code).unchecked_unwrap() }
    }
}

impl Display for Condition {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(match self {
            Condition::Equal => "EQ",
            Condition::NotEqual => "NE",
            Condition::UnsignedHigherOrSame => "CS",
            Condition::UnsignedLower => "CC",
            Condition::Negative => "MI",
            Condition::PositiveOrZero => "PL",
            Condition::Overflow => "VS",
            Condition::NoOverflow => "VC",
            Condition::UnsignedHigher => "HI",
            Condition::UnsignedLowerOrSame => "LS",
            Condition::GreaterOrEqual => "GE",
            Condition::LessThan => "LT",
            Condition::GreaterThan => "GT",
            Condition::LessThanOrEqual => "GE",

            // Can be shortened to AL but generally it is just omitted from
            // the instruction
            Condition::Always => "",
        })
    }
}
